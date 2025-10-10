//! Core router implementation

use crate::ffi::RadixTreeRaw;
use crate::route::*;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::sync::RwLock;

/// High-performance radix tree based router (optimized for concurrent reads)
///
/// This router is designed for optimal read performance:
/// - After initialization, all route data is immutable
/// - `match_route()` requires only `&self` and uses temporary iterators for lock-free queries
/// - Each query creates its own iterator, making it fully thread-safe and async-safe
/// - Regex patterns are pre-compiled during route registration (zero runtime compilation)
/// - Multiple threads/tasks can call `match_route()` concurrently without contention
pub struct RadixRouter {
    /// C-based radix tree (RwLock only for insert/remove operations)
    tree: RwLock<RadixTreeRaw>,
    /// Route storage: index -> Vec<RouteOpts> (immutable after construction)
    match_data: HashMap<usize, Vec<RouteOpts>>,
    /// Current maximum index
    match_data_index: usize,
    /// Hash-based exact path matching: path -> Vec<RouteOpts> (immutable after construction)
    hash_path: HashMap<String, Vec<RouteOpts>>,
}

impl RadixRouter {
    /// Create a new router with routes
    pub fn new(routes: Vec<RadixNode>) -> Result<Self> {
        let mut router = Self {
            tree: RwLock::new(RadixTreeRaw::new().context("Failed to create radix tree")?),
            match_data: HashMap::new(),
            match_data_index: 0,
            hash_path: HashMap::new(),
        };

        // Register all routes
        for route in routes {
            router.add_route(route)?;
        }

        Ok(router)
    }

    /// Add a single route to the router
    pub fn add_route(&mut self, route: RadixNode) -> Result<()> {
        for path in &route.paths {
            self.insert_route(path, &route)?;
        }
        Ok(())
    }

    /// Insert a route with specific path
    fn insert_route(&mut self, path: &str, route: &RadixNode) -> Result<()> {
        // Process route data
        let route_opts = self.process_route(path, route)?;

        // Optimization: use hash map for exact path matching (always enabled)
        if route_opts.path_op == PathOp::Equal {
            let routes = self.hash_path.entry(route_opts.path.clone()).or_default();
            routes.push(route_opts);
            routes.sort_by(|a, b| a.cmp_priority(b));
            return Ok(());
        }

        // Check if path already exists in radix tree
        if let Some(idx) = self
            .tree
            .read()
            .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?
            .find(route_opts.path.as_bytes())
        {
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
        if !self
            .tree
            .write()
            .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?
            .insert(route_opts.path.as_bytes(), idx as i32)
        {
            anyhow::bail!("Failed to insert path: {}", route_opts.path);
        }

        Ok(())
    }

    /// Process route data
    fn process_route(&self, path: &str, route: &RadixNode) -> Result<RouteOpts> {
        // Process HTTP methods
        let methods = route.methods.unwrap_or(RadixHttpMethod::empty());

        // Process hosts
        let hosts = route
            .hosts
            .as_ref()
            .map(|hosts| hosts.iter().map(|h| HostPattern::new(h)).collect());

        // Process path (extract parameters)
        let (actual_path, path_op, has_param) = self.parse_path(path);

        // Pre-compile regex pattern if path has parameters
        let compiled_pattern = if has_param {
            let (pattern, names) = self.generate_pattern(path)?;
            Some(std::sync::Arc::new((pattern, names)))
        } else {
            None
        };

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
            compiled_pattern,
        })
    }

    /// Parse path and extract parameter information
    fn parse_path(&self, path: &str) -> (String, PathOp, bool) {
        // Check for parameter :param
        if let Some(pos) = path.find(':') {
            let actual_path = &path[..pos];
            return (actual_path.to_string(), PathOp::PrefixMatch, true);
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

    /// Match a route (thread-safe, immutable)
    ///
    /// Returns:
    /// - `Ok(Some(MatchResult))` - Found a matching route
    /// - `Ok(None)` - No matching route found
    /// - `Err(_)` - System error (e.g., RwLock poisoned)
    pub fn match_route(&self, path: &str, opts: &RadixMatchOpts) -> Result<Option<MatchResult>> {
        // Normalize host to lowercase if present
        let normalized_opts = if let Some(host) = &opts.host {
            let mut new_opts = opts.clone();
            new_opts.host = Some(host.to_lowercase());
            new_opts
        } else {
            opts.clone()
        };

        // Storage for matched parameters
        let mut matched = HashMap::new();

        // Priority 1: Check hash_path for exact match (lock-free read)
        if let Some(routes) = self.hash_path.get(path) {
            for route in routes.iter() {
                if self.match_route_opts(route, path, &normalized_opts, &mut matched) {
                    matched.insert("_path".to_string(), path.to_string());
                    return Ok(Some(MatchResult {
                        metadata: route.metadata.clone(),
                        matched,
                    }));
                }
                matched.clear(); // Clear for next iteration
            }
        }

        // Priority 2: Use radix tree for prefix matching
        // Create a temporary iterator for this query (thread-safe and async-safe)
        let tree_guard = self
            .tree
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on radix tree: {}", e))?;

        let mut iterator = tree_guard
            .new_iterator()
            .context("Failed to create radix tree iterator")?;

        // Search for matching prefixes
        if !iterator.search(tree_guard.tree_ptr(), path.as_bytes()) {
            return Ok(None);
        }

        // Iterate through matching routes (lock-free read from match_data)
        while let Some(idx) = iterator.tree_up(path.as_bytes()) {
            if let Some(routes) = self.match_data.get(&idx) {
                for route in routes.iter() {
                    if self.match_route_opts(route, path, &normalized_opts, &mut matched) {
                        matched.insert("_path".to_string(), route.path_org.clone());
                        return Ok(Some(MatchResult {
                            metadata: route.metadata.clone(),
                            matched,
                        }));
                    }
                    matched.clear(); // Clear for next iteration
                }
            }
        }

        Ok(None)
    }

    /// Match route options
    fn match_route_opts(
        &self,
        route: &RouteOpts,
        path: &str,
        opts: &RadixMatchOpts,
        matched: &mut HashMap<String, String>,
    ) -> bool {
        // 1. HTTP method matching
        if !route.methods.is_empty() {
            if let Some(method) = &opts.method {
                if let Some(m) = RadixHttpMethod::from_str(method) {
                    if !route.methods.contains(m) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        if let Some(method) = &opts.method {
            matched.insert("_method".to_string(), method.clone());
        }

        // 2. Host matching
        if let Some(hosts) = &route.hosts {
            let mut matched_host = false;
            if let Some(host) = &opts.host {
                for pattern in hosts {
                    if pattern.matches(host) {
                        let host_value = if pattern.is_wildcard {
                            format!("*{}", pattern.pattern)
                        } else {
                            host.clone()
                        };
                        matched.insert("_host".to_string(), host_value);
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
        if !self.compare_param(path, route, matched) {
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
    fn compare_param(
        &self,
        req_path: &str,
        route: &RouteOpts,
        matched: &mut HashMap<String, String>,
    ) -> bool {
        if !route.has_param {
            return true;
        }

        // Use pre-compiled pattern (no cache lookup needed!)
        let (pattern, names) = match &route.compiled_pattern {
            Some(compiled) => {
                let arc_ref = compiled.as_ref();
                (&arc_ref.0, &arc_ref.1)
            }
            None => return true, // No pattern means no parameters to extract
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
            for (i, name) in names.iter().enumerate() {
                if let Some(cap) = captures.get(i + 1) {
                    matched.insert(name.clone(), cap.as_str().to_string());
                }
            }

            true
        } else {
            false
        }
    }

    /// Generate regex pattern for path with parameters
    fn generate_pattern(&self, path: &str) -> Result<(Regex, Vec<String>)> {
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
        let pattern = Regex::new(&pattern_str)
            .with_context(|| format!("Failed to compile regex pattern for path: {}", path))?;

        Ok((pattern, names))
    }

    /// Update an existing route
    pub fn update_route(&mut self, old_route: RadixNode, new_route: RadixNode) -> Result<()> {
        // Remove old route
        self.delete_route(old_route)?;
        // Add new route
        self.add_route(new_route)?;
        Ok(())
    }

    /// Delete a route
    pub fn delete_route(&mut self, route: RadixNode) -> Result<()> {
        for path in &route.paths {
            self.remove_route(path, &route)?;
        }
        Ok(())
    }

    /// Remove a specific route from a path
    fn remove_route(&mut self, path: &str, route: &RadixNode) -> Result<()> {
        let route_opts = self.process_route(path, route)?;

        // Check hash_path first (for exact match routes)
        if route_opts.path_op == PathOp::Equal {
            if let Some(routes) = self.hash_path.get_mut(&route_opts.path) {
                routes.retain(|r| r.id != route_opts.id);
                if routes.is_empty() {
                    self.hash_path.remove(&route_opts.path);
                }
                return Ok(());
            }
            anyhow::bail!("Route not found in hash_path: {}", route.id);
        }

        // Find in radix tree
        if let Some(idx) = self
            .tree
            .read()
            .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?
            .find(route_opts.path.as_bytes())
        {
            if let Some(routes) = self.match_data.get_mut(&idx) {
                routes.retain(|r| r.id != route_opts.id);

                if routes.is_empty() {
                    // Remove from tree if no routes left
                    self.match_data.remove(&idx);
                    self.tree
                        .write()
                        .map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?
                        .remove(route_opts.path.as_bytes());
                }
                return Ok(());
            }
        }

        anyhow::bail!("Route not found: {}", route.id)
    }
}

impl std::fmt::Debug for RadixRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadixRouter")
            .field("match_data_index", &self.match_data_index)
            .field("hash_path_count", &self.hash_path.len())
            .field("match_data_count", &self.match_data.len())
            .finish()
    }
}
