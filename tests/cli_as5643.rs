use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn complete_bie(data_item_id: u32, stored_data: &[u8]) -> Vec<u8> {
    let stored_len = u32::try_from(stored_data.len()).expect("test body length fits u32");
    let mut bytes = Vec::with_capacity(20 + stored_data.len());
    bytes.extend_from_slice(&data_item_id.to_be_bytes());
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    bytes.extend_from_slice(&2_u32.to_be_bytes());
    bytes.extend_from_slice(&stored_len.to_be_bytes());
    bytes.extend_from_slice(stored_data);
    bytes.extend_from_slice(&[0; 4]);
    bytes
}

fn temporary_capture(name: &str, bytes: &[u8]) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "aero1394-as5643-{name}-{}-{nonce}.bie",
        std::process::id()
    ));
    fs::write(&path, bytes).expect("write temporary capture");
    path
}

fn run_as5643(name: &str, bytes: &[u8]) -> Output {
    let path = temporary_capture(name, bytes);
    let output = Command::new(env!("CARGO_BIN_EXE_aero1394"))
        .args(["as5643", path.to_str().expect("Unicode temporary path")])
        .output()
        .expect("run aero1394 as5643");
    fs::remove_file(path).expect("remove temporary capture");
    output
}

/// Requirements: L3-BIE-009, L3-PRO-002, L3-PRO-003, L3-PRO-004, L3-OUT-002
#[test]
fn as5643_lists_mapped_envelope_and_vpc_values() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/end-four-records.hex"));

    let output = run_as5643("mapped", &bytes);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("ASCII stdout");
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 5);
    assert!(lines[0].starts_with(
        "record=0 offset=0x0000000000000000 data_item_id=0x00005D04 recorder_seconds=1722431146 recorder_microseconds=271487 status_and_length=0x00000074 unresolved_flags=0x00000000 data_length=116 as5643=mapped profile=aero1394-assumed-as5643b-v1 assumption_dependent=true message_id=0x00005D04 reserved_security=0x00000000 node_id=0x00000000 priority_and_payload_length=0x00000064 health_status=0x00000000 heartbeat=0x049CBDEE application_length=92 stof_transmit_offset=1400 stof_receive_offset=500 stof_datapump_offset=500 stored_vpc=0xED45F5A5 calculated_vpc=0xED45F5A5 vpc=valid payload=matched payload_name=msfcs_storesmassdata_b payload_definition=layout-v1 payload_size=92 payload_byte_order=big-endian"
    ));
    assert!(lines[0].contains(
        "payload_decode=raw_fields system_ticks=40569929459396 message_valid=0x01 eots_present=0x00 spare_byte=0x00 cm_present=0x00"
    ));
    assert!(lines[0].contains("current_weight=5603.02 current_cg_fs=490"));
    assert!(lines[0].ends_with("post_ej_iyz=-8 post_ej_ixz=40"));
    assert!(lines[1].contains("unresolved_flags=0x40000000"));
    assert!(lines[2].contains("unresolved_flags=0x40000000"));
    assert!(lines[3].contains("stored_vpc=0x158E7E3B calculated_vpc=0x158E7E3B vpc=valid"));
    assert!(lines[3].contains(
        "payload=matched payload_name=msfcs_storesmassdata_b payload_definition=layout-v1"
    ));
    assert_eq!(lines[4], "terminator_offset=0x0000000000000210 records=4");
}

/// Requirements: L3-BIE-007, L3-BIE-008, L3-BIE-009, L3-OUT-002
#[test]
fn as5643_labels_an_unknown_data_item_without_failing() {
    let bytes = complete_bie(0xDEAD_BEEF, &[0xA5; 4]);

    let output = run_as5643("unknown-id", &bytes);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).expect("ASCII stdout"),
        "record=0 offset=0x0000000000000000 data_item_id=0xDEADBEEF recorder_seconds=1 recorder_microseconds=2 status_and_length=0x00000004 unresolved_flags=0x00000000 data_length=4 as5643=unsupported reason=data_item_id\nterminator_offset=0x0000000000000014 records=1\n"
    );
}

/// Requirements: L3-BIE-007, L3-BIE-008, L3-BIE-009, L3-OUT-002
#[test]
fn as5643_labels_a_wrong_sized_supported_item_without_failing() {
    let bytes = complete_bie(0x0000_5D04, &[0x5A; 4]);

    let output = run_as5643("wrong-size", &bytes);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).expect("ASCII stdout"),
        "record=0 offset=0x0000000000000000 data_item_id=0x00005D04 recorder_seconds=1 recorder_microseconds=2 status_and_length=0x00000004 unresolved_flags=0x00000000 data_length=4 as5643=unsupported reason=stored_data_length expected=116 actual=4\nterminator_offset=0x0000000000000014 records=1\n"
    );
}
