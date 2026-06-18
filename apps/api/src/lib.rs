//! HTTP API shell for the pet-resort MVP staff workflow platform.
//!
//! The API owns runtime exposure of workflow policy, audit, and side-effect
//! gates. Staff UI and integration tests may submit inquiries, vaccine
//! documents, manager daily-brief drafts, and data-quality hygiene outcomes, but
//! this crate keeps those requests typed, audited, and review-gated before any
//! live customer message or provider write is possible.
//!
//! Runtime surfaces in this crate should be read as deterministic workflow
//! adapters: they create context packets, DTOs, and storage evidence, while app
//! and domain modules own the semantic decisions and storage owns durable
//! projections.

/// Axum routes that expose review-gated workflows and audit-friendly DTOs.
pub mod http;
