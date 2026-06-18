//! Boarding capacity decisions for room/suite availability.
//!
//! Capacity examples use semantic accommodation paths so a labor-saving agent can explain whether
//! the front desk should confirm, waitlist, or route an exception for manager review:
//!
//! ```
//! use domain::{boarding, entities};
//! use uuid::Uuid;
//!
//! let luxury_suite = boarding::capacity::SegmentCounts::builder()
//!     .accommodation(boarding::accommodation::Kind::LuxuryDogSuite)
//!     .total(boarding::capacity::RoomCount::try_new(10).unwrap())
//!     .occupied(boarding::capacity::RoomCount::try_new(10).unwrap())
//!     .build();
//! let snapshot = boarding::capacity::Snapshot::new(vec![
//!     boarding::capacity::NightlySegmentSnapshot::from_counts(luxury_suite),
//! ])
//! .unwrap();
//! let request = boarding::capacity::Request::new(
//!     entities::LocationId(Uuid::nil()),
//!     entities::Species::Dog,
//!     boarding::accommodation::Preference::Specific(boarding::accommodation::Kind::LuxuryDogSuite),
//! );
//!
//! assert_eq!(
//!     boarding::capacity::Policy.evaluate(&request, &snapshot),
//!     boarding::capacity::Decision::Waitlist {
//!         reason: boarding::capacity::WaitlistReason::EligibleSegmentFull,
//!     },
//! );
//! ```

use super::*;
use crate::policy;
use bon::Builder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed room count domain value that keeps raw primitives out of boarding workflows.
pub struct RoomCount(u16);

impl RoomCount {
    /// Promotes boundary input into a validated boarding domain value.
    pub const fn try_new(value: u16) -> std::result::Result<Self, RoomCountError> {
        Ok(Self(value))
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Domain vocabulary for room count error decisions in boarding workflows.
pub enum RoomCountError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed segment counts domain value that keeps raw primitives out of boarding workflows.
pub struct SegmentCounts {
    /// Accommodation fact promoted into this boarding contract.
    pub accommodation: accommodation::Kind,
    total: RoomCount,
    occupied: RoomCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Typed nightly segment snapshot domain value that keeps raw primitives out of boarding workflows.
pub struct NightlySegmentSnapshot {
    /// Accommodation fact promoted into this boarding contract.
    pub accommodation: accommodation::Kind,
    total: RoomCount,
    occupied: RoomCount,
}

impl NightlySegmentSnapshot {
    /// Derives this boarding value from counts data.
    pub const fn from_counts(counts: SegmentCounts) -> Self {
        Self {
            accommodation: counts.accommodation,
            total: counts.total,
            occupied: counts.occupied,
        }
    }

    /// Returns this boarding value's total.
    pub const fn total(&self) -> RoomCount {
        self.total
    }

    /// Returns this boarding value's occupied.
    pub const fn occupied(&self) -> RoomCount {
        self.occupied
    }

    /// Returns this boarding value's available rooms.
    pub const fn available_rooms(&self) -> RoomCount {
        RoomCount(self.total.get().saturating_sub(self.occupied.get()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Point-in-time source-data view used before promotion into core domain records.
pub struct Snapshot {
    segments: Vec<NightlySegmentSnapshot>,
}

impl Snapshot {
    /// Assembles this boarding value from already-validated domain parts.
    pub fn new(segments: Vec<NightlySegmentSnapshot>) -> std::result::Result<Self, SnapshotError> {
        if segments.is_empty() {
            return Err(SnapshotError::EmptyInventory);
        }
        Ok(Self { segments })
    }

    /// Returns the segments for this boarding value.
    pub fn segments(&self) -> &[NightlySegmentSnapshot] {
        &self.segments
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Domain vocabulary for snapshot error decisions in boarding workflows.
pub enum SnapshotError {
    #[error("boarding capacity snapshot requires at least one accommodation segment")]
    /// Signals that inventory was blank or missing during boarding validation.
    EmptyInventory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed request domain value that keeps raw primitives out of boarding workflows.
pub struct Request {
    /// Location id fact promoted into this boarding contract.
    pub location_id: LocationId,
    /// Species fact promoted into this boarding contract.
    pub species: crate::entities::Species,
    /// Accommodation fact promoted into this boarding contract.
    pub accommodation: accommodation::Preference,
}

impl Request {
    /// Assembles this boarding value from already-validated domain parts.
    pub const fn new(
        location_id: LocationId,
        species: crate::entities::Species,
        accommodation: accommodation::Preference,
    ) -> Self {
        Self {
            location_id,
            species,
            accommodation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for decision decisions in boarding workflows.
pub enum Decision {
    /// Available boarding policy, stay, capacity, or upsell signal.
    Available {
        /// Accommodation fact promoted into this boarding contract.
        accommodation: accommodation::Kind,
    },
    /// Waitlist boarding policy, stay, capacity, or upsell signal.
    Waitlist {
        /// Business reason staff should review before proceeding.
        reason: WaitlistReason,
    },
    /// Deny boarding policy, stay, capacity, or upsell signal.
    Deny {
        /// Business reason staff should review before proceeding.
        reason: DenialReason,
        /// Review gate fact promoted into this boarding contract.
        review_gate: policy::ReviewGate,
    },
}

impl Decision {
    /// Returns the required review gate for this boarding value.
    pub fn required_review_gate(&self) -> Option<policy::ReviewGate> {
        match self {
            Self::Deny { review_gate, .. } => Some(review_gate.clone()),
            Self::Available { .. } | Self::Waitlist { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for denial reason decisions in boarding workflows.
pub enum DenialReason {
    /// Species accommodation mismatch boarding policy, stay, capacity, or upsell signal.
    SpeciesAccommodationMismatch,
    /// No eligible segment boarding policy, stay, capacity, or upsell signal.
    NoEligibleSegment,
    /// Policy unavailable boarding policy, stay, capacity, or upsell signal.
    PolicyUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for waitlist reason decisions in boarding workflows.
pub enum WaitlistReason {
    /// Eligible segment full boarding policy, stay, capacity, or upsell signal.
    EligibleSegmentFull,
}

#[derive(Debug, Clone, Default)]
/// Typed policy domain value that keeps raw primitives out of boarding workflows.
pub struct Policy;

impl Policy {
    /// Returns the evaluate for this boarding value.
    pub fn evaluate(&self, request: &Request, snapshot: &Snapshot) -> Decision {
        let mut compatible_but_full = false;

        for wanted in request.accommodation.acceptable_kinds() {
            if !wanted.supports_species(&request.species) {
                return Decision::Deny {
                    reason: DenialReason::SpeciesAccommodationMismatch,
                    review_gate: policy::ReviewGate::ManagerApproval,
                };
            }

            for segment in snapshot.segments() {
                if segment.accommodation == *wanted {
                    if segment.available_rooms().get() > 0 {
                        return Decision::Available {
                            accommodation: *wanted,
                        };
                    }
                    compatible_but_full = true;
                }
            }
        }

        if compatible_but_full {
            Decision::Waitlist {
                reason: WaitlistReason::EligibleSegmentFull,
            }
        } else {
            Decision::Deny {
                reason: DenialReason::NoEligibleSegment,
                review_gate: policy::ReviewGate::ManagerApproval,
            }
        }
    }
}
