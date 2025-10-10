//! # router-radix
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
//! use router_radix::{RadixRouter, RadixNode, RadixHttpMethod, RadixMatchOpts};
//! use std::collections::HashMap;
//!
//! # fn main() -> anyhow::Result<()> {
//! let routes = vec![
//!     RadixNode {
//!         id: "1".to_string(),
//!         paths: vec!["/api/users".to_string()],
//!         methods: Some(RadixHttpMethod::GET),
//!         hosts: None,
//!         remote_addrs: None,
//!         vars: None,
//!         filter_fn: None,
//!         priority: 0,
//!         metadata: serde_json::json!({"handler": "get_users"}),
//!     },
//!     RadixNode {
//!         id: "2".to_string(),
//!         paths: vec!["/api/user/:id".to_string()],
//!         methods: Some(RadixHttpMethod::GET),
//!         hosts: None,
//!         remote_addrs: None,
//!         vars: None,
//!         filter_fn: None,
//!         priority: 0,
//!         metadata: serde_json::json!({"handler": "get_user"}),
//!     },
//! ];
//!
//! let router = RadixRouter::new(routes)?;
//!
//! let opts = RadixMatchOpts {
//!     method: Some("GET".to_string()),
//!     ..Default::default()
//! };
//!
//! // Match exact path
//! let result = router.match_route("/api/users", &opts)?;
//! assert!(result.is_some());
//!
//! // Match with parameter extraction
//! let result = router.match_route("/api/user/123", &opts)?;
//! assert!(result.is_some());
//! let result = result.unwrap();
//! assert_eq!(result.matched.get("id").unwrap(), "123");
//! # Ok(())
//! # }
//! ```

mod ffi;
mod route;
mod router;

// Re-export public types
pub use route::{Expr, FilterFn, HostPattern, RadixHttpMethod, RadixMatchOpts, MatchResult, RadixNode};
pub use router::RadixRouter;

// Re-export anyhow types for convenience
pub use anyhow::{Context, Result};

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, sync::Arc};

    #[test]
    fn test_basic_match() {
        let routes = vec![RadixNode {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(RadixHttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "get_users"}),
        }];

        let router = RadixRouter::new(routes).unwrap();

        let opts = RadixMatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        let result = router.match_route("/api/users", &opts).unwrap();
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.metadata["handler"], "get_users");
    }

    #[test]
    fn test_method_not_allowed() {
        let routes = vec![RadixNode {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(RadixHttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "get_users"}),
        }];

        let router = RadixRouter::new(routes).unwrap();

        let opts = RadixMatchOpts {
            method: Some("POST".to_string()),
            ..Default::default()
        };

        let result = router.match_route("/api/users", &opts).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_param_extraction() {
        let routes = vec![RadixNode {
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

        let router = RadixRouter::new(routes).unwrap();

        let opts = RadixMatchOpts::default();

        let result = router.match_route("/user/123/post/456", &opts).unwrap();

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.matched.get("id").unwrap(), "123");
        assert_eq!(result.matched.get("pid").unwrap(), "456");
    }

    #[test]
    fn test_wildcard() {
        let routes = vec![RadixNode {
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

        let router = RadixRouter::new(routes).unwrap();

        let opts = RadixMatchOpts::default();

        let result = router.match_route("/files/documents/readme.txt", &opts).unwrap();

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.matched.get("path").unwrap(), "documents/readme.txt");
    }

    #[test]
    fn test_wildcard_host() {
        let routes = vec![RadixNode {
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

        let router = RadixRouter::new(routes).unwrap();

        let opts = RadixMatchOpts {
            host: Some("api.example.com".to_string()),
            ..Default::default()
        };

        let result = router.match_route("/api", &opts).unwrap();
        assert!(result.is_some());

        // Test non-matching host
        let opts = RadixMatchOpts {
            host: Some("api.other.com".to_string()),
            ..Default::default()
        };
        let result = router.match_route("/api", &opts).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_priority() {
        let routes = vec![
            RadixNode {
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
            RadixNode {
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

        let router = RadixRouter::new(routes).unwrap();

        let opts = RadixMatchOpts::default();
        let result = router.match_route("/api/users", &opts).unwrap();

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.metadata["handler"], "high");
    }

    #[test]
    fn test_multiple_methods() {
        let routes = vec![RadixNode {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(RadixHttpMethod::GET | RadixHttpMethod::POST),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "users"}),
        }];

        let router = RadixRouter::new(routes).unwrap();

        // Test GET
        let opts = RadixMatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };
        assert!(router.match_route("/api/users", &opts).unwrap().is_some());

        // Test POST
        let opts = RadixMatchOpts {
            method: Some("POST".to_string()),
            ..Default::default()
        };
        assert!(router.match_route("/api/users", &opts).unwrap().is_some());

        // Test DELETE (not allowed)
        let opts = RadixMatchOpts {
            method: Some("DELETE".to_string()),
            ..Default::default()
        };
        assert!(router.match_route("/api/users", &opts).unwrap().is_none());
    }

    #[test]
    fn test_filter_function() {
        let routes = vec![RadixNode {
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

        let router = RadixRouter::new(routes).unwrap();

        // Without version variable
        let opts = RadixMatchOpts::default();
        assert!(router.match_route("/api/users", &opts).unwrap().is_none());

        // With correct version
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), "v2".to_string());
        let opts = RadixMatchOpts {
            vars: Some(vars),
            ..Default::default()
        };
        assert!(router.match_route("/api/users", &opts).unwrap().is_some());

        // With incorrect version
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), "v1".to_string());
        let opts = RadixMatchOpts {
            vars: Some(vars),
            ..Default::default()
        };
        assert!(router.match_route("/api/users", &opts).unwrap().is_none());
    }

    #[test]
    fn test_expression_matching() {
        use regex::Regex;

        let routes = vec![RadixNode {
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

        let router = RadixRouter::new(routes).unwrap();

        // Without variables
        let opts = RadixMatchOpts::default();
        assert!(router.match_route("/api/users", &opts).unwrap().is_none());

        // With correct variables
        let mut vars = HashMap::new();
        vars.insert("env".to_string(), "production".to_string());
        vars.insert("user_agent".to_string(), "Chrome/90.0".to_string());
        let opts = RadixMatchOpts {
            vars: Some(vars),
            ..Default::default()
        };
        assert!(router.match_route("/api/users", &opts).unwrap().is_some());

        // With incorrect env
        let mut vars = HashMap::new();
        vars.insert("env".to_string(), "development".to_string());
        vars.insert("user_agent".to_string(), "Chrome/90.0".to_string());
        let opts = RadixMatchOpts {
            vars: Some(vars),
            ..Default::default()
        };
        assert!(router.match_route("/api/users", &opts).unwrap().is_none());
    }

    #[test]
    fn test_add_and_delete_route() {
        let mut router = RadixRouter::new(vec![]).unwrap();

        // Add route
        let route = RadixNode {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(RadixHttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "get_users"}),
        };

        router.add_route(route.clone()).unwrap();

        let opts = RadixMatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        // Should match
        assert!(router.match_route("/api/users", &opts).unwrap().is_some());

        // Delete route
        router.delete_route(route).unwrap();

        // Should not match
        assert!(router.match_route("/api/users", &opts).unwrap().is_none());
    }
}
