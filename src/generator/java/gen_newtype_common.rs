//! Shared helpers for generating Newtype / Struct / Choice Java classes.
//!
//! These three long match blocks originated inside gen_newtype.rs but are useful
//! to gen_struct.rs too (same type-mapping logic), so they live here.

// ── 1) encode_arg: expression fed to Native.encode("...", encode_arg) ──────

/// Build the `encode_arg` expression and `wrap_try` flag.
///
/// Returns `(java_expression, wrap_in_try_block)`.
/// `wrap_try = true` means the expression involves Jackson MAPPER (can throw IOException),
/// so caller must wrap the encode() body in try/catch.
pub fn build_encode_arg(
    jt: &str,
    base: &str,
    hex_digits: usize,
    inner_unsigned_int: bool,
    prefix: &str,
) -> (String, bool) {
    if jt.starts_with("java.util.List<") {
        (format!("{}.MAPPER.writeValueAsString({}.toJson(_v.get(\"_\")))", base, base), true)
    } else if jt == "byte[]" {
        (format!("{}.MAPPER.writeValueAsString({}.hex((byte[]) _v.get(\"_\")))", base, base), true)
    } else if hex_digits > 0 {
        (format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base), true)
    } else if jt == "String" {
        (format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base), true)
    } else if jt == "Object" {
        ("\"null\"".into(), false)
    } else if jt.starts_with(prefix) {
        (format!("{}.MAPPER.writeValueAsString({}.toJson(_v.get(\"_\")))", base, base), true)
    } else if jt == "DefaultInnerOctetString" {
        (format!("{}.MAPPER.writeValueAsString({}.hex((byte[]) _v.get(\"_\")))", base, base), true)
    } else if jt.starts_with("DefaultInner") {
        (format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base), true)
    } else if inner_unsigned_int {
        ("String.valueOf(Integer.toUnsignedLong((int) _v.get(\"_\")))".into(), false)
    } else {
        ("String.valueOf(_v.get(\"_\"))".into(), false)
    }
}

// ── 2) decode body: _v.put("_", value) line(s) AFTER {"value":X} unwrap ────

/// Generate one or more `_v.put("_", ...)` lines for a given Java type.
///
/// All returned lines assume the caller has already:
///   1. called Native.decode() and got the JSON string
///   2. parsed it to JsonNode `_node` and unwrapped {"value": X} if present
///
/// Each returned string has NO indentation — caller applies helpers::ln(3, ...).
pub fn build_decode_puts(
    jt: &str,
    base: &str,
    hex_digits: usize,
    inner_octet_string: bool,
) -> Vec<String> {
    let line = if jt.starts_with("java.util.List<") {
        let inner = jt
            .trim_start_matches("java.util.List<")
            .trim_end_matches('>')
            .trim();
        format!(
            "r._v.put(\"_\", {}.MAPPER.convertValue(_node, new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}}));",
            base, inner
        )
    } else if jt == "byte[]" {
        format!("r._v.put(\"_\", {}.unhex(_node.asText()));", base)
    } else if jt == "String" {
        "r._v.put(\"_\", _node.asText());".into()
    } else if hex_digits > 0 {
        // BIT STRING stores hex string in _v
        "r._v.put(\"_\", _node.asText());".into()
    } else if jt == "int" || jt == "Integer" {
        "r._v.put(\"_\", _node.asInt());".into()
    } else if jt == "long" || jt == "Long" {
        "r._v.put(\"_\", _node.asLong());".into()
    } else if jt == "boolean" || jt == "Boolean" {
        "r._v.put(\"_\", _node.asBoolean());".into()
    } else if jt == "float" || jt == "Float" {
        "r._v.put(\"_\", (float) _node.asDouble());".into()
    } else if jt == "double" || jt == "Double" {
        "r._v.put(\"_\", _node.asDouble());".into()
    } else if jt == "Object" {
        "r._v.put(\"_\", null);".into()
    } else if inner_octet_string {
        format!(
            "r._v.put(\"_\", _node.asText().isEmpty() ? new byte[0] : {}.unhex(_node.asText()));",
            base
        )
    } else {
        format!(
            "r._v.put(\"_\", {}.MAPPER.readValue(_node.toString(), {}.class));",
            base, jt
        )
    };
    vec![line]
}

// ── 3) default value for a no-arg constructor ──────────────────────────────

/// Generate the default-value Java expression for a no-arg constructor.
pub fn default_val_for(jt: &str, size: usize) -> String {
    match jt {
        "String" => {
            if size > 0 { format!("\"{}\"", "x".repeat(size)) } else { "\"\"".to_string() }
        }
        "byte[]" => {
            if size > 0 { format!("new byte[{}]", size) } else { "new byte[0]".to_string() }
        }
        _ if jt.starts_with("java.util.List<") => "new java.util.ArrayList<>()".to_string(),
        "int" | "Integer" => "1".to_string(),
        "long" | "Long" => "1L".to_string(),
        "float" | "Float" => "1.5f".to_string(),
        "double" | "Double" => "2.5".to_string(),
        "boolean" | "Boolean" => "true".to_string(),
        "Object" => "null".to_string(),
        _ if jt.starts_with("DefaultInner") => {
            if size > 0 && jt == "DefaultInnerOctetString" {
                let bytes: Vec<String> = std::iter::repeat("1".to_string()).take(size).collect();
                format!("new byte[] {{ {} }}", bytes.join(", "))
            } else if size > 0 && (jt == "DefaultInnerVisibleString" || jt == "DefaultInnerUtf8String") {
                format!("\"{}\"", "x".repeat(size))
            } else {
                format!("new {}()", jt)
            }
        }
        _ => format!("new {}()", jt),
    }
}