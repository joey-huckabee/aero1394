use std::error::Error;
use std::fmt;

/// Primitive wire representation declared for a payload field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadWireType {
    /// Unsigned big- or little-endian 64-bit integer.
    Unsigned64,
    /// One byte designated as Boolean by its source definition.
    Boolean8,
    /// IEEE-754 32-bit floating-point bit pattern.
    Float32,
}

impl PayloadWireType {
    /// Returns the exact number of bytes occupied by this wire type.
    #[must_use]
    pub const fn byte_width(self) -> usize {
        match self {
            Self::Unsigned64 => 8,
            Self::Boolean8 => 1,
            Self::Float32 => 4,
        }
    }
}

/// One named, byte-aligned field in a payload definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PayloadFieldDefinition {
    name: &'static str,
    byte_offset: usize,
    wire_type: PayloadWireType,
}

impl PayloadFieldDefinition {
    /// Declares a field at an explicit byte offset.
    #[must_use]
    pub const fn new(name: &'static str, byte_offset: usize, wire_type: PayloadWireType) -> Self {
        Self {
            name,
            byte_offset,
            wire_type,
        }
    }

    /// Returns the authoritative field name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the zero-based byte offset in the application payload.
    #[must_use]
    pub const fn byte_offset(self) -> usize {
        self.byte_offset
    }

    /// Returns the declared primitive wire representation.
    #[must_use]
    pub const fn wire_type(self) -> PayloadWireType {
        self.wire_type
    }

    /// Returns the exact number of bytes occupied by this field.
    #[must_use]
    pub const fn byte_width(self) -> usize {
        self.wire_type.byte_width()
    }
}

/// A half-open byte range `[start, end)` in a payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PayloadByteRange {
    start: usize,
    end: usize,
}

impl PayloadByteRange {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns the inclusive start offset.
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive end offset.
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns the number of bytes in this range.
    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    /// Returns whether the range contains no bytes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// Successful validation of a payload field layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldLayoutValidation {
    gaps: Vec<PayloadByteRange>,
}

impl FieldLayoutValidation {
    /// Returns every uncovered byte range in ascending offset order.
    #[must_use]
    pub fn gaps(&self) -> &[PayloadByteRange] {
        &self.gaps
    }
}

/// A structurally invalid payload field declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldLayoutError {
    /// Adding the field width to its offset overflowed `usize`.
    OffsetOverflow {
        /// Field whose range could not be represented.
        field: &'static str,
        /// Declared byte offset.
        byte_offset: usize,
        /// Declared byte width.
        byte_width: usize,
    },
    /// A field extends beyond the declared payload size.
    OutOfBounds {
        /// Field whose range is invalid.
        field: &'static str,
        /// Declared byte offset.
        byte_offset: usize,
        /// Declared byte width.
        byte_width: usize,
        /// Total payload size used for validation.
        payload_size: usize,
    },
    /// Two declared field ranges overlap.
    Overlap {
        /// Earlier field in definition order.
        first: &'static str,
        /// Later field in definition order.
        second: &'static str,
    },
}

impl fmt::Display for FieldLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OffsetOverflow {
                field,
                byte_offset,
                byte_width,
            } => write!(
                formatter,
                "payload field {field} range overflows: offset {byte_offset}, width {byte_width}"
            ),
            Self::OutOfBounds {
                field,
                byte_offset,
                byte_width,
                payload_size,
            } => write!(
                formatter,
                "payload field {field} is out of bounds: offset {byte_offset}, width {byte_width}, payload size {payload_size}"
            ),
            Self::Overlap { first, second } => {
                write!(formatter, "payload fields {first} and {second} overlap")
            }
        }
    }
}

impl Error for FieldLayoutError {}

#[derive(Clone, Copy)]
struct CheckedField {
    definition: PayloadFieldDefinition,
    end: usize,
}

/// Validates field bounds and overlap, and reports all documented gaps.
pub fn validate_field_layout(
    payload_size: usize,
    fields: &[PayloadFieldDefinition],
) -> Result<FieldLayoutValidation, FieldLayoutError> {
    let mut checked = Vec::with_capacity(fields.len());
    for field in fields {
        let end = field.byte_offset().checked_add(field.byte_width()).ok_or(
            FieldLayoutError::OffsetOverflow {
                field: field.name(),
                byte_offset: field.byte_offset(),
                byte_width: field.byte_width(),
            },
        )?;
        if end > payload_size {
            return Err(FieldLayoutError::OutOfBounds {
                field: field.name(),
                byte_offset: field.byte_offset(),
                byte_width: field.byte_width(),
                payload_size,
            });
        }
        checked.push(CheckedField {
            definition: *field,
            end,
        });
    }

    for (first_index, first) in checked.iter().enumerate() {
        for second in checked.iter().skip(first_index + 1) {
            if first.definition.byte_offset() < second.end
                && second.definition.byte_offset() < first.end
            {
                return Err(FieldLayoutError::Overlap {
                    first: first.definition.name(),
                    second: second.definition.name(),
                });
            }
        }
    }

    checked.sort_by_key(|field| (field.definition.byte_offset(), field.end));
    let mut gaps = Vec::new();
    let mut cursor = 0;
    for field in checked {
        if cursor < field.definition.byte_offset() {
            gaps.push(PayloadByteRange::new(
                cursor,
                field.definition.byte_offset(),
            ));
        }
        cursor = field.end;
    }
    if cursor < payload_size {
        gaps.push(PayloadByteRange::new(cursor, payload_size));
    }

    Ok(FieldLayoutValidation { gaps })
}

#[cfg(test)]
mod tests {
    use super::*;

    const U64_FIELD: PayloadFieldDefinition =
        PayloadFieldDefinition::new("wide", 0, PayloadWireType::Unsigned64);
    const U8_FIELD: PayloadFieldDefinition =
        PayloadFieldDefinition::new("byte", 8, PayloadWireType::Boolean8);

    /// Requirements: L3-PAY-008
    #[test]
    fn reports_uncovered_ranges_in_offset_order() {
        let validation = validate_field_layout(12, &[U8_FIELD, U64_FIELD])
            .expect("non-overlapping in-bounds fields are valid");

        assert_eq!(validation.gaps(), [PayloadByteRange::new(9, 12)].as_slice());
    }

    /// Requirements: L3-PAY-008
    #[test]
    fn rejects_overlapping_fields() {
        let overlapping = PayloadFieldDefinition::new("overlap", 7, PayloadWireType::Float32);

        assert_eq!(
            validate_field_layout(12, &[U64_FIELD, overlapping]),
            Err(FieldLayoutError::Overlap {
                first: "wide",
                second: "overlap",
            })
        );
    }

    /// Requirements: L3-PAY-008
    #[test]
    fn rejects_out_of_bounds_and_overflowing_ranges() {
        let out_of_bounds = PayloadFieldDefinition::new("outside", 9, PayloadWireType::Float32);
        let overflowing =
            PayloadFieldDefinition::new("overflow", usize::MAX, PayloadWireType::Unsigned64);

        assert!(matches!(
            validate_field_layout(12, &[out_of_bounds]),
            Err(FieldLayoutError::OutOfBounds { .. })
        ));
        assert!(matches!(
            validate_field_layout(usize::MAX, &[overflowing]),
            Err(FieldLayoutError::OffsetOverflow { .. })
        ));
    }
}
