//! Core pet-resort entities and operational records.
//!
//! These structs and enums are the normalized domain facts used by workflow, policy, storage, and
//! source adapters. They should be read as external contracts: every field is either a source-backed
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
    /// Source-backed id carried by this normalized pet-resort entity.
    pub id: LocationId,
    /// Source-backed brand carried by this normalized pet-resort entity.
    pub brand: Brand,
    /// Contact or display name used by staff.
    pub name: location::Name,
    /// Source-backed timezone carried by this normalized pet-resort entity.
    pub timezone: location::Timezone,
    /// Source-backed capabilities carried by this normalized pet-resort entity.
    pub capabilities: Vec<ServiceKind>,
    /// Source-backed policies carried by this normalized pet-resort entity.
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
        /// Name carried by this variant.
        name: location::Name,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// References to the local policy set that controls automation, vaccine, and play-safety decisions.
pub struct LocationPolicyRefs {
    /// Source-backed vaccine policy ID carried by this normalized pet-resort entity.
    pub vaccine_policy_id: policy::Id,
    /// Source-backed deposit policy ID carried by this normalized pet-resort entity.
    pub deposit_policy_id: policy::Id,
    /// Source-backed playgroup policy ID carried by this normalized pet-resort entity.
    pub playgroup_policy_id: policy::Id,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Customer/account profile used for reservation ownership, consent-sensitive messaging, and follow-up work.
pub struct Customer {
    /// Source-backed id carried by this normalized pet-resort entity.
    pub id: CustomerId,
    /// Source-backed full name carried by this normalized pet-resort entity.
    pub full_name: customer::Name,
    /// Source-backed email carried by this normalized pet-resort entity.
    pub email: Option<customer::Email>,
    /// Source-backed mobile phone carried by this normalized pet-resort entity.
    pub mobile_phone: Option<customer::Phone>,
    /// Source-backed preferred contact carried by this normalized pet-resort entity.
    pub preferred_contact: ContactChannel,
    /// Source-backed portal account carried by this normalized pet-resort entity.
    pub portal_account: Option<PortalAccountRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Link to the customer portal account that supplied or owns source records.
pub struct PortalAccountRef {
    /// Source-backed provider carried by this normalized pet-resort entity.
    pub provider: PortalProvider,
    /// Source-backed external customer ID carried by this normalized pet-resort entity.
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
    /// Source-backed id carried by this normalized pet-resort entity.
    pub id: PetId,
    /// Source-backed customer ID carried by this normalized pet-resort entity.
    pub customer_id: CustomerId,
    /// Contact or display name used by staff.
    pub name: pet::Name,
    /// Source-backed species carried by this normalized pet-resort entity.
    pub species: Species,
    /// Source-backed birth date carried by this normalized pet-resort entity.
    pub birth_date: Option<NaiveDate>,
    /// Source-backed sex carried by this normalized pet-resort entity.
    pub sex: Option<Sex>,
    /// Source-backed spay neuter status carried by this normalized pet-resort entity.
    pub spay_neuter_status: SpayNeuterStatus,
    #[builder(default)]
    /// Source-backed temperament carried by this normalized pet-resort entity.
    pub temperament: TemperamentProfile,
    #[builder(default)]
    /// Source-backed care profile carried by this normalized pet-resort entity.
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
    /// Source-backed group play observation carried by this normalized pet-resort entity.
    pub group_play_observation: temperament::GroupPlayObservation,
    #[builder(default)]
    /// Source-backed people orientation carried by this normalized pet-resort entity.
    pub people_orientation: temperament::PeopleOrientation,
    #[builder(default)]
    /// Source-backed rating carried by this normalized pet-resort entity.
    pub rating: temperament::Rating,
    #[builder(default)]
    /// Source-backed behavior observations carried by this normalized pet-resort entity.
    pub behavior_observations: Vec<temperament::BehaviorObservation>,
    #[builder(default)]
    /// Source-backed staff notes carried by this normalized pet-resort entity.
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
    /// Source-backed feeding instructions carried by this normalized pet-resort entity.
    pub feeding_instructions: Option<care::FeedingInstruction>,
    /// Source-backed medications carried by this normalized pet-resort entity.
    pub medications: Vec<MedicationInstruction>,
    /// Source-backed allergies carried by this normalized pet-resort entity.
    pub allergies: Vec<care::AllergyName>,
    /// Source-backed medical conditions carried by this normalized pet-resort entity.
    pub medical_conditions: Vec<care::MedicalConditionName>,
    /// Source-backed emergency contact carried by this normalized pet-resort entity.
    pub emergency_contact: Option<care::ContactRef>,
    /// Source-backed veterinarian contact carried by this normalized pet-resort entity.
    pub veterinarian_contact: Option<care::ContactRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Medication instruction that must remain explicit for care safety and shift handoff evidence.
pub struct MedicationInstruction {
    /// Contact or display name used by staff.
    pub name: care::MedicationName,
    /// Source-backed dose carried by this normalized pet-resort entity.
    pub dose: care::MedicationDose,
    /// Source-backed schedule carried by this normalized pet-resort entity.
    pub schedule: care::MedicationSchedule,
    /// Source-backed review requirement carried by this normalized pet-resort entity.
    pub review_requirement: care::MedicationReviewRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Reservation record tying customer, pet, service, status, deposit, add-ons, and safety stops together.
pub struct Reservation {
    /// Source-backed id carried by this normalized pet-resort entity.
    pub id: reservation::Id,
    /// Source-backed location ID carried by this normalized pet-resort entity.
    pub location_id: LocationId,
    /// Source-backed customer ID carried by this normalized pet-resort entity.
    pub customer_id: CustomerId,
    /// Source-backed pet IDs carried by this normalized pet-resort entity.
    pub pet_ids: Vec<PetId>,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceKind,
    /// Source-backed status carried by this normalized pet-resort entity.
    pub status: reservation::Status,
    /// Source-backed starts at carried by this normalized pet-resort entity.
    pub starts_at: DateTime<Utc>,
    /// Source-backed ends at carried by this normalized pet-resort entity.
    pub ends_at: DateTime<Utc>,
    /// Source-backed deposit carried by this normalized pet-resort entity.
    pub deposit: Option<Deposit>,
    /// Source-backed source carried by this normalized pet-resort entity.
    pub source: reservation::Source,
    #[builder(default)]
    /// Source-backed requested add ons carried by this normalized pet-resort entity.
    pub requested_add_ons: Vec<AddOn>,
    #[builder(default)]
    /// Source-backed hard stops carried by this normalized pet-resort entity.
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

/// Shared deposit type used across the entities boundary.
pub type Deposit = payment::Deposit;
/// Shared payment status type used across the entities boundary.
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
    /// Visibility boundary that determines whether a care note may be shown to customers or only staff.
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
        /// Source-backed id carried by this normalized pet-resort entity.
        pub id: Id,
        /// Source-backed target carried by this normalized pet-resort entity.
        pub target: Target,
        /// Source-backed gate carried by this normalized pet-resort entity.
        pub gate: policy::ReviewGate,
        /// Source-backed lifecycle carried by this normalized pet-resort entity.
        pub lifecycle: Lifecycle,
        /// Source-backed requested by carried by this normalized pet-resort entity.
        pub requested_by: ActorRef,
        /// Source-backed requested at carried by this normalized pet-resort entity.
        pub requested_at: DateTime<Utc>,
        #[builder(default)]
        /// Source-backed audit refs carried by this normalized pet-resort entity.
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
            /// Source-backed decided by carried by this normalized pet-resort entity.
            decided_by: ActorRef,
            /// Source-backed decided at carried by this normalized pet-resort entity.
            decided_at: DateTime<Utc>,
        },
        /// Rejected state or source category preserved for normalized resort records.
        Rejected {
            /// Source-backed decided by carried by this normalized pet-resort entity.
            decided_by: ActorRef,
            /// Source-backed decided at carried by this normalized pet-resort entity.
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
    /// Source-backed id carried by this normalized pet-resort entity.
    pub id: DocumentId,
    /// Source-backed location ID carried by this normalized pet-resort entity.
    pub location_id: LocationId,
    /// Source-backed subject carried by this normalized pet-resort entity.
    pub subject: DocumentSubject,
    /// Source-backed classification carried by this normalized pet-resort entity.
    pub classification: document::Classification,
    /// Source-backed source carried by this normalized pet-resort entity.
    pub source: document::Source,
    /// Source-backed uploaded by actor carried by this normalized pet-resort entity.
    pub uploaded_by_actor: ActorRef,
    /// Source-backed uploaded at carried by this normalized pet-resort entity.
    pub uploaded_at: DateTime<Utc>,
    /// Source-backed original file carried by this normalized pet-resort entity.
    pub original_file: document::OriginalFile,
    /// Source-backed storage ref carried by this normalized pet-resort entity.
    pub storage_ref: document::StorageRef,
    /// Source-backed virus scan status carried by this normalized pet-resort entity.
    pub virus_scan_status: document::VirusScanStatus,
    /// Source-backed pii redaction status carried by this normalized pet-resort entity.
    pub pii_redaction_status: document::PiiRedactionStatus,
    /// Source-backed verification status carried by this normalized pet-resort entity.
    pub verification_status: document::Status,
    #[builder(default)]
    /// Source-backed audit refs carried by this normalized pet-resort entity.
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
    /// Source-backed id carried by this normalized pet-resort entity.
    pub id: VaccineRecordId,
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Source-backed vaccine name carried by this normalized pet-resort entity.
    pub vaccine_name: policy::VaccineName,
    /// Source-backed source document ID carried by this normalized pet-resort entity.
    pub source_document_id: DocumentId,
    /// Source-backed status carried by this normalized pet-resort entity.
    pub status: vaccine::Status,
    /// Source-backed effective on carried by this normalized pet-resort entity.
    pub effective_on: NaiveDate,
    /// Source-backed expires on carried by this normalized pet-resort entity.
    pub expires_on: Option<NaiveDate>,
    /// Source-backed review gate carried by this normalized pet-resort entity.
    pub review_gate: policy::ReviewGate,
    #[builder(default)]
    /// Source-backed audit refs carried by this normalized pet-resort entity.
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
    /// Source-backed id carried by this normalized pet-resort entity.
    pub id: care_note::Id,
    /// Source-backed subject carried by this normalized pet-resort entity.
    pub subject: care_note::Subject,
    /// Source-backed kind carried by this normalized pet-resort entity.
    pub kind: care_note::Kind,
    /// Source-backed visibility carried by this normalized pet-resort entity.
    pub visibility: care_note::Visibility,
    /// Source-backed body carried by this normalized pet-resort entity.
    pub body: care_note::Body,
    /// Source-backed author carried by this normalized pet-resort entity.
    pub author: ActorRef,
    /// Source-backed recorded at carried by this normalized pet-resort entity.
    pub recorded_at: DateTime<Utc>,
    #[builder(default)]
    /// Source-backed audit refs carried by this normalized pet-resort entity.
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
    /// Source-backed id carried by this normalized pet-resort entity.
    pub id: IncidentId,
    /// Source-backed location ID carried by this normalized pet-resort entity.
    pub location_id: LocationId,
    /// Source-backed primary subject carried by this normalized pet-resort entity.
    pub primary_subject: IncidentSubject,
    /// Source-backed category carried by this normalized pet-resort entity.
    pub category: incident::Category,
    /// Source-backed severity carried by this normalized pet-resort entity.
    pub severity: incident::Severity,
    /// Source-backed status carried by this normalized pet-resort entity.
    pub status: incident::Status,
    /// Source-backed reported by carried by this normalized pet-resort entity.
    pub reported_by: ActorRef,
    /// Source-backed reported at carried by this normalized pet-resort entity.
    pub reported_at: DateTime<Utc>,
    /// Source-backed summary carried by this normalized pet-resort entity.
    pub summary: incident::Summary,
    #[builder(default)]
    /// Source-backed required review gates carried by this normalized pet-resort entity.
    pub required_review_gates: Vec<policy::ReviewGate>,
    #[builder(default)]
    /// Source-backed audit refs carried by this normalized pet-resort entity.
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
    /// Source-backed id carried by this normalized pet-resort entity.
    pub id: MessageId,
    /// Source-backed subject carried by this normalized pet-resort entity.
    pub subject: MessageSubject,
    /// Source-backed direction carried by this normalized pet-resort entity.
    pub direction: message::Direction,
    /// Source-backed channel carried by this normalized pet-resort entity.
    pub channel: message::Channel,
    /// Source-backed status carried by this normalized pet-resort entity.
    pub status: message::Status,
    /// Source-backed body ref carried by this normalized pet-resort entity.
    pub body_ref: message::BodyRef,
    /// Source-backed approval gate carried by this normalized pet-resort entity.
    pub approval_gate: Option<policy::ReviewGate>,
    #[builder(default)]
    /// Source-backed audit refs carried by this normalized pet-resort entity.
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
        /// Source-backed at carried by this normalized pet-resort entity.
        pub at: DateTime<Utc>,
        /// Source-backed actor carried by this normalized pet-resort entity.
        pub actor: super::ActorRef,
        /// Source-backed subject carried by this normalized pet-resort entity.
        pub subject: Subject,
        /// Source-backed action carried by this normalized pet-resort entity.
        pub action: Action,
        /// Source-backed metadata carried by this normalized pet-resort entity.
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
            /// Source-backed provider carried by this normalized pet-resort entity.
            provider: crate::workflow::external::Provider,
            /// Source-backed id carried by this normalized pet-resort entity.
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
    /// Source-backed staff ID carried by this normalized pet-resort entity.
    Staff {
        /// Staff id carried by this variant.
        staff_id: StaffId,
    },
    /// Source-backed manager ID carried by this normalized pet-resort entity.
    Manager {
        /// Manager id carried by this variant.
        manager_id: ManagerId,
    },
    /// System state or source category preserved for normalized resort records.
    System,
    /// Source-backed workflow carried by this normalized pet-resort entity.
    Agent {
        /// Workflow carried by this variant.
        workflow: agent::Name,
    },
}
