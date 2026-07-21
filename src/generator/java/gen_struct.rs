use std::collections::HashMap;
use super::super::*;
use super::type_map::{resolve_java_type, resolve_java_type_nullable};
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
) -> String {
    let mut c = String::new();
    let has_optional = fields.iter().any(|f| f.optional);
    if let Some(doc) = asn_doc { c.push_str(doc); }
    c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
    if has_optional {
        c.push_str("@JsonInclude(JsonInclude.Include.NON_NULL)\n");
    }
    c.push_str("@Data\n");
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(1, &format!("public static final int {} = {};", name, val)));
        }
    }
    c.push_str(&helpers::ln(1, "private static final ObjectMapper MAPPER = new ObjectMapper();"));

    for f in fields {
        let jt = if f.optional {
            resolve_java_type_nullable(&f.rust_type, all, prefix)
        } else {
            resolve_java_type(&f.rust_type, all, prefix)
        };
        let fname = safe_field_name(&f.name);
        let dflt = jdefault(&jt, f.is_list);
        c.push_str(&helpers::ln(1, &format!("@JsonProperty public {} {} = {};", jt, fname, dflt)));
    }

    // encode
    helpers::enc_overload(&mut c, &format!(
        "{}{}{}{}{}{}",
        helpers::ln(2, "try {"),
        helpers::ln(3, &format!("return CmsNative.encode(\"{}\", enc,", ti.name)),
        helpers::ln(4, "MAPPER.writeValueAsString(this));"),
        helpers::ln(2, "} catch (Exception e) {"),
        helpers::ln(3, "throw new RuntimeException(e);"),
        helpers::ln(2, "}"),
    ));

    // decode
    c.push_str(&helpers::ln(1, &format!("public static {} decode(String enc, byte[] data) {{", cn)));
    c.push_str(&helpers::ln(2, "try {"));
    c.push_str(&helpers::ln(3, &format!("return MAPPER.readValue(CmsNative.decode(\"{}\", enc, data), {}.class);", ti.name, cn)));
    c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
    c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(1, "}"));
    c.push_str("}\n");
    c
}
