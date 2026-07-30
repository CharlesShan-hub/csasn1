/// 显示任意 ASN.1 类型的 JER（上帝格式）
/// 用法: cargo run --example jer_god -- <TypeName> <JSON>
/// 例如: cargo run --example jer_god -- Boolean 1
///       cargo run --example jer_god -- VisibleString '"hello"'
/// 提示: 如果 JSON 不以 { [ " 开头，会自动补双引号
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("用法: cargo run --example jer_god -- <TypeName> <JSON>");
        eprintln!("示例:");
        eprintln!("  cargo run --example jer_god -- RcbOptFlds 6800");
        eprintln!("  cargo run --example jer_god -- Boolean 1");
        eprintln!("  cargo run --example jer_god -- ServiceError 1");
        eprintln!("  cargo run --example jer_god -- Int32U 42");
        eprintln!("  cargo run --example jer_god -- VisibleString '\"hello\"'");
        std::process::exit(1);
    }

    let type_name = &args[1];
    let mut json = args[2].clone();

    // 如果 JSON 没加引号，自动补
    if !json.starts_with('"') && !json.starts_with('{') && !json.starts_with('[') {
        json = format!("\"{}\"", json);
    }

    match asn1::ffi_auto::jer_normalize(type_name, &json) {
        Ok(jer) => {
            eprintln!("=== {type_name} 上帝格式 (JER) ===");
            println!("{jer}");
            eprintln!("=== END ===");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
