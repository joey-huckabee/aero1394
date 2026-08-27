# Architecture decision records

This directory contains the durable decisions extracted from the initial
project-planning conversation. ADRs explain why the project is shaped a
particular way; they do not replace the eventual binary-format specification,
protocol notes, user guide, or API reference.

## Status meanings

- **Accepted**: part of the project baseline until superseded by another ADR.
- **Proposed**: the preferred direction, but still subject to implementation or
  capture evidence before it becomes a baseline decision.
- **Superseded**: retained for history and replaced by a later ADR.
- **Rejected**: considered and deliberately not adopted.

## Index

| ADR | Status | Decision |
| --- | --- | --- |
| [0001](0001-name-the-project-aero1394.md) | Accepted | Name the project Aero1394 |
| [0002](0002-separate-capture-protocol-profile-and-analysis-layers.md) | Accepted | Separate capture, protocol, profile, and analysis layers |
| [0003](0003-use-rust-as-the-canonical-cross-platform-implementation.md) | Accepted | Use Rust as the canonical cross-platform implementation |
| [0004](0004-start-as-a-modular-single-crate.md) | Proposed | Start as a modular single crate |
| [0005](0005-reverse-engineer-bie-with-an-evidence-led-process.md) | Accepted | Reverse-engineer BIE with an evidence-led process |
| [0006](0006-use-safe-slice-oriented-parsers-and-explicit-wire-types.md) | Proposed | Use safe, slice-oriented parsers and explicit wire types |
| [0007](0007-separate-parsing-validation-policy-and-recovery.md) | Proposed | Separate parsing, validation, policy, and recovery |
| [0008](0008-expose-one-rust-core-through-cli-and-python.md) | Accepted | Expose one Rust core through CLI and Python |
| [0009](0009-treat-documentation-and-test-evidence-as-deliverables.md) | Accepted | Treat documentation and test evidence as deliverables |
| [0010](0010-defer-chapter-10-to-a-future-input-adapter.md) | Accepted | Defer Chapter 10 to a future input adapter |
| [0011](0011-deliver-capabilities-in-evidence-gated-stages.md) | Accepted | Deliver capabilities in evidence-gated stages |

## Recording new decisions

Use the next four-digit sequence number. An ADR should state its status, date,
deciders, context, decision, alternatives, consequences, and follow-up work.
Do not silently rewrite a decision after implementation depends on it; add a
new ADR that supersedes the old one instead.
