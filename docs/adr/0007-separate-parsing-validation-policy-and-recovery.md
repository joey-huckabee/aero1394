# ADR-0007: Separate parsing, validation, policy, and recovery

- Status: Proposed
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

Flight-test recordings may contain malformed, truncated, or corrupt records.
A packet can be structurally readable even when a CRC is invalid. Conversely,
some byte sequences can appear structurally plausible without being real
record boundaries. Different tasks need different behavior: ETL may require
strict rejection, routine extraction may retain usable records with warnings,
and forensic work must expose partial results and recovery evidence.

If every parser embeds its own stop/skip/recover policy, those modes will drift
and callers will be unable to distinguish corrupt data from unsupported data or
an incomplete final record.

## Decision

Separate four concerns:

1. **Parsing** determines whether available bytes can be interpreted as a
   structure without reading out of bounds.
2. **Validation** records integrity and semantic checks without deciding
   whether the caller may use the result.
3. **Policy** decides whether findings are fatal for a particular operation.
4. **Recovery** searches for a defensible next boundary after framing is lost.

Parser errors should distinguish at least:

- truncated input, including needed and available lengths;
- structurally invalid input, including offset and reason;
- unsupported but identifiable versions or features; and
- I/O failures at the storage boundary.

Validation should use explicit outcomes such as `Valid`, `Invalid`,
`NotPresent`, and `NotChecked` rather than booleans that conflate absence with
success. An invalid CRC is normally a finding on a parsed packet, not automatic
proof that the packet cannot be inspected.

Operations will support policies with these semantics:

| Policy | Behavior |
| --- | --- |
| Strict | Reject a record when a required structural or integrity check fails |
| Permissive | Emit structurally usable records with attached findings |
| Forensic | Preserve partial evidence, raw bytes, offsets, and recovery rationale |

Names and exact command-line flags may change, but policy must be selected
outside the low-level parser.

Recovery must be an explicit strategy. A synchronization candidate should
carry its file offset, confidence category, and evidence rather than only an
offset. Candidate evidence may include:

- plausible container length and alignment;
- a plausible recorder timestamp or monotonic relationship;
- known flags or record kinds;
- a valid IEEE-1394 packet kind and consistent internal length;
- a valid applicable header or data CRC; and
- validation of one or more following records.

Confidence must be explained by evidence. The project should prefer categorical
confidence over a precise numeric score unless real labeled data supports the
score. Recovery scanning must be bounded, guarantee forward progress, and
report skipped byte ranges.

At minimum, design and tests must address:

- empty files and empty records;
- truncated file headers, records, packets, and final padding;
- zero, undersized, oversized, and arithmetic-overflowing lengths;
- alignment and padding that extend beyond available bytes;
- unknown record kinds, IEEE-1394 packet kinds, flags, or versions;
- CRC absent, not checked, valid, and invalid;
- non-monotonic, wrapping, implausible, or discontinuous timestamps;
- false synchronization candidates and valid records after corruption;
- trailing bytes and concatenated capture segments; and
- preservation of raw data when a higher protocol layer fails.

## Alternatives considered

### Treat every validation failure as a parse error

Rejected because corrupt packets can still contain useful metadata and payload
evidence, especially during recorder investigation.

### Make all parsing best-effort

Rejected because ETL and automated validation need deterministic failure
behavior and must not silently consume guessed boundaries.

### Search for the next plausible header without evidence reporting

Rejected because plausible binary patterns can create silent data loss or
misaligned decoding. Recovery must be auditable.

## Consequences

### Positive

- Strict ETL and exploratory inspection share one parser.
- Corruption is represented without discarding all decoded information.
- Recovery decisions can be tested and explained.
- Unsupported features are not mislabeled as corrupt input.

### Negative

- Result and diagnostic models are more detailed.
- Callers must select policy and handle attached findings.
- Reliable recovery requires more fixtures than stop-on-first-error parsing.

## Acceptance criteria

Accept this proposal after real BIE framing provides concrete validation checks
and recovery tests demonstrate bounded forward progress with no silent byte
skips. The exact error and finding types should not be stabilized before then.
