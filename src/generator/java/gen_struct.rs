use std::collections::HashMap;
use super::super::*;
use super::type_map::{resolve_wrapper_type, resolve_wrapper_type_nullable};
use super::helpers;
use super::helpers::{safe_field_name, jdefault};

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
    let _has_optional = fields.iter().any(|f| f.optional);
    if let Some(doc) = asn_doc { c.push_str(doc); }
    c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
    c.push_str("@Data\n");
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(1, &format!("public static final int {} = {};", name, val)));
        }
    }
    c.push_str(&helpers::ln(1, &format!("private static final ObjectMapper MAPPER = {}.createMapper();", base)));

    for f in fields {
        let jt = if f.optional {
            resolve_wrapper_type_nullable(&f.rust_type, all, prefix)
        } else {
            resolve_wrapper_type(&f.rust_type, all, prefix)
        };
        // Use ASN.1 identifier as Java field name if available, otherwise Rust field name
        let raw_name = f.identifier.as_deref().unwrap_or(&f.name);
        let fname = safe_field_name(raw_name);
        let dflt = match jt.as_str() {
            "byte[]" => {
                // Only use non-empty default for truly fixed sizes, not ranges
                let is_fixed = f.size_attr_raw.as_deref()
                    .and_then(|r| r.parse::<usize>().ok())
                    .is_some();
                if is_fixed {
                    if let Some(n) = f.size_from_attr {
                        format!("new byte[{}]", n)
                    } else {
                        "new byte[0]".to_string()
                    }
                } else {
                    "new byte[0]".to_string()
                }
            }
            "String" => {
                let is_fixed = f.size_attr_raw.as_deref()
                    .and_then(|r| r.parse::<usize>().ok())
                    .is_some();
                if is_fixed {
                    if let Some(n) = f.size_from_attr {
                        format!("\"{}\"", "x".repeat(n))
                    } else {
                        "\"\"".to_string()
                    }
                } else {
                    "\"\"".to_string()
                }
            }
            _ => jdefault(&jt, f.is_list),
        };
        if fname != raw_name {
            // Java keyword escaped → need @JsonProperty to keep original name
            c.push_str(&helpers::ln(1, &format!("@JsonProperty(\"{}\") public {} {} = {};", raw_name, jt, fname, dflt)));
        } else {
            c.push_str(&helpers::ln(1, &format!("public {} {} = {};", jt, fname, dflt)));
        }
    }

    // encode
    helpers::enc_overload(&mut c, &format!(
        "{}{}{}{}{}{}{}",
        helpers::ln(2, "try {"),
        helpers::ln(3, "String _json = MAPPER.writeValueAsString(this);"),
        helpers::ln(3, "System.err.println(\"JSON[\" + getClass().getSimpleName() + \"]: \" + _json);"),
        helpers::ln(3, &format!("return {}.encode(\"{}\", enc, _json);", native, ti.name)),
        helpers::ln(2, "} catch (Exception e) {"),
        helpers::ln(3, "throw new RuntimeException(e);"),
        helpers::ln(2, "}"),
    ));

    // decode
    c.push_str(&helpers::ln(1, &format!("public static {} decode(String enc, byte[] data) {{", cn)));
    c.push_str(&helpers::ln(2, "try {"));
    c.push_str(&helpers::ln(3, &format!("return MAPPER.readValue({}.decode(\"{}\", enc, data), {}.class);", native, ti.name, cn)));
    c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
    c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(1, "}"));
    c.push_str("}\n");
    c
}
