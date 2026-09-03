use super::*;

// ── "God format" JER output — reference shape for Java side ────

#[test]
fn boolean_jer() {
    let true_jer = rasn::jer::encode(&Boolean(1)).expect("JER");
    let false_jer = rasn::jer::encode(&Boolean(0)).expect("JER");
    eprintln!("Boolean true: {}", String::from_utf8_lossy(&true_jer.as_bytes()));
    eprintln!("Boolean false: {}", String::from_utf8_lossy(&false_jer.as_bytes()));
}

#[test]
fn int8u_jer() {
    let jer = rasn::jer::encode(&Int8U(100)).expect("JER");
    eprintln!("Int8U(100): {}", String::from_utf8_lossy(&jer.as_bytes()));
}

#[test]
fn serviceerror_jer() {
    let jer = rasn::jer::encode(&ServiceError(1)).expect("JER");
    eprintln!("ServiceError(1): {}", String::from_utf8_lossy(&jer.as_bytes()));
}

#[test]
fn data_choice_jer() {
    let jer_int32 = rasn::jer::encode(&Data::int32(Int32(42))).expect("JER");
    let jer_bool = rasn::jer::encode(&Data::Boolean(Boolean(1))).expect("JER");
    let jer_str = rasn::jer::encode(&Data::visible_string(VisibleString::from_iso646_bytes(b"hello").unwrap())).expect("JER");
    let jer_bit = rasn::jer::encode(&Data::bit_string(BitString::from_vec(vec![0xAA, 0xBB]))).expect("JER");
    eprintln!("int32(42): {}", String::from_utf8_lossy(&jer_int32.as_bytes()));
    eprintln!("Boolean(1): {}", String::from_utf8_lossy(&jer_bool.as_bytes()));
    eprintln!("visible_string: {}", String::from_utf8_lossy(&jer_str.as_bytes()));
    eprintln!("bit_string: {}", String::from_utf8_lossy(&jer_bit.as_bytes()));
}

#[test]
fn rcboptflds_jer() {
    let jer = rasn::jer::encode(&RcbOptFlds(FixedBitString::<10>::new([0x68, 0, 0, 0, 0, 0, 0, 0, 0, 0]))).expect("JER");
    eprintln!("RcbOptFlds: {}", String::from_utf8_lossy(&jer.as_bytes()));
}

#[test]
fn urcb_jer() {
    let urcb = URCB {
        rpt_id: VisibleString::from_iso646_bytes(b"report1").unwrap(),
        rpt_ena: Boolean(1),
        dat_set: ObjectReference(VisibleString::from_iso646_bytes(b"PROT/LLN0.RPIT.Ena").unwrap()),
        conf_rev: Int32U(1),
        opt_flds: RcbOptFlds(FixedBitString::<10>::new([0x68, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
        buf_tm: Int32U(1000),
        sq_num: Int16U(1),
        trg_ops: TriggerConditions(FixedBitString::<6>::new([0x01, 0, 0, 0, 0, 0])),
        intg_pd: Int32U(5000),
        gi: Boolean(0),
        resv: Boolean(0),
        owner: Some(OctetString::from(vec![0x01, 0x02, 0x03])),
    };
    let jer = rasn::jer::encode(&urcb).expect("JER");
    eprintln!("URCB: {}", String::from_utf8_lossy(&jer.as_bytes()));
}

#[test]
fn brcb_jer() {
    let brcb = BRCB {
        rpt_id: VisibleString::from_iso646_bytes(b"report1").unwrap(),
        rpt_ena: Boolean(1),
        dat_set: ObjectReference(VisibleString::from_iso646_bytes(b"PROT/LLN0.RPIT.Ena").unwrap()),
        conf_rev: Int32U(1),
        opt_flds: RcbOptFlds(FixedBitString::<10>::new([0x68, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
        buf_tm: Int32U(1000),
        sq_num: Int16U(1),
        trg_ops: TriggerConditions(FixedBitString::<6>::new([0x01, 0, 0, 0, 0, 0])),
        intg_pd: Int32U(5000),
        gi: Boolean(0),
        purge_buf: Boolean(0),
        entry_id: EntryID(FixedOctetString::<8>::new([0, 0, 0, 0, 0, 0, 0, 1])),
        time_of_entry: EntryTime(BinaryTime(FixedOctetString::<6>::new([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]))),
        resv_tms: None,
        owner: None,
    };
    let jer = rasn::jer::encode(&brcb).expect("JER");
    eprintln!("BRCB: {}", String::from_utf8_lossy(&jer.as_bytes()));
}