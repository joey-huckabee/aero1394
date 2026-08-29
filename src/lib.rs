//! Reusable core for Aero1394 capture inspection and analysis.
//!
//! The public surface currently supports format-neutral forensic inspection
//! and evidence-backed parsing of individual BIE records.

#![forbid(unsafe_code)]

pub mod bie;
pub mod forensic;
