# Data-quality hygiene measurable labor loop contract

Purpose: define the second review-gated labor-cost loop on the same deterministic app/agent rails proven by Manager Daily Brief. The loop turns source-grounded data-quality issues into ranked internal cleanup work, keeps ambiguity visible to managers/front-desk leads, validates agent drafts before staff see them, captures reviewed outcomes, and measures manual reconciliation minutes avoided.

This workflow is internal hygiene, not generic data cleanup and not autonomous source-system repair. It exists because the NVA Pet Resorts operating model has repeated manager/front-desk work across 170 locations: checking missing vaccine/source evidence, resolving duplicate customer or pet candidates, filling profile gaps, re-checking stale source freshness, and translating inconsistent service-line naming before later workflows can safely act.

## Repetitive work removed

The workflow reduces repeated source reconciliation and rework that otherwise appears in every downstream labor-saving loop:

1. Missing evidence investigation: staff repeatedly re-open PMS/provider records to find source facts that should already be attached to a reservation, stay, customer, pet, vaccine, or service-line record.
2. Duplicate candidate reconciliation: managers/front-desk leads compare likely duplicate customers or pets before retention, booking triage, checkout, and reporting work can proceed.
3. Incomplete profile cleanup: staff re-check incomplete pet/customer profile fields such as missing vaccination evidence, pet profile gaps, owner/pet relationship ambiguity, or stale contact/source evidence.
4. Service-line normalization review: managers review ambiguous or unmapped provider service names before demand, labor planning, grooming/daycare/boarding reporting, or upsell workflows consume them.
5. Source freshness review: managers decide whether stale or conflicting evidence should block downstream action, remain visible as a nonblocking issue, or be marked as wrong-source after human review.

The workflow must not make ambiguity disappear. Ambiguity becomes manager-visible work with source refs, owner persona, review gate, and outcome capture.

## Affected personas

- General manager: owns location-level hygiene prioritization, source ambiguity review, suppression decisions, and labor-savings accountability.
- Assistant general manager: can review the same queue when delegated by the general manager.
- Front-desk lead: owns customer/pet/profile cleanup tasks and duplicate-candidate investigation where the next step is internal review rather than a source-system write.
- Front-desk agent: may perform approved investigation work and collect missing source evidence, but the contract does not authorize customer sends or PMS/provider mutation.
- Regional operator or operations analyst: may read aggregated hygiene metrics and recurring issue categories, but cannot use this workflow to hide unresolved location ambiguity.

## Source concepts and current vocabulary

The implementation should start from `domain::data_quality` rather than inventing a parallel vocabulary. Current issue concepts include:

- `data_quality::Issue` carrying `kind`, `severity`, `provenance`, `source_record_ref`, `detected_at`, `resolution_status`, `visible_to_bi`, and `workflow_blocking`.
- `data_quality::Kind` variants relevant to this workflow: `MissingRequiredField`, `AssumptionInForce`, `UnknownSourceStatus`, `DuplicateSourceRecord`, `AmbiguousOwnerPetRelationship`, `UnmappedServiceType`, `LocationScopeAmbiguity`, `PaymentStateConflict`, `CheckoutEvidenceMissing`, `UnclosedReservation`, `IncompletePetProfile`, `MissingVaccinationRecord`, and `SensitivePayloadQuarantined`.
- `data_quality::FieldPath` for reservation, stay, and source fields.
- `source::RecordRef` and `source::Provenance` as the source-of-truth boundary for every operational claim.

Do not promote provider DTO names, storage codes, or HTTP payload field names into the workflow core. Boundary crates can adapt provider/storage/runtime shapes into `domain::data_quality` and app-owned workflow values.

## App-owned endpoints

Expose three app-owned surfaces, matching the Manager Daily Brief pattern.

### 1. Read-only context endpoint

```http
GET /agent/context/data-quality-hygiene?location_id=...&operating_day=...
```

The app returns a replayable context packet. Required shape:

```json
{
  "workflow": "data-quality-hygiene",
  "schema_version": "data-quality-hygiene-context-v1",
  "context_packet_id": "data-quality-hygiene-context:{location_id}:{operating_day}",
  "correlation_id": "data-quality-hygiene:{location_id}:{operating_day}",
  "location_id": "...",
  "operating_day": "2026-06-17",
  "prepared_for": "general_manager",
  "review_surface": "manager_data_quality_queue",
  "issue_candidates": [],
  "duplicate_candidates": [],
  "profile_gap_candidates": [],
  "service_line_mapping_candidates": [],
  "source_freshness_candidates": [],
  "allowed_agent_actions": [
    "summarize_source_evidence",
    "rank_hygiene_actions",
    "draft_internal_cleanup_task",
    "preserve_ambiguity_for_review",
    "estimate_reconciliation_minutes_saved"
  ],
  "blocked_actions": [
    "send_customer_message",
    "mutate_provider_or_pms_record",
    "change_staff_schedule",
    "move_refund_discount_or_payment",
    "hide_or_auto_resolve_source_ambiguity"
  ],
  "audit": {
    "source_refs": [],
    "packet_created_at": "...",
    "adapter_version": "local-data-quality-hygiene-context-v1"
  }
}
```

Context packet requirements:

- Include workflow name, schema version, stable context packet id, and correlation id.
- Scope every candidate to `location_id` and `operating_day`; cross-location ambiguity must be explicit and manager-visible.
- Group facts by source-system concept: source issue, duplicate candidate, profile gap, service-line mapping, source freshness. Do not group by prompt prose.
- Carry `source::RecordRef` evidence for each claim and `source::Provenance` for each candidate's origin.
- Preserve `domain::data_quality::Issue` details, including severity, workflow blocking status, BI visibility, and resolution status.
- Include any source transformation details the app used to produce the candidate.
- Include redaction/sensitivity metadata when evidence references vaccine, incident, payment, or quarantined payloads.
- Include allowed and blocked actions in every packet. Agents should not infer permission from absence.

If a candidate has missing provenance, the packet may expose it only as a warning/investigation item. It must not support an accepted cleanup recommendation until the draft either references existing source refs or explicitly asks a human to investigate missing evidence.

### 2. Draft endpoint

```http
POST /agent/drafts/data-quality-hygiene
```

Hermes submits a draft internal cleanup packet. Required shape:

```json
{
  "context_packet_id": "data-quality-hygiene-context:{location_id}:{operating_day}",
  "correlation_id": "data-quality-hygiene:{location_id}:{operating_day}",
  "actions": [
    {
      "action_id": "dq-missing-vaccine-evidence-...",
      "kind": "review_stale_vaccination_source_freshness",
      "priority": "high",
      "owner_persona": "front_desk_lead",
      "rationale": "Source evidence shows the profile is missing current vaccination proof; keep the ambiguity visible until staff verifies the source.",
      "source_refs": [],
      "data_quality_issue_refs": [],
      "required_review_gate": "manager_approval",
      "estimated_before_minutes": 25,
      "estimated_after_minutes": 8,
      "requested_side_effects": []
    }
  ]
}
```

Draft validation must be strict:

- The referenced `context_packet_id` and `correlation_id` must exist and be replayable.
- Every action kind must be one of the allowed data-quality hygiene action kinds below.
- Every operational claim must cite source refs from the context packet.
- Every action must cite at least one data-quality issue or source candidate from the context packet.
- Required review gates must match workflow policy for the candidate kind and severity.
- Unknown non-empty requested side effects must be rejected fail-closed.
- Known blocked side effects must be rejected with explicit evidence.
- Drafts must not mark a data-quality issue repaired, ignored, hidden, or superseded. Only reviewed outcomes can record those decisions.
- Drafts that reduce ambiguity by assertion instead of routing it to staff review must be rejected.

Rejected drafts are audit events. Store whether rejection came from unsupported action kind, missing source refs, stale packet, wrong review gate, blocked side effect, unsupported side effect, or attempted ambiguity hiding.

### 3. Outcome endpoint

```http
POST /data-quality-hygiene/actions/{action_id}/outcome
```

Captures reviewed staff/manager feedback after internal work:

```json
{
  "outcome": "completed",
  "actual_minutes": 9,
  "actor": "front_desk_lead",
  "feedback": "Found the vaccine document in the source record; PMS update still requires normal staff workflow outside the agent contract.",
  "source_refs_considered": [],
  "resolution_status_after_review": "acknowledged"
}
```

Outcome capture records labor evidence and reviewed disposition. It does not execute a provider/PMS write. Required fields:

- action id;
- actor/persona;
- outcome;
- before minutes;
- actual minutes;
- source refs considered;
- reviewed data-quality issue refs;
- optional manager feedback;
- timestamp and audit correlation id.

Outcome values:

- `completed`: staff completed the internal investigation or cleanup-preparation task.
- `deferred`: staff chose not to act today; issue remains visible.
- `suppressed_by_manager`: manager intentionally suppressed this task for the operating day; issue is not auto-resolved.
- `source_fact_was_wrong`: reviewer found the cited source evidence wrong or stale.
- `not_actionable`: issue was valid but cannot be acted on without a separate approved workflow or source-system authority.

Outcome capture may record a reviewed `data_quality::ResolutionStatus` transition only as evidence. Any actual PMS/provider correction, customer outreach, schedule update, payment action, refund, discount, or BI hiding remains outside this endpoint and requires a separate deterministic workflow with its own policy gate.

## Action kinds

Allowed internal action kinds:

- `investigate_missing_source_evidence`: find or confirm missing source evidence for required fields such as reservation/customer/pet/location/service type/source payload references.
- `reconcile_duplicate_customer_or_pet_candidate`: prepare an internal duplicate-candidate review packet for manager/front-desk lead decision.
- `complete_missing_pet_or_customer_profile_fields`: create an internal reviewed task for missing pet profile, customer profile, owner/pet relationship, or contact/source fields.
- `review_stale_vaccination_source_freshness`: route missing/stale vaccine evidence to staff review without approving or denying service eligibility.
- `normalize_ambiguous_service_line_naming`: prepare a source-grounded review task for unmapped or ambiguous service names before reporting/automation consumes them.
- `review_checkout_or_unclosed_reservation_evidence`: route checkout evidence missing, unclosed reservation, or conflicting completion signals to front-desk review.
- `escalate_sensitive_or_quarantined_payload`: route `SensitivePayloadQuarantined` or incident/payment-sensitive ambiguity to manager review without exposing raw sensitive contents to the agent.

Each action must include:

- stable action id;
- kind;
- priority;
- owner persona;
- removed manual work category;
- rationale;
- source refs from the context packet;
- data-quality issue refs or candidate ids;
- required review gates;
- labor impact estimate.

## Blocked actions

The context packet, draft validation response, and outcome capture response should all carry the blocked-action set:

- send customer message;
- mutate provider/PMS record;
- change staff schedule;
- move refunds, discounts, or payments;
- hide, auto-resolve, delete, or suppress source ambiguity without reviewed outcome evidence;
- mark vaccine, medical, behavior, incident, payment, or safety-sensitive evidence as accepted without deterministic policy and human review;
- read raw provider/database/object-store records directly from the agent runtime;
- expose quarantined sensitive payload contents to the agent.

These are blocked even when the agent draft appears source-grounded. A later implementation may hand off to a separate approved workflow, but this workflow only creates and measures internal hygiene tasks.

## Review gates

Minimum review policy:

- Missing source evidence, incomplete profile, and unmapped service-line naming: `ManagerApproval` or `FrontDeskLeadReview` depending on local operating policy.
- Duplicate customer/pet candidate: `ManagerApproval`, because merge decisions can affect customer history and later automation.
- Missing/stale vaccination source freshness: `ManagerApproval`; service eligibility or customer communication must be handled by separate vaccine/customer-message workflows.
- Payment state conflict: `ManagerApproval` plus payment-specific review before any money movement; this workflow only records the ambiguity.
- Sensitive/quarantined payload: `ManagerApproval` and redaction required; agent sees metadata, not raw sensitive payload.
- Source fact was wrong: reviewer outcome required before any resolution-status transition is stored.

## Labor metric

The metric is minutes avoided in manual source reconciliation and repeated staff rework per location/day:

`minutes_saved = before_minutes - actual_minutes`

The app should store both estimated and actual values:

- estimated before minutes: how long the manual reconciliation normally takes without a ranked source-grounded hygiene queue;
- estimated after minutes: expected staff review time when the action is source-grounded and ranked;
- actual minutes: staff-reported time captured in the outcome endpoint;
- actual minutes saved: estimated before minutes minus actual minutes.

Initial fixture-safe estimates for tests:

- missing source evidence investigation: 25 min before, 8 min after;
- duplicate customer/pet candidate reconciliation: 30 min before, 12 min after;
- incomplete pet/customer profile cleanup preparation: 20 min before, 7 min after;
- stale vaccination/source freshness review: 25 min before, 10 min after;
- ambiguous service-line naming normalization: 20 min before, 6 min after;
- checkout/unclosed reservation evidence review: 20 min before, 8 min after.

The implementation should expose totals across ranked actions and preserve per-action measurements so regional operators can see recurring causes rather than only aggregate minutes.

## Implementation-ready type surface

Suggested app module: `app::data_quality_hygiene`.

Suggested domain/app concepts:

- `data_quality_hygiene::Request`: location, operating day, prepared persona, issue candidates, optional prioritization policy.
- `data_quality_hygiene::Packet`: location, operating day, prepared persona, actions/candidates, safe agent actions, blocked actions, aggregate labor estimates.
- `data_quality_hygiene::Candidate`: app-owned wrapper around `domain::data_quality::Issue`, source refs, candidate id, candidate kind, sensitivity/redaction metadata, and source freshness.
- `data_quality_hygiene::Action`: stable id, action kind, priority, owner persona, removed manual work, rationale, source facts, required review gates, labor impact.
- `data_quality_hygiene::DraftSubmission`: context packet id, correlation id, action drafts, requested side effects.
- `data_quality_hygiene::DraftValidation`: accepted/rejected actions, rejection reasons, audit metadata.
- `data_quality_hygiene::OutcomeRecord`: action id, actor, outcome, before minutes, actual minutes, source refs considered, issue refs considered, optional reviewed resolution status.

Suggested safe agent action enum:

- `SummarizeSourceEvidence`
- `RankHygieneActions`
- `DraftInternalCleanupTask`
- `PreserveAmbiguityForReview`
- `EstimateReconciliationMinutesSaved`

Suggested blocked action enum:

- `SendCustomerMessage`
- `MutateProviderOrPmsRecord`
- `ChangeStaffSchedule`
- `MoveRefundDiscountOrPayment`
- `HideOrAutoResolveSourceAmbiguity`
- `ExposeQuarantinedSensitivePayload`

Suggested rejection reasons:

- stale or unknown context packet;
- unsupported action kind;
- missing source refs;
- source refs not present in context packet;
- missing data-quality issue refs;
- wrong review gate;
- blocked side effect requested;
- unsupported side effect requested;
- attempted ambiguity hiding;
- sensitive payload exposure attempted.

## Fixture-safe implementation slice

A code worker should be able to implement the first slice without live data or secrets:

1. Add `app::data_quality_hygiene` with typed request/packet/action/draft/outcome contracts using in-memory fixture candidates built from `domain::data_quality::Issue` and `source::Provenance`.
2. Add focused app tests proving source-grounded actions, blocked actions, review gates, ambiguity preservation, and labor estimates.
3. Add API runtime shell endpoints for the three paths above using local fixtures, mirroring Manager Daily Brief's local endpoint strategy.
4. Add storage record(s) only for accepted draft/outcome evidence, not provider mutation.
5. Add script/Hermes bridge only after app/API contracts pass, with no direct Postgres/object-store/provider access.

The first smoke should prove: context packet is read, agent/tool submits one source-grounded internal cleanup draft, app rejects a draft with `hide_or_auto_resolve_source_ambiguity`, reviewed outcome captures actual minutes, and no live side effects are attempted.

## Verification expectations for implementation

Focused gates for future code cards:

- app workflow contract tests for `app::data_quality_hygiene`;
- API contract tests for the context/draft/outcome endpoints;
- storage outcome codec tests if persistence is added;
- Hermes bridge unit tests if scripts are added;
- `cargo test -p app data_quality_hygiene` or the narrowest equivalent first;
- `cargo test --workspace --no-run` before fan-in only when the implementation card changes compiled surfaces broadly.

Docs-only changes to this file should run `git diff --check`; markdown link checks are required only when adding links.
