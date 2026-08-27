# ADR-0005: Reverse-engineer BIE with an evidence-led process

- Status: Accepted
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

No authoritative BIE specification or sample capture is currently present in
the repository. The files are believed to be produced by a DAP Technologies
FireSpy recorder, but that provenance has not yet been confirmed from file
contents, recorder metadata, or vendor documentation.

IEEE-1394 defines bus behavior; it does not by itself define how a recorder
stores captured traffic. A BIE record could contain physical-layer data, link
packets, completed transactions, AS5643 messages, or vendor metadata wrapped
around any of those representations.

Other capture representations can inform the investigation but cannot be
assumed to describe BIE. Examples identified during planning include IEEE-1394
data formats in IRIG 106 Chapter 10 and the Linux `isodump` format. Likewise,
AS5643 is a likely aerospace payload but is not evidence about the outer BIE
container.

Prematurely assigning semantic names to byte offsets would make guesses look
like API guarantees and could bias later investigation.

## Decision

Reverse-engineer BIE as an evidence-led, reproducible process. Until supported
by evidence, statements about the format must be labeled **hypothesis**, not
fact.

The preferred investigation sequence is:

1. preserve the original sample and record a cryptographic hash, size, source,
   recorder model, recorder software version, and acquisition context;
2. map file-wide statistics and offset-oriented hex views;
3. identify repeated structures and candidate record boundaries;
4. test candidate lengths, alignment, padding, byte order, and timestamps;
5. search for plausible embedded IEEE-1394 headers;
6. test applicable header and data CRC algorithms against candidate packets;
7. search for repeated AS5643 message, node, heartbeat, and STOF patterns only
   after the containing 1394 structure is plausible;
8. compare records with an independent export of the same capture interval;
9. test competing interpretations against multiple records and, when possible,
   multiple captures; and
10. document confirmed fields and counterexamples before stabilizing an API.

The investigation must distinguish three evidence levels:

| Level | Meaning |
| --- | --- |
| Unknown | Bytes or behavior have not been explained |
| Hypothesis | An interpretation fits current observations but lacks independent confirmation |
| Confirmed | Multiple observations or authoritative documentation support the interpretation and plausible alternatives were tested |

`docs/BIE-FORMAT.md` will become the evidence ledger and eventual format
specification. For each proposed field or record kind it should capture:

- absolute and record-relative offsets;
- width, byte order, units, scaling, and valid range;
- whether the value participates in a record length or checksum;
- sample observations and the captures in which they occur;
- confidence/evidence level and unresolved alternatives; and
- corner cases, version differences, and validation rules.

Names in exploratory code should remain neutral, such as `candidate_length` or
`field_0c`, until the meaning is supported. Raw bytes and source offsets must
remain inspectable even after a record is decoded.

Reference formats and protocol standards may be used to form tests, but format
similarity alone is not confirmation. The exact revision and legal source of a
standard must be recorded before normative behavior is implemented.

The planning discussion identified these comparison sources. They are research
leads, not claims about BIE:

| Comparison source | Potential use |
| --- | --- |
| IRIG 106 Chapter 10 IEEE-1394 Data Formats 0 and 1 | Compare the metadata a flight-test recorder may retain and rule in or out standard Chapter 10 framing |
| Linux `isodump` version 1 format | Recognize or rule out a known isochronous capture representation |
| IEEE-1394 specification for the selected packet form | Validate packet fields, length relationships, byte/bit order, and applicable CRCs |
| SAE AS5643 material for the selected revision | Test candidate ASM, status, heartbeat, STOF, timing, and parity interpretations |
| FireSpy software export of the same interval | Correlate record counts, timestamps, channels, packet types, and decoded values independently |

## Required starting evidence

The first milestone requires at least one representative `.bie` sample. The
strongest companion artifact is an export from the vendor application covering
the same time interval. Hardware model, firmware, recorder software version,
capture settings, and any known traffic should be retained with the sample.

Sensitive or proprietary captures must not be committed without explicit
authorization. Prefer sanitized extracts and synthetic fixtures once the
necessary structural properties are understood.

## Alternatives considered

### Assume BIE is a standard IEEE-1394 or Chapter 10 representation

Rejected because there is no current evidence for that equivalence. Doing so
would risk aligning the parser around the wrong framing and byte order.

### Infer the format from a single plausible record

Rejected because random binary data often contains plausible lengths, times,
and bit fields. Independent structural checks and next-record corroboration are
needed.

### Wait for complete vendor documentation before building tools

Rejected because an offset-oriented Rust forensic tool is useful even if the
format remains partially unknown. Documentation, if obtained, becomes another
evidence source rather than a prerequisite for every experiment.

## Consequences

### Positive

- Guesses remain visible and reversible.
- Format claims are traceable to captures or authoritative sources.
- The public library is less likely to freeze incorrect semantics.
- The same workflow can detect format versions and recorder configuration
  differences.

### Negative

- Early progress is measured in evidence rather than user-facing decoding.
- Maintaining provenance and counterexamples adds documentation work.
- Some fields may remain deliberately unnamed for several iterations.

## Follow-up actions

1. Obtain a representative BIE capture and, if possible, a matching export.
2. Create `docs/BIE-FORMAT.md` from a template that includes evidence status.
3. Implement the first forensic commands described in ADR-0011.
4. Record relevant standards and vendor references with revision information.
