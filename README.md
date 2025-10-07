# radix-router

A high-performance radix tree based HTTP router for Rust.

This is a Rust port of [lua-resty-radixtree](https://github.com/api7/lua-resty-radixtree), providing fast routing with rich matching capabilities.

The underlying radix tree implementation ([rax](https://github.com/antirez/rax)) is the same data structure used in **Redis** for Redis Streams and other internal components.

## Features

- ⚡ **High Performance**: Based on C radix tree implementation
- 🎯 **Rich Matching**: Support for exact paths, parameters, wildcards
- 🔍 **HTTP Method Matching**: Match specific HTTP methods
- 🌐 **Host Matching**: Match specific hosts with wildcard support
- 📊 **Priority Routing**: Higher priority routes match first
- 🔧 **Custom Filters**: Add custom filter functions
- 📝 **Variable Expressions**: Match based on request variables
- 🦺 **Type Safe**: Full Rust type safety

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
radix-router = "0.1"
```

## Usage

### Basic Example

```rust
use radix_router::{RadixRouter, Route, HttpMethod, MatchOpts};
use std::collections::HashMap;

// Define routes
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
        metadata: serde_json::json!({"handler": "get_users"}),
    },
];

// Create router
let mut router = RadixRouter::new(routes, None).unwrap();

// Match a request
let mut opts = MatchOpts {
    method: Some("GET".to_string()),
    matched: Some(HashMap::new()),
    ..Default::default()
};

if let Some(metadata) = router.match_route("/api/users", &mut opts) {
    println!("Matched! Metadata: {}", metadata);
}
```

### Path Parameters

```rust
let routes = vec![
    Route {
        id: "1".to_string(),
        paths: vec!["/user/:id/post/:pid".to_string()],
        methods: None,
        hosts: None,
        remote_addrs: None,
        vars: None,
        filter_fn: None,
        priority: 0,
        metadata: serde_json::json!({"handler": "user_post"}),
    },
];

let mut router = RadixRouter::new(routes, None).unwrap();

let mut opts = MatchOpts {
    matched: Some(HashMap::new()),
    ..Default::default()
};

router.match_route("/user/123/post/456", &mut opts);

// Extract parameters
let matched = opts.matched.unwrap();
assert_eq!(matched.get("id").unwrap(), "123");
assert_eq!(matched.get("pid").unwrap(), "456");
```

### Wildcards

```rust
let routes = vec![
    Route {
        id: "1".to_string(),
        paths: vec!["/files/*path".to_string()],
        methods: None,
        hosts: None,
        remote_addrs: None,
        vars: None,
        filter_fn: None,
        priority: 0,
        metadata: serde_json::json!({"handler": "serve_file"}),
    },
];

let mut router = RadixRouter::new(routes, None).unwrap();

let mut opts = MatchOpts {
    matched: Some(HashMap::new()),
    ..Default::default()
};

router.match_route("/files/documents/readme.txt", &mut opts);

let matched = opts.matched.unwrap();
assert_eq!(matched.get("path").unwrap(), "documents/readme.txt");
```

### HTTP Method Matching

```rust
let routes = vec![
    Route {
        id: "1".to_string(),
        paths: vec!["/api/users".to_string()],
        methods: Some(HttpMethod::GET | HttpMethod::POST),
        hosts: None,
        remote_addrs: None,
        vars: None,
        filter_fn: None,
        priority: 0,
        metadata: serde_json::json!({"handler": "users"}),
    },
];

let mut router = RadixRouter::new(routes, None).unwrap();

// GET request - matches
let mut opts = MatchOpts {
    method: Some("GET".to_string()),
    ..Default::default()
};
assert!(router.match_route("/api/users", &mut opts).is_some());

// DELETE request - does not match
opts.method = Some("DELETE".to_string());
assert!(router.match_route("/api/users", &mut opts).is_none());
```

### Host Matching

```rust
let routes = vec![
    Route {
        id: "1".to_string(),
        paths: vec!["/api".to_string()],
        methods: None,
        hosts: Some(vec!["*.example.com".to_string()]),
        remote_addrs: None,
        vars: None,
        filter_fn: None,
        priority: 0,
        metadata: serde_json::json!({"handler": "api"}),
    },
];

let mut router = RadixRouter::new(routes, None).unwrap();

let mut opts = MatchOpts {
    host: Some("api.example.com".to_string()),
    ..Default::default()
};

// Matches: api.example.com
assert!(router.match_route("/api", &mut opts).is_some());

// Does not match: api.other.com
opts.host = Some("api.other.com".to_string());
assert!(router.match_route("/api", &mut opts).is_none());
```

### Priority Routing

```rust
let routes = vec![
    Route {
        id: "1".to_string(),
        paths: vec!["/api/*".to_string()],
        methods: None,
        hosts: None,
        remote_addrs: None,
        vars: None,
        filter_fn: None,
        priority: 0,  // Low priority
        metadata: serde_json::json!({"handler": "api_fallback"}),
    },
    Route {
        id: "2".to_string(),
        paths: vec!["/api/users".to_string()],
        methods: None,
        hosts: None,
        remote_addrs: None,
        vars: None,
        filter_fn: None,
        priority: 10,  // High priority
        metadata: serde_json::json!({"handler": "api_users"}),
    },
];

let mut router = RadixRouter::new(routes, None).unwrap();

// Higher priority route matches first
let result = router.match_route("/api/users", &mut MatchOpts::default());
assert_eq!(result.unwrap()["handler"], "api_users");
```

### Custom Filter Functions

```rust
use std::collections::HashMap;

let routes = vec![
    Route {
        id: "1".to_string(),
        paths: vec!["/api/users".to_string()],
        methods: None,
        hosts: None,
        remote_addrs: None,
        vars: None,
        filter_fn: Some(Box::new(|vars, _opts| {
            vars.get("version").map(|v| v == "v2").unwrap_or(false)
        })),
        priority: 0,
        metadata: serde_json::json!({"handler": "users_v2"}),
    },
];

let mut router = RadixRouter::new(routes, None).unwrap();

// Without version - does not match
let mut opts = MatchOpts::default();
assert!(router.match_route("/api/users", &mut opts).is_none());

// With correct version - matches
let mut vars = HashMap::new();
vars.insert("version".to_string(), "v2".to_string());
opts.vars = Some(vars);
assert!(router.match_route("/api/users", &mut opts).is_some());
```

### Variable Expressions

```rust
use radix_router::Expr;
use regex::Regex;

let routes = vec![
    Route {
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
    },
];

let mut router = RadixRouter::new(routes, None).unwrap();

let mut vars = HashMap::new();
vars.insert("env".to_string(), "production".to_string());
vars.insert("user_agent".to_string(), "Chrome/90.0".to_string());

let mut opts = MatchOpts {
    vars: Some(vars),
    ..Default::default()
};

assert!(router.match_route("/api/users", &mut opts).is_some());
```

## Running Examples

```bash
cd radix-router
cargo run --example basic
```

## Running Tests

```bash
cargo test
```

## Thread Safety

⚠️ **Important**: `RadixRouter` is **NOT thread-safe** by default.

### Why Not Thread-Safe?

The router requires mutable references (`&mut self`) for route matching because:

1. The internal C-based radix tree uses a stateful iterator
2. The LRU pattern cache modifies its state on access
3. Concurrent access would cause data races

### Concurrent Usage Patterns

#### Option 1: Mutex Protection (Simple but Slower)

```rust
use std::sync::{Arc, Mutex};

let router = Arc::new(Mutex::new(RadixRouter::new(routes, None).unwrap()));

// In each thread
let router = router.clone();
std::thread::spawn(move || {
    let mut locked = router.lock().unwrap();
    let result = locked.match_route("/api/users", &mut opts);
});
```

**Pros**: Simple to implement  
**Cons**: All requests are serialized, limiting concurrency

#### Option 2: Thread-Local Storage (Recommended)

```rust
use std::cell::RefCell;

thread_local! {
    static ROUTER: RefCell<RadixRouter> = RefCell::new(
        RadixRouter::new(create_routes(), None).unwrap()
    );
}

// In each worker thread
ROUTER.with(|router| {
    let mut r = router.borrow_mut();
    r.match_route("/api/users", &mut opts)
});
```

**Pros**: No locking overhead, excellent performance  
**Cons**: Each thread has its own router instance (uses more memory)

#### Option 3: Clone Per-Thread (Best for Static Routes)

```rust
// Clone the router for each thread/worker
let routes = create_routes();
let workers: Vec<_> = (0..num_cpus::get())
    .map(|_| {
        let router = RadixRouter::new(routes.clone(), None).unwrap();
        std::thread::spawn(move || {
            // Use router in this thread
        })
    })
    .collect();
```

**Pros**: Simple, no shared state  
**Cons**: Memory overhead, routes can't be updated dynamically

### Recommendation

- **For web servers with fixed routes**: Use **Option 2** (thread-local) or **Option 3** (per-thread clone)
- **For dynamic route updates**: Use **Option 1** (Mutex) and accept the performance trade-off
- **For single-threaded apps**: Use the router directly without any wrapper

## Performance

The router uses a C-based radix tree for optimal performance:

- Exact path matching: ~15M+ QPS
- Parameter matching: ~5M+ QPS
- Wildcard matching: ~4M+ QPS

## Architecture

```
┌─────────────────────────────────────────┐
│         Rust API Layer                  │
│  - Route matching                       │
│  - Parameter extraction                 │
│  - Filter evaluation                    │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Rust FFI Layer                  │
│  - Safe wrappers around C functions     │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         C Layer (from Redis)            │
│  - Radix tree (rax.c)                   │
│  - Same implementation used in Redis    │
│    Streams, Cluster, and other modules  │
└─────────────────────────────────────────┘
```

## License

Apache-2.0

## Credits

Based on [lua-resty-radixtree](https://github.com/api7/lua-resty-radixtree) by APISIX.


