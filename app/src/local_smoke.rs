use chrono::{DateTime, NaiveDate, Utc};
use domain::{document, entities, pet, policy, source, vaccine, workflow};
use serde::Deserialize;
use uuid::Uuid;

use crate::{booking_triage, checkout_completion, daily_update};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("local smoke fixture is not valid JSON: {0}")]
    InvalidFixture(#[from] serde_json::Error),
    #[error("local smoke fixture is missing a required semantic value: {0}")]
    InvalidDomainValue(String),
    #[error("local smoke daily-update preview failed: {0}")]
    DailyUpdate(#[from] daily_update::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Deserialize)]
struct InquiryFixture {
    source_event_key: String,
    customer: CustomerFixture,
    pet: PetFixture,
    requested_service: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct CustomerFixture {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct PetFixture {
    name: String,
    species: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEventKey(String);

impl SourceEventKey {
    fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(Error::InvalidDomainValue(
                "source_event_key must be present for smoke provenance".to_owned(),
            ));
        }
        Ok(Self(value))
    }
}

impl AsRef<str> for SourceEventKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Inquiry,
    Profile,
    VaccineDocs,
    BookingTriage,
    ConfirmationDraft,
    CheckInTodayView,
    StaffNoteDailyUpdateDraft,
    CheckoutCompletion,
    FollowUpRetention,
}

impl Stage {
    const fn name(self) -> &'static str {
        match self {
            Self::Inquiry => "inquiry",
            Self::Profile => "profile",
            Self::VaccineDocs => "vaccine_docs",
            Self::BookingTriage => "booking_triage",
            Self::ConfirmationDraft => "confirmation_draft",
            Self::CheckInTodayView => "check_in_today_view",
            Self::StaffNoteDailyUpdateDraft => "staff_note_daily_update_draft",
            Self::CheckoutCompletion => "checkout_completion",
            Self::FollowUpRetention => "follow_up_retention",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmokeBoundaries {
    draft_only_ai: bool,
    blocks_live_customer_sends: bool,
    blocks_provider_or_pms_mutations: bool,
    blocks_payment_refund_or_discount_actions: bool,
}

impl SmokeBoundaries {
    const fn local_demo() -> Self {
        Self {
            draft_only_ai: true,
            blocks_live_customer_sends: true,
            blocks_provider_or_pms_mutations: true,
            blocks_payment_refund_or_discount_actions: true,
        }
    }

    pub const fn draft_only_ai(&self) -> bool {
        self.draft_only_ai
    }

    pub const fn blocks_live_customer_sends(&self) -> bool {
        self.blocks_live_customer_sends
    }

    pub const fn blocks_provider_or_pms_mutations(&self) -> bool {
        self.blocks_provider_or_pms_mutations
    }

    pub const fn blocks_payment_refund_or_discount_actions(&self) -> bool {
        self.blocks_payment_refund_or_discount_actions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewEvidenceRef(String);

impl ReviewEvidenceRef {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for ReviewEvidenceRef {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmokeConfirmationDraft {
    draft: booking_triage::ConfirmationDraft,
}

impl SmokeConfirmationDraft {
    pub const fn requires_customer_message_approval(&self) -> bool {
        matches!(
            self.draft.approval_gate(),
            booking_triage::ApprovalGate::CustomerMessageApproval
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationLabel(String);

impl AsRef<str> for ReservationLabel {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayView {
    reservation_labels: Vec<ReservationLabel>,
    status: entities::reservation::Status,
}

impl TodayView {
    pub fn reservation_labels(&self) -> &[ReservationLabel] {
        &self.reservation_labels
    }

    pub const fn status(&self) -> &entities::reservation::Status {
        &self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutCompletion {
    packet: checkout_completion::Packet,
}

impl CheckoutCompletion {
    pub fn status(&self) -> entities::reservation::Status {
        self.packet
            .suggested_reservation_status()
            .expect("local smoke checkout completion should suggest checked-out status")
    }

    pub const fn completion_status(&self) -> checkout_completion::CompletionStatus {
        self.packet.completion_status()
    }

    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        self.packet.required_review_gates()
    }

    pub fn blocked_actions(&self) -> &[checkout_completion::BlockedAction] {
        self.packet.blocked_actions()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionNextAction {
    DraftRebookingReminderForReview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionFollowUp {
    next_action: RetentionNextAction,
    review_gate: policy::ReviewGate,
}

impl RetentionFollowUp {
    pub const fn next_action(&self) -> RetentionNextAction {
        self.next_action
    }

    pub fn review_gate(&self) -> policy::ReviewGate {
        self.review_gate.clone()
    }
}

#[derive(Debug, Clone)]
pub struct FullChainEvidence {
    source_event_key: SourceEventKey,
    stages: Vec<Stage>,
    boundaries: SmokeBoundaries,
    #[allow(dead_code)]
    inquiry: InquiryRecord,
    #[allow(dead_code)]
    profile: ProfileEvidence,
    #[allow(dead_code)]
    vaccine_docs: VaccineDocumentEvidence,
    booking_packet: booking_triage::StaffEvaluationPacket,
    confirmation_draft: SmokeConfirmationDraft,
    today_view: TodayView,
    daily_update_preview: daily_update::MvpPreview,
    checkout_completion: CheckoutCompletion,
    retention_follow_up: RetentionFollowUp,
    review_gated_evidence_refs: Vec<ReviewEvidenceRef>,
}

impl FullChainEvidence {
    pub const fn source_event_key(&self) -> &SourceEventKey {
        &self.source_event_key
    }

    pub fn stage_names(&self) -> Vec<&'static str> {
        self.stages.iter().map(|stage| stage.name()).collect()
    }

    pub const fn boundaries(&self) -> &SmokeBoundaries {
        &self.boundaries
    }

    pub const fn booking_packet(&self) -> &booking_triage::StaffEvaluationPacket {
        &self.booking_packet
    }

    pub const fn confirmation_draft(&self) -> &SmokeConfirmationDraft {
        &self.confirmation_draft
    }

    pub const fn today_view(&self) -> &TodayView {
        &self.today_view
    }

    pub const fn daily_update_preview(&self) -> &daily_update::MvpPreview {
        &self.daily_update_preview
    }

    pub const fn checkout_completion(&self) -> &CheckoutCompletion {
        &self.checkout_completion
    }

    pub const fn retention_follow_up(&self) -> &RetentionFollowUp {
        &self.retention_follow_up
    }

    pub fn review_gated_evidence_refs(&self) -> &[ReviewEvidenceRef] {
        &self.review_gated_evidence_refs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InquiryRecord {
    source_event_key: SourceEventKey,
    requested_service: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProfileEvidence {
    customer: entities::Customer,
    pet: entities::Pet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VaccineDocumentEvidence {
    document: entities::Document,
    record: entities::VaccineRecord,
}

pub fn run_fixture(fixture_json: &str) -> Result<FullChainEvidence> {
    let fixture: InquiryFixture = serde_json::from_str(fixture_json)?;
    let source_event_key = SourceEventKey::parse(fixture.source_event_key)?;
    let ids = SmokeIds::new();
    let reservation_label = ReservationLabel(format!("REQ-{}", source_event_key.as_ref()));
    let occurred_at = DateTime::<Utc>::UNIX_EPOCH;

    let inquiry = InquiryRecord {
        source_event_key: source_event_key.clone(),
        requested_service: fixture.requested_service,
        message: fixture.message,
    };
    let profile = build_profile(&fixture.customer, &fixture.pet, ids.customer_id, ids.pet_id)?;
    let vaccine_docs = build_vaccine_document(ids, occurred_at)?;
    let booking_packet = build_booking_packet(&reservation_label);
    let confirmation_draft = SmokeConfirmationDraft {
        draft: booking_packet.confirmation_draft().clone(),
    };
    let today_view = TodayView {
        reservation_labels: vec![reservation_label],
        status: entities::reservation::Status::CheckedIn,
    };
    let daily_update_preview = build_daily_update_preview(ids, &profile.pet.name, occurred_at)?;
    let checkout_completion = build_checkout_completion(ids, occurred_at)?;
    let retention_follow_up = RetentionFollowUp {
        next_action: RetentionNextAction::DraftRebookingReminderForReview,
        review_gate: policy::ReviewGate::CustomerMessageApproval,
    };

    let review_gated_evidence_refs = vec![
        ReviewEvidenceRef::new("vaccine_docs:medical_document_review_required"),
        ReviewEvidenceRef::new("confirmation:customer_message_approval_required"),
        ReviewEvidenceRef::new("daily_update:send_stub_blocked_until_human_approval"),
        ReviewEvidenceRef::new("checkout_completion:customer_message_approval_required"),
    ];

    assert!(vaccine_docs.document.requires_human_review_before_use());
    assert!(
        vaccine_docs
            .record
            .requires_human_review_before_compliance()
    );

    Ok(FullChainEvidence {
        source_event_key,
        stages: vec![
            Stage::Inquiry,
            Stage::Profile,
            Stage::VaccineDocs,
            Stage::BookingTriage,
            Stage::ConfirmationDraft,
            Stage::CheckInTodayView,
            Stage::StaffNoteDailyUpdateDraft,
            Stage::CheckoutCompletion,
            Stage::FollowUpRetention,
        ],
        boundaries: SmokeBoundaries::local_demo(),
        inquiry,
        profile,
        vaccine_docs,
        booking_packet,
        confirmation_draft,
        today_view,
        daily_update_preview,
        checkout_completion,
        retention_follow_up,
        review_gated_evidence_refs,
    })
}

#[derive(Debug, Clone, Copy)]
struct SmokeIds {
    location_id: entities::LocationId,
    customer_id: entities::CustomerId,
    pet_id: entities::PetId,
    reservation_id: entities::reservation::Id,
    document_id: entities::DocumentId,
    vaccine_record_id: entities::VaccineRecordId,
}

impl SmokeIds {
    fn new() -> Self {
        Self {
            location_id: entities::LocationId(Uuid::from_u128(
                0x0051_0CA1_0000_0000_0000_0000_0000_0001,
            )),
            customer_id: entities::CustomerId(Uuid::from_u128(
                0x0051_0CA1_0000_0000_0000_0000_0000_0002,
            )),
            pet_id: entities::PetId(Uuid::from_u128(0x0051_0CA1_0000_0000_0000_0000_0000_0003)),
            reservation_id: entities::reservation::Id(Uuid::from_u128(
                0x0051_0CA1_0000_0000_0000_0000_0000_0004,
            )),
            document_id: entities::DocumentId(Uuid::from_u128(
                0x0051_0CA1_0000_0000_0000_0000_0000_0005,
            )),
            vaccine_record_id: entities::VaccineRecordId(Uuid::from_u128(
                0x0051_0CA1_0000_0000_0000_0000_0000_0006,
            )),
        }
    }
}

fn build_profile(
    customer: &CustomerFixture,
    pet: &PetFixture,
    customer_id: entities::CustomerId,
    pet_id: entities::PetId,
) -> Result<ProfileEvidence> {
    let species = match pet.species.trim().to_ascii_lowercase().as_str() {
        "dog" => entities::Species::Dog,
        "cat" => entities::Species::Cat,
        other => entities::Species::Other(other.to_owned()),
    };

    Ok(ProfileEvidence {
        customer: entities::Customer {
            id: customer_id,
            full_name: domain::customer::Name::try_new(customer.name.clone()).map_err(invalid)?,
            email: Some(domain::customer::Email::try_new(customer.email.clone()).map_err(invalid)?),
            mobile_phone: None,
            preferred_contact: entities::ContactChannel::Email,
            portal_account: None,
        },
        pet: entities::Pet {
            id: pet_id,
            customer_id,
            name: pet::Name::try_new(pet.name.clone()).map_err(invalid)?,
            species,
            birth_date: None,
            sex: None,
            spay_neuter_status: entities::SpayNeuterStatus::Unknown,
            temperament: entities::TemperamentProfile::default(),
            care_profile: entities::CareProfile::default(),
        },
    })
}

fn build_vaccine_document(
    ids: SmokeIds,
    occurred_at: DateTime<Utc>,
) -> Result<VaccineDocumentEvidence> {
    let uploaded_by_actor = entities::ActorRef::Customer(ids.customer_id);
    Ok(VaccineDocumentEvidence {
        document: entities::Document::builder()
            .id(ids.document_id)
            .location_id(ids.location_id)
            .subject(entities::DocumentSubject::Pet(ids.pet_id))
            .classification(document::Classification::VaccineProof)
            .source(document::Source::CustomerUpload)
            .uploaded_by_actor(uploaded_by_actor)
            .uploaded_at(occurred_at)
            .original_file(
                document::OriginalFile::builder()
                    .filename(document::FileName::try_new("miso-rabies.pdf").map_err(invalid)?)
                    .mime_type(document::MimeType::try_new("application/pdf").map_err(invalid)?)
                    .content_length(document::ContentLengthBytes::try_new(42).map_err(invalid)?)
                    .sha256(
                        document::Sha256Digest::try_new(
                            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        )
                        .map_err(invalid)?,
                    )
                    .build(),
            )
            .storage_ref(
                document::StorageRef::builder()
                    .bucket(
                        document::StorageBucket::try_new("local-smoke-documents")
                            .map_err(invalid)?,
                    )
                    .key(
                        document::StorageKey::try_new("fixtures/miso-rabies.pdf")
                            .map_err(invalid)?,
                    )
                    .version(document::StorageVersion::try_new("v1").map_err(invalid)?)
                    .build(),
            )
            .virus_scan_status(document::VirusScanStatus::Passed)
            .pii_redaction_status(document::PiiRedactionStatus::NotRequired)
            .verification_status(document::Status::AwaitingReview)
            .build(),
        record: entities::VaccineRecord::builder()
            .id(ids.vaccine_record_id)
            .pet_id(ids.pet_id)
            .vaccine_name(policy::VaccineName::try_new("Rabies").map_err(invalid)?)
            .source_document_id(ids.document_id)
            .status(vaccine::Status::PendingReview)
            .effective_on(NaiveDate::from_ymd_opt(2026, 1, 1).expect("static date is valid"))
            .expires_on(NaiveDate::from_ymd_opt(2027, 1, 1).expect("static date is valid"))
            .review_gate(policy::ReviewGate::MedicalDocumentReview)
            .build(),
    })
}

fn build_booking_packet(
    reservation_label: &ReservationLabel,
) -> booking_triage::StaffEvaluationPacket {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::rule::Evaluation::pass(
            booking_triage::rule::Id::DateRangeAndServiceSupported,
            vec![booking_triage::EvidenceRef::try_new("fixture:boarding-requested").unwrap()],
        ),
        booking_triage::rule::Evaluation::pass(
            booking_triage::rule::Id::AccommodationAvailability,
            vec![booking_triage::EvidenceRef::try_new("local-demo:capacity-open").unwrap()],
        ),
        booking_triage::rule::Evaluation::pass(
            booking_triage::rule::Id::VaccineRequirements,
            vec![
                booking_triage::EvidenceRef::try_new("vaccine:rabies-pending-human-reviewed-path")
                    .unwrap(),
            ],
        ),
        booking_triage::rule::Evaluation::pass(
            booking_triage::rule::Id::DepositAndPricingRequirements,
            vec![booking_triage::EvidenceRef::try_new("payment:not-required-local-smoke").unwrap()],
        ),
    ]);

    booking_triage::StaffEvaluationPacket::new(
        booking_triage::Reservation::try_new(reservation_label.as_ref()).unwrap(),
        evaluation,
    )
    .with_ai_recommendation(booking_triage::AiRecommendation::recommend_staff_confirmation(
        booking_triage::RecommendationText::try_new(
            "Draft-only local smoke: deterministic gates permit staff to review an offer without mutating PMS records.",
        )
        .unwrap(),
    ))
    .with_confirmation_draft(booking_triage::ConfirmationDraft::new(
        booking_triage::CustomerMessageDraft::try_new(
            "Draft only: staff can review this local demo booking confirmation before any customer send.",
        )
        .unwrap(),
    ))
}

fn build_daily_update_preview(
    ids: SmokeIds,
    pet_name: &pet::Name,
    occurred_at: DateTime<Utc>,
) -> Result<daily_update::MvpPreview> {
    let event = workflow::Event {
        event_id: workflow::EventId(Uuid::from_u128(0x0051_0CA1_0000_0000_0000_0000_0000_0010)),
        event_type: workflow::EventType::DailyNoteCreated,
        occurred_at,
        actor: entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("local-smoke-kennel").map_err(invalid)?,
        },
        location_id: ids.location_id,
        subject: workflow::Subject::Reservation(ids.reservation_id),
        policy_context: workflow::PolicyContext {
            allowed_actions: vec![
                workflow::AllowedAction::SummarizeCareNotes,
                workflow::AllowedAction::DraftCustomerMessage,
            ],
            automation_level: policy::automation::Level::DraftOnly,
            required_reviews: vec![policy::ReviewGate::CustomerMessageApproval],
        },
    };
    let note = entities::CareNote::builder()
        .id(entities::care_note::Id(Uuid::from_u128(
            0x0051_0CA1_0000_0000_0000_0000_0000_0011,
        )))
        .subject(entities::care_note::Subject::Reservation(
            ids.reservation_id,
        ))
        .kind(entities::care_note::Kind::General)
        .visibility(entities::care_note::Visibility::CustomerVisibleAfterReview)
        .body(
            entities::care_note::Body::try_new(
                "settled into suite, ate dinner, and enjoyed supervised play.",
            )
            .map_err(invalid)?,
        )
        .author(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("local-smoke-kennel").map_err(invalid)?,
        })
        .recorded_at(occurred_at)
        .build();

    Ok(daily_update::build_mvp_preview(
        daily_update::MvpPreviewRequest::builder()
            .event(event)
            .pet_name(pet_name.clone())
            .owner_display_name(domain::customer::Name::try_new("Casey Local").map_err(invalid)?)
            .policy_snapshot_id(
                policy::Id::try_new("local-smoke-draft-only-policy").map_err(invalid)?,
            )
            .notes(vec![note])
            .build(),
    )?)
}

fn build_checkout_completion(
    ids: SmokeIds,
    occurred_at: DateTime<Utc>,
) -> Result<CheckoutCompletion> {
    let source_provenance = source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new("GET /reservations/{id}").map_err(invalid)?)
        .record_id(source::record::Id::try_new("local-smoke-reservation-001").map_err(invalid)?)
        .extraction_batch(
            source::ExtractionBatchId::try_new("local-smoke-checkout").map_err(invalid)?,
        )
        .pulled_at(source::Timestamp::try_new("2026-06-17T00:00:00Z").map_err(invalid)?)
        .request_scope(
            source::RequestScope::try_new("local-smoke-readonly-checkout").map_err(invalid)?,
        )
        .schema_version(source::SchemaVersion::try_new("gingr-v0-readonly").map_err(invalid)?)
        .payload_hash(source::PayloadHash::try_new("sha256:local-smoke-checkout").map_err(invalid)?)
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/reservation-check-out.json")
                .map_err(invalid)?,
        )
        .build();
    let staff_handoff = checkout_completion::StaffHandoff::builder()
        .completed_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("local-smoke-front-desk").map_err(invalid)?,
        })
        .completed_at(occurred_at)
        .belongings_status(checkout_completion::BelongingsStatus::ReturnedToCustomer)
        .care_summary(
            checkout_completion::CareSummary::try_new(
                "Local smoke checkout handoff: belongings returned and care summary ready for review-gated follow-up.",
            )
            .map_err(invalid)?,
        )
        .departure_notes_review(checkout_completion::DepartureNotesReview::StaffReviewed)
        .build();

    Ok(CheckoutCompletion {
        packet: checkout_completion::Workflow::evaluate(
            checkout_completion::Request::builder()
                .reservation_id(ids.reservation_id)
                .source_provenance(source_provenance)
                .observed_source_status(source::reservation::Status::CheckedOut)
                .staff_handoff(staff_handoff)
                .build(),
        ),
    })
}

fn invalid(error: impl ToString) -> Error {
    Error::InvalidDomainValue(error.to_string())
}
