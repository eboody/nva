//! Worker shell for durable Postgres-backed pet-resort workflows.
//!
//! MVP workers default to deterministic agent output and stubbed side effects so
//! local development and CI cannot accidentally send customer messages or write
//! to provider systems.

/// Worker runtime configuration that defaults agents and side effects to safe modes.
pub mod runtime;
