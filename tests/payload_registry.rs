use aero1394::bie::parse_file;
use aero1394::bie_as5643::{BieAs5643MappingOutcome, map_bie_record_to_as5643};
use aero1394::forensic::FileOffset;
use aero1394::payload::msfcs_storesmassdata_b;
use aero1394::payload::{KnownPayload, PayloadContext, PayloadSelection, select_payload};

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

/// Requirements: L3-PAY-001, L3-PAY-004, L3-PAY-005
#[test]
fn selects_the_stores_mass_definition_after_protocol_decoding() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/end-four-records.hex"));
    let file = parse_file(&bytes, FileOffset::new(0)).expect("known-good BIE fixture parses");
    let record = file.records()[0];
    let mapping = map_bie_record_to_as5643(record);
    let BieAs5643MappingOutcome::AssumedAs5643bV1(message) = mapping.outcome() else {
        panic!("known BIE record must map to the provisional AS5643 profile");
    };

    let selection = select_payload(
        PayloadContext::new(record.data_item_id().get()),
        message.application_data(),
    );
    let PayloadSelection::Matched(matched) = selection else {
        panic!("known identity and application size must select one payload");
    };

    assert_eq!(*matched.definition(), msfcs_storesmassdata_b::DEFINITION);
    assert_eq!(matched.raw().bytes(), message.application_data());
    assert_eq!(matched.raw().size(), msfcs_storesmassdata_b::PAYLOAD_SIZE);
    let KnownPayload::MsfcsStoresMassDataB(payload) = matched
        .decode()
        .expect("built-in match has a typed decoder");
    assert_eq!(payload.raw_bytes(), message.application_data());
    assert_eq!(payload.message_valid().get(), 0x01);
}

/// Requirements: L3-PAY-004, L3-PAY-005, L3-PAY-006, L3-TST-005
#[test]
fn built_in_registry_preserves_an_unknown_payload() {
    let bytes = [0xA5; 92];
    let context = PayloadContext::new(0xDEAD_BEEF)
        .with_data_code("unknown-bus")
        .with_configuration("unknown-configuration");

    let PayloadSelection::Unknown(raw) = select_payload(context, &bytes) else {
        panic!("unregistered identity must remain unknown");
    };

    assert_eq!(raw.context(), context);
    assert_eq!(raw.size(), bytes.len());
    assert_eq!(raw.bytes(), bytes);
}

/// Requirements: L3-PAY-004, L3-PAY-005, L3-PAY-006
#[test]
fn built_in_registry_rejects_a_known_identity_with_another_size() {
    let bytes = [0x5A; 91];
    let context = PayloadContext::new(msfcs_storesmassdata_b::DATA_ITEM_ID);

    let PayloadSelection::Unknown(raw) = select_payload(context, &bytes) else {
        panic!("payload size is part of the registry key");
    };

    assert_eq!(
        raw.context().data_item_id(),
        msfcs_storesmassdata_b::DATA_ITEM_ID
    );
    assert_eq!(raw.size(), 91);
    assert_eq!(raw.bytes(), bytes);
}
