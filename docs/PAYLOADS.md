# Built-in application payloads

- Status: Registry implemented and complete field layout supplied; engineering semantics pending
- Last updated: 2026-08-30
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

The implemented registry source layout is:

```text
src/payload/
  mod.rs                         public payload boundary
  registry.rs                    deterministic definition selection
  msfcs_storesmassdata_b.rs      first built-in identity and size definition
```

Every payload module must declare its identity, version, match criteria, byte
order, total size, and explicit field ranges. It decodes from a checked byte
slice and preserves raw values. The complete payload layer will return a typed
known-payload variant, an explicit ambiguous-match result, or an unknown
payload containing the original bytes.

Increment 4 implements the selection boundary before field decoding. Public
`PayloadContext` supplies the data-item ID and any available data-code or
configuration selectors. `PayloadRegistry` requires exact ID and byte length,
then applies every constraint declared by a candidate. A required constraint
does not match absent context. Results explicitly distinguish one matched
definition, no match, and multiple matches; every result retains the borrowed
raw application bytes and context. Definitions and ambiguity candidates remain
in stable registry order.

The built-in `msfcs_storesmassdata_b` entry uses Aero1394 definition version
`layout-v1`. That label versions the supplied layout within this project and is
not asserted to be the still-unconfirmed source-document revision. The entry
currently declares identity, exact size, and byte order. Its field decoder and
field-range declarations are the next increment, so a registry match must not
be presented as a completed application decode.

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
8. add the applicable `docs/L3.md` requirement and Rust test markers, then
   regenerate `docs/TRACE-MATRIX.md`.

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
| Aero1394 definition version | `layout-v1` | Project-local identifier for the supplied layout |
| Data-item ID | `0x00005D04` | **Confirmed** by summary and record bytes |
| Application size | 92 bytes | **Confirmed** by summary and record geometry |
| Byte order | Big-endian for currently observed multi-byte values | **Confirmed** for timestamp and float candidates |
| Data code | `vs_bus_b3` | **Confirmed** by recorder summary |
| Platform production rate | 60 Hz | **Confirmed for the supplied platform capture** by 876 intervals over 14.6 seconds |

The payload occupies record-relative bytes `0x18..0x73` in the observed record
family. Those record offsets are not part of the Rust payload definition; the
payload module sees byte offset zero at record offset `0x18`.

### Carrier relationship

Health Status, Heartbeat, the STOF offsets, VPC, and the logical ASM header are
defined in [`AS5643.md`](AS5643.md). Their BIE stored-data offsets and omitted
header reconstruction are defined in [`BIE-FORMAT.md`](BIE-FORMAT.md).
Capture provenance and recorder sampling context are retained in
[`BIE-EVIDENCE.md`](BIE-EVIDENCE.md). They are context for locating this
payload, not fields of the Rust application structure.

### Platform production rate

This payload is produced at 60 Hz by the supplied platform implementation. The
complete-file geometry provides the direct calculation:

```text
877 records -> 876 intervals
elapsed recorder time = 14.6 seconds
876 / 14.6 = 60 payload records/second
```

This is a property of this platform payload, not a FireSpy operating rate and
not a generic IEEE-1394 or AS5643 rule. Sampling of the supplied messages was
attempted at 80 Hz, and the surrounding FireSpy sampling context supports
80 Hz and 100 Hz configurations. Actual BIE timestamps, rather than an ideal
sample grid, remain authoritative. The observed gaps do not define a formal
scheduling relationship between the platform's 60 Hz producer and capture
sampling.

### Supplied field map

The following table records the payload definition supplied on 2026-08-30.
The pasted heading was `msfcs_storesmassdatab`; this document uses the
`msfcs_storesmassdata_b` spelling from the corrected recorder summary. Field
names, capitalization, data types, counts, word IDs, and offsets otherwise
preserve the supplied definition.

| Element | Data type | Element size (bits) | Element count | Word ID | MSB | LSB | Byte offset | Bit offset |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `TimeStamp` | Unsigned long long integer | `64` | `1` | `0-1` | `0` | `31` | `0` | `0` |
| `MessageValid` | Boolean type | `8` | `1` | `2` | `0` | `7` | `8` | `0` |
| `EOTS_Present` | Boolean type | `8` | `1` | `2` | `8` | `15` | `9` | `0` |
| `spare_byte` | Boolean type | `8` | `1` | `2` | `16` | `23` | `10` | `0` |
| `CM_Present` | Boolean type | `8` | `1` | `2` | `24` | `31` | `11` | `0` |
| `CurrentStoresMassData.Weight` | Float point type | `32` | `1` | `3` | `0` | `31` | `12` | `0` |
| `CurrentStoresMassData.Cg_FS` | Float point type | `32` | `1` | `4` | `0` | `31` | `16` | `0` |
| `CurrentStoresMassData.Cg_BL` | Float point type | `32` | `1` | `5` | `0` | `31` | `20` | `0` |
| `CurrentStoresMassData.Cg_WL` | Float point type | `32` | `1` | `6` | `0` | `31` | `24` | `0` |
| `CurrentStoresMassData.Ixx` | Float point type | `32` | `1` | `7` | `0` | `31` | `28` | `0` |
| `CurrentStoresMassData.Iyy` | Float point type | `32` | `1` | `8` | `0` | `31` | `32` | `0` |
| `CurrentStoresMassData.Izz` | Float point type | `32` | `1` | `9` | `0` | `31` | `36` | `0` |
| `CurrentStoresMassData.Ixy` | Float point type | `32` | `1` | `10` | `0` | `31` | `40` | `0` |
| `CurrentStoresMassData.Iyz` | Float point type | `32` | `1` | `11` | `0` | `31` | `44` | `0` |
| `CurrentStoresMassData.Ixz` | Float point type | `32` | `1` | `12` | `0` | `31` | `48` | `0` |
| `PostEJStoresMassData.Weight` | Float point type | `32` | `1` | `13` | `0` | `31` | `52` | `0` |
| `PostEJStoresMassData.Cg_FS` | Float point type | `32` | `1` | `14` | `0` | `31` | `56` | `0` |
| `PostEJStoresMassData.Cg_BL` | Float point type | `32` | `1` | `15` | `0` | `31` | `60` | `0` |
| `PostEJStoresMassData.Cg_WL` | Float point type | `32` | `1` | `16` | `0` | `31` | `64` | `0` |
| `PostEJStoresMassData.Ixx` | Float point type | `32` | `1` | `17` | `0` | `31` | `68` | `0` |
| `PostEJStoresMassData.Iyy` | Float point type | `32` | `1` | `18` | `0` | `31` | `72` | `0` |
| `PostEJStoresMassData.Izz` | Float point type | `32` | `1` | `19` | `0` | `31` | `76` | `0` |
| `PostEJStoresMassData.Ixy` | Float point type | `32` | `1` | `20` | `0` | `31` | `80` | `0` |
| `PostEJStoresMassData.Iyz` | Float point type | `32` | `1` | `21` | `0` | `31` | `84` | `0` |
| `PostEJStoresMassData.Ixz` | Float point type | `32` | `1` | `22` | `0` | `31` | `88` | `0` |

The table covers all 92 payload bytes without gaps or overlaps:

```text
TimeStamp                         8 bytes
four Boolean elements            4 bytes
CurrentStoresMassData            10 * 4 = 40 bytes
PostEJStoresMassData             10 * 4 = 40 bytes
total                            92 bytes
```

The layout confirms the field boundaries and wire data types. It does not yet
define the numeric Boolean encoding or polarity, the physical units and
reference conventions for the mass properties, or the source document and
revision. Until those semantics are supplied, an implementation should retain
the four Boolean elements as their raw `u8` values and expose the floating-point
values without assigning engineering units.

In the observed big-endian payload, the two adjacent words form one monotonic
value. For example:

```text
payload bytes 0..7 = 00 00 24 E6 14 F0 13 B3
raw value            0x000024E614F013B3
decimal              40,570,612,356,019
```

The field definition confirms an unsigned 64-bit value, and the field is
believed to contain system ticks. Its epoch remains **not confirmed**, so the
canonical model keeps the value losslessly in a `u64`-backed `SystemTicks`
newtype. It must not be converted to a calendar date.

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

Regression over the supplied pairs was also approximately 13.5983 GHz. The
provided nominal clock calculation is:

```text
base clock              = 106.25 MHz
tick multiplier         = 2^7 = 128
nominal tick rate       = 106.25e6 * 2^7
                        = 13,600,000,000 ticks/second (13.6 GHz)

one tick (LSB)          = 1 / (106.25e6 * 2^7) seconds
                        = 7.35294117647e-11 seconds
                        = 73.5294117647 picoseconds
```

The endpoint-derived rate of 13,597,508,871 ticks/s is approximately 0.0183%
below that nominal value, so the capture strongly corroborates the calculation.
The clock epoch remains unconfirmed. Any derived seconds value must carry the
selected rate and evidence state.
Parquet metadata is an appropriate place to record the rate and LSB duration;
the raw ticks remain canonical.

### Field-to-fixture correlation

The populated final fixture has four zero-valued Boolean bytes followed by the
twenty supplied floating-point fields. Decoding the words as big-endian
IEEE-754 `f32` values produces:

| Field | Hex | Value |
| --- | --- | ---: |
| `CurrentStoresMassData.Weight` | `45AF1829` | about `5603.02002` |
| `CurrentStoresMassData.Cg_FS` | `43F50000` | `490.0` |
| `CurrentStoresMassData.Cg_BL` | `3F800000` | `1.0` |
| `CurrentStoresMassData.Cg_WL` | `428C0000` | `70.0` |
| `CurrentStoresMassData.Ixx` | `47090A00` | `35082.0` |
| `CurrentStoresMassData.Iyy` | `4497C000` | `1214.0` |
| `CurrentStoresMassData.Izz` | `470D1300` | `36115.0` |
| `CurrentStoresMassData.Ixy` | `42140000` | `37.0` |
| `CurrentStoresMassData.Iyz` | `40A00000` | `5.0` |
| `CurrentStoresMassData.Ixz` | `42D20000` | `105.0` |
| `PostEJStoresMassData.Weight` | `44BA00A4` | about `1488.02002` |
| `PostEJStoresMassData.Cg_FS` | `43FB0000` | `502.0` |
| `PostEJStoresMassData.Cg_BL` | `40800000` | `4.0` |
| `PostEJStoresMassData.Cg_WL` | `42A20000` | `81.0` |
| `PostEJStoresMassData.Ixx` | `461D4400` | `10065.0` |
| `PostEJStoresMassData.Iyy` | `43780000` | `248.0` |
| `PostEJStoresMassData.Izz` | `4620A800` | `10282.0` |
| `PostEJStoresMassData.Ixy` | `41B80000` | `23.0` |
| `PostEJStoresMassData.Iyz` | `C1000000` | `-8.0` |
| `PostEJStoresMassData.Ixz` | `42200000` | `40.0` |

The sparse startup fixture populates only a few positions, including `450.0`
and `62.5`, while preserving the same 92-byte structure. This supports one
stable payload layout whose values become populated during initialization. The
supplied table now establishes the field names, types, and offsets; the capture
values independently corroborate the big-endian interpretation. Neither source
yet establishes units, scaling, Boolean encoding, or validity relationships.

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

- the source document, revision, and handling/redistribution constraints for
  the supplied `msfcs_storesmassdata_b` field table;
- the Boolean encoding, polarity, and valid values for `MessageValid`,
  `EOTS_Present`, `spare_byte`, and `CM_Present`, including whether
  `spare_byte` is intentionally Boolean rather than a reserved raw byte;
- the units, scaling, coordinate/reference conventions, and acronym expansions
  for the weight, center-of-gravity, and inertia fields;
- the epoch of the unsigned `TimeStamp` system-tick value;
- whether ID `0x00005D04` is reused across data codes or configurations;
- the corresponding `msfcs_storesmassdata_a` definition and its corrected
  capture evidence; and
- an independently produced expected decode, if available, for comparison with
  the capture-derived fixture values.
