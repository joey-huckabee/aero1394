use aero1394::as5643::{
    ASSUMED_AS5643B_V1_APPLICATION_LEN, ASSUMED_AS5643B_V1_MESSAGE_ID,
    ASSUMED_AS5643B_V1_PROFILE_ID, VpcValidationOutcome, decode_assumed_as5643b_v1,
};
use aero1394::bie::{parse_file, parse_record};
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

/// Requirements: L3-PRO-002, L3-PRO-003, L3-PRO-004, L3-PRO-005, L3-OUT-002
#[test]
fn decodes_raw_profile_fields_from_known_good_bie_records() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/end-four-records.hex"));
    let file = parse_file(&bytes, FileOffset::new(0)).expect("known-good BIE fixture parses");
    let heartbeats = [0x049C_BDEE, 0x049C_BF8E, 0x049C_C149, 0x049C_C304];
    let payload_ticks = [
        0x0000_24E5_EC3B_E6C4,
        0x0000_24E5_FA16_0810,
        0x0000_24E6_07EB_19B1,
        0x0000_24E6_14F0_13B3,
    ];
    let vpcs = [0xED45_F5A5, 0xFB68_1911, 0x0695_7674, 0x158E_7E3B];

    for (index, record) in file.records().iter().enumerate() {
        let message =
            decode_assumed_as5643b_v1(ASSUMED_AS5643B_V1_MESSAGE_ID, record.stored_data())
                .expect("known-good retained AS5643 representation decodes");

        assert_eq!(message.profile_id(), ASSUMED_AS5643B_V1_PROFILE_ID);
        assert!(message.assumption_dependent());
        assert_eq!(message.message_id(), ASSUMED_AS5643B_V1_MESSAGE_ID);
        assert_eq!(message.reserved_security(), 0);
        assert_eq!(message.node_id(), 0);
        assert_eq!(message.priority_and_payload_length(), 0x0000_0064);
        assert_eq!(message.health_status(), 0);
        assert_eq!(message.heartbeat(), heartbeats[index]);
        assert_eq!(
            message.application_data().len(),
            ASSUMED_AS5643B_V1_APPLICATION_LEN
        );
        assert_eq!(
            u64::from_be_bytes(
                message.application_data()[..8]
                    .try_into()
                    .expect("profile guarantees an eight-byte prefix"),
            ),
            payload_ticks[index]
        );
        assert_eq!(message.stof_transmit_offset(), 1_400);
        assert_eq!(message.stof_receive_offset(), 500);
        assert_eq!(message.stof_datapump_offset(), 500);
        assert_eq!(message.stored_vpc(), vpcs[index]);
        let vpc = message.vpc_validation();
        assert_eq!(vpc.outcome(), VpcValidationOutcome::Valid);
        assert_eq!(vpc.profile_id(), Some(ASSUMED_AS5643B_V1_PROFILE_ID));
        assert!(vpc.assumption_dependent());
        assert_eq!(vpc.stored_vpc(), Some(vpcs[index]));
        assert_eq!(vpc.calculated_vpc(), Some(vpcs[index]));
        assert_eq!(vpc.not_checked_reason(), None);
        let inputs = vpc
            .calculation_inputs()
            .expect("selected profile supplies every VPC input");
        assert_eq!(inputs.message_id(), ASSUMED_AS5643B_V1_MESSAGE_ID);
        assert_eq!(inputs.reserved_security(), 0);
        assert_eq!(inputs.node_id(), 0);
        assert_eq!(inputs.priority_and_payload_length(), 0x0000_0064);
        assert_eq!(inputs.header_xor(), 0x0000_5D60);
        assert_eq!(inputs.protected_bytes(), &record.stored_data()[..112]);
        assert_eq!(message.retained_bytes(), record.stored_data());
    }
}

/// Requirements: L3-PRO-004, L3-PRO-005
#[test]
fn validates_every_startup_fixture_vpc() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/startup-four-records.hex"));
    let expected_vpcs = [0x2769_9B11, 0x11CE_B626, 0x0315_0B9E, 0xFEEB_8E2D];

    for (index, expected_vpc) in expected_vpcs.into_iter().enumerate() {
        let record_offset = index * 132;
        let (record, consumed) = parse_record(
            &bytes[record_offset..],
            FileOffset::new(u64::try_from(record_offset).expect("fixture offset fits u64")),
        )
        .expect("known-good BIE record parses");
        assert_eq!(consumed, 132);

        let message =
            decode_assumed_as5643b_v1(ASSUMED_AS5643B_V1_MESSAGE_ID, record.stored_data())
                .expect("known-good retained AS5643 representation decodes");
        let validation = message.vpc_validation();

        assert_eq!(validation.outcome(), VpcValidationOutcome::Valid);
        assert_eq!(validation.stored_vpc(), Some(expected_vpc));
        assert_eq!(validation.calculated_vpc(), Some(expected_vpc));
    }
}

/// Requirements: L3-PRO-004, L3-PRO-005
#[test]
fn reports_a_mutated_protected_word_as_invalid_without_losing_evidence() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/end-four-records.hex"));
    let file = parse_file(&bytes, FileOffset::new(0)).expect("known-good BIE fixture parses");
    let retained_bytes = file.records()[0].stored_data();
    let original = decode_assumed_as5643b_v1(ASSUMED_AS5643B_V1_MESSAGE_ID, retained_bytes)
        .expect("known-good retained AS5643 representation decodes");
    let original_vpc = original.vpc_validation();
    assert_eq!(original_vpc.outcome(), VpcValidationOutcome::Valid);

    let mut mutated_bytes = retained_bytes.to_vec();
    mutated_bytes[8] ^= 0x01;
    let mutated = decode_assumed_as5643b_v1(ASSUMED_AS5643B_V1_MESSAGE_ID, &mutated_bytes)
        .expect("mutation preserves the profile geometry");
    let validation = mutated.vpc_validation();

    assert_eq!(validation.outcome(), VpcValidationOutcome::Invalid);
    assert_eq!(validation.profile_id(), Some(ASSUMED_AS5643B_V1_PROFILE_ID));
    assert!(validation.assumption_dependent());
    assert_eq!(validation.stored_vpc(), original_vpc.stored_vpc());
    assert_ne!(validation.calculated_vpc(), validation.stored_vpc());
    assert_eq!(
        validation
            .calculation_inputs()
            .expect("selected profile supplies every VPC input")
            .protected_bytes(),
        &mutated_bytes[..112]
    );
    assert_eq!(mutated.retained_bytes(), mutated_bytes);
}
