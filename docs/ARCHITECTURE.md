# Aero1394 architecture

- Status: Stage 1 implementation; Stage 2 framing evidence available
- Last updated: 2026-08-28

## Current vertical slice

Aero1394 is one Rust package with a library target and a thin CLI binary, as
accepted in [ADR-0004](adr/0004-start-as-a-modular-single-crate.md). Only the
modules needed by implemented behavior exist.

```text
CLI: argument parsing, File open/seek, text rendering, exit behavior
                              |
                              v
Library: forensic::Hexdump -> HexdumpLine { absolute offset, raw bytes }
                              |
                              v
                         std::io::Read
```

The `forensic` module is format-neutral. It does not identify a source as BIE,
FSR, FSP, RGN, IEEE-1394, or AS5643 and does not assign semantic names to file
bytes. Its iterator:

- consumes any `Read` implementation already positioned by its caller;
- emits owned, bounded lines with an explicit `FileOffset`;
- stops at the selected byte limit or EOF;
- guarantees progress by rejecting a zero-byte line width;
- bounds line allocations to at most 256 bytes;
- handles interrupted reads; and
- reports I/O and absolute-offset overflow separately.

The CLI owns filesystem behavior because opening a path, checking its size,
seeking, writing terminal output, and choosing process exit codes are adapter
concerns. The library can therefore be reused with memory buffers, extracted
Chapter 10 payloads, or another storage adapter without invoking a process.

## Dependency direction

The binary depends on the library. The library never depends on the binary,
terminal formatting, filesystem paths, or operating-system-specific behavior.
The package currently has no third-party runtime dependencies and forbids
unsafe code.

Future BIE and protocol parsers will operate on byte slices, following
[ADR-0006](adr/0006-use-safe-slice-oriented-parsers-and-explicit-wire-types.md).
The streaming forensic reader will supply those slices but will not absorb
their parsing, validation, policy, or recovery responsibilities.

## Expected growth

Modules are added only with evidence-backed behavior. The expected dependency
direction is:

```text
forensic / input adapter -> bie -> ieee1394 -> as5643 -> payload -> analysis
```

The `payload` module is the Rust-native implementation of the profile knowledge
described by ADR-0002. Definitions are compiled into the tool and selected by
an explicit registry as decided in ADR-0012; external YAML profiles are not a
runtime requirement.

An outer container may produce raw captured bytes without successfully
decoding the next layer. Each layer must preserve the bytes and source offsets
needed to inspect unsupported or invalid input. Chapter 10 remains a future
input adapter rather than a dependency of BIE decoding.

A module should become a separate crate only when the extraction conditions in
ADR-0004 are met. Current public types are deliberately small so the package
layout does not prematurely stabilize an unknown BIE model.

## Evidence available for the next slice

The supplied BIE excerpts establish one 16-byte big-endian, length-delimited
record header and a strong zero-word EOF inference. The parser should implement
only that generic header and length boundary first, preserving each stored
region as raw bytes.

Protocol-envelope interpretation and the future typed payload registry remain
downstream operations. The complete evidence map is in `BIE-FORMAT.md`; payload
knowledge is in `PAYLOADS.md`; testable commitments are in `REQUIREMENTS.md`.

## Verification

Unit tests cover bounds, offsets, EOF, explicit unbounded mode, zero-length
requests, overflow, argument parsing, and rendering. CLI integration tests run
the compiled binary against temporary captures. CI applies formatting,
Clippy-with-warnings-denied, and all tests on Windows and Linux.
