# Built-in application payloads

- Status: Initial payload evidence; complete field definition pending
- Last updated: 2026-08-28
- Applies to: application bytes located after BIE and protocol framing

## Purpose and boundary

This document records program-specific payload layouts separately from the BIE
container specification. `docs/BIE-FORMAT.md` explains how Aero1394 locates the
application bytes. This document explains what a known payload means.

Payload support is compiled into the Rust tool as decided by
[ADR-0012](adr/0012-use-built-in-rust-payload-definitions.md). Aero1394 does not
require YAML profiles. Each supported definition will live in its own `.rs`
module behind a central registry. An unknown payload remains available as raw
bytes.

Evidence labels have the same meanings used by `BIE-FORMAT.md`: **Confirmed**,
**Inferred**, **Hypothesis**, and **Unknown**.

## Payload registry model

Definitions are matched by more than message ID whenever the necessary context
is available:

```text
required now:   data_item_id + payload_size
optional later: data_code + configuration + definition version
```

The intended source layout is:

```text
src/payload/
  mod.rs                         public payload boundary
  registry.rs                    deterministic definition selection
  value.rs                       shared raw/engineering value types
  msfcs_storesmassdata_b.rs      one built-in definition
```

Every payload module must declare its identity, version, match criteria, byte
order, total size, and explicit field ranges. It decodes from a checked byte
slice and preserves raw values. The registry returns a typed known-payload
variant, an explicit ambiguous-match result, or an unknown payload containing
the original bytes.

No payload module may reach backward into BIE parsing. The layers interact like
this:

```text
BIE record -> protocol envelope -> raw application bytes
                                      |
                                      v
                           built-in payload registry
                                      |
                           known typed or unknown raw
```

## Adding a payload definition

For each additional structure:

1. add its authoritative or observed field table to this document;
2. create `src/payload/<payload_name>.rs`;
3. give every field an explicit byte/bit range and byte order;
4. register the definition using all available match criteria;
5. add sanitized message bytes and expected field values under
   `tests/fixtures/payload/<payload_name>/`;
6. test exact size, boundary values, invalid/truncated input, and byte order;
7. test raw and engineering representations separately; and
8. link the implementation and tests from `docs/REQUIREMENTS.md`.

Adding a definition requires a normal rebuild. A future generator may emit the
same Rust modules from an authorized ICD, but runtime YAML loading is outside
the accepted design.

## `msfcs_storesmassdata_b`

### Source metadata and correction

The application definition is correlated with this corrected recorder summary:

```text
Recording Date: Wed Jul 31 08:05:48 2024
Data File: Startup.draw.data.1394.vs_bus_b3.unused.bie
Data Type: IEEE 1394
Data Code: vs_bus_b3
Hardware Type: BIE_LINUX
Data Items Recorded: msfcs_storesmassdata_b ID=00005D04 Size=92
Recorder Buffer Mode: Direct to File (local disk)
Data Set Count: 1
Recording Timetags:
  start=31:08:01:59.063844
  stop=31:08:05:46.335672
```

An earlier summary identified a different `vs_bus_a3` file and
`msfcs_storesmassdata_a`, ID `0x000035F4`, size 92. The user explicitly
corrected that association before the byte map was finalized. None of the
committed `0x00005D04` fixtures may be treated as evidence for the `_a`
payload. That second payload remains a separate future input.

### Identity and size

| Property | Value | Evidence |
| --- | --- | --- |
| Name | `msfcs_storesmassdata_b` | **Confirmed** by recorder summary |
| Data-item ID | `0x00005D04` | **Confirmed** by summary and record bytes |
| Application size | 92 bytes | **Confirmed** by summary and record geometry |
| Byte order | Big-endian for currently observed multi-byte values | **Confirmed** for timestamp and float candidates |
| Data code | `vs_bus_b3` | **Confirmed** by recorder summary |
| Platform production rate | 60 Hz | **Confirmed for the supplied platform capture** by 876 intervals over 14.6 seconds |

The payload occupies record-relative bytes `0x18..0x73` in the observed record
family. Those record offsets are not part of the Rust payload definition; the
payload module sees byte offset zero at record offset `0x18`.

### Carrier relationship

The record-specific protocol words, STOF candidates, VPC evidence, missing ASM
header hypothesis, and recorder cadence are isolated in
[`AS5643.md`](AS5643.md). They are context for locating this payload, not fields
of the Rust application structure.

### Platform production rate

This payload is produced at 60 Hz by the supplied platform implementation. The
complete-file geometry provides the direct calculation:

```text
877 records -> 876 intervals
elapsed recorder time = 14.6 seconds
876 / 14.6 = 60 payload records/second
```

This is a property of this platform payload, not a FireSpy operating rate and
not a generic IEEE-1394 or AS5643 rule. FireSpy operates at a configured 80 Hz
or 100 Hz cadence; the supplied timestamps are consistent with the 12.5 ms,
80 Hz case. The 12.5 ms and 25 ms record gaps do not by themselves define a
formal scheduling relationship between the platform's 60 Hz producer and the
recorder cadence.

### Known and provisional field map

Only the first field has been supplied from the payload definition. The
remaining layout is retained as a useful hypothesis until the full structure
is provided.

| Payload offset | Width | Current name/type | Evidence |
| ---: | ---: | --- | --- |
| `0` | 8 | `TimeStamp`, 64-bit long long integer, words 0-1 | **Confirmed** by supplied definition |
| `8` | 4 | status/validity-like raw word | Hypothesis from behavior |
| `12` | 80 | twenty aligned big-endian IEEE-754 `f32` candidates | Strong structural inference; names/units unknown |

The supplied definition describes `TimeStamp` as:

```text
Element size: 64 bits
Word ID:      0-1
MSB:          0
LSB:          31
Byte offset:  0
Bit offset:   0
```

In the observed big-endian payload, the two adjacent words form one monotonic
value. For example:

```text
payload bytes 0..7 = 00 00 24 E6 14 F0 13 B3
raw value            0x000024E614F013B3
decimal              40,570,612,356,019
```

The field is believed to contain system ticks. Its epoch, signedness, and exact
tick rate are **not confirmed**, so the canonical model keeps its 64 raw bits
losslessly (a `u64`-backed `SystemTicks` newtype is suitable without claiming
the ICD's signed interpretation). It must not be converted to a calendar date.

### Recorder time versus payload time

The record contains two independent time domains:

| Time | Wire representation | Meaning |
| --- | --- | --- |
| BIE recorder time | `u32be` Unix seconds plus `u32be` microseconds | Absolute time when the recorder stored/observed the record |
| Payload `TimeStamp` | `u64be` raw ticks | Source/application system time; epoch unknown |

Keeping the names and raw types distinct allows later latency or clock-drift
analysis without confusing the application clock with recorder wall time.

Across the supplied first and final messages:

```text
first ticks     0x000024B7DC01E3E3 = 40,372,088,726,499
final ticks     0x000024E614F013B3 = 40,570,612,356,019
tick delta                              198,523,629,520
recorder delta                                      14.6 s
empirical rate                         13,597,508,871 ticks/s
```

Regression over the supplied pairs was also approximately 13.5983 GHz. A
13.6 GHz nominal rate is therefore a strong candidate, corresponding to about
73.5294 picoseconds per nominal tick, but it remains provisional. Any derived
seconds value must carry the selected rate and evidence state. Parquet metadata
is an appropriate place to record that rate; the raw ticks remain canonical.

### Provisional float evidence

The populated final fixture has one raw word followed by twenty words that
decode plausibly as big-endian `f32` values:

| Candidate | Hex | Value |
| ---: | --- | ---: |
| 1 | `45AF1829` | about `5603.02002` |
| 2 | `43F50000` | `490.0` |
| 3 | `3F800000` | `1.0` |
| 4 | `428C0000` | `70.0` |
| 5 | `47090A00` | `35082.0` |
| 6 | `4497C000` | `1214.0` |
| 7 | `470D1300` | `36115.0` |
| 8 | `42140000` | `37.0` |
| 9 | `40A00000` | `5.0` |
| 10 | `42D20000` | `105.0` |
| 11 | `44BA00A4` | about `1488.02002` |
| 12 | `43FB0000` | `502.0` |
| 13 | `40800000` | `4.0` |
| 14 | `42A20000` | `81.0` |
| 15 | `461D4400` | `10065.0` |
| 16 | `43780000` | `248.0` |
| 17 | `4620A800` | `10282.0` |
| 18 | `41B80000` | `23.0` |
| 19 | `C1000000` | `-8.0` |
| 20 | `42200000` | `40.0` |

The sparse startup fixture populates only a few positions, including `450.0`
and `62.5`, while preserving the same 92-byte structure. This supports one
stable payload layout whose values become populated during initialization; it
does not justify field names, units, scaling, or validity rules.

The eventual Rust structure must use neutral field names only as a temporary
implementation aid. When the remaining definition is supplied, replace the
provisional table with the real names, types, offsets, units, arrays, bitfields,
and validity relationships before calling the payload decoder supported.

## Primitive and engineering representations

Built-in definitions should initially support explicit integer and floating
wire types (`u8` through `u64`, signed equivalents, `f32`, `f64`, booleans,
bitfields, and fixed byte ranges) as actual payloads require them. Arrays keep
one logical name and a count; CSV may flatten them while typed outputs may
retain list structure.

Physical decoding and engineering interpretation are separate. Scaling,
offset, units, enumerations, and validity conditions produce derived values
without discarding the original word. Unknown gaps are preserved and reported;
overlapping or out-of-bounds declared fields are rejected.

## Open inputs needed

- the remaining `msfcs_storesmassdata_b` field names, types, byte/bit offsets,
  units, arrays, and validity rules;
- whether `TimeStamp` is unsigned and the authoritative system-tick frequency;
- the meaning and byte/bit order of the word at payload offset 8;
- whether ID `0x00005D04` is reused across data codes or configurations;
- the corresponding `msfcs_storesmassdata_a` definition and its corrected
  capture evidence; and
- handling/redistribution constraints for every supplied ICD or structure.
