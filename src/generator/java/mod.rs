use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use super::*;

mod class_gen;
mod test_gen;

/// Default Java class name prefix
const DEFAULT_PREFIX: &str = "Asn";

pub struct JavaConfig {
    pub prefix: String,
    pub default_enc: String,
    pub package: String,
    pub out_dir: PathBuf,
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            prefix: DEFAULT_PREFIX.to_string(),
            default_enc: "ber".to_string(),
            package: String::new(),
            out_dir: PathBuf::from("java/src"),
        }
    }
}

/// Entry point: generate Java classes from parsed types
pub fn generate(
    types: &[TypeInfo],
    cfg: &JavaConfig,
    asn_defs: &HashMap<String, String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
) {
    fs::create_dir_all(&cfg.out_dir).expect("failed to create output directory");
    for t in types {
        let code = class_gen::gen_class(t, types, &cfg.prefix, &cfg.default_enc, &cfg.package, asn_defs, named_consts);
        fs::write(
            cfg.out_dir.join(format!("{}{}.java", cfg.prefix, t.name)),
            &code,
        )
        .unwrap();

        // Generate test file alongside the main class
        let test_code = test_gen::gen_test_class(t, types, &cfg.prefix, &cfg.package, asn_defs);
        let test_dir = derive_test_dir(&cfg.out_dir);
        fs::create_dir_all(&test_dir).expect("failed to create test output directory");
        fs::write(
            test_dir.join(format!("{}{}Test.java", cfg.prefix, t.name)),
            &test_code,
        )
        .unwrap();
    }
    fs::write(
        cfg.out_dir.join(format!("{}Native.java", cfg.prefix)),
        &gen_native(&cfg.prefix, &cfg.package),
    )
    .unwrap();
    // Generate base class
    fs::write(
        cfg.out_dir.join(format!("{}Base.java", cfg.prefix)),
        &gen_base(&cfg.prefix, &cfg.package, &cfg.default_enc),
    )
    .unwrap();
    println!(
        "✓ generated {} Java classes (incl. {}Native.java, {}Base.java) in {:?}",
        types.len(),
        cfg.prefix,
        cfg.prefix,
        cfg.out_dir
    );
    println!(
        "✓ generated {} Java test classes in {:?}",
        types.len(),
        derive_test_dir(&cfg.out_dir)
    );
}

/// Derive test source directory from main source directory
/// by replacing `src/main/java` with `src/test/java`.
fn derive_test_dir(out_dir: &PathBuf) -> PathBuf {
    let s = out_dir.to_string_lossy().to_string();
    let s = s.replace("src\\main\\java", "src\\test\\java");
    let s = s.replace("src/main/java", "src/test/java");
    PathBuf::from(s)
}

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

pub fn resolve_java_type(rt: &str, all: &[TypeInfo], prefix: &str) -> String {
    // Strip Option<...> wrapper to get inner type
    let rt = rt.trim();
    if rt.starts_with("Option <") {
        let inner = rt
            .trim_start_matches("Option <")
            .trim_end_matches('>')
            .trim()
            .to_string();
        return resolve_java_type(&inner, all, prefix);
    }

    // Handle SequenceOf<T> / Vec<T> / Box<T>
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
                    // inner_type is a raw Rust type string; resolve it
                    return resolve_java_type(inner_type, all, prefix);
                }
            }
            return format!("{}{}", prefix, s);
        }
    };
    base.to_string()
}

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

/// Generate a type literal usable by Jackson convertValue.
/// Uses TypeReference for generic List<T>, .class for everything else.
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

pub fn safe_field_name(name: &str) -> String {
    let n = camel(name);
    if JAVA_KEYWORDS.contains(&n.as_str()) {
        format!("_{}", n)
    } else {
        n
    }
}

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

fn gen_native(prefix: &str, package: &str) -> String {
    let pkg = if package.is_empty() {
        String::new()
    } else {
        format!("package {};\n\n", package)
    };
    format!(
        r#"// Auto-generated. JNI bridge to native ASN.1 codec.

{pkg}public class {pfx}Native {{
    static {{
        System.loadLibrary("asn1");
    }}

    /**
     * Encode JSON to ASN.1 binary.
     * @param typeName  ASN.1 type name (e.g. "Apdu", "Boolean")
     * @param encoding  encoding rule ("ber", "der", "aper", "uper")
     * @param json      JSON representation of the data
     * @return encoded binary data
     */
    public static native byte[] encode(String typeName, String encoding, String json);

    /**
     * Decode ASN.1 binary to JSON.
     * @param typeName  ASN.1 type name
     * @param encoding  encoding rule
     * @param data      binary encoded data
     * @return JSON representation
     */
    public static native String decode(String typeName, String encoding, byte[] data);
}}
"#,
        pkg = pkg,
        pfx = prefix,
    )
}

fn gen_base(prefix: &str, package: &str, default_enc: &str) -> String {
    let pkg = if package.is_empty() {
        String::new()
    } else {
        format!("package {};\n\n", package)
    };
    format!(
        r#"// Auto-generated. Base class for all {pfx} data types.

{pkg}import com.fasterxml.jackson.databind.ObjectMapper;

public abstract class {pfx}Base {{
    public static final String DEFAULT_ENCODING = "{enc}";

    @Override
    public String toString() {{
        try {{
            return new ObjectMapper().writeValueAsString(this);
        }} catch (Exception e) {{
            return getClass().getSimpleName() + "{{...}}";
        }}
    }}
}}
"#,
        pkg = pkg,
        pfx = prefix,
        enc = default_enc,
    )
}
