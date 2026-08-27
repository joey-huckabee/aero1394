# ADR-0003: Use Rust as the canonical cross-platform implementation

- Status: Accepted
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

The format is not yet understood, so the first implementation will necessarily
change as evidence is collected. Maintaining independent Python, Rust, and C++
decoders during that period would multiply every discovery and create ambiguity
about which behavior is authoritative.

The tool is needed first on Windows. The project must remain capable of running
on both Windows and Linux, and its parsing and analysis components will later be
embedded in Python ETL and other Chapter 10 tools.

## Decision

Rust is the only canonical implementation of BIE, IEEE-1394, AS5643, profile,
and analysis behavior. Rust will be used for both exploratory forensics and the
production implementation.

The project will:

- deliver and verify Windows behavior first;
- keep library and CLI code portable to supported Windows and Linux targets;
- avoid OS-specific parsing behavior, path assumptions, integer widths, and
  byte-order assumptions;
- keep platform-specific integration behind narrow boundaries when it is
  unavoidable; and
- add CI coverage for both operating-system families once executable code is
  present.

Python will call the Rust core through bindings; it will not contain an
independent decoder. No C++ implementation is planned. Another native-language
implementation would require a new decision and a demonstrated consumer need.

Forensic commands may be temporary or evolve rapidly, but they must use the
same low-level parsing and validation primitives intended for the library. A
throwaway second decoder is not part of the plan.

## Alternatives considered

### Prototype in Python, then rewrite in Rust

Rejected because the format discoveries, boundary behavior, and fixtures would
need to be reconciled across two implementations. Rust can support exploratory
work while still enforcing bounds and type distinctions.

### Maintain Rust, Python, and C++ implementations in conformance

Rejected for the initial and foreseeable scope. Cross-language conformance is
valuable only when multiple independent implementations are required; here it
would slow discovery without satisfying a current consumer.

### Target Windows only

Rejected because library reuse in flight-test and ETL systems requires Linux
portability, and the binary formats themselves are platform-independent.

## Consequences

### Positive

- There is one behavioral source of truth.
- Binary parsing receives Rust's bounds, ownership, and type-safety benefits.
- CLI, Python, and future integrations share fixes and validation behavior.
- Performance work benefits all consumers.

### Negative

- Early forensic experiments may take more structure than quick scripts.
- Python packaging must include a native build or platform wheel.
- Cross-platform behavior must be tested explicitly rather than assumed.

## Follow-up actions

1. Pin and document the Rust toolchain when the project is scaffolded.
2. Add Windows and Linux build, test, formatting, and lint jobs.
3. Keep OS-specific code out of protocol modules.
4. Document the supported target and Python matrices before publishing.
