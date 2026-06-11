use crate::{endpoint, response};
use domain::pet;

use super::{Error, ProviderField, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameCandidate {
    pub provider_animal_id: endpoint::AnimalId,
    pub name: pet::Name,
}

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
        provider_animal_id: endpoint::AnimalId::new(record.id),
        name,
    })
}
