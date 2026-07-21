use std::collections::HashMap;
use super::super::*;
use super::helpers;

pub fn generate(
    ti: &TypeInfo,
    _all: &[TypeInfo],
    prefix: &str,
    cn: &str,
    asn_doc: &Option<String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
    jt: &str,
) -> String {
    let mut c = String::new();
    if let Some(doc) = asn_doc { c.push_str(doc); }
    c.push_str("@Data\n");
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    // Named constants (BIT STRING values)
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(1, &format!("public static final int {} = {};", name, val)));
        }
    }
    c.push_str(&helpers::ln(1, "private static final ObjectMapper MAPPER = new ObjectMapper();"));
    c.push_str(&helpers::ln(1, &format!("@JsonProperty public {} value;", jt)));
    c.push_str(&helpers::ln(1, &format!("public {}() {{}}", cn)));
    c.push_str(&helpers::ln(1, &format!("public {}({} value) {{ this.value = value; }}", cn, jt)));

    let (encode_arg, wrap_try): (String, bool) = if jt.starts_with("java.util.List<") {
        ("MAPPER.writeValueAsString(this.value)".into(), true)
    } else if jt == "byte[]" {
        ("java.util.Base64.getEncoder().encodeToString(this.value)".into(), false)
    } else if jt == "Object" {
        ("\"\"".into(), false)
    } else {
        ("String.valueOf(this.value)".into(), false)
    };

    if wrap_try {
        c.push_str(&helpers::ln(1, "public byte[] encode(String enc) {"));
        c.push_str(&helpers::ln(2, "try {"));
        c.push_str(&helpers::ln(3, &format!("return CmsNative.encode(\"{}\", enc, {});", ti.name, encode_arg)));
        c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
        c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
        c.push_str(&helpers::ln(2, "}"));
        c.push_str(&helpers::ln(1, "}"));
        c.push_str(&helpers::ln(1, "public byte[] encode() {"));
        c.push_str(&helpers::ln(2, "return encode(DEFAULT_ENCODING);"));
        c.push_str(&helpers::ln(1, "}"));
    } else {
        helpers::enc_overload(&mut c, &format!(
            "{}",
            helpers::ln(2, &format!("return CmsNative.encode(\"{}\", enc, {});", ti.name, encode_arg)),
        ));
    }

    // decode
    c.push_str(&helpers::ln(1, &format!("public static {} decode(String enc, byte[] data) {{", cn)));
    c.push_str(&helpers::ln(2, "try {"));
    c.push_str(&helpers::ln(3, &format!("String json = CmsNative.decode(\"{}\", enc, data);", ti.name)));
    c.push_str(&helpers::ln(3, &format!("{} r = new {}();", cn, cn)));
    if jt.starts_with("java.util.List<") {
        let inner = jt.trim_start_matches("java.util.List<").trim_end_matches('>').trim();
        c.push_str(&helpers::ln(3, &format!(
            "r.value = MAPPER.readValue(json.trim(), new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}});",
            inner
        )));
    } else if jt == "byte[]" {
        c.push_str(&helpers::ln(3, "r.value = java.util.Base64.getDecoder().decode(json.trim());"));
    } else if jt == "Object" {
        c.push_str(&helpers::ln(3, "r.value = null;"));
    } else if jt == "long" {
        c.push_str(&helpers::ln(3, "r.value = MAPPER.readTree(json).get(\"value\").asLong();"));
    } else if jt == "String" {
        c.push_str(&helpers::ln(3, "r.value = MAPPER.readTree(json).get(\"value\").asText();"));
    } else if jt == "boolean" {
        c.push_str(&helpers::ln(3, "r.value = MAPPER.readTree(json).get(\"value\").asBoolean();"));
    } else if jt == "float" {
        c.push_str(&helpers::ln(3, "r.value = (float) MAPPER.readTree(json).get(\"value\").asDouble();"));
    } else if jt == "double" {
        c.push_str(&helpers::ln(3, "r.value = MAPPER.readTree(json).get(\"value\").asDouble();"));
    } else if jt == "int" {
        c.push_str(&helpers::ln(3, "r.value = MAPPER.readTree(json).get(\"value\").asInt();"));
    } else {
        c.push_str(&helpers::ln(3, &format!("r.value = MAPPER.readValue(json.trim(), {}.class);", jt)));
    }
    c.push_str(&helpers::ln(3, "return r;"));
    c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
    c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(1, "}"));
    c.push_str("}\n");
    c
}
