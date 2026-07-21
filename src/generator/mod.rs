use std::collections::HashMap;
use syn::{Item, Type, Fields};

pub mod java;

#[derive(Debug)]
pub enum TypeKind {
    Newtype { inner_type: String },
    Struct { fields: Vec<FieldInfo> },
    Choice { variants: Vec<VariantInfo> },
}

#[derive(Debug)]
pub struct FieldInfo {
    pub name: String,
    pub rust_type: String,
    pub optional: bool,
    pub is_list: bool,
}

#[derive(Debug)]
pub struct VariantInfo {
    pub name: String,
    pub inner_type: String,
}

#[derive(Debug)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
}

pub fn prompt(msg: &str, default: &str) -> String {
    use std::io::{Write, BufRead};
    let full = if default.is_empty() {
        format!("{}: ", msg)
    } else {
        format!("{} [{}]: ", msg, default)
    };
    print!("{}", full);
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line).is_ok() {
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    default.to_string()
}

pub fn extract_types(ast: &syn::File) -> Vec<TypeInfo> {
    let mut types = Vec::new();
    if let Some(syn::Item::Mod(m)) = ast.items.first() {
        if let Some((_, items)) = &m.content {
            for inner in items {
                match inner {
                    Item::Struct(s) => types.push(TypeInfo {
                        name: s.ident.to_string(),
                        kind: analyze_struct(s),
                    }),
                    Item::Enum(e) => types.push(TypeInfo {
                        name: e.ident.to_string(),
                        kind: analyze_enum(e),
                    }),
                    _ => {}
                }
            }
        }
    }
    types
}

fn attr_contains(attrs: &[syn::Attribute], pat: &str) -> bool {
    use quote::ToTokens;
    attrs
        .iter()
        .any(|a| a.into_token_stream().to_string().contains(pat))
}

fn analyze_struct(s: &syn::ItemStruct) -> TypeKind {
    if attr_contains(&s.attrs, "delegate") {
        if let Fields::Unnamed(ref u) = s.fields {
            if let Some(f) = u.unnamed.first() {
                // Store raw Rust type string; language generators resolve it
                return TypeKind::Newtype {
                    inner_type: type_str(&f.ty),
                };
            }
        }
        return TypeKind::Newtype {
            inner_type: "int".into(),
        };
    }
    let mut fields = Vec::new();
    for f in s.fields.iter() {
        let name = f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
        let rt = type_str(&f.ty);
        let optional = rt.starts_with("Option <");
        let is_list = rt.contains("Vec <") || rt.contains("SequenceOf <");
        fields.push(FieldInfo {
            name,
            rust_type: rt,
            optional,
            is_list,
        });
    }
    TypeKind::Struct { fields }
}

fn analyze_enum(e: &syn::ItemEnum) -> TypeKind {
    let variants = e
        .variants
        .iter()
        .filter_map(|v| {
            if let Fields::Unnamed(ref u) = v.fields {
                u.unnamed.first().map(|f| VariantInfo {
                    name: v.ident.to_string(),
                    inner_type: type_str(&f.ty),
                })
            } else {
                None
            }
        })
        .collect();
    TypeKind::Choice { variants }
}

pub fn type_str(ty: &Type) -> String {
    quote::quote!(#ty)
        .to_string()
        .replace(" , ", ", ")
        .replace("  ", " ")
        .trim()
        .to_string()
}

/// Build an indented line (4 spaces per indent level)
pub fn ln(indent: usize, s: &str) -> String {
    format!("{}{}\n", " ".repeat(indent * 4), s)
}

/// Extract ASN.1 type definitions from a spec file.
/// Returns a map of type_name -> definition text (including the `::=` line).
/// For anonymous inline types (e.g. `AnonymousGetAllCBValuesResponsePDUCbValueValue`),
/// falls back to the parent ASN.1 type definition via substring matching.
pub fn extract_asn1_definitions(spec_path: &str, type_names: &[&str]) -> HashMap<String, String> {
    let src = match std::fs::read_to_string(spec_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };
    let lines: Vec<&str> = src.lines().collect();

    // First pass: collect all top-level ASN.1 type definitions (dash-stripped name → definition)
    let mut all_defs: HashMap<String, (usize, String)> = HashMap::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(eq_pos) = trimmed.find("::=") {
            let before_eq = trimmed[..eq_pos].trim();
            if before_eq.is_empty() || before_eq.contains(' ') {
                continue;
            }
            // Found a top-level type definition
            let content_start = eq_pos + 3;
            let after_eq = trimmed[content_start..].trim();

            let mut def_text = String::new();
            def_text.push_str(trimmed);
            def_text.push('\n');

            // Collect all ASN.1 type names (dash-stripped) for substring matching later
            let name_dashless = before_eq.replace('-', "");

            if after_eq.contains('{') {
                let mut depth: i32 = 1;
                for j in i + 1..lines.len() {
                    let l = lines[j];
                    let code = l.split("--").next().unwrap_or("");
                    let mut stop = false;
                    for ch in code.chars() {
                        if ch == '{' { depth += 1; }
                        else if ch == '}' {
                            depth -= 1;
                            if depth == 0 { stop = true; break; }
                        }
                    }
                    def_text.push_str(l);
                    def_text.push('\n');
                    if stop { break; }
                }
            }

            all_defs.insert(name_dashless, (i, def_text.trim().to_string()));
        }
    }

    // Pre-sort: longest names first so we find the most specific parent match
    let mut sorted_names: Vec<&str> = all_defs.keys().map(|s| s.as_str()).collect();
    sorted_names.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut defs = HashMap::new();

    for tn in type_names {
        // 1) Exact match (dash-stripped) — primary
        if let Some(def) = all_defs.get(*tn) {
            defs.insert(tn.to_string(), def.1.clone());
            continue;
        }
        // 2) Try literal (with dashes) — some types retain dashes in ASN.1
        let tn_dashless = tn.replace('-', "");
        if let Some(def) = all_defs.get(&tn_dashless) {
            defs.insert(tn.to_string(), def.1.clone());
            continue;
        }

        // 3) Anonymous types: find best parent match by substring
        //    e.g. AnonymousGetAllCBValuesResponsePDUCbValueValue contains GetAllCBValuesResponsePDU
        if tn.starts_with("Anonymous") {
            for parent in &sorted_names {
                if tn.contains(parent) {
                    defs.insert(tn.to_string(), format!(
                        "(inline type within {})",
                        all_defs.get(*parent).unwrap().1.lines().next().unwrap_or("")
                    ));
                    break;
                }
            }
        }
    }

    defs
}

/// Convert dash-separated name to UPPER_SNAKE_CASE (e.g. "data-change" → "DATA_CHANGE")
pub fn constant_name(s: &str) -> String {
    s.to_uppercase().replace('-', "_")
}

/// Extract named constants from BIT STRING / ENUMERATED definitions in the ASN.1 spec.
/// Returns a map of type_name -> [(Java constant name, value)].
pub fn extract_asn1_named_constants(spec_path: &str) -> HashMap<String, Vec<(String, i32)>> {
    let src = match std::fs::read_to_string(spec_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };
    let lines: Vec<&str> = src.lines().collect();
    let mut result = HashMap::new();

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        // Look for `TypeName ::= BIT STRING {` or `TypeName ::= ENUMERATED {`
        if let Some(before_brace) = trimmed.split('{').next() {
            let before_eq = before_brace.split("::=").next().unwrap_or("").trim();
            let after_eq = before_brace.split("::=").nth(1).unwrap_or("").trim();
            if before_eq.is_empty() || !after_eq.contains("BIT STRING") && !after_eq.contains("ENUMERATED") {
                i += 1;
                continue;
            }
            // found: `TypeName ::= BIT STRING {`
            let type_name = before_eq;
            let mut constants = Vec::new();
            i += 1;
            while i < lines.len() {
                let line = lines[i].split("--").next().unwrap_or("").trim(); // strip ASN.1 comments
                if line.contains('}') {
                    break;
                }
                // Parse: `name (number),`
                if let Some(paren_start) = line.find('(') {
                    let name = line[..paren_start].trim();
                    let after_paren = &line[paren_start + 1..];
                    if let Some(paren_end) = after_paren.find(')') {
                        if let Ok(val) = after_paren[..paren_end].trim().parse::<i32>() {
                            if !name.is_empty() {
                                constants.push((constant_name(name), val));
                            }
                        }
                    }
                }
                i += 1;
            }
            if !constants.is_empty() {
                result.insert(type_name.to_string(), constants);
            }
        }
        i += 1;
    }
    result
}
