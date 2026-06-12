//! Canonical domain contracts for cross-service lead conversion triage.
//!
//! These types describe resort sales/intake state independent of any one
//! service line; `operations` retains legacy compatibility re-exports.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::entities::{CustomerId, ServiceKind};
use crate::operations::OperationalObservation;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct LeadSourceName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct CampaignName(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lead {
    pub customer_id: Option<CustomerId>,
    pub source: LeadSource,
    pub intent: LeadIntent,
    pub stage: LeadConversionStage,
    pub requested_service: Option<ServiceKind>,
    pub next_action: LeadNextAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadSource {
    WebsiteForm,
    Phone,
    Sms,
    Email,
    SocialMedia,
    LocalReferral { source_name: LeadSourceName },
    Campaign { name: CampaignName },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadIntent {
    NewCustomerIntake,
    BoardingQuote,
    DaycareTrial,
    GroomingAppointment,
    TrainingConsult,
    ExistingCustomerChange,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadConversionStage {
    New,
    ContactAttempted,
    WaitingOnCustomer,
    MissingRequirements,
    ReadyToBook,
    Converted,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadNextAction {
    DraftReply,
    RequestMissingPetProfile,
    RequestVaccineProof,
    OfferReservationTimes,
    RouteToHuman { reason: OperationalObservation },
    NoAction,
}
