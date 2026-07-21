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

    // Helper: set fields
    let set_fields = |c: &mut String, indent: usize| {
        let mut count = 0;
        for f in fields {
            if count >= 3 { break; }
            let fname = safe_field_name(&f.name);
            let jt = if f.optional { resolve_java_type_nullable(&f.rust_type, all, prefix) }
                     else { resolve_java_type(&f.rust_type, all, prefix) };
            match jt.as_str() {
                "int" | "Integer" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 42;", fname))); count += 1; }
                "long" | "Long" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 42L;", fname))); count += 1; }
                "boolean" | "Boolean" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = true;", fname))); count += 1; }
                "float" | "Float" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 1.5f;", fname))); count += 1; }
                "double" | "Double" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = 2.5;", fname))); count += 1; }
                "String" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = \"test\";", fname))); count += 1; }
                "byte[]" => { c.push_str(&helpers::ln(indent, &format!("obj.{} = new byte[0];", fname))); count += 1; }
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

    // testEncodeDecode
    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(1, "public void testEncodeDecode() throws Exception {"));
    c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    set_fields(&mut c, 2);
    c.push_str(&helpers::ln(2, "byte[] data = obj.encode(\"uper\");"));
    c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(\"uper\", data);", cn, cn)));
    c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
    c.push_str(&helpers::ln(1, "}"));
    c
}
