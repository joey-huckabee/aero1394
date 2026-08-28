# ADR-0009: Treat documentation and test evidence as deliverables

- Status: Accepted
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

The project is decoding a currently undocumented or unavailable recorder
format, then applying layered protocol interpretation and validation. A parser
can appear to work while using an incorrect length, byte order, timestamp unit,
CRC boundary, or padding rule. Source code alone will not communicate which
claims are verified, which standard revision applies, or how corner cases are
handled.

The tool will be used in an ETL pipeline and its Rust components may be reused
inside Chapter 10 tooling. Those consumers need stable behavior, provenance,
and reproducible fixtures rather than conclusions preserved only in a planning
conversation.

## Decision

Documentation and verification artifacts are part of each delivered vertical
slice, not deferred cleanup.

The documentation set will grow as evidence becomes available:

| Document | Purpose |
| --- | --- |
| `docs/BIE-FORMAT.md` | Evidence ledger and normative description of verified BIE framing |
| `docs/PAYLOADS.md` | Built-in application-payload definitions, provenance, and open fields |
| `docs/L1.md`, `docs/L2.md`, `docs/L3.md` | Product, architecture, and implementation requirements with explicit parent links |
| `docs/TRACE-MATRIX.md` | Generated traceability from requirements to verification artifacts and status |
| `docs/OUTPUTS.md` | Versioned CSV, Parquet, time-format, and adapter schema contracts |
| `docs/ARCHITECTURE.md` | Layer boundaries, data ownership, dependency direction, and data-flow pipelines |
| `docs/REVERSE-ENGINEERING.md` | Reproducible forensic methods, tools, capture provenance, and open questions |
| `docs/IEEE1394.md` | Implemented packet subset, byte/bit numbering, CRC coverage, and unsupported cases |
| `docs/AS5643.md` | Implemented message subset, timing/status interpretation, and profile boundary |
| user/CLI documentation | Commands, policies, output fields, errors, and examples |
| Python API documentation | Objects, streaming behavior, schemas, exceptions, and compatibility |

These files should be created when their first verified content exists; empty
placeholder documents are not required.

Data-flow documentation must cover normal and exceptional paths, including:

- file I/O to container record;
- container metadata and captured bytes to IEEE-1394 packet;
- packet to optional AS5643 message;
- message plus optional profile to engineering values;
- validation findings and recovery events alongside decoded data;
- strict, permissive, and forensic policy behavior if ADR-0007 is accepted; and
- conversion into CLI, machine-readable, and Python representations.

Each format rule should be paired with evidence and tests proportional to its
risk. The test strategy will include:

- small unit tests for byte order, bit extraction, checked arithmetic, lengths,
  padding, timestamps, and CRCs;
- synthetic fixtures for valid and invalid boundary cases;
- sanitized excerpts from real captures when redistribution is authorized;
- golden expected results for end-to-end records and output schemas;
- mutation, property, or fuzz testing for parser safety and forward progress;
  and
- Windows and Linux integration tests for filesystem and CLI behavior.

Fixtures must carry provenance and expected-use notes. Sensitive capture data,
proprietary ICD content, and standards text must not be copied into the
repository without permission. Protocol documentation should explain the
implementation and cite the exact authoritative revision rather than reproduce
copyrighted specifications.

Cross-language conformance suites are not required because Rust is the single
implementation. The Python binding must nevertheless run shared semantic cases
to prove it exposes Rust results correctly.

## Required corner-case record

Every implemented layer must maintain a reviewable list of:

- minimum and maximum lengths;
- alignment and padding rules;
- unknown versions, types, flags, and reserved bits;
- absent, invalid, and unchecked integrity fields;
- truncated input at each boundary;
- timestamp epochs, units, wraparound, discontinuities, and ordering;
- arithmetic overflow and resource limits;
- corruption recovery and skipped ranges; and
- information preserved when the next layer cannot decode.

The list may live with the relevant format document and tests rather than in
one global checklist.

## Alternatives considered

### Document behavior after the decoder is complete

Rejected because the format evidence drives the implementation. Delayed
documentation would make it difficult to distinguish observed behavior from
assumptions retrofitted to the code.

### Rely only on real captures

Rejected because real data rarely exercises truncation, overflow, every CRC
state, and all alignment boundaries predictably. Synthetic and generated cases
are required.

### Rely only on synthetic fixtures

Rejected because synthetic data can merely confirm the assumptions used to
generate it. At least one independently produced real capture is needed to
validate the format interpretation.

## Consequences

### Positive

- Format knowledge remains auditable and transferable.
- Incorrect assumptions are more likely to be found before API stabilization.
- ETL and Chapter 10 consumers can understand compatibility and failure modes.
- Refactoring can be checked against preserved semantic behavior.

### Negative

- Each feature requires documentation, provenance, and fixtures in addition to
  code.
- Sanitizing and licensing real capture excerpts may take significant effort.
- Golden outputs must be reviewed when intentionally changed.

## Follow-up actions

1. Add the BIE evidence ledger when the first sample arrives.
2. Add an architecture document with the first code-bearing milestone.
3. Define fixture metadata and sensitive-data rules before committing captures.
4. Require relevant documentation and tests in the completion criteria for
   each stage in ADR-0011.
