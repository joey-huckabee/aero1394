# Aero1394
Aerospace IEEE-1394 and AS5643 capture decoding and analysis

The project name and scope are established in
[ADR-0001](docs/adr/0001-name-the-project-aero1394.md).

Hey so we created an MIE Decoder tool for 1553. 
joey-huckabee/mie-decoder We have a second recorder for 1394 which generates bie binary files. Is there anything you can tell me about that if I want to create a similar decoder?

Yes. I think we can build a BIE Decoder using much of the same engineering approach as the MIE Decoder, but there is an important complication: with 1394 there are potentially three different formats/protocol layers we need to separate.

I looked through the current MIE Decoder architecture as well as the IEEE-1394/IRIG-106 and aerospace 1394 material. Your MIE project already has exactly the kind of structure I would reuse: streaming reader, pure decoder, synchronization/validation layer, models, dump functionality, error handling, and cross-language conformance.

## The first big distinction: .bie is not IEEE-1394

I could not find an authoritative public specification saying that .bie is a standardized IEEE-1394 recording format.

That strongly suggests something like:

```
BIE file
   │
   │ recorder/vendor-specific container
   ▼
1394 captured packet
   │
   │ IEEE-1394 / 1394b
   ▼
1394 link/transaction data
   │
   │ possibly SAE AS5643
   ▼
AS5643 message
   │
   │ vehicle/network profile / ICD
   ▼
Engineering parameters
```

That distinction is extremely important.

IEEE-1394 tells us how the bus packets work. It does not necessarily tell us how your recorder writes those packets to disk.

So we should reverse-engineer the .bie container first, then decode the 1394 data within it.

## There is actually a standardized flight-test representation for 1394  

This is one of the most promising things I found.

IRIG 106 defines two IEEE-1394 recording types:

| IRIG type | Meaning                                                     |
| --------- | ----------------------------------------------------------- |
| `0x58`    | IEEE-1394 Data Format 0 — transaction-level recording       |
| `0x59`    | IEEE-1394 Data Format 1 — physical-layer-oriented recording |

Format 0 can represent bus events, stream packets, and more general packet captures. Format 1 contains a count followed by individual 1394 messages with timestamp, length, bus status, transfer-overflow information, bus speed, and the captured data.

For example, the Format 1 conceptual structure includes:

```
CSDW
    packet_count

Message 0
    8-byte timestamp
    data_length
    local-buffer-overflow
    transfer-overflow
    speed
    bus-status
    raw 1394 data
    padding

Message 1
    ...

Message N
```

The data is padded to a 32-bit boundary.

Even if your BIE files are not IRIG 106, this gives us a very good reference model for what a flight-test recorder would logically want to store.

There is one byte-order trap worth remembering: IRIG packet structures are defined little-endian, while embedded bus data can remain in its native representation. 

## I suspect AS5643 may become the really interesting part

Because you're dealing with an aerospace 1394 recorder, I would investigate SAE AS5643 very early.

AS5643 is specifically:

> IEEE-1394b Interface Requirements for Military and Aerospace Vehicle Applications.

It uses IEEE-1394 as the underlying transport but restricts and extends it for deterministic aerospace networks. SAE explicitly says that the base standard is not stand-alone: the particular aircraft/network also has a network-profile or “slash sheet” specifying details for that implementation.

That is analogous to the issue you already encounter with 1553:

```
1553 Decoder

RT = 14
SA = 3
Word 7 = 0x394A
```

doesn't automatically tell you:

```
Word 7 = Latitude = 32.92731 degrees
```

You need the ICD/database.

1394/AS5643 has the same division.

## An AS5643 packet has a lot we can decode generically

One public AS5643 overview shows the structure clearly.

The underlying 1394 stream header gives us fields such as:

```
Data Length
Tag
Channel
Tcode
Sy
Header CRC
```

Then AS5643 adds an ASM — Anonymous Subscriber Message — header containing things such as:

```
Message ID
Reserved / Security
Node ID
Priority
Message Payload Data Length
```

Then the payload normally begins with:

```
Health Status Word
Heartbeat
Message data...
```

and the packet trailer contains timing information including:

```
STOF Transmit Offset
STOF Receive Offset
STOF Datapump Offset
Vertical Parity Check
```

followed by the IEEE-1394 data CRC. [AS5643/IEEE-1394](https://www.daptechnology.com/fileadmin/Images_Gen3/SAE_Mil1394/DE-MAR009PPT100E_SAE-AS563-MarketingSlides.pdf)

That is excellent material for a decoder.

## STOF also gives us a powerful integrity check

AS5643 networks commonly synchronize around Start Of Frame (STOF) messages.

A Control Computer sends a STOF periodically; the remote nodes synchronize against it, and scheduled messages occur at predetermined offsets from that STOF. A common implementation may use 80 Hz or 100 Hz, though the actual rate belongs to the network configuration/profile.

So your decoder potentially wouldn't just decode packets.

It could validate the network:

```
STOF
 │
 ├── +125 us → Message 0x0341 expected
 ├── +300 us → Message 0x0477 expected
 ├── +520 us → Message 0x0082 expected
 └── ...
```

We could eventually report things such as:

```
expected offset:   300.000 us
observed offset:   301.283 us
timing error:        1.283 us
status:            PASS
```

That starts turning Aero1394 into an AS5643 analysis tool, rather than just a binary converter.

## I would design BIE Decoder as a layered decoder

I would not build one giant decode_bie_record() function.
I'd structure it roughly like this:

| Layer      | Responsibility                                        |
| ---------- | ----------------------------------------------------- |
| `bie`      | Proprietary file/container framing                    |
| `ieee1394` | Raw 1394 packet parsing                               |
| `as5643`   | AS5643 message parsing                                |
| `profile`  | Aircraft/network-specific definitions                 |
| `signals`  | Engineering-unit decoding                             |
| `analysis` | timing, health, sequence, CRC, missing-message checks |

The internal models might look conceptually like:

```
BieRecord
    file_offset
    record_length
    recorder_timestamp
    recorder_flags
    raw_packet

        │
        ▼

Ieee1394Packet
    packet_type
    tcode
    channel
    tag
    sy
    speed
    data_length
    header_crc
    header_crc_valid
    data_crc
    data_crc_valid
    payload

        │
        ▼

As5643Message
    message_id
    node_id
    priority
    payload_length
    health_status
    heartbeat
    stof_tx_offset
    stof_rx_offset
    stof_datapump_offset
    vpc
    data_words

        │
        ▼

DecodedSignal
    parameter_name
    raw_value
    engineering_value
    units
    validity
```

This separation would make the library much more useful later.

## We can reuse a surprising amount of your MIE architecture

Your MIE decoder currently has this conceptual arrangement:

```
reader
   │
   ├── sync
   │
   └── decode
         │
         ▼
       models
         │
         ▼
       writer
```

with dump, filtering, merging, ordering, error handling, etc.

That is nearly exactly what I would use again.

But there is one major difference.

Your MIE format has no strong per-record magic value. You therefore developed structural validation and look-ahead chaining to find and maintain record synchronization.

1394 potentially gives us much stronger validation.

If the BIE file contains complete IEEE-1394 packets, we can combine:

```
BIE length plausible
        AND
timestamp plausible
        AND
1394 packet type plausible
        AND
1394 header length plausible
        AND
header CRC correct
        AND
data CRC correct
        AND
next BIE record valid
```

A CRC-valid packet is a far stronger resynchronization candidate than a plausible 16-bit MIE Type Word.

So the BIE decoder's recovery code may actually become more reliable.

## I would also make dump much more powerful
Something like:

```
aero1394 dump recording.bie --records 3
```

could eventually produce:

```
Record 18421
File Offset:       0x002891C0
Record Length:     184 bytes
Capture Time:      12:04:31.438822

BIE
  Flags:           0x0012
  Capture Channel: 1
  Speed:           S400

IEEE-1394
  Packet:          Async Stream
  TCODE:           0xA
  Channel:         27
  Tag:             0
  SY:              0
  Data Length:     152
  Header CRC:      PASS
  Data CRC:        PASS

AS5643
  Message ID:      0x0342
  Node ID:         0x00000017
  Priority:        3
  Payload Length:  112
  Health:          0x00000000
  Heartbeat:       18421

Timing
  STOF TX Offset:  312.500 us
  STOF RX Offset:  315.029 us

Payload
00000000  00 00 00 00 00 00 47 F5 ...
```

That would be enormously useful for reverse engineering and flight-test debugging.

## The CLI should probably evolve beyond what MIE Decoder has

I'd keep the familiar commands, but add protocol-oriented ones:

```
aero1394 inspect recording.bie

aero1394 count recording.bie

aero1394 dump recording.bie --records 10

aero1394 decode recording.bie -o messages.csv

aero1394 validate recording.bie

aero1394 channels recording.bie

aero1394 messages recording.bie

aero1394 timeline recording.bie
```

And eventually:

```
aero1394 decode recording.bie \
    --profile aircraft.toml \
    -o signals.csv
```

The distinction between these two commands is important:

```
decode recording.bie
```
means:
> Tell me what was actually on the 1394 bus.
Whereas:
```
decode recording.bie --profile aircraft.toml
```
means:
> Tell me what those aircraft messages mean.

## I would not start by writing Rust + Python + C++ simultaneously

This is one thing I would do differently than where the MIE Decoder ended up.

Your current MIE project now has independent Rust, Python, and C++ implementations constrained to identical behavior.

That's great once the format is understood.

For BIE, I would initially do:

```
Phase 1
Python forensic/prototype decoder

Phase 2
BIE-FORMAT.md becomes authoritative

Phase 3
Golden capture fixtures

Phase 4
Rust production decoder

Phase 5
Cross-language conformance

Phase 6
C++ implementation if required
```

Otherwise we'll spend three times as much effort changing parsers every time we discover that byte 12 didn't mean what we thought it meant.

## There are a couple of useful public reference implementations, but no turnkey answer

The open-source libirig106 library knows the IRIG 106 packet framework, but interestingly its published support matrix currently lists IEEE-1394 formats 0x58 and 0x59 as unsupported.

So there's probably not a mature open-source decoder we can simply drop in.

There is also a Linux isodump format for capturing 1394 isochronous packets. It has an easy-to-recognize 32-byte header beginning:

```
1394 isodump v1\0
```
and stores packet data in big-endian bus order.

I don't expect your BIE recorder to be using that format, but it gives us another useful known capture representation to compare against.

## The most important thing to determine first

Before writing the real decoder, we need to establish exactly what one .bie record represents.

There are several possibilities:

| Possibility | What BIE contains                                      |
| ----------- | ------------------------------------------------------ |
| A           | Raw PHY-level 1394 symbols/packets                     |
| B           | IEEE-1394 link packets                                 |
| C           | Completed 1394 transactions                            |
| D           | AS5643 ASM messages                                    |
| E           | Vendor records containing metadata + one of the above  |
| F           | IRIG-106-like structures wrapped in a proprietary file |

My guess is E, but I would not encode that assumption into software yet.

And if this is an aerospace vehicle bus, I would put a good amount of probability on the payload ultimately being AS5643. SAE's aerospace standard is specifically designed for this environment, including deterministic scheduling and network-specific profiles.

## What I would do next

Give me one representative .bie file. It does not need to be huge. Even a few megabytes is probably plenty initially.

If you also have the recorder vendor's software, the most valuable companion artifact would be an export from the same recording, such as CSV/text/packet dump.

Then I would start exactly the way we approached MIE:

```
1. Hex-map the file.
2. Find repeating record boundaries.
3. Determine endianess.
4. Identify length fields.
5. Identify timestamps.
6. Look for embedded valid IEEE-1394 headers.
7. Test 1394 CRCs.
8. Look for AS5643 Message IDs / Node IDs / STOF patterns.
9. Correlate records against vendor output.
10. Write BIE-FORMAT.md before building the production decoder.
```

I think this one could actually become more capable than MIE Decoder. A generic BIE → IEEE-1394 → AS5643 → aircraft-profile stack would give us not only message extraction, but CRC checking, network health, heartbeat monitoring, STOF timing analysis, dropped-message detection, channel utilization, node health, and ultimately engineering-parameter extraction. That is the architecture I would aim for rather than stopping at “binary to CSV.”

## My Response

This all sounds great and I agree we should reduce the number of versions. But instead of Python we will be creating a rust version exclusively.

## Your Response

Agreed. I would make Rust the only implementation, including the reverse-engineering/forensics phase. We do not need Python as a disposable prototype; Rust can be exploratory without becoming messy if we deliberately separate the forensic code from the stable decoder API.

I would change the plan to:

```
BIE reverse engineering
        ↓
Rust forensic reader
        ↓
BIE-FORMAT.md
        ↓
Rust protocol decoder
        ↓
IEEE-1394 decoder
        ↓
AS5643 decoder
        ↓
Network/profile decoder
        ↓
Engineering parameters
```

The important part is that we do not prematurely turn guessed BIE fields into a public Rust API.

## Proposed repository

A possible future workspace layout is:

```
aero1394/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
│
├── crates/
│   ├── bie-core/
│   ├── bie-1394/
│   ├── bie-as5643/
│   ├── bie-profile/
│   ├── bie-analysis/
│   └── bie-cli/
│
├── docs/
│   ├── BIE-FORMAT.md
│   ├── IEEE1394.md
│   ├── AS5643.md
│   ├── ARCHITECTURE.md
│   ├── REVERSE-ENGINEERING.md
│   └── adr/
│
├── tests/
│   ├── fixtures/
│   └── golden/
│
└── tools/
    └── ...
```
But I would not necessarily start with all six crates.

Initially I would keep it simpler:

```
aero1394/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── bie/
│   ├── ieee1394/
│   ├── as5643/
│   ├── analysis/
│   └── hex/
├── tests/
└── docs/
```

Once the protocol boundaries stabilize, we can split those modules into crates.

That avoids the engineering-team-overhead problem we have discussed before.

## Phase 1: Rust forensic decoder

The first version should deliberately be an investigative tool.

Something like:

```
aero1394 inspect recording.bie
aero1394 hexdump recording.bie --offset 0 --length 4096
aero1394 scan recording.bie
aero1394 records recording.bie --limit 20
aero1394 entropy recording.bie
```

And eventually:

```
aero1394 scan recording.bie --1394
```

The scanner should be able to say:

```
Candidate record boundary
-------------------------
File offset:       0x00012480
Candidate length:  184
Timestamp:         2026-08-27T12:31:04.281734
Endian:            little

Embedded IEEE-1394 candidate:
  Offset:          +24
  TCODE:           0xA
  Data length:     152
  Header CRC:      VALID
  Data CRC:        VALID

Confidence:        HIGH
```

That becomes our Rust equivalent of an interactive forensic notebook.

## Rust actually gives us a nice advantage here

Because we're dealing with binary formats, I would make the parser slice-oriented from day one.

For example:

```rust
pub fn parse_record(input: &[u8]) -> Result<(BieRecord<'_>, &[u8]), DecodeError>
```
rather than:
```rust
pub fn parse_record(reader: &mut File) -> ...
```
The layers then become composable:
```rust
let bie = BieRecord::parse(bytes)?;
let packet = Ieee1394Packet::parse(bie.payload())?;
let asm = As5643Message::parse(packet.payload())?;
```
That gives us a very clean pipeline:
```
&[u8]
  │
  ▼
BieRecord<'a>
  │
  ▼
Ieee1394Packet<'a>
  │
  ▼
As5643Message<'a>
  │
  ▼
DecodedMessage
```
Notice the lifetimes.

We should initially borrow the underlying bytes rather than copy them.

For large recorder files, that matters

## I would design around zero-copy parsing

For example:
```rust
pub struct BieRecord<'a> {
    pub offset: u64,
    pub timestamp: RecorderTimestamp,
    pub flags: BieFlags,
    pub payload: &'a [u8],
}
```
Then:
```rust
pub struct Ieee1394Packet<'a> {
    pub tcode: TransactionCode,
    pub channel: Channel,
    pub data_length: usize,
    pub payload: &'a [u8],
    pub header_crc: CrcStatus,
    pub data_crc: CrcStatus,
}
```
And:
```rust
pub struct As5643Message<'a> {
    pub message_id: MessageId,
    pub node_id: NodeId,
    pub priority: Priority,
    pub health: HealthStatus,
    pub heartbeat: Heartbeat,
    pub payload: &'a [u8],
}
```
No unnecessary allocations.

No giant intermediate buffers.

## Use Rust types to prevent decoder mistakes

This is an area where I would go further than we did with some of the earlier tools.

Don't do this everywhere:

```rust
message_id: u16,
node_id: u32,
channel: u8,
timestamp: u64,
```
Instead:
```rust
#[repr(transparent)]
pub struct MessageId(u16);

#[repr(transparent)]
pub struct NodeId(u32);

#[repr(transparent)]
pub struct Channel(u8);

#[repr(transparent)]
pub struct FileOffset(u64);
```
And distinguish timestamp domains:
```rust
pub struct RecorderTime(u64);

pub struct StofOffset(u32);

pub struct BusCycleTime(u16);
```
This prevents accidentally comparing:
```
recorder timestamp
```
to:
```
STOF offset
```
just because both happen to be integers.

That's exactly the kind of protection Rust is good at providing.

## Endianness should be explicit

I do not want code scattered throughout the project like:

```rust
u32::from_le_bytes(...)
```
Instead we'd establish low-level helpers:
```rust
fn read_u16_le(input: &[u8]) -> Result<u16, DecodeError>;
fn read_u32_le(input: &[u8]) -> Result<u32, DecodeError>;

fn read_u16_be(input: &[u8]) -> Result<u16, DecodeError>;
fn read_u32_be(input: &[u8]) -> Result<u32, DecodeError>;
```
Or typed cursor methods:
```rust
cursor.read_le::<u32>()?;
cursor.read_be::<u32>()?;
```
because we may encounter exactly this:
```
BIE recorder metadata
        little endian

IEEE-1394 bus packet
        big endian

AS5643 payload
        network-defined
```
Having explicit boundaries prevents some truly painful bugs.

## We should avoid unsafe

My initial design goal would be:

```
#![forbid(unsafe_code)]
```
Especially during reverse engineering.

It is tempting to do:

```rust
let header = unsafe {
    &*(ptr as *const BieHeader)
};
```
Don't.

Binary structures frequently contain:

* padding
* packed fields
* non-native byte order
* variable lengths
* bit fields
* alignment differences

So mapping a file directly onto a Rust struct is usually the wrong abstraction anyway.

Parse the wire representation explicitly.

## The parser should distinguish three types of failure
This matters a great deal for corrupted recorder files.

I'd likely define:
```rust
pub enum DecodeError {
    Truncated { needed: usize, available: usize },

    Invalid {
        offset: u64,
        reason: InvalidReason,
    },

    Unsupported {
        feature: UnsupportedFeature,
    },
}
```
And separately model validation results.

For example:

```
pub enum CrcStatus {
    Valid,
    Invalid {
        expected: u32,
        observed: u32,
    },
    NotPresent,
    NotChecked,
}
```
An invalid CRC is not necessarily a parser failure.

We may still want to decode the record.

That distinction becomes important for flight-test data.

## I want strict and forensic modes

This will be one of the most useful features.

Normal processing:

```
aero1394 decode file.bie --strict
```
could stop/reject invalid records.

Forensic mode:
```
aero1394 inspect file.bie --forensic
```
could say:
```
Record 48512

BIE framing:       VALID
1394 header:       VALID
Header CRC:        VALID
Payload CRC:       FAILED
AS5643 structure:  PLAUSIBLE

Action:
  packet decoded but marked corrupt
```
Those modes shouldn't change the underlying parser.

They change policy.

Conceptually:
```
parser
   ↓
DecodeResult

policy
   ├─ strict
   ├─ permissive
   └─ forensic
```
I strongly prefer that over baking recovery logic into every parsing function.

## I strongly prefer that over baking recovery logic into every parsing function.
This was one of the hardest aspects of MIE.

For BIE I would explicitly make it:
```rust
pub trait SyncStrategy {
    fn find_next(
        &self,
        data: &[u8],
        current_offset: usize,
    ) -> Option<SyncCandidate>;
}
```
And a candidate could contain:
```rust
pub struct SyncCandidate {
    pub offset: usize,
    pub confidence: Confidence,
    pub evidence: SyncEvidence,
}
```
Where the evidence might be:
```
✓ plausible BIE length
✓ plausible timestamp
✓ known flags
✓ valid 1394 tcode
✓ valid IEEE-1394 header CRC
✓ valid data CRC
✓ next record also validates
```
Then:
```
Confidence = 98 / 100
```
Not because "98" itself is scientifically meaningful, but because multiple independent structural checks are much better than:
```
"I found bytes that look like a header."
```

## Rust can also make the scanner extremely fast
Once the structure is understood, we can eventually use memory mapping for large files:
```
10 GB BIE
  │
  mmap
  ▼
&[u8]
  │
  ├── record
  ├── record
  ├── record
  └── record
```
and parse without reading/copying the entire file into temporary buffers.

For the first implementation, however, I would probably use buffered I/O unless file size makes mmap particularly attractive.

The parser itself should remain independent:

```rust
fn parse_record(bytes: &[u8]) -> ...
```
Then the storage layer can eventually support:
```
BufReader<File>
memory map
network stream
test fixture
in-memory buffer
```
without changing the decoder.

## The architecture I ultimately want
This is the bigger picture:

```
                         Aero1394
                             │
                 ┌───────────┴───────────┐
                 │                       │
             File Layer             Analysis Layer
                 │                       │
              BIE                    statistics
                 │                    validation
                 ▼                     timing
             IEEE-1394               anomalies
                 │                       │
                 ▼                       │
              AS5643  ◄─────────────────┘
                 │
                 ▼
          Network Profile
                 │
                 ▼
         Signal Definitions
                 │
                 ▼
        Engineering Values
```
That lets us ask progressively more sophisticated questions.

### Level 1
```
Can I read the BIE file?
```

### Level 2
```
What 1394 traffic did the recorder capture?
```

### Level 3
```
Which AS5643 messages occurred?
```

### Level 4
```
Were the network timing and message behavior correct?
```

### Level 5
```
What aircraft parameters do those messages represent?
```
That should be the long-term design.

## Rust-only implementation

Rust will be used for both the forensic/reverse-engineering stage and the eventual high-performance production decoder, while keeping guessed BIE semantics isolated until they have been proven.

When we get the first BIE sample, I would start with a very small Rust inspect implementation whose only jobs are file statistics, offset-oriented hex inspection, repeating-pattern discovery, candidate length/timestamp detection, and 1394-header scanning. We shouldn't even call something a BieHeader until the evidence supports it.


## Additional information to get started with (From me)
- I believe the BIE files are recorded by DAP Technologies firespy.
- We need to convert this conversation to detailed ADRs.
- I need the first version we build in Rust to run on windows but the project should be able to run on Windows and Linux ultimately.  
- We need this fully documented including the format of the BIE binary and the different data flow pipelines including corner cases which must be checked for.
- I am guessing we will eventually want to consume CH.10 files as well, but that will be much later because I need the BIE binary decoding now.
- I need to be able to build a Rust CLI as well as a a python package which uses O3 so I can integrate this into my ETL Pipeline.
- We will be making a tool which I will use immedietely but then we will use components of the libraries in my CH.10 tools.
