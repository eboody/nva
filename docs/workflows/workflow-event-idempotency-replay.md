# Workflow event idempotency and replay rules

Purpose: define a recommended idempotency and replay model for the MVP workflow events in `domain/src/workflow.rs`. This artifact is a design recommendation, not an approved implementation contract.

Status: approval-gated design draft. The idempotency model itself requires human/reviewer approval before implementation. Do not wire live provider writes, outbound customer messages, payment actions, reservation mutations, or authoritative task creation from this document until the model is explicitly approved and covered by tests.

## Recommendation summary

Use a durable workflow inbox/outbox model with two distinct keys:

1. `source_event_key` deduplicates ingestion of the same external or internal fact.
2. `side_effect_key` deduplicates each resulting effect such as task creation, draft creation, provider write, or outbound customer message.

Recommended key shape:

```text
source_event_key = v1:{location_id}:{event_type}:{primary_subject}:{source_kind}:{source_fingerprint}
side_effect_key = v1:{location_id}:{event_id}:{effect_kind}:{effect_subject}:{effect_intent}:{approval_or_policy_version}
```

Where:

- `location_id` is required on every `WorkflowEvent` and prevents cross-location duplicate suppression.
- `event_type` is the semantic `WorkflowEventType`, not a raw provider event name.
- `primary_subject` follows `WorkflowSubject`: reservation for stay/booking/check-in/out workflows, pet for pet-profile/vaccine/care/incident workflows, customer for account/message/loyalty/membership workflows, and external for unmapped provider records.
- `source_kind` identifies where the fact came from: `provider_webhook`, `provider_read_reconciliation`, `staff_action`, `customer_portal`, `scheduled_policy_eval`, `system_backfill`, or `manual_import`.
- `source_fingerprint` is the most stable upstream identity available. Prefer a verified provider delivery id if one exists; otherwise use a semantic tuple such as provider + provider event type + normalized entity id + provider updated timestamp/version. If no upstream version exists, use a canonical hash of the minimal verified source fields that define the fact, not the whole raw payload.
- `effect_kind` is one of `internal_task`, `draft_customer_message`, `provider_write`, `outbound_customer_message`, `audit_record`, or `review_request`.
- `effect_subject` is the concrete record affected by the effect: task kind + domain subject, draft id, provider entity id, customer/channel/message template, or audit subject.
- `effect_intent` describes the semantic action, for example `document_review_needed`, `daily_update_draft`, `suggest_checked_in`, `send_deposit_reminder`, or `mark_provider_reservation_checked_out`.
- `approval_or_policy_version` is required for effects whose legality or wording depends on approval or policy. A new approval version may intentionally produce a new effect; a replay of the same approval must not.

This two-key model intentionally treats inbound duplicate suppression and outbound exactly-once safety as separate concerns. Provider/webhook retries and backfills are common; they should not create duplicate tasks or messages. Conversely, two different source events can legitimately converge on one staff task or one message draft, so side-effect keys must be checked independently.

## Global invariants

1. Ingest before acting.
   - A verified event is durably stored in an inbox with `source_event_key`, canonical event fields, source metadata, processing status, and audit reference before any side effect is attempted.
   - Asynchronous receivers may return provider success only after durable acceptance, not after JSON parsing alone.

2. Raw provider payloads do not drive idempotency directly.
   - Provider webhooks must pass signature/verification and semantic mapping first.
   - Raw JSON field order, incidental HTML, or undocumented payload extras must not change the source key.
   - The raw payload can be retained for audit/quarantine, but the key uses normalized verified facts.

3. Duplicate source events are acknowledged as duplicates, not reprocessed blindly.
   - If the same `source_event_key` arrives with the same canonical fingerprint, record a duplicate-observed audit row and return the already-known processing status.
   - If the same key arrives with a conflicting canonical fingerprint, do not overwrite prior truth. Create a reconciliation/review event and suppress customer/provider side effects until resolved.

4. Side effects are idempotent and claim-before-execute.
   - Before creating a staff task, customer-message draft, provider write, outbound send, or audit materialization, insert or claim the `side_effect_key` in a durable outbox/effect ledger.
   - If the key already exists as `succeeded`, return the prior effect reference.
   - If the key exists as `in_progress` and is not expired, do not execute again.
   - If the key exists as `failed_retryable`, retry only the same effect with the same key and payload unless a new approval/policy version intentionally creates a replacement effect.
   - If the key exists as `failed_permanent` or `blocked`, require human/system remediation before another attempt.

5. Replays are deterministic by version.
   - Reprocessing the same inbox event with the same policy version and approvals must produce the same recommendations and the same side-effect keys.
   - Replaying under a newer policy version is a new evaluation but must not send, write, or close anything without new side-effect keys and required approvals.

6. Effects are append-only and externally referenced.
   - Store provider request id/response id, task id, draft id, message id, approval id, and audit event id on the effect ledger.
   - Never infer success only from local process completion. For provider writes and outbound sends, record the external provider/message reference or leave the effect in `unknown_needs_reconciliation`.

7. Customer messages have two gates.
   - Draft creation is idempotent by draft intent and source facts.
   - Actual outbound send is a separate effect keyed by approved draft/version, recipient, channel, template/copy version, and send intent. Replaying a draft event must not send. Replaying a send attempt with the same key must return the already-sent message reference or reconcile unknown status before retry.

8. Provider writes have approval and reconciliation gates.
   - Provider mutations are never direct consequences of workflow events in the MVP. They require an approved executable action, a provider tool boundary, a side-effect key, and reconciliation of the provider response.
   - Retrying an unknown provider write must first check provider state or use a provider-supported idempotency key if available. If provider state cannot be checked, block for human reconciliation.

9. Task creation is safe but not unbounded.
   - Internal task creation may be allowed when policy approves the task trigger, but duplicate suppression must be by task semantic intent, not by source event id alone.
   - Multiple source events that identify the same unresolved work should attach evidence to one task or update its source list, not create task spam.

10. Audit records preserve both suppression and action.
    - Log first-seen event, duplicate observed, replay requested, side effect claimed, side effect succeeded/failed/blocked, conflict detected, and human approval transitions.
    - Duplicate suppression is itself an auditable outcome.

## Replay behavior classes

| Replay class | Meaning | Allowed behavior | Forbidden behavior |
| --- | --- | --- | --- |
| Source duplicate | Same source fact delivered again, usually provider retry or manual duplicate import | Return existing inbox/effect status; append duplicate audit observation | Create duplicate task, draft, provider write, or customer send |
| Deterministic recompute | Same event, same policy version, same approvals | Recompute recommendations and verify same side-effect keys; repair missing internal read models if safe | Change customer/provider-visible outcome or create new effect key |
| Retryable effect retry | Prior effect failed before confirmed success | Retry the exact same side-effect key and payload after checking lease/unknown status | Create a replacement effect just because the worker crashed |
| Unknown external status reconciliation | Worker crashed or timed out after provider send/write request and before success was recorded | Query provider/message system by side-effect key, external request id, recipient/template, or provider state; mark succeeded if already applied | Blindly send/write again |
| Policy-version replay | Human/system intentionally re-evaluates old event under a new approved policy version | Produce new recommendations/effects only where policy version changes the authorized action | Treat new policy as retroactive authority for live sends/writes without required approval |
| Backfill/import replay | Historical events are loaded to build read models or tasks | Default to read-model/audit rebuild and review packets; side effects disabled unless explicitly approved for backfill | Send old customer messages, write old provider state, or create current staff tasks from stale events without a backfill policy |

## Per-event idempotency table

| MVP event type | Primary subject and source key shape | Duplicate handling | Replay behavior | Retry and side-effect safety |
| --- | --- | --- | --- | --- |
| `InquiryReceived` | Subject: `Customer` when known, otherwise `External`. Key: `location + InquiryReceived + customer/external lead id + source_kind + lead/provider event id or normalized contact+submitted_at hash`. | Merge duplicate inquiry evidence into one lead/review packet when contact + source + submitted timestamp match. Conflicting contact/profile fields create review instead of overwrite. | Replays may rebuild lead summary and missing-info recommendation. Backfills must not send conversion messages unless campaign/send approval exists. | Internal lead/follow-up task key: `internal_task:customer/external:inquiry_follow_up:{policy_version}`. Draft message key includes template/copy version. Outbound customer send has separate approved-send key, so duplicate lead webhooks cannot double-message a customer. |
| `CustomerRegistered` | Subject: `Customer`. Key: `location + CustomerRegistered + customer_id/provider_owner_id + source_kind + provider owner created/updated version`. | Same owner-created event is suppressed. Owner-created plus owner-edited with new version is a new source event but should update one customer reconciliation packet. | Replays may rebuild customer profile completeness and portal-readiness tasks. They must not resend welcome/onboarding messages from registration alone. | Task key: `internal_task:customer:customer_profile_review`. Draft key: `draft_customer_message:customer:welcome_or_missing_info:{template_version}`. Send key requires approved draft/version + recipient/channel; replay returns prior message id. |
| `PetProfileCreated` | Subject: `Pet`. Key: `location + PetProfileCreated + pet_id/provider_animal_id + source_kind + provider animal created/updated version`. | Suppress exact duplicate profile creation. If duplicate source contains changed care/medical/behavior fields, route to profile reconciliation instead of replacing verified facts. | Replays may rebuild pet-profile completeness/readiness state and suggest document/care-review tasks. They must not infer eligibility/group-play approval. | Task key: `internal_task:pet:pet_profile_review` or task-kind-specific key. Completion requires staff evidence; AI replay cannot close task. No provider write is allowed from this event without an approved mutation action. |
| `VaccineDocumentUploaded` | Subject: `Pet`. Key: `location + VaccineDocumentUploaded + pet_id + document_id/provider_file_id + source_kind + uploaded_at/version or content hash`. | Exact duplicate upload attaches no new task. Same file uploaded twice may link duplicate document evidence to one document-review task. Different documents create distinct review evidence. | Replays may rerun OCR/extraction suggestions and rebuild a vaccine-review packet. They must not mark vaccines verified or eligibility approved. | OCR/extraction side effects use idempotent document-processing keys. Document-review task key: `internal_task:pet:document_review:{document_id}`. Provider writes and customer messages are disallowed unless a later approved workflow requests them. |
| `BookingRequested` | Subject: `Reservation`. Key: `location + BookingRequested + reservation_id/provider_reservation_id + source_kind + request version/submitted_at hash`. | Duplicate request delivery must not create another reservation-triage task or another customer acknowledgement. Conflicting dates/services/pets create reservation reconciliation/manager review. | Replays may recompute readiness, missing-info, deposit-policy, capacity, and review recommendations from the same policy version. New policy replay may change recommendations but not mutate provider state. | Task keys are per unresolved intent: `reservation_triage`, `missing_vaccine`, `deposit_review`, `capacity_review`. Draft acknowledgement/reminder keys are separate. Provider status writes such as offer/confirm/reject require approved executable action and provider-write key. |
| `BookingTriageNeeded` | Subject: `Reservation`. Key: `location + BookingTriageNeeded + reservation_id + triage_reason_set_hash + policy_version`. | Same triage reason set maps to one open triage task/review packet. New reasons append evidence or create subtask only if task-kind policy approves. | Replays may rebuild triage packet and verify task state. They must not multiply open tasks when an unresolved one exists. | Internal task key: `internal_task:reservation:booking_triage:{reason_set_hash}:{policy_version}`. Provider writes and customer sends remain blocked; draft messages require customer-message approval before send. |
| `BookingConfirmationNeeded` | Subject: `Reservation`. Key: `location + BookingConfirmationNeeded + reservation_id + readiness_snapshot_hash + policy_version`. | Duplicate confirmation-needed signals update the same confirmation review packet if readiness facts are unchanged. Conflicts such as capacity/payment/vaccine ambiguity block confirmation and create review. | Replays may recompute readiness. Same approval replay must not confirm twice; new approval/policy creates a new provider-write candidate. | Provider confirmation key: `provider_write:reservation:confirm:{approval_id}:{policy_version}`. If prior confirm status is unknown, reconcile provider reservation status before retry. Customer confirmation send key is separate and requires approved copy/recipient/channel. |
| `DailyNoteCreated` | Subject: `Reservation` when stay-scoped, else `Pet`. Key: `location + DailyNoteCreated + reservation_id/pet_id + note_id/provider_note_id + source_kind + note created/updated version`. | Exact duplicate note is suppressed. Edited note with a new version is a distinct source event and should update the daily-update evidence set, not create duplicate report tasks. | Replays may rebuild approved-evidence summaries and draft updates. They must not send Pawgress/daily updates from note creation alone. | Draft key: `draft_customer_message:reservation:daily_update:{evidence_set_hash}:{template_version}`. Send key: `outbound_customer_message:reservation:daily_update:{approved_draft_id}:{recipient}:{channel}`. Duplicate notes cannot double-send because send depends on approved draft id/version. |
| `DailyUpdateNeeded` | Subject: `Reservation`. Key: `location + DailyUpdateNeeded + reservation_id + service_day + update_window + policy_version`. | Multiple triggers for the same reservation/day/window converge on one daily-update draft/review task. | Replays may regenerate the same draft if unsent or compare current draft to evidence. Backfills must not generate stale customer sends. | Task key: `internal_task:reservation:daily_update_review:{service_day}:{window}`. Draft and send keys are separate. If send attempt status is unknown, check message provider/outbox before retrying. |
| `IncidentCreated` | Subject: `Pet` and linked `Reservation` when available. Key: `location + IncidentCreated + incident_id/provider_incident_id + source_kind + incident version`. | Exact duplicate incident event is suppressed. Incident edits/updates should create new evidence versions but not duplicate customer or manager tasks if one incident-review task is open. | Replays may rebuild incident packet, manager/care review state, and suppressed-message reason. They must not send customer explanation or update pet eligibility automatically. | Manager/care review task key: `internal_task:pet_or_reservation:incident_review:{incident_id}`. Customer incident explanation draft requires manager gate; outbound send key requires approved copy/version. Provider writes for eligibility/status require separate approved action. |
| `CheckoutCompleted` | Subject: `Reservation`. Key: `location + CheckoutCompleted + reservation_id/provider_reservation_id + source_kind + checkout event/version/timestamp`. | Duplicate checkout events must not create duplicate cleaning turnover, payment follow-up, review request, or thank-you sends. Conflicting invoice/balance/provider status creates reconciliation. | Replays may rebuild checkout readiness/summary, turnover need, billing reconciliation, and review-request eligibility. They must not mark provider checkout, charge/refund, or message customer unless separately approved. | Cleaning task key: `internal_task:reservation:cleaning_turnover:{accommodation_or_reservation}`. Billing task key: `internal_task:reservation:checkout_balance_review`. Review/thank-you send keys depend on approved eligibility/copy. Unknown provider checkout writes require provider state reconciliation before retry. |
| `ReviewRequestEligible` | Subject: `Customer` or `Reservation`. Key: `location + ReviewRequestEligible + customer/reservation + eligibility_snapshot_hash + policy_version`. | Duplicate eligibility signals produce at most one active review-request draft/send candidate per stay/customer/campaign window. | Replays may re-check suppression rules such as incident, complaint, opt-out, or already-sent campaign. They must not send if eligibility is stale or suppression facts changed. | Draft key: `draft_customer_message:customer:review_request:{campaign_version}:{reservation_id}`. Send key includes approved draft id, recipient/channel, campaign version, and suppression-check timestamp/version. Duplicate checkout/review events cannot double-send. |
| `MembershipChanged` | Subject: `Customer`. Key: `location + MembershipChanged + customer_id + membership/account id + source_kind + membership version/effective_at`. | Same membership version is suppressed. Conflicting provider/customer claims create membership reconciliation instead of changing billing/benefit truth. | Replays may rebuild benefits summary and staff review tasks. They must not grant credit, alter billing, or send membership promises without approved policy. | Internal task key: `internal_task:customer:membership_review:{membership_id}`. Provider writes/benefit adjustments require approved executable action. Customer-message send key uses approved template/copy and membership policy version. |
| `LoyaltyCreditAvailable` | Subject: `Customer`. Key: `location + LoyaltyCreditAvailable + customer_id + credit/ledger id + source_kind + credit version`. | Duplicate credit availability does not create duplicate reminders or apply credit twice. Conflicting amount/currency/expiry creates loyalty/payment reconciliation. | Replays may rebuild staff/customer draft explanation from trusted ledger state. Backfills must not notify customers about expired/old credits unless campaign policy approves it. | Applying credit is a provider/payment-sensitive write and needs approval plus `provider_write:customer_or_invoice:apply_loyalty_credit:{approval_id}:{credit_id}`. Outbound reminder key includes approved draft, recipient/channel, campaign/policy version. |

## Side-effect-specific rules

### Internal staff tasks

Recommended task idempotency key:

```text
v1:{location_id}:internal_task:{task_kind}:{domain_subject}:{semantic_reason}:{policy_version}
```

Rules:

- If an open task with the same key exists, attach the new event/evidence reference instead of creating another task.
- If a task with the same key is completed, replay should not reopen it unless a new source version or policy version creates a materially new obligation.
- AI/workflow replay must never complete a task. Completion requires authorized staff/manager evidence.
- Task creation at scale remains approval-gated by task-kind policy.

Safe duplicate suppression example:

```text
Two `VaccineDocumentUploaded` webhooks for document D-17 arrive.
The inbox stores one source event and one duplicate observation.
The outbox sees `internal_task:DocumentReview:pet P-9:document D-17` already open.
Result: one document-review task with two source observations, no duplicate staff work.
```

### Customer-message drafts

Recommended draft key:

```text
v1:{location_id}:draft_customer_message:{message_intent}:{customer_id}:{reservation_or_pet}:{evidence_set_hash}:{template_or_copy_version}:{policy_version}
```

Rules:

- Drafts can be regenerated idempotently from the same evidence/policy/template version.
- If evidence changes, create a new draft version or mark the old draft stale; do not mutate a sent draft's historical content.
- Draft creation does not imply send approval.

Safe replay example:

```text
`DailyUpdateNeeded` for reservation R-44/day 2026-06-12 is replayed after a worker crash.
The same evidence set and template version produce the same draft key.
The existing draft D-31 is returned and no customer message is sent.
```

### Outbound customer messages

Recommended send key:

```text
v1:{location_id}:outbound_customer_message:{approved_draft_id}:{approved_draft_version}:{recipient_id}:{channel}:{send_intent}:{approval_id}
```

Rules:

- Sending is a separate outbox effect from drafting.
- A duplicate send request with the same key returns the prior message provider id.
- If status is unknown after timeout/crash, reconcile with the message provider/outbox before retrying.
- Never use event replay as authority to send; require approval id or separately approved deterministic send policy.

No-double-message example:

```text
A staff member approves daily update draft D-31 for SMS to customer C-12.
The send worker claims `outbound_customer_message:D-31:v2:C-12:sms:daily_update:approval A-7`.
The process crashes after the SMS provider accepts the message but before local success is saved.
Retry first queries by provider request id/outbox key. If SMS M-88 already exists, mark succeeded with M-88 and do not send again.
```

### Provider writes

Recommended provider-write key:

```text
v1:{location_id}:provider_write:{provider}:{provider_entity_type}:{provider_entity_id}:{write_intent}:{approval_id}:{policy_version}
```

Rules:

- Provider writes are disabled for MVP workflow events unless a separate approved executable-action path exists.
- Use provider-native idempotency keys when available; otherwise store the local side-effect key and reconcile provider state before retry.
- Unknown external status blocks blind retry for non-idempotent operations such as reservation confirmation, checkout, refund, waiver, discount, credit application, or customer communication.
- Provider write payloads must be built from approved semantic state, not raw provider payload replay.

No-double-provider-write example:

```text
`BookingConfirmationNeeded` produces an approved provider-write candidate to confirm reservation G-76390.
The provider-write key is tied to approval A-42 and policy P-5.
A retry sees the key in `unknown_needs_reconciliation`; it reads Gingr/provider reservation status before issuing another write.
If the reservation is already confirmed by the prior attempt, the effect is marked succeeded with the provider status reference and no second confirm call is made.
```

### Payment, loyalty, refund, waiver, discount, and credit effects

Rules:

- Treat money movement and financial account mutation as provider writes with stricter reconciliation.
- Customer claims, screenshots, staff free text, and unverified webhooks are not payment truth.
- Duplicate cancellation, checkout, loyalty, or payment-related events must not double-count history, apply credit twice, forfeit twice, refund twice, or send duplicate payment messages.
- Unknown status after payment-provider call requires payment/provider reconciliation and usually manager review.

## Event-state storage recommendation

Minimum durable tables/ledgers implied by this model:

| Store | Key | Purpose |
| --- | --- | --- |
| `workflow_event_inbox` | `source_event_key` unique | Durable acceptance of verified semantic events and duplicate/conflict observations. |
| `workflow_event_processing_run` | `event_id + processor_version + policy_version` | Tracks attempts, deterministic replay, failures, and backfill runs. |
| `workflow_side_effect_ledger` | `side_effect_key` unique | Claim-before-execute ledger for tasks, drafts, sends, provider writes, and materialized audit/review effects. |
| `workflow_effect_attempt` | `side_effect_key + attempt_number` | Records retries, leases, provider request/response references, unknown status, and reconciliation outcomes. |
| `workflow_replay_request` | replay id | Records who/what requested replay, reason, policy version, dry-run vs execute mode, and side-effect enablement. |

Recommended statuses:

- Inbox: `accepted`, `duplicate_observed`, `conflict_needs_review`, `rejected_permanent`, `processing`, `processed`, `failed_retryable`, `failed_permanent`.
- Side effect: `planned`, `claimed`, `in_progress`, `succeeded`, `failed_retryable`, `failed_permanent`, `blocked_for_approval`, `unknown_needs_reconciliation`, `suppressed_duplicate`, `cancelled_by_policy`.

## Approval and tradeoffs

Recommended approval decision:

Adopt the two-key inbox/outbox model before implementing workflow event processing. It is the safest MVP choice because it handles provider retries, worker crashes, backfills, and human-approved execution without mixing duplicate source events with duplicate side effects.

Tradeoffs:

- More storage and implementation complexity than a single `event_id` unique constraint.
- Requires discipline around canonical source fingerprints and semantic effect intents.
- Requires reconciliation APIs or operational procedures for unknown provider/message status.
- Policy-version and approval-version keys make behavior auditable but add migration/backfill work when policy changes.
- Hashing minimal verified source facts avoids raw-payload instability, but poor canonicalization can either suppress real changes or create duplicate work. This must be covered by tests per event type.

Alternatives rejected for MVP:

- `event_id` only: too weak because duplicate provider deliveries may receive different local ids, and one event can produce several side effects.
- Raw payload hash only: unstable across provider formatting changes and unsafe before semantic verification.
- Provider entity id only: too coarse; owner/pet/reservation records can have multiple meaningful event versions and effect intents.
- Exactly-once worker assumption: unrealistic under webhooks, retries, crashes, and asynchronous provider/message calls.

Approval gate before implementation:

A human/reviewer must approve:

1. The canonical `source_event_key` fields for each actual provider/source adapter.
2. The side-effect key dimensions for task creation, drafts, outbound sends, and provider writes.
3. Which event types may create live internal tasks automatically versus drafts/review packets only.
4. The reconciliation procedure for unknown provider-write and outbound-message statuses.
5. Test fixtures proving duplicate suppression, replay, retry, and no-double-send/no-double-write behavior for each MVP event type.
