//! Core pet-resort entities and operational records.
//!
//! ## Operator-summary
//!
//! This module supports the shared staff view of pets, customers, reservations, care profiles,
//! documents, vaccine records, care notes, incidents, messages, and approval records. It can
//! reduce labor by keeping the facts needed for triage, safety review, handoff, document review,
//! customer-message approval, and manager queues in one normalized shape instead of scattering
//! them across source-system payloads and free-text notes.
//!
//! It must not automate live booking changes, provider writes, customer sends, payment/refund
//! actions, medical/vaccine/behavior decisions, incident closure, or policy exceptions.
//! Authoritative facts remain the named source record, source document/storage object, policy
//! snapshot, reviewer approval, audit event, and typed domain value for each field. Review
//! gates protect pets, customers, and staff by tying sensitive records to explicit approval
//! targets and lifecycle states before downstream workflows may treat them as cleared.
//!
//! These structs and enums are the normalized domain facts used by workflow, policy, storage, and
//! source adapters. They should be read as normalized operating records: every field is either a source-backed
//! fact, a reviewable derived state, or a safety/labor signal used to reduce manual resort work
//! without bypassing manager, medical, behavior, payment, or customer-message gates.

use chrono::{DateTime, NaiveDate, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use bon::Builder;

use crate::{
    agent, care, customer, document, incident, location, message, payment, pet, policy, portal,
    temperament, vaccine,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable identifier for a resort location across source imports, policies, reports, and workflows.
pub struct LocationId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable identifier for the customer/account responsible for pets, reservations, messages, and payments.
pub struct CustomerId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable identifier for a pet whose care, temperament, vaccine, and reservation facts drive safety decisions.
pub struct PetId(pub Uuid);

/// Reservation-facing source vocabulary embedded in core entity records.
pub mod reservation {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use super::PortalProvider;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Provider or source identifier retained as the stable join key.
    pub struct Id(pub Uuid);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Normalized lifecycle states used to reconcile source-system data with domain workflows.
    pub enum Status {
        /// Inquiry state or source category preserved for normalized resort records.
        Inquiry,
        /// Reservation has been requested but not yet confirmed.
        Requested,
        /// Missing info state or source category preserved for normalized resort records.
        MissingInfo,
        /// Vaccine pending state or source category preserved for normalized resort records.
        VaccinePending,
        /// Special review state or source category preserved for normalized resort records.
        SpecialReview,
        /// Waitlisted state or source category preserved for normalized resort records.
        Waitlisted,
        /// Offered state or source category preserved for normalized resort records.
        Offered,
        /// Reservation has been accepted by the resort.
        Confirmed,
        /// Pet has arrived and is in care.
        CheckedIn,
        /// Active state or source category preserved for normalized resort records.
        Active,
        /// Pet has left care and the stay is complete.
        CheckedOut,
        /// Reservation is no longer active.
        Cancelled,
        /// Rejected state or source category preserved for normalized resort records.
        Rejected,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Origin channel for a reservation or operational fact before it becomes trusted domain evidence.
    pub enum Source {
        /// Portal state or source category preserved for normalized resort records.
        Portal(PortalProvider),
        /// Website form state or source category preserved for normalized resort records.
        WebsiteForm,
        /// Phone transcript state or source category preserved for normalized resort records.
        PhoneTranscript,
        /// Sms state or source category preserved for normalized resort records.
        Sms,
        /// Email state or source category preserved for normalized resort records.
        Email,
        /// Staff created state or source category preserved for normalized resort records.
        StaffCreated,
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
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
pub struct StaffId(String);

/// Manager identifier used when approvals, overrides, or escalations require accountable leadership.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
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
pub struct ManagerId(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Resort location record that scopes local capabilities, timezone, brand, and policy references.
pub struct Location {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: LocationId,
    /// Brand retained from source records for staff review, safety gates, and workflow joins.
    pub brand: Brand,
    /// Contact or display name used by staff.
    pub name: location::Name,
    /// Timezone retained from source records for staff review, safety gates, and workflow joins.
    pub timezone: location::Timezone,
    /// Capabilities retained from source records for staff review, safety gates, and workflow joins.
    pub capabilities: Vec<ServiceKind>,
    /// Policies retained from source records for staff review, safety gates, and workflow joins.
    pub policies: LocationPolicyRefs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Brand family used to group multi-site operating records without losing local resort identity.
pub enum Brand {
    /// Nva pet resorts state or source category preserved for normalized resort records.
    NvaPetResorts,
    /// Pet suites state or source category preserved for normalized resort records.
    PetSuites,
    /// Contact or display name used by staff.
    NeighborhoodPetResort {
        /// Name attached to this variant for reviewers and adapters.
        name: location::Name,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// References to the local policy set that controls automation, vaccine, and play-safety decisions.
pub struct LocationPolicyRefs {
    /// Vaccine policy id retained from source records for staff review, safety gates, and workflow joins.
    pub vaccine_policy_id: policy::Id,
    /// Deposit policy id retained from source records for staff review, safety gates, and workflow joins.
    pub deposit_policy_id: policy::Id,
    /// Playgroup policy id retained from source records for staff review, safety gates, and workflow joins.
    pub playgroup_policy_id: policy::Id,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
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
    /// Gingr reservation and pet-care operating system.
    Gingr,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Pet profile carrying identity, species, age, sex, sterilization, temperament, and care facts for safe service decisions.
pub struct Pet {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: PetId,
    /// Customer id retained from source records for staff review, safety gates, and workflow joins.
    pub customer_id: CustomerId,
    /// Contact or display name used by staff.
    pub name: pet::Name,
    /// Species retained from source records for staff review, safety gates, and workflow joins.
    pub species: Species,
    /// Birth date retained from source records for staff review, safety gates, and workflow joins.
    pub birth_date: Option<NaiveDate>,
    /// Sex retained from source records for staff review, safety gates, and workflow joins.
    pub sex: Option<Sex>,
    /// Spay neuter status retained from source records for staff review, safety gates, and workflow joins.
    pub spay_neuter_status: SpayNeuterStatus,
    #[builder(default)]
    /// Temperament retained from source records for staff review, safety gates, and workflow joins.
    pub temperament: TemperamentProfile,
    #[builder(default)]
    /// Care profile retained from source records for staff review, safety gates, and workflow joins.
    pub care_profile: CareProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Pet species category used by boarding/daycare/play policies and labor planning.
pub enum Species {
    /// Dog guest, using dog-specific policy and capacity rules.
    Dog,
    /// Cat guest, using cat-specific policy and accommodation rules.
    Cat,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Recorded pet sex when the source system supplies it.
pub enum Sex {
    /// Female pet sex recorded for profile and policy context.
    Female,
    /// Male pet sex recorded for profile and policy context.
    Male,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Spay/neuter status used by group-play eligibility, safety review, and policy gating.
pub enum SpayNeuterStatus {
    /// Pet has been spayed for policy and playgroup eligibility checks.
    Spayed,
    /// Pet has been neutered for policy and playgroup eligibility checks.
    Neutered,
    /// Pet is intact and may trigger extra policy review.
    Intact,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder, Default)]
/// Temperament evidence used to decide group-play, individual care, and behavior-review routing.
pub struct TemperamentProfile {
    #[builder(default)]
    /// Group play observation retained from source records for staff review, safety gates, and workflow joins.
    pub group_play_observation: temperament::GroupPlayObservation,
    #[builder(default)]
    /// People orientation retained from source records for staff review, safety gates, and workflow joins.
    pub people_orientation: temperament::PeopleOrientation,
    #[builder(default)]
    /// Rating retained from source records for staff review, safety gates, and workflow joins.
    pub rating: temperament::Rating,
    #[builder(default)]
    /// Behavior observations retained from source records for staff review, safety gates, and workflow joins.
    pub behavior_observations: Vec<temperament::BehaviorObservation>,
    #[builder(default)]
    /// Staff notes retained from source records for staff review, safety gates, and workflow joins.
    pub staff_notes: Vec<temperament::StaffNote>,
}

impl TemperamentProfile {
    /// Reports whether temperament facts require staff evaluation before group play or similar services.
    pub fn needs_staff_play_evaluation(&self) -> bool {
        self.group_play_observation.needs_staff_evaluation()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Feeding, medication, handling, and special-care summary used for staff handoffs and briefings.
pub struct CareProfile {
    /// Feeding instructions retained from source records for staff review, safety gates, and workflow joins.
    pub feeding_instructions: Option<care::FeedingInstruction>,
    /// Medications retained from source records for staff review, safety gates, and workflow joins.
    pub medications: Vec<MedicationInstruction>,
    /// Allergies retained from source records for staff review, safety gates, and workflow joins.
    pub allergies: Vec<care::AllergyName>,
    /// Medical conditions retained from source records for staff review, safety gates, and workflow joins.
    pub medical_conditions: Vec<care::MedicalConditionName>,
    /// Emergency contact retained from source records for staff review, safety gates, and workflow joins.
    pub emergency_contact: Option<care::ContactRef>,
    /// Veterinarian contact retained from source records for staff review, safety gates, and workflow joins.
    pub veterinarian_contact: Option<care::ContactRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Medication instruction that must remain explicit for care safety and shift handoff evidence.
pub struct MedicationInstruction {
    /// Contact or display name used by staff.
    pub name: care::MedicationName,
    /// Dose retained from source records for staff review, safety gates, and workflow joins.
    pub dose: care::MedicationDose,
    /// Schedule retained from source records for staff review, safety gates, and workflow joins.
    pub schedule: care::MedicationSchedule,
    /// Review requirement retained from source records for staff review, safety gates, and workflow joins.
    pub review_requirement: care::MedicationReviewRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Reservation record tying customer, pet, service, status, deposit, add-ons, and safety stops together.
pub struct Reservation {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: reservation::Id,
    /// Location id retained from source records for staff review, safety gates, and workflow joins.
    pub location_id: LocationId,
    /// Customer id retained from source records for staff review, safety gates, and workflow joins.
    pub customer_id: CustomerId,
    /// Pet ids retained from source records for staff review, safety gates, and workflow joins.
    pub pet_ids: Vec<PetId>,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceKind,
    /// Status retained from source records for staff review, safety gates, and workflow joins.
    pub status: reservation::Status,
    /// Starts at retained from source records for staff review, safety gates, and workflow joins.
    pub starts_at: DateTime<Utc>,
    /// Ends at retained from source records for staff review, safety gates, and workflow joins.
    pub ends_at: DateTime<Utc>,
    /// Deposit retained from source records for staff review, safety gates, and workflow joins.
    pub deposit: Option<Deposit>,
    /// Source retained from source records for staff review, safety gates, and workflow joins.
    pub source: reservation::Source,
    #[builder(default)]
    /// Requested add ons retained from source records for staff review, safety gates, and workflow joins.
    pub requested_add_ons: Vec<AddOn>,
    #[builder(default)]
    /// Hard stops retained from source records for staff review, safety gates, and workflow joins.
    pub hard_stops: Vec<HardStop>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Resort service line used for labor planning, capacity, policy, upsell, and workflow routing.
pub enum ServiceKind {
    /// Overnight stay service line.
    Boarding,
    /// Single-day play visit without overnight lodging.
    DayPlay,
    /// Daytime boarding care with lodging-style supervision.
    DayBoarding,
    /// Grooming service line or care-note category.
    Grooming,
    /// Training service line or care-note category.
    Training,
    /// Day-spa service package.
    DaySpa,
}

/// Shared deposit type used by reservation, payment, and approval records.
pub type Deposit = payment::Deposit;
/// Shared payment status used by checkout, deposit, refund, and approval records.
pub type PaymentStatus = payment::DepositStatus;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Optional reservation add-ons that affect labor, revenue, care planning, or customer follow-up.
pub enum AddOn {
    /// Group-play add-on or accommodation feature.
    GroupPlay,
    /// Individual play add-on for pets not suited to group play.
    IndividualPlay,
    /// Premium suite with webcam visibility.
    WebcamSuite,
    /// Bath offered before departure from boarding.
    ExitBath,
    /// Progress report shared with the customer during care.
    PawgressReport,
    /// Medication service that requires care instructions.
    MedicationAdministration,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(crate::reservation::AddOnLabel),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Non-ignorable condition that blocks or routes a reservation before staff or customer action proceeds.
pub enum HardStop {
    /// Missing required vaccine state or source category preserved for normalized resort records.
    MissingRequiredVaccine(policy::VaccineName),
    /// Ineligible for group play state or source category preserved for normalized resort records.
    IneligibleForGroupPlay(policy::play::IneligibilityReason),
    /// Pet is in heat and requires policy handling.
    InHeat,
    /// Age below minimum weeks state or source category preserved for normalized resort records.
    AgeBelowMinimumWeeks(crate::reservation::AgeThreshold),
    /// Medical or medication information requires review before service.
    MedicalOrMedicationReviewRequired,
    /// Behavior history requires review before service.
    BehaviorReviewRequired,
    /// Deposit must be collected before the booking is secure.
    DepositRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable identifier for a document artifact used as vaccine, waiver, medical, or incident evidence.
pub struct DocumentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable identifier for a vaccine compliance record tied to a pet and proof document.
pub struct VaccineRecordId(pub Uuid);

/// Care-note vocabulary for staff-visible, customer-visible, and internal handoff notes.
pub mod care_note {
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use super::{IncidentId, PetId, reservation};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Provider or source identifier retained as the stable join key.
    pub struct Id(pub Uuid);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Subject that a care, document, incident, audit, or message record is about.
    pub enum Subject {
        /// Pet record participating in the workflow.
        Pet(PetId),
        /// Reservation record participating in the workflow.
        Reservation(reservation::Id),
        /// Incident record participating in the workflow.
        Incident(IncidentId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Care-note category used to route safety, feeding, medication, behavior, and staff handoff information.
    pub enum Kind {
        /// Feeding state or source category preserved for normalized resort records.
        Feeding,
        /// Medication state or source category preserved for normalized resort records.
        Medication,
        /// Medical state or source category preserved for normalized resort records.
        Medical,
        /// Behavior state or source category preserved for normalized resort records.
        Behavior,
        /// Grooming service line or care-note category.
        Grooming,
        /// Training service line or care-note category.
        Training,
        /// General state or source category preserved for normalized resort records.
        General,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Visibility rule that determines whether a care note may be shown to customers or only staff.
    pub enum Visibility {
        /// Internal only state or source category preserved for normalized resort records.
        InternalOnly,
        /// Customer visible state or source category preserved for normalized resort records.
        CustomerVisible,
        /// Customer visible after review state or source category preserved for normalized resort records.
        CustomerVisibleAfterReview,
    }

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 2000),
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
    pub struct Body(String);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable identifier for a pet, customer, or operational incident requiring evidence and follow-up.
pub struct IncidentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable identifier for a customer or internal message workflow.
pub struct MessageId(pub Uuid);

/// Approval record vocabulary for review-gated automation outcomes.
pub mod approval {
    use bon::Builder;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use super::{
        ActorRef, DocumentId, IncidentId, MessageId, VaccineRecordId, policy, reservation,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Provider or source identifier retained as the stable join key.
    pub struct Id(pub Uuid);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Approval record showing who decided, what target was reviewed, and what lifecycle state resulted.
    pub struct Record {
        /// Id retained from source records for staff review, safety gates, and workflow joins.
        pub id: Id,
        /// Target retained from source records for staff review, safety gates, and workflow joins.
        pub target: Target,
        /// Gate retained from source records for staff review, safety gates, and workflow joins.
        pub gate: policy::ReviewGate,
        /// Lifecycle retained from source records for staff review, safety gates, and workflow joins.
        pub lifecycle: Lifecycle,
        /// Requested by retained from source records for staff review, safety gates, and workflow joins.
        pub requested_by: ActorRef,
        /// Requested at retained from source records for staff review, safety gates, and workflow joins.
        pub requested_at: DateTime<Utc>,
        #[builder(default)]
        /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
        pub audit_refs: Vec<crate::audit::EventId>,
    }

    impl Record {
        /// Returns the normalized operational status represented by this record.
        pub fn status(&self) -> Status {
            self.lifecycle.status()
        }

        /// Reports whether this approval gate currently applies to the target workflow.
        pub fn is_applicable(&self) -> bool {
            matches!(self.lifecycle, Lifecycle::Approved { .. })
        }

        /// Reports whether the review lifecycle has reached an approval, rejection, or non-applicable endpoint.
        pub fn is_terminal_decision(&self) -> bool {
            self.lifecycle.is_terminal_decision()
        }

        /// Returns the accountable actor and timestamp when the review reached a terminal decision.
        pub fn decision_actor_and_time(&self) -> Option<(&ActorRef, DateTime<Utc>)> {
            self.lifecycle.decision_actor_and_time()
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Operational artifact that an approval gate is allowed to approve, reject, or mark non-applicable.
    pub enum Target {
        /// Reservation record participating in the workflow.
        Reservation(reservation::Id),
        /// Customer or pet document participating in review.
        Document(DocumentId),
        /// Vaccination document or status record under review.
        VaccineRecord(VaccineRecordId),
        /// Incident record participating in the workflow.
        Incident(IncidentId),
        /// Customer communication record participating in approval.
        Message(MessageId),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Approval lifecycle state for draft, requested, approved, rejected, or non-applicable review gates.
    pub enum Lifecycle {
        /// Approval requested state or source category preserved for normalized resort records.
        ApprovalRequested,
        /// Approved state or source category preserved for normalized resort records.
        Approved {
            /// Decided by retained from source records for staff review, safety gates, and workflow joins.
            decided_by: ActorRef,
            /// Decided at retained from source records for staff review, safety gates, and workflow joins.
            decided_at: DateTime<Utc>,
        },
        /// Rejected state or source category preserved for normalized resort records.
        Rejected {
            /// Decided by retained from source records for staff review, safety gates, and workflow joins.
            decided_by: ActorRef,
            /// Decided at retained from source records for staff review, safety gates, and workflow joins.
            decided_at: DateTime<Utc>,
        },
        /// Reservation is no longer active.
        Cancelled,
        /// Superseded state or source category preserved for normalized resort records.
        Superseded,
    }

    impl Lifecycle {
        /// Returns the normalized operational status represented by this record.
        pub fn status(&self) -> Status {
            match self {
                Self::ApprovalRequested => Status::ApprovalRequested,
                Self::Approved { .. } => Status::Approved,
                Self::Rejected { .. } => Status::Rejected,
                Self::Cancelled => Status::Cancelled,
                Self::Superseded => Status::Superseded,
            }
        }

        /// Reports whether the review lifecycle has reached an approval, rejection, or non-applicable endpoint.
        pub fn is_terminal_decision(&self) -> bool {
            matches!(self, Self::Approved { .. } | Self::Rejected { .. })
        }

        /// Returns the accountable actor and timestamp when the review reached a terminal decision.
        pub fn decision_actor_and_time(&self) -> Option<(&ActorRef, DateTime<Utc>)> {
            match self {
                Self::Approved {
                    decided_by,
                    decided_at,
                }
                | Self::Rejected {
                    decided_by,
                    decided_at,
                } => Some((decided_by, *decided_at)),
                Self::ApprovalRequested | Self::Cancelled | Self::Superseded => None,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Normalized lifecycle states used to reconcile source-system data with domain workflows.
    pub enum Status {
        /// Approval requested state or source category preserved for normalized resort records.
        ApprovalRequested,
        /// Approved state or source category preserved for normalized resort records.
        Approved,
        /// Rejected state or source category preserved for normalized resort records.
        Rejected,
        /// Reservation is no longer active.
        Cancelled,
        /// Superseded state or source category preserved for normalized resort records.
        Superseded,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Document record tying storage, classification, source, scan, redaction, and review status together.
pub struct Document {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: DocumentId,
    /// Location id retained from source records for staff review, safety gates, and workflow joins.
    pub location_id: LocationId,
    /// Subject retained from source records for staff review, safety gates, and workflow joins.
    pub subject: DocumentSubject,
    /// Classification retained from source records for staff review, safety gates, and workflow joins.
    pub classification: document::Classification,
    /// Source retained from source records for staff review, safety gates, and workflow joins.
    pub source: document::Source,
    /// Uploaded by actor retained from source records for staff review, safety gates, and workflow joins.
    pub uploaded_by_actor: ActorRef,
    /// Uploaded at retained from source records for staff review, safety gates, and workflow joins.
    pub uploaded_at: DateTime<Utc>,
    /// Original file retained from source records for staff review, safety gates, and workflow joins.
    pub original_file: document::OriginalFile,
    /// Storage ref retained from source records for staff review, safety gates, and workflow joins.
    pub storage_ref: document::StorageRef,
    /// Virus scan status retained from source records for staff review, safety gates, and workflow joins.
    pub virus_scan_status: document::VirusScanStatus,
    /// Pii redaction status retained from source records for staff review, safety gates, and workflow joins.
    pub pii_redaction_status: document::PiiRedactionStatus,
    /// Verification status retained from source records for staff review, safety gates, and workflow joins.
    pub verification_status: document::Status,
    #[builder(default)]
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl Document {
    /// Reports whether the document must be reviewed before agents or staff treat it as usable evidence.
    pub fn requires_human_review_before_use(&self) -> bool {
        matches!(
            self.verification_status,
            document::Status::Received
                | document::Status::Extracting
                | document::Status::ExtractionFailed
                | document::Status::AwaitingReview
                | document::Status::QuarantinedRejected
        ) || !matches!(self.virus_scan_status, document::VirusScanStatus::Passed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Entity or workflow subject a document is evidence for.
pub enum DocumentSubject {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Incident record participating in the workflow.
    Incident(IncidentId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Vaccine compliance record linking pet, vaccine name, expiration, proof document, and review status.
pub struct VaccineRecord {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: VaccineRecordId,
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Vaccine name retained from source records for staff review, safety gates, and workflow joins.
    pub vaccine_name: policy::VaccineName,
    /// Source document id retained from source records for staff review, safety gates, and workflow joins.
    pub source_document_id: DocumentId,
    /// Status retained from source records for staff review, safety gates, and workflow joins.
    pub status: vaccine::Status,
    /// Effective on retained from source records for staff review, safety gates, and workflow joins.
    pub effective_on: NaiveDate,
    /// Expires on retained from source records for staff review, safety gates, and workflow joins.
    pub expires_on: Option<NaiveDate>,
    /// Review gate retained from source records for staff review, safety gates, and workflow joins.
    pub review_gate: policy::ReviewGate,
    #[builder(default)]
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl VaccineRecord {
    /// Reports whether vaccine proof is still unverified, rejected, or otherwise unsafe for compliance automation.
    pub fn requires_human_review_before_compliance(&self) -> bool {
        matches!(
            self.status,
            vaccine::Status::SuggestedExtracted
                | vaccine::Status::PendingReview
                | vaccine::Status::Rejected
                | vaccine::Status::ExceptionRequested
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Care note with author, visibility, subject, body, source, and review-sensitive timestamps.
pub struct CareNote {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: care_note::Id,
    /// Subject retained from source records for staff review, safety gates, and workflow joins.
    pub subject: care_note::Subject,
    /// Kind retained from source records for staff review, safety gates, and workflow joins.
    pub kind: care_note::Kind,
    /// Visibility retained from source records for staff review, safety gates, and workflow joins.
    pub visibility: care_note::Visibility,
    /// Body retained from source records for staff review, safety gates, and workflow joins.
    pub body: care_note::Body,
    /// Author retained from source records for staff review, safety gates, and workflow joins.
    pub author: ActorRef,
    /// Recorded at retained from source records for staff review, safety gates, and workflow joins.
    pub recorded_at: DateTime<Utc>,
    #[builder(default)]
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl CareNote {
    /// Reports whether this care note may be surfaced to customers without an additional approval gate.
    pub fn is_customer_visible_without_review(&self) -> bool {
        matches!(self.visibility, care_note::Visibility::CustomerVisible)
            && !matches!(
                self.kind,
                care_note::Kind::Medication | care_note::Kind::Medical | care_note::Kind::Behavior
            )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Incident record used for manager attention, safety follow-up, customer messaging, and audit evidence.
pub struct Incident {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: IncidentId,
    /// Location id retained from source records for staff review, safety gates, and workflow joins.
    pub location_id: LocationId,
    /// Primary subject retained from source records for staff review, safety gates, and workflow joins.
    pub primary_subject: IncidentSubject,
    /// Category retained from source records for staff review, safety gates, and workflow joins.
    pub category: incident::Category,
    /// Severity retained from source records for staff review, safety gates, and workflow joins.
    pub severity: incident::Severity,
    /// Status retained from source records for staff review, safety gates, and workflow joins.
    pub status: incident::Status,
    /// Reported by retained from source records for staff review, safety gates, and workflow joins.
    pub reported_by: ActorRef,
    /// Reported at retained from source records for staff review, safety gates, and workflow joins.
    pub reported_at: DateTime<Utc>,
    /// Summary retained from source records for staff review, safety gates, and workflow joins.
    pub summary: incident::Summary,
    #[builder(default)]
    /// Required review gates retained from source records for staff review, safety gates, and workflow joins.
    pub required_review_gates: Vec<policy::ReviewGate>,
    #[builder(default)]
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl Incident {
    /// Reports whether the incident is still active enough to require manager attention.
    pub fn requires_manager_attention(&self) -> bool {
        matches!(
            self.status,
            incident::Status::NeedsManagerReview | incident::Status::LegalHold
        ) || matches!(
            self.severity,
            incident::Severity::High | incident::Severity::Critical
        ) || self
            .required_review_gates
            .contains(&policy::ReviewGate::ManagerApproval)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Entity or workflow subject affected by an incident.
pub enum IncidentSubject {
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Resort location record participating in the workflow.
    Location(LocationId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Customer/internal message record that tracks subject, channel, draft/reference body, approval, and delivery state.
pub struct Message {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: MessageId,
    /// Subject retained from source records for staff review, safety gates, and workflow joins.
    pub subject: MessageSubject,
    /// Direction retained from source records for staff review, safety gates, and workflow joins.
    pub direction: message::Direction,
    /// Channel retained from source records for staff review, safety gates, and workflow joins.
    pub channel: message::Channel,
    /// Status retained from source records for staff review, safety gates, and workflow joins.
    pub status: message::Status,
    /// Body ref retained from source records for staff review, safety gates, and workflow joins.
    pub body_ref: message::BodyRef,
    /// Approval gate retained from source records for staff review, safety gates, and workflow joins.
    pub approval_gate: Option<policy::ReviewGate>,
    #[builder(default)]
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl Message {
    /// Reports whether the message is still a draft or awaiting approval before any outbound send.
    pub fn requires_approval_before_send(&self) -> bool {
        self.approval_gate.is_some()
            || matches!(self.status, message::Status::ApprovalRequested)
            || matches!(self.direction, message::Direction::OutboundDraft)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Entity or workflow subject that a message refers to.
pub enum MessageSubject {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Incident record participating in the workflow.
    Incident(IncidentId),
    /// Approval decision record participating in audit history.
    Approval(approval::Id),
}

/// Audit vocabulary for source-backed event trails across automated and staff actions.
pub mod audit {
    use chrono::{DateTime, Utc};
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    use super::{
        CustomerId, DocumentId, IncidentId, LocationId, MessageId, PetId, VaccineRecordId,
        approval, care_note, reservation,
    };

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Audit event capturing actor, subject, action, timestamp, and metadata evidence.
    pub struct Event {
        /// At retained from source records for staff review, safety gates, and workflow joins.
        pub at: DateTime<Utc>,
        /// Actor retained from source records for staff review, safety gates, and workflow joins.
        pub actor: super::ActorRef,
        /// Subject retained from source records for staff review, safety gates, and workflow joins.
        pub subject: Subject,
        /// Action retained from source records for staff review, safety gates, and workflow joins.
        pub action: Action,
        /// Metadata retained from source records for staff review, safety gates, and workflow joins.
        pub metadata: BTreeMap<MetadataKey, MetadataValue>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Subject that a care, document, incident, audit, or message record is about.
    pub enum Subject {
        /// Customer record participating in the workflow.
        Customer(CustomerId),
        /// Pet record participating in the workflow.
        Pet(PetId),
        /// Reservation record participating in the workflow.
        Reservation(reservation::Id),
        /// Resort location record participating in the workflow.
        Location(LocationId),
        /// Customer or pet document participating in review.
        Document(DocumentId),
        /// Vaccination document or status record under review.
        VaccineRecord(VaccineRecordId),
        /// Care note state or source category preserved for normalized resort records.
        CareNote(care_note::Id),
        /// Incident record participating in the workflow.
        Incident(IncidentId),
        /// Customer communication record participating in approval.
        Message(MessageId),
        /// Approval decision record participating in audit history.
        Approval(approval::Id),
        /// Workflow event state or source category preserved for normalized resort records.
        WorkflowEvent(crate::workflow::EventId),
        /// External system object referenced from domain history.
        External {
            /// Provider retained from source records for staff review, safety gates, and workflow joins.
            provider: crate::workflow::external::Provider,
            /// Id retained from source records for staff review, safety gates, and workflow joins.
            id: crate::workflow::external::Id,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Auditable action category produced by staff, source ingestion, policy, approval, or automation.
    pub enum Action {
        /// Customer profile updated state or source category preserved for normalized resort records.
        CustomerProfileUpdated,
        /// Pet profile updated state or source category preserved for normalized resort records.
        PetProfileUpdated,
        /// Reservation status suggested state or source category preserved for normalized resort records.
        ReservationStatusSuggested,
        /// Reservation status changed state or source category preserved for normalized resort records.
        ReservationStatusChanged,
        /// Policy decision recorded state or source category preserved for normalized resort records.
        PolicyDecisionRecorded,
        /// Document received state or source category preserved for normalized resort records.
        DocumentReceived,
        /// Vaccine record review requested state or source category preserved for normalized resort records.
        VaccineRecordReviewRequested,
        /// Incident status changed state or source category preserved for normalized resort records.
        IncidentStatusChanged,
        /// Message approval requested state or source category preserved for normalized resort records.
        MessageApprovalRequested,
        /// Approval decision recorded state or source category preserved for normalized resort records.
        ApprovalDecisionRecorded,
        /// Workflow event recorded state or source category preserved for normalized resort records.
        WorkflowEventRecorded,
        /// Extension point for provider-specific values not modeled directly.
        Extension(ActionLabel),
    }

    /// Human-readable audit action label for imported or locally defined operational events.
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
    pub struct ActionLabel(String);

    /// Audit metadata key used to preserve source evidence without flattening it into prose.
    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 80),
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
    pub struct MetadataKey(String);

    /// Audit metadata value attached to an event for review, reporting, or source repair.
    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 500),
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
    pub struct MetadataValue(String);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Actor that performed or is accountable for an audited action.
pub enum ActorRef {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Staff id retained from source records for staff review, safety gates, and workflow joins.
    Staff {
        /// Staff id attached to this variant for reviewers and adapters.
        staff_id: StaffId,
    },
    /// Manager id retained from source records for staff review, safety gates, and workflow joins.
    Manager {
        /// Manager id attached to this variant for reviewers and adapters.
        manager_id: ManagerId,
    },
    /// System state or source category preserved for normalized resort records.
    System,
    /// Workflow retained from source records for staff review, safety gates, and workflow joins.
    Agent {
        /// Workflow attached to this variant for reviewers and adapters.
        workflow: agent::Name,
    },
}
