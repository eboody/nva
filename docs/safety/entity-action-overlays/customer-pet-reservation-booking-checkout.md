# Customer, pet, reservation, booking, checkout, and retention safety overlay

This overlay helps front-desk agents, front-desk leads, general managers, care/behavior reviewers, medical/document reviewers, payment reviewers, and retention operators reduce repeated booking, stay, checkout, and follow-up review work for customer/pet/reservation actions by showing what source facts automation may read, what it may draft or recommend, what a human must approve, and which outcome/audit record proves safe use.

The page is an operator safety overlay, not a grant of live execution authority. Provider/PMS facts, payment records, source documents, local policy, and human approvals remain authoritative. Agent work stays inside read, summarize, rank, recommend, draft, internal-task, and outcome-record boundaries unless a separate approved system of record performs the live action.

## 1. Plain-English entity/action definition and labor-cost problem

The entity/action family is the customer, pet, and reservation record set used for booking intake, availability/capacity review, waitlist routing, confirmation/rejection drafting, cancellation review, check-in/out status, checkout handoff, and retention follow-up.

The repeated labor and error costs are:

- Front desk manually checks customer intent, pet profile, service, dates, capacity, vaccines/documents, deposits, behavior/care notes, and provider status before responding to booking requests.
- Leads and managers manually decide whether a request should be offered, waitlisted, rejected, routed to special review, or left in missing-info/data-quality cleanup.
- Checkout staff manually compare source checkout status with belongings, care summaries, departure-note review, payment exceptions, and follow-up blockers.
- Retention staff manually hunt for safe follow-up opportunities after completed stays while rechecking consent, source evidence, and customer-message safety.

Safe outcome: automation may prepare source-backed review packets, ranked queues, draft customer/internal language for approval, deterministic policy checks, manager/internal task drafts, and outcome records; it must not perform live booking, checkout, payment, provider/PMS, customer-send, medical/vaccine/behavior, schedule/capacity, destructive cleanup, or policy-change actions.

## 2. Workflows/contracts featuring it and adjacent entities

| Workflow or contract | Entity/action role | Adjacent entities | Source path or doc evidence |
| --- | --- | --- | --- |
| Booking triage | Reviews customer, pet, service, reservation, hard-stop, deposit, and policy evidence before a booking can be recommended or drafted for review. | Customer, pet, reservation, service, vaccine/document evidence, care/behavior notes, deposit/payment, policy, source ref, staff evaluation packet, confirmation draft, audit event draft. | [Booking Triage operator page](../../workflows/operator/booking-triage.md); [app/src/booking_triage.rs](../../../app/src/booking_triage.rs); [domain/src/entities.rs](../../../domain/src/entities.rs); [domain/src/reservation/README.md](../../../domain/src/reservation/README.md). |
| Boarding capacity and waitlist review | Determines whether a compatible room/suite segment appears available, full/waitlisted, or denied for manager review from a source inventory snapshot. | Location, species, accommodation preference, capacity snapshot, manager review gate, waitlist reason, denial reason. | [domain/src/boarding/capacity.rs](../../../domain/src/boarding/capacity.rs); [domain/src/boarding/minimum_stay.rs](../../../domain/src/boarding/minimum_stay.rs). |
| Cancellation and deposit/refund review | Explains cancellation notice and penalty evidence without waiving fees or promising exceptions. | Reservation, deposit/payment record, cancellation policy, manager/payment reviewer. | [domain/src/boarding/cancellation.rs](../../../domain/src/boarding/cancellation.rs); [domain/src/policy.rs](../../../domain/src/policy.rs). |
| Checkout completion | Compares source checkout/PMS status with staff handoff evidence before suggesting checkout completion or routing to handoff/source reconciliation. | Reservation id, source provenance, source reservation status, belongings, care summary, departure-note review, manager/customer-message review gates, audit-event drafts. | [Checkout Completion operator page](../../workflows/operator/checkout-completion.md); [app/src/checkout_completion.rs](../../../app/src/checkout_completion.rs); [domain/src/source.rs](../../../domain/src/source.rs). |
| Retention follow-up | Uses completed checkout evidence, customer/contact permission, source-grounded opportunity evidence, and consent/channel rules before preparing follow-up drafts or suppression tasks. | Customer, reservation, checkout packet, contact permission, opportunity evidence, source refs, message channel, staff review packet, outcome record. | [Grooming Rebooking / Retention operator page](../../workflows/operator/grooming-rebooking-retention.md); [app/src/crm_retention.rs](../../../app/src/crm_retention.rs). |
| Manager daily brief | Ranks source-grounded manager actions such as demand/staffing review, checkout exceptions, retention queue review, and data-quality issues while blocking side effects. | Location, operating day, service-demand facts, checkout packets, retention packets, manager brief actions, labor estimates, outcome records. | [app/src/manager_daily_brief.rs](../../../app/src/manager_daily_brief.rs); [storage/src/operations.rs](../../../storage/src/operations.rs). |
| Shared entity/domain model | Normalizes customer, pet, reservation, service, add-on, hard-stop, source, policy, and approval vocabulary used by the app workflows. | Customer, pet, reservation status/source, service kind, add-ons, hard stops, review gates, workflow packets, record refs/provenance. | [domain/src/entities.rs](../../../domain/src/entities.rs); [domain/src/reservation/mod.rs](../../../domain/src/reservation/mod.rs); [domain/src/workflow.rs](../../../domain/src/workflow.rs); [domain/src/policy.rs](../../../domain/src/policy.rs); [domain/src/source.rs](../../../domain/src/source.rs). |
| Outcome/storage evidence | Stores durable operational evidence where implemented, especially manager daily-brief and data-quality labor outcomes plus source refs. | Stored source record refs, outcome codes, before/after labor minutes, feedback, issue/source references, service-line records. | [storage/src/operations.rs](../../../storage/src/operations.rs). Dedicated booking/checkout storage projections are gaps unless later code adds them. |

## 3. Who/what is authoritative

| Fact or decision | Authoritative source | What it can prove | What it does not prove |
| --- | --- | --- | --- |
| Customer, pet, reservation, service, add-on, and hard-stop facts | Provider/PMS/read-model evidence promoted into [domain/src/entities.rs](../../../domain/src/entities.rs), with source/provenance from [domain/src/source.rs](../../../domain/src/source.rs). | The packet is tied to a named customer/pet/reservation/source fact. | Live permission to confirm, reject, cancel, check in/out, send a message, or mutate the provider/PMS. |
| Reservation lifecycle source state | Provider/PMS record or approved execution evidence, normalized as `domain::entities::reservation::Status` or observed as `domain::source::reservation::Status`. | Which lifecycle state was observed or suggested by a workflow. | That an agent executed or may execute the lifecycle transition. |
| Business invariant and review vocabulary | [domain/src/policy.rs](../../../domain/src/policy.rs), [domain/src/reservation/mod.rs](../../../domain/src/reservation/mod.rs), [domain/src/boarding/capacity.rs](../../../domain/src/boarding/capacity.rs), [domain/src/boarding/minimum_stay.rs](../../../domain/src/boarding/minimum_stay.rs), and [domain/src/boarding/cancellation.rs](../../../domain/src/boarding/cancellation.rs). | Which semantic rules, thresholds, review gates, capacity outcomes, and penalty vocabularies exist. | Provider write-back permission, payment movement, customer-send approval, or local-policy override. |
| Booking triage packet | [app/src/booking_triage.rs](../../../app/src/booking_triage.rs) `DeterministicResult`, `StaffEvaluationPacket`, `ConfirmationDraft`, `SafeAgentAction`, `BlockedAction`, and `AuditEventDraft`. | Readiness bucket, approval gates, blocked actions, safe draft/internal work, and audit-event draft markers. | Live confirmation/rejection, waitlist movement, provider/PMS mutation, payment action, vaccine/care/behavior approval, or message send. |
| Checkout packet | [app/src/checkout_completion.rs](../../../app/src/checkout_completion.rs) `Request`, `Packet`, `CompletionStatus`, `StaffHandoff`, `SafeAgentAction`, `BlockedAction`, and `AuditEventDraft`. | Whether source checkout plus staff handoff evidence supports a checkout-completion suggestion or review route. | Live PMS checkout mutation, checkout closeout, payment/refund/discount movement, capacity release, or customer message send. |
| Retention packet/outcome | [app/src/crm_retention.rs](../../../app/src/crm_retention.rs) `ContactPermission`, `FollowUpEligibility`, `StaffReviewPacket`, `Packet`, `SafeAgentAction`, `BlockedAction`, and `OutcomeRecord`. | Eligibility/suppression, required review gates, source refs, draft channel, blocked actions, and staff-recorded follow-up outcome shape. | Live customer outreach, booking/rebooking creation, discount application, payment movement, provider/PMS/calendar mutation, or consent override. |
| Manager daily brief/action value | [app/src/manager_daily_brief.rs](../../../app/src/manager_daily_brief.rs) and [storage/src/operations.rs](../../../storage/src/operations.rs). | Ranked source-grounded manager actions, estimated/actual labor minutes, feedback/outcome evidence where stored. | Staff schedule changes, customer messages, provider/PMS writes, money movement, hidden data-quality cleanup, or guaranteed ROI. |
| Human approval | Named operational reviewer and approval record/review gate. | Permission for that one reviewed downstream step. | Permission for unrelated actions or broad future automation. |

## 4. Agent may read

The agent/app workflow may read only source-backed facts needed for the specific packet or review queue:

- Customer/account identity, contact preference, portal account reference, and customer id when carried by `domain::entities::Customer` or workflow packets.
- Pet identity, species, age/birth date when present, sex, spay/neuter status, temperament profile, care profile, medications, allergies, medical conditions, veterinarian/emergency contacts, and hard-stop evidence when present in `domain::entities::Pet` and `Reservation`.
- Reservation id, location id, customer id, pet ids, service kind, lifecycle status, dates, deposit status, source channel, add-ons, and hard stops from `domain::entities::Reservation`.
- Reservation policy vocabulary such as minimum age, add-on label, and transition reason from `domain::reservation`, but only as explanation/evidence, not execution authority.
- Boarding capacity snapshots by location/species/accommodation preference from `domain::boarding::capacity::Request` and `Snapshot`; scope is the source inventory snapshot, not broad room-management control.
- Boarding minimum-stay and cancellation policy values from `domain::boarding::minimum_stay` and `domain::boarding::cancellation`; scope is evidence for review, not fee/exception execution.
- Booking-triage source evidence, readiness buckets, rule evaluations, evidence refs, approval gates, safe actions, blocked actions, and audit-event draft markers from `app::booking_triage`.
- Checkout source provenance, observed source reservation status, reservation id, staff handoff actor/time, belongings status, care summary, departure-note review, completion status, safe actions, blocked actions, and audit-event draft markers from `app::checkout_completion`.
- Retention checkout packet, contact permission, allowed/preferred channels, consent status, source record refs, source-grounded opportunity evidence, eligibility, draft channel, and outcome evidence from `app::crm_retention`.
- Manager daily-brief source facts, checkout/retention packets, source refs, operating day/location scope, demand facts, labor estimates, action priorities, and outcome records from `app::manager_daily_brief` and `storage::operations` where implemented.
- `domain::source::RecordRef` and `Provenance` values, timestamps, source system names, source record ids, adapter versions, and source data-quality errors used to keep recommendations auditable.

If evidence is missing, stale, ambiguous, conflicting, unmapped, unscoped, or unsupported by source refs, the safe behavior is to keep the item in review/data-quality cleanup, suppress customer-facing drafts, or fail closed instead of inferring readiness.

## 5. Agent may draft/recommend/rank/record

| Allowed action | Artifact it may produce | Boundary |
| --- | --- | --- |
| Summarize booking evidence | Booking-triage evidence summary or internal task draft from `SafeAgentAction::{EvidenceSummary,InternalTaskDraft,ManagerPacketDraft}`. | Review packet/internal-task only; no provider/customer/payment side effect. |
| Recommend booking review route | `ReadinessBucket`, `ApprovalGate`, `FailureCode`, `BlockedAction`, and `AgentRecommendedAction` in `app::booking_triage`. | Source-backed recommendation only; staff/manager/reviewer owns action. |
| Draft missing-info or confirmation copy | `CustomerMessageDraft` inside `ConfirmationDraft`, only when deterministic gate allows `DraftConfirmationAllowed`. | Draft-only with `CustomerMessageApproval`; never send directly. |
| Suggest reservation status for staff review | `StaffEvaluationPacket::suggested_status` mapping readiness to normalized status. | Suggestion only; provider/PMS lifecycle state is unchanged. |
| Rank or flag capacity/waitlist outcomes | `boarding::capacity::Decision::{Available,Waitlist,Deny}` with reasons and `ManagerApproval` review gate on denial. | Evidence for staff/manager review; no hold, release, room assignment, overbooking, or waitlist movement. |
| Summarize cancellation/deposit evidence | Cancellation `Penalty`, deposit readiness, or payment review packet. | Explanation/review only; no waiver, forfeit, refund, charge, discount, or reconciliation. |
| Summarize checkout evidence | `checkout_completion::SafeAgentAction::SummarizeCheckoutEvidence` and `CreateInternalHandoffTask`. | Internal/review packet only. |
| Suggest checkout completion | `CompletionStatus::StaffVerifiedCheckout`, optional suggested `reservation::Status::CheckedOut`, and audit-event draft when source status and staff handoff agree. | Suggestion/review only; no live PMS closeout, capacity release, billing closeout, or message send. |
| Draft retention follow-up | `crm_retention::SafeAgentAction::DraftCustomerFollowUpForReview` when eligible and source-grounded. | Draft-only with customer-message approval; no send, discount, rebooking, or provider write. |
| Record retention outcome evidence | `crm_retention::OutcomeRecord` with reservation/customer ids, actor, timestamp, outcome, provenance, and opportunity evidence. | Records staff evidence only; does not prove the agent contacted the customer or booked a stay. |
| Rank manager actions | `manager_daily_brief::BriefAction` and `SafeAgentAction::RankManagerActions` from source-grounded facts. | Manager-review/internal-task only; no schedule/customer/payment/provider side effect. |
| Record labor feedback/outcome where implemented | Manager daily-brief or data-quality storage records in `storage::operations`, plus app outcome records. | Outcome/value evidence only; no guarantee of future ROI. |

## 6. Agent must not do directly

| Blocked direct action | Why blocked | Correct safe path |
| --- | --- | --- |
| Send customer messages by email, SMS, phone, portal, or marketing channel | Customer trust, consent, DNC, medical/payment/safety wording, and channel preference require human/customer-message approval. | Draft message and route to customer-message reviewer/approved sender with source refs and suppression flags. |
| Confirm, reject, cancel, change, check in, or check out a booking/reservation in a provider/PMS | Provider/PMS lifecycle state is a system-of-record action, not a documentation or agent recommendation. | Prepare a staff evaluation packet, suggested status, and audit-event draft for staff/manager/provider-system execution. |
| Mutate provider/PMS/customer/calendar records | Source authority and integration safety require approved write-back paths and audit. | Draft provider update or internal task; route to staff/IT/security/product owner as appropriate. |
| Hold/release capacity, assign rooms/runs/groups, overbook, or move waitlist position | Capacity and placement affect pet safety, staffing, revenue, and customer promises. | Summarize capacity decision/waitlist reason and route to front-desk lead or manager. |
| Charge, refund, waive, discount, forfeit, reconcile, or move deposits/payments | Money movement requires verified payment records and payment/accounting or manager approval. | Prepare payment/deposit review evidence and blocked-action outcome; route to payment/accounting reviewer. |
| Approve vaccines/documents, medical readiness, medication/care exceptions, behavior/group-play exceptions, or incident-sensitive decisions | Pet safety and policy compliance require qualified reviewers. | Route to medical/vaccine reviewer, care team, behavior/daycare lead, or manager with source documents and hard-stop evidence. |
| Promise availability, acceptance, price, discount, refund, policy exception, or completion | Promises create customer and operational obligations that source packets do not authorize. | Use review-safe language such as “staff will review” and require the relevant approval. |
| Change staff schedules, payroll, staffing plan, or operating policy | Staff scheduling and policy changes require manager/product/ops authority. | Rank manager action or draft internal task for general manager/product/ops owner. |
| Hide, delete, overwrite, or destructively clean source records/data-quality issues | Destructive cleanup can erase audit evidence and source disagreement. | Record source/data-quality issue, route to data-quality/IT/security/product owner, and preserve refs. |
| Treat provider evidence as business policy | Provider payloads are evidence until promoted through domain/app policy contracts. | Cite source refs and policy snapshot; route unmapped/conflicting provider values to review. |

## 7. Required human reviewer role(s) and approval condition

| Role | Approves for this entity/action family | Does not approve |
| --- | --- | --- |
| Front-desk agent or front-desk lead | Routine intake completeness, missing-info follow-up drafts, source-backed queue work, checkout handoff quality, and internal tasks. | Manager exceptions, money movement, medical/vaccine/behavior approval, provider write-back authority, or policy changes. |
| Manager/general manager | Capacity/waitlist exceptions, staffing-sensitive decisions, cancellation exceptions, suppression/escalation, checkout handoff review, data-quality triage priority, and customer-trust decisions. Maps to `ManagerApproval` where represented. | Medical/vaccine validity unless qualified; direct payment processing unless assigned; IT integration/security scope. |
| Medical/vaccine qualified staff | Vaccine proof, medical/document ambiguity, medication/care readiness, and vaccine/document hard stops. Maps to medical/document review gates such as `MedicalDocumentReview`. | Payment/refund decisions, provider integration permission, broad customer-send approval outside reviewed medical wording. |
| Behavior/daycare lead | Temperament, group-play, behavior safety, in-heat/group-play exceptions, incident-care implications, and behavior hard stops. Maps to `BehaviorReview` where represented. | Payments, provider write-back, broad policy changes, or medical/vaccine proof validity. |
| Care team reviewer | Special-care, medication, allergy, mobility, feeding, and care-handoff acceptance. Maps to care-team review gates where represented. | Payment decisions, provider writes, customer sends outside approved care wording. |
| Customer-message reviewer/approved sender | Final recipient, channel, timing, body, consent/contact preference, suppression, and sensitive wording for booking, checkout, or retention drafts. Maps to `CustomerMessageApproval`. | Provider/PMS mutation, payment movement, medical/behavior/vaccine approval, or capacity changes. |
| Payment/accounting reviewer | Deposits, refunds, waivers, discounts, balances, receipts, duplicate/amount/provider ambiguity, and cancellation penalty outcomes. Maps to payment/refund/deposit review gates where represented. | Medical/behavior/schedule decisions, provider integration scope, or general customer-message body outside payment wording. |
| IT/security | Integration scope, provider write-back security, secrets, logging, rate limits, tool-port failure modes, data-quality cleanup safety, and blocked external side effects. | Business-policy approval, individual message/customer/payment decisions. |
| Product/ops owner | Whether a workflow, write port, outcome record, or policy level is allowed to exist; how it should be tested and documented. | Individual live-action approval without the proper operational reviewer. |

## 8. Required source evidence before a recommendation

| Recommendation or draft | Required source evidence first | If missing/stale/ambiguous |
| --- | --- | --- |
| Booking readiness bucket or staff packet | Reservation id/source, customer and pet ids, service/date facts, pet profile, policy snapshot, deposit status, hard stops, evidence refs, and source provenance. | Route to missing-info, special-review, failed-safe, or data-quality cleanup; do not recommend live confirmation/rejection. |
| Confirmation or missing-info customer draft | Deterministic booking result that allows a draft, customer contact preference, source-backed reason, approval gate, and no unresolved blocked action. | Suppress draft or make internal task; require customer-message approval before send. |
| Waitlist/capacity recommendation | Fresh inventory/capacity snapshot with location, species, accommodation preference, segment counts, timestamp/provenance, and manager gate for denial/override. | Route to capacity/manager review; do not promise availability, hold rooms, assign rooms, or move waitlist. |
| Cancellation/deposit review | Cancellation notice/penalty policy, reservation/source facts, deposit/payment evidence, local policy snapshot, and payment/manager reviewer context. | Route to manager/payment review; do not forfeit, waive, refund, discount, or promise fee outcome. |
| Check-in/out or checkout-completion suggestion | Observed source reservation status, source provenance, reservation id, staff handoff actor/time, belongings status, care summary, departure-note review, and review gates. | Route to source reconciliation or staff handoff review; do not suggest checked-out status if source or handoff evidence does not agree. |
| Retention follow-up draft | Staff-verified checkout packet, customer/reservation ids, contact permission with source refs, allowed/preferred channel, consent granted, opportunity evidence/provenance, and review gates. | Suppress draft, mark ineligible, create internal staff review task, or route to manager review; do not contact customer. |
| Manager daily-brief action | Location, operating day, source facts with source refs, checkout/retention packets where used, demand facts, labor estimate, owner persona, and blocked-action validation. | Do not rank as source-grounded; create data-quality issue or require manager/product/ops review. |
| Labor-value claim | Outcome record or feedback with actor/reviewer, timestamp, disposition, before/after or actual minutes, source refs, wrong-source findings, and reporting group. | State as estimate or gap only; do not claim measured savings or ROI. |

## 9. Outcome/audit record proving safe use and value measurement

| Proof needed | Example field or record | What it proves | What it does not prove |
| --- | --- | --- | --- |
| Source evidence | `domain::source::RecordRef`, `domain::source::Provenance`, `storage::operations::StoredSourceRecordRef`, source system, record type/id, observed timestamp, adapter version. | Recommendation was grounded in a named source fact. | Human approval or downstream completion. |
| Booking draft/recommendation | `booking_triage::StaffEvaluationPacket`, `DeterministicResult`, `ReadinessBucket`, `ApprovalGate`, `BlockedAction`, `AuditEventDraft::{PolicyDecisionRecorded,ReservationStatusSuggested,ConfirmationDraftGenerated,MessageApprovalRequested}`. | Staff packet and draft/audit markers were prepared. | Provider/PMS booking change, message send, payment movement, or vaccine/behavior/care approval happened. |
| Checkout draft/recommendation | `checkout_completion::Packet`, `CompletionStatus`, optional suggested `CheckedOut`, required review gates, `AuditEventDraft`, and blocked actions. | Source checkout and staff handoff evidence were classified. | Live PMS checkout/closeout, billing action, capacity release, or customer send happened. |
| Retention outcome | `crm_retention::OutcomeRecord` with reservation id, customer id, recorded actor/timestamp, `FollowUpOutcome`, source provenance, and opportunity evidence. | Staff disposition/outcome was recorded for follow-up value. | Agent contacted the customer, booked the next stay, or applied a discount. |
| Manager daily-brief labor value | `manager_daily_brief::OutcomeRecord` and storage manager daily-brief outcome records with before/actual minutes, minutes saved, outcome, recorded_by, source refs. | Reviewed manager action and measured labor minutes for that action where implemented. | Guaranteed future savings or permission for schedule/provider/customer/payment side effects. |
| Data-quality/value hygiene | Storage data-quality outcome records and source issue refs where used. | Wrong-source/source-quality findings and cleanup value can be audited. | Authority to delete or overwrite source records without approval. |
| Human approval | Approval record/review gate, reviewer role, approval status, decision reason, timestamp, source packet id. | Sensitive step was reviewed by the proper role. | Authority for unrelated future actions. |
| Blocked side-effect proof | Requested side effect, validation result, blocked action reason, `live_side_effects_allowed: false` or equivalent, correlation id. | Unsafe action was rejected or kept draft-only. | That no other system outside the workflow acted. |

Explicit value-measurement fields for this family should include: minutes avoided finding/reading source records; booking handle time; checkout audit time; rework reduced; wrong-source findings; missing/stale/conflicting source counts; staff/manager disposition; suppressed/deferred/approved/send-by-human counts; actual minutes; before/after minutes; minutes saved; feedback; reporting group; source refs; issue refs; packet/action/draft/outcome/correlation ids.

Current evidence supports app-level booking/checkout/retention packets, retention outcome records, manager daily-brief labor outcome records, and storage source refs. Dedicated durable booking-triage and checkout-completion outcome projections are not identified in the current storage evidence, so production labor-value reporting for those flows is a gap unless a later storage/API contract adds it.

## 10. Source/Rustdoc/test evidence links

Shared safety and navigation evidence:

- [Review boundaries matrix](../review-boundaries-matrix.md)
- [Source evidence map](../source-evidence-map.md)
- [Entity atlas review/safety boundaries](../../design/entity-atlas-review-safety-boundaries.md)
- [Overlay index](./README.md)
- [Overlay template](./template.md)

Operator workflow evidence:

- [Booking Triage](../../workflows/operator/booking-triage.md)
- [Checkout Completion](../../workflows/operator/checkout-completion.md)
- [Grooming Rebooking / Retention](../../workflows/operator/grooming-rebooking-retention.md)

Source evidence:

- [app/src/booking_triage.rs](../../../app/src/booking_triage.rs) for booking readiness, safe agent actions, blocked actions, confirmation draft gates, suggested status, audit-event drafts, and hard-stop/deposit evaluation.
- [app/src/checkout_completion.rs](../../../app/src/checkout_completion.rs) for checkout packet inputs, staff handoff, completion status, safe actions, blocked actions, required review gates, suggested checked-out status, and audit-event drafts.
- [app/src/crm_retention.rs](../../../app/src/crm_retention.rs) for retention opportunity evidence, contact permission, eligibility/suppression, review packet, blocked actions, safe actions, source refs, and outcome record.
- [app/src/manager_daily_brief.rs](../../../app/src/manager_daily_brief.rs) for manager action ranking, blocked side effects, labor estimates, source-grounded action checks, and outcome records.
- [app/src/agents.rs](../../../app/src/agents.rs) for agent prompt packets, workflow agents, forbidden action/review-gate context, and baseline safety language.
- [domain/src/entities.rs](../../../domain/src/entities.rs) for customer, pet, reservation, service, add-on, hard-stop, care/temperament, status/source, and actor/approval entities.
- [domain/src/reservation/README.md](../../../domain/src/reservation/README.md) and [domain/src/reservation/mod.rs](../../../domain/src/reservation/mod.rs) for reservation support vocabulary, minimum age, add-on labels, and transition reasons.
- [domain/src/boarding/capacity.rs](../../../domain/src/boarding/capacity.rs), [domain/src/boarding/minimum_stay.rs](../../../domain/src/boarding/minimum_stay.rs), and [domain/src/boarding/cancellation.rs](../../../domain/src/boarding/cancellation.rs) for capacity, waitlist/denial, minimum-stay, and cancellation/penalty policy evidence.
- [domain/src/workflow.rs](../../../domain/src/workflow.rs) for workflow events, policy context, recommended actions, allowed/blocked action vocabulary, results, and verification notes.
- [domain/src/policy.rs](../../../domain/src/policy.rs) for automation levels, vaccine requirements, group-play review, and `ReviewGate` vocabulary.
- [domain/src/source.rs](../../../domain/src/source.rs) for source systems, record refs, provenance, source snapshots, source reservation status, and data-quality promotion errors.
- [storage/src/operations.rs](../../../storage/src/operations.rs) for stored source refs, manager daily-brief labor outcome storage, data-quality outcome storage, and storage caveats that records do not authorize live side effects.

Test evidence:

- [app/tests/booking_triage_mvp.rs](../../../app/tests/booking_triage_mvp.rs) covers vaccine-review blocking, confirmation draft gating, audit-event drafts, behavior/special review, hard rejection dominance, and premature-draft rejection.
- [app/tests/checkout_completion_workflow_contracts.rs](../../../app/tests/checkout_completion_workflow_contracts.rs) covers checkout routes, manager/customer-message gates, source-not-checked-out behavior, and audit-event draft suppression/creation.
- [app/tests/crm_retention_workflow_contracts.rs](../../../app/tests/crm_retention_workflow_contracts.rs) covers retention eligibility, contact-consent suppression, blocked actions, source refs, and outcome capture.
- [app/tests/workflow_service_composition_contracts.rs](../../../app/tests/workflow_service_composition_contracts.rs) is cited by booking workflow docs for booking/deposit composition and payment-movement blocking.

Rustdoc evidence after running `cargo doc --no-deps --workspace` should exist under local `target/doc/` module paths for `app::booking_triage`, `app::checkout_completion`, `app::crm_retention`, `app::manager_daily_brief`, `domain::entities`, `domain::reservation`, `domain::boarding`, `domain::workflow`, `domain::policy`, `domain::source`, and `storage::operations`. This page cites source paths instead of relying on generated Rustdoc being present.

## 11. Open gaps or owner decisions

| Gap or owner decision | Why it matters | Safest current behavior | Evidence needed to close |
| --- | --- | --- | --- |
| Missing parent source inventory artifact under `docs/safety/entity-action-overlays/source-inventory.md` | Overlay writers lack the planned entity/action-specific inventory artifact. | Use [source-evidence-map.md](../source-evidence-map.md), source/Rustdoc/tests, and mark gaps explicitly. | Add the inventory artifact or update the overlay index to point to the replacement. |
| Dedicated durable booking-triage outcome projection not identified in storage | Booking labor-value claims need durable evidence, not intent. | Describe booking savings as estimated or app-packet supported; require outcome record fields before measured ROI claims. | Storage/API contract with source refs, disposition, handle time, wrong-source findings, reviewer, timestamp, and correlation id. |
| Dedicated durable checkout-completion outcome projection not identified in storage | Checkout audit savings and closeout value need durable outcome capture. | Use checkout packet/audit-event drafts for local proof; do not claim production checkout labor savings. | Storage/API contract for checkout outcome, handoff disposition, minutes saved/avoided, source refs, reviewer, and blocked side effects. |
| Provider/PMS write-back authority is not shown by these overlays | Live booking/cancellation/check-in/out actions require a separately approved execution path. | Keep provider actions as drafts/internal tasks/recommendations only. | Product/ops/IT/security-approved write-port contract, tests, approval record, audit logging, rollback/rate-limit behavior, and safety docs. |
| Capacity hold/release and waitlist movement execution are not authorized by capacity evidence | Availability evidence can be stale and customer-impacting. | Route to front-desk lead/manager review; do not promise or execute holds, releases, assignments, overbooking, or waitlist moves. | Source snapshot freshness rules, approved capacity write path, manager approval contract, and tests. |
| Payment/refund/discount/forfeit execution is outside cited app packets | Money movement and fee promises require payment authority. | Draft payment/deposit review packets only; route to payment/accounting or manager. | Payment ledger/source integration, approval gate, audit tests, and storage outcome evidence. |
| Medical/vaccine/behavior/care approvals require qualified humans | Pet safety decisions cannot be inferred from source summaries. | Route hard stops to medical/vaccine, care, behavior/daycare lead, or manager review. | Reviewer approval record, policy snapshot, source document refs, and tests proving blocked automation. |
| Retention outcome persistence is app-level in cited evidence, not necessarily a dedicated durable storage table | Follow-up value must survive audit/reporting if used for operations metrics. | Cite `crm_retention::OutcomeRecord` as app outcome shape and avoid durable production persistence claims unless storage adds it. | Durable storage/API record mapping retention outcome fields to source refs, actor, timestamp, disposition, and metrics. |
| Customer-message send path is not authorized by draft evidence | Consent/channel/body approval and customer trust require review. | Draft only and route to approved sender; suppress when consent/source evidence is missing or opted out. | Approved message-send contract, consent source evidence, approval record, send audit, suppression tests. |
| Policy changes and automation-level changes need product/ops ownership | A docs page cannot promote draft-only workflows to live execution. | Treat `automation::Level` and review gates as current safety boundaries; do not broaden them in prose. | Product/ops decision, source code change, tests, Rustdoc, and updated safety matrix. |

## Final reviewer checklist

- Does the first section name a concrete labor or error cost for a pet-resort role? Yes: front-desk booking review, manager/lead routing, checkout audit, and retention follow-up hunting.
- Can a non-coder route each listed entity/action to the right reviewer? Yes: Section 7 maps front desk, manager, medical/vaccine, behavior/daycare, care team, customer-message, payment/accounting, IT/security, and product/ops roles.
- Are “agent may read,” “agent may draft/recommend/rank/record,” and “agent must not do directly” separate? Yes: Sections 4, 5, and 6.
- Are source evidence, human approval, draft creation, and outcome proof clearly different? Yes: Sections 3, 8, and 9 separate them.
- Are blocked actions precise and entity-specific? Yes: Section 6 names customer sends, provider/PMS writes, capacity/waitlist/room actions, payments/refunds/discounts, medical/vaccine/behavior/care approvals, destructive cleanup, schedules, and policy changes.
- Does every labor-value claim require outcome/audit evidence rather than intent? Yes: Section 9 requires source refs, reviewer/actor, disposition, minutes, wrong-source findings, and outcome records; Section 11 marks missing durable projections as gaps.
- Are source/Rustdoc/test links current and local? Yes: Section 10 uses local relative links and source/test paths.
- Are gaps marked as gaps instead of being filled with assumptions? Yes: Section 11 lists current gaps and safest behavior.
