/// Prints the JER ("God format") encoding of any ASN.1 type.
/// Usage: cargo run --example jer_god -- <TypeName> <JSON>
/// e.g.   cargo run --example jer_god -- Boolean 1
///        cargo run --example jer_god -- VisibleString '"hello"'
/// Hint: if the JSON does not start with { [ or ", double quotes are added automatically.
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: cargo run --example jer_god -- <TypeName> <JSON>");
        eprintln!("Examples:");
        eprintln!("  cargo run --example jer_god -- RcbOptFlds 6800");
        eprintln!("  cargo run --example jer_god -- Boolean 1");
        eprintln!("  cargo run --example jer_god -- ServiceError 1");
        eprintln!("  cargo run --example jer_god -- Int32U 42");
        eprintln!("  cargo run --example jer_god -- VisibleString '\"hello\"'");
        std::process::exit(1);
    }

    let type_name = &args[1];
    let mut json = args[2].clone();

    // Auto-quote JSON if it is not already a string/object/array.
    if !json.starts_with('"') && !json.starts_with('{') && !json.starts_with('[') {
        json = format!("\"{}\"", json);
    }

    match asn1::ffi_auto::jer_normalize(type_name, &json) {
        Ok(jer) => {
            eprintln!("=== {type_name} JER God format ===");
            println!("{jer}");
            eprintln!("=== END ===");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
