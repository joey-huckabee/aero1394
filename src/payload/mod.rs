//! Deterministic selection of built-in application payload definitions.
//!
//! Payload selection is independent of BIE and protocol parsing. Callers
//! provide the available identity context and the exact application bytes.
//! Unknown and ambiguous inputs retain those bytes for later inspection.

mod registry;

pub mod msfcs_storesmassdata_b;

pub use registry::{
    AmbiguousPayload, MatchedPayload, PayloadByteOrder, PayloadContext, PayloadDefinition,
    PayloadRegistry, PayloadSelection, RawPayload, select_payload,
};
