//! # radix-router
//!
//! A high-performance radix tree based HTTP router for Rust.
//!
//! This is a Rust port of [lua-resty-radixtree](https://github.com/api7/lua-resty-radixtree),
//! providing fast routing with support for:
//! - Path parameters (`:name`)
//! - Wildcards (`*`)
//! - HTTP method matching
//! - Host matching (with wildcards)
//! - Priority-based routing
//! - Custom filter functions
//! - Variable expressions
//!
//! ## Example
//!
//! ```rust
//! use radix_router::{RadixRouter, Route, HttpMethod, MatchOpts};
//! use std::collections::HashMap;
//!
//! let routes = vec![
//!     Route {
//!         id: "1".to_string(),
//!         paths: vec!["/api/users".to_string()],
//!         methods: Some(HttpMethod::GET),
//!         hosts: None,
//!         remote_addrs: None,
//!         vars: None,
//!         filter_fn: None,
//!         priority: 0,
//!         metadata: serde_json::json!({"handler": "get_users"}),
//!     },
//!     Route {
//!         id: "2".to_string(),
//!         paths: vec!["/api/user/:id".to_string()],
//!         methods: Some(HttpMethod::GET),
//!         hosts: None,
//!         remote_addrs: None,
//!         vars: None,
//!         filter_fn: None,
//!         priority: 0,
//!         metadata: serde_json::json!({"handler": "get_user"}),
//!     },
//! ];
//!
//! let mut router = RadixRouter::new(routes, None).unwrap();
//!
//! let mut opts = MatchOpts {
//!     method: Some("GET".to_string()),
//!     matched: Some(HashMap::new()),
//!     ..Default::default()
//! };
//!
//! // Match exact path
//! let result = router.match_route("/api/users", &mut opts);
//! assert!(result.is_some());
//!
//! // Match with parameter extraction
//! let result = router.match_route("/api/user/123", &mut opts);
//! assert!(result.is_some());
//! assert_eq!(opts.matched.as_ref().unwrap().get("id").unwrap(), "123");
//! ```

mod ffi;
mod route;
mod router;

// Re-export public types
pub use route::{Expr, FilterFn, HostPattern, HttpMethod, MatchOpts, Route};
pub use router::{RadixRouter, RouterOpts};

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, sync::Arc};

    #[test]
    fn test_basic_match() {
        let routes = vec![Route {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "get_users"}),
        }];

        let mut router = RadixRouter::new(routes, None).unwrap();

        let mut opts = MatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        let result = router.match_route("/api/users", &mut opts);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["handler"], "get_users");
    }

    #[test]
    fn test_method_not_allowed() {
        let routes = vec![Route {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "get_users"}),
        }];

        let mut router = RadixRouter::new(routes, None).unwrap();

        let mut opts = MatchOpts {
            method: Some("POST".to_string()),
            ..Default::default()
        };

        let result = router.match_route("/api/users", &mut opts);
        assert!(result.is_none());
    }

    #[test]
    fn test_param_extraction() {
        let routes = vec![Route {
            id: "1".to_string(),
            paths: vec!["/user/:id/post/:pid".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "user_post"}),
        }];

        let mut router = RadixRouter::new(routes, None).unwrap();

        let mut opts = MatchOpts {
            matched: Some(HashMap::new()),
            ..Default::default()
        };

        let result = router.match_route("/user/123/post/456", &mut opts);

        assert!(result.is_some());
        let matched = opts.matched.as_ref().unwrap();
        assert_eq!(matched.get("id").unwrap(), "123");
        assert_eq!(matched.get("pid").unwrap(), "456");
    }

    #[test]
    fn test_wildcard() {
        let routes = vec![Route {
            id: "1".to_string(),
            paths: vec!["/files/*path".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "serve_file"}),
        }];

        let mut router = RadixRouter::new(routes, None).unwrap();

        let mut opts = MatchOpts {
            matched: Some(HashMap::new()),
            ..Default::default()
        };

        let result = router.match_route("/files/documents/readme.txt", &mut opts);

        assert!(result.is_some());
        let matched = opts.matched.as_ref().unwrap();
        assert_eq!(matched.get("path").unwrap(), "documents/readme.txt");
    }

    #[test]
    fn test_wildcard_host() {
        let routes = vec![Route {
            id: "1".to_string(),
            paths: vec!["/api".to_string()],
            methods: None,
            hosts: Some(vec!["*.example.com".to_string()]),
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "api"}),
        }];

        let mut router = RadixRouter::new(routes, None).unwrap();

        let mut opts = MatchOpts {
            host: Some("api.example.com".to_string()),
            ..Default::default()
        };

        let result = router.match_route("/api", &mut opts);
        assert!(result.is_some());

        // Test non-matching host
        opts.host = Some("api.other.com".to_string());
        let result = router.match_route("/api", &mut opts);
        assert!(result.is_none());
    }

    #[test]
    fn test_priority() {
        let routes = vec![
            Route {
                id: "1".to_string(),
                paths: vec!["/api/*".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "low"}),
            },
            Route {
                id: "2".to_string(),
                paths: vec!["/api/users".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 10,
                metadata: serde_json::json!({"handler": "high"}),
            },
        ];

        let mut router = RadixRouter::new(routes, None).unwrap();

        let mut opts = MatchOpts::default();
        let result = router.match_route("/api/users", &mut opts);

        assert!(result.is_some());
        assert_eq!(result.unwrap()["handler"], "high");
    }

    #[test]
    fn test_multiple_methods() {
        let routes = vec![Route {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(HttpMethod::GET | HttpMethod::POST),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "users"}),
        }];

        let mut router = RadixRouter::new(routes, None).unwrap();

        // Test GET
        let mut opts = MatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };
        assert!(router.match_route("/api/users", &mut opts).is_some());

        // Test POST
        opts.method = Some("POST".to_string());
        assert!(router.match_route("/api/users", &mut opts).is_some());

        // Test DELETE (not allowed)
        opts.method = Some("DELETE".to_string());
        assert!(router.match_route("/api/users", &mut opts).is_none());
    }

    #[test]
    fn test_filter_function() {
        let routes = vec![Route {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: Some(Arc::new(|vars, _opts| {
                vars.get("version").map(|v| v == "v2").unwrap_or(false)
            })),
            priority: 0,
            metadata: serde_json::json!({"handler": "users_v2"}),
        }];

        let mut router = RadixRouter::new(routes, None).unwrap();

        // Without version variable
        let mut opts = MatchOpts::default();
        assert!(router.match_route("/api/users", &mut opts).is_none());

        // With correct version
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), "v2".to_string());
        opts.vars = Some(vars);
        assert!(router.match_route("/api/users", &mut opts).is_some());

        // With incorrect version
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), "v1".to_string());
        opts.vars = Some(vars);
        assert!(router.match_route("/api/users", &mut opts).is_none());
    }

    #[test]
    fn test_expression_matching() {
        use regex::Regex;

        let routes = vec![Route {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: Some(vec![
                Expr::Eq("env".to_string(), "production".to_string()),
                Expr::Regex("user_agent".to_string(), Regex::new("Chrome").unwrap()),
            ]),
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "users"}),
        }];

        let mut router = RadixRouter::new(routes, None).unwrap();

        // Without variables
        let mut opts = MatchOpts::default();
        assert!(router.match_route("/api/users", &mut opts).is_none());

        // With correct variables
        let mut vars = HashMap::new();
        vars.insert("env".to_string(), "production".to_string());
        vars.insert("user_agent".to_string(), "Chrome/90.0".to_string());
        opts.vars = Some(vars);
        assert!(router.match_route("/api/users", &mut opts).is_some());

        // With incorrect env
        let mut vars = HashMap::new();
        vars.insert("env".to_string(), "development".to_string());
        vars.insert("user_agent".to_string(), "Chrome/90.0".to_string());
        opts.vars = Some(vars);
        assert!(router.match_route("/api/users", &mut opts).is_none());
    }

    #[test]
    fn test_add_and_delete_route() {
        let mut router = RadixRouter::new(vec![], None).unwrap();

        // Add route
        let route = Route {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "get_users"}),
        };

        router.add_route(route.clone()).unwrap();

        let mut opts = MatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        // Should match
        assert!(router.match_route("/api/users", &mut opts).is_some());

        // Delete route
        router.delete_route(route).unwrap();

        // Should not match
        assert!(router.match_route("/api/users", &mut opts).is_none());
    }
}
