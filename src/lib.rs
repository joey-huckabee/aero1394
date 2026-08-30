//! Reusable core for Aero1394 capture inspection and analysis.
//!
//! The public surface currently supports format-neutral forensic inspection
//! and evidence-backed parsing of individual records and strict complete BIE
//! byte slices. The provisional AS5643 profile decodes its retained raw
//! envelope and validates VPC independently of BIE and application payload
//! types.

#![forbid(unsafe_code)]

pub mod as5643;
pub mod bie;
pub mod forensic;
