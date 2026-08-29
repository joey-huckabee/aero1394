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

| Offset | Width (bytes) | Description |
| ---: | ---: | --- |
| `0x00` | 4 | Nonzero data-item identifier encoded as a big-endian unsigned 32-bit integer. |
| `0x04` | 4 | Whole seconds of the recorder timestamp, encoded as big-endian unsigned Unix seconds. |
| `0x08` | 4 | Microsecond component of the recorder timestamp, encoded as a big-endian unsigned 32-bit integer. |
| `0x0C` | 4 | Raw big-endian status/length word. The low 16 bits contain `data_length`; the high 16 bits contain `unresolved_flags`. |
| `0x10` | N | Stored-data bytes, where `N` is exactly the `data_length` declared by the preceding status/length word. |

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
message-specific timing are not part of the generic container grammar. The BIE
parser must preserve the exact bytes and length without importing a protocol or
built-in payload decoder. This specification records a separate, explicitly
scoped interpretation for the supplied record family below; that interpretation
does not change the generic parser contract.

The supplied BIE stored-data region does not contain all information normally
expected in a complete IEEE-1394 wire packet. No complete link header, header
CRC, or data CRC has been identified around the application bytes. BIE may
retain a partial or normalized representation, but the current format evidence
does not establish which wire information was removed or transformed. The
higher-level evidence needed to define an IEEE-1394 wire decoder is tracked in
[`IEEE1394.md`](IEEE1394.md).

The normative AS5643 protocol boundary is tracked independently in
[`AS5643.md`](AS5643.md), and application definitions are kept in
[`PAYLOADS.md`](PAYLOADS.md). The BIE-specific observations below do not define
either standard.

The BIE container does not encode the configured sample-attempt rate or define
a scheduling relationship among acquisition, AS5643 frames, and payload
production. Those values belong to capture provenance and the appropriate
downstream layer.

## Observed stored-data layout for the supplied record family

For records with data-item ID `0x00005D04`, the stored-data region is 116
bytes. The following split is reproducible across the supplied BIE fixtures.
Offsets are relative to the beginning of `stored_data`, not the BIE record.
Protocol names in the table are candidates derived by comparison with AS5643;
they are not confirmed AS5643 fields.

| Stored-data offset | Width | BIE interpretation | Evidence |
| ---: | ---: | --- | --- |
| `0x00` | 4 | Neutral protocol word 0; health-status position candidate | Inferred; observed value is zero |
| `0x04` | 4 | Neutral protocol word 1; heartbeat position candidate | Position inferred; behavior contradicts the expected `+1` sequence |
| `0x08` | 92 | Application bytes for data-item ID `0x00005D04` | **Confirmed** by independent summary size and field alignment |
| `0x64` | 4 | STOF transmit-offset candidate, observed `1400` | Strong inference |
| `0x68` | 4 | STOF receive-offset candidate, observed `500` | Strong inference |
| `0x6C` | 4 | STOF datapump-offset candidate, observed `500` | Strong inference |
| `0x70` | 4 | VPC candidate | Strong inference from repeated XOR/complement behavior |

The byte accounting is exact for this record family:

```text
neutral protocol words    8
application payload      92
three STOF candidates    12
VPC candidate             4
                         ---
stored data             116 (0x74)
```

This layout is an observed BIE representation, not a universal BIE record
shape or a normative AS5643 message grammar. The generic BIE parser returns all
116 bytes as opaque stored data. A later, explicitly selected interpretation
may expose the split while preserving every raw word.

### Heartbeat discrepancy

AS5643 places Health Status and Heartbeat ahead of message data, which makes
the first two stored words plausible positional candidates. The second word
does not exhibit the expected simple heartbeat increment. Near the end of the
supplied recording it changes as follows:

```text
049CBDEE
049CBF8E   delta 416
049CC149   delta 443
049CC304   delta 443
```

The stable BIE-facing model must therefore retain neutral names such as
`protocol_word_0` and `protocol_word_1`. Plausible explanations include a BIE
transformation, producer behavior that differs from the expected sequence, a
different meaning for the supplied size, or another normalization layer. None
is currently established.

### VPC evidence and missing-header hypothesis

For every complete supplied record checked, complementing the XOR of all
visible words from protocol word 0 through the third STOF candidate produces a
value whose XOR difference from the stored VPC candidate is constant:

```text
visible calculation XOR stored VPC = 0x00005D60
```

For this message, that residual also equals:

```text
data_item_id                 0x00005D04
XOR protected payload size  0x00000064  (92 + 8)
                             ----------
                             0x00005D60
```

This is strong BIE-specific evidence that the stored representation may omit
or normalize a four-word ASM header from an original AS5643 packet. Message ID
and protected payload size explain the residual exactly. Security, node,
priority, and their encodings remain unknown; they may be zero, cancel under
XOR, or be represented elsewhere.

The repository tests preserve the residual as evidence. They do not report the
VPC as normatively valid until the missing header inputs, applicable AS5643
revision, coverage, and ordering are established.

### Evidence required to confirm the stored-data interpretation

- the exact meaning and encoding of both pre-payload words in BIE storage;
- whether BIE removes, transforms, or separately stores IEEE-1394 or AS5643
  fields;
- the four candidate omitted ASM-header words, including node, security, and
  priority;
- known-good and known-bad VPC examples from an independent decoder;
- records with different message IDs and protected payload sizes; and
- controlled heartbeat, STOF, missing-message, and independently varied
  capture-sampling and AS5643 frame-rate cases.

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
