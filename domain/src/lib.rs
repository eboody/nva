//! Typed foundation for a 170-location pet-resort workflow/agent platform.
//!
//! This crate intentionally models *business contracts* before implementation details:
//! entities, workflow events, agent identity values, and policy decisions.

/// Crate-level agent surface for the Gingr integration boundary.
pub mod agent;
/// Crate-level analytics surface for the Gingr integration boundary.
pub mod analytics;
/// Crate-level audit surface for the Gingr integration boundary.
pub mod audit;
/// Crate-level boarding surface for the Gingr integration boundary.
pub mod boarding;
/// Crate-level care surface for the Gingr integration boundary.
pub mod care;
/// Crate-level customer surface for the Gingr integration boundary.
pub mod customer;
pub mod daily_brief;
/// Crate-level data quality surface for the Gingr integration boundary.
pub mod data_quality;
pub mod daycare;
/// Crate-level document surface for the Gingr integration boundary.
pub mod document;
/// Crate-level entities surface for the Gingr integration boundary.
pub mod entities;
/// Crate-level grooming surface for the Gingr integration boundary.
pub mod grooming;
/// Crate-level incident surface for the Gingr integration boundary.
pub mod incident;
pub mod lead;
/// Crate-level location surface for the Gingr integration boundary.
pub mod location;
/// Crate-level message surface for the Gingr integration boundary.
pub mod message;
/// Crate-level money surface for the Gingr integration boundary.
pub mod money;
pub mod operations;
pub mod payment;
/// Crate-level pet surface for the Gingr integration boundary.
pub mod pet;
/// Crate-level policy surface for the Gingr integration boundary.
pub mod policy;
/// Crate-level portal surface for the Gingr integration boundary.
pub mod portal;
pub mod reputation;
/// Crate-level reservation surface for the Gingr integration boundary.
pub mod reservation;
/// Crate-level retail surface for the Gingr integration boundary.
pub mod retail;
pub mod source;
pub mod staff;
/// Crate-level temperament surface for the Gingr integration boundary.
pub mod temperament;
/// Crate-level training surface for the Gingr integration boundary.
pub mod training;
/// Crate-level vaccine surface for the Gingr integration boundary.
pub mod vaccine;
/// Crate-level workflow surface for the Gingr integration boundary.
pub mod workflow;
