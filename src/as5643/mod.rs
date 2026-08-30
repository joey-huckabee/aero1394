//! Evidence-backed AS5643 Anonymous Subscriber Message profiles.
//!
//! The current profile decodes the retained payload and trailer representation
//! established by the supplied BIE evidence. It does not decode an IEEE-1394
//! packet and does not assign application-specific meaning to payload bytes.

use std::error::Error;
use std::fmt;

/// Stable identifier for the current assumption-dependent profile.
pub const ASSUMED_AS5643B_V1_PROFILE_ID: &str = "aero1394-assumed-as5643b-v1";

/// Message ID supported by the current assumption-dependent profile.
pub const ASSUMED_AS5643B_V1_MESSAGE_ID: MessageId = MessageId::new(0x0000_5D04);

/// Number of application bytes declared by the current profile.
pub const ASSUMED_AS5643B_V1_APPLICATION_LEN: usize = 92;

/// Number of retained bytes after the omitted four-word ASM header.
pub const ASSUMED_AS5643B_V1_RETAINED_LEN: usize = 116;

const RESERVED_SECURITY: u32 = 0;
const NODE_ID: u32 = 0;
const PRIORITY_AND_PAYLOAD_LENGTH: u32 = 0x0000_0064;
const APPLICATION_START: usize = 8;
const APPLICATION_END: usize = APPLICATION_START + ASSUMED_AS5643B_V1_APPLICATION_LEN;
const STOF_TRANSMIT_OFFSET: usize = APPLICATION_END;
const STOF_RECEIVE_OFFSET: usize = STOF_TRANSMIT_OFFSET + 4;
const STOF_DATAPUMP_OFFSET: usize = STOF_RECEIVE_OFFSET + 4;
const VPC_OFFSET: usize = STOF_DATAPUMP_OFFSET + 4;

/// An AS5643 Anonymous Subscriber Message identifier.
///
/// This type is intentionally distinct from any input container's identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MessageId(u32);

impl MessageId {
    /// Creates a message identifier from its raw AS5643 value.
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

/// Explicit logical ASM words used to calculate a Vertical Parity Check.
///
/// `protected_bytes` begins with Health Status and ends with the STOF Datapump
/// Offset. It excludes both the reconstructed four-word ASM header and the VPC
/// word itself. The bytes must contain complete big-endian 32-bit words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VpcCalculationInputs<'a> {
    message_id: MessageId,
    reserved_security: u32,
    node_id: u32,
    priority_and_payload_length: u32,
    protected_bytes: &'a [u8],
}

impl<'a> VpcCalculationInputs<'a> {
    /// Creates a VPC input set from explicit logical ASM fields.
    #[must_use]
    pub const fn new(
        message_id: MessageId,
        reserved_security: u32,
        node_id: u32,
        priority_and_payload_length: u32,
        protected_bytes: &'a [u8],
    ) -> Self {
        Self {
            message_id,
            reserved_security,
            node_id,
            priority_and_payload_length,
            protected_bytes,
        }
    }

    /// Returns the logical AS5643 Message ID.
    #[must_use]
    pub const fn message_id(self) -> MessageId {
        self.message_id
    }

    /// Returns the raw reserved/security header word.
    #[must_use]
    pub const fn reserved_security(self) -> u32 {
        self.reserved_security
    }

    /// Returns the raw Node ID header word.
    #[must_use]
    pub const fn node_id(self) -> u32 {
        self.node_id
    }

    /// Returns the raw priority/payload-length header word.
    #[must_use]
    pub const fn priority_and_payload_length(self) -> u32 {
        self.priority_and_payload_length
    }

    /// Returns the XOR of the four logical ASM-header words.
    #[must_use]
    pub const fn header_xor(self) -> u32 {
        self.message_id.get()
            ^ self.reserved_security
            ^ self.node_id
            ^ self.priority_and_payload_length
    }

    /// Returns the exact bytes protected after the logical ASM header.
    #[must_use]
    pub const fn protected_bytes(self) -> &'a [u8] {
        self.protected_bytes
    }
}

/// A reason that VPC calculation could not consume the supplied inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VpcCalculationError {
    /// The protected region does not contain complete 32-bit words.
    ProtectedDataNotWordAligned {
        /// Number of protected bytes supplied.
        actual: usize,
    },
}

impl fmt::Display for VpcCalculationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtectedDataNotWordAligned { actual } => write!(
                formatter,
                "VPC calculation requires complete 32-bit words, received {actual} protected bytes"
            ),
        }
    }
}

impl Error for VpcCalculationError {}

/// Result category for a VPC validation attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VpcValidationOutcome {
    /// The stored and calculated VPC words are equal.
    Valid,
    /// The stored and calculated VPC words differ.
    Invalid,
    /// No complete stored VPC word was available.
    NotPresent,
    /// A stored VPC was available, but required calculation inputs were not.
    NotChecked,
}

/// A reason that an available VPC word was not checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VpcNotCheckedReason {
    /// The caller could not provide the logical ASM calculation inputs.
    MissingCalculationInputs,
    /// The protected byte region did not contain complete 32-bit words.
    Calculation(VpcCalculationError),
}

/// An auditable VPC validation result.
///
/// Optional values remain explicit so absence is not confused with an invalid
/// parity word. When calculation inputs were available, they are retained even
/// if alignment prevented the calculation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VpcValidation<'a> {
    outcome: VpcValidationOutcome,
    profile_id: Option<&'static str>,
    assumption_dependent: bool,
    stored_vpc: Option<u32>,
    calculated_vpc: Option<u32>,
    calculation_inputs: Option<VpcCalculationInputs<'a>>,
    not_checked_reason: Option<VpcNotCheckedReason>,
}

impl<'a> VpcValidation<'a> {
    /// Returns the validation result category.
    #[must_use]
    pub const fn outcome(self) -> VpcValidationOutcome {
        self.outcome
    }

    /// Returns the selected profile identifier when validation used one.
    #[must_use]
    pub const fn profile_id(self) -> Option<&'static str> {
        self.profile_id
    }

    /// Returns whether the validation depends on provisional profile inputs.
    #[must_use]
    pub const fn assumption_dependent(self) -> bool {
        self.assumption_dependent
    }

    /// Returns the stored VPC when a complete word was available.
    #[must_use]
    pub const fn stored_vpc(self) -> Option<u32> {
        self.stored_vpc
    }

    /// Returns the calculated VPC when the inputs were complete and aligned.
    #[must_use]
    pub const fn calculated_vpc(self) -> Option<u32> {
        self.calculated_vpc
    }

    /// Returns the exact calculation inputs when they were available.
    #[must_use]
    pub const fn calculation_inputs(self) -> Option<VpcCalculationInputs<'a>> {
        self.calculation_inputs
    }

    /// Returns why an available VPC was not checked.
    #[must_use]
    pub const fn not_checked_reason(self) -> Option<VpcNotCheckedReason> {
        self.not_checked_reason
    }
}

/// Calculates VPC as the complement of the XOR of all supplied ASM words.
///
/// The four reconstructed header words are explicit inputs. Protected bytes
/// are read as consecutive big-endian `u32` values.
pub fn calculate_vpc(inputs: VpcCalculationInputs<'_>) -> Result<u32, VpcCalculationError> {
    if !inputs.protected_bytes().len().is_multiple_of(4) {
        return Err(VpcCalculationError::ProtectedDataNotWordAligned {
            actual: inputs.protected_bytes().len(),
        });
    }

    let (words, remainder) = inputs.protected_bytes().as_chunks::<4>();
    debug_assert!(remainder.is_empty());
    let xor_value = words
        .iter()
        .map(|bytes| u32::from_be_bytes(*bytes))
        .fold(inputs.header_xor(), |accumulator, word| accumulator ^ word);

    Ok(!xor_value)
}

/// Validates an optional stored VPC against optional calculation inputs.
///
/// A missing stored word is `NotPresent`. An available stored word is
/// `NotChecked` when the inputs are missing or cannot be consumed. Otherwise,
/// the stored and calculated values determine `Valid` or `Invalid`.
#[must_use]
pub fn validate_vpc<'a>(
    calculation_inputs: Option<VpcCalculationInputs<'a>>,
    stored_vpc: Option<u32>,
) -> VpcValidation<'a> {
    validate_vpc_with_context(calculation_inputs, stored_vpc, None, false)
}

fn validate_vpc_with_context<'a>(
    calculation_inputs: Option<VpcCalculationInputs<'a>>,
    stored_vpc: Option<u32>,
    profile_id: Option<&'static str>,
    assumption_dependent: bool,
) -> VpcValidation<'a> {
    let Some(stored_vpc) = stored_vpc else {
        return VpcValidation {
            outcome: VpcValidationOutcome::NotPresent,
            profile_id,
            assumption_dependent,
            stored_vpc: None,
            calculated_vpc: calculation_inputs.and_then(|inputs| calculate_vpc(inputs).ok()),
            calculation_inputs,
            not_checked_reason: None,
        };
    };

    let Some(inputs) = calculation_inputs else {
        return VpcValidation {
            outcome: VpcValidationOutcome::NotChecked,
            profile_id,
            assumption_dependent,
            stored_vpc: Some(stored_vpc),
            calculated_vpc: None,
            calculation_inputs: None,
            not_checked_reason: Some(VpcNotCheckedReason::MissingCalculationInputs),
        };
    };

    match calculate_vpc(inputs) {
        Ok(calculated_vpc) => VpcValidation {
            outcome: if stored_vpc == calculated_vpc {
                VpcValidationOutcome::Valid
            } else {
                VpcValidationOutcome::Invalid
            },
            profile_id,
            assumption_dependent,
            stored_vpc: Some(stored_vpc),
            calculated_vpc: Some(calculated_vpc),
            calculation_inputs: Some(inputs),
            not_checked_reason: None,
        },
        Err(error) => VpcValidation {
            outcome: VpcValidationOutcome::NotChecked,
            profile_id,
            assumption_dependent,
            stored_vpc: Some(stored_vpc),
            calculated_vpc: None,
            calculation_inputs: Some(inputs),
            not_checked_reason: Some(VpcNotCheckedReason::Calculation(error)),
        },
    }
}

/// Raw fields decoded under profile `aero1394-assumed-as5643b-v1`.
///
/// The logical ASM header is reconstructed from the supplied message ID and
/// explicit profile constants. `retained_bytes` preserves the exact input so
/// later integrity and payload layers can audit every decoded field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssumedAs5643bV1Message<'a> {
    message_id: MessageId,
    health_status: u32,
    heartbeat: u32,
    application_data: &'a [u8],
    stof_transmit_offset: u32,
    stof_receive_offset: u32,
    stof_datapump_offset: u32,
    stored_vpc: u32,
    vpc_protected_bytes: &'a [u8],
    retained_bytes: &'a [u8],
}

impl<'a> AssumedAs5643bV1Message<'a> {
    /// Returns the stable profile identifier used for this result.
    #[must_use]
    pub const fn profile_id(&self) -> &'static str {
        ASSUMED_AS5643B_V1_PROFILE_ID
    }

    /// Returns whether interpretation depends on provisional profile inputs.
    #[must_use]
    pub const fn assumption_dependent(&self) -> bool {
        true
    }

    /// Returns the logical AS5643 Message ID.
    #[must_use]
    pub const fn message_id(&self) -> MessageId {
        self.message_id
    }

    /// Returns the reconstructed raw reserved/security header word.
    #[must_use]
    pub const fn reserved_security(&self) -> u32 {
        RESERVED_SECURITY
    }

    /// Returns the reconstructed raw Node ID header word.
    #[must_use]
    pub const fn node_id(&self) -> u32 {
        NODE_ID
    }

    /// Returns the reconstructed raw priority/payload-length header word.
    #[must_use]
    pub const fn priority_and_payload_length(&self) -> u32 {
        PRIORITY_AND_PAYLOAD_LENGTH
    }

    /// Returns the raw Health Status word.
    #[must_use]
    pub const fn health_status(&self) -> u32 {
        self.health_status
    }

    /// Returns the raw Heartbeat word.
    #[must_use]
    pub const fn heartbeat(&self) -> u32 {
        self.heartbeat
    }

    /// Returns the exact application bytes without decoding them.
    #[must_use]
    pub const fn application_data(&self) -> &'a [u8] {
        self.application_data
    }

    /// Returns the raw STOF Transmit Offset word.
    #[must_use]
    pub const fn stof_transmit_offset(&self) -> u32 {
        self.stof_transmit_offset
    }

    /// Returns the raw STOF Receive Offset word.
    #[must_use]
    pub const fn stof_receive_offset(&self) -> u32 {
        self.stof_receive_offset
    }

    /// Returns the raw STOF Datapump Offset word.
    #[must_use]
    pub const fn stof_datapump_offset(&self) -> u32 {
        self.stof_datapump_offset
    }

    /// Returns the retained VPC word without validating it.
    #[must_use]
    pub const fn stored_vpc(&self) -> u32 {
        self.stored_vpc
    }

    /// Returns the explicit reconstructed and retained inputs used for VPC.
    #[must_use]
    pub const fn vpc_calculation_inputs(&self) -> VpcCalculationInputs<'a> {
        VpcCalculationInputs::new(
            self.message_id,
            RESERVED_SECURITY,
            NODE_ID,
            PRIORITY_AND_PAYLOAD_LENGTH,
            self.vpc_protected_bytes,
        )
    }

    /// Calculates and validates the retained VPC under this profile.
    #[must_use]
    pub fn vpc_validation(&self) -> VpcValidation<'a> {
        validate_vpc_with_context(
            Some(self.vpc_calculation_inputs()),
            Some(self.stored_vpc),
            Some(ASSUMED_AS5643B_V1_PROFILE_ID),
            true,
        )
    }

    /// Returns every retained byte supplied to the decoder.
    #[must_use]
    pub const fn retained_bytes(&self) -> &'a [u8] {
        self.retained_bytes
    }
}

/// A failure to decode the current assumption-dependent AS5643 profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssumedAs5643bV1DecodeError {
    /// The supplied Message ID does not select this profile.
    UnsupportedMessageId {
        /// Message ID supplied by the caller.
        actual: MessageId,
    },
    /// The retained payload/trailer representation has the wrong length.
    InvalidRetainedLength {
        /// Exact byte count required by the profile.
        expected: usize,
        /// Byte count supplied by the caller.
        actual: usize,
    },
}

impl fmt::Display for AssumedAs5643bV1DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMessageId { actual } => write!(
                formatter,
                "AS5643 profile {ASSUMED_AS5643B_V1_PROFILE_ID} does not support message ID 0x{:08X}",
                actual.get()
            ),
            Self::InvalidRetainedLength { expected, actual } => write!(
                formatter,
                "AS5643 profile {ASSUMED_AS5643B_V1_PROFILE_ID} requires {expected} retained bytes, received {actual}"
            ),
        }
    }
}

impl Error for AssumedAs5643bV1DecodeError {}

/// Decodes the retained payload and trailer for the current provisional profile.
///
/// The caller supplies the logical Message ID independently of the retained
/// bytes. A BIE adapter may reconstruct that value from a data-item ID, while
/// another input adapter may obtain it from an actual ASM header.
pub fn decode_assumed_as5643b_v1(
    message_id: MessageId,
    retained_bytes: &[u8],
) -> Result<AssumedAs5643bV1Message<'_>, AssumedAs5643bV1DecodeError> {
    if message_id != ASSUMED_AS5643B_V1_MESSAGE_ID {
        return Err(AssumedAs5643bV1DecodeError::UnsupportedMessageId { actual: message_id });
    }
    if retained_bytes.len() != ASSUMED_AS5643B_V1_RETAINED_LEN {
        return Err(AssumedAs5643bV1DecodeError::InvalidRetainedLength {
            expected: ASSUMED_AS5643B_V1_RETAINED_LEN,
            actual: retained_bytes.len(),
        });
    }

    let invalid_length = || AssumedAs5643bV1DecodeError::InvalidRetainedLength {
        expected: ASSUMED_AS5643B_V1_RETAINED_LEN,
        actual: retained_bytes.len(),
    };

    Ok(AssumedAs5643bV1Message {
        message_id,
        health_status: read_u32_be(retained_bytes, 0).ok_or_else(invalid_length)?,
        heartbeat: read_u32_be(retained_bytes, 4).ok_or_else(invalid_length)?,
        application_data: retained_bytes
            .get(APPLICATION_START..APPLICATION_END)
            .ok_or_else(invalid_length)?,
        stof_transmit_offset: read_u32_be(retained_bytes, STOF_TRANSMIT_OFFSET)
            .ok_or_else(invalid_length)?,
        stof_receive_offset: read_u32_be(retained_bytes, STOF_RECEIVE_OFFSET)
            .ok_or_else(invalid_length)?,
        stof_datapump_offset: read_u32_be(retained_bytes, STOF_DATAPUMP_OFFSET)
            .ok_or_else(invalid_length)?,
        stored_vpc: read_u32_be(retained_bytes, VPC_OFFSET).ok_or_else(invalid_length)?,
        vpc_protected_bytes: retained_bytes
            .get(..VPC_OFFSET)
            .ok_or_else(invalid_length)?,
        retained_bytes,
    })
}

fn read_u32_be(input: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let bytes: [u8; 4] = input.get(offset..end)?.try_into().ok()?;
    Some(u32::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Requirements: L3-PRO-002
    #[test]
    fn rejects_a_message_id_outside_the_selected_profile() {
        let error = decode_assumed_as5643b_v1(
            MessageId::new(0x0000_35F4),
            &[0; ASSUMED_AS5643B_V1_RETAINED_LEN],
        )
        .expect_err("different message ID must not select this profile");

        assert_eq!(
            error,
            AssumedAs5643bV1DecodeError::UnsupportedMessageId {
                actual: MessageId::new(0x0000_35F4),
            }
        );
    }

    /// Requirements: L3-PRO-002
    #[test]
    fn rejects_short_and_long_retained_representations() {
        for actual in [
            ASSUMED_AS5643B_V1_RETAINED_LEN - 1,
            ASSUMED_AS5643B_V1_RETAINED_LEN + 1,
        ] {
            let bytes = vec![0; actual];
            let error = decode_assumed_as5643b_v1(ASSUMED_AS5643B_V1_MESSAGE_ID, &bytes)
                .expect_err("wrong retained length must fail");

            assert_eq!(
                error,
                AssumedAs5643bV1DecodeError::InvalidRetainedLength {
                    expected: ASSUMED_AS5643B_V1_RETAINED_LEN,
                    actual,
                }
            );
        }
    }

    /// Requirements: L3-PRO-004
    #[test]
    fn distinguishes_absent_and_unavailable_vpc_results() {
        let inputs = VpcCalculationInputs::new(MessageId::new(1), 2, 4, 8, &[]);
        let calculated = calculate_vpc(inputs).expect("empty protected region is word-aligned");

        let absent = validate_vpc(Some(inputs), None);
        assert_eq!(absent.outcome(), VpcValidationOutcome::NotPresent);
        assert_eq!(absent.profile_id(), None);
        assert!(!absent.assumption_dependent());
        assert_eq!(absent.stored_vpc(), None);
        assert_eq!(absent.calculated_vpc(), Some(calculated));
        assert_eq!(absent.calculation_inputs(), Some(inputs));
        assert_eq!(absent.not_checked_reason(), None);

        let unavailable = validate_vpc(None, Some(calculated));
        assert_eq!(unavailable.outcome(), VpcValidationOutcome::NotChecked);
        assert_eq!(unavailable.profile_id(), None);
        assert!(!unavailable.assumption_dependent());
        assert_eq!(unavailable.stored_vpc(), Some(calculated));
        assert_eq!(unavailable.calculated_vpc(), None);
        assert_eq!(unavailable.calculation_inputs(), None);
        assert_eq!(
            unavailable.not_checked_reason(),
            Some(VpcNotCheckedReason::MissingCalculationInputs)
        );
    }

    /// Requirements: L3-PRO-004
    #[test]
    fn reports_unaligned_protected_data_as_not_checked() {
        let inputs = VpcCalculationInputs::new(MessageId::new(1), 2, 4, 8, &[0, 0, 0]);
        let result = validate_vpc(Some(inputs), Some(0));

        assert_eq!(result.outcome(), VpcValidationOutcome::NotChecked);
        assert_eq!(result.stored_vpc(), Some(0));
        assert_eq!(result.calculated_vpc(), None);
        assert_eq!(result.calculation_inputs(), Some(inputs));
        assert_eq!(
            result.not_checked_reason(),
            Some(VpcNotCheckedReason::Calculation(
                VpcCalculationError::ProtectedDataNotWordAligned { actual: 3 }
            ))
        );
    }
}
