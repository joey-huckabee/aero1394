# ADR-0004: Start as a modular single crate

- Status: Proposed
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

The logical design has clear BIE, IEEE-1394, AS5643, profile, analysis, CLI,
and Python boundaries. Those boundaries do not yet have stable data models,
however, because no representative BIE file has been characterized.

Creating a workspace with a crate for every anticipated layer would commit the
project to package boundaries before the dependencies and reuse patterns are
known. Keeping all behavior in an unstructured binary crate would create the
opposite problem: presentation, I/O, and parsing would become entangled.

## Decision

Start with one Rust package containing a library target and a thin CLI binary.
Represent architectural boundaries as Rust modules, adding only the modules
needed by the current vertical slice. A likely shape is:

```text
Cargo.toml
src/
  lib.rs
  main.rs
  bie/
  ieee1394/
  as5643/
  analysis/
  forensic/
tests/
docs/
```

This tree is illustrative rather than a requirement to create empty modules.
The library owns parsing, validation, models, and analysis. The binary owns
argument handling, filesystem interaction, exit behavior, and presentation.

Split a module into a crate only when at least one of these conditions exists:

- it is reused independently by another repository or distributable;
- it has a meaningfully different dependency or feature footprint;
- it needs independent release/versioning policy;
- compile-time or platform isolation provides a measurable benefit; or
- its public boundary has stabilized and a crate boundary improves enforcement.

A later workspace might contain `aero1394`, `aero1394-bie`,
`aero1394-ch10`, and `aero1394-cli`, but those names do not require an early
split. The Python extension may become a separate package when packaging needs
justify it.

## Alternatives considered

### Create one crate per planned layer immediately

Rejected for now because it adds manifests, features, version relationships,
and public APIs around still-unknown format semantics.

### Build only a binary

Rejected because the Rust API and Python extension are first-class consumers.
Core behavior must be usable without invoking a process or parsing terminal
output.

### Build one flat module

Rejected because it would obscure the separation established in ADR-0002 and
make later extraction unnecessarily difficult.

## Consequences

### Positive

- Initial changes remain easy to make while BIE knowledge is incomplete.
- Module boundaries communicate the intended architecture without workspace
  overhead.
- Library-first implementation prevents CLI behavior from becoming the API.

### Negative

- Module visibility alone is weaker than crate-level dependency enforcement.
- Python packaging may eventually require a workspace adjustment.
- Extracting crates later will cause some import and feature churn.

## Acceptance criteria

Accept this proposal when the first Cargo package is scaffolded and the initial
forensic vertical slice fits without circular module dependencies. Supersede it
if a concrete packaging, reuse, or platform constraint requires a workspace at
the outset.
