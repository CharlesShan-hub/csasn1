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
    variants: &[VariantInfo],
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
    c.push_str(&helpers::ln(1, &format!("private static final ObjectMapper MAPPER = {}.MAPPER;", base)));

    // No-arg constructor picks the first variant as default.
    // All data stored in _v: {"_choice": "variantName", "variantName": value}.
    if let Some(first) = variants.first() {
        let json_key = first.identifier.as_deref().unwrap_or(&first.name);
        let jt = resolve_wrapper_type(&first.inner_type, all, prefix);
        let init = match jt.as_str() {
            "int" => " = 1".to_string(),
            "long" => " = 1L".to_string(),
            "float" => " = 1.5f".to_string(),
            "double" => " = 2.5".to_string(),
            "String" => " = \"x\"".to_string(),
            "byte[]" => " = new byte[1]".to_string(),
            "boolean" => " = true".to_string(),
            _ => format!(" = new {}()", jt),
        };
        // Extract the raw init value (the part after " = ")
        let init_val = init.trim_start_matches(" = ");
        let init_val = if jt.starts_with(prefix) || jt.starts_with("DefaultInner") {
            // For user-defined types and DefaultInner*, use ._v for SEQUENCE/CHOICE
            if jt.starts_with(prefix) && !jt.starts_with("DefaultInner") {
                format!("new {}()._v", jt)
            } else {
                init_val.to_string()
            }
        } else {
            init_val.to_string()
        };
        c.push_str(&helpers::ln(1, &format!("public {}() {{", cn)));
        c.push_str(&helpers::ln(2, &format!("_v.put(\"_choice\", \"{}\");", json_key)));
        c.push_str(&helpers::ln(2, &format!("_v.put(\"{}\", {});", json_key, init_val)));
        c.push_str(&helpers::ln(1, "}"));
    }

    // Typesafe setters — set _choice + variant value in _v
    for v in variants {
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        let setter_name = format!("set{}{}", &v.name[..1].to_uppercase(), &v.name[1..]);
        // Use Object for all setter params — Jackson passes the raw JSON value,
        // and we wrap non-Map values in {"_": v} so _v is always consistent.
        let param_type = "Object".to_string();
        c.push_str(&helpers::ln(1, &format!("@JsonSetter(\"{}\")", json_key)));
        c.push_str(&helpers::ln(1, &format!("public void {}({} v) {{", setter_name, param_type)));
        c.push_str(&helpers::ln(2, &format!("_v.put(\"_choice\", \"{}\");", json_key)));
        c.push_str(&helpers::ln(2, &format!("if (v instanceof java.util.Map) {{ _v.put(\"{}\", v); }} else {{ java.util.LinkedHashMap<String, Object> _w = new java.util.LinkedHashMap<>(); _w.put(\"_\", v); _v.put(\"{}\", _w); }}", json_key, json_key)));
        c.push_str(&helpers::ln(1, "}"));
    }

    // encode + encodeTest
    helpers::gen_encode_methods(&mut c, cn, &native, &ti.name,
                                &format!("MAPPER.writeValueAsString({}.toJson(_v))", base),
                                false, &[]);

    // decode
    helpers::gen_decode_method(&mut c, cn, &native, &ti.name);

    c.push_str("}\n");
    c
}
