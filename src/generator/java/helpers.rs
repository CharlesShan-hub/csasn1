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
        if c == '-' {
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

/// Convert a Rust-style name to a safe Java field name (camelCase, avoid keywords).
pub fn safe_field_name(name: &str) -> String {
    let n = camel(name);
    if JAVA_KEYWORDS.contains(&n.as_str()) {
        format!("_{}", n)
    } else {
        n
    }
}

/// Default value for a Java type (used in field initialization).
pub fn jdefault(jt: &str, is_list: bool) -> &'static str {
    if is_list {
        return "new java.util.ArrayList<>()";
    }
    match jt {
        "int" => "0",
        "long" => "0L",
        "boolean" => "false",
        "float" => "0.0f",
        "double" => "0.0",
        "Integer" | "Long" | "Boolean" | "Float" | "Double" | "String" | "byte[]" => "null",
        _ => "null",
    }
}

/// Derive test source directory by replacing `src/main/java` with `src/test/java`.
pub fn derive_test_dir(out_dir: &PathBuf) -> PathBuf {
    let s = out_dir.to_string_lossy().to_string();
    let s = s.replace("src\\main\\java", "src\\test\\java");
    let s = s.replace("src/main/java", "src/test/java");
    PathBuf::from(s)
}

/// Derive Maven module root by stripping `src/main/java/...` from out_dir.
pub fn derive_project_root(out_dir: &PathBuf) -> PathBuf {
    let s = out_dir.to_string_lossy().to_string();
    if let Some(pos) = s.find("src\\main\\java") {
        PathBuf::from(&s[..pos])
    } else if let Some(pos) = s.find("src/main/java") {
        PathBuf::from(&s[..pos])
    } else {
        out_dir.clone()
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
