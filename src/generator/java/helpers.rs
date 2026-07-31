use std::path::PathBuf;

// Java reserved words — cannot be used as field names
const JAVA_KEYWORDS: &[&str] = &[
    "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
    "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
    "finally", "float", "for", "goto", "if", "implements", "import", "instanceof", "int",
    "interface", "long", "native", "new", "package", "private", "protected", "public",
    "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
    "throw", "throws", "transient", "try", "void", "volatile", "while", "true", "false",
    "null",
];

/// Convert a Rust-style name to a safe Java field name (avoid keywords, no camelCase).
#[allow(dead_code)]
pub fn safe_field_name(name: &str) -> String {
    if JAVA_KEYWORDS.contains(&name) {
        format!("_{}", name)
    } else {
        name.to_string()
    }
}

/// Generate a Java default initialization from a Rust default expression.
///
/// Rust defaults look like `Type(value)`, e.g. `Boolean(1)` or `Int32(5)`.
/// Maps to `new JavaType(value)`, e.g. `new InnerBoolean(1)`.
pub fn jdefault_with_value(jt: &str, rust_expr: &str) -> String {
    // Parse "Type(value)" or "Type ( value )" to extract the inner value
    let trimmed = rust_expr.trim();
    let raw_value = if let Some(paren_start) = trimmed.find('(') {
        if let Some(paren_end) = trimmed.rfind(')') {
            trimmed[paren_start + 1..paren_end].trim().to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        trimmed.to_string()
    };
    // For primitive Java types, return just the value, not "new int(value)"
    match jt {
        "int" | "long" | "boolean" | "float" | "double" => return raw_value,
        _ => format!("new {}({})", jt, raw_value),
    }
}

/// Default value for a Java type (used in field initialization).
pub fn jdefault(jt: &str, is_list: bool) -> String {
    if is_list {
        return "new java.util.ArrayList<>()".to_string();
    }
    match jt {
        "int" => "1".to_string(),
        "long" => "1L".to_string(),
        "boolean" => "true".to_string(),
        "float" => "1.5f".to_string(),
        "double" => "2.5".to_string(),
        "Integer" | "Long" | "Boolean" | "Float" | "Double" => "null".to_string(),
        "String" => "\"x\"".to_string(),
        "byte[]" => "new byte[]{ 1 }".to_string(),
        // Wrapper types (user-defined ASN.1 types) — create new instance for non-null default
        _ => format!("new {}()", jt),
    }
}

/// Convert Java package name to a relative directory path.
/// e.g. "com.example.csasn1" → "com/example/csasn1"
pub fn package_to_path(pkg: &str) -> PathBuf {
    if pkg.is_empty() {
        PathBuf::new()
    } else {
        PathBuf::from(pkg.replace('.', "/"))
    }
}

/// Build an indented line (4 spaces per indent level).
pub fn ln(indent: usize, s: &str) -> String {
    format!("{}{}\n", " ".repeat(indent * 4), s)
}

/// Generate encode methods for struct/choice types (no _set filtering on encodeArg).
/// If `encodeArg` is "this", MAPPER.valueToTree is used and opt_fields are stripped.
pub fn gen_encode_methods(c: &mut String, _cn: &str, native: &str, type_name: &str,
                          encode_arg: &str, has_optional: bool, opt_fields: &[&str]) {
    // encode() — strict
    c.push_str(&ln(1, "public byte[] encode() {"));
    c.push_str(&ln(2, "String _json = null;"));
    c.push_str(&ln(2, "String _vStr = null;"));
    c.push_str(&ln(2, "try {"));
    if has_optional {
        c.push_str(&ln(3, "com.fasterxml.jackson.databind.node.ObjectNode _root = MAPPER.valueToTree(this);"));
        for fname in opt_fields {
            c.push_str(&ln(3, &format!("if (!_set.contains(\"{}\")) _root.remove(\"{}\");", fname, fname)));
        }
        c.push_str(&ln(3, "_json = MAPPER.writeValueAsString(_root);"));
    } else {
        c.push_str(&ln(3, "_vStr = MAPPER.writeValueAsString(_v);"));
        c.push_str(&ln(3, &format!("_json = {};", encode_arg)));
    }
    c.push_str(&ln(3, &format!("return {}.encode(\"{}\", DEFAULT_ENCODING, _json);", native, type_name)));
    c.push_str(&ln(2, "} catch (Exception e) {"));
    c.push_str(&ln(3, &format!("throw new RuntimeException(\"encode {} failed, _v=\" + _vStr + \", json=\" + _json, e);", type_name)));
    c.push_str(&ln(2, "}"));
    c.push_str(&ln(1, "}"));

    // encodeTest() — lenient (all fields)
    c.push_str(&ln(1, "public byte[] encodeTest() {"));
    c.push_str(&ln(2, "String _json = null;"));
    c.push_str(&ln(2, "String _vStr = null;"));
    c.push_str(&ln(2, "try {"));
    c.push_str(&ln(3, "_vStr = MAPPER.writeValueAsString(_v);"));
    c.push_str(&ln(3, &format!("_json = {};", encode_arg)));
    c.push_str(&ln(3, "System.err.println(\"_v: \" + _vStr);"));
    c.push_str(&ln(3, "System.err.println(\"JSON: \" + _json);"));
    c.push_str(&ln(3, &format!("return {}.encode(\"{}\", DEFAULT_ENCODING, _json);", native, type_name)));
    c.push_str(&ln(2, "} catch (Exception e) {"));
    c.push_str(&ln(3, &format!("throw new RuntimeException(\"encodeTest {} failed, _v=\" + _vStr + \", json=\" + _json, e);", type_name)));
    c.push_str(&ln(2, "}"));
    c.push_str(&ln(1, "}"));
}

/// Generate `decode(byte[])` for struct/choice types.
pub fn gen_decode_method(c: &mut String, cn: &str, native: &str, type_name: &str) {
    c.push_str(&ln(1, &format!("public static {} decode(byte[] data) {{", cn)));
    c.push_str(&ln(2, "try {"));
    c.push_str(&ln(3, &format!("return MAPPER.readValue({}.decode(\"{}\", DEFAULT_ENCODING, data), {}.class);", native, type_name, cn)));
    c.push_str(&ln(2, "} catch (Exception e) {"));
    c.push_str(&ln(3, "throw new RuntimeException(e);"));
    c.push_str(&ln(2, "}"));
    c.push_str(&ln(1, "}"));
}

/// Return the generator version string (e.g. "csasn1 v0.1.0").
/// No timestamp — the file system / git already tracks generation time, and a
/// timestamp would produce noisy diffs on every regeneration.
pub fn gen_version() -> String {
    format!("csasn1 v{}", env!("CARGO_PKG_VERSION"))
}
/// Parse SIZE constraint from an ASN.1 definition line.
/// Returns `(min, max)` where both are `Some` for fixed/constrained sizes.
/// - `SIZE(8)`       → (Some(8), Some(8))
/// - `SIZE(0..129)`  → (Some(0), Some(129))
/// - `SIZE(1..MAX)`  → (Some(1), None)
/// - no SIZE found   → None
pub fn parse_asn1_size(def: &str) -> Option<(Option<usize>, Option<usize>)> {
    let def_clean = def.split("--").next().unwrap_or(def); // strip ASN.1 comments
    let paren_start = def_clean.find("SIZE").and_then(|p| {
        let after = &def_clean[p + 4..];
        after.find('(').map(|q| p + 4 + q + 1)
    })?;
    let rest = &def_clean[paren_start..];
    let paren_end = rest.find(')')?;
    let content = rest[..paren_end].trim();

    // Fixed: SIZE(8)
    if let Ok(n) = content.parse::<usize>() {
        return Some((Some(n), Some(n)));
    }

    // Range: SIZE(M..N) or SIZE(M..MAX)
    if let Some(dotdot) = content.find("..") {
        let min_s = content[..dotdot].trim();
        let max_s = content[dotdot + 2..].trim();
        let min = if min_s.is_empty() { None } else { min_s.parse::<usize>().ok() };
        let max = if max_s == "MAX" { None } else { max_s.parse::<usize>().ok() };
        return Some((min, max));
    }

    None
}

/// Generate a sensible test data size from an ASN.1 definition.
/// Falls back to 2 if no SIZE constraint is found.
pub fn test_data_size(def: Option<&str>) -> usize {
    let size = match def.and_then(parse_asn1_size) {
        Some((Some(min), Some(max))) if min == max => max as usize, // fixed SIZE(N) -> N
        Some((Some(min), None)) => (min + 1) as usize,              // min only -> min+1
        Some((Some(min), Some(_))) if min > 0 => (min + 1) as usize, // range with min>0 -> min+1
        Some((_, Some(_))) => 1,                                     // range min=0 or max-only -> 1
        _ => 2,                                                      // default
    };
    if size == 0 { 1 } else { size }
}

/// Resolve test data size through ASN.1 type alias chain.
/// e.g. TimeStamp ::= UtcTime → look up UtcTime's SIZE(8).
pub fn resolve_size(type_name: &str, asn_defs: &std::collections::HashMap<String, String>) -> usize {
    let mut seen = std::collections::HashSet::new();
    let mut current = type_name.to_string();
    loop {
        if seen.contains(&current) { return 2; }
        seen.insert(current.clone());
        match asn_defs.get(&current).map(|s| s.as_str()) {
            Some(def) => {
                let sz = test_data_size(Some(def));
                if sz != 2 { return sz; }
                // Check if it's a simple alias: Type ::= OtherType (no {, no BIT STRING)
                if let Some(eq_pos) = def.find("::=") {
                    let after = def[eq_pos + 3..].trim();
                    if !after.contains('{') && !after.contains("BIT STRING") {
                        if let Some(next) = after.split_whitespace().next() {
                            current = next.to_string();
                            continue;
                        }
                    }
                }
                return sz;
            }
            None => return 2,
        }
    }
}
