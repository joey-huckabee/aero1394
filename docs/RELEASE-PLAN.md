# Aero1394 v0.1.0 release plan

- Status: Active
- Target: `v0.1.0`
- Stability: Initial development release; public APIs may evolve before `1.0.0`
- Last updated: 2026-08-29

## Release statement

`v0.1.0` will be the first useful, evidence-backed Aero1394 release. It will
inspect files safely and parse the confirmed BIE container framing without
claiming IEEE-1394, AS5643, or application-payload semantics that have not yet
been implemented.

This is a pre-`1.0.0` release in the SemVer initial-development series. The
version is `0.1.0`, not a SemVer prerelease suffix such as `0.1.0-alpha.1`.

## Included capability

- The Rust library parses individual BIE records, strict complete BIE byte
  slices, and bounded record streams using explicit big-endian fields and
  checked offset arithmetic.
- Parsed records retain their absolute file offset, raw status/length word,
  unresolved flags, raw recorder time, unknown data-item IDs, and exact stored
  data.
- Strict file parsing recognizes the four-byte zero sentinel and distinguishes
  record truncation, a missing terminator, trailing data, and offset overflow.
- The CLI retains bounded `hexdump` behavior and adds a human-readable BIE
  record inventory command backed by the same library parser.
- Sanitized FireSpy-derived fixtures and synthetic boundary cases verify the
  supported BIE contract.
- Windows and Linux CI run formatting, lint, tests, and trace-matrix freshness
  checks with the pinned Rust toolchain.

## Explicitly excluded

- IEEE-1394 packet decoding and CRC validation;
- AS5643 field decoding, STOF interpretation, VPC APIs, or heartbeat analysis;
- application-payload and engineering-unit decoding;
- recovery or resynchronization after malformed BIE data;
- automatic format detection;
- stable machine-readable output schemas;
- Python bindings and package publication; and
- crates.io publication.

The AS5643 and payload documents remain evidence and future implementation
contracts, not claims about `v0.1.0` runtime behavior.

## Incremental delivery

| Increment | Functional result | Exit evidence | Status |
| --- | --- | --- | --- |
| 1. Record framing | Parse one complete non-terminator record without copying its stored data. | Unit boundaries and known-good startup fixture; commit `5b8f0c9`. | Complete |
| 2. File framing | Parse a strict complete BIE slice through its sentinel. | Multi-record, empty, end-fixture, truncation, missing-terminator, trailing-data, and overflow tests. | Complete |
| 3. Record inventory CLI | List record number, offset, ID, recorder time, raw status, flags, and body length. | CLI success/error integration tests using sanitized fixtures. | Complete |
| 4. Release hardening | Make large-input behavior and distributable binaries reproducible. | Bounded streaming, exact CI checks, release builds on Windows and Linux, CLI help review, release notes, and packaged-artifact smoke tests. | In progress |

Each increment must remain independently functional and receive its own
reviewable commit. Protocol interpretation will begin only after this release
gate or a separately documented scope decision.

Current hardening evidence:

- [x] Locked Windows release build succeeds with Rust 1.98.0.
- [x] The Windows release binary passes `--version` and `records --help` smoke
  checks.
- [x] Record inventory uses two bounded passes and retains at most one encoded
  BIE record plus fixed-size I/O buffers.
- [x] Versioned release notes and deterministic ZIP/`tar.gz` plus SHA-256
  tooling are committed.
- [x] Repeated local Windows packaging produced the same SHA-256 digest, and
  both ZIP and `tar.gz` extraction/smoke paths passed.
- [ ] The locked Linux release build and packaged-binary smoke test pass in CI.
- [ ] Retained Windows and Linux workflow artifacts are manually inspected.
- [ ] A clean CI-passing commit is explicitly tagged and published.

## Artifact construction

The cross-platform packaging command is:

```text
python scripts/package-release.py --platform <LABEL> --archive-format <zip|tar.gz> --binary <PATH>
```

It verifies the binary version and `records --help`, creates a normalized
archive containing the binary, `LICENSE`, `README.md`, and versioned release
notes, extracts and smoke-tests the packaged binary, and writes a sibling
`.sha256` file. When running for a Git tag, it also requires the tag to equal
`v` plus the Cargo package version.

The existing CI matrix executes packaging on every Windows and Linux run.
Artifacts are retained only for a manual workflow run or a `v*` tag. The
workflow deliberately does not create a tag or publish a GitHub release; those
remain explicit maintainer actions after both retained candidates are
inspected.

## Release gates

### Functional

- A caller can inspect unknown bytes with bounded hex output.
- A caller can parse a complete supported BIE file and enumerate every record.
- Sentinel-only input is accepted as the documented structural empty form.
- Unsupported IDs and unresolved flags remain inspectable rather than being
  rejected or assigned speculative meanings.
- The CLI reports malformed input without panicking and returns a nonzero exit
  status.

### Verification

- `cargo fmt --all -- --check` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- `cargo test --all-targets --all-features` passes on Windows and Linux.
- `python scripts/build-trace-matrix.py --check` passes.
- Every implemented BIE behavior has a requirement marker and the generated
  trace matrix is current.

### Documentation and packaging

- `README.md`, `ROADMAP.md`, and `docs/ARCHITECTURE.md` describe the shipped
  boundary and do not imply protocol decoding.
- CLI usage and errors are documented using only sanitized inputs.
- `Cargo.toml` remains at `0.1.0`, `Cargo.lock` is committed, and
  `cargo build --release --locked` succeeds on Windows and Linux.
- The `v0.1.0` tag is created only from a clean commit that passed CI.
- Release artifacts include Windows and Linux CLI binaries, license and readme
  files, release notes, and SHA-256 checksums.

## Compatibility promise

Before `1.0.0`, Rust APIs and CLI presentation may change between minor
versions. Changes must still be documented, evidence-backed, and must not
silently reinterpret raw BIE values. The confirmed BIE bytes, raw fields, and
fixture provenance remain the compatibility anchor.
