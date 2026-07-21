use std::collections::HashMap;
use super::super::*;
use super::helpers;
use super::test_newtype;
use super::test_struct;
use super::test_choice;

/// Dispatch to the correct test generator based on TypeKind.
pub fn gen_test_class(
    ti: &TypeInfo,
    all: &[TypeInfo],
    prefix: &str,
    package: &str,
    _asn_defs: &HashMap<String, String>,
) -> String {
    let cn = format!("{}{}", prefix, ti.name);
    let pkg = if package.is_empty() { String::new() }
              else { format!("package {};\n\n", package) };
    let mut c = String::new();
    c.push_str(&format!("// Auto-generated. Tests for {}\n\n", cn));
    c.push_str(&pkg);
    c.push_str("import org.junit.Test;\n");
    c.push_str("import com.fasterxml.jackson.databind.ObjectMapper;\n");
    c.push_str("import static org.junit.Assert.*;\n\n");
    c.push_str(&format!("public class {}Test {{\n\n", cn));
    c.push_str(&helpers::ln(1, "private static final ObjectMapper MAPPER = new ObjectMapper();"));
    c.push('\n');

    match &ti.kind {
        TypeKind::Newtype { .. } => {
            c.push_str(&test_newtype::generate(ti, all, prefix, &cn));
        }
        TypeKind::Struct { fields } => {
            c.push_str(&test_struct::generate(ti, all, prefix, &cn, fields));
        }
        TypeKind::Choice { variants } => {
            c.push_str(&test_choice::generate(ti, all, prefix, &cn, variants));
        }
    }

    c.push_str("}\n");
    c
}
