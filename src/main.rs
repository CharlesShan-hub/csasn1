//! csasn1 — ASN.1 编解码工具
//!
//! 自动从 specs/dlt2811.asn 生成 Rust 代码（通过 build.rs），
//! 演示 BER/PER 编码解码。

#![allow(non_camel_case_types, non_snake_case)]

#[path = "generated.rs"]
mod generated;

use generated::dlt2811_sample::*;
use rasn::types::OctetString;

fn main() {
    println!("=== DL/T 2811 ASN.1 编解码示例 ===\n");

    // ── 1. ControlCode 编解码 ──
    println!("── 1. ControlCode (4×布尔标志) ──");
    let ctrl = ControlCode::new(true, false, true, Int8U(5));
    let ber = rasn::ber::encode(&ctrl).unwrap();
    let aper = rasn::aper::encode(&ctrl).unwrap();
    println!("原始: next={}, resp={}, err={}, pi={}",
        ctrl.next, ctrl.resp, ctrl.err, ctrl.pi .0);
    println!("BER ({} bytes): {:02X?}", ber.len(), ber);
    println!("APER ({} bytes): {:02X?}", aper.len(), aper);
    let decoded: ControlCode = rasn::ber::decode(&ber).unwrap();
    assert_eq!(ctrl, decoded);
    println!();

    // ── 2. Apch (应用层协议控制头) ──
    println!("── 2. Apch SEQUENCE ──");
    let apch = Apch::new(ctrl, Int8U(1), Int16U(100));
    let ber = rasn::ber::encode(&apch).unwrap();
    let aper = rasn::aper::encode(&apch).unwrap();
    println!("BER ({} bytes): {:02X?}", ber.len(), ber);
    println!("APER ({} bytes)", aper.len());
    let decoded: Apch = rasn::ber::decode(&ber).unwrap();
    assert_eq!(apch, decoded);
    println!();

    // ── 3. Data CHOICE (多种数据类型) ──
    println!("── 3. Data CHOICE ──");
    let data_int32 = Data::int32(Int32(-1000));
    let data_bool = Data::boolean(true);
    let data_float = Data::float32(OctetString::from(
        vec![0x44, 0x48, 0x00, 0x00],  // f32::to_be_bytes(800.0)
    ));

    for (name, data) in [("int32", &data_int32), ("bool", &data_bool), ("float", &data_float)] {
        let ber = rasn::ber::encode(data).unwrap();
        let aper = rasn::aper::encode(data).unwrap();
        println!("  {:<8} BER({}B)= {:02X?}", name, ber.len(), ber);
        println!("  {:<8} APER({}B)", name, aper.len());
    }
    println!();

    // ── 4. Apdu (完整应用层帧) ──
    println!("── 4. Apdu 完整帧 ──");
    let asdu = OctetString::from(vec![0x01, 0x02, 0x03, 0x04]);
    let apdu = Apdu::new(
        Apch::new(
            ControlCode::new(false, true, false, Int8U(0)),
            Int8U(0x51),  // GetServerDirectory
            Int16U(0),    // FL=0
        ),
        asdu,
    );
    let ber = rasn::ber::encode(&apdu).unwrap();
    let aper = rasn::aper::encode(&apdu).unwrap();
    println!("BER ({} bytes): {:02X?}", ber.len(), ber);
    println!("APER ({} bytes)", aper.len());
    println!("BER vs APER 压缩率: {:.1}%",
        (1.0 - aper.len() as f64 / ber.len() as f64) * 100.0);
    let decoded: Apdu = rasn::ber::decode(&ber).unwrap();
    assert_eq!(apdu, decoded);
    println!("解码验证 ✓");
    println!();

    // ── 5. 编码长度对比 ──
    println!("── 5. 编码长度对比 ──");
    let types: [(&str, Vec<u8>, Vec<u8>); 4] = [
        ("ControlCode", rasn::ber::encode(&ctrl).unwrap(), rasn::aper::encode(&ctrl).unwrap()),
        ("Apch",        rasn::ber::encode(&apch).unwrap(), rasn::aper::encode(&apch).unwrap()),
        ("Data(int32)", rasn::ber::encode(&data_int32).unwrap(), rasn::aper::encode(&data_int32).unwrap()),
        ("Apdu",        rasn::ber::encode(&apdu).unwrap(), rasn::aper::encode(&apdu).unwrap()),
    ];
    println!("{:<20} {:>8} {:>8} {:>10}", "类型", "BER", "APER", "压缩率");
    for (name, ber, aper) in &types {
        let ratio = (1.0 - aper.len() as f64 / ber.len() as f64) * 100.0;
        println!("{:<20} {:>4}B {:>4}B {:>8.1}%", name, ber.len(), aper.len(), ratio);
    }
}
