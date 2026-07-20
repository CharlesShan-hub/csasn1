//! csasn1 — ASN.1 编解码学习工具
//!
//! 自动从 ASN.1 定义文件生成 Rust 代码（通过 build.rs）

#![allow(non_camel_case_types, non_snake_case)]

#[path = "generated.rs"]
mod generated;

fn main() {
    println!("ASN.1 编译通过！");
}
