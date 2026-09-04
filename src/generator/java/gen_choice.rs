use super::super::*;
use super::gen_newtype_common::render_sample_factory_choice;
use super::helpers;
use super::type_map::resolve_wrapper_type;
use std::collections::HashMap;

/// Choice variant value expression — shared by the no-arg constructor
/// (unset default: 0/""/false) and sample() (non-zero test filler).
/// User-typed variants: empty instance by default; sample() recurses into
/// `X.sample()._v` unless the type is cyclic (infinite expansion guard).
fn choice_variant_value(
    jt: &str,
    prefix: &str,
    sample: bool,
    recursive: &std::collections::HashSet<String>,
) -> String {
    match jt {
        "int" => if sample { "1".to_string() } else { "0".to_string() },
        "long" => if sample { "1L".to_string() } else { "0L".to_string() },
        "float" => if sample { "1.5f".to_string() } else { "0f".to_string() },
        "double" => if sample { "2.5".to_string() } else { "0".to_string() },
        "String" => if sample { "\"x\"".to_string() } else { "\"\"".to_string() },
        "byte[]" => if sample { "new byte[1]".to_string() } else { "new byte[0]".to_string() },
        "boolean" => if sample { "true".to_string() } else { "false".to_string() },
        _ if jt.starts_with(prefix) => {
            if sample && !recursive.contains(jt) {
                format!("{}.sample()._v", jt)
            } else {
                format!("new {}()._v", jt)
            }
        }
        _ => format!("new {}()", jt),
    }
}

pub fn generate(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    cn: &str,
    asn_doc: &Option<String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
    variants: &[VariantInfo],
    recursive: &std::collections::HashSet<String>,
) -> String {
    let mut c = String::new();
    let base = format!("{}Base", prefix);
    if let Some(doc) = asn_doc {
        c.push_str(doc);
    }
    c.push_str("@JsonIgnoreProperties(ignoreUnknown = true)\n");
    c.push_str("@JsonInclude(JsonInclude.Include.NON_NULL)\n");
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(
                1,
                &format!("public static final int {} = {};", name, val),
            ));
        }
    }

    // No-arg constructor picks the first variant as default. Nested SEQUENCE
    // field defaults (e.g. `ctlVal` → new InnerData()._v) rely on this seed so
    // a fresh instance is always encodeable. Data lives in _v:
    // {"_choice": "variantName", "_": value}.
    if let Some(first) = variants.first() {
        let json_key = first.identifier.as_deref().unwrap_or(&first.name);
        let jt = resolve_wrapper_type(&first.inner_type, all, prefix);
        let init_val = choice_variant_value(&jt, prefix, false, recursive);
        c.push_str(&helpers::ln(1, &format!("public {}() {{", cn)));
        c.push_str(&helpers::ln(
            2,
            &format!("_v.put(\"_choice\", \"{}\");", json_key),
        ));
        c.push_str(&helpers::ln(2, &format!("_v.put(\"_\", {});", init_val)));
        c.push_str(&helpers::ln(1, "}"));
    }

    // Defensive guard: encode() throws if "_choice" is missing (e.g. after an
    // explicit _v.clear()), instead of silently emitting an empty CHOICE.
    c.push_str(&helpers::ln(1, "@Override"));
    c.push_str(&helpers::ln(
        1,
        "protected boolean isChoice() { return true; }",
    ));

    // Typesafe setters — set _choice + variant value in _v
    for v in variants {
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        let setter_name = format!("set{}{}", &v.name[..1].to_uppercase(), &v.name[1..]);
        // Use Object for all setter params — Jackson passes the raw JSON value,
        // and _setValue wraps non-Map values in {"_": v} so _v is always consistent.
        c.push_str(&helpers::ln(1, &format!("@JsonSetter(\"{}\")", json_key)));
        c.push_str(&helpers::ln(
            1,
            &format!("public void {}(Object v) {{", setter_name),
        ));
        c.push_str(&helpers::ln(
            2,
            &format!("_v.put(\"_choice\", \"{}\");", json_key),
        ));
        c.push_str(&helpers::ln(2, "_setValue(v);"));
        c.push_str(&helpers::ln(1, "}"));
    }

    // encode()/encodeTest() inherited from {base} — the base implementation
    // (toJson(_v) + InnerNative.encode) already handles the CHOICE shape.

    // typeName — ASN.1 dispatch name for native encode/decode
    c.push_str(&helpers::ln(1, "@Override"));
    c.push_str(&helpers::ln(
        1,
        &format!("protected String typeName() {{ return \"{}\"; }}", ti.name),
    ));

    // decode
    c.push_str(&helpers::ln(
        1,
        &format!("public static {} decode(byte[] data) {{", cn),
    ));
    c.push_str(&helpers::ln(
        2,
        &format!(
            "return {}.decode({}.class, \"{}\", data);",
            base, cn, ti.name
        ),
    ));
    c.push_str(&helpers::ln(1, "}"));

    // sample() — test-filler factory (first variant, non-zero values)
    if let Some(first) = variants.first() {
        let json_key = first.identifier.as_deref().unwrap_or(&first.name);
        let jt = resolve_wrapper_type(&first.inner_type, all, prefix);
        let sample_val = choice_variant_value(&jt, prefix, true, recursive);
        c.push_str(&render_sample_factory_choice(cn, json_key, &sample_val));
    }

    c.push_str("}\n");
    c
}
