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
pub fn extract_asn1_definitions(spec_path: &str, type_names: &[&str]) -> HashMap<String, String> {
    let src = match std::fs::read_to_string(spec_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };
    let lines: Vec<&str> = src.lines().collect();
    let mut defs = HashMap::new();

    for tn in type_names {
        // Find the line containing `TypeName ::=`
        // Rust strips dashes from ASN.1 names (e.g. GetAllCBValues-RequestPDU → GetAllCBValuesRequestPDU),
        // so match both the literal name and the dash-stripped form.
        let mut start: Option<usize> = None;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let before_eq = trimmed.split("::=").next().unwrap_or("").trim();
            if before_eq == *tn || before_eq.replace('-', "") == *tn {
                start = Some(i);
                break;
            }
        }

        let s = match start {
            None => continue,
            Some(idx) => idx,
        };

        let def_line = lines[s];
        let trimmed = def_line.trim();

        // Find the content after `::=`
        let content_start = trimmed.find("::=").map(|p| p + 3).unwrap_or(0);
        let after_eq = &trimmed[content_start..].trim();

        let mut def_text = String::new();
        def_text.push_str(trimmed);
        def_text.push('\n');

        // If definition continues with `{`, find matching closing brace
        if after_eq.contains('{') {
            let mut depth: i32 = 1; // the `{` on the first line already opened one level
            for j in s + 1..lines.len() {
                let l = lines[j];
                // Strip ASN.1 comments (-- to end of line) before brace counting
                let code = l.split("--").next().unwrap_or("");
                let mut stop = false;
                for ch in code.chars() {
                    if ch == '{' {
                        depth += 1;
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            stop = true;
                            break;
                        }
                    }
                }
                def_text.push_str(l);
                def_text.push('\n');
                if stop {
                    break;
                }
            }
        }

        defs.insert(tn.to_string(), def_text.trim().to_string());
    }

    defs
}
