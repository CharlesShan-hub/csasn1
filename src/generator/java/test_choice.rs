use std::collections::HashMap;
use super::super::*;
use super::type_map::resolve_wrapper_type;
use super::helpers;
use super::helpers::safe_field_name;

pub fn generate(ti: &TypeInfo, all: &[TypeInfo], prefix: &str, cn: &str, variants: &[VariantInfo], _asn_defs: &HashMap<String, String>) -> String {
    let mut c = String::new();

    // Generate a CHOICE roundtrip test for EACH variant
    for v in variants {
        let fname = safe_field_name(&v.name);
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);

        c.push_str(&helpers::ln(1, "@Test"));
        c.push_str(&helpers::ln(1, &format!("public void testAlternative{}() throws Exception {{", v.name)));
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, &format!("obj._choice = \"{}\";", json_key)));

        match jt.as_str() {
            "int" => {
                c.push_str(&helpers::ln(2, &format!("obj.{} = 42;", fname)));
            }
            "long" => {
                c.push_str(&helpers::ln(2, &format!("obj.{} = 42L;", fname)));
            }
            "boolean" => {
                c.push_str(&helpers::ln(2, &format!("obj.{} = true;", fname)));
            }
            "float" => {
                c.push_str(&helpers::ln(2, &format!("obj.{} = 1.5f;", fname)));
            }
            "double" => {
                c.push_str(&helpers::ln(2, &format!("obj.{} = 2.5;", fname)));
            }
            "String" => {
                c.push_str(&helpers::ln(2, &format!("obj.{} = \"test-value\";", fname)));
            }
            "byte[]" => {
                c.push_str(&helpers::ln(2, &format!("obj.{} = new byte[]{{(byte)0xAA, (byte)0xBB}};", fname)));
            }
            s if s.starts_with("java.util.List<") => {
                c.push_str(&helpers::ln(2, &format!("obj.{} = new java.util.ArrayList<>();", fname)));
            }
            "Object" => {
                // ASN.1 ANY type — skip CHOICE roundtrip, just verify no crash
                c.push_str(&helpers::ln(2, "// Object (ANY) — skip; no meaningful value"));
            }
            _ => {
                // User-defined type (e.g. InnerServiceError, InnerQuality) — create instance
                c.push_str(&helpers::ln(2, &format!("obj.{} = new {}();", fname, jt)));
            }
        }

        c.push_str(&helpers::ln(2, "byte[] data = obj.encodeTest();"));
        c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(data);", cn, cn)));
        c.push_str(&helpers::ln(2, "assertEquals(obj._choice, d._choice);"));
        c.push_str(&helpers::ln(1, "}"));
        c.push('\n');
    }

    c
}
