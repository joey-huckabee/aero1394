# Changelog

All notable changes to Aero1394 are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Every change to behavior, public contracts, documentation, tooling, or release
process must update the `Unreleased` section in the same commit.

## [Unreleased]

### Added

- Established this changelog as a required part of every development increment.
- Added the evidence-gated `v0.2.0` plan for AS5643 envelope decoding, integrity
  validation, BIE integration, and the first built-in payload definition.
- Added a BIE-independent decoder for the retained 116-byte
  `aero1394-assumed-as5643b-v1` representation. It exposes the distinct AS5643
  Message ID, reconstructed raw header words, Health Status, Heartbeat, a
  borrowed 92-byte application region, three STOF offsets, stored VPC, profile
  identity, assumption state, and complete retained bytes.
- Added exact-size and profile-selection errors plus golden tests over all four
  final BIE fixture records.
- Added AS5643 VPC calculation from explicit reconstructed header inputs and
  structured `Valid`, `Invalid`, `NotPresent`, and `NotChecked` results that
  retain stored/calculated values and available calculation evidence.
- Added golden VPC checks for all eight supplied records, a protected-data
  mutation test, and unavailable/unaligned-input tests.
- Added a separate BIE-to-AS5643 adapter that selects the provisional profile
  only for data item `0x00005D04` with exactly 116 stored bytes and preserves
  complete records for unsupported identities and layouts.
- Added the `as5643` CLI command for assumption-labeled envelope and VPC
  presentation, including non-failing unsupported-ID and wrong-size results.
- Added a separate built-in payload registry that selects definitions by exact
  data-item ID and payload size, supports optional data-code and configuration
  constraints, and explicitly preserves matched, unknown, and ambiguous raw
  payloads.
- Registered the 92-byte `msfcs_storesmassdata_b` identity as Aero1394 layout
  version `layout-v1` and added payload-recognition metadata to `as5643` CLI
  output without claiming that application fields have been decoded.
- Added checked payload field metadata with explicit wire types and byte ranges,
  including validation that rejects overlaps and out-of-bounds declarations and
  reports uncovered gaps.
- Added exact-length, big-endian raw decoding for every supplied
  `msfcs_storesmassdata_b` field. System ticks, Boolean-designated bytes, exact
  IEEE-754 `f32` bits, unscaled float values, and all 92 input bytes remain
  accessible without inferred engineering semantics.
- Added populated and sparse payload-only golden fixtures, typed registry
  dispatch, and raw Stores Mass field presentation in the `as5643` CLI.
- Added provisional `msfcs_storesmassdata_b` semantics without replacing raw
  values: strict `0`/`1` Booleans, message-valid state, informational presence
  flags, reserved-byte checks, direct unscaled IEEE-754 values, and elapsed
  seconds derived from the documented nominal 13.6 GHz system-tick rate.
- Added deterministic non-fatal payload warnings for invalid messages,
  unexpected Boolean encodings, nonzero reserved bytes, and NaN/infinite float
  values; warning-bearing payloads continue to expose every decoded field.
- Added final-scope `v0.2.0` release notes for Windows and Linux candidate
  packages.
- Added tag-only release metadata gates that reject mismatched versions, draft
  release notes, invalid release dates, and undated changelog versions.
- Expanded release-package inspection to verify exact archive membership and
  safely extract artifacts and smoke-test every CLI help surface, including the
  `as5643` command.

### Changed

- Replaced the provisional `msfcs_storesmassdata_b` payload hypothesis with the
  complete user-supplied 92-byte field table, confirming an unsigned timestamp,
  four Boolean elements, and twenty named floating-point fields while retaining
  unresolved units, Boolean encoding, and source metadata as explicit inputs.
- Corrected the payload-fixture description: populated records contain the four
  Boolean-designated bytes `01 00 00 00`, while sparse startup records contain
  `00 01 00 00`; the subsequent provisional semantic contract interprets those
  bytes without changing their retained raw values.
- Established CLI exit codes `0` for clean success, `1` for usage or operational
  errors, and `2` for successful decoding with one or more payload warnings.
- Recorded the internal-ICD source limitation, the system-configuration scope
  of ID `0x00005D04`, every provisional payload decision, and all meanings,
  units, acronym expansions, and timestamp-epoch facts that remain uncertain.
- Advanced project planning from the completed `v0.1.0` BIE-framing release to
  `v0.2.0` protocol-envelope development.
- Advanced Cargo package metadata to `0.2.0` and updated the architecture and
  AS5643 implementation status for the raw-envelope increment.
- Froze the `v0.2.0` functional scope, advanced its release-hardening increment,
  and deferred confirmed Stores Mass engineering semantics until the required
  evidence is available.

## [0.1.0] - 2026-08-29

### Added

- Added bounded, offset-aware hexadecimal inspection through the Rust library
  and CLI.
- Added evidence-backed parsing for individual BIE records, strict complete BIE
  files, the four-byte terminator, and bounded record streaming.
- Added the `records` CLI command for raw BIE record inventories without
  speculative protocol interpretation.
- Added sanitized FireSpy-derived fixtures, requirements traceability, and the
  BIE, AS5643, timing, and payload evidence contracts.
- Added deterministic ZIP and `tar.gz` construction, SHA-256 files,
  packaged-binary smoke tests, and Windows/Linux CI release candidates.

### Fixed

- Normalized trace-matrix newline comparison so the exact check works on both
  Windows and Linux checkouts.

[Unreleased]: https://github.com/joey-huckabee/aero1394/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/joey-huckabee/aero1394/releases/tag/v0.1.0
