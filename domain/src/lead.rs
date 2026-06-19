//! Cross-service lead conversion triage for resort sales follow-up.
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
/// Lead triage record that turns source contact evidence into safe sales follow-up work.
pub struct Triage {
    /// Existing customer record when staff can link the lead to a known account.
    pub customer_id: Option<CustomerId>,
    /// Channel or campaign that explains where the lead came from.
    pub source: Source,
    /// Service or change the customer appears to be asking about.
    pub intent: Intent,
    /// Sales stage used to rank follow-up labor and booking readiness.
    pub stage: ConversionStage,
    /// Requested resort service when the source evidence is specific enough.
    pub requested_service: Option<ServiceKind>,
    /// Staff-safe next step; automation may draft, route, or summarize but not book or promise capacity.
    pub next_action: NextAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Lead source retained so marketing and intake teams can audit where demand originated.
pub enum Source {
    /// Lead originated from a website form and may be routed to intake follow-up.
    WebsiteForm,
    /// Lead originated from a phone call or voicemail that staff may need to summarize or return.
    Phone,
    /// Lead originated from SMS and should respect texting consent and response boundaries.
    Sms,
    /// Lead originated from email and can support draft replies after staff-safe triage.
    Email,
    /// Lead originated from social media and may need attribution or identity verification.
    SocialMedia,
    /// Local referral name staff can verify before attributing the lead source.
    LocalReferral {
        /// Referral source name preserved for staff attribution and deduplication.
        source_name: SourceName,
    },
    /// Contact or display name used by staff.
    Campaign {
        /// Campaign name preserved for marketing attribution and follow-up reporting.
        name: CampaignName,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Prospect intent signal used to route boarding, daycare, grooming, or training follow-up.
pub enum Intent {
    /// New prospect asking about onboarding, requirements, availability, or first booking.
    NewCustomerIntake,
    /// Prospect wants boarding pricing or availability that staff must confirm before promising.
    BoardingQuote,
    /// Prospect is asking about daycare trial or evaluation; staff must confirm eligibility steps.
    DaycareTrial,
    /// Prospect wants grooming scheduling, which depends on service, pet, and capacity review.
    GroomingAppointment,
    /// Prospect wants training consultation routed to the appropriate trainer or intake path.
    TrainingConsult,
    /// Existing customer appears to need a booking or profile change rather than new intake.
    ExistingCustomerChange,
    /// Lead intent is unclear; automation may summarize but staff must classify before booking promises.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Conversion stage that separates new inquiries from booked, lost, or inactive demand.
pub enum ConversionStage {
    /// New lead awaiting first staff or automated draft response.
    New,
    /// Staff or automation attempted contact and the next step depends on response evidence.
    ContactAttempted,
    /// Lead is paused until the customer supplies missing answers or documents.
    WaitingOnCustomer,
    /// Lead cannot be booked until vaccine, pet profile, policy, or other requirements are confirmed.
    MissingRequirements,
    /// Intake evidence looks booking-ready, but staff must still confirm capacity and policies before committing.
    ReadyToBook,
    /// Lead has converted into booked or active customer work and should avoid duplicate sales follow-up.
    Converted,
    /// Lead is inactive or declined, retained for attribution and future analysis.
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Human-safe next step for converting a lead without overpromising capacity or policy.
pub enum NextAction {
    /// Automation may draft a reply, but sending still follows channel and approval gates.
    DraftReply,
    /// Ask for pet profile details needed before eligibility or booking review.
    RequestMissingPetProfile,
    /// Ask for vaccine proof before trial, daycare, boarding, or grooming readiness decisions.
    RequestVaccineProof,
    /// Staff-confirmed availability can be offered; automation must not invent or hold times.
    OfferReservationTimes,
    /// Route to staff when source facts, policy, or customer context are too ambiguous for automation.
    RouteToHuman {
        /// Business reason staff should review before proceeding.
        reason: operations::operational::Observation,
    },
    /// No current follow-up is appropriate, usually because the lead is converted, lost, or waiting.
    NoAction,
}
