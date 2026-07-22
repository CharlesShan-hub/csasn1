use std::collections::HashMap;
use super::super::*;
use super::type_map::{resolve_wrapper_type, resolve_java_type};
use super::helpers;
use super::helpers::safe_field_name;

/// Find a TypeInfo by matching the full Java type name (with prefix).
fn find_type<'a>(jt: &str, all: &'a [TypeInfo], prefix: &str) -> Option<&'a TypeInfo> {
    let jt_stripped = jt.strip_prefix(prefix).unwrap_or(jt);
    all.iter().find(|ti| {
        let anon = format!("Anonymous{}", ti.name);
        jt_stripped == ti.name.as_str() || jt_stripped == anon.as_str()
    })
}

pub fn generate(_ti: &TypeInfo, all: &[TypeInfo], prefix: &str, cn: &str, fields: &[FieldInfo], asn_defs: &HashMap<String, String>) -> String {
    let mut c = String::new();

    // Helper: set all fields to initial values (no nulls left)
    let set_fields = |c: &mut String, indent: usize| {
        for f in fields {
            let raw_name = f.identifier.as_deref().unwrap_or(&f.name);
            let fname = safe_field_name(raw_name);
            let jt = resolve_wrapper_type(&f.rust_type, all, prefix);
            match jt.as_str() {
                "int" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 1;", fname))); }
                "long" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 1L;", fname))); }
                "boolean" => {
                    // Dead code: all booleans are now CmsBoolean (INTEGER 0..1) wrapper
                }
                "float" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 1.5f;", fname))); }
                "double" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 2.5;", fname))); }
                "String" => {
                    let sz = helpers::test_data_size(asn_defs.get(&f.rust_type).map(|s| s.as_str()));
                    c.push_str(&helpers::ln(indent, &format!("obj.{} = \"{}\";", fname, "x".repeat(sz))));
                }
                s if s == "byte[]" => {
                    let sz = helpers::test_data_size(asn_defs.get(&f.rust_type).map(|s| s.as_str()));
                    // Fallback: extract size from inline FixedOctetString < N > (e.g. "FixedOctetString < 6 >")
                    let sz = if sz <= 2 && f.rust_type.contains("FixedOctetString") {
                        f.rust_type
                            .split(|c: char| !c.is_ascii_digit())
                            .filter_map(|s| s.parse::<usize>().ok())
                            .next()
                            .unwrap_or(sz)
                    } else { sz };
                    // Fallback: use size from rasn attribute (e.g. OctetString with #[rasn(size("6"))])
                    let sz = if sz <= 2 { f.size_from_attr.unwrap_or(sz) } else { sz };
                    c.push_str(&helpers::ln(indent, &format!("obj.{} = new byte[{}];", fname, sz)));
                }
                s if s.starts_with("java.util.List<") => {
                    let inner = s.trim_start_matches("java.util.List<").trim_end_matches('>').trim();
                    c.push_str(&helpers::ln(indent, &format!("obj.{} = java.util.Collections.singletonList(new {}());", fname, inner)));
                }
                s if s.starts_with("java.util.Map<") => {
                    c.push_str(&helpers::ln(indent, &format!("obj.{} = new java.util.HashMap<>();", fname)));
                }
                _ => {
                    c.push_str(&helpers::ln(indent, &format!("obj.{} = new {}();", fname, jt)));
                    // If it's a CHOICE type, set _choice to first variant
                    if let Some(ti) = find_type(&jt, all, prefix) {
                        if let TypeKind::Choice { variants } = &ti.kind {
                            if let Some(first) = variants.first() {
                                let json_key = first.identifier.as_deref().unwrap_or(&first.name);
                                let vfname = safe_field_name(&first.name);
                                let vjt = resolve_wrapper_type(&first.inner_type, all, prefix);
                                c.push_str(&helpers::ln(indent, &format!("obj.{}._choice = \"{}\";", fname, json_key)));
                                match vjt.as_str() {
                                    "int" => c.push_str(&helpers::ln(indent, &format!("obj.{}.{} = 1;", fname, vfname))),
                                    "long" => c.push_str(&helpers::ln(indent, &format!("obj.{}.{} = 1L;", fname, vfname))),
                                    // "boolean" is dead code, all booleans use CmsBoolean now
                                    "float" => c.push_str(&helpers::ln(indent, &format!("obj.{}.{} = 1.5f;", fname, vfname))),
                                    "double" => c.push_str(&helpers::ln(indent, &format!("obj.{}.{} = 2.5;", fname, vfname))),
                                    "String" => c.push_str(&helpers::ln(indent, &format!("obj.{}.{} = \"test\";", fname, vfname))),
                                    "byte[]" => c.push_str(&helpers::ln(indent, &format!("obj.{}.{} = new byte[0];", fname, vfname))),
                                    _ => c.push_str(&helpers::ln(indent, &format!("obj.{}.{} = new {}();", fname, vfname, vjt))),
                                }
                            }
                        }
                    }
                    // For byte[]/String wrappers (e.g. TimeStamp), set value to the expected size
                    if let Some(ti) = find_type(&jt, all, prefix) {
                        let ultimate_type = resolve_java_type(&ti.name, all, prefix);
                        if ultimate_type == "byte[]" || ultimate_type == "String" {
                            let sz = helpers::resolve_size(&ti.name, asn_defs);
                            if sz > 1 {
                                if ultimate_type == "byte[]" {
                                    c.push_str(&helpers::ln(indent, &format!("obj.{}.value = new byte[{}];", fname, sz)));
                                } else {
                                    c.push_str(&helpers::ln(indent, &format!("obj.{}.value = \"{}\";", fname, "x".repeat(sz))));
                                }
                            }
                        }
                    }
                    // For List wrappers (e.g. SEQUENCE OF), set value to non-null list
                    if let Some(ti) = find_type(&jt, all, prefix) {
                        if let TypeKind::Newtype { inner_type } = &ti.kind {
                            let inner_jt = resolve_wrapper_type(inner_type, all, prefix);
                            if inner_jt.starts_with("java.util.List<") {
                                let inner = inner_jt.trim_start_matches("java.util.List<").trim_end_matches('>').trim();
                                c.push_str(&helpers::ln(indent, &format!("obj.{}.value = java.util.Collections.singletonList(new {}());", fname, inner)));
                            }
                        }
                    }
                    // For struct sub-objects, initialize byte[] fields with size constraints
                    if let Some(ti) = find_type(&jt, all, prefix) {
                        if let TypeKind::Struct { fields: sub_fields } = &ti.kind {
                            for sf in sub_fields {
                                let s_jt = resolve_wrapper_type(&sf.rust_type, all, prefix);
                                if s_jt == "byte[]" {
                                    if let Some(sz) = sf.size_from_attr {
                                        let sfname = safe_field_name(sf.identifier.as_deref().unwrap_or(&sf.name));
                                        c.push_str(&helpers::ln(indent, &format!("obj.{}.{} = new byte[{}];", fname, sfname, sz)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(1, "public void testEncodeDecodeAper() throws Exception {"));
    c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    set_fields(&mut c, 2);
    c.push_str(&helpers::ln(2, "byte[] data = obj.encode(\"aper\");"));
    c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(\"aper\", data);", cn, cn)));
    c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
    c.push_str(&helpers::ln(1, "}"));
    c
}
