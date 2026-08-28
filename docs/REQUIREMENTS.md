# Aero1394 requirements

- Status: Baseline for the next implementation stages
- Last updated: 2026-08-28
- Scope: observed BIE framing, built-in payload decoding, time, outputs, and verification

## How to read this document

These requirements turn the durable conclusions from the supplied development
conversation into testable repository commitments. They do not elevate an
unverified protocol interpretation into fact. `docs/BIE-FORMAT.md` remains the
evidence ledger, while ADRs record why design choices were made.

Requirement states are:

| State | Meaning |
| --- | --- |
| Active | Required for the next applicable implementation stage |
| Evidence-limited | Required only for the observed BIE variant or record family until more captures generalize it |
| Deferred | Desired behavior whose prerequisite definition or implementation stage has not arrived |

Verification methods are inspection (`I`), automated test (`T`), and
comparison with independent evidence (`E`).

## BIE container and record requirements

| ID | State | Requirement | Verify |
| --- | --- | --- | --- |
| BIE-001 | Evidence-limited | The decoder shall parse the observed record header as four explicit big-endian `u32` words: data-item ID, recorder seconds, recorder microseconds, and raw status/length. | T, E |
| BIE-002 | Evidence-limited | The decoder shall derive stored-data length from `status_and_length & 0x0000_FFFF` for the supported observed variant and use checked arithmetic for `16 + length`. | T |
| BIE-003 | Active | The decoder shall use the encoded length rather than assuming every record is 132 bytes. | T, I |
| BIE-004 | Evidence-limited | A zero data-item word where the next record begins shall terminate the supported observed file variant; bytes after it shall be reported rather than silently ignored. | T |
| BIE-005 | Active | Physical EOF before a complete header or declared stored-data body shall be reported as truncation with needed/available sizes and absolute offset. | T |
| BIE-006 | Active | The decoder shall preserve the complete raw status/length word even when the low length bits are exposed separately and upper status semantics are unknown. | T, I |
| BIE-007 | Active | Parsed records shall retain their absolute file offset and raw stored-data bytes. | T, I |
| BIE-008 | Active | Unknown data-item IDs and unsupported stored-data layouts shall remain inspectable and shall not be mislabeled as corrupt solely because no typed payload exists. | T |
| BIE-009 | Evidence-limited | For data item `0x00005D04`, 92 application bytes shall be located within the 116-byte stored region without assigning unconfirmed names to the two preceding words. | T, E |
| BIE-010 | Active | Parsing, validation, policy, and recovery shall remain separate as required by ADR-0007. | I, T |

## Time requirements

| ID | State | Requirement | Verify |
| --- | --- | --- | --- |
| TIM-001 | Evidence-limited | The outer recorder time shall decode as unsigned big-endian Unix seconds plus unsigned big-endian microseconds for the supported observed variant. | T, E |
| TIM-002 | Active | Raw seconds and microseconds shall remain accessible after conversion to a calendar instant. | T, I |
| TIM-003 | Active | The implementation shall reject or attach a validation finding to microsecond values outside `0..999_999`; it shall not normalize malformed wire values silently. | T |
| TIM-004 | Active | Timestamp arithmetic shall widen raw `u32` seconds before addition or multiplication and shall not impose a signed 2038 limit. | T, I |
| TIM-005 | Active | Timezone selection and text format shall be independent presentation options; ISO-8601 UTC shall be the default. | T |
| TIM-006 | Deferred | Presentation shall support ISO-8601, year/day-of-year, recorder-summary `DD:HH:MM:SS.ffffff`, and numeric Unix forms. | T |
| TIM-007 | Active | Payload system ticks and BIE recorder time shall use distinct names and types. Payload ticks shall not be converted to calendar time without a confirmed epoch. | T, I |
| TIM-008 | Deferred | A derived payload elapsed-seconds value shall preserve the raw `u64` ticks and identify the selected tick rate and its evidence state. | T |

## Protocol-envelope and integrity requirements

| ID | State | Requirement | Verify |
| --- | --- | --- | --- |
| PRO-001 | Active | The BIE layer shall return stored bytes without requiring IEEE-1394, AS5643, or application decoding. | T, I |
| PRO-002 | Evidence-limited | The observed `0x00005D04` envelope shall preserve two neutral pre-payload words, three STOF candidates, and the final VPC candidate as raw `u32be` values. | T, E |
| PRO-003 | Active | Unconfirmed health, heartbeat, STOF, and missing-ASM-header semantics shall be labeled provisional in APIs and output. | I |
| PRO-004 | Deferred | VPC validation shall expose `Valid`, `Invalid`, `NotPresent`, and `NotChecked`-equivalent outcomes and retain the stored VPC and calculation inputs. | T |
| PRO-005 | Evidence-limited | Golden tests for `0x00005D04` shall preserve the observed `0x00005D60` visible-word VPC residual as evidence until normative ASM-header coverage is established. | T, E |

## Built-in payload requirements

| ID | State | Requirement | Verify |
| --- | --- | --- | --- |
| PAY-001 | Active | Application decoding shall be implemented in a separate `payload` layer; BIE and protocol modules shall treat application bytes as opaque. | I, T |
| PAY-002 | Active | Supported payload definitions shall be Rust source modules compiled into Aero1394; runtime YAML profiles shall not be required. | I |
| PAY-003 | Active | Each definition shall declare a stable name, definition version, exact size, byte order, explicit field ranges, and match criteria. | I, T |
| PAY-004 | Active | Registry selection shall use data-item ID and payload size at minimum and shall allow data-code/configuration constraints when available. ID alone shall not be treated as globally unique. | T, I |
| PAY-005 | Active | Registry selection shall be deterministic and distinguish one match, no match, and ambiguous matches. | T |
| PAY-006 | Active | An unknown payload shall retain its data-item ID, context, size, and raw bytes. | T |
| PAY-007 | Active | Payload fields shall be decoded with checked slice access and explicit byte order; native layout casts, packed-struct reads, and unsafe code are prohibited. | I, T |
| PAY-008 | Active | Declared fields that overlap unexpectedly or extend beyond payload size shall fail definition validation; documented gaps shall remain visible. | T |
| PAY-009 | Deferred | Raw physical values shall remain available when scaling, offsets, units, enumeration labels, bitfields, arrays, or validity rules produce engineering values. | T |
| PAY-010 | Deferred | `msfcs_storesmassdata_b` support shall not be declared complete until the supplied 92-byte structure is documented and golden-tested field by field. | I, T, E |
| PAY-011 | Active | Adding a payload shall require a payload-document update, one Rust module, one registry entry, sanitized fixtures, boundary/byte-order tests, and a traceability update. | I |
| PAY-012 | Evidence-limited | The first payload definition shall record 60 Hz as a platform-specific production rate and shall not present it as the FireSpy, IEEE-1394, or AS5643 operating cadence. | I, E |

## Canonical data and output requirements

| ID | State | Requirement | Verify |
| --- | --- | --- | --- |
| OUT-001 | Deferred | One canonical typed Rust record shall feed CLI text, CSV, Parquet, and Python adapters rather than using one adapter's representation as the internal model. | I, T |
| OUT-002 | Active | Raw wire values and validation findings shall not be replaced by formatted or derived values. | T, I |
| OUT-003 | Deferred | CSV shall have an explicitly versioned, golden-tested header and stable column order. IDs and diagnostic words may use fixed-width uppercase hexadecimal text. | T |
| OUT-004 | Deferred | Parquet shall use non-lossy portable types, preserve on-wire `f32` as physical float, and represent recorder time as a UTC microsecond timestamp. | T, I |
| OUT-005 | Deferred | An inferred payload tick rate shall be stored once as output metadata with an evidence state rather than repeated as an unexplained row value. | T |
| OUT-006 | Active | Human-oriented formatting changes shall not be treated as machine-schema compatibility. | I |

## Fixture and verification requirements

| ID | State | Requirement | Verify |
| --- | --- | --- | --- |
| TST-001 | Active | Sanitized test data shall include four consecutive sparse startup records, four consecutive populated end records with their original zero word, and a synthetic zero-word-only input. | I |
| TST-002 | Active | Fixture metadata shall state byte count, expected ID, timestamps, status/length, payload boundary, trailer words, and whether records are consecutive. | I |
| TST-003 | Active | Tests shall verify fixture hex decoding itself before using fixture bytes as parser evidence. Invalid digits and wrong byte counts shall fail. | T |
| TST-004 | Active | Parser tests shall cover valid termination, missing termination, truncation in every fixed header word and in stored data, zero/oversized length, and bytes after the terminator. | T |
| TST-005 | Active | Tests shall cover unknown payload fallback and ambiguous registry selection before the registry is called stable. | T |
| TST-006 | Active | Any test derived from a real or simulated message shall preserve its provenance and handling authorization without committing restricted source captures. | I |

## Traceability and delivery gates

| Area | Decision/evidence | Current verification artifact | Implementation state |
| --- | --- | --- | --- |
| BIE framing | ADR-0005, ADR-0006, `BIE-FORMAT.md` | `tests/fixtures/bie/*.hex` | Parser pending |
| IEEE-1394 boundary | ADR-0002, `IEEE1394.md` | Comparison constraints; packet fixtures pending | Evidence only |
| AS5643-derived envelope | ADR-0002, `AS5643.md` | VPC-residual checks in `tests/bie_fixtures.rs` | Evidence only |
| Error policy | ADR-0007 | Existing forensic boundary tests; BIE corrupt cases pending | Partial |
| Built-in payloads | ADR-0012, `PAYLOADS.md` | BIE messages preserve the first payload; field fixtures pending full definition | Design accepted |
| Canonical outputs | ADR-0013, `OUTPUTS.md` | Provisional CSV row and Parquet mapping | Proposed |
| CLI/Python parity | ADR-0008 | Existing CLI/library hexdump tests | Partial |
| Evidence staging | ADR-0009, ADR-0011 | Documentation and fixture set | Active |

The next implementation slice may claim Stage 2 BIE framing only after the
active BIE and fixture requirements have automated tests. Built-in payload
support begins when the remaining payload field definition is supplied; the
provisional float interpretation alone is not an implementation contract.
