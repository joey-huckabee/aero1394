# ADR-0012: Use built-in Rust payload definitions

- Status: Accepted
- Date: 2026-08-28
- Deciders: Project maintainers

## Context

The BIE container and AS5643-derived framing can locate an application payload,
but only a program ICD or equivalent definition can assign types and
engineering meaning to its bytes. Additional data structures will be supplied
as the investigation proceeds.

An earlier design discussion considered external YAML profiles. The project
maintainer instead wants payload structures represented as Rust source and
built into the tool. This makes payload support reviewable, type checked, and
versioned with the decoder, but it needs a predictable extension path so each
new payload does not become a special case in the BIE parser.

This decision refines the network/profile layer from ADR-0002 and Stage 5 from
ADR-0011. A profile remains a separate knowledge layer; its implementation in
Aero1394 is a registry of Rust-native payload definitions rather than a runtime
YAML schema.

## Decision

Application payloads are opaque to the `bie`, `ieee1394`, and `as5643` layers.
Typed application decoding lives in a separate `payload` module. Each supported
payload is defined in its own Rust source file and compiled into Aero1394.

A likely module shape is:

```text
src/
  payload/
    mod.rs
    registry.rs
    value.rs
    <payload_name>.rs
```

The exact trait and enum names will be selected with the first implementation,
but every built-in definition must provide these capabilities:

- stable payload name and definition version;
- match criteria including data-item ID and exact payload size;
- optional bus/data-code or configuration constraints when ID and size are not
  unique;
- explicit byte order, offsets, widths, and primitive wire types;
- checked decoding from a byte slice without native-struct casts;
- access to raw bytes and raw field values;
- typed decoded fields and documented engineering conversions when known; and
- definition provenance and evidence state for meanings that remain inferred.

Selection must go through one explicit registry. The BIE parser must not import
or match individual payload modules. Registry behavior must be deterministic:

1. collect definitions matching ID and payload size;
2. apply available bus/configuration constraints;
3. decode only when exactly one definition remains;
4. report ambiguity when more than one remains; and
5. preserve an unknown payload as raw bytes when none remains.

Message ID alone is not a globally unique schema key. The initial registry may
use `(data_item_id, payload_size)` because those are the only confirmed
selectors, while leaving room for data code and configuration/version.

Payload modules must not use `#[repr(C)]`, packed structs, `transmute`, or host
byte order as the wire contract. They follow ADR-0006 and decode explicit
ranges from slices. Gaps are allowed when documented; overlaps and out-of-range
fields are errors.

External YAML profiles are not part of the required runtime design. A future
code generator may consume an authorized ICD or neutral interchange format,
but generated output must be ordinary reviewable Rust modules using the same
registry and tests. Adding runtime-loadable schemas or plugins requires a new
ADR because it changes validation, compatibility, and trust boundaries.

## Adding another payload

A contributor adds support through one repeatable workflow:

1. record the authorized field definition and evidence in `docs/PAYLOADS.md`;
2. add one `src/payload/<name>.rs` module with explicit offsets and types;
3. register its match criteria in `src/payload/registry.rs`;
4. add representative message bytes and expected values under
   `tests/fixtures/payload/<name>/`;
5. test exact-size matching, every field boundary, byte order, unknown and
   ambiguous selection, and any engineering conversion; and
6. add or update the applicable L3 requirement and test markers, then
   regenerate `docs/TRACE-MATRIX.md`.

The module is available after the normal Aero1394 build. No deployed YAML file
or user-managed profile directory is required.

## Alternatives considered

### Load YAML payload profiles at runtime

Rejected for the current product direction. It improves no-rebuild
customization but moves type and range errors to runtime, creates a profile
distribution/versioning surface, and conflicts with the maintainer's desired
built-in Rust structures.

### Put every payload directly in the BIE decoder

Rejected because application meanings would contaminate generic container and
protocol parsing, prevent raw decoding when an ICD is absent, and make future
input adapters depend on BIE-specific code.

### Match only on data-item ID

Rejected because IDs can be reused on another bus, configuration, software
load, or payload revision.

### Require a known payload to parse a record

Rejected because unknown application bytes remain useful for inventory,
timing, integrity checks, and forensic analysis.

## Consequences

### Positive

- Payload definitions are type checked, reviewed, tested, and versioned with
  the decoder.
- The generic BIE and AS5643-derived layers stay reusable.
- Unknown payloads remain inspectable.
- Adding support follows a documented module/registry/fixture workflow.

### Negative

- A new payload or definition correction requires rebuilding Aero1394.
- Program-specific definitions can increase binary size and may need feature
  gating if the catalog becomes large or handling restrictions differ.
- Users cannot author arbitrary payload schemas without Rust development.

## Follow-up actions

1. Maintain the implemented `payload` module, registry, and first raw decoder as
   the extension pattern for additional definitions.
2. Keep unresolved engineering semantics in `docs/PAYLOADS.md` until their
   authorized source metadata and meanings are provided.
3. Define feature-gating and controlled-data handling before accepting a
   restricted payload definition.
