use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    ClassicDogSuite,
    LuxuryDogSuite,
    CatCondo,
}

impl Kind {
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
pub enum Preference {
    Specific(Kind),
    AnyOf(Vec<Kind>),
}

impl Preference {
    pub fn acceptable_kinds(&self) -> &[Kind] {
        match self {
            Self::Specific(kind) => std::slice::from_ref(kind),
            Self::AnyOf(kinds) => kinds.as_slice(),
        }
    }
}
