//! Storage adapter contracts for pet-resort domain models.
//!
//! `domain` owns the domain language. This crate owns persistence-shaped
//! records, stable storage codes, JSON codecs, and explicit promotion/demotion
//! between storage records and core domain types.

pub mod operations;
pub mod service_line;

pub use operations::{CodecError, RecordKind, Result};
