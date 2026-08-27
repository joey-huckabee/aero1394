# ADR-0006: Use safe, slice-oriented parsers and explicit wire types

- Status: Proposed
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

Recorder files can be large, truncated, corrupt, and a mixture of byte orders.
The likely processing path combines recorder metadata, IEEE-1394 bus bytes,
AS5643 fields, and profile-defined payloads. Mapping these bytes directly onto
native Rust structs would be vulnerable to padding, alignment, native-endian,
packed-bit-field, and variable-length mistakes.

Tying parsers to `File` would also prevent the same parser from operating on a
buffered stream, a memory-mapped region, a network source, an in-memory test
fixture, or captured bytes extracted from Chapter 10.

## Decision

Make stable parsers operate on byte slices and return both a parsed view and an
unambiguous consumed length or remainder. A representative shape is:

```rust
pub fn parse_record(input: &[u8]) -> Result<(BieRecord<'_>, usize), DecodeError>;
```

This signature is illustrative; the exact API will be selected during the
first implementation. The required properties are:

- parsing is independent of filesystem I/O;
- every read checks available bytes before indexing;
- length, offset, alignment, and padding arithmetic uses checked operations;
- borrowed payloads are used where they simplify composition and avoid large
  copies;
- an owned representation is available at presentation, serialization, Python,
  or long-term-storage boundaries where borrowing is inappropriate; and
- errors and validation findings preserve the absolute file offset supplied by
  the caller.

The project should use `#![forbid(unsafe_code)]` in its own crates unless a
future ADR documents a measured need, a contained boundary, and its safety
argument. Dependencies that encapsulate platform facilities such as memory
mapping must be assessed separately.

Do not transmute or cast file bytes into Rust wire structs. Decode fields
explicitly. Byte order must be named at the layer boundary rather than inferred
from the host:

```rust
read_u32_le(recorder_bytes)?;
read_u32_be(bus_bytes)?;
```

Low-level helpers or a checked cursor may centralize those operations, but
callers must remain able to see which byte order applies. Mixed-endian data is
expected to be possible.

Use small domain types where they prevent unit or identity confusion, for
example `FileOffset`, `MessageId`, `NodeId`, `RecorderTime`, and `StofOffset`.
Newtypes must provide concrete safety or clarity; primitive fields need not be
wrapped mechanically.

The parser should remain compatible with buffered I/O first. Memory mapping is
an optimization to consider only after file sizes and profiling justify it.
Zero-copy parsing is a direction, not a prohibition on small or necessary
allocations.

## Alternatives considered

### Parse directly from `Read` or `File`

Rejected for protocol functions because it couples decoding to storage and
makes nested parsing and fixture tests harder. A reader can still supply slices
to the parser.

### Map wire layouts onto `#[repr(C)]` or packed structs

Rejected because BIE layout is unknown and wire formats may include explicit
byte order, bit fields, padding, and variable-length data. Native layout is not
the wire contract.

### Copy every field and payload into owned structures

Rejected as the default because large recordings and nested parsing can incur
avoidable allocation and copying. Owned forms remain appropriate at API
boundaries that cannot express borrowing safely.

### Require memory mapping immediately

Rejected because it adds platform and lifetime complexity before performance
measurements exist. Slice-oriented parsers keep that option open.

## Consequences

### Positive

- The same parser supports files, Chapter 10 payloads, fixtures, and fuzzing.
- Bounds and byte order are explicit and reviewable.
- Large payloads can flow through layers without mandatory copies.
- Domain types reduce accidental comparison of unrelated numeric fields.

### Negative

- Lifetimes can make internal models and iterators more complex.
- Python and serialization surfaces need owned conversions.
- Checked parsing is more verbose than native-struct casting.

## Acceptance criteria

Accept this proposal after a forensic parser demonstrates the approach on a
real BIE sample and after its borrowed and owned boundaries are reviewed for
CLI and Python use. Any exception to the safe-code rule requires a superseding
ADR.
