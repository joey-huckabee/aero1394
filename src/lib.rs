//! Reusable core for Aero1394 capture inspection and analysis.
//!
//! The public surface currently supports format-neutral forensic inspection
//! and evidence-backed parsing of individual records and strict complete BIE
//! byte slices. The provisional AS5643 profile decodes its retained raw
//! envelope and validates VPC independently of BIE and application payload
//! types. A separate adapter maps only explicitly supported BIE identities and
//! layouts to that profile while preserving unsupported records. The payload
//! registry independently selects compiled-in definitions while retaining raw
//! bytes for unknown and ambiguous inputs. Its first typed decoder exposes all
//! raw `msfcs_storesmassdata_b` fields without inferring engineering semantics.

#![forbid(unsafe_code)]

pub mod as5643;
pub mod bie;
pub mod bie_as5643;
pub mod forensic;
pub mod payload;
