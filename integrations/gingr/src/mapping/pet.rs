use crate::{endpoint, response};
use domain::pet;

use super::{Error, ProviderField, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Pet mapping candidate produced from Gingr animal name fields.
pub struct NameCandidate {
    /// Persisted provider animal id value for this record.
    pub provider_animal_id: endpoint::AnimalId,
    /// Human-readable display name paired with the stable code.
    pub name: pet::Name,
}

/// Extracts the pet name Gingr exposed for animal-to-domain mapping.
pub fn name_candidate(record: &response::AnimalRecord) -> Result<NameCandidate> {
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

    Ok(NameCandidate {
        provider_animal_id: record.id,
        name,
    })
}
