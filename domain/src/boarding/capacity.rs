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
pub struct RoomCount(u16);

impl RoomCount {
    pub const fn try_new(value: u16) -> std::result::Result<Self, RoomCountError> {
        Ok(Self(value))
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RoomCountError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct SegmentCounts {
    pub accommodation: accommodation::Kind,
    total: RoomCount,
    occupied: RoomCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NightlySegmentSnapshot {
    pub accommodation: accommodation::Kind,
    total: RoomCount,
    occupied: RoomCount,
}

impl NightlySegmentSnapshot {
    pub const fn from_counts(counts: SegmentCounts) -> Self {
        Self {
            accommodation: counts.accommodation,
            total: counts.total,
            occupied: counts.occupied,
        }
    }

    pub const fn total(&self) -> RoomCount {
        self.total
    }

    pub const fn occupied(&self) -> RoomCount {
        self.occupied
    }

    pub const fn available_rooms(&self) -> RoomCount {
        RoomCount(self.total.get().saturating_sub(self.occupied.get()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    segments: Vec<NightlySegmentSnapshot>,
}

impl Snapshot {
    pub fn new(segments: Vec<NightlySegmentSnapshot>) -> std::result::Result<Self, SnapshotError> {
        if segments.is_empty() {
            return Err(SnapshotError::EmptyInventory);
        }
        Ok(Self { segments })
    }

    pub fn segments(&self) -> &[NightlySegmentSnapshot] {
        &self.segments
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SnapshotError {
    #[error("boarding capacity snapshot requires at least one accommodation segment")]
    EmptyInventory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub location_id: LocationId,
    pub species: crate::entities::Species,
    pub accommodation: accommodation::Preference,
}

impl Request {
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
pub enum Decision {
    Available {
        accommodation: accommodation::Kind,
    },
    Waitlist {
        reason: WaitlistReason,
    },
    Deny {
        reason: DenialReason,
        review_gate: policy::ReviewGate,
    },
}

impl Decision {
    pub fn required_review_gate(&self) -> Option<policy::ReviewGate> {
        match self {
            Self::Deny { review_gate, .. } => Some(review_gate.clone()),
            Self::Available { .. } | Self::Waitlist { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DenialReason {
    SpeciesAccommodationMismatch,
    NoEligibleSegment,
    PolicyUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaitlistReason {
    EligibleSegmentFull,
}

#[derive(Debug, Clone, Default)]
pub struct Policy;

impl Policy {
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
