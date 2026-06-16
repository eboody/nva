//! Service-line-owned storage records and code tables.
//!
//! The `domain::<service-line>` modules own service-line concepts. These modules own the
//! persistence-facing shapes and explicit promotion/demotion at the storage
//! boundary.

pub mod boarding;
pub mod daycare;
pub mod grooming;
pub mod retail;
pub mod training;
