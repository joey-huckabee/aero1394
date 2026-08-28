# BIE binary format research

- Status: Discovery; representative simulation excerpts have been correlated with recorder summary metadata
- Last updated: 2026-08-28
- Applies to: the `.bie` aerospace IEEE-1394 recordings expected by Aero1394

## Purpose

This document is the evidence ledger for the BIE capture container. It records
what is known, what has only been inferred, what has been ruled out, and what
must be learned before implementing a production parser.

The verified subset is described at byte level below. Protocol names remain
provisional where the observed layout has not yet been checked against an
authoritative BIE or AS5643 definition.

Evidence labels used below are:

| Label | Meaning |
| --- | --- |
| Confirmed | Supported by authoritative documentation or independently reproducible evidence |
| Inferred | A conclusion drawn from confirmed behavior, but not a documented byte layout |
| Hypothesis | Plausible and worth testing against a sample |
| Unknown | No adequate evidence yet |

## Critical research finding

No publicly accessible DAP Technology document located during this research
defines a `.bie` FireSpy recording format or even associates the `.bie`
extension with FireSpy.

DAP's official operation manual instead identifies the native FireSpy Recorder
file as `.fsr`. The same manual documents customer-facing exports with the
extensions `.fsp`, `.bin`, `.rgn`, `.txt`, `.csv`, `.hex`, and `.qdl`. Its file
format chapter defines `.fsp` and `.rgn` binary layouts, but not the native
`.fsr` layout and not `.bie` ([DAP-OM]).

This leaves a provenance conflict even though a repeatable record structure is
now visible in the supplied simulation excerpts:

| Claim | Status | Evidence and implication |
| --- | --- | --- |
| The target files use the `.bie` extension | Confirmed by supplied summary metadata | The complete capture is not committed; sanitized hex fixtures preserve selected records |
| The target files were produced by `BIE_LINUX` hardware/software | Confirmed by supplied summary metadata | This does not establish that BIE is a FireSpy-native format |
| The target files were produced by FireSpy | Hypothesis | DAP documentation uses `.fsr` for FireSpy recordings |
| `.bie` is a legacy or internal DAP format | Unknown | No public DAP reference was found |
| `.bie` is a renamed `.fsr`, `.rgn`, `.fsp`, or Chapter 10 file | Hypothesis | Must be tested by signature and contents, never by extension alone |
| `.bie` contains an AS5643-derived stored region | Inferred | Length geometry, STOF-like trailer values, and VPC behavior agree across supplied records |

## Supplied capture evidence

The current byte map comes from excerpts of one simulation recording and its
corrected summary metadata. The relevant summary values are:

```text
Data File: Startup.draw.data.1394.vs_bus_b3.unused.bie
Data Type: IEEE 1394
Data Code: vs_bus_b3
Hardware Type: BIE_LINUX
Recorder Buffer Mode: Direct to File (local disk)
Data Set Count: 1
Recording Timetags:
  start=31:08:01:59.063844
  stop=31:08:05:46.335672
```

The summary reports a recording date of Wednesday, July 31, 2024. The complete
capture is not in the repository, so its digest, recorder version, and handling
classification remain unknown. The supplied end-of-file offsets imply this
geometry:

```text
877 records * 132 bytes = 115,764 bytes
zero word at EOF         =       4 bytes
total                    = 115,768 bytes (0x1C438)
```

Selected sanitized records are retained as machine-readable hexadecimal test
inputs under [`tests/fixtures/bie`](../tests/fixtures/bie/README.md). They are
evidence for the observed record family, not permission to generalize every
`.bie` producer or record type.

The negative search result does not prove that no BIE specification exists.
It may be available only in a serial-number-gated download, SDK, support
document, contract data package, older product release, or another vendor's
documentation. DAP offers a demo FireDiagnostics download through a request
form and product downloads through a FireSpy serial-number page
([DAP-DOWNLOAD]).

## DAP FireSpy recording findings

### Native recording name and extension

The 2017 DAP operation manual says that Recorder `Open` loads an analyzer
recording with the `.fsr` extension and that `Save As` writes the same kind of
file. It shows `SBP2example.fsr` as an example and documents the standalone
Recorder accepting or creating `filename.fsr` ([DAP-OM], pp. 62, 65, 74-75).

The manual does not give the `.fsr` byte layout. Chapter 22 says it documents
the formats intended for customer use, then covers `.hex`, `.qdl`, `.fsp`,
`.rgn`, signal-definition CSV, and Mil1394 XML. The omission of `.fsr` means
that chapter cannot be used as a native recording-file ICD ([DAP-OM],
pp. 359-367).

### Behavior the native recording can represent

The following are behavior-level findings. They are useful parser requirements
if the target later proves to be `.fsr` or a related format, but they do not
establish field offsets.

| Finding | Evidence | Status |
| --- | --- | --- |
| Recordings contain packets and non-packet events | Recorder views and export selection include packets, bus resets, and events | Confirmed |
| Prefix-only and acknowledge-only observations exist | Recorder export offers explicit options for these cases | Confirmed |
| Packets can carry error information | Text export can include erroneous packets and associated errors | Confirmed |
| Time is captured | Export can include the packet/bus-reset start value of a 49.152 MHz internal counter | Confirmed |
| UTC timestamps can be present in recording files | DAP fixed saving files containing many UTC timestamps in FireDiagnostics 7.0.18 | Confirmed |
| Timestamp corruption is a known condition | FireDiagnostics 7.0.18 added an option to ignore timestamp errors while loading a corrupted recording | Confirmed |
| Multi-node/multi-bus provenance matters | Recorder supports synchronized devices and node A/B/C data | Confirmed |
| Protocol-analysis settings can be embedded | The manual says settings are stored in the recording file | Confirmed |
| Topology may exist before the first in-capture bus reset | Later releases store pre-recording SelfIDs and display topology before the first reset | Confirmed |
| Older recording variants exist | FireDiagnostics 7.0.22 fixed loading older recordings containing pre-recording SelfIDs | Confirmed |
| Capture/download defects can affect recorded data | DAP release notes describe corruption and prefix-only/event download defects fixed in 2024-2025 | Confirmed |

The version-history findings come from DAP's official FireDiagnostics 7.0
release notes ([DAP-7]).

### API availability does not imply a public file layout

DAP publishes a Windows DLL API with C/C++ headers and LabVIEW wrappers. The
official API page lists Recorder control and data retrieval, plus IEEE-1394 and
AS5643 functions ([DAP-API]). This may provide a supported extraction route, but
the public page does not describe `.fsr` or `.bie` bytes.

FireDiagnostics 6.0 release notes mention a LabVIEW example that creates a
FireSpy recording file and fixes related to saving `.fsr` files ([DAP-6]). The
installer, headers, examples, and any fuller API reference are therefore
high-value artifacts to request or inspect. They were not available in the
current workspace, and the public download path requires a serial number or a
demo-download request.

## Officially documented DAP binary exports

These are not BIE definitions. They are useful for identification, as possible
intermediate formats, and as independent evidence when correlated with a target
capture.

### FireSpy packet file (`.fsp`)

The operation manual completely describes version 1.0.0 of this format:

- the file is an array of 32-bit little-endian values;
- the first word is file ID `0x705346AE`;
- the second word is version `0xLLMMHH00`;
- version 1.0.0 is `0x00000100`;
- each block begins with one 32-bit header;
- the upper 12 header bits are the block ID;
- the lower 20 header bits are the unpadded byte length;
- block data is padded with zero bytes to a 4-byte boundary; and
- block ID 0 contains packet bytes and is the only defined block in that
  manual revision ([DAP-OM], pp. 360-361).

The version 1.0.0 prefix should therefore be these eight on-disk bytes:

```text
AE 46 53 70  00 01 00 00
```

Logical layout:

```text
u32le file_id = 0x705346AE
u32le version = 0x00000100

repeat until EOF:
    u32le block_header
        block_id    = block_header >> 20
        data_length = block_header & 0x000F_FFFF
    u8 data[data_length]
    u8 zero_padding[align4(data_length) - data_length]
```

An `.fsp` export retains packet bytes but the documented block has no capture
timestamp, bus/node source, packet error metadata, bus-reset event, or
acknowledge association. It is not a substitute for the native capture when
timing and network analysis matter.

### Recorder Regeneration file (`.rgn`)

DAP describes `.rgn` as a binary export used to regenerate recorded stream
traffic while preserving relative transmit intervals. It is the closest public
DAP specification to the logical records Aero1394 expects ([DAP-OM],
pp. 66-67, 361-363).

The documented file-level fields are:

- a 32-bit file ID `0xAE52476E`;
- a 3-bit time-format value:
  - 0: no per-item time field;
  - 1: 64-bit absolute time;
  - 2: 32-bit frame/cycle-offset time;
- an 8-bit file version, documented as version 1;
- a list of field definitions terminated by field type 0; and
- a list of items terminated by item type 0.

The manual excerpt does not explicitly state `.rgn` on-disk byte order. It must
not be inferred from the explicit little-endian rule for `.fsp`.

Documented item kinds are:

| ID | Item | Selected contents |
| --- | --- | --- |
| 0 | End marker | Terminates the item list |
| 1 | Start of frame/cycle | FireSpy node, optional fields, previous frame length in microseconds |
| 2 | Unformatted packet | FireSpy node, speed code, optional fields, packet size in quadlets, complete raw packet including CRCs |
| 3 | Stream packet | FireSpy node, speed, optional fields, header/data flags and sizes, optional time, header, header CRC, data, data CRC |

Node values 0, 1, and 2 represent nodes A, B, and C. Speed codes 0 through 4
represent unknown, 100, 200, 400, and 800 Mbit/s. For stream packets, flag bit
`0x1` in the header or data flags indicates the corresponding CRC error.

Questions that the manual does not settle for Aero1394 include:

- byte order;
- behavior for unknown field or item IDs;
- whether every error and prefix-only observation can be represented;
- whether asynchronous, PHY, acknowledge, and bus-reset events survive export;
- whether an absolute time has an epoch and encoding shared with the native
  recording; and
- whether later FireDiagnostics releases extended version 1.

If the original vendor application can open the target capture, exporting the
same interval as `.rgn`, `.fsp`, CSV, and text would provide exceptionally
useful correlation data. `.rgn` should still be treated as a separate input
adapter, not relabeled as BIE.

### Raw binary (`.bin`)

DAP says raw binary export writes binary data on quadlet boundaries. The export
dialog can optionally emit data only, omitting header and CRC ([DAP-OM],
pp. 66-67). The manual does not define a file header or record-boundary table for
this export. It is useful for packet-byte comparison but is a poor source for
capture time, event, error, and provenance semantics.

### Textual exports

The text and CSV exports can include timestamp, packet attributes, data, and
errors. Hex and quadlet files have documented textual encodings. These exports
are not lossless capture containers, but they are valuable as independently
decoded expected results for a matching binary interval ([DAP-OM], pp. 66-67,
359-360).

## AS5643 ICD and network-profile findings

An AS5643 ICD or slash sheet is not a BIE file definition. It becomes relevant
only after the container and IEEE-1394 packet have been decoded.

The current SAE catalog identifies AS5643B, reaffirmed 2025-04-28, as the base
standard. SAE explicitly says it is not stand-alone: vehicle-specific details
belong in a network-profile slash sheet and physical-layer slash sheet
([SAE-AS5643B]).

DAP's manual documents a configurable `Mil1394Settings.xml` and says an example
is installed with FireDiagnostics. It also tells customers to contact DAP
support about an XML file for a specific program, naming JSF as an example.
([DAP-OM], pp. 259-261, 329-330). DAP's older 4.3 release notes say separate SAE
and JSF Mil1394 XML examples were shipped ([DAP-43]).

The documented XML profile can define at least:

- pre-assigned channels and device names;
- channel plus Message ID selection;
- payload signal quadlet and bit offsets;
- signed, unsigned, floating-point, hexadecimal, and enumerated values;
- factor, offset, range, and units;
- ASM header and trailer field interpretations; and
- STOF packet interpretation.

No public program ICD or downloadable DAP Mil1394 XML example was located in
this research. The installed FireDiagnostics examples, if available from the
recording workstation or vendor package, are the next legitimate source. A
program-specific ICD may be controlled, proprietary, CUI, or ITAR-restricted
and must not be committed or transmitted without authorization.

## Alternative-format identification checks

The `.bie` token is not unique. ITU-T T.82 also calls the top-level JBIG image
data structure a **bi-level image entity (BIE)** ([ITU-T82]). This does not suggest
that the expected aerospace capture is an image; it means the extension or term
alone cannot identify the format.

Use content to test and rule out these alternatives:

| Candidate | Identification evidence | Interpretation |
| --- | --- | --- |
| DAP `.fsp` v1.0.0 | Starts `AE 46 53 70 00 01 00 00` | Documented packet export, not native recording |
| DAP `.rgn` v1 | Logical file ID `0xAE52476E`; byte order still to verify | Documented regeneration export |
| DAP `.fsr` | No public signature located | Native FireSpy recording according to DAP |
| IRIG 106 Chapter 10 | Standard Chapter 10 synchronization/header followed by data types such as `0x58` or `0x59` | Standardized recorder container, not proof of BIE |
| JBIG T.82 BIE | A T.82-conforming bi-level image header and data stream | Unrelated image meaning of BIE |

IRIG 106-11 defines `0x58` as IEEE-1394 transaction data (Format 0) and `0x59`
as IEEE-1394 physical-layer data (Format 1) ([IRIG106-11]). A renamed Chapter 10
file is one hypothesis to test, but no evidence currently connects it to the
target files.

## Current BIE byte map

### File and record grammar

The observed file is a sequence of big-endian, length-delimited records ending
in a zero word. The zero word is a **strong inference** for an EOF sentinel: it
occurs exactly where another nonzero data-item ID would begin, but confirmation
from another complete file or producer documentation is still required.

```text
file := record* zero_word

record :=
    data_item_id       u32be
    recorder_seconds   u32be
    recorder_useconds  u32be
    status_and_length  u32be
    stored_data        u8[data_length]

zero_word   := 00 00 00 00
data_length := status_and_length & 0x0000_FFFF
record_size := 16 + data_length
```

The decoder must not make 132 bytes a universal record size. That size follows
only for the supplied record family because its stored-data length is `0x0074`,
or 116 bytes.

### Observed record header

| Offset | Width | Interpretation | Evidence |
| ---: | ---: | --- | --- |
| `0x00` | 4 | Nonzero data-item ID | **Confirmed** by exact match with summary metadata |
| `0x04` | 4 | Unsigned Unix seconds, big-endian | **Confirmed** by recording date and time correlation |
| `0x08` | 4 | Microseconds within the second, big-endian | **Confirmed** by time correlation and 12.5 ms deltas |
| `0x0C` | 4 | Raw status/length word | **Confirmed** structurally; upper-bit semantics unknown |
| `0x10` | N | Stored data, where `N` is the low 16-bit length | **Confirmed** for supplied records |

The observed status/length words are `0x00000074` and `0x40000074`.
`0x0074` consistently selects 116 following bytes and chains to the next
record. The meaning of `0x40000000` is unknown. Its similarity to an IRIG 106
IEEE-1394 Format 1 status/length word remains a comparison lead, not proof that
the BIE record is a Chapter 10 intra-packet.

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
`2106-02-07T06:28:15Z`; interpreting it as signed would instead introduce the
unwanted 2038 limit. Raw seconds and microseconds remain part of the decoded
model even when a calendar representation is available.

### Stored-data boundary

At the BIE layer, the declared stored-data region is opaque. Its internal
protocol envelope, application identity, field layout, integrity behavior, and
message-specific timing do not belong to the container format. The BIE parser
must preserve the exact bytes and length without importing a built-in payload
definition.

The currently observed downstream-protocol evidence is kept in
[`AS5643.md`](AS5643.md), and application definitions are kept in
[`PAYLOADS.md`](PAYLOADS.md). Both are isolated from the generic BIE grammar.

### Remaining unknowns

- whether the zero word is required by every producer and whether bytes may
  legally follow it;
- whether `data_length` always occupies the low 16 bits and what every upper
  status bit means;
- other data-item sizes, IDs, record kinds, and empty-recording behavior;
- recorder/version metadata, bus/node/channel provenance, and file indexing;
- which downstream protocol or application definition applies to stored data;
- behavior for corrupt, prefix-only, acknowledge-only, reset, or event records;
  and
- whether the format varies across `BIE_LINUX` versions or hardware.

## Additional capture evidence required

For the next complete sample, preserve the original file and record:

- original filename and extension;
- producing application, recorder model, firmware, and software version;
- capture settings, bus count, and whether Mil1394 mode was enabled;
- file size, modification time, and SHA-256 hash;
- redistribution, classification, CUI, ITAR, and proprietary-data constraints;
- first and last 256 bytes;
- printable strings and known signatures;
- file-wide byte-frequency/entropy and repeated-pattern results; and
- whether FireDiagnostics Recorder can open the file without renaming it.

For the same short capture interval, obtain as many independent views as the
vendor application permits:

- native save;
- `.rgn` regeneration export;
- `.fsp` packet export;
- text and CSV export with time, data, and errors;
- raw `.bin` export with and without the data-only option; and
- screenshots or property reports showing packet count, time range, bus/node,
  software version, and protocol mode.

Controlled captures are particularly valuable: one known stream packet, two
different payload lengths, a bus reset, an acknowledge-only event, a
prefix-only observation, an intentionally bad CRC, a STOF plus one ASM, and
traffic on more than one analyzer node.

## Questions for DAP Technology or the actual recorder vendor

1. Does any released or legacy DAP product create or read files with a `.bie`
   extension? If so, which product and versions?
2. Is `.bie` a native capture, temporary recorder, export, cache, index, or
   customer-specific integration format?
3. Is it related to `.fsr`, `.fsp`, `.rgn`, IRIG 106, or another container?
4. Is a file-format ICD, SDK header, C/C++ reader API, LabVIEW example, or
   redistributable reference implementation available?
5. What are the file signature, versioning rules, byte order, length units,
   alignment, timestamp domains, event types, error flags, and checksums?
6. How are multiple buses/nodes, bus resets, SelfIDs, prefix-only observations,
   acknowledgements, PHY events, packet errors, UTC time, and protocol settings
   stored?
7. Which export retains the most information while having a documented and
   redistributable format?
8. Are there separate format versions by FireSpy generation, firmware, normal
   versus Mil1394 mode, or FireDiagnostics release?
9. Can the relevant Mil1394 settings XML or program network profile be provided,
   and under what handling restrictions?

DAP lists support contacts and warns customers not to send CUI or ITAR data
without advance approval and use of its secure upload process ([DAP-SUPPORT];
[DAP-CONTACT]). Do not attach a real capture to an ordinary support email.

## Parser constraints while provenance and variants remain unresolved

- Do not identify a format solely from `.bie` or any other extension.
- Do not use `.fsr`, `.fsp`, or `.rgn` structures as BIE structures without a
  signature match and independent correlation.
- Implement only the verified record subset and keep provisional protocol
  fields under neutral names.
- Do not expose guessed BIE or AS5643 semantics in the stable Rust or Python
  API.
- Preserve raw bytes, absolute offsets, and the evidence for each detected
  structure.
- Report ambiguous matches rather than selecting the first plausible format.
- Keep BIE, FSR, FSP, RGN, Chapter 10, IEEE-1394, and AS5643 as distinct layers
  and format identities.
- Refuse unsupported versions safely and retain enough context for forensic
  output.

The initial format-neutral Rust command can capture these observations without
assigning field meanings:

```text
cargo run --release -- hexdump path/to/capture.bie --offset 0 --length 256
```

See [Reverse-engineering BIE captures](REVERSE-ENGINEERING.md) for bounded
range selection, provenance recording, and handling guidance.

If the target files prove to be native FireSpy `.fsr` files with a nonstandard
extension, update this document with the identifying evidence and decide
whether the adapter should be named `fsr` rather than encoding the mistaken
name in the API.

## Research log

### 2026-08-28

- Correlated supplied simulation-record excerpts with corrected summary
  metadata for `vs_bus_b3`.
- Established the 16-byte big-endian record header and 116-byte stored-data
  boundary for the observed record family.
- Correlated recorder seconds/microseconds with the July 31, 2024 local
  recording window.
- Recorded the exact 877-record-plus-zero-word file geometry and retained
  sanitized startup and populated records as test fixtures.
- Separated IEEE-1394 comparison constraints, AS5643-derived evidence, and
  application definitions into `docs/IEEE1394.md`, `docs/AS5643.md`, and
  `docs/PAYLOADS.md` respectively.

### 2026-08-27

- Searched DAP's public site, current FireSpy pages, operation manual, API page,
  release histories, download page, support page, and product documentation for
  `BIE`, `.bie`, recorder formats, and recording-file APIs.
- Downloaded the current official operation-manual PDF from DAP and searched
  its extracted text. It contains `.fsr`, `.fsp`, `.rgn`, and the other exports
  documented above, but no `BIE` or `.bie` occurrence.
- Confirmed that the public software-download route is serial-number gated and
  that the demo package requires a request form.
- Checked the current machine for a standard DAP Technology installation and
  `Mil1394Settings.xml`; none was found in the usual installation locations.
- Checked authoritative SAE, IRIG 106, and ITU sources for downstream protocol,
  alternative-container, and name-collision context.
- Did not find a public BIE recorder ICD, a DAP BIE specification, a public
  program network ICD, or a public parser implementation attributable to DAP.

## Sources

- **DAP-OM** — [1394 Analyzer Operation Manual, DAP Technology, dated
  2017-09-01](https://www.daptechnology.com/fileadmin/manuals/OperationManual.pdf)
- **DAP-API** — [FireSuite API product page](https://www.daptechnology.com/products/software/firediagnostics-suite/firesuite-api)
- **DAP-DOWNLOAD** — [FireDiagnostics software download page](https://www.daptechnology.com/support/downloads)
- **DAP-7** — [FireDiagnostics Suite 7.0 release history](https://www.daptechnology.com/products/software/firediagnostics-suite/versions/7-0)
- **DAP-6** — [FireDiagnostics Suite 6.0 release history](https://www.daptechnology.com/products/software/firediagnostics-suite/versions/6-0)
- **DAP-43** — [FireDiagnostics Suite 4.3 release history](https://www.daptechnology.com/products/software/firediagnostics-suite/versions/4-3)
- **DAP-SUPPORT** — [DAP Technology support page](https://www.daptechnology.com/support/)
- **DAP-CONTACT** — [DAP Technology contact and controlled-data notice](https://www.daptechnology.com/contact)
- **SAE-AS5643B** — [SAE AS5643B, reaffirmed
  2025-04-28](https://saemobilus.sae.org/standards/as5643b-ieee-1394b-interface-requirements-military-aerospace-vehicle-applications)
- **IRIG106-11** — [IRIG 106-11 Chapter 10](https://www.irig106.org/docs/106-11/chapter10.pdf)
- **ITU-T82** — [ITU-T Recommendation T.82](https://www.itu.int/rec/T-REC-T.82)

[DAP-OM]: https://www.daptechnology.com/fileadmin/manuals/OperationManual.pdf
[DAP-API]: https://www.daptechnology.com/products/software/firediagnostics-suite/firesuite-api
[DAP-DOWNLOAD]: https://www.daptechnology.com/support/downloads
[DAP-7]: https://www.daptechnology.com/products/software/firediagnostics-suite/versions/7-0
[DAP-6]: https://www.daptechnology.com/products/software/firediagnostics-suite/versions/6-0
[DAP-43]: https://www.daptechnology.com/products/software/firediagnostics-suite/versions/4-3
[DAP-SUPPORT]: https://www.daptechnology.com/support/
[DAP-CONTACT]: https://www.daptechnology.com/contact
[SAE-AS5643B]: https://saemobilus.sae.org/standards/as5643b-ieee-1394b-interface-requirements-military-aerospace-vehicle-applications
[IRIG106-11]: https://www.irig106.org/docs/106-11/chapter10.pdf
[ITU-T82]: https://www.itu.int/rec/T-REC-T.82
