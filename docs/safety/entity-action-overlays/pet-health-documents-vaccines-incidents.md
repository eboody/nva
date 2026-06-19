# Pet health, documents, vaccines, temperament, and incidents safety overlay

This overlay helps medical/vaccine reviewers, daycare leads, care teams, front-desk leads, and managers reduce stale-proof chasing, free-text care-note rereading, behavior-safety handoffs, and incident follow-up rework by showing what source facts automation may read, what it may draft or recommend, what a human must approve, and which outcome/audit record proves safe use.

Safe outcome: automation may assemble reviewable evidence, draft internal tasks or customer-message copy for approval, rank safety queues, and record reviewed dispositions; it must not approve vaccine/medical/behavior/incident decisions, send customers messages, mutate Gingr/PMS/provider records, change schedules/capacity, move money, delete documents, or change local policy.

## 1. Plain-English entity/action definition and labor-cost problem

This family covers the pet-safety records staff use before a pet can enter care, group play, boarding, daily updates, or owner follow-up:

- documents and uploads: vaccine proof, waivers, photos, medical records, incident evidence, customer uploads, staff scans, email ingest, provider poll/webhook evidence, and migration imports;
- vaccine/medical evidence: extracted vaccine names/dates, current/expired/rejected/exception states, local policy snapshots, and medical-document review gates;
- care notes and medical/care instructions: feeding instructions, allergies, medication names/doses/schedules, medical conditions, veterinarian/emergency-contact facts, and special-care review reasons;
- temperament and behavior: group-play observations, people orientation, ratings, behavior observations such as bite history, dog/human selectivity, escape risk, food guarding, or manager-review flags;
- incidents: injury, altercation, behavior, medication, escape, property, or customer-service incidents with severity, lifecycle status, redacted summaries, evidence documents, customer-message review, legal hold, and audit history;
- group-play/safety eligibility and approval actions: conservative play policy decisions, behavior-review gates, medical-document review gates, care-team approval, manager escalation, and customer-message approval.

The labor problem is that staff otherwise re-open uploads, pet notes, incident reports, and provider records every time they triage a booking, prepare a daily update, assess daycare eligibility, or explain a safety follow-up. The safe automation win is a source-backed review packet or queue item that says what evidence exists, why review is needed, who owns the decision, and what value was captured after review.

## 2. Workflows/contracts featuring it and adjacent entities

| Workflow or contract | Entity/action role | Adjacent entities | Source path or doc evidence |
| --- | --- | --- | --- |
| Document intake and OCR | Classifies documents, preserves immutable storage/hash/metadata, extracts candidate text, and routes uncertainty to human review. | Pet, customer, source document, storage ref, OCR result, review reason, audit event. | [`domain/src/document.rs`](../../../domain/src/document.rs); [`app/src/tools.rs`](../../../app/src/tools.rs) `documents`; [`apps/api/tests/vaccine_document_workflow_contract.rs`](../../../apps/api/tests/vaccine_document_workflow_contract.rs). |
| Vaccine-document workflow | Turns uploaded proof into pending-review vaccine records; staff approval can update eligibility while preserving lineage. | Pet, customer, vaccine record, document, medical-document review packet, approval record, eligibility state. | [`domain/src/vaccine.rs`](../../../domain/src/vaccine.rs); [`app/src/agents.rs`](../../../app/src/agents.rs) `vaccine-document`; [`apps/api/tests/vaccine_document_workflow_contract.rs`](../../../apps/api/tests/vaccine_document_workflow_contract.rs). |
| Booking triage | Uses vaccine, medical/care, and behavior findings as deterministic review gates before staff rely on booking readiness. | Reservation, pet, customer, policy snapshot, source evidence refs, readiness bucket, approval gate, blocked action. | [`app/src/booking_triage.rs`](../../../app/src/booking_triage.rs) `ApprovalGate::{MedicalDocumentReview, BehaviorReview, CareTeamApproval}`, `FailureCode::{MissingOrUnverifiedVaccine, BehaviorExceptionRequiresReview, SpecialCareRequiresReview}`. |
| Care-team handoff and daily care updates | Summarizes reviewed feeding, allergy, medication, medical-condition, and contact facts for shift handoffs or customer-safe drafts. | Care profile, staff note, message draft, customer-message approval, medical/care reviewer, source refs. | [`domain/src/care.rs`](../../../domain/src/care.rs); [`app/src/agents.rs`](../../../app/src/agents.rs) `daily-care-update`; [`app/src/tools.rs`](../../../app/src/tools.rs) `messaging`. |
| Temperament / group-play eligibility | Preserves behavior signals and asks behavior/daycare review before group-play access or customer-facing language changes. | Pet profile, temperament profile, play policy, incident history, behavior review gate, manager approval. | [`domain/src/temperament.rs`](../../../domain/src/temperament.rs); [`domain/src/policy.rs`](../../../domain/src/policy.rs) `play::ConservativePolicy`; [`domain/src/entities.rs`](../../../domain/src/entities.rs) `Pet`/`TemperamentProfile`. |
| Incident escalation | Classifies safety/customer-service incidents by category, severity, status, evidence, customer-message review, and legal/manager follow-up. | Pet, customer, reservation, evidence document, manager task, message draft, approval record, audit event. | [`domain/src/incident.rs`](../../../domain/src/incident.rs); [`app/src/agents.rs`](../../../app/src/agents.rs) `incident-escalation`; [`app/src/tools.rs`](../../../app/src/tools.rs) `media::CapturePurpose::IncidentReview`. |
| Shared safety and outcome vocabulary | Carries review gates, allowed/draft actions, blocked actions, source provenance, verification notes, and durable outcome evidence. | `ReviewGate`, `AllowedAction`, `RecommendedAction`, `RecordRef`, `Provenance`, `StoredSourceRecordRef`, outcome records. | [`domain/src/policy.rs`](../../../domain/src/policy.rs); [`domain/src/workflow.rs`](../../../domain/src/workflow.rs); [`domain/src/source.rs`](../../../domain/src/source.rs); [`storage/src/operations.rs`](../../../storage/src/operations.rs); [`../review-boundaries-matrix.md`](../review-boundaries-matrix.md). |

## 3. Who/what is authoritative

| Fact or decision | Authoritative source | What it can prove | What it does not prove |
| --- | --- | --- | --- |
| Uploaded document exists | Immutable stored object, original filename/MIME/content length/SHA-256, storage bucket/key/version, intake route, virus-scan and PII-redaction state in `domain::document`. | The resort has a specific file or extracted artifact available for review. | Vaccine validity, medical acceptance, behavior clearance, incident resolution, customer-send approval, deletion/retention approval, or provider write-back permission. |
| Extracted document/OCR candidate | `app::tools::documents::{document, ocr}` request/result and OCR review reasons such as low confidence, ambiguous dates, or missing fields. | What text/classification the tool candidate produced and why it needs review. | That OCR confidence is approval, that dates are accepted, or that broad raw document access is allowed. |
| Vaccine status | `domain::vaccine::Status` plus reviewed document and local vaccine policy evidence. | Whether a vaccine record is suggested, pending review, verified current/expired, rejected, exception-requested/approved, or superseded. | Live check-in/booking confirmation, group-play clearance, policy exception, or customer messaging without a reviewer. |
| Care instruction | `domain::care` redacted feeding/allergy/medication/medical/contact values and `MedicationReviewRequirement`. | The workflow has redacted care evidence and whether care-team review is required. | Medical advice, medication change, care-task completion, or customer-visible statement approval. |
| Temperament/behavior signal | `domain::temperament` observations and `domain::policy::play::ConservativePolicy`. | Which source/staff observations trigger behavior/daycare review and group-play ineligibility under the conservative policy. | Final group assignment, training advice, behavior exception approval, or customer blame. |
| Incident status/severity | `domain::incident::{Category, Severity, Status, Summary}` plus evidence documents and audit history. | What kind of source-backed incident is being routed and whether manager/customer-message/legal review is visible. | Legal/medical conclusion, disciplinary action, refund/discount, closure, or external communication. |
| Reviewer gate | `domain::policy::ReviewGate` and booking-triage `ApprovalGate`. | Which reviewer lane must clear before sensitive work advances. | Approval by itself, or authority for unrelated customer/provider/payment/schedule actions. |
| Workflow packet/action | `app::booking_triage::DeterministicResult`, `StaffEvaluationPacket`, safe agent actions, blocked actions, `AgentPromptPacket`. | What the app may summarize, draft, rank, validate, or route for review. | Live resort execution or permission to bypass source-of-record systems. |
| Outcome/audit record | API audit events, workflow events/results, source refs, `storage::operations` stored evidence/outcome records where present. | What staff reviewed, approved/rejected/deferred/corrected, which source refs were used, and what labor value was captured. | That the agent took the downstream live action. |

## 4. Agent may read

Within a scoped workflow packet, automation may read or inspect only source-backed facts needed for review:

- pet/customer/reservation identifiers and local policy references needed to attach the evidence to the correct subject; `domain::entities::Pet` carries temperament and care profiles, while `LocationPolicyRefs` carries vaccine and playgroup policy references;
- document metadata and storage references: classification, source route, status, virus-scan status, PII-redaction status, original file metadata, SHA-256 digest, and storage bucket/key/version;
- document/OCR intake request fields: expected content (`VaccineProof`, `MedicationInstructions`, `BoardingAgreement`, `IncidentReport`), intake source (`CustomerUpload`, `StaffScan`, `PortalImport`), classification result, extracted text, and review reasons (`LowConfidence`, `AmbiguousDates`, `MissingRequiredFields`);
- vaccine status and review state: `SuggestedExtracted`, `PendingReview`, `VerifiedCurrent`, `VerifiedExpired`, `Rejected`, `ExceptionRequested`, `ExceptionApproved`, and `Superseded`;
- care values after redaction/validation: feeding instructions, allergy labels, medical-condition labels, medical notes, medication name/dose/schedule, care contacts, and medication review reason;
- temperament values: staff notes, provider-specific behavior labels, group-play observation, people orientation, rating, and behavior observations;
- incident values: category, severity, status, redacted summary, evidence documents, affected subjects, required gates, and audit history;
- booking-triage rule findings for vaccine, behavior, and special-care failures, including evidence refs and readiness buckets;
- source refs/provenance: source system, endpoint, record id, extraction batch, timestamp, request scope, schema version, payload hash, raw payload ref, and related record roles;
- existing review history, approval status, audit events, workflow event ids, correlation ids, and prior outcome/disposition records when included by the app.

Scope limits: reading must stay within the specific location, pet, customer, reservation, document, incident, source snapshot, or workflow packet. Markdown here does not authorize broad provider browsing, raw-document exposure, camera/media access, payment records, customer threads, or secrets.

## 5. Agent may draft/recommend/rank/record

Automation may prepare review work, not final sensitive decisions. Allowed artifacts include:

- evidence summaries explaining which document, vaccine, care, temperament, or incident facts were found and which source refs support them;
- OCR/extraction candidate fields and uncertainty flags for a medical/vaccine qualified reviewer;
- document-review tasks, vaccine-review packets, clearer-proof request drafts, and missing-information request drafts;
- booking-triage staff evaluation packets with readiness buckets such as `VaccinePending` or `SpecialReview`, approval gates, rule evaluations, evidence refs, and blocked actions;
- internal care-team tasks for medication/special-care ambiguity, shift-handoff review, or updated care-note confirmation;
- behavior/daycare lead review packets for group-play observation, bite history, human selectivity, escape risk, food guarding, stale observations, or manager-review behavior flags;
- manager incident packets that summarize incident category/severity/status, missing fields, affected subjects, customer-message needs, legal-hold status, and evidence documents;
- customer-message drafts for approval when the app contract allows drafting, preserving recipient/channel/body/review policy and never sending directly;
- media snapshot requests/results only as review evidence for pet-status/facility-safety/incident-review context; unavailable results such as camera offline, permission denied, or retention expired must become review/failure evidence rather than silent retries;
- reviewed outcome records after a human disposition: reviewer/actor, approval status, decision reason, source refs, blocked action reasons, actual minutes, minutes avoided, wrong-source findings, and correlation id.

Authority level wording: use `DraftOnly`, `InternalTaskOnly`, `ManagerApprovalRequired`, or named review gates for these workflows unless a later source/Rustdoc/test contract proves a deterministic approved path. Do not write that the agent “approves,” “clears,” “sends,” “updates,” “closes,” “changes eligibility,” or “fixes source data” unless a reviewed outcome record shows a human/system-of-record performed that separate step.

## 6. Agent must not do directly

| Blocked direct action | Why blocked | Correct safe path |
| --- | --- | --- |
| Mark a document verified, rejected, superseded, archived, deleted, retained, or safe to expose broadly. | Document state controls compliance, PII, evidence retention, and safety decisions. | Route to medical/vaccine qualified staff, manager, IT/security, or records owner with document refs, scan/redaction state, and audit trail. |
| Treat OCR or extracted dates as vaccine/medical approval. | OCR confidence is evidence, not source-of-record review. | Create a medical-document review packet with low-confidence/ambiguous/missing-field reasons and require approval/rejection. |
| Mark vaccine valid/current/expired/waived, approve an exception, or clear check-in/eligibility from proof. | Vaccine validity and exceptions affect pet safety and local policy. | Medical/vaccine qualified staff reviews the document/policy/source refs; manager handles policy exceptions where local policy requires. |
| Accept or change medication, allergy, medical condition, special-care, veterinarian, or emergency-contact instructions. | Care data is sensitive and can affect health and staff work. | Care-team or medical/vaccine qualified staff approves; automation can draft a care-review task and record disposition. |
| Approve behavior exception, group-play eligibility, live group assignment, or daycare/boarding safety clearance. | Behavior safety affects pets, staff, and customers. | Behavior/daycare lead or manager reviews temperament, incident history, local play policy, and source notes. |
| Close, resolve, reopen, suppress, or legally characterize an incident. | Incident handling can affect safety, legal exposure, customer trust, and staff accountability. | Manager/legal/compliance lane reviews incident packet; automation may summarize and flag missing fields. |
| Send an owner/customer message about vaccines, care, health, behavior, incidents, refusals, or eligibility. | Sensitive customer communication needs final recipient/channel/body/timing approval. | Customer-message reviewer/approved sender approves a draft after medical/behavior/manager gates clear as applicable. |
| Mutate Gingr/PMS/provider records, create/delete/merge customers or pets, edit documents, update eligibility, or write incident/care/vaccine outcomes to a provider. | Provider/source-of-record mutation needs approved integration scope and human/system authority. | Create internal task or review packet; provider/system write happens only through an approved non-agent path. |
| Confirm/reject/cancel a booking, check in/out, release waitlist, change rooms/capacity/schedule, or complete care tasks. | Operational changes affect capacity, labor, safety, and customer commitments. | Staff/manager executes in the source system after reviewing the packet. |
| Capture/refund/void/retry payments, waive/forfeit deposits, discount/credit/write off charges, or change rates/taxes/fees. | Money movement is outside pet-health evidence authority and requires payment/accounting review. | Payment/accounting review packet if the health/safety issue has billing implications. |
| Hide source conflicts, wrong-source findings, stale evidence, low-confidence OCR, rejected documents, or failed media/document access. | Hidden ambiguity causes unsafe automation and rework. | Fail closed, record the uncertainty, and route to reviewer with source refs. |
| Change vaccine, play, care, incident, retention, privacy, or local SOP policy. | Policy authority belongs to product/ops/compliance/security/local leadership. | Mark an owner decision needed and keep the current safest behavior. |

## 7. Required human reviewer role(s) and approval condition

| Role | Usually approves | Does not approve | Review gate / packet |
| --- | --- | --- | --- |
| Medical/vaccine qualified staff | Vaccine proof, vaccine date/name/source ambiguity, medical-document evidence, medication/special-care ambiguity, and whether a document can affect readiness. | Payment/refund decisions, provider write-back scope, broad policy changes, or general customer-message approval outside the medical wording. | `domain::policy::ReviewGate::MedicalDocumentReview`; `app::booking_triage::ApprovalGate::MedicalDocumentReview`; vaccine-document review packet. |
| Care-team lead / trained care staff | Feeding/allergy/medication/special-care handoff quality and whether care notes are clear enough for staff tasks. | Medical diagnosis, provider mutation, money movement, or policy exception. | Booking-triage `CareTeamApproval`; internal care-review task; workflow verification note. |
| Behavior/daycare lead | Temperament observation review, group-play readiness recommendation, behavior flags, bite/selectivity/escalation review, and playgroup risk handoff. | Payment, provider write-back, final policy changes, or customer-message sends. | `domain::policy::ReviewGate::BehaviorReview`; `app::booking_triage::ApprovalGate::BehaviorReview`; behavior-review packet. |
| General manager / authorized manager | Incident escalation, customer-trust decisions, policy exceptions, service eligibility exceptions, suppression/escalation, staff/capacity implications, and high/critical incident handling. | Vaccine validity unless trained/assigned, payment processing unless authorized, IT/security integration scope. | `ReviewGate::ManagerApproval`; incident escalation packet; manager task/outcome record. |
| Customer-message reviewer / approved sender | Final recipient, channel, timing, body, tone, contact preference/consent, and whether owner-facing health/behavior/incident text should be sent. | Medical/behavior approval, provider writes, payment movement, or eligibility changes. | `ReviewGate::CustomerMessageApproval`; `app::tools::messaging::ReviewPolicy`; draft id/status. |
| Payment/accounting reviewer | Billing adjustments triggered by health/incident/booking decisions: refunds, waivers, deposits, credits, discounts, duplicate/amount/provider ambiguity. | Medical, behavior, schedule, or care decisions. | `ReviewGate::RefundOrDepositException` if money implications appear. |
| IT/security | Integration scope, raw document/media exposure, secrets, logging, retention, rate limits, tool failure modes, provider write-back security. | Business policy approval or individual resort operational decisions. | Tool-port review, security ticket, product/ops owner decision. |
| Product/ops owner | Whether a workflow/port may exist, what policy level applies, and which deterministic production path is approved. | Individual live action without the operational reviewer. | Policy/change approval artifact plus tests/source update. |

## 8. Required source evidence before a recommendation

| Recommendation or draft | Required source evidence first | If missing/stale/ambiguous |
| --- | --- | --- |
| Vaccine proof extraction packet | Pet id, customer id, document ref/storage key/hash, filename/MIME/content length, source route, extracted vaccine name/date/expiration if available, schema version, confidence, local vaccine policy snapshot, source refs. | Route to `MedicalDocumentReview`; keep eligibility false/pending; do not mark current, waive, confirm booking, or send customer message. |
| Medical/care instruction review | Redacted care fields, medication dose/schedule, allergy/condition labels, source note/document refs, reviewer history, changed-field reason, reservation/service context. | Create care-team review task; do not apply changed instructions, complete tasks, or use the fact in customer-visible copy. |
| Temperament/group-play review | Pet profile, species, spay/neuter status, group-play observation, rating, behavior observations, staff/source notes, incident history, local playgroup policy, source refs. | Route to `BehaviorReview` or manager; keep group-play assignment/eligibility blocked until review. |
| Incident manager packet | Original source/staff report, category/severity/status, redacted summary, evidence documents/media refs, affected pets/customers/reservation, missing fields, customer-message need, legal-hold/reopen state, source refs. | Escalate to manager/legal/compliance as appropriate; do not close, resolve, message, discount/refund, or hide the issue. |
| Customer-message draft about health/safety | Reviewed facts only, approved wording constraints/SOP snippet if present, recipient/channel/contact preference, medical/behavior/manager approval status, source refs, draft review policy. | Draft-only or do not draft if facts are unreviewed/sensitive; never send. |
| Booking-triage health/safety readiness | Rule evaluation evidence refs, vaccine/care/behavior failure code, approval gate, readiness bucket, blocked actions, staff decision boundary. | Use `ReviewPacketOnly`; do not confirm/reject/cancel, mutate provider record, send customer message, or move payment. |
| Outcome/value capture | Context packet id, review packet id, audit event, reviewer/actor, decision reason, source refs, approval status, blocked actions, actual minutes, wrong-source findings, correlation id. | Record a gap or rejected outcome; do not claim labor savings or safe completion from draft creation alone. |

## 9. Outcome/audit record proving safe use and value measurement

Safe use is proven by a chain of evidence, not by a single source fact:

| Proof needed | Example field or record | What it proves | What it does not prove |
| --- | --- | --- | --- |
| Source evidence | `domain::source::RecordRef`, `Provenance`, document storage ref/hash, OCR schema/confidence, source refs, policy snapshot. | The recommendation or review packet was grounded in a particular source artifact/snapshot. | Approval, customer send, provider write, eligibility change, or labor value. |
| Draft/recommendation | `AgentPromptPacket`, `StaffEvaluationPacket`, readiness bucket, safe agent actions, draft id/status, review packet id, internal task id. | Automation prepared reviewable work inside app constraints. | That the downstream live action happened. |
| Human approval/rejection | Review gate, reviewer role/staff id, status, timestamp, decision reason, audit event such as `approval.decision.recorded`. | The sensitive decision was reviewed by the named role. | Authority for unrelated payments, policy changes, provider writes, or sends. |
| Blocked action proof | `BlockedAction::{ConfirmBooking, RejectRequest, AcceptSpecialCare, ApproveBehaviorException, MutateProviderRecord, SendCustomerMessage, MovePayment}`, requested side effects, validation result. | The system knew what it was not allowed to do and stopped or routed it. | That staff later performed the action. |
| Outcome/value | Disposition, feedback, actual minutes, before/after minutes, minutes avoided, handle-time reduction, rework reduced, wrong-source findings, reporting group, correlation id. | Reviewed work happened and labor/safety value can be measured. | Guaranteed ROI or permission to automate future decisions. |

Value-measurement row for this family:

| Value measure | How to record it safely |
| --- | --- |
| Minutes avoided | Reviewer records minutes not spent reopening vaccine documents, pet notes, incident reports, or source screens because the packet summarized source refs and uncertainty. |
| Rework reduced | Count rejected/unclear proofs routed once with reason codes instead of repeated owner/staff back-and-forth; capture wrong-source findings and stale evidence. |
| Handle time reduced | Compare time to prepare a vaccine/care/behavior/incident review packet before and after source-backed drafts; include actual minutes, not estimates alone. |
| Wrong-source findings | Record low-confidence OCR, ambiguous dates, conflicting pet identity, missing required fields, stale temperament observation, or incident evidence mismatch as review outcomes. |
| Manager/staff disposition | Store approved, rejected, deferred, corrected, escalated, suppressed, or owner-decision-needed with reviewer/actor and reason. |
| Outcome capture | Link context packet id, review packet id, source refs, audit events, blocked actions, and correlation id so a non-coder can prove the agent stayed draft/review-only. |

The vaccine-document API tests demonstrate this distinction: upload creates `awaiting_review` document state, `pending_review` vaccine record, a `medical_document_review` packet, `rabies_current: false`, and audit events; staff approval later updates document verification, vaccine status, eligibility, approval status, and audit lineage. Staff rejection keeps eligibility false and marks the extracted record rejected.

## 10. Source/Rustdoc/test evidence links

Core source and Rustdoc module paths:

- [`../../../domain/src/document.rs`](../../../domain/src/document.rs): `domain::document::{Classification, Source, Status, VirusScanStatus, PiiRedactionStatus, StorageRef, OriginalFile}`.
- [`../../../domain/src/vaccine.rs`](../../../domain/src/vaccine.rs): `domain::vaccine::Status` for suggested/pending/verified/expired/rejected/exception/superseded states.
- [`../../../domain/src/care.rs`](../../../domain/src/care.rs): `domain::care::{FeedingInstruction, AllergyName, MedicalConditionName, MedicalNote, MedicationName, MedicationDose, MedicationSchedule, MedicationReviewRequirement}`.
- [`../../../domain/src/temperament.rs`](../../../domain/src/temperament.rs): `domain::temperament::{GroupPlayObservation, PeopleOrientation, Rating, BehaviorObservation}` and behavior-review evidence helpers.
- [`../../../domain/src/incident.rs`](../../../domain/src/incident.rs): `domain::incident::{Category, Severity, Status, Summary}`.
- [`../../../domain/src/entities.rs`](../../../domain/src/entities.rs): `domain::entities::{Pet, TemperamentProfile, CareProfile, LocationPolicyRefs}` and normalized pet/customer/reservation ids.
- [`../../../domain/src/policy.rs`](../../../domain/src/policy.rs): `domain::policy::ReviewGate`, denial reasons, automation authority levels, and `play::ConservativePolicy` behavior-review decisions.
- [`../../../domain/src/workflow.rs`](../../../domain/src/workflow.rs): workflow events, policy context, allowed actions, recommended actions, review reasons, risk flags, verification notes, and task/message fields.
- [`../../../domain/src/source.rs`](../../../domain/src/source.rs): `domain::source::{RecordRef, Provenance}` and source-system lineage fields.
- [`../../../app/src/agents.rs`](../../../app/src/agents.rs): `AgentSpec`, `WorkflowAgent`, `AgentPromptPacket`, and baseline specs for `vaccine-document`, `daily-care-update`, and `incident-escalation`.
- [`../../../app/src/booking_triage.rs`](../../../app/src/booking_triage.rs): health/safety `ApprovalGate`, `FailureCode`, `SafeAgentAction`, `BlockedAction`, `DeterministicResult`, and review-packet boundaries.
- [`../../../app/src/tools.rs`](../../../app/src/tools.rs): `messaging`, `documents`, `media`, and `hermes` tool-port draft/evidence boundaries.
- [`../../../storage/src/operations.rs`](../../../storage/src/operations.rs): stored source refs and durable outcome/labor evidence boundary; storage records do not authorize live provider writes or customer messaging.

Docs and tests:

- [`../review-boundaries-matrix.md`](../review-boundaries-matrix.md): reviewer lanes for document/vaccine/medical evidence, daily care updates, booking triage, tool ports, and blocked actions.
- [`../source-evidence-map.md`](../source-evidence-map.md): citation map for safety claims and caution against implying live side effects.
- [`../../design/entity-atlas-review-safety-boundaries.md`](../../design/entity-atlas-review-safety-boundaries.md): review-gate family page and named review gate meanings.
- [`../../../apps/api/tests/vaccine_document_workflow_contract.rs`](../../../apps/api/tests/vaccine_document_workflow_contract.rs): upload, extraction, review-packet, approval/rejection, eligibility, and audit-lineage proof.
- [`../../../apps/api/tests/data_quality_hygiene_agent_contract.rs`](../../../apps/api/tests/data_quality_hygiene_agent_contract.rs): internal cleanup packet and blocked side-effect proof that ambiguity should become review work, not hidden source repair.
- [`../../../apps/api/tests/manager_daily_brief_outcome_capture_contract.rs`](../../../apps/api/tests/manager_daily_brief_outcome_capture_contract.rs): outcome capture rejects blocked side effects including customer sends, provider/PMS mutation, schedule changes, money movement, and source issue hiding.

## 11. Open gaps or owner decisions

| Gap or owner decision | Why it matters | Safest current behavior | Evidence needed to close |
| --- | --- | --- | --- |
| No separate API contract test found for temperament/group-play or incident escalation outcomes in this pass. | Non-coder docs can cite domain/app specs, but not a full runtime proof for review-packet persistence. | Keep behavior and incident actions review-packet-only; cite domain/app source and mark runtime outcome proof as needed. | API tests or workflow tests proving behavior/incident packets, review gates, blocked actions, and outcomes. |
| Source inventory artifact `source-inventory.md` is still missing per parent handoff. | Writers lack a single family-by-family source list produced by the parent board. | Use `../source-evidence-map.md`, this overlay, and source paths above until the inventory exists. | Completed source inventory artifact or updated parent index. |
| Production policy for live sends/provider writes/eligibility mutation is not proven by this overlay. | Product copy could overstate automation authority. | Draft/review-only; no autonomous customer sends, provider writes, eligibility changes, or incident/care/vaccine approvals. | Approved deterministic production policy, source/Rustdoc changes, tests, and approval records. |
| Retention/destructive cleanup ownership for documents is not fully specified here. | Document deletion/retention affects PII, evidence, and compliance. | Do not delete/alter/archive documents autonomously; route to records/IT/security/manager as appropriate. | Retention policy source, storage lifecycle contract, reviewer role, and destructive-action tests. |
| Labor value fields for this specific family may not have a dedicated storage projection yet. | Product value should be measured, not asserted from drafts. | Record available reviewer disposition, minutes avoided, rework reduced, wrong-source findings, and correlation ids in reviewed outcome/audit records. | Dedicated health/document/behavior/incident outcome projection or API contract showing value fields. |

## Final reviewer checklist

- The opening names concrete resort labor reduced: stale proof chasing, care-note rereading, behavior-safety handoffs, and incident follow-up rework.
- A non-coder can route documents/vaccine/medical evidence, care notes, temperament/group-play, incidents, and customer-message drafts to the correct reviewer.
- “Agent may read,” “agent may draft/recommend/rank/record,” and “agent must not do directly” are separate.
- Source evidence, draft creation, human approval, downstream live action, and outcome proof are separated.
- Blocked actions are precise: customer sends, provider/PMS writes, schedule/capacity changes, payments/refunds/discounts, medical/vaccine/behavior/incident approvals, destructive cleanup, and policy changes.
- Value measurement requires outcome/audit fields, not intent or estimated ROI.
- Source/Rustdoc/test links are local and current as of this docs pass.
