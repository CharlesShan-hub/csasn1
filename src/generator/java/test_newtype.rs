use super::super::*;
use super::type_map::resolve_java_type;
use super::helpers;
use super::helpers::jdefault;

pub fn generate(ti: &TypeInfo, all: &[TypeInfo], prefix: &str, cn: &str) -> String {
    let jt = resolve_java_type(&ti.name, all, prefix);
    let jt = if jt == cn {
        resolve_java_type(
            match &ti.kind { TypeKind::Newtype { inner_type } => inner_type, _ => unreachable!() },
            all, prefix,
        )
    } else { jt };

    let mut c = String::new();

    // testDefault
    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(1, "public void testDefault() {"));
    c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    if jt == "int" || jt == "long" || jt == "float" || jt == "double" || jt == "boolean" {
        c.push_str(&helpers::ln(2, &format!("assertEquals({}, obj.value);", jdefault(&jt, false))));
    } else {
        c.push_str(&helpers::ln(2, "assertNull(obj.value);"));
    }
    c.push_str(&helpers::ln(1, "}"));
    c.push('\n');

    // testValueConstructor
    if jt != "byte[]" && !jt.starts_with("java.util.List<") {
        c.push_str(&helpers::ln(1, "@Test"));
        c.push_str(&helpers::ln(1, "public void testValueConstructor() {"));
        match jt.as_str() {
            "int" => c.push_str(&helpers::ln(2, &format!("{} obj = new {}(42);", cn, cn))),
            "long" => c.push_str(&helpers::ln(2, &format!("{} obj = new {}(42L);", cn, cn))),
            "boolean" => c.push_str(&helpers::ln(2, &format!("{} obj = new {}(true);", cn, cn))),
            "float" => c.push_str(&helpers::ln(2, &format!("{} obj = new {}(1.5f);", cn, cn))),
            "double" => c.push_str(&helpers::ln(2, &format!("{} obj = new {}(2.5);", cn, cn))),
            "String" => c.push_str(&helpers::ln(2, &format!("{} obj = new {}(\"hello\");", cn, cn))),
            "Object" => c.push_str(&helpers::ln(2, &format!("{} obj = new {}(null);", cn, cn))),
            _ => c.push_str(&helpers::ln(2, &format!("{} obj = new {}(null);", cn, cn))),
        }
        if jt != "Object" { c.push_str(&helpers::ln(2, "assertNotNull(obj);")); }
        c.push_str(&helpers::ln(1, "}"));
        c.push('\n');
    }

    // testJsonRoundTrip
    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(1, "public void testJsonRoundTrip() throws Exception {"));
    if jt == "int" || jt == "long" || jt == "float" || jt == "double" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(42);", cn, cn)));
    } else if jt == "boolean" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(true);", cn, cn)));
    } else if jt == "String" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(\"test\");", cn, cn)));
        c.push_str(&helpers::ln(2, "obj.value = \"test\";"));
    } else if jt == "byte[]" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, "obj.value = new byte[]{0x01, 0x02};"));
    } else {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    }
    c.push_str(&helpers::ln(2, &format!("String json = MAPPER.writeValueAsString(obj);")));
    c.push_str(&helpers::ln(2, &format!("{} d = MAPPER.readValue(json, {}.class);", cn, cn)));
    c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
    c.push_str(&helpers::ln(1, "}"));

    // testEncodeDecode
    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(1, "public void testEncodeDecode() throws Exception {"));
    if jt == "int" || jt == "long" || jt == "float" || jt == "double" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(42);", cn, cn)));
    } else if jt == "boolean" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(true);", cn, cn)));
    } else if jt == "String" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, "obj.value = \"test\";"));
    } else if jt == "byte[]" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, "obj.value = new byte[]{0x01, 0x02};"));
    } else {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    }
    c.push_str(&helpers::ln(2, "byte[] data = obj.encode(\"uper\");"));
    c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(\"uper\", data);", cn, cn)));
    c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
    c.push_str(&helpers::ln(1, "}"));
    c
}
