use std::collections::HashMap;
use super::super::*;
use super::{resolve_java_type, resolve_java_type_nullable, safe_field_name, jdefault};

pub fn gen_test_class(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    package: &str,
    asn_defs: &HashMap<String, String>,
) -> String {
    let cn = format!("{}{}", prefix, ti.name);
    let pkg = if package.is_empty() {
        String::new()
    } else {
        format!("package {};\n\n", package)
    };
    let mut c = String::new();
    c.push_str(&format!("// Auto-generated. Tests for {}\n\n", cn));
    c.push_str(&pkg);
    c.push_str("import org.junit.Test;\n");
    c.push_str("import com.fasterxml.jackson.databind.ObjectMapper;\n");
    c.push_str("import static org.junit.Assert.*;\n\n");
    c.push_str(&format!("public class {}Test {{\n\n", cn));
    c.push_str(&ln(1, "private static final ObjectMapper MAPPER = new ObjectMapper();"));
    c.push('\n');

    match &ti.kind {
        TypeKind::Newtype { inner_type: _ } => {
            let jt = resolve_java_type(&ti.name, all, prefix);
            let jt = if jt == cn {
                resolve_java_type(
                    match &ti.kind {
                        TypeKind::Newtype { inner_type } => inner_type,
                        _ => unreachable!(),
                    },
                    all,
                    prefix,
                )
            } else {
                jt
            };

            // testDefault
            c.push_str(&ln(1, "@Test"));
            c.push_str(&ln(1, "public void testDefault() {"));
            c.push_str(&ln(2, &format!("{} obj = new {}();", cn, cn)));
            if jt == "int" || jt == "long" || jt == "float" || jt == "double" || jt == "boolean" {
                c.push_str(&ln(2, &format!("assertEquals({}, obj.value);", jdefault(&jt, false))));
            } else {
                c.push_str(&ln(2, "assertNull(obj.value);"));
            }
            c.push_str(&ln(1, "}"));
            c.push('\n');

            // testValueConstructor
            if jt != "byte[]" && !jt.starts_with("java.util.List<") {
                c.push_str(&ln(1, "@Test"));
                c.push_str(&ln(1, "public void testValueConstructor() {"));
                match jt.as_str() {
                    "int" => c.push_str(&ln(2, &format!("{} obj = new {}(42);", cn, cn))),
                    "long" => c.push_str(&ln(2, &format!("{} obj = new {}(42L);", cn, cn))),
                    "boolean" => c.push_str(&ln(2, &format!("{} obj = new {}(true);", cn, cn))),
                    "float" => c.push_str(&ln(2, &format!("{} obj = new {}(1.5f);", cn, cn))),
                    "double" => c.push_str(&ln(2, &format!("{} obj = new {}(2.5);", cn, cn))),
                    "String" => c.push_str(&ln(2, &format!("{} obj = new {}(\"hello\");", cn, cn))),
                    "Object" => c.push_str(&ln(2, &format!("{} obj = new {}(null);", cn, cn))),
                    _ => c.push_str(&ln(2, &format!("{} obj = new {}(null);", cn, cn))),
                }
                if jt != "Object" {
                    c.push_str(&ln(2, "assertNotNull(obj);"));
                }
                c.push_str(&ln(1, "}"));
                c.push('\n');
            }

            // testJsonRoundTrip
            c.push_str(&ln(1, "@Test"));
            c.push_str(&ln(1, "public void testJsonRoundTrip() throws Exception {"));
            if jt == "int" || jt == "long" || jt == "float" || jt == "double" {
                c.push_str(&ln(2, &format!("{} obj = new {}(42);", cn, cn)));
            } else if jt == "boolean" {
                c.push_str(&ln(2, &format!("{} obj = new {}(true);", cn, cn)));
            } else if jt == "String" {
                c.push_str(&ln(2, &format!("{} obj = new {}(\"test\");", cn, cn)));
                c.push_str(&ln(2, "obj.value = \"test\";"));
            } else if jt == "byte[]" {
                c.push_str(&ln(2, &format!("{} obj = new {}();", cn, cn)));
                c.push_str(&ln(2, "obj.value = new byte[]{0x01, 0x02};"));
            } else {
                c.push_str(&ln(2, &format!("{} obj = new {}();", cn, cn)));
            }
            c.push_str(&ln(2, &format!("String json = MAPPER.writeValueAsString(obj);")));
            c.push_str(&ln(2, &format!("{} d = MAPPER.readValue(json, {}.class);", cn, cn)));
            c.push_str(&ln(2, "assertEquals(obj, d);"));
            c.push_str(&ln(1, "}"));
        }

        TypeKind::Struct { fields } => {
            // testDefault
            c.push_str(&ln(1, "@Test"));
            c.push_str(&ln(1, "public void testDefault() {"));
            c.push_str(&ln(2, &format!("{} obj = new {}();", cn, cn)));
            for f in fields {
                let fname = safe_field_name(&f.name);
                let jt = if f.optional {
                    resolve_java_type_nullable(&f.rust_type, all, prefix)
                } else {
                    resolve_java_type(&f.rust_type, all, prefix)
                };
                let dflt = jdefault(&jt, f.is_list);
                if f.optional {
                    c.push_str(&ln(2, &format!("assertNull(obj.{});", fname)));
                } else if f.is_list {
                    c.push_str(&ln(2, &format!("assertNotNull(obj.{});", fname)));
                } else if jt == "byte[]" {
                    c.push_str(&ln(2, &format!("assertNull(obj.{});", fname)));
                } else if dflt == "0" {
                    c.push_str(&ln(2, &format!("assertEquals(0, obj.{});", fname)));
                } else if dflt == "0L" {
                    c.push_str(&ln(2, &format!("assertEquals(0L, obj.{});", fname)));
                } else if dflt == "false" {
                    c.push_str(&ln(2, &format!("assertFalse(obj.{});", fname)));
                } else if dflt == "0.0f" {
                    c.push_str(&ln(2, &format!("assertEquals(0.0f, obj.{}, 0.001f);", fname)));
                } else if dflt == "0.0" {
                    c.push_str(&ln(2, &format!("assertEquals(0.0, obj.{}, 0.001);", fname)));
                } else {
                    c.push_str(&ln(2, &format!("assertNull(obj.{});", fname)));
                }
            }
            c.push_str(&ln(1, "}"));
            c.push('\n');

            // testJsonRoundTrip
            c.push_str(&ln(1, "@Test"));
            c.push_str(&ln(1, "public void testJsonRoundTrip() throws Exception {"));
            c.push_str(&ln(2, &format!("{} obj = new {}();", cn, cn)));
            // Set first few simple fields to non-default values
            let mut set_count = 0;
            for f in fields {
                if set_count >= 3 {
                    break;
                }
                let fname = safe_field_name(&f.name);
                let jt = if f.optional {
                    resolve_java_type_nullable(&f.rust_type, all, prefix)
                } else {
                    resolve_java_type(&f.rust_type, all, prefix)
                };
                match jt.as_str() {
                    "int" | "Integer" => {
                        c.push_str(&ln(2, &format!("obj.{} = 42;", fname)));
                        set_count += 1;
                    }
                    "long" | "Long" => {
                        c.push_str(&ln(2, &format!("obj.{} = 42L;", fname)));
                        set_count += 1;
                    }
                    "boolean" | "Boolean" => {
                        c.push_str(&ln(2, &format!("obj.{} = true;", fname)));
                        set_count += 1;
                    }
                    "float" | "Float" => {
                        c.push_str(&ln(2, &format!("obj.{} = 1.5f;", fname)));
                        set_count += 1;
                    }
                    "double" | "Double" => {
                        c.push_str(&ln(2, &format!("obj.{} = 2.5;", fname)));
                        set_count += 1;
                    }
                    "String" => {
                        c.push_str(&ln(2, &format!("obj.{} = \"test\";", fname)));
                        set_count += 1;
                    }
                    "byte[]" => {
                        c.push_str(&ln(2, &format!("obj.{} = new byte[0];", fname)));
                        set_count += 1;
                    }
                    _ => {}
                }
            }
            c.push_str(&ln(2, "String json = MAPPER.writeValueAsString(obj);"));
            c.push_str(&ln(2, &format!("{} d = MAPPER.readValue(json, {}.class);", cn, cn)));
            c.push_str(&ln(2, "assertEquals(obj, d);"));
            c.push_str(&ln(1, "}"));
        }

        TypeKind::Choice { variants } => {
            // Generate test for up to 2 variants
            let test_variants: Vec<_> = variants.iter().take(2).collect();
            for (i, v) in test_variants.iter().enumerate() {
                let fname = safe_field_name(&v.name);
                let test_name = if i == 0 {
                    format!("testChoice{}", v.name)
                } else {
                    format!("testChoice{}", v.name)
                };
                c.push_str(&ln(1, "@Test"));
                c.push_str(&ln(1, &format!("public void {}() throws Exception {{", test_name)));
                c.push_str(&ln(2, &format!("{} obj = new {}();", cn, cn)));
                c.push_str(&ln(2, &format!("obj._choice = \"{}\";", v.name)));
                let jt = resolve_java_type(&v.inner_type, all, prefix);
                match jt.as_str() {
                    "int" => c.push_str(&ln(2, &format!("obj.{} = 42;", fname))),
                    "long" => c.push_str(&ln(2, &format!("obj.{} = 42L;", fname))),
                    "boolean" => c.push_str(&ln(2, &format!("obj.{} = true;", fname))),
                    "float" => c.push_str(&ln(2, &format!("obj.{} = 1.5f;", fname))),
                    "double" => c.push_str(&ln(2, &format!("obj.{} = 2.5;", fname))),
                    "String" => c.push_str(&ln(2, &format!("obj.{} = \"test\";", fname))),
                    "byte[]" => c.push_str(&ln(2, &format!("obj.{} = new byte[0];", fname))),
                    s if s.starts_with("java.util.List<") => {}
                    _ => {
                        // reference type — try to create an instance via default constructor
                        c.push_str(&ln(2, &format!("obj.{} = new {}();", fname, jt)));
                    }
                }
                c.push_str(&ln(2, "String json = MAPPER.writeValueAsString(obj);"));
                c.push_str(&ln(2, &format!("{} d = MAPPER.readValue(json, {}.class);", cn, cn)));
                c.push_str(&ln(2, "assertEquals(obj, d);"));
                c.push_str(&ln(1, "}"));
                c.push('\n');
            }
        }
    }

    c.push_str("}\n");
    c
}
