use super::super::*;
use super::gen_newtype_common::{
    build_decode_puts, build_encode_arg, default_val_for, render_ctor_bitstring,
    render_ctor_unsigned, render_decode, render_encode_plain, render_encode_wrapped,
    render_sample_factory, sample_value_for,
};
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
    let inner_bit_string = matches!(&ti.kind,
        TypeKind::Newtype { inner_type, .. }
            if inner_type.starts_with("FixedBitString") || inner_type.starts_with("BitString"));

    let inner_unsigned_int = matches!(&ti.kind,
        TypeKind::Newtype { inner_type, .. } if inner_type == "u32");

    let (hex_digits, bit_count) = if inner_bit_string || named_consts.contains_key(&ti.name) {
        if let Some(def) = asn_defs.get(&ti.name) {
            if let Some((Some(min), Some(max))) = helpers::parse_asn1_size(def) {
                let bits = max.max(min);
                let hd = std::cmp::max(2, ((bits + 7) / 8) * 2);
                (hd, bits)
            } else { (2, 8) }
        } else { (2, 8) }
    } else { (0, 0) };

    let size = match &ti.kind {
        TypeKind::Newtype { size_from_attr, .. } => {
            size_from_attr.unwrap_or_else(|| helpers::resolve_size(&ti.name, asn_defs))
        }
        _ => helpers::resolve_size(&ti.name, asn_defs),
    };

    let base = format!("{}Base", prefix);
    let native = format!("{}Native", prefix);

    // ── Resolve TypeSpec for Rust inner type (JSON-driven strategies) ──
    let rust_inner = match &ti.kind {
        TypeKind::Newtype { inner_type, .. } => {
            inner_type.split('<').next().unwrap_or(inner_type).to_string()
        }
        _ => String::new(),
    };
    let spec = crate::generator::java::type_registry::lookup(&rust_inner);

    let mut c = String::new();
    if let Some(doc) = asn_doc {
        c.push_str(doc);
    }
    c.push_str(&format!("public class {} extends {}Base {{\n", cn, prefix));
    if let Some(entries) = named_consts.get(&ti.name) {
        for (name, val) in entries {
            c.push_str(&helpers::ln(1, &format!("public static final int {} = {};", name, val)));
        }
    }

    // ── Constructor block ──
    if hex_digits > 0 {
        c.push_str(&render_ctor_bitstring(cn, jt, &base, hex_digits, bit_count));
    } else if inner_unsigned_int {
        c.push_str(&render_ctor_unsigned(cn));
    } else {
        let default_val = default_val_for(jt, size, spec, prefix);
        if default_val.is_empty() {
            c.push_str(&helpers::ln(1, &format!("public {}() {{}}", cn)));
        } else {
            c.push_str(&helpers::ln(1, &format!("public {}() {{ _v.put(\"_\", {}); }}", cn, default_val)));
        }
        let json_creator_type = if jt.starts_with("java.util.List<") {
                "Object".to_string()
            } else if let Some(s) = spec {
                s.creator.clone()
            } else {
                match jt {
                    "int" | "Integer" => "int",
                    "long" | "Long" => "long",
                    "boolean" | "Boolean" => "boolean",
                    "float" | "Float" | "double" | "Double" => "double",
                    _ => "String",
                }.to_string()
            };
        let json_creator_body = if jt.starts_with("java.util.List<") {
            let inner = jt.trim_start_matches("java.util.List<").trim_end_matches('>').trim();
            format!(
                "{{ {} r = new {}(); r._v.put(\"_\", {}.MAPPER.convertValue(v, new com.fasterxml.jackson.core.type.TypeReference<java.util.List<{}>>() {{}})); return r; }}",
                cn, cn, base, inner
            )
        } else if jt == "byte[]" {
            format!("{{ {} r = new {}(); r._v.put(\"_\", {}.unhex(v)); return r; }}", cn, cn, base)
        } else {
            format!("{{ {} r = new {}(); r._v.put(\"_\", v); return r; }}", cn, cn)
        };
        c.push_str(&helpers::ln(1, "@JsonCreator"));
        c.push_str(&helpers::ln(1, &format!("public static {} fromJson({} v) {}", cn, json_creator_type, json_creator_body)));
        let ctor_type = if let Some(s) = spec {
                s.ctor.clone()
            } else {
                match jt {
                    "Integer" | "int" => "Integer",
                    "Long" | "long" => "Long",
                    "Boolean" | "boolean" => "Boolean",
                    "Float" | "float" => "Float",
                    "Double" | "double" => "Double",
                    "String" => "String",
                    _ => jt,
                }.to_string()
            };
        if ctor_type != jt || !jt.starts_with("DefaultInner") {
            c.push_str(&helpers::ln(1, &format!("public {}({} v) {{ this(); _v.put(\"_\", v); }}", cn, ctor_type)));
        }
        c.push_str(&helpers::ln(1, "@JsonValue"));
        c.push_str(&helpers::ln(1, "public Object toJsonValue() { return _v.get(\"_\"); }"));
    }

    // ── Encode / decode ──
    // typeName — ASN.1 dispatch name (kept in sync with struct/choice classes)
    c.push_str(&helpers::ln(1, "@Override"));
    c.push_str(&helpers::ln(
        1,
        &format!("protected String typeName() {{ return \"{}\"; }}", ti.name),
    ));

    let inner_octet_string = matches!(&ti.kind,
        TypeKind::Newtype { inner_type, .. }
            if inner_type.starts_with("OctetString") || inner_type.starts_with("FixedOctetString"));

    let (encode_arg, wrap_try) = build_encode_arg(jt, &base, hex_digits, inner_unsigned_int, prefix, spec);
    if wrap_try {
        c.push_str(&render_encode_wrapped(&native, &ti.name, &encode_arg));
    } else {
        c.push_str(&render_encode_plain(&native, &ti.name, &encode_arg));
    }

    let put_lines: Vec<String> = build_decode_puts(jt, &base, hex_digits, inner_octet_string, spec)
        .iter()
        .map(|l| helpers::ln(3, l))
        .collect();
    c.push_str(&render_decode(cn, &native, &ti.name, &base, &put_lines));

    // ── sample() test-filler factory ──
    let sample_expr = if hex_digits > 0 {
        format!("\"{}\"", "0".repeat(hex_digits))
    } else if inner_unsigned_int {
        "1".to_string()
    } else {
        sample_value_for(jt, size, spec, prefix)
    };
    c.push_str(&render_sample_factory(cn, &sample_expr));

    c.push_str("}\n");
    c
}
