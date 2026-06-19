//! Gingr integration surfaces for DTO mapping, endpoints, transport, and webhook verification.
//!
//! # Operator framing
//!
//! Use this crate page to understand how raw Gingr provider facts enter the NVA
//! system before they become trusted domain evidence. It matters to operators
//! because reservations, pets, customers, labor, retail, and webhook events from
//! Gingr are useful for drafting briefs or recommendations only after they pass
//! through redacted transport, DTO quarantine, mapping, and domain promotion.
//!
//! The next step is to open `endpoint` for the source surface being read,
//! `mapping` for how provider vocabulary is translated, `transport` for
//! redacted request/response handling, or `webhook` for inbound event
//! verification. These Rust modules document integration mechanics; they do not
//! authorize provider writes, customer messages, schedule changes, or policy
//! overrides.
//!
//! Crosswalk navigation: Gingr surfaces are source-entry evidence until a mapper
//! or `domain::source` contract normalizes them. Use
//! `docs/entity-atlas/contract-crosswalk/source-provider-flows.md` for the
//! provider DTO -> mapper/source -> domain/app/storage path and the tests that
//! prove each supported promotion or explicit provider-surface gap.
/// Validated tenant URL, provider label, and redacted API-key configuration for Gingr requests.
pub mod config;
/// Raw Gingr DTO surfaces that are intentionally quarantined before NVA domain promotion.
pub mod dto;
pub mod endpoint;
pub mod mapping;
/// HTTP and provider response envelopes that retain raw Gingr evidence for later decoding.
pub mod response;
/// Secret-aware request capture, redaction, and transport abstractions for Gingr endpoints.
pub mod transport;
pub mod webhook;

pub use config::{ApiKey, BaseUrl, Provider, Subdomain};
