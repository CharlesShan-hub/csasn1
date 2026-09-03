use super::model::{FieldInfo, TypeInfo, TypeKind, VariantInfo};
use std::collections::HashMap;
use syn::{Fields, Type};

// ── Public entry point ──────────────────────────────────────────

pub fn extract_types(ast: &syn::File) -> Vec<TypeInfo> {
    let mut types = Vec::new();
    let mut default_fns: HashMap<String, String> = HashMap::new();

    let Some(syn::Item::Mod(module)) = ast.items.iter().find(|i| matches!(i, syn::Item::Mod(_)))
    else {
        return types;
    };
    let Some((_, items)) = &module.content else {
        return types;
    };

    // Pass 1: collect `*_default` function bodies (e.g. `fn xxx() -> Boolean { Boolean(1) }`).
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

// ── Private helpers ────────────────────────────────────────────

fn block_expr(block: &syn::Block) -> String {
    let s = quote::quote!(#block).to_string();
    let s = s.trim();
    let s = s.strip_prefix('{').unwrap_or(s);
    let s = s.strip_suffix('}').unwrap_or(s);
    s.trim().to_string()
}

/// Iterates over `#[rasn(...)]` attributes; invokes `f` on each top-level Meta
/// until it returns true. Handles nested parens like `tag(context, 0)`.
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

fn attr_contains(attrs: &[syn::Attribute], pat: &str) -> bool {
    for_each_rasn_meta(attrs, |meta| meta.path().is_ident(pat))
}

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

fn parse_size_value(val: &str) -> Option<usize> {
    if let Ok(n) = val.parse::<usize>() {
        return Some(n);
    }
    val.find("..=")
        .and_then(|eq_pos| val[eq_pos + 3..].trim().parse::<usize>().ok())
}

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
        let identifier = rasn_str_value(&f.attrs, "identifier");
        let (size_from_attr, size_attr_raw) = extract_size_from_attrs(&f.attrs);
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