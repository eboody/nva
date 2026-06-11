# Grooming implication 07: Groomer notes and service history

Purpose: define the grooming-owned domain contract for recording completed-service history, preserving groomer notes, and using that history safely for scheduling, duration estimates, rebooking, reminders, and staff review. This is a modeling artifact for later Rust/domain cards; it does not implement provider IO or change any member-facing behavior.

Source context:

- `docs/domain/petsuites/grooming/service-domain-map.md`
- `domain/src/operations.rs`, especially `operations::grooming::{Contract, HistoryRequirement, BreedCoatTimeEstimate, AppointmentMinutes, RebookingCadence}`
- `domain/tests/petsuites_core_service_contracts.rs`
- `domain/tests/domain_quality_patterns.rs` for validated note/newtype and redacted-debug patterns

Assumptions:

- PetSuites grooming notes can contain style preferences, groomer observations, product use, photo/media references, customer preference details, and operational handling notes.
- Care/medical/temperament facts may influence grooming, but the grooming context should reference those facts through typed care/temperament identifiers or review flags instead of duplicating sensitive records into generic notes.
- Provider systems may store historical grooming notes as free text. Storage/adapters may retain raw payloads, but domain behavior should see parsed semantic fields or an explicit `ReviewRequired` parse outcome.
- AI may summarize and structure notes, but generated history is draft-only until a staff member or groomer approves it.

## 1. Operational story

### Trigger

The story starts when one of these events occurs:

1. A grooming appointment is completed and the groomer needs to record the outcome.
2. A staff member prepares a new grooming appointment and needs the prior style/service history.
3. A rebooking or reminder workflow evaluates whether the pet is due for grooming.
4. An AI workflow summarizes legacy/provider notes into structured history for staff review.
5. A customer contacts the resort with a preference, complaint, style correction, or request that should affect future grooming.

### Actors

- `StaffRole::Groomer`: primary author/approver for service outcome, style instructions, coat observations, product usage, and next-service recommendation.
- `StaffRole::FrontDesk`: may view approved history, draft customer follow-up, and create review tasks; should not silently rewrite groomer notes.
- `StaffRole::Manager`: approves sensitive corrections, complaint/incident-linked edits, customer-facing remediation, and overrides to note retention or visibility.
- Customer/member: source of preferences and feedback, but not the direct owner of internal notes.
- Pet: the subject of history; identity comes from `entities::PetId`.
- AI agent: may draft structured entries, summaries, rebooking recommendations, and prep packets; it never commits durable staff history, hides concerning facts, or sends member-facing messages by itself.

### Inputs

- Completed appointment identity: future `operations::grooming::AppointmentId`, plus optional external/provider ID at the adapter boundary.
- Typed identities: `entities::LocationId`, `entities::PetId`, `entities::CustomerId`, optional `entities::ReservationId`, optional `entities::StaffId` for groomer.
- Service details: grooming offering, completed date/time, duration, price/deposit references if needed, and whether it was standalone or attached to boarding/daycare checkout.
- Outcome facts: completed/cancelled/no-show, finish quality, service limitations, customer satisfaction, complaint/escalation state.
- Notes: style/cut instructions, customer preference, groomer observations, product instructions, coat/skin observations, photo/media refs, next cadence recommendation.
- Neighboring context: care/medical/temperament references, customer communication consent, reservation checkout context, and existing service history.

### Decisions

- Is this history entry complete enough to become durable, or is a required field missing?
- Are note categories safely separated: style/customer preference vs groomer observation vs care/medical/temperament reference?
- Is any content sensitive enough to require redaction, manager review, or a care-module reference instead of local copying?
- Did the groomer recommend a next cadence outside the ordinary 2-8 week window? If so, does it carry a reason and review boundary?
- Can the history support a duration estimate or rebooking recommendation, or should the recommendation be low-confidence/staff-review-required?
- Is a generated summary faithful to source notes, or must it remain draft/review-required?
- Is the correction append-only with audit trail, or is a manager-approved amendment required?

### Outputs

- Durable `operations::grooming::ServiceHistoryEntry` after staff/groomer approval.
- Optional `operations::grooming::HistoryDraft` when source is AI/provider parsing/front-desk intake and approval is still pending.
- Optional `operations::grooming::GroomingPrepPacket` for a future appointment, containing approved style summary, product/coat considerations, photo refs, and care/handling references.
- Optional `operations::grooming::RebookingRecommendation` with due window, rationale, source history entry, and review requirement.
- Optional workflow/staff tasks: groomer history review, manager review, customer follow-up, care/medical review, provider-note parse review.
- Audit event recording who drafted, approved, amended, or suppressed a history detail and why.

### Success state

A completed grooming service has a typed, approved history entry that:

- belongs to exactly one pet and location;
- identifies the completed service and date;
- preserves groomer-authored meaning without collapsing categories into one free-text blob;
- references care/medical/temperament facts instead of copying sensitive records;
- is usable for future duration estimates, groomer prep, rebooking, and reminder drafts;
- records approval and audit metadata; and
- does not create member-facing messages, charges, booking changes, or hidden note rewrites without explicit approval.

### Failure and exception states

- Missing completed appointment, pet, location, offering, or service date: reject entry construction with `history::Error::MissingRequiredField` or keep a `HistoryDraft::NeedsStaffCompletion`.
- Blank/oversized note text: reject through note newtypes such as `StyleNote::try_new` or `GroomerObservation::try_new`.
- Medical/temperament content entered as generic style text: preserve as `HistoryDraft::NeedsCareReferenceReview` and create a care/document review task; do not copy it into approved style notes.
- AI summary conflicts with source note/photo evidence: draft is `NeedsGroomerReview` with evidence spans; no durable overwrite.
- Customer disputes or complaint-linked correction: append an amendment requiring manager review; do not delete original staff history unless retention policy explicitly permits it.
- Provider raw note cannot be parsed into safe semantic fields: store raw payload in boundary storage, surface `ProviderNoteParseReviewRequired`, and require human classification.
- Rebooking cadence is outside ordinary policy: allow only `GroomerRecommended` or `ManagerOverride` with rationale; do not sneak 1/12-week values through a naked integer.

## 2. Domain types to add or refine

Recommended semantic module shape:

```rust
operations::grooming::history
operations::grooming::history::entry
operations::grooming::history::note
operations::grooming::history::approval
operations::grooming::history::photo
operations::grooming::history::repository
```

The caller-facing path can re-export central concepts as `operations::grooming::ServiceHistoryEntry`, `operations::grooming::HistoryRepository`, and `operations::grooming::HistoryPolicy`, while keeping note-specific vocabulary under `operations::grooming::history`.

### Entities / aggregate roots

- `operations::grooming::ServiceHistoryEntry`
  - Approved record of a completed grooming service.
  - Invariants: requires `HistoryEntryId`, `LocationId`, `PetId`, completed service date/time, completed offering, outcome, author/approver metadata, and at least one meaningful history component: outcome, note, product use, photo ref, cadence recommendation, or limitation.
  - Must not expose raw provider note text as the only domain field.

- `operations::grooming::HistoryDraft`
  - Pending entry from provider import, AI summarization, front-desk intake, or groomer-in-progress editing.
  - Invariants: records source, source evidence, current review state, and why approval is needed.
  - Can become `ServiceHistoryEntry` only through `HistoryPolicy::approve`.

- `operations::grooming::HistoryAmendment`
  - Append-only correction or clarification to an approved entry.
  - Invariants: carries original entry ID, amendment reason, author, approval boundary, timestamp, and optional member-facing impact flag.

- `operations::grooming::GroomingPrepPacket`
  - Read model for future appointment preparation.
  - Invariants: contains only approved history plus explicit review-required references; no raw unapproved AI summary.

### Value objects / newtypes

- `operations::grooming::history::EntryId`
  - Non-empty provider/domain ID or UUID wrapper. Provider IDs are promoted at boundary.

- `operations::grooming::history::StyleNote`
  - Trimmed, non-empty, bounded text for haircut/style/customer preference.
  - Redacted `Debug` if content can identify customer/pet specifics.
  - Examples: desired cut length, face/feet/tail style, owner preference, avoid shaving unless approved.

- `operations::grooming::history::GroomerObservation`
  - Trimmed, non-empty, bounded internal observation about coat condition, service difficulty, tolerance, or service outcome.
  - Should distinguish operational observations from diagnoses.

- `operations::grooming::history::ProductUsageNote`
  - Product or shampoo/conditioner usage details.
  - Invariant: medical/medicated products set `ReviewRequirement::CareOrMedicalReview` unless already backed by a care-policy reference.

- `operations::grooming::history::CustomerPreferenceNote`
  - Customer-stated preference relevant to future service.
  - Invariant: member-facing promises or price/deposit expectations are not encoded here; those belong to customer/payment workflow.

- `operations::grooming::history::MediaRef`
  - Reference to photo/media storage, not inline binary data.
  - Invariants: non-empty external key/URI, media type, capture context, visibility classification, and retention/audit metadata.

- `operations::grooming::history::SourceEvidenceRef`
  - Links a draft/summary back to provider note ID, staff task, message, photo, or appointment record.

- `operations::grooming::history::RetentionClass`
  - `Operational`, `CustomerPreference`, `SensitiveCareReference`, `ComplaintOrIncidentLinked`, `ProviderRawPayload`.
  - Drives review, visibility, and audit requirements.

- `operations::grooming::history::RevisionReason`
  - `GroomerCorrection`, `CustomerPreferenceUpdate`, `ManagerClarification`, `ProviderImportCorrection`, `ComplaintResolution`, `SafetyOrCareEscalation`.

### Enums / semantic state

- `operations::grooming::history::EntrySource`
  - `GroomerAuthored`, `FrontDeskIntake`, `ProviderImport`, `AiStructuredDraft`, `CustomerFeedback`, `ManagerAmendment`.

- `operations::grooming::history::ServiceOutcome`
  - `CompletedAsPlanned`, `CompletedWithLimitations`, `PartiallyCompleted`, `CustomerDeclined`, `Cancelled`, `NoShow`, `ManagerReviewRequired`.

- `operations::grooming::history::NoteKind`
  - `StyleInstruction`, `GroomerObservation`, `CustomerPreference`, `ProductUse`, `CareReference`, `ComplaintOrCorrection`.
  - Use this only when a polymorphic list is actually needed; otherwise prefer distinct typed fields.

- `operations::grooming::history::ApprovalState`
  - `Draft`, `NeedsGroomerReview`, `NeedsCareReview`, `NeedsManagerReview`, `Approved`, `Rejected`, `SupersededByAmendment`.

- `operations::grooming::history::Visibility`
  - `InternalOnly`, `ShareableWithCustomerAfterApproval`, `ManagerOnly`, `CareTeamOnly`, `ProviderRawOnly`.

- `operations::grooming::history::RecommendationSource`
  - `GroomerRecommended`, `PolicyCadence`, `HistoryPattern`, `AiSuggestedPendingReview`, `CustomerRequested`.

### Policies / services

- `operations::grooming::HistoryPolicy`
  - Truthful owner of construction, approval, amendment, redaction classification, and conversion from draft to durable history.

- `operations::grooming::HistorySummarizationPolicy`
  - Truthful owner of AI/provider-note summary safety: evidence required, category separation, review state, and redaction.

- `operations::grooming::GroomingPrepPolicy`
  - Truthful owner of selecting approved history for the next appointment prep packet.

- `operations::grooming::RebookingPolicy`
  - Uses service history and cadence to produce due/overdue recommendations.

- `operations::grooming::EstimationPolicy`
  - Uses approved history and current pet/coat context to refine duration estimates.

### Repositories / stores

- `operations::grooming::HistoryRepository`
  - Domain-facing port for approved entries, drafts, amendments, and prep queries.

- `storage::operations::grooming::history::EntryRecord`
  - Boundary record preserving provider/raw storage shape while converting to domain entry through explicit promotion.

- `storage::operations::grooming::history::ProviderNoteRecord`
  - Raw provider note payload retained only at boundary. Domain must receive parsed fields or a review-required draft.

## 3. Relationship map between types

### Entities and value objects

- `ServiceHistoryEntry`
  - owns: `history::EntryId`, `entities::PetId`, `entities::LocationId`, `ServiceOutcome`, approved `StyleNote`s, `GroomerObservation`s, `ProductUsageNote`s, `MediaRef`s, optional `RebookingRecommendation`, approval/audit metadata.
  - references: `operations::grooming::AppointmentId`, `entities::CustomerId`, `entities::ReservationId`, `entities::StaffId`, care/temperament reference IDs.

- `HistoryDraft`
  - owns: source, parsed notes, raw-source evidence refs, `ApprovalState`, `ReviewRequirement`, parse warnings.
  - becomes: `ServiceHistoryEntry` only through `HistoryPolicy::approve`.

- `HistoryAmendment`
  - owns: amendment text/category, `RevisionReason`, author/approver, approval state.
  - appends to: `ServiceHistoryEntry`.

- `GroomingPrepPacket`
  - reads from: approved entries and unresolved review flags.
  - feeds: appointment prep, estimate, schedule review, and staff tasks.

### Policies

- `HistoryPolicy`
  - validates construction and approval of entries/drafts/amendments.
  - owns redaction/visibility classification and note-category separation.

- `HistorySummarizationPolicy`
  - converts raw/provider/AI summaries into `HistoryDraft`, not durable entries.
  - requires evidence spans and review state.

- `GroomingPrepPolicy`
  - selects current approved style/product/history context for the next appointment.
  - never includes rejected or unapproved AI content as fact.

- `RebookingPolicy`
  - reads `ServiceHistoryEntry` and `RebookingCadence` to produce recommendations.

- `EstimationPolicy`
  - reads approved history plus breed/coat/service context to produce `DurationEstimate`.

### Repositories / stores

- `HistoryRepository`
  - `append_approved`, `save_draft`, `append_amendment`, `history_for_pet`, `prep_history_for_pet`, `unresolved_reviews`.

- `ContractRepository`
  - supplies `Contract.history` and retention/review defaults per location.

- `CalendarRepository`
  - consumes prep/estimate outputs but does not own notes.

- `storage::operations::grooming::history::*Record`
  - stores raw provider payloads and semantic records; promotes to domain only through checked conversions.

### Workflow events and staff tasks

Workflow events to introduce/refine:

- `workflow::Event::GroomingServiceCompleted { appointment_id, pet_id }`
- `workflow::Event::GroomingHistoryDrafted { draft_id, source }`
- `workflow::Event::GroomingHistoryApproved { entry_id, approver }`
- `workflow::Event::GroomingHistoryReviewRequired { draft_id, reason }`
- `workflow::Event::GroomingHistoryAmended { entry_id, amendment_id }`

Staff task kinds to add only when existing variants are insufficient:

- `operations::StaffTaskKind::GroomingHistoryReview { pet_id, draft_id }`
- `operations::StaffTaskKind::GroomingPrepReview { pet_id, appointment_id }`
- `operations::StaffTaskKind::GroomingManagerReview { pet_id, reason }`

Existing `DocumentReview`, `CustomerFollowUp`, and `IncidentFollowUp` can be used if they remain semantically truthful; do not overload them to mean groomer note approval when the distinction matters.

### Agent specs and tools

- `agents::grooming::HistorySummarizerSpec`
  - Input: typed history prompt packet with raw/provider source refs, allowed note categories, care-reference policy, and redaction policy.
  - Output: `HistoryDraft` plus evidence refs, risk flags, confidence, and review requirement.

- `agents::grooming::PrepPacketSpec`
  - Input: approved history entries and appointment request.
  - Output: draft prep packet and review flags.

- `tools::grooming::history::AppendDraftTool`
  - May save draft only; cannot approve durable history.

- `tools::grooming::history::ApproveEntryTool`
  - Requires typed staff approval token and audit metadata.

- `tools::grooming::history::ProviderImportTool`
  - Imports raw notes into boundary records and produces review-required domain drafts.

## 4. Interaction contract

Rust-like pseudo-signatures below intentionally put behavior on owners rather than free-floating helpers.

```rust
pub mod operations::grooming::history {
    pub type Result<T> = core::result::Result<T, Error>;

    pub struct EntryId(/* non-empty provider/domain id */);
    pub struct StyleNote(/* trimmed bounded sensitive text */);
    pub struct GroomerObservation(/* trimmed bounded sensitive text */);
    pub struct ProductUsageNote(/* trimmed bounded text + review hints */);
    pub struct MediaRef { /* storage key, media type, visibility, source */ }
    pub struct SourceEvidenceRef(/* provider note/task/message/photo id */);

    pub enum ServiceOutcome {
        CompletedAsPlanned,
        CompletedWithLimitations { reason: GroomerObservation },
        PartiallyCompleted { reason: GroomerObservation },
        Cancelled,
        NoShow,
        ManagerReviewRequired { reason: ReviewReason },
    }

    pub enum ApprovalState {
        Draft,
        NeedsGroomerReview,
        NeedsCareReview,
        NeedsManagerReview,
        Approved,
        Rejected,
        SupersededByAmendment,
    }

    pub struct HistoryDraft { /* typed draft fields, source, evidence, review */ }
    pub struct ServiceHistoryEntry { /* approved immutable core fields */ }
    pub struct HistoryAmendment { /* append-only correction */ }
}
```

### Construction and approval

```rust
impl operations::grooming::HistoryPolicy {
    pub fn draft_from_completed_service(
        &self,
        service: operations::grooming::CompletedService,
        notes: operations::grooming::history::DraftNotes,
        source: operations::grooming::history::EntrySource,
    ) -> operations::grooming::history::Result<operations::grooming::history::HistoryDraft>;

    pub fn classify_review_requirement(
        &self,
        draft: &operations::grooming::history::HistoryDraft,
        care_refs: &care::ReviewSnapshot,
    ) -> operations::grooming::ReviewRequirement;

    pub fn approve(
        &self,
        draft: operations::grooming::history::HistoryDraft,
        approval: policy::ApprovalToken<operations::StaffRole>,
    ) -> operations::grooming::history::Result<operations::grooming::history::ServiceHistoryEntry>;

    pub fn amend(
        &self,
        entry: &operations::grooming::history::ServiceHistoryEntry,
        amendment: operations::grooming::history::HistoryAmendmentDraft,
        approval: policy::ApprovalToken<operations::StaffRole>,
    ) -> operations::grooming::history::Result<operations::grooming::history::HistoryAmendment>;
}
```

Behavior notes:

- `draft_from_completed_service` may produce a draft requiring review; it should not skip approval merely because fields are syntactically valid.
- `approve` is the only domain transition from draft to durable entry.
- `amend` appends; it does not overwrite original groomer-authored content without an explicit retention policy and manager approval.
- Product/care/medical-sensitive notes are classified before approval and may require care or manager review.

### Repository contracts

```rust
pub trait operations::grooming::HistoryRepository {
    fn save_draft(
        &self,
        draft: &operations::grooming::history::HistoryDraft,
    ) -> operations::grooming::history::Result<()>;

    fn append_approved_entry(
        &self,
        entry: &operations::grooming::history::ServiceHistoryEntry,
    ) -> operations::grooming::history::Result<()>;

    fn append_amendment(
        &self,
        amendment: &operations::grooming::history::HistoryAmendment,
    ) -> operations::grooming::history::Result<()>;

    fn history_for_pet(
        &self,
        pet_id: entities::PetId,
        location_id: Option<entities::LocationId>,
    ) -> operations::grooming::history::Result<Vec<operations::grooming::history::ServiceHistoryEntry>>;

    fn unresolved_history_reviews(
        &self,
        location_id: entities::LocationId,
    ) -> operations::grooming::history::Result<Vec<operations::grooming::history::HistoryDraft>>;
}
```

Storage behavior:

- Provider/raw notes are loaded by boundary adapters and converted into `HistoryDraft` or parse-review failures.
- `history_for_pet` returns approved semantic history, not raw provider rows.
- Repository writes should persist audit metadata alongside history; audit logging is not an optional side effect hidden in application code.

### Prep, estimation, and rebooking contracts

```rust
impl operations::grooming::GroomingPrepPolicy {
    pub fn packet_for_appointment(
        &self,
        appointment: &operations::grooming::AppointmentRequest,
        history: &[operations::grooming::history::ServiceHistoryEntry],
        unresolved_reviews: &[operations::grooming::history::HistoryDraft],
    ) -> operations::grooming::GroomingPrepPacket;
}

impl operations::grooming::EstimationPolicy {
    pub fn estimate_from_history(
        &self,
        request: &operations::grooming::AppointmentRequest,
        pet_profile: &pet::Profile,
        history: &[operations::grooming::history::ServiceHistoryEntry],
        contract: &operations::grooming::Contract,
    ) -> operations::grooming::DurationEstimate;
}

impl operations::grooming::RebookingPolicy {
    pub fn recommend_from_history(
        &self,
        pet_id: entities::PetId,
        history: &[operations::grooming::history::ServiceHistoryEntry],
        contract: &operations::grooming::Contract,
        today: chrono::NaiveDate,
    ) -> Option<operations::grooming::RebookingRecommendation>;
}
```

Behavior notes:

- Prep packets can mention unresolved review flags but must not present them as approved facts.
- Duration estimates should explain whether history was used as `EstimateBasis::GroomerHistory` and whether review is required.
- Rebooking recommendations should identify the source entry and cadence basis: policy default, groomer recommendation, customer request, or AI-suggested pending review.

### AI summarization contract

```rust
impl agents::grooming::HistorySummarizerSpec {
    pub fn build_prompt_packet(
        source: operations::grooming::history::ProviderNotePacket,
        policy: &operations::grooming::HistorySummarizationPolicy,
    ) -> agents::PromptPacket;

    pub fn parse_agent_output(
        output: agents::StructuredOutput,
    ) -> operations::grooming::history::Result<operations::grooming::history::HistoryDraft>;
}
```

Behavior notes:

- Agent output must include evidence refs for each summarized field.
- Missing evidence or sensitive content yields `ApprovalState::NeedsGroomerReview` / `NeedsCareReview` / `NeedsManagerReview`.
- Agent output cannot directly call `append_approved_entry`, send a message, or update a provider appointment.

## 5. Review and approval contract

Automation levels:

- Fully automatic:
  - Validate note newtypes for trim/non-empty/length.
  - Classify obvious blank/oversized/malformed provider notes as parse-review-required.
  - Assemble prep packets from already approved history.
  - Draft rebooking recommendations from approved entries and deterministic cadence.

- AI draft/recommendation only:
  - Summarize legacy/provider notes into typed fields.
  - Suggest style-summary wording for a prep packet.
  - Identify likely product/coat concerns from prior notes.
  - Draft customer-facing rebooking/reminder text.

- Groomer/staff approval required:
  - Any new durable `ServiceHistoryEntry` from AI, provider import, front-desk intake, or incomplete source evidence.
  - Any style interpretation that could change cut/service outcome.
  - Any prep-packet summary that includes unresolved or contradictory notes.
  - Ordinary groomer-authored completed-service note approval, unless a future local policy explicitly treats same-session groomer submission as approved with audit.

- Care or manager approval required:
  - Medical, allergy, injury, temperament, safety, complaint, refund, or legal-sensitive content.
  - Amendment of an already approved entry when the change could alter future service, customer communication, or dispute history.
  - Rebooking/cadence decisions tied to no-show penalties, deposit requirements, service limitations, or customer complaint remediation.
  - Any member-facing message that discloses, apologizes for, or asks about grooming outcome concerns.

Audit trail:

- Record source (`GroomerAuthored`, `ProviderImport`, `AiStructuredDraft`, etc.), source evidence refs, draft author, approver, approval role, timestamps, approval decision, revision reason, and visibility/retention class.
- Preserve original notes and append amendments. Avoid destructive rewrite semantics.
- Redact sensitive note bodies from debug logs and generic operational briefs; expose only typed labels/review reasons where possible.

Customer/member-facing boundaries:

- Internal groomer notes are not automatically customer-visible.
- Customer-facing summaries, apologies, rebooking offers, price/deposit statements, reminders, and prep instructions require communication consent and approval through workflow/messaging policy.
- AI cannot hide concerning facts, rewrite complaint history, or silently transform internal notes into member-facing language.
- Care/medical suitability claims must be routed to care/manager review; grooming history can say what happened operationally, not make veterinary determinations.

## 6. Test contracts

Suggested domain tests should read as executable glossary entries.

### Domain construction and invariants

1. `grooming_history_entry_requires_pet_location_service_date_outcome_and_approval`
   - A durable entry cannot be built without typed identity, completed service facts, outcome, and approval metadata.

2. `grooming_style_notes_trim_reject_blank_and_redact_debug_output`
   - `history::StyleNote::try_new("  teddy bear face  ")` stores trimmed text; blank text errors; `Debug` does not leak the body if marked sensitive.

3. `grooming_history_entry_separates_style_notes_groomer_observations_product_use_and_care_refs`
   - Medical/temperament/care references are distinct from style instructions and product use.

4. `grooming_product_usage_note_for_medicated_product_requires_care_review`
   - Medicated/sensitive product usage cannot become ordinary approved style history without review.

5. `grooming_history_draft_from_provider_note_requires_source_evidence_and_review_state`
   - Provider imports become drafts with evidence refs and approval state, not approved entries.

6. `grooming_ai_summary_without_evidence_cannot_be_approved_as_service_history`
   - AI summaries missing evidence remain review-required.

7. `grooming_history_amendment_is_append_only_and_records_revision_reason`
   - Corrections append an amendment with reason/approver and preserve the original entry.

8. `grooming_history_visibility_class_prevents_manager_only_notes_from_prep_packet_customer_copy`
   - Manager-only/internal content can inform staff review but is not placed in customer-facing drafts.

### Policy behavior

9. `grooming_prep_packet_uses_approved_history_and_flags_unresolved_reviews`
   - Approved notes appear in prep; unresolved drafts appear as review flags, not facts.

10. `grooming_estimation_policy_marks_groomer_history_as_estimate_basis_when_prior_duration_used`
    - A prior approved history entry can inform `DurationEstimate` with explicit basis and review requirement.

11. `grooming_rebooking_policy_uses_last_completed_history_entry_and_cadence_to_mark_due_window`
    - Last completed service plus six-week cadence yields a due/overdue recommendation with rationale.

12. `grooming_rebooking_recommendation_outside_ordinary_cadence_requires_groomer_or_manager_rationale`
    - Recommendations outside 2-8 weeks cannot be represented as plain cadence integers.

13. `grooming_history_policy_routes_care_or_complaint_content_to_manager_or_care_review`
    - Sensitive content creates review tasks/approval boundary instead of becoming ordinary notes.

14. `grooming_history_policy_rejects_member_facing_send_from_unapproved_history_summary`
    - A rebooking/reminder/customer-message draft cannot be sent using unapproved history content.

### Repository/storage/boundary tests

15. `grooming_history_repository_returns_approved_semantic_entries_not_raw_provider_notes`
    - Domain repository returns `ServiceHistoryEntry` values; raw note payloads remain adapter/storage concerns.

16. `grooming_provider_note_record_promotes_parseable_payload_to_review_required_history_draft`
    - Boundary conversion creates semantic draft fields and review state.

17. `grooming_provider_note_record_preserves_unparseable_raw_payload_at_boundary_with_parse_review_error`
    - Unparseable raw text is not discarded or promoted into a generic note blob.

18. `grooming_history_record_roundtrips_visibility_retention_source_and_approval_metadata`
    - Storage codec preserves audit/approval semantics.

### Workflow/agent tests

19. `grooming_history_summarizer_returns_draft_with_evidence_risk_flags_and_approval_boundary`
    - Agent output is typed and review-gated.

20. `grooming_completed_service_event_creates_grooming_history_review_task_when_notes_need_approval`
    - Workflow creates a groomer/manager task based on review requirement.

21. `grooming_prep_agent_does_not_include_rejected_or_superseded_history_as_current_fact`
    - Prep summaries honor approval/amendment state.

22. `grooming_customer_message_draft_from_history_requires_consent_and_staff_approval_before_send`
    - Member-facing communication is draft-only without consent and approval.

## 7. Integration notes for later serialized Rust code card

Likely files touched:

- `domain/src/operations.rs`
  - Short-term location for `operations::grooming::history` if the operations module remains single-file.
  - Better follow-up refactor: split `domain/src/operations/grooming/{mod.rs,history.rs,history/error.rs}` once module boundaries become large enough.

- `domain/src/workflow.rs`
  - Add workflow events or typed workflow/task outputs for grooming history drafted/approved/review-required/amended.

- `domain/src/agents.rs` or future `domain/src/agents/grooming.rs`
  - Add agent spec input/output packets for history summarization and prep packets.

- `domain/src/tools.rs` or future `domain/src/tools/grooming/history.rs`
  - Add draft/approval/provider-import tool contracts. Keep execution ports separate from domain policy.

- `domain/src/care.rs`, `domain/src/temperament.rs`, `domain/src/pet.rs`
  - Add or reuse typed reference IDs/snapshots for care/medical/temperament facts. Grooming should depend on references/snapshots, not duplicate sensitive facts.

- `domain/tests/petsuites_core_service_contracts.rs`
  - Keep current `HistoryRequirement::KeepStyleNotesAndPhotos` contract test and add focused history behavior tests as the API appears.

- `domain/tests/domain_quality_patterns.rs`
  - Reuse note-newtype and redacted-debug patterns for style/groomer observations.

- Storage crate/tests, if present in the implementation card:
  - Add `storage::operations::grooming::history::*Record` codecs and provider-note parse tests.

Migration/refactor risks:

- `operations::grooming::HistoryRequirement` currently conflates retention policy with actual service-history entities. Later code should preserve the contract field but add `ServiceHistoryEntry` and `HistoryPolicy` rather than stuffing more booleans into `HistoryRequirement`.
- Current grooming module is embedded in `operations.rs`. Adding all history types there may make the file noisy; a module split should preserve canonical public paths and avoid parallel aliases.
- Existing `operations::GroomingService` and future `operations::grooming::ServiceOffering` may temporarily coexist. History entries should choose one canonical service/offering owner when implementation starts.
- Note text is sensitive. Avoid deriving ordinary `Debug` on note types if the body may include customer/pet specifics; follow the temperament staff-note redaction pattern.
- Provider payloads may be messy. Do not make raw provider notes the domain model just to get roundtrips passing.
- Audit semantics should not be a generic optional `Vec<String>`; model source, approval, revision, visibility, and retention explicitly.
- Do not let AI-generated summaries become the authoritative history path. The type transition must require approval.

Dependencies on other implications/domain slices:

- Scheduling/duration estimation implication: history feeds `EstimateBasis::GroomerHistory` and review requirements.
- Rebooking/reminders implication: history supplies last completed service, next cadence, and draft rationale.
- No-show/cancellation implication: cancelled/no-show outcomes and repeat behavior can affect deposit/manager-review decisions.
- Cross-sell implication: approved history can suppress/shape recommendations; unapproved or sensitive history should create review tasks instead.
- Care/temperament domain: grooming must reference care/medical/behavior facts via typed refs/snapshots.
- Customer/messaging consent: member-facing summaries and rebooking/reminder drafts require consent and approval.

Implementation sequencing recommendation:

1. Add the minimal `operations::grooming::history` note newtypes, `ServiceOutcome`, `ApprovalState`, `HistoryDraft`, and `ServiceHistoryEntry` with semantic tests for construction/category separation.
2. Add `HistoryPolicy::approve` and append-only amendment behavior with review-state tests.
3. Add repository trait and storage boundary records after the domain shape is stable.
4. Add prep/rebooking/estimation integrations only after approved history can be queried semantically.
5. Add AI summarizer/tool contracts last, keeping them draft-only and evidence-backed.
