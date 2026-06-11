# Daily task generation from reservations

Purpose: define proposed default rules for turning reservation, stay-lifecycle, care-plan, operating-day, and workflow-event data into staff task drafts or created staff tasks.

Status: draft operating model. These rules are not approved live policy. The task auto-generation gate remains open: production behavior must be approved by managers/location policy before automation creates authoritative tasks at scale, changes reservation status, sends customer messages, charges/refunds money, or marks care/safety/payment/customer-facing work complete.

## Source anchors

Use this document with:

- `docs/workflows/staff-operations-parts/inputs.md` as the canonical input packet.
- `domain/src/operations.rs` for current `StaffTask`, `StaffTaskKind`, `StaffTaskStatus`, `StaffTaskPriority`, `StaffTaskAssignment`, `StaffRole`, and `StaffTaskSource`.
- `domain/src/entities.rs` for reservation, pet, customer, service, status, add-on, hard-stop, actor, and audit anchors.
- `domain/src/workflow.rs` for workflow events, allowed actions, review gates, and recommended actions.
- Boarding implication docs for check-in/check-out, medication/feeding/behavior notes, shift handoffs, daycare group assignment, staff ratios, and fast front-desk throughput.
- Payments-pricing workflow docs for payment-sensitive check-in and checkout boundaries.

## Generation posture

Daily task generation should be modeled as a planner with four output levels, not as one universal auto-create switch.

1. Suppress: do not create a task because the trigger is outside scope, stale, duplicate, cancelled, or unresolved enough that creating work would be misleading.
2. Draft recommendation: create a reviewable task intent with source evidence, due basis, suggested role, priority rationale, and blocking reason. This is the safe default for ambiguous, sensitive, policy-dependent, or unapproved rules.
3. Auto-create internal task: create an ordinary staff task when the trigger, source evidence, due time, assignee role, and completion evidence are deterministic and approved by policy.
4. Manager-review task: create a task in `NeedsManagerReview` or with manager assignment when the underlying work may be required but cannot be delegated to ordinary staff without review.

Until the approval gate is closed, all rules below are proposed defaults. The implementation may draft/recommend them; production auto-creation requires explicit approved policy per task category, source, priority, and assignee.

## Shared task contract

Every generated task, whether draft or created, must carry:

- Location and operating day.
- Task kind or proposed future task kind.
- Reservation, pet, customer, daily-brief, or workflow-event source.
- Human-readable title that names the pet/reservation and reason without exposing unnecessary sensitive text.
- Due time and the rule that produced it.
- Assignment role or named staff owner if shift assignment is already approved.
- Priority and priority rationale.
- Completion evidence requirement.
- Review gate, if any.
- Source snapshot IDs, profile versions, workflow-event IDs, and policy refs used by the generator.
- Duplicate key: category + source entity + operating day + due slot + rule version, so reruns update/reconcile instead of creating duplicates.

Current source model fields:

- `StaffTask.kind`: `CheckInPrep`, `CheckOutPrep`, `Feeding`, `MedicationAdministration`, `PlaygroupAssessment`, `CleaningTurnover`, `DailyUpdateDraft`, `DocumentReview`, `IncidentFollowUp`, or `CustomerFollowUp`.
- `StaffTask.status`: `Open`, `InProgress`, `Blocked`, `NeedsManagerReview`, `Completed`, or `Cancelled`.
- `StaffTask.assignment`: `Unassigned`, `Staff(StaffId)`, or `Role(StaffRole)`.
- `StaffTask.source`: `Reservation`, `Pet`, `Customer`, `DailyBrief`, `WorkflowEvent`, or `StaffCreated`.

Modeling caveat: current task kinds are intentionally coarse. Some rules below name future sub-intents such as feeding verification, dose skip recording, or daily room clean. Until the task model is refined, encode the sub-intent in title/body/evidence metadata while using the nearest current `StaffTaskKind`.

## Operating-day planner inputs

The planner should run from operating-day state plus reservation/stay lifecycle, not a generic cron list. It should consume:

- Location, operating date, service calendar, and local policy refs.
- Arrivals/departures from reservations with statuses such as `Confirmed`, `CheckedIn`, `Active`, `CheckedOut`, `MissingInfo`, `VaccinePending`, `SpecialReview`, `Waitlisted`, `Cancelled`, and `Rejected`.
- Pet profiles: species, care profile, medication instructions, allergies, medical conditions, temperament/group-play evidence, emergency/vet contacts, vaccine/document status, and last-reviewed timestamps.
- Stay care plan: feeding, medication, special handling, play/enrichment, add-ons, incidents, handoff obligations, and daily-update state.
- Capacity, room/accommodation assignment, staffing/labor snapshot, daycare ratio snapshot, and yard/playgroup roster where available.
- Payment/deposit/readiness signals from trusted payment surfaces only.
- Existing tasks and completion evidence, so the planner reconciles instead of duplicating.
- Workflow events from customer edits, staff notes, manager approvals, incidents, no-shows, late changes, and provider imports.

## Due-time defaults

Exact times are policy inputs. Until approved, use relative due-time rules in artifacts and drafts:

- Pre-arrival tasks: due before the expected arrival/check-in window with enough lead time for staff review.
- Arrival blockers: due at or before arrival and surfaced in the front-desk readiness lane.
- Active-stay recurring care: due at care-instruction schedule slots or location-defined daily windows.
- Medication: due at the reviewed dose time, with pre-dose preparation/check lead time and overdue escalation policy.
- Feeding: due at the reviewed feeding time or service window, with exception follow-up before shift handoff.
- Play/enrichment: due before the relevant play block or daycare roster cutoff.
- Cleaning/housekeeping: daily care due during approved cleaning windows; turnover due immediately after departure/release and before room returns to inventory.
- Daily update: draft due after enough care/play/evidence exists and before the location's customer-update review/send window.
- Checkout prep: due before expected pickup/departure, with final readiness verified before release.

When exact policy is missing, draft a task with `due_at` based on the nearest known lifecycle event and mark the due rule as provisional instead of inventing clock times.

## Priority defaults

- Critical: safety/medical/medication exception, missing critical check-in requirement at arrival, incident follow-up blocking release or communication, or manager-required care decision.
- High: due-soon medication, care-plan blocker, checkout readiness blocker, ratio/capacity risk, customer-facing update requiring review before a promised window.
- Normal: routine feeding, routine cleaning, routine check-in prep, routine checkout prep, daily update draft.
- Low: non-urgent follow-up, optional enrichment/rebooking prompt, future-day prep with ample lead time.

The current domain model already treats high/critical tasks, medication tasks, document reviews, incident follow-ups, blocked status, and manager-review status as manager-attention candidates.

## Category rules

### 1. Check-in prep

Current task kind: `CheckInPrep { reservation_id }`.

Triggers:

- Reservation is `Confirmed`, `Offered`, `MissingInfo`, `VaccinePending`, or `SpecialReview` and has an expected arrival today or in the approved pre-arrival window.
- Customer, provider, or staff update changes a requirement that affects arrival readiness.
- Daily brief identifies expected arrivals or front-desk bottleneck risk.
- Reservation hard stop, missing document, care-plan blocker, deposit/payment gate, capacity/room assignment issue, or special handling flag exists.

Due-time rules:

- Pre-arrival prep is due before the arrival/check-in window.
- Same-day newly confirmed or changed reservations are due immediately or before arrival, whichever is sooner.
- If the guest is already at the front desk, the task becomes a front-desk blocker and should be high/critical depending on safety/payment/capacity impact.

Recurrence:

- One active check-in prep task per reservation per operating day.
- Regenerate/reopen when material readiness evidence changes, but reconcile against the duplicate key rather than creating parallel prep tasks.
- Cancel/suppress if reservation is `Cancelled`, `Rejected`, or already checked in with readiness accepted.

Assignment role:

- Default: `Role(FrontDesk)`.
- Assign or escalate to `Role(LeadStaff)` for care/room/throughput conflicts.
- Assign or escalate to `Role(Manager)` for hard stops, policy exceptions, capacity/ratio overrides, payment/refund/waiver exceptions, sensitive customer-facing language, or group-play reinstatement/override.
- Generate linked care tasks for kennel/play staff when feeding, medication, behavior, or special-handling review is needed.

Completion evidence:

- Arrival/readiness packet reviewed with identity, reservation status, service dates, pet profile, vaccine/document readiness, care-plan readiness, required signatures/forms, belongings plan, room/yard/capacity assignment, and payment/deposit readiness as applicable.
- Staff actor, timestamp, source snapshot, and explicit remaining gates.
- If completed with expected collection, evidence must name what remains collectable at arrival and why it is non-blocking.

Exception handling:

- Missing or stale vaccine/document proof creates/links `DocumentReview` and keeps check-in prep blocked or manager-review according to policy.
- Medication, feeding, allergy, medical, behavior, or handling ambiguity creates care-review tasks and may block check-in.
- Payment/deposit/refund/waiver/forfeiture issues route to manager/payment review; the task generator must not infer payment clearance.
- Capacity/room/staffing mismatch routes to lead/manager review.
- No-show/late arrival creates follow-up or manager review only under approved policy.

Customer-visible impact:

- Controls whether front desk can say the stay appears ready, ready with expected collection, or needs review.
- Does not itself send customer messages, accept/reject booking, update provider status, or promise an exception.

Audit requirements:

- Record trigger source, reservation status snapshot, readiness checklist, review gates, actor who completed/approved, status suggestion, and any provider/system mutation separately.
- Preserve evidence for why any blocker was downgraded to non-blocking.

Auto-generated vs manager-approved:

- Proposed auto-create: ordinary pre-arrival readiness task for confirmed same-day arrivals with deterministic reservation/source evidence.
- Draft/review only until approved: all production auto-creation rules.
- Manager-approved: hard stops, capacity/ratio exceptions, payment/refund/waiver decisions, safety/medical ambiguity, sensitive customer language, and provider status changes such as final `CheckedIn` mutation.

### 2. Feeding

Current task kind: `Feeding { pet_id }`.

Triggers:

- Active or checked-in stay has a reviewed feeding plan for the pet.
- Feeding instruction is missing, conflicting, newly changed, waived, or staff-review-required.
- Daily handoff carries an open feeding obligation.
- Staff records refusal, partial meal, wrong food, allergy concern, appetite change, or other exception.

Due-time rules:

- Verification/review is due before check-in acceptance or before the first scheduled feeding slot.
- Routine feeding is due at the reviewed meal time or approved meal window.
- Exception/refusal follow-up is due before the end of the current shift or sooner if medical/safety policy requires.

Recurrence:

- Create one execution task per pet per scheduled feeding occurrence during an active stay.
- Create one verification task when feeding instructions are first needed or materially changed.
- Continue recurrence across operating days until checkout/release, cancellation, or care-plan change.
- Future scheduled feeding tasks must be invalidated or regenerated when instructions change.

Assignment role:

- Default: `Role(KennelTechnician)`.
- Front desk may own collection of missing customer-provided details, but not final care interpretation if sensitive/ambiguous.
- Lead staff handles ordinary care-plan conflicts.
- Manager handles allergy/medical/waiver/safety-sensitive exceptions, repeated refusal, or customer-facing sensitive explanation.

Completion evidence:

- Staff actor, timestamp, feeding plan/source version, what was offered, whether eaten/refused/partial, exception note if any, and follow-up/handoff status.
- For verification: approved instruction source, reviewer, effective time, and whether future feeding tasks may be generated.

Exception handling:

- Missing/conflicting instruction: create review task; do not create routine execution tasks except as blocked drafts.
- Refusal, vomiting, allergy concern, appetite change, wrong food, or owner dispute: record exception, create handoff/watchlist item, and escalate according to severity.
- AI-extracted/free-text instruction is not executable until staff-approved.

Customer-visible impact:

- Feeding evidence may support a daily update or checkout report after review.
- Feeding problems, allergy/medical concerns, repeated refusal, or sensitive wording require staff/manager review before customer-facing text.

Audit requirements:

- Source provenance for instruction, staff completion actor, time, exception reason, task invalidation on instruction change, and link to daily update text if referenced.

Auto-generated vs manager-approved:

- Proposed auto-create: routine feeding execution tasks from reviewed care plans and approved recurrence windows.
- Draft/review only: feeding tasks from unreviewed imported/customer/AI text.
- Manager-approved: waiving required feeding info, allergy/medical exceptions, repeated refusal escalation, sensitive customer messages, and any policy exception.

### 3. Medications / special care

Current task kind: `MedicationAdministration { pet_id }`; special care may use `MedicationAdministration`, `IncidentFollowUp`, `CustomerFollowUp`, or a future special-care task kind depending on the model refinement.

Triggers:

- Active or checked-in stay has a medication instruction with reviewed name, dose, route, schedule, storage/handling requirement, and authorization source.
- Medication instruction is missing a required field, newly changed, conflicting, expired, source-unverified, or needs vet/manager clarification.
- Dose due, dose overdue, dose skipped, pet refused medication, medication unavailable, side-effect signal, medical condition note, allergy flag, or special handling requirement appears.
- Daily handoff carries open medication/special-care obligation.

Due-time rules:

- Verification is due before check-in acceptance or before first dose, whichever comes first.
- Administration is due at the reviewed dose time with any approved preparation/check window.
- Overdue/skipped/exception escalation is immediate or before shift handoff under local policy; safety-critical events are critical priority.

Recurrence:

- One task per medication dose occurrence unless local policy explicitly groups doses.
- Verification task per medication instruction version before execution recurrence begins.
- Regenerate future dose tasks when schedule/dose/source changes; preserve past evidence.
- Continue until checkout/end of schedule/discontinuation, with explicit audit for discontinued or skipped doses.

Assignment role:

- Default execution: `Role(KennelTechnician)` only after verification and if local policy permits that role.
- Verification/ordinary conflicts: `Role(LeadStaff)` or authorized care reviewer.
- Manager/vet clarification: `Role(Manager)` plus external vet/customer follow-up as policy allows.
- Front desk may collect owner/vet contact details but cannot resolve medical ambiguity alone.

Completion evidence:

- Staff actor, timestamp, medication instruction version, scheduled dose time, actual administration/skip time, dose outcome, exception reason if any, and handoff/watchlist outcome.
- Verification evidence must include reviewed name, dose, route, schedule, source/authorization, storage/handling, reviewer, and effective period.

Exception handling:

- Missing dose/schedule/route/authorization: do not create executable administration task; create blocked verification/clarification task.
- Changed medication during stay: invalidate future dose tasks tied to old instruction and create review event.
- Refusal, skipped dose, wrong dose risk, unavailable medication, side-effect, or medical emergency: escalate to manager/vet/emergency workflow and suppress routine customer update auto-send.
- AI may summarize medication evidence but must not infer medication instructions or mark doses complete.

Customer-visible impact:

- Medication and medical facts are sensitive. Daily updates or checkout reports that mention them require staff or manager review, and often manager approval for exceptions.

Audit requirements:

- Full source provenance, instruction versioning, dose task IDs, actor/timestamp, exception escalation, vet/customer contact attempts, approval gates, and links to any customer-facing drafts.

Auto-generated vs manager-approved:

- Proposed auto-create: routine dose tasks from reviewed medication plans when local policy authorizes the staff role.
- Draft/review only: all medication tasks from unverified, ambiguous, imported, AI-extracted, changed, or incomplete evidence.
- Manager-approved: vet clarification, skipped/incorrect/refused dose handling where policy requires, medical emergency/safety escalations, customer-facing medication exception language, and any waiver of required medication data.

### 4. Playtime / playgroup attendance

Current task kind: `PlaygroupAssessment { pet_id }`; execution/attendance may need a future play session, enrichment, roster, or care-lane task kind.

Triggers:

- Daycare/day-play/day-boarding reservation is expected or active.
- Boarding stay includes playtime, individual play, daycare add-on, group-play eligibility request, or enrichment add-on.
- Pet lacks current temperament/group-play review, has behavior/safety flag, has vaccine/spay-neuter/age/care constraints, or has recent incident/reinstatement question.
- Daily roster/ratio/capacity/staffing snapshot changes.
- Staff records behavior observation, group mismatch, incident, fatigue, special-handling need, or reassignment need.

Due-time rules:

- Eligibility/assessment task due before the play block or daycare roster cutoff.
- Attendance/roster execution due for each approved play block.
- Incident/reassignment follow-up due immediately or before the pet returns to group play.

Recurrence:

- For daycare: per operating day and per play block/roster slot as approved by location policy.
- For boarding add-ons: per requested play/enrichment occurrence.
- Reassess when temperament evidence, care restrictions, incident status, staffing/ratio/capacity, or owner request changes.

Assignment role:

- Default: provisional playgroup/daycare staff role; map to `Role(KennelTechnician)` until a separate role exists.
- `Role(LeadStaff)` for roster/care-lane adjustments and ordinary behavior observations.
- `Role(Manager)` for capacity/ratio exceptions, reinstatement after incident, suspension/removal decisions, sensitive customer-facing behavior language, or policy override.

Completion evidence:

- Eligibility source snapshot, group/care lane or individual-care decision, staff actor, time/block, attendance outcome, playgroup/yard if modeled, behavior observations, incidents/exceptions, and whether daily update may reference the session.

Exception handling:

- Unknown/stale/conflicting temperament, vaccine, spay/neuter, age, care, incident, staffing, or capacity evidence: do not default to group play; create staff/manager review.
- Ratio/staffing/capacity mismatch: block or route to manager/lead review, never silently overbook.
- Incident or aggressive/fearful behavior: create incident follow-up, suppress auto-send, and require manager review for sensitive language.

Customer-visible impact:

- Attendance and positive routine observations can support daily updates after review.
- Incidents, safety, exclusion, group-play denial, reinstatement, or behavior-sensitive claims require staff/manager review before customer-facing text.

Audit requirements:

- Eligibility inputs, reviewer/actor, roster/capacity/staffing snapshot, ratio basis, decision, attendance evidence, incident links, reassignment/reinstatement approvals, and customer-message review state.

Auto-generated vs manager-approved:

- Proposed auto-create: draft/ordinary assessment task for pets with requested daycare/play service and complete policy evidence; routine attendance tasks only after approved play-lane policy exists.
- Draft/review only: playgroup suggestions until the playgroup suggestion automation gate is approved.
- Manager-approved: capacity/ratio exceptions, incident reinstatement/suspension, safety-sensitive behavior decisions, and customer-facing exception language.

### 5. Cleaning / room reset / housekeeping

Current task kind: `CleaningTurnover { reservation_id }`; daily housekeeping may need future room/area identity and daily-cleaning task kinds.

Triggers:

- Reservation/stay checks in or becomes active and room/condo/kennel/yard/accommodation assignment exists.
- Daily operating-day housekeeping window for occupied rooms/condos or shared areas.
- Pet elimination, accident, illness, spill, contamination, bedding/water/bowl issue, incident, or staff observation requires cleaning.
- Reservation departs/checks out/release completed and room/accommodation must be reset before returning capacity to inventory.
- Room assignment changes or stay extends.

Due-time rules:

- Occupied-room daily housekeeping due during approved daily care/cleaning windows.
- Exception cleanup due immediately or by severity policy.
- Turnover/reset due immediately after departure/release and before the accommodation is marked sellable/available.

Recurrence:

- Daily cleaning recurrence for active occupied accommodation per policy.
- Event-driven cleanup for exceptions.
- One turnover/reset task per reservation/accommodation departure, regenerated if room assignment changes.

Assignment role:

- Default: `Role(KennelTechnician)` or approved housekeeping role if added later.
- `Role(LeadStaff)` for blocked room status, repeated sanitation issue, or capacity impact.
- `Role(Manager)` for disease/contamination/quarantine/safety policy, capacity release override, or customer compensation/refund implications.

Completion evidence:

- Staff actor, timestamp, room/condo/kennel/yard identifier if modeled, cleaning type, supplies/checklist used if required, before/after or inspection evidence if policy requires, and capacity-release readiness.

Exception handling:

- Missing room/accommodation identity: draft task cannot safely control capacity; route to front desk/lead to identify assignment.
- Biohazard/disease/contamination/quarantine: manager/safety review; do not return room to inventory automatically.
- Pet still present or belongings/medications unresolved: block turnover until release/belongings workflow allows it.

Customer-visible impact:

- Usually internal. Can affect checkout timing, capacity availability, and customer follow-up if belongings/accident/illness/damage is involved.
- Customer-facing mention of illness, incident, damage, or compensation requires review.

Audit requirements:

- Trigger, room/accommodation identity, cleaning checklist/evidence, actor/timestamp, blocked reason, capacity-release decision, and manager approval for exceptions.

Auto-generated vs manager-approved:

- Proposed auto-create: routine turnover after checkout/release and daily occupied-room housekeeping once room identity and cadence policy exist.
- Draft/review only: cleaning tasks without room identity, without approved cadence, or tied to sensitive incidents.
- Manager-approved: quarantine/contamination, room-capacity override, disease/safety issue, compensation/refund/customer-sensitive follow-up.

### 6. Daily customer update draft / review

Current task kind: `DailyUpdateDraft { reservation_id }`.

Triggers:

- Active boarding/daycare/day-boarding stay reaches the location's daily update window and enough reviewed care/play/evidence exists.
- Customer preference, membership/package, service type, or Pawgress/update policy requires or recommends an update.
- Significant care/play/grooming/training evidence arrives that should be summarized.
- Sensitive incident/medical/behavior/payment/policy fact exists and a suppression/review decision is needed.
- Checkout report is needed before departure.

Due-time rules:

- Draft due after the meaningful evidence window and before staff review/send window.
- Review due before promised customer-send time or checkout.
- Sensitive-content review due immediately when an incident/medical/behavior/payment blocker would otherwise make a routine update inaccurate or unsafe.

Recurrence:

- At most one daily draft/review workflow per reservation per operating day unless a manager approves additional incident/follow-up messaging.
- Continue daily while stay is active if policy requires updates.
- Cancel/suppress on checkout/cancellation when no report is due, but preserve suppression reason.

Assignment role:

- Draft generation may be `ActorRef::Agent`/system as draft-only.
- Default review: `Role(FrontDesk)` or `Role(LeadStaff)` depending on location workflow.
- `Role(Manager)` for medical, medication exception, allergy, incident, behavior-sensitive, safety, payment, eligibility, refusal, legal, or policy-exception content.

Completion evidence:

- Draft source evidence list, excluded/suppressed evidence reasons, reviewer actor, approved/sent/suppressed state, channel if sent by an approved tool, and timestamp.
- Completion of the task means the draft/review workflow is resolved; it does not imply the message was sent unless approved send evidence exists.

Exception handling:

- Insufficient evidence: create internal follow-up or suppress with reason, not hallucinated cheerful copy.
- Sensitive content: route to manager review and suppress auto-send.
- Conflicting care/task evidence: create staff clarification task before customer-facing approval.
- Payment/checkout/eligibility dispute: do not include or resolve without manager/payment review.

Customer-visible impact:

- Directly affects customer communications. AI may draft but must not send unless an approved send policy/tool path exists.
- Routine positive summaries may be staff-reviewed; sensitive summaries require manager approval.

Audit requirements:

- Source task IDs/evidence, AI draft prompt/model/version if applicable, reviewer, approval state, send/suppression decision, customer-visible text version, and links to incidents/care exceptions referenced.

Auto-generated vs manager-approved:

- Proposed auto-create: internal draft task when active stay policy says updates are due.
- Proposed draft-only automation: AI summarization from approved evidence.
- Manager-approved: sending messages involving health, medication, allergy, incident, behavior, safety, payment, eligibility, refusal, or policy exceptions; enabling any unattended customer send.

### 7. Checkout prep

Current task kind: `CheckOutPrep { reservation_id }`.

Triggers:

- Active or checked-in stay has expected departure/pickup today or inside approved prep window.
- Customer changes pickup time, adds checkout service, requests late pickup, or updates release/contact/belongings detail.
- Grooming/bath/training/add-on completion is needed before release.
- Open feeding/medication/incident/behavior/daily-update/payment/belongings/room-turnover dependencies exist.
- Staff marks pet ready for release or checkout flow begins at front desk.

Due-time rules:

- Prep due before expected pickup/departure.
- Same-day pickup changes due immediately.
- Release blockers due before pet handoff.
- Cleaning turnover task due at release/checkout completion, not before pet and belongings are out unless policy supports pre-cleaning.

Recurrence:

- One active checkout prep task per reservation per operating day.
- Regenerate/reopen when departure time, service/add-on completion, payment, daily-update, incident, medication, feeding, or belongings state changes.
- Cancel/suppress if reservation is cancelled/no-show before check-in or already checked out with post-checkout tasks resolved.

Assignment role:

- Default: `Role(FrontDesk)`.
- `Role(KennelTechnician)` for pet readiness, belongings, care-note handoff, and post-release turnover linkage.
- `Role(Groomer)` / `Role(Trainer)` for grooming/training add-on readiness where applicable.
- `Role(LeadStaff)` for unresolved care/add-on/room/handoff conflicts.
- `Role(Manager)` for payment/refund/waiver/forfeiture/discount/credit exceptions, incidents, medical/behavior sensitive release notes, late pickup policy exceptions, or release hard stops.

Completion evidence:

- Departure/release packet reviewed: pet identity, authorized release/customer identity if modeled, care/add-on completion or approved exception, belongings/medications reconciled, daily update/report reviewed/sent/suppressed, final balance/payment state routed, and turnover task created.
- Actor, timestamp, source snapshots, remaining post-checkout follow-ups, and status suggestion.

Exception handling:

- Open medication/feeding/incident/behavior task: block release packet or require lead/manager resolution before customer communication.
- Payment/final-balance/refund/waiver issue: route to manager/payment review; do not mark resolved from inference.
- Missing belongings/medications, unauthorized pickup, late pickup, or disputed service/add-on: create blocker/review task.
- Checkout status/provider update remains an approved action, not an automatic side effect of prep completion.

Customer-visible impact:

- Controls front-desk readiness, release explanation, checkout report, and whether a customer-facing summary can be sent.
- Does not itself collect money, refund/waive charges, release restricted pets, send messages, or mutate provider checkout status.

Audit requirements:

- Source evidence, readiness checklist, actor/timestamp, customer/release identity evidence if modeled, payment/review gate refs, approved/suppressed report, status suggestion, provider mutation record, and turnover task link.

Auto-generated vs manager-approved:

- Proposed auto-create: routine checkout prep for expected same-day departures and routine turnover link after release.
- Draft/review only: checkout prep with unresolved care/payment/incident/add-on/belongings/release-authority ambiguity.
- Manager-approved: payment/refund/waiver/credit/forfeiture decisions, late pickup exceptions, incident/medical/behavior-sensitive release messaging, release hard-stop overrides, and final provider checkout mutation.

## Cross-category exception rules

The planner should choose review over automation when any of these are true:

- Source facts are missing, stale, contradictory, provider-unverified, or AI-extracted only.
- The task would imply medical, medication, allergy, feeding, behavior, incident, safety, eligibility, capacity, staffing, payment, refund, waiver, forfeiture, discount, credit, cancellation, no-show, or legal policy judgment.
- A customer-facing message would mention sensitive facts or policy exceptions.
- The task would mutate an external provider/system, reservation status, payment state, room inventory, group-play status, or customer-message send state.
- The same duplicate key already has an open/in-progress/completed task and should be reconciled instead of duplicated.
- The assignee role is unresolved by the pilot role taxonomy.

## Reconciliation and cancellation

On each planner run:

1. Build candidate tasks from current operating-day state.
2. Match candidates to existing tasks by duplicate key.
3. Update due/priority/review metadata only when the source evidence is newer and the task is not completed by a human.
4. Do not overwrite human completion evidence.
5. Cancel or mark stale only when lifecycle evidence makes the work irrelevant, such as cancellation before check-in, checkout complete, instruction discontinued, play block removed, or room assignment changed.
6. Preserve audit for every generated, updated, blocked, cancelled, completed, or suppressed candidate.

## Audit event minimums

Every task-generation decision should leave enough audit data to reconstruct why staff saw the task:

- Rule/category/version.
- Triggering event/entity and source snapshot IDs.
- Policy refs and unresolved policy caveats.
- Duplicate key and reconciliation action.
- Generated status, priority, due time, assignment, and review gate.
- Suppression/block reason when no ordinary task was created.
- Actor: system/agent for generation, staff/manager for approval/completion.
- Customer-visible linkage: draft ID, review state, sent/suppressed state, or none.
- External mutation linkage if any approved provider/payment/message write occurs later.

## Proposed policy decisions still needed

Managers/location operators must approve or supply:

- Which categories may be auto-created vs draft-only in production.
- Exact due-time windows and escalation SLAs per service/location.
- Role taxonomy and role permissions for create/assign/start/block/complete/cancel/escalate.
- Feeding, medication, playgroup, cleaning, daily update, and checkout completion evidence standards.
- Playgroup suggestion/attendance automation boundaries.
- Room/accommodation model and whether cleaning completion controls sellable capacity.
- Customer-update requirement by service, package, customer preference, and content sensitivity.
- Provider integration mode for check-in/out, reservation status, message send, payment, and inventory updates.
- Required audit retention and reportability for task-generation and completion.

## Conservative default

When a rule is useful but not yet approved, generate a draft recommendation with source evidence and review gate. When the source is ambiguous or sensitive, generate a review task or suppress automation with a reason. Staff-facing speed is only acceptable when the task is truthful, sourced, deduplicated, assigned to an authorized role, and clear about what remains manager-approved.
