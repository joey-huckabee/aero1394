# ADR-0008: Expose one Rust core through CLI and Python

- Status: Accepted
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

Aero1394 must be useful immediately as a command-line investigation and
decoding tool. The same functionality must also integrate into a Python ETL
pipeline. Protocol components are expected to be reused by other Rust-based
Chapter 10 tools later.

Independent CLI, Python, and library implementations would create behavioral
drift in field interpretation, validation, recovery, and timestamp handling.
Parsing human-oriented CLI output from Python would be brittle and would lose
typed diagnostics.

## Decision

Maintain one authoritative Rust library and expose it through two primary
surfaces:

1. a thin Rust CLI named `aero1394`; and
2. a Python distribution and import package named `aero1394`, backed by the
   Rust library through PyO3.

The CLI is responsible for arguments, file selection, output formatting, exit
codes, and terminal diagnostics. It must call public or intentionally shared
library operations for parsing, validation, recovery, and analysis.

The Python extension is an adapter, not a second decoder. It should:

- expose owned, Python-safe results rather than leaking Rust borrowing details;
- preserve structured errors, validation findings, source offsets, and raw
  values needed by ETL;
- map Rust failures to a documented Python exception hierarchy;
- release the GIL around long-running Rust work when safe and useful;
- provide streaming or batched iteration so large captures do not require one
  in-memory Python object graph; and
- version any tabular or serialized output schema independently from cosmetic
  CLI formatting.

The initial forensic CLI is expected to need operations in this family:

```text
aero1394 inspect <capture>
aero1394 hexdump <capture> --offset <n> --length <n>
aero1394 scan <capture>
aero1394 records <capture> --limit <n>
```

As knowledge becomes verified, the surface may grow to include `count`,
`dump`, `decode`, `validate`, `channels`, `messages`, and `timeline`. These
names describe intent, not a commitment to freeze arguments before workflows
are exercised with real files.

Human-readable output may evolve during reverse engineering. Machine-readable
output and Python return types require explicit schemas, compatibility notes,
and tests before they are called stable.

## Alternatives considered

### Implement the ETL decoder directly in Python

Rejected because it would duplicate the canonical Rust behavior and require
cross-language conformance for every format discovery.

### Invoke the CLI as a Python subprocess

Rejected as the primary integration because process startup, output parsing,
error transport, and large data exchange are inferior to a typed in-process
API. The CLI remains available for pipeline orchestration when isolation is
desirable.

### Expose only a Rust library initially

Rejected because interactive forensics and Python ETL are both explicit product
requirements. They may be delivered in stages, but the core API must account
for both consumers.

## Consequences

### Positive

- CLI and Python consumers receive identical decoding behavior.
- Other Rust tools can reuse the library without shelling out.
- Performance and correctness work occurs in one place.
- Structured findings remain usable in automated pipelines.

### Negative

- PyO3 packaging requires builds or wheels for every supported Python and OS
  combination.
- Borrowed internal models need intentional owned conversion boundaries.
- Public machine-readable schemas require compatibility discipline.

## Follow-up actions

1. Design the first library operation and CLI command as one vertical slice.
2. Choose and document CLI exit-code and diagnostic conventions.
3. Define the minimum supported Python and platform matrix before packaging.
4. Add Python binding tests against the same golden cases used by Rust.
5. Confirm and reserve package names before public release.
