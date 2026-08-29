# BIE binary format

- Status: Internal format; observed framing is implementation-ready and `0x40000000` needs resolution
- Last updated: 2026-08-28
- Applies to: internally defined `.bie` aerospace IEEE-1394 recordings

## Purpose

This document is the evidence ledger and parser contract for the internal BIE
capture format. It defines the supported file and record grammar, distinguishes
wire facts from downstream interpretation, and records the remaining semantic
work without assigning meanings that the available evidence cannot support.

BIE is not a DAP Technology or FireSpy file format. FireSpy and FireTrac may be
part of a surrounding capture, simulation, or analysis environment, but they
do not define ownership or the byte layout documented here. Candidate support
for other input containers belongs in the forward-looking
[`ROADMAP.md`](../ROADMAP.md), not in this format definition.

Evidence labels used below are:

| Label | Meaning |
| --- | --- |
| Confirmed | Supported by the internal definition or independently reproducible evidence |
| Inferred | A conclusion drawn from confirmed behavior, but not yet independently defined |
| Hypothesis | Plausible and worth testing against another sample |
| Needs resolution | Preserved and tracked, but its semantic meaning is not yet known |

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

## File and record grammar

A BIE file is a sequence of big-endian, length-delimited records ending in a
zero-word sentinel:

```text
file := record* zero_word

record :=
    data_item_id       u32be
    recorder_seconds   u32be
    recorder_useconds  u32be
    status_and_length  u32be
    stored_data        u8[data_length]

zero_word        := 00 00 00 00
data_length      := status_and_length & 0x0000_FFFF
unresolved_flags := status_and_length & 0xFFFF_0000
record_size      := 16 + data_length
```

The decoder must not make 132 bytes a universal record size. That size follows
only for the supplied record family because its stored-data length is `0x0074`,
or 116 bytes.

No separate magic, version header, metadata table, or index is represented in
the current grammar. If a future internal definition adds one, it must be
treated as an explicit format version rather than guessed from incidental
bytes.

## Record header

| Offset | Width | Interpretation | Evidence |
| ---: | ---: | --- | --- |
| `0x00` | 4 | Nonzero data-item ID | **Confirmed** by exact match with recorder summary metadata |
| `0x04` | 4 | Unsigned Unix seconds, big-endian | **Confirmed** by recording date and time correlation |
| `0x08` | 4 | Microseconds within the second, big-endian | **Confirmed** by time correlation |
| `0x0C` | 4 | Raw status/length word | **Confirmed** structurally; `0x40000000` **needs resolution** |
| `0x10` | N | Stored data, where `N` is the low 16-bit length | **Confirmed** for supplied records |

### Status and length word

The supplied records contain `0x00000074` and `0x40000074`. In both cases the
low value `0x0074` selects 116 following bytes and chains exactly to the next
record boundary.

The meaning of upper flag `0x40000000` is not resolved. It occurs in the second
and third records of each supplied four-record excerpt and is clear in the
first and fourth. That pattern is evidence that it changes per record, but it
does not establish an event, error, direction, validity, or sampling meaning.

Until the investigation in
[`ROADMAP.md`](../ROADMAP.md#resolve-bie-status-flag-0x40000000) is complete,
the parser must:

- preserve the complete raw `status_and_length` word;
- expose the low 16-bit `data_length` separately;
- expose the high 16 bits under a neutral name such as `unresolved_flags`;
- retain `0x40000000` in fixtures and machine-readable output; and
- assign no boolean or event name to the flag.

### Recorder timestamp

The first supplied record contains:

```text
66 AA 36 9B 00 0B 2F C9
```

This decodes to Unix second `1722431131` plus `733129` microseconds, or
`2024-07-31T13:05:31.733129Z`. In America/Chicago on that date it is
`08:05:31.733129`, inside the independently reported recording window. The
final supplied record decodes to `2024-07-31T13:05:46.333129Z`, about 2.543 ms
before the reported stop time.

The raw seconds value must be modeled as `u32` and widened before arithmetic.
If it is an unsigned Unix counter, it wraps after
`2106-02-07T06:28:15Z`; interpreting it as signed would instead introduce an
unwanted 2038 limit. Raw seconds and microseconds remain part of the decoded
model even when a calendar representation is available.

## Supported parser contract

Parsing and validation use these rules:

- Decode every header word as big-endian and use checked arithmetic for each
  offset and `16 + data_length` calculation.
- Interpret a zero data-item ID at a record boundary as the four-byte
  terminator. A clean file ends immediately after it; report any following
  bytes as trailing data.
- Report physical EOF in a header or declared body as truncation. Report EOF
  exactly at a record boundary without the zero word as a missing terminator.
- Accept any nonzero data-item ID and any low-16-bit stored-data length,
  including zero, structurally. Payload support, a known ID, and the observed
  116-byte size are not container-validity requirements.
- Preserve the absolute record offset, all four raw header words, and the exact
  stored-data bytes. Unknown IDs and unresolved flags remain inspectable.
- Parse recorder seconds and microseconds as raw `u32` values. Validation flags
  microseconds greater than `999999`; timestamp monotonicity is an optional
  sequence check, not a framing requirement.
- Accept the four-byte sentinel-only form as the structural empty
  representation for synthetic and defensive tests. No supplied producer-made
  empty file confirms that convention yet.

The maximum body length expressible by the current length field is 65,535
bytes, so the maximum structurally representable record is 65,551 bytes. A
caller may impose a smaller resource policy without changing the wire grammar.

## Recognition and failure classification

The `.bie` extension alone is insufficient for automatic recognition.
Recognition requires at least one complete record and a complete chain from
absolute offset zero to a terminal zero word, with every declared body fitting
inside the input. The sentinel-only empty form requires explicit format
selection. Valid microsecond values and plausible timestamps increase
confidence but do not replace structural chaining.

When the caller explicitly selects BIE, an incomplete chain is malformed input
and receives a precise missing-header, missing-body, missing-terminator,
overflow, or trailing-data diagnostic. During automatic detection, bytes that
cannot establish the record chain remain unrecognized rather than being
forced into the BIE model.

## Stored-data boundary

At the BIE layer, the declared stored-data region is opaque. Its internal
protocol envelope, application identity, field layout, integrity behavior, and
message-specific timing do not belong to the container format. The BIE parser
must preserve the exact bytes and length without importing a built-in payload
definition.

The supplied BIE stored-data region does not contain all information normally
expected in a complete IEEE-1394 wire packet. No complete link header, header
CRC, or data CRC has been identified around the application bytes. BIE may
retain a partial or normalized representation, but the current format evidence
does not establish which wire information was removed or transformed. The
higher-level evidence needed to define an IEEE-1394 wire decoder is tracked in
[`IEEE1394.md`](IEEE1394.md).

The currently observed downstream-protocol evidence is kept in
[`AS5643.md`](AS5643.md), and application definitions are kept in
[`PAYLOADS.md`](PAYLOADS.md). Both are isolated from the generic BIE grammar.

The BIE container does not encode the configured sample-attempt rate or define
a scheduling relationship among acquisition, AS5643 frames, and payload
production. Those values belong to capture provenance and the appropriate
downstream layer.

## Implementation dispositions

| Question | Current parser policy | Follow-up |
| --- | --- | --- |
| How is a file terminated? | Require the zero-word sentinel and report following bytes. | Confirm the empty-file convention with an internally produced empty capture. |
| How is stored-data length derived? | Use the low 16 bits and preserve the complete raw word. | Add fixtures with lengths other than 116 bytes. |
| What does `0x40000000` mean? | **Needs resolution.** Preserve it as an unresolved flag and assign no semantic name. | Complete the dedicated [`ROADMAP.md`](../ROADMAP.md#resolve-bie-status-flag-0x40000000) investigation. |
| Are other IDs or zero-length records valid? | Accept them structurally and leave unknown contents raw. | Add internally defined examples and expected semantics. |
| Where are bus, node, channel, and recorder configuration stored? | They are not part of the current grammar; accept them as external provenance. | Define versioned BIE metadata only if the internal format is extended. |
| Which downstream protocol or payload applies? | The BIE layer does not decide; it returns opaque stored data. | Resolve in the IEEE-1394, AS5643, and payload layers. |
| How are event-like records represented? | Preserve their generic framing and raw stored data. | Add controlled reset, prefix-only, acknowledge-only, and error cases. |

## Evidence required to extend the BIE definition

For each additional complete internal sample, preserve:

- original filename and extension;
- producing application and version;
- capture settings, configured sample-attempt rate, bus count, and protocol
  mode;
- file size, modification time, and SHA-256 hash;
- redistribution and handling constraints;
- first and last 256 bytes;
- known record count, data-item IDs, and data sizes; and
- expected behavior for every intentionally varied condition.

Controlled captures are particularly valuable: an empty recording, two
different payload lengths, multiple data-item IDs, a bus reset, an
acknowledge-only event, a prefix-only observation, an intentionally bad CRC,
and configured 80 Hz and 100 Hz sampling cases.

## Compatibility constraints

- Implement the internal BIE definition independently of future input
  adapters listed in [`ROADMAP.md`](../ROADMAP.md).
- Implement only evidenced fields and keep provisional protocol fields under
  neutral names.
- Do not expose guessed BIE or downstream semantics in a stable Rust or Python
  API.
- Preserve raw bytes, absolute offsets, and the evidence for each detected
  structure.
- Report ambiguous automatic detection rather than selecting the first
  plausible format.
- Treat any future BIE header or grammar change as a versioned extension.

The format-neutral Rust command remains useful for bounded observations:

```text
cargo run --release -- hexdump path/to/capture.bie --offset 0 --length 256
```

See [Reverse-engineering BIE captures](REVERSE-ENGINEERING.md) for bounded
range selection, provenance recording, and handling guidance.

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
- Kept FireTrac only as possible source-environment context; it is not BIE
  format provenance.

## Sources

- **DAP-MIL1394** — [Mil1394 (SAE AS5643) specification overview](https://www.daptechnology.com/mil1394)
- **DAP-FIRETRAC** — [FireTrac Mil1394 product page](https://www.daptechnology.com/products/interface-solutions/firetrac-mil1394/)

[DAP-MIL1394]: https://www.daptechnology.com/mil1394
[DAP-FIRETRAC]: https://www.daptechnology.com/products/interface-solutions/firetrac-mil1394/
