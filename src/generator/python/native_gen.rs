use super::helpers;

/// Generate _native.py — ctypes bridge
pub fn gen_native(_prefix: &str, _package: &str, default_enc: &str) -> String {
    include_str!("templates/_native.py.txt")
        .replace("__VER__", &helpers::gen_header_comment())
        .replace("__ENC__", default_enc)
        .to_string()
}

/// Generate _base.py — serialization helpers
pub fn gen_base(_prefix: &str, _package: &str) -> String {
    include_str!("templates/_base.py.txt")
        .replace("__VER__", &helpers::gen_header_comment())
        .to_string()
}

/// Generate __init__.py — static header; type exports are appended by mod.rs
pub fn gen_init(_prefix: &str, _package: &str) -> String {
    include_str!("templates/__init__.py.txt")
        .replace("__VER__", &helpers::gen_header_comment())
        .to_string()
}

/// Generate pixi.toml
pub fn gen_pixi_toml(prefix: &str, package: &str) -> String {
    let pkg_name = if package.is_empty() {
        format!("{}-data", prefix.to_lowercase())
    } else {
        package.to_string()
    };
    include_str!("templates/pixi.toml.txt")
        .replace("__PKG_NAME__", &pkg_name)
        .to_string()
}
