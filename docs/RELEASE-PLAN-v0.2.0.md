# Aero1394 v0.2.0 release plan

- Status: Complete
- Target: `v0.2.0`
- Theme: Evidence-backed AS5643 envelope and first built-in payload
- Last updated: 2026-08-30

## Release statement

`v0.2.0` will turn the confirmed BIE stored-data mapping for data item
`0x00005D04` into a reusable AS5643 protocol-envelope API. It will preserve the
complete raw bytes, expose Health Status, Heartbeat, application data, three
STOF offsets, and VPC, and identify every result as assumption-dependent under
profile `aero1394-assumed-as5643b-v1`.

The release also targets the first Rust-native application payload definition
for `msfcs_storesmassdata_b`. Payload work begins only from an authorized,
complete field definition and remains downstream of the protocol envelope.

## Dependency order

```text
BIE record
    -> explicit BIE-to-AS5643 profile mapping
        -> raw AS5643 envelope
            -> VPC validation
                -> payload registry
                    -> msfcs_storesmassdata_b decoder
```

The supplied BIE data does not retain an identifiable IEEE-1394 wire packet,
so IEEE-1394 header or CRC decoding is not a prerequisite and is not included
in this release.

## Incremental delivery

| Increment | Functional result | Exit evidence | Status |
| --- | --- | --- | --- |
| 1. Raw envelope | Decode the verified 116-byte stored representation into raw AS5643 fields and a borrowed 92-byte application region. | Golden fixture values, exact-length errors, explicit profile ID, preserved bytes, and L3 trace markers. | Complete |
| 2. Integrity | Calculate VPC from explicit reconstructed header inputs and return valid, invalid, absent, or unchecked outcomes. | Known-good and mutated fixtures with stored/calculated values retained. | Complete |
| 3. BIE integration | Map only the supported BIE identity/size/profile combination and expose decoded envelope values without changing generic BIE parsing. | Unknown ID and wrong-size fallbacks plus CLI integration tests. | Complete |
| 4. Payload registry | Select payload definitions deterministically and distinguish one match, no match, and ambiguity. | Registry tests covering all three outcomes and raw unknown preservation. | Complete |
| 5a. First payload raw fields | Decode all 92 bytes of `msfcs_storesmassdata_b` with supplied names, primitive types, exact ranges, and preserved raw values. | Sanitized populated/sparse golden values, exact-length and byte-order tests, definition validation, and updated traceability. | Complete |
| 5b. Provisional payload semantics | Add strict Boolean interpretation, non-fatal findings, message validity, reserved-byte policy, direct IEEE-754 values, and nominal elapsed seconds without replacing raw values. | Warning-bearing tests, populated/sparse fixtures, explicit uncertainty labels, and CLI exit-code checks. | Complete |
| 5c. Confirmed engineering semantics | Add confirmed units, coordinate/reference conventions, acronym/group meanings, and timestamp epoch without replacing raw values. | Authorized source metadata and independent expected engineering values. | Deferred beyond `v0.2.0`; evidence inputs pending |
| 6. Release hardening | Package and inspect `v0.2.0` on Windows and Linux. | Exact CI gates, release notes, checksums, packaged-binary smoke tests, tag-run inspection, and changelog finalization. | Complete |

Each increment must be independently functional, update `CHANGELOG.md`, and
receive its own reviewable commit.

## Payload evidence needed

The field layout for `msfcs_storesmassdata_b` was supplied on 2026-08-30 and
confirms its 92-byte size, field names, primitive types, signedness, word IDs,
and byte/bit offsets. The source is an internal ICD, but no further ICD name or
revision is currently available. Provisional decisions establish strict
Boolean encoding, flag polarity, warning behavior, unscaled IEEE-754 values,
and nominal 13.6 GHz elapsed-time conversion. The remaining inputs are:

- ICD name/revision metadata, if it later becomes available;
- team confirmation of `MessageValid` polarity;
- units, coordinate/reference conventions, acronym expansions, and group/field
  meanings where still unspecified;
- confirmation of the system-startup `TimeStamp` epoch hypothesis; and
- at least one independently expected decoded message or field listing that can
  be represented by sanitized golden test values.

Restricted source documents do not need to be committed. Their permitted
facts can be transcribed into the field table and sanitized fixtures with
provenance and handling constraints recorded.

## Explicitly excluded

- IEEE-1394 wire headers, link/PHY event families, and IEEE CRC validation;
- automatic profile detection from byte patterns;
- guessed Health Status bit names or Heartbeat sequence-gap semantics;
- STOF schedule-compliance claims without a verified frame-time anchor;
- confirmed Stores Mass engineering interpretation without the required source
  evidence;
- runtime YAML payload definitions;
- stable CSV, Parquet, or Python APIs; and
- crates.io publication.

## Release gates

- Every public parser is range-safe, uses explicit big-endian decoding, and
  preserves the raw input required to audit its result.
- Assumption-derived fields carry the selected profile identifier and an
  assumption-dependent marker.
- Generic BIE parsing remains independent of AS5643 and payload modules.
- All implemented L3 requirements have test or named non-test evidence and the
  generated trace matrix is current.
- `CHANGELOG.md` describes every merged increment and is finalized under a
  dated `0.2.0` heading before tagging.
- Formatting, strict Clippy, all tests, the trace check, locked release builds,
  packaging, and packaged-binary smoke tests pass on Windows and Linux.

Confirmed engineering semantics are a post-`v0.2.0` evidence-gated increment,
not a release gate. The release retains the raw primitives and labels every
provisional interpretation so later evidence can refine semantics without
changing the preserved input.

## Current hardening evidence

- [x] The package version and pinned Rust toolchain are `0.2.0` and `1.98.0`.
- [x] The local Windows trace check, formatting check, strict Clippy run, and
  complete Rust test suite pass.
- [x] A locked local Windows release build passes. Repeated ZIP and `tar.gz`
  construction produced byte-identical SHA-256 digests, and both extracted
  binaries passed the expanded smoke checks.
- [x] Release packaging checks the binary version, every CLI help surface,
  exact archive membership, extracted binary behavior, and SHA-256 output.
- [x] Tag builds reject a version mismatch, draft release notes, or a changelog
  without a valid dated `0.2.0` heading.
- [x] Final-scope release notes and deterministic ZIP/`tar.gz` tooling are
  present for candidate packaging.
- [x] Locked Windows and Linux release builds and packaged-binary smoke tests
  passed in push workflow `33334541950` and retained-candidate workflow
  `33334581096`.
- [x] Both retained candidates from workflow `33334581096` were downloaded;
  their checksums matched, each archive contained exactly the expected four
  members, and the Windows binary repeated every version/help smoke check.
- [x] Clean commit `ee37b29` passed workflow `33334793475`, was tagged
  `v0.2.0`, and passed tag workflow `33334855411`. Both tag-built checksums and
  archive member sets were verified, the Windows binary repeated every smoke
  check, and the four tag-built assets were published without modification.
- [x] The tag-built Linux archive matched the inspected candidate
  byte-for-byte. The tag-built Windows ZIP and its checksum are the
  authoritative published Windows artifacts.
