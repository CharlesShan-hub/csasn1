use super::super::*;
use super::gen_newtype_common::render_sample_factory_puts;
use super::helpers;
use super::type_map::resolve_wrapper_type;
use std::collections::HashMap;

/// Per-field size: #[size] attr (fixed) or ASN.1 SIZE resolution (skip sentinel 2).
fn field_size(f: &FieldInfo, asn_defs: &HashMap<String, String>) -> Option<usize> {
    let is_fixed = f
        .size_attr_raw
        .as_deref()
        .and_then(|r| r.parse::<usize>().ok())
        .is_some();
    if is_fixed { f.size_from_attr } else { None }.or_else(|| {
        let sz = helpers::resolve_size(&f.rust_type, asn_defs);
        if sz > 0 && sz != 2 { Some(sz) } else { None }
    })
}

/// Per-field value expression — shared by the no-arg constructor (default mode:
/// clean "unset" state) and sample() (sample mode: non-zero SIZE-compliant filler).
/// User-typed fields: default mode / cyclic types use an empty instance;
/// sample mode recurses into `X.sample()._v` for non-cyclic types.
/// List fields are mode-independent (empty container).
/// ASN.1 DEFAULT values always win (semantic, not positional).
fn field_value_expr(
    f: &FieldInfo,
    jt: &str,
    prefix: &str,
    asn_defs: &HashMap<String, String>,
    sample: bool,
    recursive: &std::collections::HashSet<String>,
) -> String {
    // ASN.1 DEFAULT wins in default mode — the field is semantically "set"
    // (e.g. `moreFollows Boolean DEFAULT 1` must construct as 1, not empty 0).
    // Sample mode keeps the non-zero filler so DEFAULT-0 fields still test loudly.
    // Wrapper types use the same `new X(v)._v` map form as the prefix branch,
    // so rebind() aliases the map instead of storing an InnerBase instance.
    if !sample {
        if let Some(ref dv) = f.default_value {
            let v = helpers::jdefault_with_value(jt, dv);
            if jt.starts_with(prefix) && !jt.starts_with("DefaultInner") {
                return format!("{}._v", v);
            }
            return v;
        }
    }
    match jt {
        "DefaultInnerOctetString" => {
            // default: empty; sample: SIZE-compliant bytes filled with 1
            if !sample {
                "new DefaultInnerOctetString(new byte[0])".to_string()
            } else {
                let n = field_size(f, asn_defs).unwrap_or(1);
                let bytes: Vec<String> = std::iter::repeat("1".to_string()).take(n).collect();
                format!("new DefaultInnerOctetString(new byte[] {{ {} }})", bytes.join(", "))
            }
        }
        "DefaultInnerVisibleString" | "DefaultInnerUtf8String" => {
            // default: "" (unset); sample: SIZE-compliant "xxx" or "x"
            let n = if sample { field_size(f, asn_defs).unwrap_or(1) } else { 0 };
            format!("new {}(\"{}\")", jt, "x".repeat(n))
        }
        "Integer" => if sample { "1".to_string() } else { "0".to_string() },
        "Long" => if sample { "1L".to_string() } else { "0L".to_string() },
        "Boolean" => if sample { "true".to_string() } else { "false".to_string() },
        "Float" => if sample { "1.5f".to_string() } else { "0f".to_string() },
        "Double" => if sample { "2.5".to_string() } else { "0".to_string() },
        _ => {
            if jt.starts_with(prefix) {
                if sample && !recursive.contains(jt) {
                    format!("{}.sample()._v", jt)
                } else {
                    format!("new {}()._v", jt)
                }
            } else if jt.starts_with("java.util.List<") {
                "new java.util.ArrayList<>()".to_string()
            } else {
                match jt {
                    "int" => if sample { "1".to_string() } else { "0".to_string() },
                    "long" => if sample { "1L".to_string() } else { "0L".to_string() },
                    "boolean" => if sample { "true".to_string() } else { "false".to_string() },
                    "float" => if sample { "1.5f".to_string() } else { "0f".to_string() },
                    "double" => if sample { "2.5".to_string() } else { "0".to_string() },
                    "String" => if sample { "\"x\"".to_string() } else { "\"\"".to_string() },
                    "byte[]" => if sample { "new byte[]{ 1 }".to_string() } else { "new byte[0]".to_string() },
                    _ => format!("new {}()", jt),
                }
            }
        }
    }
}

pub fn generate(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    cn: &str,
    asn_doc: &Option<String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
    fields: &[FieldInfo],
    asn_defs: &HashMap<String, String>,
    recursive: &std::collections::HashSet<String>,
) -> String {
    let mut c = String::new();
    let base = format!("{}Base", prefix);
    if let Some(doc) = asn_doc {
        c.push_str(doc);
    }
    c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
    c.push_str("@JsonInclude(JsonInclude.Include.NON_NULL)\n");
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(
                1,
                &format!("public static final int {} = {};", name, val),
            ));
        }
    }

    // Constructor — populate _v with defaults.
    // OPTIONAL fields without an ASN.1 DEFAULT get NO default: Jackson decode
    // runs this ctor first, so a default here would survive deserialization and
    // resurface on the next encode. Required fields keep defaults so a fresh
    // instance is always encodeable.
    c.push_str(&helpers::ln(1, &format!("public {}() {{", cn)));
    for f in fields {
        if f.optional && f.default_value.is_none() {
            continue;
        }
        let raw_name = f.identifier.as_deref().unwrap_or(&f.name);
        let jt = resolve_wrapper_type(&f.rust_type, all, prefix);
        let dflt = field_value_expr(f, &jt, prefix, asn_defs, false, recursive);
        c.push_str(&helpers::ln(
            2,
            &format!("_v.put(\"{}\", {});", raw_name, dflt),
        ));
    }
    c.push_str(&helpers::ln(1, "}"));

    // sample() — test-filler factory: every field gets a value (incl. OPTIONAL),
    // unlike the ctor which must leave bare OPTIONAL fields untouched (Jackson).
    let mut puts = Vec::new();
    for f in fields {
        let raw_name = f.identifier.as_deref().unwrap_or(&f.name);
        let jt = resolve_wrapper_type(&f.rust_type, all, prefix);
        let val = field_value_expr(f, &jt, prefix, asn_defs, true, recursive);
        puts.push(helpers::ln(2, &format!("r._v.put(\"{}\", {});", raw_name, val)));
    }
    c.push_str(&render_sample_factory_puts(cn, &puts));

    // @JsonAnySetter — populate _v during Jackson deserialisation (no _optional tracking)
    c.push_str(&helpers::ln(1, "@JsonAnySetter"));
    c.push_str(&helpers::ln(
        1,
        "public void setField(String key, Object value) {",
    ));
    c.push_str(&helpers::ln(2, "if (key.startsWith(\"_\")) return;"));
    c.push_str(&helpers::ln(2, "_v.put(key, value);"));
    c.push_str(&helpers::ln(1, "}"));

    // encode()/encodeTest() inherited from {base} — this type has no special
    // JSON shape (fields live directly in _v), so the base implementation
    // (InnerBase.toJson(_v) + InnerNative.encode) applies unchanged.

    // typeName — ASN.1 dispatch name for native encode/decode
    c.push_str(&helpers::ln(1, "@Override"));
    c.push_str(&helpers::ln(
        1,
        &format!("protected String typeName() {{ return \"{}\"; }}", ti.name),
    ));

    // decode
    c.push_str(&helpers::ln(
        1,
        &format!("public static {} decode(byte[] data) {{", cn),
    ));
    c.push_str(&helpers::ln(
        2,
        &format!(
            "return {}.decode({}.class, \"{}\", data);",
            base, cn, ti.name
        ),
    ));
    c.push_str(&helpers::ln(1, "}"));
    c.push_str("}\n");
    c
}
