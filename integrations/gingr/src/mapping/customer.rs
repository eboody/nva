use crate::{endpoint, response};
use domain::{customer, entities};

use super::{Error, ProviderField, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Customer mapping candidate produced from Gingr owner contact fields.
pub struct ContactCandidate {
    /// Gingr owner identifier kept as source evidence for the mapped customer.
    pub provider_owner_id: endpoint::OwnerId,
    /// Customer name assembled from Gingr owner fields; useful for staff-facing drafts but not proof of legal identity.
    pub full_name: customer::Name,
    /// Email address observed from Gingr and carried as customer-contact evidence.
    pub email: Option<customer::Email>,
    /// Mobile phone observed from Gingr and carried as customer-contact evidence.
    pub mobile_phone: Option<customer::Phone>,
    /// Preferred contact channel inferred from provider contact fields for NVA workflow routing.
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
