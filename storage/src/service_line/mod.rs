//! Service-line-owned storage records and code tables.
//!
//! The `domain::<service-line>` modules own service-line concepts such as
//! boarding suites, daycare eligibility, grooming rebooking cadence, training
//! programs, and retail partner products. These modules own the
//! persistence-facing shapes and explicit promotion/demotion at the storage
//! gate.
//!
//! Keep provider/source-system identifiers and durable codes here. Keep labor,
//! safety, eligibility, and customer-facing decisions in domain/app workflows so
//! runtime surfaces cannot treat a stored code as authority to override review
//! gates or invent availability.
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use storage::service_line::{boarding, grooming, training};
//!
//! let stored_suite = boarding::AccommodationCode::LuxurySuite;
//! let promoted_suite: domain::operations::lodging_offer::Accommodation = stored_suite.into();
//! assert_eq!(boarding::AccommodationCode::from(promoted_suite), stored_suite);
//!
//! let stored_grooming_cadence = grooming::StoredCadenceWeeks::try_new(6)?;
//! let promoted_grooming_cadence: domain::grooming::rebooking::CadenceWeeks =
//!     stored_grooming_cadence.try_into()?;
//! assert_eq!(promoted_grooming_cadence.get(), 6);
//!
//! let stored_training_duration = training::StoredProgramDurationWeeks::try_new(3)?;
//! let promoted_training_duration: domain::training::program::DurationWeeks =
//!     stored_training_duration.try_into()?;
//! assert_eq!(promoted_training_duration.get(), 3);
//! # Ok(())
//! # }
//! ```

/// Boarding persistence shape for service-line rules.
pub mod boarding;
/// Daycare persistence shape for service-line rules.
pub mod daycare;
/// Grooming persistence shape for service-line rules.
pub mod grooming;
/// Retail persistence shape for service-line rules.
pub mod retail;
/// Training persistence shape for service-line rules.
pub mod training;
