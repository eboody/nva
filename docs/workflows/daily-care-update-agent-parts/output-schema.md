# Daily care update agent output schema

Purpose: define the workflow-specific structured output schema for the daily-care update agent. This artifact is intended to feed a later complete daily-care update workflow definition. It does not authorize autonomous customer sends, provider/PMS writes, media publishing, care-task completion, incident communication, medical advice, payment action, or policy exceptions.

Status: draft schema contract. It is aligned to the canonical input packet in `docs/workflows/daily-care-update-agent-parts/inputs.md`, the current `WorkflowEventType::DailyNoteCreated` and `DailyUpdateNeeded` anchors, and the current `WorkflowResult<T>`/typed prompt-packet posture described by the parent handoff. Low-risk auto-send is modeled as a future policy state; until an approved send policy exists, production callers should validate `should_send=false` or require an approval override before delivery.

## Source anchors

Use these repo-local sources and handoff constraints as the canonical basis for this schema:

- `docs/workflows/daily-care-update-agent-parts/inputs.md` — canonical daily-care update inputs, evidence requirements, missing policy caveats, output validation/escalation rules, and conservative downstream rule.
- `docs/workflows/workflow-event-idempotency-replay.md` — idempotency and replay expectations for `DailyNoteCreated` and `DailyUpdateNeeded`.
- `docs/domain/petsuites/boarding/service-domain-map.md` — Pawgress report, photo/video update add-on, boarding care/report surfaces, and approval-state vocabulary.
- `docs/domain/petsuites/boarding/implications/06-medication-feeding-behavior-notes.md` — medication, feeding, behavior, care-review, staff-note, handoff, and Pawgress/customer-message review triggers.
- `docs/domain/petsuites/daycare/implications/05-pet-health-behavior-notes.md` — daycare health/behavior note source, customer-safe projection, review-state, and daily-care-update participation.
- `docs/domain/petsuites/daycare/implications/04-incident-tracking.md` — incident, media, customer-notice, and safety review boundaries.
- `docs/workflows/payments-pricing.md` and `docs/workflows/payments-pricing-parts/ai-boundaries.md` — payment-sensitive/customer-facing message constraints that daily updates must not cross.
- `domain/src/entities.rs`, `domain/src/care.rs`, `domain/src/operations.rs`, `domain/src/workflow.rs`, and `domain/src/tools.rs` — current Rust anchors for pets, reservations, care profiles, staff tasks, daily update drafts, workflow events/results, message drafting, and media refs.

Known source gaps to preserve explicitly:

- No approved standalone photo/privacy/consent policy exists yet.
- No approved daily-update cadence policy, per-location Pawgress/update requirement policy, or auto-send policy exists yet.
- No approved customer-message tone/template/signature/staff-initials policy exists yet.
- No first-class implemented `CareNote` aggregate exists in `domain/src`; use the parent handoff lifecycle states and current task/workflow/message surfaces until the data model is implemented.
- No typed meal amount/status, bathroom/elimination value, photo review state, photo consent state, poor-photo reason, or daily-update approval state exists as a dedicated domain type today.

## Envelope placement

The agent output should be embedded as the `structured_output` payload of a `WorkflowResult<DailyCareUpdateOutput>` style result.

Minimum envelope expectations:

```json
{
  "schema_name": "DailyCareUpdateOutput",
  "schema_version": "2026-06-11",
  "workflow_name": "daily-care-update",
  "event_id": "workflow_event_id",
  "subject": { "type": "reservation_day_window", "id": "reservation_id:2026-06-11:pm" },
  "status": "completed | needs_human_review | needs_more_information | failed_safely | rejected_by_policy",
  "summary": "Operator-safe summary of the draft/update decision.",
  "structured_output": { "...": "DailyCareUpdateOutput" },
  "recommended_actions": [],
  "risk_flags": [],
  "verification": [],
  "human_review_reason": null
}
```

Envelope mapping rules:

- `status=completed` means the schema was produced and validated. It does not mean the customer message was sent.
- `status=needs_human_review` must be used when `structured_output.requires_review=true`.
- `status=needs_more_information` should be used when required evidence is missing and no safe customer draft should be produced.
- `status=rejected_by_policy` should be used for deterministic policy denial, suppressed sends, opt-out, missing consent where required, or a no-send update window.
- `status=failed_safely` should be used for parsing/validation failure, unsafe output, contradictory evidence that cannot be represented safely, or runtime/tool failure. No side effects are allowed.

## Top-level structured output

Required top-level object:

```json
{
  "customer_message": {
    "body": "Milo had a relaxed afternoon and enjoyed his individual play session. He ate dinner as expected, and the team will continue following his regular care plan.",
    "channel_hint": "portal | sms | email | printed_report | unknown",
    "template_id": null,
    "language": "en-US",
    "tone": "warm_concise_factual",
    "media_refs": [],
    "audience": "customer",
    "redaction_profile": "customer_safe_daily_update_v1"
  },
  "internal_flags": [],
  "should_send": false,
  "requires_review": true,
  "review_reason": "customer_message_approval_not_configured",
  "included_facts": [],
  "omitted_facts": []
}
```

All seven top-level fields are required on every successful parse. Use empty arrays where there are no flags/facts; use `customer_message.body=null` for do-not-send cases instead of omitting `customer_message`.

## Required fields

### 1. `customer_message`

`customer_message` is the proposed customer-facing daily update, or an explicit null-body message object when the correct outcome is no customer draft.

Type:

```ts
type CustomerMessageDraft = {
  body: string | null;
  channel_hint: "portal" | "sms" | "email" | "printed_report" | "unknown";
  template_id: string | null;
  language: string;
  tone: "warm_concise_factual" | "neutral_staff_review" | "none";
  media_refs: string[];
  audience: "customer";
  redaction_profile: string;
};
```

Validation rules:

- `customer_message` is required and must be an object.
- `body` may be `null` only when `should_send=false` and either `requires_review=true` or a do-not-send policy state applies.
- Non-null `body` must be concise, customer-safe, and source-grounded. Recommended validation range: 1 to 900 characters for portal/email/printed report; SMS callers may apply a stricter deterministic channel limit before sending.
- Non-null `body` must not contain raw internal notes, provider payloads, medical diagnosis/advice, legal/liability language, payment details, availability/booking promises, unsupported claims, other customers, other pets, unapproved staff identifiers, or unapproved policy explanations.
- Every factual sentence or media mention in `body` must trace to at least one `included_facts[]` item whose `customer_visibility` is `customer_safe_after_review` or `customer_facing_approved`. If `customer_visibility=customer_safe_after_review`, the output must set `requires_review=true` unless deterministic policy has already recorded an approval ref.
- `media_refs[]` must contain only approved media/document refs, never raw blobs or direct camera snapshots. If the message says or implies that a photo is attached, at least one `included_facts[]` item of `fact_kind=photo_media` must reference the same media ref and have approved consent/review state.
- `template_id` is nullable until templates are approved. When non-null, it must refer to an approved template version in policy context; the model must not invent template IDs.
- `redaction_profile` must name the deterministic redaction/minimization profile used by the prompt/output validator. Do not include a free-form redaction policy in customer copy.

Customer-message content rules:

- Allowed routine content, when source-backed and policy-safe: meal/feeding status, play/enrichment participation, mood/demeanor, rest, bathroom/elimination status, general care-plan adherence, and approved photo/media mention.
- Sensitive or review-gated content: medication details/exceptions, feeding refusal or medically significant appetite changes, behavior restrictions, anxiety/stress flags, incidents, injuries, illness symptoms, sanitation/safety concerns, customer complaints, policy exceptions, payment/billing, eligibility, or conflicting/missing evidence.
- The draft may use warm phrasing, but it must not invent cheerful filler. If only sparse facts exist, write a sparse factual draft or require review; do not fabricate activities or emotions.

### 2. `internal_flags`

`internal_flags` is the machine-readable reason set for review, suppression, audit, and downstream task creation. It is not shown to customers.

Type:

```ts
type InternalFlag = {
  code: string;
  severity: "info" | "needs_staff_review" | "needs_manager_review" | "do_not_send" | "runtime_error";
  message: string;
  source_refs: string[];
  recommended_action: "none" | "staff_review" | "manager_review" | "collect_more_info" | "replace_or_review_photo" | "create_internal_task" | "suppress_update" | "dead_letter";
};
```

Recommended `code` values:

- `customer_message_approval_not_configured`
- `missing_required_update_evidence`
- `missing_or_unapproved_photo`
- `poor_or_sensitive_photo`
- `photo_permission_denied`
- `photo_retention_expired`
- `photo_conflicts_with_text`
- `incident_or_safety_signal`
- `medical_or_medication_review_required`
- `feeding_exception_review_required`
- `behavior_review_required`
- `raw_internal_note_not_customer_safe`
- `unverified_provider_or_customer_claim`
- `conflicting_staff_notes`
- `care_task_completion_not_verified`
- `policy_gap_requires_review`
- `opt_out_or_suppression`
- `duplicate_or_replay_no_new_send`
- `validation_failed_safe`

Validation rules:

- `internal_flags` is required and must be an array.
- Use `[]` for a clean draft with no warnings.
- Each non-info flag must have at least one `source_refs[]` entry unless the flag comes from missing policy/input; for policy gaps, use a policy/context ref such as `policy:daily_update_auto_send:missing`.
- Any flag with `severity=needs_staff_review`, `needs_manager_review`, `do_not_send`, or `runtime_error` must set `requires_review=true` or `should_send=false` as described in the invariants.
- `message` must be staff/operator-safe and must not include unnecessary PII, raw medical text, full raw notes, or raw provider payloads.

### 3. `should_send`

`should_send` is the agent's recommendation that the draft is eligible for delivery after all deterministic validators and approval gates have passed. It is not itself a send command.

Type: boolean.

Validation rules:

- Required.
- Default/conservative value for v1 production is `false` because no approved daily-care auto-send policy exists yet.
- `true` is valid only when all of the following are true:
  - `requires_review=false`;
  - `review_reason=null`;
  - `customer_message.body` is non-null and non-empty;
  - every customer-facing fact is represented in `included_facts[]` with sufficient source refs and customer-safe visibility;
  - no `internal_flags[]` item has severity other than `info`;
  - policy context contains explicit approved authority for the channel, service line, location, cadence/window, consent/opt-out state, media use if any, and template/tone rules;
  - deterministic runtime validation passed and recorded an approval or safe-send policy ref.
- `should_send=false` may still include a useful customer-message draft for staff review.
- `should_send=true` never authorizes provider/PMS writes, care-task completion, incident publication, payment action, or media capture.

### 4. `requires_review`

`requires_review` tells the application to route the output to staff/manager review before any customer-facing delivery.

Type: boolean.

Validation rules:

- Required.
- Must be `true` whenever review-gated evidence, missing policy, sensitive facts, missing required input, stale/conflicting evidence, unapproved media, incident/safety signal, medication/health/behavior concern, customer-message approval gap, validation warning, or do-not-send explanation exists.
- May be `true` with `customer_message.body=null` for suppression/no-send cases.
- May be `true` with a non-null draft when the safest output is a review packet rather than no draft.

### 5. `review_reason`

`review_reason` is the concise operator-facing reason for review, suppression, or no-send.

Type: string or null.

Recommended enum-like values:

- `customer_message_approval_not_configured`
- `missing_required_update_evidence`
- `missing_or_unapproved_photo`
- `photo_quality_or_privacy_review_required`
- `incident_or_safety_review_required`
- `medical_or_medication_review_required`
- `feeding_exception_review_required`
- `behavior_review_required`
- `raw_note_requires_customer_safe_projection`
- `conflicting_or_stale_evidence`
- `policy_gap_requires_review`
- `consent_or_opt_out_blocks_send`
- `duplicate_or_replay_no_new_send`
- `validation_failed_safe`
- `not_applicable_no_update_due`

Validation rules:

- Required when `requires_review=true`; it must be a non-empty string.
- Must be `null` when `requires_review=false`.
- Should match the highest-severity `internal_flags[]` item when flags are present.
- Must be operator-safe and concise. Detailed evidence belongs in `internal_flags[]`, `included_facts[]`, `omitted_facts[]`, audit logs, or review tasks.

### 6. `included_facts`

`included_facts` is the customer-message citation list. It records each source-backed fact the agent used in `customer_message.body` or `customer_message.media_refs[]`.

Type:

```ts
type IncludedFact = {
  fact_id: string;
  fact_kind: "feeding" | "play_enrichment" | "mood_behavior" | "bathroom_elimination" | "medication" | "photo_media" | "care_plan" | "rest" | "grooming_bath_training" | "general_status" | "other";
  customer_text_span: string | null;
  normalized_value: string;
  source_refs: EvidenceRef[];
  customer_visibility: "customer_facing_approved" | "customer_safe_after_review";
  review_state: "approved" | "needs_staff_review" | "needs_manager_review";
  freshness: "current_window" | "same_day" | "prior_day" | "stale";
  confidence: "high" | "medium" | "low";
};

type EvidenceRef = {
  ref: string;
  source_kind: "care_note" | "staff_task_evidence" | "media_ref" | "incident" | "care_profile_snapshot" | "reservation" | "policy" | "audit_event" | "provider_import" | "workflow_event";
  source_id: string;
  version_or_hash: string | null;
  observed_at: string | null;
  actor_ref: string | null;
  visibility: "internal_only" | "customer_provided" | "customer_safe_after_review" | "customer_facing_approved";
};
```

Validation rules:

- Required and must be an array.
- Empty array is valid only when `customer_message.body=null` or the draft contains no factual claims beyond a deterministic template placeholder that is separately policy-approved.
- Every factual sentence/phrase in `customer_message.body` must map to one or more `included_facts[]` items via `customer_text_span` or equivalent sentence/span identifier.
- `source_refs[]` must include stable refs back to staff notes, care-note versions, task evidence, media refs, incident refs, policy refs, or audit events. Do not use vague labels such as `staff note` without an id/version/hash.
- `normalized_value` must be the redacted semantic value used for drafting, not raw staff free text when that text contains sensitive/internal details.
- `fact_kind=medication`, `feeding` with exception/refusal, `mood_behavior` with concern/restriction, `photo_media`, or any incident-derived fact must have `review_state=approved` to be auto-send eligible. Otherwise set `requires_review=true`.
- `freshness=stale` facts may support context for staff review but must not justify `should_send=true` without deterministic policy approval.
- `confidence` is evidence/model confidence only; it is never approval authority.

Mapping to staff notes/photos:

- For staff notes, `source_refs[].source_kind` should be `care_note`, `staff_task_evidence`, or `provider_import` and must carry the note/task/provider id plus source version/hash used by the prompt packet. If the staff note was corrected/voided after drafting, validators must either rebuild the output or set `requires_review=true` with an appropriate flag.
- For photos/media, `source_refs[].source_kind=media_ref`; `source_id` is the stable `MediaRef`/document ref. The fact must also include consent/review state through `customer_visibility` and `review_state`. Raw image bytes, camera ids beyond approved refs, and thumbnails should not be copied into the output.
- For policy/template facts, use `source_kind=policy` and a stable policy/template version ref.
- For audit traceability, source refs should be sufficient to reconstruct which prompt-packet inputs were used without storing raw sensitive prompt text in the LLM response.

### 7. `omitted_facts`

`omitted_facts` records source-backed facts that were available to the agent but intentionally excluded from `customer_message.body`.

Type:

```ts
type OmittedFact = {
  fact_id: string;
  fact_kind: "feeding" | "play_enrichment" | "mood_behavior" | "bathroom_elimination" | "medication" | "photo_media" | "incident_safety" | "medical_health" | "payment_policy" | "staff_internal" | "other";
  omission_reason: "internal_only" | "sensitive_requires_review" | "not_customer_safe" | "unsupported_or_unverified" | "stale" | "conflicting" | "duplicate" | "not_relevant_to_update" | "missing_required_policy" | "photo_unavailable_or_unsuitable";
  source_refs: EvidenceRef[];
  review_state: "no_review_needed" | "needs_staff_review" | "needs_manager_review" | "blocked_by_policy";
  staff_note: string;
};
```

Validation rules:

- Required and must be an array.
- Use `[]` when no considered facts were omitted.
- Every omitted sensitive, conflicting, stale, unverified, internal-only, or photo/media-unsuitable fact that affects send/review/suppression must be represented here or in `internal_flags[]`.
- `staff_note` must be concise and operator-safe; do not quote raw internal notes unless the audit retention/redaction policy explicitly allows it for the review audience.
- Omitted incident, medical/health, medication exception, feeding exception, behavior concern, unsafe photo, missing-photo, or policy/consent facts must set `requires_review=true` or `should_send=false`.

Mapping to staff notes/photos:

- Omitted staff-note facts must preserve the same `EvidenceRef` stability requirements as `included_facts[]`: note/task/provider id, source version/hash, observed time when known, and actor ref when available.
- Omitted photo/media facts must cite the media ref or media-attempt ref and the reason it was omitted: unavailable, poor quality, wrong pet, unsafe context, consent unknown/denied, retention expired, conflicts with text, or sensitive incident/safety context.
- If a required photo is absent, record an omitted fact with `fact_kind=photo_media`, `omission_reason=photo_unavailable_or_unsuitable`, and a policy/task/source ref explaining why a photo was required.

## Cross-field invariants

Deterministic runtime validation must enforce these invariants before persistence, review routing, or any side effect:

1. Required fields: `customer_message`, `internal_flags`, `should_send`, `requires_review`, `review_reason`, `included_facts`, and `omitted_facts` must all be present.
2. Review reason: `requires_review=true` implies `review_reason` is a non-empty string. `requires_review=false` implies `review_reason=null`.
3. Review blocks send: `requires_review=true` implies `should_send=false` unless a separate post-review deterministic override records an approval id outside the AI output. The AI output itself must not claim that override.
4. No message without body: `should_send=true` implies `customer_message.body` is non-null, non-empty, validation-passed, and channel/policy eligible.
5. Null body cannot send: `customer_message.body=null` implies `should_send=false`.
6. Flags block send: any `internal_flags[]` item with severity other than `info` implies `should_send=false`. Staff/manager/do-not-send/runtime flags also imply `requires_review=true` unless the flag is `duplicate_or_replay_no_new_send` with no customer impact and deterministic policy handles it as a no-op.
7. Facts back copy: every customer-facing factual claim and every media mention must be represented in `included_facts[]` with source refs and safe visibility.
8. Unsafe facts omitted: sensitive/internal/unverified/conflicting/stale facts considered by the agent must be represented in `omitted_facts[]` or `internal_flags[]`; they must not silently disappear when they affect review, suppression, or customer interpretation.
9. Source refs are stable: included/omitted facts must reference staff notes, task evidence, media refs, policy refs, workflow events, or audit events by id and version/hash when available.
10. Raw text separation: raw internal staff notes, provider payloads, incident narratives, medical details, and payment data must not appear in `customer_message.body`; use redacted semantic values and evidence refs.
11. AI cannot complete care: the output may cite staff task evidence but must not mark feeding, medication, play, bathroom, cleaning, incident, photo, or handoff tasks complete.
12. AI cannot send: even when `should_send=true`, delivery is a separate deterministic/human-approved action with its own idempotency/effect ledger and audit record.
13. Replay is no-op or rebuild: duplicate/replay events must converge on the same reservation/day/window draft/review packet. If no evidence changed, set `should_send=false` with `duplicate_or_replay_no_new_send` rather than creating a new send.
14. Correction invalidates stale copy: if any included source note/media is corrected, voided, expired, or superseded before send, validators must rebuild the output or route to review.
15. Policy gaps are explicit: missing photo/consent/tone/cadence/auto-send policy must become a review reason or do-not-send condition, not an implicit approval.

## Runtime validation and audit logging compatibility

Recommended validation path:

1. Parse the model response as strict JSON against `DailyCareUpdateOutput`.
2. Reject unknown top-level fields unless the schema version explicitly permits extension fields.
3. Validate field types, enum values, required/nullability rules, and max lengths.
4. Validate cross-field invariants above.
5. Validate each customer-facing sentence/span against `included_facts[]` and each fact's source visibility/review state.
6. Validate `internal_flags[]` against policy/context and required review gates.
7. Validate that omitted sensitive facts are represented and not leaked into `customer_message.body`.
8. Validate replay/idempotency: same location + reservation/day/window + event type + source version/hash maps to the same draft/review packet unless source evidence changed.
9. Persist the validated structured output, validation result, source manifest, policy snapshot/version, redaction profile, prompt-packet manifest/hash, model/schema version, and recommended action ids in audit storage.
10. Store refs/hashes and redacted summaries by default; do not store raw prompts, raw staff notes, raw provider payloads, raw photos, or full model traces unless an approved retention/privacy policy requires and permits them.

Failure behavior:

- Parse/schema failure: do not send; create `validation_failed_safe` flag if a partial safe packet can be represented, otherwise fail the `WorkflowResult` safely and route to review/dead-letter.
- Unsupported claim in customer copy: do not send; set `requires_review=true`, `review_reason=validation_failed_safe`, and include a flag/source pointer if possible.
- Missing required evidence: do not fabricate; set `requires_review=true` or `status=needs_more_information` with missing-evidence flags/tasks.
- Policy denial or opt-out: set `should_send=false`; use `review_reason=consent_or_opt_out_blocks_send` or a policy-specific no-send reason.

Audit fields to capture outside or alongside this structured output:

- `workflow_event_id`, event type, source event version/hash, idempotency key, reservation/day/window key, location id, pet id(s), customer id, and policy snapshot refs.
- Agent spec/version, schema name/version, prompt-packet manifest/hash, redaction profile, model/runtime id, validator version, and validation pass/fail details.
- Source evidence manifest: staff note ids/versions, task evidence ids, media refs, incident refs, provider refs, care profile snapshot ids, audit refs, and which were included vs omitted.
- Review routing: required role, review reason, internal flags, created review task/draft id, approval/override id when later approved, and any post-review changes.
- Effect ledger: send attempt id, delivery channel, recipient ref, approval id, provider/message id, retry status, and sent/published timestamp. This ledger is separate from the AI output and must not be created by the AI alone.

## Examples

### Example A: clean auto-send eligible after explicit policy approval

This shape is valid only in an environment where deterministic policy context already approves low-risk auto-send for the location/channel/template and validators can prove consent, source grounding, and no review flags. In the current draft policy posture, production should keep this as a compatibility target rather than live behavior.

```json
{
  "customer_message": {
    "body": "Milo had a relaxed afternoon and enjoyed his individual play session. He ate dinner as expected, and we included a photo from today's play time.",
    "channel_hint": "portal",
    "template_id": "daily_update_routine_v1",
    "language": "en-US",
    "tone": "warm_concise_factual",
    "media_refs": ["media_789"],
    "audience": "customer",
    "redaction_profile": "customer_safe_daily_update_v1"
  },
  "internal_flags": [],
  "should_send": true,
  "requires_review": false,
  "review_reason": null,
  "included_facts": [
    {
      "fact_id": "fact_play_1",
      "fact_kind": "play_enrichment",
      "customer_text_span": "Milo had a relaxed afternoon and enjoyed his individual play session.",
      "normalized_value": "individual play completed; demeanor relaxed",
      "source_refs": [
        {
          "ref": "staff_task_evidence:task_456:v3",
          "source_kind": "staff_task_evidence",
          "source_id": "task_456",
          "version_or_hash": "v3",
          "observed_at": "2026-06-11T19:15:00Z",
          "actor_ref": "staff:123",
          "visibility": "customer_facing_approved"
        }
      ],
      "customer_visibility": "customer_facing_approved",
      "review_state": "approved",
      "freshness": "current_window",
      "confidence": "high"
    },
    {
      "fact_id": "fact_feeding_1",
      "fact_kind": "feeding",
      "customer_text_span": "He ate dinner as expected",
      "normalized_value": "dinner eaten as expected per reviewed feeding evidence",
      "source_refs": [
        {
          "ref": "care_note:note_222:v1",
          "source_kind": "care_note",
          "source_id": "note_222",
          "version_or_hash": "v1",
          "observed_at": "2026-06-11T22:05:00Z",
          "actor_ref": "staff:221",
          "visibility": "customer_facing_approved"
        }
      ],
      "customer_visibility": "customer_facing_approved",
      "review_state": "approved",
      "freshness": "current_window",
      "confidence": "high"
    },
    {
      "fact_id": "fact_photo_1",
      "fact_kind": "photo_media",
      "customer_text_span": "we included a photo from today's play time",
      "normalized_value": "approved play photo attached",
      "source_refs": [
        {
          "ref": "media_ref:media_789:approved",
          "source_kind": "media_ref",
          "source_id": "media_789",
          "version_or_hash": "sha256:abc123",
          "observed_at": "2026-06-11T19:20:00Z",
          "actor_ref": "staff:123",
          "visibility": "customer_facing_approved"
        }
      ],
      "customer_visibility": "customer_facing_approved",
      "review_state": "approved",
      "freshness": "current_window",
      "confidence": "high"
    }
  ],
  "omitted_facts": []
}
```

### Example B: review-required draft

```json
{
  "customer_message": {
    "body": "Luna spent part of the afternoon resting and had a gentle potty walk. The team is keeping a close eye on her and will continue following her care plan.",
    "channel_hint": "portal",
    "template_id": null,
    "language": "en-US",
    "tone": "neutral_staff_review",
    "media_refs": [],
    "audience": "customer",
    "redaction_profile": "customer_safe_daily_update_v1"
  },
  "internal_flags": [
    {
      "code": "medical_or_medication_review_required",
      "severity": "needs_manager_review",
      "message": "A staff note mentions possible stomach upset; manager/care review is required before customer wording is sent.",
      "source_refs": ["care_note:note_333:v2"],
      "recommended_action": "manager_review"
    },
    {
      "code": "customer_message_approval_not_configured",
      "severity": "needs_staff_review",
      "message": "No approved low-risk auto-send policy is configured for daily care updates.",
      "source_refs": ["policy:daily_update_auto_send:missing"],
      "recommended_action": "staff_review"
    }
  ],
  "should_send": false,
  "requires_review": true,
  "review_reason": "medical_or_medication_review_required",
  "included_facts": [
    {
      "fact_id": "fact_rest_1",
      "fact_kind": "rest",
      "customer_text_span": "Luna spent part of the afternoon resting",
      "normalized_value": "rested during afternoon window",
      "source_refs": [
        {
          "ref": "care_note:note_331:v1",
          "source_kind": "care_note",
          "source_id": "note_331",
          "version_or_hash": "v1",
          "observed_at": "2026-06-11T20:00:00Z",
          "actor_ref": "staff:144",
          "visibility": "customer_safe_after_review"
        }
      ],
      "customer_visibility": "customer_safe_after_review",
      "review_state": "needs_manager_review",
      "freshness": "current_window",
      "confidence": "medium"
    },
    {
      "fact_id": "fact_bathroom_1",
      "fact_kind": "bathroom_elimination",
      "customer_text_span": "had a gentle potty walk",
      "normalized_value": "potty walk completed; no customer-facing exception included",
      "source_refs": [
        {
          "ref": "staff_task_evidence:task_992:v1",
          "source_kind": "staff_task_evidence",
          "source_id": "task_992",
          "version_or_hash": "v1",
          "observed_at": "2026-06-11T20:35:00Z",
          "actor_ref": "staff:144",
          "visibility": "customer_safe_after_review"
        }
      ],
      "customer_visibility": "customer_safe_after_review",
      "review_state": "needs_manager_review",
      "freshness": "current_window",
      "confidence": "medium"
    }
  ],
  "omitted_facts": [
    {
      "fact_id": "omitted_health_1",
      "fact_kind": "medical_health",
      "omission_reason": "sensitive_requires_review",
      "source_refs": [
        {
          "ref": "care_note:note_333:v2",
          "source_kind": "care_note",
          "source_id": "note_333",
          "version_or_hash": "v2",
          "observed_at": "2026-06-11T20:10:00Z",
          "actor_ref": "staff:144",
          "visibility": "internal_only"
        }
      ],
      "review_state": "needs_manager_review",
      "staff_note": "Health-related observation withheld from customer copy pending manager/care review. Use source note for full details."
    }
  ]
}
```

### Example C: do-not-send / no customer draft

```json
{
  "customer_message": {
    "body": null,
    "channel_hint": "portal",
    "template_id": null,
    "language": "en-US",
    "tone": "none",
    "media_refs": [],
    "audience": "customer",
    "redaction_profile": "customer_safe_daily_update_v1"
  },
  "internal_flags": [
    {
      "code": "incident_or_safety_signal",
      "severity": "do_not_send",
      "message": "An incident/safety signal is linked to this update window; suppress routine update until manager/safety review decides customer communication.",
      "source_refs": ["incident:inc_555:v1", "workflow_event:event_777:v1"],
      "recommended_action": "suppress_update"
    },
    {
      "code": "missing_or_unapproved_photo",
      "severity": "needs_staff_review",
      "message": "Location policy requires a same-day photo, but the only media attempt was permission-denied.",
      "source_refs": ["policy:photo_required:loc_1:v4", "media_attempt:media_attempt_101:v1"],
      "recommended_action": "replace_or_review_photo"
    }
  ],
  "should_send": false,
  "requires_review": true,
  "review_reason": "incident_or_safety_review_required",
  "included_facts": [],
  "omitted_facts": [
    {
      "fact_id": "omitted_incident_1",
      "fact_kind": "incident_safety",
      "omission_reason": "sensitive_requires_review",
      "source_refs": [
        {
          "ref": "incident:inc_555:v1",
          "source_kind": "incident",
          "source_id": "inc_555",
          "version_or_hash": "v1",
          "observed_at": "2026-06-11T18:40:00Z",
          "actor_ref": "staff:201",
          "visibility": "internal_only"
        }
      ],
      "review_state": "needs_manager_review",
      "staff_note": "Routine daily update suppressed because the incident workflow owns customer communication until reviewed."
    },
    {
      "fact_id": "omitted_photo_1",
      "fact_kind": "photo_media",
      "omission_reason": "photo_unavailable_or_unsuitable",
      "source_refs": [
        {
          "ref": "media_attempt:media_attempt_101:v1",
          "source_kind": "media_ref",
          "source_id": "media_attempt_101",
          "version_or_hash": "v1",
          "observed_at": "2026-06-11T19:00:00Z",
          "actor_ref": "system:media_capture",
          "visibility": "internal_only"
        },
        {
          "ref": "policy:photo_required:loc_1:v4",
          "source_kind": "policy",
          "source_id": "photo_required:loc_1",
          "version_or_hash": "v4",
          "observed_at": null,
          "actor_ref": null,
          "visibility": "internal_only"
        }
      ],
      "review_state": "blocked_by_policy",
      "staff_note": "Required photo was unavailable because capture permission was denied; create a replacement/review task instead of implying a photo exists."
    }
  ]
}
```

## Compatibility notes for implementation

- Treat `DailyCareUpdateOutput` as the workflow-specific payload inside the runtime's generic `WorkflowResult<T>` envelope.
- Keep the top-level schema stable and required so AI runtime output validation can fail closed.
- Prefer closed enums for validator-critical fields (`severity`, `recommended_action`, `fact_kind`, `omission_reason`, `review_state`, `customer_visibility`) and allow controlled extension only by schema version.
- Keep `included_facts[]` and `omitted_facts[]` as audit/linkage arrays, not prose-only explanations. They are the bridge from customer copy back to staff notes, task evidence, media refs, policy refs, and audit events.
- Do not use the AI output as the durable effect ledger. The application should persist approvals, sends, provider writes, retries, and delivery receipts separately with idempotency keys.
- Redact/minimize by default: use source refs, versions, hashes, semantic values, and concise operator-safe notes rather than raw notes, raw photos, raw provider payloads, payment details, or unnecessary PII.
- Downstream code may add a strongly typed domain struct later, but the domain model should preserve these semantic invariants: customer copy is source-grounded, review gates block sends, omitted facts are auditable, and AI suggestions never execute side effects by themselves.
