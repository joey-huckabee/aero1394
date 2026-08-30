# Aero1394 architecture

- Status: BIE framing, AS5643 envelope/VPC, explicit BIE mapping, and CLI implemented
- Last updated: 2026-08-30

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

Library: &[u8] + FileOffset -> bie::parse_file -> BieFile { borrowed records }

CLI: records path -> validate BieReader pass -> render BieReader pass

Library: MessageId + retained &[u8] -> as5643 profile -> raw envelope
                                                     -> VpcValidation

Library: BieRecord -> bie_as5643 mapping -> mapped envelope or unsupported

CLI: as5643 path -> validate BieReader pass -> map and render BieReader pass
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

The `records` adapter makes two bounded passes over its seekable input. The
first pass validates the complete framing without writing inventory lines, so
ordinary malformed inputs do not produce a partial listing. The adapter then
rewinds the same file and renders records during the second pass. `BieReader`
retains at most one encoded record (65,551 bytes) and uses a fixed 8 KiB scratch
buffer while checking for trailing data; capture size does not determine its
working-memory allocation.

The `bie` module parses either one non-terminator record or a strict complete
file from a byte slice. It derives body boundaries from encoded low-16-bit
lengths, preserves unknown IDs and unresolved flags, chains variable-length
records, and requires the terminal zero word to be the final four bytes. The
file result owns only its record-view collection; every stored-data region
continues to borrow the caller's input. Framing reports truncation, missing
termination, trailing data, and offset overflow without performing validation,
protocol interpretation, or recovery. `BieReader` composes the same record
parser with `Read` while bounding memory independently of source size.

The `bie_as5643` adapter depends on both the generic BIE and AS5643 modules. It
maps only data item `0x00005D04` with exactly 116 stored bytes to profile
`aero1394-assumed-as5643b-v1`. Every result retains the original `BieRecord`;
unknown identities and other stored-data sizes are explicit unsupported
outcomes rather than parser failures. The `as5643` CLI presents this adapter's
result using the same validate-before-render two-pass behavior as `records`.

Release packaging is a repository adapter implemented in
`scripts/package-release.py`. It verifies the compiled binary, creates a
timestamp-normalized ZIP or `tar.gz` containing the binary, license, readme,
and release notes, extracts that archive, smoke-tests the packaged binary, and
writes a sibling SHA-256 file. Normal CI runs both platform packaging paths;
manual runs and version tags retain artifacts without publishing a release.

## Dependency direction

The binary depends on the library. The library never depends on the binary,
terminal formatting, filesystem paths, or operating-system-specific behavior.
The package currently has no third-party runtime dependencies and forbids
unsafe code.

The BIE record parser operates on byte slices, following
[ADR-0006](adr/0006-use-safe-slice-oriented-parsers-and-explicit-wire-types.md),
and later protocol parsers will follow the same boundary. The streaming
forensic reader can supply those slices but will not absorb their parsing,
validation, policy, or recovery responsibilities.

## Protocol and format hierarchy

ADR-0002, ADR-0009, and ADR-0011 establish separate ownership for the
IEEE-1394, AS5643, capture-container, and application-payload knowledge. For
the three related format documents, knowledge narrows in this order:

```text
IEEE1394.md
    IEEE-1394 wire representations and packet behavior
        |
        v
AS5643.md
    AS5643 protocol interpretation, independent of capture format
        |
        v
BIE-FORMAT.md
    BIE container grammar and observations specific to BIE storage
```

These arrows express documentation specificity, not standard ownership or
runtime decode order. IEEE-1394 documentation must not acquire AS5643 or BIE
layout rules. AS5643 documentation may specialize supported IEEE-1394 forms
but must not acquire BIE layout rules. BIE documentation owns the outer file
grammar and any evidence about fields that this particular container retains,
removes, or normalizes. Program-specific application fields remain in
[`PAYLOADS.md`](PAYLOADS.md), outside all three generic format contracts.

This ownership follows the accepted boundaries in
[ADR-0002](adr/0002-separate-capture-protocol-profile-and-analysis-layers.md),
the document responsibilities in
[ADR-0009](adr/0009-treat-documentation-and-test-evidence-as-deliverables.md),
and the evidence gates in
[ADR-0011](adr/0011-deliver-capabilities-in-evidence-gated-stages.md).

## Expected processing flow

Modules are added only with evidence-backed behavior. The expected processing
flow is:

```text
BIE or future input adapter
    -> captured-event boundary
        -> ieee1394
            -> as5643
                -> payload
                    -> analysis
```

The input adapter may be unable to produce the complete IEEE-1394
representation required by the next layer. It always returns preserved raw
bytes and provenance. A caller may additionally select a named provisional
profile that reconstructs missing protocol inputs from explicit assumptions;
those results carry the profile identifier and an assumption-dependent marker
rather than being presented as verified wire facts. Protocol and payload
modules never depend on BIE types.

The `payload` module is the Rust-native implementation of the profile knowledge
described by ADR-0002. Definitions are compiled into the tool and selected by
an explicit registry as decided in ADR-0012; external YAML profiles are not a
runtime requirement.

An outer container may produce raw captured bytes without successfully
decoding the next layer. Each layer must preserve the bytes and source offsets
needed to inspect unsupported or invalid input. Chapter 10 remains a future
input adapter rather than a dependency of BIE decoding.

A module should become a separate crate only when the extraction conditions in
ADR-0004 are met. Current public types are deliberately small so unresolved BIE
semantics, including status flag `0x40000000`, do not become accidental API
guarantees.

## Evidence available for the next slice

The generic BIE parser and bounded reader preserve the confirmed container
fields and exact stored regions. The BIE-independent `as5643` module now
decodes the retained 116-byte representation for the explicitly selected
`aero1394-assumed-as5643b-v1` profile. It exposes raw/reconstructed envelope
fields and a borrowed 92-byte application region while retaining the complete
input and assumption marker. It calculates the VPC from explicit reconstructed
header inputs, preserves the stored and calculated words, and distinguishes
valid, invalid, absent, and unchecked outcomes.

The separate `bie_as5643` adapter now maps only the supported BIE identity and
retained size to the named profile, and the CLI presents mapped and unsupported
outcomes. The next slice is the deterministic typed payload registry;
application-field semantics remain downstream. The definitive contracts are
in `BIE-FORMAT.md`, `AS5643.md`, and `PAYLOADS.md`; live verification status is
in `TRACE-MATRIX.md`.

## Verification

Unit and integration tests cover forensic bounds, BIE framing/streaming,
AS5643 profile selection and retained-size errors, golden raw envelope values,
known-good and corrupted VPC outcomes, argument parsing, and rendering. CLI
integration tests run the compiled binary against temporary mapped, unknown-ID,
and wrong-size captures. CI applies formatting, Clippy-with-warnings-denied,
and all tests on Windows and Linux.
