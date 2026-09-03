use super::*;

// ── Data scalar APER roundtrips ─────────────────────────────────

#[test]
fn data_float32_aper() {
    let bytes = [0x40, 0x48, 0xF5, 0xC3];
    let orig = Data::float32(Float32(FixedOctetString::<4>::new(bytes)));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_float64_aper() {
    let bytes = [0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18];
    let orig = Data::float64(Float64(FixedOctetString::<8>::new(bytes)));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_utctime_aper() {
    let orig = Data::utc_time(UtcTime(FixedOctetString::<8>::new(*b"20260724")));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_binarytime_aper() {
    let orig = Data::binary_time(BinaryTime(FixedOctetString::<6>::new([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_octet_string_aper() {
    let orig = Data::octet_string(OctetString::from(vec![0xAA, 0xBB, 0xCC]));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_visible_string_aper() {
    let orig = Data::visible_string(VisibleString::from_iso646_bytes(b"hello").expect("valid"));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_unicode_string_aper() {
    let orig = Data::unicode_string(Utf8String::from("你好世界"));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_bit_string_aper() {
    let orig = Data::bit_string(BitString::from_vec(vec![0xAB, 0xCD]));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_boolean_aper() {
    let orig = Data::Boolean(Boolean(1));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_int8_aper() {
    let orig = Data::int8(Int8(-42));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_int16_aper() {
    let orig = Data::int16(Int16(-1000));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_int32_aper() {
    let orig = Data::int32(Int32(-100000));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_int64_aper() {
    let orig = Data::int64(Int64(-9999999999i64));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_int32u_aper() {
    let orig = Data::int32u(Int32U(3000000000u32));
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}