# BIE binary format

- Status: Current internal specification; framing defined and `0x40000000` unresolved
- Last updated: 2026-08-28
- Applies to: internally defined `.bie` aerospace IEEE-1394 recordings

## Purpose

This document is the definitive format specification and parser contract for
BIE, an internally defined capture format that is not a DAP Technology,
FireSpy, or FireTrac format. It defines the supported file and record grammar,
required parser behavior, recognition rules, stored-data boundary, and
compatibility constraints. Unresolved fields are explicitly preserved without
assigning unsupported meanings.

The grammar and parser requirements below are normative for the currently
supported BIE format unless a field is explicitly marked with one of these
status labels:

| Label | Meaning |
| --- | --- |
| Confirmed | Supported by the internal definition or independently reproducible evidence |
| Inferred | A conclusion drawn from confirmed behavior, but not yet independently defined |
| Hypothesis | Plausible and worth testing against another sample |
| Needs resolution | Preserved and tracked, but its semantic meaning is not yet known |

## File and record grammar

A BIE file contains zero or more length-delimited records followed by one
zero-word sentinel. All multi-byte record fields are encoded in big-endian
byte order.

```text
file        := record* zero_word_sentinel
record      := record_header stored_data
record_size := 16 + data_length
```

### Record structure

Each record consists of a fixed 16-byte header followed immediately by the
number of stored-data bytes declared in `status_and_length`.

| Record offset | Width (bytes) | Field and description |
| ---: | ---: | --- |
| `0x00` | 4 | `data_item_id` — Nonzero unsigned 32-bit identifier for the stored data. |
| `0x04` | 4 | `recorder_seconds` — Whole seconds of the recorder timestamp, represented as unsigned Unix seconds. |
| `0x08` | 4 | `recorder_useconds` — Microsecond component of the recorder timestamp, represented as an unsigned 32-bit integer. |
| `0x0C` | 4 | `status_and_length` — Raw control word containing unresolved flags in the high 16 bits and `data_length` in the low 16 bits. |
| `0x10` | N | `stored_data` — Exactly `data_length` bytes; `N` may range from 0 through 65,535. |

### Status and length word

The two components of `status_and_length` are calculated as follows:

```text
data_length      := status_and_length & 0x0000_FFFF
unresolved_flags := status_and_length & 0xFFFF_0000
```

`data_length` counts the entire `stored_data` region, not only the application
payload. The following example defines the 132-byte records in the supplied
record family:

| Quantity | Calculation or composition | Example value |
| --- | --- | ---: |
| Raw `status_and_length` | Big-endian `u32` | `0x00000074` |
| `unresolved_flags` | `0x00000074 & 0xFFFF0000` | `0x00000000` |
| `data_length` | `0x00000074 & 0x0000FFFF` | `0x0074` = 116 bytes |
| Stored-data contents | 8 AS5643 payload-header bytes + 92 application bytes + 12 STOF-offset bytes + 4 VPC bytes | 116 bytes |
| Fixed record header | Four 4-byte fields | 16 bytes |
| Complete record | `16 + data_length` | 132 bytes (`0x84`) |

The alternative observed value `0x40000074` has
`unresolved_flags = 0x40000000` and the same `data_length` of 116 bytes.

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

### Zero-word sentinel

The zero-word sentinel is exactly four zero bytes at the offset where the next
record's `data_item_id` would otherwise begin:

```text
zero_word_sentinel := 00 00 00 00
```

The sentinel terminates the BIE file. It is not a record and is not followed by
the remaining 12 header bytes or a stored-data region. A conforming file ends
immediately after the sentinel; any following bytes are trailing data. Because
zero is reserved for the sentinel, every record has a nonzero `data_item_id`.

No separate magic, version header, metadata table, or index is represented in
the current grammar. Any future addition of a file-level structure requires an
explicit format version.

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
message-specific timing are not part of the generic container grammar. The BIE
parser must preserve the exact bytes and length without importing a protocol or
built-in payload decoder. This specification records a separate, explicitly
scoped interpretation for the supplied record family below; that interpretation
does not change the generic parser contract.

For the mapped record family, the BIE stored-data region is a normalized
protocol representation rather than a complete IEEE-1394 wire packet. It
omits the IEEE-1394 link header, header CRC, and data CRC, as well as the
four-word AS5643 ASM header. [`IEEE1394.md`](IEEE1394.md) owns the excluded
wire-level definitions.

The assumed AS5643 field definitions are specified independently in
[`AS5643.md`](AS5643.md), and application definitions are kept in
[`PAYLOADS.md`](PAYLOADS.md). The table below defines only how this BIE record
family stores that logical AS5643 structure.

The BIE container does not encode the configured sample-attempt rate or define
a scheduling relationship among acquisition, AS5643 frames, and payload
production. Those values belong to capture provenance and the appropriate
downstream layer.

## AS5643 stored-data mapping for data-item `0x00005D04`

For records with data-item ID `0x00005D04`, BIE stores the AS5643 payload
header, 92 application bytes, and AS5643 packet trailer as a 116-byte
`stored_data` region. BIE omits the four-word ASM header and the surrounding
IEEE-1394 header and CRC fields. Offsets below are relative to the beginning of
`stored_data`, not the BIE record.

| Stored-data offset | Width | Stored field | AS5643 definition |
| ---: | ---: | --- | --- |
| `0x00` | 4 | Health Status | [`AS5643.md`](AS5643.md#health-status) |
| `0x04` | 4 | Heartbeat | [`AS5643.md`](AS5643.md#heartbeat) |
| `0x08` | 92 | Application data for message `0x00005D04` | [`AS5643.md`](AS5643.md#application-data) and [`PAYLOADS.md`](PAYLOADS.md#msfcs_storesmassdata_b) |
| `0x64` | 4 | STOF Transmit Offset | [`AS5643.md`](AS5643.md#stof-transmit-offset) |
| `0x68` | 4 | STOF Receive Offset | [`AS5643.md`](AS5643.md#stof-receive-offset) |
| `0x6C` | 4 | STOF Datapump Offset | [`AS5643.md`](AS5643.md#stof-datapump-offset) |
| `0x70` | 4 | Vertical Parity Check | [`AS5643.md`](AS5643.md#vertical-parity-check) |

The byte accounting is:

```text
Health Status and Heartbeat    8
application data              92
three STOF offsets            12
Vertical Parity Check          4
                              ---
stored_data                  116 (0x74)
```

The retained AS5643 payload and trailer begin 16 bytes after the omitted
logical ASM header. Thus, for every retained field:

```text
logical_ASM_offset = BIE_stored_data_offset + 0x10
```

The omitted ASM header is reconstructed for this record family as follows:

| AS5643 header field | BIE source or profile value | Definition |
| --- | --- | --- |
| Message ID | BIE `data_item_id` = `0x00005D04` | [`AS5643.md`](AS5643.md#message-id) |
| Reserved/security | Profile constant `0x00000000` | [`AS5643.md`](AS5643.md#reservedsecurity-word) |
| Node ID | Profile constant `0x00000000` | [`AS5643.md`](AS5643.md#node-id) |
| Priority/payload length | Profile constant `0x00000064` | [`AS5643.md`](AS5643.md#priority-and-payload-length) |

The generic BIE parser continues to return `stored_data` as an opaque byte
region. The AS5643 decoder applies profile
`aero1394-assumed-as5643b-v1` only after the record family is selected.
Heartbeat freshness evaluation, STOF-offset interpretation, and VPC validation
follow the linked AS5643 definitions; they are not BIE framing rules.

The Rust `bie_as5643` adapter implements that selection without adding protocol
knowledge to the BIE parser. It requires both data item `0x00005D04` and an
exact 116-byte stored region. Other IDs return `UnsupportedDataItem`; the known
ID with another length returns `UnsupportedStoredDataLength` with expected and
actual sizes. Both outcomes retain the complete parsed record and are not
classified as corrupt merely because no supported profile matches.

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
