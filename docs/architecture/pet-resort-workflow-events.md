# Pet resort workflow events

Status: canonical integration draft for implementation planning. This artifact defines the event-driven backbone for safe pet-resort workflow-agent invocation. It does not authorize production customer messages, provider/PMS writes, payment actions, reservation mutations, document/vaccine approvals, incident closure, care-task completion, or customer-facing policy decisions.

## 1. Purpose and scope

Workflow events are the semantic triggers that connect customer/provider/staff/system activity to bounded workflow agents and deterministic product code.

The purpose of this contract is to make the backbone explicit:

1. Adapters, staff actions, schedulers, and deterministic evaluators produce durable `WorkflowEvent` records.
2. Application-owned queue workers turn those events into source-grounded prompt packets or non-LLM workflow jobs.
3. Agents/workflows return `WorkflowResult<T>` envelopes containing extracts, summaries, drafts, recommendations, internal-task proposals, risk flags, and review reasons.
4. Deterministic policy, idempotency, approval records, outbox records, and audited adapters decide whether anything is executed.

The safe default is unchanged across this document:

- AI/workflows may read approved context, extract structured facts, summarize, detect gaps, draft internal tasks/messages, recommend review gates, suggest status/eligibility review, and flag risk.
- Humans approve customer-facing/outbound messages and policy-sensitive side effects unless explicit product/security/policy approval later creates a narrow deterministic path.
- Provider writes, payment/refund/waiver/credit operations, booking confirmation/rejection/cancellation/waitlist movement, vaccine/medical/behavior/play eligibility decisions, incident closure, and care-task completion are not direct consequences of an event or agent result.

## 2. Source inputs and assumptions

Primary source anchors:

- `README.md`: Rust-first platform direction: typed packets instead of free-form strings; app owns policy/state/writes; Hermes/LLM agents are bounded assistants.
- `docs/workflows/staff-operations-parts/inputs.md`: current substitute for the missing product map. It defines the first product shape as internal staff/manager operations, core actors, services, MVP emphasis, and conservative automation posture.
- `docs/workflows/staff-operations.md`: product surfaces and integration notes for operating-day dashboards, arrival/departure worklists, care boards, document review, incidents, manager review, message approvals, notes, handoffs, and AI review packets.
- `docs/architecture/pet-resort-data-model.md`: canonical entity/data-model artifact: customer, pet, reservation, service, room/suite, staff, task, document, vaccine, care note, incident, message, payment/deposit, audit event, and approval gates.
- `docs/architecture/workflow-result-envelope.md`: proposed result envelope contract and review semantics.
- `docs/workflows/workflow-event-idempotency-replay.md`: two-key inbox/outbox idempotency and per-event replay recommendations.
- `docs/data-model/workflow-queue-retry-dead-letter.md`: queue, retry, dead-letter, operator controls, and human-visible failure model.
- `docs/architecture/pet-resort-ai-runtime.md`: app-owned durable queue/inbox runtime pattern and prompt/result validation boundary.
- `domain/src/workflow.rs`, `domain/src/entities.rs`, `domain/src/policy.rs`, and `domain/src/operations.rs`: current Rust domain vocabulary for events, policy context, allowed actions, results, actors, reservation statuses, staff tasks, operations surfaces, and review gates.
- Gingr/PMS integration docs under `docs/integrations/gingr/`: provider events are boundary inputs that must be verified and semantically mapped before becoming workflow events.

Assumptions to preserve until explicitly changed:

- `docs/product/pet-resort-product-map.md` is missing. Use staff-operations inputs plus the canonical data model and workflow artifacts as the product-map substitute.
- First product surface is internal staff/manager workflow for one resort or a small resort group, with a later path to multi-location vertical SaaS.
- Core actors are customer/owner, staff/caregiver, lead/manager/admin, vet/emergency contact as referenced evidence, system/scheduler, external provider integration, and bounded AI workflow worker. Pet is a subject, not an actor.
- Core services are dog boarding, cat boarding, dog daycare/day play, day boarding/individual play, grooming/bathing/DaySpa, training where offered, loyalty/membership/reputation workflows where relevant, and optional webcam/customer-update experiences.
- MVP workflows include inquiry intake, pet/profile creation, document/vaccine review, booking triage/confirmation packet, staff operations/daily care, incidents, checkout, customer messaging, payment-sensitive review, and CRM/review request eligibility.
- V1 remains draft/review by default for final vaccine/eligibility/group-play decisions, booking acceptance/exceptions, deposits/payment/refund/loyalty handling, high-risk customer-facing sends, and any live provider mutation.

## 3. Workflow event envelope

A `WorkflowEvent` is a normalized domain event, not a raw webhook, raw provider event name, raw customer message, or model-generated instruction.

Recommended wire shape:

```json
{
  "event_id": "evt_...",
  "event_type": "booking.triage_needed",
  "schema_version": "workflow-event/v1",
  "occurred_at": "2026-06-11T12:00:00Z",
  "recorded_at": "2026-06-11T12:00:03Z",
  "source": {
    "kind": "policy_evaluator",
    "source_event_key": "v1:loc_1:booking.triage_needed:reservation:res_123:scheduled_policy_eval:...",
    "provider": null,
    "provider_event_id": null,
    "source_refs": ["reservation:res_123", "policy:booking:v7"]
  },
  "actor": { "kind": "system", "id": "booking-readiness-evaluator" },
  "location_id": "loc_1",
  "subject": { "kind": "reservation", "id": "res_123" },
  "related_ids": {
    "customer_ids": ["cust_123"],
    "pet_ids": ["pet_123"],
    "reservation_ids": ["res_123"],
    "document_ids": [],
    "task_ids": [],
    "incident_ids": [],
    "message_ids": [],
    "payment_ids": [],
    "audit_event_ids": [],
    "external_refs": []
  },
  "payload": {},
  "policy_context": {
    "policy_snapshot_id": "policy:booking:v7",
    "automation_level": "draft_only",
    "allowed_actions": ["read_entities", "create_internal_task", "draft_customer_message", "suggest_reservation_status", "flag_risk"],
    "required_reviews": ["manager_approval", "customer_message_approval"],
    "source_trust": "source_backed",
    "freshness": "fresh"
  },
  "approval_requirements": [
    {
      "gate": "customer_message_approval",
      "boundary": "outbound_customer_message",
      "reason": "Booking follow-up copy is customer-facing.",
      "blocked_actions": ["send_customer_message"]
    }
  ],
  "causation_ids": ["evt_booking_requested_..."],
  "correlation_id": "corr_..."
}
```

Field definitions and invariants:

| Field | Required | Definition | Invariants |
| --- | --- | --- | --- |
| `event_id` | yes | Stable app-owned event id. | Unique per accepted semantic event. Do not use raw provider delivery IDs as domain IDs without adapter normalization. |
| `event_type` | yes | One of the MVP catalog names below. | Behavior branches on semantic event type, not raw provider/webhook strings. |
| `schema_version` | yes | Event envelope version. | Validators reject unknown incompatible versions. |
| `occurred_at` | yes | Source fact time or scheduler/evaluator time for derived events. | Preserve provider/source time separately from record time. |
| `recorded_at` | yes | Time the app accepted the normalized event. | Used for audit and queue ordering; not a substitute for `occurred_at`. |
| `source` | yes | Source kind, dedupe key, provider refs, and evidence refs. | Raw payloads stay in boundary/evidence storage and are referenced, not copied into the event payload unless a workflow-specific policy allows it. |
| `actor` | yes | Customer, staff, manager, system, agent, or external integration attribution. | Actor attribution is not authority. Permission and approval are determined by policy context and approval records. |
| `location_id` | yes | Location/policy/capacity scope. | Required on every event; prevents cross-location dedupe and policy leakage. |
| `subject` | yes | One primary subject: customer, pet, reservation, document, incident, message, task, payment, or external. | Keep subject singular. Put secondary entities in `related_ids`. |
| `related_ids` | yes | Other entities needed to explain or process the event. | Use semantic IDs. Provider IDs are `external_refs` until reconciled. |
| `payload` | yes | Typed event-specific facts. | Contains minimal verified facts and evidence refs. It is not policy authority and should not contain unbounded raw JSON/OCR/free text. |
| `policy_context` | yes | Policy snapshot, automation level, allowed actions, required reviews, source trust, freshness. | Validator-owned authority. Agents may not expand allowed actions or remove gates. |
| `approval_requirements` | yes | Gate records naming required review, boundary, reason, and blocked actions. | Absence of a gate is not execution approval; a positive automation/policy decision is needed. |
| `causation_ids` | no | Prior workflow/source/audit IDs that caused this event. | Preserve lineage for replay and debugging. |
| `correlation_id` | yes | Trace ID across event, queue, agent call, result, task/draft/outbox, and audit. | Used for audit; not an idempotency key by itself. |

Envelope invariants:

1. Durable before processing: the normalized event and `source_event_key` are stored before any workflow, model invocation, task creation, draft creation, outbox entry, or provider call.
2. Source quarantine: raw webhooks, customer text, OCR, files, payment payloads, staff free text, and model output are evidence until validated and reviewed as needed.
3. Policy context narrows authority: `allowed_actions` and `required_reviews` are hard inputs from deterministic app policy, not model suggestions.
4. Review gates are sticky: retry/replay cannot remove a required gate except through an explicit newer policy snapshot or approval audit record.
5. A workflow event may produce many effects, but each effect needs its own idempotency key and approval/execution path.
6. Events never imply execution. A `booking.confirmation_needed` event means a confirmation packet/review path may be prepared; it does not confirm the reservation.

## 4. MVP event type catalog

Event names in this artifact use dotted documentation/wire names. Current Rust enums may use PascalCase names; implementation should map explicitly rather than deriving behavior from string casing.

Allowed action vocabulary used in the catalog:

- `ReadEntities`: read supplied or scoped app-owned context.
- `ExtractStructuredData`: extract sourced structured facts, especially from documents or intake text.
- `CreateInternalTask`: draft or create internal tasks only if task policy permits.
- `DraftCustomerMessage`: draft only; sending is separate and approval-gated by default.
- `SuggestReservationStatus`: recommendation only; provider/app mutation requires approved execution.
- `SuggestPlayEligibility`: recommendation/review only; no final playgroup decision.
- `SummarizeCareNotes`: summarize approved/scoped care evidence.
- `FlagRisk`: emit risk flags and review routing signals.

### 4.1 Inbound/customer/provider/staff events

| Event type | Source trigger | Actor | Subject | Expected payload | Policy / allowed actions | Approval and result expectations |
| --- | --- | --- | --- | --- | --- | --- |
| `inquiry.received` | Lead form, portal inquiry, phone/email/SMS/chat entered by staff, verified provider lead webhook/poll. | Customer/owner, staff, or external integration/system. | Customer if mapped; otherwise external lead. | Contact-channel refs, requested service/date range, pet summary if supplied, intake text/evidence refs, consent/contact preference if known. | Contact policy, service/location intake policy, source trust. Allowed: `ReadEntities`, `CreateInternalTask`, `DraftCustomerMessage`, `FlagRisk`. | Acknowledgement/follow-up is a draft requiring approval unless a future deterministic receipt-only path is approved. Result: intake summary, missing-info/review task, optional draft, `success`, `blocked`, or `needs_human_review`. |
| `pet_profile.created` | Customer/staff creates pet; provider animal import; profile reconciliation creates canonical pet. | Customer/owner, staff, or system import. | Pet. | Species/name/sex/spay-neuter/age/care-profile refs, source attribution, completeness/conflict snapshot. | Pet-profile completeness, vaccine/play/care policies. Allowed: `ReadEntities`, `ExtractStructuredData` for imports, `CreateInternalTask`, `SuggestPlayEligibility` as review-gated, `FlagRisk`. | AI may not approve medical/behavior facts, service eligibility, or group play. Result: profile intake summary, review tasks, missing-info draft, `success`, `blocked`, or `needs_human_review`. |
| `document.uploaded` | Customer/staff/provider upload or scanned paper document. | Customer/owner, staff, or system import. | Document or pet when mapped. | Document kind hint, storage/evidence refs, uploader/source, file metadata, expected content, scan/OCR/extraction status. | Document intake, privacy/retention, vaccine/source verification. Allowed: `ReadEntities`, `ExtractStructuredData`, `CreateInternalTask`, `FlagRisk`. | Upload does not verify vaccine/medical facts. Optional customer receipt/missing-info copy is draft/review. Result: document intake summary, extraction-needed recommendation, document-review task, `success`, `blocked`, or `needs_human_review`. |
| `booking.requested` | Customer requests boarding/daycare/grooming/training/DaySpa; staff enters request; verified provider reservation/request event. | Customer/owner, staff, or system import. | Reservation or external request. | Requested service/location/dates/times, pet ids, add-ons, accommodation preference, notes/evidence refs, deposit/payment refs if any. | Service, capacity, pet-profile/vaccine/readiness, deposit/payment, customer-message policies. Allowed: `ReadEntities`, `CreateInternalTask`, `SuggestReservationStatus`, `DraftCustomerMessage`, `FlagRisk`. | No autonomous confirmation, rejection, waitlist movement, overbooking, room allocation, payment action, or customer commitment. Result: booking triage packet and draft tasks/messages; usually `blocked` or `needs_human_review` until facts/review are satisfied. |
| `daily_note.created` | Staff/provider creates care, feeding, medication, grooming, training, play, photo/media, or handoff note during service. | Staff/caregiver/groomer/trainer/lead/manager or system import. | Reservation for stay-scoped note; pet for longitudinal note. | Note category, staff/source refs, task linkage, media refs, care/add-on/incident flags, shareability/sensitivity markers. | Care-plan, daily-update, incident/safety/medical review policies. Allowed: `ReadEntities`, `SummarizeCareNotes`, `CreateInternalTask`, `DraftCustomerMessage`, `FlagRisk`. | Customer-visible updates are drafts. Medical/medication/allergy/behavior/incident/safety facts require review before inclusion. Result: note-ingestion summary, daily-update-needed recommendation, risk/task routing. |
| `incident.created` | Staff/provider reports injury, altercation, behavior, safety, medication, escape, property, or customer-service incident. | Staff/lead/manager or verified provider import. | Incident, pet, or reservation depending on mapping. | Incident category/severity, time/place, involved subjects, immediate actions, evidence refs, notification/follow-up flags. | Safety/incident, medical/vet escalation, manager/legal/compliance, play-eligibility, customer-message restrictions. Allowed: `ReadEntities`, `CreateInternalTask`, `FlagRisk`, `DraftCustomerMessage` as review-gated, `SuggestPlayEligibility` as review-gated. | Incident customer messages, liability/medical statements, closure, downgrades, and play reinstatement are gated. Result: incident triage/manager packet, follow-up tasks, optional draft, usually `needs_human_review`. |
| `checkout.completed` | Staff completes checkout/release; verified provider checkout/poll; system import reconciles departure. | Staff/front desk, manager for exceptions, or system import. | Reservation. | Checkout timestamp, released-to ref, completed/incomplete care/add-on/task evidence, payment/final-balance refs, belongings, daily-report state, turnover trigger refs. | Checkout/release, payment/deposit, incident/care review, cleaning/capacity, customer-summary policy. Allowed: `ReadEntities`, `CreateInternalTask`, `SummarizeCareNotes`, `DraftCustomerMessage`, `SuggestReservationStatus`, `FlagRisk`. | Receipts, apologies, incident summaries, review requests, rebooking, final reports, provider status changes, and capacity return require approved paths. Result: checkout packet, turnover/follow-up/review eligibility tasks, `success` for internal summaries or `needs_human_review` for exceptions. |

### 4.2 Internally scheduled/derived events

| Event type | Source trigger | Actor | Subject | Expected payload | Policy / allowed actions | Approval and result expectations |
| --- | --- | --- | --- | --- | --- | --- |
| `vaccine.extraction_needed` | Derived from `document.uploaded` or profile/reservation requirements when vaccine proof needs OCR/extraction/review. | System or workflow worker. | Pet/document. | Required vaccine policy refs, document evidence id, extraction reason, freshness/source status, current known vaccine refs. | Vaccine/source verification, medical/privacy review. Allowed: `ReadEntities`, `ExtractStructuredData`, `CreateInternalTask`, `FlagRisk`. | Extraction suggestions are not verified vaccine facts or eligibility. Missing/unclear customer follow-up drafts require approval. Result: structured vaccine suggestions, document-review task, `success`, `blocked`, or `needs_human_review`. |
| `booking.triage_needed` | Derived after request/profile/document/capacity/provider update when readiness/eligibility/availability must be evaluated. | System or workflow worker. | Reservation. | Triage reason, requested service/date summary, current gaps, capacity/payment/document/policy refs, source evidence refs. | Booking triage, capacity, pet/vaccine/play/care, deposit/payment, staff/manager gates. Allowed: `ReadEntities`, `CreateInternalTask`, `SuggestReservationStatus`, `DraftCustomerMessage`, `FlagRisk`. | Customer follow-up, offer, denial, waitlist, confirmation, provider mutation, and payment actions are gated. Result: triage decision packet, status suggestion, tasks/drafts, `blocked`, `needs_human_review`, `no_action`, or safe `success` only for internal recommendation completion. |
| `booking.confirmation_needed` | Derived when triage indicates a booking can be offered/confirmed pending human approval, deposit/payment step, or provider write-back. | System or workflow worker. | Reservation. | Confirmation basis, dates/service/add-ons, hold/expiration, remaining conditions, deposit/payment refs, proposed customer-copy refs. | Confirmation, capacity hold, deposit/payment, staff/manager authority, customer-message gate. Allowed: `ReadEntities`, `DraftCustomerMessage`, `SuggestReservationStatus`, `CreateInternalTask`, `FlagRisk`. | Confirmation message and provider reservation status write are human-approval-gated. AI must not promise space, charge, or update provider state. Result: confirmation review packet, draft copy, approved-by-required task, `needs_human_review` until executed by approved path. |
| `daily_update.needed` | Scheduled cadence, customer preference, care-note threshold, media availability, active-stay policy, checkout-prep rule. | System or workflow worker. | Reservation. | Update reason/cadence/window, evidence refs, approved/excluded/sensitive note refs, channel preference, suppression reason if any. | Daily-update, customer preference/contact, care/incident/medical review, evidence approval. Allowed: `ReadEntities`, `SummarizeCareNotes`, `DraftCustomerMessage`, `CreateInternalTask`, `FlagRisk`. | Outbound daily/Pawgress update is approval-gated unless a future deterministic path is approved. Sensitive facts require staff/manager review or suppression. Result: daily-update draft packet, report-review task, suppression/no-send result, `needs_human_review`, `blocked`, or internal `success`. |
| `review_request.eligible` | Derived after checkout when policy checks show the stay/service may be eligible for review/reputation request. | System or workflow worker. | Reservation or customer. | Eligibility basis, checkout evidence, exclusions, incident/sentiment/payment/open-follow-up flags, proposed channel/timing, prior contact history. | Review/reputation, contact preference, suppression, customer-message gate. Allowed: `ReadEntities`, `DraftCustomerMessage`, `CreateInternalTask`, `FlagRisk`. | Review request is outbound customer messaging; gate by default. Suppress or manager-route when incidents, unresolved issues, payment disputes, complaints, or negative sentiment exist. Result: eligibility decision, draft/suppression reason/task, `needs_human_review` or internal `success` for no-send. |

## 5. Idempotency, replay, duplicate, and retry-safety rules

Use a durable two-key inbox/outbox model:

```text
source_event_key = v1:{location_id}:{event_type}:{primary_subject}:{source_kind}:{source_fingerprint}
side_effect_key = v1:{location_id}:{event_id}:{effect_kind}:{effect_subject}:{effect_intent}:{approval_or_policy_version}
```

Global rules:

1. `source_event_key` deduplicates ingestion of the same semantic fact.
2. `side_effect_key` deduplicates each effect: internal task, draft message, approved customer send, provider write, audit materialization, or review request.
3. Duplicate source events are acknowledged and audited; they do not blindly re-run side effects.
4. Conflicting duplicate keys route to reconciliation/review instead of overwriting canonical truth.
5. Replay with the same policy/approval version should reproduce the same recommendations and side-effect keys.
6. Replay under a newer policy is a new evaluation; it still cannot send/write/approve without required approval and outbox records.
7. Provider writes and outbound customer sends use approved immutable outbox records. Retry reuses the exact approved payload and reconciles unknown external status before another attempt.
8. Backfills default to read-model/audit rebuild and review packets. They must not send stale customer messages or mutate provider state unless a backfill policy explicitly enables a narrow effect.

Per-event rules:

| Event type | Source key shape | Duplicate handling | Replay behavior | Retry / side-effect safety |
| --- | --- | --- | --- | --- |
| `inquiry.received` | `location + inquiry.received + customer/external lead id + source_kind + lead/provider event id or normalized contact+submitted_at hash` | Merge duplicate evidence into one lead/review packet; conflicting contact/profile fields create review. | Rebuild lead summary and missing-info recommendation; backfills do not send conversion messages. | Follow-up task key by inquiry intent; draft key includes template/copy version; outbound send has separate approved-send key. |
| `pet_profile.created` | `location + pet_profile.created + pet_id/provider_animal_id + source_kind + created/updated version` | Suppress exact duplicate; changed medical/care/behavior fields route to profile reconciliation. | Rebuild completeness/readiness state; never infer service/group-play eligibility approval. | Profile/document/review task keys are task-kind-specific; provider writes require approved mutation action. |
| `document.uploaded` | `location + document.uploaded + document_id/provider_file_id + pet/customer/reservation subject + uploaded_at/version or content hash` | Same file/upload links to existing evidence/task; different docs create distinct review evidence. | Rerun OCR/extraction suggestions; do not mark vaccines verified or eligibility approved. | Extraction uses document-processing key; document-review task key includes document id; no customer/provider effect without later approved workflow. |
| `vaccine.extraction_needed` | `location + vaccine.extraction_needed + pet_id + document_id + required_vaccine_set + policy_version` | Same document/policy requirement maps to one extraction/review packet. | Recompute extraction suggestions against same evidence/policy; newer policy may create new requirement packet. | OCR/extraction retries are idempotent by document/version; verified vaccine record changes require medical document review. |
| `booking.requested` | `location + booking.requested + reservation/provider_request id + source_kind + request version/submitted_at hash` | Suppress duplicate request delivery; conflicts in dates/services/pets create reservation reconciliation/manager review. | Recompute readiness, missing-info, deposit, capacity, and review recommendations; no provider mutation. | Task keys per unresolved intent; drafts separate; offer/confirm/reject provider writes require approved executable action. |
| `booking.triage_needed` | `location + booking.triage_needed + reservation_id + triage_reason_set_hash + policy_version` | Same reason set maps to one open triage packet/task; new reasons append evidence or create policy-approved subtasks. | Rebuild triage packet and verify task state; do not multiply open tasks. | Customer sends/provider writes blocked; status update remains suggestion until approved outbox/action exists. |
| `booking.confirmation_needed` | `location + booking.confirmation_needed + reservation_id + readiness_snapshot_hash + policy_version` | Duplicate signal updates same confirmation packet when readiness unchanged; conflicts block confirmation. | Recompute readiness; same approval replay must not confirm twice. | Provider confirmation key uses approval id/policy; unknown prior write reconciles provider reservation before retry. Customer send key is separate. |
| `daily_note.created` | `location + daily_note.created + note_id/provider_note_id + reservation_or_pet + note created/updated version` | Exact duplicate suppressed; edited note updates evidence set without duplicate report tasks. | Rebuild approved-evidence summaries and drafts; note creation alone never sends Pawgress/daily update. | Draft key includes evidence-set/template; send key includes approved draft id/version/recipient/channel. |
| `daily_update.needed` | `location + daily_update.needed + reservation_id + service_day + update_window + policy_version` | Multiple triggers converge on one daily-update draft/review task per reservation/day/window. | Regenerate same draft if unsent or compare current draft to evidence; backfills do not send stale updates. | Review task, draft, and send keys are separate; unknown send status reconciles message provider/outbox first. |
| `incident.created` | `location + incident.created + incident_id/provider_incident_id + source_kind + incident version` | Duplicate incident suppressed; updates add evidence to existing incident-review packet/task when open. | Rebuild incident/manager/care review state and suppression reason; do not close/downgrade/notify automatically. | Manager/care review task key includes incident id; customer explanation send requires manager/customer-message approval; play/provider effects require approved action. |
| `checkout.completed` | `location + checkout.completed + reservation/provider_reservation id + source_kind + checkout event/version/timestamp` | Duplicate checkout does not duplicate turnover, payment follow-up, review request, or thank-you sends; conflicts route to reconciliation. | Rebuild checkout summary, turnover need, billing reconciliation, and review eligibility; no automatic charge/refund/provider status write. | Cleaning/billing/review task keys by intent; review/thank-you sends require approved eligibility/copy; unknown provider writes reconcile before retry. |
| `review_request.eligible` | `location + review_request.eligible + customer/reservation + eligibility_snapshot_hash + campaign/policy_version` | At most one active review-request draft/send candidate per stay/customer/campaign window. | Re-check suppression facts; stale eligibility or changed incident/payment/sentiment state blocks send. | Draft key includes campaign/reservation; send key includes approved draft, recipient/channel, campaign, suppression-check version, and approval. |

## 6. Workflow result envelope

Every workflow handler or agent returns a result envelope. The result is a reviewable decision-support artifact, not proof of external execution.

Canonical shape:

```json
{
  "status": "needs_human_review",
  "summary": "Short operator-safe summary of the result.",
  "structured_output": {},
  "recommended_actions": [],
  "draft_messages": [],
  "tasks_to_create": [],
  "risk_flags": [],
  "verification": {
    "evidence": [],
    "unchecked_sources": [],
    "redactions": [],
    "confidence": "source_backed"
  },
  "human_review_reason": "Required when status or policy requires review."
}
```

Status semantics:

| Status | Meaning | Side-effect behavior |
| --- | --- | --- |
| `success` | The workflow completed allowed analysis or deterministic internal operation. | Does not imply external side effects. Provider writes, customer sends, payments, booking acceptance, eligibility approval, care completion, or incident closure still require separately approved commands/audit records. |
| `needs_human_review` | Useful output exists, but policy, uncertainty, sensitivity, or action type requires human review. | No external send/mutation/payment/care completion may occur from this result. Route to the named review gate. |
| `blocked` | Required source data, credentials, policy, adapter availability, or approval is missing. | No side effects except optional draft/review task when policy permits. |
| `failed` | Processing failed safely. | No side effects. Preserve redacted failure evidence and retry only through orchestration rules. |
| `no_action` | Event was valid but no workflow action, draft, task, or review is needed. | No side effects. Verification explains why it is intentionally ignored/already satisfied. |

Field invariants:

- `status` is required and is never execution authority.
- `summary` is operator-safe and minimized; no unnecessary PII, secrets, raw payment/provider payloads, or customer-facing claims.
- `structured_output` is typed per workflow; confidence and extraction are not authority.
- `recommended_actions` are proposals. Names should be verbs like `request_human_review`, `suggest_reservation_status`, or `prepare_provider_lookup`, not `confirmed` or `message_sent`.
- `draft_messages` are content artifacts. Default `send_policy` is `draft_only` or `requires_approval`.
- `tasks_to_create` are drafts unless task-kind policy allows live creation; completion authority is separate.
- `risk_flags` route review; they do not approve or deny outcomes by themselves.
- `verification` cites stable source refs and redaction notes; raw artifacts stay in governed evidence storage.
- `human_review_reason` is required for `needs_human_review`, policy-required gates, sensitive drafts, provider/payment/customer-message actions, missing prerequisites, or approval-gated task creation.

Example: booking confirmation packet result:

```json
{
  "status": "needs_human_review",
  "summary": "Booking appears ready for a confirmation offer, but customer copy and provider status write require approval.",
  "structured_output": {
    "reservation_id": "res_123",
    "suggested_status": "offered",
    "remaining_conditions": ["staff approval", "customer-message approval", "provider write approval"]
  },
  "recommended_actions": [
    {
      "type": "suggest_reservation_status",
      "status": "offered",
      "review_gate": "manager_approval"
    }
  ],
  "draft_messages": [
    {
      "audience": "customer",
      "channel": "email",
      "body": "We can offer the requested stay pending final staff confirmation and any remaining requirements.",
      "send_policy": "requires_approval(customer_message_approval)",
      "source_refs": ["reservation:res_123", "policy:booking:v7"]
    }
  ],
  "tasks_to_create": [
    {
      "kind": "CustomerFollowUp",
      "title": "Review confirmation offer for res_123",
      "creation_policy": "requires_review(manager_approval)",
      "evidence_refs": ["workflow_event:evt_123"]
    }
  ],
  "risk_flags": ["customer_message_requires_review", "provider_write_requires_approval"],
  "verification": {
    "evidence": [
      { "source_type": "reservation", "source_id": "res_123", "summary": "Request and readiness packet were present." }
    ],
    "unchecked_sources": ["live provider write status"],
    "redactions": ["customer contact details omitted"],
    "confidence": "source_backed"
  },
  "human_review_reason": "Customer-facing confirmation and provider status changes require approval before execution."
}
```

Review semantics:

- A human may approve, reject, return for changes, or supersede a recommendation/draft/task.
- Applying an approval creates a separate approval/audit record and, for sends/writes, an immutable outbox record.
- Editing approved message/write payloads in place is forbidden. Material changes create a new draft and approval.
- Agent results may request approval but may not approve their own suggestions.

## 7. Queue, retry, and dead-letter model

Workflow processing should use an application-owned durable queue/inbox. The queue is not an LLM runtime feature; it is product infrastructure.

Recommended flow:

```text
verified source/staff/scheduler input
  -> semantic WorkflowEvent + source_event_key stored durably
  -> workflow_queue row(s) created with location, subject, event type, workflow kind, idempotency key, policy version, required review gates
  -> worker claims row with lease and attempt record
  -> workflow/agent returns WorkflowResult<T>
  -> app validates schema, correlation, policy, allowed actions, review gates, redaction, idempotency
  -> app persists result/task/draft/review packet or approved outbox record
  -> deterministic adapters execute only approved outbox actions
```

Current MVP storage/runtime mapping:

| Surface | Role in durable processing | Safety invariant |
| --- | --- | --- |
| `workflow_events` | Durable semantic inbox: accepted event kind, subject, source/idempotency key, payload, and timestamps are stored before any worker processing. | Event presence is not execution authority; it only starts a bounded analysis/review path. |
| `workflow_results` | Worker output envelope linked back to one `workflow_event_id`. | Results are reviewable evidence/recommendations, not proof that a customer message, provider write, payment action, eligibility decision, or care completion happened. |
| `approval_records` | Human/staff/manager decision record for a specific gate and target. | Only approved records with matching target/gate can support publishable outbox records. |
| `outbox_records` | Immutable, idempotent side-effect candidate after approval. | The migration requires a matching approved approval record; pending/claimed rows keep the approval immutable while execution is unresolved. |
| `audit_events` | Append-only trace of workflow, review, outbox, and operator decisions. | Audit rows preserve correlation/lineage and cannot be updated or deleted by retry/replay. |
| `apps/worker` runtime | Local worker shell/contract for claimed workflow/outbox work. | Defaults to `FakeDeterministic` agents and `Stubbed` side effects; there is no live adapter for customer sends, provider/PMS writes, or payment movement. |

The practical processing contract today is deliberately conservative: a claimed workflow/outbox record may be inspected, mapped to deterministic/fake workflow output, and routed to review/audit evidence. It remains `ReviewGatedStub` until a separate approved path exists. This gives reviewers a durable story without implying that the worker can publish to Gingr, email/SMS, payment providers, or customer-visible channels.

Queue row essentials:

- `queue_id`, `event_id`, `location_id`, `subject_kind`, `subject_id`, `event_type`, `workflow_kind`.
- `status`: `queued`, `claimed`, `succeeded`, `waiting_for_human`, `retry_scheduled`, `dead_lettered`, `cancelled`, `superseded`.
- `priority`, `available_at`, `claimed_at`, `claim_expires_at`, `claimed_by`, `attempt_count`, `max_attempts`, `next_retry_at`.
- Safe failure fields: `last_error_class`, `last_error_summary`, `last_error_redaction`, `failure_visibility`, `human_failure_title`, `human_failure_detail`.
- Gate/execution fields: `required_review_gates`, `approval_record_id`, `approved_action_id`, `idempotency_scope`, `idempotency_key`.
- Refs: `payload_ref`, `result_ref`, `audit_event_ids`, `policy_version`, `schema_version`, supersession links.

State model:

```text
queued -> claimed
claimed -> succeeded | waiting_for_human | retry_scheduled | dead_lettered | superseded | cancelled
retry_scheduled -> claimed when available and leaseable
waiting_for_human -> queued after human supplies missing information or resolves review
waiting_for_human -> superseded | cancelled
-dead_lettered -> queued only by explicit operator re-run under current policy
-dead_lettered -> superseded | cancelled
succeeded/cancelled/superseded are terminal for that row
```

Automatic retry is allowed only for infrastructure/provider failures that do not change business meaning or approval requirements:

- transient provider/network timeout;
- HTTP 429 with `Retry-After`;
- HTTP 5xx/provider unavailable;
- database serialization/deadlock;
- worker crash/lease expiry before committing result;
- temporary dependency unavailable.

Do not auto-retry these as infrastructure failures:

- missing or expired approval;
- policy denial;
- missing vaccine/profile/payment/care evidence;
- medical, behavior, eligibility, capacity, incident, refund/waiver/discount, or customer-message ambiguity;
- semantic-mapping/schema validation failure that needs repair;
- provider 4xx meaning invalid/forbidden/conflict requiring reconciliation.

Default transient backoff:

| Failure number | Delay |
| --- | --- |
| 1 | 1 minute |
| 2 | 5 minutes |
| 3 | 15 minutes |
| 4 | 1 hour |
| 5 | dead-letter unless operator extends with reason |

Dead-letter rows must expose human-safe failure state:

- status badge: waiting for staff/manager/engineering, retrying, failed-needs-review, dead-lettered, cancelled, superseded;
- affected location, subject, event type, workflow kind;
- safe error summary, last attempted time, attempt count, next retry if any;
- required review gates and missing evidence;
- operational impact;
- safe actions by role;
- blocked actions;
- audit/history timeline.

Operator controls:

- `Retry now`: transient failures only; preserves gates and approved payload refs.
- `Re-run under current policy`: replacement row, new audit event, re-reads canonical event/evidence, re-evaluates policy.
- `Assign review`: creates/assigns internal staff/manager/engineering task for missing evidence or approval.
- `Mark superseded`: link to a newer event/job.
- `Cancel`: terminal stop with reason.
- `Extend retry budget`: engineering/admin-only, audited.

Blocked controls:

- no force success;
- no customer send from failed draft without approval/outbox;
- no provider write retry with missing/expired/out-of-scope approval;
- no re-run that downgrades gates silently;
- no editing approved outbox payloads in place.

## 8. Human approval gates and outbound customer-message-triggering events

Approval model:

1. Workflow events and results can request review; they cannot approve themselves.
2. Approval records are separate durable records with approver, role/authority, gate, subject, payload/draft/action version, policy snapshot, scope, expiration if any, decision, and audit refs.
3. Approved side effects become immutable outbox actions keyed by `approved_action_id`.
4. Retry of an approved action reuses the exact approved payload. If payload, subject, recipient, destination, policy, or risk state changes materially, create a new draft/action and approval.
5. Review gates are not cleared by queue retry, model confidence, duplicate suppression, or absence of a negative flag.

Core gates to preserve:

| Gate | Required for |
| --- | --- |
| `medical_document_review` | Vaccine/medical final approval, ambiguous/expired/missing records, medical exceptions, source proof. |
| `behavior_review` | Group-play eligibility, bite/aggression/anxiety/escape/resource-guarding flags, incident-driven restrictions, temperament clearance. |
| `manager_approval` | Capacity/ratio exceptions, overbooking, waitlist exceptions, service acceptance/denial exceptions, profile merges, incident closure exceptions, destructive/superseding changes. |
| `refund_or_deposit_exception` | Waivers, refunds, credits, fee exceptions, direct payment mutations, deposit overrides. |
| `customer_message_approval` | Customer-facing outbound messages, especially medical/safety/legal/payment/service denial/incident/high-risk AI-drafted or non-templated sends. |
| `provider_write_approval` | Provider/PMS reservation/customer/pet/document/message/payment writes unless a narrow deterministic path is approved. |

Outbound customer-message-triggering event types:

| Event type | Message implication | Default send posture |
| --- | --- | --- |
| `inquiry.received` | Acknowledgement, missing-info, follow-up, lead conversion copy. | Draft/review; possible future deterministic receipt-only acknowledgement requires product approval. |
| `pet_profile.created` | Missing profile facts or care/vaccine follow-up. | Draft/review; medical/behavior/eligibility wording needs staff/manager review. |
| `document.uploaded` | Receipt, unclear-document request, missing proof request. | Draft/review; vaccine/medical interpretation needs medical document review. |
| `vaccine.extraction_needed` | Missing/unclear vaccine proof request. | Usually internal review first; any customer message draft/review. |
| `booking.requested` | Acknowledgement, missing-info, availability/follow-up copy. | Draft/review; no availability promise, acceptance, denial, waitlist, payment, or policy-exception copy without approval. |
| `booking.triage_needed` | Missing-info, waitlist, denial, offer, exception explanation. | Draft/review; sensitive/capacity/payment/eligibility content requires gate. |
| `booking.confirmation_needed` | Offer/confirmation copy, deposit/payment instruction, remaining-condition copy. | Human approval plus provider/payment/outbox path before send/write. |
| `daily_note.created` | Possible daily update ingredient or sensitive note follow-up. | Draft/review; note creation alone cannot send. |
| `daily_update.needed` | Pawgress/daily update. | Draft/review by default; future deterministic sends require approved template/fact/recipient/suppression/idempotency policy. |
| `incident.created` | Owner notification, follow-up, apology, incident summary. | Never auto-send in MVP; manager/customer-message approval required. |
| `checkout.completed` | Receipt/final report, follow-up, apology, rebooking, review request trigger. | Draft/review; receipts/payment/reports/review requests require approved policy/outbox path. |
| `review_request.eligible` | Review/reputation request. | Draft/review unless deterministic campaign eligibility/send path is approved with suppression checks. |

Provider-write/payment-sensitive implications:

- `booking.confirmation_needed`, `booking.triage_needed`, `booking.requested`, and `checkout.completed` may propose reservation status changes, capacity release, or payment/deposit follow-up. They cannot execute those changes without approval and outbox/provider-write idempotency.
- `document.uploaded` and `vaccine.extraction_needed` may propose vaccine records. They cannot verify compliance or change eligibility without medical/document review.
- `incident.created` may propose restrictions/escalations. It cannot close incidents, assign liability, alter play eligibility, or send owner/legal/public messages without the appropriate gates.
- `daily_note.created` and `daily_update.needed` may summarize care. They cannot mark care/medication/feeding tasks complete or send customer updates from unapproved evidence.

## 9. Open questions and implementation follow-ups

Implementation follow-ups:

1. Add or update Rust domain types for `WorkflowEvent` envelope fields that are not yet explicit: `schema_version`, `recorded_at`, `source_event_key`, `source` metadata, `approval_requirements`, `causation_ids`, and `correlation_id`.
2. Decide whether `location_id` stays top-level only, also appears in `related_ids`, or becomes a typed required field on both event and queue rows.
3. Add explicit `ExternalIntegration` actor/ref vocabulary if provider identity, permissions, or audit search need behavior beyond passive source refs.
4. Create durable stores/ledgers: `workflow_event_inbox`, `workflow_queue`, `workflow_attempts`, `workflow_side_effect_ledger` or action-specific outbox tables, `workflow_outbox`, `workflow_replay_request`.
5. Register workflow-specific `structured_output` schemas for inquiry, pet/profile, document/vaccine, booking triage, booking confirmation, daily care/update, incident, checkout, and review-request eligibility.
6. Implement validators that reject result actions outside `policy_context.allowed_actions`, missing required review gates, mismatched event/subject/workflow/schema, and unsafe raw data in logs.
7. Build fixtures proving duplicate source suppression, conflicting duplicate review, deterministic replay, backfill no-send/no-write, no-double-task, no-double-draft, no-double-customer-send, no-double-provider-write, unknown-status reconciliation, and dead-letter visibility for every MVP event type.
8. Define exact provider/Gingr mapping rules for each adapter event and source fingerprint. Raw provider event names should map into semantic events only after verification and identity reconciliation.
9. Decide which internal task kinds may be auto-created in production, with source freshness, due-time, duplicate-key, assignment, rate-limit, and completion-evidence policies.
10. Define approved customer-message automation, if any: templates, copy ownership, facts, recipients, channels, consent/suppression, triggers, review-bypass criteria, idempotency keys, safe logs, rollback, and monitoring.
11. Define pilot role authority for medical document review, behavior review, manager approvals, payment/deposit exceptions, provider-write approvals, and customer-message approvals.
12. Define location-specific policies for vaccines, capacity, playgroup ratios, deposits/refunds/waivers, cancellation/no-show windows, daily update cadence, review-request suppression, retention/redaction, and provider choices.
13. Decide whether `no_action` is a Rust status or maps to existing `RejectedByPolicy`/`Completed` variants during migration.
14. Align dotted event names with Rust enum names and generated schema names; do not let string mapping become implicit business logic.

Open product/architecture questions:

- Which workflows are allowed to create live staff tasks versus draft task recommendations in MVP?
- Is any routine customer-message class safe for deterministic auto-send, or is all customer-facing outbound review-gated for v1?
- What minimum v1 payment boundary is intended: manual references, generated payment links, or direct gateway operations behind approval gates?
- Which provider/PMS writes are in MVP, and which remain read-only/import-only?
- How should multi-pet bookings be modeled for event subjects: one reservation subject with pet `related_ids`, per-pet child events, or both?
- What is the approved reconciliation procedure when provider/message/payment external status is unknown after timeout?
- Should manager review queues be first-class aggregates, specialized staff-task projections, or both?
- What retention/redaction policy applies to raw provider payloads, documents/OCR, incident media, customer messages, rejected model outputs, queue errors, and audit evidence?

Acceptance checklist for implementation cards:

- Every workflow event has a durable source key, location scope, subject, policy context, allowed actions, review gates, evidence refs, and audit correlation.
- Every effect has its own idempotency key and, when needed, approval/outbox record.
- Workflow results are validated structured envelopes; prose is not executable.
- Queue retry distinguishes transient infrastructure failures from policy/human-review outcomes.
- Dead-letter state is visible, redacted, actionable, and unable to offer unsafe force-success/send/write controls.
- Customer-message/provider/payment/care/eligibility/incident side effects cannot occur from event replay, model confidence, duplicate delivery, or queue retry alone.
