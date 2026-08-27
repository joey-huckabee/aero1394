//! Reusable core for Aero1394 capture inspection and analysis.
//!
//! The current public surface is deliberately limited to format-neutral
//! forensic inspection. BIE structures will be added only after the file
//! layout is supported by capture evidence.

#![forbid(unsafe_code)]

pub mod forensic;
