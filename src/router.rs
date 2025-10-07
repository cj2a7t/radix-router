//! Core router implementation

use crate::ffi::RadixTreeRaw;
use crate::route::*;
use lru::LruCache;
use regex::Regex;
use std::collections::HashMap;
use std::num::NonZeroUsize;

/// Router configuration options
#[derive(Debug, Clone)]
pub struct RouterOpts {
    /// Disable parameter matching
    pub no_param_match: bool,
    /// Disable path cache optimization
    pub disable_path_cache: bool,
}

impl Default for RouterOpts {
    fn default() -> Self {
        Self {
            no_param_match: false,
            disable_path_cache: false,
        }
    }
}

/// High-performance radix tree based router
pub struct RadixRouter {
    /// C-based radix tree
    tree: RadixTreeRaw,
    /// Route storage: index -> Vec<RouteOpts>
    match_data: HashMap<usize, Vec<RouteOpts>>,
    /// Current maximum index
    match_data_index: usize,
    /// Hash-based exact path matching: path -> Vec<RouteOpts>
    hash_path: HashMap<String, Vec<RouteOpts>>,
    /// Pattern cache for parameter extraction
    pattern_cache: LruCache<String, (Regex, Vec<String>)>,
    /// Router options
    opts: RouterOpts,
}

impl RadixRouter {
    /// Create a new router with routes
    pub fn new(routes: Vec<Route>, opts: Option<RouterOpts>) -> Result<Self, String> {
        let opts = opts.unwrap_or_default();
        let mut router = Self {
            tree: RadixTreeRaw::new(),
            match_data: HashMap::new(),
            match_data_index: 0,
            hash_path: HashMap::new(),
            pattern_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            opts,
        };

        // Register all routes
        for route in routes {
            router.add_route(route)?;
        }

        Ok(router)
    }

    /// Add a single route to the router
    pub fn add_route(&mut self, route: Route) -> Result<(), String> {
        for path in &route.paths {
            self.insert_route(path, &route)?;
        }
        Ok(())
    }

    /// Insert a route with specific path
    fn insert_route(&mut self, path: &str, route: &Route) -> Result<(), String> {
        // Process route data
        let route_opts = self.process_route(path, route)?;

        // Optimization: use hash map for exact path matching
        if !self.opts.disable_path_cache && route_opts.path_op == PathOp::Equal {
            let routes = self.hash_path.entry(route_opts.path.clone()).or_default();
            routes.push(route_opts);
            routes.sort_by(|a, b| a.cmp_priority(b));
            return Ok(());
        }

        // Check if path already exists in radix tree
        if let Some(idx) = self.tree.find(route_opts.path.as_bytes()) {
            // Path exists, add to existing route array
            if let Some(routes) = self.match_data.get_mut(&idx) {
                routes.push(route_opts);
                routes.sort_by(|a, b| a.cmp_priority(b));
                return Ok(());
            }
        }

        // New path, allocate new index
        self.match_data_index += 1;
        let idx = self.match_data_index;

        self.match_data.insert(idx, vec![route_opts.clone()]);

        // Insert into radix tree
        if !self.tree.insert(route_opts.path.as_bytes(), idx as i32) {
            return Err(format!("Failed to insert path: {}", route_opts.path));
        }

        Ok(())
    }

    /// Process route data
    fn process_route(&self, path: &str, route: &Route) -> Result<RouteOpts, String> {
        // Process HTTP methods
        let methods = route.methods.unwrap_or(HttpMethod::empty());

        // Process hosts
        let hosts = route
            .hosts
            .as_ref()
            .map(|hosts| hosts.iter().map(|h| HostPattern::new(h)).collect());

        // Process path (extract parameters)
        let (actual_path, path_op, has_param) = self.parse_path(path);

        // Clone filter function if present
        let filter_fn = if let Some(ref f) = route.filter_fn {
            Some(f.clone())
        } else {
            None
        };

        Ok(RouteOpts {
            id: route.id.clone(),
            path: actual_path,
            path_org: path.to_string(),
            path_op,
            has_param,
            methods,
            hosts,
            vars: route.vars.clone(),
            filter_fn,
            priority: route.priority,
            metadata: route.metadata.clone(),
        })
    }

    /// Parse path and extract parameter information
    fn parse_path(&self, path: &str) -> (String, PathOp, bool) {
        // Check for parameter :param
        if !self.opts.no_param_match {
            if let Some(pos) = path.find(':') {
                let actual_path = &path[..pos];
                return (actual_path.to_string(), PathOp::PrefixMatch, true);
            }
        }

        // Check for wildcard *
        if let Some(pos) = path.find('*') {
            let actual_path = &path[..pos];
            let has_param = pos != path.len() - 1;
            return (actual_path.to_string(), PathOp::PrefixMatch, has_param);
        }

        // Exact path match
        (path.to_string(), PathOp::Equal, false)
    }

    /// Match a route
    pub fn match_route(&mut self, path: &str, opts: &mut MatchOpts) -> Option<&serde_json::Value> {
        // Clear previous match results
        if let Some(matched) = &mut opts.matched {
            matched.clear();
        }

        // Lowercase host for matching
        if let Some(host) = &opts.host {
            opts.host = Some(host.to_lowercase());
        }

        // Priority 1: Check hash_path for exact match
        if let Some(routes) = self.hash_path.get(path).cloned() {
            for (i, route) in routes.iter().enumerate() {
                if self.match_route_opts(route, path, opts) {
                    if let Some(matched) = &mut opts.matched {
                        matched.insert("_path".to_string(), path.to_string());
                    }
                    return self
                        .hash_path
                        .get(path)
                        .and_then(|r| r.get(i))
                        .map(|r| &r.metadata);
                }
            }
        }

        // Priority 2: Use radix tree for prefix matching
        if !self.tree.search(path.as_bytes()) {
            return None;
        }

        // Iterate through matching routes
        while let Some(idx) = self.tree.tree_up(path.as_bytes()) {
            if let Some(routes) = self.match_data.get(&idx).cloned() {
                for (i, route) in routes.iter().enumerate() {
                    if self.match_route_opts(route, path, opts) {
                        if let Some(matched) = &mut opts.matched {
                            matched.insert("_path".to_string(), route.path_org.clone());
                        }
                        return self
                            .match_data
                            .get(&idx)
                            .and_then(|r| r.get(i))
                            .map(|r| &r.metadata);
                    }
                }
            }
        }

        None
    }

    /// Match route options
    fn match_route_opts(&mut self, route: &RouteOpts, path: &str, opts: &mut MatchOpts) -> bool {
        // 1. HTTP method matching
        if !route.methods.is_empty() {
            if let Some(method) = &opts.method {
                if let Some(m) = HttpMethod::from_str(method) {
                    if !route.methods.contains(m) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        if let Some(matched) = &mut opts.matched {
            if let Some(method) = &opts.method {
                matched.insert("_method".to_string(), method.clone());
            }
        }

        // 2. Host matching
        if let Some(hosts) = &route.hosts {
            let mut matched_host = false;
            if let Some(host) = &opts.host {
                for pattern in hosts {
                    if pattern.matches(host) {
                        if let Some(m) = &mut opts.matched {
                            let host_value = if pattern.is_wildcard {
                                format!("*{}", pattern.pattern)
                            } else {
                                host.clone()
                            };
                            m.insert("_host".to_string(), host_value);
                        }
                        matched_host = true;
                        break;
                    }
                }
            }

            if !matched_host {
                return false;
            }
        }

        // 3. Parameter matching
        if !self.compare_param(path, route, opts) {
            return false;
        }

        // 4. Variable expression matching
        if let Some(vars) = &route.vars {
            if let Some(req_vars) = &opts.vars {
                for expr in vars {
                    if !expr.eval(req_vars) {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }

        // 5. Custom filter function
        if let Some(filter_fn) = &route.filter_fn {
            let vars = opts.vars.as_ref().cloned().unwrap_or_default();
            if !filter_fn(&vars, opts) {
                return false;
            }
        }

        true
    }

    /// Extract parameters from path
    fn compare_param(&mut self, req_path: &str, route: &RouteOpts, opts: &mut MatchOpts) -> bool {
        // If matched is not requested and no params, just check path prefix
        if opts.matched.is_none() && !route.has_param {
            return true;
        }

        if !route.has_param {
            return true;
        }

        // Get or generate pattern
        let (pattern, names) = match self.pattern_cache.get(&route.path_org) {
            Some(cached) => cached.clone(),
            None => {
                let (pat, names) = self.generate_pattern(&route.path_org);
                self.pattern_cache
                    .put(route.path_org.clone(), (pat.clone(), names.clone()));
                (pat, names)
            }
        };

        if names.is_empty() {
            return true;
        }

        // Match and extract parameters
        if let Some(captures) = pattern.captures(req_path) {
            // Check if full path matches
            if captures.get(0).map(|m| m.as_str()) != Some(req_path) {
                return false;
            }

            // Extract parameters
            if let Some(matched) = &mut opts.matched {
                for (i, name) in names.iter().enumerate() {
                    if let Some(cap) = captures.get(i + 1) {
                        matched.insert(name.clone(), cap.as_str().to_string());
                    }
                }
            }

            true
        } else {
            false
        }
    }

    /// Generate regex pattern for path with parameters
    fn generate_pattern(&self, path: &str) -> (Regex, Vec<String>) {
        let mut names = Vec::new();
        let parts: Vec<&str> = path.split('/').collect();
        let mut pattern_parts = Vec::new();

        for part in parts {
            if part.is_empty() {
                pattern_parts.push("".to_string());
                continue;
            }

            if part.starts_with(':') {
                // Parameter: :name
                names.push(part[1..].to_string());
                pattern_parts.push(r"([^/]+)".to_string());
            } else if part.starts_with('*') {
                // Wildcard: *name or *
                let name = if part.len() > 1 {
                    part[1..].to_string()
                } else {
                    ":ext".to_string()
                };
                names.push(name);
                pattern_parts.push(r"(.*)".to_string());
            } else {
                pattern_parts.push(regex::escape(part));
            }
        }

        let pattern_str = format!("^{}$", pattern_parts.join("/"));
        let pattern = Regex::new(&pattern_str).expect("Invalid regex pattern");

        (pattern, names)
    }

    /// Update an existing route
    pub fn update_route(&mut self, old_route: Route, new_route: Route) -> Result<(), String> {
        // Remove old route
        self.delete_route(old_route)?;
        // Add new route
        self.add_route(new_route)?;
        Ok(())
    }

    /// Delete a route
    pub fn delete_route(&mut self, route: Route) -> Result<(), String> {
        for path in &route.paths {
            self.remove_route(path, &route)?;
        }
        Ok(())
    }

    /// Remove a specific route from a path
    fn remove_route(&mut self, path: &str, route: &Route) -> Result<(), String> {
        let route_opts = self.process_route(path, route)?;

        // Check hash_path first
        if !self.opts.disable_path_cache && route_opts.path_op == PathOp::Equal {
            if let Some(routes) = self.hash_path.get_mut(&route_opts.path) {
                routes.retain(|r| r.id != route_opts.id);
                if routes.is_empty() {
                    self.hash_path.remove(&route_opts.path);
                }
                return Ok(());
            }
            return Err(format!("Route not found in hash_path: {}", route.id));
        }

        // Find in radix tree
        if let Some(idx) = self.tree.find(route_opts.path.as_bytes()) {
            if let Some(routes) = self.match_data.get_mut(&idx) {
                routes.retain(|r| r.id != route_opts.id);

                if routes.is_empty() {
                    // Remove from tree if no routes left
                    self.match_data.remove(&idx);
                    self.tree.remove(route_opts.path.as_bytes());
                }
                return Ok(());
            }
        }

        Err(format!("Route not found: {}", route.id))
    }
}

impl std::fmt::Debug for RadixRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadixRouter")
            .field("match_data_index", &self.match_data_index)
            .field("hash_path_count", &self.hash_path.len())
            .field("match_data_count", &self.match_data.len())
            .field("opts", &self.opts)
            .finish()
    }
}
