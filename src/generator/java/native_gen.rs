/// Generate CmsNative.java — JNA bridge to the Rust asn1.dll.
pub fn gen_native(prefix: &str, package: &str) -> String {
    let pkg = if package.is_empty() {
        String::new()
    } else {
        format!("package {};\n\n", package)
    };
    let ver = super::helpers::gen_version();
    include_str!("templates/Native.java.txt")
        .replace("__VER__", &ver)
        .replace("__PKG__", &pkg)
        .replace("__PFX__", prefix)
        .to_string()
}

/// Generate CmsBase.java — abstract base class for all data types.
pub fn gen_base(prefix: &str, package: &str, default_enc: &str) -> String {
    let pkg = if package.is_empty() {
        String::new()
    } else {
        format!("package {};\n\n", package)
    };
    let ver = super::helpers::gen_version();
    include_str!("templates/Base.java.txt")
        .replace("__VER__", &ver)
        .replace("__PKG__", &pkg)
        .replace("__PFX__", prefix)
        .replace("__ENC__", default_enc)
        .to_string()
}

/// Generate V.java — semantic helpers for the _v unified data store.
/// Keeps _v key names and shape logic in one place (generated alongside InnerBase).
pub fn gen_v(package: &str) -> String {
    let pkg = if package.is_empty() {
        String::new()
    } else {
        format!("package {};\n\n", package)
    };
    include_str!("templates/V.java.txt")
        .replace("__VER__", &super::helpers::gen_version())
        .replace("__PKG__", &pkg)
        .to_string()
}
