/// Performance benchmarks for different routing scenarios
/// This example measures and compares performance across various route types and patterns
use radix_router::{HttpMethod, MatchOpts, RadixRouter, Route};
use std::time::Instant;

fn benchmark(name: &str, iterations: usize, f: impl Fn()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    let ns_per_op = elapsed.as_nanos() / iterations as u128;

    println!(
        "  {:45} {:>12.0} ops/sec  {:>8} ns/op",
        name, ops_per_sec, ns_per_op
    );
}

fn main() -> anyhow::Result<()> {
    println!("=== Router Performance Benchmarks ===\n");

    let iterations = 1_000_000;
    println!("Running {} iterations for each benchmark\n", iterations);

    // Benchmark 1: Exact path matching
    println!("Benchmark 1: Exact Path Matching");
    {
        let routes = vec![
            Route {
                id: "1".to_string(),
                paths: vec!["/api/users".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"id": 1}),
            },
            Route {
                id: "2".to_string(),
                paths: vec!["/api/posts".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"id": 2}),
            },
            Route {
                id: "3".to_string(),
                paths: vec!["/api/comments".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"id": 3}),
            },
        ];

        let router = RadixRouter::new(routes)?;
        let opts = MatchOpts::default();

        benchmark("Exact match (hash lookup)", iterations, || {
            let _ = router.match_route("/api/users", &opts).ok();
        });
    }
    println!();

    // Benchmark 2: Single parameter extraction
    println!("Benchmark 2: Parameter Extraction");
    {
        let routes = vec![Route {
            id: "param".to_string(),
            paths: vec!["/api/user/:id".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "param"}),
        }];

        let router = RadixRouter::new(routes)?;
        let opts = MatchOpts::default();

        benchmark("Single parameter", iterations, || {
            let _ = router.match_route("/api/user/12345", &opts).ok();
        });
    }
    println!();

    // Benchmark 3: Multiple parameters
    println!("Benchmark 3: Multiple Parameters");
    {
        let routes = vec![Route {
            id: "multi_param".to_string(),
            paths: vec!["/api/user/:uid/post/:pid/comment/:cid".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "multi_param"}),
        }];

        let router = RadixRouter::new(routes)?;
        let opts = MatchOpts::default();

        benchmark("Three parameters", iterations, || {
            let _ = router
                .match_route("/api/user/123/post/456/comment/789", &opts)
                .ok();
        });
    }
    println!();

    // Benchmark 4: Wildcard matching
    println!("Benchmark 4: Wildcard Matching");
    {
        let routes = vec![Route {
            id: "wildcard".to_string(),
            paths: vec!["/files/*path".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "wildcard"}),
        }];

        let router = RadixRouter::new(routes)?;
        let opts = MatchOpts::default();

        benchmark("Short wildcard path", iterations, || {
            let _ = router.match_route("/files/doc.pdf", &opts).ok();
        });

        benchmark("Long wildcard path", iterations, || {
            let _ = router
                .match_route("/files/a/b/c/d/e/f/g/document.pdf", &opts)
                .ok();
        });
    }
    println!();

    // Benchmark 5: HTTP method matching
    println!("Benchmark 5: HTTP Method Matching");
    {
        let routes = vec![Route {
            id: "method".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(HttpMethod::GET | HttpMethod::POST | HttpMethod::PUT),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "method"}),
        }];

        let router = RadixRouter::new(routes)?;

        let opts_get = MatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        benchmark("Method matching (allowed)", iterations, || {
            let _ = router.match_route("/api/users", &opts_get).ok();
        });

        let opts_delete = MatchOpts {
            method: Some("DELETE".to_string()),
            ..Default::default()
        };

        benchmark("Method matching (rejected)", iterations, || {
            let _ = router.match_route("/api/users", &opts_delete).ok();
        });
    }
    println!();

    // Benchmark 6: Host matching
    println!("Benchmark 6: Host Matching");
    {
        let routes = vec![Route {
            id: "host".to_string(),
            paths: vec!["/api".to_string()],
            methods: None,
            hosts: Some(vec!["api.example.com".to_string()]),
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "host"}),
        }];

        let router = RadixRouter::new(routes)?;

        let opts = MatchOpts {
            host: Some("api.example.com".to_string()),
            ..Default::default()
        };

        benchmark("Exact host match", iterations, || {
            let _ = router.match_route("/api", &opts).ok();
        });
    }
    println!();

    // Benchmark 7: Wildcard host matching
    println!("Benchmark 7: Wildcard Host Matching");
    {
        let routes = vec![Route {
            id: "wildcard_host".to_string(),
            paths: vec!["/api".to_string()],
            methods: None,
            hosts: Some(vec!["*.example.com".to_string()]),
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "wildcard_host"}),
        }];

        let router = RadixRouter::new(routes)?;

        let opts = MatchOpts {
            host: Some("api.example.com".to_string()),
            ..Default::default()
        };

        benchmark("Wildcard host match", iterations, || {
            let _ = router.match_route("/api", &opts).ok();
        });
    }
    println!();

    // Benchmark 8: Priority-based routing
    println!("Benchmark 8: Priority-Based Routing");
    {
        let routes = vec![
            Route {
                id: "low".to_string(),
                paths: vec!["/api/*".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"priority": "low"}),
            },
            Route {
                id: "medium".to_string(),
                paths: vec!["/api/users".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 5,
                metadata: serde_json::json!({"priority": "medium"}),
            },
            Route {
                id: "high".to_string(),
                paths: vec!["/api/users".to_string()],
                methods: Some(HttpMethod::GET),
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 10,
                metadata: serde_json::json!({"priority": "high"}),
            },
        ];

        let router = RadixRouter::new(routes)?;

        let opts = MatchOpts {
            method: Some("GET".to_string()),
            ..Default::default()
        };

        benchmark("Priority resolution (3 candidates)", iterations, || {
            let _ = router.match_route("/api/users", &opts).ok();
        });
    }
    println!();

    // Benchmark 9: Router with many routes
    println!("Benchmark 9: Large Router (100 routes)");
    {
        let mut routes = Vec::new();
        for i in 0..100 {
            routes.push(Route {
                id: format!("route_{}", i),
                paths: vec![format!("/api/endpoint_{}", i)],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"id": i}),
            });
        }

        let router = RadixRouter::new(routes)?;
        let opts = MatchOpts::default();

        benchmark("Match first route (100 total)", iterations, || {
            let _ = router.match_route("/api/endpoint_0", &opts).ok();
        });

        benchmark("Match middle route (100 total)", iterations, || {
            let _ = router.match_route("/api/endpoint_50", &opts).ok();
        });

        benchmark("Match last route (100 total)", iterations, || {
            let _ = router.match_route("/api/endpoint_99", &opts).ok();
        });

        benchmark("No match (100 total)", iterations, || {
            let _ = router.match_route("/api/nonexistent", &opts).ok();
        });
    }
    println!();

    // Benchmark 10: Complex routing scenario
    println!("Benchmark 10: Complex Real-World Scenario");
    {
        let routes = vec![
            Route {
                id: "api_users".to_string(),
                paths: vec!["/api/v1/users".to_string()],
                methods: Some(HttpMethod::GET | HttpMethod::POST),
                hosts: Some(vec!["api.example.com".to_string()]),
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 10,
                metadata: serde_json::json!({"handler": "users"}),
            },
            Route {
                id: "api_user_detail".to_string(),
                paths: vec!["/api/v1/user/:id".to_string()],
                methods: Some(HttpMethod::GET | HttpMethod::PUT | HttpMethod::DELETE),
                hosts: Some(vec!["api.example.com".to_string()]),
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 10,
                metadata: serde_json::json!({"handler": "user_detail"}),
            },
            Route {
                id: "static_files".to_string(),
                paths: vec!["/static/*path".to_string()],
                methods: None,
                hosts: None,
                remote_addrs: None,
                vars: None,
                filter_fn: None,
                priority: 0,
                metadata: serde_json::json!({"handler": "static"}),
            },
        ];

        let router = RadixRouter::new(routes)?;

        let opts = MatchOpts {
            method: Some("GET".to_string()),
            host: Some("api.example.com".to_string()),
            ..Default::default()
        };

        benchmark("Complex: exact + method + host", iterations, || {
            let _ = router.match_route("/api/v1/users", &opts).ok();
        });

        benchmark("Complex: param + method + host", iterations, || {
            let _ = router.match_route("/api/v1/user/12345", &opts).ok();
        });

        let opts_static = MatchOpts::default();
        benchmark("Complex: wildcard (no constraints)", iterations, || {
            let _ = router
                .match_route("/static/css/main.css", &opts_static)
                .ok();
        });
    }
    println!();

    println!("=== Benchmark Summary ===");
    println!("• Exact path matching: Fastest (hash-based lookup)");
    println!("• Parameter extraction: Very fast (pre-compiled regex)");
    println!("• Wildcard matching: Fast (minimal overhead)");
    println!("• Method/Host matching: Negligible overhead");
    println!("• Large routers: O(1) hash lookup for exact, O(log n) for prefix");
    println!("• Complex scenarios: Performance scales linearly with constraints");

    Ok(())
}
