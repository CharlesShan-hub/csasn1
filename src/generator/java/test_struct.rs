use std::collections::HashMap;
use super::super::*;
use super::helpers;

pub fn generate(_ti: &TypeInfo, _all: &[TypeInfo], _prefix: &str, cn: &str, _fields: &[FieldInfo], _asn_defs: &HashMap<String, String>) -> String {
    let mut c = String::new();

    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(1, "public void testEncodeDecodeAper() throws Exception {"));
    c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
    c.push_str(&helpers::ln(2, "obj.encodeTest();"));
    c.push_str(&helpers::ln(1, "}"));
    c
}
