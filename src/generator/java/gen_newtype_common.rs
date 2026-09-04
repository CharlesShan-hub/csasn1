//! Shared helpers for generating Newtype / Struct / Choice Java classes.
//!
//! Strategy: condition overrides first (hex_digits, unsigned_int),
//! then a short dispatch on Java type — no 50-line if-else chains.

// ── 1) encode_arg ────────────────────────────────────────────────────────

/// Build `(java_encode_expression, wrap_in_try_block)`.
pub fn build_encode_arg(
    jt: &str,
    base: &str,
    hex_digits: usize,
    inner_unsigned_int: bool,
    prefix: &str,
) -> (String, bool) {
    // ── Condition overrides (highest priority) ──
    if hex_digits > 0 {
        return (format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base), true);
    }
    if inner_unsigned_int {
        return ("String.valueOf(Integer.toUnsignedLong((int) _v.get(\"_\")))".into(), false);
    }
    if jt.starts_with("java.util.List<") {
        return (format!("{}.MAPPER.writeValueAsString({}.toJson(_v.get(\"_\")))", base, base), true);
    }
    if jt.starts_with(prefix) {
        return (format!("{}.MAPPER.writeValueAsString({}.toJson(_v.get(\"_\")))", base, base), true);
    }

    // ── Strategy dispatch ──
    match jt {
        "Object" | "()" => ("\"null\"".into(), false),
        "byte[]" | "DefaultInnerOctetString" => {
            (format!("{}.MAPPER.writeValueAsString({}.hex((byte[]) _v.get(\"_\")))", base, base), true)
        }
        "String" | "DefaultInnerVisibleString" | "DefaultInnerUtf8String" => {
            (format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base), true)
        }
        _ if jt.starts_with("DefaultInner") => {
            (format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base), true)
        }
        _ => ("String.valueOf(_v.get(\"_\"))".into(), false),
    }
}

// ── 2) decode puts ───────────────────────────────────────────────────────

/// Generate `_v.put("_", ...)` lines (after {"value": X} unwrap).
pub fn build_decode_puts(
    jt: &str,
    base: &str,
    hex_digits: usize,
    inner_octet_string: bool,
) -> Vec<String> {
    let line = if jt.starts_with("java.util.List<") {
        let inner = jt.trim_start_matches("java.util.List<").trim_end_matches('>').trim();
        format!(
            "r._v.put(\"_\", {}.MAPPER.convertValue(_node, new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}}));",
            base, inner
        )
    } else if hex_digits > 0 {
        "r._v.put(\"_\", _node.asText());".into()
    } else if inner_octet_string {
        format!("r._v.put(\"_\", _node.asText().isEmpty() ? new byte[0] : {}.unhex(_node.asText()));", base)
    } else {
        match jt {
            "byte[]"    => format!("r._v.put(\"_\", {}.unhex(_node.asText()));", base),
            "String"    => "r._v.put(\"_\", _node.asText());".into(),
            "int" | "Integer"   => "r._v.put(\"_\", _node.asInt());".into(),
            "long" | "Long"     => "r._v.put(\"_\", _node.asLong());".into(),
            "boolean" | "Boolean" => "r._v.put(\"_\", _node.asBoolean());".into(),
            "float" | "Float"   => "r._v.put(\"_\", (float) _node.asDouble());".into(),
            "double" | "Double" => "r._v.put(\"_\", _node.asDouble());".into(),
            "Object"    => "r._v.put(\"_\", null);".into(),
            _ => format!("r._v.put(\"_\", {}.MAPPER.readValue(_node.toString(), {}.class));", base, jt),
        }
    };
    vec![line]
}

// ── 3) default value ─────────────────────────────────────────────────────

/// Generate default-value Java expression for a no-arg constructor.
pub fn default_val_for(jt: &str, size: usize) -> String {
    if jt.starts_with("java.util.List<") {
        return "new java.util.ArrayList<>()".to_string();
    }
    match jt {
        "String" => if size > 0 { format!("\"{}\"", "x".repeat(size)) } else { "\"\"".to_string() },
        "byte[]" => if size > 0 { format!("new byte[{}]", size) } else { "new byte[0]".to_string() },
        "int" | "Integer"   => "1".to_string(),
        "long" | "Long"     => "1L".to_string(),
        "float" | "Float"   => "1.5f".to_string(),
        "double" | "Double" => "2.5".to_string(),
        "boolean" | "Boolean" => "true".to_string(),
        "Object" => "null".to_string(),
        "DefaultInnerOctetString" => {
            if size > 0 {
                let bytes: Vec<String> = std::iter::repeat("1".to_string()).take(size).collect();
                format!("new byte[] {{ {} }}", bytes.join(", "))
            } else {
                "new byte[0]".to_string()
            }
        }
        "DefaultInnerVisibleString" | "DefaultInnerUtf8String" => {
            if size > 0 { format!("\"{}\"", "x".repeat(size)) } else { format!("new {}()", jt) }
        }
        _ => format!("new {}()", jt),
    }
}
