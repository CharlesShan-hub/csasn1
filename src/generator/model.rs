use std::fs;
use std::path::Path;

// ── Data model ────────────────────────────────────────────────

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

// ── Shared utilities ───────────────────────────────────────────

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
