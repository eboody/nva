//! Worker shell for safe local/background pet-resort workflow execution.
//!
//! MVP workers default to deterministic agent output and stubbed side effects so
//! local development and CI cannot accidentally send customer messages or write
//! to provider systems. The worker runtime belongs above storage: it decides how
//! to execute workflow packets, while storage records merely preserve evidence
//! and domain/app modules own semantic approval rules.

/// Worker runtime configuration that defaults agents and side effects to safe modes.
pub mod runtime;
