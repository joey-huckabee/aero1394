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
- A `layout-v1` registry entry for `msfcs_storesmassdata_b` plus CLI recognition
  metadata that remains distinct from application-field decoding.

## Still planned

- complete `msfcs_storesmassdata_b` decoding after source metadata and the
  remaining Boolean and engineering-unit semantics are confirmed.

## Evidence boundary

The current decoder applies explicit reconstruction assumptions documented in
`docs/AS5643.md`. It does not decode IEEE-1394 wire framing, assign Health
Status bit meanings, convert Heartbeat deltas into lost-message counts, judge
STOF schedule compliance, or interpret application bytes.

This draft is not a compatibility or publication promise. It will be finalized
from `CHANGELOG.md` only after every `v0.2.0` release gate passes.
