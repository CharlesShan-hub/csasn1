use std::collections::HashMap;
use super::super::*;
use super::type_map::{resolve_wrapper_type, resolve_java_type, java_type_ref};
use super::helpers;
use super::helpers::safe_field_name;

pub fn generate(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    cn: &str,
    asn_doc: &Option<String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
    variants: &[VariantInfo],
) -> String {
    let mut c = String::new();
    let base = format!("{}Base", prefix);
    let native = format!("{}Native", prefix);
    if let Some(doc) = asn_doc { c.push_str(doc); }
    c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
    c.push_str("@Data\n");
    c.push_str("@lombok.experimental.Accessors(chain = true, fluent = true)\n");
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(1, &format!("public static final int {} = {};", name, val)));
        }
    }
    c.push_str(&helpers::ln(1, "@JsonIgnore public String _choice;"));
    c.push_str(&helpers::ln(1, &format!("private static final ObjectMapper MAPPER = {}.createMapper();", base)));

    // No-arg constructor picks the first variant as default
    // and initializes ALL variant fields so that Lombok Inner*.equals()
    // works correctly (no null-vs-default mismatches after decode).
    if let Some(first) = variants.first() {
        let fname = safe_field_name(&first.name);
        let json_key = first.identifier.as_deref().unwrap_or(&first.name);
        let jt = resolve_wrapper_type(&first.inner_type, all, prefix);
        let init = match jt.as_str() {
            "int" => " = 1".to_string(),
            "long" => " = 1L".to_string(),
            "float" => " = 1.5f".to_string(),
            "double" => " = 2.5".to_string(),
            "String" => " = \"\"".to_string(),
            "byte[]" => " = new byte[0]".to_string(),
            "boolean" => " = true".to_string(),
            _ => format!(" = new {}()", jt),
        };
        let def_init = |jt: &str| -> String {
              match jt {
                  "int" => " = 0".to_string(),
                  "Integer" => " = 0".to_string(),
                  "long" => " = 0L".to_string(),
                  "float" => " = 0.0f".to_string(),
                  "double" => " = 0.0".to_string(),
                  "String" => " = \"\"".to_string(),
                  "byte[]" => " = new byte[0]".to_string(),
                  "boolean" => " = false".to_string(),
                  _ if jt.starts_with("java.util.List") => " = new java.util.ArrayList<>()".to_string(),
                  // All object types: leave null to avoid circular init and Jackson errors.
                  // CmsChoice.syncFromInner handles null → default via reflection when selected.
                  _ => " = null".to_string(),
              }
          };
        c.push_str(&helpers::ln(1, &format!("public {}() {{", cn)));
        c.push_str(&helpers::ln(2, &format!("this._choice = \"{}\";", json_key)));
        c.push_str(&helpers::ln(2, &format!("this.{}{};", fname, init)));
        for v in variants.iter().skip(1) {
            let v_jt = resolve_wrapper_type(&v.inner_type, all, prefix);
            let v_fname = safe_field_name(&v.name);
            c.push_str(&helpers::ln(2, &format!("this.{}{};", v_fname, def_init(&v_jt))));
        }
        c.push_str(&helpers::ln(1, "}"));
    }

    for (i, v) in variants.iter().enumerate() {
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);
        let fname = safe_field_name(&v.name);
        c.push_str(&helpers::ln(1, &format!("@JsonIgnore public {} {};", jt, fname)));
        // Named int constant for this variant
        let const_name = v.name.to_uppercase();
        c.push_str(&helpers::ln(1, &format!("public static final int {} = {};", const_name, i)));
    }

    // Typesafe setters — set _choice + field in one call
    for v in variants {
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);
        let fname = safe_field_name(&v.name);
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        let setter_name = format!("set{}{}", &v.name[..1].to_uppercase(), &v.name[1..]);
        c.push_str(&helpers::ln(1, &format!("public void {}({} v) {{", setter_name, jt)));
        c.push_str(&helpers::ln(2, &format!("this._choice = \"{}\";", json_key)));
        c.push_str(&helpers::ln(2, &format!("this.{} = v;", fname)));
        c.push_str(&helpers::ln(1, "}"));
    }

    // Generic value(int choice, Object val) using named constants
    c.push_str(&helpers::ln(1, "public void value(int choice, Object val) {"));
    c.push_str(&helpers::ln(2, "switch (choice) {"));
    for v in variants {
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);
        let const_name = v.name.to_uppercase();
        let setter_name = format!("set{}{}", &v.name[..1].to_uppercase(), &v.name[1..]);
        c.push_str(&helpers::ln(3, &format!("case {}: {}(({}) val); break;", const_name, setter_name, jt)));
    }
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(1, "}"));

    // Detect variable-length BIT STRING: inner_type starts with "BitString" but NOT "FixedBitString"
    fn is_variable_bit_string(inner_type: &str) -> bool {
        inner_type.trim_start_matches("Option<").trim_end_matches('>').trim()
            .starts_with("BitString")
            && !inner_type.contains("FixedBitString")
    }

    /// Detect types that wrap a @JsonValue byte[] (e.g. InnerFloat32 wraps byte[] via FixedOctetString).
    /// These need special handling in serialize/deserialize because Jackson's @JsonAnyGetter
    /// may not respect @JsonValue on the wrapper object.
    fn is_byte_array_wrapper(inner_type: &str, all: &[TypeInfo], prefix: &str) -> bool {
        let wrapper = resolve_wrapper_type(inner_type, all, prefix);
        let inner = resolve_java_type(inner_type, all, prefix);
        wrapper != inner && inner == "byte[]"
    }

    // Serialize (only output the active branch)
    c.push_str(&helpers::ln(1, "@JsonAnyGetter"));
    c.push_str(&helpers::ln(1, "public java.util.Map<String, Object> serializeChoice() {"));
    c.push_str(&helpers::ln(2, "java.util.Map<String, Object> map = new java.util.HashMap<String, Object>();"));
    c.push_str(&helpers::ln(2, "if (_choice != null) {"));
    for v in variants {
        let fname = safe_field_name(&v.name);
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        if is_variable_bit_string(&v.inner_type) {
            // JER variable-length BIT STRING requires {"value":<hex>,"length":<bits>}
            c.push_str(&helpers::ln(3, &format!(
                "if (\"{}\".equals(_choice)) {{\
                 java.util.Map<String, Object> bs = new java.util.HashMap<>();\
                 bs.put(\"value\", {}.hex({}));\
                 bs.put(\"length\", {}.length * 8);\
                 map.put(\"{}\", bs);\
                 }}",
                json_key, base, fname, fname, json_key
            )));
        } else if is_byte_array_wrapper(&v.inner_type, all, prefix) {
            // @JsonValue byte[] wrapper: serialize as bare hex string (rasn JER expects hex, not {"value":...})
            c.push_str(&helpers::ln(3, &format!(
                "if (\"{}\".equals(_choice)) map.put(\"{}\", {}.hex({}.value));",
                json_key, json_key, base, fname
            )));
        } else {
            c.push_str(&helpers::ln(3, &format!("if (\"{}\".equals(_choice)) map.put(\"{}\", {});", json_key, json_key, fname)));
        }
    }
    c.push_str(&helpers::ln(2, "}"));
    c.push_str(&helpers::ln(2, "return map;"));
    c.push_str(&helpers::ln(1, "}"));

    // Deserialize
    c.push_str(&helpers::ln(1, "@JsonAnySetter"));
    c.push_str(&helpers::ln(1, "public void deserializeChoice(String key, Object value) {"));
    c.push_str(&helpers::ln(2, "if (\"_choice\".equals(key)) return;"));
    c.push_str(&helpers::ln(2, "this._choice = key;"));
    for v in variants {
        let fname = safe_field_name(&v.name);
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        c.push_str(&helpers::ln(2, &format!("if (\"{}\".equals(key)) {{", json_key)));
        if jt == "byte[]" && is_variable_bit_string(&v.inner_type) {
            // JER variable-length BIT STRING: extract "value" field from the map
            c.push_str(&helpers::ln(3, &format!(
                "if (value instanceof java.util.Map) {{\
                 this.{} = {}.unhex(((java.util.Map<String, String>) value).get(\"value\"));\
                 }}",
                fname, base
            )));
        } else if is_byte_array_wrapper(&v.inner_type, all, prefix) {
            // @JsonValue byte[] wrapper: value is a hex string from JER
            c.push_str(&helpers::ln(3, &format!(
                "if (value instanceof String) {{\
                 this.{} = new {}();\
                 this.{}.value = {}.unhex((String) value);\
                 }}",
                fname, jt, fname, base
            )));
        } else {
            c.push_str(&helpers::ln(3, &format!("this.{} = MAPPER.convertValue(value, {});", fname, java_type_ref(&jt))));
        }
        c.push_str(&helpers::ln(2, "}"));
    }
    c.push_str(&helpers::ln(1, "}"));

    // encode + encodeTest
    helpers::gen_encode_methods(&mut c, cn, &native, &ti.name, "MAPPER.writeValueAsString(this)",
                                false, &[]);

    // decode
    helpers::gen_decode_method(&mut c, cn, &native, &ti.name);

    // Custom equals — compares only _choice + selected variant value
    c.push_str(&helpers::ln(1, "@Override"));
    c.push_str(&helpers::ln(1, "public boolean equals(Object o) {"));
    c.push_str(&helpers::ln(2, "if (this == o) return true;"));
    c.push_str(&helpers::ln(2, "if (o == null || getClass() != o.getClass()) return false;"));
    c.push_str(&helpers::ln(2, &format!("{} that = ({}) o;", cn, cn)));
    c.push_str(&helpers::ln(2, "if (!java.util.Objects.equals(_choice, that._choice)) return false;"));
    c.push_str(&helpers::ln(2, "if (_choice == null) return false;"));
    c.push_str(&helpers::ln(2, "String c = _choice;"));
    for v in variants {
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        let fname = safe_field_name(&v.name);
        c.push_str(&helpers::ln(2, &format!("if (\"{}\".equals(c)) return java.util.Objects.equals({}, that.{});", json_key, fname, fname)));
    }
    c.push_str(&helpers::ln(2, "return false;"));
    c.push_str(&helpers::ln(1, "}"));
    c.push_str(&helpers::ln(1, "@Override"));
    c.push_str(&helpers::ln(1, "public int hashCode() {"));
    c.push_str(&helpers::ln(2, "return java.util.Objects.hash(_choice);"));
    c.push_str(&helpers::ln(1, "}"));

    c.push_str("}\n");
    c
}
