# `msfcs_storesmassdata_b` payload fixtures

- Status: Sanitized payload-only derivatives of authorized BIE fixtures
- Payload definition: `msfcs_storesmassdata_b`
- Aero1394 definition version: `layout-v1`
- Data-item ID context: `0x00005D04`
- Payload size: 92 bytes each
- Byte order: Big-endian

These fixtures contain only the 92 application bytes defined in
`docs/PAYLOADS.md`. They are derived without modification from captures the
project maintainer supplied and authorized for use as test evidence. The source
BIE excerpts and their provenance are retained under `tests/fixtures/bie/`.

`populated.hex` is the application region from record 3, the last record in
`end-four-records.hex`. Its BIE record starts at byte `0x18C`, and its payload
occupies record-relative bytes `0x18..0x73` (fixture bytes `0x1A4..0x1FF`).

`sparse-startup.hex` is the application region from record 0 in
`startup-four-records.hex`. Its payload occupies fixture bytes `0x18..0x73`.

The expected raw distinctions are intentional:

| Fixture | `TimeStamp` bits | Four Boolean-designated bytes |
| --- | --- | --- |
| `populated.hex` | `0x000024E614F013B3` | `01 00 00 00` |
| `sparse-startup.hex` | `0x000024B7DC01E3E3` | `00 01 00 00` |

No Boolean polarity, engineering units, validity relationship, or timestamp
epoch is inferred by these fixtures. Golden tests compare all integer/byte
values and all twenty exact IEEE-754 bit patterns before exposing `f32` values.
