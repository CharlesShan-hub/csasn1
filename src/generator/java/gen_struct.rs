use std::collections::HashMap;
use super::super::*;
use super::type_map::{resolve_wrapper_type, resolve_wrapper_type_nullable};
use super::helpers;
use super::helpers::{safe_field_name, jdefault};

pub fn generate(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    cn: &str,
    asn_doc: &Option<String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
    fields: &[FieldInfo],
    _asn_defs: &HashMap<String, String>,
) -> String {
    let mut c = String::new();
    let base = format!("{}Base", prefix);
    let native = format!("{}Native", prefix);
    let has_optional = fields.iter().any(|f| f.optional);
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
    c.push_str(&helpers::ln(1, &format!("private static final ObjectMapper MAPPER = {}.createMapper();", base)));

    for f in fields {
        let jt = if f.optional {
            resolve_wrapper_type_nullable(&f.rust_type, all, prefix)
        } else {
            resolve_wrapper_type(&f.rust_type, all, prefix)
        };
        let raw_name = f.identifier.as_deref().unwrap_or(&f.name);
        let fname = safe_field_name(raw_name);
        let dflt = match jt.as_str() {
            "byte[]" => {
                let is_fixed = f.size_attr_raw.as_deref()
                    .and_then(|r| r.parse::<usize>().ok())
                    .is_some();
                if is_fixed {
                    if let Some(n) = f.size_from_attr {
                        format!("new byte[{}]", n)
                    } else {
                        "new byte[0]".to_string()
                    }
                } else {
                    "new byte[0]".to_string()
                }
            }
            "String" => {
                let is_fixed = f.size_attr_raw.as_deref()
                    .and_then(|r| r.parse::<usize>().ok())
                    .is_some();
                if is_fixed {
                    if let Some(n) = f.size_from_attr {
                        format!("\"{}\"", "x".repeat(n))
                    } else {
                        "\"\"".to_string()
                    }
                } else {
                    "\"\"".to_string()
                }
            }
            _ => jdefault(&jt, f.is_list),
        };
        if fname != raw_name {
            c.push_str(&helpers::ln(1, &format!("@JsonProperty(\"{}\") public {} {} = {};", raw_name, jt, fname, dflt)));
        } else {
            c.push_str(&helpers::ln(1, &format!("public {} {} = {};", jt, fname, dflt)));
        }
    }

    // _set tracking for OPTIONAL fields
    if has_optional {
        c.push_str(&helpers::ln(1, "@com.fasterxml.jackson.annotation.JsonIgnore"));
        c.push_str(&helpers::ln(1, "@lombok.EqualsAndHashCode.Exclude"));
        c.push_str(&helpers::ln(1, "public transient java.util.Set<String> _set = new java.util.HashSet<>();"));
        // Fluent setter for each OPTIONAL field (tracks _set)
        for f in fields {
            if !f.optional { continue; }
            let raw_name = f.identifier.as_deref().unwrap_or(&f.name);
            let fname = safe_field_name(raw_name);
            let jt = resolve_wrapper_type_nullable(&f.rust_type, all, prefix);
            c.push_str(&helpers::ln(1, &format!("public {} {}({} value) {{", cn, fname, jt)));
            c.push_str(&helpers::ln(2, &format!("this.{} = value;", fname)));
            c.push_str(&helpers::ln(2, &format!("this._set.add(\"{}\");", raw_name)));
            c.push_str(&helpers::ln(2, "return this;"));
            c.push_str(&helpers::ln(1, "}"));
        }
    }

    // Collect OPTIONAL field names for strict encode filtering
    let opt_names: Vec<&str> = fields.iter()
        .filter(|f| f.optional)
        .map(|f| f.identifier.as_deref().unwrap_or(&f.name))
        .collect();

    // encode + encodeTest + decode
    helpers::gen_encode_methods(&mut c, cn, &native, &ti.name, "MAPPER.writeValueAsString(this)",
                                has_optional, &opt_names);
    helpers::gen_decode_method(&mut c, cn, &native, &ti.name);
    c.push_str("}\n");
    c
}
