use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use super::*;

pub mod helpers;
pub mod gen_type;
pub mod native_gen;
pub mod test_gen;

pub struct PythonConfig {
    pub prefix: String,
    pub package: String,
    pub default_enc: String,
    pub out_dir: PathBuf,
}

fn pkg_dir_name() -> String {
    "cms_data".to_string()
}

/// Topological sort: ensure types that depend on other types come after their dependencies
fn topo_sort_types(types: &[TypeInfo], prefix: &str) -> Vec<usize> {
    let all_names: HashSet<&str> = types.iter().map(|t| t.name.as_str()).collect();
    let name_to_idx: HashMap<&str, usize> = types.iter().enumerate()
        .map(|(i, t)| (t.name.as_str(), i)).collect();

    // Extract dependency names from a type reference string
    // rust_type can be "Boolean", "Int8U", "CmsBoolean", "Vec <Int8U>", etc.
    fn extract_dep(s: &str, prefix: &str, all_names: &HashSet<&str>) -> Vec<String> {
        let s = s.trim();
        // Direct match: the type name itself (e.g. "Boolean", "Int8U")
        if all_names.contains(s) {
            return vec![s.to_string()];
        }
        // Prefixed: "<prefix>Name" (e.g. "CmsBoolean")
        if s.starts_with(prefix) {
            let rest = s.trim_start_matches(prefix);
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if all_names.contains(name.as_str()) { return vec![name]; }
        }
        // Wrapped: "Vec <Type>", "SequenceOf <Type>", "Option <Type>"
        for wrapper in &["Vec <", "SequenceOf <", "Option <"] {
            if s.starts_with(wrapper) {
                let inner = s.trim_start_matches(wrapper).trim_end_matches('>').trim();
                return extract_dep(inner, prefix, all_names);
            }
        }
        vec![]
    }

    let n = types.len();
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    let mut in_degree: Vec<usize> = vec![0; n];

    for i in 0..n {
        let deps: Vec<String> = match &types[i].kind {
            TypeKind::Newtype { inner_type } => extract_dep(inner_type, prefix, &all_names),
            TypeKind::Struct { fields } => {
                fields.iter().flat_map(|f| extract_dep(&f.rust_type, prefix, &all_names)).collect()
            }
            TypeKind::Choice { variants } => {
                variants.iter().flat_map(|v| extract_dep(&v.inner_type, prefix, &all_names)).collect()
            }
        };
        for dep in deps {
            if let Some(&j) = name_to_idx.get(dep.as_str()) {
                if i != j {
                    adj[j].push(i);
                    in_degree[i] += 1;
                }
            }
        }
    }

    // Kahn's algorithm
    let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut order = Vec::with_capacity(n);
    while let Some(idx) = queue.pop() {
        order.push(idx);
        for &next in &adj[idx] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                queue.push(next);
            }
        }
    }
    // If cycles, append remaining
    for i in 0..n {
        if !order.contains(&i) { order.push(i); }
    }
    order
}

/// Entry point: generate all Python files
pub fn generate(types: &[TypeInfo], cfg: &PythonConfig, asn_defs: &HashMap<String, String>,
                _named_consts: &HashMap<String, Vec<(String, i32)>>) {
    let project_root = &cfg.out_dir;
    let src_dir = project_root.join("src").join(pkg_dir_name());
    let test_dir = project_root.join("tests");

    fs::create_dir_all(&src_dir).expect("failed to create src dir");
    fs::create_dir_all(&test_dir).expect("failed to create test dir");

    // _native.py
    fs::write(src_dir.join("_native.py"), &native_gen::gen_native(&cfg.prefix, &cfg.package, &cfg.default_enc))
        .expect("failed to write _native.py");

    // _base.py
    fs::write(src_dir.join("_base.py"), &native_gen::gen_base(&cfg.prefix, &cfg.package))
        .expect("failed to write _base.py");

    // pixi.toml — 仅首次生成
    let pixi_path = project_root.join("pixi.toml");
    if !pixi_path.exists() {
        fs::write(&pixi_path, &native_gen::gen_pixi_toml(&cfg.prefix, &cfg.package))
            .expect("failed to write pixi.toml");
        println!("  wrote pixi.toml");
    }

    // Copy asn1.dll to resources
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let dll_name = if cfg!(target_os = "windows") { "asn1.dll" } else { "libasn1.so" };
            let dll_src = exe_dir.join(dll_name);
            if dll_src.exists() {
                let res_dir = src_dir.join("resources");
                fs::create_dir_all(&res_dir).ok();
                fs::copy(&dll_src, res_dir.join(dll_name))
                    .expect("failed to copy asn1.dll");
            }
        }
    }

    // tests/__init__.py
    fs::write(test_dir.join("__init__.py"), "")
        .expect("failed to write tests/__init__.py");

    // Compute topological order for type generation (dependencies first)
    let topo_order = topo_sort_types(types, &cfg.prefix);
    let sorted_types: Vec<&TypeInfo> = topo_order.iter().map(|&i| &types[i]).collect();
    // Generate _types.py with all type classes
    let mut all_types_code = String::new();
    all_types_code.push_str(&helpers::gen_header_comment());
    all_types_code.push_str("# Generated ASN.1 data types\n\n");
    all_types_code.push_str("import json\nfrom dataclasses import dataclass, field\n");
    all_types_code.push_str("from typing import Any\n\n");
    all_types_code.push_str("from ._base import to_json, to_json_strict, from_json, bit_string_hex, parse_bit_string_hex\n");
    all_types_code.push_str("from ._native import encode, decode_raw\n\n");

    for ti in &sorted_types {
        let class_code = gen_type::gen_type_class(ti, types, &cfg.prefix, &cfg.package, asn_defs);
        all_types_code.push_str(&class_code);
        all_types_code.push('\n');
    }
    fs::write(src_dir.join("_types.py"), &all_types_code)
        .expect("failed to write _types.py");

    // Update __init__.py with type exports
    let mut init_code = native_gen::gen_init(&cfg.prefix, &cfg.package);
    init_code.push_str(&format!("from ._types import (\n"));
    for ti in types {
        init_code.push_str(&format!("    {pf}{tn},\n", pf = cfg.prefix, tn = ti.name));
    }
    init_code.push_str(")\n\n");
    init_code.push_str("__all__ = [\n");
    for ti in types {
        init_code.push_str(&format!("    \"{pf}{tn}\",\n", pf = cfg.prefix, tn = ti.name));
    }
    init_code.push_str("]\n");
    fs::write(src_dir.join("__init__.py"), &init_code)
        .expect("failed to write __init__.py");

    // Generate test file
    let mut all_tests = String::new();
    all_tests.push_str(&format!("# Auto-generated by {}. Tests\n", super::java::helpers::gen_version()));
    all_tests.push_str("import pytest\n");
    all_tests.push_str("from cms_data import (\n");
    for ti in types {
        all_tests.push_str(&format!("    {pf}{tn},\n", pf = cfg.prefix, tn = ti.name));
    }
    all_tests.push_str(")\n\n");
    for ti in types {
        all_tests.push_str(&test_gen::gen_test(ti, types, &cfg.prefix, &cfg.package, asn_defs));
        all_tests.push('\n');
    }
    fs::write(test_dir.join("test_types.py"), &all_tests)
        .expect("failed to write test_types.py");

    println!("✓ generated Python package in {:?}", project_root);
}
