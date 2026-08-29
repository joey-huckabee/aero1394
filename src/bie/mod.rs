//! Safe, slice-oriented parsing for the internal BIE capture format.
//!
//! This module parses individual non-terminator records and strict complete
//! BIE byte slices. Validation, policy, and recovery remain separate later
//! layers.

use crate::forensic::FileOffset;
use std::error::Error;
use std::fmt;

/// Number of bytes in the fixed BIE record header.
pub const RECORD_HEADER_LEN: usize = 16;

/// Number of bytes in the terminal zero word.
pub const FILE_TERMINATOR_LEN: usize = 4;

const DATA_LENGTH_MASK: u32 = 0x0000_FFFF;
const UNRESOLVED_FLAGS_MASK: u32 = 0xFFFF_0000;

/// A BIE container data-item identifier.
///
/// This is kept distinct from a downstream AS5643 Message ID even when a BIE
/// profile maps both identities to the same numeric value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataItemId(u32);

impl DataItemId {
    /// Creates a data-item identifier from its raw BIE value.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw identifier.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Raw recorder timestamp fields from a BIE record header.
///
/// Calendar conversion and microsecond-range validation are intentionally not
/// framing responsibilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecorderTime {
    seconds: u32,
    microseconds: u32,
}

impl RecorderTime {
    /// Returns raw unsigned Unix seconds from the record header.
    #[must_use]
    pub const fn seconds(self) -> u32 {
        self.seconds
    }

    /// Returns the raw microsecond component without validating its range.
    #[must_use]
    pub const fn microseconds(self) -> u32 {
        self.microseconds
    }
}

/// The raw BIE status/length word and its structural components.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatusAndLength(u32);

impl StatusAndLength {
    /// Returns the complete raw word.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns the stored-data length encoded in the low 16 bits.
    #[must_use]
    pub const fn data_length(self) -> usize {
        (self.0 & DATA_LENGTH_MASK) as usize
    }

    /// Returns the uninterpreted high 16 bits.
    #[must_use]
    pub const fn unresolved_flags(self) -> u32 {
        self.0 & UNRESOLVED_FLAGS_MASK
    }
}

/// One structurally complete BIE record borrowing its stored-data bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BieRecord<'a> {
    file_offset: FileOffset,
    data_item_id: DataItemId,
    recorder_time: RecorderTime,
    status_and_length: StatusAndLength,
    stored_data: &'a [u8],
}

impl<'a> BieRecord<'a> {
    /// Returns the absolute offset where this record begins.
    #[must_use]
    pub const fn file_offset(&self) -> FileOffset {
        self.file_offset
    }

    /// Returns the BIE data-item identifier.
    #[must_use]
    pub const fn data_item_id(&self) -> DataItemId {
        self.data_item_id
    }

    /// Returns the raw recorder timestamp fields.
    #[must_use]
    pub const fn recorder_time(&self) -> RecorderTime {
        self.recorder_time
    }

    /// Returns the raw status/length word and its structural components.
    #[must_use]
    pub const fn status_and_length(&self) -> StatusAndLength {
        self.status_and_length
    }

    /// Returns the exact stored-data bytes declared by the record header.
    #[must_use]
    pub const fn stored_data(&self) -> &'a [u8] {
        self.stored_data
    }

    /// Returns the complete encoded record size.
    #[must_use]
    pub const fn encoded_len(&self) -> usize {
        RECORD_HEADER_LEN + self.stored_data.len()
    }
}

/// One structurally complete BIE file borrowing its records' stored data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BieFile<'a> {
    records: Vec<BieRecord<'a>>,
    terminator_offset: FileOffset,
    encoded_len: usize,
}

impl<'a> BieFile<'a> {
    /// Returns all records in file order.
    #[must_use]
    pub fn records(&self) -> &[BieRecord<'a>] {
        &self.records
    }

    /// Returns the absolute offset of the terminal zero word.
    #[must_use]
    pub const fn terminator_offset(&self) -> FileOffset {
        self.terminator_offset
    }

    /// Returns the complete encoded size, including the terminal zero word.
    #[must_use]
    pub const fn encoded_len(&self) -> usize {
        self.encoded_len
    }

    /// Returns whether the file contains no records before its terminator.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// A failure to parse one non-terminator BIE record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BieRecordParseError {
    /// Fewer than 16 header bytes were available.
    TruncatedHeader {
        /// Absolute offset where the header begins.
        offset: FileOffset,
        /// Complete header size required by the format.
        needed: usize,
        /// Header bytes available in the supplied slice.
        available: usize,
    },
    /// Zero is reserved for the file terminator and cannot identify a record.
    ZeroDataItemId {
        /// Absolute offset of the zero data-item word.
        offset: FileOffset,
    },
    /// The complete declared stored-data body was not available.
    TruncatedStoredData {
        /// Absolute offset where stored data begins.
        offset: FileOffset,
        /// Stored-data bytes declared by the header.
        needed: usize,
        /// Stored-data bytes available in the supplied slice.
        available: usize,
    },
    /// The absolute offset after the record could not be represented by `u64`.
    OffsetOverflow {
        /// Absolute offset where the record begins.
        offset: FileOffset,
        /// Complete encoded size of the record.
        record_len: usize,
    },
}

impl fmt::Display for BieRecordParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "truncated BIE record header at 0x{:016x}: needed {needed} bytes, available {available}",
                offset.get()
            ),
            Self::ZeroDataItemId { offset } => write!(
                formatter,
                "zero BIE data-item ID at 0x{:016x} is reserved for the file terminator",
                offset.get()
            ),
            Self::TruncatedStoredData {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "truncated BIE stored data at 0x{:016x}: needed {needed} bytes, available {available}",
                offset.get()
            ),
            Self::OffsetOverflow { offset, record_len } => write!(
                formatter,
                "BIE record offset overflow for {record_len} bytes at 0x{:016x}",
                offset.get()
            ),
        }
    }
}

impl Error for BieRecordParseError {}

/// A failure to parse one strict, complete BIE file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BieFileParseError {
    /// A non-terminator record was malformed or incomplete.
    Record(BieRecordParseError),
    /// The input ended at a record boundary without a terminal zero word.
    MissingTerminator {
        /// Absolute offset where the terminator was required.
        offset: FileOffset,
    },
    /// Bytes remained after a terminal zero word.
    TrailingData {
        /// Absolute offset of the first trailing byte.
        offset: FileOffset,
        /// Number of bytes following the terminator.
        trailing_bytes: usize,
    },
    /// An absolute file offset could not be represented by `u64`.
    OffsetOverflow {
        /// Absolute offset before the failed advance.
        offset: FileOffset,
        /// Number of bytes in the failed advance.
        byte_count: usize,
    },
}

impl fmt::Display for BieFileParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Record(error) => error.fmt(formatter),
            Self::MissingTerminator { offset } => write!(
                formatter,
                "missing BIE file terminator at 0x{:016x}",
                offset.get()
            ),
            Self::TrailingData {
                offset,
                trailing_bytes,
            } => {
                let unit = if *trailing_bytes == 1 {
                    "byte"
                } else {
                    "bytes"
                };
                write!(
                    formatter,
                    "{trailing_bytes} trailing {unit} after BIE terminator at 0x{:016x}",
                    offset.get()
                )
            }
            Self::OffsetOverflow { offset, byte_count } => write!(
                formatter,
                "BIE file offset overflow advancing {byte_count} bytes from 0x{:016x}",
                offset.get()
            ),
        }
    }
}

impl Error for BieFileParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Record(error) => Some(error),
            Self::MissingTerminator { .. }
            | Self::TrailingData { .. }
            | Self::OffsetOverflow { .. } => None,
        }
    }
}

impl From<BieRecordParseError> for BieFileParseError {
    fn from(error: BieRecordParseError) -> Self {
        Self::Record(error)
    }
}

/// Parses a strict, complete BIE byte slice.
///
/// Records are chained using their encoded lengths until a four-byte zero
/// sentinel is found. The sentinel must be the final bytes in `input`.
/// Recovery and resynchronization are deliberately not framing behavior.
pub fn parse_file(input: &[u8], file_offset: FileOffset) -> Result<BieFile<'_>, BieFileParseError> {
    let mut records = Vec::new();
    let mut cursor = 0_usize;

    loop {
        let current_offset = checked_file_offset(file_offset, cursor)?;
        let remaining = &input[cursor..];

        if remaining.is_empty() {
            return Err(BieFileParseError::MissingTerminator {
                offset: current_offset,
            });
        }

        if remaining.len() >= FILE_TERMINATOR_LEN
            && remaining[..FILE_TERMINATOR_LEN] == [0; FILE_TERMINATOR_LEN]
        {
            let trailing_bytes = remaining.len() - FILE_TERMINATOR_LEN;
            let terminator_end = checked_file_offset(current_offset, FILE_TERMINATOR_LEN)?;
            if trailing_bytes != 0 {
                return Err(BieFileParseError::TrailingData {
                    offset: terminator_end,
                    trailing_bytes,
                });
            }

            return Ok(BieFile {
                records,
                terminator_offset: current_offset,
                encoded_len: input.len(),
            });
        }

        let (record, consumed) = parse_record(remaining, current_offset)?;
        cursor = cursor
            .checked_add(consumed)
            .ok_or(BieFileParseError::OffsetOverflow {
                offset: current_offset,
                byte_count: consumed,
            })?;
        records.push(record);
    }
}

/// Parses one non-terminator BIE record from the beginning of `input`.
///
/// The returned byte count is derived from the encoded low-16-bit stored-data
/// length. Bytes after that count are not inspected. A file parser can use the
/// count to advance to the next record boundary. A zero data-item ID is
/// rejected here because recognizing the four-byte file terminator is a
/// file-level responsibility.
pub fn parse_record(
    input: &[u8],
    file_offset: FileOffset,
) -> Result<(BieRecord<'_>, usize), BieRecordParseError> {
    if input.len() < RECORD_HEADER_LEN {
        return Err(BieRecordParseError::TruncatedHeader {
            offset: file_offset,
            needed: RECORD_HEADER_LEN,
            available: input.len(),
        });
    }

    let data_item_id = DataItemId::new(read_u32_be(input, 0));
    if data_item_id.get() == 0 {
        return Err(BieRecordParseError::ZeroDataItemId {
            offset: file_offset,
        });
    }

    let recorder_time = RecorderTime {
        seconds: read_u32_be(input, 4),
        microseconds: read_u32_be(input, 8),
    };
    let status_and_length = StatusAndLength(read_u32_be(input, 12));
    let data_length = status_and_length.data_length();
    let record_len =
        RECORD_HEADER_LEN
            .checked_add(data_length)
            .ok_or(BieRecordParseError::OffsetOverflow {
                offset: file_offset,
                record_len: usize::MAX,
            })?;

    let stored_data_offset = file_offset
        .get()
        .checked_add(RECORD_HEADER_LEN as u64)
        .ok_or(BieRecordParseError::OffsetOverflow {
            offset: file_offset,
            record_len,
        })?;
    file_offset.get().checked_add(record_len as u64).ok_or(
        BieRecordParseError::OffsetOverflow {
            offset: file_offset,
            record_len,
        },
    )?;

    if input.len() < record_len {
        return Err(BieRecordParseError::TruncatedStoredData {
            offset: FileOffset::new(stored_data_offset),
            needed: data_length,
            available: input.len() - RECORD_HEADER_LEN,
        });
    }

    let stored_data = &input[RECORD_HEADER_LEN..record_len];
    Ok((
        BieRecord {
            file_offset,
            data_item_id,
            recorder_time,
            status_and_length,
            stored_data,
        },
        record_len,
    ))
}

fn read_u32_be(input: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(
        input[offset..offset + 4]
            .try_into()
            .expect("record header length was checked"),
    )
}

fn checked_file_offset(
    offset: FileOffset,
    byte_count: usize,
) -> Result<FileOffset, BieFileParseError> {
    let byte_count_u64 = u64::try_from(byte_count)
        .map_err(|_| BieFileParseError::OffsetOverflow { offset, byte_count })?;
    offset
        .get()
        .checked_add(byte_count_u64)
        .map(FileOffset::new)
        .ok_or(BieFileParseError::OffsetOverflow { offset, byte_count })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record_bytes(
        data_item_id: u32,
        seconds: u32,
        microseconds: u32,
        status_and_length: u32,
        stored_data: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(RECORD_HEADER_LEN + stored_data.len());
        bytes.extend_from_slice(&data_item_id.to_be_bytes());
        bytes.extend_from_slice(&seconds.to_be_bytes());
        bytes.extend_from_slice(&microseconds.to_be_bytes());
        bytes.extend_from_slice(&status_and_length.to_be_bytes());
        bytes.extend_from_slice(stored_data);
        bytes
    }

    /// Requirements: L3-BIE-001, L3-BIE-002, L3-BIE-003, L3-BIE-006,
    /// Requirements: L3-BIE-007, L3-BIE-008, L3-PRO-001, L3-TIM-001,
    /// Requirements: L3-TIM-002
    #[test]
    fn parses_explicit_fields_variable_length_and_unknown_id() {
        let mut bytes = record_bytes(
            0xDEAD_BEEF,
            0xFEDC_BA98,
            1_234_567,
            0x4000_0003,
            &[0xAA, 0xBB, 0xCC],
        );
        bytes.extend_from_slice(&[0xDD, 0xEE]);

        let (record, consumed) =
            parse_record(&bytes, FileOffset::new(0x120)).expect("record should parse");

        assert_eq!(consumed, 19);
        assert_eq!(record.encoded_len(), 19);
        assert_eq!(record.file_offset(), FileOffset::new(0x120));
        assert_eq!(record.data_item_id(), DataItemId::new(0xDEAD_BEEF));
        assert_eq!(record.recorder_time().seconds(), 0xFEDC_BA98);
        assert_eq!(record.recorder_time().microseconds(), 1_234_567);
        assert_eq!(record.status_and_length().raw(), 0x4000_0003);
        assert_eq!(record.status_and_length().data_length(), 3);
        assert_eq!(record.status_and_length().unresolved_flags(), 0x4000_0000);
        assert_eq!(record.stored_data(), [0xAA, 0xBB, 0xCC]);
    }

    /// Requirements: L3-BIE-003, L3-BIE-008
    #[test]
    fn accepts_a_nonzero_id_with_zero_length_stored_data() {
        let bytes = record_bytes(7, 1, 2, 0xABCD_0000, &[]);

        let (record, consumed) =
            parse_record(&bytes, FileOffset::new(0)).expect("zero-length body is valid");

        assert_eq!(consumed, RECORD_HEADER_LEN);
        assert!(record.stored_data().is_empty());
        assert_eq!(record.status_and_length().unresolved_flags(), 0xABCD_0000);
    }

    /// Requirements: L3-BIE-005
    #[test]
    fn reports_a_truncated_header() {
        let error =
            parse_record(&[0x12; 15], FileOffset::new(0x80)).expect_err("partial header must fail");

        assert_eq!(
            error,
            BieRecordParseError::TruncatedHeader {
                offset: FileOffset::new(0x80),
                needed: 16,
                available: 15,
            }
        );
    }

    /// Requirements: L3-BIE-002, L3-BIE-005
    #[test]
    fn reports_declared_and_available_body_sizes() {
        let bytes = record_bytes(1, 2, 3, 4, &[0xAA, 0xBB]);
        let error =
            parse_record(&bytes, FileOffset::new(0x100)).expect_err("short stored data must fail");

        assert_eq!(
            error,
            BieRecordParseError::TruncatedStoredData {
                offset: FileOffset::new(0x110),
                needed: 4,
                available: 2,
            }
        );
    }

    #[test]
    fn rejects_zero_data_item_id_as_a_record() {
        let bytes = record_bytes(0, 0, 0, 0, &[]);

        let error = parse_record(&bytes, FileOffset::new(0x40))
            .expect_err("zero identifies the file terminator, not a record");

        assert_eq!(
            error,
            BieRecordParseError::ZeroDataItemId {
                offset: FileOffset::new(0x40),
            }
        );
    }

    /// Requirements: L3-BIE-002, L3-BIE-007
    #[test]
    fn rejects_an_unrepresentable_record_end_offset() {
        let bytes = record_bytes(1, 2, 3, 0, &[]);

        let error = parse_record(&bytes, FileOffset::new(u64::MAX - 15))
            .expect_err("record end offset must not wrap");

        assert_eq!(
            error,
            BieRecordParseError::OffsetOverflow {
                offset: FileOffset::new(u64::MAX - 15),
                record_len: 16,
            }
        );
    }

    /// Requirements: L3-BIE-003, L3-BIE-004, L3-BIE-007, L3-BIE-008
    #[test]
    fn parses_variable_length_records_through_the_file_terminator() {
        let first = record_bytes(0x1111_1111, 10, 20, 0x4000_0001, &[0xAA]);
        let second = record_bytes(0x2222_2222, 30, 40, 0x0000_0002, &[0xBB, 0xCC]);
        let terminator_relative_offset = first.len() + second.len();
        let mut bytes = first;
        bytes.extend_from_slice(&second);
        bytes.extend_from_slice(&[0; FILE_TERMINATOR_LEN]);

        let file = parse_file(&bytes, FileOffset::new(0x100)).expect("complete file parses");

        assert_eq!(file.records().len(), 2);
        assert_eq!(file.records()[0].file_offset(), FileOffset::new(0x100));
        assert_eq!(file.records()[0].stored_data(), [0xAA]);
        assert_eq!(file.records()[1].file_offset(), FileOffset::new(0x100 + 17));
        assert_eq!(file.records()[1].stored_data(), [0xBB, 0xCC]);
        assert_eq!(
            file.terminator_offset(),
            FileOffset::new(
                0x100
                    + u64::try_from(terminator_relative_offset).expect("test file offset fits u64")
            )
        );
        assert_eq!(file.encoded_len(), bytes.len());
        assert!(!file.is_empty());
    }

    /// Requirements: L3-BIE-004, L3-TST-001
    #[test]
    fn accepts_the_sentinel_only_empty_form() {
        let bytes = [0; FILE_TERMINATOR_LEN];

        let file = parse_file(&bytes, FileOffset::new(0x20)).expect("empty form parses");

        assert!(file.is_empty());
        assert_eq!(file.terminator_offset(), FileOffset::new(0x20));
        assert_eq!(file.encoded_len(), FILE_TERMINATOR_LEN);
    }

    /// Requirements: L3-BIE-004
    #[test]
    fn reports_trailing_data_after_the_terminator() {
        let bytes = [0, 0, 0, 0, 0xAA, 0xBB];

        let error = parse_file(&bytes, FileOffset::new(0x80))
            .expect_err("trailing bytes must not be accepted");

        assert_eq!(
            error,
            BieFileParseError::TrailingData {
                offset: FileOffset::new(0x84),
                trailing_bytes: 2,
            }
        );
    }

    /// Requirements: L3-BIE-005
    #[test]
    fn reports_a_missing_terminator_at_a_record_boundary() {
        let bytes = record_bytes(1, 2, 3, 0, &[]);

        let error = parse_file(&bytes, FileOffset::new(0x100))
            .expect_err("a complete record does not replace the terminator");

        assert_eq!(
            error,
            BieFileParseError::MissingTerminator {
                offset: FileOffset::new(0x110),
            }
        );
    }

    /// Requirements: L3-BIE-005
    #[test]
    fn reports_a_partial_terminal_word_as_a_truncated_header() {
        let error = parse_file(&[0, 0, 0], FileOffset::new(0x40))
            .expect_err("a partial zero word is not a terminator");

        assert_eq!(
            error,
            BieFileParseError::Record(BieRecordParseError::TruncatedHeader {
                offset: FileOffset::new(0x40),
                needed: RECORD_HEADER_LEN,
                available: 3,
            })
        );
    }

    /// Requirements: L3-BIE-002, L3-BIE-005
    #[test]
    fn preserves_stored_data_truncation_details_at_file_level() {
        let bytes = record_bytes(1, 2, 3, 4, &[0xAA, 0xBB]);

        let error = parse_file(&bytes, FileOffset::new(0x100))
            .expect_err("an incomplete record body makes the file malformed");

        assert_eq!(
            error,
            BieFileParseError::Record(BieRecordParseError::TruncatedStoredData {
                offset: FileOffset::new(0x110),
                needed: 4,
                available: 2,
            })
        );
    }

    /// Requirements: L3-BIE-002, L3-BIE-007
    #[test]
    fn rejects_an_unrepresentable_terminator_end_offset() {
        let bytes = [0; FILE_TERMINATOR_LEN];

        let error = parse_file(&bytes, FileOffset::new(u64::MAX - 3))
            .expect_err("the terminator end offset must not wrap");

        assert_eq!(
            error,
            BieFileParseError::OffsetOverflow {
                offset: FileOffset::new(u64::MAX - 3),
                byte_count: FILE_TERMINATOR_LEN,
            }
        );
    }
}
