# router_radix

<div align="center">

**A high-performance, thread-safe radix tree based HTTP router for Rust**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

*Based on Redis's radix tree implementation*

</div>

---

## 📖 Table of Contents

- [About](#-about)
- [Features](#-features)
- [Quick Start](#-quick-start)
- [Usage Guide](#-usage-guide)
  - [Basic Routing](#basic-routing)
  - [Path Parameters](#path-parameters)
  - [Wildcards](#wildcards)
  - [HTTP Methods](#http-methods)
  - [Host Matching](#host-matching)
  - [Priority Routing](#priority-routing)
  - [Advanced Features](#advanced-features)
- [Error Handling](#-error-handling)
- [Concurrency & Thread Safety](#-concurrency--thread-safety)
- [Performance](#-performance)
- [Examples & Tests](#-examples--tests)
- [Architecture](#-architecture)
- [License](#-license)

---

## 🎯 About

`router_radix` is a Rust port of [lua-resty-radixtree](https://github.com/api7/lua-resty-radixtree), providing fast and flexible HTTP routing. The underlying radix tree ([rax](https://github.com/antirez/rax)) is the same battle-tested data structure used in **Redis** for Redis Streams and internal routing.

**Why router_radix?**
- ⚡ High performance with lock-free queries
- 🔒 Thread-safe with zero contention
- 🎯 Rich matching capabilities
- 🦺 Type-safe with proper error handling
- 🚀 Production-ready

---

## ✨ Features

- **Path Matching**: Exact paths, parameters (`:id`), wildcards (`*path`)
- **HTTP Methods**: Match specific methods (GET, POST, etc.)
- **Host Matching**: Match hosts with wildcard support (`*.example.com`)
- **Priority Routing**: Higher priority routes match first
- **Custom Filters**: Add custom logic with filter functions
- **Variable Expressions**: Match based on request variables with regex support
- **Thread-Safe**: Lock-free queries, safe for concurrent access
- **Async Compatible**: Works with Tokio, async-std, etc.
- **Type-Safe**: Full Rust type safety with `anyhow` error handling

---

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# router_radix = "0.1.0"
# anyhow = "1.0"
# serde_json = "1.0"
```

### Hello Router

```rust
use router_radix::{RadixRouter, Route, HttpMethod, MatchOpts};

fn main() -> anyhow::Result<()> {
    // Create routes
    let routes = vec![
        Route {
            id: "get_users".to_string(),
            paths: vec!["/api/users".to_string()],
            methods: Some(HttpMethod::GET),
            hosts: None,
            remote_addrs: None,
            vars: None,
            filter_fn: None,
            priority: 0,
            metadata: serde_json::json!({"handler": "list_users"}),
        },
    ];

    // Initialize router
    let router = RadixRouter::new(routes)?;

    // Match a request
    let opts = MatchOpts {
        method: Some("GET".to_string()),
        ..Default::default()
    };

    if let Some(result) = router.match_route("/api/users", &opts)? {
        println!("✓ Matched! Handler: {}", result.metadata["handler"]);
        println!("  Params: {:?}", result.matched);
    }

    Ok(())
}
```

---

## 📚 Usage Guide

### Basic Routing

Match exact paths:

```rust
let routes = vec![
    Route {
        id: "home".to_string(),
        paths: vec!["/".to_string()],
        methods: None,
        hosts: None,
        remote_addrs: None,
        vars: None,
        filter_fn: None,
        priority: 0,
        metadata: serde_json::json!({"page": "home"}),
    },
];

let router = RadixRouter::new(routes)?;
let result = router.match_route("/", &MatchOpts::default())?;
```

### Path Parameters

Extract dynamic segments from paths:

```rust
let routes = vec![
    Route {
        id: "user_detail".to_string(),
        paths: vec!["/user/:id/post/:pid".to_string()],
        // ... other fields
        metadata: serde_json::json!({"handler": "get_user_post"}),
    },
];

let router = RadixRouter::new(routes)?;
let result = router.match_route("/user/123/post/456", &MatchOpts::default())?
    .expect("should match");

assert_eq!(result.matched.get("id").unwrap(), "123");
assert_eq!(result.matched.get("pid").unwrap(), "456");
```

### Wildcards

Match remaining path segments:

```rust
let routes = vec![
    Route {
        id: "static_files".to_string(),
        paths: vec!["/files/*path".to_string()],
        // ... other fields
        metadata: serde_json::json!({"handler": "serve_file"}),
    },
];

let router = RadixRouter::new(routes)?;
let result = router.match_route("/files/css/main.css", &MatchOpts::default())?
    .expect("should match");

assert_eq!(result.matched.get("path").unwrap(), "css/main.css");
```

### HTTP Methods

Match specific HTTP methods:

```rust
let routes = vec![
    Route {
        id: "users_api".to_string(),
        paths: vec!["/api/users".to_string()],
        methods: Some(HttpMethod::GET | HttpMethod::POST), // Multiple methods
        // ... other fields
        metadata: serde_json::json!({"handler": "users"}),
    },
];

let router = RadixRouter::new(routes)?;

// GET - matches
let opts = MatchOpts {
    method: Some("GET".to_string()),
    ..Default::default()
};
assert!(router.match_route("/api/users", &opts)?.is_some());

// DELETE - doesn't match
let opts = MatchOpts {
    method: Some("DELETE".to_string()),
    ..Default::default()
};
assert!(router.match_route("/api/users", &opts)?.is_none());
```

### Host Matching

Route based on hostname with wildcard support:

```rust
let routes = vec![
    Route {
        id: "api_subdomain".to_string(),
        paths: vec!["/api".to_string()],
        methods: None,
        hosts: Some(vec!["*.example.com".to_string()]), // Wildcard
        // ... other fields
        metadata: serde_json::json!({"handler": "api"}),
    },
];

let router = RadixRouter::new(routes)?;

let opts = MatchOpts {
    host: Some("api.example.com".to_string()),
    ..Default::default()
};
assert!(router.match_route("/api", &opts)?.is_some());
```

### Priority Routing

Higher priority routes are matched first:

```rust
let routes = vec![
    Route {
        id: "catch_all".to_string(),
        paths: vec!["/api/*".to_string()],
        priority: 0, // Lower priority
        metadata: serde_json::json!({"handler": "fallback"}),
        // ... other fields
    },
    Route {
        id: "specific".to_string(),
        paths: vec!["/api/users".to_string()],
        priority: 10, // Higher priority - matches first
        metadata: serde_json::json!({"handler": "users"}),
        // ... other fields
    },
];

let router = RadixRouter::new(routes)?;
let result = router.match_route("/api/users", &MatchOpts::default())?
    .expect("should match");

assert_eq!(result.metadata["handler"], "users"); // Higher priority wins
```

### Advanced Features

#### Custom Filter Functions

Add custom matching logic:

```rust
use std::sync::Arc;
use std::collections::HashMap;

let routes = vec![
    Route {
        id: "v2_api".to_string(),
        paths: vec!["/api/data".to_string()],
        filter_fn: Some(Arc::new(|vars, _opts| {
            // Custom logic: check API version
            vars.get("version").map(|v| v == "v2").unwrap_or(false)
        })),
        // ... other fields
        metadata: serde_json::json!({"handler": "api_v2"}),
    },
];

let router = RadixRouter::new(routes)?;

// With version variable - matches
let mut vars = HashMap::new();
vars.insert("version".to_string(), "v2".to_string());
let opts = MatchOpts {
    vars: Some(vars),
    ..Default::default()
};
assert!(router.match_route("/api/data", &opts)?.is_some());

// Without version - doesn't match
assert!(router.match_route("/api/data", &MatchOpts::default())?.is_none());
```

#### Variable Expressions

Match based on request variables:

```rust
use router_radix::Expr;
use regex::Regex;

let routes = vec![
    Route {
        id: "prod_api".to_string(),
        paths: vec!["/api/users".to_string()],
        vars: Some(vec![
            Expr::Eq("env".to_string(), "production".to_string()),
            Expr::Regex("user_agent".to_string(), Regex::new("Chrome")?),
        ]),
        // ... other fields
        metadata: serde_json::json!({"handler": "prod_users"}),
    },
];

let router = RadixRouter::new(routes)?;

let mut vars = HashMap::new();
vars.insert("env".to_string(), "production".to_string());
vars.insert("user_agent".to_string(), "Chrome/120.0".to_string());

let opts = MatchOpts {
    vars: Some(vars),
    ..Default::default()
};
assert!(router.match_route("/api/users", &opts)?.is_some());
```

---

## 🛡️ Error Handling

The router uses `anyhow::Result` for proper error handling:

```rust
use router_radix::{RadixRouter, MatchOpts};
use anyhow::Context;

fn handle_request(router: &RadixRouter, path: &str) -> anyhow::Result<String> {
    let opts = MatchOpts::default();
    
    match router.match_route(path, &opts)? {
        Some(result) => {
            Ok(format!("Handler: {}", result.metadata["handler"]))
        }
        None => {
            Ok("404 Not Found".to_string())
        }
    }
    // System errors (e.g., lock errors) propagate via ?
}
```

**Return Value Semantics:**
- `Ok(Some(MatchResult))` → Route found and matched
- `Ok(None)` → No matching route (normal case, not an error)
- `Err(anyhow::Error)` → System error (e.g., internal lock failure)

---

## 🔒 Concurrency & Thread Safety

The router is **fully thread-safe** and optimized for concurrent access:

### Architecture

- **Lock-Free Queries**: Each query creates its own iterator
- **Immutable Routes**: Route data is immutable after initialization
- **Pre-compiled Patterns**: Regex compiled once at startup
- **Zero Contention**: Multiple threads query without blocking

### Usage with Multiple Threads

```rust
use std::sync::Arc;
use std::thread;

fn main() -> anyhow::Result<()> {
    let routes = vec![/* your routes */];
    let router = Arc::new(RadixRouter::new(routes)?);

    // Share across threads
    let mut handles = vec![];
    for i in 0..8 {
        let router = Arc::clone(&router);
        handles.push(thread::spawn(move || {
            let opts = MatchOpts {
                method: Some("GET".to_string()),
                ..Default::default()
            };
            // Lock-free concurrent access
            router.match_route("/api/users", &opts)
        }));
    }

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}
```

### Dynamic Routes (Optional)

For dynamic route updates, wrap in an additional `RwLock`:

```rust
use std::sync::{Arc, RwLock};

let router = Arc::new(RwLock::new(RadixRouter::new(vec![])?));

// Write (exclusive)
router.write().unwrap().add_route(new_route)?;

// Read (shared, many concurrent readers)
router.read().unwrap().match_route("/path", &opts)?;
```

⚠️ **Best Practice**: Initialize routes at startup for best performance.

---

## ⚡ Performance

### Benchmark Results (Release Mode)

**Single Thread:**
- Exact match: **15M+ ops/sec** (hash-based, O(1))
- Parameter extraction: **5M+ ops/sec**
- Wildcard matching: **4M+ ops/sec**

**Multi-threaded (8 threads):**
- Near-linear scaling
- **Zero contention** on query path
- Suitable for high-concurrency servers

### Run Benchmarks

```bash
# Performance benchmark
cargo run --example benchmark --release

# Concurrency test
cargo run --example concurrency_test --release

# Stress test (10,000 routes, 16 threads)
cargo run --example stress_test --release
```

---

## 🧪 Examples & Tests

### Built-in Examples

The project includes comprehensive examples:

| Example | Description | Lines |
|---------|-------------|-------|
| `basic.rs` | Basic usage and core features | 235 |
| `edge_cases.rs` | Boundary conditions and edge cases | 460 |
| `integration.rs` | Real-world API gateway scenarios | 630 |
| `vars_filter_test.rs` | Advanced filters and expressions | 506 |
| `benchmark.rs` | Performance benchmarks | 413 |
| `concurrency_test.rs` | Multi-threaded performance | 174 |
| `stress_test.rs` | Large-scale stress testing | 319 |

### Run Examples

```bash
# Basic examples
cargo run --example basic
cargo run --example edge_cases
cargo run --example integration
cargo run --example vars_filter_test

# Performance tests (use --release)
cargo run --example benchmark --release
cargo run --example concurrency_test --release
cargo run --example stress_test --release

# Run all tests
./run_all_tests.sh --release
```

### Run Unit Tests

```bash
cargo test
```

📖 **For detailed documentation**, see [`examples/README.md`](examples/README.md)

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│             Rust API Layer                      │
│  • Route matching & parameter extraction        │
│  • Filter evaluation & priority sorting         │
│  • Error handling (anyhow)                      │
│  • Thread-safe querying (RwLock + iterators)    │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│             Rust FFI Layer                      │
│  • Safe wrappers around C functions             │
│  • RAII for resource management                 │
│  • Memory safety guarantees                     │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         C Layer (Redis rax)                     │
│  • Radix tree implementation                    │
│  • Battle-tested in Redis production            │
└─────────────────────────────────────────────────┘
```

### Key Components

- **`RadixRouter`**: Main router struct with thread-safe API
- **`Route`**: Route definition with matching rules
- **`MatchOpts`**: Request matching options
- **`MatchResult`**: Matched route with extracted parameters
- **`Expr`**: Variable expression for conditional matching
- **`FilterFn`**: Custom filter function type

---

## 📄 License

Apache-2.0

---

## 🙏 Credits

- Based on [lua-resty-radixtree](https://github.com/api7/lua-resty-radixtree) by Apache APISIX
- Radix tree implementation from [Redis](https://github.com/redis/redis) by Salvatore Sanfilippo
- Inspired by high-performance routing needs in API gateways

---

<div align="center">

**Built with ❤️ for high-performance routing**

[Report Bug](https://github.com/cj2a7t/router_radix/issues) · [Request Feature](https://github.com/cj2a7t/router_radix/issues)

</div>
