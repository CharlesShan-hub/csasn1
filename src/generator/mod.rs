use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{Fields, Type};

pub mod java;
pub mod python;

#[derive(Debug)]
pub enum TypeKind {
    Newtype {
        inner_type: String,
        size_from_attr: Option<usize>,
    },
    Struct {
        fields: Vec<FieldInfo>,
    },
    Choice {
        variants: Vec<VariantInfo>,
    },
}

#[derive(Debug)]
pub struct FieldInfo {
    pub name: String,
    pub rust_type: String,
    pub optional: bool,
    pub is_list: bool,
    pub identifier: Option<String>,
    pub size_from_attr: Option<usize>,
    pub size_attr_raw: Option<String>,
    /// Default value extracted from rasn `default = "fn_name"` attribute,
    /// resolved by looking up the function body (e.g. "Boolean(1)").
    pub default_value: Option<String>,
}

#[derive(Debug)]
pub struct VariantInfo {
    pub name: String,
    pub inner_type: String,
    pub identifier: Option<String>,
}

#[derive(Debug)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
}

/// Locates and deploys the native codec library (asn1.dll / libasn1.so) into the
/// `resources` directory of a generated artifact.
///
/// The DLL is produced by `cargo build` next to the generator executable; this
/// helper centralizes find-and-copy so Java and Python generators share one
/// implementation instead of duplicating deployment logic.
///
/// Returns whether deployment succeeded (a missing DLL is not an error, it is
/// simply skipped).
pub fn deploy_native_lib(resources_dir: &Path) -> bool {
    let Ok(exe_path) = std::env::current_exe() else {
        return false;
    };
    let Some(exe_dir) = exe_path.parent() else {
        return false;
    };

    let dll_name = if cfg!(target_os = "windows") {
        "asn1.dll"
    } else {
        "libasn1.so"
    };
    let dll_src = exe_dir.join(dll_name);
    if !dll_src.exists() {
        return false;
    }

    fs::create_dir_all(resources_dir).ok();
    match fs::copy(&dll_src, resources_dir.join(dll_name)) {
        Ok(_) => {
            println!("  copied {} to {}", dll_name, resources_dir.display());
            true
        }
        Err(e) => {
            eprintln!(
                "  warning: failed to copy {} to {}: {}",
                dll_name,
                resources_dir.display(),
                e
            );
            false
        }
    }
}

pub fn prompt(msg: &str, default: &str) -> String {
    use std::io::{BufRead, Write};
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

    // Build a map of default function name → function body expression.
    // The format is: fn <name> () -> <Type> { <body> }
    let mut default_fns: HashMap<String, String> = HashMap::new();

    let Some(syn::Item::Mod(module)) = ast.items.iter().find(|i| matches!(i, syn::Item::Mod(_)))
    else {
        return types;
    };
    let Some((_, items)) = &module.content else {
        return types;
    };

    // Pass 1: collect the body expressions of every `*_default` function.
    for inner in items {
        if let syn::Item::Fn(func) = inner {
            let name = func.sig.ident.to_string();
            if name.ends_with("_default") {
                default_fns.insert(name, block_expr(&func.block));
            }
        }
    }

    // Pass 2: collect struct and enum types.
    for inner in items {
        match inner {
            syn::Item::Struct(s) => types.push(TypeInfo {
                name: s.ident.to_string(),
                kind: analyze_struct(s, &default_fns),
            }),
            syn::Item::Enum(e) => types.push(TypeInfo {
                name: e.ident.to_string(),
                kind: analyze_enum(e),
            }),
            _ => {}
        }
    }
    types
}

/// Extracts the expression inside a function body block as a string, stripping the
/// outer `{` `}` braces and surrounding whitespace.
fn block_expr(block: &syn::Block) -> String {
    let s = quote::quote!(#block).to_string();
    let s = s.trim();
    let s = s.strip_prefix('{').unwrap_or(s);
    let s = s.strip_suffix('}').unwrap_or(s);
    s.trim().to_string()
}

/// Iterates over the top-level metas inside `#[rasn(...)]` attributes, invoking `f`
/// on each until it returns `true`.
///
/// Compatible with nested parenthesized forms such as `tag(context, 0)`, which
/// `parse_nested_meta` does not handle.
fn for_each_rasn_meta(attrs: &[syn::Attribute], mut f: impl FnMut(&syn::Meta) -> bool) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("rasn") {
            continue;
        }
        let Ok(list) = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        ) else {
            continue;
        };
        for meta in list {
            if f(&meta) {
                return true;
            }
        }
    }
    false
}

/// Returns whether a `#[rasn(...)]` attribute contains the given marker (e.g. `delegate`).
fn attr_contains(attrs: &[syn::Attribute], pat: &str) -> bool {
    for_each_rasn_meta(attrs, |meta| meta.path().is_ident(pat))
}

/// Extracts a string value from `#[rasn(name = "value")]`, e.g. `identifier = "xxx"` → "xxx".
fn rasn_str_value(attrs: &[syn::Attribute], name: &str) -> Option<String> {
    let mut result = None;
    for_each_rasn_meta(attrs, |meta| {
        if meta.path().is_ident(name) {
            if let syn::Meta::NameValue(nv) = meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    result = Some(s.value());
                }
            }
            return true;
        }
        false
    });
    result
}

/// Extracts the string value of a `#[rasn(size("..."))]` attribute.
fn rasn_size_str(attrs: &[syn::Attribute]) -> Option<String> {
    let mut result = None;
    for_each_rasn_meta(attrs, |meta| {
        if meta.path().is_ident("size") {
            if let syn::Meta::List(list) = meta {
                if let Ok(expr) = list.parse_args::<syn::Expr>() {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = expr
                    {
                        result = Some(s.value());
                    }
                }
            }
            return true;
        }
        false
    });
    result
}

/// Parses a size string (`"N"` or `"min..=max"`) into its upper bound.
fn parse_size_value(val: &str) -> Option<usize> {
    if let Ok(n) = val.parse::<usize>() {
        return Some(n);
    }
    val.find("..=")
        .and_then(|eq_pos| val[eq_pos + 3..].trim().parse::<usize>().ok())
}

/// Extract size attributes from a list of syn attributes (works for both struct-level and field-level).
fn extract_size_from_attrs(attrs: &[syn::Attribute]) -> (Option<usize>, Option<String>) {
    let raw = rasn_size_str(attrs);
    let size = raw.as_deref().and_then(parse_size_value);
    (size, raw)
}

fn analyze_struct(s: &syn::ItemStruct, default_fns: &HashMap<String, String>) -> TypeKind {
    if attr_contains(&s.attrs, "delegate") {
        let size_from_attr = extract_size_from_attrs(&s.attrs).0;
        if let Fields::Unnamed(ref u) = s.fields {
            if let Some(f) = u.unnamed.first() {
                return TypeKind::Newtype {
                    inner_type: type_str(&f.ty),
                    size_from_attr,
                };
            }
        }
        return TypeKind::Newtype {
            inner_type: "int".into(),
            size_from_attr,
        };
    }
    let mut fields = Vec::new();
    for f in s.fields.iter() {
        let name = f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
        let rt = type_str(&f.ty);
        let optional = rt.starts_with("Option <");
        let is_list = rt.contains("Vec <") || rt.contains("SequenceOf <");

        // Extract ASN.1 identifier from rasn attribute: identifier = "xxx"
        let identifier = rasn_str_value(&f.attrs, "identifier");

        // Extract size from rasn attribute: size ("N") or size ("min..=max")
        let (size_from_attr, size_attr_raw) = extract_size_from_attrs(&f.attrs);

        // Extract default value from rasn attribute: default = "fn_name"
        let default_value = rasn_str_value(&f.attrs, "default")
            .and_then(|fn_name| default_fns.get(&fn_name).cloned());

        fields.push(FieldInfo {
            name,
            rust_type: rt,
            optional,
            is_list,
            identifier,
            size_from_attr,
            size_attr_raw,
            default_value,
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
                u.unnamed.first().map(|f| {
                    // Extract ASN.1 identifier from rasn attribute: identifier = "xxx"
                    let identifier = rasn_str_value(&v.attrs, "identifier");
                    VariantInfo {
                        name: v.ident.to_string(),
                        inner_type: type_str(&f.ty),
                        identifier,
                    }
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
                    defs.insert(
                        tn.to_string(),
                        format!(
                            "(inline type within {})",
                            all_defs
                                .get(*parent)
                                .unwrap()
                                .1
                                .lines()
                                .next()
                                .unwrap_or("")
                        ),
                    );
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
            if before_eq.is_empty()
                || !after_eq.contains("BIT STRING") && !after_eq.contains("ENUMERATED")
            {
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
