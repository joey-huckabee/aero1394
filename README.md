# Aero1394

Aero1394 is a Rust-first toolkit for decoding and analyzing aerospace
IEEE-1394 traffic, with SAE AS5643 support as the primary protocol target.

The immediate goal is to decode `.bie` recordings believed to come from a DAP
Technologies FireSpy recorder. That provenance and the binary layout are still
working hypotheses: no BIE format has been verified in this repository yet.

## Project status

**Stage 1 forensic inspection.** The first Rust library and CLI vertical slice
is available. It performs bounded, offset-aware hex inspection without
asserting a BIE record schema. No representative BIE capture is present in the
repository, so BIE framing and protocol decoding remain unimplemented.

The next milestone is a small Rust forensic CLI that can characterize a sample
without embedding guessed field meanings in the public API.

Initial vendor research uncovered a format-provenance mismatch: DAP documents
native FireSpy recordings as `.fsr`, not `.bie`. See the
[BIE format research ledger](docs/BIE-FORMAT.md) before making container-format
assumptions.

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

Each line contains a 16-digit absolute file offset, hexadecimal bytes, and an
ASCII preview. The default 256-byte limit prevents an accidental terminal dump
of a large recording. Hex-dump output contains source bytes and must be handled
with the same sensitivity as the capture.

See [Reverse-engineering BIE captures](docs/REVERSE-ENGINEERING.md) for the
evidence workflow and [current architecture](docs/ARCHITECTURE.md) for the
library/CLI boundary.

## Processing model

```text
BIE capture ---------+
                     +--> IEEE-1394 --> AS5643 --> network profile --> analysis
Chapter 10 capture --+                                            --> signals
```

BIE is the first and urgent input format. Chapter 10 is a future input adapter,
not part of the initial implementation. Container parsing, bus decoding,
protocol decoding, network-specific interpretation, and analysis remain
separate layers.

## Planned deliverables

- a reusable Rust library;
- a Rust CLI, delivered for Windows first and kept portable to Linux;
- a Python package backed by the same Rust core through PyO3 for ETL use;
- an evidence-backed BIE format specification;
- IEEE-1394 and AS5643 decoding and validation;
- later profile, signal, timing, health, and anomaly analysis.

## Input needed for the first milestone

The most useful starting artifacts are:

1. a representative `.bie` capture, even if it is only a few megabytes;
2. an export of the same interval from the recorder software, if available;
3. recorder hardware and software version information; and
4. any non-sensitive network profile or ICD material needed to interpret the
   payload after the generic protocols are decoded.

Capture data must not be committed until its provenance, sensitivity, and
redistribution terms are understood. Small synthetic or sanitized fixtures can
then be derived for automated tests.

## Architecture decisions

The planning conversation that originally occupied this README has been
converted into detailed architecture decision records. See the
[ADR index](docs/adr/README.md) for the accepted baseline, proposals that still
need implementation evidence, and the staged delivery plan.

The project name and scope are established in
[ADR-0001](docs/adr/0001-name-the-project-aero1394.md).

## License

Licensed under the terms in [LICENSE](LICENSE).
