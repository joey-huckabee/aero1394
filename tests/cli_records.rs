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

fn temporary_capture(name: &str, bytes: &[u8]) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "aero1394-records-{name}-{}-{nonce}.bie",
        std::process::id()
    ));
    fs::write(&path, bytes).expect("write temporary capture");
    path
}

fn run_records(name: &str, bytes: &[u8]) -> Output {
    let path = temporary_capture(name, bytes);
    let output = Command::new(env!("CARGO_BIN_EXE_aero1394"))
        .args(["records", path.to_str().expect("Unicode temporary path")])
        .output()
        .expect("run aero1394 records");
    fs::remove_file(path).expect("remove temporary capture");
    output
}

/// Requirements: L3-BIE-004, L3-OUT-001, L3-OUT-002, L3-OUT-006, L3-TST-001
#[test]
fn records_lists_the_complete_end_fixture() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/end-four-records.hex"));

    let output = run_records("complete", &bytes);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("ASCII stdout");
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 5);
    assert_eq!(
        lines[0],
        "record=0 offset=0x0000000000000000 data_item_id=0x00005D04 recorder_seconds=1722431146 recorder_microseconds=271487 status_and_length=0x00000074 unresolved_flags=0x00000000 data_length=116"
    );
    assert!(lines[1].contains("unresolved_flags=0x40000000"));
    assert!(lines[2].contains("unresolved_flags=0x40000000"));
    assert_eq!(lines[4], "terminator_offset=0x0000000000000210 records=4");
}

/// Requirements: L3-BIE-004, L3-TST-001
#[test]
fn records_accepts_the_sentinel_only_fixture() {
    let bytes = fixture_bytes(include_str!("fixtures/bie/empty-recording.hex"));

    let output = run_records("empty", &bytes);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).expect("ASCII stdout"),
        "terminator_offset=0x0000000000000000 records=0\n"
    );
}

/// Requirements: L3-BIE-005
#[test]
fn records_reports_a_missing_terminator() {
    let bytes = [0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0];

    let output = run_records("missing-terminator", &bytes);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("ASCII stderr"),
        "error: missing BIE file terminator at 0x0000000000000010\n"
    );
}

/// Requirements: L3-BIE-005
#[test]
fn records_reports_a_truncated_header() {
    let output = run_records("truncated-header", &[0, 0, 1]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("ASCII stderr"),
        "error: truncated BIE record header at 0x0000000000000000: needed 16 bytes, available 3\n"
    );
}

/// Requirements: L3-BIE-004
#[test]
fn records_reports_trailing_data() {
    let output = run_records("trailing", &[0, 0, 0, 0, 0xAA]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("ASCII stderr"),
        "error: 1 trailing byte after BIE terminator at 0x0000000000000004\n"
    );
}
