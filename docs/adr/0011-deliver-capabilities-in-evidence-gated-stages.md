# ADR-0011: Deliver capabilities in evidence-gated stages

- Status: Accepted
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

The desired long-term tool spans recorder forensics, BIE parsing, IEEE-1394,
AS5643, profiles, engineering values, and network analysis. Only the product
goals and likely recorder provenance are currently known. Implementing the full
stack against guessed bytes would create polished APIs and output around an
unverified foundation.

At the same time, the tool is intended for immediate practical use. Each stage
therefore needs to produce a useful vertical slice and the evidence required to
start the next stage.

## Decision

Deliver Aero1394 in the following evidence-gated stages.

### Stage 0: Evidence acquisition and repository baseline

- Obtain a representative BIE capture and matching vendor export if possible.
- Record source, versions, hashes, sensitivity, and redistribution constraints.
- Establish the ADR baseline and a clean project README.

Exit evidence: at least one usable sample and enough provenance to investigate
it safely.

### Stage 1: Rust forensic inspection

- Scaffold the library and CLI without asserting a BIE record schema.
- Provide file statistics and bounded, offset-oriented hex inspection.
- Discover repeated byte patterns and candidate boundaries.
- Report candidate lengths, byte orders, timestamps, and embedded packet
  structures with their evidence.
- Keep all output explicitly observational or hypothetical.

Likely commands include `inspect`, `hexdump`, `scan`, and `records`; exact names
can change after use.

Exit evidence: reproducible observations identify one or more candidate record
layouts across many records without out-of-bounds or non-progress behavior.

### Stage 2: Verified BIE framing

- Parse confirmed file and record structures.
- Preserve unknown fields, raw bytes, offsets, and version information.
- Implement truncation, alignment, length, padding, and recovery behavior.
- Publish the verified portion of `docs/BIE-FORMAT.md` with fixtures and tests.
- Expose a first useful streaming or batched API for ETL when semantics are
  stable enough to name.

Exit evidence: BIE record counts and boundaries correlate with an independent
export or other authoritative source across representative captures.

### Stage 3: IEEE-1394 decoding

- Classify supported captured packet or transaction forms.
- Decode verified header fields and lengths with explicit byte/bit numbering.
- Implement applicable header/data CRC checks.
- Inventory channels, packet types, speeds, timestamps, and integrity findings.

Exit evidence: decoded fields and CRC coverage agree with authoritative
IEEE-1394 material and known or independently decoded traffic.

### Stage 4: AS5643 decoding

- Detect AS5643 only from supported IEEE-1394 packet forms.
- Decode confirmed message identity, node, priority, payload/status, heartbeat,
  timing, parity, and other applicable fields.
- Retain non-AS5643 and malformed traffic as inspectable IEEE-1394 data.

Exit evidence: structures match the selected AS5643 revision and, where
possible, a known network or vendor export.

### Stage 5: Profiles and engineering signals

- Define a versioned, validated network-profile schema.
- Map messages and payload fields to engineering values without changing the
  generic protocol decoder.
- Represent units, scaling, validity, enumerations, and missing profile data.

Exit evidence: decoded values agree with authorized ICD material and known
source values.

### Stage 6: Network analysis

- Add STOF-relative timing, expected-message schedules, heartbeat and sequence
  checks, missing-message detection, node health, utilization, and anomaly
  reporting.
- Distinguish measured values, configured expectations, and derived findings.

Exit evidence: analysis is verified against controlled or independently known
timing and fault cases.

The CLI and Python adapter evolve alongside the earliest stage that has stable
semantics to expose; they are not separate decoder phases. A stage may deliver
partial protocol support when its supported subset and unsupported cases are
explicitly documented.

Do not start a later layer merely because a byte pattern looks familiar. Its
entry evidence must be sufficient to keep container guesses from becoming
protocol facts.

## Alternatives considered

### Design and implement the complete long-term architecture first

Rejected because most public types would be based on unverified format
assumptions and would be expensive to change after capture evidence arrives.

### Stop after binary-to-CSV conversion

Rejected because generic validation, timing, health, and reusable protocol
components are core project goals. CSV is an output option, not the system
boundary.

### Wait for every protocol layer before releasing useful commands

Rejected because forensic inspection, BIE framing, packet inventory, and
validation each provide independent operational value.

## Consequences

### Positive

- Each milestone is useful and supported by explicit evidence.
- Incorrect early assumptions have a limited blast radius.
- Documentation, tests, CLI, and Python behavior advance with the core.
- Project status and next work remain legible.

### Negative

- High-level AS5643 and engineering output arrives only after lower layers are
  defensible.
- Some commands and exploratory terminology may change in early stages.
- Exit evidence can reveal that a stage needs more samples than expected.

## Immediate next step

Acquire and characterize one representative BIE capture plus a matching vendor
export if available. Once that input exists, scaffold the Stage 1 Rust vertical
slice rather than guessing a record API in advance.
