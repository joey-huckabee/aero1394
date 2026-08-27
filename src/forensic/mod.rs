//! Format-neutral primitives for examining unknown binary captures.

use std::error::Error;
use std::fmt;
use std::io::{self, Read};

/// Maximum accepted line width, keeping every read and allocation bounded.
pub const MAX_BYTES_PER_LINE: usize = 256;

/// An absolute byte offset in the source capture.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FileOffset(u64);

impl FileOffset {
    /// Creates an absolute file offset.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric byte offset.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    fn checked_add(self, byte_count: u64) -> Option<Self> {
        self.0.checked_add(byte_count).map(Self)
    }
}

/// Bounds controlling a streaming hex dump.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HexdumpConfig {
    bytes_per_line: usize,
    byte_limit: Option<u64>,
}

impl HexdumpConfig {
    /// Creates a configuration.
    ///
    /// `byte_limit` is `None` only when the caller explicitly wants to read
    /// until EOF. Line widths are bounded to prevent accidental large
    /// allocations from untrusted command input.
    pub fn new(bytes_per_line: usize, byte_limit: Option<u64>) -> Result<Self, HexdumpConfigError> {
        if bytes_per_line == 0 {
            return Err(HexdumpConfigError::ZeroWidth);
        }
        if bytes_per_line > MAX_BYTES_PER_LINE {
            return Err(HexdumpConfigError::WidthTooLarge {
                actual: bytes_per_line,
                maximum: MAX_BYTES_PER_LINE,
            });
        }

        Ok(Self {
            bytes_per_line,
            byte_limit,
        })
    }

    /// Returns the number of source bytes represented by a full output line.
    #[must_use]
    pub const fn bytes_per_line(self) -> usize {
        self.bytes_per_line
    }

    /// Returns the maximum source bytes to read, or `None` for EOF.
    #[must_use]
    pub const fn byte_limit(self) -> Option<u64> {
        self.byte_limit
    }
}

/// A rejected hex-dump configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HexdumpConfigError {
    /// A zero-byte output line would make no forward progress.
    ZeroWidth,
    /// A requested line width exceeded the allocation bound.
    WidthTooLarge {
        /// Requested bytes per line.
        actual: usize,
        /// Largest accepted bytes per line.
        maximum: usize,
    },
}

impl fmt::Display for HexdumpConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroWidth => formatter.write_str("bytes per line must be at least 1"),
            Self::WidthTooLarge { actual, maximum } => write!(
                formatter,
                "bytes per line must not exceed {maximum}; received {actual}"
            ),
        }
    }
}

impl Error for HexdumpConfigError {}

/// One observed range of bytes from a source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HexdumpLine {
    offset: FileOffset,
    bytes: Vec<u8>,
}

impl HexdumpLine {
    /// Returns the absolute source offset of the first byte.
    #[must_use]
    pub const fn offset(&self) -> FileOffset {
        self.offset
    }

    /// Returns the observed source bytes without interpreting them.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// An error encountered while consuming a hex-dump stream.
#[derive(Debug)]
pub enum HexdumpError {
    /// The source could not be read.
    Io(io::Error),
    /// Advancing the absolute source offset would overflow `u64`.
    OffsetOverflow {
        /// Offset at which the read began.
        offset: FileOffset,
        /// Bytes read before the overflow was detected.
        bytes_read: usize,
    },
}

impl fmt::Display for HexdumpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "could not read source bytes: {error}"),
            Self::OffsetOverflow { offset, bytes_read } => write!(
                formatter,
                "file offset overflow after reading {bytes_read} bytes at 0x{:016x}",
                offset.get()
            ),
        }
    }
}

impl Error for HexdumpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::OffsetOverflow { .. } => None,
        }
    }
}

/// A bounded, streaming iterator over uninterpreted source bytes.
///
/// The caller owns positioning the source and supplies the absolute offset at
/// which reading begins. This keeps filesystem concerns outside the reusable
/// forensic core and also permits in-memory or extracted capture data.
pub struct Hexdump<R> {
    reader: R,
    next_offset: FileOffset,
    bytes_per_line: usize,
    remaining: Option<u64>,
    pending_error: Option<io::Error>,
    finished: bool,
}

impl<R: Read> Hexdump<R> {
    /// Creates a streaming dump over a reader already positioned at
    /// `start_offset`.
    #[must_use]
    pub fn new(reader: R, start_offset: FileOffset, config: HexdumpConfig) -> Self {
        Self {
            reader,
            next_offset: start_offset,
            bytes_per_line: config.bytes_per_line(),
            remaining: config.byte_limit(),
            pending_error: None,
            finished: false,
        }
    }
}

impl<R: Read> Iterator for Hexdump<R> {
    type Item = Result<HexdumpLine, HexdumpError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        if let Some(error) = self.pending_error.take() {
            self.finished = true;
            return Some(Err(HexdumpError::Io(error)));
        }

        let target_len = match self.remaining {
            Some(0) => {
                self.finished = true;
                return None;
            }
            Some(remaining) => remaining.min(self.bytes_per_line as u64) as usize,
            None => self.bytes_per_line,
        };

        let mut bytes = vec![0_u8; target_len];
        let mut bytes_read = 0;
        let mut reached_eof = false;

        while bytes_read < target_len {
            match self.reader.read(&mut bytes[bytes_read..]) {
                Ok(0) => {
                    reached_eof = true;
                    break;
                }
                Ok(count) => bytes_read += count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) if bytes_read > 0 => {
                    self.pending_error = Some(error);
                    break;
                }
                Err(error) => {
                    self.finished = true;
                    return Some(Err(HexdumpError::Io(error)));
                }
            }
        }

        if bytes_read == 0 {
            self.finished = true;
            return None;
        }

        bytes.truncate(bytes_read);
        let line_offset = self.next_offset;
        let Some(next_offset) = self.next_offset.checked_add(bytes_read as u64) else {
            self.finished = true;
            return Some(Err(HexdumpError::OffsetOverflow {
                offset: line_offset,
                bytes_read,
            }));
        };
        self.next_offset = next_offset;

        if let Some(remaining) = &mut self.remaining {
            *remaining -= bytes_read as u64;
        }
        if reached_eof {
            self.finished = true;
        }

        Some(Ok(HexdumpLine {
            offset: line_offset,
            bytes,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn rejects_non_progressing_and_excessive_widths() {
        assert_eq!(
            HexdumpConfig::new(0, Some(1)),
            Err(HexdumpConfigError::ZeroWidth)
        );
        assert_eq!(
            HexdumpConfig::new(MAX_BYTES_PER_LINE + 1, Some(1)),
            Err(HexdumpConfigError::WidthTooLarge {
                actual: MAX_BYTES_PER_LINE + 1,
                maximum: MAX_BYTES_PER_LINE,
            })
        );
    }

    #[test]
    fn emits_bounded_lines_with_absolute_offsets() {
        let config = HexdumpConfig::new(4, Some(6)).expect("valid configuration");
        let lines = Hexdump::new(
            Cursor::new((0_u8..10).collect::<Vec<_>>()),
            FileOffset::new(0x20),
            config,
        )
        .collect::<Result<Vec<_>, _>>()
        .expect("read succeeds");

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].offset(), FileOffset::new(0x20));
        assert_eq!(lines[0].bytes(), &[0, 1, 2, 3]);
        assert_eq!(lines[1].offset(), FileOffset::new(0x24));
        assert_eq!(lines[1].bytes(), &[4, 5]);
    }

    #[test]
    fn unbounded_mode_stops_at_eof() {
        let config = HexdumpConfig::new(3, None).expect("valid configuration");
        let lines = Hexdump::new(
            Cursor::new([0x10, 0x11, 0x12, 0x13]),
            FileOffset::new(0),
            config,
        )
        .collect::<Result<Vec<_>, _>>()
        .expect("read succeeds");

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].bytes(), &[0x10, 0x11, 0x12]);
        assert_eq!(lines[1].bytes(), &[0x13]);
    }

    #[test]
    fn zero_byte_limit_does_not_read() {
        let config = HexdumpConfig::new(16, Some(0)).expect("valid configuration");
        let mut dump = Hexdump::new(Cursor::new([0xAA]), FileOffset::new(0), config);

        assert!(dump.next().is_none());
    }

    #[test]
    fn reports_offset_overflow() {
        let config = HexdumpConfig::new(1, Some(1)).expect("valid configuration");
        let mut dump = Hexdump::new(Cursor::new([0xAA]), FileOffset::new(u64::MAX), config);

        assert!(matches!(
            dump.next(),
            Some(Err(HexdumpError::OffsetOverflow {
                offset,
                bytes_read: 1
            })) if offset == FileOffset::new(u64::MAX)
        ));
        assert!(dump.next().is_none());
    }
}
