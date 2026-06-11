# Staff dashboard views

Purpose: define internal staff dashboard views for operational awareness and daily execution. These views are staff/manager operating surfaces, not customer-facing policy or automation authority.

Status: draft workflow definition based on `docs/workflows/staff-operations-parts/inputs.md` and the current Rust/domain contracts. Use this as a product/design input until pilot policy, permissions, task-generation rules, and provider integration boundaries are approved.

Core safety posture:

- Views may summarize source records, highlight readiness, draft internal tasks, and produce AI-assisted suggestions when source facts and review gates are visible.
- Views must not silently mutate reservation status, approve check-in/check-out, create authoritative task automation, return capacity to inventory, send customer messages, charge/refund money, approve documents/vaccines, or decide playgroup/medical/safety exceptions without approved deterministic tool paths and human review gates.
- Customer-safe facts and internal-only facts must remain visually distinct. Internal-only facts include staff notes, care/behavior watchlist details, incident timelines, manager-review reasons, payment/reconciliation details, AI confidence, and draft message rationale.
- Staff task completion requires authorized staff/manager evidence. AI summaries, OCR, uploaded media, or inferred facts are not completion evidence by themselves.

## Shared dashboard semantics

### Global filters

Every view should support these common filters unless a narrower view explicitly says otherwise:

- Location and operating date.
- Service kind: boarding, day play, day boarding, grooming, training, DaySpa.
- Reservation/stay state: requested, missing info, vaccine pending, special review, waitlisted, offered, confirmed, checked in, active, checked out, cancelled, rejected.
- Staff task state: open, in progress, blocked, needs manager review, completed, cancelled.
- Staff task priority: low, normal, high, critical.
- Assignment: unassigned, assigned role, assigned named staff, shift owner.
- Review gate: front-desk collection, care-team review, lead review, manager review, provider/write-back approval, customer-message approval.
- Data freshness/source state: verified/current, stale, missing, conflicting, untrusted, AI draft.

### Global urgency model

Use consistent urgency/status language across views:

| Label | Meaning | Typical owner | Allowed view behavior |
| --- | --- | --- | --- |
| Ready | Required facts are present and no visible review gate blocks routine staff work. | Role-specific staff | Show next action and required evidence. Do not auto-complete. |
| Ready with expected collection | A predictable front-desk collection item is missing, but it does not require care/manager judgment. | Front desk | Show collection checklist and customer-safe script draft if approved for review. |
| Needs front-desk collection | Missing identity, contact, signature, belongings, document copy, or routine payment-method confirmation blocks throughput. | Front desk | Draft collection task or reminder; do not infer final eligibility/payment truth. |
| Needs care review | Feeding, medication, allergy, medical, temperament, handling, play, or observation facts are missing/uncertain. | Kennel, playgroup staff, lead staff | Route to care review; AI may summarize evidence and uncertainty only. |
| Needs manager review | Override, hard stop, capacity/ratio exception, policy exception, payment/refund/waiver/discount issue, incident/safety sensitivity, or customer-message sensitivity exists. | Manager/admin | Lock final action behind manager approval. |
| Blocked | A required source, policy, provider state, eligibility fact, capacity/labor fact, or safety decision is unavailable or conflicting. | Assigned owner plus escalation owner | Show blocker, source gaps, and escalation path; do not mark ready. |
| Critical now | Time-sensitive safety/care/customer throughput issue; late medication, active incident, unsafe ratio, departure blocked at pickup, etc. | Lead staff/manager | Promote to top, require acknowledgement, preserve audit trail. |

### Shared audit and source fields

Every row/card should be able to reveal:

- Source anchors: reservation id, pet id, customer id, staff task id, workflow event id, document id, policy snapshot/ref, provider ref when available.
- Source state: verified, missing, stale, conflicting, untrusted, or AI draft.
- Actor and timestamp for assignment, status changes, completion evidence, approval, provider action, and customer send.
- Review gate and authority boundary: who may approve, what action is blocked until approval, and why.
- Customer-facing-safe summary separated from internal-only notes and manager-only reasoning.

## Arrivals

### Primary users/roles

- Front desk / host: primary owner for check-in throughput, identity, routine collection, belongings, signatures, and customer-safe handoff.
- Kennel technician / care staff: care-plan review, room/yard prep, feeding/medication clarification, special handling readiness.
- Playgroup / daycare staff: same-day play eligibility, care-lane readiness, group-play review signals.
- Lead staff: triage of blocked or cross-team arrivals.
- Manager/admin: capacity, policy, payment, hard-stop, group-play override, sensitive-message, or safety exceptions.
- AI workflow worker: source-backed readiness packet, gap detection, draft internal tasks, status suggestions only.

### Data shown

- Reservation, pet, customer, service kind, expected arrival window, length of stay/daycare session, add-ons, arrival channel/source.
- Current reservation status and suggested next status, clearly labeled as suggestion.
- Readiness packet: identity/customer profile, pet profile, vaccines/documents, care instructions, medications, allergies, medical conditions, temperament/group-play observations, emergency/vet contact, room/yard/care-lane assignment if present, deposit/payment readiness from trusted sources only.
- Open tasks tied to arrival: `CheckInPrep`, `DocumentReview`, `CustomerFollowUp`, feeding/medication clarification, room/yard prep, belongings intake, daily-update setup, playgroup assessment.
- Internal-only alerts: hard stops, manager-review reasons, conflicting source facts, unsafe/stale AI extraction, staff notes that are not customer safe.

### Filters and sort order

- Filter by arrival window: overdue, now/next hour, today, tomorrow, future.
- Filter by readiness lane: ready, ready with expected collection, front-desk collection, care review, manager review, blocked/waitlisted.
- Filter by service kind, group-play requested/eligible/unknown, vaccine/document status, payment/deposit status, assigned room/care lane, assigned staff/role.
- Sort by urgency lane first, then scheduled arrival time, then priority, then unassigned/blocked tasks.

### Urgency/status semantics

- Critical now: pet/customer is present or expected imminently and there is a safety, care, missing eligibility, payment exception, or capacity/ratio blocker.
- Needs manager review: hard stop, overbooking/capacity exception, payment/refund/waiver issue, sensitive incident/behavior history, group-play reinstatement/suspension, or policy exception.
- Needs care review: unresolved feeding/medication/allergy/medical/temperament/handling facts.
- Ready with expected collection: routine document copy, signature, contact preference, belongings label, or payment-method confirmation when policy allows front-desk collection.

### Staff actions

- Open/start/assign arrival prep tasks.
- Mark collection item received with evidence and source reference.
- Add belongings inventory and intake notes.
- Request care-team review, document review, payment follow-up, room/yard prep, or playgroup assessment.
- Record staff check-in packet completion evidence.
- Escalate blocker to lead/manager.
- Prepare but not execute a reservation status update unless provider write-back is approved.

### Manager-only controls

- Override or approve capacity/staffing exceptions, hard stops, special-review clearance, policy exceptions, late/no-show handling, payment/refund/waiver/discount decisions, group-play reinstatement/suspension, and sensitive customer-facing explanations.
- Approve provider write-back for check-in status where the integration mode requires human review.
- Configure which arrival task recommendations may become live tasks; until approved, AI tasking remains draft/recommendation.

### AI-assisted fields

- Arrival readiness summary with cited source fields.
- Missing-info checklist and source-state explanation.
- Draft staff tasks with due basis, priority rationale, suggested assignee, and review gate.
- Customer-safe collection script drafts for staff review.
- Suggested reservation status (`CheckedIn`, `SpecialReview`, etc.) with rationale; never an autonomous mutation.

### Audit/safety notes

- AI must not infer executable medication/feeding instructions from vague notes.
- Payment/deposit facts must come from trusted payment/deposit surfaces; customer or staff free-text claims are not payment truth.
- Final eligibility, vaccine clearance, group-play approval, capacity exception, and provider check-in mutation require the appropriate human/deterministic gate.

## Departures

### Primary users/roles

- Front desk / host: checkout throughput, customer pickup, belongings reconciliation, routine balance follow-up, customer-safe packet delivery.
- Kennel technician / care staff: care completion, medication/feeding/incident note resolution, pet release readiness.
- Groomer/bath/DaySpa staff and trainer: add-on completion evidence and checkout-linked follow-up where applicable.
- Lead staff: blocked departure triage and shift handoff.
- Manager/admin: payment/refund/waiver/discount, late pickup, incident/medical/sensitive-message, and release exceptions.
- AI workflow worker: departure readiness packet, draft checkout tasks/messages, status suggestion only.

### Data shown

- Reservation/stay, pet/customer, service kind, scheduled checkout/pickup window, late-departure risk, room/condo/yard/care-lane assignment.
- Required tasks: feeding, medications, play/enrichment, grooming/bath/training add-ons, incident follow-up, daily update/Pawgress report, belongings, payment/final balance, room/turnover.
- Release packet: care summary, unresolved exceptions, approved/suppressed customer update state, pickup authorization if modeled, customer-safe notes vs internal-only notes.
- Trusted billing status summary without raw payment/provider payloads.

### Filters and sort order

- Filter by pickup window: overdue, now/next hour, today, tomorrow.
- Filter by readiness: ready for pickup, care incomplete, report/message approval needed, payment/balance review, belongings review, incident/manager review, turnover pending.
- Filter by service kind, room/yard/condo, add-ons, late-departure risk, assigned owner.
- Sort by pickup time, then critical blockers, then manager-review items, then open checkout-prep tasks.

### Urgency/status semantics

- Critical now: pet/customer present for pickup while release, payment, incident, medical, or manager gate is unresolved.
- Needs manager review: payment exception, refund/waiver/discount, late pickup policy, unresolved incident/medical/safety matter, sensitive customer explanation.
- Blocked: required care/add-on evidence missing, unresolved medication/feeding/incident facts, missing approved customer report, or room turnover not created before capacity return.

### Staff actions

- Start/complete checkout prep tasks with evidence.
- Reconcile belongings and flag missing items.
- Confirm care/add-on completion or block with reason.
- Draft/report customer update for approval.
- Create payment follow-up/reconciliation task; do not charge/refund/waive.
- Create room/yard/condo turnover task before marking capacity as sellable through any approved path.
- Escalate late pickup, incident, release, or payment exceptions.

### Manager-only controls

- Approve release exceptions, late pickup handling, payment/refund/waiver/discount/credit/forfeiture decisions, incident/customer-sensitive wording, capacity return exceptions, and provider checkout/status mutation.

### AI-assisted fields

- Departure readiness summary with unresolved blockers.
- Draft customer-safe pickup/care summary for review.
- Draft internal checkout and turnover tasks.
- Suggested status (`CheckedOut`, `Active` with late-departure risk, `SpecialReview`) with source rationale only.

### Audit/safety notes

- Room capacity must not return to sellable inventory from an AI suggestion alone.
- Customer-facing reports involving health, medication, behavior, incident, safety, or payment require staff/manager review before send.
- Payment-sensitive details must remain minimal and role-appropriate.

## Active stays

### Primary users/roles

- Kennel technician / care staff: daily care execution, observations, feeding/medication handoff, cleaning.
- Playgroup/daycare staff: play/enrichment observation and care-lane execution.
- Groomer/bath/trainer: add-on execution and completion evidence.
- Lead staff: shift packet owner, blocked-care triage, assignment handoff.
- Manager/admin: safety, medical, incident, capacity/labor, policy, and customer-message exceptions.
- AI workflow worker: daily brief summarization, watchlist detection, handoff packet drafts.

### Data shown

- Active pets/stays by location/date/service, room/condo/yard/care-lane/group, assigned staff/role, current task burden.
- Care plan state: ready, blocked for staff review, blocked for manager/vet clarification.
- Open tasks: feeding, medication, cleaning, play/enrichment, daily update, incident follow-up, customer follow-up.
- Watchlist signals: medication due, feeding exception, anxiety/stress, behavior review, safety/care risk, late departure risk.
- Shift handoff items with owner, due timing, severity, source evidence, and next action.

### Filters and sort order

- Filter by service kind, room/condo/yard/group/care lane, assigned staff/role, due time, task kind, task status, watchlist reason, review gate.
- Sort by critical care/safety items, due/overdue tasks, medication timing, blocked tasks, then room/group order.

### Urgency/status semantics

- Critical now: active safety/incident signal, medication due/overdue, unsafe staffing/ratio signal, unresolved severe care ambiguity.
- Needs care review: ambiguous care plan, feeding exception, behavior/handling uncertainty.
- Needs manager review: safety risk, medical/vet clarification beyond staff authority, capacity/labor mismatch, incident escalation, sensitive customer communication.

### Staff actions

- Accept shift handoff and assign next owner.
- Start/block/complete care tasks with evidence.
- Record observation notes and source them as customer-safe or internal-only.
- Escalate care ambiguity, incident, medical, behavior, or staffing/capacity risks.
- Draft daily updates from approved evidence for review.

### Manager-only controls

- Approve safety/medical/incident/customer-sensitive handling, staffing/capacity exceptions, suppression/release of customer updates, and policy changes to care cadence or task generation.

### AI-assisted fields

- Daily brief and shift-handoff summary.
- Care watchlist explanation with cited tasks/notes.
- Suggested task drafts and reassignment suggestions.
- Draft daily/Pawgress update from approved evidence only.

### Audit/safety notes

- Staff notes that are not customer safe must not leak into message drafts without review.
- AI summaries do not close care tasks or resolve source conflicts.
- Every continuing obligation needs explicit next owner, due timing, evidence, and review gate.

## Feeding

### Primary users/roles

- Kennel technician / care staff: primary execution and evidence.
- Lead staff: blocked/exception triage and shift handoff.
- Front desk: collection of routine clarifications from customer when routed and customer-safe.
- Manager/admin: allergy/medical/high-risk ambiguity or customer-sensitive exception.
- AI workflow worker: instruction summarization and exception detection, not feeding authorization.

### Data shown

- Pet/stay, room/care lane, feeding instruction source, meal schedule, portion/prep notes, allergies, refusal/exception history, owner/customer clarification state, due time, assigned staff, completion evidence.
- Instruction source status: reviewed/current, missing, stale, conflicting, customer-claim-only, AI-extracted draft.
- Related tasks: feeding, feeding review/clarification, customer follow-up, incident/medical follow-up.

### Filters and sort order

- Filter by due window, overdue, assigned staff/role, room/group, source state, allergy flag, exception/refusal, review gate.
- Sort by overdue/critical, then due time, then allergy/medical risk, then room order.

### Urgency/status semantics

- Critical now: overdue feeding for a medically sensitive pet, allergy conflict, refusal with health risk, or conflicting instructions at feeding time.
- Needs care review: incomplete instructions, refusal, appetite change, stress/behavior concern.
- Needs manager/vet clarification: medical condition, allergy conflict, repeated refusal, or customer-sensitive explanation.

### Staff actions

- Verify instruction source before execution.
- Prepare/feed/record meal with evidence.
- Record refusal, partial intake, or exception.
- Request customer/care/manager clarification.
- Carry unresolved feeding obligation into shift handoff.

### Manager-only controls

- Approve exceptions involving medical/allergy risk, disputed instructions, sensitive customer messages, or policy changes to feeding cadence.

### AI-assisted fields

- Source-backed feeding instruction summary.
- Missing/conflicting instruction detector.
- Draft clarification task or customer-safe question for review.
- Refusal/exception summary for handoff.

### Audit/safety notes

- AI must not invent portions, schedules, allergies, or medical relevance.
- Feeding completion requires staff evidence; OCR/free-text/AI extraction is only a draft source until reviewed.

## Medications

### Primary users/roles

- Kennel technician / authorized care staff: execution only after reviewed medication instruction is present.
- Lead staff: blocked dose triage and handoff.
- Manager/admin: medical/vet/customer-sensitive exceptions, skipped-dose handling beyond staff authority.
- Front desk: routed collection of missing non-clinical information when appropriate.
- AI workflow worker: medication schedule summarization and risk flagging only.

### Data shown

- Pet/stay, medication name, dose, schedule, administration route if modeled, source/review status, due time, assigned staff, prior completion/skipped-dose evidence, allergies/medical conditions, vet/emergency contact, clarification state.
- Related tasks: `MedicationAdministration`, medication verification, skip/exception, customer/vet/manager follow-up.

### Filters and sort order

- Filter by due/overdue, medication source state, assigned staff/role, room/care lane, skipped/exception state, manager/vet clarification needed.
- Sort by overdue/critical, then due time, then source ambiguity, then room order.

### Urgency/status semantics

- Critical now: dose due/overdue, medication conflict, missing reviewed name/dose/schedule at administration time, skipped dose requiring escalation.
- Needs care review: unclear storage/prep/administration note, stale schedule, missing evidence.
- Needs manager/vet review: conflicting medication facts, adverse reaction, owner dispute, sensitive health communication, repeated missed/skipped dose.

### Staff actions

- Verify reviewed name/dose/schedule/source before administration.
- Administer dose and record evidence.
- Block dose with explicit reason when source is missing/conflicting.
- Record skipped dose/exception and escalate.
- Carry unresolved medication obligation into shift handoff.

### Manager-only controls

- Approve exception handling, sensitive customer/vet communication, and any policy around who may administer/verify medications.

### AI-assisted fields

- Due medication timeline from reviewed records.
- Missing/conflicting/stale medication instruction flag.
- Draft escalation packet with source facts and uncertainty.
- Handoff summary for unresolved doses.

### Audit/safety notes

- AI must never infer executable medication instructions from vague notes, images, or customer prose.
- Medication tasks require reviewed name, dose, schedule, and source before they can be treated as executable.
- Skipped-dose or adverse-signal records are internal/sensitive unless manager-approved for customer communication.

## Cleaning / room-turnover

### Primary users/roles

- Kennel technician / care staff: daily cleaning and room/condo/yard turnover execution.
- Lead staff: assignment balancing and blocked turnover triage.
- Front desk/manager: capacity-readiness visibility, not cleaning completion unless policy allows.
- Manager/admin: capacity return exceptions, policy/cadence configuration.
- AI workflow worker: draft turnover/daily-cleaning tasks from departure/active-stay signals only.

### Data shown

- Room/condo/yard identifier where modeled, reservation/stay link, pet/service, cleaning type: daily care vs departure turnover, due time, assigned staff/role, status, completion evidence, blocker reason, capacity impact.
- Related departure/check-in: next expected arrival, capacity hold, late departure risk, incident/contamination/special-cleaning flags if modeled.

### Filters and sort order

- Filter by cleaning type, room/yard/condo, status, assigned staff/role, due/overdue, capacity-impacting, blocked, next-arrival time.
- Sort by next-arrival/capacity criticality, overdue, blocked, then room order.

### Urgency/status semantics

- Critical now: room/yard/condo needed for imminent arrival or capacity release but turnover is incomplete/blocked.
- Needs care/lead review: cleaning blocked by pet still present, incident/contamination, unclear room identity, missing completion evidence.
- Needs manager review: capacity return exception or policy override.

### Staff actions

- Accept/complete daily cleaning or turnover task with evidence.
- Block task with reason and source.
- Link turnover to departure and next arrival where available.
- Escalate capacity-impacting blockers.

### Manager-only controls

- Approve capacity return exceptions, cleaning policy/cadence, and any automation that treats cleaning completion as inventory availability.

### AI-assisted fields

- Draft turnover task when departure readiness indicates one is required.
- Capacity-impact summary and next-arrival risk.
- Suggested priority based on due time and capacity impact.

### Audit/safety notes

- A room should not return to sellable inventory solely because AI predicted or drafted a turnover task.
- Cleaning completion requires staff evidence and, where policy requires, supervisor/manager approval.

## Playgroups

### Primary users/roles

- Playgroup/daycare staff: primary execution, observation, and staff-confirmed care-lane/group decision.
- Kennel technician: alternative individual-care lane execution where role taxonomy has not split playgroup staff.
- Lead staff: roster balancing, blocked assignment triage, incident/reassignment handoff.
- Manager/admin: ratio/capacity exceptions, reinstatement/suspension, safety/behavior-sensitive decisions.
- AI workflow worker: evidence collection and draft suggestions only; playgroup suggestion automation remains an approval gate.

### Data shown

- Pet/daycare or boarding stay, requested play/enrichment, service kind, group-play eligibility state, vaccine/spay-neuter/temperament/incident source state, care-lane/group assignment if staff-confirmed, staff coverage/ratio signal, capacity signal, playgroup assessment task, behavior observations.
- Distinguish customer-safe play summary from internal-only behavior/safety notes.

### Filters and sort order

- Filter by day/time block, group/care lane, staff/ratio state, eligibility state, vaccine/document state, temperament state, incident history, reassignment needed, manager review.
- Sort by critical safety/ratio issues, unassigned pets, upcoming play block, blocked assessments, then group/care lane.

### Urgency/status semantics

- Critical now: pet is in/near play block with unsafe ratio, conflicting eligibility, active behavior/safety signal, or incident/reassignment need.
- Needs staff review: unknown/stale/conflicting temperament, first-time assessment, unclear care lane, observation needed.
- Needs manager review: group-play override, reinstatement after incident, suspension, capacity/ratio exception, sensitive customer-facing behavior language.

### Staff actions

- Review eligibility evidence and confirm care lane/group where policy allows.
- Record observations and reassignment signals.
- Move pet to individual-care review when uncertain; do not default to group play.
- Escalate incident, behavior, ratio, or capacity concerns.
- Draft customer-safe play note for review.

### Manager-only controls

- Approve playgroup automation policy, capacity/ratio exceptions, group-play reinstatement/suspension, incident-sensitive messages, and final overrides.

### AI-assisted fields

- Evidence packet: vaccine/spay-neuter/temperament/incident/staffing/capacity facts and gaps.
- Draft suggested lane/state such as `NeedsStaffReview` or candidate care lane, visibly non-authoritative.
- Draft observation summary and customer-safe note for review.

### Audit/safety notes

- Unknown, stale, missing, or conflicting temperament/vaccine/spay-neuter/care/incident/staffing/capacity facts must not default to group play.
- Staff confirmation is required for behavior-based assignments; manager approval is required for overrides, reinstatement, capacity/ratio exceptions, and sensitive customer-facing language.

## Documents needing review

### Primary users/roles

- Front desk / host: collects/upload documents and routes missing proof.
- Lead staff: triages queue and operational blockers.
- Manager/admin or authorized reviewer: final approval/rejection/exception for vaccine/eligibility proof according to policy.
- AI workflow worker: OCR/extraction, gap detection, and review packet draft only.

### Data shown

- Pet/customer/reservation, document type, upload/source, OCR/extracted fields, vaccine/proof status, source confidence, expiration/freshness, missing fields, linked arrival/stay, operational blocker impact.
- Internal-only reviewer notes and AI extraction rationale separated from customer-safe request copy.

### Filters and sort order

- Filter by document type, review status, arrival date, vaccine pending, missing/expired/conflicting, AI extraction confidence/source state, assigned reviewer, service kind.
- Sort by imminent arrival/blocking status, critical/manager-review items, oldest pending review, then customer/pet.

### Urgency/status semantics

- Critical now: arrival/check-in is blocked by missing/expired/conflicting required proof.
- Needs reviewer: OCR extracted fields need human verification, proof type unknown, missing expiration/source details.
- Needs manager review: exception request, disputed document, policy ambiguity, hard stop, sensitive eligibility communication.

### Staff actions

- Request missing document/proof with approved/customer-safe language.
- Attach/source document to pet/reservation.
- Mark document as received/pending review, not approved unless authorized.
- Assign/reassign review task and link blocker to arrival/reservation.

### Manager-only controls

- Approve/reject vaccine/eligibility proof where policy requires manager/authorized reviewer; approve exceptions and sensitive customer language.

### AI-assisted fields

- OCR/extraction of names, dates, vaccine labels, clinic/vet source, expiration candidates, missing fields.
- Review packet: extracted facts, uncertainty, conflicts, source image/ref, linked reservations impacted.
- Draft customer request for missing/unclear proof.

### Audit/safety notes

- OCR/AI extraction is not document approval.
- Vaccine/source verification and eligibility effects require authorized review and audit record.
- Customer-facing eligibility/refusal language is sensitive and should be reviewed.

## Incidents

### Primary users/roles

- Kennel/playgroup/care staff: initial observation and immediate safety tasking.
- Lead staff: triage, containment, shift handoff, assignment of follow-ups.
- Manager/admin: incident classification, customer/vet/emergency communication, playgroup suspension/reinstatement, policy/legal-sensitive review.
- Front desk: customer coordination only after approved wording/path.
- AI workflow worker: timeline/evidence summary and draft follow-up packets only.

### Data shown

- Pet/stay, incident type/category if modeled, time, location/room/group, involved pets/staff, severity, current safety state, immediate actions taken, open follow-up tasks, manager-review state, customer/vet communication state, related playgroup/feeding/medication/document blockers.
- Customer-safe summary separated from internal-only notes, witness notes, manager rationale, and AI summary.

### Filters and sort order

- Filter by active/open, severity, manager review, customer/vet communication needed, playgroup impact, medication/medical involvement, service kind, location/area, assigned owner.
- Sort by active safety/critical, manager-review pending, communication due, follow-up due, newest.

### Urgency/status semantics

- Critical now: active safety/medical concern, unresolved containment, emergency/vet escalation, severe injury, missing owner/manager acknowledgment.
- Needs manager review: any incident requiring customer communication, policy exception, playgroup suspension/reinstatement, medical/vet decision, legal/privacy-sensitive facts.
- Needs care review: observation follow-up, behavior monitoring, routine care-plan update after lead triage.

### Staff actions

- Record initial observation and immediate actions with timestamp/source.
- Create/complete follow-up care tasks with evidence.
- Escalate to lead/manager and link affected stay/pets/tasks.
- Draft internal handoff and, when appropriate, customer message for review.

### Manager-only controls

- Approve incident classification, customer/vet/emergency communication, playgroup suspension/reinstatement, policy/legal-sensitive language, and any provider/customer-facing incident status mutation.

### AI-assisted fields

- Incident timeline from staff evidence and workflow events.
- Missing-evidence checklist.
- Draft manager review packet and customer-safe message for review.
- Suggested follow-up tasks; not auto-created as authoritative until task-generation rules are approved.

### Audit/safety notes

- Incident records are sensitive and often not customer-safe in raw form.
- AI must not decide fault, liability, medical conclusions, or customer-facing wording.
- Preserve immutable evidence and actor/timestamp trail for all edits, approvals, and sends.

## Message approvals

### Primary users/roles

- Front desk / host: routine customer-message review for approved/customer-safe scenarios.
- Care staff/lead staff: care-note fact review and customer-safe wording input.
- Manager/admin: sensitive health, medication, incident, behavior, safety, eligibility, refusal, payment, policy exception, refund/waiver/discount, complaint, or legal-sensitive messages.
- AI workflow worker: draft generation and evidence summarization only.

### Data shown

- Draft message, recipient/customer/pet/reservation, channel, message type, source evidence, reviewer required, sensitivity tags, current approval state, send eligibility, prior related sends, suppression reason if any.
- Explicit separation of customer-facing draft from internal rationale and source evidence.

### Filters and sort order

- Filter by approval state: draft, staff review, manager review, approved, sent, suppressed/cancelled.
- Filter by sensitivity tag, message type, channel, due/operational trigger, assigned reviewer, service kind, related incident/payment/document/care blocker.
- Sort by critical operational due time, manager-review sensitivity, pickup/arrival timing, oldest pending approval.

### Urgency/status semantics

- Critical now: customer is waiting or operational action is blocked, but message needs approval because of sensitive facts or policy/payment impact.
- Needs manager review: health/medical/medication/allergy, behavior/incident/safety, eligibility/refusal, payment/refund/waiver/discount, policy exception, complaint, legal/regulatory/privacy signal, or ambiguous source truth.
- Staff review: routine source-backed collection/reminder/update on approved template/send path.

### Staff actions

- Review draft against source evidence.
- Edit customer-safe wording.
- Request care/manager fact review.
- Approve routine staff-authorized messages only when policy/template/send path allows.
- Suppress draft with reason.

### Manager-only controls

- Approve sensitive customer-facing sends, exception language, payment/refund/waiver/discount/cancellation messages, incident/medical/behavior explanations, eligibility/refusal language, and any non-template send path.

### AI-assisted fields

- Draft message from cited approved evidence.
- Sensitivity classifier with reason and required reviewer.
- Source conflict/missing fact warnings.
- Customer-safe rewrite suggestion; internal rationale stays internal.

### Audit/safety notes

- AI must not send customer-facing messages unless a separately approved deterministic send path has fixed recipient, facts, template, and send condition.
- Prior AI summaries are not source truth.
- All approvals/sends need actor, timestamp, draft version, source refs, and reason for suppression/override.

## Manager attention roll-up

Although each view has its own manager gates, a staff dashboard should expose a cross-view manager attention roll-up based on current source semantics:

- `StaffTask` items in `Blocked` or `NeedsManagerReview` state.
- High/critical priority tasks.
- Incident, medication, and document-review task kinds.
- `OperationsRisk` values requiring manager attention: capacity constraints, labor mismatches, pet safety/care risk, and revenue/payment leakage where manager approval is needed.
- Draft customer messages with sensitive tags.
- Playgroup, capacity, payment, policy, hard-stop, provider mutation, and customer-facing gates.

The roll-up is a routing aid, not an approval substitute. It should link back to the source view and exact review packet rather than flattening sensitive details into a generic notification.

## Open implementation questions

- Which provisional roles should become durable permissions: playgroup/daycare staff, bather, authorized medication staff, document reviewer, payment reconciliation specialist?
- Which task recommendations may be auto-created in production, and under what source freshness/review-gate policy?
- What exact SLA/due-time rules apply to feeding, medication, cleaning, document review, daily updates, incident follow-up, and check-in/out prep?
- What room/yard/condo model controls cleaning and capacity return?
- Which provider/Gingr integration mode is allowed for each mutation: read-only, copy/paste, approved write-back, or no integration?
- What fields are customer-safe by default for daily updates, incidents, behavior, health, feeding, medication, and payment-related messages?
