//! SpacetimeDB realtime runtime adapter for NVA operational review loops.
//!
//! This crate is intentionally an adapter crate, not a domain crate. SpacetimeDB
//! table structs are storage/read-model rows, reducers are command-boundary
//! entrypoints, and app/domain crates remain the source of business rules.

pub mod adapter;
pub mod authz;
pub mod read_model;
pub mod reducers;
pub mod runtime;
pub mod storage;
pub mod tables;

#[cfg(test)]
mod realtime_queue_tests;

pub use reducers::*;
