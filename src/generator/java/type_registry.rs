//! Loads Rust-type→Java-type mappings + encode/decode/default strategies
//! from `type_map.json` (compiled via include_str!).
//!
//! Lookup: exact match first, then prefix match (longest wins).

use std::collections::HashMap;
use std::sync::OnceLock;

/// Full type spec: Rust type → Java type + encode/decode/default strategies.
#[derive(Debug)]
#[allow(dead_code)]
pub struct TypeSpec {
    pub java:    String,
    pub encode:  String,
    pub decode:  String,
    pub default: String,
    pub creator: String,
    pub ctor:    String,
}

#[derive(Debug)]
struct TypeMap {
    exact:  HashMap<String, TypeSpec>,
    /// Prefix mappings sorted by key length descending (longest match first).
    prefix: Vec<(String, TypeSpec)>,
}

impl TypeMap {
    fn from_json(json: &str) -> Self {
        let root: serde_json::Value = serde_json::from_str(json).expect("type_map.json must be valid JSON");

        let mut exact = HashMap::new();
        if let Some(obj) = root.get("exact").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if let Some(spec) = Self::parse_spec(v) {
                    exact.insert(k.clone(), spec);
                }
            }
        }

        let mut prefix_raw = Vec::new();
        if let Some(obj) = root.get("prefix").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if let Some(spec) = Self::parse_spec(v) {
                    prefix_raw.push((k.clone(), spec));
                }
            }
        }
        prefix_raw.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        TypeMap { exact, prefix: prefix_raw }
    }

    fn parse_spec(v: &serde_json::Value) -> Option<TypeSpec> {
        let obj = v.as_object()?;
        Some(TypeSpec {
            java:    obj.get("java")?.as_str()?.to_string(),
            encode:  obj.get("encode")?.as_str()?.to_string(),
            decode:  obj.get("decode")?.as_str()?.to_string(),
            default: obj.get("default")?.as_str()?.to_string(),
            creator: obj.get("creator")?.as_str()?.to_string(),
            ctor:    obj.get("ctor")?.as_str()?.to_string(),
        })
    }

    fn lookup(&self, rt: &str) -> Option<&TypeSpec> {
        if let Some(spec) = self.exact.get(rt) {
            return Some(spec);
        }
        for (k, spec) in &self.prefix {
            if rt.starts_with(k) {
                return Some(spec);
            }
        }
        None
    }
}

fn global() -> &'static TypeMap {
    static MAP: OnceLock<TypeMap> = OnceLock::new();
    MAP.get_or_init(|| {
        let json = include_str!("type_map.json");
        TypeMap::from_json(json)
    })
}

/// Look up a Rust type name → full TypeSpec.
pub fn lookup(rt: &str) -> Option<&'static TypeSpec> {
    global().lookup(rt)
}

/// Convenience: just the Java type string (backward compatible).
pub fn lookup_java(rt: &str) -> Option<&'static str> {
    lookup(rt).map(|s| s.java.as_str())
}
