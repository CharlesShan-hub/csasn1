use super::*;

// ── Data compound + FFI adapter roundtrips ───────────────────

/// Full JER→APER encode + APER→JER decode (matches Java's flow via InnerNative)
#[test]
fn data_float32_ffi_roundtrip() {
    let bytes = [0x40, 0x48, 0xF5, 0xC3];
    let orig = Data::float32(Float32(FixedOctetString::<4>::new(bytes)));
    let jer_encoded = rasn::jer::encode(&orig).expect("JER encode");
    let decoded: Data = rasn::jer::decode(&jer_encoded).expect("JER decode back");
    assert_eq!(orig, decoded, "JER roundtrip failed");

    let aper_encoded = rasn::aper::encode(&orig).expect("APER encode");
    let aper_decoded: Data = rasn::aper::decode(&aper_encoded).expect("APER decode");
    assert_eq!(orig, aper_decoded, "Full FFI roundtrip failed");
}

#[test]
fn data_array_float64_aper() {
    let float64_bytes = [0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18];
    let inner = Data::float64(Float64(FixedOctetString::<8usize>::new(float64_bytes)));
    let orig = Data::array(vec![inner]);
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

#[test]
fn data_array_mixed_aper() {
    let d1 = Data::int32(Int32(12345));
    let d2 = Data::Boolean(Boolean(1));
    let float64_bytes = [0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18];
    let d3 = Data::float64(Float64(FixedOctetString::<8usize>::new(float64_bytes)));
    let orig = Data::array(vec![d1, d2, d3]);
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig, decoded);
}

/// Roundtrip with moreFollows DEFAULT TRUE — verifies PER handles BOOLEAN DEFAULT correctly.
#[test]
fn morefollows_default_true_aper() {
    let orig = GetAllDataDefinitionResponsePDU {
        data: GetAllDataDefinitionResponsePDUData(vec![]),
        more_follows: Boolean(1),
    };
    let encoded = rasn::aper::encode(&orig).expect("APER encode");
    let decoded: GetAllDataDefinitionResponsePDU = rasn::aper::decode(&encoded).expect("APER decode");
    assert_eq!(orig.more_follows, decoded.more_follows, "DEFAULT TRUE should survive");
    assert_eq!(orig, decoded);
}

/// FFI path: Jackson-style JSON → JER decode → APER encode → APER decode → JER encode → Jackson wrap
#[test]
fn ffi_encode_decode_float32() {
    let json = r#"{"_choice":"float32","float32":"4048F5C3"}"#;
    let unwrapped = {
        let mut map: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(json.trim()).expect("valid JSON");
        map.remove("_choice");
        assert_eq!(map.len(), 1, "should have 1 key after removing _choice");
        serde_json::to_string(&map).expect("serialize map")
    };
    assert_eq!(unwrapped, r#"{"float32":"4048F5C3"}"#);

    let v: Data = rasn::jer::decode(&unwrapped).expect("JER decode");
    let encoded = rasn::aper::encode(&v).expect("APER encode");
    let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
    let roundtrip_jer = rasn::jer::encode(&decoded).expect("JER encode");

    let t = String::from_utf8_lossy(&roundtrip_jer.as_bytes()).trim().to_string();
    let wrapped = if t.starts_with('{') { t } else { format!("{{\"value\":{t}}}") };
    assert!(wrapped.contains("4048F5C3"), "Missing expected hex");
}