# Aero1394 roadmap

- Scope: incomplete and forward-looking work only
- Last updated: 2026-08-30

## Roadmap rules

This file contains future work, investigation gates, and candidate input
formats. Completed work is removed from this file and recorded in the relevant
format document, requirement, test, release note, or ADR.

Candidate items are not compatibility promises. Each item must have authorized
sample data, a defined ownership boundary, and testable acceptance criteria
before implementation begins.

## Deliver the `v0.2.0` AS5643 and payload path

The dependency order, small functional increments, payload evidence request,
and exit gates are defined in
[`docs/RELEASE-PLAN-v0.2.0.md`](docs/RELEASE-PLAN-v0.2.0.md).

Raw AS5643 envelope decoding, VPC validation, explicit BIE mapping, CLI
presentation, and the deterministic built-in payload registry are implemented.
The next increment is raw typed decoding for `msfcs_storesmassdata_b`. Its field
layout is supplied, while source metadata and unresolved Boolean,
engineering-unit, and validity semantics remain required before its complete
engineering decoder can be called supported.

## Extend the internal BIE path

### Resolve BIE status flag `0x40000000`

The high flag appears in the second and third records of both supplied
four-record excerpts. Its meaning is not defined and must not be guessed.

Future investigation must obtain one or more of:

- the internal producer field definition;
- controlled captures that toggle one recorder or traffic condition at a time;
- an independently decoded record/event listing for the same interval; or
- additional internal messages that provide counterexamples to candidate
  meanings.

Test at least packet validity, direction, event type, sample-attempt result,
buffer state, and error-state hypotheses. A repeating record position alone is
not evidence of a scheduler meaning.

Exit gate: the bit has a documented name and polarity, positive and negative
fixtures, a stable raw-to-semantic mapping, and updated BIE requirements. Until
then, expose it only through the raw word and `unresolved_flags`.

### Model capture-rate provenance

Represent the configured sample-attempt rate independently from actual BIE
timestamps, AS5643 frame rate, and application payload production rate.

The supplied sample provenance records an 80 Hz attempt rate. Future controlled
evidence must cover both supported FireSpy sampling configurations, 80 Hz and
100 Hz, while retaining jitter, skipped attempts, and actual record gaps.

Exit gate: canonical capture metadata can distinguish configured rate from
observed intervals, and tests prove that no ideal 12.5 ms or 10 ms grid is
synthesized over the raw timestamps.

## Add future input-format adapters

Every format below is a separate adapter. None is an alias, source definition,
or provenance explanation for internal BIE files.

### FireSpy recorder files (`.fsr`)

DAP Technology identifies `.fsr` as the native FireSpy Recorder file. The
public operation manual does not define its byte layout.

Entry gate:

- an authorized native `.fsr` sample;
- an identifying signature or API-supported reader path;
- recorder model and FireDiagnostics version;
- an independently exported view of the same interval; and
- a new ADR defining whether Aero1394 parses the bytes or integrates an
  existing vendor API.

### FireSpy packet files (`.fsp`)

The documented version 1.0.0 packet export uses little-endian 32-bit words and
starts with these bytes:

```text
AE 46 53 70  00 01 00 00
```

Blocks encode a 12-bit block ID, a 20-bit unpadded byte length, and zero padding
to a four-byte boundary. The documented packet block does not preserve all
native capture timing, source, event, and error context.

Entry gate: representative exports, metadata-loss expectations, malformed
block fixtures, and a clear use case for packet-only import.

### Recorder regeneration files (`.rgn`)

The documented regeneration export uses logical file ID `0xAE52476E`, version
1, field definitions, and an item stream that can represent frame starts,
unformatted packets, and stream packets. The public manual excerpt does not
state the on-disk byte order.

Entry gate: confirm byte order from a real export, preserve relative transmit
intervals, define supported item kinds and unknown-item behavior, and test CRC
error indications and termination.

### IRIG 106 Chapter 10 IEEE-1394 data

Chapter 10 is a future standardized input adapter as decided by
[`ADR-0010`](docs/adr/0010-defer-chapter-10-to-a-future-input-adapter.md).
IRIG 106-11 identifies data type `0x58` for IEEE-1394 transaction data and
`0x59` for IEEE-1394 physical-layer data.

Entry gate: select the exact IRIG 106 revision and IEEE-1394 data format,
obtain representative captures, define normalized timestamp/source metadata,
and keep the adapter limited to Aero1394's IEEE-1394/AS5643 scope.

## Evaluate FireTrac acquisition integration

DAP Technology documents FireTrac support for Windows, VxWorks, Linux, QNX,
LabVIEW RT, and RTX64, including customized data-recorder, simulator, and
monitor applications. A future integration may use FireTrac or FireStack APIs
for controlled capture generation and validation.

Entry gate: a concrete acquisition or simulation use case, available hardware
or API access, redistribution approval, and an adapter boundary that does not
mistake the internal BIE format for a DAP-defined container.

## Extend built-in payload coverage

For each additional authorized payload structure:

- document the field layout in `docs/PAYLOADS.md`;
- add a dedicated Rust module and deterministic registry entry;
- retain sanitized messages and expected decoded values;
- add boundary, byte-order, ambiguity, and engineering-conversion tests; and
- update L3 requirements and regenerate the trace matrix.

Runtime YAML profiles remain outside the accepted design.

## Sources for future adapters

- [DAP 1394 Analyzer Operation Manual](https://www.daptechnology.com/fileadmin/manuals/OperationManual.pdf)
- [DAP FireTrac Mil1394](https://www.daptechnology.com/products/interface-solutions/firetrac-mil1394/)
- [DAP Mil1394 overview](https://www.daptechnology.com/mil1394)
- [IRIG 106-11 Chapter 10](https://www.irig106.org/docs/106-11/chapter10.pdf)
