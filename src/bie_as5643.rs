//! Explicit mappings from BIE records to supported AS5643 profiles.
//!
//! This adapter depends on both layers so the generic BIE parser remains
//! independent of AS5643. Unsupported identities and layouts preserve the
//! complete parsed BIE record instead of becoming decode errors.

use crate::as5643::{
    ASSUMED_AS5643B_V1_MESSAGE_ID, ASSUMED_AS5643B_V1_PROFILE_ID, ASSUMED_AS5643B_V1_RETAINED_LEN,
    AssumedAs5643bV1DecodeError, AssumedAs5643bV1Message, decode_assumed_as5643b_v1,
};
use crate::bie::{BieRecord, DataItemId};

/// BIE data-item identity mapped by the current assumption-dependent profile.
pub const ASSUMED_AS5643B_V1_BIE_DATA_ITEM_ID: DataItemId =
    DataItemId::new(ASSUMED_AS5643B_V1_MESSAGE_ID.get());

/// The outcome of applying the supported BIE-to-AS5643 mappings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BieAs5643MappingOutcome<'a> {
    /// The record matched and decoded under the named provisional profile.
    AssumedAs5643bV1(AssumedAs5643bV1Message<'a>),
    /// No supported mapping exists for the BIE data-item identity.
    UnsupportedDataItem,
    /// The identity matched, but the stored representation has another size.
    UnsupportedStoredDataLength {
        /// Exact retained length required by the selected mapping.
        expected: usize,
        /// Stored-data length preserved in the BIE record.
        actual: usize,
    },
}

/// A parsed BIE record together with its optional AS5643 interpretation.
///
/// The raw record is retained for every outcome, including unsupported ones.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BieAs5643Mapping<'a> {
    record: BieRecord<'a>,
    outcome: BieAs5643MappingOutcome<'a>,
}

impl<'a> BieAs5643Mapping<'a> {
    /// Returns the complete parsed BIE record supplied to the adapter.
    #[must_use]
    pub const fn record(self) -> BieRecord<'a> {
        self.record
    }

    /// Returns the mapped or unsupported interpretation outcome.
    #[must_use]
    pub const fn outcome(self) -> BieAs5643MappingOutcome<'a> {
        self.outcome
    }

    /// Returns the selected profile identifier when the record was mapped.
    #[must_use]
    pub const fn profile_id(self) -> Option<&'static str> {
        match self.outcome {
            BieAs5643MappingOutcome::AssumedAs5643bV1(_) => Some(ASSUMED_AS5643B_V1_PROFILE_ID),
            BieAs5643MappingOutcome::UnsupportedDataItem
            | BieAs5643MappingOutcome::UnsupportedStoredDataLength { .. } => None,
        }
    }
}

/// Applies the explicit supported BIE-to-AS5643 mapping to one parsed record.
///
/// Selection requires both data-item ID `0x00005D04` and exactly 116 retained
/// bytes. A numerically equal BIE data-item ID is explicitly reconstructed as
/// the current profile's distinct AS5643 Message ID.
#[must_use]
pub fn map_bie_record_to_as5643<'a>(record: BieRecord<'a>) -> BieAs5643Mapping<'a> {
    let outcome = if record.data_item_id() != ASSUMED_AS5643B_V1_BIE_DATA_ITEM_ID {
        BieAs5643MappingOutcome::UnsupportedDataItem
    } else if record.stored_data().len() != ASSUMED_AS5643B_V1_RETAINED_LEN {
        BieAs5643MappingOutcome::UnsupportedStoredDataLength {
            expected: ASSUMED_AS5643B_V1_RETAINED_LEN,
            actual: record.stored_data().len(),
        }
    } else {
        match decode_assumed_as5643b_v1(ASSUMED_AS5643B_V1_MESSAGE_ID, record.stored_data()) {
            Ok(message) => BieAs5643MappingOutcome::AssumedAs5643bV1(message),
            Err(AssumedAs5643bV1DecodeError::InvalidRetainedLength { expected, actual }) => {
                BieAs5643MappingOutcome::UnsupportedStoredDataLength { expected, actual }
            }
            Err(AssumedAs5643bV1DecodeError::UnsupportedMessageId { .. }) => {
                BieAs5643MappingOutcome::UnsupportedDataItem
            }
        }
    };

    BieAs5643Mapping { record, outcome }
}
