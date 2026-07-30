use std::collections::HashMap;
use super::super::*;
use super::type_map::resolve_wrapper_type;
use super::helpers;

pub fn generate(_ti: &TypeInfo, all: &[TypeInfo], prefix: &str, cn: &str, variants: &[VariantInfo], _asn_defs: &HashMap<String, String>) -> String {
    let mut c = String::new();

    // Generate a CHOICE roundtrip test for EACH variant
    for v in variants {
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);

        c.push_str(&helpers::ln(1, "@Test"));
        c.push_str(&helpers::ln(1, &format!("public void testAlternative{}() throws Exception {{", v.name)));
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, "obj._v.clear();"));
        c.push_str(&helpers::ln(2, &format!("obj._v.put(\"_choice\", \"{}\");", json_key)));

        // Handle NULL types separately — no roundtrip decode/assert
        if jt == "Object" {
            c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", null);", json_key)));
            c.push_str(&helpers::ln(2, "// ASN.1 NULL type: just verify encode doesn't crash"));
            c.push_str(&helpers::ln(2, "obj.encodeTest();"));
            c.push_str(&helpers::ln(1, "}"));
            c.push('\n');
            continue;
        }

        match jt.as_str() {
            "int" | "Integer" => {
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", 42);", json_key)));
            }
            "long" | "Long" => {
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", 42L);", json_key)));
            }
            "boolean" | "Boolean" => {
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", true);", json_key)));
            }
            "float" | "Float" => {
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", 1.5f);", json_key)));
            }
            "double" | "Double" => {
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", 2.5);", json_key)));
            }
            "String" => {
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", \"test-value\");", json_key)));
            }
            "byte[]" => {
                // BIT STRING in JER: {"length": N, "value": "HEX"}
                c.push_str(&helpers::ln(2, &format!("java.util.LinkedHashMap<String, Object> _bs = new java.util.LinkedHashMap<>();")));
                c.push_str(&helpers::ln(2, "_bs.put(\"length\", 2);"));
                c.push_str(&helpers::ln(2, &format!("_bs.put(\"value\", \"AA\");")));
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", _bs);", json_key)));
            }
            s if s.starts_with("java.util.List<") => {
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", new java.util.ArrayList<>());", json_key)));
            }
            s if s.starts_with(prefix) && !s.starts_with("DefaultInner") => {
                // User-defined Inner* type: store its _v map
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", new {}()._v);", json_key, s)));
            }
            _ => {
                // DefaultInner* or other — create instance directly
                c.push_str(&helpers::ln(2, &format!("obj._v.put(\"{}\", new {}());", json_key, jt)));
            }
        }

        c.push_str(&helpers::ln(2, "byte[] data = obj.encodeTest();"));
        c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(data);", cn, cn)));
        c.push_str(&helpers::ln(2, "assertEquals(obj._v.get(\"_choice\"), d._v.get(\"_choice\"));"));
        c.push_str(&helpers::ln(1, "}"));
        c.push('\n');
    }

    c
}
