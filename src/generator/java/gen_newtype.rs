use super::super::*;
use super::helpers;
use std::collections::HashMap;

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
        TypeKind::Newtype { inner_type, .. } => {
            inner_type.starts_with("FixedBitString") || inner_type.starts_with("BitString")
        }
        _ => false,
    };

    // Detect unsigned 32-bit integer types that need unsigned long conversion
    // u32 maps to Java int but values > i32::MAX (2147483647) can't be represented
    let inner_unsigned_int = match &ti.kind {
        TypeKind::Newtype { inner_type, .. } => inner_type == "u32",
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
    } else {
        (0, 0)
    };

    // Size from #[size] attribute on the newtype struct, fallback to ASN.1 text parsing
    let size = match &ti.kind {
        TypeKind::Newtype { size_from_attr, .. } => {
            size_from_attr.unwrap_or_else(|| helpers::resolve_size(&ti.name, asn_defs))
        }
        _ => helpers::resolve_size(&ti.name, asn_defs),
    };

    let base = format!("{}Base", prefix);
    let native = format!("{}Native", prefix);

    let mut c = String::new();
    if let Some(doc) = asn_doc {
        c.push_str(doc);
    }
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(
                1,
                &format!("public static final int {} = {};", name, val),
            ));
        }
    }
    if hex_digits > 0 {
        let default_hex = "0".repeat(hex_digits);
        c.push_str(&helpers::ln(
            1,
            &format!("public {}() {{ _v.put(\"_\", \"{}\"); }}", cn, default_hex),
        ));
        c.push_str(&helpers::ln(
            1,
            &format!(
                "public {}({} v) {{ this(); _v.put(\"_\", {}.bitStringHex(v, {})); }}",
                cn, jt, base, bit_count
            ),
        ));
        c.push_str(&helpers::ln(1, "@JsonValue"));
        c.push_str(&helpers::ln(1, "@Override"));
        c.push_str(&helpers::ln(
            1,
            &format!("public Object toJsonValue() {{ return _v.get(\"_\"); }}"),
        ));
        c.push_str(&helpers::ln(1, "@JsonCreator"));
        c.push_str(&helpers::ln(
            1,
            &format!(
                "public static {} fromJson(String hex) {{ return new {}(hex); }}",
                cn, cn
            ),
        ));
        c.push_str(&helpers::ln(
            1,
            &format!(
                "public {}(String hex) {{ this(); _v.put(\"_\", hex); }}",
                cn
            ),
        ));
    } else {
        if inner_unsigned_int {
            c.push_str(&helpers::ln(
                1,
                &format!("public {}() {{ _v.put(\"_\", 0); }}", cn),
            ));
            c.push_str(&helpers::ln(1, "@JsonValue"));
            c.push_str(&helpers::ln(1, "@Override"));
            c.push_str(&helpers::ln(1, &format!("public Object toJsonValue() {{ return Integer.toUnsignedLong((int) _v.get(\"_\")); }}")));
            c.push_str(&helpers::ln(
                1,
                &format!(
                    "public {}(long v) {{ this(); _v.put(\"_\", (int) v); }}",
                    cn
                ),
            ));
            c.push_str(&helpers::ln(1, "@JsonCreator"));
            c.push_str(&helpers::ln(
                1,
                &format!(
                    "public static {} fromJson(long v) {{ return new {}(v); }}",
                    cn, cn
                ),
            ));
        } else {
            let default_val = match jt {
                "String" => {
                    let sz = size;
                    if sz > 0 {
                        format!("\"{}\"", "x".repeat(sz))
                    } else {
                        "\"\"".to_string()
                    }
                }
                "byte[]" => {
                    let sz = size;
                    if sz > 0 {
                        format!("new byte[{}]", sz)
                    } else {
                        "new byte[0]".to_string()
                    }
                }
                _ if jt.starts_with("java.util.List<") => "new java.util.ArrayList<>()".to_string(),
                "int" | "Integer" => "1".to_string(),
                "long" | "Long" => "1L".to_string(),
                "float" | "Float" => "1.5f".to_string(),
                "double" | "Double" => "2.5".to_string(),
                "boolean" | "Boolean" => "true".to_string(),
                "Object" => "null".to_string(),
                _ if jt.starts_with("DefaultInner") => {
                    let sz = size;
                    if sz > 0 && jt == "DefaultInnerOctetString" {
                        let bytes: Vec<String> =
                            std::iter::repeat("1".to_string()).take(sz).collect();
                        format!("new byte[] {{ {} }}", bytes.join(", "))
                    } else if sz > 0
                        && (jt == "DefaultInnerVisibleString" || jt == "DefaultInnerUtf8String")
                    {
                        format!("\"{}\"", "x".repeat(sz))
                    } else {
                        format!("new {}()", jt)
                    }
                }
                _ => format!("new {}()", jt),
            };
            if default_val.is_empty() {
                c.push_str(&helpers::ln(1, &format!("public {}() {{}}", cn)));
            } else {
                c.push_str(&helpers::ln(
                    1,
                    &format!("public {}() {{ _v.put(\"_\", {}); }}", cn, default_val),
                ));
            }
            let json_creator_type = if jt.starts_with("java.util.List<") {
                "Object"
            } else {
                match jt {
                    "int" | "Integer" => "int",
                    "long" | "Long" => "long",
                    "boolean" | "Boolean" => "boolean",
                    "float" | "Float" | "double" | "Double" => "double",
                    _ => "String",
                }
            };
            let json_creator_body = if jt.starts_with("java.util.List<") {
                let inner = jt
                    .trim_start_matches("java.util.List<")
                    .trim_end_matches('>')
                    .trim();
                format!(
                    "{{ {} r = new {}(); r._v.put(\"_\", {}.MAPPER.convertValue(v, new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}})); return r; }}",
                    cn, cn, base, inner
                )
            } else if jt == "byte[]" {
                format!(
                    "{{ {} r = new {}(); r._v.put(\"_\", {}.unhex(v)); return r; }}",
                    cn, cn, base
                )
            } else {
                format!(
                    "{{ {} r = new {}(); r._v.put(\"_\", v); return r; }}",
                    cn, cn
                )
            };
            c.push_str(&helpers::ln(1, "@JsonCreator"));
            c.push_str(&helpers::ln(
                1,
                &format!(
                    "public static {} fromJson({} v) {}",
                    cn, json_creator_type, json_creator_body
                ),
            ));
            let ctor_type = match jt {
                "Integer" | "int" => "Integer",
                "Long" | "long" => "Long",
                "Boolean" | "boolean" => "Boolean",
                "Float" | "float" => "Float",
                "Double" | "double" => "Double",
                "String" => "String",
                _ => jt,
            };
            if ctor_type != jt || !jt.starts_with("DefaultInner") {
                c.push_str(&helpers::ln(
                    1,
                    &format!(
                        "public {}({} v) {{ this(); _v.put(\"_\", v); }}",
                        cn, ctor_type
                    ),
                ));
            }
            c.push_str(&helpers::ln(1, "@JsonValue"));
            c.push_str(&helpers::ln(
                1,
                "public Object toJsonValue() { return _v.get(\"_\"); }",
            ));
        }
    }

    let inner_octet_string = match &ti.kind {
        TypeKind::Newtype { inner_type, .. } => {
            inner_type.starts_with("OctetString") || inner_type.starts_with("FixedOctetString")
        }
        _ => false,
    };
    let (encode_arg, wrap_try): (String, bool) = if jt.starts_with("java.util.List<") {
        (
            format!(
                "{}.MAPPER.writeValueAsString({}.toJson(_v.get(\"_\")))",
                base, base
            ),
            true,
        )
    } else if jt == "byte[]" {
        (
            format!(
                "{}.MAPPER.writeValueAsString({}.hex((byte[]) _v.get(\"_\")))",
                base, base
            ),
            true,
        )
    } else if hex_digits > 0 {
        (
            format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base),
            true,
        )
    } else if jt == "String" {
        (
            format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base),
            true,
        )
    } else if jt == "Object" {
        ("\"null\"".into(), false)
    } else if jt.starts_with(prefix) {
        (
            format!(
                "{}.MAPPER.writeValueAsString({}.toJson(_v.get(\"_\")))",
                base, base
            ),
            true,
        )
    } else if jt == "DefaultInnerOctetString" {
        (
            format!(
                "{}.MAPPER.writeValueAsString({}.hex((byte[]) _v.get(\"_\")))",
                base, base
            ),
            true,
        )
    } else if jt.starts_with("DefaultInner") {
        (
            format!("{}.MAPPER.writeValueAsString(_v.get(\"_\"))", base),
            true,
        )
    } else if inner_unsigned_int {
        (
            "String.valueOf(Integer.toUnsignedLong((int) _v.get(\"_\")))".into(),
            false,
        )
    } else {
        ("String.valueOf(_v.get(\"_\"))".into(), false)
    };

    if wrap_try {
        c.push_str(&helpers::ln(1, "public byte[] encode() {"));
        c.push_str(&helpers::ln(2, "try {"));
        c.push_str(&helpers::ln(
            3,
            &format!(
                "return {}.encode(\"{}\", DEFAULT_ENCODING, {});",
                native, ti.name, encode_arg
            ),
        ));
        c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
        c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
        c.push_str(&helpers::ln(2, "}"));
        c.push_str(&helpers::ln(1, "}"));
        c.push_str(&helpers::ln(1, "public byte[] encodeTest() {"));
        c.push_str(&helpers::ln(2, "try {"));
        c.push_str(&helpers::ln(
            3,
            &format!(
                "return {}.encode(\"{}\", DEFAULT_ENCODING, {});",
                native, ti.name, encode_arg
            ),
        ));
        c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
        c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
        c.push_str(&helpers::ln(2, "}"));
        c.push_str(&helpers::ln(1, "}"));
    } else {
        c.push_str(&helpers::ln(1, "public byte[] encode() {"));
        c.push_str(&helpers::ln(
            2,
            &format!(
                "return {}.encode(\"{}\", DEFAULT_ENCODING, {});",
                native, ti.name, encode_arg
            ),
        ));
        c.push_str(&helpers::ln(1, "}"));
        c.push_str(&helpers::ln(1, "public byte[] encodeTest() {"));
        c.push_str(&helpers::ln(
            2,
            &format!(
                "return {}.encode(\"{}\", DEFAULT_ENCODING, {});",
                native, ti.name, encode_arg
            ),
        ));
        c.push_str(&helpers::ln(1, "}"));
    }

    // decode
    c.push_str(&helpers::ln(
        1,
        &format!("public static {} decode(byte[] data) {{", cn),
    ));
    c.push_str(&helpers::ln(2, "try {"));
    c.push_str(&helpers::ln(
        3,
        &format!(
            "String json = {}.decode(\"{}\", DEFAULT_ENCODING, data);",
            native, ti.name
        ),
    ));
    c.push_str(&helpers::ln(3, &format!("{} r = new {}();", cn, cn)));
    // Rust wraps bare values as {"value": X} — unwrap before extracting
    c.push_str(&helpers::ln(
        3,
        &format!(
            "com.fasterxml.jackson.databind.JsonNode _node = {}.MAPPER.readTree(json);",
            base
        ),
    ));
    c.push_str(&helpers::ln(
        3,
        "if (_node.isObject() && _node.has(\"value\")) _node = _node.get(\"value\");",
    ));
    if jt.starts_with("java.util.List<") {
        let inner = jt
            .trim_start_matches("java.util.List<")
            .trim_end_matches('>')
            .trim();
        c.push_str(&helpers::ln(3, &format!(
            "r._v.put(\"_\", {}.MAPPER.convertValue(_node, new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}}));",
            base, inner
        )));
    } else if jt == "byte[]" {
        c.push_str(&helpers::ln(
            3,
            &format!("r._v.put(\"_\", {}.unhex(_node.asText()));", base),
        ));
    } else if jt == "String" {
        c.push_str(&helpers::ln(3, "r._v.put(\"_\", _node.asText());"));
    } else if hex_digits > 0 {
        // BIT STRING stores hex string in _v (same shape as constructor)
        c.push_str(&helpers::ln(3, "r._v.put(\"_\", _node.asText());"));
    } else if jt == "int" || jt == "Integer" {
        c.push_str(&helpers::ln(3, "r._v.put(\"_\", _node.asInt());"));
    } else if jt == "long" || jt == "Long" {
        c.push_str(&helpers::ln(3, "r._v.put(\"_\", _node.asLong());"));
    } else if jt == "boolean" || jt == "Boolean" {
        c.push_str(&helpers::ln(3, "r._v.put(\"_\", _node.asBoolean());"));
    } else if jt == "float" || jt == "Float" {
        c.push_str(&helpers::ln(
            3,
            "r._v.put(\"_\", (float) _node.asDouble());",
        ));
    } else if jt == "double" || jt == "Double" {
        c.push_str(&helpers::ln(3, "r._v.put(\"_\", _node.asDouble());"));
    } else if jt == "Object" {
        c.push_str(&helpers::ln(3, "r._v.put(\"_\", null);"));
    } else if inner_octet_string {
        c.push_str(&helpers::ln(3, &format!("r._v.put(\"_\", _node.asText().isEmpty() ? new byte[0] : {}.unhex(_node.asText()));", base)));
    } else {
        c.push_str(&helpers::ln(
            3,
            &format!(
                "r._v.put(\"_\", {}.MAPPER.readValue(_node.toString(), {}.class));",
                base, jt
            ),
        ));
    }
    c.push_str(&helpers::ln(3, "return r;"));
    c.push_str(&helpers::ln(2, "} catch (Exception e) {"));
    c.push_str(&helpers::ln(3, "throw new RuntimeException(e);"));
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(1, "}"));
    c.push_str("}\n");
    c
}
