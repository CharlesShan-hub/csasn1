use std::collections::HashMap;
use super::super::*;
use super::type_map::resolve_java_type;
use super::helpers;

pub fn generate(ti: &TypeInfo, all: &[TypeInfo], prefix: &str, cn: &str, asn_defs: &HashMap<String, String>) -> String {
    let jt = resolve_java_type(&ti.name, all, prefix);
    let jt = if jt == cn {
        resolve_java_type(
            match &ti.kind { TypeKind::Newtype { inner_type } => inner_type, _ => unreachable!() },
            all, prefix,
        )
    } else { jt };

    let size = helpers::resolve_size(&ti.name, asn_defs);

    let mut c = String::new();

    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(1, "public void testEncodeDecodeAper() throws Exception {"));
    if jt == "int" || jt == "long" || jt == "float" || jt == "double" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(1);", cn, cn)));
    // "boolean" is dead code, all booleans use CmsBoolean (INTEGER wrapper) now
    } else if jt == "String" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, &format!("obj.value = \"{}\";", "x".repeat(size))));
    } else if jt == "byte[]" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, &format!("obj.value = new byte[{}];", size)));
    } else if jt.starts_with("java.util.List<") {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, "obj.value = new java.util.ArrayList<>();"));
    } else {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    }
    c.push_str(&helpers::ln(2, "byte[] data = obj.encode(\"aper\");"));
    c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(\"aper\", data);", cn, cn)));
    c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
    c.push_str(&helpers::ln(1, "}"));
    c
}
