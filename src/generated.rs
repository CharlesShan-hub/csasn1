#[allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused,
    clippy::too_many_arguments
)]
pub mod dlt2811_data_types {
    extern crate alloc;
    use core::borrow::Borrow;
    use rasn::prelude::*;
    use std::sync::LazyLock;
    #[doc = " ====================================================================== "]
    #[doc = " 8.3.3 GetLogicalNodeDirectory (Service Code 0x53) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=10"))]
    pub struct ACSIClass(pub u8);
    #[doc = " ====================================================================== "]
    #[doc = " 8.2.3 Abort (Service Code 0x03) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "Abort-RequestPDU")]
    pub struct AbortRequestPDU {
        #[rasn(size("0..=64"), tag(context, 0), identifier = "associationId")]
        pub association_id: OctetString,
        #[rasn(value("0..=5"), tag(context, 1))]
        pub reason: u8,
    }
    impl AbortRequestPDU {
        pub fn new(association_id: OctetString, reason: u8) -> Self {
            Self {
                association_id,
                reason,
            }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.5.4 AddCause "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=27"))]
    pub struct AddCause(pub u8);
    #[doc = "  "]
    #[doc = " DL/T 2811 Protocol Layer ? APCH / ASDU definitions "]
    #[doc = " "]
    #[doc = " This is the foundation layer for all services. "]
    #[doc = "  "]
    #[doc = " ====================================================================== "]
    #[doc = " APCH ? Application Protocol Control Header (4 bytes) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct Apch {
        pub cc: ControlCode,
        pub sc: Int8U,
        pub fl: Int16U,
    }
    impl Apch {
        pub fn new(cc: ControlCode, sc: Int8U, fl: Int16U) -> Self {
            Self { cc, sc, fl }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " APDU ? Complete application-layer frame "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct Apdu {
        pub apch: Apch,
        #[rasn(size("0..=65531"))]
        pub asdu: OctetString,
    }
    impl Apdu {
        pub fn new(apch: Apch, asdu: OctetString) -> Self {
            Self { apch, asdu }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " ASDU ? Application Service Data Unit (base for all services) "]
    #[doc = " ====================================================================== "]
    #[doc = " reqId is the first field in every ASDU "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct Asdu {
        #[rasn(identifier = "reqId")]
        pub req_id: Int16U,
    }
    impl Asdu {
        pub fn new(req_id: Int16U) -> Self {
            Self { req_id }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "Associate-ErrorPDU")]
    pub struct AssociateErrorPDU(pub ServiceError);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct AssociateRequestPDUAuthenticationParameter {
        #[rasn(tag(context, 0), identifier = "signatureCertificate")]
        pub signature_certificate: OctetString,
        #[rasn(tag(context, 1), identifier = "signedTime")]
        pub signed_time: UtcTime,
        #[rasn(tag(context, 2), identifier = "signedValue")]
        pub signed_value: OctetString,
    }
    impl AssociateRequestPDUAuthenticationParameter {
        pub fn new(
            signature_certificate: OctetString,
            signed_time: UtcTime,
            signed_value: OctetString,
        ) -> Self {
            Self {
                signature_certificate,
                signed_time,
                signed_value,
            }
        }
    }
    #[doc = " "]
    #[doc = " DL/T 2811 Association Services ? Associate / Release / Abort "]
    #[doc = " "]
    #[doc = " ====================================================================== "]
    #[doc = " 8.2.1 Associate (Service Code 0x01) "]
    #[doc = " ====================================================================== "]
    #[doc = " Request: serverAccessPointReference + authenticationParameter "]
    #[doc = " Response+: associationId + serviceError + authenticationParameter "]
    #[doc = " Response-: serviceError "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "Associate-RequestPDU")]
    pub struct AssociateRequestPDU {
        #[rasn(
            size("0..=129"),
            tag(context, 0),
            identifier = "serverAccessPointReference"
        )]
        pub server_access_point_reference: Option<VisibleString>,
        #[rasn(tag(context, 1), identifier = "authenticationParameter")]
        pub authentication_parameter: Option<AssociateRequestPDUAuthenticationParameter>,
    }
    impl AssociateRequestPDU {
        pub fn new(
            server_access_point_reference: Option<VisibleString>,
            authentication_parameter: Option<AssociateRequestPDUAuthenticationParameter>,
        ) -> Self {
            Self {
                server_access_point_reference,
                authentication_parameter,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct AssociateResponsePDUAuthenticationParameter {
        #[rasn(tag(context, 0), identifier = "signatureCertificate")]
        pub signature_certificate: OctetString,
        #[rasn(tag(context, 1), identifier = "signedTime")]
        pub signed_time: UtcTime,
        #[rasn(tag(context, 2), identifier = "signedValue")]
        pub signed_value: OctetString,
    }
    impl AssociateResponsePDUAuthenticationParameter {
        pub fn new(
            signature_certificate: OctetString,
            signed_time: UtcTime,
            signed_value: OctetString,
        ) -> Self {
            Self {
                signature_certificate,
                signed_time,
                signed_value,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "Associate-ResponsePDU")]
    pub struct AssociateResponsePDU {
        #[rasn(size("0..=64"), tag(context, 0), identifier = "associationId")]
        pub association_id: OctetString,
        #[rasn(tag(context, 1), identifier = "serviceError")]
        pub service_error: ServiceError,
        #[rasn(tag(context, 2), identifier = "authenticationParameter")]
        pub authentication_parameter: Option<AssociateResponsePDUAuthenticationParameter>,
    }
    impl AssociateResponsePDU {
        pub fn new(
            association_id: OctetString,
            service_error: ServiceError,
            authentication_parameter: Option<AssociateResponsePDUAuthenticationParameter>,
        ) -> Self {
            Self {
                association_id,
                service_error,
                authentication_parameter,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct BRCB {
        #[rasn(size("0..=129"), tag(context, 1), identifier = "rptID")]
        pub rpt_id: VisibleString,
        #[rasn(tag(context, 2), identifier = "rptEna")]
        pub rpt_ena: bool,
        #[rasn(tag(context, 3), identifier = "datSet")]
        pub dat_set: ObjectReference,
        #[rasn(tag(context, 4), identifier = "confRev")]
        pub conf_rev: Int32U,
        #[rasn(tag(context, 5), identifier = "optFlds")]
        pub opt_flds: RcbOptFlds,
        #[rasn(tag(context, 6), identifier = "bufTm")]
        pub buf_tm: Int32U,
        #[rasn(tag(context, 7), identifier = "sqNum")]
        pub sq_num: Int16U,
        #[rasn(tag(context, 8), identifier = "trgOps")]
        pub trg_ops: TriggerConditions,
        #[rasn(tag(context, 9), identifier = "intgPd")]
        pub intg_pd: Int32U,
        #[rasn(tag(context, 10))]
        pub gi: bool,
        #[rasn(tag(context, 11), identifier = "purgeBuf")]
        pub purge_buf: bool,
        #[rasn(tag(context, 12), identifier = "entryID")]
        pub entry_id: EntryID,
        #[rasn(tag(context, 13), identifier = "timeOfEntry")]
        pub time_of_entry: EntryTime,
        #[rasn(tag(context, 14), identifier = "resvTms")]
        pub resv_tms: Option<Int16>,
        #[rasn(size("0..=64"), tag(context, 15))]
        pub owner: Option<OctetString>,
    }
    impl BRCB {
        pub fn new(
            rpt_id: VisibleString,
            rpt_ena: bool,
            dat_set: ObjectReference,
            conf_rev: Int32U,
            opt_flds: RcbOptFlds,
            buf_tm: Int32U,
            sq_num: Int16U,
            trg_ops: TriggerConditions,
            intg_pd: Int32U,
            gi: bool,
            purge_buf: bool,
            entry_id: EntryID,
            time_of_entry: EntryTime,
            resv_tms: Option<Int16>,
            owner: Option<OctetString>,
        ) -> Self {
            Self {
                rpt_id,
                rpt_ena,
                dat_set,
                conf_rev,
                opt_flds,
                buf_tm,
                sq_num,
                trg_ops,
                intg_pd,
                gi,
                purge_buf,
                entry_id,
                time_of_entry,
                resv_tms,
                owner,
            }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.2.2 BinaryTime / EntryTime "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct BinaryTime(pub FixedOctetString<6usize>);
    #[doc = " ====================================================================== "]
    #[doc = " 7.1.1 BOOLEAN "]
    #[doc = " ====================================================================== "]
    #[doc = " PER encoding: constrained integer (0..1), 1 bit "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=1"))]
    pub struct Boolean(pub u8);
    #[doc = " ====================================================================== "]
    #[doc = " 7.5.3 Check "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct Check(pub FixedBitString<2usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct ControlCode {
        pub next: bool,
        pub resp: bool,
        pub err: bool,
        pub pi: Int8U,
    }
    impl ControlCode {
        pub fn new(next: bool, resp: bool, err: bool, pi: Int8U) -> Self {
            Self {
                next,
                resp,
                err,
                pi,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "CreateDataSet-ErrorPDU")]
    pub struct CreateDataSetErrorPDU(pub ServiceError);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousCreateDataSetRequestPDUMemberData {
        #[rasn(tag(context, 0))]
        pub reference: ObjectReference,
        #[rasn(tag(context, 1))]
        pub fc: FunctionalConstraint,
    }
    impl AnonymousCreateDataSetRequestPDUMemberData {
        pub fn new(reference: ObjectReference, fc: FunctionalConstraint) -> Self {
            Self { reference, fc }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct CreateDataSetRequestPDUMemberData(
        pub SequenceOf<AnonymousCreateDataSetRequestPDUMemberData>,
    );
    #[doc = " ====================================================================== "]
    #[doc = " 8.5.3 CreateDataSet (Service Code 0x41) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "CreateDataSet-RequestPDU")]
    pub struct CreateDataSetRequestPDU {
        #[rasn(tag(context, 0), identifier = "datasetReference")]
        pub dataset_reference: ObjectReference,
        #[rasn(tag(context, 1), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
        #[rasn(tag(context, 2), identifier = "memberData")]
        pub member_data: CreateDataSetRequestPDUMemberData,
    }
    impl CreateDataSetRequestPDU {
        pub fn new(
            dataset_reference: ObjectReference,
            reference_after: Option<ObjectReference>,
            member_data: CreateDataSetRequestPDUMemberData,
        ) -> Self {
            Self {
                dataset_reference,
                reference_after,
                member_data,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(delegate, identifier = "CreateDataSet-ResponsePDU")]
    pub struct CreateDataSetResponsePDU(pub ());
    #[doc = " ====================================================================== "]
    #[doc = " 7.7 Data (CHOICE of 24 alternatives) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice)]
    pub enum Data {
        #[rasn(tag(context, 0))]
        error(ServiceError),
        #[rasn(tag(context, 1))]
        array(SequenceOf<Data>),
        #[rasn(tag(context, 2))]
        structure(SequenceOf<Data>),
        #[rasn(tag(context, 3))]
        boolean(bool),
        #[rasn(tag(context, 4))]
        int8(Int8),
        #[rasn(tag(context, 5))]
        int16(Int16),
        #[rasn(tag(context, 6))]
        int32(Int32),
        #[rasn(tag(context, 7))]
        int64(Int64),
        #[rasn(tag(context, 8))]
        int8u(Int8U),
        #[rasn(tag(context, 9))]
        int16u(Int16U),
        #[rasn(tag(context, 10))]
        int32u(Int32U),
        #[rasn(tag(context, 11))]
        int64u(Int64U),
        #[rasn(tag(context, 12))]
        float32(Float32),
        #[rasn(tag(context, 13))]
        float64(Float64),
        #[rasn(tag(context, 14), identifier = "bit-string")]
        bit_string(BitString),
        #[rasn(tag(context, 15), identifier = "octet-string")]
        octet_string(OctetString),
        #[rasn(tag(context, 16), identifier = "visible-string")]
        visible_string(VisibleString),
        #[rasn(tag(context, 17), identifier = "unicode-string")]
        unicode_string(Utf8String),
        #[rasn(tag(context, 18), identifier = "utc-time")]
        utc_time(UtcTime),
        #[rasn(tag(context, 19), identifier = "binary-time")]
        binary_time(BinaryTime),
        #[rasn(tag(context, 20))]
        quality(Quality),
        #[rasn(tag(context, 21))]
        dbpos(Dbpos),
        #[rasn(tag(context, 22))]
        tcmd(Tcmd),
        #[rasn(tag(context, 23))]
        check(Check),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct DataDefinitionArray {
        #[rasn(tag(context, 1), identifier = "numberOfElement")]
        pub number_of_element: Int32,
        #[rasn(tag(context, 2), identifier = "elementType")]
        pub element_type: DataDefinition,
    }
    impl DataDefinitionArray {
        pub fn new(number_of_element: Int32, element_type: DataDefinition) -> Self {
            Self {
                number_of_element,
                element_type,
            }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousDataDefinitionStructure {
        #[rasn(tag(context, 0))]
        pub name: ObjectName,
        #[rasn(tag(context, 1))]
        pub fc: Option<FunctionalConstraint>,
        #[rasn(tag(context, 2), identifier = "type")]
        pub r_type: DataDefinition,
    }
    impl AnonymousDataDefinitionStructure {
        pub fn new(
            name: ObjectName,
            fc: Option<FunctionalConstraint>,
            r_type: DataDefinition,
        ) -> Self {
            Self { name, fc, r_type }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct DataDefinitionStructure(pub SequenceOf<AnonymousDataDefinitionStructure>);
    #[doc = " ====================================================================== "]
    #[doc = " 7.8 DataDefinition "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice)]
    pub enum DataDefinition {
        #[rasn(tag(context, 0))]
        error(ServiceError),
        #[rasn(tag(context, 1))]
        array(Box<DataDefinitionArray>),
        #[rasn(tag(context, 2))]
        structure(DataDefinitionStructure),
        #[rasn(tag(context, 3))]
        boolean(()),
        #[rasn(tag(context, 4))]
        int8(()),
        #[rasn(tag(context, 5))]
        int16(()),
        #[rasn(tag(context, 6))]
        int32(()),
        #[rasn(tag(context, 7))]
        int64(()),
        #[rasn(tag(context, 8))]
        int8u(()),
        #[rasn(tag(context, 9))]
        int16u(()),
        #[rasn(tag(context, 10))]
        int32u(()),
        #[rasn(tag(context, 11))]
        int64u(()),
        #[rasn(tag(context, 12))]
        float32(()),
        #[rasn(tag(context, 13))]
        float64(()),
        #[rasn(tag(context, 14), identifier = "bit-string")]
        bit_string(Integer),
        #[rasn(tag(context, 15), identifier = "octet-string")]
        octet_string(Integer),
        #[rasn(tag(context, 16), identifier = "visible-string")]
        visible_string(Integer),
        #[rasn(tag(context, 17), identifier = "unicode-string")]
        unicode_string(Integer),
        #[rasn(tag(context, 18), identifier = "utc-time")]
        utc_time(()),
        #[rasn(tag(context, 19), identifier = "binary-time")]
        binary_time(()),
        #[rasn(tag(context, 20))]
        quality(()),
        #[rasn(tag(context, 21))]
        dbpos(()),
        #[rasn(tag(context, 22))]
        tcmd(()),
        #[rasn(tag(context, 23))]
        check(()),
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.5 Dbpos "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct Dbpos(pub FixedBitString<2usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "DeleteDataSet-ErrorPDU")]
    pub struct DeleteDataSetErrorPDU(pub ServiceError);
    #[doc = " ====================================================================== "]
    #[doc = " 8.5.4 DeleteDataSet (Service Code 0x42) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "DeleteDataSet-RequestPDU")]
    pub struct DeleteDataSetRequestPDU {
        #[rasn(tag(context, 0), identifier = "datasetReference")]
        pub dataset_reference: ObjectReference,
    }
    impl DeleteDataSetRequestPDU {
        pub fn new(dataset_reference: ObjectReference) -> Self {
            Self { dataset_reference }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(delegate, identifier = "DeleteDataSet-ResponsePDU")]
    pub struct DeleteDataSetResponsePDU(pub ());
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.8 EntryID "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct EntryID(pub FixedOctetString<8usize>);
    #[doc = " BinaryTime ::= SEQUENCE {     "]
    #[doc = "     msOfDay       Int32U,    milliseconds since midnight (0..86399999) "]
    #[doc = "     daysSince1984 Int16U     days since 1984-01-01 "]
    #[doc = " } "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct EntryTime(pub BinaryTime);
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.9 EntryTime "]
    #[doc = " ====================================================================== "]
    #[doc = " EntryTime ::= BinaryTime "]
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.10 FileEntry "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct FileEntry {
        #[rasn(size("0..=129"), tag(context, 0), identifier = "fileName")]
        pub file_name: VisibleString,
        #[rasn(tag(context, 1), identifier = "fileSize")]
        pub file_size: Int32U,
        #[rasn(tag(context, 2), identifier = "lastModified")]
        pub last_modified: UtcTime,
        #[rasn(tag(context, 3), identifier = "checkSum")]
        pub check_sum: Int32U,
    }
    impl FileEntry {
        pub fn new(
            file_name: VisibleString,
            file_size: Int32U,
            last_modified: UtcTime,
            check_sum: Int32U,
        ) -> Self {
            Self {
                file_name,
                file_size,
                last_modified,
                check_sum,
            }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.1.4 Floating-Point Types"]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct Float32(pub FixedOctetString<4usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct Float64(pub FixedOctetString<8usize>);
    #[doc = " ====================================================================== "]
    #[doc = " 7.4 FC "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("2"))]
    pub struct FunctionalConstraint(pub VisibleString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetAllCBValues-ErrorPDU")]
    pub struct GetAllCBValuesErrorPDU(pub ServiceError);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice)]
    pub enum GetAllCBValuesRequestPDUReference {
        #[rasn(tag(context, 0))]
        ldName(ObjectName),
        #[rasn(tag(context, 1))]
        lnReference(ObjectReference),
    }
    #[doc = " ====================================================================== "]
    #[doc = " 8.3.6 GetAllCBValues (Service Code 0x56) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetAllCBValues-RequestPDU")]
    pub struct GetAllCBValuesRequestPDU {
        #[rasn(tag(context, 0))]
        pub reference: GetAllCBValuesRequestPDUReference,
        #[rasn(tag(context, 1), identifier = "acsiClass")]
        pub acsi_class: ACSIClass,
        #[rasn(tag(context, 2), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
    }
    impl GetAllCBValuesRequestPDU {
        pub fn new(
            reference: GetAllCBValuesRequestPDUReference,
            acsi_class: ACSIClass,
            reference_after: Option<ObjectReference>,
        ) -> Self {
            Self {
                reference,
                acsi_class,
                reference_after,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice)]
    pub enum AnonymousGetAllCBValuesResponsePDUCbValueValue {
        #[rasn(tag(context, 0))]
        brcb(BRCB),
        #[rasn(tag(context, 1))]
        urcb(URCB),
        #[rasn(tag(context, 2))]
        lcb(LCB),
        #[rasn(tag(context, 3))]
        sgecb(SGECB),
        #[rasn(tag(context, 4))]
        gocb(GoCB),
        #[rasn(tag(context, 5))]
        msvcb(MSVCB),
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousGetAllCBValuesResponsePDUCbValue {
        #[rasn(tag(context, 0))]
        pub reference: SubReference,
        #[rasn(tag(context, 1))]
        pub value: AnonymousGetAllCBValuesResponsePDUCbValueValue,
    }
    impl AnonymousGetAllCBValuesResponsePDUCbValue {
        pub fn new(
            reference: SubReference,
            value: AnonymousGetAllCBValuesResponsePDUCbValueValue,
        ) -> Self {
            Self { reference, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct GetAllCBValuesResponsePDUCbValue(
        pub SequenceOf<AnonymousGetAllCBValuesResponsePDUCbValue>,
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetAllCBValues-ResponsePDU")]
    pub struct GetAllCBValuesResponsePDU {
        #[rasn(tag(context, 0), identifier = "cbValue")]
        pub cb_value: GetAllCBValuesResponsePDUCbValue,
        #[rasn(
            tag(context, 1),
            default = "get_all_cbvalues_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetAllCBValuesResponsePDU {
        pub fn new(cb_value: GetAllCBValuesResponsePDUCbValue, more_follows: bool) -> Self {
            Self {
                cb_value,
                more_follows,
            }
        }
    }
    fn get_all_cbvalues_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetAllDataDefinition-ErrorPDU")]
    pub struct GetAllDataDefinitionErrorPDU(pub ServiceError);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice)]
    pub enum GetAllDataDefinitionRequestPDUReference {
        #[rasn(tag(context, 0))]
        ldName(ObjectName),
        #[rasn(tag(context, 1))]
        lnReference(ObjectReference),
    }
    #[doc = " ====================================================================== "]
    #[doc = " 8.3.5 GetAllDataDefinition (Service Code 0x55) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetAllDataDefinition-RequestPDU")]
    pub struct GetAllDataDefinitionRequestPDU {
        #[rasn(tag(context, 0))]
        pub reference: GetAllDataDefinitionRequestPDUReference,
        #[rasn(tag(context, 1))]
        pub fc: Option<FunctionalConstraint>,
        #[rasn(tag(context, 2), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
    }
    impl GetAllDataDefinitionRequestPDU {
        pub fn new(
            reference: GetAllDataDefinitionRequestPDUReference,
            fc: Option<FunctionalConstraint>,
            reference_after: Option<ObjectReference>,
        ) -> Self {
            Self {
                reference,
                fc,
                reference_after,
            }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousGetAllDataDefinitionResponsePDUData {
        #[rasn(tag(context, 0))]
        pub reference: SubReference,
        #[rasn(tag(context, 1), identifier = "cdcType")]
        pub cdc_type: Option<VisibleString>,
        #[rasn(tag(context, 2))]
        pub definition: DataDefinition,
    }
    impl AnonymousGetAllDataDefinitionResponsePDUData {
        pub fn new(
            reference: SubReference,
            cdc_type: Option<VisibleString>,
            definition: DataDefinition,
        ) -> Self {
            Self {
                reference,
                cdc_type,
                definition,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct GetAllDataDefinitionResponsePDUData(
        pub SequenceOf<AnonymousGetAllDataDefinitionResponsePDUData>,
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetAllDataDefinition-ResponsePDU")]
    pub struct GetAllDataDefinitionResponsePDU {
        #[rasn(tag(context, 0))]
        pub data: GetAllDataDefinitionResponsePDUData,
        #[rasn(
            tag(context, 1),
            default = "get_all_data_definition_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetAllDataDefinitionResponsePDU {
        pub fn new(data: GetAllDataDefinitionResponsePDUData, more_follows: bool) -> Self {
            Self { data, more_follows }
        }
    }
    fn get_all_data_definition_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetAllDataValues-ErrorPDU")]
    pub struct GetAllDataValuesErrorPDU(pub ServiceError);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice)]
    pub enum GetAllDataValuesRequestPDUReference {
        #[rasn(tag(context, 0))]
        ldName(ObjectName),
        #[rasn(tag(context, 1))]
        lnReference(ObjectReference),
    }
    #[doc = " ====================================================================== "]
    #[doc = " 8.3.4 GetAllDataValues (Service Code 0x54) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetAllDataValues-RequestPDU")]
    pub struct GetAllDataValuesRequestPDU {
        #[rasn(tag(context, 0))]
        pub reference: GetAllDataValuesRequestPDUReference,
        #[rasn(tag(context, 1))]
        pub fc: Option<FunctionalConstraint>,
        #[rasn(tag(context, 2), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
    }
    impl GetAllDataValuesRequestPDU {
        pub fn new(
            reference: GetAllDataValuesRequestPDUReference,
            fc: Option<FunctionalConstraint>,
            reference_after: Option<ObjectReference>,
        ) -> Self {
            Self {
                reference,
                fc,
                reference_after,
            }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousGetAllDataValuesResponsePDUData {
        #[rasn(tag(context, 0))]
        pub reference: SubReference,
        #[rasn(tag(context, 1))]
        pub value: Data,
    }
    impl AnonymousGetAllDataValuesResponsePDUData {
        pub fn new(reference: SubReference, value: Data) -> Self {
            Self { reference, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct GetAllDataValuesResponsePDUData(
        pub SequenceOf<AnonymousGetAllDataValuesResponsePDUData>,
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetAllDataValues-ResponsePDU")]
    pub struct GetAllDataValuesResponsePDU {
        #[rasn(tag(context, 0))]
        pub data: GetAllDataValuesResponsePDUData,
        #[rasn(
            tag(context, 1),
            default = "get_all_data_values_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetAllDataValuesResponsePDU {
        pub fn new(data: GetAllDataValuesResponsePDUData, more_follows: bool) -> Self {
            Self { data, more_follows }
        }
    }
    fn get_all_data_values_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetDataDefinition-ErrorPDU")]
    pub struct GetDataDefinitionErrorPDU(pub ServiceError);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousGetDataDefinitionRequestPDUData {
        #[rasn(tag(context, 0))]
        pub reference: ObjectReference,
        #[rasn(tag(context, 1))]
        pub fc: Option<FunctionalConstraint>,
    }
    impl AnonymousGetDataDefinitionRequestPDUData {
        pub fn new(reference: ObjectReference, fc: Option<FunctionalConstraint>) -> Self {
            Self { reference, fc }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct GetDataDefinitionRequestPDUData(
        pub SequenceOf<AnonymousGetDataDefinitionRequestPDUData>,
    );
    #[doc = " ====================================================================== "]
    #[doc = " 8.4.4 GetDataDefinition (Service Code 0x13) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataDefinition-RequestPDU")]
    pub struct GetDataDefinitionRequestPDU {
        #[rasn(tag(context, 0))]
        pub data: GetDataDefinitionRequestPDUData,
    }
    impl GetDataDefinitionRequestPDU {
        pub fn new(data: GetDataDefinitionRequestPDUData) -> Self {
            Self { data }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousGetDataDefinitionResponsePDUData {
        #[rasn(tag(context, 0), identifier = "cdcType")]
        pub cdc_type: Option<VisibleString>,
        #[rasn(tag(context, 1))]
        pub definition: DataDefinition,
    }
    impl AnonymousGetDataDefinitionResponsePDUData {
        pub fn new(cdc_type: Option<VisibleString>, definition: DataDefinition) -> Self {
            Self {
                cdc_type,
                definition,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct GetDataDefinitionResponsePDUData(
        pub SequenceOf<AnonymousGetDataDefinitionResponsePDUData>,
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataDefinition-ResponsePDU")]
    pub struct GetDataDefinitionResponsePDU {
        #[rasn(tag(context, 0))]
        pub data: GetDataDefinitionResponsePDUData,
        #[rasn(
            tag(context, 1),
            default = "get_data_definition_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetDataDefinitionResponsePDU {
        pub fn new(data: GetDataDefinitionResponsePDUData, more_follows: bool) -> Self {
            Self { data, more_follows }
        }
    }
    fn get_data_definition_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetDataDirectory-ErrorPDU")]
    pub struct GetDataDirectoryErrorPDU(pub ServiceError);
    #[doc = " ====================================================================== "]
    #[doc = " 8.4.3 GetDataDirectory (Service Code 0x14) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataDirectory-RequestPDU")]
    pub struct GetDataDirectoryRequestPDU {
        #[rasn(tag(context, 0), identifier = "dataReference")]
        pub data_reference: ObjectReference,
        #[rasn(tag(context, 1), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
    }
    impl GetDataDirectoryRequestPDU {
        pub fn new(
            data_reference: ObjectReference,
            reference_after: Option<ObjectReference>,
        ) -> Self {
            Self {
                data_reference,
                reference_after,
            }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousGetDataDirectoryResponsePDUDataAttribute {
        #[rasn(tag(context, 0))]
        pub reference: SubReference,
        #[rasn(tag(context, 1))]
        pub fc: Option<FunctionalConstraint>,
    }
    impl AnonymousGetDataDirectoryResponsePDUDataAttribute {
        pub fn new(reference: SubReference, fc: Option<FunctionalConstraint>) -> Self {
            Self { reference, fc }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct GetDataDirectoryResponsePDUDataAttribute(
        pub SequenceOf<AnonymousGetDataDirectoryResponsePDUDataAttribute>,
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataDirectory-ResponsePDU")]
    pub struct GetDataDirectoryResponsePDU {
        #[rasn(tag(context, 0), identifier = "dataAttribute")]
        pub data_attribute: GetDataDirectoryResponsePDUDataAttribute,
        #[rasn(
            tag(context, 1),
            default = "get_data_directory_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetDataDirectoryResponsePDU {
        pub fn new(
            data_attribute: GetDataDirectoryResponsePDUDataAttribute,
            more_follows: bool,
        ) -> Self {
            Self {
                data_attribute,
                more_follows,
            }
        }
    }
    fn get_data_directory_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetDataSetDirectory-ErrorPDU")]
    pub struct GetDataSetDirectoryErrorPDU(pub ServiceError);
    #[doc = " ====================================================================== "]
    #[doc = " 8.5.5 GetDataSetDirectory (Service Code 0x43) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataSetDirectory-RequestPDU")]
    pub struct GetDataSetDirectoryRequestPDU {
        #[rasn(tag(context, 0), identifier = "datasetReference")]
        pub dataset_reference: ObjectReference,
        #[rasn(tag(context, 1), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
    }
    impl GetDataSetDirectoryRequestPDU {
        pub fn new(
            dataset_reference: ObjectReference,
            reference_after: Option<ObjectReference>,
        ) -> Self {
            Self {
                dataset_reference,
                reference_after,
            }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousGetDataSetDirectoryResponsePDUMemberData {
        #[rasn(tag(context, 0))]
        pub reference: ObjectReference,
        #[rasn(tag(context, 1))]
        pub fc: FunctionalConstraint,
    }
    impl AnonymousGetDataSetDirectoryResponsePDUMemberData {
        pub fn new(reference: ObjectReference, fc: FunctionalConstraint) -> Self {
            Self { reference, fc }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct GetDataSetDirectoryResponsePDUMemberData(
        pub SequenceOf<AnonymousGetDataSetDirectoryResponsePDUMemberData>,
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataSetDirectory-ResponsePDU")]
    pub struct GetDataSetDirectoryResponsePDU {
        #[rasn(tag(context, 0), identifier = "memberData")]
        pub member_data: GetDataSetDirectoryResponsePDUMemberData,
        #[rasn(
            tag(context, 1),
            default = "get_data_set_directory_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetDataSetDirectoryResponsePDU {
        pub fn new(
            member_data: GetDataSetDirectoryResponsePDUMemberData,
            more_follows: bool,
        ) -> Self {
            Self {
                member_data,
                more_follows,
            }
        }
    }
    fn get_data_set_directory_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetDataSetValues-ErrorPDU")]
    pub struct GetDataSetValuesErrorPDU(pub ServiceError);
    #[doc = " ====================================================================== "]
    #[doc = " 6.5.1 GetDataSetValues (Service Code 0x44) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataSetValues-RequestPDU")]
    pub struct GetDataSetValuesRequestPDU {
        #[rasn(tag(context, 0), identifier = "datasetReference")]
        pub dataset_reference: ObjectReference,
        #[rasn(tag(context, 1), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
    }
    impl GetDataSetValuesRequestPDU {
        pub fn new(
            dataset_reference: ObjectReference,
            reference_after: Option<ObjectReference>,
        ) -> Self {
            Self {
                dataset_reference,
                reference_after,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataSetValues-ResponsePDU")]
    pub struct GetDataSetValuesResponsePDU {
        #[rasn(tag(context, 0))]
        pub value: SequenceOf<Data>,
        #[rasn(
            tag(context, 1),
            default = "get_data_set_values_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetDataSetValuesResponsePDU {
        pub fn new(value: SequenceOf<Data>, more_follows: bool) -> Self {
            Self {
                value,
                more_follows,
            }
        }
    }
    fn get_data_set_values_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetDataValues-ErrorPDU")]
    pub struct GetDataValuesErrorPDU(pub ServiceError);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousGetDataValuesRequestPDUData {
        #[rasn(tag(context, 0))]
        pub reference: ObjectReference,
        #[rasn(tag(context, 1))]
        pub fc: Option<FunctionalConstraint>,
    }
    impl AnonymousGetDataValuesRequestPDUData {
        pub fn new(reference: ObjectReference, fc: Option<FunctionalConstraint>) -> Self {
            Self { reference, fc }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct GetDataValuesRequestPDUData(pub SequenceOf<AnonymousGetDataValuesRequestPDUData>);
    #[doc = " ====================================================================== "]
    #[doc = " 8.4.1 GetDataValues (Service Code 0x11) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataValues-RequestPDU")]
    pub struct GetDataValuesRequestPDU {
        #[rasn(tag(context, 0))]
        pub data: GetDataValuesRequestPDUData,
    }
    impl GetDataValuesRequestPDU {
        pub fn new(data: GetDataValuesRequestPDUData) -> Self {
            Self { data }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetDataValues-ResponsePDU")]
    pub struct GetDataValuesResponsePDU {
        #[rasn(tag(context, 0))]
        pub value: SequenceOf<Data>,
        #[rasn(
            tag(context, 1),
            default = "get_data_values_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetDataValuesResponsePDU {
        pub fn new(value: SequenceOf<Data>, more_follows: bool) -> Self {
            Self {
                value,
                more_follows,
            }
        }
    }
    fn get_data_values_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetLogicalDeviceDirectory-ErrorPDU")]
    pub struct GetLogicalDeviceDirectoryErrorPDU(pub ServiceError);
    #[doc = " ====================================================================== "]
    #[doc = " 8.3.2 GetLogicalDeviceDirectory (Service Code 0x52) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetLogicalDeviceDirectory-RequestPDU")]
    pub struct GetLogicalDeviceDirectoryRequestPDU {
        #[rasn(tag(context, 0), identifier = "ldName")]
        pub ld_name: Option<ObjectName>,
        #[rasn(tag(context, 1), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
    }
    impl GetLogicalDeviceDirectoryRequestPDU {
        pub fn new(ld_name: Option<ObjectName>, reference_after: Option<ObjectReference>) -> Self {
            Self {
                ld_name,
                reference_after,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetLogicalDeviceDirectory-ResponsePDU")]
    pub struct GetLogicalDeviceDirectoryResponsePDU {
        #[rasn(tag(context, 0), identifier = "lnReference")]
        pub ln_reference: SequenceOf<SubReference>,
        #[rasn(
            tag(context, 1),
            default = "get_logical_device_directory_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetLogicalDeviceDirectoryResponsePDU {
        pub fn new(ln_reference: SequenceOf<SubReference>, more_follows: bool) -> Self {
            Self {
                ln_reference,
                more_follows,
            }
        }
    }
    fn get_logical_device_directory_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetLogicalNodeDirectory-ErrorPDU")]
    pub struct GetLogicalNodeDirectoryErrorPDU(pub ServiceError);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice)]
    pub enum GetLogicalNodeDirectoryRequestPDUReference {
        #[rasn(tag(context, 0))]
        ldName(ObjectName),
        #[rasn(tag(context, 1))]
        lnReference(ObjectReference),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetLogicalNodeDirectory-RequestPDU")]
    pub struct GetLogicalNodeDirectoryRequestPDU {
        #[rasn(tag(context, 0))]
        pub reference: GetLogicalNodeDirectoryRequestPDUReference,
        #[rasn(tag(context, 1), identifier = "acsiClass")]
        pub acsi_class: ACSIClass,
        #[rasn(tag(context, 2), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
    }
    impl GetLogicalNodeDirectoryRequestPDU {
        pub fn new(
            reference: GetLogicalNodeDirectoryRequestPDUReference,
            acsi_class: ACSIClass,
            reference_after: Option<ObjectReference>,
        ) -> Self {
            Self {
                reference,
                acsi_class,
                reference_after,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetLogicalNodeDirectory-ResponsePDU")]
    pub struct GetLogicalNodeDirectoryResponsePDU {
        #[rasn(tag(context, 0))]
        pub reference: SequenceOf<SubReference>,
        #[rasn(
            tag(context, 1),
            default = "get_logical_node_directory_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetLogicalNodeDirectoryResponsePDU {
        pub fn new(reference: SequenceOf<SubReference>, more_follows: bool) -> Self {
            Self {
                reference,
                more_follows,
            }
        }
    }
    fn get_logical_node_directory_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GetServerDirectory-ErrorPDU")]
    pub struct GetServerDirectoryErrorPDU(pub ServiceError);
    #[doc = " ====================================================================== "]
    #[doc = " 8.3.1 GetServerDirectory (Service Code 0x51) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetServerDirectory-RequestPDU")]
    pub struct GetServerDirectoryRequestPDU {
        #[rasn(value("0..=2"), tag(context, 0), identifier = "objectClass")]
        pub object_class: u8,
        #[rasn(tag(context, 1), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
    }
    impl GetServerDirectoryRequestPDU {
        pub fn new(object_class: u8, reference_after: Option<ObjectReference>) -> Self {
            Self {
                object_class,
                reference_after,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "GetServerDirectory-ResponsePDU")]
    pub struct GetServerDirectoryResponsePDU {
        #[rasn(tag(context, 0))]
        pub reference: SequenceOf<ObjectReference>,
        #[rasn(
            tag(context, 1),
            default = "get_server_directory_response_pdu_more_follows_default",
            identifier = "moreFollows"
        )]
        pub more_follows: bool,
    }
    impl GetServerDirectoryResponsePDU {
        pub fn new(reference: SequenceOf<ObjectReference>, more_follows: bool) -> Self {
            Self {
                reference,
                more_follows,
            }
        }
    }
    fn get_server_directory_response_pdu_more_follows_default() -> bool {
        true
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct GoCB {
        #[rasn(tag(context, 1), identifier = "goEna")]
        pub go_ena: bool,
        #[rasn(size("0..=129"), tag(context, 2), identifier = "goID")]
        pub go_id: VisibleString,
        #[rasn(tag(context, 3), identifier = "datSet")]
        pub dat_set: ObjectReference,
        #[rasn(tag(context, 4), identifier = "confRev")]
        pub conf_rev: Int32U,
        #[rasn(tag(context, 5), identifier = "ndsCom")]
        pub nds_com: bool,
        #[rasn(tag(context, 6), identifier = "dstAddress")]
        pub dst_address: Option<PhyComAddr>,
    }
    impl GoCB {
        pub fn new(
            go_ena: bool,
            go_id: VisibleString,
            dat_set: ObjectReference,
            conf_rev: Int32U,
            nds_com: bool,
            dst_address: Option<PhyComAddr>,
        ) -> Self {
            Self {
                go_ena,
                go_id,
                dat_set,
                conf_rev,
                nds_com,
                dst_address,
            }
        }
    }
    #[doc = " PER encoding: constrained integer, 16 bits aligned "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("-32768..=32767"))]
    pub struct Int16(pub i16);
    #[doc = " PER encoding: constrained integer, 16 bits aligned "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=65535"))]
    pub struct Int16U(pub u16);
    #[doc = " PER encoding: constrained integer, 24 bits aligned "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=16777215"))]
    pub struct Int24U(pub u32);
    #[doc = " PER encoding: constrained integer, 32 bits aligned "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("-2147483648..=2147483647"))]
    pub struct Int32(pub i32);
    #[doc = " PER encoding: constrained integer, 32 bits aligned "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=4294967295"))]
    pub struct Int32U(pub u32);
    #[doc = " PER encoding: length determinant + content bytes "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("-9223372036854775808..=9223372036854775807"))]
    pub struct Int64(pub i64);
    #[doc = " PER encoding: length determinant + content bytes "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=18446744073709551615"))]
    pub struct Int64U(pub u64);
    #[doc = " ====================================================================== "]
    #[doc = " 7.1.2 Integer Types "]
    #[doc = " ====================================================================== "]
    #[doc = " PER encoding: constrained integer, 8 bits aligned "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("-128..=127"))]
    pub struct Int8(pub i8);
    #[doc = " PER encoding: constrained integer, 8 bits aligned "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=255"))]
    pub struct Int8U(pub u8);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct LCB {
        #[rasn(tag(context, 1), identifier = "logEna")]
        pub log_ena: bool,
        #[rasn(tag(context, 2), identifier = "datSet")]
        pub dat_set: ObjectReference,
        #[rasn(tag(context, 3), identifier = "trgOps")]
        pub trg_ops: TriggerConditions,
        #[rasn(tag(context, 4), identifier = "intgPd")]
        pub intg_pd: Int32U,
        #[rasn(tag(context, 5), identifier = "logRef")]
        pub log_ref: ObjectReference,
        #[rasn(tag(context, 6), identifier = "optFlds")]
        pub opt_flds: Option<LcbOptFlds>,
        #[rasn(tag(context, 7), identifier = "bufTm")]
        pub buf_tm: Option<Int32U>,
    }
    impl LCB {
        pub fn new(
            log_ena: bool,
            dat_set: ObjectReference,
            trg_ops: TriggerConditions,
            intg_pd: Int32U,
            log_ref: ObjectReference,
            opt_flds: Option<LcbOptFlds>,
            buf_tm: Option<Int32U>,
        ) -> Self {
            Self {
                log_ena,
                dat_set,
                trg_ops,
                intg_pd,
                log_ref,
                opt_flds,
                buf_tm,
            }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.6.5 LcbOptFlds "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct LcbOptFlds(pub FixedBitString<1usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct MSVCB {
        #[rasn(tag(context, 1), identifier = "svEna")]
        pub sv_ena: bool,
        #[rasn(size("0..=129"), tag(context, 2), identifier = "msvID")]
        pub msv_id: VisibleString,
        #[rasn(tag(context, 3), identifier = "datSet")]
        pub dat_set: ObjectReference,
        #[rasn(tag(context, 4), identifier = "confRev")]
        pub conf_rev: Int32U,
        #[rasn(tag(context, 5), identifier = "smpMod")]
        pub smp_mod: Option<SmpMod>,
        #[rasn(tag(context, 6), identifier = "smpRate")]
        pub smp_rate: Int16U,
        #[rasn(tag(context, 7), identifier = "optFlds")]
        pub opt_flds: MsvcbOptFlds,
        #[rasn(tag(context, 8), identifier = "dstAddress")]
        pub dst_address: Option<PhyComAddr>,
    }
    impl MSVCB {
        pub fn new(
            sv_ena: bool,
            msv_id: VisibleString,
            dat_set: ObjectReference,
            conf_rev: Int32U,
            smp_mod: Option<SmpMod>,
            smp_rate: Int16U,
            opt_flds: MsvcbOptFlds,
            dst_address: Option<PhyComAddr>,
        ) -> Self {
            Self {
                sv_ena,
                msv_id,
                dat_set,
                conf_rev,
                smp_mod,
                smp_rate,
                opt_flds,
                dst_address,
            }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.6.6 MsvcbOptFlds "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct MsvcbOptFlds(pub FixedBitString<5usize>);
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.1 ObjectName "]
    #[doc = " ====================================================================== "]
    #[doc = " PER encoding: VisibleString with max length 64 "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=64"))]
    pub struct ObjectName(pub VisibleString);
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.2 ObjectReference "]
    #[doc = " ====================================================================== "]
    #[doc = " Format: LDName/LNName[.Name[....]] "]
    #[doc = " PER encoding: VisibleString with max length 129 "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=129"))]
    pub struct ObjectReference(pub VisibleString);
    #[doc = " ====================================================================== "]
    #[doc = " 7.5.1 Control Blocks Attributes "]
    #[doc = " ====================================================================== "]
    #[doc = " ====================================================================== "]
    #[doc = " 7.5.2 Originator "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct Originator {
        #[rasn(value("0..=8"), tag(context, 0), identifier = "orCat")]
        pub or_cat: u8,
        #[rasn(size("0..=64"), tag(context, 1), identifier = "orIdent")]
        pub or_ident: OctetString,
    }
    impl Originator {
        pub fn new(or_cat: u8, or_ident: OctetString) -> Self {
            Self { or_cat, or_ident }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.12 PhyComAddr "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct PhyComAddr {
        #[rasn(size("6"), tag(context, 0))]
        pub addr: OctetString,
        #[rasn(tag(context, 1))]
        pub priority: Int8U,
        #[rasn(tag(context, 2))]
        pub vid: Int16U,
        #[rasn(tag(context, 3))]
        pub appid: Int16U,
    }
    impl PhyComAddr {
        pub fn new(addr: OctetString, priority: Int8U, vid: Int16U, appid: Int16U) -> Self {
            Self {
                addr,
                priority,
                vid,
                appid,
            }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.6 Quality "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct Quality(pub FixedBitString<13usize>);
    #[doc = " ====================================================================== "]
    #[doc = " 7.6.4 RcbOptFlds "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct RcbOptFlds(pub FixedBitString<10usize>);
    #[doc = " ====================================================================== "]
    #[doc = " 7.6.3 ReasonCode "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct ReasonCode(pub FixedBitString<7usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "Release-ErrorPDU")]
    pub struct ReleaseErrorPDU(pub ServiceError);
    #[doc = " ====================================================================== "]
    #[doc = " 8.2.2 Release (Service Code 0x02) "]
    #[doc = " ====================================================================== "]
    #[doc = " Release-RequestPDU: associationId "]
    #[doc = " Release-ResponsePDU: associationId + serviceError "]
    #[doc = " Release-ErrorPDU: serviceError "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "Release-RequestPDU")]
    pub struct ReleaseRequestPDU {
        #[rasn(size("0..=64"), tag(context, 0), identifier = "associationId")]
        pub association_id: OctetString,
    }
    impl ReleaseRequestPDU {
        pub fn new(association_id: OctetString) -> Self {
            Self { association_id }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "Release-ResponsePDU")]
    pub struct ReleaseResponsePDU {
        #[rasn(size("0..=64"), tag(context, 0), identifier = "associationId")]
        pub association_id: OctetString,
        #[rasn(tag(context, 1), identifier = "serviceError")]
        pub service_error: ServiceError,
    }
    impl ReleaseResponsePDU {
        pub fn new(association_id: OctetString, service_error: ServiceError) -> Self {
            Self {
                association_id,
                service_error,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct SGCB {
        #[rasn(tag(context, 1), identifier = "numOfSG")]
        pub num_of_sg: Int8U,
        #[rasn(tag(context, 2), identifier = "actSG")]
        pub act_sg: Int8U,
        #[rasn(tag(context, 3), identifier = "editSG")]
        pub edit_sg: Int8U,
        #[rasn(tag(context, 4), identifier = "tActEdt")]
        pub t_act_edt: TimeStamp,
        #[rasn(tag(context, 5), identifier = "resvTms")]
        pub resv_tms: Option<Int16U>,
    }
    impl SGCB {
        pub fn new(
            num_of_sg: Int8U,
            act_sg: Int8U,
            edit_sg: Int8U,
            t_act_edt: TimeStamp,
            resv_tms: Option<Int16U>,
        ) -> Self {
            Self {
                num_of_sg,
                act_sg,
                edit_sg,
                t_act_edt,
                resv_tms,
            }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.11 ServiceError "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=12"))]
    pub struct ServiceError(pub u8);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SetDataSetValues-ErrorPDU")]
    pub struct SetDataSetValuesErrorPDU {
        #[rasn(tag(context, 0))]
        pub result: SequenceOf<ServiceError>,
    }
    impl SetDataSetValuesErrorPDU {
        pub fn new(result: SequenceOf<ServiceError>) -> Self {
            Self { result }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 8.5.2 SetDataSetValues (Service Code 0x45) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SetDataSetValues-RequestPDU")]
    pub struct SetDataSetValuesRequestPDU {
        #[rasn(tag(context, 0), identifier = "datasetReference")]
        pub dataset_reference: ObjectReference,
        #[rasn(tag(context, 1), identifier = "referenceAfter")]
        pub reference_after: Option<ObjectReference>,
        #[rasn(tag(context, 2))]
        pub value: SequenceOf<Data>,
    }
    impl SetDataSetValuesRequestPDU {
        pub fn new(
            dataset_reference: ObjectReference,
            reference_after: Option<ObjectReference>,
            value: SequenceOf<Data>,
        ) -> Self {
            Self {
                dataset_reference,
                reference_after,
                value,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(delegate, identifier = "SetDataSetValues-ResponsePDU")]
    pub struct SetDataSetValuesResponsePDU(pub ());
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SetDataValues-ErrorPDU")]
    pub struct SetDataValuesErrorPDU {
        #[rasn(tag(context, 0))]
        pub result: SequenceOf<ServiceError>,
    }
    impl SetDataValuesErrorPDU {
        pub fn new(result: SequenceOf<ServiceError>) -> Self {
            Self { result }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SEQUENCE")]
    pub struct AnonymousSetDataValuesRequestPDUData {
        #[rasn(tag(context, 0))]
        pub reference: ObjectReference,
        #[rasn(tag(context, 1))]
        pub fc: Option<FunctionalConstraint>,
        #[rasn(tag(context, 2))]
        pub value: Data,
    }
    impl AnonymousSetDataValuesRequestPDUData {
        pub fn new(
            reference: ObjectReference,
            fc: Option<FunctionalConstraint>,
            value: Data,
        ) -> Self {
            Self {
                reference,
                fc,
                value,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct SetDataValuesRequestPDUData(pub SequenceOf<AnonymousSetDataValuesRequestPDUData>);
    #[doc = " ====================================================================== "]
    #[doc = " 8.4.2 SetDataValues (Service Code 0x12) "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(identifier = "SetDataValues-RequestPDU")]
    pub struct SetDataValuesRequestPDU {
        #[rasn(tag(context, 0))]
        pub data: SetDataValuesRequestPDUData,
    }
    impl SetDataValuesRequestPDU {
        pub fn new(data: SetDataValuesRequestPDUData) -> Self {
            Self { data }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(delegate, identifier = "SetDataValues-ResponsePDU")]
    pub struct SetDataValuesResponsePDU(pub ());
    #[doc = " ====================================================================== "]
    #[doc = " 7.6.7 SmpMod "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=2"))]
    pub struct SmpMod(pub u8);
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.3 SubReference "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=129"))]
    pub struct SubReference(pub VisibleString);
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.7 Tcmd "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct Tcmd(pub FixedBitString<2usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct TimeQuality(pub FixedBitString<8usize>);
    #[doc = " ====================================================================== "]
    #[doc = " 7.3.4 TimeStamp "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct TimeStamp(pub UtcTime);
    #[doc = " ====================================================================== "]
    #[doc = " 7.6.1 Control Blocks "]
    #[doc = " ====================================================================== "]
    #[doc = " SGCB  -> 8.6.6  "]
    #[doc = " BRCB  -> 8.7.2  "]
    #[doc = " URCB  -> 8.7.4  "]
    #[doc = " LCB   -> 8.8.2  "]
    #[doc = " GoCB  -> 8.9.4  "]
    #[doc = " MSVCB -> 8.10.2 "]
    #[doc = " ====================================================================== "]
    #[doc = " 7.6.2 TriggerConditions "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct TriggerConditions(pub FixedBitString<6usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    pub struct URCB {
        #[rasn(size("0..=129"), tag(context, 1), identifier = "rptID")]
        pub rpt_id: VisibleString,
        #[rasn(tag(context, 2), identifier = "rptEna")]
        pub rpt_ena: bool,
        #[rasn(tag(context, 3), identifier = "datSet")]
        pub dat_set: ObjectReference,
        #[rasn(tag(context, 4), identifier = "confRev")]
        pub conf_rev: Int32U,
        #[rasn(tag(context, 5), identifier = "optFlds")]
        pub opt_flds: RcbOptFlds,
        #[rasn(tag(context, 6), identifier = "bufTm")]
        pub buf_tm: Int32U,
        #[rasn(tag(context, 7), identifier = "sqNum")]
        pub sq_num: Int16U,
        #[rasn(tag(context, 8), identifier = "trgOps")]
        pub trg_ops: TriggerConditions,
        #[rasn(tag(context, 9), identifier = "intgPd")]
        pub intg_pd: Int32U,
        #[rasn(tag(context, 10))]
        pub gi: bool,
        #[rasn(tag(context, 14))]
        pub resv: bool,
        #[rasn(size("0..=64"), tag(context, 15))]
        pub owner: Option<OctetString>,
    }
    impl URCB {
        pub fn new(
            rpt_id: VisibleString,
            rpt_ena: bool,
            dat_set: ObjectReference,
            conf_rev: Int32U,
            opt_flds: RcbOptFlds,
            buf_tm: Int32U,
            sq_num: Int16U,
            trg_ops: TriggerConditions,
            intg_pd: Int32U,
            gi: bool,
            resv: bool,
            owner: Option<OctetString>,
        ) -> Self {
            Self {
                rpt_id,
                rpt_ena,
                dat_set,
                conf_rev,
                opt_flds,
                buf_tm,
                sq_num,
                trg_ops,
                intg_pd,
                gi,
                resv,
                owner,
            }
        }
    }
    #[doc = " ====================================================================== "]
    #[doc = " 7.1.5 String Types"]
    #[doc = " ====================================================================== "]
    #[doc = " PER encoding: length prefix + 8-bit characters "]
    #[doc = " VisibleString ::= VisibleString "]
    #[doc = " VisibleString constrained variants (used throughout the spec) "]
    #[doc = " VisibleString129 ::= VisibleString (SIZE (0..129)) "]
    #[doc = " VisibleString255 ::= VisibleString (SIZE (0..255)) "]
    #[doc = " PER encoding: length prefix + UTF-8 bytes "]
    #[doc = " UTF8String ::= UTF8String "]
    #[doc = " PER encoding: length prefix + raw bytes "]
    #[doc = " OctetString ::= OCTET STRING "]
    #[doc = " BitString ::= BIT STRING "]
    #[doc = " ====================================================================== "]
    #[doc = " 7.1.6 ENUMERATED "]
    #[doc = " ====================================================================== "]
    #[doc = " ENUMERATED ::= Int8 "]
    #[doc = " ENUMERATED ::= Int16 (>128) "]
    #[doc = " ====================================================================== "]
    #[doc = " 7.1.7 CODED ENUM "]
    #[doc = " ====================================================================== "]
    #[doc = " CODEDENUM ::= BIT STRING (SIZE(0..n)) "]
    #[doc = " ====================================================================== "]
    #[doc = " 7.1.8 Packed List "]
    #[doc = " ====================================================================== "]
    #[doc = " PackedList ::= BIT STRING (SIZE(0..n)) "]
    #[doc = " ====================================================================== "]
    #[doc = " 7.2.1 UtcTime "]
    #[doc = " ====================================================================== "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct UtcTime(pub FixedOctetString<8usize>);
}
