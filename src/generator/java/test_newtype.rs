use super::super::*;
use super::helpers;
use super::type_map::resolve_java_type;
use std::collections::HashMap;

pub fn generate(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    cn: &str,
    asn_defs: &HashMap<String, String>,
) -> String {
    let jt = resolve_java_type(&ti.name, all, prefix);
    let jt = if jt == cn {
        resolve_java_type(
            match &ti.kind {
                TypeKind::Newtype { inner_type, .. } => inner_type,
                _ => unreachable!(),
            },
            all,
            prefix,
        )
    } else {
        jt
    };

    let size = match &ti.kind {
        TypeKind::Newtype { size_from_attr, .. } => {
            size_from_attr.unwrap_or_else(|| helpers::resolve_size(&ti.name, asn_defs))
        }
        _ => helpers::resolve_size(&ti.name, asn_defs),
    };

    let mut c = String::new();

    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(
        1,
        "public void testEncodeDecodeAper() throws Exception {",
    ));
    if jt == "int" || jt == "Integer" || jt == "boolean" || jt == "Boolean" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(1);", cn, cn)));
    } else if jt == "long" || jt == "Long" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(1L);", cn, cn)));
    } else if jt == "float" || jt == "Float" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(1.5f);", cn, cn)));
    } else if jt == "double" || jt == "Double" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}(2.5);", cn, cn)));
    } else if jt == "String" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(
            2,
            &format!("obj._v.put(\"_\", \"{}\");", "x".repeat(size)),
        ));
    } else if jt == "byte[]" {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        let bytes: String = std::iter::repeat("1")
            .take(size)
            .collect::<Vec<_>>()
            .join(", ");
        c.push_str(&helpers::ln(
            2,
            &format!("obj._v.put(\"_\", new byte[] {{ {} }});", bytes),
        ));
    } else if jt.starts_with("java.util.List<") {
        let inner = jt
            .trim_start_matches("java.util.List<")
            .trim_end_matches('>')
            .trim();
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(
            2,
            &format!(
                "obj._v.put(\"_\", java.util.Collections.singletonList(new {}()));",
                inner
            ),
        ));
    } else {
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    }
    c.push_str(&helpers::ln(2, "byte[] data = obj.encodeTest();"));
    c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(data);", cn, cn)));
    c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
    c.push_str(&helpers::ln(1, "}"));
    c
}
