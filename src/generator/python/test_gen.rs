use std::collections::HashMap;
use super::super::*;
use super::helpers;

/// Generate test class for a single type
pub fn gen_test(ti: &TypeInfo, all: &[TypeInfo], prefix: &str, _package: &str,
                asn_defs: &HashMap<String, String>) -> String {
    let cn = format!("{}{}", prefix, ti.name);
    let mut c = String::new();
    c.push_str(&format!("class Test{cn}:\n"));
    c.push_str("    def test_encode_decode(self):\n");
    c.push_str(&format!("        obj = {cn}()\n"));

    /// Extract size digits from a type string like "FixedOctetString < 6usize >"
    fn extract_size(s: &str) -> usize {
        s.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0)
    }

    match &ti.kind {
        TypeKind::Newtype { inner_type } => {
            let py_type = helpers::resolve_py_type(inner_type, all, prefix);
            match py_type.as_str() {
                "int" => c.push_str("        obj.value = 1\n"),
                "str" => {
                    let sz = super::super::java::helpers::resolve_size(&ti.name, asn_defs);
                    if sz > 0 {
                        c.push_str(&format!("        obj.value = \"x\" * {}\n", sz));
                    } else {
                        c.push_str("        obj.value = \"test\"\n");
                    }
                }
                "bytes" => {
                    let sz = extract_size(inner_type);
                    if sz > 0 {
                        c.push_str(&format!("        obj.value = b\"\\x00\" * {}\n", sz));
                    } else {
                        c.push_str("        obj.value = b\"\\x01\"\n");
                    }
                }
                _ => {}
            }
        }
        TypeKind::Struct { fields } => {
            for f in fields {
                let raw = f.identifier.as_deref().unwrap_or(&f.name);
                let name = helpers::py_safe_name(raw);
                let py_type = helpers::resolve_py_type(&f.rust_type, all, prefix);
                match py_type.as_str() {
                    "int" => c.push_str(&format!("        obj.{} = 1\n", name)),
                    "str" => {
                        let sz = extract_size(&f.rust_type);
                        if sz > 0 {
                            c.push_str(&format!("        obj.{} = \"x\" * {}\n", name, sz));
                        } else {
                            c.push_str(&format!("        obj.{} = \"test\"\n", name));
                        }
                    }
                    "bytes" => {
                        let sz = f.size_from_attr.unwrap_or(extract_size(&f.rust_type));
                        if sz > 0 {
                            c.push_str(&format!("        obj.{} = b\"\\x00\" * {}\n", name, sz));
                        } else {
                            c.push_str(&format!("        obj.{} = b\"\\x01\"\n", name));
                        }
                    }
                    "list" => c.push_str(&format!("        obj.{} = []\n", name)),
                    _ => {
                        if all.iter().any(|t| format!("{}{}", prefix, t.name) == py_type) {
                            c.push_str(&format!("        obj.{} = {}()\n", name, py_type));
                        }
                    }
                }
            }
        }
        TypeKind::Choice { .. } => {}
    }

    c.push_str("        data = obj.encode_test()\n");
    c.push_str(&format!("        decoded = {cn}.decode(data)\n"));
    c.push_str("        assert obj == decoded\n");
    c
}
