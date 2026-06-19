//! Storage adapter rules for pet-resort domain models.
//!
//! # Operator framing
//!
//! Use this crate page when you need to know what gets persisted for reports,
//! fixtures, projections, or read models after source facts have been promoted
//! into the NVA domain language. It matters to operators because stored records
//! can support audits, manager briefs, and workflow evidence, but they are not
//! themselves approval to change reservations, send messages, collect payments,
//! or repair provider data.
//!
//! The next step is to open `service_line` for boarding/daycare/grooming/etc.
//! record projections or `operations` for shared codec and record-kind rules.
//! The public framing explains storage's role; the public Rustdoc below keeps
//! the API details for implementers.
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
//!
//! Crosswalk navigation: this crate is the persistence/projection step, not the
//! entry or workflow authority. Use
//! `docs/entity-atlas/contract-crosswalk/storage-persistence.md` to see which
//! entities have storage records, which are intentionally not persisted, and
//! which source/Rustdoc/test links prove the projection boundary.

pub mod operations;
pub mod service_line;

pub use operations::{CodecError, RecordKind, Result};
