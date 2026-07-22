use std::collections::HashMap;
use super::super::*;
use super::type_map::resolve_wrapper_type;
use super::helpers;
use super::helpers::safe_field_name;

pub fn generate(_ti: &TypeInfo, all: &[TypeInfo], prefix: &str, cn: &str, variants: &[VariantInfo], asn_defs: &HashMap<String, String>) -> String {
    let mut c = String::new();
    let test_variants: Vec<_> = variants.iter().take(2).collect();

    for v in &test_variants {
        let fname = safe_field_name(&v.name);
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        c.push_str(&helpers::ln(1, "@Test"));
        c.push_str(&helpers::ln(1, &format!("public void testEncodeDecodeAper{}() throws Exception {{", v.name)));
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, &format!("obj._choice = \"{}\";", json_key)));
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);
        match jt.as_str() {
            "int" => c.push_str(&helpers::ln(2, &format!("obj.{} = 42;", fname))),
            "long" => c.push_str(&helpers::ln(2, &format!("obj.{} = 42L;", fname))),
            "boolean" => c.push_str(&helpers::ln(2, &format!("obj.{} = true;", fname))),
            "float" => c.push_str(&helpers::ln(2, &format!("obj.{} = 1.5f;", fname))),
            "double" => c.push_str(&helpers::ln(2, &format!("obj.{} = 2.5;", fname))),
            "String" => c.push_str(&helpers::ln(2, &format!("obj.{} = \"test\";", fname))),
            "byte[]" => c.push_str(&helpers::ln(2, &format!("obj.{} = new byte[0];", fname))),
            s if s.starts_with("java.util.List<") => {}
            _ => {
                c.push_str(&helpers::ln(2, &format!("obj.{} = new {}();", fname, jt)));
                // Initialize nested struct fields
                if let Some(field_ti) = all.iter().find(|ti| format!("{}{}", prefix, ti.name) == jt) {
                    if let TypeKind::Struct { fields: sub_fields } = &field_ti.kind {
                        for sf in sub_fields {
                            let s_jt = resolve_wrapper_type(&sf.rust_type, all, prefix);
                            let sfname = safe_field_name(sf.identifier.as_deref().unwrap_or(&sf.name));
                            match s_jt.as_str() {
                                "byte[]" => {
                                    let sz = helpers::resolve_size(&sf.rust_type, asn_defs);
                                    if sz > 1 {
                                        c.push_str(&helpers::ln(2, &format!("obj.{}.{} = new byte[{}];", fname, sfname, sz)));
                                    }
                                }
                                "String" => {
                                    let sz = helpers::test_data_size(asn_defs.get(&sf.rust_type).map(|s| s.as_str()));
                                    if sz > 1 {
                                        c.push_str(&helpers::ln(2, &format!("obj.{}.{} = \"{}\";", fname, sfname, "x".repeat(sz))));
                                    }
                                }
                                _ => {
                                    if let Some(sf_ti) = all.iter().find(|ti| format!("{}{}", prefix, ti.name) == s_jt) {
                                        let ultimate = super::type_map::resolve_java_type(&sf_ti.name, all, prefix);
                                        if ultimate == "byte[]" || ultimate == "String" {
                                            let sz = helpers::resolve_size(&sf_ti.name, asn_defs);
                                            if sz > 1 {
                                                if ultimate == "byte[]" {
                                                    c.push_str(&helpers::ln(2, &format!("obj.{}.{}.value = new byte[{}];", fname, sfname, sz)));
                                                } else {
                                                    c.push_str(&helpers::ln(2, &format!("obj.{}.{}.value = \"{}\";", fname, sfname, "x".repeat(sz))));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        c.push_str(&helpers::ln(2, "byte[] data = obj.encode(\"aper\");"));
        c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(\"aper\", data);", cn, cn)));
        c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
        c.push_str(&helpers::ln(1, "}"));
        c.push('\n');
    }
    c
}
