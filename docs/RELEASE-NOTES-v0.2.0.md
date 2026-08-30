# Aero1394 v0.2.0 release notes

- Release series: Initial development (`0.x`)
- Target tag: `v0.2.0`
- Supported artifact platforms: Windows x86-64 and Linux x86-64

These notes define the frozen `v0.2.0` candidate scope. `CHANGELOG.md` remains
the authoritative record of work accumulated before release.

## Highlights

- A BIE-independent raw decoder for the retained 116-byte representation of
  provisional profile `aero1394-assumed-as5643b-v1`.
- A distinct AS5643 Message ID type, explicit profile identity, and an
  assumption-dependent result marker.
- Raw reconstructed header words, Health Status, Heartbeat, a borrowed 92-byte
  application region, three STOF offsets, stored VPC, and complete retained
  input bytes.
- Explicit failures for a Message ID outside the selected profile and retained
  representations that are shorter or longer than the confirmed profile size.
- Golden verification against every record in the final sanitized BIE fixture.
- VPC calculation from explicit reconstructed header inputs, with structured
  valid, invalid, absent, and unchecked results that retain audit evidence.
- Golden VPC validation across all eight supplied records plus corrupted and
  unavailable-input cases.
- An explicit BIE-to-AS5643 adapter that requires the supported data-item ID
  and stored size while preserving unknown and wrong-size records.
- An `as5643` CLI command that presents raw envelope fields, reconstructed
  inputs, profile assumptions, stored/calculated VPC, and validation outcomes.
- A documented 92-byte `msfcs_storesmassdata_b` field map correlated with the
  captured fixture values.
- A deterministic built-in payload registry with exact identity/size matching,
  optional context constraints, and explicit matched, unknown, and ambiguous
  outcomes that preserve raw application bytes.
- A `layout-v1` registry entry for `msfcs_storesmassdata_b` plus selection
  metadata kept distinct from its downstream raw field decode.
- Checked metadata for all 25 supplied Stores Mass fields, including explicit
  offsets, primitive wire types, complete coverage, and invalid-layout checks.
- Exact big-endian raw decoding for the unsigned system ticks, four
  Boolean-designated bytes, and twenty IEEE-754 `f32` fields, with the original
  bytes and float bit patterns retained.
- Populated and sparse payload-only golden fixtures plus raw field presentation
  in the `as5643` CLI.
- Additive provisional payload semantics: strict `0`/`1` Boolean decoding,
  message-valid state, informational presence flags, reserved-byte checks,
  direct unscaled IEEE-754 values, and system ticks expressed as provisional
  elapsed seconds at the documented nominal 13.6 GHz rate.
- Non-fatal payload findings that retain and display every decoded field, with
  CLI exit codes `0` for clean success, `1` for errors, and `2` for successful
  decoding with warnings.

## Deliberate exclusions

- confirmed units, coordinate/reference conventions, acronym expansions,
  group meanings, and timestamp epoch for `msfcs_storesmassdata_b`;
- IEEE-1394 wire headers, link/PHY event families, and IEEE CRC validation;
- automatic profile detection from byte patterns;
- Health Status bit names, Heartbeat loss claims, or STOF schedule-compliance
  claims without supporting evidence;
- stable CSV, Parquet, or Python APIs; and
- crates.io publication.

## Evidence boundary

The decoder applies explicit reconstruction assumptions documented in
`docs/AS5643.md`. Raw Stores Mass primitives remain available alongside strict
provisional Boolean, validity, warning, and elapsed-time interpretations. The
release does not decode IEEE-1394 wire framing, assign Health Status bit
meanings, convert Heartbeat deltas into lost-message counts, judge STOF
schedule compliance, or claim confirmed engineering meaning for application
primitives.

## Usage

Decode the supported AS5643 envelope and built-in payload from a complete BIE
file:

```text
aero1394 as5643 capture.bie
```

List raw BIE records without protocol interpretation:

```text
aero1394 records capture.bie
```

Use `aero1394 <COMMAND> --help` for command details.

## Compatibility

This is a pre-`1.0.0` initial-development release. Rust APIs and human-oriented
CLI formatting may evolve between minor versions. Future changes must remain
evidence-backed and must not silently reinterpret preserved raw values.
