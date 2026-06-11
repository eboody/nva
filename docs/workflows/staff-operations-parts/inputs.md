# Staff operations inputs

Purpose: canonicalize the source inputs that downstream staff-operations workflow cards should use. This file is an input packet, not live operating policy. Anything marked provisional is a best-judgment modeling assumption that must remain reviewable until the relevant location policy, manager decision, or implementation artifact approves it.

Status: draft input collection for staff-operations definitions. Do not use this document to authorize live reservation updates, care decisions, playgroup assignments, payments, customer messages, staff scheduling changes, or task auto-generation in production.

## Source index

Primary sources checked:

- `docs/product/pet-resort-product-map.md` — requested canonical product-map path, but the file is not present in this repo at the time of this collection. Use the product-map board handoff metadata below until the artifact is copied into the repo.
- Kanban handoff `t_32dc302c` (`Collect canonical input constraints for data model`) — product scope, MVP scope, services, vaccination policy, booking/capacity assumptions, approval gates, Rust foundation constraints, and missing product-map artifact notes.
- `docs/architecture/domain-contract-skeleton.md` — current semantic domain-contract skeleton and future expansion rules.
- `docs/quality/semantic-code-doctrine-inventory.md` — implemented semantic baseline and known debt.
- `domain/src/entities.rs` — current location, customer, pet, reservation, service, deposit, audit, staff/manager actor, and reservation-status types.
- `domain/src/operations.rs` — current operating-day, daily brief, arrival/departure snapshot, labor, pet-care watchlist, operations action/risk, staff-task, staff-role, and lead/reputation/retention surfaces.
- `domain/src/workflow.rs` — current workflow-event, policy-context, allowed-action, workflow-result, and recommended-action envelopes.
- `/home/eran/.hermes/kanban/boards/pet-resort-meta/docs/context/nva-pet-resorts-context.md` — PetSuites/NVA public-context summary used by the meta board.
- `docs/domain/petsuites/boarding/implications/04-check-in-check-out-flows.md` — arrival/departure actors, inputs, decisions, outputs, and approval boundaries.
- `docs/domain/petsuites/boarding/implications/06-medication-feeding-behavior-notes.md` — stay-scoped care review, feeding/medication/behavior task planning, handoff, and safety boundaries.
- `docs/domain/petsuites/boarding/implications/07-staff-shift-handoffs.md` — shift-handoff triggers, actors, packet contents, owners, severity, evidence, and review gates.
- `docs/domain/petsuites/daycare/implications/02-group-assignment.md` — playgroup/care-lane assignment, eligibility, capacity/staff coverage, and automation boundary.
- `docs/domain/petsuites/daycare/implications/03-staff-to-pet-ratios.md` — ratio/staff-coverage policy, risk states, and manager override boundaries.
- `docs/domain/petsuites/daycare/implications/08-fast-front-desk-throughput.md` — front-desk readiness lanes, fast check-in/out inputs, and review gates.
- `docs/workflows/payments-pricing.md` and `docs/workflows/payments-pricing-parts/ai-boundaries.md` — deposit/payment and payment-sensitive reservation boundaries that staff-operations tasking must respect.

Missing or caveated sources:

- Product-map artifact missing from the requested repo path. The data-model handoff says earlier product-map workspace artifacts were also missing when checked and that board handoff metadata was used instead.
- Final `pet-resort-data-model` board artifact was not found in repo or the visible kanban board file search. Use the completed `t_32dc302c` metadata and existing Rust/domain-contract docs as the canonical data-model input for now.
- Many PetSuites policy details are public context, not pilot-approved local policy. Treat dollar amounts, exact timing, ratios, check-in windows, late rules, and customer-facing copy as unresolved unless a later approved policy artifact says otherwise.

## Product and scope inputs

From `t_32dc302c` and the PetSuites/NVA context:

- First product shape: internal staff/manager workflow and operations tool for a single pet resort or small resort group, with a later path to vertical SaaS after proof.
- Core actors: customer/owner, pet as deterministic record, front-line staff/caregiver, manager/admin, vet/emergency contact, and bounded AI workflow workers.
- Core services to account for: dog boarding, cat boarding, dog daycare/day play, day boarding/individual play, grooming/bathing/DaySpa, training where offered, loyalty/memberships, and optional webcam/customer update experience.
- MVP emphasis: intake/missing-info handling, pet/customer/reservation records, vaccine/document review queues, staff tasking, care/incident notes, capacity/availability snapshots, audit events, and draft messages/summaries.
- Manual for v1 unless later approved: final vaccine/eligibility/group-play decisions, booking acceptance/exceptions, deposits/payment/refund/loyalty handling, high-risk customer-facing sends, and any live Gingr/provider mutation beyond approved integration boundaries.
- Automation posture: AI/workflows may read, summarize, detect gaps, draft internal tasks/messages, and recommend review gates; they must not act as autonomous medical, safety, payment, eligibility, capacity-exception, or customer-message authorities.

## Provisional staff roles

These roles are provisional labels for workflow design. Map them to the existing `operations::StaffRole` enum where possible and do not infer payroll, permissions, or real titles from the labels.

| Provisional role | Existing source anchor | Likely responsibilities | Review/approval notes |
| --- | --- | --- | --- |
| Front desk / host | `operations::StaffRole::FrontDesk`; boarding/daycare front-desk implication docs | Customer arrival/pickup interaction, identity/reservation lookup, missing requirement collection, signatures/belongings/payment follow-up, customer-safe scripts, checkout prep. | Does not clear care/medical/behavior uncertainty, group-play exceptions, payment/refund exceptions, or sensitive messages without the correct gate. |
| Kennel technician / care staff | `operations::StaffRole::KennelTechnician`; boarding care/handoff docs | Feeding, medication execution after review, potty walks, cleaning/housekeeping, pet observations, room/condo care, daily task evidence, handoff items. | Medication/feeding/medical/behavior ambiguity creates review tasks; AI may not mark care complete. |
| Playgroup / daycare staff | Not a separate existing enum variant; currently closest to `KennelTechnician` or `LeadStaff` depending on implementation | Daycare roster execution, group-play observation, playgroup or individual-care lane confirmation, incident/behavior observation, reassignment signals. | Provisional role split. Downstream cards should decide whether to add `PlaygroupStaff`/`DaycareLead` or keep using `KennelTechnician`/`LeadStaff`. Playgroup assignment automation is an approval gate. |
| Groomer / bath / DaySpa staff | `operations::StaffRole::Groomer`; `entities::ServiceKind::{Grooming, DaySpa}`; grooming/exit-bath docs | Grooming appointments, baths, exit baths, service completion evidence, grooming notes, rebooking signals, grooming-linked checkout tasks. | Customer-facing recommendations and any charge/package/refund implications stay draft/review-gated. |
| Trainer | `operations::StaffRole::Trainer`; `entities::ServiceKind::Training` | Training consults/sessions where offered, training add-on evidence, follow-up notes and package/rebooking signals. | Training availability varies by location. Sensitive behavior claims require review. |
| Lead staff / shift lead | `operations::StaffRole::LeadStaff`; handoff docs | Shift packet acceptance, ordinary blocked-task resolution, incoming owner assignment, care/room/roster escalation triage. | Manager-only items cannot be cleared by shift lead unless approved policy grants it. |
| Manager / admin | `operations::StaffRole::Manager`; `entities::ActorRef::Manager`; approval gates in data-model and implication docs | Capacity/ratio overrides, safety/incident/medical/payment exceptions, sensitive customer language, policy configuration, staff/schedule review, final approval gates. | Required for manager-review gates, policy exceptions, refunds/waivers/forfeitures/discounts/credits, sensitive incident/medical/behavior messaging, overbooking/waitlist exceptions, and group-play reinstatement/override decisions. |
| AI workflow worker / system | `entities::ActorRef::{Agent, System}`; `workflow::AllowedAction` | Read entities, extract structured data, draft internal tasks/messages, suggest reservation status, suggest play eligibility, summarize care notes, flag risk. | Must preserve `policy::AutomationLevel` and `policy::ReviewGate`; suggestions/drafts only unless deterministic, verified, and explicitly allowed. |

Unresolved role assumptions:

- Whether “playgroup staff” deserves its own durable role or remains a kennel/lead assignment detail.
- Whether “bath” is separate from groomer in staffing/task assignment or only a grooming service subtype.
- Which front-desk actions are ordinary staff authority vs manager/admin authority for each pilot location.
- Whether task queues should assign to roles only, named staff only, or both with shift ownership.

## Daily operational process inputs

The staff-operations workflow should be modeled around operating-day state plus reservation/stay lifecycle, not a single generic task list.

### Pre-arrival / operating-day preparation

Source anchors: `operations::ResortOperatingDay`, `ResortDailyBrief`, `DailyBriefSection::Occupancy`, `ArrivalsAndDepartures`, `Labor`, `PetCareWatchlist`; boarding/daycare prep docs.

Inputs:

- Location/date snapshot, service capacities, labor/scheduled staff snapshot, arrivals/departures, open care/watchlist/revenue/follow-up signals.
- Reservation statuses from `entities::ReservationStatus`: `Inquiry`, `Requested`, `MissingInfo`, `VaccinePending`, `SpecialReview`, `Waitlisted`, `Offered`, `Confirmed`, `CheckedIn`, `Active`, `CheckedOut`, `Cancelled`, `Rejected`.
- Pet/customer/care/profile evidence: species, age, spay/neuter, vaccine proof, feeding, medications, allergies, medical conditions, temperament/group-play observation, emergency/vet contacts.
- Payment/deposit readiness only from trusted payment/deposit surfaces and approved policy snapshots.

Provisional task intents:

- `CheckInPrep`, `DocumentReview`, `CustomerFollowUp`, `PlaygroupAssessment`, `MedicationAdministration`/verification, `Feeding`/feeding review, `CleaningTurnover`, `DailyUpdateDraft`, `IncidentFollowUp`.

### Arrivals / check-in

Source anchors: boarding check-in/check-out doc, daycare fast-front-desk doc.

Staff workflow state should distinguish:

- Ready to check in: identity, reservation status, profile, eligibility, care instructions, capacity/assignment, deposit/payment preconditions, and customer-facing wording are resolved.
- Ready with expected collection: front desk can collect a predictable non-policy-blocking item such as a document copy, signature, contact preference, belongings label, or payment-method confirmation if approved by policy.
- Needs front-desk collection: missing information blocks throughput but is collectable without care/manager judgment.
- Needs care team review: feeding, medication, allergy, medical, temperament, group-play, incident, or handling uncertainty requires kennel/play/lead review.
- Needs manager review: hard stop, override, capacity/ratio exception, policy exception, payment/refund/waiver/discount issue, sensitive incident/customer message, or group-play reinstatement/suspension.
- Blocked or waitlisted: source unavailable, safety/policy block, capacity/staffing unresolved, missing required document, or unresolved deposit/payment gate.

Outputs should be packets and tasks, not silent state mutation:

- Arrival/readiness packet with review gates and source/audit refs.
- Staff task drafts for document review, feeding/medication clarification, room/yard prep, belongings intake, payment follow-up, Pawgress/daily-update setup, and playgroup assessment.
- Status suggestion only (`CheckedIn`, `SpecialReview`, etc.) until a provider update is approved and executed through a bounded tool.

### Active stays / day care execution

Source anchors: boarding care notes doc, daycare group assignment and ratio docs, operations task model.

Daily care states to model:

- Care plan ready / care plan blocked for staff review / care plan blocked for manager or vet clarification.
- Feeding obligations: verify instruction, prepare meal, feed pet, record refusal/exception, escalate exception.
- Medication obligations: verify medication, administer dose, record skipped dose, request vet clarification, escalate medication exception. Medication tasks require reviewed name/dose/schedule/source; AI must not infer executable medication instructions from vague notes.
- Play/enrichment obligations: group-play eligibility, playgroup/care-lane assignment, staff coverage/ratio decision, individual day boarding/cat enrichment alternatives, incident or behavior review.
- Cleaning/housekeeping obligations: room/condo daily care, elimination cleanup, water bowl/housekeeping, room turnover after departure. Exact cadence is policy/location input.
- Incident/behavior/safety signals: create care watchlist and manager/lead review; suppress autonomous customer-facing sends.
- Shift handoff: every continuing obligation gets an explicit next owner, severity, evidence, due timing, and review gate.

### Daily update / Pawgress / customer summary

Source anchors: PetSuites context, boarding care/check-out docs, `workflow::WorkflowEventType::{DailyNoteCreated, DailyUpdateNeeded}`, `workflow::AllowedAction::SummarizeCareNotes`, `DailyUpdateDraft` task kind.

Provisional model:

- AI may summarize approved task evidence and draft daily/Pawgress updates.
- Staff/manager approval is required before sending customer-facing text, especially if it references medical, medication, allergy, behavior, incident, safety, payment, or policy-exception facts.
- Daily update state should track source evidence, draft/review/approved/sent/suppressed status, and reason for suppression if applicable.

### Departures / check-out

Source anchors: boarding check-in/check-out doc, daycare fast-front-desk doc, payments-pricing docs.

Departure readiness should verify:

- Reservation/stay is checked in or active and eligible for departure workflow.
- Required care/add-on/play/grooming/training tasks are complete or explicitly review-resolved.
- Medication/feeding/incident/behavior notes are resolved enough to release the pet and communicate truthfully.
- Payment/final balance/package consumption is resolved or routed to the correct review gate.
- Belongings are reconciled.
- Daily update/Pawgress/customer report is approved/sent or intentionally suppressed with review reason.
- Room/yard/condo turnover task exists before capacity returns to sellable inventory.

Outputs remain drafts/recommendations until approved:

- Departure/release packet, checkout prep tasks, belongings review, payment follow-up, report review, cleaning turnover, incident follow-up, customer follow-up, and optional exit bath/grooming/training follow-up.
- Status suggestion to `CheckedOut`, `Active` with late-departure risk, or `SpecialReview` as appropriate.

## Data-model input summary

Use these current Rust/domain contracts as source inputs; do not wait for a missing final data-model artifact unless downstream cards specifically need it.

Existing implemented anchors:

- Identity and actors: `LocationId`, `CustomerId`, `PetId`, `ReservationId`, `StaffId`, `ManagerId`, `ActorRef::{Customer, Staff, Manager, System, Agent}`.
- Location/policy: `Location`, `Brand`, `LocationPolicyRefs` with vaccine/deposit/playgroup policy refs.
- Customer/pet: `Customer`, `PortalAccountRef`, `ContactChannel`, `Pet`, `Species`, `Sex`, `SpayNeuterStatus`, `CareProfile`, `MedicationInstruction`, `TemperamentProfile`.
- Reservation: `Reservation`, `ServiceKind::{Boarding, DayPlay, DayBoarding, Grooming, Training, DaySpa}`, `ReservationStatus`, `ReservationSource`, `AddOn`, `HardStop`.
- Audit: `AuditEvent`, `AuditSubject`, `AuditAction`, typed metadata key/value.
- Operations: `ResortOperatingDay`, `ResortDailyBrief`, `DailyBriefSection`, `OccupancySnapshot`, `ArrivalDepartureSnapshot`, `LaborSnapshot`, `CustomerFollowUp`, `PetCareWatch`, `OperationsRisk`, `OperationsAction`, `StaffTask`, `StaffTaskKind`, `StaffTaskStatus`, `StaffTaskPriority`, `StaffTaskAssignment`, `StaffRole`, `StaffTaskSource`.
- Workflow: `WorkflowEvent`, `WorkflowEventType`, `PolicyContext`, `AllowedAction`, `WorkflowResult`, `WorkflowStatus`, `RecommendedAction`.

Caveats and future expansion rules:

- Reservation lifecycle is currently runtime enum-oriented; use typestate only when workflow code needs phase-specific method legality.
- Public tuple ID constructors remain known debt before persistence/reconciliation behavior depends on construction policy.
- Vaccine source verification and richer medical/source proof remain future policy vocabularies before branching behavior grows.
- Audit/actor extraction should occur only when permissions, actor-scoped behavior, or audit search needs behavior beyond passive logging.
- Operations capacity/labor values are report/review signals until approved scheduling/capacity rules make them decision policies.

## Operational task-model assumptions

Current source model:

- Task record: `operations::StaffTask` with location, kind, title, status, priority, due time, assignment, source, and optional completion evidence.
- Assignment: `StaffTaskAssignment::{Unassigned, Staff(StaffId), Role(StaffRole)}`.
- Source: `StaffTaskSource::{Reservation, Pet, Customer, DailyBrief, WorkflowEvent, StaffCreated}`.
- Status: `Open`, `InProgress`, `Blocked`, `NeedsManagerReview`, `Completed`, `Cancelled`.
- Priority: `Low`, `Normal`, `High`, `Critical`.
- Manager attention rule currently triggers for blocked/manager-review status, high/critical priority, and incident, medication, or document-review task kinds.

Current task kinds:

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

Provisional gaps for downstream task-model cards:

- Staff task kinds do not yet distinguish feeding verification vs feeding execution vs refusal/exception recording.
- Medication task kind does not yet distinguish verify/administer/skip/clarification/escalation.
- Daycare/group-play tasks do not yet distinguish initial temperament assessment, roster assignment, ratio/capacity review, same-day reassignment, or incident reinstatement review.
- Handoff packet items are richer than `StaffTask`; downstream cards should decide whether packet items generate staff tasks, reference existing tasks, or both.
- Daily update/Pawgress draft state likely needs its own review/approval/sent/suppressed state rather than only `DailyUpdateDraft` task completion.
- Grooming/bath/training add-on completion evidence may need service-specific task kinds or a cross-service add-on task model.
- Cleaning tasks may need room/kennel/condo/yard identity and turnover-vs-daily-cleaning distinction before capacity can return to inventory.

## Approval gates to preserve

Human approval gates that must survive downstream decomposition:

1. Task auto-generation rules.
   - Approval needed before automation creates live staff tasks at scale or marks them authoritative.
   - Safe provisional behavior: AI/domain workflows may draft task recommendations with source, due basis, assignment suggestion, priority rationale, and review gate. Production creation rules require approved policy by trigger/kind/priority/assignee.
   - Completion must require authorized staff/manager evidence; AI summaries or media/free-text evidence alone must not close care/safety/payment/customer-facing tasks.

2. Playgroup suggestion automation.
   - Approval needed before any automated group-play, playgroup lane, daycare roster, or care-lane suggestion is treated as staff-ready.
   - Safe provisional behavior: automation can collect evidence and propose `NeedsStaffReview`, `AssignedToCareLane`, or draft playgroup suggestions, but unknown/stale/conflicting temperament, vaccine, spay/neuter, care, incident, staff-coverage, or capacity facts must not default to group play.
   - Staff confirmation is required for behavior-based assignments; manager approval is required for overrides, reinstatement after incidents, capacity/ratio exceptions, or sensitive customer-facing language.

Other gates carried from source inputs:

- Medical, medication, allergy, feeding, behavior, incident, and safety ambiguity.
- Vaccine/document approval and licensed-vet proof verification.
- Deposit/payment/checkout balance, refund, waiver, forfeiture, discount, credit, and cancellation exceptions.
- Booking acceptance/rejection exceptions, overbooking, waitlist release, capacity holds, and late departure exceptions.
- Customer-facing messages involving health, safety, legal, payment, incident, eligibility, refusal, or policy exceptions.
- Provider/system mutations such as changing reservation status, confirming check-in/out, sending messages, charging/refunding money, or returning room capacity to sellable inventory.

## Open questions for downstream cards

1. Should the product-map artifact be restored/copied to `docs/product/pet-resort-product-map.md`, or should downstream docs cite the kanban handoff metadata until synthesis is complete?
2. What is the approved pilot role taxonomy: front desk, kennel tech, playgroup staff, groomer/bather, trainer, lead staff, manager/admin, or a smaller set?
3. Which staff roles may create, assign, start, block, complete, cancel, or escalate each staff-task kind?
4. Which task kinds may be auto-drafted vs auto-created in production, and what source evidence/freshness is required?
5. What SLA/due-time rules apply to check-in prep, medication, feeding, cleaning, daily update, checkout prep, document review, and incident follow-up tasks?
6. Are daily updates/Pawgress reports required for all stays, only boarding, only specific add-ons, or location/customer preference dependent?
7. What exact location policies govern deposit collection at check-in, final balance at checkout, late pickup, checkout time, cancellation, and no-show handling?
8. What staff-to-pet ratio, capacity, and playgroup lane policies apply by location, service variant, time block, and incident posture?
9. Should cleaning/turnover tasks control when room capacity returns to inventory, or remain advisory until a room/accommodation model exists?
10. How should grooming/bath/training add-ons participate in staff handoffs, daily updates, checkout readiness, and revenue follow-up?
11. What provider/Gingr integration mode is MVP: read-only import, copy/paste, approved write-back, or no integration?
12. Which audit/event records must be retained for every task draft, created task, assignment, completion, approval, status suggestion, and provider action?

## Conservative downstream rule

When source facts or policy are missing, stale, contradictory, sensitive, or provider-unverified, downstream staff-operations workflows should create a review state or draft internal task, not invent policy or silently mark work ready. Staff-facing speed is valuable only when the readiness state is explainable, sourced, and review-gated.