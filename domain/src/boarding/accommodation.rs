use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for kind decisions in boarding workflows.
pub enum Kind {
    /// Classic dog suite boarding policy, stay, capacity, or upsell signal.
    ClassicDogSuite,
    /// Luxury dog suite boarding policy, stay, capacity, or upsell signal.
    LuxuryDogSuite,
    /// Cat condo boarding policy, stay, capacity, or upsell signal.
    CatCondo,
}

impl Kind {
    /// Returns this boarding value's supports species.
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
/// Domain vocabulary for preference decisions in boarding workflows.
pub enum Preference {
    /// Specific boarding policy, stay, capacity, or upsell signal.
    Specific(Kind),
    /// Any of boarding policy, stay, capacity, or upsell signal.
    AnyOf(Vec<Kind>),
}

impl Preference {
    /// Returns the acceptable kinds for this boarding value.
    pub fn acceptable_kinds(&self) -> &[Kind] {
        match self {
            Self::Specific(kind) => std::slice::from_ref(kind),
            Self::AnyOf(kinds) => kinds.as_slice(),
        }
    }
}
