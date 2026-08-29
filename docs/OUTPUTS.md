# Decoded output schemas

- Status: Provisional design; no decoder or stable machine schema implemented
- Last updated: 2026-08-29

## Boundary

Output adapters consume one canonical decoded Rust record as proposed by
[ADR-0013](adr/0013-separate-canonical-data-from-output-presentation.md). CSV,
Parquet, CLI text, and Python objects do not define the parser's internal data
model. Raw words and validation outcomes remain available when presentation
adds names, dates, engineering values, or formatting.

The schema below preserves the current design discussion for
`msfcs_storesmassdata_b`. It is provisional until the complete payload
definition is supplied and the machine schema is versioned and golden-tested.

## Time presentation

Recorder time defaults to ISO-8601 UTC. Format and timezone are independent
choices. Intended formats include:

| Format | UTC example |
| --- | --- |
| ISO | `2024-07-31T13:05:46.333129Z` |
| Day of year | `2024:213:13:05:46.333129` |
| Recorder summary | `31:08:05:46.333129` |
| Unix | `1722431146.333129` |

The recorder-summary form is interpreted as `DD:HH:MM:SS.ffffff` using the
separately known recording date. It is ambiguous without that date. Payload
system ticks are a different time domain and are never rendered as a calendar
date without a confirmed epoch.

## Provisional CSV

One decoded BIE record maps to one row. The proposed header is:

```csv
TIME_STAMP,DELTA,RECORD_INDEX,DATA_ITEM_ID,DATA_ITEM_NAME,RECORD_STATUS,DATA_LENGTH,AS5643_PROFILE,AS5643_ASSUMPTION_DEPENDENT,HEALTH_STATUS,HEARTBEAT,SYSTEM_TICKS,SYSTEM_TIME_SECONDS,PAYLOAD_STATUS,FLOAT01,FLOAT02,FLOAT03,FLOAT04,FLOAT05,FLOAT06,FLOAT07,FLOAT08,FLOAT09,FLOAT10,FLOAT11,FLOAT12,FLOAT13,FLOAT14,FLOAT15,FLOAT16,FLOAT17,FLOAT18,FLOAT19,FLOAT20,STOF_TX,STOF_RX,STOF_DATAPUMP,VPC,VPC_VALID
```

`HEALTH_STATUS`, `HEARTBEAT`, the STOF fields, and `VPC` use the provisional
AS5643 profile defined in [`AS5643.md`](AS5643.md). `PAYLOAD_STATUS` and
`FLOAT01..20` are placeholders that must be replaced by the supplied payload
field names before the schema becomes stable. `AS5643_PROFILE` identifies the
selected interpretation, and `AS5643_ASSUMPTION_DEPENDENT` is true whenever
that interpretation uses working assumptions rather than an authoritative
network profile.

An approximate row from the populated fixture is:

```csv
2024-07-31T13:05:46.333129Z,,876,00005D04,msfcs_storesmassdata_b,00000000,116,aero1394-assumed-as5643b-v1,true,00000000,049CC304,40570612356019,,01000000,5603.02002,490.0,1.0,70.0,35082.0,1214.0,36115.0,37.0,5.0,105.0,1488.02002,502.0,4.0,81.0,10065.0,248.0,10282.0,23.0,-8.0,40.0,1400,500,500,158E7E3B,true
```

`VPC_VALID` is true under the selected provisional profile. The canonical
validation result must additionally retain `assumption_dependent = true`, the
reconstructed ASM-header inputs, and the selected profile identifier.

Human-facing hexadecimal fields use eight uppercase digits. `DELTA` is elapsed
recorder seconds since the previous matching record. `SYSTEM_TIME_SECONDS` is
nullable and requires an explicitly selected tick rate.

## Provisional Parquet mapping

Parquet should retain analytics-friendly physical types instead of copying CSV
text. A candidate schema is:

```text
message aero1394_record {
    required int64  recorder_time_utc (TIMESTAMP(MICROS,true));
    optional double delta_seconds;
    required int64  record_index;

    required int64  data_item_id;
    optional binary data_item_name (STRING);
    required int64  record_status;
    required int32  data_length;

    required binary as5643_profile (STRING);
    required boolean as5643_assumption_dependent;
    required int64  health_status;
    required int64  heartbeat;

    required int64  system_ticks;
    optional double system_time_seconds;
    required int64  payload_status;

    required float  float01;
    required float  float02;
    required float  float03;
    required float  float04;
    required float  float05;
    required float  float06;
    required float  float07;
    required float  float08;
    required float  float09;
    required float  float10;
    required float  float11;
    required float  float12;
    required float  float13;
    required float  float14;
    required float  float15;
    required float  float16;
    required float  float17;
    required float  float18;
    required float  float19;
    required float  float20;

    required int64  stof_tx;
    required int64  stof_rx;
    required int64  stof_datapump;
    required int64  vpc;
    optional boolean vpc_valid;
}
```

Unsigned 32-bit wire values use `INT64` to avoid signed-32-bit loss across
readers. On-wire IEEE-754 single-precision values remain physical `FLOAT`.
Before stabilizing `system_ticks` as signed `INT64`, the implementation must
define behavior for raw `u64` values above `i64::MAX`; a fixed 8-byte binary or
supported unsigned logical annotation may be safer for the full wire domain.

Candidate file metadata for a provisional tick conversion is:

```text
aero1394.system_tick_rate_hz = "13600000000"
aero1394.system_tick_period_ps = "73.5294117647"
aero1394.system_tick_rate_basis = "106250000 * 2^7"
aero1394.system_tick_rate_status = "inferred"
```

The raw tick column is canonical. Correcting the rate changes derived seconds
and metadata, not the stored wire value.

## Stability gate

Before CSV or Parquet is called stable, the project must:

1. replace provisional payload names with the supplied Rust definition;
2. choose and publish a schema version;
3. settle nullable/invalid-field behavior and unsigned `u64` mapping;
4. golden-test the exact CSV header, sample row, Parquet schema, and metadata;
5. document timezone and day-of-year edge cases; and
6. verify CLI, Rust, and Python consumers against the same decoded record.
