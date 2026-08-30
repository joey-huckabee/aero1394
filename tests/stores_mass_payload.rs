use aero1394::bie::parse_record;
use aero1394::bie_as5643::{BieAs5643MappingOutcome, map_bie_record_to_as5643};
use aero1394::forensic::FileOffset;
use aero1394::payload::PayloadWireType;
use aero1394::payload::msfcs_storesmassdata_b::{
    FIELD_DEFINITIONS, NOMINAL_SYSTEM_TICK_PERIOD_SECONDS, NOMINAL_SYSTEM_TICK_RATE_HZ,
    PAYLOAD_SIZE, StoresMassData, StoresMassWarning, decode,
};

fn hex_bytes(text: &str) -> Vec<u8> {
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

fn fixture_bytes(text: &str) -> Vec<u8> {
    let bytes = hex_bytes(text);
    assert_eq!(bytes.len(), PAYLOAD_SIZE, "fixture must be one payload");
    bytes
}

fn mapped_application(bytes: &[u8], record_offset: usize) -> &[u8] {
    let (record, _) = parse_record(
        bytes
            .get(record_offset..)
            .expect("source record offset is in bounds"),
        FileOffset::new(u64::try_from(record_offset).expect("test offset fits u64")),
    )
    .expect("source BIE record parses");
    let mapping = map_bie_record_to_as5643(record);
    let BieAs5643MappingOutcome::AssumedAs5643bV1(message) = mapping.outcome() else {
        panic!("source BIE record maps to the expected AS5643 profile");
    };
    message.application_data()
}

fn mass_bits(fields: StoresMassData) -> [u32; 10] {
    [
        fields.weight().bits(),
        fields.cg_fs().bits(),
        fields.cg_bl().bits(),
        fields.cg_wl().bits(),
        fields.ixx().bits(),
        fields.iyy().bits(),
        fields.izz().bits(),
        fields.ixy().bits(),
        fields.iyz().bits(),
        fields.ixz().bits(),
    ]
}

/// Requirements: L3-PAY-003, L3-PAY-013
#[test]
fn field_definitions_match_every_supplied_name_type_and_offset() {
    let expected = [
        ("TimeStamp", 0, PayloadWireType::Unsigned64),
        ("MessageValid", 8, PayloadWireType::Boolean8),
        ("EOTS_Present", 9, PayloadWireType::Boolean8),
        ("spare_byte", 10, PayloadWireType::Boolean8),
        ("CM_Present", 11, PayloadWireType::Boolean8),
        ("CurrentStoresMassData.Weight", 12, PayloadWireType::Float32),
        ("CurrentStoresMassData.Cg_FS", 16, PayloadWireType::Float32),
        ("CurrentStoresMassData.Cg_BL", 20, PayloadWireType::Float32),
        ("CurrentStoresMassData.Cg_WL", 24, PayloadWireType::Float32),
        ("CurrentStoresMassData.Ixx", 28, PayloadWireType::Float32),
        ("CurrentStoresMassData.Iyy", 32, PayloadWireType::Float32),
        ("CurrentStoresMassData.Izz", 36, PayloadWireType::Float32),
        ("CurrentStoresMassData.Ixy", 40, PayloadWireType::Float32),
        ("CurrentStoresMassData.Iyz", 44, PayloadWireType::Float32),
        ("CurrentStoresMassData.Ixz", 48, PayloadWireType::Float32),
        ("PostEJStoresMassData.Weight", 52, PayloadWireType::Float32),
        ("PostEJStoresMassData.Cg_FS", 56, PayloadWireType::Float32),
        ("PostEJStoresMassData.Cg_BL", 60, PayloadWireType::Float32),
        ("PostEJStoresMassData.Cg_WL", 64, PayloadWireType::Float32),
        ("PostEJStoresMassData.Ixx", 68, PayloadWireType::Float32),
        ("PostEJStoresMassData.Iyy", 72, PayloadWireType::Float32),
        ("PostEJStoresMassData.Izz", 76, PayloadWireType::Float32),
        ("PostEJStoresMassData.Ixy", 80, PayloadWireType::Float32),
        ("PostEJStoresMassData.Iyz", 84, PayloadWireType::Float32),
        ("PostEJStoresMassData.Ixz", 88, PayloadWireType::Float32),
    ];

    assert_eq!(FIELD_DEFINITIONS.len(), expected.len());
    for (actual, (name, offset, wire_type)) in FIELD_DEFINITIONS.iter().zip(expected) {
        assert_eq!(actual.name(), name);
        assert_eq!(actual.byte_offset(), offset);
        assert_eq!(actual.wire_type(), wire_type);
    }
}

/// Requirements: L3-TIM-007, L3-TIM-008, L3-PAY-007, L3-PAY-013, L3-PAY-014
#[test]
fn populated_fixture_decodes_all_fields_to_expected_raw_values() {
    let bytes = fixture_bytes(include_str!(
        "fixtures/payload/msfcs_storesmassdata_b/populated.hex"
    ));
    let payload = decode(&bytes).expect("populated payload decodes");

    assert_eq!(payload.system_ticks().get(), 0x0000_24E6_14F0_13B3);
    assert_eq!(payload.message_valid().get(), 0x01);
    assert_eq!(payload.eots_present().get(), 0x00);
    assert_eq!(payload.spare_byte().get(), 0x00);
    assert_eq!(payload.cm_present().get(), 0x00);
    assert_eq!(payload.message_valid().as_bool(), Some(true));
    assert_eq!(payload.eots_present().as_bool(), Some(false));
    assert_eq!(payload.cm_present().as_bool(), Some(false));
    assert!(payload.warnings().is_empty());
    assert_eq!(NOMINAL_SYSTEM_TICK_RATE_HZ, 13_600_000_000);
    assert!((NOMINAL_SYSTEM_TICK_PERIOD_SECONDS - 73.529_411_764_7e-12).abs() < 1e-22);
    assert!(
        (payload.system_ticks().provisional_elapsed_seconds()
            - 40_570_612_356_019_f64 / 13_600_000_000_f64)
            .abs()
            < f64::EPSILON
    );
    assert_eq!(
        mass_bits(payload.current_stores_mass_data()),
        [
            0x45AF_1829,
            0x43F5_0000,
            0x3F80_0000,
            0x428C_0000,
            0x4709_0A00,
            0x4497_C000,
            0x470D_1300,
            0x4214_0000,
            0x40A0_0000,
            0x42D2_0000,
        ]
    );
    assert_eq!(
        mass_bits(payload.post_ej_stores_mass_data()),
        [
            0x44BA_00A4,
            0x43FB_0000,
            0x4080_0000,
            0x42A2_0000,
            0x461D_4400,
            0x4378_0000,
            0x4620_A800,
            0x41B8_0000,
            0xC100_0000,
            0x4220_0000,
        ]
    );
    assert_eq!(payload.raw_bytes(), bytes);
}

/// Requirements: L3-TIM-007, L3-TIM-008, L3-PAY-007, L3-PAY-013, L3-PAY-014
#[test]
fn sparse_fixture_preserves_distinct_raw_flags_and_zero_fields() {
    let bytes = fixture_bytes(include_str!(
        "fixtures/payload/msfcs_storesmassdata_b/sparse-startup.hex"
    ));
    let payload = decode(&bytes).expect("sparse payload decodes");

    assert_eq!(payload.system_ticks().get(), 0x0000_24B7_DC01_E3E3);
    assert_eq!(payload.message_valid().get(), 0x00);
    assert_eq!(payload.eots_present().get(), 0x01);
    assert_eq!(payload.spare_byte().get(), 0x00);
    assert_eq!(payload.cm_present().get(), 0x00);
    assert_eq!(payload.message_valid().as_bool(), Some(false));
    assert_eq!(payload.eots_present().as_bool(), Some(true));
    assert_eq!(payload.warnings(), [StoresMassWarning::MessageInvalid]);
    assert_eq!(
        mass_bits(payload.current_stores_mass_data()),
        [0, 0x43E1_0000, 0, 0x427A_0000, 0, 0, 0, 0, 0, 0,]
    );
    assert_eq!(
        mass_bits(payload.post_ej_stores_mass_data()),
        [0, 0x43E1_0000, 0, 0x427A_0000, 0, 0, 0, 0, 0, 0,]
    );
    assert_eq!(payload.raw_bytes(), bytes);
}

/// Requirements: L3-PAY-009, L3-PAY-013, L3-PAY-014, L3-OUT-002
#[test]
fn semantic_warnings_preserve_unusual_boolean_float_and_reserved_values() {
    let mut bytes = fixture_bytes(include_str!(
        "fixtures/payload/msfcs_storesmassdata_b/populated.hex"
    ));
    bytes[8] = 0x02;
    bytes[9] = 0x03;
    bytes[10] = 0x02;
    bytes[11] = 0xFF;
    bytes[12..16].copy_from_slice(&0x7FC0_1234_u32.to_be_bytes());
    bytes[88..92].copy_from_slice(&0x7F80_0000_u32.to_be_bytes());

    let payload = decode(&bytes).expect("warning-bearing payload still decodes");

    assert_eq!(payload.message_valid().get(), 0x02);
    assert_eq!(payload.message_valid().as_bool(), None);
    assert!(payload.current_stores_mass_data().weight().value().is_nan());
    assert!(
        payload
            .post_ej_stores_mass_data()
            .ixz()
            .value()
            .is_infinite()
    );
    assert_eq!(payload.raw_bytes(), bytes);
    assert_eq!(
        payload.warnings(),
        [
            StoresMassWarning::InvalidBooleanEncoding {
                field: "MessageValid",
                value: 0x02,
            },
            StoresMassWarning::InvalidBooleanEncoding {
                field: "EOTS_Present",
                value: 0x03,
            },
            StoresMassWarning::InvalidBooleanEncoding {
                field: "spare_byte",
                value: 0x02,
            },
            StoresMassWarning::InvalidBooleanEncoding {
                field: "CM_Present",
                value: 0xFF,
            },
            StoresMassWarning::ReservedByteNonZero { value: 0x02 },
            StoresMassWarning::NonFiniteFloat {
                field: "CurrentStoresMassData.Weight",
                bits: 0x7FC0_1234,
            },
            StoresMassWarning::NonFiniteFloat {
                field: "PostEJStoresMassData.Ixz",
                bits: 0x7F80_0000,
            },
        ]
    );
}

/// Requirements: L3-PAY-013, L3-TST-006
#[test]
fn payload_fixtures_equal_their_documented_source_bie_regions() {
    let populated = fixture_bytes(include_str!(
        "fixtures/payload/msfcs_storesmassdata_b/populated.hex"
    ));
    let sparse = fixture_bytes(include_str!(
        "fixtures/payload/msfcs_storesmassdata_b/sparse-startup.hex"
    ));
    let end_bie = hex_bytes(include_str!("fixtures/bie/end-four-records.hex"));
    let startup_bie = hex_bytes(include_str!("fixtures/bie/startup-four-records.hex"));

    assert_eq!(populated, mapped_application(&end_bie, 3 * 132));
    assert_eq!(sparse, mapped_application(&startup_bie, 0));
}
