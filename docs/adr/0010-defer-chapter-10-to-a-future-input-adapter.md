# ADR-0010: Defer Chapter 10 to a future input adapter

- Status: Accepted
- Date: 2026-08-27
- Deciders: Project maintainers

## Context

The immediate operational need is BIE decoding. Chapter 10 recordings may
eventually supply IEEE-1394 traffic to Aero1394, and the resulting protocol and
analysis components are expected to be reused by existing Chapter 10 tools.

Planning identified the values `0x58` and `0x59`, described respectively as
IEEE-1394 Data Formats 0 and 1, as potentially relevant IRIG 106 Chapter 10
reference points. Those identifiers and meanings must be verified against the
exact Chapter 10 revision chosen for implementation. The required packet
representation and relationship to available captures have not yet been
established. Building a general Chapter 10 toolkit would also be much broader
than Aero1394's IEEE-1394/AS5643 responsibility.

## Decision

Do not implement Chapter 10 input during the initial BIE milestones.

Keep the protocol and analysis layers independent of BIE so a future Chapter 10
adapter can produce the same logical captured-event input. Do not add Chapter
10 dependencies, traits, configuration, CLI commands, or empty modules until a
real Chapter 10 use case is scheduled.

Future Chapter 10 scope is limited to extracting the IEEE-1394/AS5643 traffic
and metadata needed by Aero1394. General Chapter 10 packet types, recorder
management, and unrelated avionics buses remain the responsibility of existing
or separate tools.

Before implementation, a new ADR or an update that supersedes this one must
record:

- the exact IRIG 106 revision and supported IEEE-1394 data format(s);
- representative input captures and expected metadata;
- whether Aero1394 owns the Chapter 10 adapter or consumes an existing crate;
- the normalized metadata boundary with protocol decoding;
- timestamp and ordering semantics across containers; and
- package and feature impacts.

Chapter 10 work should begin only after the BIE-to-IEEE-1394 boundary is
supported by evidence or when a concrete integration need changes the priority.

## Alternatives considered

### Implement BIE and Chapter 10 inputs together

Rejected because BIE is urgent and still unknown. Two container investigations
would delay the first useful decoder and encourage a speculative shared API.

### Make Aero1394 a general Chapter 10 library

Rejected because it expands the product into unrelated data types and buses.
ADR-0001 deliberately names aerospace IEEE-1394 as the stable scope.

### Ignore Chapter 10 in the architecture

Rejected because container-independent protocol code is an explicit reuse goal.
The design should preserve the option without paying implementation cost now.

## Consequences

### Positive

- Work remains focused on the immediate BIE requirement.
- Protocol code is designed for reuse without premature Chapter 10 abstractions.
- Existing Chapter 10 tooling can remain the system boundary later.

### Negative

- Aero1394 will not initially decode IEEE-1394 captured inside Chapter 10.
- The eventual normalized capture model may require changes once both formats
  are understood.
- Some Chapter 10 reference material cannot be used as proof of BIE layout.

## Follow-up actions

1. Keep Chapter 10 out of the initial backlog except for reference research.
2. Preserve recorder-independent protocol inputs and outputs.
3. Revisit this decision when BIE decoding is operational and a representative
   Chapter 10 capture or integration request exists.
