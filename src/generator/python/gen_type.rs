use std::collections::HashMap;
use super::super::*;
use super::helpers;

/// Generate a complete Python class for each ASN.1 type
pub fn gen_type_class(ti: &TypeInfo, all: &[TypeInfo], prefix: &str, _package: &str,
                      asn_defs: &HashMap<String, String>) -> String {
    let cn = format!("{}{}", prefix, ti.name);
    let mut c = String::new();

    c.push_str(&format!("@dataclass\n"));
    c.push_str(&format!("class {}:\n", cn));
    // Docstring from ASN.1 definition
    if let Some(def) = asn_defs.get(&ti.name) {
        c.push_str("    \"\"\"\n");
        c.push_str("    ```asn1\n");
        for line in def.lines() {
            c.push_str(&format!("    {}\n", line));
        }
        c.push_str("    ```\n");
        c.push_str("    \"\"\"\n");
    }

    match &ti.kind {
        TypeKind::Newtype { inner_type } => {
            let py_type = helpers::resolve_py_type(inner_type, all, prefix);
            let default = helpers::py_default(&py_type);
            c.push_str(&format!("    value: {} = {}\n", py_type, default));
        }
        TypeKind::Struct { fields } => {
            let has_optional = fields.iter().any(|f| f.optional);
            for f in fields {
                c.push_str(&helpers::gen_field(f, all, prefix));
                c.push('\n');
            }
            if has_optional {
                c.push_str("    _set: set[str] = field(default_factory=set, repr=False, compare=False)\n");
            }
        }
        TypeKind::Choice { variants } => {
            c.push_str("    _choice: str = \"\"\n");
            for v in variants {
                let name = helpers::py_safe_name(&v.name);
                let py_type = helpers::resolve_py_type(&v.inner_type, all, prefix);
                c.push_str(&format!("    {}: {} = None\n", name, py_type));
            }
        }
    }

    // Encode method
    c.push_str("\n    def encode(self) -> bytes:\n");
    c.push_str(&format!("        return encode(to_json(self), \"{}\")\n", ti.name));
    // Encode test method
    c.push_str("\n    def encode_test(self) -> bytes:\n");
    c.push_str(&format!("        return encode(to_json(self), \"{}\")\n", ti.name));
    // Decode classmethod
    c.push_str(&format!("\n    @classmethod\n"));
    c.push_str(&format!("    def decode(cls, data: bytes) -> \"{cn}\":\n"));
    c.push_str(&format!("        return from_json(decode_raw(data, \"{}\"), cls)\n", ti.name));

    c
}
