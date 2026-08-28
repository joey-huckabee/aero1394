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

fn be_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("fixture has a complete u32"),
    )
}

fn be_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("fixture has a complete u64"),
    )
}

fn assert_common_record(bytes: &[u8], base: usize) {
    assert_eq!(be_u32(bytes, base), 0x0000_5D04);
    assert_eq!(be_u32(bytes, base + 12) & 0xFFFF, 116);
    assert_eq!(be_u32(bytes, base + 116), 1_400);
    assert_eq!(be_u32(bytes, base + 120), 500);
    assert_eq!(be_u32(bytes, base + 124), 500);
}

fn assert_observed_vpc_residual(bytes: &[u8], base: usize) {
    let visible_xor = (base + 16..base + 128)
        .step_by(4)
        .map(|offset| be_u32(bytes, offset))
        .fold(0, |accumulator, word| accumulator ^ word);
    let calculated_vpc = !visible_xor;
    let stored_vpc = be_u32(bytes, base + 128);

    assert_eq!(calculated_vpc ^ stored_vpc, 0x0000_5D60);
}

#[test]
fn empty_recording_is_one_zero_word() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/empty-recording.hex"));

    assert_eq!(bytes, [0, 0, 0, 0]);
}

#[test]
fn startup_fixture_preserves_four_consecutive_records() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/startup-four-records.hex"));
    let microseconds = [733_129, 745_629, 758_129, 783_129];
    let status_lengths = [0x0000_0074, 0x4000_0074, 0x4000_0074, 0x0000_0074];
    let payload_ticks = [
        0x0000_24B7_DC01_E3E3,
        0x0000_24B7_EAA6_C807,
        0x0000_24B7_F87D_77FB,
        0x0000_24B8_0582_0DA2,
    ];
    let vpcs = [0x2769_9B11, 0x11CE_B626, 0x0315_0B9E, 0xFEEB_8E2D];

    assert_eq!(bytes.len(), 528);
    for record_index in 0..4 {
        let base = record_index * 132;
        assert_common_record(&bytes, base);
        assert_observed_vpc_residual(&bytes, base);
        assert_eq!(be_u32(&bytes, base + 4), 0x66AA_369B);
        assert_eq!(be_u32(&bytes, base + 8), microseconds[record_index]);
        assert_eq!(be_u32(&bytes, base + 12), status_lengths[record_index]);
        assert_eq!(be_u64(&bytes, base + 24), payload_ticks[record_index]);
        assert_eq!(be_u32(&bytes, base + 128), vpcs[record_index]);
    }
}

#[test]
fn end_fixture_preserves_four_records_and_original_terminator() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/end-four-records.hex"));
    let microseconds = [0x0004_247F, 0x0004_82CD, 0x0004_B3A1, 0x0005_1549];
    let status_lengths = [0x0000_0074, 0x4000_0074, 0x4000_0074, 0x0000_0074];
    let protocol_word_1 = [0x049C_BDEE, 0x049C_BF8E, 0x049C_C149, 0x049C_C304];
    let payload_ticks = [
        0x0000_24E5_EC3B_E6C4,
        0x0000_24E5_FA16_0810,
        0x0000_24E6_07EB_19B1,
        0x0000_24E6_14F0_13B3,
    ];
    let vpcs = [0xED45_F5A5, 0xFB68_1911, 0x0695_7674, 0x158E_7E3B];

    assert_eq!(bytes.len(), 532);
    for record_index in 0..4 {
        let base = record_index * 132;
        assert_common_record(&bytes, base);
        assert_observed_vpc_residual(&bytes, base);
        assert_eq!(be_u32(&bytes, base + 4), 0x66AA_36AA);
        assert_eq!(be_u32(&bytes, base + 8), microseconds[record_index]);
        assert_eq!(be_u32(&bytes, base + 12), status_lengths[record_index]);
        assert_eq!(be_u32(&bytes, base + 20), protocol_word_1[record_index]);
        assert_eq!(be_u64(&bytes, base + 24), payload_ticks[record_index]);
        assert_eq!(be_u32(&bytes, base + 32), 0x0100_0000);
        assert_eq!(be_u32(&bytes, base + 128), vpcs[record_index]);
    }
    assert_eq!(f32::from_bits(be_u32(&bytes, 40)), 490.0);
    assert_eq!(f32::from_bits(be_u32(&bytes, 108)), -8.0);
    assert_eq!(&bytes[528..], &[0, 0, 0, 0]);
}
