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

    let mut bit_count: usize = 0; // >0 for FixedBitString types
    let mut has_optional = false;
    let mut field_names: Vec<(String, bool)> = Vec::new(); // (name, is_optional) for Struct types

    match &ti.kind {
        TypeKind::Newtype { inner_type } => {
            let py_type = helpers::resolve_py_type(inner_type, all, prefix);
            let default = helpers::py_default(&py_type);
            if py_type == "bytes" && inner_type.starts_with("FixedOctetString") {
                let size_str: String = inner_type.chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(size) = size_str.parse::<usize>() {
                    if size > 0 {
                        c.push_str(&format!("    value: bytes = b\"\\x00\" * {}\n", size));
                    } else { c.push_str("    value: bytes = b\"\"\n"); }
                } else {
                    eprintln!("WARN: cannot parse FixedOctetString size from '{}'", inner_type);
                    c.push_str(&format!("    value: {} = {}\n", py_type, default));
                }
            } else if inner_type.starts_with("FixedBitString") {
                let size_str: String = inner_type.chars().filter(|c| c.is_ascii_digit()).collect();
                bit_count = size_str.parse().unwrap_or(0);
                if bit_count > 0 {
                    c.push_str(&format!("    value: {} = {}\n", py_type, default));
                    c.push_str(&format!("    _bit_count: int = {}\n", bit_count));
                } else { c.push_str(&format!("    value: {} = {}\n", py_type, default)); }
            } else if py_type == "str" {
                let fixed_size = asn_defs.get(&ti.name)
                    .and_then(|def| super::super::java::helpers::parse_asn1_size(def))
                    .and_then(|(min, max)| if min == max && min.is_some() { min } else { None })
                    .unwrap_or(0);
                if fixed_size > 0 {
                    c.push_str(&format!("    value: {} = \"x\" * {}\n", py_type, fixed_size));
                } else { c.push_str(&format!("    value: {} = {}\n", py_type, default)); }
            } else if default == "None" && py_type != "Any" {
                c.push_str(&format!("    value: {} = field(default_factory=lambda: {}())\n", py_type, py_type));
            } else { c.push_str(&format!("    value: {} = {}\n", py_type, default)); }
        }
        TypeKind::Struct { fields } => {
            has_optional = fields.iter().any(|f| f.optional);
            for f in fields {
                let raw = f.identifier.as_deref().unwrap_or(&f.name);
                let name = helpers::py_safe_name(raw);
                field_names.push((name.clone(), f.optional));
                c.push_str(&helpers::gen_field(f, all, prefix));
                c.push('\n');
            }
            if has_optional {
                c.push_str("    _set: set[str] = field(default_factory=set, repr=False, compare=False)\n");
            }
        }
        TypeKind::Choice { variants } => {
            if let Some(first) = variants.first() {
                let first_name = helpers::py_safe_name(&first.name);
                let first_type = helpers::resolve_py_type(&first.inner_type, all, prefix);
                let first_default = helpers::py_default(&first_type);
                c.push_str(&format!("    _choice: str = \"{}\"\n", first_name));
                if first_default == "None" && first_type != "Any" {
                    c.push_str(&format!("    {}: {} = field(default_factory=lambda: {}())\n", first_name, first_type, first_type));
                } else { c.push_str(&format!("    {}: {} = {}\n", first_name, first_type, first_default)); }
            }
            for v in variants.iter().skip(1) {
                let name = helpers::py_safe_name(&v.name);
                let py_type = helpers::resolve_py_type(&v.inner_type, all, prefix);
                c.push_str(&format!("    {}: {} = None\n", name, py_type));
            }
        }
    }

    // Fluent setters (for Struct types) — use with_ prefix to avoid name collision
    for (fname, is_opt) in &field_names {
        c.push_str(&format!("\n    def with_{}(self, value):\n", fname));
        c.push_str(&format!("        self.{} = value\n", fname));
        if *is_opt {
            c.push_str("        if hasattr(self, '_set'):\n");
            c.push_str(&format!("            self._set.add(\"{}\")\n", fname));
        }
        c.push_str("        return self\n");
    }

    // Encode/Decode methods
    if bit_count > 0 {
        c.push_str("\n    def encode(self) -> bytes:\n");
        c.push_str(&format!("        return encode(json.dumps(bit_string_hex(self.value, {})), \"{}\")\n", bit_count, ti.name));
        c.push_str("\n    def encode_test(self) -> bytes:\n");
        c.push_str(&format!("        return encode(json.dumps(bit_string_hex(self.value, {})), \"{}\")\n", bit_count, ti.name));
        c.push_str(&format!("\n    @classmethod\n"));
        c.push_str(&format!("    def decode(cls, data: bytes) -> \"{cn}\":\n"));
        c.push_str(&format!("        return cls(value=parse_bit_string_hex(json.loads(decode_raw(data, \"{}\")).get(\"value\", \"0\"), {}))\n", ti.name, bit_count));
    } else {
        c.push_str("\n    def encode(self) -> bytes:\n");
        if has_optional && !field_names.is_empty() {
            let required: Vec<&str> = field_names.iter()
                .filter(|(_, opt)| !opt)
                .map(|(n, _)| n.as_str()).collect();
            if !required.is_empty() {
                c.push_str(&format!("        _required = {{{}}}\n", required.iter().map(|n| format!("\"{}\"", n)).collect::<Vec<_>>().join(", ")));
                c.push_str("        _missing = _required - self._set\n");
                c.push_str("        if _missing:\n");
                c.push_str("            raise ValueError(f\"Required fields not set: {_missing}\")\n");
            }
        }
        c.push_str(&format!("        return encode(to_json_strict(self), \"{}\")\n", ti.name));
        c.push_str("\n    def encode_test(self) -> bytes:\n");
        c.push_str(&format!("        return encode(to_json(self), \"{}\")\n", ti.name));
        c.push_str(&format!("\n    @classmethod\n"));
        c.push_str(&format!("    def decode(cls, data: bytes) -> \"{cn}\":\n"));
        c.push_str(&format!("        return from_json(decode_raw(data, \"{}\"), cls)\n", ti.name));
    }

    c
}
