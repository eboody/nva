//! Boarding capacity decisions for room/suite availability.
//!
//! Capacity examples use semantic accommodation paths so a workflow assistant can explain whether
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
//!     entities::LocationId::new(uuid::Uuid::from_u128(1)),
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
use nonempty::NonEmpty;
use std::collections::BTreeSet;

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
/// Validation errors for room-count promotion from provider or staff-entered data.
pub enum RoomCountError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
/// Proof that recorded occupancy exceeds recorded capacity and must be reconciled.
pub struct OverOccupancy {
    total: RoomCount,
    occupied: RoomCount,
    excess: RoomCount,
}

impl OverOccupancy {
    /// Constructs contradiction evidence only when occupied rooms exceed total rooms.
    pub const fn try_new(
        total: RoomCount,
        occupied: RoomCount,
    ) -> std::result::Result<Self, OverOccupancyError> {
        if occupied.get() <= total.get() {
            return Err(OverOccupancyError::NotOverOccupied { total, occupied });
        }

        Ok(Self {
            total,
            occupied,
            excess: RoomCount(occupied.get() - total.get()),
        })
    }

    /// Returns the recorded total capacity involved in the contradiction.
    pub const fn total(self) -> RoomCount {
        self.total
    }

    /// Returns the recorded occupied count involved in the contradiction.
    pub const fn occupied(self) -> RoomCount {
        self.occupied
    }

    /// Returns how many occupied rooms exceed the recorded total capacity.
    pub const fn excess(self) -> RoomCount {
        self.excess
    }
}

impl<'de> Deserialize<'de> for OverOccupancy {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawOverOccupancy {
            total: RoomCount,
            occupied: RoomCount,
            excess: RoomCount,
        }

        let raw = RawOverOccupancy::deserialize(deserializer)?;
        let evidence = Self::try_new(raw.total, raw.occupied).map_err(serde::de::Error::custom)?;
        if evidence.excess != raw.excess {
            return Err(serde::de::Error::custom(
                OverOccupancyError::IncorrectExcess {
                    expected: evidence.excess,
                    actual: raw.excess,
                },
            ));
        }
        Ok(evidence)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation errors for over-occupancy reconciliation evidence.
pub enum OverOccupancyError {
    #[error("occupied room count {occupied:?} does not exceed total room count {total:?}")]
    /// The supplied counts are ordinary available or full occupancy, not a contradiction.
    NotOverOccupied {
        /// Recorded total capacity.
        total: RoomCount,
        /// Recorded occupied rooms.
        occupied: RoomCount,
    },
    #[error("over-occupancy excess must be {expected:?}, not {actual:?}")]
    /// Serialized evidence carried an excess inconsistent with total and occupied counts.
    IncorrectExcess {
        /// Excess derived from occupied minus total.
        expected: RoomCount,
        /// Excess supplied by serialized input.
        actual: RoomCount,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Exhaustive semantic classification of one accommodation segment's occupancy.
pub enum OccupancyState {
    /// Recorded occupancy is below total capacity.
    Available {
        /// Rooms remaining according to the internally consistent counts.
        available: RoomCount,
    },
    /// Recorded occupancy exactly equals total capacity.
    Full,
    /// Recorded occupancy exceeds total capacity and cannot authorize confirmation or waitlisting.
    OverOccupied(OverOccupancy),
}

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

    /// Classifies the counts without collapsing contradictory over-occupancy into an ordinary full state.
    pub const fn occupancy_state(&self) -> OccupancyState {
        if self.occupied.get() < self.total.get() {
            OccupancyState::Available {
                available: RoomCount(self.total.get() - self.occupied.get()),
            }
        } else if self.occupied.get() == self.total.get() {
            OccupancyState::Full
        } else {
            OccupancyState::OverOccupied(OverOccupancy {
                total: self.total,
                occupied: self.occupied,
                excess: RoomCount(self.occupied.get() - self.total.get()),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Point-in-time boarding inventory evidence used to make confirm/waitlist/deny decisions.
pub struct Snapshot {
    segments: NonEmpty<NightlySegmentSnapshot>,
}

impl Snapshot {
    /// Creates a capacity snapshot from one or more nightly accommodation segments.
    pub fn new(segments: Vec<NightlySegmentSnapshot>) -> std::result::Result<Self, SnapshotError> {
        let segments = NonEmpty::from_vec(segments).ok_or(SnapshotError::EmptyInventory)?;
        let mut seen = BTreeSet::new();
        for segment in &segments {
            if !seen.insert(segment.accommodation) {
                return Err(SnapshotError::DuplicateAccommodation {
                    accommodation: segment.accommodation,
                });
            }
        }
        Ok(Self { segments })
    }

    /// Returns the nightly accommodation inventory segments considered by capacity policy.
    pub fn segments(&self) -> impl ExactSizeIterator<Item = &NightlySegmentSnapshot> {
        self.segments.iter()
    }
}

impl<'de> Deserialize<'de> for Snapshot {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawSnapshot {
            segments: Vec<NightlySegmentSnapshot>,
        }

        Self::new(RawSnapshot::deserialize(deserializer)?.segments)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Snapshot validation errors that prevent safe capacity automation.
pub enum SnapshotError {
    #[error("boarding capacity snapshot requires at least one accommodation segment")]
    /// No inventory segments were available, so automation must not infer availability.
    EmptyInventory,
    #[error("boarding capacity snapshot contains duplicate {accommodation:?} inventory")]
    /// More than one segment claimed authority for the same accommodation.
    DuplicateAccommodation {
        /// Accommodation represented by conflicting duplicate segments.
        accommodation: accommodation::Kind,
    },
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
    AvailableForReview {
        /// Accommodation that can be offered from the available source inventory.
        accommodation: accommodation::Kind,
        /// Human approval required before inventory evidence can become an offer or confirmation.
        review_gate: policy::ReviewGate,
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
    /// Source counts contradict one another, so staff must reconcile inventory before proceeding.
    ReconciliationRequired {
        /// Typed evidence preserving the contradictory counts and computed excess.
        anomaly: OverOccupancy,
        /// Human review gate required before corrected capacity evidence may be trusted.
        review_gate: policy::ReviewGate,
    },
}

impl Decision {
    /// Returns the human review gate required before staff override a denied capacity decision.
    pub fn required_review_gate(&self) -> Option<policy::ReviewGate> {
        match self {
            Self::AvailableForReview { review_gate, .. }
            | Self::Deny { review_gate, .. }
            | Self::ReconciliationRequired { review_gate, .. } => Some(review_gate.clone()),
            Self::Waitlist { .. } => None,
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
    /// Evaluates a boarding request against room inventory and returns confirm, waitlist, or denial evidence.
    pub fn evaluate(&self, request: &Request, snapshot: &Snapshot) -> Decision {
        let mut compatible_but_full = false;
        let acceptable_accommodations = request.accommodation.acceptable_kinds();

        for wanted in &acceptable_accommodations {
            if !wanted.supports_species(&request.species) {
                return Decision::Deny {
                    reason: DenialReason::SpeciesAccommodationMismatch,
                    review_gate: policy::ReviewGate::ManagerApproval,
                };
            }

            for segment in snapshot.segments() {
                match segment.occupancy_state() {
                    OccupancyState::OverOccupied(anomaly) if segment.accommodation == *wanted => {
                        return Decision::ReconciliationRequired {
                            anomaly,
                            review_gate: policy::ReviewGate::ManagerApproval,
                        };
                    }
                    OccupancyState::Available { .. }
                    | OccupancyState::Full
                    | OccupancyState::OverOccupied(_) => {}
                }
            }
        }

        for wanted in acceptable_accommodations {
            for segment in snapshot.segments() {
                if segment.accommodation == wanted {
                    match segment.occupancy_state() {
                        OccupancyState::Available { .. } => {
                            return Decision::AvailableForReview {
                                accommodation: wanted,
                                review_gate: policy::ReviewGate::ManagerApproval,
                            };
                        }
                        OccupancyState::Full => compatible_but_full = true,
                        OccupancyState::OverOccupied(anomaly) => {
                            return Decision::ReconciliationRequired {
                                anomaly,
                                review_gate: policy::ReviewGate::ManagerApproval,
                            };
                        }
                    }
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
