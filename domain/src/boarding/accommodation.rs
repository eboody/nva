//! Boarding accommodation vocabulary for suite/condo matching and species-safe capacity decisions.
//!
//! These accommodation rules keep room-type preferences explicit so automation can recommend availability
//! without inventing unsupported species accommodations or collapsing premium-suite choices.

use super::*;
use nonempty::NonEmpty;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    /// An ordered, non-empty list of distinct room types when the guest can accept alternatives.
    AnyOf(Alternatives),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Non-empty, duplicate-free accommodation alternatives in guest preference order.
pub struct Alternatives(NonEmpty<Kind>);

impl Alternatives {
    /// Validates ordered alternatives without erasing the guest's preference order.
    pub fn try_new(alternatives: Vec<Kind>) -> Result<Self, AlternativesError> {
        let alternatives = NonEmpty::from_vec(alternatives).ok_or(AlternativesError::Empty)?;
        let mut seen = BTreeSet::new();
        for accommodation in alternatives.iter().copied() {
            if !seen.insert(accommodation) {
                return Err(AlternativesError::Duplicate { accommodation });
            }
        }
        Ok(Self(alternatives))
    }

    /// Iterates over acceptable accommodations in guest preference order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Kind> {
        self.0.iter()
    }
}

impl<'de> Deserialize<'de> for Alternatives {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(Vec::<Kind>::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures for ordered accommodation alternatives.
pub enum AlternativesError {
    #[error("boarding accommodation alternatives require at least one accommodation")]
    /// No accommodation could satisfy an `AnyOf` preference.
    Empty,
    #[error("boarding accommodation alternatives contain duplicate {accommodation:?}")]
    /// An accommodation appeared more than once in the preference order.
    Duplicate {
        /// Accommodation duplicated in the preference input.
        accommodation: Kind,
    },
}

impl Preference {
    /// Exposes the acceptable accommodation kinds in evaluation order for capacity policy.
    pub fn acceptable_kinds(&self) -> Vec<Kind> {
        match self {
            Self::Specific(kind) => vec![*kind],
            Self::AnyOf(kinds) => kinds.iter().copied().collect(),
        }
    }
}
