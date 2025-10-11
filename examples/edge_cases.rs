/// Edge cases and boundary conditions testing
/// This example tests various edge cases and boundary conditions to ensure router robustness
use router_radix::{RadixHttpMethod, RadixMatchOpts, RadixRouter, RadixNode};

fn main() -> anyhow::Result<()> {
    println!("=== Edge Cases & Boundary Conditions Test ===\n");

    // Test 1: Empty and root paths
    println!("Test 1: Empty and root paths");
    {
        let routes = vec![
            RadixNode {
                id: "root".to_string(),
                paths: vec!["/".to_string()],
                methods: Some(RadixHttpMethod::GET),
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "root"}),
            },
            RadixNode {
                id: "api".to_string(),
                paths: vec!["/api".to_string()],
                methods: Some(RadixHttpMethod::GET),
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "api"}),
            },
        ];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;
        let opts = RadixMatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        let result = router.match_route("/", &opts)?;
        assert!(result.is_some());
        println!("  ✓ Root path '/' matched");

        let result = router.match_route("/api", &opts)?;
        assert!(result.is_some());
        println!("  ✓ Path '/api' matched");
    }
    println!();

    // Test 2: Paths with special characters
    println!("Test 2: Paths with special characters");
    {
        let routes = vec![
            RadixNode {
                id: "special1".to_string(),
                paths: vec!["/api/user-profile".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "user_profile"}),
            },
            RadixNode {
                id: "special2".to_string(),
                paths: vec!["/api/user_data".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "user_data"}),
            },
            RadixNode {
                id: "special3".to_string(),
                paths: vec!["/api/user.info".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "user_info"}),
            },
        ];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;
        let opts = RadixMatchOpts::default();

        assert!(router.match_route("/api/user-profile", &opts)?.is_some());
        println!("  ✓ Path with hyphen '-' matched");

        assert!(router.match_route("/api/user_data", &opts)?.is_some());
        println!("  ✓ Path with underscore '_' matched");

        assert!(router.match_route("/api/user.info", &opts)?.is_some());
        println!("  ✓ Path with dot '.' matched");
    }
    println!();

    // Test 3: Very long paths
    println!("Test 3: Very long paths");
    {
        let long_path =
            "/api/v1/users/profiles/details/personal/information/extended/metadata/attributes";
        let routes = vec![RadixNode {
            id: "long".to_string(),
            paths: vec![long_path.to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "long_path"}),
        }];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;
        let opts = RadixMatchOpts::default();

        let result = router.match_route(long_path, &opts)?;
        assert!(result.is_some());
        println!("  ✓ Very long path matched (length: {})", long_path.len());
    }
    println!();

    // Test 4: Similar paths (prefix matching)
    println!("Test 4: Similar paths (prefix matching)");
    {
        let routes = vec![
            RadixNode {
                id: "1".to_string(),
                paths: vec!["/api/user".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "user"}),
            },
            RadixNode {
                id: "2".to_string(),
                paths: vec!["/api/users".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "users"}),
            },
            RadixNode {
                id: "3".to_string(),
                paths: vec!["/api/user/:id".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "user_id"}),
            },
        ];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;
        let opts = RadixMatchOpts::default();

        let result = router.match_route("/api/user", &opts)?.unwrap();
        assert_eq!(result.metadata["handler"], "user");
        println!("  ✓ '/api/user' matched correctly");

        let result = router.match_route("/api/users", &opts)?.unwrap();
        assert_eq!(result.metadata["handler"], "users");
        println!("  ✓ '/api/users' matched correctly");

        let result = router.match_route("/api/user/123", &opts)?.unwrap();
        assert_eq!(result.metadata["handler"], "user_id");
        println!("  ✓ '/api/user/123' matched correctly");
    }
    println!();

    // Test 5: Multiple wildcards (conflicting routes)
    println!("Test 5: Multiple wildcard patterns");
    {
        let routes = vec![
            RadixNode {
                id: "wild1".to_string(),
                paths: vec!["/files/*path".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 5,
                metadata: serde_json::json!({"handler": "files"}),
            },
            RadixNode {
                id: "wild2".to_string(),
                paths: vec!["/files/public/*".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 10,
                metadata: serde_json::json!({"handler": "public_files"}),
            },
        ];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;
        let opts = RadixMatchOpts::default();

        // More specific route (with priority) should match first
        let result = router.match_route("/files/public/doc.pdf", &opts)?.unwrap();
        println!("  ✓ Matched: {}", result.metadata["handler"]);
        assert_eq!(result.matched.get("_path").unwrap(), "/files/public/*");
    }
    println!();

    // Test 6: Parameters with special values
    println!("Test 6: Parameters with special values");
    {
        let routes = vec![RadixNode {
            id: "param".to_string(),
            paths: vec!["/api/resource/:id".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "resource"}),
        }];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;
        let opts = RadixMatchOpts::default();

        // Test with UUID
        let result = router
            .match_route("/api/resource/550e8400-e29b-41d4-a716-446655440000", &opts)?
            .unwrap();
        assert_eq!(
            result.matched.get("id").unwrap(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        println!("  ✓ UUID parameter extracted");

        // Test with number
        let result = router.match_route("/api/resource/12345", &opts)?.unwrap();
        assert_eq!(result.matched.get("id").unwrap(), "12345");
        println!("  ✓ Numeric parameter extracted");

        // Test with encoded string
        let result = router
            .match_route("/api/resource/hello-world_123", &opts)?
            .unwrap();
        assert_eq!(result.matched.get("id").unwrap(), "hello-world_123");
        println!("  ✓ Alphanumeric parameter extracted");
    }
    println!();

    // Test 7: Trailing slashes
    println!("Test 7: Trailing slashes");
    {
        let routes = vec![RadixNode {
            id: "slash".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "users"}),
        }];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;
        let opts = RadixMatchOpts::default();

        let result = router.match_route("/api/users", &opts)?;
        assert!(result.is_some());
        println!("  ✓ Path without trailing slash matched");

        let result = router.match_route("/api/users/", &opts)?;
        println!(
            "  ℹ Path with trailing slash: {}",
            if result.is_some() {
                "matched"
            } else {
                "not matched"
            }
        );
    }
    println!();

    // Test 8: Case sensitivity
    println!("Test 8: Case sensitivity");
    {
        let routes = vec![RadixNode {
            id: "case".to_string(),
            paths: vec!["/API/Users".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "users"}),
        }];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;
        let opts = RadixMatchOpts::default();

        let result = router.match_route("/API/Users", &opts)?;
        assert!(result.is_some());
        println!("  ✓ Exact case matched");

        let result = router.match_route("/api/users", &opts)?;
        println!(
            "  ℹ Different case: {}",
            if result.is_some() {
                "matched (case insensitive)"
            } else {
                "not matched (case sensitive)"
            }
        );
    }
    println!();

    // Test 9: Empty router
    println!("Test 9: Empty router");
    {
        let router = RadixRouter::new()?;
        let opts = RadixMatchOpts::default();

        let result = router.match_route("/any/path", &opts)?;
        assert!(result.is_none());
        println!("  ✓ Empty router returns None");
    }
    println!();

    // Test 10: Host with port number
    println!("Test 10: Host with port number");
    {
        let routes = vec![RadixNode {
            id: "host_port".to_string(),
            paths: vec!["/api".to_string()],
            methods: None,
            hosts: Some(vec!["example.com".to_string()]),
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "api"}),
        }];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;

        // Test without port
        let opts = RadixMatchOpts {
            host: Some("example.com".to_string()),
            ..Default::default()
        };
        assert!(router.match_route("/api", &opts)?.is_some());
        println!("  ✓ Host without port matched");

        // Test with port
        let opts = RadixMatchOpts {
            host: Some("example.com:8080".to_string()),
            ..Default::default()
        };
        let result = router.match_route("/api", &opts)?;
        println!(
            "  ℹ Host with port: {}",
            if result.is_some() {
                "matched"
            } else {
                "not matched"
            }
        );
    }
    println!();

    // Test 11: All HTTP methods
    println!("Test 11: All HTTP methods");
    {
        let all_methods = RadixHttpMethod::GET
            | RadixHttpMethod::POST
            | RadixHttpMethod::PUT
            | RadixHttpMethod::DELETE
            | RadixHttpMethod::PATCH
            | RadixHttpMethod::HEAD
            | RadixHttpMethod::OPTIONS;

        let routes = vec![RadixNode {
            id: "all_methods".to_string(),
            paths: vec!["/api/resource".to_string()],
            methods: Some(all_methods),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "resource"}),
        }];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;

        let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        for method in methods {
            let opts = RadixMatchOpts {
                method: Some(method.to_string()),
                ..Default::default()
            };
            assert!(router.match_route("/api/resource", &opts)?.is_some());
            println!("  ✓ Method {} matched", method);
        }
    }
    println!();

    // Test 12: Nested parameters
    println!("Test 12: Nested parameters");
    {
        let routes = vec![RadixNode {
            id: "nested".to_string(),
            paths: vec!["/org/:org_id/team/:team_id/user/:user_id".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "nested"}),
        }];

        let mut router = RadixRouter::new()?;
        router.add_routes(routes)?;
        let opts = RadixMatchOpts::default();

        let result = router
            .match_route("/org/acme/team/engineering/user/john", &opts)?
            .unwrap();
        assert_eq!(result.matched.get("org_id").unwrap(), "acme");
        assert_eq!(result.matched.get("team_id").unwrap(), "engineering");
        assert_eq!(result.matched.get("user_id").unwrap(), "john");
        println!("  ✓ All nested parameters extracted correctly");
        println!("    org_id: {}", result.matched.get("org_id").unwrap());
        println!("    team_id: {}", result.matched.get("team_id").unwrap());
        println!("    user_id: {}", result.matched.get("user_id").unwrap());
    }
    println!();

    println!("=== All Edge Cases Tests Passed ✓ ===");
    Ok(())
}
