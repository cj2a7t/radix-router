/// Stress test with large number of routes and concurrent operations
/// This example tests router behavior under heavy load
use router_radix::{HttpMethod, MatchOpts, RadixRouter, Route};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    println!("=== Router Stress Test ===\n");

    // Test 1: Large number of routes
    println!("Test 1: Creating router with large number of routes");
    let num_routes = 10_000;
    println!("  Creating {} routes...", num_routes);

    let start = Instant::now();
    let mut routes = Vec::new();

    for i in 0..num_routes {
        let route_type = i % 5;
        let path = match route_type {
            0 => format!("/api/exact/{}", i),
            1 => format!("/api/param/{}/:id", i),
            2 => format!("/api/wildcard/{}/*path", i),
            3 => format!("/api/multi/{}/user/:uid/post/:pid", i),
            _ => format!("/api/complex/{}/resource/:rid", i),
        };

        routes.push(Route {
            id: format!("route_{}", i),
            paths: vec![path],
            methods: Some(HttpMethod::GET | HttpMethod::POST),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: i as i32 % 10,
            metadata: serde_json::json!({
                "route_id": i,
                "type": route_type,
            }),
        });
    }

    let router = RadixRouter::new(routes)?;
    let creation_time = start.elapsed();

    println!(
        "  ✓ Router created in {:.2}s ({:.2} routes/sec)",
        creation_time.as_secs_f64(),
        num_routes as f64 / creation_time.as_secs_f64()
    );
    println!();

    // Test 2: Sequential queries on large router
    println!("Test 2: Sequential query performance (large router)");
    let test_paths = vec![
        ("/api/exact/0", "first route"),
        ("/api/exact/5000", "middle route"),
        ("/api/exact/9999", "last route"),
        ("/api/param/100/abc123", "param route"),
        ("/api/wildcard/200/path/to/file.txt", "wildcard route"),
        ("/api/multi/300/user/u123/post/p456", "multi-param route"),
        ("/api/nonexistent/path", "no match"),
    ];

    let opts = MatchOpts {
        method: Some("GET".to_string()),
        ..Default::default()
    };

    let iterations = 100_000;
    for (path, desc) in &test_paths {
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = router.match_route(path, &opts).ok();
        }
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!("  {:30} - {:>12.0} ops/sec", desc, ops_per_sec);
    }
    println!();

    // Test 3: Concurrent stress test
    println!("Test 3: Concurrent stress test");
    let num_threads = 16;
    let queries_per_thread = 100_000;
    println!(
        "  Running {} threads, {} queries each ({} total)",
        num_threads,
        queries_per_thread,
        num_threads * queries_per_thread
    );

    let router = Arc::new(router);
    let start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let router_clone = Arc::clone(&router);

        let handle = thread::spawn(move || {
            let opts = MatchOpts {
                method: Some("GET".to_string()),
                ..Default::default()
            };

            let paths = vec![
                format!("/api/exact/{}", thread_id * 100),
                format!("/api/param/{}/test123", thread_id * 100),
                format!("/api/wildcard/{}/some/path.txt", thread_id * 100),
            ];

            for _ in 0..queries_per_thread {
                for path in &paths {
                    let _ = router_clone.match_route(path, &opts).ok();
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let total_time = start.elapsed();
    let total_queries = num_threads * queries_per_thread * 3; // 3 paths per iteration
    let qps = total_queries as f64 / total_time.as_secs_f64();

    println!("  ✓ Completed in {:.2}s", total_time.as_secs_f64());
    println!("  ✓ Throughput: {:.0} queries/sec", qps);
    println!(
        "  ✓ Average latency: {:.2} µs/query",
        total_time.as_micros() as f64 / total_queries as f64
    );
    println!();

    // Test 4: Memory usage test (route addition/deletion)
    println!("Test 4: Dynamic route management stress test");
    let mut dynamic_router = RadixRouter::new(vec![])?;

    println!("  Adding 1000 routes dynamically...");
    let start = Instant::now();
    let mut added_routes = Vec::new();

    for i in 0..1000 {
        let route = Route {
            id: format!("dynamic_{}", i),
            paths: vec![format!("/dynamic/route/{}", i)],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"id": i}),
        };

        dynamic_router.add_route(route.clone())?;
        added_routes.push(route);
    }

    let add_time = start.elapsed();
    println!(
        "  ✓ Added in {:.2}ms ({:.0} routes/sec)",
        add_time.as_millis(),
        1000.0 / add_time.as_secs_f64()
    );

    // Verify routes work
    let opts = MatchOpts {
        method: Some("GET".to_string()),
        ..Default::default()
    };
    assert!(dynamic_router
        .match_route("/dynamic/route/500", &opts)?
        .is_some());
    println!("  ✓ Routes are queryable");

    // Delete all routes
    println!("  Deleting 1000 routes...");
    let start = Instant::now();
    for route in added_routes {
        dynamic_router.delete_route(route)?;
    }
    let delete_time = start.elapsed();
    println!(
        "  ✓ Deleted in {:.2}ms ({:.0} routes/sec)",
        delete_time.as_millis(),
        1000.0 / delete_time.as_secs_f64()
    );

    // Verify routes are gone
    assert!(dynamic_router
        .match_route("/dynamic/route/500", &opts)?
        .is_none());
    println!("  ✓ Routes successfully removed");
    println!();

    // Test 5: Pathological cases
    println!("Test 5: Pathological cases");

    // Very deep nesting
    {
        let deep_path = (0..20)
            .map(|i| format!("level{}", i))
            .collect::<Vec<_>>()
            .join("/");
        let full_path = format!("/{}", deep_path);

        let routes = vec![Route {
            id: "deep".to_string(),
            paths: vec![full_path.clone()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "deep"}),
        }];

        let router = RadixRouter::new(routes)?;
        let opts = MatchOpts::default();

        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = router.match_route(&full_path, &opts).ok();
        }
        let elapsed = start.elapsed();

        println!(
            "  ✓ Deep nesting (20 levels): {:.0} ops/sec",
            10_000.0 / elapsed.as_secs_f64()
        );
    }

    // Many parameters
    {
        let param_parts: Vec<String> = (0..10).map(|i| format!(":param{}", i)).collect();
        let param_path = format!("/{}", param_parts.join("/"));
        let test_path = "/a/b/c/d/e/f/g/h/i/j";

        let routes = vec![Route {
            id: "params".to_string(),
            paths: vec![param_path],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "params"}),
        }];

        let router = RadixRouter::new(routes)?;
        let opts = MatchOpts::default();

        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = router.match_route(test_path, &opts).ok();
        }
        let elapsed = start.elapsed();

        println!(
            "  ✓ Many parameters (10): {:.0} ops/sec",
            10_000.0 / elapsed.as_secs_f64()
        );
    }

    // Very long path
    {
        let long_segment = "a".repeat(100);
        let long_path = format!("/{}/{}/{}", long_segment, long_segment, long_segment);

        let routes = vec![Route {
            id: "long".to_string(),
            paths: vec![long_path.clone()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "long"}),
        }];

        let router = RadixRouter::new(routes)?;
        let opts = MatchOpts::default();

        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = router.match_route(&long_path, &opts).ok();
        }
        let elapsed = start.elapsed();

        println!(
            "  ✓ Very long path (300 chars): {:.0} ops/sec",
            10_000.0 / elapsed.as_secs_f64()
        );
    }
    println!();

    println!("=== Stress Test Summary ===");
    println!("✅ Successfully handled {} routes", num_routes);
    println!(
        "✅ Concurrent queries: {:.0} qps with {} threads",
        qps, num_threads
    );
    println!("✅ Dynamic route management works correctly");
    println!("✅ Pathological cases handled efficiently");
    println!("\n=== All Stress Tests Passed ✓ ===");

    Ok(())
}
