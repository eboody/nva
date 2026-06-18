use crate::{endpoint, response};
use domain::{customer, entities};

use super::{Error, ProviderField, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Customer mapping candidate produced from Gingr owner contact fields.
pub struct ContactCandidate {
    /// Persisted provider owner id value for this record.
    pub provider_owner_id: endpoint::OwnerId,
    /// Persisted full name value for this record.
    pub full_name: customer::Name,
    /// Persisted email value for this record.
    pub email: Option<customer::Email>,
    /// Persisted mobile phone value for this record.
    pub mobile_phone: Option<customer::Phone>,
    /// Persisted preferred contact value for this record.
    pub preferred_contact: entities::ContactChannel,
}

/// Extracts customer contact fields Gingr exposed for owner-to-domain mapping.
pub fn contact_candidate(record: &response::OwnerRecord) -> Result<ContactCandidate> {
    let full_name = record
        .display_name()
        .ok_or(Error::MissingRequiredProviderField {
            field: ProviderField::OwnerName,
        })?;
    let full_name =
        customer::Name::try_new(full_name).map_err(|err| Error::InvalidDomainValue {
            field: ProviderField::OwnerName,
            reason: err.to_string(),
        })?;
    let email = record
        .email
        .as_ref()
        .map(|email| customer::Email::try_new(email.as_str()))
        .transpose()
        .map_err(|err| Error::InvalidDomainValue {
            field: ProviderField::OwnerName,
            reason: err.to_string(),
        })?;
    let mobile_phone = record
        .cell_phone
        .as_deref()
        .map(customer::Phone::try_new)
        .transpose()
        .map_err(|err| Error::InvalidDomainValue {
            field: ProviderField::OwnerName,
            reason: err.to_string(),
        })?;
    let preferred_contact = if email.is_some() {
        entities::ContactChannel::Email
    } else if mobile_phone.is_some() {
        entities::ContactChannel::Sms
    } else {
        entities::ContactChannel::Portal
    };

    Ok(ContactCandidate {
        provider_owner_id: record.id,
        full_name,
        email,
        mobile_phone,
        preferred_contact,
    })
}
