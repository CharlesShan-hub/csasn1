use super::super::*;

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

    let base = match rt {
        "bool" => "boolean",
        "u8" | "i8" | "u16" | "i16" | "u32" | "i32" => "int",
        "u64" | "i64" => "long",
        "f32" => "float",
        "f64" => "double",
        s if s == "String" || s.starts_with("Utf8String") || s.starts_with("VisibleString") => "String",
        s if s.starts_with("OctetString") || s.starts_with("FixedOctetString") => "byte[]",
        s if s.starts_with("Integer") => "int",
        s if s.starts_with("FixedBitString") => "int",
        s if s.starts_with("BitString") => "byte[]",
        "()" => "Object",
        s => {
            if let Some(ti) = all.iter().find(|t| t.name == s) {
                if let TypeKind::Newtype { ref inner_type } = ti.kind {
                    return resolve_java_type(inner_type, all, prefix);
                }
            }
            return format!("{}{}", prefix, s);
        }
    };
    base.to_string()
}

/// Resolve type to a nullable Java type (boxed primitives for optional fields).
pub fn resolve_java_type_nullable(rt: &str, all: &[TypeInfo], prefix: &str) -> String {
    let t = resolve_java_type(rt, all, prefix);
    let base = match t.as_str() {
        "int" => "Integer",
        "long" => "Long",
        "boolean" => "Boolean",
        "float" => "Float",
        "double" => "Double",
        other => return other.to_string(),
    };
    base.to_string()
}

/// Generate a type literal for Jackson convertValue.
/// Uses TypeReference for List<T>, .class for everything else.
pub fn java_type_ref(jt: &str) -> String {
    if jt.starts_with("java.util.List<") {
        let inner = jt
            .trim_start_matches("java.util.List<")
            .trim_end_matches('>')
            .trim();
        format!(
            "new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}}",
            inner
        )
    } else {
        match jt {
            "int" => "int.class",
            "long" => "long.class",
            "boolean" => "boolean.class",
            "float" => "float.class",
            "double" => "double.class",
            "byte[]" => "byte[].class",
            "String" => "String.class",
            _ => return format!("{}.class", jt),
        }
        .to_string()
    }
}
