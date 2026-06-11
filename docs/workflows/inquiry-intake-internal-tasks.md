# Inquiry intake internal task creation

Purpose: define the internal tasks that the `inquiry-intake` agent may draft or create from `inquiry.received` / `WorkflowEventType::InquiryReceived` events. This is a workflow-design artifact, not live production policy.

Status: draft task-creation contract. Until a later implementation card encodes and approves these rules, production behavior should treat these as `WorkflowResult.tasks_to_create` / `RecommendedAction::InternalTask` proposals. Live task auto-creation requires approved task-kind policy, idempotency keys, source evidence storage, and tests. This document does not authorize booking promises, availability promises, provider writes, medical/vaccine/behavior decisions, payment actions, or customer-facing sends.

## Source anchors

Use with:

- `docs/workflows/inquiry-intake-inputs.md` — canonical inquiry-intake input constraints and missing assumptions.
- `docs/architecture/pet-resort-workflow-events.md` — `inquiry.received` event envelope, allowed actions, approval posture, and result expectations.
- `docs/workflows/workflow-event-idempotency-replay.md` — source-event and side-effect keys for duplicate-safe task creation.
- `docs/architecture/workflow-result-envelope.md` — `tasks_to_create` / task draft semantics and review policy.
- `docs/workflows/staff-operations-parts/inputs.md` — provisional staff roles and approval boundaries.
- `docs/workflows/staff-operations-parts/daily-task-generation.md` — shared staff task contract, priority defaults, and task duplication posture.
- `domain/src/agents.rs` — baseline `inquiry-intake` and `lead-conversion` specs.
- `domain/src/workflow.rs` — `AllowedAction::CreateInternalTask`, `WorkflowResult`, `WorkflowStatus`, and `RecommendedAction::InternalTask`.
- `domain/src/operations.rs` — current `StaffTask`, `StaffTaskKind`, `StaffTaskPriority`, `StaffTaskAssignment`, `StaffRole`, `StaffTaskSource`, lead source/intent/stage/next-action surfaces.

## Shared task contract

Every inquiry-created task, whether draft or live-created under approved policy, must carry:

- `location_id` and policy snapshot/version.
- Task kind plus inquiry-specific sub-intent when the current `StaffTaskKind` is coarse.
- Title/body written for internal staff, not as customer-facing copy.
- Owner role or assignment queue.
- Due basis, due time, priority, and priority rationale.
- Creation trigger with event/source/evidence references.
- Required payload fields used to build and later complete the task.
- Completion evidence required from staff/manager/tooling.
- Related links: inquiry/lead/source message, customer when mapped, pet(s) when mapped, reservation/request when created, documents or availability snapshot where relevant.
- Review gate or creation policy: `DraftOnly`, `AutoCreateAllowed { policy_ref }`, or `RequiresReview { gate }`.
- Idempotency key and duplicate-handling action.
- Emitted events/audit observations for proposed, created, updated, suppressed, completed, cancelled, and escalated outcomes.

Current `StaffTaskKind` mapping is intentionally pragmatic:

- `call_customer` -> `StaffTaskKind::CustomerFollowUp { reason }`.
- `verify_docs` -> `StaffTaskKind::DocumentReview { pet_id }` when a pet is mapped; otherwise a `CustomerFollowUp`/review packet until a document- or inquiry-scoped task kind exists.
- `manager_review` -> nearest domain task kind with `StaffTaskStatus::NeedsManagerReview`; for pure inquiry policy review use `CustomerFollowUp` with manager assignment until a generic manager-review task kind exists.
- `check_availability` -> `CustomerFollowUp` or booking/readiness review packet until a reservation-scoped availability task kind exists; it must not imply an availability promise.
- `request_behavior_notes` -> `CustomerFollowUp` when the work is to collect notes from the customer; `PlaygroupAssessment { pet_id }` or manager/lead review only after a pet exists and policy requires internal behavior review.

## Task definitions

### 1. Call customer

Canonical intent: `call_customer`.

Creation trigger:

- `inquiry.received` contains enough contact detail for phone follow-up and the intake result cannot safely proceed from structured fields alone.
- Customer explicitly requests a call or supplies only a phone number.
- Source channel is phone transcript/voicemail/call-back request and the transcript is incomplete, uncertain, or staff-entered.
- Missing service/date/pet/contact-preference fields are better resolved synchronously by staff.
- Conflicting contact identity, consent, or repeated messages need human reconciliation before drafts/sends.

Do not create when:

- The same open `call_customer` task already exists for the same unresolved inquiry intent.
- Contact preference or consent forbids phone contact, unless a manager-approved exception policy exists.
- There is no callable phone number; create `request_missing_contact` / message-draft work instead.

Required payload:

- `location_id`, `workflow_event_id`, `source_event_key`, `policy_version`.
- `lead_id` or external source/provider id when not mapped.
- Optional `customer_id`; customer display name or safe label.
- Source channel and source message/transcript/evidence ids.
- Phone number reference, not necessarily raw number in AI/runtime logs; contact preference/consent status if known.
- Missing-field checklist and questions to ask, each tied to source evidence.
- Requested service/date range/pet summary if supplied.
- Sensitivity flags: payment, medical, vaccine, behavior, incident, capacity, or policy-exception content.

Owner/role:

- Default: `StaffRole::FrontDesk`.
- `StaffRole::LeadStaff` when the call is mainly operational/care-lane clarification.
- `StaffRole::Manager` when the customer asks for policy exception, complaint escalation, sensitive incident/behavior/medical/payment discussion, or contact-consent ambiguity cannot be resolved by front desk.

Due/priority defaults:

- Due: same business day or before the requested service-date decision window, whichever is sooner; immediate if the requested date is today/tomorrow or the customer asked for urgent callback.
- Priority: `Normal` for ordinary missing information; `High` for due-soon booking/date, repeated unanswered attempts, customer frustration, or a blocked near-term request; `Critical` only for safety/incident/urgent animal-welfare content routed to the proper escalation workflow.

Completion evidence:

- Staff actor, timestamp, call attempt outcome, number/contact reference used, and whether contact was reached.
- Structured notes for facts collected: service/date, pet(s), required docs, behavior/care notes, contact preference, and unresolved questions.
- If voicemail/no answer: approved next attempt or message-draft decision.
- Any sensitive/payment/medical/behavior/policy claims marked as customer-provided evidence, not verified truth.

Related links:

- Required: inquiry/lead/source-message/workflow event.
- Optional: customer, pet(s), reservation/request, draft message, call transcript/recording evidence, provider lead id.

Emitted events/audit observations:

- `internal_task.proposed` or `internal_task.created` with task kind `call_customer`.
- `inquiry.follow_up_needed` / `customer_contact.call_needed` as implementation-local audit names if event vocabulary is later expanded.
- `internal_task.updated` when duplicate evidence is attached.
- `customer_contact.attempted`, `customer_contact.completed`, or `internal_task.completed` when staff provides evidence.
- `manager_review.requested` when the call uncovers policy/sensitive escalation.

### 2. Verify docs

Canonical intent: `verify_docs`.

Creation trigger:

- Inquiry indicates boarding/daycare/day boarding/group play/grooming/training service and required documents or vaccines are missing, uploaded, stale, ambiguous, or source-unverified.
- Customer attaches a document, screenshot, email, or provider-imported file related to vaccines, forms, agreements, medication instructions, or pet requirements.
- Pet/reservation readiness cannot proceed without document review.
- `document.uploaded` / `VaccineDocumentUploaded` evidence is linked to the inquiry and needs review/extraction.

Do not create when:

- No pet can be mapped and the only action is to ask the customer to upload a document; use customer follow-up until a pet or document evidence exists.
- The same open document-review task already covers the same pet + document/evidence + requirement.
- The document was already staff-verified under the same requirement/policy version.

Required payload:

- `location_id`, `workflow_event_id`, `source_event_key`, `policy_version`.
- `customer_id` or external lead id; `pet_id` when mapped; pet name/species if safe.
- Document/evidence ids, upload/source channel, document kind hint, source freshness, and file hash/content reference where available.
- Requested service/date range and required-document policy refs.
- Current known document/vaccine status: missing, uploaded-unreviewed, stale, conflicting, rejected, or verified.
- Explicit review questions: identify doc type, extract candidate dates, verify source/freshness, request replacement, or escalate.

Owner/role:

- Default: `StaffRole::FrontDesk` for ordinary form/vaccine-proof collection and routing.
- `StaffRole::LeadStaff` or authorized care reviewer for care/medication instruction review.
- `StaffRole::Manager` for medical ambiguity, policy exceptions, rejected/expired document disputes, or any decision that would affect eligibility/customer-facing denial.

Due/priority defaults:

- Due: before booking triage/check-in readiness; for near-term requested dates, due immediately or before the next customer follow-up draft is approved.
- Priority: `Normal` for future inquiries; `High` when requested service date is near, check-in/booking triage is blocked, or customer is waiting on eligibility; `Critical` only if document content signals safety/medical emergency and should route to incident/medical escalation rather than ordinary intake.

Completion evidence:

- Reviewer actor/role, timestamp, document/evidence id reviewed, requirement/policy version, and disposition.
- Extracted candidate facts with confidence/source refs, or explicit `unreadable`/`wrong document`/`needs replacement` outcome.
- Statement of whether the document is accepted, rejected, needs manager review, or remains customer-provided unverified evidence.
- Any follow-up task/message draft created for missing/replacement documentation.

Related links:

- Required: inquiry/lead/workflow event and document/evidence id.
- Required when available: customer and pet.
- Optional: reservation/request, provider document id, vaccine extraction job, customer follow-up draft.

Emitted events/audit observations:

- `internal_task.proposed` or `internal_task.created` with task kind `verify_docs` / `DocumentReview`.
- `document.review_requested`; later `document.reviewed`, `document.rejected`, `document.needs_replacement`, or `vaccine.suggestion_created` if extraction is separated.
- `internal_task.updated` when repeated customer messages attach more files to the same review.
- `booking_triage.blocked_on_docs` as an audit/reason when applicable, not a booking decision.

### 3. Manager review

Canonical intent: `manager_review`.

Creation trigger:

- Inquiry contains or implies a policy exception: overbooking, waitlist/priority exception, age/spay-neuter/group-play exception, special handling exception, refund/discount/waiver/deposit dispute, cancellation/no-show dispute, or unusual service request.
- Content includes sensitive medical, medication, allergy, behavior, incident, safety, legal/liability, complaint, payment, or capacity/staffing ambiguity.
- Source facts conflict across customer/provider/staff records and could affect booking eligibility, customer-facing wording, provider state, or staff safety.
- The inquiry-intake agent cannot determine a safe owner/role for another task without manager authority.

Do not create when:

- The issue is an ordinary missing information request with no policy/sensitive implication.
- A manager-review task is already open for the same inquiry subject and semantic reason; attach the new evidence instead.
- The same sensitive issue belongs to a more specific already-open incident, payment, document, or behavior review task; link to that task rather than duplicating.

Required payload:

- `location_id`, `workflow_event_id`, `source_event_key`, `policy_version`.
- Inquiry/lead/customer/pet/reservation links available.
- Review reason enum/string from a constrained list: `policy_exception`, `capacity_or_waitlist_exception`, `medical_or_medication`, `behavior_or_safety`, `payment_or_deposit`, `complaint_or_legal`, `identity_or_conflicting_sources`, `customer_message_sensitive`, `other_manager_gate`.
- Minimal source excerpts/evidence refs supporting the review reason.
- Actions blocked pending review: customer message send, booking promise, availability offer, provider write, document acceptance, group-play eligibility, payment decision, or staff assignment.
- Recommended safe next step, if any, such as call customer, ask for docs, request behavior notes, or create booking triage packet.

Owner/role:

- Default: `StaffRole::Manager`.
- `StaffRole::LeadStaff` may receive ordinary operational triage only when the policy says no manager approval is required; otherwise keep manager assignment.

Due/priority defaults:

- Due: before any customer-facing response that could mention the sensitive issue, before staff proceeds with booking/eligibility routing, and before the requested date if near-term.
- Priority: `High` by default because it blocks intake; `Critical` for safety/incident/medical emergency/legal escalations; `Normal` only for future low-risk policy clarifications with no blocked customer response.

Completion evidence:

- Manager actor, timestamp, reviewed evidence refs, decision/disposition, and approved next action(s).
- Explicit approval/rejection/suppression of customer-facing wording if any draft was involved.
- Any policy snapshot or exception id if an exception is approved.
- Which blocked actions remain blocked and which downstream task/message/triage packet should proceed.

Related links:

- Required: inquiry/lead/source event and manager-review reason.
- Optional: customer, pet(s), reservation/request, documents, availability snapshot, incident/payment/document/behavior task ids, draft message ids.

Emitted events/audit observations:

- `internal_task.proposed` or `internal_task.created` with task kind `manager_review`.
- `manager_review.requested`; later `manager_review.approved`, `manager_review.rejected`, `manager_review.returned_for_info`, or `manager_review.escalated`.
- `draft_customer_message.approval_requested` when review is specifically for wording.
- `internal_task.updated` when repeated messages add evidence to the open manager review.

### 4. Check availability

Canonical intent: `check_availability`.

Creation trigger:

- Inquiry includes requested service/date/time range and the customer expects staff to tell them whether space or appointment time may be available.
- Availability/capacity snapshot is missing, stale, service-specific, location-specific, or requires human interpretation.
- Date/service/pet count/add-ons are sufficient to check availability but not sufficient for booking confirmation.
- Grooming/training appointment availability requires a staff/groomer/trainer calendar check not available to the intake agent.

Do not create when:

- Required date/service/location/pet count is missing; create `call_customer` or missing-info follow-up first.
- A check-availability task is already open for the same inquiry + service/date window + pet count + location + policy version.
- A current availability/readiness packet already exists and is linked to the inquiry under the same policy/snapshot version.

Required payload:

- `location_id`, `workflow_event_id`, `source_event_key`, `policy_version`.
- Inquiry/lead/customer id or external lead id.
- Requested service, date/time window, duration/stay length when known, pet count/species, add-ons, and location.
- Current availability/capacity/calendar snapshot id if present, with freshness timestamp.
- Known blockers: missing docs, pet profile gaps, behavior/medical/care constraints, deposit/payment gate, service not offered, or requested exception.
- Output requested: internal availability packet or booking-triage recommendation, not customer-facing promise.

Owner/role:

- Default: `StaffRole::FrontDesk` for ordinary boarding/daycare/date lookup.
- `StaffRole::Groomer` for grooming/DaySpa calendar checks when service-specific calendar ownership matters.
- `StaffRole::Trainer` for training consult/session availability.
- `StaffRole::LeadStaff` for daycare/playgroup/care-lane capacity or staffing interpretation.
- `StaffRole::Manager` for waitlist, overcapacity, ratio/staffing, holiday/peak, policy exception, or any availability decision that could imply acceptance/denial.

Due/priority defaults:

- Due: before the next approved customer follow-up; same business day for active sales inquiries; immediate when requested arrival/appointment is same day or next day.
- Priority: `Normal` for future dates with complete data; `High` for near-term requested date, high-value service, repeated customer follow-up, or availability nearing full; `Critical` only when capacity/safety/staffing risk is already active and should route to manager/operations escalation.

Completion evidence:

- Staff actor/role, timestamp, snapshot/calendar refs checked, service/date window checked, and result category: possible slot(s), no obvious slot, waitlist/review needed, stale data, or manager approval required.
- Explicit statement that no booking/availability was promised to the customer unless a separately approved customer message/send or provider action occurred.
- Any downstream booking triage, customer follow-up draft, or manager review task created.

Related links:

- Required: inquiry/lead/source event and requested service/date evidence.
- Optional: customer, pet(s), reservation/request, availability/capacity/calendar snapshot, waitlist/review task, draft response.

Emitted events/audit observations:

- `internal_task.proposed` or `internal_task.created` with task kind `check_availability`.
- `availability.check_requested`; later `availability.snapshot_checked`, `availability.check_needs_manager`, or `availability.check_stale`.
- `booking.triage_needed` may be derived after a staff-completed availability packet, but availability check completion is still not booking confirmation.
- `internal_task.updated` when repeated messages revise the date/service window.

### 5. Request behavior notes

Canonical intent: `request_behavior_notes`.

Creation trigger:

- Inquiry mentions daycare, day play, group play, boarding with play add-on, training, special handling, reactivity, fear, aggression, anxiety, escape risk, bite history, incident history, or other behavior/safety-relevant facts.
- Pet profile lacks current temperament/behavior notes required for the requested service or location policy.
- Customer asks whether a pet can join group play or needs special accommodation.
- Existing behavior facts are stale, conflicting, imported from unverified source text, or materially changed by the new inquiry.

Do not create when:

- Requested service does not require behavior notes and no behavior/safety signal exists.
- The same open behavior-notes request/review covers the same pet + service intent + policy version.
- The inquiry content signals an actual incident/safety event; route to incident/manager review instead of ordinary behavior-note collection.

Required payload:

- `location_id`, `workflow_event_id`, `source_event_key`, `policy_version`.
- Customer/lead id and pet id when mapped; if pet is not mapped, pet name/species/description from source evidence as an external lead reference.
- Requested service/date and why behavior notes are required.
- Current temperament/care-profile/behavior evidence refs and freshness status when available.
- Questions or structured note fields to collect: group-play history, dog/cat/social comfort, handling triggers, separation anxiety, bite/escape/incident history, medication/sedation caveat, special handling, prior daycare/boarding experience.
- Review gate: behavior/safety notes are customer-provided evidence until staff/lead/manager reviews them.

Owner/role:

- Default for customer collection: `StaffRole::FrontDesk` as `CustomerFollowUp`.
- Default for internal assessment after notes exist: `StaffRole::LeadStaff` or `KennelTechnician`/playgroup staff equivalent using `PlaygroupAssessment` where pet is mapped.
- `StaffRole::Manager` for aggression, bite, escape, severe anxiety, prior incident, service denial/exception, reinstatement, or sensitive customer-facing behavior language.

Due/priority defaults:

- Due: before daycare/playgroup/boarding acceptance or before the first play/assessment window; for future inquiries, before booking triage finalization.
- Priority: `Normal` for future requested service; `High` for near-term daycare/boarding/play request or known behavior ambiguity; `Critical` for urgent safety/incident signals routed to manager/incident escalation.

Completion evidence:

- Collected behavior-note source, staff actor, timestamp, and structured fields completed/unknown.
- Staff/lead/manager review state: needs assessment, approved for further triage, group-play review needed, individual-care lane suggested, manager review required, or incident escalation required.
- Source refs and explicit uncertainty markers; AI/customer free text must not be treated as final eligibility.
- Any customer-message draft or booking-triage status must remain separate and approval-gated.

Related links:

- Required: inquiry/lead/source event and behavior-note reason.
- Optional/required when mapped: customer, pet, reservation/request, prior behavior profile, incident ids, playgroup assessment task, draft follow-up message.

Emitted events/audit observations:

- `internal_task.proposed` or `internal_task.created` with task kind `request_behavior_notes`.
- `behavior_notes.requested`; later `behavior_notes.received`, `behavior_review.requested`, `playgroup_assessment.requested`, or `manager_review.requested`.
- `internal_task.updated` when repeated messages add behavior details to the same open request.
- `booking_triage.blocked_on_behavior` when behavior notes block booking/play triage.

## Duplicate suppression and repeated customer-message rules

Use two separate keys:

1. Source ingestion key from `docs/workflows/workflow-event-idempotency-replay.md`:

```text
source_event_key = v1:{location_id}:InquiryReceived:{customer_or_external_subject}:{source_kind}:{lead_or_message_id_or_normalized_contact_submitted_at_hash}
```

2. Task side-effect key:

```text
task_key = v1:{location_id}:internal_task:{task_intent}:{domain_subject}:{semantic_reason}:{policy_version}
```

Recommended inquiry-specific `domain_subject` values:

- Known customer only: `customer:{customer_id}`.
- Known pet-specific task: `pet:{pet_id}`.
- Known reservation/request: `reservation:{reservation_id}`.
- Unmapped lead: `external:{provider}:{lead_id}` or `external:contact_hash:{normalized_contact_hash}`.
- Document-specific review: include `document:{document_id}` in `semantic_reason` or use a document-scoped subject once modeled.

Task-specific semantic reasons:

- `call_customer`: missing field set hash + callback/contact reason, e.g. `missing:{service,date,pet_count}:callback_requested`.
- `verify_docs`: requirement/document tuple, e.g. `vaccine_proof:{pet_id}:{document_id_or_requirement_hash}`.
- `manager_review`: constrained manager reason + blocked action set, e.g. `capacity_exception:customer_message_and_offer_blocked`.
- `check_availability`: requested service + normalized date window + pet count + location + snapshot/policy version, e.g. `boarding:2026-07-03_to_2026-07-07:2dogs`.
- `request_behavior_notes`: pet/service behavior requirement + freshness/version, e.g. `dayplay:temperament_missing_or_stale`.

Rules:

- If an open task with the same `task_key` exists, do not create another task. Attach the new inquiry event/source-message/evidence id, update the task body/checklist if new non-conflicting facts arrived, and emit a duplicate/update audit observation.
- If the new message contains the same facts but different wording, treat it as duplicate evidence.
- If the new message adds materially new facts that still belong to the same unresolved work, update the existing task and preserve both evidence refs.
- If the new message conflicts with prior facts that matter for contact identity, service/date, pet identity, documents, behavior, capacity, payment, or policy, do not overwrite the existing task as if resolved. Add conflict evidence and create or update `manager_review` / reconciliation work.
- If a task was completed and the same customer repeats the same request with no new source version, do not reopen automatically; append audit evidence and return the completed task reference if the prior completion still applies.
- If a task was completed but new source version/policy version/date/service/pet/document facts create a materially new obligation, create a new task with a new `semantic_reason` or policy-version dimension and link the prior task as history.
- Never dedupe solely by raw message text or local event id. Normalize around location, subject, task intent, unresolved semantic reason, policy version, and stable source ids.
- Do not let repeated customer messages create repeated customer-message drafts or sends. Draft keys and outbound send keys are separate from task keys; sends require approved draft/version and approval id.
- Backfills/import replays should default to review packets or suppressed task proposals unless a backfill policy explicitly allows creating current staff tasks from historical inquiries.

## Output shape for inquiry-intake agent

When the agent identifies one of these tasks, the workflow result should include a task draft with:

```json
{
  "kind": "CustomerFollowUp | DocumentReview | PlaygroupAssessment | nearest current StaffTaskKind",
  "task_intent": "call_customer | verify_docs | manager_review | check_availability | request_behavior_notes",
  "title": "Internal title with lead/pet/request reason",
  "body": "Checklist, source refs, blocked actions, and completion instructions",
  "assignment": { "role": "FrontDesk | LeadStaff | Manager | Groomer | Trainer | KennelTechnician" },
  "priority": "Low | Normal | High | Critical",
  "due_basis": "relative rule and source timestamp/date",
  "source": { "workflow_event_id": "...", "source_event_key": "...", "evidence_refs": [] },
  "related_ids": { "lead_id": null, "customer_id": null, "pet_ids": [], "reservation_id": null, "source_message_ids": [] },
  "completion_evidence_required": [],
  "creation_policy": "DraftOnly | AutoCreateAllowed | RequiresReview",
  "task_key": "v1:..."
}
```

Safe defaults:

- `creation_policy`: `DraftOnly` unless location policy explicitly approves this exact trigger and assignment for live task creation.
- `status`: `NeedsMoreInformation` when the result mainly asks staff/customer for missing facts; `NeedsHumanReview` when manager/sensitive review is required; `Completed` only when the agent produced an internal packet with no live side effect and no review needed.
- `AllowedAction::CreateInternalTask` permits task drafting/creation only within policy. It does not permit customer sends, provider writes, booking confirmation, final document/vaccine approval, group-play eligibility, or payment decisions.

## Implementation gaps to preserve

- There is no explicit inquiry/lead aggregate in the current core entity model beyond `WorkflowSubject::External`, reservation `Inquiry` status, provider lead refs, and `operations::Lead`. Keep `lead_id`/`inquiry_id` fields explicit in design until the aggregate is modeled.
- Current `StaffTaskKind` lacks dedicated `CallCustomer`, `ManagerReview`, `CheckAvailability`, and `RequestBehaviorNotes` variants. Use current kinds plus `task_intent` metadata for now, or add typed variants in a later data-model card.
- Chat widget is not represented in current `ReservationSource`, `ContactChannel`, or `DeliveryChannel`; duplicate/source keys must not silently collapse chat if transcript/consent/send behavior differs.
- Exact due-time clocks, service-specific document requirements, availability snapshot freshness, behavior-note questionnaires, and auto-create policies are location/policy inputs and should remain configurable/reviewable.
