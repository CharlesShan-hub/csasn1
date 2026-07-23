use std::collections::HashMap;
use super::super::*;
use super::type_map::{resolve_wrapper_type, java_type_ref};
use super::helpers;
use super::helpers::safe_field_name;

pub fn generate(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    cn: &str,
    asn_doc: &Option<String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
    variants: &[VariantInfo],
) -> String {
    let mut c = String::new();
    let base = format!("{}Base", prefix);
    let native = format!("{}Native", prefix);
    if let Some(doc) = asn_doc { c.push_str(doc); }
    c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
    c.push_str("@Data\n");
    c.push_str("@lombok.experimental.Accessors(chain = true, fluent = true)\n");
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(1, &format!("public static final int {} = {};", name, val)));
        }
    }
    c.push_str(&helpers::ln(1, "@JsonIgnore public String _choice;"));
    c.push_str(&helpers::ln(1, &format!("private static final ObjectMapper MAPPER = {}.createMapper();", base)));

    // No-arg constructor picks the first variant as default
    if let Some(first) = variants.first() {
        let fname = safe_field_name(&first.name);
        let json_key = first.identifier.as_deref().unwrap_or(&first.name);
        let jt = resolve_wrapper_type(&first.inner_type, all, prefix);
        let init = match jt.as_str() {
            "int" => " = 1".to_string(),
            "long" => " = 1L".to_string(),
            "float" => " = 1.5f".to_string(),
            "double" => " = 2.5".to_string(),
            "String" => " = \"\"".to_string(),
            "byte[]" => " = new byte[0]".to_string(),
            _ => format!(" = new {}()", jt),
        };
        c.push_str(&helpers::ln(1, &format!("public {}() {{", cn)));
        c.push_str(&helpers::ln(2, &format!("this._choice = \"{}\";", json_key)));
        c.push_str(&helpers::ln(2, &format!("this.{}{};", fname, init)));
        c.push_str(&helpers::ln(1, "}"));
    }

    for v in variants {
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);
        let fname = safe_field_name(&v.name);
        c.push_str(&helpers::ln(1, &format!("@JsonIgnore public {} {};", jt, fname)));
    }

    // Serialize (only output the active branch)
    c.push_str(&helpers::ln(1, "@JsonAnyGetter"));
    c.push_str(&helpers::ln(1, "public java.util.Map<String, Object> serializeChoice() {"));
    c.push_str(&helpers::ln(2, "java.util.Map<String, Object> map = new java.util.HashMap<String, Object>();"));
    c.push_str(&helpers::ln(2, "if (_choice != null) {"));
    for v in variants {
        let fname = safe_field_name(&v.name);
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        c.push_str(&helpers::ln(3, &format!("if (\"{}\".equals(_choice)) map.put(\"{}\", {});", json_key, json_key, fname)));
    }
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(2, "return map;"));
    c.push_str(&helpers::ln(1, "}"));

    // Deserialize
    c.push_str(&helpers::ln(1, "@JsonAnySetter"));
    c.push_str(&helpers::ln(1, "public void deserializeChoice(String key, Object value) {"));
    c.push_str(&helpers::ln(2, "if (\"_choice\".equals(key)) return;"));
    c.push_str(&helpers::ln(2, "this._choice = key;"));
    for v in variants {
        let fname = safe_field_name(&v.name);
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);
        let tref = java_type_ref(&jt);
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        c.push_str(&helpers::ln(2, &format!("if (\"{}\".equals(key)) {{", json_key)));
        c.push_str(&helpers::ln(3, &format!("this.{} = MAPPER.convertValue(value, {});", fname, tref)));
        c.push_str(&helpers::ln(2, "}"));
    }
    c.push_str(&helpers::ln(1, "}"));

    // encode + encodeTest
    helpers::gen_encode_methods(&mut c, cn, &native, &ti.name, "MAPPER.writeValueAsString(this)",
                                false, &[]);

    // decode
    helpers::gen_decode_method(&mut c, cn, &native, &ti.name);
    c.push_str("}\n");
    c
}
