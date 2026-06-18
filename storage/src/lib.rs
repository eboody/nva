//! Storage adapter contracts for pet-resort domain models.
//!
//! `domain` owns the NVA pet-resort language: service lines, portfolio facts,
//! workflow gates, source evidence, and labor-saving outcomes. This crate owns
//! the persistence projection of that language: stable storage codes, JSON
//! codecs, flattened records shaped for databases and fixtures, and explicit
//! promotion/demotion between storage records and core domain types.
//!
//! Storage code is intentionally not the business model. It may preserve source
//! system vocabulary such as Gingr record identifiers, provider codes, and
//! serialized DTO shapes, but application and domain layers must promote those
//! facts into semantic domain values before using them to approve customer
//! messages, manager actions, labor evidence, or data-quality repairs.

pub mod operations;
pub mod service_line;

pub use operations::{CodecError, RecordKind, Result};
