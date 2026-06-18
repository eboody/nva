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
/// Non-negative count of rooms in a boarding accommodation segment.
pub struct RoomCount(u16);

impl RoomCount {
    /// Promotes a source-system room count into the boarding capacity domain.
    pub const fn try_new(value: u16) -> std::result::Result<Self, RoomCountError> {
        Ok(Self(value))
    }

    /// Returns the raw room count for source adapters, reports, and serialization.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation errors for room-count promotion from boundary data.
pub enum RoomCountError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Builder-facing source counts for one accommodation segment on a boarding night.
pub struct SegmentCounts {
    /// Accommodation segment these counts describe.
    pub accommodation: accommodation::Kind,
    total: RoomCount,
    occupied: RoomCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Immutable nightly capacity snapshot for one accommodation segment.
pub struct NightlySegmentSnapshot {
    /// Accommodation segment these counts describe.
    pub accommodation: accommodation::Kind,
    total: RoomCount,
    occupied: RoomCount,
}

impl NightlySegmentSnapshot {
    /// Freezes builder-provided segment counts into a nightly snapshot used by capacity policy.
    pub const fn from_counts(counts: SegmentCounts) -> Self {
        Self {
            accommodation: counts.accommodation,
            total: counts.total,
            occupied: counts.occupied,
        }
    }

    /// Returns total rooms known for this accommodation segment.
    pub const fn total(&self) -> RoomCount {
        self.total
    }

    /// Returns occupied rooms already committed for this accommodation segment.
    pub const fn occupied(&self) -> RoomCount {
        self.occupied
    }

    /// Returns remaining rooms after committed occupancy, saturating at zero for dirty data.
    pub const fn available_rooms(&self) -> RoomCount {
        RoomCount(self.total.get().saturating_sub(self.occupied.get()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Point-in-time boarding inventory evidence used to make confirm/waitlist/deny decisions.
pub struct Snapshot {
    segments: Vec<NightlySegmentSnapshot>,
}

impl Snapshot {
    /// Creates a capacity snapshot from one or more nightly accommodation segments.
    pub fn new(segments: Vec<NightlySegmentSnapshot>) -> std::result::Result<Self, SnapshotError> {
        if segments.is_empty() {
            return Err(SnapshotError::EmptyInventory);
        }
        Ok(Self { segments })
    }

    /// Returns the source-derived accommodation segments considered by capacity policy.
    pub fn segments(&self) -> &[NightlySegmentSnapshot] {
        &self.segments
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Snapshot validation errors that prevent safe capacity automation.
pub enum SnapshotError {
    #[error("boarding capacity snapshot requires at least one accommodation segment")]
    /// No inventory segments were available, so automation must not infer availability.
    EmptyInventory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Boarding capacity request for a location, species, and accommodation preference.
pub struct Request {
    /// Resort location whose room inventory is the authority for this check.
    pub location_id: LocationId,
    /// Pet species used to reject incompatible room types before availability is promised.
    pub species: crate::entities::Species,
    /// Accommodation preference requested by the guest or staff workflow.
    pub accommodation: accommodation::Preference,
}

impl Request {
    /// Creates a capacity request from already-identified location, species, and preference values.
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
/// Capacity outcome an agent may present to staff when handling a boarding request.
pub enum Decision {
    /// A compatible accommodation segment has at least one available room.
    Available {
        /// Accommodation that can be offered from the available source inventory.
        accommodation: accommodation::Kind,
    },
    /// Compatible accommodation exists but is currently full, so staff should route to waitlist.
    Waitlist {
        /// Source-grounded reason for the waitlist or denial outcome.
        reason: WaitlistReason,
    },
    /// The request cannot be confirmed from the supplied source evidence and requires a review gate.
    Deny {
        /// Source-grounded reason for the waitlist or denial outcome.
        reason: DenialReason,
        /// Human approval gate required before overriding the denied capacity decision.
        review_gate: policy::ReviewGate,
    },
}

impl Decision {
    /// Returns the human review gate required before staff override a denied capacity decision.
    pub fn required_review_gate(&self) -> Option<policy::ReviewGate> {
        match self {
            Self::Deny { review_gate, .. } => Some(review_gate.clone()),
            Self::Available { .. } | Self::Waitlist { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons boarding capacity policy must deny confirmation from available evidence.
pub enum DenialReason {
    /// Requested accommodation type does not support the pet species.
    SpeciesAccommodationMismatch,
    /// No source inventory segment matches the requested compatible accommodation kinds.
    NoEligibleSegment,
    /// Local policy data required for the capacity check was unavailable.
    PolicyUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons a boarding request should be waitlisted instead of confirmed.
pub enum WaitlistReason {
    /// The room type is valid for the pet, but all matching rooms are occupied.
    EligibleSegmentFull,
}

#[derive(Debug, Clone, Default)]
/// Deterministic boarding capacity policy that does not invent inventory.
pub struct Policy;

impl Policy {
    /// Evaluates a boarding request against source-derived inventory and returns confirm, waitlist, or denial evidence.
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
