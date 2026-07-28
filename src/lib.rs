//! csasn1 — ASN.1 编解码库 (cdylib)
//!
//! 编译输出: target/debug/csasn1.dll
//! ffi_auto.rs 由 build.rs 自动生成

pub mod ffi_auto;

#[cfg(test)]
#[path = "generated.rs"]
mod generated;

#[cfg(test)]
mod tests {
    use super::generated::dlt2811_data_types::*;
    use rasn::prelude::*;

    /// Roundtrip Data::float32 (FixedOctetString<4>) via APER.
    #[test]
    fn test_data_float32_aper_roundtrip() {
        let bytes = [0x40, 0x48, 0xF5, 0xC3]; // IEEE 754: 3.14f32
        let orig = Data::float32(Float32(FixedOctetString::<4>::new(bytes)));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        eprintln!("Float32 Data APER encoded ({} bytes): {:02x?}", encoded.len(), encoded);
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode as Data");
        assert_eq!(orig, decoded, "Float32 APER roundtrip failed");
    }

    /// Full JER→APER encode + APER→JER decode (matches Java's flow via InnerNative)
    #[test]
    fn test_data_float32_ffi_roundtrip() {
        // First, check what JER format rasn produces for Data::float32
        let bytes = [0x40, 0x48, 0xF5, 0xC3]; // IEEE 754: 3.14f32
        let orig = Data::float32(Float32(FixedOctetString::<4>::new(bytes)));
        let jer_encoded = rasn::jer::encode(&orig).expect("JER encode");
        eprintln!("Rust JER output: {}", String::from_utf8_lossy(&jer_encoded.as_bytes()));

        // Now decode the JER back
        let decoded: Data = rasn::jer::decode(&jer_encoded).expect("JER decode back");
        assert_eq!(orig, decoded, "JER roundtrip failed");

        // Now try the full FFI flow: JER→APER encode + APER→JER decode
        let aper_encoded = rasn::aper::encode(&orig).expect("APER encode");
        eprintln!("APER encoded ({} bytes): {:02x?}", aper_encoded.len(), aper_encoded);
        let aper_decoded: Data = rasn::aper::decode(&aper_encoded).expect("APER decode");
        let roundtrip_jer = rasn::jer::encode(&aper_decoded).expect("JER encode");
        eprintln!("Roundtrip JER: {}", String::from_utf8_lossy(&roundtrip_jer.as_bytes()));
        assert_eq!(orig, aper_decoded, "Full FFI roundtrip failed");
    }

    /// Roundtrip Data::float64 (FixedOctetString<8>) via APER.
    #[test]
    fn test_data_float64_aper_roundtrip() {
        let bytes = [0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18]; // IEEE 754: 3.14159f64
        let orig = Data::float64(Float64(FixedOctetString::<8>::new(bytes)));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Float64 APER roundtrip failed");
    }

    /// Roundtrip Data::utc_time (FixedOctetString<8>) via APER.
    #[test]
    fn test_data_utctime_aper_roundtrip() {
        let bytes = *b"20260724";
        let orig = Data::utc_time(UtcTime(FixedOctetString::<8>::new(bytes)));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "UtcTime APER roundtrip failed");
    }

    /// Roundtrip Data::binary_time (FixedOctetString<6>) via APER.
    #[test]
    fn test_data_binarytime_aper_roundtrip() {
        let bytes = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let orig = Data::binary_time(BinaryTime(FixedOctetString::<6>::new(bytes)));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "BinaryTime APER roundtrip failed");
    }

    /// Roundtrip Data::octet_string (variable-length OCTET STRING) via APER.
    #[test]
    fn test_data_octet_string_aper_roundtrip() {
        let orig = Data::octet_string(OctetString::from(vec![0xAA, 0xBB, 0xCC]));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "OctetString APER roundtrip failed");
    }

    /// Roundtrip Data::visible_string via APER.
    #[test]
    fn test_data_visible_string_aper_roundtrip() {
        let orig = Data::visible_string(
            VisibleString::from_iso646_bytes(b"hello").expect("valid visible string")
        );
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "VisibleString APER roundtrip failed");
    }

    /// Roundtrip Data::unicode_string via APER.
    #[test]
    fn test_data_unicode_string_aper_roundtrip() {
        let orig = Data::unicode_string(Utf8String::from("你好世界"));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Utf8String APER roundtrip failed");
    }

    /// Roundtrip Data::bit_string via APER.
    #[test]
    fn test_data_bit_string_aper_roundtrip() {
        let orig = Data::bit_string(BitString::from_vec(vec![0xAB, 0xCD]));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "BitString APER roundtrip failed");
    }

    /// JER→APER→JER roundtrip for FixedBitString<10> (RcbOptFlds).
    /// Matches the Java flow: encode sends JER hex "0068" (bits 1,2,4 set).
    #[test]
    fn test_rcboptflds_jer_aper_jer() {
        // bits 1,2,4 set → byte0=0x68 (MSB: bit0→bit7, bit1→bit6, bit2→bit5, bit4→bit3)
        let bit_data: [u8; 10] = [0x68, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let orig = RcbOptFlds(FixedBitString::<10>::new(bit_data));
        eprintln!("orig: {orig:?}");

        // Step 1: JER encode
        let jer = rasn::jer::encode(&orig).expect("JER encode");
        eprintln!("JER: {}", String::from_utf8_lossy(&jer.as_bytes()));

        // Step 2: JER decode → APER encode (Java's InnerNative.encode path)
        let decoded: RcbOptFlds = rasn::jer::decode(&jer).expect("JER decode back");
        let aper = rasn::aper::encode(&decoded).expect("APER encode");
        eprintln!("APER encoded ({} bytes): {:02x?}", aper.len(), aper);

        // Step 3: APER decode → JER encode (Java's InnerNative.decode path)
        let aper_decoded: RcbOptFlds = rasn::aper::decode(&aper).expect("APER decode");
        let jer2 = rasn::jer::encode(&aper_decoded).expect("JER encode");
        eprintln!("JER after APER: {}", String::from_utf8_lossy(&jer2.as_bytes()));

        assert_eq!(orig, aper_decoded, "RcbOptFlds JER→APER→JER roundtrip failed");
    }

    /// Roundtrip Data::Boolean via APER.
    #[test]
    fn test_data_boolean_aper_roundtrip() {
        let orig = Data::Boolean(Boolean(1)); // true
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Boolean APER roundtrip failed");
    }

    /// Roundtrip Data::int32u via APER.
    #[test]
    fn test_data_int32u_aper_roundtrip() {
        let orig = Data::int32u(Int32U(3000000000u32));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Int32U APER roundtrip failed");
    }

    /// Roundtrip Data::int8 via APER.
    #[test]
    fn test_data_int8_aper_roundtrip() {
        let orig = Data::int8(Int8(-42));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Int8 APER roundtrip failed");
    }

    /// Roundtrip Data::int16 via APER.
    #[test]
    fn test_data_int16_aper_roundtrip() {
        let orig = Data::int16(Int16(-1000));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Int16 APER roundtrip failed");
    }

    /// Roundtrip Data::int32 via APER.
    #[test]
    fn test_data_int32_aper_roundtrip() {
        let orig = Data::int32(Int32(-100000));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Int32 APER roundtrip failed");
    }

    /// Roundtrip Data::int64 via APER.
    #[test]
    fn test_data_int64_aper_roundtrip() {
        let orig = Data::int64(Int64(-9999999999i64));
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Int64 APER roundtrip failed");
    }

    /// Roundtrip Data::array with float64 via APER.
    #[test]
    fn test_data_array_with_float64_aper_roundtrip() {
        let float64_bytes = [0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18]; // 3.14159
        let inner = Data::float64(Float64(FixedOctetString::<8usize>::new(float64_bytes)));
        let orig = Data::array(vec![inner]);
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        eprintln!("Array<Float64> APER encoded ({} bytes): {:02x?}", encoded.len(), encoded);
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Array<Float64> APER roundtrip failed");
    }

    /// Match Java CmsDataTest.roundup_array_of_data: Int32 + Boolean + Float64 in array
    #[test]
    fn test_data_array_mixed_aper_roundtrip() {
        let d1 = Data::int32(Int32(12345));
        let d2 = Data::Boolean(Boolean(1));
        let float64_bytes = [0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18]; // 3.14159
        let d3 = Data::float64(Float64(FixedOctetString::<8usize>::new(float64_bytes)));
        let orig = Data::array(vec![d1, d2, d3]);
        let encoded = rasn::aper::encode(&orig).expect("APER encode");
        eprintln!("Array mixed APER encoded ({} bytes): {:02x?}", encoded.len(), encoded);
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        assert_eq!(orig, decoded, "Array mixed APER roundtrip failed");
    }

    /// Test a single float32 CHOICE going through the FFI path using InnerNative-style JSON
    #[test]
    fn test_ffi_encode_decode_float32() {
        // Build JSON the same way Java does: InnerData with _choice + float32
        let json = r#"{"_choice":"float32","float32":"4048F5C3"}"#;
        // unwrap_jackson_value (reproducing the logic in ffi_auto.rs)
        let unwrapped = {
            let mut map: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(json.trim()).expect("valid JSON");
            map.remove("_choice");
            assert_eq!(map.len(), 1, "After removing _choice, should have 1 key, got: {map:?}");
            serde_json::to_string(&map).expect("serialize map")
        };
        eprintln!("Unwrapped JSON: {unwrapped}");
        assert_eq!(unwrapped, r#"{"float32":"4048F5C3"}"#, "unwrap should strip _choice");

        // Now try JER decode + APER encode
        let v: Data = rasn::jer::decode(&unwrapped).expect("JER decode");
        eprintln!("JER decoded: {:?}", v);
        let encoded = rasn::aper::encode(&v).expect("APER encode");
        eprintln!("APER encoded ({} bytes): {:02x?}", encoded.len(), encoded);

        // Now APER decode + JER encode
        let decoded: Data = rasn::aper::decode(&encoded).expect("APER decode");
        let roundtrip_jer = rasn::jer::encode(&decoded).expect("JER encode");
        eprintln!("Roundtrip JER: {}", String::from_utf8_lossy(&roundtrip_jer.as_bytes()));
        // wrap_in_jackson (reproducing the logic)
        let t = String::from_utf8_lossy(&roundtrip_jer.as_bytes()).trim().to_string();
        let wrapped = if t.starts_with('{') { t } else { format!("{{\"value\":{t}}}") };
        eprintln!("Wrapped: {wrapped}");
        assert!(wrapped.contains("4048F5C3"), "Missing expected hex in decoded JSON");
    }
}
