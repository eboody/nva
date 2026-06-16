//! Canonical domain contracts for cross-service lead conversion triage.
//!
//! These types describe resort sales/intake state independent of any one
//! service line; `operations` retains deprecated legacy compatibility re-exports.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::entities::{CustomerId, ServiceKind};
use crate::operations;

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
pub struct SourceName(String);

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
pub struct Triage {
    pub customer_id: Option<CustomerId>,
    pub source: Source,
    pub intent: Intent,
    pub stage: ConversionStage,
    pub requested_service: Option<ServiceKind>,
    pub next_action: NextAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    WebsiteForm,
    Phone,
    Sms,
    Email,
    SocialMedia,
    LocalReferral { source_name: SourceName },
    Campaign { name: CampaignName },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    NewCustomerIntake,
    BoardingQuote,
    DaycareTrial,
    GroomingAppointment,
    TrainingConsult,
    ExistingCustomerChange,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversionStage {
    New,
    ContactAttempted,
    WaitingOnCustomer,
    MissingRequirements,
    ReadyToBook,
    Converted,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NextAction {
    DraftReply,
    RequestMissingPetProfile,
    RequestVaccineProof,
    OfferReservationTimes,
    RouteToHuman {
        reason: operations::operational::Observation,
    },
    NoAction,
}
