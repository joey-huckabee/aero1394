use aero1394::as5643::{
    ASSUMED_AS5643B_V1_APPLICATION_LEN, ASSUMED_AS5643B_V1_MESSAGE_ID,
    ASSUMED_AS5643B_V1_PROFILE_ID, decode_assumed_as5643b_v1,
};
use aero1394::bie::parse_file;
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

/// Requirements: L3-PRO-002, L3-PRO-003, L3-OUT-002
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
        assert_eq!(message.retained_bytes(), record.stored_data());
    }
}
