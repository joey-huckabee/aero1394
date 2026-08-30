use aero1394::bie::{BieFileParseError, DataItemId, parse_file, parse_record};
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

/// Requirements: L3-TST-001, L3-BIE-004
#[test]
fn empty_recording_is_one_zero_word() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/empty-recording.hex"));

    assert_eq!(bytes, [0, 0, 0, 0]);
    let file = parse_file(&bytes, FileOffset::new(0)).expect("empty BIE fixture parses");
    assert!(file.is_empty());
    assert_eq!(file.terminator_offset(), FileOffset::new(0));
}

/// Requirements: L3-TST-001, L3-BIE-001, L3-BIE-002, L3-BIE-003,
/// Requirements: L3-BIE-006, L3-BIE-007
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
        let (record, consumed) = parse_record(
            &bytes[base..],
            FileOffset::new(u64::try_from(base).expect("fixture offset fits u64")),
        )
        .expect("known-good BIE record parses");

        assert_eq!(consumed, 132);
        assert_eq!(record.file_offset().get(), base as u64);
        assert_eq!(record.data_item_id(), DataItemId::new(0x0000_5D04));
        assert_eq!(record.recorder_time().seconds(), 0x66AA_369B);
        assert_eq!(
            record.recorder_time().microseconds(),
            microseconds[record_index]
        );
        assert_eq!(
            record.status_and_length().raw(),
            status_lengths[record_index]
        );
        assert_eq!(record.status_and_length().data_length(), 116);
        assert_eq!(record.stored_data(), &bytes[base + 16..base + 132]);

        assert_common_record(&bytes, base);
        assert_eq!(be_u32(&bytes, base + 4), 0x66AA_369B);
        assert_eq!(be_u32(&bytes, base + 8), microseconds[record_index]);
        assert_eq!(be_u32(&bytes, base + 12), status_lengths[record_index]);
        assert_eq!(be_u64(&bytes, base + 24), payload_ticks[record_index]);
        assert_eq!(be_u32(&bytes, base + 128), vpcs[record_index]);
    }

    assert_eq!(
        parse_file(&bytes, FileOffset::new(0)).expect_err("excerpt has no file terminator"),
        BieFileParseError::MissingTerminator {
            offset: FileOffset::new(528),
        }
    );
}

/// Requirements: L3-TST-001, L3-BIE-004
#[test]
fn end_fixture_preserves_four_records_and_original_terminator() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/end-four-records.hex"));
    let microseconds = [0x0004_247F, 0x0004_82CD, 0x0004_B3A1, 0x0005_1549];
    let status_lengths = [0x0000_0074, 0x4000_0074, 0x4000_0074, 0x0000_0074];
    let heartbeats = [0x049C_BDEE, 0x049C_BF8E, 0x049C_C149, 0x049C_C304];
    let payload_ticks = [
        0x0000_24E5_EC3B_E6C4,
        0x0000_24E5_FA16_0810,
        0x0000_24E6_07EB_19B1,
        0x0000_24E6_14F0_13B3,
    ];
    let vpcs = [0xED45_F5A5, 0xFB68_1911, 0x0695_7674, 0x158E_7E3B];

    assert_eq!(bytes.len(), 532);
    let file = parse_file(&bytes, FileOffset::new(0)).expect("complete end fixture parses");
    assert_eq!(file.records().len(), 4);
    assert_eq!(file.terminator_offset(), FileOffset::new(528));
    assert_eq!(file.encoded_len(), bytes.len());

    for record_index in 0..4 {
        let base = record_index * 132;
        assert_eq!(
            file.records()[record_index].file_offset().get(),
            base as u64
        );
        assert_eq!(
            file.records()[record_index].data_item_id(),
            DataItemId::new(0x0000_5D04)
        );
        assert_common_record(&bytes, base);
        assert_eq!(be_u32(&bytes, base + 4), 0x66AA_36AA);
        assert_eq!(be_u32(&bytes, base + 8), microseconds[record_index]);
        assert_eq!(be_u32(&bytes, base + 12), status_lengths[record_index]);
        assert_eq!(be_u32(&bytes, base + 20), heartbeats[record_index]);
        assert_eq!(be_u64(&bytes, base + 24), payload_ticks[record_index]);
        assert_eq!(be_u32(&bytes, base + 32), 0x0100_0000);
        assert_eq!(be_u32(&bytes, base + 128), vpcs[record_index]);
    }
    assert_eq!(f32::from_bits(be_u32(&bytes, 40)), 490.0);
    assert_eq!(f32::from_bits(be_u32(&bytes, 108)), -8.0);
    assert_eq!(&bytes[528..], &[0, 0, 0, 0]);
}
