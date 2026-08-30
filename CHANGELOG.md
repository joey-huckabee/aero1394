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

### Changed

- Advanced project planning from the completed `v0.1.0` BIE-framing release to
  `v0.2.0` protocol-envelope development.

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
