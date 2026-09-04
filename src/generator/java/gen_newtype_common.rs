//! Shared helpers for generating Newtype / Struct / Choice Java classes.
//!
//! Strategy: condition overrides first (hex_digits, unsigned_int),
//! then JSON-driven strategy dispatch via TypeSpec.

use super::type_registry::TypeSpec;

// ── 1) encode_arg ────────────────────────────────────────────────────────

/// Build `(java_encode_expression, wrap_in_try_block)`.
pub fn build_encode_arg(
    jt: &str,
    base: &str,
    hex_digits: usize,
    inner_unsigned_int: bool,
    prefix: &str,
    spec: Option<&TypeSpec>,
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

    // ── Strategy dispatch: JSON spec or fallback to jt-based default ──
    let strategy = spec.map(|s| s.encode.as_str()).unwrap_or("");
    match strategy {
        "null_lit"   => ("\"null\"".into(), false),
        "mapper_hex" => (format!("{}.MAPPER.writeValueAsString({}.hex((byte[]) _v.get(\"_\")))", base, base), true),
        "mapper" | "mapper_to_json" | "hex_str" => {
            (format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base), true)
        }
        "unsigned" => ("String.valueOf(Integer.toUnsignedLong((int) _v.get(\"_\")))".into(), false),
        "plain" | _ => ("String.valueOf(_v.get(\"_\"))".into(), false),
    }
}

// ── 2) decode puts ───────────────────────────────────────────────────────

/// Generate `_v.put("_", ...)` lines (after {"value": X} unwrap).
pub fn build_decode_puts(
    jt: &str,
    base: &str,
    hex_digits: usize,
    inner_octet_string: bool,
    spec: Option<&TypeSpec>,
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
        // ── Strategy dispatch ──
        let strategy = spec.map(|s| s.decode.as_str()).unwrap_or("");
        match strategy {
            "as_text"   => "r._v.put(\"_\", _node.asText());".into(),
            "as_int"    => "r._v.put(\"_\", _node.asInt());".into(),
            "as_long"   => "r._v.put(\"_\", _node.asLong());".into(),
            "as_boolean" => "r._v.put(\"_\", _node.asBoolean());".into(),
            "as_float"  => "r._v.put(\"_\", (float) _node.asDouble());".into(),
            "as_double" => "r._v.put(\"_\", _node.asDouble());".into(),
            "null"      => "r._v.put(\"_\", null);".into(),
            "unhex"     => format!("r._v.put(\"_\", {}.unhex(_node.asText()));", base),
            _ => {
                // Fallback: jt-based dispatch for types not in JSON (e.g. "byte[]")
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
            }
        }
    };
    vec![line]
}

// ── 3) method templates (fixed Java skeletons from .java.txt files) ─────

/// Render the Newtype constructor block for BIT STRING types (fixed hex defaults).
pub fn render_ctor_bitstring(cn: &str, jt: &str, base: &str, hex_digits: usize, bit_count: usize) -> String {
    fill(
        include_str!("templates/ctor_bitstring.java.txt"),
        &[
            ("__CLASS__", cn.to_string()),
            ("__JT__", jt.to_string()),
            ("__BASE__", base.to_string()),
            ("__HEX__", "0".repeat(hex_digits)),
            ("__BITS__", bit_count.to_string()),
        ],
    )
}

/// Render the Newtype constructor block for unsigned-int types (u32 stored in int).
pub fn render_ctor_unsigned(cn: &str) -> String {
    fill(
        include_str!("templates/ctor_unsigned.java.txt"),
        &[("__CLASS__", cn.to_string())],
    )
}

/// Fill a template: normalize trailing newlines, replace placeholders, end with one \n.
fn fill(template: &'static str, pairs: &[(&str, String)]) -> String {
    let mut s = template.trim_end_matches('\n').to_string();
    for (k, v) in pairs {
        s = s.replace(k, v);
    }
    s.push('\n');
    s
}

/// Render the Newtype encode() method (try/catch variant — MAPPER calls can throw).
pub fn render_encode_wrapped(native: &str, type_name: &str, encode_arg: &str) -> String {
    fill(
        include_str!("templates/encode_wrapped.java.txt"),
        &[
            ("__NATIVE__", native.to_string()),
            ("__TYPE_NAME__", type_name.to_string()),
            ("__ENCODE_ARG__", encode_arg.to_string()),
        ],
    )
}

/// Render the Newtype encode() method (plain variant — no exception wrapping).
pub fn render_encode_plain(native: &str, type_name: &str, encode_arg: &str) -> String {
    fill(
        include_str!("templates/encode_plain.java.txt"),
        &[
            ("__NATIVE__", native.to_string()),
            ("__TYPE_NAME__", type_name.to_string()),
            ("__ENCODE_ARG__", encode_arg.to_string()),
        ],
    )
}

/// Render the full Newtype static decode() method.
/// `put_lines` must already be indented (helpers::ln(3, ...)).
pub fn render_decode(cn: &str, native: &str, type_name: &str, base: &str, put_lines: &[String]) -> String {
    fill(
        include_str!("templates/decode.java.txt"),
        &[
            ("__CLASS__", cn.to_string()),
            ("__NATIVE__", native.to_string()),
            ("__TYPE_NAME__", type_name.to_string()),
            ("__BASE__", base.to_string()),
            // Consume the placeholder's own newline — put lines carry their own \n
            ("__DECODE_PUTS__\n", put_lines.join("")),
        ],
    )
}

/// Render the static sample() test-filler factory.
pub fn render_sample_factory(cn: &str, sample_expr: &str) -> String {
    fill(
        include_str!("templates/sample_factory.java.txt"),
        &[
            ("__CLASS__", cn.to_string()),
            ("__SAMPLE__", sample_expr.to_string()),
        ],
    )
}

/// Render the struct-style sample() factory (multi-field puts).
/// `put_lines` must already be indented (helpers::ln(2, ...)).
pub fn render_sample_factory_puts(cn: &str, put_lines: &[String]) -> String {
    fill(
        include_str!("templates/sample_factory_puts.java.txt"),
        &[
            ("__CLASS__", cn.to_string()),
            // Consume the placeholder's own newline — put lines carry their own \n
            ("__SAMPLE_PUTS__\n", put_lines.join("")),
        ],
    )
}

/// Render the choice-style sample() factory (first variant + value).
pub fn render_sample_factory_choice(cn: &str, variant: &str, sample_expr: &str) -> String {
    fill(
        include_str!("templates/sample_factory_choice.java.txt"),
        &[
            ("__CLASS__", cn.to_string()),
            ("__VARIANT__", variant.to_string()),
            ("__SAMPLE__", sample_expr.to_string()),
        ],
    )
}

// ── 4) default / sample values ───────────────────────────────────────────

/// Java class names of user types that participate in a reference cycle
/// (directly or transitively). `sample()` must NOT recurse into these —
/// it stops at `new X()._v` to avoid infinite expansion; non-cyclic user
/// fields use `X.sample()._v` so nested SIZE constraints stay satisfied.
pub fn recursive_type_names(all: &[super::super::TypeInfo], prefix: &str) -> std::collections::HashSet<String> {
    // Edges: java class name → referenced user-type class names.
    let mut edges: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for t in all {
        let cn = format!("{}{}", prefix, t.name);
        let is_user = |jt: &str| jt.starts_with(prefix) && !jt.starts_with("DefaultInner");
        let outs: Vec<String> = match &t.kind {
            super::super::TypeKind::Struct { fields } => fields
                .iter()
                .map(|f| super::type_map::resolve_wrapper_type(&f.rust_type, all, prefix))
                .filter(|jt| is_user(jt))
                .collect(),
            super::super::TypeKind::Choice { variants } => variants
                .iter()
                .map(|v| super::type_map::resolve_wrapper_type(&v.inner_type, all, prefix))
                .filter(|jt| is_user(jt))
                .collect(),
            super::super::TypeKind::Newtype { inner_type, .. } => {
                let jt = super::type_map::resolve_wrapper_type(inner_type, all, prefix);
                if is_user(&jt) { vec![jt] } else { vec![] }
            }
        };
        edges.insert(cn, outs);
    }

    // T is recursive ⟺ T is reachable from T itself.
    let mut result = std::collections::HashSet::new();
    for start in edges.keys() {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut stack: Vec<String> = edges.get(start).cloned().unwrap_or_default();
        while let Some(cur) = stack.pop() {
            if cur == *start {
                result.insert(start.clone());
                break;
            }
            if seen.insert(cur.clone()) {
                if let Some(nexts) = edges.get(&cur) {
                    stack.extend(nexts.iter().cloned());
                }
            }
        }
    }
    result
}

/// Clean "unset" default for the no-arg constructor (reads `spec.default`).
pub fn default_val_for(jt: &str, size: usize, spec: Option<&TypeSpec>, prefix: &str) -> String {
    value_for_strategy(jt, size, spec.map(|s| s.default.as_str()).unwrap_or(""), false, prefix)
}

/// Non-zero, SIZE-compliant test filler (reads `spec.sample`).
///
/// Differs from `default` where a non-zero value makes decode bugs fail loudly,
/// e.g. u32 default is 0 but its sample is 1.
pub fn sample_value_for(jt: &str, size: usize, spec: Option<&TypeSpec>, prefix: &str) -> String {
    value_for_strategy(jt, size, spec.map(|s| s.sample.as_str()).unwrap_or(""), true, prefix)
}

/// Shared strategy dispatch for default/sample value generation.
/// Strategy names come from `type_map.json` (`default` / `sample` fields).
/// The fallback branch (type not in JSON) mirrors the JSON semantics by mode:
/// `sample == false` → unset state, `sample == true` → non-zero SIZE-compliant filler.
fn value_for_strategy(jt: &str, size: usize, strategy: &str, sample: bool, prefix: &str) -> String {
    if jt.starts_with("java.util.List<") {
        return "new java.util.ArrayList<>()".to_string();
    }

    match strategy {
        "static:true"   => "true".to_string(),
        "static:false"  => "false".to_string(),
        "static:null"   => "null".to_string(),
        "static:0"      => "0".to_string(),
        "static:0L"     => "0L".to_string(),
        "static:0f"     => "0f".to_string(),
        "static:1"      => "1".to_string(),
        "static:1L"     => "1L".to_string(),
        "static:1.5f"   => "1.5f".to_string(),
        "static:2.5"    => "2.5".to_string(),
        "empty_string"  => "\"\"".to_string(),
        "empty_bytes"   => "new byte[0]".to_string(),
        "size_string" => {
            if size > 0 { format!("\"{}\"", "x".repeat(size)) } else { "\"\"".to_string() }
        }
        "size_bytes" => {
            if size > 0 { format!("new byte[{}]", size) } else { "new byte[0]".to_string() }
        }
        "size_bytes_vals" => {
            if size > 0 {
                let bytes: Vec<String> = std::iter::repeat("1".to_string()).take(size).collect();
                format!("new byte[] {{ {} }}", bytes.join(", "))
            } else {
                "new byte[0]".to_string()
            }
        }
        _ => {
            // User-typed inner (e.g. GetFileAttributeValuesResponsePDU ::= FileEntry):
            // default → empty instance; sample → nested sample instance.
            if jt.starts_with(prefix) && !jt.starts_with("DefaultInner") {
                return if sample { format!("{}.sample()", jt) } else { format!("new {}()", jt) };
            }
            // Fallback for types not in JSON
            match jt {
                "int" | "Integer"     => if sample { "1".to_string() } else { "0".to_string() },
                "long" | "Long"       => if sample { "1L".to_string() } else { "0L".to_string() },
                "boolean" | "Boolean" => if sample { "true".to_string() } else { "false".to_string() },
                "float" | "Float"     => if sample { "1.5f".to_string() } else { "0f".to_string() },
                "double" | "Double"   => if sample { "2.5".to_string() } else { "0".to_string() },
                "String" => {
                    let n = if sample && size > 0 { size } else { if sample { 1 } else { 0 } };
                    format!("\"{}\"", "x".repeat(n))
                }
                "byte[]" => {
                    let n = if sample && size > 0 { size } else { if sample { 1 } else { 0 } };
                    let bytes: Vec<String> = std::iter::repeat("1".to_string()).take(n).collect();
                    format!("new byte[] {{ {} }}", bytes.join(", "))
                }
                // Wrapper types (ASN.1 alias chains like TimeStamp → UtcTime are
                // not in the JSON): delegate to the wrapper's own semantics.
                "DefaultInnerOctetString" => {
                    if !sample {
                        "new DefaultInnerOctetString(new byte[0])".to_string()
                    } else {
                        let n = if size > 0 { size } else { 1 };
                        let bytes: Vec<String> = std::iter::repeat("1".to_string()).take(n).collect();
                        format!("new DefaultInnerOctetString(new byte[] {{ {} }})", bytes.join(", "))
                    }
                }
                "DefaultInnerVisibleString" | "DefaultInnerUtf8String" => {
                    let n = if sample { if size > 0 { size } else { 1 } } else { 0 };
                    format!("new {}(\"{}\")", jt, "x".repeat(n))
                }
                _ => format!("new {}()", jt),
            }
        }
    }
}
