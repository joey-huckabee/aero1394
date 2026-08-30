# Aero1394 v0.2.0 release notes (draft)

- Status: Unreleased
- Release series: Initial development (`0.x`)
- Target tag: `v0.2.0`

This file is packaged by normal CI for smoke testing. `CHANGELOG.md` is the
authoritative record of work accumulated before release.

## Included so far

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

## Still planned

- confirmed engineering interpretation of `msfcs_storesmassdata_b` after the
  remaining unit, coordinate/reference, acronym, group-meaning, and epoch
  semantics are supplied.

## Evidence boundary

The current decoder applies explicit reconstruction assumptions documented in
`docs/AS5643.md`. It does not decode IEEE-1394 wire framing, assign Health
Status bit meanings, convert Heartbeat deltas into lost-message counts, judge
STOF schedule compliance, or assign engineering meaning to decoded application
primitives.

This draft is not a compatibility or publication promise. It will be finalized
from `CHANGELOG.md` only after every `v0.2.0` release gate passes.
