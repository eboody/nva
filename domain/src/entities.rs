use chrono::{DateTime, NaiveDate, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use bon::Builder;

use crate::{
    agent, audit, care, customer, document, incident, location, message, payment, pet, policy,
    portal, reservation, temperament, vaccine,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LocationId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CustomerId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PetId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ReservationId(pub Uuid);

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
pub struct Location {
    pub id: LocationId,
    pub brand: Brand,
    pub name: location::Name,
    pub timezone: location::Timezone,
    pub capabilities: Vec<ServiceKind>,
    pub policies: LocationPolicyRefs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Brand {
    NvaPetResorts,
    PetSuites,
    NeighborhoodPetResort { name: location::Name },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationPolicyRefs {
    pub vaccine_policy_id: policy::Id,
    pub deposit_policy_id: policy::Id,
    pub playgroup_policy_id: policy::Id,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Customer {
    pub id: CustomerId,
    pub full_name: customer::Name,
    pub email: Option<customer::Email>,
    pub mobile_phone: Option<customer::Phone>,
    pub preferred_contact: ContactChannel,
    pub portal_account: Option<PortalAccountRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalAccountRef {
    pub provider: PortalProvider,
    pub external_customer_id: portal::CustomerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortalProvider {
    Gingr,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactChannel {
    Email,
    Sms,
    Phone,
    Portal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Pet {
    pub id: PetId,
    pub customer_id: CustomerId,
    pub name: pet::Name,
    pub species: Species,
    pub birth_date: Option<NaiveDate>,
    pub sex: Option<Sex>,
    pub spay_neuter_status: SpayNeuterStatus,
    #[builder(default)]
    pub temperament: TemperamentProfile,
    #[builder(default)]
    pub care_profile: CareProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Species {
    Dog,
    Cat,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sex {
    Female,
    Male,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpayNeuterStatus {
    Spayed,
    Neutered,
    Intact,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder, Default)]
pub struct TemperamentProfile {
    #[builder(default)]
    pub group_play_observation: temperament::GroupPlayObservation,
    #[builder(default)]
    pub people_orientation: temperament::PeopleOrientation,
    #[builder(default)]
    pub rating: temperament::TemperamentRating,
    #[builder(default)]
    pub behavior_observations: Vec<temperament::BehaviorObservation>,
    #[builder(default)]
    pub staff_notes: Vec<temperament::StaffNote>,
}

impl TemperamentProfile {
    pub fn needs_staff_play_evaluation(&self) -> bool {
        self.group_play_observation.needs_staff_evaluation()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CareProfile {
    pub feeding_instructions: Option<care::FeedingInstruction>,
    pub medications: Vec<MedicationInstruction>,
    pub allergies: Vec<care::AllergyName>,
    pub medical_conditions: Vec<care::MedicalConditionName>,
    pub emergency_contact: Option<care::ContactRef>,
    pub veterinarian_contact: Option<care::ContactRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct MedicationInstruction {
    pub name: care::MedicationName,
    pub dose: care::MedicationDose,
    pub schedule: care::MedicationSchedule,
    pub review_requirement: care::MedicationReviewRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Reservation {
    pub id: ReservationId,
    pub location_id: LocationId,
    pub customer_id: CustomerId,
    pub pet_ids: Vec<PetId>,
    pub service: ServiceKind,
    pub status: ReservationStatus,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub deposit: Option<Deposit>,
    pub source: ReservationSource,
    #[builder(default)]
    pub requested_add_ons: Vec<AddOn>,
    #[builder(default)]
    pub hard_stops: Vec<HardStop>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceKind {
    Boarding,
    DayPlay,
    DayBoarding,
    Grooming,
    Training,
    DaySpa,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReservationStatus {
    Inquiry,
    Requested,
    MissingInfo,
    VaccinePending,
    SpecialReview,
    Waitlisted,
    Offered,
    Confirmed,
    CheckedIn,
    Active,
    CheckedOut,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReservationSource {
    Portal(PortalProvider),
    WebsiteForm,
    PhoneTranscript,
    Sms,
    Email,
    StaffCreated,
}

pub type Deposit = payment::Deposit;
pub type PaymentStatus = payment::DepositStatus;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddOn {
    GroupPlay,
    IndividualPlay,
    WebcamSuite,
    ExitBath,
    PawgressReport,
    MedicationAdministration,
    Other(reservation::AddOnLabel),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardStop {
    MissingRequiredVaccine(policy::VaccineName),
    IneligibleForGroupPlay(policy::PlayIneligibilityReason),
    InHeat,
    AgeBelowMinimumWeeks(reservation::AgeThreshold),
    MedicalOrMedicationReviewRequired,
    BehaviorReviewRequired,
    DepositRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VaccineRecordId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CareNoteId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IncidentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ApprovalId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Document {
    pub id: DocumentId,
    pub location_id: LocationId,
    pub subject: DocumentSubject,
    pub classification: document::Classification,
    pub source: document::Source,
    pub uploaded_by_actor: ActorRef,
    pub uploaded_at: DateTime<Utc>,
    pub original_file: document::OriginalFile,
    pub storage_ref: document::StorageRef,
    pub virus_scan_status: document::VirusScanStatus,
    pub pii_redaction_status: document::PiiRedactionStatus,
    pub verification_status: document::Status,
    #[builder(default)]
    pub audit_refs: Vec<audit::EventId>,
}

impl Document {
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
pub enum DocumentSubject {
    Customer(CustomerId),
    Pet(PetId),
    Reservation(ReservationId),
    Incident(IncidentId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct VaccineRecord {
    pub id: VaccineRecordId,
    pub pet_id: PetId,
    pub vaccine_name: policy::VaccineName,
    pub source_document_id: DocumentId,
    pub status: vaccine::Status,
    pub effective_on: NaiveDate,
    pub expires_on: Option<NaiveDate>,
    pub review_gate: policy::ReviewGate,
    #[builder(default)]
    pub audit_refs: Vec<audit::EventId>,
}

impl VaccineRecord {
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
pub struct CareNote {
    pub id: CareNoteId,
    pub subject: CareNoteSubject,
    pub kind: CareNoteKind,
    pub visibility: CareNoteVisibility,
    pub body: CareNoteBody,
    pub author: ActorRef,
    pub recorded_at: DateTime<Utc>,
    #[builder(default)]
    pub audit_refs: Vec<audit::EventId>,
}

impl CareNote {
    pub fn is_customer_visible_without_review(&self) -> bool {
        matches!(self.visibility, CareNoteVisibility::CustomerVisible)
            && !matches!(
                self.kind,
                CareNoteKind::Medication | CareNoteKind::Medical | CareNoteKind::Behavior
            )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CareNoteSubject {
    Pet(PetId),
    Reservation(ReservationId),
    Incident(IncidentId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CareNoteKind {
    Feeding,
    Medication,
    Medical,
    Behavior,
    Grooming,
    Training,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CareNoteVisibility {
    InternalOnly,
    CustomerVisible,
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
pub struct CareNoteBody(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Incident {
    pub id: IncidentId,
    pub location_id: LocationId,
    pub primary_subject: IncidentSubject,
    pub category: incident::Category,
    pub severity: incident::Severity,
    pub status: incident::Status,
    pub reported_by: ActorRef,
    pub reported_at: DateTime<Utc>,
    pub summary: incident::Summary,
    #[builder(default)]
    pub required_review_gates: Vec<policy::ReviewGate>,
    #[builder(default)]
    pub audit_refs: Vec<audit::EventId>,
}

impl Incident {
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
pub enum IncidentSubject {
    Pet(PetId),
    Reservation(ReservationId),
    Customer(CustomerId),
    Location(LocationId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Message {
    pub id: MessageId,
    pub subject: MessageSubject,
    pub direction: message::Direction,
    pub channel: message::Channel,
    pub status: message::Status,
    pub body_ref: message::BodyRef,
    pub approval_gate: Option<policy::ReviewGate>,
    #[builder(default)]
    pub audit_refs: Vec<audit::EventId>,
}

impl Message {
    pub fn requires_approval_before_send(&self) -> bool {
        self.approval_gate.is_some()
            || matches!(self.status, message::Status::ApprovalRequested)
            || matches!(self.direction, message::Direction::OutboundDraft)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageSubject {
    Customer(CustomerId),
    Pet(PetId),
    Reservation(ReservationId),
    Incident(IncidentId),
    Approval(ApprovalId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct ApprovalRecord {
    pub id: ApprovalId,
    pub target: ApprovalTarget,
    pub gate: policy::ReviewGate,
    pub lifecycle: ApprovalLifecycle,
    pub requested_by: ActorRef,
    pub requested_at: DateTime<Utc>,
    #[builder(default)]
    pub audit_refs: Vec<audit::EventId>,
}

impl ApprovalRecord {
    pub fn status(&self) -> ApprovalStatus {
        self.lifecycle.status()
    }

    pub fn is_applicable(&self) -> bool {
        matches!(self.lifecycle, ApprovalLifecycle::Approved { .. })
    }

    pub fn is_terminal_decision(&self) -> bool {
        self.lifecycle.is_terminal_decision()
    }

    pub fn decision_actor_and_time(&self) -> Option<(&ActorRef, DateTime<Utc>)> {
        self.lifecycle.decision_actor_and_time()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalTarget {
    Reservation(ReservationId),
    Document(DocumentId),
    VaccineRecord(VaccineRecordId),
    Incident(IncidentId),
    Message(MessageId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalLifecycle {
    ApprovalRequested,
    Approved {
        decided_by: ActorRef,
        decided_at: DateTime<Utc>,
    },
    Rejected {
        decided_by: ActorRef,
        decided_at: DateTime<Utc>,
    },
    Cancelled,
    Superseded,
}

impl ApprovalLifecycle {
    pub fn status(&self) -> ApprovalStatus {
        match self {
            Self::ApprovalRequested => ApprovalStatus::ApprovalRequested,
            Self::Approved { .. } => ApprovalStatus::Approved,
            Self::Rejected { .. } => ApprovalStatus::Rejected,
            Self::Cancelled => ApprovalStatus::Cancelled,
            Self::Superseded => ApprovalStatus::Superseded,
        }
    }

    pub fn is_terminal_decision(&self) -> bool {
        matches!(self, Self::Approved { .. } | Self::Rejected { .. })
    }

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
pub enum ApprovalStatus {
    ApprovalRequested,
    Approved,
    Rejected,
    Cancelled,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub at: DateTime<Utc>,
    pub actor: ActorRef,
    pub subject: AuditSubject,
    pub action: AuditAction,
    pub metadata: BTreeMap<AuditMetadataKey, AuditMetadataValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditSubject {
    Customer(CustomerId),
    Pet(PetId),
    Reservation(ReservationId),
    Location(LocationId),
    Document(DocumentId),
    VaccineRecord(VaccineRecordId),
    CareNote(CareNoteId),
    Incident(IncidentId),
    Message(MessageId),
    Approval(ApprovalId),
    WorkflowEvent(crate::workflow::EventId),
    External {
        provider: crate::workflow::external::Provider,
        id: crate::workflow::external::Id,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    CustomerProfileUpdated,
    PetProfileUpdated,
    ReservationStatusSuggested,
    ReservationStatusChanged,
    PolicyDecisionRecorded,
    DocumentReceived,
    VaccineRecordReviewRequested,
    IncidentStatusChanged,
    MessageApprovalRequested,
    ApprovalDecisionRecorded,
    WorkflowEventRecorded,
    Extension(AuditActionLabel),
}

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
pub struct AuditActionLabel(String);

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
pub struct AuditMetadataKey(String);

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
pub struct AuditMetadataValue(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorRef {
    Customer(CustomerId),
    Staff { staff_id: StaffId },
    Manager { manager_id: ManagerId },
    System,
    Agent { workflow: agent::Name },
}
