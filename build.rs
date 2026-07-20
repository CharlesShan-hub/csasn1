use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=specs/dlt2811.asn");

    // 使用 rasn-compiler 从 .asn1 文件生成 Rust 代码
    // 类型状态模式：先设输出模式，再加源文件，最后编译
    rasn_compiler::Compiler::<
        rasn_compiler::prelude::RasnBackend,
        rasn_compiler::CompilerMissingParams,
    >::new()
        .set_output_mode(rasn_compiler::OutputMode::SingleFile(
            PathBuf::from("src/generated.rs"),
        ))
        .add_asn_by_path(PathBuf::from("specs/dlt2811.asn"))
        .compile()
        .expect("ASN.1 编译失败");
}
