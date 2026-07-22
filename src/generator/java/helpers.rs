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

fn camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '-' || c == '_' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Convert a Rust-style name to a safe Java field name (avoid keywords, no camelCase).
pub fn safe_field_name(name: &str) -> String {
    if JAVA_KEYWORDS.contains(&name) {
        format!("_{}", name)
    } else {
        name.to_string()
    }
}

/// Convert a Rust-style name to camelCase Java field name (also avoids keywords).
pub fn camel_case_name(name: &str) -> String {
    let n = camel(name);
    if JAVA_KEYWORDS.contains(&n.as_str()) {
        format!("_{}", n)
    } else {
        n
    }
}

/// Default value for a Java type (used in field initialization).
pub fn jdefault(jt: &str, is_list: bool) -> String {
    if is_list {
        return "new java.util.ArrayList<>()".to_string();
    }
    match jt {
        "int" => "0".to_string(),
        "long" => "0L".to_string(),
        "boolean" => "false".to_string(),
        "float" => "0.0f".to_string(),
        "double" => "0.0".to_string(),
        "Integer" | "Long" | "Boolean" | "Float" | "Double" | "String" | "byte[]" => "null".to_string(),
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

/// Generate the encode(String) + encode() overload pair.
pub fn enc_overload(c: &mut String, body: &str) {
    c.push_str(&ln(1, "public byte[] encode(String enc) {"));
    c.push_str(body);
    c.push_str(&ln(1, "}"));
    c.push_str(&ln(1, "public byte[] encode() {"));
    c.push_str(&ln(2, "return encode(DEFAULT_ENCODING);"));
    c.push_str(&ln(1, "}"));
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
        Some((Some(min), Some(max))) if min == max => max,       // fixed
        Some((Some(min), Some(max))) => (min + max) / 2 + 1,     // mid-range
        Some((_, Some(max))) => max,                              // max only
        Some((Some(min), None)) => min + 1,                       // min only
        _ => return 2,                                            // default
    };
    if size == 0 { 2 } else { size }
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
