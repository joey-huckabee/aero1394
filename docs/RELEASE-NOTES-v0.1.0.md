# Aero1394 v0.1.0 release notes

- Release series: Initial development (`0.x`)
- Target tag: `v0.1.0`
- Supported artifact platforms: Windows x86-64 and Linux x86-64

## Highlights

- Bounded, offset-aware hexadecimal inspection for unknown capture bytes.
- Safe parsing of individual records and strict complete files in the confirmed
  internal BIE format.
- Bounded `Read`-based BIE framing that retains at most one encoded record plus
  fixed-size I/O buffers.
- A `records` command that lists raw BIE record identity, recorder time,
  status/length, unresolved flags, body length, and terminal offset.
- Precise diagnostics for truncated headers and bodies, missing terminators,
  trailing bytes, I/O failures, and absolute-offset overflow.
- Sanitized FireSpy-derived golden fixtures and generated requirements
  traceability.

## Evidence boundary

This release implements the internally defined BIE container contract. It does
not claim that BIE is a FireSpy or DAP Technology file format. The known-good
fixture values were derived from authorized BIE captures recorded with a
FireSpy; they are not randomly generated protocol examples.

The BIE `data_item_id` remains distinct from an AS5643 Message ID. The current
profile documents an evidence-backed mapping for data item `0x00005D04`, but
the generic parser and `records` output do not apply that interpretation.

## Deliberate exclusions

- IEEE-1394 packet decoding and CRC validation;
- AS5643 Health, Heartbeat, STOF, or VPC runtime decoding;
- application-payload and engineering-unit decoding;
- malformed-stream recovery or resynchronization;
- automatic format detection;
- stable CSV, Parquet, or other machine-readable schemas; and
- Python bindings or package publication.

These exclusions prevent observed or provisional values from being presented
as implemented protocol semantics.

## Usage

Inspect a bounded byte range:

```text
aero1394 hexdump capture.bie
```

List raw records from a complete BIE file:

```text
aero1394 records capture.bie
```

Use `aero1394 <COMMAND> --help` for command details.

## Compatibility

This is a pre-`1.0.0` initial-development release. Rust APIs and human-oriented
CLI formatting may evolve between minor versions. Future changes must remain
evidence-backed and must not silently reinterpret preserved raw values.
