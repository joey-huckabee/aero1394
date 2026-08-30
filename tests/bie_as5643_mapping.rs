use aero1394::as5643::{ASSUMED_AS5643B_V1_PROFILE_ID, VpcValidationOutcome};
use aero1394::bie::{DataItemId, parse_file, parse_record};
use aero1394::bie_as5643::{
    ASSUMED_AS5643B_V1_BIE_DATA_ITEM_ID, BieAs5643MappingOutcome, map_bie_record_to_as5643,
};
use aero1394::forensic::FileOffset;

fn fixture_bytes(text: &str) -> Vec<u8> {
    text.split_whitespace()
        .map(|token| {
            assert_eq!(token.len(), 2, "fixture token must contain one byte");
            assert!(
                token.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "fixture token must be hexadecimal: {token}"
            );
            u8::from_str_radix(token, 16).expect("validated fixture hex")
        })
        .collect()
}

fn encoded_record(data_item_id: u32, stored_data: &[u8]) -> Vec<u8> {
    let stored_len = u32::try_from(stored_data.len()).expect("test body length fits u32");
    let mut bytes = Vec::with_capacity(16 + stored_data.len());
    bytes.extend_from_slice(&data_item_id.to_be_bytes());
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    bytes.extend_from_slice(&2_u32.to_be_bytes());
    bytes.extend_from_slice(&stored_len.to_be_bytes());
    bytes.extend_from_slice(stored_data);
    bytes
}

/// Requirements: L3-BIE-007, L3-BIE-009, L3-PRO-002, L3-PRO-003, L3-PRO-004
#[test]
fn maps_the_supported_bie_identity_and_layout_to_the_named_profile() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/end-four-records.hex"));
    let file = parse_file(&bytes, FileOffset::new(0)).expect("known-good BIE fixture parses");

    for record in file.records() {
        let mapping = map_bie_record_to_as5643(*record);
        assert_eq!(mapping.record(), *record);
        assert_eq!(mapping.profile_id(), Some(ASSUMED_AS5643B_V1_PROFILE_ID));

        let BieAs5643MappingOutcome::AssumedAs5643bV1(message) = mapping.outcome() else {
            panic!("supported record must map to the current profile");
        };
        assert_eq!(message.retained_bytes(), record.stored_data());
        assert_eq!(
            message.vpc_validation().outcome(),
            VpcValidationOutcome::Valid
        );
    }
}

/// Requirements: L3-BIE-007, L3-BIE-008, L3-BIE-009
#[test]
fn preserves_an_unknown_data_item_as_an_unsupported_mapping() {
    let bytes = encoded_record(0xDEAD_BEEF, &[0xA5; 116]);
    let (record, _) = parse_record(&bytes, FileOffset::new(0x200)).expect("test record parses");

    let mapping = map_bie_record_to_as5643(record);

    assert_eq!(mapping.record(), record);
    assert_eq!(mapping.record().stored_data(), &[0xA5; 116]);
    assert_eq!(mapping.profile_id(), None);
    assert_eq!(
        mapping.outcome(),
        BieAs5643MappingOutcome::UnsupportedDataItem
    );
}

/// Requirements: L3-BIE-007, L3-BIE-008, L3-BIE-009
#[test]
fn preserves_a_supported_id_with_an_unsupported_stored_length() {
    let bytes = encoded_record(ASSUMED_AS5643B_V1_BIE_DATA_ITEM_ID.get(), &[0x5A; 115]);
    let (record, _) = parse_record(&bytes, FileOffset::new(0x300)).expect("test record parses");

    let mapping = map_bie_record_to_as5643(record);

    assert_eq!(mapping.record(), record);
    assert_eq!(
        mapping.record().data_item_id(),
        DataItemId::new(0x0000_5D04)
    );
    assert_eq!(mapping.record().stored_data(), &[0x5A; 115]);
    assert_eq!(mapping.profile_id(), None);
    assert_eq!(
        mapping.outcome(),
        BieAs5643MappingOutcome::UnsupportedStoredDataLength {
            expected: 116,
            actual: 115,
        }
    );
}
