use std::fs;
use std::path::PathBuf;

#[path = "generator/mod.rs"]
mod generator;
use generator::*;

fn main() {
    let mut spec_path = "specs/dlt2811.asn".to_string();
    let mut out_dir = PathBuf::from("java/src");
    let mut target_lang = "java".to_string();

    // Generator-specific config
    let mut prefix = "Cms".to_string();
    let mut default_enc = "ber".to_string();
    let mut package = String::new();

    let args: Vec<String> = std::env::args().collect();
    let interactive = args.len() <= 1;

    // --- Interactive mode ---
    if interactive {
        println!("── csasn1 interactive mode ──");
        spec_path = prompt("ASN.1 spec file", &spec_path);
        out_dir = PathBuf::from(prompt("Output directory", &out_dir.to_string_lossy()));
        target_lang = prompt("Target language (java)", &target_lang);
        prefix = prompt("Class prefix", &prefix);
        default_enc = prompt("Default encoding (ber/der/aper/uper)", &default_enc);
        package = prompt("Java package (empty = none)", "");
        println!();
    }

    // --- CLI arg parse ---
    let mut args_iter = args.into_iter().peekable();
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--src" => spec_path = args_iter.next().expect("--src requires a value"),
            "--out" | "--dest" => {
                out_dir = PathBuf::from(args_iter.next().expect("--out/--dest requires a value"))
            }
            "--prefix" => prefix = args_iter.next().expect("--prefix requires a value"),
            "--enc" => default_enc = args_iter.next().expect("--enc requires a value"),
            "--package" => package = args_iter.next().expect("--package requires a value"),
            "--bin" | "--lang" => {
                target_lang = args_iter.next().expect("--bin/--lang requires a value")
            }
            _ => {}
        }
    }

    // --src .asn files auto-map to the generated .rs via build.rs
    let src_path = if spec_path.ends_with(".asn") {
        "src/generated.rs".to_string()
    } else {
        spec_path.clone()
    };

    let src = fs::read_to_string(&src_path).unwrap_or_else(|e| {
        eprintln!("Error: failed to read {}: {}", src_path, e);
        std::process::exit(1);
    });
    let ast = syn::parse_file(&src).unwrap_or_else(|e| {
        eprintln!("Error: failed to parse {}: {}", src_path, e);
        std::process::exit(1);
    });

    // Extract types (shared, language-agnostic)
    let types = generator::extract_types(&ast);

    // Extract ASN.1 definitions for doc comments
    let type_names: Vec<&str> = types.iter().map(|t| t.name.as_str()).collect();
    let asn_defs = generator::extract_asn1_definitions(&spec_path, &type_names.as_slice());

    // Extract named constants from BIT STRING / ENUMERATED definitions
    let named_consts = generator::extract_asn1_named_constants(&spec_path);

    // Dispatch to the chosen language generator
    match target_lang.to_lowercase().as_str() {
        "java" => {
            let cfg = generator::java::JavaConfig {
                prefix,
                default_enc,
                package,
                out_dir,
            };
            generator::java::generate(&types, &cfg, &asn_defs, &named_consts);
        }
        "python" | "py" => {
            let cfg = generator::python::PythonConfig {
                prefix,
                default_enc,
                package,
                out_dir,
            };
            generator::python::generate(&types, &cfg, &asn_defs, &named_consts);
        }
        other => {
            eprintln!(
                "Error: unsupported target language '{}'. Supported: java, python",
                other
            );
            std::process::exit(1);
        }
    }
}
