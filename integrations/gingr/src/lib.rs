//! Gingr integration contracts, DTO mappings, endpoints, transport, and webhook verification.
/// Crate-level config surface for the Gingr integration boundary.
pub mod config;
/// Crate-level dto surface for the Gingr integration boundary.
pub mod dto;
pub mod endpoint;
pub mod mapping;
/// Crate-level response surface for the Gingr integration boundary.
pub mod response;
/// Crate-level transport surface for the Gingr integration boundary.
pub mod transport;
pub mod webhook;

pub use config::{ApiKey, BaseUrl, Provider, Subdomain};
