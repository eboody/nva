//! Customer/account profiles, contact routing, and portal references.

use bon::Builder;
use serde::{Deserialize, Serialize};

use super::identifiers::CustomerId;
use crate::{customer, portal};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Customer/account profile used for reservation ownership, consent-sensitive messaging, and follow-up work.
pub struct Customer {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: CustomerId,
    /// Full name retained from source records for staff review, safety gates, and workflow joins.
    pub full_name: customer::Name,
    /// Email retained from source records for staff review, safety gates, and workflow joins.
    pub email: Option<customer::Email>,
    /// Mobile phone retained from source records for staff review, safety gates, and workflow joins.
    pub mobile_phone: Option<customer::Phone>,
    /// Preferred contact retained from source records for staff review, safety gates, and workflow joins.
    pub preferred_contact: ContactChannel,
    /// Portal account retained from source records for staff review, safety gates, and workflow joins.
    pub portal_account: Option<PortalAccountRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Link to the customer portal account that supplied or owns source records.
pub struct PortalAccountRef {
    /// Provider retained from source records for staff review, safety gates, and workflow joins.
    pub provider: PortalProvider,
    /// External customer id retained from source records for staff review, safety gates, and workflow joins.
    pub external_customer_id: portal::CustomerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Portal provider that owns the account or operational record.
pub enum PortalProvider {
    /// Customer portal hosted by the active external provider.
    ProviderHosted,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Customer contact channel preference or observed route used by draft/message workflows.
pub enum ContactChannel {
    /// Email state or source category preserved for normalized resort records.
    Email,
    /// Sms state or source category preserved for normalized resort records.
    Sms,
    /// Phone state or source category preserved for normalized resort records.
    Phone,
    /// Portal state or source category preserved for normalized resort records.
    Portal,
}
