# ADR-0001: Name the project Aero1394

- Status: Accepted
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

The repository began under the working name `bie-decoder` because its first and
most urgent job is decoding `.bie` recordings produced by an internal
simulation recording workflow. That name describes the initial file format,
but not the long-lived responsibility of the software.

The planned processing pipeline separates capture containers from the traffic
they carry:

```text
BIE capture ---------+
                     +--> IEEE-1394 --> AS5643 --> profile --> analysis
Chapter 10 capture --+
```

BIE is an internally defined capture container. Chapter 10 may become a
second input container later. The stable subject of the project is the
IEEE-1394 and AS5643 traffic, validation, timing, and engineering data exposed
through those containers.

The project will provide a Rust library and CLI on Windows and Linux. It will
also expose Rust functionality to Python through PyO3 for ETL integration.
Protocol and analysis components should remain reusable by other Chapter 10
tools without depending on BIE-specific types.

A project name therefore needs to:

- describe aerospace IEEE-1394 rather than one capture container;
- remain accurate when Chapter 10 input is added;
- work consistently as a repository, executable, Rust crate, and Python
  package name;
- avoid implying that the project handles every avionics bus or every Chapter
  10 data type; and
- avoid vendor and product trademarks in public identifiers.

## Decision

Name the project **Aero1394** and use `aero1394` for machine-facing names.

The naming convention is:

| Surface | Name |
| --- | --- |
| Project and documentation | `Aero1394` |
| Git repository | `aero1394` |
| CLI executable | `aero1394` |
| Primary Rust crate or facade | `aero1394` |
| Python distribution and import package | `aero1394` |
| BIE adapter, if split into a crate | `aero1394-bie` |
| Future Chapter 10 adapter, if split into a crate | `aero1394-ch10` |
| CLI crate, if split from the library | `aero1394-cli` |

The initial implementation may remain a single crate while the file format is
being established. These names do not require an immediate multi-crate
workspace.

BIE remains a first-class format and the immediate delivery priority, but it is
an input adapter rather than the project identity. Chapter 10 support is in
scope only where it supplies IEEE-1394/AS5643 traffic to this pipeline. A
general-purpose Chapter 10 toolkit or multi-bus avionics decoder is outside the
scope of this decision.

Public identifiers will not use vendor product names. Product names may be used
in documentation when describing verified compatibility or capture context.

Before publishing packages, maintainers must confirm and reserve the exact
names on crates.io and PyPI. A preliminary name search is not a reservation.

## Alternatives considered

### Keep `bie-decoder`

Rejected because it makes the first capture container appear to be the product
boundary and becomes misleading once another container is supported.

### `as5643-decoder`

Rejected because raw IEEE-1394 inspection, capture validation, and forensic
work must be possible even when a packet is not AS5643 or cannot yet be parsed
as AS5643.

### `avionics1394`

This is accurate, but it is longer and less distinctive than `aero1394` without
adding a meaningful scope distinction.

### `mil1394`

Rejected because it can be confused with the unrelated `MIL-STD-1394` naming
and could imply a narrower military-only audience.

### A generic avionics or flight-data name

Rejected because it implies support for buses and Chapter 10 data types that
this project is not intended to own.

## Consequences

### Positive

- The project name remains accurate for both BIE and future Chapter 10 inputs.
- CLI and package names communicate the IEEE-1394 focus.
- BIE parsing can evolve independently from IEEE-1394, AS5643, profile, and
  analysis layers.
- Reusable Rust components can be integrated into other flight-test tooling.

### Negative

- Existing local clones, Git remotes, documentation, and scripts using
  `bie-decoder` must be updated.
- The name alone does not communicate BIE support, so package metadata and the
  README must do so explicitly.
- A future expansion into unrelated buses would require a new architectural
  decision and possibly a broader umbrella project.

## Follow-up actions

1. Rename the GitHub repository and local checkout to `aero1394`.
2. Update the Git remote and project-facing documentation.
3. Use `aero1394` when scaffolding Cargo and PyO3 package metadata.
4. Reserve registry names when the packages are ready to publish.
