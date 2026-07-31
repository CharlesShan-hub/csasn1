use std::collections::HashMap;
use super::super::*;
use super::type_map::resolve_wrapper_type;
use super::helpers;

pub fn generate(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    cn: &str,
    asn_doc: &Option<String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
    fields: &[FieldInfo],
    asn_defs: &HashMap<String, String>,
) -> String {
    let mut c = String::new();
    let base = format!("{}Base", prefix);
    let native = format!("{}Native", prefix);
    if let Some(doc) = asn_doc { c.push_str(doc); }
    c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
    c.push_str("@JsonInclude(JsonInclude.Include.NON_NULL)\n");
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(1, &format!("public static final int {} = {};", name, val)));
        }
    }
    c.push_str(&helpers::ln(1, &format!("private static final ObjectMapper MAPPER = {}.createMapper();", base)));

    // Constructor — populate _v with defaults for ALL fields.
    // OPTIONAL fields get valid defaults too; encode always serialises whatever is in _v.
    c.push_str(&helpers::ln(1, &format!("public {}() {{", cn)));
    for f in fields {
        let raw_name = f.identifier.as_deref().unwrap_or(&f.name);
        let jt = resolve_wrapper_type(&f.rust_type, all, prefix);
        let dflt = match jt.as_str() {
            "DefaultInnerOctetString" => {
                let is_fixed = f.size_attr_raw.as_deref()
                    .and_then(|r| r.parse::<usize>().ok())
                    .is_some();
                let n = if is_fixed { f.size_from_attr } else { None }
                    .or_else(|| {
                        let sz = helpers::resolve_size(&f.rust_type, asn_defs);
                        if sz > 0 && sz != 2 { Some(sz) } else { None }
                    });
                if let Some(n) = n {
                    let bytes: Vec<String> = std::iter::repeat("1".to_string()).take(n).collect();
                    format!("new DefaultInnerOctetString(new byte[] {{ {} }})", bytes.join(", "))
                } else {
                    "new DefaultInnerOctetString(new byte[]{ 1 })".to_string()
                }
            }
            "DefaultInnerVisibleString" | "DefaultInnerUtf8String" => {
                let is_fixed = f.size_attr_raw.as_deref()
                    .and_then(|r| r.parse::<usize>().ok())
                    .is_some();
                let n = if is_fixed { f.size_from_attr } else { None }
                    .or_else(|| {
                        let sz = helpers::resolve_size(&f.rust_type, asn_defs);
                        if sz > 0 && sz != 2 { Some(sz) } else { None }
                    });
                if let Some(n) = n {
                    format!("new DefaultInnerVisibleString(\"{}\")", "x".repeat(n))
                } else {
                    "new DefaultInnerVisibleString(\"x\")".to_string()
                }
            }
            "Integer" => "1".to_string(),
            "Long" => "1L".to_string(),
            "Boolean" => "true".to_string(),
            "Float" => "1.5f".to_string(),
            "Double" => "2.5".to_string(),
            _ => {
                if jt.starts_with(prefix) {
                    format!("new {}()._v", jt)
                } else if jt.starts_with("java.util.List<") {
                    "new java.util.ArrayList<>()".to_string()
                } else if let Some(ref dv) = f.default_value {
                    helpers::jdefault_with_value(&jt, dv)
                } else {
                    helpers::jdefault(&jt, f.is_list)
                }
            }
        };
        c.push_str(&helpers::ln(2, &format!("_v.put(\"{}\", {});", raw_name, dflt)));
    }
    c.push_str(&helpers::ln(1, "}"));

    // @JsonAnySetter — populate _v during Jackson deserialisation (no _optional tracking)
    c.push_str(&helpers::ln(1, "@JsonAnySetter"));
    c.push_str(&helpers::ln(1, "public void setField(String key, Object value) {"));
    c.push_str(&helpers::ln(2, "if (key.startsWith(\"_\")) return;"));
    c.push_str(&helpers::ln(2, "_v.put(key, value);"));
    c.push_str(&helpers::ln(1, "}"));

    // encode — serialize _v as-is, call Rust library
    c.push_str(&helpers::ln(1, "public byte[] encode() {"));
    c.push_str(&helpers::ln(2, "String _json = null;"));
    c.push_str(&helpers::ln(2, "try {"));
    c.push_str(&helpers::ln(3, &format!("_json = MAPPER.writeValueAsString({}.toJson(_v));", base)));
    c.push_str(&helpers::ln(3, &format!("return {}.encode(\"{}\", DEFAULT_ENCODING, _json);", native, ti.name)));
    c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
    c.push_str(&helpers::ln(3, &format!("throw new RuntimeException(\"encode {} failed, json=\" + _json, e);", ti.name)));
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(1, "}"));

    // encodeTest — same as encode but also prints JSON (for debugging without Rust lib)
    c.push_str(&helpers::ln(1, "public byte[] encodeTest() {"));
    c.push_str(&helpers::ln(2, "String _json = null;"));
    c.push_str(&helpers::ln(2, "try {"));
    c.push_str(&helpers::ln(3, &format!("_json = MAPPER.writeValueAsString({}.toJson(_v));", base)));
    c.push_str(&helpers::ln(3, "System.err.println(\"JSON: \" + _json);"));
    c.push_str(&helpers::ln(3, "return new byte[0];"));
    c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
    c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(1, "}"));

    // decode
    c.push_str(&helpers::ln(1, &format!("public static {} decode(byte[] data) {{", cn)));
    c.push_str(&helpers::ln(2, "try {"));
    c.push_str(&helpers::ln(3, &format!("return MAPPER.readValue({}.decode(\"{}\", DEFAULT_ENCODING, data), {}.class);", native, ti.name, cn)));
    c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
    c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(1, "}"));
    c.push_str("}\n");
    c
}
