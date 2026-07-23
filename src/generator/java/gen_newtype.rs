use std::collections::HashMap;
use super::super::*;
use super::helpers;

pub fn generate(
    ti: &TypeInfo,
    _all: &[TypeInfo],
    prefix: &str,
    cn: &str,
    asn_doc: &Option<String>,
    asn_defs: &HashMap<String, String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
    jt: &str,
) -> String {
    // Detect BIT STRING → need hex formatting in JER
    let inner_bit_string = match &ti.kind {
        TypeKind::Newtype { inner_type } => {
            inner_type.starts_with("FixedBitString") || inner_type.starts_with("BitString")
        }
        _ => false,
    };
    let (hex_digits, bit_count) = if inner_bit_string || named_consts.contains_key(&ti.name) {
        if let Some(def) = asn_defs.get(&ti.name) {
            if let Some((Some(min), Some(max))) = helpers::parse_asn1_size(def) {
                let bits = max.max(min);
                let hd = std::cmp::max(2, ((bits + 7) / 8) * 2);
                (hd, bits)
            } else {
                (2, 8)
            }
        } else {
            (2, 8)
        }
    } else { (0, 0) };

    let base = format!("{}Base", prefix);
    let native = format!("{}Native", prefix);

    let mut c = String::new();
    if let Some(doc) = asn_doc { c.push_str(doc); }
    c.push_str("@Data\n");
    c.push_str("@lombok.experimental.Accessors(chain = true, fluent = true)\n");
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(1, &format!("public static final int {} = {};", name, val)));
        }
    }
    c.push_str(&helpers::ln(1, &format!("private static final ObjectMapper MAPPER = {}.createMapper();", base)));
    if hex_digits > 0 {
        c.push_str(&helpers::ln(1, &format!("public {} value;", jt)));
        c.push_str(&helpers::ln(1, &format!("public {}() {{}}", cn)));
        c.push_str(&helpers::ln(1, &format!("public {}({} value) {{ this.value = value; }}", cn, jt)));
        c.push_str(&helpers::ln(1, "@JsonValue"));
        c.push_str(&helpers::ln(1, &format!("public String toJsonValue() {{ return {}.bitStringHex(this.value, {}); }}", base, bit_count)));
        c.push_str(&helpers::ln(1, "@JsonCreator"));
        c.push_str(&helpers::ln(1, &format!("public {}(String hex) {{ this.value = {}.parseBitStringHex(hex, {}); }}", cn, base, bit_count)));
    } else {
        let default_val = match jt {
            "String" => {
                let sz = helpers::resolve_size(&ti.name, asn_defs);
                if sz > 0 { format!(" = \"{}\"", "x".repeat(sz)) } else { " = \"\"".to_string() }
            }
            "byte[]" => {
                let sz = helpers::resolve_size(&ti.name, asn_defs);
                if sz > 0 { format!(" = new byte[{}]", sz) } else { " = new byte[0]".to_string() }
            }
            _ if jt.starts_with("java.util.List<") => " = new java.util.ArrayList<>()".to_string(),
            _ => "".to_string(),
        };
        c.push_str(&helpers::ln(1, &format!("@JsonValue public {} value{};", jt, default_val)));
        c.push_str(&helpers::ln(1, &format!("public {}() {{}}", cn)));
        c.push_str(&helpers::ln(1, "@JsonCreator"));
        c.push_str(&helpers::ln(1, &format!("public {}({} value) {{ this.value = value; }}", cn, jt)));
    }

    let (encode_arg, wrap_try): (String, bool) = if jt.starts_with("java.util.List<") {
        ("MAPPER.writeValueAsString(this.value)".into(), true)
    } else if jt == "byte[]" {
        (format!("MAPPER.writeValueAsString({}.hex(this.value))", base), true)
    } else if hex_digits > 0 {
        (format!("MAPPER.writeValueAsString({}.bitStringHex(this.value, {}))", base, bit_count), true)
    } else if jt == "String" {
        ("MAPPER.writeValueAsString(this.value)".into(), true)
    } else if jt == "Object" {
        ("\"null\"".into(), false)
    } else if jt.starts_with(prefix) {
        ("MAPPER.writeValueAsString(this.value)".into(), true)
    } else {
        ("String.valueOf(this.value)".into(), false)
    };

    if wrap_try {
        c.push_str(&helpers::ln(1, "public byte[] encode() {"));
        c.push_str(&helpers::ln(2, "try {"));
        c.push_str(&helpers::ln(3, &format!("return {}.encode(\"{}\", DEFAULT_ENCODING, {});", native, ti.name, encode_arg)));
        c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
        c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
        c.push_str(&helpers::ln(2, "}"));
        c.push_str(&helpers::ln(1, "}"));
        c.push_str(&helpers::ln(1, "public byte[] encodeTest() {"));
        c.push_str(&helpers::ln(2, "try {"));
        c.push_str(&helpers::ln(3, &format!("return {}.encode(\"{}\", DEFAULT_ENCODING, {});", native, ti.name, encode_arg)));
        c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
        c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
        c.push_str(&helpers::ln(2, "}"));
        c.push_str(&helpers::ln(1, "}"));
    } else {
        c.push_str(&helpers::ln(1, "public byte[] encode() {"));
        c.push_str(&helpers::ln(2, &format!("return {}.encode(\"{}\", DEFAULT_ENCODING, {});", native, ti.name, encode_arg)));
        c.push_str(&helpers::ln(1, "}"));
        c.push_str(&helpers::ln(1, "public byte[] encodeTest() {"));
        c.push_str(&helpers::ln(2, &format!("return {}.encode(\"{}\", DEFAULT_ENCODING, {});", native, ti.name, encode_arg)));
        c.push_str(&helpers::ln(1, "}"));
    }

    // decode
    c.push_str(&helpers::ln(1, &format!("public static {} decode(byte[] data) {{", cn)));
    c.push_str(&helpers::ln(2, "try {"));
    c.push_str(&helpers::ln(3, &format!("String json = {}.decode(\"{}\", DEFAULT_ENCODING, data);", native, ti.name)));
    c.push_str(&helpers::ln(3, &format!("{} r = new {}();", cn, cn)));
    if jt.starts_with("java.util.List<") {
        let inner = jt.trim_start_matches("java.util.List<").trim_end_matches('>').trim();
        c.push_str(&helpers::ln(3, &format!(
            "r.value = MAPPER.convertValue(MAPPER.readTree(json).get(\"value\"), new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}});",
            inner
        )));
    } else if jt == "byte[]" {
        c.push_str(&helpers::ln(3, &format!("r.value = {}.unhex(MAPPER.readTree(json).get(\"value\").asText());", base)));
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
    } else if hex_digits > 0 {
        c.push_str(&helpers::ln(3, &format!("r.value = {}.parseBitStringHex(MAPPER.readTree(json).get(\"value\").asText(), {});", base, bit_count)));
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
