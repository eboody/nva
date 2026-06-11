//! HTTP API shell for the pet-resort MVP staff workflow platform.
//!
//! The API owns workflow policy, audit, and side-effect gates. The staff UI may
//! request actions, but this crate is where those requests will become typed,
//! audited application commands.

pub mod http;
