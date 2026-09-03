use super::*;

// ── BRCB / URCB / RcbOptFlds / TriggerConditions ────────────────

/// JER→APER→JER roundtrip for FixedBitString<10> (RcbOptFlds).
/// Matches Java flow: bits 1,2,4 set → byte0=0x68.
#[test]
fn rcboptflds_jer_aper_jer() {
    let bit_data: [u8; 10] = [0x68, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let orig = RcbOptFlds(FixedBitString::<10>::new(bit_data));

    // Step 1: JER encode
    let jer = rasn::jer::encode(&orig).expect("JER encode");
    // Step 2: JER decode → APER encode (Java's InnerNative.encode)
    let decoded: RcbOptFlds = rasn::jer::decode(&jer).expect("JER decode");
    let aper = rasn::aper::encode(&decoded).expect("APER encode");
    // Step 3: APER decode → JER encode (Java's InnerNative.decode)
    let aper_decoded: RcbOptFlds = rasn::aper::decode(&aper).expect("APER decode");

    assert_eq!(orig, aper_decoded, "RcbOptFlds JER→APER→JER failed");
}

/// BRCB APER roundtrip: mirrors Java default-value test.
#[test]
fn brcb_aper_defaults() {
    let brcb = BRCB {
        rpt_id: VisibleString::from_iso646_bytes(b"x").unwrap(),
        rpt_ena: Boolean(1),
        dat_set: ObjectReference(
            VisibleString::from_iso646_bytes(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").unwrap(),
        ),
        conf_rev: Int32U(0),
        opt_flds: RcbOptFlds(FixedBitString::<10>::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
        buf_tm: Int32U(0),
        sq_num: Int16U(1),
        trg_ops: TriggerConditions(FixedBitString::<6>::new([0, 0, 0, 0, 0, 0])),
        intg_pd: Int32U(0),
        gi: Boolean(1),
        purge_buf: Boolean(1),
        entry_id: EntryID(FixedOctetString::<8>::new([1, 1, 1, 1, 1, 1, 1, 1])),
        time_of_entry: EntryTime(BinaryTime(FixedOctetString::<6>::new([1, 1, 1, 1, 1, 1]))),
        resv_tms: None,
        owner: None,
    };
    let encoded = rasn::aper::encode(&brcb).expect("APER encode BRCB");
    let decoded: BRCB = rasn::aper::decode(&encoded).expect("APER decode BRCB");
    assert_eq!(brcb, decoded, "BRCB APER failed");
}