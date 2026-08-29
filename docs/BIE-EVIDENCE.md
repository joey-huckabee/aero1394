# BIE specification development evidence

- Status: Working development record; non-normative
- Last updated: 2026-08-28

## Purpose

This companion document preserves the capture provenance, environment context,
and chronological research notes used while developing the BIE specification.
It is intentionally non-normative. The definitive file grammar, parser
requirements, and compatibility rules are maintained in
[`BIE-FORMAT.md`](BIE-FORMAT.md).

## Supplied capture evidence

The current byte map is correlated with excerpts of one internal simulation
recording and its corrected recorder summary. The container-relevant summary
values are:

```text
Data File: Startup.draw.data.1394.vs_bus_b3.unused.bie
Data Type: IEEE 1394
Data Code: vs_bus_b3
Recorder Buffer Mode: Direct to File (local disk)
Data Set Count: 1
Recording Timetags:
  start=31:08:01:59.063844
  stop=31:08:05:46.335672
```

The summary reports a recording date of Wednesday, July 31, 2024. The complete
capture is not committed, so its digest and handling classification are not
available in the repository. The supplied end-of-file offsets imply this
geometry for the observed recording:

```text
877 records * 132 bytes = 115,764 bytes
zero word at EOF         =       4 bytes
total                    = 115,768 bytes (0x1C438)
```

Selected sanitized records are retained as machine-readable hexadecimal test
inputs under [`tests/fixtures/bie`](../tests/fixtures/bie/README.md). They are
evidence for the current definition and do not contain the complete source
capture.

### Capture-rate context

Sampling of the supplied messages was attempted at 80 Hz. An 80 Hz attempt
rate has a nominal interval of 12.5 ms; a 100 Hz configuration has a nominal
interval of 10 ms. FireSpy sampling in the surrounding test environment may be
configured for either 80 Hz or 100 Hz, consistent with the typical AS5643 STOF
frame rates documented by DAP Technology ([DAP-MIL1394]).

The BIE timestamps remain authoritative for what was actually recorded. The
supplied excerpts contain exact 12.5 ms and 25 ms gaps as well as a 24.142 ms
gap, so a decoder must not synthesize an ideal sampling grid or invent missing
records. The configured sample-attempt rate, actual record timestamps, AS5643
frame rate, and application payload production rate are separate values.

### FireTrac environment note

DAP Technology documents FireTrac support for Linux and customized
data-recorder, simulator, and monitoring applications ([DAP-FIRETRAC]). That
is relevant deployment context for an internal recorder, but it does not make
BIE a FireTrac, FireSpy, or DAP-defined file format.

## Research log

### 2026-08-28

- Confirmed that BIE is an internally defined format and removed the earlier
  external-format provenance hypothesis.
- Established the 16-byte big-endian record header and length-delimited stored
  data for the supplied record family.
- Correlated recorder seconds and microseconds with the July 31, 2024 local
  recording window.
- Recorded the 877-record-plus-zero-word geometry and retained sanitized
  startup and populated records as test fixtures.
- Defined sentinel, truncation, trailing-data, unknown-ID, timestamp, and
  recognition behavior for the parser.
- Marked `0x40000000` as needing resolution and moved its investigation into
  the forward-looking roadmap.
- Recorded that sampling was attempted at 80 Hz while preserving actual BIE
  timestamps and keeping 80 Hz/100 Hz capture configuration distinct from
  payload production timing.
- Adopted a provisional AS5643 profile for Health Status, Heartbeat, STOF
  offsets, the omitted ASM header, and VPC; retained only the BIE byte mapping
  in the BIE specification.
- Kept FireTrac only as possible source-environment context; it is not BIE
  format provenance.

## Sources

- **DAP-MIL1394** — [Mil1394 (SAE AS5643) specification overview](https://www.daptechnology.com/mil1394)
- **DAP-FIRETRAC** — [FireTrac Mil1394 product page](https://www.daptechnology.com/products/interface-solutions/firetrac-mil1394/)

[DAP-MIL1394]: https://www.daptechnology.com/mil1394
[DAP-FIRETRAC]: https://www.daptechnology.com/products/interface-solutions/firetrac-mil1394/
