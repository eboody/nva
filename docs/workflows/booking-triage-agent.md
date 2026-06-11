# Booking triage agent

Status: canonical workflow artifact for the current booking-triage workstream. This document synthesizes the completed lifecycle, deterministic-rule, output-schema, AI-role, and test-scenario cards into one integration contract for product, data-model, task-model, and implementation planning.

This document is a workflow/specification artifact only. It does not authorize live booking confirmation, rejection, check-in, checkout, provider/PMS mutation, capacity hold/release, room/run/group assignment, payment movement, vaccine clearance, special-care acceptance, behavior exception, or customer-message sending.

Anything marked proposed, draft, review-gated, or approval-gated requires explicit manager/location policy approval before it becomes production behavior. AI recommendations are advisory over deterministic rule outputs; they do not create availability, policy exceptions, or execution authority.

## Scope

This workflow covers booking triage for boarding, day play/daycare, day boarding, grooming, training, DaySpa, and approved add-ons at a pet resort or small resort group. It consumes typed request, customer, pet, provider, policy, availability, staffing, vaccine/document, payment, behavior, care, and audit snapshots. It produces readiness/status recommendations, missing-information requests, review packets, draft messages, and internal tasks.

Out of scope unless separately approved:

- autonomous booking confirmation, rejection, check-in, checkout, provider/PMS mutation, room/run/group assignment, capacity hold/release, waitlist promotion, or cancellation execution;
- autonomous vaccine/document approval, medical/care/special-handling acceptance, behavior/group-play exception, incident restriction change, payment/deposit action, fee/waiver/refund/forfeit decision, or policy exception;
- autonomous customer-message sending, except a future explicitly approved receipt-only acknowledgement policy that avoids availability, confirmation, rejection, waitlist priority, price/payment, vaccine/medical/behavior judgment, special-care acceptance, or exception claims.

## Source gaps and assumptions

The completed input cards define the intended integration contract but do not provide final pilot/location policy values for every operational threshold. Until approved policy sources exist, deterministic rules must return `unknown` or `needs_human_approval` rather than inventing values.

Known unresolved source gaps:

- Exact room/run/condo/suite counts, size bands, species constraints, turn-over buffers, overbooking posture, and hold/release permissions by location.
- Exact staff ratios, role qualifications, break/absence handling, special-care load thresholds, medication capability limits, and late pickup coverage policy.
- Exact vaccine requirements by species/service/location/date window, accepted proof types, document-review owners, pending-vaccine deadlines, and exception authority.
- Exact deposit amounts, refundability, holiday surcharge/deposit overlays, payment timing, fee schedules, waiver/discount/credit authority, and payment-message templates.
- Exact holiday/blackout/local-event calendars, minimum stays, manager-hold intervals, cancellation overlays, and waitlist-priority policy.
- Exact behavior restriction taxonomy, group-play eligibility policy, incident reinstatement policy, anxiety/special-handling acceptance criteria, and sensitive-message copy.
- Exact customer-message send policy, approved templates, recipient/channel constraints, redaction policy, and approval ids.

Assumptions used by this artifact:

- Provider/PMS state remains the system of record for executed reservation/check-in/check-out/cancellation states; this workflow can only recommend or prepare approved updates.
- Deterministic policy/rule code owns hard stops, readiness buckets, evidence refs, freshness checks, and approval gates.
- AI owns explanatory text, summarization, draft messages, and internal task wording only, and those outputs are rejected if they conflict with deterministic results.
- Missing, stale, conflicting, unmapped, or unsupported source facts route to `missing_info`, `vaccine_pending`, `special_review`, `waitlisted`, or `failed_safely`; they never become a pass.

## Booking state machine

There are two related state concepts:

- `booking_lifecycle_state`: the customer/request/provider lifecycle state described below. This is authoritative only when backed by provider state, deterministic execution evidence, or human approval.
- `recommended_status`: the triage/readiness bucket returned in the structured output schema. It may recommend that staff route the request toward a lifecycle state, but it does not execute the transition.

AI may explain and recommend based on deterministic evidence; it must not create a lifecycle transition, must not invent availability, and must not override hard policy. Provider writes, customer-visible messages, capacity operations, payment actions, confirmation, rejection, special-care acceptance, and behavior exceptions require the approval gates named in this document.

### Lifecycle states

| State | Meaning | Entry criteria / source of truth | Allowed deterministic transitions | AI/customer-message boundary |
| --- | --- | --- | --- | --- |
| `inquiry` | Customer is asking about service, dates, fit, price, policy, or options before a complete booking request exists. | Inquiry source exists from web, phone, SMS, email, portal, or staff note; required booking facts may be absent. | `booking_request` when minimum request identity/service/date/pet facts exist; `missing_info` when facts are insufficient; `cancelled` when customer withdraws. | AI may draft neutral intake questions. No availability, price/deposit, vaccine, behavior, special-care, confirmation, or rejection claims without deterministic evidence and approval. |
| `booking_request` | A concrete service/date/pet request exists and is ready for deterministic triage. | Typed request record or normalized provider/import record with customer or source ref, service kind, location, dates/window, and at least one pet or explicit unknown marker. | `missing_info`, `vaccine_pending`, `special_review`, `waitlisted`, `offered`, `confirmed` only with prior approval/provider evidence, `rejected` only through approved rejection/hard-denial path, `cancelled`. | AI may summarize request and draft review packet. It must not promise space or mutate provider state. |
| `missing_info` | Required request, customer, pet, policy, provider, payment, care, behavior, or source facts are missing/stale/conflicting. | Deterministic rule emits `unknown`/missing input or validation detects schema/source gap. | `booking_request` when facts arrive; `vaccine_pending` for vaccine/document-specific gaps; `special_review` for policy/care/behavior/payment/staff ambiguity; `cancelled`; `rejected` only with approved path. | AI may draft missing-info requests and internal tasks. Sensitive copy requires approval. Missing facts never become inferred passes. |
| `vaccine_pending` | Vaccine proof/document review is required before booking confirmation or service eligibility. | Vaccine rule sees missing, expired, unverified, OCR-only, unmapped, pending, or conflicting proof, or a policy-defined pending path. | `booking_request`/`offered` after document approval and other rules pass; `special_review` for exceptions; `rejected` only by approved vaccine/policy path; `cancelled`. | AI can request documents and explain review status. It cannot mark vaccines verified, clear eligibility, or approve group play/daycare. |
| `special_review` | Human review is required for medical/care/medication, behavior, incident, staff coverage, capacity exception, payment/deposit, holiday, late pickup, unsupported service, provider conflict, or other policy exception. | Deterministic rule emits `needs_human_approval`, `hard_block` with exception path, or `unknown` that cannot be resolved by routine customer info. | `booking_request`, `waitlisted`, `offered`, `confirmed` with approval/execution evidence, `rejected` with approval/hard-denial path, `cancelled`. | AI may prepare manager/care/behavior/payment packets. It cannot accept exceptions or send sensitive decisions. |
| `waitlisted` | Request cannot be offered now because capacity/staffing/holiday/hold posture blocks immediate availability, and waitlist policy exists. | Fresh deterministic availability/staffing/policy evidence shows full/limited/closed posture with waitlist allowed, or prior human waitlist state exists. | `offered` after staff/manager approval and fresh availability; `booking_request` for alternate dates; `cancelled`; `rejected` with approved path. | AI may draft waitlist review language. It cannot promote from waitlist, promise priority, or imply future availability. |
| `offered` | Staff-approved offer exists; customer may need to accept, provide deposit, resolve documents, or confirm details. | Human/staff approval evidence or provider offer state. Any inventory/price/deposit facts are source-backed. | `confirmed` after all gates/payment/document requirements pass and execution is approved; `missing_info`; `vaccine_pending`; `special_review`; `waitlisted` if offer expires/capacity changes by policy; `cancelled`; `rejected` with approval. | AI may draft offer follow-up only from approved offer evidence. It cannot create the offer, hold capacity, collect payment, or claim confirmation. |
| `confirmed` | Booking is confirmed in provider/system-of-record or through an approved execution path. | Existing provider confirmed state, or approved confirmation workflow with provider write/confirmation evidence. | `checked_in`, `cancelled`, `special_review` if new blocking evidence appears, `checked_out` only through provider/staff lifecycle evidence. | AI may summarize confirmed facts and draft staff tasks. It cannot newly confirm a booking. Customer confirmation messages require approved send path. |
| `checked_in` | Pet/customer has arrived and staff/provider check-in was completed. | Provider/staff check-in evidence with timestamp, location, pet/customer/reservation refs. | `active`, `cancelled` only if check-in reversal/no-show/cancellation policy supports it, `special_review` for unresolved care/behavior/payment issue. | AI may summarize check-in context for staff. It cannot check in pets or override unresolved prerequisites. |
| `active` | Stay/day service is currently being executed. | Provider/staff active-stay evidence after check-in; care/task workflows own execution details. | `checked_out`, `special_review` for incident/care/payment/extension issue, `cancelled` only through approved operational path. | AI may draft internal handoff/tasks. It cannot make care/medical/behavior/payment decisions or send sensitive updates without approval. |
| `checked_out` | Stay/service is complete and customer/pet departure is recorded. | Provider/staff checkout evidence, including final timing and unresolved payment/care notes as refs. | Terminal for booking triage except post-stay follow-up/CRM workflows; `special_review` for unresolved late pickup/payment/incident issue. | AI may summarize completion and draft follow-up tasks/messages for approval. It cannot close financial/care exceptions. |
| `cancelled` | Customer/staff/provider has cancelled or withdrawn request/booking under policy. | Provider cancellation state, customer withdrawal evidence, or approved cancellation execution. | Usually terminal. May return to `inquiry`/`booking_request` only if a new request is created. | AI may draft acknowledgement only if policy/template permits. It cannot cancel or waive/charge fees. |
| `rejected` | Request is denied under deterministic hard policy or approved manager/staff rejection path. | Deterministic hard-denial rule plus approved automation path, or human/manager rejection approval evidence. | Terminal for this request. New request starts at `inquiry` or `booking_request`. | AI may draft rejection language only for approval. It cannot reject autonomously or send denial language. |

### Transition invariants

- `confirmed`, `checked_in`, `active`, `checked_out`, `cancelled`, and `rejected` are execution/provider lifecycle states, not ordinary AI recommendations.
- `confirmed booking automation` gate is required before any new confirmation, provider status update, room/run/group assignment, capacity hold/release, or waitlist promotion.
- `rejection` gate is required before rejection state transition or customer-facing denial unless an explicitly approved deterministic hard-denial automation exists.
- `special-care acceptance` gate is required before accepting medical, medication, feeding, allergy, mobility, isolation, anxiety, or other special handling beyond routine policy.
- `behavior exceptions` gate is required before group-play override, temperament exception, incident reinstatement/suspension change, aggression/anxiety handling exception, or sensitive behavior message.
- Any stale/unknown/conflicting availability, vaccine, payment, staffing, holiday, behavior, or special-care evidence blocks `confirmed` and customer availability promises.

### State-to-output mapping

| Deterministic finding | Recommended status | Lifecycle effect |
| --- | --- | --- |
| Complete request, all hard rules pass, fresh capacity/staffing/payment/vaccine evidence, no exceptions | `ready_for_staff_approval` | Stay in `booking_request` until staff executes `offered` or `confirmed`. |
| Missing routine facts or stale/conflicting source facts | `missing_info` | Route lifecycle to `missing_info` until facts are provided/reconciled. |
| Vaccine proof missing/unverified/expired/pending | `vaccine_pending` | Route lifecycle to `vaccine_pending`; no confirmation/check-in/group play clearance. |
| Care/medical/medication/behavior/payment/holiday/staff/capacity/provider conflict or exception | `special_review` | Route lifecycle to `special_review`; execution waits for named gate. |
| Capacity/staffing/holiday posture blocks immediate offer but waitlist is allowed | `waitlisted` | Lifecycle may become `waitlisted` only through provider/staff-approved path. |
| Prior staff-approved offer exists | `offered` | Lifecycle can be `offered`; confirmation remains gated. |
| Prior provider/approval evidence already confirms | `confirmed` | Reflect existing provider/execution state; AI did not create it. |
| Deterministic hard-denial or human rejection exists | `rejected` | Reflect approved denial path; customer message remains approval-gated. |
| Critical source/policy/schema conflict prevents safe recommendation | `failed_safely` | Do not change lifecycle; create review/data-cleanup task. |

## Deterministic rules

The booking triage agent must run deterministic hard rules before any AI explanation, recommendation, or draft message. Each rule consumes typed snapshots and policy refs; it must never infer missing policy values, invent availability, assign inventory, overbook, release holds, waive payment rules, or override hard policy.

### Rule contract

Every deterministic rule emits:

- `rule_id`: stable identifier from the table below.
- `decision`: `pass`, `hard_block`, `needs_human_approval`, `unknown`, or `not_applicable`.
- `readiness_bucket`: `ready_for_staff_approval`, `missing_info`, `vaccine_pending`, `special_review`, `waitlisted`, `offered`, `confirmed`, or `rejected` when safe. `confirmed` and `rejected` are only valid when an approved execution path or prior human/provider state already exists.
- `evidence_refs`: typed source refs used by the rule, such as request id, policy version, availability snapshot id, vaccine document id, payment reference, staff coverage snapshot id, incident/restriction id, or manager approval id.
- `failure_code`: stable reason for `hard_block`, `needs_human_approval`, or `unknown`.
- `human_approval_required`: `none`, `staff_approval`, `manager_approval`, `medical_document_review`, `behavior_review`, `care_team_approval`, `payment_manager_approval`, or `customer_message_approval`.
- `safe_agent_actions`: evidence summary, internal task draft, manager packet draft, customer-safe script draft, or missing-info request draft.

Decision semantics:

- `pass`: this rule found no blocker. It does not confirm the booking.
- `hard_block`: policy or feasibility evidence definitively blocks the requested action. AI may explain and draft internal/customer-safe language, but rejection or provider mutation still requires the approved rejection/execution path.
- `needs_human_approval`: the request may be possible only through a named human gate.
- `unknown`: required evidence is missing, stale, conflicting, unmapped, or unsupported. Unknown never becomes a pass.
- `not_applicable`: the rule does not apply to the requested service/species/date/pet set.

### Hard blocking rules

| Rule | Deterministic inputs | Pass condition | Hard block / unknown cases | Human approval before action |
| --- | --- | --- | --- | --- |
| `date_range_and_service_supported` | Request id/source/status, location id/timezone, service kind, requested start/end, operating calendar, service catalog/policy version. | Service is offered at the location and requested dates/window are valid local operating dates. Boarding has a positive stay range. | `unknown` when service catalog, timezone, policy version, or requested dates are missing/stale/conflicting. `hard_block` when service is not offered, date range is invalid, or requested window is outside policy. | Staff/manager approval for any exception, unsupported service alternative, or customer-facing denial. |
| `accommodation_availability` | Accommodation request, species, nightly inventory by accommodation/care lane, booked/held/closed/turnover counts, waitlist posture, snapshot freshness/evidence refs. | Every requested night has an available, compatible accommodation or care lane under current policy and freshness rules. | `unknown` when availability snapshot is absent, stale, conflicting, not mapped by accommodation, or policy refs are missing. `hard_block` when required accommodation is full, closed, held, out of service, unsupported for species, or the stay would exceed capacity. | Staff approval before offering/holding/assigning inventory. Manager approval for over-capacity, hold release, or waitlist promotion. |
| `size_capacity_room_or_group_fit` | Pet species/size/age/sex/spay-neuter, accommodation dimensions/classes, size-capacity policy, group/play-lane roster, compatibility policy, temperament/incident evidence. | Pet fits the requested room/run/suite/condo or group/care lane without violating size capacity, species, safety, or compatibility policy. | `unknown` when size, species, room class, compatibility, or temperament evidence is missing/stale. `hard_block` when pet cannot safely fit the requested room/group or requested group play is policy-ineligible. | Behavior/staff/manager approval for group-play exceptions, room-fit exceptions, split-room decisions, or alternate care-lane acceptance. |
| `service_capacity_and_addons` | Service catalog, add-on catalog, grooming/training/daycare/boarding capacity, staff skill coverage, add-on availability, package/membership applicability. | Requested primary service and add-ons are offered, capacity exists for the date/window, and required staff/equipment/space are available. | `unknown` when catalog/add-on availability/staff skill data is missing or stale. `hard_block` when add-on is unavailable, service capacity is full, equipment/space is closed, or package does not apply. | Staff approval before adding services; manager approval for unsupported add-ons, package exceptions, or capacity exceptions. |
| `vaccine_requirements` | Species, service kind, vaccine policy version, required vaccine list, document source, licensed-vet proof, verification/review status, expiration dates. | Required vaccines/documents are present, verified, unexpired for the requested stay/window, and policy-matched. | `unknown` when proof is missing, OCR-only, unverified, unmapped, expiry is unclear, or policy version is missing. `hard_block` when required vaccine is expired/missing under policy and no approved vaccine-pending path exists. | Medical/document review before treating proof as accepted. Manager/medical approval for vaccine-pending handling or exceptions. |
| `vaccine_pending_handling` | Vaccine review status, upload/request timestamps, local pending policy, due dates, customer communication state, requested service/date proximity. | Pending state is allowed by policy and routed to `vaccine_pending` with a document-review task before any confirmation. | `unknown` when pending policy or review owner is missing. `hard_block` when policy forbids pending reservations for the service/date or deadline has passed. | Medical/document review and, if customer-facing, customer-message approval. AI cannot final-approve vaccine documents. |
| `deposit_and_pricing_requirements` | Price/deposit policy version, quote/estimate, taxes/fees/surcharges, deposit amount/due/refundability, payment/deposit status, payment reference, customer/package context. | Required price/deposit values are policy-backed and payment/deposit state satisfies the action being considered. | `unknown` when pricing/deposit policy, quote, payment reference, reconciliation, or package applicability is missing/conflicting. `hard_block` for unpaid/failed/late/disputed/partial required deposit where policy requires collection before offer/confirmation/check-in. | Payment/manager approval for waiver, refund, forfeiture, discount, credit, manual reconciliation, or policy exception. AI cannot charge/refund/waive or alter policy. |
| `holiday_blackout_minimum_stay` | Holiday/peak/local-event calendar, blackout/manager-hold intervals, minimum-stay rules, deposit/cancellation overlays, capacity-hold policy, requested dates/nights. | Requested stay satisfies holiday/peak restrictions, blackout rules, minimum stay, and holiday deposit/cancellation overlays. | `unknown` when holiday calendar, blackout window, minimum-stay policy, or overlay policy is missing/stale. `hard_block` when dates are blacked out, manager-held, below minimum stay, or violate holiday restriction. | Manager approval for blackout/holiday exceptions, minimum-stay exceptions, waitlist promotion, or customer-facing denial. |
| `staff_coverage_constraints` | Staff schedule, roles/qualifications, shift windows, breaks/absences, lane assignments, ratio policy, expected pet counts, care-load clusters, turnover/late-departure backlog. | Coverage meets ratio, qualification, lane, turnover, and special-care load requirements for the requested service/window. | `unknown` when schedule, ratio policy, staff qualifications, or expected counts are missing/stale. `hard_block` when staff coverage is insufficient or required qualified staff are unavailable. | Manager/staff approval for staffing exceptions, reduced capacity, or alternate staffing plan. |
| `behavior_restrictions` | Temperament profile, group-play observation, behavior observations, bite/human selectivity/escape/guarding history, incident restrictions, manager-review flags, service request. | No policy hard stop exists for the requested service/lane, or the request is routed to an allowed individual/alternate care path. | `unknown` when temperament or incident evidence is missing/stale. `hard_block` for unresolved suspending incident, bite/aggression restriction, prohibited group-play condition, or manager-review flag that blocks the requested lane. | Behavior review/manager approval for behavior exceptions, group-play override, reinstatement after suspension, or sensitive owner-facing language. |
| `anxiety_aggression_exception_handling` | Anxiety/stress notes, aggression/bite history, handling plan, staff safety constraints, prior approvals, room/care-lane fit, incident history. | Existing approved handling plan supports the requested service without exceeding staff/room/care limits. | `unknown` when notes are vague, approval is absent/stale, or handling plan is not source-backed. `hard_block` when aggression/anxiety risk exceeds accepted policy for the requested service. | Behavior/manager/care-team approval before accepting exception, changing restrictions, or sending sensitive message. |
| `medication_special_care_limits` | Medication name/dose/schedule/source/review status, feeding/allergy/medical notes, special-care needs, vet/emergency contacts, staff qualification/capacity, isolation/mobility needs. | Care requirements are reviewed, executable, within service/staff limits, and supported by current policy. | `unknown` when medication/care notes are ambiguous, source is vague, vet clarification is needed, or special-care capacity is missing. `hard_block` when requested care exceeds policy/staff capability or required review is incomplete. | Care-team/manager/vet-document approval for special-care acceptance, medication ambiguity, medical exception, or customer-facing medical language. |
| `multi_pet_constraints` | Full pet set, per-pet service/accommodation/care/vaccine/behavior/payment outcomes, same-room/split-room policy, household compatibility, species/size/sex/spay-neuter facts. | Every pet independently passes required hard rules and the combined group fits room/care/staff/payment policy. | `unknown` when any pet lacks required evidence or same-room/split-room policy is missing. `hard_block` when one pet blocks the booking, household grouping is unsafe, or requested shared accommodation is unsupported. | Staff/manager approval for split-room alternatives, partial booking, household compatibility exception, or customer-facing offer/denial. |
| `late_pickup_checkout_impact` | Requested pickup/checkout window, location late checkout/pickup policy, same-day arrivals, room turnover backlog, staffing, surcharge/add-on policy candidate, payment requirements. | Requested timing fits operating and turnover policy without blocking next arrival or staff coverage. | `unknown` when late policy, turnover state, staffing, or surcharge/deposit impact is missing. `hard_block` when late pickup/checkout violates policy, blocks turnover, or lacks required staff/payment coverage. | Staff/manager approval for extended checkout, surcharge/fee handling, room hold/release, or customer-facing promise. |

### Soft risk flags

Soft risk flags never override hard rules. They add context to the manager/staff packet after hard-rule evaluation:

- Near-capacity stay, high occupancy, or limited room-type flexibility where a pass exists but operational risk is elevated.
- Holiday/peak demand period where policy allows the stay but deposits, cancellation terms, or staffing should be double-checked.
- First-time customer, incomplete preference history, unclear preferred contact channel, or possible duplicate customer/pet record.
- Expiring-soon vaccine that remains valid for the requested stay but may need proactive reminder.
- Mild anxiety, special feeding preference, senior pet, mobility note, or owner concern that is within normal service limits.
- Multiple add-ons or complex package/membership context that passes policy but may need front-desk explanation.
- Late pickup risk that is still within policy but may stress turnover or staffing.
- Multi-pet stay that passes room/care rules but requires staff awareness, split-care instructions, or explicit customer explanation.

Soft flags may create internal tasks, draft reminders, or appear in staff notes. They must not become autonomous rejection, autonomous confirmation, autonomous customer promises, or autonomous policy exceptions.

### Failure and unknown handling

- Missing input: return `unknown` with `failure_code = missing_required_input`, list missing typed fields, and route to `missing_info` or the relevant review bucket.
- Stale source: return `unknown` with `failure_code = stale_snapshot`; do not reuse old capacity, vaccine, payment, or staff evidence for availability promises.
- Conflicting source: return `unknown` with `failure_code = conflicting_source`; include both evidence refs and route to staff/manager review.
- Unsupported mapping: return `unknown` with `failure_code = unmapped_provider_value`; quarantine raw provider values and request adapter/policy mapping.
- Policy gap: return `unknown` with `failure_code = missing_policy`; do not invent local policy for holiday dates, deposit amounts, blackout windows, room counts, ratios, special-care limits, or late checkout rules.
- Hard policy denial: return `hard_block`; AI may draft a manager packet or customer-safe script, but actual rejection/status update requires the approved rejection path.
- Exception path: return `needs_human_approval`; include the named approval gate and evidence needed for the approver.

### Required approval gates

These outcomes require human approval before customer-visible action or provider mutation:

- Confirming a booking, updating live provider status, holding/releasing/assigning rooms, overbooking, or promoting from waitlist: staff-approved execution path, with manager approval for exceptions.
- Rejecting a request: human/manager-approved rejection path or explicitly encoded deterministic hard-denial path approved for automation.
- Accepting special-care, medication, medical, or vaccine-pending exceptions: care-team/medical document/manager approval as applicable.
- Behavior exceptions, group-play overrides, incident reinstatement/suspension changes, or aggression/anxiety handling exceptions: behavior review and manager approval.
- Charging, refunding, waiving, discounting, forfeiting, reconciling, or altering payment/deposit policy: payment/manager approval.
- Customer-facing messages about medical, vaccine, incident, behavior, safety, payment, policy exceptions, rejection, or availability promises: customer-message approval unless a pre-approved script exists for the exact deterministic state.

### Non-negotiable AI boundary

AI can explain deterministic outcomes, summarize evidence, surface missing/stale/conflicting inputs, draft internal tasks, draft manager packets, and draft customer-safe scripts. AI cannot invent availability, infer room counts, override capacity/staff/holiday/vaccine/payment/behavior/special-care policy, confirm a booking, reject a customer, promise a room/group/service, mutate provider records, or move money without an explicit approved execution path.


## Output schema

The booking triage agent returns `WorkflowResult<BookingTriageOutput>`. The outer workflow result carries the runtime status, summary, generic recommended actions, workflow-level risk flags, verification notes, and optional human review reason. `BookingTriageOutput` is the typed `structured_output` and must keep deterministic rule outputs separate from AI-authored explanations, drafts, and task wording.

Top-level JSON shape:

```json
{
  "schema_version": "booking_triage_output.v1",
  "request_ref": {
    "reservation_id": "res_12345",
    "location_id": "loc_42",
    "source_channel": "Portal",
    "source_evidence_refs": ["provider:g:reservation:12345"],
    "evaluated_at": "2026-06-11T13:30:00Z",
    "input_snapshot_refs": ["policy:loc_42:v3", "availability_snapshot:loc_42:2026-06-11T13:25Z"]
  },
  "deterministic_result": {
    "recommended_status": "missing_info",
    "status_reason_codes": ["missing_or_unverified_vaccine"],
    "approval_gates": ["confirmed_booking_automation", "medical_or_vaccine_document_review"],
    "rule_evaluations": [],
    "allowed_next_actions": ["request_missing_info", "create_internal_task"],
    "blocked_actions": []
  },
  "ai_assistive_output": {
    "missing_info": [],
    "risk_flags": [],
    "availability_summary": {},
    "recommended_room_or_group": null,
    "draft_customer_message": null,
    "internal_tasks": []
  },
  "validation": {
    "confidence": "medium",
    "unknowns": [],
    "provenance": []
  }
}
```

### Top-level required fields

| Field | Type | Required | Owner | Validation |
| --- | --- | --- | --- | --- |
| `schema_version` | string enum | yes | deterministic | Must equal `booking_triage_output.v1`; reject unknown major versions. |
| `request_ref` | object | yes | deterministic | Source/request identity and evidence refs only; no raw provider JSON or unredacted staff notes. |
| `deterministic_result` | object | yes | deterministic rules/validator | Only policy, typed source, prior approval, and freshness outputs. LLM text must not populate or upgrade it. |
| `ai_assistive_output` | object | yes | AI | Explanations, summaries, draft customer copy, and draft staff tasks. Advisory only. |
| `validation` | object | yes | validator | Provenance, confidence, and unknown markers used to prevent generated prose from masquerading as confirmed facts. |

### `request_ref`

```ts
type RequestRef = {
  reservation_id?: string;
  external_request_id?: string;
  customer_id?: string;
  pet_ids: string[];                   // min 1 when pets are known
  location_id: string;
  source_channel: "Portal" | "WebsiteForm" | "PhoneTranscript" | "Sms" | "Email" | "StaffCreated" | "ProviderImport" | "Unknown";
  source_evidence_refs: string[];      // min 1 unless source_channel = "Unknown"
  evaluated_at: string;                // RFC3339 UTC
  input_snapshot_refs: string[];       // typed policy/provider/availability/payment snapshots used
}
```

Rules: `evaluated_at` must be at or after included snapshot timestamps; `source_channel = "Unknown"` requires a matching `missing_info` or `validation.unknowns` marker; refs point to quarantined/redacted sources and never embed raw provider payloads.

### `deterministic_result`

Authoritative rule output. Produced by application policy code or a schema validator, not by generated prose.

```ts
type DeterministicResult = {
  recommended_status: RecommendedStatus;
  status_reason_codes: StatusReasonCode[];
  approval_gates: ApprovalGate[];
  rule_evaluations: RuleEvaluation[];
  allowed_next_actions: AllowedNextAction[];
  blocked_actions: BlockedAction[];
}
```

`recommended_status` is required and must be one of:

- `ready_for_staff_approval` — deterministic checks pass, but staff approval/provider execution is still required before confirmation.
- `missing_info` — collectable customer/front-desk information or documents are absent.
- `vaccine_pending` — vaccine/document proof is missing, unverified, expired, stale, or requires document/medical review.
- `special_review` — care, medical, medication, behavior, incident, staffing, capacity, payment, holiday, unsupported service, or policy exception requires a human owner.
- `waitlisted` — current capacity/staffing/holiday posture blocks immediate offer but a waitlist path exists.
- `offered` — a staff-approved offer exists and may be pending deposit or confirmation.
- `confirmed` — valid only when prior approval/execution evidence or already-confirmed provider state is present; AI must never independently output a new confirmed booking.
- `rejected` — valid only with deterministic hard-denial policy evidence or human/manager approval.
- `failed_safely` — stale/conflicting critical source, schema failure, unsupported request, or provider inconsistency prevents a trustworthy recommendation.

`StatusReasonCode` enum values: `missing_customer_contact`, `missing_pet_identity`, `missing_service_or_dates`, `missing_or_unverified_vaccine`, `medical_or_medication_review`, `feeding_or_allergy_ambiguity`, `behavior_review_required`, `incident_or_safety_review`, `group_play_ineligible`, `special_care_review_required`, `capacity_available_pending_approval`, `capacity_limited_or_full`, `availability_snapshot_stale`, `provider_conflict`, `unsupported_service_or_accommodation`, `staffing_ratio_review`, `holiday_or_blackout_review`, `deposit_or_payment_review`, `manager_policy_exception_required`, `prior_human_approval_present`.

`ApprovalGate` enum values: `confirmed_booking_automation`, `rejection`, `special_care_acceptance`, `behavior_exception`, `medical_or_vaccine_document_review`, `payment_or_deposit_exception`, `capacity_or_overbooking_exception`, `customer_message_approval`, `provider_write_approval`.

Required gate rules:

- `confirmed` requires `confirmed_booking_automation` approval provenance or an already-confirmed provider state.
- `rejected` requires the `rejection` gate unless a deterministic hard-denial rule evaluation is present.
- Accepting medication ambiguity, medical/special handling, isolation, special room fit, or care-load exceptions requires `special_care_acceptance`.
- Any group-play override, bite/selectivity override, incident reinstatement, or behavior restriction exception requires `behavior_exception`.
- Customer-facing rejection, medical/vaccine judgment, incident/behavior/safety language, payment exception, or availability promise requires `customer_message_approval`.

```ts
type RuleEvaluation = {
  rule_id: string;
  rule_family: "identity" | "service_dates" | "vaccine" | "medical_care" | "behavior" | "capacity" | "staffing" | "payment" | "holiday" | "provider_integrity";
  outcome: "pass" | "fail" | "needs_review" | "unknown" | "not_applicable";
  subject_ref?: string;
  reason_code: StatusReasonCode;
  evidence_refs: string[];             // min 1 unless outcome = "unknown"
  policy_ref?: string;
  observed_at?: string;                // RFC3339 when source-backed
  freshness: "fresh" | "stale" | "unknown" | "not_time_sensitive";
}
```

`AllowedNextAction` enum values: `draft_customer_message`, `create_internal_task`, `request_missing_info`, `request_human_review`, `prepare_manager_packet`, `prepare_provider_update_for_approval`, `no_action`.

```ts
type BlockedAction = {
  action: "confirm_booking" | "reject_booking" | "accept_special_care" | "override_behavior_policy" | "promise_availability" | "assign_room_or_group" | "release_or_hold_capacity" | "charge_or_refund" | "send_customer_message" | "provider_write";
  blocked_until: ApprovalGate | "missing_info_resolved" | "fresh_snapshot_available" | "policy_source_added";
  reason_code: StatusReasonCode;
}
```

### `ai_assistive_output`

The required fields below are generated or assembled by the agent from typed facts. They are advisory; validators must reject outputs that conflict with `deterministic_result`.

#### `missing_info`

Required array. Empty when no missing information is known.

```ts
type MissingInfoItem = {
  item_id: string;
  field: "customer_contact" | "pet_identity" | "service_dates" | "vaccine_document" | "vaccine_expiration" | "medication_name" | "medication_dose" | "medication_schedule" | "feeding_instruction" | "allergy_detail" | "medical_condition" | "temperament_evidence" | "incident_resolution" | "spay_neuter_status" | "deposit_status" | "room_preference" | "multi_pet_rooming_preference" | "policy_source" | "provider_snapshot";
  subject_ref?: string;
  required_for: "triage" | "confirmation" | "group_play" | "special_care" | "payment" | "customer_message";
  severity: "blocks_automation" | "blocks_staff_decision" | "nice_to_have";
  prompt_for_customer?: string;
  internal_note?: string;
  evidence_refs: string[];
}
```

`prompt_for_customer` must ask for information only; it must not diagnose, reject, approve vaccines, approve group play, or promise availability.

Example: `{"item_id":"missing-vaccine-rabies-pet_9","field":"vaccine_document","subject_ref":"pet:pet_9","required_for":"confirmation","severity":"blocks_automation","prompt_for_customer":"Could you upload the most recent rabies vaccination record for Maple? Our team will review it before confirming the stay.","evidence_refs":["policy:vaccine:loc_42:v3"]}`.

#### `risk_flags`

Required array. Human-readable risk projections backed by deterministic rules or source-integrity markers.

```ts
type RiskFlag = {
  flag: "vaccine_missing_or_unverified" | "medical_review" | "medication_ambiguous" | "feeding_allergy_ambiguity" | "behavior_review" | "incident_safety_review" | "group_play_ineligible" | "special_care_load" | "capacity_full_or_limited" | "stale_availability" | "provider_conflict" | "staffing_ratio" | "holiday_blackout" | "deposit_payment" | "sensitive_customer_message";
  severity: "info" | "review" | "blocker";
  subject_ref?: string;
  source: "deterministic_rule" | "source_integrity" | "ai_summary";
  evidence_refs: string[];
  explanation?: string;
}
```

A blocker risk must force `recommended_status` to `missing_info`, `vaccine_pending`, `special_review`, `waitlisted`, `rejected`, or `failed_safely`; it must not coexist with `confirmed` unless prior human approval evidence resolves it.

#### `availability_summary`

Required object. It summarizes capacity evidence without becoming a promise.

```ts
type AvailabilitySummary = {
  posture: "available_pending_staff_approval" | "limited" | "full" | "waitlist_available" | "closed_or_blackout" | "unsupported" | "unknown" | "conflicting";
  service_kind: "Boarding" | "DayPlay" | "DayBoarding" | "Grooming" | "Training" | "DaySpa" | "Unknown";
  date_range_label?: string;
  capacity_basis: "room_inventory" | "playgroup_roster" | "care_lane" | "staffing_ratio" | "holiday_policy" | "provider_state" | "not_evaluated";
  freshness: "fresh" | "stale" | "unknown";
  source_snapshot_refs: string[];
  confidence: "high" | "medium" | "low" | "unknown";
  summary_text: string;
}
```

Rules: `available_pending_staff_approval` must be phrased as pending staff approval, never “confirmed,” “reserved,” “held,” or “guaranteed”; stale/unknown freshness forces low/unknown confidence and blocks `confirmed`; missing/conflicting capacity maps to `unknown` or `conflicting`, not optimistic availability.

#### `recommended_room_or_group`

Nullable object. Use `null` when room/group fit is not evaluated, unavailable, stale, unsafe, or requires an ungranted human decision. It is a recommendation candidate, never an assignment.

```ts
type RecommendedRoomOrGroup = {
  recommendation_type: "room_candidate" | "run_candidate" | "suite_candidate" | "cat_condo_candidate" | "playgroup_candidate" | "individual_enrichment_candidate" | "care_lane_candidate";
  candidate_label: string;
  candidate_ref?: string;
  fit_basis: "species" | "size" | "multi_pet" | "service_variant" | "temperament" | "medical_care" | "staffing" | "customer_preference" | "policy";
  confidence: "high" | "medium" | "low" | "unknown";
  requires_gate: ApprovalGate | null;
  evidence_refs: string[];
  caveat: string;
}
```

`requires_gate` must be `provider_write_approval` for any room/run/group provider update and `behavior_exception` when group-play eligibility is not a deterministic pass. Candidate wording must not imply assignment.

#### `draft_customer_message`

Nullable object. Use `null` when no customer response should be drafted or when the content should first become a manager packet.

```ts
type DraftCustomerMessage = {
  channel: "email" | "sms" | "portal" | "phone_script";
  audience: "customer" | "staff_to_customer";
  purpose: "request_missing_info" | "acknowledge_request" | "explain_pending_review" | "offer_waitlist" | "staff_approved_offer" | "manager_approved_rejection";
  body: string;
  sensitivity: "routine" | "policy" | "medical" | "behavior" | "incident" | "payment" | "rejection" | "availability";
  requires_approval: boolean;
  required_gate?: ApprovalGate;
  source_fact_refs: string[];
  unsafe_claims_absent: boolean;
}
```

Validation: medical/vaccine judgments, behavior, incidents, payment/deposit exceptions, rejection, availability promises, and confirmed booking language require approval; `staff_approved_offer` and `manager_approved_rejection` require approval provenance; reject bodies that claim “confirmed,” “guaranteed,” “approved for group play,” “medically cleared,” or “rejected” without matching provenance.

#### `internal_tasks`

Required array. Draft staff tasks only; they do not execute provider/payment mutations.

```ts
type InternalTask = {
  task_type: "front_desk_followup" | "vaccine_review" | "care_team_review" | "behavior_review" | "manager_review" | "capacity_recheck" | "staffing_review" | "payment_review" | "provider_data_cleanup";
  title: string;
  body: string;
  priority: "low" | "normal" | "high" | "urgent";
  owner_role: "front_desk" | "manager" | "care_team" | "behavior_lead" | "medical_document_reviewer" | "payments" | "operations";
  due_by?: string;
  related_refs: string[];
  source: "deterministic_rule" | "ai_draft_from_rule";
  requires_approval_before_completion?: ApprovalGate;
}
```

Task type, minimum priority, owner role, and approval gate must be consistent with deterministic reason codes. Tasks must not tell staff to administer medication without reviewed medication name/dose/schedule/pet/dates, or tell staff to confirm, reject, overbook, charge, refund, waive, release holds, or write provider status without approval.

### `validation`

```ts
type OutputValidation = {
  confidence: "high" | "medium" | "low" | "unknown";
  unknowns: UnknownMarker[];
  provenance: ProvenanceRef[];
  validator_notes?: string[];
}

type UnknownMarker = {
  field: string;
  reason: "not_provided" | "stale" | "conflicting_sources" | "unsupported_by_policy" | "not_evaluated";
  blocks: ("confirmation" | "rejection" | "special_care_acceptance" | "behavior_exception" | "customer_message" | "room_or_group_recommendation")[];
  evidence_refs: string[];
}

type ProvenanceRef = {
  ref: string;
  source_kind: "provider_snapshot" | "policy_snapshot" | "staff_note" | "customer_submission" | "document_review" | "availability_snapshot" | "payment_snapshot" | "prior_human_approval" | "derived_rule";
  observed_at?: string;
  freshness: "fresh" | "stale" | "unknown" | "not_time_sensitive";
  supports: string[];
}
```

Every non-null availability, room/group, customer-message, and risk field must cite provenance through evidence refs, source fact refs, or `validation.provenance.supports`. Unknowns that block confirmation, rejection, special-care acceptance, behavior exceptions, customer messages, or room/group recommendations must force `missing_info`, `vaccine_pending`, `special_review`, `waitlisted`, or `failed_safely`. `confidence = "high"` is only valid when critical snapshots are fresh, no blocking unknowns exist, and no unresolved provider conflicts exist; generated explanations cannot upgrade confidence.

### Cross-field validation rules

1. `recommended_status = "confirmed"` requires prior approval/execution provenance or an already-confirmed provider state and must not include `confirm_booking` in `blocked_actions`.
2. `recommended_status = "rejected"` requires the `rejection` approval gate or a deterministic hard-denial rule; customer rejection drafts require approval unless prior rejection approval is present.
3. Any `unknown` or `needs_review` outcome for medical, medication, special care, behavior, incident, capacity, staffing, payment, holiday, or provider integrity must route to `special_review`, `missing_info`, `vaccine_pending`, `waitlisted`, or `failed_safely`.
4. `availability_summary.posture` of `unknown`, `conflicting`, `limited`, `full`, `closed_or_blackout`, or `unsupported` requires `recommended_room_or_group = null` unless clearly marked as a staff-review candidate and no availability promise appears.
5. Playgroup candidates require deterministic group-play pass; non-dog species, intact/unknown spay-neuter where policy blocks play, bite/selectivity evidence, stale temperament evidence, or unresolved incident restrictions require `behavior_exception` or no candidate.
6. Customer drafts with `medical`, `behavior`, `incident`, `payment`, `rejection`, or `availability` sensitivity must set `requires_approval = true` unless matching prior approval is present.
7. No field may include raw provider JSON, payment instrument details, unredacted internal staff notes, or unsupported policy interpretations.
8. `source_channel = "Unknown"` requires `missing_info` or an `UnknownMarker` that blocks confirmation.

### Complete example

```json
{
  "schema_version": "booking_triage_output.v1",
  "request_ref": {
    "reservation_id": "res_12345",
    "customer_id": "cust_77",
    "pet_ids": ["pet_9"],
    "location_id": "loc_42",
    "source_channel": "Portal",
    "source_evidence_refs": ["provider:g:reservation:12345"],
    "evaluated_at": "2026-06-11T13:30:00Z",
    "input_snapshot_refs": ["policy:loc_42:v3", "availability_snapshot:loc_42:2026-06-11T13:25Z"]
  },
  "deterministic_result": {
    "recommended_status": "missing_info",
    "status_reason_codes": ["missing_or_unverified_vaccine", "medical_or_medication_review"],
    "approval_gates": ["confirmed_booking_automation", "medical_or_vaccine_document_review", "customer_message_approval"],
    "rule_evaluations": [
      {"rule_id":"vaccine.required.rabies.loc_42.v3","rule_family":"vaccine","outcome":"needs_review","subject_ref":"pet:pet_9","reason_code":"missing_or_unverified_vaccine","evidence_refs":["policy:loc_42:v3"],"policy_ref":"policy:loc_42:v3","freshness":"not_time_sensitive"},
      {"rule_id":"medication.executable_details.required","rule_family":"medical_care","outcome":"unknown","subject_ref":"pet:pet_9","reason_code":"medical_or_medication_review","evidence_refs":[],"freshness":"unknown"}
    ],
    "allowed_next_actions": ["request_missing_info", "create_internal_task", "draft_customer_message"],
    "blocked_actions": [
      {"action":"confirm_booking","blocked_until":"missing_info_resolved","reason_code":"missing_or_unverified_vaccine"},
      {"action":"provider_write","blocked_until":"provider_write_approval","reason_code":"missing_or_unverified_vaccine"}
    ]
  },
  "ai_assistive_output": {
    "missing_info": [
      {"item_id":"missing-vaccine-rabies-pet_9","field":"vaccine_document","subject_ref":"pet:pet_9","required_for":"confirmation","severity":"blocks_automation","prompt_for_customer":"Could you upload Maple's current rabies vaccination record? Our team will review it before confirming the stay.","evidence_refs":["policy:loc_42:v3"]},
      {"item_id":"missing-medication-dose-pet_9","field":"medication_dose","subject_ref":"pet:pet_9","required_for":"special_care","severity":"blocks_staff_decision","prompt_for_customer":"Could you confirm Maple's medication dose and schedule for the requested stay?","evidence_refs":[]}
    ],
    "risk_flags": [
      {"flag":"vaccine_missing_or_unverified","severity":"blocker","subject_ref":"pet:pet_9","source":"deterministic_rule","evidence_refs":["policy:loc_42:v3"],"explanation":"Required vaccine proof still needs staff review."}
    ],
    "availability_summary": {
      "posture":"available_pending_staff_approval",
      "service_kind":"Boarding",
      "date_range_label":"June 21-24, 2026",
      "capacity_basis":"room_inventory",
      "freshness":"fresh",
      "source_snapshot_refs":["availability_snapshot:loc_42:2026-06-11T13:25Z"],
      "confidence":"medium",
      "summary_text":"A compatible boarding accommodation appears available in the latest snapshot, pending staff approval and no provider changes. This is not a confirmed booking."
    },
    "recommended_room_or_group": null,
    "draft_customer_message": {
      "channel":"email",
      "audience":"staff_to_customer",
      "purpose":"request_missing_info",
      "body":"Thanks for your boarding request for Maple. Before our team can review the booking, could you upload Maple's current rabies record and confirm the medication dose and schedule? Once we have those details, staff can review the stay and follow up.",
      "sensitivity":"medical",
      "requires_approval":true,
      "required_gate":"customer_message_approval",
      "source_fact_refs":["missing-vaccine-rabies-pet_9", "missing-medication-dose-pet_9"],
      "unsafe_claims_absent":true
    },
    "internal_tasks": [
      {"task_type":"vaccine_review","title":"Review Maple's vaccine record before booking confirmation","body":"Customer needs to provide current rabies proof. Confirm against location vaccine policy before any booking confirmation.","priority":"high","owner_role":"medical_document_reviewer","related_refs":["pet:pet_9", "reservation:res_12345"],"source":"ai_draft_from_rule","requires_approval_before_completion":"medical_or_vaccine_document_review"}
    ]
  },
  "validation": {
    "confidence": "medium",
    "unknowns": [{"field":"pets[pet_9].medication.dose","reason":"not_provided","blocks":["confirmation", "special_care_acceptance"],"evidence_refs":[]}],
    "provenance": [
      {"ref":"policy:loc_42:v3","source_kind":"policy_snapshot","freshness":"not_time_sensitive","supports":["deterministic_result.rule_evaluations[0]", "ai_assistive_output.missing_info[0]"]},
      {"ref":"availability_snapshot:loc_42:2026-06-11T13:25Z","source_kind":"availability_snapshot","observed_at":"2026-06-11T13:25:00Z","freshness":"fresh","supports":["ai_assistive_output.availability_summary"]}
    ],
    "validator_notes": ["No field confirms the booking or assigns a room/group."]
  }
}
```

## Test scenarios

Use these scenarios as deterministic fixtures for `booking.triage_needed` tests. Each fixture should be represented with typed snapshots, evidence refs, policy versions, and redacted provider refs rather than raw provider payloads.

Common assertions for every scenario:

- The agent may recommend a readiness/status bucket and draft explanations/tasks; it must not set provider reservation status, hold/release capacity, confirm, reject, charge, refund, waive, or send customer messages.
- `recommended_status` is a booking-triage readiness recommendation, not an executed reservation status.
- Any missing, stale, or conflicting source fact must appear in `missing_info` or `risk_flags`; the agent must not smooth over uncertainty.
- Customer-facing content is a `draft_message` only unless an explicit deterministic send policy and all required approvals are present. Sensitive health, medication, behavior, payment, rejection, exception, or availability language remains draft-only pending human approval.
- Availability-sensitive scenarios must cite `availability_snapshot.snapshot_id`, `observed_at`, freshness, and capacity posture. If availability is full, stale, unknown, held, or conflicting, the agent must recommend waitlist/review rather than inventing availability.

### Scenario BT-001: Easy booking

Purpose: happy-path boarding request where all deterministic checks pass, while preserving the staff approval boundary before confirmation/provider write-back.

Minimal input fixture:

```yaml
request:
  reservation_id: res_easy_001
  source: Portal
  location_id: loc_001
  service_kind: Boarding
  dates: {check_in: 2026-07-13, check_out: 2026-07-16, nights: 3}
  pets: [pet_maple]
  add_ons: [individual_play]
customer:
  customer_id: cus_001
  preferred_contact: email
pets:
  - pet_id: pet_maple
    species: Dog
    age_weeks: 156
    sex: Female
    spay_neuter: Spayed
    profile_complete: true
    vaccines: {required_verified_current: true, policy_version: vacc_v1}
    temperament: {group_play_eligible: true, no_restrictions: true, evidence_current: true}
    care: {medications: [], allergies: [], medical_flags: []}
policy_snapshot:
  policy_id: booking_policy_v1
  deposit: {required: false}
  holiday: {is_holiday_period: false}
availability_snapshot:
  snapshot_id: cap_easy_001
  observed_at: 2026-06-11T09:00:00-05:00
  freshness: fresh
  dog_boarding: {available: 4, held: 0, closed: 0, waitlist_open: true}
  staffing: {coverage_decision: sufficient}
payment_snapshot:
  deposit_status: not_required
```

Deterministic rule results:

- Pet profile and policy snapshot are complete enough for `ReadyForPolicyDecision`.
- Required vaccine proof is verified/current.
- No medical, medication, behavior, age, in-heat, species, or spay/neuter hard stop.
- Fresh capacity and staffing evidence show request can be offered subject to staff approval.
- No holiday/deposit/payment block.

Expected output:

- `recommended_status`: `ready_for_staff_approval`.
- `missing_info`: `[]`.
- `risk_flags`: `[]` or only non-blocking audit/minimization flags.
- Approval gates: `confirmed_booking_automation`, `provider_write_approval`, and `customer_message_approval` if staff sends the draft offer/confirmation copy.
- Message/task expectations:
  - Draft an internal staff approval packet summarizing dates, pet, vaccine evidence, capacity snapshot, and no detected hard stops.
  - Optionally draft a customer confirmation/offer message, but mark it `send_policy: RequiresApproval` or `DraftOnly` until staff approval and any provider write-back path are approved.
  - Must not output `confirmed` and must not claim a room/run/group was assigned unless that assignment exists as approved source evidence.

### Scenario BT-002: Full dates / no availability

Purpose: prove the AI recommends waitlist/review rather than inventing availability.

Minimal input fixture:

```yaml
request:
  reservation_id: res_full_001
  source: WebsiteForm
  location_id: loc_001
  service_kind: Boarding
  dates: {check_in: 2026-08-01, check_out: 2026-08-05, nights: 4}
  pets: [pet_luna]
pets:
  - pet_id: pet_luna
    species: Dog
    profile_complete: true
    vaccines: {required_verified_current: true}
    temperament: {no_restrictions: true, evidence_current: true}
policy_snapshot:
  policy_id: booking_policy_v1
availability_snapshot:
  snapshot_id: cap_full_001
  observed_at: 2026-06-11T09:00:00-05:00
  freshness: fresh
  dog_boarding: {available: 0, held: 3, closed: 1, waitlist_open: true}
  staffing: {coverage_decision: sufficient_for_current_roster_only}
payment_snapshot:
  deposit_status: not_required
```

Deterministic rule results:

- Eligibility/profile/vaccine checks pass.
- Capacity is fresh but full for at least one requested night; held/closed rooms cannot be treated as available.
- Waitlist is open; no denial needed from the fixture.

Expected output:

- `recommended_status`: `waitlisted`.
- `missing_info`: `[]` unless a waitlist preference/alternate date is required by local policy.
- `risk_flags`: [`capacity_full`, `availability_promise_prohibited`].
- Approval gates: `capacity_or_overbooking_exception` or staff/manager waitlist approval before waitlist promotion/offer, plus `customer_message_approval` for availability-sensitive customer language.
- Message/task expectations:
  - Create/draft an internal waitlist task with requested dates, service, and capacity snapshot id.
  - Customer message, if drafted, must avoid promising availability or a room. It may say staff will review options/waitlist, but remains draft-only if policy requires human review for waitlist language.
  - Must not recommend `ready_for_staff_approval`, `offered`, or `confirmed`.

### Scenario BT-003: Missing vaccines

Purpose: route missing or unverified vaccine proof to document/medical review and missing-info collection.

Minimal input fixture:

```yaml
request:
  reservation_id: res_vax_001
  source: Portal
  location_id: loc_001
  service_kind: DayPlay
  dates: {service_date: 2026-07-20}
  pets: [pet_rocket]
pets:
  - pet_id: pet_rocket
    species: Dog
    profile_complete: true
    vaccines:
      required_verified_current: false
      missing_required: [Rabies, Bordetella]
      document_review_status: no_document_on_file
    temperament: {group_play_eligible: true, evidence_current: true}
policy_snapshot:
  policy_id: daycare_policy_v1
availability_snapshot:
  snapshot_id: day_cap_vax_001
  observed_at: 2026-06-11T09:00:00-05:00
  freshness: fresh
  daycare: {available: 6}
```

Deterministic rule results:

- Missing required vaccine proof is a hard review route independent of capacity.
- AI/OCR cannot mark vaccines verified or approve group play.
- Capacity may be present but cannot override vaccine/document block.

Expected output:

- `recommended_status`: `vaccine_pending`.
- `missing_info`: [`Rabies proof`, `Bordetella proof`] with required source/document expectations.
- `risk_flags`: [`missing_required_vaccine_proof`, `document_review_required`, `eligibility_not_approved`].
- Approval gates: `medical_or_vaccine_document_review`, `confirmed_booking_automation` before any confirmation, and `customer_message_approval` for vaccine/eligibility-sensitive copy.
- Message/task expectations:
  - Draft a document-request task for front desk/customer success.
  - Customer message may request vaccine proof using approved template language, but must not say the booking/daycare is approved.
  - Must not create an executable check-in/group-play task.

### Scenario BT-004: Holiday deposit requirement

Purpose: route holiday/peak deposit and policy overlay without taking payment or promising a holiday booking.

Minimal input fixture:

```yaml
request:
  reservation_id: res_holiday_001
  source: PhoneTranscript
  location_id: loc_001
  service_kind: Boarding
  dates: {check_in: 2026-11-25, check_out: 2026-11-30, nights: 5}
  pets: [pet_biscuit]
pets:
  - pet_id: pet_biscuit
    species: Dog
    profile_complete: true
    vaccines: {required_verified_current: true}
    temperament: {no_restrictions: true, evidence_current: true}
policy_snapshot:
  policy_id: holiday_booking_policy_v1
  holiday:
    is_holiday_period: true
    name: Thanksgiving
    minimum_stay_met: true
  deposit:
    required: true
    amount_policy_ref: dep_holiday_2026
availability_snapshot:
  snapshot_id: cap_holiday_001
  observed_at: 2026-06-11T09:00:00-05:00
  freshness: fresh
  dog_boarding: {available: 2, held: 0, waitlist_open: true}
payment_snapshot:
  deposit_status: required_unpaid
  payment_reference: null
```

Deterministic rule results:

- Eligibility and fresh capacity are acceptable subject to human approval.
- Holiday policy requires deposit before confirmation/hold finalization.
- Unpaid deposit/payment-sensitive state blocks automatic confirmation.

Expected output:

- `recommended_status`: `special_review`.
- `missing_info`: [`holiday deposit payment/approval evidence`] when unpaid.
- `risk_flags`: [`holiday_policy_overlay`, `deposit_required_unpaid`, `payment_sensitive_content`].
- Approval gates: `payment_or_deposit_exception` for payment/deposit exception handling, `confirmed_booking_automation`/`provider_write_approval` before offer-to-confirm execution, and `customer_message_approval` for holiday/deposit/availability language.
- Message/task expectations:
  - Draft a payment/deposit follow-up task with policy refs and amount source, not raw card/payment data.
  - Customer message must be draft-only pending human/payment approval because it includes holiday/deposit and availability-sensitive language.
  - Must not charge, waive, refund, mark deposit paid, release/hold capacity, or confirm.

### Scenario BT-005: Multiple pets

Purpose: evaluate each pet separately and route multi-pet room-fit/split-stay ambiguity without collapsing pet-level facts.

Minimal input fixture:

```yaml
request:
  reservation_id: res_multi_001
  source: Portal
  location_id: loc_001
  service_kind: Boarding
  dates: {check_in: 2026-07-22, check_out: 2026-07-25, nights: 3}
  pets: [pet_ace, pet_nova]
  preferences: {same_room_requested: true}
pets:
  - pet_id: pet_ace
    species: Dog
    profile_complete: true
    vaccines: {required_verified_current: true}
    temperament: {no_restrictions: true, evidence_current: true}
  - pet_id: pet_nova
    species: Dog
    profile_complete: true
    vaccines: {required_verified_current: true}
    temperament: {no_restrictions: true, evidence_current: true}
policy_snapshot:
  policy_id: booking_policy_v1
  multi_pet: {same_room_requires_staff_fit_review: true}
availability_snapshot:
  snapshot_id: cap_multi_001
  observed_at: 2026-06-11T09:00:00-05:00
  freshness: fresh
  dog_boarding:
    standard_available: 2
    multi_pet_suite_available: 0
    split_room_available: true
```

Deterministic rule results:

- Pet-level vaccine/profile checks pass for both pets.
- Requested same-room fit is unavailable or requires staff fit review; split-room alternative may exist.
- Multi-pet accommodation cannot be inferred from aggregate dog capacity.

Expected output:

- `recommended_status`: `special_review`.
- `missing_info`: [`same-room vs split-room preference/approval`] if not already confirmed by customer/staff.
- `risk_flags`: [`multi_pet_room_fit_review`, `accommodation_not_promised`].
- Approval gates: `capacity_or_overbooking_exception` or staff room-fit approval for accommodation exceptions, `provider_write_approval` before room/provider updates, and `customer_message_approval` for offer/rooming language.
- Message/task expectations:
  - Draft internal task for staff to review multi-pet room fit and possible split-room alternative.
  - Draft customer question about same-room vs split-room only if policy permits; otherwise hold for staff.
  - Preserve pet-specific evidence and do not mark both pets approved based on only one pet's documents.

### Scenario BT-006: Medication needs

Purpose: route medication details to care-team review unless name/dose/schedule/source are complete and reviewed.

Minimal input fixture:

```yaml
request:
  reservation_id: res_med_001
  source: Email
  location_id: loc_001
  service_kind: Boarding
  dates: {check_in: 2026-07-18, check_out: 2026-07-21, nights: 3}
  pets: [pet_sadie]
pets:
  - pet_id: pet_sadie
    species: Dog
    profile_complete: true
    vaccines: {required_verified_current: true}
    temperament: {no_restrictions: true, evidence_current: true}
    care:
      medications:
        - name: "heart pill"
          dose: null
          schedule: "morning?"
          source: customer_email
          review_status: unreviewed
policy_snapshot:
  policy_id: care_policy_v1
availability_snapshot:
  snapshot_id: cap_med_001
  observed_at: 2026-06-11T09:00:00-05:00
  freshness: fresh
  dog_boarding: {available: 3}
  staffing: {coverage_decision: sufficient, medication_staff_available: unknown}
```

Deterministic rule results:

- Vaccine/profile/capacity do not clear medication ambiguity.
- Missing exact medication name/dose/schedule/reviewed source blocks executable medication tasks and automatic special-care acceptance.
- Medication-sensitive customer copy requires approval.

Expected output:

- `recommended_status`: `special_review`.
- `missing_info`: [`medication exact name`, `dose`, `administration schedule`, `vet/customer instruction source`, `care-team review`].
- `risk_flags`: [`medical_or_medication_ambiguity`, `special_care_review_required`, `customer_message_requires_review`].
- Approval gates: `special_care_acceptance`, `medical_or_vaccine_document_review` when medical/vet proof is needed, and `customer_message_approval`.
- Message/task expectations:
  - Draft care-team clarification/review task; task must be review/draft only, not executable medication administration.
  - Customer message asking for medication details must be draft-only pending approval because it concerns medical/medication instructions.
  - Must not state that medication care is accepted.

### Scenario BT-007: Anxiety

Purpose: route anxiety/special handling to staff/care review without treating it as automatic rejection.

Minimal input fixture:

```yaml
request:
  reservation_id: res_anxiety_001
  source: Portal
  location_id: loc_001
  service_kind: Boarding
  dates: {check_in: 2026-07-26, check_out: 2026-07-28, nights: 2}
  pets: [pet_milo]
pets:
  - pet_id: pet_milo
    species: Dog
    profile_complete: true
    vaccines: {required_verified_current: true}
    temperament:
      evidence_current: true
      group_play_status: needs_intro_observation
      notes: [separation_anxiety, crate_stress]
    care: {medications: [], allergies: [], medical_flags: []}
policy_snapshot:
  policy_id: behavior_care_policy_v1
availability_snapshot:
  snapshot_id: cap_anxiety_001
  observed_at: 2026-06-11T09:00:00-05:00
  freshness: fresh
  dog_boarding: {available: 2, quiet_area_available: unknown}
  staffing: {coverage_decision: sufficient, special_handling_capacity: limited}
```

Deterministic rule results:

- Anxiety and needs-intro observation require behavior/staff review for group play and care-lane fit.
- Boarding may still be possible, but special handling/quiet-area fit is unknown or limited.
- No autonomous behavior exception or care-lane assignment.

Expected output:

- `recommended_status`: `special_review`.
- `missing_info`: [`special handling plan`, `quiet-area/room-fit availability`, `staff behavior review`] when not already approved.
- `risk_flags`: [`anxiety_or_stress_handling`, `behavior_review_required`, `special_care_room_fit_unknown`].
- Approval gates: `special_care_acceptance`, `behavior_exception` if group play/behavior handling is affected, and `customer_message_approval` for handling-plan language.
- Message/task expectations:
  - Draft internal behavior/care review task summarizing source evidence and requested stay.
  - Customer-facing reassurance or handling-plan language must be draft-only pending staff approval.
  - Must not reject solely because anxiety exists, and must not promise quiet room/special handling unless source evidence says it is approved/available.

### Scenario BT-008: Aggression / behavior restriction

Purpose: route bite/aggression/restriction evidence to manager/behavior review and require draft-only customer language.

Minimal input fixture:

```yaml
request:
  reservation_id: res_behavior_001
  source: StaffCreated
  location_id: loc_001
  service_kind: DayPlay
  dates: {service_date: 2026-07-19}
  pets: [pet_tank]
  add_ons: [group_play]
pets:
  - pet_id: pet_tank
    species: Dog
    profile_complete: true
    vaccines: {required_verified_current: true}
    temperament:
      evidence_current: true
      group_play_status: restricted
      restrictions: [no_group_play, manager_review_required]
    incidents:
      - incident_id: inc_001
        type: bite_or_altercation
        status: unresolved_restriction
policy_snapshot:
  policy_id: behavior_policy_v1
availability_snapshot:
  snapshot_id: day_cap_behavior_001
  observed_at: 2026-06-11T09:00:00-05:00
  freshness: fresh
  daycare: {group_play_available: 8, individual_care_available: 1}
```

Deterministic rule results:

- Behavior restriction and unresolved incident override generic group-play capacity.
- AI cannot reinstate group play, accept a behavior exception, or reject/send sensitive customer language without approval.
- Alternative individual care may be reviewed if policy/source supports it.

Expected output:

- `recommended_status`: `special_review`; if local policy defines a deterministic hard-denial recommendation, route as `special_review` with rejection/denial approval required rather than autonomous `rejected`.
- `missing_info`: [`manager behavior decision`, `approved alternate care lane`] if not present.
- `risk_flags`: [`behavior_restriction`, `unresolved_incident`, `group_play_ineligible_or_unapproved`, `customer_message_requires_review`].
- Approval gates: `behavior_exception`, `rejection` if the request is denied, and `customer_message_approval` for behavior/safety/rejection language.
- Message/task expectations:
  - Draft manager review packet with incident/restriction refs and possible individual-care alternative if source-backed.
  - Any customer-facing explanation must be `DraftOnly` / `RequiresApproval` because it concerns behavior/safety/rejection risk.
  - Must not create a group-play assignment, mark eligibility approved, or send denial/restriction language.

### Scenario BT-009: Late pickup

Purpose: route late checkout/pickup requests through local policy and operations capacity without inventing fees or extended-care availability.

Minimal input fixture:

```yaml
request:
  reservation_id: res_late_001
  source: Sms
  location_id: loc_001
  service_kind: Boarding
  dates: {check_in: 2026-07-10, check_out: 2026-07-12, nights: 2}
  pets: [pet_olive]
  requested_late_pickup: {date: 2026-07-12, time: "18:30"}
pets:
  - pet_id: pet_olive
    species: Dog
    profile_complete: true
    vaccines: {required_verified_current: true}
    temperament: {no_restrictions: true, evidence_current: true}
policy_snapshot:
  policy_id: checkout_policy_v1
  late_pickup:
    policy_known: false
    approved_fee_schedule_ref: null
availability_snapshot:
  snapshot_id: ops_late_001
  observed_at: 2026-06-11T09:00:00-05:00
  freshness: fresh
  dog_boarding: {available_checkout_day: 0}
  operations: {turnover_backlog: high, late_departure_risk: high}
  staffing: {coverage_decision: limited_after_standard_checkout}
payment_snapshot:
  late_fee_status: unknown_or_not_assessed
```

Deterministic rule results:

- Base booking eligibility may pass, but late pickup policy/fee/coverage is unknown or risky.
- AI cannot invent a late pickup fee, waive/charge one, or promise extended care/room retention.
- Operations/staff review is required because late departure affects turnover/capacity.

Expected output:

- `recommended_status`: `special_review` for late-pickup/operations review; base stay may be separately `ready_for_staff_approval` only if represented as a distinct non-late-pickup alternative.
- `missing_info`: [`approved late pickup policy`, `late fee/surcharge policy ref`, `staff coverage decision`, `room turnover/extended-care approval`].
- `risk_flags`: [`late_pickup_policy_unknown`, `turnover_or_capacity_risk`, `payment_sensitive_content`].
- Approval gates: `capacity_or_overbooking_exception` for turnover/capacity exception handling, `payment_or_deposit_exception` for late-fee handling, and `customer_message_approval` if fees/availability are mentioned.
- Message/task expectations:
  - Draft staff task to review late pickup feasibility, policy, and fee source.
  - Customer SMS/email copy must be draft-only until staff confirms policy and availability; it must not quote unapproved fees or promise late pickup.
  - Must not mark the booking confirmed with late pickup included unless approved evidence exists.

## AI recommendation role

### Responsibility split: deterministic engine vs. AI interpretation

The booking triage workflow keeps policy, capacity, pricing, eligibility, and approval decisions deterministic and auditable. AI recommendations are an interpretation and drafting layer over typed rule outputs; they are not the source of truth for availability, prices, vaccines, room/group assignment, staff capacity, or provider/customer-visible state.

Deterministic engine responsibilities:

- Normalize typed request, customer, pet, provider, policy, availability, staffing, payment, and audit snapshots.
- Evaluate hard stops and review routes from versioned policy snapshots: vaccine/document requirements, age/spay-neuter/in-heat rules, group-play/behavior restrictions, incident flags, special-care/medication/medical limits, deposit/payment posture, holiday/blackout/minimum-stay rules, capacity/staffing fit, and provider conflicts.
- Emit typed rule results, readiness buckets, evidence refs, freshness decisions, policy versions, audit refs, and required approval gates.
- Enforce hard policy and execution gates for provider writes, customer-visible sends, payment actions, booking confirmation, rejection, special-care acceptance, and behavior exceptions.

AI interpretation/drafting responsibilities:

- Explain feasibility only from deterministic rule outputs and cited evidence.
- Identify missing, stale, conflicting, unmapped, or ambiguous inputs that block safe triage.
- Recommend next workflow steps: waitlist review, missing-info follow-up, document/vaccine review, care/medical/medication review, behavior/staff/manager review, payment review, capacity/staffing verification, or internal task creation.
- Draft customer and internal messages for review, including neutral missing-info requests, customer-safe waitlist language, staff task descriptions, and manager-review packets.
- Summarize uncertainty, confidence limits, source gaps, and the exact approval gate required before execution.

### Allowed AI actions

The AI recommendation layer may:

- Read the minimized booking triage packet and deterministic rule-result envelope.
- Summarize request facts, per-pet facts, policy matches, capacity/staffing posture, payment/deposit posture, and provider/source freshness.
- Explain why a deterministic result maps to `ready_for_staff_approval`, `missing_info`, `vaccine_pending`, `special_review`, `waitlisted`, `offered`, `confirmed`, or `rejected`. The AI explanation must not itself create the state transition.
- Flag missing information such as contact details, pet age/species/sex/spay-neuter status, vaccine proof, medication name/dose/schedule, feeding/allergy/medical details, temperament/group-play evidence, requested dates/service/accommodation, deposit/payment state, provider/source IDs, or policy refs.
- Treat customer free text, screenshots, emails, SMS/chat messages, phone transcripts, uploads, and raw provider webhooks as evidence/claims requiring deterministic reconciliation unless already mapped and verified.
- Recommend internal next actions and draft internal tasks or manager packets with citations, uncertainty labels, redactions, and approval-gate labels.
- Draft customer messages for review when they avoid prohibited claims and clearly label the needed approval gate.

### Prohibited AI actions

The AI recommendation layer must not:

- Invent or infer availability, prices, deposit amounts, refundability, vaccine status, temperament/group-play approval, room/run/condo/suite assignment, playgroup assignment, staff capacity, service availability, holiday rules, or policy exceptions.
- Override deterministic hard stops, stale-data decisions, provider conflicts, policy versions, capacity/staffing constraints, or prior human-review requirements.
- Confirm a booking, move a reservation to `confirmed`, update live provider reservation status, hold/release capacity, assign rooms/runs/groups, overbook, or promote from waitlist.
- Reject a customer booking/request or send rejection language without the approved rejection path.
- Accept special-care, medical, medication, feeding, allergy, handling, isolation, anxiety, or behavior exceptions without staff/manager approval.
- Override behavior/group-play restrictions, reinstate/suspend after incidents, clear bite/incident flags, or make final safety determinations.
- Charge, refund, waive, discount, forfeit, alter payment/deposit policy, or state money terms not present in policy-backed deterministic outputs.
- Send customer-facing messages involving confirmation, rejection, special-care acceptance, behavior exceptions, medical/incident/safety concerns, payment/deposit terms, policy exceptions, or availability promises without the appropriate approved-send gate.
- Put raw provider JSON, broad inbox/message history, unredacted documents, raw payment data, or unrelated customer/pet history into prompt context when minimized typed packets or source refs are sufficient.

### Escalation conditions

The AI should recommend human review or internal task creation when any of the following are present:

- Missing, stale, conflicting, or unmapped deterministic inputs; missing source IDs; provider conflicts; policy-version uncertainty; or freshness uncertainty.
- Vaccine/document proof is absent, expired, unverified, customer-provided-only, unreadable, or inconsistent with policy.
- Medication, feeding, allergy, medical, mobility, isolation, anxiety, or vet-contact details are vague, high-risk, or require special handling.
- Temperament/group-play status is unknown, stale, contradictory, newly concerning, or affected by bite history, human/dog selectivity, incident restrictions, stress observations, or manager-review flags.
- Capacity/staffing data is stale, unavailable, full, held, closed, over-capacity, holiday/blackout affected, ratio-constrained, special-care constrained, multi-pet constrained, or accommodation/care-lane incompatible.
- Payment/deposit state is missing, failed, late, disputed, partial, waiver/refund/discount/forfeit-related, or not reconciled to an approved policy-backed source.
- Customer-facing language would mention rejection, denial, waitlist priority, medical/behavior/safety concerns, incident history, policy exception, payment consequence, or availability/confirmation.
- A recommended action requires one of the explicit approval gates below.

### Confidence and unknown handling

AI output must be conservative and evidence-bound:

- Treat deterministic rule outputs and approved source snapshots as the only basis for feasibility explanations.
- Represent free-text/customer-provided/provider-raw data as claims or evidence requiring reconciliation unless already mapped and verified by the deterministic layer.
- Label uncertainty explicitly with statuses such as `known`, `missing`, `stale`, `conflicting`, `customer_claimed`, `provider_unmapped`, `policy_gap`, and `requires_review`.
- If any required fact is unknown, recommend the narrowest safe follow-up or review route; do not fill gaps with likely values or prior assumptions.
- If confidence is limited by missing/stale/conflicting evidence, state what evidence would resolve the uncertainty and which role must approve or verify it.
- Do not produce customer-visible certainty stronger than the deterministic output. For example, prefer "staff can review availability for those dates" over "we have space" unless an approved deterministic customer-message path explicitly permits that statement.

### Customer-message automation boundary and approval gates

Customer-message draft creation, outbound send, and booking/provider mutation are separate actions.

Default boundary:

- AI may draft customer messages for staff review.
- Customer-message sends require an approved draft/version, recipient/channel, intent, evidence refs, redaction policy, and approval id.
- Missing-info follow-ups may be drafted in neutral language, but sensitive or consequential sends remain approval-gated.
- Waitlist, rejection, confirmation, special-care, behavior, incident/safety, medical/vaccine, payment/deposit, exception, and availability-promise messages require human approval before sending.
- A future deterministic, receipt-only acknowledgement may be automated only if an explicit policy enables it and it does not mention availability, confirmation, rejection, waitlist priority, price/deposit/payment consequence, vaccine/medical/behavior judgment, special-care acceptance, or policy exception.

Required human approval gates:

- Confirmed booking automation: any transition to confirmed state, live provider status update, customer-visible confirmation, room/run/group assignment, capacity hold/release, or waitlist promotion.
- Rejection: any customer request denial, rejection state transition, or rejection/denial customer message.
- Special-care acceptance: any acceptance of medical, medication, feeding, allergy, mobility, isolation, anxiety, vet-clarification, or other special handling exception.
- Behavior exceptions: any override or acceptance involving group play, temperament, bite/incident history, dog/human selectivity, suspension/reinstatement, or staff/manager behavior-review flags.

### Recommended AI output shape

Each AI recommendation should be returned as a structured packet containing:

- `recommended_bucket`: deterministic readiness/review bucket being explained, not independently created by AI.
- `deterministic_outcome_refs[]`: rule-result IDs, policy versions, capacity/staffing snapshot IDs, payment/deposit refs, provider/source refs, and audit refs.
- `explanation`: concise staff-facing explanation of the deterministic result.
- `missing_or_ambiguous_inputs[]`: facts needed, current status (`missing`, `stale`, `conflicting`, `customer_claimed`, `provider_unmapped`, `policy_gap`, `requires_review`), and source/evidence refs.
- `recommended_next_actions[]`: waitlist review, missing-info follow-up, document/vaccine review, care/medical/medication review, behavior review, staff capacity verification, payment/manager review, internal task draft, or approval packet.
- `drafts[]`: optional customer/internal draft messages labeled with audience, intent, sensitivity, approval requirement, and prohibited claims avoided.
- `approval_gates[]`: explicit gates required before execution.
- `prohibited_claims_avoided[]`: availability, price/payment, vaccine, room/group, staff capacity, confirmation/rejection, or exception claims intentionally not made.
- `confidence`: conservative confidence/unknown summary grounded in deterministic evidence.
