# ADR-0002: Separate capture, protocol, profile, and analysis layers

- Status: Accepted
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

The first requested input is an internally defined `.bie` recording. A BIE
file is not the IEEE-1394 wire format itself. It may contain capture framing,
timestamps, channel information, status flags, padding, or completed
transactions.

The captured bytes may then contain IEEE-1394 packets. Some of those packets
may carry SAE AS5643 messages. Interpreting an AS5643 payload as aircraft
parameters additionally requires a network profile, slash sheet, ICD, or
equivalent configuration. These are different knowledge domains with different
sources of truth and different rates of change.

Chapter 10 may eventually provide another source of IEEE-1394 traffic. Protocol
and analysis code must be reusable by that future adapter and by other flight
test tools without depending on BIE-specific types.

## Decision

Use a layered processing model:

```text
capture container
    -> captured bus event
        -> IEEE-1394 packet
            -> optional AS5643 message
                -> optional network/profile interpretation
                    -> validation and analysis results
                        -> optional engineering signals
```

The logical responsibilities are:

| Layer | Responsibility |
| --- | --- |
| `bie` | Recorder-file framing, capture metadata, and raw captured bytes |
| future `ch10` | Chapter 10 framing and extraction of IEEE-1394 capture data |
| `ieee1394` | Bus packet fields, lengths, packet kinds, and applicable CRCs |
| `as5643` | AS5643 message structure, status, heartbeat, and timing fields |
| `profile` | Network-specific message and signal definitions |
| `analysis` | Timing, sequence, integrity, health, utilization, and anomalies |
| presentation | CLI output, serialized records, and Python-facing objects |

Dependencies must follow the data flow without pointing back to a particular
container. In particular:

- IEEE-1394 parsing must accept captured bytes and metadata without importing
  BIE types.
- AS5643 parsing must not require a network profile.
- Generic decoding must remain useful when a packet is not AS5643 or a profile
  is unavailable.
- Profile and signal decoding must not be required for packet inspection or
  validation.
- Analysis findings must retain source offsets and validation evidence so a
  user can trace them back to capture bytes.

Shared interfaces should be introduced only after at least two real consumers
demonstrate the common boundary. The project will not create a speculative
container trait solely because Chapter 10 is planned.

## Alternatives considered

### Decode the complete file in one BIE-specific operation

Rejected because recorder framing, bus packet structure, AS5643 semantics, and
aircraft-specific payload meanings would become coupled. It would also make the
protocol code difficult to reuse with Chapter 10.

### Treat BIE as the project boundary

Rejected because BIE is the urgent input format, not the stable subject of the
software. The durable responsibility is aerospace IEEE-1394 and AS5643
decoding and analysis.

### Require a profile before decoding

Rejected because raw packet inspection, CRC checks, channel discovery, and
message inventory remain valuable without an ICD. Profile data may also be
unavailable or sensitive.

## Consequences

### Positive

- Unverified BIE assumptions are isolated from standard protocol code.
- Chapter 10 tools can reuse protocol and analysis components later.
- Each layer can be tested with synthetic inputs independently.
- Users can obtain progressively richer results as protocol and profile
  information becomes available.

### Negative

- Intermediate models and conversion boundaries require deliberate design.
- Some metadata may not map cleanly across capture formats.
- End-to-end diagnostics must carry provenance through several layers.

## Follow-up actions

1. Define the minimum captured-event model only after examining a BIE sample.
2. Record data-flow diagrams, ownership, and validation boundaries in
   `docs/ARCHITECTURE.md` when implementation starts.
3. Ensure tests can exercise IEEE-1394 and AS5643 parsers without constructing
   a BIE record.
