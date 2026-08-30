# Aero1394

Aero1394 is a Rust-first toolkit for decoding and analyzing aerospace
IEEE-1394 traffic, with SAE AS5643 support as the primary protocol target.

The immediate goal is to decode internally defined `.bie` recordings used by
the simulation workflow. Sanitized record excerpts support the current BIE
record family and its implementation contract.

## Project status

**`v0.1.0` published; `v0.2.0` release-candidate hardening underway.**
The Rust library and CLI perform bounded, offset-aware hex inspection. Supplied
simulation excerpts and recorder summary metadata establish a 16-byte
big-endian header and length-delimited stored data for the observed record
family. The library now safely parses one complete non-terminator BIE record
or a strict complete BIE byte slice while preserving raw values, absolute
offsets, and exact stored data. Whole-file parsing requires the four-byte zero
sentinel and reports truncation, a missing terminator, or trailing bytes.
The `records` CLI lists those raw BIE fields without assigning protocol
semantics. The `as5643` library module now decodes the confirmed retained
116-byte representation under the explicit assumption-dependent profile while
leaving its 92 application bytes opaque. A separate BIE adapter maps only data
item `0x00005D04` with exactly 116 stored bytes, and the `as5643` CLI exposes
the decoded envelope and VPC finding without changing generic BIE parsing. A
separate deterministic payload registry now recognizes the 92-byte
`msfcs_storesmassdata_b` layout and preserves unknown or ambiguous application
bytes. Its raw decoder exposes unsigned system ticks, four Boolean-designated
bytes, and twenty unscaled big-endian `f32` fields while retaining every
original bit. An additive provisional semantic view applies strict Boolean
encoding, validity and reserved-byte findings, non-finite-float warnings, and
nominal 13.6 GHz elapsed-time conversion. Confirmed engineering units,
coordinate/reference conventions, and timestamp epoch remain evidence-gated
beyond `v0.2.0`.

The first initial-development release, [`v0.1.0`](https://github.com/joey-huckabee/aero1394/releases/tag/v0.1.0),
is published with verified Windows and Linux archives and checksums. Further
protocol and payload semantics remain evidence-gated.

The frozen candidate scope and remaining release gates are in the
[`v0.2.0` release plan](docs/RELEASE-PLAN-v0.2.0.md). Every development
increment is recorded in the [changelog](CHANGELOG.md) before it is committed.
The completed [`v0.1.0` release plan](docs/RELEASE-PLAN.md) remains as release
evidence.

See the [internal BIE format contract](docs/BIE-FORMAT.md) for the supported
grammar, explicit field status, and unresolved status flag. Capture provenance
and specification-development history are retained separately in the
[BIE development evidence](docs/BIE-EVIDENCE.md).

## Build and inspect a capture

The project pins Rust 1.98.0. Build and run the first 256 bytes of a capture:

```text
cargo run --release -- hexdump path/to/capture.bie
```

Inspect a specific range using decimal or hexadecimal byte counts:

```text
cargo run --release -- hexdump path/to/capture.bie --offset 0x1000 --length 512
```

Change the line width or deliberately dump the entire file:

```text
cargo run --release -- hexdump path/to/capture.bie --width 32
cargo run --release -- hexdump path/to/capture.bie --length all > capture.hexdump.txt
```

List the raw container fields for every record in a complete BIE file:

```text
cargo run --release -- records path/to/capture.bie
```

The inventory reports the record index, absolute offset, BIE data-item ID, raw
recorder timestamp fields, raw status/length word, unresolved flags, and body
length. It does not label the BIE ID as an AS5643 Message ID or decode stored
data. The command makes two bounded passes: the first validates without
emitting output, then the second rewinds and renders while retaining at most
one 65,551-byte BIE record plus fixed-size I/O buffers.

Decode the supported assumption-dependent AS5643 envelope from a complete BIE
file:

```text
cargo run --release -- as5643 path/to/capture.bie
```

The command prints the BIE identity and raw metadata, selected profile and
assumption marker, reconstructed ASM-header words, Health Status, Heartbeat,
application length, STOF offsets, stored/calculated VPC, and validation result.
For a mapped envelope, it also reports whether the application bytes matched a
built-in payload definition, including the definition name, Aero1394 layout
version, exact size, and byte order. The registered Stores Mass payload also
prints all raw primitive fields plus provisional Boolean, validity, warning,
and elapsed-time interpretations. Float values have no confirmed units,
Boolean polarity remains explicitly provisional, and system ticks have no
confirmed epoch.
Unknown data-item IDs and a known ID with another stored-data length are
reported as `unsupported` while remaining successful inspectable records. This
human-readable output is not yet a stable machine schema.

Each hex-dump line contains a 16-digit absolute file offset, hexadecimal bytes,
and an ASCII preview. The default 256-byte limit prevents an accidental
terminal dump of a large recording. Hex-dump output contains source bytes and
must be handled with the same sensitivity as the capture.

## Build a release candidate

After the locked release build, create and smoke-test a deterministic Windows
archive and checksum with:

```text
cargo build --release --locked
python scripts/package-release.py --platform windows-x86_64 --archive-format zip --binary target/release/aero1394.exe
```

CI runs the corresponding ZIP or `tar.gz` packaging path on Windows and Linux.
Manual workflow runs and version tags retain the candidates as workflow
artifacts, but do not publish a GitHub release; `v0.1.0` was published manually
after artifact inspection. See the
[`v0.2.0` release notes](docs/RELEASE-NOTES-v0.2.0.md) for the candidate scope
and the [`v0.1.0` release notes](docs/RELEASE-NOTES-v0.1.0.md) for the previous
shipped boundary.

See [Reverse-engineering BIE captures](docs/REVERSE-ENGINEERING.md) for the
evidence workflow and [current architecture](docs/ARCHITECTURE.md) for the
library/CLI boundary.

## Processing model

```text
BIE capture --> IEEE-1394 --> AS5643 --> built-in payload --> analysis
                                                               --> signals
```

BIE is the current input format. Container parsing, bus decoding, protocol
decoding, network-specific interpretation, and analysis remain separate
layers. Candidate future input adapters are listed only as forward work in the
[roadmap](ROADMAP.md).

## Planned deliverables

- a reusable Rust library;
- a Rust CLI, delivered for Windows first and kept portable to Linux;
- a Python package backed by the same Rust core through PyO3 for ETL use;
- an evidence-backed BIE format specification;
- IEEE-1394 and AS5643 decoding and validation;
- later built-in payload, signal, timing, health, and anomaly analysis.

## Input needed for the next milestones

The most useful next artifacts are:

1. another complete `.bie` capture with a different data-item size or ID;
2. an export of the same interval from the recorder software, if available;
3. recorder hardware and software version information; and
4. the remaining authorized application-payload structures and field metadata.

Capture data must not be committed until its provenance, sensitivity, and
redistribution terms are understood. Small synthetic or sanitized fixtures can
then be derived for automated tests.

See the [L1 product requirements](docs/L1.md),
[L2 architectural requirements](docs/L2.md),
[L3 implementation requirements](docs/L3.md), and generated
[trace matrix](docs/TRACE-MATRIX.md) for testable behavior and live
traceability. [Built-in payload definitions](docs/PAYLOADS.md) describe the
Rust-native extension model and current application-definition evidence. The
[provisional output schemas](docs/OUTPUTS.md) preserve the CSV, Parquet, and
time presentation direction without making it part of BIE parsing. Future-only
work is maintained in [`ROADMAP.md`](ROADMAP.md).

## Architecture decisions

The planning conversation that originally occupied this README has been
converted into detailed architecture decision records. See the
[ADR index](docs/adr/README.md) for the accepted baseline, proposals that still
need implementation evidence, and the staged delivery plan.

The project name and scope are established in
[ADR-0001](docs/adr/0001-name-the-project-aero1394.md).

## License

Licensed under the terms in [LICENSE](LICENSE).
