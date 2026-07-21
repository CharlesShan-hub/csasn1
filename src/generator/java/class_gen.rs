use std::collections::HashMap;
use super::super::*;
use super::{resolve_java_type, resolve_java_type_nullable, java_type_ref, safe_field_name, jdefault};

pub fn gen_class(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    default_enc: &str,
    package: &str,
    asn_defs: &HashMap<String, String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
) -> String {
    let cn = format!("{}{}", prefix, ti.name);
    let mut c = String::new();
    c.push_str(&format!("// Auto-generated. ASN.1 type: {}\n\n", ti.name));
    if !package.is_empty() {
        c.push_str(&format!("package {};\n\n", package));
    }
    c.push_str("import com.fasterxml.jackson.annotation.*;\n");
    c.push_str("import com.fasterxml.jackson.databind.*;\n");
    c.push_str("import lombok.Data;\n\n");

    // Build ASN.1 definition doc comment (emitted right before class declaration)
    let asn_doc = asn_defs.get(&ti.name).map(|def| {
        let mut d = format!("/**\n");
        d.push_str(" * <pre>{@code\n");
        for line in def.lines() {
            d.push_str(&format!(" * {}\n", line));
        }
        d.push_str(" * }</pre>\n");
        d.push_str(" */\n");
        d
    });

    let enc_overload = |c: &mut String, body: &str| {
        c.push_str(&ln(1, "public byte[] encode(String enc) {"));
        c.push_str(body);
        c.push_str(&ln(1, "}"));
        c.push_str(&ln(1, "public byte[] encode() {"));
        c.push_str(&ln(2, "return encode(DEFAULT_ENCODING);"));
        c.push_str(&ln(1, "}"));
    };
    let gen_named_consts = |c: &mut String| {
        if let Some(entries) = named_consts.get(&ti.name) {
            for (name, val) in entries {
                c.push_str(&ln(1, &format!("public static final int {} = {};", name, val)));
            }
        }
    };

    match &ti.kind {
        TypeKind::Newtype { inner_type } => {
            let jt = resolve_java_type(&ti.name, all, prefix);
            // If self-referential (type name == class name), resolve the inner raw type
            let jt = if jt == cn {
                resolve_java_type(inner_type, all, prefix)
            } else {
                jt
            };

            if let Some(ref doc) = asn_doc { c.push_str(doc); }
            c.push_str("@Data\n");
            c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
            gen_named_consts(&mut c);
            c.push_str(&ln(
                1,
                "private static final ObjectMapper MAPPER = new ObjectMapper();",
            ));
            c.push_str(&ln(1, &format!("@JsonProperty public {} value;", jt)));
            c.push_str(&ln(1, &format!("public {}() {{}}", cn)));
            c.push_str(&ln(
                1,
                &format!("public {}({} value) {{ this.value = value; }}", cn, jt),
            ));
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
                c.push_str(&ln(1, "public byte[] encode(String enc) {"));
                c.push_str(&ln(2, "try {"));
                c.push_str(&ln(
                    3,
                    &format!(
                        "return CmsNative.encode(\"{}\", enc, {});",
                        ti.name, encode_arg
                    ),
                ));
                c.push_str(&ln(2, "} catch (Exception e) {"));
                c.push_str(&ln(3, "throw new RuntimeException(e);"));
                c.push_str(&ln(2, "}"));
                c.push_str(&ln(1, "}"));
                c.push_str(&ln(1, "public byte[] encode() {"));
                c.push_str(&ln(2, "return encode(DEFAULT_ENCODING);"));
                c.push_str(&ln(1, "}"));
            } else {
                enc_overload(
                    &mut c,
                    &ln(
                        2,
                        &format!(
                            "return CmsNative.encode(\"{}\", enc, {});",
                            ti.name, encode_arg
                        ),
                    ),
                );
            }
            c.push_str(&ln(
                1,
                &format!(
                    "public static {} decode(String enc, byte[] data) {{",
                    cn
                ),
            ));
            c.push_str(&ln(2, "try {"));
            c.push_str(&ln(
                3,
                &format!(
                    "String json = CmsNative.decode(\"{}\", enc, data);",
                    ti.name
                ),
            ));
            c.push_str(&ln(3, &format!("{} r = new {}();", cn, cn)));
            if jt.starts_with("java.util.List<") {
                let inner = jt
                    .trim_start_matches("java.util.List<")
                    .trim_end_matches('>')
                    .trim();
                c.push_str(&ln(
                    3,
                    &format!(
                        "r.value = MAPPER.readValue(json.trim(), new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}});",
                        inner
                    ),
                ));
            } else if jt == "byte[]" {
                c.push_str(&ln(
                    3,
                    "r.value = java.util.Base64.getDecoder().decode(json.trim());",
                ));
            } else if jt == "Object" {
                c.push_str(&ln(3, "r.value = null;"));
            } else if jt == "long" {
                c.push_str(&ln(
                    3,
                    "r.value = Long.parseLong(json.trim());",
                ));
            } else if jt == "String" {
                c.push_str(&ln(
                    3,
                    "r.value = json.trim();",
                ));
            } else if jt == "boolean" {
                c.push_str(&ln(
                    3,
                    "r.value = Boolean.parseBoolean(json.trim());",
                ));
            } else if jt == "float" {
                c.push_str(&ln(
                    3,
                    "r.value = Float.parseFloat(json.trim());",
                ));
            } else if jt == "double" {
                c.push_str(&ln(
                    3,
                    "r.value = Double.parseDouble(json.trim());",
                ));
            } else if jt == "int" {
                c.push_str(&ln(
                    3,
                    "r.value = Integer.parseInt(json.trim());",
                ));
            } else {
                // Custom type — use ObjectMapper for JSON deserialization
                c.push_str(&ln(
                    3,
                    &format!(
                        "r.value = MAPPER.readValue(json.trim(), {}.class);",
                        jt
                    ),
                ));
            }
            c.push_str(&ln(3, "return r;"));
            c.push_str(&ln(2, "} catch (Exception e) {"));
            c.push_str(&ln(3, "throw new RuntimeException(e);"));
            c.push_str(&ln(2, "}"));
            c.push_str(&ln(1, "}"));
            c.push_str("}\n");
        }

        TypeKind::Struct { fields } => {
            let has_optional = fields.iter().any(|f| f.optional);
            if let Some(ref doc) = asn_doc { c.push_str(doc); }
            c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
            if has_optional {
                c.push_str("@JsonInclude(JsonInclude.Include.NON_NULL)\n");
            }
            c.push_str("@Data\n");
            c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
            gen_named_consts(&mut c);
            c.push_str(&ln(
                1,
                "private static final ObjectMapper MAPPER = new ObjectMapper();",
            ));

            for f in fields {
                let jt = if f.optional {
                    resolve_java_type_nullable(&f.rust_type, all, prefix)
                } else {
                    resolve_java_type(&f.rust_type, all, prefix)
                };
                let fname = safe_field_name(&f.name);
                let dflt = jdefault(&jt, f.is_list);
                c.push_str(&ln(
                    1,
                    &format!("@JsonProperty public {} {} = {};", jt, fname, dflt),
                ));
            }

            // encode
            enc_overload(
                &mut c,
                &format!(
                    "{}{}{}{}{}{}",
                    ln(2, "try {"),
                    ln(
                        3,
                        &format!(
                            "return CmsNative.encode(\"{}\", enc,",
                            ti.name
                        ),
                    ),
                    ln(4, "MAPPER.writeValueAsString(this));"),
                    ln(2, "} catch (Exception e) {"),
                    ln(3, "throw new RuntimeException(e);"),
                    ln(2, "}"),
                ),
            );

            // decode
            c.push_str(&ln(
                1,
                &format!(
                    "public static {} decode(String enc, byte[] data) {{",
                    cn
                ),
            ));
            c.push_str(&ln(2, "try {"));
            c.push_str(&ln(3, &format!(
                "return MAPPER.readValue(CmsNative.decode(\"{}\", enc, data), {}.class);",
                ti.name, cn
            )));
            c.push_str(&ln(2, "} catch (Exception e) {"));
            c.push_str(&ln(3, "throw new RuntimeException(e);"));
            c.push_str(&ln(2, "}"));
            c.push_str(&ln(1, "}"));
            c.push_str("}\n");
        }

        TypeKind::Choice { variants } => {
            if let Some(ref doc) = asn_doc { c.push_str(doc); }
            c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
            c.push_str("@Data\n");
            c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
            gen_named_consts(&mut c);
            c.push_str(&ln(1, "public String _choice;"));
            c.push_str(&ln(
                1,
                "private static final ObjectMapper MAPPER = new ObjectMapper();",
            ));

            for v in variants {
                let jt = resolve_java_type(&v.inner_type, all, prefix);
                let fname = safe_field_name(&v.name);
                c.push_str(&ln(
                    1,
                    &format!("@JsonIgnore public {} {};", jt, fname),
                ));
            }

            // Serialization: only output the active branch (handle null _choice)
            c.push_str(&ln(1, "@JsonAnyGetter"));
            c.push_str(&ln(
                1,
                "public java.util.Map<String, Object> serializeChoice() {",
            ));
            c.push_str(&ln(2, "java.util.Map<String, Object> map = new java.util.HashMap<String, Object>();"));
            c.push_str(&ln(2, "if (_choice != null) {"));
            c.push_str(&ln(3, "map.put(\"_choice\", _choice);"));
            for v in variants {
                let fname = safe_field_name(&v.name);
                c.push_str(&ln(
                    3,
                    &format!(
                        "if (\"{}\".equals(_choice)) map.put(\"{}\", {});",
                        v.name, v.name, fname
                    ),
                ));
            }
            c.push_str(&ln(2, "}"));
            c.push_str(&ln(2, "return map;"));
            c.push_str(&ln(1, "}"));

            // Deserialization
            c.push_str(&ln(1, "@JsonAnySetter"));
            c.push_str(&ln(
                1,
                "public void deserializeChoice(String key, Object value) {",
            ));
            c.push_str(&ln(2, "if (\"_choice\".equals(key)) return;"));
            c.push_str(&ln(2, "this._choice = key;"));
            for v in variants {
                let fname = safe_field_name(&v.name);
                let jt = resolve_java_type(&v.inner_type, all, prefix);
                let tref = java_type_ref(&jt);
                c.push_str(&ln(
                    2,
                    &format!("if (\"{}\".equals(key)) {{", v.name),
                ));
                c.push_str(&ln(
                    3,
                    &format!(
                        "this.{} = MAPPER.convertValue(value, {});",
                        fname, tref
                    ),
                ));
                c.push_str(&ln(2, "}"));
            }
            c.push_str(&ln(1, "}"));

            // encode
            enc_overload(
                &mut c,
                &format!(
                    "{}{}{}{}{}",
                    ln(2, "try {"),
                    ln(
                        3,
                        &format!(
                            "return CmsNative.encode(\"{}\", enc, MAPPER.writeValueAsString(this));",
                            ti.name
                        ),
                    ),
                    ln(2, "} catch (Exception e) {"),
                    ln(3, "throw new RuntimeException(e);"),
                    ln(2, "}"),
                ),
            );

            // decode
            c.push_str(&ln(
                1,
                &format!(
                    "public static {} decode(String enc, byte[] data) {{",
                    cn
                ),
            ));
            c.push_str(&ln(2, "try {"));
            c.push_str(&ln(
                3,
                &format!(
                    "return MAPPER.readValue(CmsNative.decode(\"{}\", enc, data), {}.class);",
                    ti.name, cn
                ),
            ));
            c.push_str(&ln(2, "} catch (Exception e) {"));
            c.push_str(&ln(3, "throw new RuntimeException(e);"));
            c.push_str(&ln(2, "}"));
            c.push_str(&ln(1, "}"));
            c.push_str("}\n");
        }
    }
    c
}
