//! Route definitions and data structures

use bitflags::bitflags;
use std::{collections::HashMap, sync::Arc};

bitflags! {
    /// HTTP methods represented as bit flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RadixHttpMethod: u16 {
        const GET     = 1 << 0;
        const POST    = 1 << 1;
        const PUT     = 1 << 2;
        const DELETE  = 1 << 3;
        const PATCH   = 1 << 4;
        const HEAD    = 1 << 5;
        const OPTIONS = 1 << 6;
        const CONNECT = 1 << 7;
        const TRACE   = 1 << 8;
        const PURGE   = 1 << 9;
    }
}

impl RadixHttpMethod {
    /// Parse HTTP method from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(RadixHttpMethod::GET),
            "POST" => Some(RadixHttpMethod::POST),
            "PUT" => Some(RadixHttpMethod::PUT),
            "DELETE" => Some(RadixHttpMethod::DELETE),
            "PATCH" => Some(RadixHttpMethod::PATCH),
            "HEAD" => Some(RadixHttpMethod::HEAD),
            "OPTIONS" => Some(RadixHttpMethod::OPTIONS),
            "CONNECT" => Some(RadixHttpMethod::CONNECT),
            "TRACE" => Some(RadixHttpMethod::TRACE),
            "PURGE" => Some(RadixHttpMethod::PURGE),
            _ => None,
        }
    }

    /// Parse multiple HTTP methods from slice
    pub fn from_slice(methods: &[&str]) -> Self {
        let mut result = RadixHttpMethod::empty();
        for method in methods {
            if let Some(m) = Self::from_str(method) {
                result |= m;
            }
        }
        result
    }
}

/// Host pattern for matching
#[derive(Debug, Clone)]
pub struct HostPattern {
    pub is_wildcard: bool,
    pub pattern: String,
}

impl HostPattern {
    /// Create a new host pattern
    pub fn new(pattern: &str) -> Self {
        if pattern.starts_with('*') {
            Self {
                is_wildcard: true,
                pattern: pattern[1..].to_lowercase(),
            }
        } else {
            Self {
                is_wildcard: false,
                pattern: pattern.to_lowercase(),
            }
        }
    }

    /// Check if host matches this pattern
    pub fn matches(&self, host: &str) -> bool {
        let host = host.to_lowercase();
        if self.is_wildcard {
            host.ends_with(&self.pattern)
        } else {
            host == self.pattern
        }
    }
}

/// Expression for variable matching (simplified version)
#[derive(Debug, Clone)]
pub enum Expr {
    /// Equality: var == value
    Eq(String, String),
    /// Inequality: var != value
    Neq(String, String),
    /// Greater than: var > value
    Gt(String, String),
    /// Less than: var < value
    Lt(String, String),
    /// In array: var in [values]
    In(String, Vec<String>),
    /// Regex match: var =~ pattern
    Regex(String, regex::Regex),
}

impl Expr {
    /// Evaluate expression against variables
    pub fn eval(&self, vars: &HashMap<String, String>) -> bool {
        match self {
            Expr::Eq(key, value) => vars.get(key).map(|v| v == value).unwrap_or(false),
            Expr::Neq(key, value) => vars.get(key).map(|v| v != value).unwrap_or(true),
            Expr::In(key, values) => vars.get(key).map(|v| values.contains(v)).unwrap_or(false),
            Expr::Regex(key, pattern) => {
                vars.get(key).map(|v| pattern.is_match(v)).unwrap_or(false)
            }
            Expr::Gt(key, value) => vars
                .get(key)
                .and_then(|v| {
                    let vn = v.parse::<f64>().ok()?;
                    let val = value.parse::<f64>().ok()?;
                    Some(vn > val)
                })
                .unwrap_or(false),
            Expr::Lt(key, value) => vars
                .get(key)
                .and_then(|v| {
                    let vn = v.parse::<f64>().ok()?;
                    let val = value.parse::<f64>().ok()?;
                    Some(vn < val)
                })
                .unwrap_or(false),
        }
    }
}

/// Filter function type
pub type FilterFn = Arc<dyn Fn(&HashMap<String, String>, &RadixMatchOpts) -> bool + Send + Sync>;

/// RadixNode definition - represents a route node in the radix tree
#[derive(Clone)]
pub struct RadixNode {
    /// Unique route ID
    pub id: String,
    /// Path(s) for this route
    pub paths: Vec<String>,
    /// Allowed HTTP methods (None means all)
    pub methods: Option<RadixHttpMethod>,
    /// Host patterns (None means all)
    pub hosts: Option<Vec<String>>,
    /// Remote address filters (CIDR notation)
    pub remote_addrs: Option<Vec<String>>,
    /// Variable expressions
    pub vars: Option<Vec<Expr>>,
    /// Custom filter function
    pub filter_fn: Option<FilterFn>,
    /// Route priority (higher = more important)
    pub priority: i32,
    /// Metadata associated with the route
    pub metadata: serde_json::Value,
}

/// Match options for route matching (input only)
#[derive(Debug, Clone, Default)]
pub struct RadixMatchOpts {
    /// HTTP method
    pub method: Option<String>,
    /// Host header
    pub host: Option<String>,
    /// Remote address
    pub remote_addr: Option<String>,
    /// Request variables
    pub vars: Option<HashMap<String, String>>,
}

/// Match result containing metadata and extracted parameters
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Route ID
    pub id: String,
    /// Route metadata
    pub metadata: serde_json::Value,
    /// Matched path parameters and other extracted values
    pub matched: HashMap<String, String>,
}

/// Path operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathOp {
    /// Exact match (=)
    Equal,
    /// Prefix match (<=)
    PrefixMatch,
}

/// Internal route options (processed route)
#[derive(Clone)]
pub(crate) struct RouteOpts {
    pub id: String,
    /// Actual match path (truncated at param/wildcard)
    pub path: String,
    /// Original path
    pub path_org: String,
    /// Path operation
    pub path_op: PathOp,
    /// Whether path contains parameters
    pub has_param: bool,

    pub methods: RadixHttpMethod,
    pub hosts: Option<Vec<HostPattern>>,
    pub vars: Option<Vec<Expr>>,
    pub filter_fn: Option<FilterFn>,

    pub priority: i32,
    pub metadata: serde_json::Value,

    /// Pre-compiled regex pattern for parameter extraction (if has_param=true)
    /// Using Arc to make cloning cheap
    pub compiled_pattern: Option<std::sync::Arc<(regex::Regex, Vec<String>)>>,
}

impl RouteOpts {
    /// Compare priority (for sorting)
    pub fn cmp_priority(&self, other: &Self) -> std::cmp::Ordering {
        match other.priority.cmp(&self.priority) {
            std::cmp::Ordering::Equal => {
                // Same priority, compare path length (longer first)
                other.path_org.len().cmp(&self.path_org.len())
            }
            ord => ord,
        }
    }
}

impl std::fmt::Debug for RadixNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadixNode")
            .field("id", &self.id)
            .field("paths", &self.paths)
            .field("methods", &self.methods)
            .field("hosts", &self.hosts)
            .field("priority", &self.priority)
            .finish()
    }
}

impl std::fmt::Debug for RouteOpts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteOpts")
            .field("id", &self.id)
            .field("path", &self.path)
            .field("path_org", &self.path_org)
            .field("path_op", &self.path_op)
            .field("has_param", &self.has_param)
            .field("methods", &self.methods)
            .field("priority", &self.priority)
            .finish()
    }
}
