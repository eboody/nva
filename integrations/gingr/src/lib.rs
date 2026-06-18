//! Gingr integration contracts, DTO mappings, endpoints, transport, and webhook verification.
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
