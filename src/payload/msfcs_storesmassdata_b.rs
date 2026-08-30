//! Wire decoder and provisional semantics for `msfcs_storesmassdata_b`.
//!
//! The supplied field names, primitive types, and byte ranges are implemented
//! exactly. Raw values remain authoritative when provisional Boolean,
//! validity, and system-time interpretations produce derived values or
//! warnings. Engineering units, coordinate references, acronym expansions,
//! mass-data group meanings, and the system-tick epoch remain unresolved.

use super::{
    FieldLayoutError, PayloadByteOrder, PayloadDefinition, PayloadFieldDefinition, PayloadWireType,
    validate_field_layout,
};
use std::error::Error;
use std::fmt;

/// Stable payload name from the corrected recorder summary.
pub const NAME: &str = "msfcs_storesmassdata_b";

/// Aero1394's version for the supplied field layout.
///
/// This is not presented as a revision of the unconfirmed source document.
pub const DEFINITION_VERSION: &str = "layout-v1";

/// Data-item identity associated with the supplied layout.
pub const DATA_ITEM_ID: u32 = 0x0000_5D04;

/// Exact number of application bytes in the supplied layout.
pub const PAYLOAD_SIZE: usize = 92;

/// Byte order corroborated by the supplied capture values.
pub const BYTE_ORDER: PayloadByteOrder = PayloadByteOrder::BigEndian;

/// Provisional system-tick rate derived from `106.25 MHz * 2^7`.
pub const NOMINAL_SYSTEM_TICK_RATE_HZ: u64 = 13_600_000_000;

/// Provisional duration of one system tick, in seconds.
pub const NOMINAL_SYSTEM_TICK_PERIOD_SECONDS: f64 = 1.0 / NOMINAL_SYSTEM_TICK_RATE_HZ as f64;

const TIMESTAMP_OFFSET: usize = 0;
const MESSAGE_VALID_OFFSET: usize = 8;
const EOTS_PRESENT_OFFSET: usize = 9;
const SPARE_BYTE_OFFSET: usize = 10;
const CM_PRESENT_OFFSET: usize = 11;
const CURRENT_STORES_MASS_DATA_OFFSET: usize = 12;
const POST_EJ_STORES_MASS_DATA_OFFSET: usize = 52;

const WEIGHT_OFFSET: usize = 0;
const CG_FS_OFFSET: usize = 4;
const CG_BL_OFFSET: usize = 8;
const CG_WL_OFFSET: usize = 12;
const IXX_OFFSET: usize = 16;
const IYY_OFFSET: usize = 20;
const IZZ_OFFSET: usize = 24;
const IXY_OFFSET: usize = 28;
const IYZ_OFFSET: usize = 32;
const IXZ_OFFSET: usize = 36;

/// All 25 supplied fields in authoritative definition order.
pub const FIELD_DEFINITIONS: [PayloadFieldDefinition; 25] = [
    PayloadFieldDefinition::new("TimeStamp", TIMESTAMP_OFFSET, PayloadWireType::Unsigned64),
    PayloadFieldDefinition::new(
        "MessageValid",
        MESSAGE_VALID_OFFSET,
        PayloadWireType::Boolean8,
    ),
    PayloadFieldDefinition::new(
        "EOTS_Present",
        EOTS_PRESENT_OFFSET,
        PayloadWireType::Boolean8,
    ),
    PayloadFieldDefinition::new("spare_byte", SPARE_BYTE_OFFSET, PayloadWireType::Boolean8),
    PayloadFieldDefinition::new("CM_Present", CM_PRESENT_OFFSET, PayloadWireType::Boolean8),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Weight",
        CURRENT_STORES_MASS_DATA_OFFSET + WEIGHT_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Cg_FS",
        CURRENT_STORES_MASS_DATA_OFFSET + CG_FS_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Cg_BL",
        CURRENT_STORES_MASS_DATA_OFFSET + CG_BL_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Cg_WL",
        CURRENT_STORES_MASS_DATA_OFFSET + CG_WL_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Ixx",
        CURRENT_STORES_MASS_DATA_OFFSET + IXX_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Iyy",
        CURRENT_STORES_MASS_DATA_OFFSET + IYY_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Izz",
        CURRENT_STORES_MASS_DATA_OFFSET + IZZ_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Ixy",
        CURRENT_STORES_MASS_DATA_OFFSET + IXY_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Iyz",
        CURRENT_STORES_MASS_DATA_OFFSET + IYZ_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "CurrentStoresMassData.Ixz",
        CURRENT_STORES_MASS_DATA_OFFSET + IXZ_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Weight",
        POST_EJ_STORES_MASS_DATA_OFFSET + WEIGHT_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Cg_FS",
        POST_EJ_STORES_MASS_DATA_OFFSET + CG_FS_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Cg_BL",
        POST_EJ_STORES_MASS_DATA_OFFSET + CG_BL_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Cg_WL",
        POST_EJ_STORES_MASS_DATA_OFFSET + CG_WL_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Ixx",
        POST_EJ_STORES_MASS_DATA_OFFSET + IXX_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Iyy",
        POST_EJ_STORES_MASS_DATA_OFFSET + IYY_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Izz",
        POST_EJ_STORES_MASS_DATA_OFFSET + IZZ_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Ixy",
        POST_EJ_STORES_MASS_DATA_OFFSET + IXY_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Iyz",
        POST_EJ_STORES_MASS_DATA_OFFSET + IYZ_OFFSET,
        PayloadWireType::Float32,
    ),
    PayloadFieldDefinition::new(
        "PostEJStoresMassData.Ixz",
        POST_EJ_STORES_MASS_DATA_OFFSET + IXZ_OFFSET,
        PayloadWireType::Float32,
    ),
];

/// Built-in registry definition for the supplied layout.
pub const DEFINITION: PayloadDefinition = PayloadDefinition::new(
    NAME,
    DEFINITION_VERSION,
    DATA_ITEM_ID,
    PAYLOAD_SIZE,
    BYTE_ORDER,
)
.with_fields(&FIELD_DEFINITIONS);

/// Source/application system ticks with an unconfirmed epoch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SystemTicks(u64);

impl SystemTicks {
    /// Creates a raw tick value without assigning a time epoch.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the exact unsigned wire value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns provisional elapsed seconds at the nominal 13.6 GHz rate.
    ///
    /// System startup is the current epoch hypothesis, but it is not
    /// confirmed. This value must not be interpreted as calendar time.
    #[must_use]
    pub fn provisional_elapsed_seconds(self) -> f64 {
        self.0 as f64 / NOMINAL_SYSTEM_TICK_RATE_HZ as f64
    }
}

/// One source-designated Boolean byte retaining its exact wire value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RawBooleanByte(u8);

impl RawBooleanByte {
    /// Creates a raw Boolean-designated byte without interpreting it.
    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// Returns the exact byte value.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Interprets the strict provisional encoding `0 = false`, `1 = true`.
    ///
    /// Any other byte is retained but has no Boolean interpretation.
    #[must_use]
    pub const fn as_bool(self) -> Option<bool> {
        match self.0 {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

/// One IEEE-754 `f32` wire value retaining its exact bits.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RawF32(u32);

impl RawF32 {
    /// Creates a value from the exact IEEE-754 bits.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Returns the exact IEEE-754 wire bits.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Interprets the retained bits as an unscaled `f32` value.
    #[must_use]
    pub const fn value(self) -> f32 {
        f32::from_bits(self.0)
    }
}

/// Ten unscaled mass-property values from one supplied field group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoresMassData {
    weight: RawF32,
    cg_fs: RawF32,
    cg_bl: RawF32,
    cg_wl: RawF32,
    ixx: RawF32,
    iyy: RawF32,
    izz: RawF32,
    ixy: RawF32,
    iyz: RawF32,
    ixz: RawF32,
}

impl StoresMassData {
    /// Returns the raw `Weight` field.
    #[must_use]
    pub const fn weight(self) -> RawF32 {
        self.weight
    }

    /// Returns the raw `Cg_FS` field.
    #[must_use]
    pub const fn cg_fs(self) -> RawF32 {
        self.cg_fs
    }

    /// Returns the raw `Cg_BL` field.
    #[must_use]
    pub const fn cg_bl(self) -> RawF32 {
        self.cg_bl
    }

    /// Returns the raw `Cg_WL` field.
    #[must_use]
    pub const fn cg_wl(self) -> RawF32 {
        self.cg_wl
    }

    /// Returns the raw `Ixx` field.
    #[must_use]
    pub const fn ixx(self) -> RawF32 {
        self.ixx
    }

    /// Returns the raw `Iyy` field.
    #[must_use]
    pub const fn iyy(self) -> RawF32 {
        self.iyy
    }

    /// Returns the raw `Izz` field.
    #[must_use]
    pub const fn izz(self) -> RawF32 {
        self.izz
    }

    /// Returns the raw `Ixy` field.
    #[must_use]
    pub const fn ixy(self) -> RawF32 {
        self.ixy
    }

    /// Returns the raw `Iyz` field.
    #[must_use]
    pub const fn iyz(self) -> RawF32 {
        self.iyz
    }

    /// Returns the raw `Ixz` field.
    #[must_use]
    pub const fn ixz(self) -> RawF32 {
        self.ixz
    }

    const fn values(self) -> [RawF32; 10] {
        [
            self.weight,
            self.cg_fs,
            self.cg_bl,
            self.cg_wl,
            self.ixx,
            self.iyy,
            self.izz,
            self.ixy,
            self.iyz,
            self.ixz,
        ]
    }
}

/// A non-fatal semantic finding on a successfully decoded payload.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StoresMassWarning {
    /// `MessageValid` is provisionally interpreted as false.
    MessageInvalid,
    /// A Boolean-designated byte is neither `0` nor `1`.
    InvalidBooleanEncoding {
        /// Authoritative field name.
        field: &'static str,
        /// Exact unexpected wire byte.
        value: u8,
    },
    /// The reserved `spare_byte` is expected to remain zero.
    ReservedByteNonZero {
        /// Exact unexpected wire byte.
        value: u8,
    },
    /// An unscaled IEEE-754 field contains NaN or infinity.
    NonFiniteFloat {
        /// Authoritative field name.
        field: &'static str,
        /// Exact IEEE-754 wire bits.
        bits: u32,
    },
}

impl fmt::Display for StoresMassWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MessageInvalid => formatter.write_str("message_invalid"),
            Self::InvalidBooleanEncoding { field, value } => {
                write!(formatter, "invalid_boolean_encoding:{field}=0x{value:02X}")
            }
            Self::ReservedByteNonZero { value } => {
                write!(formatter, "reserved_byte_nonzero:spare_byte=0x{value:02X}")
            }
            Self::NonFiniteFloat { field, bits } => {
                write!(formatter, "non_finite_float:{field}=0x{bits:08X}")
            }
        }
    }
}

/// All raw fields decoded from one exact 92-byte Stores Mass payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MsfcsStoresMassDataB<'a> {
    system_ticks: SystemTicks,
    message_valid: RawBooleanByte,
    eots_present: RawBooleanByte,
    spare_byte: RawBooleanByte,
    cm_present: RawBooleanByte,
    current_stores_mass_data: StoresMassData,
    post_ej_stores_mass_data: StoresMassData,
    raw_bytes: &'a [u8],
}

impl<'a> MsfcsStoresMassDataB<'a> {
    /// Returns the `TimeStamp` field as raw system ticks.
    #[must_use]
    pub const fn system_ticks(self) -> SystemTicks {
        self.system_ticks
    }

    /// Returns the raw `MessageValid` byte.
    #[must_use]
    pub const fn message_valid(self) -> RawBooleanByte {
        self.message_valid
    }

    /// Returns the raw `EOTS_Present` byte.
    #[must_use]
    pub const fn eots_present(self) -> RawBooleanByte {
        self.eots_present
    }

    /// Returns the raw reserved `spare_byte` byte.
    #[must_use]
    pub const fn spare_byte(self) -> RawBooleanByte {
        self.spare_byte
    }

    /// Returns the raw `CM_Present` byte.
    #[must_use]
    pub const fn cm_present(self) -> RawBooleanByte {
        self.cm_present
    }

    /// Returns the ten raw `CurrentStoresMassData` fields.
    #[must_use]
    pub const fn current_stores_mass_data(self) -> StoresMassData {
        self.current_stores_mass_data
    }

    /// Returns the ten raw `PostEJStoresMassData` fields.
    #[must_use]
    pub const fn post_ej_stores_mass_data(self) -> StoresMassData {
        self.post_ej_stores_mass_data
    }

    /// Returns all 92 bytes supplied to the decoder.
    #[must_use]
    pub const fn raw_bytes(self) -> &'a [u8] {
        self.raw_bytes
    }

    /// Returns all non-fatal semantic findings in deterministic field order.
    ///
    /// Findings never suppress decoding. The raw fields and complete source
    /// bytes remain available even when this collection is non-empty.
    #[must_use]
    pub fn warnings(self) -> Vec<StoresMassWarning> {
        let mut warnings = Vec::new();

        if self.message_valid.as_bool() == Some(false) {
            warnings.push(StoresMassWarning::MessageInvalid);
        }

        for (field, value) in [
            ("MessageValid", self.message_valid),
            ("EOTS_Present", self.eots_present),
            ("spare_byte", self.spare_byte),
            ("CM_Present", self.cm_present),
        ] {
            if value.as_bool().is_none() {
                warnings.push(StoresMassWarning::InvalidBooleanEncoding {
                    field,
                    value: value.get(),
                });
            }
        }

        if self.spare_byte.get() != 0 {
            warnings.push(StoresMassWarning::ReservedByteNonZero {
                value: self.spare_byte.get(),
            });
        }

        for (definition, value) in FIELD_DEFINITIONS[5..].iter().zip(
            self.current_stores_mass_data
                .values()
                .into_iter()
                .chain(self.post_ej_stores_mass_data.values()),
        ) {
            if !value.value().is_finite() {
                warnings.push(StoresMassWarning::NonFiniteFloat {
                    field: definition.name(),
                    bits: value.bits(),
                });
            }
        }

        warnings
    }
}

/// Failure to decode the supplied Stores Mass layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// The application payload does not have the exact declared size.
    InvalidLength {
        /// Required byte count.
        expected: usize,
        /// Supplied byte count.
        actual: usize,
    },
    /// The built-in field declarations are internally invalid.
    InvalidDefinition(FieldLayoutError),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => write!(
                formatter,
                "payload {NAME}@{DEFINITION_VERSION} requires {expected} bytes, received {actual}"
            ),
            Self::InvalidDefinition(error) => write!(
                formatter,
                "payload {NAME}@{DEFINITION_VERSION} has an invalid definition: {error}"
            ),
        }
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLength { .. } => None,
            Self::InvalidDefinition(error) => Some(error),
        }
    }
}

/// Decodes all supplied fields using checked, explicit big-endian reads.
pub fn decode(input: &[u8]) -> Result<MsfcsStoresMassDataB<'_>, DecodeError> {
    if input.len() != PAYLOAD_SIZE {
        return Err(DecodeError::InvalidLength {
            expected: PAYLOAD_SIZE,
            actual: input.len(),
        });
    }
    validate_field_layout(PAYLOAD_SIZE, &FIELD_DEFINITIONS)
        .map_err(DecodeError::InvalidDefinition)?;

    let invalid_length = || DecodeError::InvalidLength {
        expected: PAYLOAD_SIZE,
        actual: input.len(),
    };

    Ok(MsfcsStoresMassDataB {
        system_ticks: SystemTicks::new(
            read_u64_be(input, TIMESTAMP_OFFSET).ok_or_else(invalid_length)?,
        ),
        message_valid: RawBooleanByte::new(
            read_u8(input, MESSAGE_VALID_OFFSET).ok_or_else(invalid_length)?,
        ),
        eots_present: RawBooleanByte::new(
            read_u8(input, EOTS_PRESENT_OFFSET).ok_or_else(invalid_length)?,
        ),
        spare_byte: RawBooleanByte::new(
            read_u8(input, SPARE_BYTE_OFFSET).ok_or_else(invalid_length)?,
        ),
        cm_present: RawBooleanByte::new(
            read_u8(input, CM_PRESENT_OFFSET).ok_or_else(invalid_length)?,
        ),
        current_stores_mass_data: decode_mass_data(input, CURRENT_STORES_MASS_DATA_OFFSET)
            .ok_or_else(invalid_length)?,
        post_ej_stores_mass_data: decode_mass_data(input, POST_EJ_STORES_MASS_DATA_OFFSET)
            .ok_or_else(invalid_length)?,
        raw_bytes: input,
    })
}

fn decode_mass_data(input: &[u8], base_offset: usize) -> Option<StoresMassData> {
    Some(StoresMassData {
        weight: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(WEIGHT_OFFSET)?)?),
        cg_fs: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(CG_FS_OFFSET)?)?),
        cg_bl: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(CG_BL_OFFSET)?)?),
        cg_wl: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(CG_WL_OFFSET)?)?),
        ixx: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(IXX_OFFSET)?)?),
        iyy: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(IYY_OFFSET)?)?),
        izz: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(IZZ_OFFSET)?)?),
        ixy: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(IXY_OFFSET)?)?),
        iyz: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(IYZ_OFFSET)?)?),
        ixz: RawF32::from_bits(read_u32_be(input, base_offset.checked_add(IXZ_OFFSET)?)?),
    })
}

fn read_u8(input: &[u8], offset: usize) -> Option<u8> {
    input.get(offset).copied()
}

fn read_u32_be(input: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let bytes: [u8; 4] = input.get(offset..end)?.try_into().ok()?;
    Some(u32::from_be_bytes(bytes))
}

fn read_u64_be(input: &[u8], offset: usize) -> Option<u64> {
    let end = offset.checked_add(8)?;
    let bytes: [u8; 8] = input.get(offset..end)?.try_into().ok()?;
    Some(u64::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Requirements: L3-PAY-003, L3-PAY-008, L3-PAY-013
    #[test]
    fn supplied_fields_cover_all_92_bytes_without_gaps_or_overlaps() {
        let validation = validate_field_layout(PAYLOAD_SIZE, &FIELD_DEFINITIONS)
            .expect("supplied field map must remain structurally valid");

        assert!(validation.gaps().is_empty());
        assert_eq!(DEFINITION.fields(), FIELD_DEFINITIONS);
        assert_eq!(FIELD_DEFINITIONS[0].name(), "TimeStamp");
        assert_eq!(FIELD_DEFINITIONS[24].name(), "PostEJStoresMassData.Ixz");
    }

    /// Requirements: L3-PAY-007, L3-PAY-013
    #[test]
    fn rejects_short_and_long_payloads() {
        assert_eq!(
            decode(&[0; PAYLOAD_SIZE - 1]),
            Err(DecodeError::InvalidLength {
                expected: PAYLOAD_SIZE,
                actual: PAYLOAD_SIZE - 1,
            })
        );
        assert_eq!(
            decode(&[0; PAYLOAD_SIZE + 1]),
            Err(DecodeError::InvalidLength {
                expected: PAYLOAD_SIZE,
                actual: PAYLOAD_SIZE + 1,
            })
        );
    }

    /// Requirements: L3-TIM-007, L3-PAY-007, L3-PAY-013
    #[test]
    fn decodes_explicit_big_endian_boundaries_and_preserves_bits() {
        let mut input = [0; PAYLOAD_SIZE];
        input[TIMESTAMP_OFFSET..TIMESTAMP_OFFSET + 8]
            .copy_from_slice(&0x0102_0304_0506_0708_u64.to_be_bytes());
        input[MESSAGE_VALID_OFFSET] = 0xA5;
        input[EOTS_PRESENT_OFFSET] = 0x5A;
        input[CURRENT_STORES_MASS_DATA_OFFSET..CURRENT_STORES_MASS_DATA_OFFSET + 4]
            .copy_from_slice(&0x7FC0_1234_u32.to_be_bytes());
        input[POST_EJ_STORES_MASS_DATA_OFFSET + IXZ_OFFSET
            ..POST_EJ_STORES_MASS_DATA_OFFSET + IXZ_OFFSET + 4]
            .copy_from_slice(&0x8000_0000_u32.to_be_bytes());

        let decoded = decode(&input).expect("exact test payload decodes");

        assert_eq!(decoded.system_ticks().get(), 0x0102_0304_0506_0708);
        assert_eq!(decoded.message_valid().get(), 0xA5);
        assert_eq!(decoded.eots_present().get(), 0x5A);
        assert_eq!(
            decoded.current_stores_mass_data().weight().bits(),
            0x7FC0_1234
        );
        assert!(decoded.current_stores_mass_data().weight().value().is_nan());
        assert_eq!(decoded.post_ej_stores_mass_data().ixz().bits(), 0x8000_0000);
        assert_eq!(decoded.raw_bytes(), input);
    }
}
