use std::collections::HashMap;
use super::super::*;
use super::type_map::resolve_java_type;
use super::gen_newtype;
use super::gen_struct;
use super::gen_choice;

/// Dispatch to the correct type-specific generator based on TypeKind.
pub fn gen_class(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    _default_enc: &str,
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

    match &ti.kind {
        TypeKind::Newtype { .. } => {
            let jt = resolve_java_type(&ti.name, all, prefix);
            let jt = if jt == cn {
                resolve_java_type(
                    match &ti.kind {
                        TypeKind::Newtype { inner_type } => inner_type,
                        _ => unreachable!(),
                    },
                    all, prefix,
                )
            } else { jt };
            c.push_str(&gen_newtype::generate(ti, all, prefix, &cn, &asn_doc, asn_defs, named_consts, &jt));
        }
        TypeKind::Struct { fields } => {
            c.push_str(&gen_struct::generate(ti, all, prefix, &cn, &asn_doc, named_consts, fields, asn_defs));
        }
        TypeKind::Choice { variants } => {
            c.push_str(&gen_choice::generate(ti, all, prefix, &cn, &asn_doc, named_consts, variants));
        }
    }
    c
}
