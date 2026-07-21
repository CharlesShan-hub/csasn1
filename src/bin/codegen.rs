use std::fs;
use std::path::PathBuf;
use syn::{Item, Type, Fields};
use quote::ToTokens;

/// Default Java class name prefix
const DEFAULT_PREFIX: &str = "Cms";

#[derive(Debug)]
enum TypeKind {
    Newtype { inner_type: String },
    Struct { fields: Vec<FieldInfo> },
    Choice { variants: Vec<VariantInfo> },
}

#[derive(Debug)]
struct FieldInfo {
    name: String,
    rust_type: String,
    optional: bool,
    is_list: bool,
}

#[derive(Debug)]
struct VariantInfo { name: String, inner_type: String }

#[derive(Debug)]
struct TypeInfo { name: String, kind: TypeKind }

fn prompt(msg: &str, default: &str) -> String {
    use std::io::{Write, BufRead};
    let full = if default.is_empty() {
        format!("{}: ", msg)
    } else {
        format!("{} [{}]: ", msg, default)
    };
    print!("{}", full);
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line).is_ok() {
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() { return trimmed; }
    }
    default.to_string()
}

fn main() {
    let mut spec_path = "specs/dlt2811.asn".to_string();
    let mut out_dir = PathBuf::from("java/src");
    let mut prefix = DEFAULT_PREFIX.to_string();
    let mut default_enc = "ber".to_string();
    let mut package = String::new();

    let args: Vec<String> = std::env::args().collect();
    let interactive = args.len() <= 1;

    if interactive {
        println!("── csasn1 interactive mode ──");
        spec_path = prompt("ASN.1 spec file", &spec_path);
        out_dir = PathBuf::from(prompt("Output directory", &out_dir.to_string_lossy()));
        prefix = prompt("Class prefix", &prefix);
        default_enc = prompt("Default encoding (ber/der/aper/uper)", &default_enc);
        package = prompt("Java package (empty = none)", "");
        println!();
    }

    let mut args_iter = args.into_iter().peekable();
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--src" => spec_path = args_iter.next().expect("--src requires a value"),
            "--out" | "--dest" => out_dir = PathBuf::from(args_iter.next().expect("--out/--dest requires a value")),
            "--prefix" => prefix = args_iter.next().expect("--prefix requires a value"),
            "--enc" => default_enc = args_iter.next().expect("--enc requires a value"),
            "--package" => package = args_iter.next().expect("--package requires a value"),
            "--bin" => {
                let _bin = args_iter.next().expect("--bin requires a value");
            }
            _ => {}
        }
    }

    // --src .asn files auto-map to the generated .rs via build.rs
    let src_path = if spec_path.ends_with(".asn") {
        "src/generated.rs".to_string()
    } else {
        spec_path.clone()
    };

    let src = fs::read_to_string(&src_path).expect(&format!("failed to read {}", src_path));
    let ast = syn::parse_file(&src).expect("failed to parse generated.rs");
    let types = extract_types(&ast, &prefix);

    // Generate type classes
    fs::create_dir_all(&out_dir).expect("failed to create output directory");
    for t in &types {
        let code = gen_class(t, &types, &prefix, &default_enc, &package);
        fs::write(out_dir.join(format!("{}{}.java", prefix, t.name)), &code).unwrap();
    }

    // Generate Native bridge class
    fs::write(out_dir.join(format!("{}Native.java", prefix)), &gen_native(&prefix, &package)).unwrap();
    println!("✓ generated {} Java classes (incl. {}Native.java) in {:?}", types.len(), prefix, out_dir);
}

fn extract_types(ast: &syn::File, prefix: &str) -> Vec<TypeInfo> {
    let mut types = Vec::new();
    if let Some(syn::Item::Mod(m)) = ast.items.first() {
        if let Some((_, items)) = &m.content {
            for inner in items {
                match inner {
                    Item::Struct(s) => types.push(TypeInfo {
                        name: s.ident.to_string(), kind: analyze_struct(s, prefix),
                    }),
                    Item::Enum(e) => types.push(TypeInfo {
                        name: e.ident.to_string(), kind: analyze_enum(e),
                    }),
                    _ => {}
                }
            }
        }
    }
    types
}

fn attr_contains(attrs: &[syn::Attribute], pat: &str) -> bool {
    attrs.iter().any(|a| a.into_token_stream().to_string().contains(pat))
}

fn analyze_struct(s: &syn::ItemStruct, prefix: &str) -> TypeKind {
    if attr_contains(&s.attrs, "delegate") {
        if let Fields::Unnamed(ref u) = s.fields {
            if let Some(f) = u.unnamed.first() {
                return TypeKind::Newtype { inner_type: resolve_java_type(&type_str(&f.ty), &[], prefix) };
            }
        }
        return TypeKind::Newtype { inner_type: "int".into() };
    }
    let mut fields = Vec::new();
    for f in s.fields.iter() {
        let name = f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
        let rt = type_str(&f.ty);
        let optional = rt.starts_with("Option <");
        let is_list = rt.contains("Vec <") || rt.contains("SequenceOf <");
        fields.push(FieldInfo { name, rust_type: rt, optional, is_list });
    }
    TypeKind::Struct { fields }
}

fn analyze_enum(e: &syn::ItemEnum) -> TypeKind {
    let variants = e.variants.iter().filter_map(|v| {
        if let Fields::Unnamed(ref u) = v.fields {
            u.unnamed.first().map(|f| VariantInfo {
                name: v.ident.to_string(),
                inner_type: type_str(&f.ty),
            })
        } else { None }
    }).collect();
    TypeKind::Choice { variants }
}

fn type_str(ty: &Type) -> String {
    quote::quote!(#ty).to_string()
        .replace(" , ", ", ").replace("  ", " ").trim().to_string()
}

fn resolve_java_type(rt: &str, all: &[TypeInfo], prefix: &str) -> String {
    // Strip Option<...> wrapper to get inner type
    let rt = rt.trim();
    if rt.starts_with("Option <") {
        let inner = rt.trim_start_matches("Option <")
            .trim_end_matches('>').trim().to_string();
        return resolve_java_type(&inner, all, prefix);
    }

    // Handle SequenceOf<T> / Vec<T>
    if rt.starts_with("SequenceOf <") || rt.starts_with("Vec <") {
        let inner = rt.trim_start_matches("SequenceOf <").trim_start_matches("Vec <")
            .trim_end_matches('>').trim().to_string();
        let inner_java = resolve_java_type(&inner, all, prefix);
        return format!("java.util.List<{}>", inner_java);
    }

    let base = match rt {
        "bool" => "boolean",
        "u8" | "i8" | "u16" | "i16" | "u32" | "i32" => "int",
        "u64" | "i64" => "long",
        "f32" => "float" , "f64" => "double",
        s if s == "String" || s.starts_with("Utf8String") || s.starts_with("VisibleString") => "String",
        s if s.starts_with("OctetString") || s.starts_with("FixedOctetString") => "byte[]",
        s if s.starts_with("Integer") => "int",
        s if s.starts_with("FixedBitString") => "int",
        s if s.starts_with("BitString") => "byte[]",
        s => {
            if let Some(ti) = all.iter().find(|t| t.name == s) {
                if let TypeKind::Newtype { ref inner_type } = ti.kind {
                    return inner_type.clone();
                }
            }
            return format!("{}{}", prefix, s);
        }
    };
    base.to_string()
}

fn resolve_java_type_nullable(rt: &str, all: &[TypeInfo], prefix: &str) -> String {
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

/// Generate a type literal usable by Jackson convertValue
/// Uses TypeReference for generic List<T>, .class for everything else
fn java_type_ref(jt: &str) -> String {
    if jt.starts_with("java.util.List<") {
        let inner = jt.trim_start_matches("java.util.List<")
            .trim_end_matches('>').trim();
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
    "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char",
    "class", "const", "continue", "default", "do", "double", "else", "enum",
    "extends", "final", "finally", "float", "for", "goto", "if", "implements",
    "import", "instanceof", "int", "interface", "long", "native", "new",
    "package", "private", "protected", "public", "return", "short", "static",
    "strictfp", "super", "switch", "synchronized", "this", "throw", "throws",
    "transient", "try", "void", "volatile", "while", "true", "false", "null",
];

fn safe_field_name(name: &str) -> String {
    let n = camel(name);
    if JAVA_KEYWORDS.contains(&n.as_str()) {
        format!("_{}", n)
    } else {
        n
    }
}

fn camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '-' { upper = true; } else if upper {
            out.push(c.to_ascii_uppercase()); upper = false;
        } else { out.push(c); }
    }
    out
}

fn jdefault(jt: &str, is_list: bool) -> &'static str {
    if is_list { return "new java.util.ArrayList<>()"; }
    match jt { "int" => "0", "long" => "0L", "boolean" => "false",
        "float" => "0.0f", "double" => "0.0",
        "Integer" | "Long" | "Boolean" | "Float" | "Double" | "String" | "byte[]" => "null",
        _ => "null" }
}

// ─── Helpers: indented line builder ───

fn ln(indent: usize, s: &str) -> String {
    format!("{}{}\n", " ".repeat(indent * 4), s)
}

fn gen_class(ti: &TypeInfo, all: &[TypeInfo], prefix: &str, default_enc: &str, package: &str) -> String {
    let cn = format!("{}{}", prefix, ti.name);
    let mut c = String::new();
    c.push_str(&format!("// Auto-generated. ASN.1 type: {}\n\n", ti.name));
    if !package.is_empty() {
        c.push_str(&format!("package {};\n\n", package));
    }
    c.push_str("import com.fasterxml.jackson.annotation.*;\n");
    c.push_str("import com.fasterxml.jackson.databind.*;\n\n");
    let enc_const = |c: &mut String| {
        c.push_str(&ln(1, &format!("public static final String DEFAULT_ENCODING = \"{}\";", default_enc)));
    };
    let enc_overload = |c: &mut String, body: &str| {
        c.push_str(&ln(1, "public byte[] encode(String enc) {"));
        c.push_str(body);
        c.push_str(&ln(1, "}"));
        c.push_str(&ln(1, "public byte[] encode() {"));
        c.push_str(&ln(2, "return encode(DEFAULT_ENCODING);"));
        c.push_str(&ln(1, "}"));
    };

    match &ti.kind {
        TypeKind::Newtype { inner_type } => {
            let jt = resolve_java_type(&ti.name, all, prefix);
            let jt = if jt == cn { inner_type.clone() } else { jt };
            let wrap = |p: &str| -> String { match p { "int" => "Integer".into(), "long" => "Long".into(), "boolean" => "Boolean".into(), "float" => "Float".into(), "double" => "Double".into(), _ => p.to_string() } };
            let obj_type = wrap(&jt);

            c.push_str(&format!("public class {} {{\n", cn));
            enc_const(&mut c);
            c.push_str(&ln(1, &format!("public {} value;", jt)));
            c.push_str(&ln(1, &format!("public {}() {{}}", cn)));
            c.push_str(&ln(1, &format!("public {}({} value) {{ this.value = value; }}", cn, jt)));
            enc_overload(&mut c, &ln(2, &format!(
                "return CmsNative.encode(\"{}\", enc, String.valueOf(this.value));\n",
                ti.name
            )));
            c.push_str(&ln(1, &format!(
                "public static {} decode(String enc, byte[] data) {{", cn
            )));
            c.push_str(&ln(2, "try {"));
            c.push_str(&ln(3, &format!(
                "String json = CmsNative.decode(\"{}\", enc, data);",
                ti.name
            )));
            c.push_str(&ln(3, &format!("{} r = new {}();", cn, cn)));
            c.push_str(&ln(3, &format!(
                "r.value = {}.parseInt(json.trim());", obj_type
            )));
            c.push_str(&ln(3, "return r;"));
            c.push_str(&ln(2, "} catch (Exception e) {"));
            c.push_str(&ln(3, "throw new RuntimeException(e);"));
            c.push_str(&ln(2, "}"));
            c.push_str(&ln(1, "}"));
            c.push_str("}\n");
        }

        TypeKind::Struct { fields } => {
            let has_optional = fields.iter().any(|f| f.optional);
            c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
            if has_optional {
                c.push_str("@JsonInclude(JsonInclude.Include.NON_NULL)\n");
            }
            c.push_str(&format!("public class {} {{\n", cn));
            enc_const(&mut c);

            for f in fields {
                let jt = if f.optional {
                    resolve_java_type_nullable(&f.rust_type, all, prefix)
                } else {
                    resolve_java_type(&f.rust_type, all, prefix)
                };
                let fname = safe_field_name(&f.name);
                let dflt = jdefault(&jt, f.is_list);
                c.push_str(&ln(1, &format!("@JsonProperty public {} {} = {};", jt, fname, dflt)));
            }

            // encode
            enc_overload(&mut c, &format!(
                "{}{}{}{}{}{}",
                ln(2, "try {"),
                ln(3, &format!(
                    "return CmsNative.encode(\"{}\", enc,",
                    ti.name
                )),
                ln(4, "new ObjectMapper().writeValueAsString(this));"),
                ln(2, "} catch (Exception e) {"),
                ln(3, "throw new RuntimeException(e);"),
                ln(2, "}"),
            ));

            // decode
            c.push_str(&ln(1, &format!(
                "public static {} decode(String enc, byte[] data) {{", cn
            )));
            c.push_str(&ln(2, "try {"));
            c.push_str(&ln(3, "return new ObjectMapper()"));
            c.push_str(&ln(4, &format!(
                ".readValue(CmsNative.decode(\"{}\", enc, data), {}.class);",
                ti.name, cn
            )));
            c.push_str(&ln(2, "} catch (Exception e) {"));
            c.push_str(&ln(3, "throw new RuntimeException(e);"));
            c.push_str(&ln(2, "}"));
            c.push_str(&ln(1, "}"));
            c.push_str("}\n");
        }

        TypeKind::Choice { variants } => {
            c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
            c.push_str(&format!("public class {} {{\n", cn));
            enc_const(&mut c);
            c.push_str(&ln(1, "public String _choice;"));
            c.push_str(&ln(1, "private static final ObjectMapper MAPPER = new ObjectMapper();"));

            for v in variants {
                let jt = resolve_java_type(&v.inner_type, all, prefix);
                let fname = safe_field_name(&v.name);
                c.push_str(&ln(1, &format!("@JsonIgnore public {} {};", jt, fname)));
            }

            // Serialization: only output the active branch (handle null _choice)
            c.push_str(&ln(1, "@JsonAnyGetter"));
            c.push_str(&ln(1, "public java.util.Map<String, Object> serializeChoice() {"));
            c.push_str(&ln(2, "var map = new java.util.HashMap<String, Object>();"));
            c.push_str(&ln(2, "if (_choice != null) {"));
            c.push_str(&ln(3, "map.put(\"_choice\", _choice);"));
            for v in variants {
                let fname = safe_field_name(&v.name);
                c.push_str(&ln(3, &format!(
                    "if (\"{}\".equals(_choice)) map.put(\"{}\", {});",
                    v.name, v.name, fname
                )));
            }
            c.push_str(&ln(2, "}"));
            c.push_str(&ln(2, "return map;"));
            c.push_str(&ln(1, "}"));

            // Deserialization
            c.push_str(&ln(1, "@JsonAnySetter"));
            c.push_str(&ln(1, "public void deserializeChoice(String key, Object value) {"));
            c.push_str(&ln(2, "if (\"_choice\".equals(key)) return;"));
            c.push_str(&ln(2, "this._choice = key;"));
            for v in variants {
                let fname = safe_field_name(&v.name);
                let jt = resolve_java_type(&v.inner_type, all, prefix);
                let tref = java_type_ref(&jt);
                c.push_str(&ln(2, &format!("if (\"{}\".equals(key)) {{", v.name)));
                c.push_str(&ln(3, &format!(
                    "this.{} = MAPPER.convertValue(value, {});",
                    fname, tref
                )));
                c.push_str(&ln(2, "}"));
            }
            c.push_str(&ln(1, "}"));

            // encode
            enc_overload(&mut c, &format!(
                "{}{}{}{}{}",
                ln(2, "try {"),
                ln(3, &format!(
                    "return CmsNative.encode(\"{}\", enc, MAPPER.writeValueAsString(this));",
                    ti.name
                )),
                ln(2, "} catch (Exception e) {"),
                ln(3, "throw new RuntimeException(e);"),
                ln(2, "}"),
            ));

            // decode
            c.push_str(&ln(1, &format!(
                "public static {} decode(String enc, byte[] data) {{", cn
            )));
            c.push_str(&ln(2, "try {"));
            c.push_str(&ln(3, &format!(
                "return MAPPER.readValue(CmsNative.decode(\"{}\", enc, data), {}.class);",
                ti.name, cn
            )));
            c.push_str(&ln(2, "} catch (Exception e) {"));
            c.push_str(&ln(3, "throw new RuntimeException(e);"));
            c.push_str(&ln(2, "}"));
            c.push_str(&ln(1, "}"));
            c.push_str("}\n");
        }
    }
    c
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
        System.loadLibrary("csasn1");
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
