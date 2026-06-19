# Staff operations workflow

> Successor route: this is a detailed specification/supporting-proof artifact, not the current reader spine. Start with the [docs successor and archive map](../design/successor-archive-map.md#older-workflow-and-specification-docs), [workflow-to-entity map](../design/workflow-to-entity-navigation-map.md), and [operator workflow index](operator/README.md) before using this page for current claims.

Status: detailed supporting-proof artifact for the staff-operations workstream. This document synthesizes the completed part artifacts in `docs/workflows/staff-operations-parts/` into one integration contract for product, data-model, task-model, and implementation planning; use the successor routes above for current reader navigation.

This document is a workflow/specification artifact only. It does not authorize live reservation updates, care decisions, playgroup assignments, medication or feeding decisions, payment actions, customer messages, provider writes, staff schedule changes, room-capacity release, or production task automation.

Anything marked proposed, draft, or review-gated requires explicit manager/location policy approval before it becomes production behavior. Human-gated automation in this document remains proposed / requires approval.

## Scope

This workflow covers the internal staff/manager operating surface for a pet resort or small resort group:

- daily operating-day dashboards;
- reservation-derived staff task planning;
- active stay execution and shift handoff;
- feeding, medication, cleaning, play/enrichment, document, incident, and message-approval work;
- staff note capture and customer-safe evidence boundaries;
- playgroup/compatibility support;
- manager review queues and approval gates;
- integration notes for product maps, data model, and task model.

Out of scope unless separately approved:

- autonomous booking acceptance/rejection, check-in, checkout, provider/PMS mutation, or room-capacity return;
- autonomous care/medical/medication/safety/eligibility/playgroup decisions;
- autonomous customer-message send, payment charge/refund/waiver/discount/credit, or policy exception;
- approved exact location policies for SLAs, ratios, task cadence, staff permissions, or customer-facing copy.

## Source artifacts

Canonical staff-operations part docs:

- `docs/workflows/staff-operations-parts/inputs.md`
- `docs/workflows/staff-operations-parts/dashboard-views.md`
- `docs/workflows/staff-operations-parts/daily-task-generation.md`
- `docs/workflows/staff-operations-parts/playgroups-compatibility.md`
- `docs/workflows/staff-operations-parts/staff-note-ux.md`
- `docs/workflows/staff-operations-parts/manager-review-queue.md`

Additional source anchors from those parts include current domain contracts in `domain/src/entities.rs`, `domain/src/operations.rs`, `domain/src/workflow.rs`, boarding/daycare implication docs, and `docs/workflows/payments-pricing.md` for payment-sensitive boundaries.

Known caveats inherited from the input packet:

- `docs/product/pet-resort-product-map.md` was requested as a product-map source but is not present in this repo at synthesis time.
- Some product-map and final data-model context is represented through completed kanban handoffs and current domain-contract/Rust docs rather than a single restored artifact.
- Exact pilot/location policy remains unresolved for role taxonomy, SLAs, check-in windows, ratio thresholds, task cadence, document proof rules, pricing/payment policy, and customer-facing wording.

## Core operating assumptions

Staff operations should be modeled around an operating day plus reservation/stay lifecycle, not a generic checklist.

The workflow should consume:

- location and operating date;
- reservations and stay state;
- service kind: boarding, day play, day boarding, grooming, training, DaySpa, and approved add-ons;
- customer, pet, care profile, vaccine/document, temperament, behavior, medication, feeding, allergy, emergency/vet, and contact evidence;
- room/condo/kennel/yard/care-lane assignment where modeled;
- capacity, labor, staff coverage, and ratio snapshots when available;
- task state, staff notes, incidents, handoff items, draft messages, and manager review items;
- trusted payment/deposit/readiness signals from approved payment surfaces only.

Source facts may be verified/current, missing, stale, conflicting, untrusted, provider-unverified, or AI-draft. Missing or conflicting facts should create review states, not silent readiness.

Automation posture:

- AI/workflows may read source records, summarize evidence, detect gaps, draft internal tasks/messages, prepare review packets, and suggest next states.
- AI/workflows must not act as autonomous medical, safety, medication, feeding, eligibility, payment, capacity-exception, playgroup, provider-mutation, or customer-message authorities.
- AI summaries, OCR, media analysis, free text, and confidence scores are not final evidence by themselves.
- Staff/manager actors remain responsible for care completion, approvals, and customer-facing or provider-facing decisions.

## Staff roles

These role labels are provisional workflow roles. They should map to the current `operations::StaffRole` enum where possible and should not be treated as approved payroll, permissions, or real titles until pilot policy confirms them.

| Role | Primary workflow responsibilities | Approval boundaries |
| --- | --- | --- |
| Front desk / host | Arrival and departure throughput, customer identity/reservation lookup, routine collection, belongings, signatures, document intake, customer-safe scripts, checkout prep. | Cannot clear care/medical/behavior uncertainty, group-play exceptions, payment/refund/waiver issues, capacity exceptions, or sensitive messages without the relevant gate. |
| Kennel technician / care staff | Feeding, reviewed medication execution, potty/walk/enrichment, room/condo care, cleaning, daily care evidence, observations, shift handoff. | Cannot infer executable medication/feeding instructions or approve sensitive customer communication. |
| Playgroup / daycare staff | Daycare roster execution, playgroup/care-lane confirmation, group observation, reassignment signals, individual-care lane execution. | Provisional split from kennel/lead roles. Staff confirms groupings; automation does not. Overrides/reinstatement/capacity/ratio exceptions require manager approval. |
| Groomer / bath / DaySpa staff | Grooming/bath/service evidence, service completion, grooming notes, checkout-linked follow-up. | Injury/stress/condition claims, charge/package/refund implications, and sensitive recommendations require review. |
| Trainer | Training session evidence, follow-up notes, package/rebooking signals where offered. | Sensitive behavior claims and package/charge implications require review. |
| Lead staff / shift lead | Handoff acceptance, ordinary blocked-task triage, assignment balancing, cross-team escalation, incoming owner assignment. | Manager-only items cannot be cleared unless approved policy grants that authority. |
| Manager / admin | Capacity/ratio and policy exceptions, safety/incident/medical/payment/customer-sensitive decisions, approval queues, policy configuration, final gates. | Required for manager-review gates, refunds/waivers/discounts/credits/forfeitures, sensitive incident/medical/behavior messaging, overbooking/waitlist exceptions, provider write-back approvals, and group-play overrides/reinstatement. |
| AI workflow worker / system | Source-backed summaries, evidence packets, draft tasks/messages, suggested statuses, risk/gap detection. | Suggestions/drafts only unless a future deterministic, verified, template-bound, explicitly approved policy narrows the action. |

Open role questions:

- Whether playgroup/daycare staff is first-class or modeled as kennel/lead assignments.
- Whether bather is separate from groomer.
- Which roles may create, assign, start, block, complete, cancel, escalate, or approve each task and note type.
- Whether assignment is role-only, named staff, shift owner, or all three.

## Shared status and urgency language

Use common readiness/status lanes across dashboards, task generation, review queues, and handoffs:

| Label | Meaning | Typical owner | Allowed behavior |
| --- | --- | --- | --- |
| Ready | Required facts are present and no visible review gate blocks routine staff work. | Role-specific staff | Show next action and completion evidence. Do not auto-complete. |
| Ready with expected collection | A predictable collection item is missing but does not require care/manager judgment. | Front desk | Show collection checklist and customer-safe script draft if approved for review. |
| Needs front-desk collection | Routine identity, contact, signature, belongings, document copy, or allowed payment-method confirmation blocks throughput. | Front desk | Draft collection task/reminder. Do not infer final eligibility or payment truth. |
| Needs care review | Feeding, medication, allergy, medical, temperament, handling, play, or observation facts are missing or uncertain. | Kennel/playgroup/lead | Route to care review; AI may summarize uncertainty only. |
| Needs manager review | Override, hard stop, capacity/ratio exception, policy exception, payment/refund/waiver/discount issue, incident/safety sensitivity, or customer-message sensitivity exists. | Manager/admin | Lock final action behind manager approval. |
| Blocked | A required source, policy, provider state, eligibility fact, capacity/labor fact, or safety decision is unavailable or conflicting. | Assigned owner plus escalation owner | Show blocker, source gaps, and escalation path. Do not mark ready. |
| Critical now | Time-sensitive safety/care/customer-throughput issue such as late medication, active incident, unsafe ratio, or pickup blocked by release facts. | Lead staff/manager | Promote to top, require acknowledgement, preserve audit. |

Every row/card should reveal source anchors, source state, review gate, required authority, customer-safe summary, internal-only notes, manager-only rationale, actor/timestamp history, and audit references.

## Dashboard views

The staff dashboard should expose role-aware operating views. Views are routing and execution aids, not approval substitutes.

### 1. Arrivals

Purpose: prepare and execute check-in readiness.

Data shown:

- reservation, pet, customer, service, arrival window, add-ons, status, expected length of stay/session;
- readiness packet: identity, pet profile, vaccine/documents, care plan, feeding, medications, allergies, medical conditions, temperament/group-play observations, emergency/vet contact, room/yard/care-lane assignment, trusted deposit/payment readiness;
- open tasks: `CheckInPrep`, `DocumentReview`, `CustomerFollowUp`, feeding/medication clarification, room/yard prep, belongings intake, daily-update setup, playgroup assessment;
- status suggestion only, clearly labeled.

Staff actions:

- start/assign arrival prep tasks;
- mark routine collection evidence received;
- add belongings intake notes;
- request care/document/payment/room/playgroup review;
- escalate blockers;
- prepare, but not execute, provider status update unless approved.

Manager-only controls:

- hard-stop clearance, capacity/staffing exception, policy exception, payment/refund/waiver/discount decision, group-play reinstatement/suspension/override, sensitive customer wording, and provider check-in mutation.

### 2. Departures

Purpose: prepare checkout, pet release, customer summary, belongings, payment routing, and turnover.

Data shown:

- reservation/stay, pet/customer, pickup window, late-departure risk, room/condo/yard/care-lane assignment;
- required care/add-on/play/grooming/training tasks;
- daily update/Pawgress/checkout report approval state;
- belongings and medication return status;
- trusted billing/payment summary without raw payment payloads;
- room/turnover task state.

Staff actions:

- complete checkout prep with evidence;
- reconcile belongings and care/add-on completion;
- draft customer update for approval;
- create payment follow-up/reconciliation task without charging/refunding/waiving;
- create turnover task before capacity can return through any approved path;
- escalate late pickup, release, incident, care, or payment exceptions.

Manager-only controls:

- release exceptions, late pickup handling, payment/refund/waiver/discount/credit/forfeiture decisions, sensitive incident/medical/behavior wording, capacity return exceptions, provider checkout/status mutation.

### 3. Active stays

Purpose: run daily care, active stay watchlists, shift handoff, and care-plan execution.

Data shown:

- active pets/stays by location/date/service, room/group/care lane, assigned staff/role, current task burden;
- care plan state: ready, blocked for staff review, blocked for manager/vet clarification;
- feeding, medication, cleaning, play/enrichment, daily update, incident, and customer follow-up tasks;
- watchlist signals and handoff items with owner, due timing, severity, source evidence, and next action.

Staff actions:

- accept shift handoff;
- start/block/complete care tasks with evidence;
- record observation notes as internal-only or customer-safe candidates;
- escalate care ambiguity, incident, medical, behavior, staffing, or capacity risk;
- draft daily update from approved evidence for review.

Manager-only controls:

- safety/medical/incident/customer-sensitive handling, staffing/capacity exceptions, suppression/release of customer updates, care-cadence policy changes.

### 4. Feeding

Purpose: make feeding obligations visible, executable only when reviewed, and auditable.

Data shown:

- pet/stay, feeding instruction source and status, meal schedule, portion/prep notes, allergies, due time, assigned staff, completion evidence, refusal/exception history, clarification state.

Staff actions:

- verify reviewed instruction source;
- prepare/feed/record meal with evidence;
- record refusal, partial intake, or exception;
- request customer/care/manager clarification;
- carry unresolved obligation into handoff.

Boundaries:

- AI must not invent portions, schedules, allergies, or medical relevance.
- Feeding completion requires staff evidence.
- Refusals, allergy/medical concerns, repeated issues, or sensitive customer language require review.

### 5. Medications

Purpose: keep dose obligations time-critical, source-backed, and restricted to reviewed instructions.

Data shown:

- pet/stay, medication name, dose, route if modeled, schedule, source/review status, due time, assigned staff, prior completion/skipped-dose evidence, allergies/medical conditions, vet/emergency contact, clarification state.

Staff actions:

- verify reviewed name/dose/schedule/source before administration;
- administer and record evidence if role-authorized;
- block dose with explicit reason when source is missing/conflicting;
- record skipped dose/exception and escalate;
- carry unresolved medication obligation into handoff.

Boundaries:

- AI must never infer executable medication instructions from vague notes, images, or customer prose.
- Medication tasks require reviewed name, dose, schedule, and source before executable work begins.
- Skipped-dose, adverse-signal, medical, or sensitive customer communication paths require manager/vet/customer review according to policy.

### 6. Cleaning / room-turnover

Purpose: track daily housekeeping, exception cleanup, and departure turnover before capacity return.

Data shown:

- room/condo/kennel/yard identifier where modeled, reservation/stay link, cleaning type, due time, assigned staff/role, status, completion evidence, blocker reason, capacity impact, next arrival risk.

Staff actions:

- accept/complete daily cleaning or turnover task with evidence;
- block task with reason and source;
- link turnover to departure and next arrival where available;
- escalate capacity-impacting blockers.

Boundaries:

- A room should not return to sellable inventory solely because AI predicted or drafted a turnover task.
- Capacity return exception, quarantine/contamination, disease/safety issue, or compensation/refund implication requires manager review.

### 7. Playgroups

Purpose: support safe playgroup/care-lane operations while preserving staff confirmation and manager exception gates.

Data shown:

- pet/daycare or boarding stay, requested play/enrichment, group-play eligibility state, vaccine/spay-neuter/temperament/incident source state, size/weight band, known friends, exclusions/do-not-pair list, behavior flags, manager-note presence, current staff-confirmed assignment, staffing/ratio/capacity state, assessment task, observations.

Staff actions:

- review evidence and confirm care lane/group where policy allows;
- record observations and reassignment signals;
- move pet to individual-care review when uncertain;
- reject or override suggestions with typed reasons;
- escalate incident, behavior, ratio, capacity, or sensitive wording.

Boundaries:

- Playgroup suggestion automation is proposed / requires approval.
- AI suggestions are evidence packets and candidate groupings only; staff confirms all groupings and same-day reassignments.
- Unknown/stale/conflicting temperament, vaccine, spay/neuter, care, incident, staff-coverage, or capacity facts must not default to group play.
- Manager approval is required for overrides, reinstatement/suspension after incidents, capacity/ratio exceptions, and sensitive behavior/customer-facing language.

### 8. Documents needing review

Purpose: expose vaccine/document uncertainty that affects arrival, play, eligibility, or communication.

Data shown:

- pet/customer/reservation, document type, upload/source, extracted fields, vaccine/proof state, source confidence, expiration/freshness, missing fields, linked arrival/stay, blocker impact, internal reviewer notes, AI extraction rationale.

Staff actions:

- request missing proof using approved/customer-safe language;
- attach/source document;
- mark received/pending review;
- assign/reassign review task;
- link blocker to arrival/reservation.

Boundaries:

- OCR/AI extraction is not document approval.
- Vaccine/source verification and eligibility consequences require authorized review and audit.
- Customer-facing eligibility/refusal language is sensitive and should be reviewed.

### 9. Incidents

Purpose: route safety, medical, behavior, injury, escape, hazard, or sensitive events to manager/lead review and follow-up.

Data shown:

- pet/stay, incident type/category if modeled, time, location, involved pets/staff, severity, current safety state, immediate actions taken, open follow-up tasks, manager-review state, customer/vet communication state, related blockers.

Staff actions:

- record initial observation and immediate action with timestamp/source;
- create/complete follow-up care tasks with evidence;
- escalate to lead/manager;
- draft internal handoff and customer message for review where appropriate.

Boundaries:

- Raw incidents are internal/sensitive by default.
- AI must not decide fault, liability, medical conclusions, severity closure, or customer-facing wording.
- Manager approval is required for incident classification, customer/vet/emergency communication, playgroup suspension/reinstatement, legal/privacy-sensitive language, and provider/customer-facing mutation.

### 10. Message approvals

Purpose: manage customer-facing drafts, especially daily updates, document requests, incident follow-ups, checkout summaries, complaints, and payment/policy-sensitive copy.

Data shown:

- draft message, recipient, channel, message type, source evidence, sensitivity tags, reviewer required, approval state, send eligibility, related sends, suppression reason, and internal rationale separated from customer text.

Staff actions:

- review draft against cited evidence;
- edit customer-safe wording;
- request care/manager fact review;
- approve only routine staff-authorized messages if policy/template/send path allows;
- suppress draft with reason.

Boundaries:

- AI must not send customer-facing messages unless a separately approved deterministic send path has fixed recipient, facts, template, and send condition.
- Sensitive facts require manager review: health, medication, allergy, behavior, incident, safety, eligibility/refusal, payment/refund/waiver/discount, complaint, policy exception, legal/privacy.

## Daily task generation from reservations

Task generation is proposed / requires approval until managers/location policy approve production rules by category, source, priority, assignee, and completion evidence.

The planner should produce one of four output levels:

1. Suppress: no task because the trigger is stale, duplicate, cancelled, out of scope, or too unresolved.
2. Draft recommendation: reviewable task intent with source evidence, due basis, suggested role, priority rationale, duplicate key, and blocking reason. This is the safe default while rules are unapproved.
3. Auto-create internal task: only after approved policy says the trigger, source evidence, due time, role, and completion evidence are deterministic.
4. Manager-review task: created when work may be required but cannot be delegated to ordinary staff without review.

Every generated task should carry:

- location and operating day;
- task kind or proposed future task kind;
- reservation, pet, customer, daily brief, workflow event, or staff-created source;
- human-readable title;
- due time and due rule;
- assignment role/named staff/shift owner when authorized;
- priority and rationale;
- completion evidence requirement;
- review gate and source snapshot/policy refs;
- duplicate key: category + source entity + operating day + due slot + rule version;
- audit fields for generated, updated, suppressed, blocked, cancelled, completed, or reconciled decisions.

Current source task kinds:

- `CheckInPrep { reservation_id }`
- `CheckOutPrep { reservation_id }`
- `Feeding { pet_id }`
- `MedicationAdministration { pet_id }`
- `PlaygroupAssessment { pet_id }`
- `CleaningTurnover { reservation_id }`
- `DailyUpdateDraft { reservation_id }`
- `DocumentReview { pet_id }`
- `IncidentFollowUp { pet_id }`
- `CustomerFollowUp { customer_id, reason }`

### Check-in prep

Triggers:

- expected arrival today or within approved pre-arrival window;
- reservation state such as confirmed/offered/missing info/vaccine pending/special review;
- customer/provider/staff update that affects readiness;
- hard stop, missing document, care-plan blocker, deposit/payment gate, capacity/room issue, special handling flag.

Due basis: before arrival/check-in window; immediate for same-day changes or front-desk presence.

Default owner: front desk, with linked care/lead/manager review tasks where needed.

Completion evidence: reviewed arrival packet with identity, reservation status, service dates, pet profile, document readiness, care-plan readiness, signatures/forms, belongings, room/capacity assignment, payment/deposit readiness where trusted, actor/timestamp/source snapshot, and remaining gates.

Proposed auto-create: ordinary pre-arrival readiness task for deterministic same-day arrivals. Requires approval before production auto-creation.

Manager-approved: hard stops, capacity/ratio exceptions, payment/refund/waiver decisions, safety/medical ambiguity, sensitive customer language, and final provider `CheckedIn` mutation.

### Feeding

Triggers:

- active/checked-in stay has reviewed feeding plan;
- feeding instruction missing, conflicting, changed, waived, or staff-review-required;
- handoff carries feeding obligation;
- refusal, partial meal, wrong food, allergy concern, appetite change, vomiting, or exception is recorded.

Due basis: verification before check-in/first feeding; execution at reviewed feeding time/window; exception follow-up before shift handoff or sooner by policy.

Default owner: kennel technician; front desk may collect routine customer details; lead/manager handles ambiguity or sensitive exceptions.

Completion evidence: staff actor, timestamp, feeding plan/source version, food offered, eaten/refused/partial state, exception note, handoff/follow-up status.

Proposed auto-create: routine feeding execution from reviewed care plans and approved recurrence windows. Requires approval before production auto-creation.

Manager-approved: allergy/medical exceptions, repeated refusal escalation, sensitive customer messages, waiver of required feeding info, policy exceptions.

### Medications / special care

Triggers:

- active/checked-in stay has reviewed medication name, dose, route if modeled, schedule, storage/handling, and source;
- medication instruction missing, newly changed, conflicting, expired, source-unverified, or needs vet/manager clarification;
- dose due/overdue/skipped/refused, medication unavailable, side-effect signal, medical condition note, allergy flag, special handling requirement.

Due basis: verification before check-in/first dose; administration at reviewed dose time; overdue/skipped/exception escalation immediately or before handoff by policy.

Default owner: authorized care staff where local policy permits; lead or manager/vet for verification, ambiguity, or high-risk exceptions.

Completion evidence: staff actor, timestamp, instruction version, scheduled dose time, actual administration/skip time, outcome, exception reason, handoff/watchlist outcome.

Proposed auto-create: routine dose tasks from reviewed medication plans when local policy authorizes the role. Requires approval before production auto-creation.

Draft/review only: all medication tasks from unverified, ambiguous, imported, AI-extracted, changed, or incomplete evidence.

Manager-approved: vet clarification, skipped/incorrect/refused dose handling where policy requires, medical emergency/safety escalation, customer-facing medication exception language, waiver of required data.

### Playtime / playgroup attendance

Triggers:

- daycare/day-play/day-boarding expected or active;
- boarding stay includes playtime, individual play, daycare add-on, group-play request, or enrichment;
- missing/current temperament/group-play review, behavior/safety flag, vaccine/spay-neuter/age/care constraint, incident/reinstatement question;
- roster/ratio/capacity/staffing snapshot changes;
- staff observation or incident changes safe assignment.

Due basis: assessment before play block/roster cutoff; attendance per approved play block; incident/reassignment follow-up before return to group play.

Default owner: playgroup/daycare staff, mapped to kennel technician until role taxonomy is approved; lead for roster adjustment; manager for overrides.

Completion evidence: eligibility snapshot, group/care-lane or individual-care decision, staff actor, time/block, attendance outcome, yard/group if modeled, behavior observations, incidents/exceptions, daily-update eligibility.

Proposed auto-create: assessment task for requested play service with complete policy evidence; routine attendance only after approved play-lane policy exists.

Draft/review only: playgroup suggestions until the playgroup suggestion automation gate is approved.

Manager-approved: capacity/ratio exceptions, incident reinstatement/suspension, safety-sensitive behavior decisions, and customer-facing exception language.

### Cleaning / room reset / housekeeping

Triggers:

- reservation/stay checks in or becomes active and accommodation assignment exists;
- daily housekeeping window;
- event cleanup from elimination, illness, spill, contamination, incident, or observation;
- departure/release completed and room must be reset;
- room assignment changes or stay extends.

Due basis: daily occupied-room window; exception cleanup immediately/by severity; turnover immediately after release and before capacity return.

Default owner: kennel technician/housekeeping role if added; lead for blocked room/capacity impact; manager for quarantine/contamination/safety/capacity override.

Completion evidence: staff actor, timestamp, accommodation id where modeled, cleaning type, checklist/supplies if required, before/after or inspection evidence if policy requires, capacity-release readiness.

Proposed auto-create: routine turnover after checkout/release and daily housekeeping once room identity and cadence policy exist. Requires approval before production auto-creation.

Manager-approved: quarantine/contamination, room-capacity override, disease/safety issue, compensation/refund/customer-sensitive follow-up.

### Daily customer update draft / review

Triggers:

- active boarding/daycare/day-boarding stay reaches update window and enough reviewed care/play evidence exists;
- customer preference, membership/package, service type, or policy requires update;
- significant care/play/grooming/training evidence arrives;
- sensitive incident/medical/behavior/payment/policy fact requires suppression/review decision;
- checkout report is needed.

Due basis: draft after meaningful evidence window; review before promised send window or checkout; sensitive review immediately if unsafe to send routine copy.

Default owner: AI/system may draft only; front desk/lead may review routine drafts if policy allows; manager reviews sensitive content.

Completion evidence: source evidence list, excluded/suppressed reasons, reviewer actor, approved/sent/suppressed state, channel if sent by approved tool, timestamp.

Proposed auto-create: internal draft task when active-stay policy says updates are due.

Proposed draft-only automation: AI summarization from approved evidence.

Manager-approved: sending messages involving health, medication, allergy, incident, behavior, safety, payment, eligibility, refusal, or policy exceptions; enabling any unattended customer send.

### Checkout prep

Triggers:

- active/checked-in stay has departure/pickup today or within approved prep window;
- pickup time/service/add-on/release/contact/belongings detail changes;
- grooming/bath/training/add-on completion needed;
- open feeding/medication/incident/behavior/daily-update/payment/belongings/turnover dependency exists;
- pet ready for release or front-desk checkout starts.

Due basis: before pickup/departure; immediate for same-day changes; release blockers before handoff; turnover at release/checkout completion.

Default owner: front desk; kennel/groomer/trainer/lead/manager linked by blocker type.

Completion evidence: departure packet reviewed, pet identity, authorized release/customer identity if modeled, care/add-on completion or approved exception, belongings/medications reconciled, report reviewed/sent/suppressed, payment state routed, turnover task created, actor/timestamp/source snapshot/status suggestion.

Proposed auto-create: routine checkout prep for expected same-day departures and routine turnover link after release. Requires approval before production auto-creation.

Manager-approved: payment/refund/waiver/credit/forfeiture decisions, late pickup exceptions, incident/medical/behavior-sensitive release messaging, release hard-stop overrides, final provider checkout mutation.

### Cross-category conservative rule

The planner should choose review over automation when:

- source facts are missing, stale, contradictory, provider-unverified, or AI-extracted only;
- the task implies medical, medication, allergy, feeding, behavior, incident, safety, eligibility, capacity, staffing, payment, refund, waiver, forfeiture, discount, credit, cancellation, no-show, or legal policy judgment;
- customer-facing text would mention sensitive facts or exceptions;
- the task would mutate provider/system state, reservation status, payment state, room inventory, group-play status, or message-send state;
- duplicate work already exists;
- assignee role or permission is unresolved.

## Playgroup and compatibility support

Playgroup support should help staff see compatibility evidence, not replace staff judgment.

Required compatibility inputs:

- pet identity, species, age/life stage, sex, spay/neuter status where policy-relevant;
- size or weight band, with source and freshness;
- temperament profile, group-play observation, intro-assessment state, and staff-entered notes;
- care facts that affect group safety: medications, allergies, medical conditions, feeding/food-guarding concerns, handling instructions, escape risk, quiet-room needs;
- behavior flags and incident history: bite history, dog/human selectivity, anxiety, food guarding, escape risk, manager-review flag, unresolved incident restrictions, suspension/reinstatement state;
- manager notes, permissioned and redacted by role;
- exclusions / do-not-pair restrictions with scope, source, reason, approval actor, and expiry/review date;
- known friends / preferred companions with observation source, confidence, and recency;
- prior group history, same-day reassignments, rest/split periods, rejected candidates, incident invalidations;
- reservation/stay/service mode, vaccine/document eligibility, hard stops, add-ons, medication/feeding/grooming/training conflicts;
- occupancy, roster, lane capacity, staff coverage, ratio/capacity status, and policy snapshot.

Suggested internal states:

- `EvidenceIncomplete`
- `NeedsStaffReview`
- `AutomationProposed`
- `StaffConfirmed`
- `RejectedByStaff`
- `ManagerReviewRequired`
- `TemporarilySuspended`
- `IndividualCareRecommended`
- `WaitlistOrCapacityHold`

Suggestion workflow:

1. Build evidence packet with source refs, freshness, review gates, redacted rationale, and missing/conflicting facts.
2. Decide care mode before matching companions: group play, individual care, hybrid play/rest, cat enrichment, or waitlist/hold.
3. Generate candidate groupings with size, temperament, exclusions, known friends, behavior flags, incident status, care constraints, staffing/capacity, confidence as review aid only, and alternative paths.
4. Staff confirms, moves, rejects, assigns individual care, escalates, or marks unavailable/waitlisted with typed reason.
5. Audit records AI candidate, displayed evidence version, staff/manager actor, timestamp, outcome, rationale, review gates, override/rejection reason, and customer-message state.

Hard blocker order:

1. Species/service-mode mismatch.
2. Vaccine/document/hard-stop or attendance ineligibility.
3. Unresolved incident suspension or manager-only restriction.
4. Do-not-pair exclusions.
5. Care/medical/handling notes that block or alter group play.
6. Staff coverage/capacity insufficiency.
7. Stale/unknown temperament, size, or group observation.
8. Known friends as positive evidence only after safety gates pass.

Typed staff rejection reasons should include size mismatch, energy mismatch, temperament mismatch, known exclusion, known friend not enough evidence, behavior flag requires assessment, incident pending review, care/medical constraint, medication/feeding timing conflict, staff coverage concern, capacity concern, needs intro assessment, stale/missing evidence, manager restriction present, customer/reservation state blocks, and other staff judgment.

Manager override reasons should include approved ratio/capacity exception, incident reinstatement, policy exception, care accommodation, do-not-pair modification, sensitive communication language, and waitlist release. Overrides must be scoped, audited, and expiry/review-policy-bound; they do not rewrite the original denial/review reason as ordinary eligibility.

Incident feedback loop:

- Staff incident/behavior observations invalidate affected current assignments or mark `TemporarilySuspended`.
- Follow-up tasks route to lead/manager.
- Compatibility evidence gains incident status, restrictions, behavior flags, special handling, reassessment due date, or reinstatement conditions.
- Future suggestions show the restriction and require review until cleared.
- Customer-facing language remains draft/suppressed until manager-approved.

Customer-safe play language should be practical and non-diagnostic. Do not expose raw labels such as aggressive, bite risk, dog selective, food guarding, internal ratios, employee names, unreviewed incident conclusions, manager-only notes, or do-not-pair counterpart details without explicit approval.

## Staff note UX

Staff notes are fast capture surfaces for operational evidence. They are not automatic care decisions, customer messages, provider mutations, or task completion by themselves.

UX goals:

- capture useful evidence quickly during care, front desk, playgroup, grooming, training, and handoff moments;
- prefer structured quick buttons/chips for common events while preserving free text;
- make internal-only vs customer-safe boundaries visible at capture time;
- route risky notes into staff/lead/manager review;
- retain auditability for actor, time, location, subject, source channel, and later review;
- support future voice-to-text without allowing transcripts to become unchecked authority.

Primary capture surfaces:

1. Pet/stay timeline quick capture: note type, quick buttons, free text, optional photo, internal/customer-safe toggle, flag/escalate action, attach-to selector.
2. Task completion evidence panel: task-specific chips, required evidence prompts, completion/block/escalation, review gate display, attachments where allowed.
3. Shift handoff capture: next owner, severity, due timing, acceptance criteria, subject link, flag/escalate action.
4. Customer-update draft evidence drawer: customer-safe candidate marker, approved-to-mention chips, suppress reason, sensitive marker, manager-review marker, source evidence links.

Required note types should include feeding, medication, potty/elimination, behavior/temperament, playgroup/enrichment, health/medical, incident/safety, grooming/bath/DaySpa, training, belongings/front desk, document/vaccine, payment/policy, customer-message draft, and general operational notes.

Structured quick chips should cover:

- routine care: ate all, ate some, refused, water refreshed, potty normal, cleaned room, walk complete, enrichment complete;
- exceptions: refused food, skipped medication, vomiting, diarrhea, injury observed, behavior concern, missing belongings, document unreadable;
- review gates: needs staff review, needs shift lead, needs manager, needs vet/customer clarification, do not use for customer update yet;
- communication safety: customer-safe routine highlight, internal-only, manager wording review, suppress from update, draft follow-up;
- play/daycare: eligible evidence observed, needs temperament review, individual care today, moved playgroup, rest break, incident restriction;
- handoff: carry to next shift, next owner required, due before pickup, safety critical, evidence incomplete.

Chip invariants:

- every chip has stable semantic code, display label, allowed roles, default audience, severity, and review-gate mapping;
- free text can add context but cannot weaken a chip's review gate;
- chip sets are context-filtered by task and surface;
- local labels may vary, but semantic codes remain portable for audit/automation.

Free-text rules:

- default to internal-only unless explicitly marked as customer-safe candidate or entered in a customer-update evidence drawer;
- encourage factual observations: what happened, when, by whom, what follow-up is needed;
- warn for sensitive categories such as health, medication, allergy, injury, aggression, payment/refund, legal/policy exception, staff/customer conflict, or unnecessary personal data;
- never silently reinterpret free text into executable medication, medical, eligibility, payment, or message decisions.

Photos/attachments:

- are evidence references, not automatic truth;
- may support routine customer-update photos, meal/medication package label review, document/vaccine review, belongings, cleaning/maintenance evidence, incident/injury evidence;
- must have purpose and review state;
- restricted evidence cannot be used in customer updates without authorized review and redaction if needed.

Future voice-to-text:

- creates a draft free-text note with transcript source, actor, timestamp, subject, quality/confidence metadata, and correction history;
- must be reviewed/accepted by the speaking staff member before it can complete tasks, enter handoff, or feed drafts;
- should prompt for type/attachment;
- conservatively marks sensitive transcripts internal-only/review-required;
- must not capture executable medication instructions without structured verification.

Audience states:

- `InternalOnly`: visible to staff/manager workflows; not customer eligible except as reviewed source.
- `CustomerSafeCandidate`: staff thinks it may be used; still requires draft review.
- `CustomerApprovedSource`: authorized actor approved selected note content as source evidence.
- `RestrictedSensitive`: incident, health, medication, allergy, aggression, policy, payment, legal, personnel, or privacy-sensitive evidence.
- `SuppressedFromCustomer`: intentionally excluded with reason and reviewer.

AI/system cannot author live staff observations, attest that an event happened, close care tasks, downgrade restricted notes, approve customer-message sources, or send customer messages.

## Manager review queues

A manager-review item is an explicit approval/escalation object, not a generic note. It must identify subject, source trigger, owner, priority/severity, due basis, allowed actions, forbidden actions, audit evidence, customer-visible outcome, and closure evidence.

Recommended queue states:

- `Open`
- `NeedsInfo`
- `InReview`
- `DelegatedForCollection`
- `Approved`
- `Rejected`
- `Suppressed`
- `Escalated`
- `Closed`
- `Cancelled`

Use priority as review ordering, not authority to skip evidence:

- Critical: animal safety, injury/escape/bite/aggression, medication ambiguity inside window, medical distress, legal/privacy threat, unsafe ratio, release blocked by safety facts.
- High: arrival/check-in today, same-day daycare eligibility, special-care before stay start, unresolved complaint, sensitive draft, document blocker, payment/policy exception.
- Normal: non-urgent document ambiguity, future special-care, routine approval draft, customer follow-up tone review.
- Low: retrospective quality review, training feedback, non-live cleanup.

Required queue categories:

### Incidents

Triggers: incident follow-up task, care watchlist, daily brief risk, staff incident note, handoff item, complaint, provider/import flag.

Covers injury, illness, bite/aggression, escape/lost pet, medication/feeding exception with possible harm, facility hazard, staff/customer safety, property damage, or sensitive behavior event.

Manager owns disposition and customer-facing approval. Lead/care staff collect evidence and immediate safety actions. Closure requires manager decision, safety action evidence, customer communication sent/suppressed where applicable, and linked follow-up tasks.

### Rejected AI outputs

Triggers: staff/manager rejects AI summary, draft task, draft message, document extraction, playgroup suggestion, status suggestion, daily update, incident/complaint summary, or validator flags unsafe/unsupported output.

Manager owns high-risk, customer-facing, safety, payment, legal/privacy, and policy-overreach rejections. AI cannot self-approve a regenerated output. Closure requires suppression, corrected-and-approved draft, conversion to another queue item, or engineering/template/policy follow-up.

### Special-care bookings

Triggers: reservation/request/stay includes medication, complex feeding, allergies, medical condition, senior/puppy/kitten care, behavior concern, individual-care need, special accommodation, vet/emergency constraint, capacity/staffing/ratio issue, overbooking/waitlist release, group-play reinstatement, or customer-requested exception.

Manager owns booking acceptance exceptions, capacity/ratio overrides, group-play reinstatement/suspension/override, policy exceptions, and high-risk wording. Lead/care staff review executable care details and collect evidence. Front desk collects routine info but cannot approve medical/safety/capacity exceptions.

### Document uncertainty

Triggers: document review task, uploaded file, OCR/import conflict, missing/stale/illegible/wrong document, unsupported species/service requirement, customer/staff dispute, low-confidence extraction, or check-in/playgroup eligibility blocker.

Authorized reviewer/manager owns final eligibility-impacting approval/rejection. AI may extract and summarize but cannot verify final vaccine/document eligibility. Closure requires verified decision, pending/blocked state with follow-up, replacement/clinic/vet confirmation, and approved/suppressed customer message if applicable.

### Approval drafts / customer-message drafts

Triggers: AI or staff drafts daily/Pawgress update, incident message, complaint response, document request, booking acceptance/decline, special-care instruction, checkout summary, payment/deposit reminder, policy exception explanation, grooming/training follow-up, or support response.

Manager owns sensitive, policy-exception, incident, complaint, payment/refund/waiver, rejection/refusal, legal/privacy drafts. Front desk/lead may review routine factual drafts only if policy grants authority. AI may draft but cannot self-approve or send except under future approved deterministic template/send paths.

### Complaints

Triggers: customer message/call/review/social/imported note expresses dissatisfaction, dispute, care concern, incident concern, billing/refund complaint, staff conduct issue, missing update, booking/policy objection, or staff escalates interaction.

Manager owns complaint disposition and customer response. Payment reconciliation owns factual payment investigation; manager owns refunds/waivers/credits. Legal/compliance/privacy or higher management owns threats, privacy/regulatory, fraud/abuse, or high-risk public-review matters. AI cannot close complaints based on sentiment or respond directly outside approved deterministic template/send paths.

Escalation workflow:

1. Detect and draft with source refs and uncertainty labels.
2. Normalize into category, typed subject, source trigger, priority, owner, due basis, required fields.
3. Route to manager/admin for final decision and assign sub-tasks only for evidence collection/factual preparation.
4. Freeze unsafe execution until review closes: customer sends, provider writes, payment/refund/waiver actions, booking exceptions, group-play reinstatement/override, and care-plan execution where evidence is ambiguous.
5. Manager approves, approves with constraints, rejects, suppresses, requests info, delegates collection, or escalates.
6. Execute approved external actions only through bounded audited message/provider/payment/staff-task workflows.
7. Close with decision reason, audit event, closure evidence, and child task/execution references.
8. Feed quality loops without expanding automation authority unless explicitly approved.

## Human approval gates

These gates must survive product, data-model, task-model, and implementation decomposition.

### Playgroup suggestion automation

Status: proposed / requires approval.

Safe behavior before approval:

- collect evidence;
- build compatibility packets;
- surface missing/stale/conflicting facts;
- draft candidate groupings and alternative care lanes;
- show AI confidence only as review aid;
- require staff confirmation for every grouping and same-day reassignment;
- require manager approval for overrides, reinstatement/suspension after incidents, capacity/ratio exceptions, and sensitive language.

Forbidden before approval:

- auto-confirming groupings;
- defaulting unknown/stale/conflicting evidence to group play;
- bulk auto-confirm;
- changing provider/reservation status or customer-facing playgroup commitments;
- hiding manager-only restrictions behind a generic green state.

### Task auto-generation rules

Status: proposed / requires approval.

Safe behavior before approval:

- draft task recommendations with source, due basis, assignment suggestion, priority rationale, duplicate key, completion evidence, and review gate;
- create manager-review items for sensitive or ambiguous work;
- reconcile duplicates in draft state;
- keep completion authority with human staff/manager evidence.

Production auto-creation requires approved policy by task category, source, priority, assignee/role, due rule, duplicate-key behavior, and evidence standard.

Forbidden before approval:

- creating live authoritative task automation at scale;
- marking tasks complete from AI/OCR/media/free text;
- creating executable medication/feeding tasks from unreviewed notes;
- implying capacity return, provider status changes, customer sends, payments, refunds, waivers, or booking decisions from generated tasks.

### Other gates carried from source workflows

- medical, medication, allergy, feeding, behavior, incident, and safety ambiguity;
- vaccine/document approval and source verification;
- deposit/payment/balance, refund, waiver, forfeiture, discount, credit, cancellation, and policy exceptions;
- booking acceptance/rejection exceptions, overbooking, waitlist release, capacity holds, and late departure;
- customer-facing messages involving health, safety, legal, payment, incident, eligibility, refusal, complaint, or policy exceptions;
- provider/system mutations such as check-in/out status, customer send, charge/refund, capacity return, reservation mutation, or PMS/Gingr write-back.

## Integration notes

### Product map

The product map should treat staff operations as the internal execution layer connecting intake, booking, documents/vaccines, payments/pricing, daily care updates, incident escalation, customer messaging, CRM/retention, and provider/PMS integration.

Product surfaces to represent:

- operating-day dashboard;
- arrivals and departures worklists;
- active-stays care board;
- feeding and medications queues;
- cleaning/turnover board;
- playgroup roster/compatibility board;
- documents needing review;
- incidents and manager review queues;
- message approval queue;
- note capture/mobile staff UX;
- shift handoff packet;
- AI review-packet/draft surfaces.

Product-map caveat: until `docs/product/pet-resort-product-map.md` is restored or created, cite this artifact plus the staff-operations part docs as the canonical staff-operations product surface contract.

### Data model

Current anchors to preserve:

- identity and actors: `LocationId`, `CustomerId`, `PetId`, `ReservationId`, `StaffId`, `ManagerId`, `ActorRef::{Customer, Staff, Manager, System, Agent}`;
- location/policy: `Location`, `Brand`, `LocationPolicyRefs`;
- customer/pet: `Customer`, `PortalAccountRef`, `ContactChannel`, `Pet`, `Species`, `Sex`, `SpayNeuterStatus`, `CareProfile`, `MedicationInstruction`, `TemperamentProfile`;
- reservation: `Reservation`, `ServiceKind`, `ReservationStatus`, `ReservationSource`, `AddOn`, `HardStop`;
- operations: `ResortOperatingDay`, `ResortDailyBrief`, `DailyBriefSection`, `OccupancySnapshot`, `ArrivalDepartureSnapshot`, `LaborSnapshot`, `CustomerFollowUp`, `PetCareWatch`, `OperationsRisk`, `OperationsAction`, `StaffTask`, `StaffRole`, `StaffTaskSource`;
- workflow: `WorkflowEvent`, `WorkflowEventType`, `PolicyContext`, `AllowedAction`, `WorkflowResult`, `WorkflowStatus`, `RecommendedAction`;
- audit: `AuditEvent`, `AuditSubject`, `AuditAction`, typed metadata.

Likely model additions or refinements:

- operating-day dashboard projections and readiness packets;
- stay/care-plan state separate from reservation status where needed;
- manager-review queue aggregate with category, state, priority, severity, owner, due basis, source refs, evidence refs, review gate, allowed actions, customer-visible outcome, and closure evidence;
- staff note model with type, semantic chips, audience state, attachment targets, source channel, revisions, redactions, escalation, and audit transitions;
- daily update draft state with source evidence, reviewer, approved/sent/suppressed state;
- compatibility evidence packet, do-not-pair restriction, known companion preference, manager-only compatibility constraint, playgroup suggestion, rationale, confirmation outcome, rejection/override/invalidation reason;
- room/accommodation/yard identity and capacity-readiness state if cleaning controls inventory;
- richer medication/feeding instruction versioning and verification state;
- provider/PMS references and approved write-back execution events, kept distinct from recommendations.

Semantic invariants:

- a playgroup suggestion is not an assignment until staff confirmation exists;
- a manager override is an exception with scope, reason, actor, and expiry/review policy, not ordinary eligibility;
- AI-extracted note/document/media evidence is draft until reviewed;
- customer-safe candidate is not sendable approval;
- queue approval is not execution;
- provider/payment/message mutations create separate audited execution events;
- missing/stale/conflicting source facts route to review, not inferred readiness.

### Task model

Current task model can carry MVP work if task bodies/evidence metadata encode sub-intents, but downstream model refinement should decide whether to add first-class task kinds or linked subtypes.

Current compatibility:

- use `StaffTask` with location, kind, title, status, priority, due time, assignment, source, completion evidence;
- use `StaffTaskAssignment::{Unassigned, Staff(StaffId), Role(StaffRole)}`;
- use `StaffTaskStatus::{Open, InProgress, Blocked, NeedsManagerReview, Completed, Cancelled}`;
- use `StaffTaskPriority::{Low, Normal, High, Critical}`;
- use `StaffTaskSource::{Reservation, Pet, Customer, DailyBrief, WorkflowEvent, StaffCreated}`.

Task-model gaps to resolve:

- feeding verification vs execution vs refusal/exception;
- medication verification vs administration vs skip/exception/clarification;
- playgroup initial assessment vs roster assignment vs attendance vs reassignment vs reinstatement review;
- daily occupied-room cleaning vs departure turnover vs exception cleanup;
- daily update draft vs review vs approved/sent/suppressed state;
- grooming/bath/training add-on tasks and checkout readiness links;
- shift handoff packet items that may reference or create tasks;
- manager-review queue items that may link to one or more tasks but are not just tasks;
- duplicate/reconciliation keys for generated tasks;
- completion evidence schemas per task type.

Manager attention should include blocked/needs-manager-review tasks, high/critical priority, incidents, medications, documents, capacity/labor risks, pet safety/care risks, revenue/payment leakage where approval is needed, sensitive draft messages, playgroup/capacity/payment/policy/provider/customer-facing gates.

## Open implementation questions

1. Which pilot role taxonomy is approved, including playgroup/daycare staff, bather, authorized medication staff, document reviewer, payment reconciliation, and manager/admin variants?
2. Which task categories may be auto-created in production, and what source freshness, due-time, duplicate-key, role, and completion-evidence policies are required?
3. What exact SLAs/due windows apply to check-in prep, document review, feeding, medication, cleaning, playgroups, daily updates, incidents, message approvals, checkout prep, and complaints?
4. What staff-to-pet ratio, capacity, playgroup lane, individual-care, and reinstatement policies apply by location/service/time block?
5. What provider/Gingr integration mode is MVP for reads, writes, reservation status, notes, tasks, messages, payments, documents, and inventory/capacity?
6. Which daily updates/Pawgress/checkout reports are required by service, package, customer preference, location policy, or membership?
7. Which customer-message templates are approved for routine staff approval, manager approval, deterministic send paths, or draft-only workflows?
8. What room/accommodation/yard model controls cleaning and capacity return?
9. What retention/redaction policy applies to staff notes, incident media, document images, medication/medical evidence, payment/policy notes, rejected AI outputs, and audit records?
10. Should manager review queues be first-class aggregates, specialized `StaffTask` projections, or both?

## Conservative default

When source facts, policies, permissions, or integration authority are unclear, the staff-operations system should create a sourced review state or draft recommendation, not invent policy or silently mark work ready. Staff-facing speed is valuable only when the readiness state is truthful, source-backed, role-aware, audit-preserving, and explicit about what remains proposed / requires approval.
