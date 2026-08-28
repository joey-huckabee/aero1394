# ADR-0013: Separate canonical data from output presentation

- Status: Proposed
- Date: 2026-08-28
- Deciders: Project maintainers

## Context

Aero1394 is expected to feed human inspection, CSV workflows, Parquet
analytics, Python ETL, and other Rust tools. The supplied BIE records also have
two different time domains: an absolute recorder timestamp and an application
payload `TimeStamp` believed to contain system ticks.

If CSV formatting, Parquet physical types, timezone conversion, or an inferred
tick rate are embedded in the parser model, a display choice can silently alter
the preserved evidence. Conversely, emitting only raw bytes would make routine
analysis unnecessarily difficult.

## Decision

Maintain one canonical typed Rust record between decoding and presentation.
It preserves raw wire values and bytes, source offsets, validation outcomes,
and any typed payload variant. Derived values are attached without replacing
their raw source.

CSV, Parquet, CLI text, and Python objects are presentation adapters over that
model:

```text
BIE bytes -> canonical decoded record -> CSV
                                      -> Parquet
                                      -> CLI text
                                      -> Python objects
```

The outer BIE recorder time and payload time are distinct fields:

- recorder time is raw `u32` Unix seconds plus raw `u32` microseconds and may
  also be exposed as a UTC instant;
- payload `TimeStamp` remains a raw `u64` system-tick value until its epoch and
  frequency are confirmed; and
- elapsed payload seconds are optional derived data whose tick rate and
  evidence state accompany the result.

Time formatting and timezone selection are separate presentation choices.
ISO-8601 UTC is the default. Day-of-year, recorder-summary, and numeric Unix
forms may be selected without changing the underlying instant. A timezone
option changes only rendering.

CSV is a stable, human-oriented flat contract. IDs, status words, and VPC values
may use fixed-width uppercase hexadecimal text. Arrays are flattened with
stable indices.

Parquet is strongly typed for analytics. Unsigned 32-bit wire words use a
portable non-lossy representation such as `INT64`; on-wire `f32` values remain
physical `FLOAT`; recorder time uses a microsecond UTC timestamp; raw system
ticks use `INT64` only when the full unsigned range and target tooling have a
documented safe mapping. Tick-rate metadata belongs in file/schema metadata,
not repeated in every row.

Any machine-readable output schema is independently versioned and golden
tested. Cosmetic CLI output is not the interchange schema.

## Alternatives considered

### Make the CSV row the internal data model

Rejected because text formatting would erase numeric types, conflate absent
and invalid values, and make Parquet/Python consumers reverse presentation
choices.

### Store only converted engineering values

Rejected because calibration and field meanings are still being established.
Raw values are necessary for reproducibility and later reinterpretation.

### Convert payload ticks to calendar time

Rejected unless an epoch is independently established. The current evidence
supports a monotonic system time, not an absolute date.

## Consequences

### Positive

- All adapters share one decoder and preserve forensic evidence.
- Display formats and timezone choices cannot mutate the underlying timestamp.
- Corrections to an inferred tick rate do not require re-decoding raw values.
- CSV remains readable while Parquet remains analytics-friendly.

### Negative

- Output adapters need explicit schema and compatibility tests.
- Some values have both raw and derived representations.
- Unsigned Parquet compatibility requires a documented mapping decision.

## Acceptance criteria

Accept this proposal when the first decoded-record type and one machine-readable
adapter demonstrate raw-value preservation, independent time formatting, and a
golden schema test.
