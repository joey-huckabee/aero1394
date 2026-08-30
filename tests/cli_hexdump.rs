use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_capture(name: &str, bytes: &[u8]) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "aero1394-{name}-{}-{nonce}.bin",
        std::process::id()
    ));
    fs::write(&path, bytes).expect("write temporary capture");
    path
}

#[test]
fn hexdump_obeys_offset_and_length() {
    let path = temporary_capture("bounded", &[0x00, 0x01, 0x02, b'A', 0xFF, 0x05]);
    let output = Command::new(env!("CARGO_BIN_EXE_aero1394"))
        .args([
            "hexdump",
            path.to_str().expect("Unicode temporary path"),
            "--offset",
            "0x2",
            "--length",
            "3",
        ])
        .output()
        .expect("run aero1394");
    fs::remove_file(path).expect("remove temporary capture");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("ASCII stdout");
    assert!(stdout.starts_with("0000000000000002  02 41 FF"));
    assert!(stdout.ends_with("|.A.|\n"));
    assert_eq!(stdout.lines().count(), 1);
}

#[test]
fn explicit_all_reads_to_eof_in_multiple_lines() {
    let path = temporary_capture("all", &[0, 1, 2, 3, 4, 5]);
    let output = Command::new(env!("CARGO_BIN_EXE_aero1394"))
        .args([
            "hexdump",
            path.to_str().expect("Unicode temporary path"),
            "--length",
            "all",
            "--width",
            "4",
        ])
        .output()
        .expect("run aero1394");
    fs::remove_file(path).expect("remove temporary capture");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("ASCII stdout");
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("0000000000000000  00 01 02 03"));
    assert!(lines[1].starts_with("0000000000000004  04 05"));
}

#[test]
fn offset_beyond_eof_is_an_operational_error() {
    let path = temporary_capture("offset", &[0, 1]);
    let output = Command::new(env!("CARGO_BIN_EXE_aero1394"))
        .args([
            "hexdump",
            path.to_str().expect("Unicode temporary path"),
            "--offset",
            "3",
        ])
        .output()
        .expect("run aero1394");
    fs::remove_file(path).expect("remove temporary capture");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("ASCII stderr")
            .contains("beyond the 2-byte file")
    );
}

/// Requirements: L3-OUT-007
#[test]
fn usage_errors_return_exit_code_one() {
    let output = Command::new(env!("CARGO_BIN_EXE_aero1394"))
        .arg("unknown-command")
        .output()
        .expect("run aero1394");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("ASCII stderr")
            .contains("error: unknown command 'unknown-command'")
    );
}
