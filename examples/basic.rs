use radix_router::{HttpMethod, MatchOpts, RadixRouter, Route};

fn main() -> anyhow::Result<()> {
    // Create routes
    let routes = vec![
        Route {
            id: "1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({
                "handler": "get_users",
                "upstream": "user-service:8001"
            }),
        },
        Route {
            id: "2".to_string(),
            paths: vec!["/api/user/:id".to_string()],
            methods: Some(HttpMethod::GET | HttpMethod::PUT | HttpMethod::DELETE),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({
                "handler": "user_detail",
                "upstream": "user-service:8001"
            }),
        },
        Route {
            id: "3".to_string(),
            paths: vec!["/api/user/:id/posts".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({
                "handler": "user_posts",
                "upstream": "post-service:8002"
            }),
        },
        Route {
            id: "4".to_string(),
            paths: vec!["/admin/*path".to_string()],
            methods: None,
            hosts: Some(vec!["admin.example.com".to_string()]),
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 10,
            metadata: serde_json::json!({
                "handler": "admin",
                "upstream": "admin-service:8003"
            }),
        },
        Route {
            id: "5".to_string(),
            paths: vec!["/api/*".to_string()],
            methods: None,
            hosts: Some(vec!["*.api.example.com".to_string()]),
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({
                "handler": "api_wildcard",
                "upstream": "api-gateway:8000"
            }),
        },
    ];

    // Create router
    let router = RadixRouter::new(routes)?;

    println!("=== Radix Router Examples ===\n");

    // Example 1: Exact path match
    {
        let opts = MatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        println!("1. Exact path match:");
        println!("   Path: /api/users");
        println!("   Method: GET");

        if let Some(result) = router.match_route("/api/users", &opts)? {
            println!("   ✓ Matched!");
            println!(
                "   Metadata: {}",
                serde_json::to_string_pretty(&result.metadata).unwrap()
            );
            println!("   Matched params: {:?}", result.matched);
        } else {
            println!("   ✗ No match");
        }
        println!();
    }

    // Example 2: Parameter extraction
    {
        let opts = MatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        println!("2. Parameter extraction:");
        println!("   Path: /api/user/12345");
        println!("   Method: GET");

        if let Some(result) = router.match_route("/api/user/12345", &opts)? {
            println!("   ✓ Matched!");
            println!(
                "   Metadata: {}",
                serde_json::to_string_pretty(&result.metadata).unwrap()
            );
            println!("   Matched params: {:?}", result.matched);
        } else {
            println!("   ✗ No match");
        }
        println!();
    }

    // Example 3: Multiple parameters
    {
        let opts = MatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        println!("3. Multiple parameters:");
        println!("   Path: /api/user/12345/posts");
        println!("   Method: GET");

        if let Some(result) = router.match_route("/api/user/12345/posts", &opts)? {
            println!("   ✓ Matched!");
            println!(
                "   Metadata: {}",
                serde_json::to_string_pretty(&result.metadata).unwrap()
            );
            println!("   Matched params: {:?}", result.matched);
        } else {
            println!("   ✗ No match");
        }
        println!();
    }

    // Example 4: Wildcard matching
    {
        let opts = MatchOpts {
            host: Some("admin.example.com".to_string()),
            ..Default::default()
        };

        println!("4. Wildcard path:");
        println!("   Path: /admin/dashboard/settings");
        println!("   Host: admin.example.com");

        if let Some(result) = router.match_route("/admin/dashboard/settings", &opts)? {
            println!("   ✓ Matched!");
            println!(
                "   Metadata: {}",
                serde_json::to_string_pretty(&result.metadata).unwrap()
            );
            println!("   Matched params: {:?}", result.matched);
        } else {
            println!("   ✗ No match");
        }
        println!();
    }

    // Example 5: Wildcard host
    {
        let opts = MatchOpts {
            host: Some("v1.api.example.com".to_string()),
            ..Default::default()
        };

        println!("5. Wildcard host:");
        println!("   Path: /api/health");
        println!("   Host: v1.api.example.com");

        if let Some(result) = router.match_route("/api/health", &opts)? {
            println!("   ✓ Matched!");
            println!(
                "   Metadata: {}",
                serde_json::to_string_pretty(&result.metadata).unwrap()
            );
            println!("   Matched params: {:?}", result.matched);
        } else {
            println!("   ✗ No match");
        }
        println!();
    }

    // Example 6: Method not allowed
    {
        let opts = MatchOpts {
            method: Some("POST".to_string()),
            ..Default::default()
        };

        println!("6. Method not allowed:");
        println!("   Path: /api/users");
        println!("   Method: POST (route only allows GET)");

        if let Some(result) = router.match_route("/api/users", &opts)? {
            println!("   ✓ Matched!");
            println!(
                "   Metadata: {}",
                serde_json::to_string_pretty(&result.metadata).unwrap()
            );
        } else {
            println!("   ✗ No match (method not allowed)");
        }
        println!();
    }

    // Example 7: Multiple methods allowed
    {
        let opts = MatchOpts {
            method: Some("PUT".to_string()),
            ..Default::default()
        };

        println!("7. Multiple methods allowed:");
        println!("   Path: /api/user/12345");
        println!("   Method: PUT (route allows GET, PUT, DELETE)");

        if let Some(result) = router.match_route("/api/user/12345", &opts)? {
            println!("   ✓ Matched!");
            println!(
                "   Metadata: {}",
                serde_json::to_string_pretty(&result.metadata).unwrap()
            );
            println!("   Matched params: {:?}", result.matched);
        } else {
            println!("   ✗ No match");
        }
        println!();
    }

    println!("=== Router Debug Info ===");
    println!("{:?}", router);

    Ok(())
}
