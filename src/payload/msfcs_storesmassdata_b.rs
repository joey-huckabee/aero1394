//! Registry identity for the `msfcs_storesmassdata_b` payload.
//!
//! This increment identifies the supplied 92-byte layout without decoding its
//! fields. Raw field decoding remains a separate evidence-gated increment.

use super::{PayloadByteOrder, PayloadDefinition};

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

/// Built-in registry definition for the supplied layout identity.
pub const DEFINITION: PayloadDefinition = PayloadDefinition::new(
    NAME,
    DEFINITION_VERSION,
    DATA_ITEM_ID,
    PAYLOAD_SIZE,
    BYTE_ORDER,
);
