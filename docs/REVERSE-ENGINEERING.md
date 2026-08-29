# Reverse-engineering BIE captures

- Status: Stage 1 working method
- Last updated: 2026-08-28

This procedure turns observations from an unknown capture into reproducible
evidence without assuming that the extension identifies its contents. The
format claims and open questions belong in [BIE-FORMAT.md](BIE-FORMAT.md).
Application-specific layouts belong in [PAYLOADS.md](PAYLOADS.md).

## Protect and record the source

Work only with a capture you are authorized to inspect. Keep the original
read-only and outside the repository until provenance, sensitivity, and
redistribution rules are known. Record at least:

- original filename and file size;
- SHA-256 digest;
- acquisition date and context;
- recorder hardware, firmware, and software versions;
- export or transfer steps;
- known traffic and capture settings; and
- handling and redistribution restrictions.

On PowerShell, record size and SHA-256 without changing the file:

```powershell
(Get-Item -LiteralPath 'C:\captures\sample.bie').Length
Get-FileHash -Algorithm SHA256 -LiteralPath 'C:\captures\sample.bie'
```

On Linux:

```sh
stat --format='%s' /captures/sample.bie
sha256sum /captures/sample.bie
```

Do not send a real capture to a vendor by ordinary email when it may contain
CUI, ITAR-controlled, proprietary, or otherwise restricted information. Follow
the controlled-transfer guidance recorded in `BIE-FORMAT.md`.

## Produce bounded observations

The default command reads only the first 256 bytes:

```text
cargo run --release -- hexdump C:\captures\sample.bie
```

Every line contains the absolute 64-bit file offset, raw hexadecimal bytes,
and a printable-ASCII preview. No BIE meaning is assigned to any byte.

Inspect a candidate region by offset and length:

```text
cargo run --release -- hexdump C:\captures\sample.bie --offset 0x1000 --length 0x200
```

Change presentation width without changing the selected bytes:

```text
cargo run --release -- hexdump C:\captures\sample.bie --offset 4096 --length 512 --width 32
```

The command accepts decimal values and `0x`-prefixed hexadecimal values;
underscores may separate digits. An offset past EOF is an error. A range that
ends past EOF stops cleanly at EOF.

Dumping a complete file requires an explicit request:

```text
cargo run --release -- hexdump C:\captures\sample.bie --length all > sample.hexdump.txt
```

The text output reproduces source bytes and inherits the capture's handling
restrictions. Prefer small ranges tied to a stated observation rather than
creating unnecessary full-file derivatives.

## Record an observation

For every potentially meaningful pattern, record:

1. capture digest or stable internal identifier;
2. command and Aero1394 version or commit;
3. absolute range inspected;
4. literal observed bytes;
5. whether the pattern repeats and at which offsets;
6. candidate interpretation labeled **Hypothesis**;
7. competing interpretations and counterexamples; and
8. evidence needed to confirm or reject the hypothesis.

Do not rename an unknown value after a protocol concept merely because one bit
pattern is plausible. Compare multiple records and, when possible, a matching
independently decoded view. The evidence sequence and confidence terms are
defined in [ADR-0005](adr/0005-reverse-engineer-bie-with-an-evidence-led-process.md).

When a supplied message is authorized for repository use, preserve the minimum
sanitized bytes needed for a golden case under `tests/fixtures`. Document byte
count, source relationship, expected raw values, and whether messages were
actually consecutive. Do not reconstruct a fictional continuous capture by
concatenating excerpts from different file offsets.

## Current command boundary

`hexdump` is observational only. It does not currently:

- identify BIE or future input-adapter signatures listed in
  [`ROADMAP.md`](../ROADMAP.md);
- calculate or validate record lengths;
- locate repeated structures;
- infer timestamps or byte order;
- parse IEEE-1394 packets; or
- decode AS5643 traffic.

Those behaviors should arrive as separate forensic operations only when their
outputs can distinguish observations, hypotheses, and confirmed rules.
