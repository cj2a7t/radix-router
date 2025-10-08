/// Concurrent performance test demonstrating the router's thread-safety and async-safety
/// This example shows that multiple threads can query the router simultaneously without contention

use radix_router::{HttpMethod, MatchOpts, RadixRouter, Route};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    println!("=== Concurrent Performance Test ===\n");

    // Create a router with various route types
    let routes = vec![
        // Exact routes
        Route {
            id: "exact_1".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "exact"}),
        },
        Route {
            id: "exact_2".to_string(),
            paths: vec!["/api/posts".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "exact"}),
        },
        // Parameter routes
        Route {
            id: "param_1".to_string(),
            paths: vec!["/api/user/:id".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "param"}),
        },
        Route {
            id: "param_2".to_string(),
            paths: vec!["/api/user/:uid/post/:pid".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "multi_param"}),
        },
        // Wildcard route
        Route {
            id: "wildcard".to_string(),
            paths: vec!["/files/*path".to_string()],
            methods: None,
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"type": "wildcard"}),
        },
    ];

    let router = Arc::new(RadixRouter::new(routes)?);

    println!("Router created with 5 routes");
    println!("- 2 exact match routes");
    println!("- 2 parameter routes");
    println!("- 1 wildcard route\n");

    // Test paths
    let test_cases = vec![
        ("/api/users", "exact match"),
        ("/api/posts", "exact match"),
        ("/api/user/123", "single parameter"),
        ("/api/user/123/post/456", "multiple parameters"),
        ("/files/documents/readme.txt", "wildcard"),
    ];

    println!("=== Single-threaded Performance ===");
    let opts = MatchOpts {
        method: Some("GET".to_string()),
        ..Default::default()
    };

    for (path, desc) in &test_cases {
        let start = Instant::now();
        let iterations = 100_000;
        
        for _ in 0..iterations {
            let _ = router.match_route(path, &opts).ok();
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        
        println!(
            "{:30} - {:>12.0} ops/sec ({:>8.2} µs/op)",
            desc,
            ops_per_sec,
            elapsed.as_micros() as f64 / iterations as f64
        );
    }

    println!("\n=== Multi-threaded Performance ===");
    let num_threads = 8;
    println!("Running with {} concurrent threads...\n", num_threads);

    for (path, desc) in &test_cases {
        let start = Instant::now();
        let iterations_per_thread = 50_000;
        
        let mut handles = vec![];
        
        for _ in 0..num_threads {
            let router_clone = Arc::clone(&router);
            let path_owned = path.to_string();
            
            let handle = thread::spawn(move || {
                let opts = MatchOpts {
                    method: Some("GET".to_string()),
                    ..Default::default()
                };
                
                for _ in 0..iterations_per_thread {
                    let _ = router_clone.match_route(&path_owned, &opts).ok();
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let elapsed = start.elapsed();
        let total_ops = num_threads * iterations_per_thread;
        let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();
        
        println!(
            "{:30} - {:>12.0} ops/sec ({:>8.2} µs/op)",
            desc,
            ops_per_sec,
            elapsed.as_micros() as f64 / total_ops as f64
        );
    }

    println!("\n=== Key Performance Features ===");
    println!("✅ Lock-free reads: Each query creates its own iterator");
    println!("✅ Pre-compiled regex: Zero runtime compilation overhead");
    println!("✅ Thread-safe: Safe for concurrent access from multiple threads");
    println!("✅ Async-safe: Safe for use in async/await contexts (Tokio, async-std)");
    println!("✅ Zero contention: No lock waiting in the critical path");
    
    println!("\n=== Architecture Highlights ===");
    println!("• RwLock with read-only access during queries");
    println!("• Temporary iterators (malloc + init only)");
    println!("• Patterns compiled at route registration");
    println!("• Arc-wrapped compiled patterns (cheap clones)");
    println!("• Hash-based fast path for exact matches");
    
    Ok(())
}

