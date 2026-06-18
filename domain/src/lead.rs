//! Canonical domain contracts for cross-service lead conversion triage.
//!
//! These types describe resort sales/intake state independent of any one
//! service line. They promote web/phone/SMS/source facts into validated sales
//! workflow state so follow-up labor, booking readiness, and revenue opportunities
//! are visible without letting an agent invent availability or bypass staff review.

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
/// Validated local-referral/source name for lead provenance.
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
/// Validated campaign name used to connect lead work to marketing sources.
pub struct CampaignName(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Source-derived lead triage record for sales follow-up and booking workflows.
pub struct Triage {
    /// Customer id fact promoted into this lead contract.
    pub customer_id: Option<CustomerId>,
    /// Source fact promoted into this lead contract.
    pub source: Source,
    /// Intent fact promoted into this lead contract.
    pub intent: Intent,
    /// Stage fact promoted into this lead contract.
    pub stage: ConversionStage,
    /// Requested service fact promoted into this lead contract.
    pub requested_service: Option<ServiceKind>,
    /// Next action fact promoted into this lead contract.
    pub next_action: NextAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for source decisions in lead workflows.
pub enum Source {
    /// Website form sales lead state, source, or follow-up signal.
    WebsiteForm,
    /// Phone sales lead state, source, or follow-up signal.
    Phone,
    /// Sms sales lead state, source, or follow-up signal.
    Sms,
    /// Email sales lead state, source, or follow-up signal.
    Email,
    /// Social media sales lead state, source, or follow-up signal.
    SocialMedia,
    /// Source name fact promoted into this lead contract.
    LocalReferral {
        /// Source name carried by this variant.
        source_name: SourceName,
    },
    /// Contact or display name used by staff.
    Campaign {
        /// Name carried by this variant.
        name: CampaignName,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for intent decisions in lead workflows.
pub enum Intent {
    /// New customer intake sales lead state, source, or follow-up signal.
    NewCustomerIntake,
    /// Boarding quote sales lead state, source, or follow-up signal.
    BoardingQuote,
    /// Daycare trial sales lead state, source, or follow-up signal.
    DaycareTrial,
    /// Grooming appointment sales lead state, source, or follow-up signal.
    GroomingAppointment,
    /// Training consult sales lead state, source, or follow-up signal.
    TrainingConsult,
    /// Existing customer change sales lead state, source, or follow-up signal.
    ExistingCustomerChange,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for conversion stage decisions in lead workflows.
pub enum ConversionStage {
    /// New sales lead state, source, or follow-up signal.
    New,
    /// Contact attempted sales lead state, source, or follow-up signal.
    ContactAttempted,
    /// Waiting on customer sales lead state, source, or follow-up signal.
    WaitingOnCustomer,
    /// Missing requirements sales lead state, source, or follow-up signal.
    MissingRequirements,
    /// Ready to book sales lead state, source, or follow-up signal.
    ReadyToBook,
    /// Converted sales lead state, source, or follow-up signal.
    Converted,
    /// Lost sales lead state, source, or follow-up signal.
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Human-safe next step for converting a lead without overpromising capacity or policy.
pub enum NextAction {
    /// Draft reply sales lead state, source, or follow-up signal.
    DraftReply,
    /// Request missing pet profile sales lead state, source, or follow-up signal.
    RequestMissingPetProfile,
    /// Request vaccine proof sales lead state, source, or follow-up signal.
    RequestVaccineProof,
    /// Offer reservation times sales lead state, source, or follow-up signal.
    OfferReservationTimes,
    /// Route to human sales lead state, source, or follow-up signal.
    RouteToHuman {
        /// Business reason staff should review before proceeding.
        reason: operations::operational::Observation,
    },
    /// No action sales lead state, source, or follow-up signal.
    NoAction,
}
