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
        "bool" => "boolean".to_string(),
        "u8" | "i8" | "u16" | "i16" | "u32" | "i32" => "int".to_string(),
        "u64" | "i64" => "long".to_string(),
        "f32" => "float".to_string(),
        "f64" => "double".to_string(),
        s if s == "String" => "String".to_string(),
        s if s.starts_with("VisibleString") => "DefaultInnerVisibleString".to_string(),
        s if s.starts_with("Utf8String") => "DefaultInnerUtf8String".to_string(),
        s if s.starts_with("OctetString") || s.starts_with("FixedOctetString") => {
            "DefaultInnerOctetString".to_string()
        }
        s if s.starts_with("Integer") => "int".to_string(),
        s if s.starts_with("FixedBitString") => "int".to_string(),
        s if s.starts_with("BitString") => "byte[]".to_string(),
        "()" => "Object".to_string(),
        s => {
            if let Some(ti) = all.iter().find(|t| t.name == s) {
                if let TypeKind::Newtype { ref inner_type, .. } = ti.kind {
                    return resolve_java_type(inner_type, all, prefix);
                }
            }
            return format!("{}{}", prefix, s);
        }
    };
    base.to_string()
}

/// Resolve a Rust type to its Java wrapper type (does NOT unwrap newtypes).
/// e.g. "Boolean" → "CmsBoolean" instead of "int"
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

    let base = match rt {
        "bool" => boxed("boolean"),
        "u8" | "i8" | "u16" | "i16" | "u32" | "i32" => boxed("int"),
        "u64" | "i64" => boxed("long"),
        "f32" => boxed("float"),
        "f64" => boxed("double"),
        s if s == "String" => "String".to_string(),
        s if s.starts_with("VisibleString") => "DefaultInnerVisibleString".to_string(),
        s if s.starts_with("Utf8String") => "DefaultInnerUtf8String".to_string(),
        s if s.starts_with("OctetString") || s.starts_with("FixedOctetString") => {
            "DefaultInnerOctetString".to_string()
        }
        s if s.starts_with("Integer") => boxed("int"),
        s if s.starts_with("FixedBitString") => boxed("int"),
        s if s.starts_with("BitString") => "byte[]".to_string(),
        "()" => "Object".to_string(),
        // For any user-defined type (including newtypes), return the wrapper name.
        // Unlike resolve_java_type, we do NOT recurse into newtypes here.
        s => return format!("{}{}", prefix, s),
    };
    base.to_string()
}

/// Generate a type literal for Jackson convertValue.
/// Uses TypeReference for List<T>, .class for everything else.
#[allow(dead_code)]
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
            "int" | "Integer" => "Integer.class",
            "long" | "Long" => "Long.class",
            "boolean" | "Boolean" => "Boolean.class",
            "float" | "Float" => "Float.class",
            "double" | "Double" => "Double.class",
            "byte[]" => "byte[].class",
            "String" => "String.class",
            _ => return format!("{}.class", jt),
        }
        .to_string()
    }
}
