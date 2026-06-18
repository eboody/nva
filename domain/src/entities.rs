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
/// Typed location id domain value that keeps raw primitives out of entities workflows.
pub struct LocationId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed customer id domain value that keeps raw primitives out of entities workflows.
pub struct CustomerId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed pet id domain value that keeps raw primitives out of entities workflows.
pub struct PetId(pub Uuid);

/// Reservation boundary for entities contracts.
pub mod reservation {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use super::PortalProvider;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Provider or source identifier retained as the stable join key.
    pub struct Id(pub Uuid);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Normalized reservation states observed during source-data ingestion.
    pub enum Status {
        /// Inquiry category on a core customer, pet, reservation, or audit record.
        Inquiry,
        /// Reservation has been requested but not yet confirmed.
        Requested,
        /// Missing info category on a core customer, pet, reservation, or audit record.
        MissingInfo,
        /// Vaccine pending category on a core customer, pet, reservation, or audit record.
        VaccinePending,
        /// Special review category on a core customer, pet, reservation, or audit record.
        SpecialReview,
        /// Waitlisted category on a core customer, pet, reservation, or audit record.
        Waitlisted,
        /// Offered category on a core customer, pet, reservation, or audit record.
        Offered,
        /// Reservation has been accepted by the resort.
        Confirmed,
        /// Pet has arrived and is in care.
        CheckedIn,
        /// Active category on a core customer, pet, reservation, or audit record.
        Active,
        /// Pet has left care and the stay is complete.
        CheckedOut,
        /// Reservation is no longer active.
        Cancelled,
        /// Rejected category on a core customer, pet, reservation, or audit record.
        Rejected,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for source decisions in entities workflows.
    pub enum Source {
        /// Portal category on a core customer, pet, reservation, or audit record.
        Portal(PortalProvider),
        /// Website form category on a core customer, pet, reservation, or audit record.
        WebsiteForm,
        /// Phone transcript category on a core customer, pet, reservation, or audit record.
        PhoneTranscript,
        /// Sms category on a core customer, pet, reservation, or audit record.
        Sms,
        /// Email category on a core customer, pet, reservation, or audit record.
        Email,
        /// Staff created category on a core customer, pet, reservation, or audit record.
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
/// Typed location domain value that keeps raw primitives out of entities workflows.
pub struct Location {
    /// Id fact promoted into this entities contract.
    pub id: LocationId,
    /// Brand fact promoted into this entities contract.
    pub brand: Brand,
    /// Contact or display name used by staff.
    pub name: location::Name,
    /// Timezone fact promoted into this entities contract.
    pub timezone: location::Timezone,
    /// Capabilities fact promoted into this entities contract.
    pub capabilities: Vec<ServiceKind>,
    /// Policies fact promoted into this entities contract.
    pub policies: LocationPolicyRefs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for brand decisions in entities workflows.
pub enum Brand {
    /// Nva pet resorts category on a core customer, pet, reservation, or audit record.
    NvaPetResorts,
    /// Pet suites category on a core customer, pet, reservation, or audit record.
    PetSuites,
    /// Contact or display name used by staff.
    NeighborhoodPetResort {
        /// Name carried by this variant.
        name: location::Name,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed location policy refs domain value that keeps raw primitives out of entities workflows.
pub struct LocationPolicyRefs {
    /// Vaccine policy id fact promoted into this entities contract.
    pub vaccine_policy_id: policy::Id,
    /// Deposit policy id fact promoted into this entities contract.
    pub deposit_policy_id: policy::Id,
    /// Playgroup policy id fact promoted into this entities contract.
    pub playgroup_policy_id: policy::Id,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed customer domain value that keeps raw primitives out of entities workflows.
pub struct Customer {
    /// Id fact promoted into this entities contract.
    pub id: CustomerId,
    /// Full name fact promoted into this entities contract.
    pub full_name: customer::Name,
    /// Email fact promoted into this entities contract.
    pub email: Option<customer::Email>,
    /// Mobile phone fact promoted into this entities contract.
    pub mobile_phone: Option<customer::Phone>,
    /// Preferred contact fact promoted into this entities contract.
    pub preferred_contact: ContactChannel,
    /// Portal account fact promoted into this entities contract.
    pub portal_account: Option<PortalAccountRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed portal account ref domain value that keeps raw primitives out of entities workflows.
pub struct PortalAccountRef {
    /// Provider fact promoted into this entities contract.
    pub provider: PortalProvider,
    /// External customer id fact promoted into this entities contract.
    pub external_customer_id: portal::CustomerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for portal provider decisions in entities workflows.
pub enum PortalProvider {
    /// Gingr reservation and pet-care operating system.
    Gingr,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for contact channel decisions in entities workflows.
pub enum ContactChannel {
    /// Email category on a core customer, pet, reservation, or audit record.
    Email,
    /// Sms category on a core customer, pet, reservation, or audit record.
    Sms,
    /// Phone category on a core customer, pet, reservation, or audit record.
    Phone,
    /// Portal category on a core customer, pet, reservation, or audit record.
    Portal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed pet domain value that keeps raw primitives out of entities workflows.
pub struct Pet {
    /// Id fact promoted into this entities contract.
    pub id: PetId,
    /// Customer id fact promoted into this entities contract.
    pub customer_id: CustomerId,
    /// Contact or display name used by staff.
    pub name: pet::Name,
    /// Species fact promoted into this entities contract.
    pub species: Species,
    /// Birth date fact promoted into this entities contract.
    pub birth_date: Option<NaiveDate>,
    /// Sex fact promoted into this entities contract.
    pub sex: Option<Sex>,
    /// Spay neuter status fact promoted into this entities contract.
    pub spay_neuter_status: SpayNeuterStatus,
    #[builder(default)]
    /// Temperament fact promoted into this entities contract.
    pub temperament: TemperamentProfile,
    #[builder(default)]
    /// Care profile fact promoted into this entities contract.
    pub care_profile: CareProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for species decisions in entities workflows.
pub enum Species {
    /// Dog guest, using dog-specific policy and capacity rules.
    Dog,
    /// Cat guest, using cat-specific policy and accommodation rules.
    Cat,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for sex decisions in entities workflows.
pub enum Sex {
    /// Female pet sex recorded for profile and policy context.
    Female,
    /// Male pet sex recorded for profile and policy context.
    Male,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for spay neuter status decisions in entities workflows.
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
/// Typed temperament profile domain value that keeps raw primitives out of entities workflows.
pub struct TemperamentProfile {
    #[builder(default)]
    /// Group play observation fact promoted into this entities contract.
    pub group_play_observation: temperament::GroupPlayObservation,
    #[builder(default)]
    /// People orientation fact promoted into this entities contract.
    pub people_orientation: temperament::PeopleOrientation,
    #[builder(default)]
    /// Rating fact promoted into this entities contract.
    pub rating: temperament::Rating,
    #[builder(default)]
    /// Behavior observations fact promoted into this entities contract.
    pub behavior_observations: Vec<temperament::BehaviorObservation>,
    #[builder(default)]
    /// Staff notes fact promoted into this entities contract.
    pub staff_notes: Vec<temperament::StaffNote>,
}

impl TemperamentProfile {
    /// Returns the needs staff play evaluation for this entities value.
    pub fn needs_staff_play_evaluation(&self) -> bool {
        self.group_play_observation.needs_staff_evaluation()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Typed care profile domain value that keeps raw primitives out of entities workflows.
pub struct CareProfile {
    /// Feeding instructions fact promoted into this entities contract.
    pub feeding_instructions: Option<care::FeedingInstruction>,
    /// Medications fact promoted into this entities contract.
    pub medications: Vec<MedicationInstruction>,
    /// Allergies fact promoted into this entities contract.
    pub allergies: Vec<care::AllergyName>,
    /// Medical conditions fact promoted into this entities contract.
    pub medical_conditions: Vec<care::MedicalConditionName>,
    /// Emergency contact fact promoted into this entities contract.
    pub emergency_contact: Option<care::ContactRef>,
    /// Veterinarian contact fact promoted into this entities contract.
    pub veterinarian_contact: Option<care::ContactRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed medication instruction domain value that keeps raw primitives out of entities workflows.
pub struct MedicationInstruction {
    /// Contact or display name used by staff.
    pub name: care::MedicationName,
    /// Dose fact promoted into this entities contract.
    pub dose: care::MedicationDose,
    /// Schedule fact promoted into this entities contract.
    pub schedule: care::MedicationSchedule,
    /// Review requirement fact promoted into this entities contract.
    pub review_requirement: care::MedicationReviewRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed reservation domain value that keeps raw primitives out of entities workflows.
pub struct Reservation {
    /// Id fact promoted into this entities contract.
    pub id: reservation::Id,
    /// Location id fact promoted into this entities contract.
    pub location_id: LocationId,
    /// Customer id fact promoted into this entities contract.
    pub customer_id: CustomerId,
    /// Pet ids fact promoted into this entities contract.
    pub pet_ids: Vec<PetId>,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceKind,
    /// Status fact promoted into this entities contract.
    pub status: reservation::Status,
    /// Starts at fact promoted into this entities contract.
    pub starts_at: DateTime<Utc>,
    /// Ends at fact promoted into this entities contract.
    pub ends_at: DateTime<Utc>,
    /// Deposit fact promoted into this entities contract.
    pub deposit: Option<Deposit>,
    /// Source fact promoted into this entities contract.
    pub source: reservation::Source,
    #[builder(default)]
    /// Requested add ons fact promoted into this entities contract.
    pub requested_add_ons: Vec<AddOn>,
    #[builder(default)]
    /// Hard stops fact promoted into this entities contract.
    pub hard_stops: Vec<HardStop>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for service kind decisions in entities workflows.
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
/// Domain vocabulary for add on decisions in entities workflows.
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
/// Domain vocabulary for hard stop decisions in entities workflows.
pub enum HardStop {
    /// Missing required vaccine category on a core customer, pet, reservation, or audit record.
    MissingRequiredVaccine(policy::VaccineName),
    /// Ineligible for group play category on a core customer, pet, reservation, or audit record.
    IneligibleForGroupPlay(policy::play::IneligibilityReason),
    /// Pet is in heat and requires policy handling.
    InHeat,
    /// Age below minimum weeks category on a core customer, pet, reservation, or audit record.
    AgeBelowMinimumWeeks(crate::reservation::AgeThreshold),
    /// Medical or medication information requires review before service.
    MedicalOrMedicationReviewRequired,
    /// Behavior history requires review before service.
    BehaviorReviewRequired,
    /// Deposit must be collected before the booking is secure.
    DepositRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed document id domain value that keeps raw primitives out of entities workflows.
pub struct DocumentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed vaccine record id domain value that keeps raw primitives out of entities workflows.
pub struct VaccineRecordId(pub Uuid);

/// Care note boundary for entities contracts.
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
    /// Domain vocabulary for subject decisions in entities workflows.
    pub enum Subject {
        /// Pet record participating in the workflow.
        Pet(PetId),
        /// Reservation record participating in the workflow.
        Reservation(reservation::Id),
        /// Incident record participating in the workflow.
        Incident(IncidentId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for kind decisions in entities workflows.
    pub enum Kind {
        /// Feeding category on a core customer, pet, reservation, or audit record.
        Feeding,
        /// Medication category on a core customer, pet, reservation, or audit record.
        Medication,
        /// Medical category on a core customer, pet, reservation, or audit record.
        Medical,
        /// Behavior category on a core customer, pet, reservation, or audit record.
        Behavior,
        /// Grooming service line or care-note category.
        Grooming,
        /// Training service line or care-note category.
        Training,
        /// General category on a core customer, pet, reservation, or audit record.
        General,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for visibility decisions in entities workflows.
    pub enum Visibility {
        /// Internal only category on a core customer, pet, reservation, or audit record.
        InternalOnly,
        /// Customer visible category on a core customer, pet, reservation, or audit record.
        CustomerVisible,
        /// Customer visible after review category on a core customer, pet, reservation, or audit record.
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
/// Typed incident id domain value that keeps raw primitives out of entities workflows.
pub struct IncidentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed message id domain value that keeps raw primitives out of entities workflows.
pub struct MessageId(pub Uuid);

/// Approval boundary for entities contracts.
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
    /// Typed record domain value that keeps raw primitives out of entities workflows.
    pub struct Record {
        /// Id fact promoted into this entities contract.
        pub id: Id,
        /// Target fact promoted into this entities contract.
        pub target: Target,
        /// Gate fact promoted into this entities contract.
        pub gate: policy::ReviewGate,
        /// Lifecycle fact promoted into this entities contract.
        pub lifecycle: Lifecycle,
        /// Requested by fact promoted into this entities contract.
        pub requested_by: ActorRef,
        /// Requested at fact promoted into this entities contract.
        pub requested_at: DateTime<Utc>,
        #[builder(default)]
        /// Audit refs fact promoted into this entities contract.
        pub audit_refs: Vec<crate::audit::EventId>,
    }

    impl Record {
        /// Returns the status for this entities value.
        pub fn status(&self) -> Status {
            self.lifecycle.status()
        }

        /// Returns the is applicable for this entities value.
        pub fn is_applicable(&self) -> bool {
            matches!(self.lifecycle, Lifecycle::Approved { .. })
        }

        /// Returns the is terminal decision for this entities value.
        pub fn is_terminal_decision(&self) -> bool {
            self.lifecycle.is_terminal_decision()
        }

        /// Returns the decision actor and time for this entities value.
        pub fn decision_actor_and_time(&self) -> Option<(&ActorRef, DateTime<Utc>)> {
            self.lifecycle.decision_actor_and_time()
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for target decisions in entities workflows.
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
    /// Domain vocabulary for lifecycle decisions in entities workflows.
    pub enum Lifecycle {
        /// Approval requested category on a core customer, pet, reservation, or audit record.
        ApprovalRequested,
        /// Approved category on a core customer, pet, reservation, or audit record.
        Approved {
            /// Decided by fact promoted into this entities contract.
            decided_by: ActorRef,
            /// Decided at fact promoted into this entities contract.
            decided_at: DateTime<Utc>,
        },
        /// Rejected category on a core customer, pet, reservation, or audit record.
        Rejected {
            /// Decided by fact promoted into this entities contract.
            decided_by: ActorRef,
            /// Decided at fact promoted into this entities contract.
            decided_at: DateTime<Utc>,
        },
        /// Reservation is no longer active.
        Cancelled,
        /// Superseded category on a core customer, pet, reservation, or audit record.
        Superseded,
    }

    impl Lifecycle {
        /// Returns the status for this entities value.
        pub fn status(&self) -> Status {
            match self {
                Self::ApprovalRequested => Status::ApprovalRequested,
                Self::Approved { .. } => Status::Approved,
                Self::Rejected { .. } => Status::Rejected,
                Self::Cancelled => Status::Cancelled,
                Self::Superseded => Status::Superseded,
            }
        }

        /// Returns the is terminal decision for this entities value.
        pub fn is_terminal_decision(&self) -> bool {
            matches!(self, Self::Approved { .. } | Self::Rejected { .. })
        }

        /// Returns the decision actor and time for this entities value.
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
    /// Normalized reservation states observed during source-data ingestion.
    pub enum Status {
        /// Approval requested category on a core customer, pet, reservation, or audit record.
        ApprovalRequested,
        /// Approved category on a core customer, pet, reservation, or audit record.
        Approved,
        /// Rejected category on a core customer, pet, reservation, or audit record.
        Rejected,
        /// Reservation is no longer active.
        Cancelled,
        /// Superseded category on a core customer, pet, reservation, or audit record.
        Superseded,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed document domain value that keeps raw primitives out of entities workflows.
pub struct Document {
    /// Id fact promoted into this entities contract.
    pub id: DocumentId,
    /// Location id fact promoted into this entities contract.
    pub location_id: LocationId,
    /// Subject fact promoted into this entities contract.
    pub subject: DocumentSubject,
    /// Classification fact promoted into this entities contract.
    pub classification: document::Classification,
    /// Source fact promoted into this entities contract.
    pub source: document::Source,
    /// Uploaded by actor fact promoted into this entities contract.
    pub uploaded_by_actor: ActorRef,
    /// Uploaded at fact promoted into this entities contract.
    pub uploaded_at: DateTime<Utc>,
    /// Original file fact promoted into this entities contract.
    pub original_file: document::OriginalFile,
    /// Storage ref fact promoted into this entities contract.
    pub storage_ref: document::StorageRef,
    /// Virus scan status fact promoted into this entities contract.
    pub virus_scan_status: document::VirusScanStatus,
    /// Pii redaction status fact promoted into this entities contract.
    pub pii_redaction_status: document::PiiRedactionStatus,
    /// Verification status fact promoted into this entities contract.
    pub verification_status: document::Status,
    #[builder(default)]
    /// Audit refs fact promoted into this entities contract.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl Document {
    /// Returns the requires human review before use for this entities value.
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
/// Domain vocabulary for document subject decisions in entities workflows.
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
/// Typed vaccine record domain value that keeps raw primitives out of entities workflows.
pub struct VaccineRecord {
    /// Id fact promoted into this entities contract.
    pub id: VaccineRecordId,
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Vaccine name fact promoted into this entities contract.
    pub vaccine_name: policy::VaccineName,
    /// Source document id fact promoted into this entities contract.
    pub source_document_id: DocumentId,
    /// Status fact promoted into this entities contract.
    pub status: vaccine::Status,
    /// Effective on fact promoted into this entities contract.
    pub effective_on: NaiveDate,
    /// Expires on fact promoted into this entities contract.
    pub expires_on: Option<NaiveDate>,
    /// Review gate fact promoted into this entities contract.
    pub review_gate: policy::ReviewGate,
    #[builder(default)]
    /// Audit refs fact promoted into this entities contract.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl VaccineRecord {
    /// Returns the requires human review before compliance for this entities value.
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
/// Typed care note domain value that keeps raw primitives out of entities workflows.
pub struct CareNote {
    /// Id fact promoted into this entities contract.
    pub id: care_note::Id,
    /// Subject fact promoted into this entities contract.
    pub subject: care_note::Subject,
    /// Kind fact promoted into this entities contract.
    pub kind: care_note::Kind,
    /// Visibility fact promoted into this entities contract.
    pub visibility: care_note::Visibility,
    /// Body fact promoted into this entities contract.
    pub body: care_note::Body,
    /// Author fact promoted into this entities contract.
    pub author: ActorRef,
    /// Recorded at fact promoted into this entities contract.
    pub recorded_at: DateTime<Utc>,
    #[builder(default)]
    /// Audit refs fact promoted into this entities contract.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl CareNote {
    /// Returns the is customer visible without review for this entities value.
    pub fn is_customer_visible_without_review(&self) -> bool {
        matches!(self.visibility, care_note::Visibility::CustomerVisible)
            && !matches!(
                self.kind,
                care_note::Kind::Medication | care_note::Kind::Medical | care_note::Kind::Behavior
            )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed incident domain value that keeps raw primitives out of entities workflows.
pub struct Incident {
    /// Id fact promoted into this entities contract.
    pub id: IncidentId,
    /// Location id fact promoted into this entities contract.
    pub location_id: LocationId,
    /// Primary subject fact promoted into this entities contract.
    pub primary_subject: IncidentSubject,
    /// Category fact promoted into this entities contract.
    pub category: incident::Category,
    /// Severity fact promoted into this entities contract.
    pub severity: incident::Severity,
    /// Status fact promoted into this entities contract.
    pub status: incident::Status,
    /// Reported by fact promoted into this entities contract.
    pub reported_by: ActorRef,
    /// Reported at fact promoted into this entities contract.
    pub reported_at: DateTime<Utc>,
    /// Summary fact promoted into this entities contract.
    pub summary: incident::Summary,
    #[builder(default)]
    /// Required review gates fact promoted into this entities contract.
    pub required_review_gates: Vec<policy::ReviewGate>,
    #[builder(default)]
    /// Audit refs fact promoted into this entities contract.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl Incident {
    /// Returns the requires manager attention for this entities value.
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
/// Domain vocabulary for incident subject decisions in entities workflows.
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
/// Typed message domain value that keeps raw primitives out of entities workflows.
pub struct Message {
    /// Id fact promoted into this entities contract.
    pub id: MessageId,
    /// Subject fact promoted into this entities contract.
    pub subject: MessageSubject,
    /// Direction fact promoted into this entities contract.
    pub direction: message::Direction,
    /// Channel fact promoted into this entities contract.
    pub channel: message::Channel,
    /// Status fact promoted into this entities contract.
    pub status: message::Status,
    /// Body ref fact promoted into this entities contract.
    pub body_ref: message::BodyRef,
    /// Approval gate fact promoted into this entities contract.
    pub approval_gate: Option<policy::ReviewGate>,
    #[builder(default)]
    /// Audit refs fact promoted into this entities contract.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl Message {
    /// Returns the requires approval before send for this entities value.
    pub fn requires_approval_before_send(&self) -> bool {
        self.approval_gate.is_some()
            || matches!(self.status, message::Status::ApprovalRequested)
            || matches!(self.direction, message::Direction::OutboundDraft)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for message subject decisions in entities workflows.
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

/// Audit boundary for entities contracts.
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
    /// Typed event domain value that keeps raw primitives out of entities workflows.
    pub struct Event {
        /// At fact promoted into this entities contract.
        pub at: DateTime<Utc>,
        /// Actor fact promoted into this entities contract.
        pub actor: super::ActorRef,
        /// Subject fact promoted into this entities contract.
        pub subject: Subject,
        /// Action fact promoted into this entities contract.
        pub action: Action,
        /// Metadata fact promoted into this entities contract.
        pub metadata: BTreeMap<MetadataKey, MetadataValue>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for subject decisions in entities workflows.
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
        /// Care note category on a core customer, pet, reservation, or audit record.
        CareNote(care_note::Id),
        /// Incident record participating in the workflow.
        Incident(IncidentId),
        /// Customer communication record participating in approval.
        Message(MessageId),
        /// Approval decision record participating in audit history.
        Approval(approval::Id),
        /// Workflow event category on a core customer, pet, reservation, or audit record.
        WorkflowEvent(crate::workflow::EventId),
        /// External system object referenced from domain history.
        External {
            /// Provider fact promoted into this entities contract.
            provider: crate::workflow::external::Provider,
            /// Id fact promoted into this entities contract.
            id: crate::workflow::external::Id,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for action decisions in entities workflows.
    pub enum Action {
        /// Customer profile updated category on a core customer, pet, reservation, or audit record.
        CustomerProfileUpdated,
        /// Pet profile updated category on a core customer, pet, reservation, or audit record.
        PetProfileUpdated,
        /// Reservation status suggested category on a core customer, pet, reservation, or audit record.
        ReservationStatusSuggested,
        /// Reservation status changed category on a core customer, pet, reservation, or audit record.
        ReservationStatusChanged,
        /// Policy decision recorded category on a core customer, pet, reservation, or audit record.
        PolicyDecisionRecorded,
        /// Document received category on a core customer, pet, reservation, or audit record.
        DocumentReceived,
        /// Vaccine record review requested category on a core customer, pet, reservation, or audit record.
        VaccineRecordReviewRequested,
        /// Incident status changed category on a core customer, pet, reservation, or audit record.
        IncidentStatusChanged,
        /// Message approval requested category on a core customer, pet, reservation, or audit record.
        MessageApprovalRequested,
        /// Approval decision recorded category on a core customer, pet, reservation, or audit record.
        ApprovalDecisionRecorded,
        /// Workflow event recorded category on a core customer, pet, reservation, or audit record.
        WorkflowEventRecorded,
        /// Extension point for provider-specific values not modeled directly.
        Extension(ActionLabel),
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
    pub struct ActionLabel(String);

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
/// Domain vocabulary for actor ref decisions in entities workflows.
pub enum ActorRef {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Staff id fact promoted into this entities contract.
    Staff {
        /// Staff id carried by this variant.
        staff_id: StaffId,
    },
    /// Manager id fact promoted into this entities contract.
    Manager {
        /// Manager id carried by this variant.
        manager_id: ManagerId,
    },
    /// System category on a core customer, pet, reservation, or audit record.
    System,
    /// Workflow fact promoted into this entities contract.
    Agent {
        /// Workflow carried by this variant.
        workflow: agent::Name,
    },
}
