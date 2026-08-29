# BIE hexadecimal fixtures

These whitespace-delimited hexadecimal files preserve records derived from
known-good BIE captures recorded using a FireSpy and sanitized for the
repository during the 2026-08-28 investigation. They are text so their exact
bytes remain reviewable. A test helper should reject non-hex tokens and convert
each pair to one byte before passing the result to a parser.

The fixtures describe the currently observed internal BIE record family. They
are not a complete internal conformance corpus.

The supplied capture context records that message sampling was attempted at
80 Hz. The fixture timestamps preserve actual observations, including 12.5 ms,
25 ms, and 24.142 ms gaps; tests must not replace them with an ideal sampling
grid. FireSpy sampling supports 80 Hz and 100 Hz configurations, which remain
capture provenance rather than BIE wire fields.

## `startup-four-records.hex`

- 528 bytes: four consecutive 132-byte records reconstructed exactly across
  the first dump and its explicitly identified continuation.
- No terminator is appended; this is an interior file excerpt.
- Every record has data-item ID `0x00005D04` and stored-data length 116.
- Recorder microseconds: `733129`, `745629`, `758129`, `783129`.
- Status/length words: `0x00000074`, `0x40000074`, `0x40000074`,
  `0x00000074`.
- Stored VPC values: `0x27699B11`, `0x11CEB626`, `0x03150B9E`,
  `0xFEEB8E2D`.
- AS5643 trailer values in every record: STOF TX 1400, RX 500, datapump 500.
- The application data is mostly sparse and includes aligned `450.0` and
  `62.5` float candidates.

## `end-four-records.hex`

- 532 bytes: the final four complete 132-byte records, copied from original
  file offsets `0x1C224..0x1C433`, followed by the original zero word at
  `0x1C434..0x1C437`.
- Every record has data-item ID `0x00005D04`, stored-data length 116, and
  payload raw status candidate `0x01000000`.
- Heartbeat values: `0x049CBDEE`, `0x049CBF8E`, `0x049CC149`,
  `0x049CC304`.
- Payload ticks: `0x000024E5EC3BE6C4`, `0x000024E5FA160810`,
  `0x000024E607EB19B1`, `0x000024E614F013B3`.
- Stored VPC values: `0xED45F5A5`, `0xFB681911`, `0x06957674`,
  `0x158E7E3B`.
- The final recorder time is `2024-07-31T13:05:46.333129Z`.
- Twenty populated float candidates are listed in `docs/PAYLOADS.md`.

## `empty-recording.hex`

This synthetic fixture is only `00 00 00 00`. It tests the inferred empty-file
interpretation without claiming that an empty source recording was supplied.
An authoritative source has not yet confirmed whether every BIE producer emits
this exact representation for an empty recording.
