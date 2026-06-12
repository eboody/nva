//! Typed foundation for a 170-location pet-resort workflow/agent platform.
//!
//! This crate intentionally models *business contracts* before implementation details:
//! entities, workflow events, agent identity values, and policy decisions.

pub mod agent;
pub mod audit;
pub mod boarding;
pub mod care;
pub mod customer;
pub mod daily_brief;
pub mod daycare;
pub mod document;
pub mod entities;
pub mod grooming;
pub mod incident;
pub mod lead;
pub mod location;
pub mod message;
pub mod money;
pub mod operations;
pub mod payment;
pub mod pet;
pub mod policy;
pub mod portal;
pub mod reputation;
pub mod reservation;
pub mod retail;
pub mod staff;
pub mod temperament;
pub mod training;
pub mod vaccine;
pub mod workflow;
