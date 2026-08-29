//! Reusable core for Aero1394 capture inspection and analysis.
//!
//! The public surface currently supports format-neutral forensic inspection
//! and evidence-backed parsing of individual records and strict complete BIE
//! byte slices.

#![forbid(unsafe_code)]

pub mod bie;
pub mod forensic;
