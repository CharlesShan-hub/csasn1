use super::super::*;
use super::type_registry;

fn boxed(jt: &str) -> String {
    match jt {
        "int" => "Integer".into(),
        "long" => "Long".into(),
        "boolean" => "Boolean".into(),
        "float" => "Float".into(),
        "double" => "Double".into(),
        _ => jt.to_string(),
    }
}

/// Resolve a Rust type name to a Java type string.
pub fn resolve_java_type(rt: &str, all: &[TypeInfo], prefix: &str) -> String {
    let rt = rt.trim();
    if rt.starts_with("Option <") {
        let inner = rt
            .trim_start_matches("Option <")
            .trim_end_matches('>')
            .trim()
            .to_string();
        return resolve_java_type(&inner, all, prefix);
    }

    if rt.starts_with("SequenceOf <") || rt.starts_with("Vec <") {
        let inner = rt
            .trim_start_matches("SequenceOf <")
            .trim_start_matches("Vec <")
            .trim_end_matches('>')
            .trim()
            .to_string();
        let inner_java = resolve_java_type(&inner, all, prefix);
        return format!("java.util.List<{}>", boxed(&inner_java));
    }
    if rt.starts_with("Box <") {
        let inner = rt
            .trim_start_matches("Box <")
            .trim_end_matches('>')
            .trim()
            .to_string();
        return resolve_java_type(&inner, all, prefix);
    }

    // Try JSON registry first (exact → prefix)
    if let Some(java) = type_registry::lookup_java(rt) {
        return java.to_string();
    }

    // User-defined type: if it's a Newtype, recurse into inner_type
    if let Some(ti) = all.iter().find(|t| t.name == rt) {
        if let TypeKind::Newtype { ref inner_type, .. } = ti.kind {
            return resolve_java_type(inner_type, all, prefix);
        }
    }
    format!("{}{}", prefix, rt)
}

/// Resolve a Rust type to its Java wrapper type (does NOT unwrap newtypes).
pub fn resolve_wrapper_type(rt: &str, all: &[TypeInfo], prefix: &str) -> String {
    let rt = rt.trim();
    if rt.starts_with("Option <") {
        let inner = rt
            .trim_start_matches("Option <")
            .trim_end_matches('>')
            .trim()
            .to_string();
        return resolve_wrapper_type(&inner, all, prefix);
    }
    if rt.starts_with("SequenceOf <") || rt.starts_with("Vec <") {
        let inner = rt
            .trim_start_matches("SequenceOf <")
            .trim_start_matches("Vec <")
            .trim_end_matches('>')
            .trim()
            .to_string();
        let inner_java = resolve_wrapper_type(&inner, all, prefix);
        return format!("java.util.List<{}>", inner_java);
    }
    if rt.starts_with("Box <") {
        let inner = rt
            .trim_start_matches("Box <")
            .trim_end_matches('>')
            .trim()
            .to_string();
        return resolve_wrapper_type(&inner, all, prefix);
    }

    // Try JSON registry
    if let Some(java) = type_registry::lookup_java(rt) {
        return if java != "Object" { boxed(java) } else { java.to_string() };
    }

    // User-defined type → wrapper name
    format!("{}{}", prefix, rt)
}
