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
    // Check inner type name (more reliable than named_consts for some types)
    let inner_bit_string = match &ti.kind {
        TypeKind::Newtype { inner_type } => {
            inner_type.starts_with("FixedBitString") || inner_type.starts_with("BitString")
        }
        _ => false,
    };
    let (hex_digits, bit_count) = if inner_bit_string || named_consts.contains_key(&ti.name) {
        // Determine hex digit count from SIZE constraint in ASN.1 definition
        if let Some(def) = asn_defs.get(&ti.name) {
            if let Some((Some(min), Some(max))) = helpers::parse_asn1_size(def) {
                // For fixed SIZE(N), use N bits → ceil(N/8) * 2 hex chars
                let bits = max.max(min);
                let hd = std::cmp::max(2, ((bits + 7) / 8) * 2);
                (hd, bits)
            } else {
                (2, 8) // default for BIT STRING without known size
            }
        } else {
            (2, 8)
        }
    } else { (0, 0) };

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
    if hex_digits > 0 {
        // BIT STRING: @JsonValue returns hex string, not raw int
        c.push_str(&helpers::ln(1, &format!("public {} value;", jt)));
        c.push_str(&helpers::ln(1, &format!("public {}() {{}}", cn)));
        c.push_str(&helpers::ln(1, &format!("public {}({} value) {{ this.value = value; }}", cn, jt)));
        c.push_str(&helpers::ln(1, "@JsonValue"));
        c.push_str(&helpers::ln(1, &format!("public String toJsonValue() {{ return CmsBase.bitStringHex(this.value, {}); }}", bit_count)));
        c.push_str(&helpers::ln(1, "@JsonCreator"));
        c.push_str(&helpers::ln(1, &format!("public {}(String hex) {{ this.value = CmsBase.parseBitStringHex(hex, {}); }}", cn, bit_count)));
    } else {
        let size = helpers::resolve_size(&ti.name, asn_defs);
        let default_val = match jt {
            "String" => {
                if size > 1 { format!(" = \"{}\"", "x".repeat(size)) } else { " = \"\"".to_string() }
            }
            "byte[]" => format!(" = new byte[{}]", size),
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
        ("MAPPER.writeValueAsString(CmsBase.hex(this.value))".into(), true)
    } else if hex_digits > 0 {
        // BIT STRING — format as hex string with correct bit ordering
        (format!("MAPPER.writeValueAsString(CmsBase.bitStringHex(this.value, {}))", bit_count), true)
    } else if jt == "String" {
        // String needs JSON-quoting via MAPPER
        ("MAPPER.writeValueAsString(this.value)".into(), true)
    } else if jt == "Object" {
        // NULL type — JER expects JSON null
        ("\"null\"".into(), false)
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
            "r.value = MAPPER.convertValue(MAPPER.readTree(json).get(\"value\"), new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}});",
            inner
        )));
    } else if jt == "byte[]" {
        c.push_str(&helpers::ln(3, "r.value = CmsBase.unhex(MAPPER.readTree(json).get(\"value\").asText());"));
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
        // BIT STRING: JER returns hex string, parse with correct bit ordering
        c.push_str(&helpers::ln(3, &format!("r.value = CmsBase.parseBitStringHex(MAPPER.readTree(json).get(\"value\").asText(), {});", bit_count)));
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
