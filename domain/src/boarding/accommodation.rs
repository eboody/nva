//! Boarding accommodation vocabulary for suite/condo matching and species-safe capacity decisions.
//!
//! These accommodation rules keep room-type preferences explicit so automation can recommend availability
//! without inventing unsupported species accommodations or collapsing premium-suite choices.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Accommodation kinds used when matching a boarding request to room or suite inventory.
pub enum Kind {
    /// Standard dog boarding suite option used for baseline dog-room capacity.
    ClassicDogSuite,
    /// Premium dog boarding suite option used for capacity matching and upgrade offers.
    LuxuryDogSuite,
    /// Cat lodging option that must not be matched to dog boarding requests.
    CatCondo,
}

impl Kind {
    /// Reports whether this accommodation can safely serve the requested pet species.
    pub const fn supports_species(self, species: &crate::entities::Species) -> bool {
        matches!(
            (self, species),
            (
                Self::ClassicDogSuite | Self::LuxuryDogSuite,
                crate::entities::Species::Dog
            ) | (Self::CatCondo, crate::entities::Species::Cat)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Guest or staff accommodation preference supplied to a boarding capacity check.
pub enum Preference {
    /// A single requested room type that should be evaluated before alternatives.
    Specific(Kind),
    /// A bounded list of acceptable room types when the guest can accept alternatives.
    AnyOf(Vec<Kind>),
}

impl Preference {
    /// Exposes the acceptable accommodation kinds in evaluation order for capacity policy.
    pub fn acceptable_kinds(&self) -> &[Kind] {
        match self {
            Self::Specific(kind) => std::slice::from_ref(kind),
            Self::AnyOf(kinds) => kinds.as_slice(),
        }
    }
}
