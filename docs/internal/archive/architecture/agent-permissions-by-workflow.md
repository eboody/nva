# Agent permissions by workflow type

Purpose: define the conservative read/write/tool/data envelope for pet-resort AI workers. This matrix is an implementation and security-review input; it does not approve production agent permissions, live provider write access, or customer-message automation.

Status: draft permission policy. Human approval is still required for the final agent-permission model and for any customer-message automation policy.

## Source anchors

- `domain/src/agents.rs` baseline agent specs and forbidden actions.
- `domain/src/workflow.rs` `AllowedAction`, `PolicyContext`, `WorkflowResult`, `RecommendedAction`, and review-gate shape.
- `domain/src/policy.rs` `AutomationLevel` and `ReviewGate` values.
- `domain/src/tools.rs` app-owned store, reservation, payment, portal, and runtime trait boundaries.
- `docs/workflows/staff-operations-parts/inputs.md` staff operations, care, document, incident, daily-update, and approval-gate inputs.
- `docs/workflows/payments-pricing.md` and `docs/workflows/payments-pricing-parts/ai-boundaries.md` payment-sensitive boundaries.
- `docs/integrations/gingr/sdk-readiness-review.md` provider write/customer-message/raw-payload risk boundaries.

## Global permission principles

1. App-owned runtime boundary: agents receive typed prompt packets and return validated `WorkflowResult`-style drafts, recommendations, risk flags, verification notes, and review requests. Application policy, deterministic validators, staff/manager approvals, and bounded tools own side effects.
2. Least privilege by workflow: each worker gets only the entities, documents, history, audit context, and tools needed for that workflow and subject/location/time window.
3. Draft/recommend by default: agents may draft messages, internal tasks, status suggestions, and review packets. They may not directly mutate reservations, customer records, pet/care records, vaccine truth, payments, capacity, staff schedules, provider records, or outbound communication state unless a separately approved deterministic path executes after review.
4. Review gates are data, not prose: every output that needs approval must carry the specific gate (`ManagerApproval`, `MedicalDocumentReview`, `BehaviorReview`, `CustomerMessageApproval`, `RefundOrDepositException`) plus source refs and rationale.
5. No model-only authority: confidence, urgency, sentiment, customer pressure, or prior AI output is not approval evidence.
6. Audit always: prompt packet creation, source snapshot IDs, agent result validation, draft/task/status recommendations, human approvals/rejections, provider actions, outbound sends, suppressions, and failures must produce safe audit/workflow events.

## Customer-message automation levels

Use these labels consistently in implementation and review:

| Label | Meaning | Examples |
| --- | --- | --- |
| `DraftOnly` | Agent may write copy, but a human must approve before queueing or sending. | Incident messages, eligibility/refusal, payment/refund, medical/behavior, legal/safety, ambiguous facts. |
| `QueueForReview` | Agent may place a draft into a review queue with recipient/channel/fact packet, but cannot send. | Missing-info follow-up, vaccine-document request, ordinary booking clarification. |
| `DeterministicAutoSendOnly` | An approved non-agent send path may send from fixed facts/templates/conditions; AI-authored copy is allowed only after template, facts, recipient, trigger, and suppression rules are pre-approved. | Routine reminder using approved template and verified facts. |
| `NeverAutoSend` | No autonomous send; manager or authorized staff must explicitly approve the final text and send action. | Incident/safety/medical/legal/payment exception/refusal/complaint/dispute messages. |

Default before security/product approval: `QueueForReview` for low-risk routine customer drafts, `DraftOnly` or `NeverAutoSend` for sensitive drafts, and no AI-controlled direct sends.

## Workflow permission matrix

| Agent / workflow type | Read permissions | Write permissions | Forbidden actions | High-risk actions requiring human approval | Customer-facing message policy | Safe log redaction rules |
| --- | --- | --- | --- | --- | --- | --- |
| Intake | Location/service catalog; approved intake requirements; customer/portal identity match candidates; customer contact preferences; pet basics; requested service/date/source channel; recent intake/message thread for the same inquiry; minimal audit/workflow events for dedupe. | Missing-info summary; structured extracted fields with confidence/source refs; duplicate/ambiguous-match flags; draft internal intake tasks; follow-up message drafts; audit/workflow result records. No direct customer/pet/reservation creation unless an approved deterministic import path verifies and writes. | Confirm bookings; promise availability; create or merge customer/pet records from ambiguous matches; overwrite contact preferences; send sensitive messages; infer medical/vaccine/payment truth from free text. | Identity merge; first-time customer creation with ambiguous data; contact preference changes; any response involving payment, policy exception, refusal, safety, behavior, or medical/vaccine interpretation. | Routine missing-info drafts may be `QueueForReview`; deterministic auto-send only after approved template/facts/recipient/suppression policy. Sensitive or ambiguous intake remains `DraftOnly`. | Redact contact PII in model/debug logs except role-appropriate recipient/channel labels; redact raw message bodies unless needed in a review packet; replace attachments with document IDs; log source refs, field names, and missing/ambiguous states rather than full text. |
| Booking | Reservation request; location policy snapshot; service availability/capacity snapshot; customer/pet/reservation records needed for the request; vaccine eligibility status, not raw documents unless required; deposit/payment status as semantic values; relevant message history; audit/workflow history for prior offers/rejections. | Booking triage packet; eligibility/capacity/deposit/vaccine gap flags; internal tasks; draft customer explanations; reservation status suggestions; review-gate requests; audit/workflow result records. No direct status/provider mutation. | Confirm, reject, cancel, waitlist-release, check in/out, or modify reservations directly; invent availability or rates; override hard stops; waive deposits/fees; allocate rooms/capacity; alter policy snapshots; send booking/payment-sensitive messages autonomously. | Booking acceptance/rejection; overbooking/capacity exceptions; waitlist release; holiday/peak/minimum-stay exceptions; deposit/refund/waiver/discount/forfeit decisions; group-play or medical/behavior exceptions; customer commitments. | Routine factual booking clarification is `QueueForReview`; confirmations/rejections/payment-sensitive/eligibility/refusal/policy-exception messages are `DraftOnly` or `NeverAutoSend` unless a deterministic approved send path covers the exact case. | Redact payment provider refs beyond semantic status; redact prices unless sourced to approved policy snapshot; do not log raw provider payloads, unmasked contact PII, or full customer messages outside review packets; log policy IDs/snapshot IDs and decision reasons. |
| Document / vaccine | Uploaded file metadata and document images/OCR for assigned pet only; approved vaccine policy snapshot; existing verified vaccine records; pet species/service/reservation context; document review history; audit logs for prior verification/rejection. | OCR/extraction candidates; source/crop/page refs; uncertainty flags; document-review tasks; suggested vaccine-record updates; draft customer request for clearer proof; audit/workflow result records. Verified records remain human/system-written after review. | Final-approve uncertain medical documents; mark a vaccine valid/expired/waived directly; infer licensed-vet proof when absent; delete/alter uploads; expose document images broadly; use OCR confidence as approval. | Licensed-vet source verification; ambiguous vaccine names/dates/pet identity; expired/missing vaccines; waivers/exceptions; eligibility effects; customer messages explaining denial or medical requirements. | Requests for missing/clearer proof may be `QueueForReview`; denial/eligibility/medical interpretation is `DraftOnly` with medical-document review. No autonomous send when the proof affects check-in, eligibility, or refusal. | Redact document images, OCR text, vet/customer addresses, phone/email, signatures, medical notes, and file URLs in logs; use document IDs, pet/reservation IDs, extracted field labels, confidence/uncertainty, and reviewer IDs. |
| Messaging / customer communications | Customer contact preferences and channel consent; approved templates/SOP snippets; subject-specific facts already verified by the owning workflow; conversation history for the same thread; prior outbound send/audit status; suppression/escalation flags. | Message drafts; tone/safety flags; fact-check checklist; recipient/channel suggestions; suppression reason; review queue item; send audit request packet. Direct send only through separately approved deterministic send service, never from model runtime. | Bypass opt-out/contact preferences; send without approved path; invent facts; hide concerning facts; make medical/legal/payment/refund/availability promises; publish public responses; delete or edit message history; contact unrelated recipients. | Any health, medication, allergy, behavior, incident, safety, legal, payment, refund, eligibility, refusal, complaint, bad-review, policy-exception, or ambiguous-fact message; template/policy changes; new automation trigger approval. | Low-risk routine operational drafts can be `QueueForReview`; approved deterministic templates may be `DeterministicAutoSendOnly`; sensitive threads are `DraftOnly` or `NeverAutoSend`. | Redact recipient address/phone except channel and last-4/contact alias where needed; redact full bodies in system logs, keeping message ID, template ID, risk class, review gate, and send/suppression status; never log secrets or provider payloads. |
| Incident / escalation | Incident report; pet/customer/reservation/stay context; staff observations; care/medical/behavior history needed for the incident; related photos/media metadata only when required; prior incident/escalation audit; applicable SOP and emergency-contact/vet context. | Incident summary; severity/risk flags; missing-field checklist; manager/lead/vet/customer review tasks; draft owner/manager packets; care watchlist/status suggestions; audit/workflow result records. No closure or external notification authority. | Close or downgrade incidents; diagnose or provide veterinary/legal conclusions; alter care/behavior truth; suppress required escalation; assign blame/admit fault; send owner/legal/public messages without manager approval; delete media/notes. | Severity classification with operational consequence; owner notification; vet/emergency contact outreach; playgroup suspension/reinstatement; refunds/credits/waivers tied to incident; legal/compliance/privacy escalation; public review response. | Incident-related customer/public messages are `NeverAutoSend` unless a manager explicitly approves final text and send action; internal manager packet only may be automatic. | Redact injury/medical detail, staff names, customer PII, photos/videos, witness statements, and exact location within facility except in authorized review packets; log incident ID, risk category, required gate, source refs, and escalation status. |
| Staff operations / daily care notes | Operating-day snapshot; arrivals/departures; staff-task queues; reservation/stay status; pet care profile; feeding/medication/allergy/medical/temperament data needed for assigned tasks; capacity/labor summaries; approved policy refs; task/audit history. | Daily brief sections; care/handoff summaries; task recommendations/drafts; blocked/manager-review flags; daily/Pawgress update drafts; status suggestions; care-watchlist suggestions; audit/workflow result records. AI must not mark care complete or mutate schedules/capacity. | Complete care/safety/medication/feeding tasks; infer executable medication instructions; change staff schedules; assign capacity/rooms as authority; return rooms to sellable inventory; decide group-play eligibility; send daily updates autonomously; hide concerning observations. | Medication/feeding/medical/allergy ambiguity; behavior/playgroup assignment or reinstatement; capacity/ratio/staffing exceptions; room release/checkout readiness; task auto-generation at scale; staff schedule changes; sensitive daily update language. | Warm routine daily-update drafts are `QueueForReview`; messages mentioning health, medication, allergy, behavior, incident, safety, policy exception, payment, or ambiguous facts are `DraftOnly`/`NeverAutoSend`. | Redact care instructions, medication names/doses/schedules, allergies, medical conditions, behavior/staff notes, staff identifiers, photos/media, and exact kennel/camera details outside authorized care packets; log task IDs, status/risk classes, due/review gates, and source refs. |

## Cross-workflow data classes

| Data class | Default access | Notes |
| --- | --- | --- |
| Core IDs and routing context | Allowed when scoped to location, subject, event, and workflow. | Prefer `LocationId`, `CustomerId`, `PetId`, `ReservationId`, `WorkflowEventId`, task IDs, document IDs, policy snapshot IDs. |
| Customer contact PII | Minimized; only workflows that draft/contact customers may read needed channel/preference fields. | Redact in logs and prompt traces unless the recipient/channel is required for review. |
| Pet medical/care/behavior | Need-to-know by document, incident, staff-care, and booking eligibility workflows. | Never expose broadly to messaging/intake except as approved, minimal customer-safe wording. |
| Vaccine documents/OCR | Document/vaccine workflow only by default; booking reads verified status. | Raw images/OCR are high sensitivity; use document refs elsewhere. |
| Payment/deposit/billing | Booking/payment-sensitive workflows read semantic status only unless reconciliation requires more. | Raw provider payloads/secrets stay behind adapters; agents cannot move money. |
| Message history | Same subject/thread/channel only. | Cross-thread/customer history requires explicit rationale and approval. |
| Audit logs | Read source/result/review refs needed for dedupe and traceability. | Do not expose raw PII or secrets from audit metadata to agents; use safe metadata. |
| Media/camera/photos | Incident/staff-care only when necessary and explicitly scoped. | Log media IDs and labels, not images/URLs, outside authorized review. |
| Staff/labor/schedule | Staff operations and manager brief only as summaries unless scheduling workflow is approved. | Agents may flag risk; they may not change schedules. |

## Never-direct changes

No agent type may directly execute these actions before explicit implementation/security/product approval:

- Live Gingr/provider writes, including reservation status changes, check-in/out, owner/pet record mutation, vaccine approval, invoice/payment changes, document deletion, or outbound messaging.
- Payment capture, retry, void, refund, waiver, discount, credit, forfeiture, write-off, rate/tax/fee changes, or policy-snapshot edits.
- Final booking acceptance/rejection/cancellation/waitlist release, capacity/room allocation, room return to inventory, or overbooking/ratio exceptions.
- Final medical/vaccine/eligibility/playgroup/incident/safety decisions.
- Staff schedule changes, payroll/timeclock changes, or final care-task completion.
- Deleting, suppressing, or materially editing audit logs, source records, uploaded documents, care notes, messages, or incident records.
- Customer/public message sends outside approved deterministic send paths and required human gates.

## Implementation requirements

1. Represent permissions as named capabilities, not ad-hoc prompt text: read scopes, output action scopes, forbidden actions, review gates, customer-message automation level, and redaction profile.
2. Bind each agent run to a `PolicyContext` containing allowed actions, `AutomationLevel`, required reviews, location, subject, source snapshot IDs, and policy version/ref.
3. Enforce data minimization before prompt construction. A worker should never receive a field it would be forbidden to log or reason over for that workflow.
4. Validate agent output against allowed actions. Discard or convert disallowed outputs into `RejectedByPolicy` or `NeedsHumanReview`; do not silently execute them.
5. Require idempotency/dedupe keys for task drafts, message drafts, status suggestions, and provider action requests.
6. Persist audit/workflow events for agent inputs, outputs, validator decisions, human approvals/rejections, sends/actions, redactions applied, and failures.
7. Treat permission changes and customer-message automation changes as approval-gated configuration releases with security review, test fixtures, rollback plan, and audit trail.

## Open approval gates

- Final workflow-specific read scopes and tool names.
- Which task drafts may become auto-created internal tasks and under what dedupe/rate limits.
- Which customer-message categories, if any, may use deterministic auto-send paths.
- Which staff/manager roles can approve each review gate and whether approval may be delegated by location.
- Provider/Gingr write-back mode for MVP: read-only import, copy/paste assist, approved write-back, or no integration.
