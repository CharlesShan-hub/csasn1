use super::super::*;
use super::type_map::{resolve_java_type, resolve_java_type_nullable};
use super::helpers;
use super::helpers::{safe_field_name, jdefault};

pub fn generate(_ti: &TypeInfo, all: &[TypeInfo], prefix: &str, cn: &str, fields: &[FieldInfo]) -> String {
    let mut c = String::new();

    // testDefault
    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(1, "public void testDefault() {"));
    c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    for f in fields {
        let fname = safe_field_name(&f.name);
        let jt = if f.optional { resolve_java_type_nullable(&f.rust_type, all, prefix) }
                 else { resolve_java_type(&f.rust_type, all, prefix) };
        let dflt = jdefault(&jt, f.is_list);
        if f.optional { c.push_str(&helpers::ln(2, &format!("assertNull(obj.{});", fname))); }
        else if f.is_list { c.push_str(&helpers::ln(2, &format!("assertNotNull(obj.{});", fname))); }
        else if jt == "byte[]" { c.push_str(&helpers::ln(2, &format!("assertNull(obj.{});", fname))); }
        else if dflt == "0" { c.push_str(&helpers::ln(2, &format!("assertEquals(0, obj.{});", fname))); }
        else if dflt == "0L" { c.push_str(&helpers::ln(2, &format!("assertEquals(0L, obj.{});", fname))); }
        else if dflt == "false" { c.push_str(&helpers::ln(2, &format!("assertFalse(obj.{});", fname))); }
        else if dflt == "0.0f" { c.push_str(&helpers::ln(2, &format!("assertEquals(0.0f, obj.{}, 0.001f);", fname))); }
        else if dflt == "0.0" { c.push_str(&helpers::ln(2, &format!("assertEquals(0.0, obj.{}, 0.001);", fname))); }
        else { c.push_str(&helpers::ln(2, &format!("assertNull(obj.{});", fname))); }
    }
    c.push_str(&helpers::ln(1, "}"));
    c.push('\n');

    // Helper: set fields (all primitive+string fields, and nested objects for required fields)
    let set_fields = |c: &mut String, indent: usize| {
        for f in fields {
            let fname = safe_field_name(&f.name);
            let jt = if f.optional { resolve_java_type_nullable(&f.rust_type, all, prefix) }
                     else { resolve_java_type(&f.rust_type, all, prefix) };
            match jt.as_str() {
                "int" | "Integer" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 1;", fname))); }
                "long" | "Long" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 1L;", fname))); }
                "boolean" | "Boolean" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = true;", fname))); }
                "float" | "Float" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 1.5f;", fname))); }
                "double" | "Double" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 2.5;", fname))); }
                "String" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = \"test\";", fname))); }
                "byte[]" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = new byte[]{{0x01, 0x02}};", fname))); }
                s if s.starts_with("java.util.List<") && !f.optional => {
                    let inner = s.trim_start_matches("java.util.List<").trim_end_matches('>').trim();
                    let item = match inner {
                        "Integer" => "Integer.valueOf(1)".into(),
                        "Long" => "Long.valueOf(1L)".into(),
                        "Boolean" => "Boolean.valueOf(true)".into(),
                        "Float" => "Float.valueOf(1.5f)".into(),
                        "Double" => "Double.valueOf(2.5)".into(),
                        "String" => "\"test\"".into(),
                        _ => format!("new {}()", inner),
                    };
                    c.push_str(&helpers::ln(indent, &format!("obj.{} = java.util.Collections.singletonList({});", fname, item)));
                }
                s if !f.optional && !s.starts_with("java.util.List<") && !s.starts_with("java.util.Map<") => {
                    let init = match s {
                        "Integer" => "1".into(),
                        "Long" => "1L".into(),
                        "Boolean" => "true".into(),
                        "Float" => "1.5f".into(),
                        "Double" => "2.5".into(),
                        _ => format!("new {}()", s),
                    };
                    c.push_str(&helpers::ln(indent, &format!("if (obj.{} == null) obj.{} = {};", fname, fname, init)));
                }
                _ => {}
            }
        }
    };

    // testJsonRoundTrip
    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(1, "public void testJsonRoundTrip() throws Exception {"));
    c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    set_fields(&mut c, 2);
    c.push_str(&helpers::ln(2, "String json = MAPPER.writeValueAsString(obj);"));
    c.push_str(&helpers::ln(2, &format!("{} d = MAPPER.readValue(json, {}.class);", cn, cn)));
    c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
    c.push_str(&helpers::ln(1, "}"));

    // testEncodeDecode — skipped for struct types due to complex nested data generation requirements
     // Simple newtype types are better suited for encode/decode round-trip testing.
    c
}
