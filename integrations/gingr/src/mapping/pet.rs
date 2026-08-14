use crate::{endpoint, response};
use domain::pet;

use super::{Error, Promoted, ProviderField, Result, Version};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Pet mapping candidate produced from Gingr animal name fields.
pub struct NameCandidate {
    /// Gingr animal identifier kept as source evidence for the mapped pet.
    pub provider_animal_id: endpoint::AnimalId,
    /// Provider display label retained for operator context; NVA-specific naming rules are applied downstream.
    pub name: pet::Name,
}

/// Extracts the pet name Gingr exposed for animal-to-domain mapping.
pub fn name_candidate(
    record: &response::AnimalRecord,
    provenance: domain::source::Provenance,
) -> Result<Promoted<NameCandidate>> {
    let name = record
        .name
        .as_deref()
        .ok_or(Error::MissingRequiredProviderField {
            field: ProviderField::AnimalName,
        })?;
    let name = pet::Name::try_new(name).map_err(|err| Error::InvalidDomainValue {
        field: ProviderField::AnimalName,
        reason: err.to_string(),
    })?;

    Promoted::from_gingr_record(
        NameCandidate {
            provider_animal_id: record.id,
            name,
        },
        record.id.to_string(),
        provenance,
        Version::PetNameV1,
    )
}
