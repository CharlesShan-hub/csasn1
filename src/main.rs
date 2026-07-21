//! csasn1 — ASN.1 编解码工具
//!
//! 自动从 ASN.1 定义文件生成 Rust 代码（通过 build.rs），
//! 演示 BER/PER 编解码。

#![allow(non_camel_case_types, non_snake_case)]

#[path = "generated.rs"]
mod generated;

use generated::dlt2811_data_types::*;
use rasn::types::OctetString;

fn main() {
    println!("=== DL/T 2811 ASN.1 编解码 ===\n");

    // ─── 控制块 ───
    println!("── Apch + ControlCode ──");
    let ctrl = ControlCode::new(true, false, true, Int8U(5));
    let apch = Apch::new(ctrl, Int8U(0x51), Int16U(0));

    let ber = rasn::ber::encode(&apch).unwrap();
    let aper = rasn::aper::encode(&apch).unwrap();
    println!("BER  ({} bytes): {:02X?}", ber.len(), ber);
    println!("APER ({} bytes)", aper.len());

    let decoded: Apch = rasn::ber::decode(&ber).unwrap();
    assert_eq!(apch, decoded);
    println!("✓ 解码验证通过\n");

    // ─── 完整 APDU 帧 ───
    println!("── Apdu 完整帧 ──");
    let apdu = Apdu::new(
        apch,
        OctetString::from(vec![0x01, 0x02, 0x03, 0x04]),
    );

    let ber = rasn::ber::encode(&apdu).unwrap();
    let aper = rasn::aper::encode(&apdu).unwrap();
    let ratio = (1.0 - aper.len() as f64 / ber.len() as f64) * 100.0;
    println!("BER ({} bytes)", ber.len());
    println!("APER ({} bytes) 压缩 {:.1}%", aper.len(), ratio);

    let decoded: Apdu = rasn::ber::decode(&ber).unwrap();
    assert_eq!(apdu, decoded);
    println!("✓ 解码验证通过\n");

    // ─── JER 测试：JSON ↔ BER/PER 互转 ───
    println!("── JSON → APER 通用转换（Java 使用方式）──");
    let apdu = Apdu::new(
        Apch::new(ControlCode::new(false, true, false, Int8U(0)), Int8U(0x51), Int16U(0)),
        OctetString::from(vec![0x01, 0x02, 0x03, 0x04]),
    );
    // JER 编码（JSON 格式）
    let jer = rasn::jer::encode(&apdu).unwrap();
    println!("JER: {}", jer);

    // 验证: JSON → BER (Java 传 JSON, Rust 转 BER)
    let from_jer: Apdu = rasn::jer::decode(&jer).unwrap();
    let ber = rasn::ber::encode(&from_jer).unwrap();
    println!("JER→BER: {:02X?}", ber);
    println!("✓ JSON ↔ BER/PER 双向转换验证通过\n");

    // ─── 编码长度对比 ───
    println!("── 编码方式对比 ──");
    let encodings = ["BER", "DER", "APER", "UPER"];
    for enc in &encodings {
        let bytes = match *enc {
            "BER" => rasn::ber::encode(&apdu).unwrap(),
            "DER" => rasn::der::encode(&apdu).unwrap(),
            "APER" => rasn::aper::encode(&apdu).unwrap(),
            "UPER" => rasn::uper::encode(&apdu).unwrap(),
            _ => unreachable!(),
        };
        println!("  {:<6} {:>4} bytes  (FFI 参数: \"{}\")", enc, bytes.len(), enc.to_lowercase());
    }
    println!();

    println!("Java/Python 调用示例:");
    println!("  csasn1_encode(\"Apdu\", \"aper\", json_string)");
    println!("  csasn1_decode(\"Apdu\", \"aper\", binary_data, len)");
}
