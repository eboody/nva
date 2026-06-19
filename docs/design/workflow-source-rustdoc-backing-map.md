# Workflow source and Rustdoc backing map

Purpose: this is the citation map for the workflow-first operator pages. It separates code-derived evidence from operator-facing explanation so later pages can cite concrete source/Rustdoc targets without implying that planned automation is already shipped.

Scope: the map covers these planned operator pages:

- Manager Daily Brief
- Booking Triage
- Data Quality Hygiene
- Checkout Completion
- Grooming Rebooking / Retention
- Daily Updates / Pawgress drafts
- Regional Labor Exceptions / future portfolio view

Use this file as a backing artifact, not as final operator copy. If a claim below is not backed by a source file, test, design note, or generated Rustdoc/source-view target, it is labeled as a gap/planned.

## Shared source-of-record and safety boundaries

### Code-derived evidence

- Source lineage is represented by `domain::source::{Provenance, RecordRef, System}` in [`domain/src/source.rs`](../../domain/src/source.rs) and generated Rustdoc source view [`target/doc/src/domain/source.rs.html`](../../target/doc/src/domain/source.rs.html). `System` includes Gingr, BI, labor scheduling, timeclock, payroll, capacity inventory, POS, and manual import; `Provenance`/`RecordRef` preserve endpoint, record id, batch, schema, payload hash, raw payload ref, and request scope.
- Data-quality evidence is represented by `domain::data_quality::{Issue, Kind, FieldPath, Severity, ResolutionStatus}` in [`domain/src/data_quality.rs`](../../domain/src/data_quality.rs) and Rustdoc source view [`target/doc/src/domain/data_quality.rs.html`](../../target/doc/src/domain/data_quality.rs.html). The contract keeps missing fields, assumptions, unknown source statuses, duplicates, payment conflicts, checkout gaps, incomplete profiles, stale/missing vaccination facts, and quarantined sensitive payloads visible rather than silently normalizing them.
- Human approval gates are represented by `domain::policy::ReviewGate` in [`domain/src/policy.rs`](../../domain/src/policy.rs) and Rustdoc source view [`target/doc/src/domain/policy.rs.html`](../../target/doc/src/domain/policy.rs.html). Current gate vocabulary includes manager approval, medical document review, behavior review, customer-message approval, and refund/deposit exception.
- Generic workflow event/action vocabulary is represented by `domain::workflow::{Event, PolicyContext, AllowedAction, RecommendedAction, Result}` in [`domain/src/workflow.rs`](../../domain/src/workflow.rs) and Rustdoc source view [`target/doc/src/domain/workflow.rs.html`](../../target/doc/src/domain/workflow.rs.html).
- Agent packets/specs are represented by `app::agents::{AgentSpec, WorkflowAgent, AgentPromptPacket, baseline_agent_specs}` in [`app/src/agents.rs`](../../app/src/agents.rs) and Rustdoc source view [`target/doc/src/app/agents.rs.html`](../../target/doc/src/app/agents.rs.html). Module docs explicitly say packets are read/draft boundaries, not authority to mutate bookings, messages, schedules, deposits, incident records, or policy decisions.
- Normalized pet-resort entities are represented by `domain::entities::{Location, Customer, Pet, Reservation, ServiceKind, HardStop, CareNote, Message, approval::*}` in [`domain/src/entities.rs`](../../domain/src/entities.rs) and Rustdoc source view [`target/doc/src/domain/entities.rs.html`](../../target/doc/src/domain/entities.rs.html).
- Gingr read endpoints and mapping evidence live in [`integrations/gingr/src/endpoint/reservations.rs`](../../integrations/gingr/src/endpoint/reservations.rs), [`integrations/gingr/src/endpoint/owners_animals.rs`](../../integrations/gingr/src/endpoint/owners_animals.rs), [`integrations/gingr/src/endpoint/labor_ops.rs`](../../integrations/gingr/src/endpoint/labor_ops.rs), [`integrations/gingr/src/endpoint/commerce_retail.rs`](../../integrations/gingr/src/endpoint/commerce_retail.rs), [`integrations/gingr/src/mapping/pet.rs`](../../integrations/gingr/src/mapping/pet.rs), and [`integrations/gingr/src/mapping/customer.rs`](../../integrations/gingr/src/mapping/customer.rs). Generated Rustdoc exists under [`target/doc/gingr/`](../../target/doc/gingr/index.html), including reservations, owners/animals, labor ops, commerce/retail, and mapping pages.
- Labor/outcome persistence currently exists for manager daily brief and data-quality hygiene in `storage::operations` at [`storage/src/operations.rs`](../../storage/src/operations.rs) and Rustdoc source view [`target/doc/src/storage/operations.rs.html`](../../target/doc/src/storage/operations.rs.html). The storage record vocabulary includes `ManagerDailyBriefOutcomeRecord`, `DataQualityHygieneOutcomeRecord`, reporting groups, source refs, persona/action/outcome codes, and portfolio/brand records.

### Operator-facing explanation

- Source data means app-promoted facts with provenance and record refs. Raw provider payloads, model memory, unverified AI summaries, and read-model projections are not live authority by themselves.
- Agents may summarize, rank, draft, classify, and prepare review packets. They must not confirm bookings, mutate Gingr/PMS/POS records, send customer messages, change labor schedules, move payments/refunds/discounts, hide source ambiguity, or approve safety/medical/behavior exceptions without the named human gate.
- Labor-savings language should cite outcome records or say the outcome path is planned. It should not claim verified production savings unless an outcome record or audit explicitly backs it.

## Manager Daily Brief

Intended page path: `docs/workflows/operator/manager-daily-brief.md`

### Code-derived evidence to cite

- Workflow module: `app::manager_daily_brief` in [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs), Rustdoc source view [`target/doc/src/app/manager_daily_brief.rs.html`](../../target/doc/src/app/manager_daily_brief.rs.html).
- Key types: `Request`, `Packet`, `BriefAction`, `BriefActionKind`, `BriefActionPriority`, `SourceFact`, `SourceFactKind`, `ScopedCheckoutPacket`, `ScopedRetentionPacket`, `SafeAgentAction`, `BlockedAction`, `LaborImpactEstimate`, `OutcomeRecord`, `FeedbackOutcome`.
- Source facts: `SourceFact` carries `source::RecordRef` values and `Packet::all_actions_are_source_grounded()` checks source grounding.
- Related workflow packets: `ScopedCheckoutPacket` embeds `checkout_completion::Packet`; `ScopedRetentionPacket` embeds `crm_retention::Packet`.
- Domain evidence: `domain::daily_brief` in [`domain/src/daily_brief.rs`](../../domain/src/daily_brief.rs); `domain::analytics::service_demand::Fact` in [`domain/src/analytics.rs`](../../domain/src/analytics.rs) uses `source::RecordRef` and data-quality status for labor/demand facts.
- Agent spec: `baseline_agent_specs()` in [`app/src/agents.rs`](../../app/src/agents.rs) includes `manager-daily-brief`, allowed read/task tools, forbidden schedule/message/occupancy invention actions, and manager approval.
- Storage evidence: `storage::operations::ManagerDailyBriefOutcomeRecord`, `ManagerDailyBriefReportingGroup`, `ManagerDailyBriefOutcomeCode`, and `StoredSourceRecordRef` in [`storage/src/operations.rs`](../../storage/src/operations.rs).
- Tests: [`app/tests/manager_daily_brief_workflow_contracts.rs`](../../app/tests/manager_daily_brief_workflow_contracts.rs) and cross-workflow composition in [`app/tests/workflow_service_composition_contracts.rs`](../../app/tests/workflow_service_composition_contracts.rs).
- Design evidence: [`docs/design/manager-daily-brief-measurable-labor-loop.md`](manager-daily-brief-measurable-labor-loop.md) and [`docs/design/labor-cost-reduction-crosswalk.md`](labor-cost-reduction-crosswalk.md).

### Source data facts and boundaries

- Source facts can include service demand, labor/staffing risk, checkout packets, retention packets, and data-quality issues, but each action should cite `source::RecordRef` or a downstream packet that has provenance.
- Gingr/provider records, BI/labor/timeclock/POS/read-model facts, checkout packets, and retention packets are evidence inputs. The manager brief packet is a review surface, not the source of record.

### Agent packet/spec/tool-port evidence

- Agent spec permits read/task-prep style tools such as reservation/labor/care-note reads and task creation; it forbids invented occupancy, schedule changes, and unapproved customer sends.
- `SafeAgentAction` permits summarizing source evidence, ranking/recommending internal review work, drafting internal tasks/customer messages for review, estimating labor impact, and recording reviewed outcome feedback.

### Human approval gates and blocked live actions

- Gates: manager approval for management actions; customer-message approval when a retention/customer follow-up draft is present.
- Blocked: changing staff schedules, mutating provider/PMS records, sending customer messages, moving refunds/discounts/payments, and hiding source data-quality issues.

### Gaps / planned wording

- The manager brief app contract and storage outcome record are present, but final operator page, UI route, production data connector, and verified production labor savings are not proven here. Operator copy should say local/app contract and outcome-capture path, not shipped production automation.

## Booking Triage

Intended page path: `docs/workflows/operator/booking-triage.md`

### Code-derived evidence to cite

- Workflow module: `app::booking_triage` in [`app/src/booking_triage.rs`](../../app/src/booking_triage.rs), Rustdoc source view [`target/doc/src/app/booking_triage.rs.html`](../../target/doc/src/app/booking_triage.rs.html).
- Key types: typed `Request` states, `Reservation`, `PetProfile`, `PolicySnapshot`, `DeterministicResult`, `ReadinessBucket`, `ApprovalGate`, `FailureCode`, `SafeAgentAction`, `BlockedAction`, `StaffDecisionBoundary`, `AiRecommendation`, `ConfirmationDraft`, `AuditEventDraft`, `StaffEvaluationPacket`, `Service`.
- Rules/evidence: `rule::{Id, Decision, ReviewFinding, Evaluation}` covers deterministic pass/unknown/human approval/hard block decisions with `EvidenceRef` values.
- Domain evidence: `domain::policy::ReviewGate`, `domain::workflow`, normalized reservation entities in `domain/src/entities.rs`, and reservation domain policy in [`domain/src/reservation/mod.rs`](../../domain/src/reservation/mod.rs).
- Agent spec: `baseline_agent_specs()` includes `booking-triage`, availability/policy read and draft-message tools, and forbidden inventory/policy/deposit overreach.
- Tests: [`app/tests/booking_triage_mvp.rs`](../../app/tests/booking_triage_mvp.rs) and [`app/tests/workflow_service_composition_contracts.rs`](../../app/tests/workflow_service_composition_contracts.rs).
- Existing workflow docs: [`docs/workflows/booking-triage-agent.md`](../workflows/booking-triage-agent.md) and [`docs/workflows/booking-triage-parts/inputs.md`](../workflows/booking-triage-parts/inputs.md).

### Source data facts and boundaries

- Source inputs include reservation/request identity, customer/pet profile completeness, policy snapshots, vaccine/document evidence, deposits/payments, behavior/care/special handling, service/availability facts, and source evidence refs.
- Deterministic app rules decide readiness/gates; provider/PMS and payment systems remain the source of record for live booking, availability, and money movement.

### Agent packet/spec/tool-port evidence

- `SafeAgentAction` and `AiRecommendation` support explaining deterministic results, ranking missing-info/review work, and drafting review packets/customer-safe text.
- `StaffEvaluationPacket` carries deterministic result, optional AI recommendation, confirmation draft, and audit-event drafts for staff review.

### Human approval gates and blocked live actions

- Gates: manager/staff approval for confirmations, denials, waitlist/offer moves, behavior/care/vaccine exceptions, special care, payment/deposit questions, and customer-facing messages.
- Blocked: `ConfirmBooking`, `RejectRequest`, `MutateProviderRecord`, `SendCustomerMessage`, behavior/special-care exception approval, and payment movement unless authorized.

### Gaps / planned wording

- The module provides an app/service contract and tests, but live Gingr writeback, live availability reservation holds, payment moves, and final operator UI are not proven. Say triage packet/draft exists; do not imply autonomous booking execution.

## Data Quality Hygiene

Intended page path: `docs/workflows/operator/data-quality-hygiene.md`

### Code-derived evidence to cite

- Workflow module: `app::data_quality_hygiene` in [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs), Rustdoc source view [`target/doc/src/app/data_quality_hygiene.rs.html`](../../target/doc/src/app/data_quality_hygiene.rs.html).
- Key types: `Request`, `Packet`, `Candidate`, `CandidateKind`, `SourceFreshness`, `Sensitivity`, `Action`, `ActionKind`, `ActionPriority`, `SafeAgentAction`, `BlockedAction`, `DraftSubmission`, `DraftValidation`, `DraftRejectionReason`, `OutcomeRecord`, `FeedbackOutcome`, `LaborImpactEstimate`.
- Domain evidence: `domain::data_quality::{Issue, Kind, FieldPath, Severity, ResolutionStatus}` and `domain::source::{RecordRef, Provenance}`.
- Storage evidence: `storage::operations::DataQualityHygieneOutcomeRecord`, `DataQualityHygieneReportingGroup`, `DataQualityHygieneOutcomeCode`, `DataQualityResolutionStatusCode`, and `StoredSourceRecordRef` in [`storage/src/operations.rs`](../../storage/src/operations.rs).
- Tests: [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../app/tests/data_quality_hygiene_workflow_contracts.rs).
- Design evidence: [`docs/design/data-quality-hygiene-labor-loop.md`](data-quality-hygiene-labor-loop.md) and source evidence map docs under [`docs/safety/`](../safety/).

### Source data facts and boundaries

- Source inputs include data-quality issues, candidate source record refs, entity/field paths, source freshness, resolution status, sensitivity/redaction, workflow-blocking flags, and BI visibility.
- Data-quality issues are source-fact defects or ambiguity that must stay visible; the hygiene workflow is not a provider/PMS repair engine.

### Agent packet/spec/tool-port evidence

- Safe actions support summarizing source evidence, grouping/ranking cleanup work, drafting internal cleanup tasks, estimating labor impact, and recording reviewed outcomes.
- `Packet::validate_draft()` checks context packet id, required source refs, review gates, and blocked side effects before accepting a draft.

### Human approval gates and blocked live actions

- Gates: manager/front-desk/operations review for duplicates, missing evidence, stale facts, profile gaps, vaccination freshness, source conflicts, service-line mapping, and sensitive/quarantined payloads.
- Blocked: mutating provider/PMS records, hiding or auto-resolving source ambiguity, exposing sensitive/quarantined payloads, and treating stale/unknown context as accepted.

### Gaps / planned wording

- App and storage outcome contracts are present. Runtime/API shells, provider repair tooling, and final operator page are not proven by this map. Say the workflow can rank/draft/validate hygiene work; do not claim automated source cleanup.

## Checkout Completion

Intended page path: `docs/workflows/operator/checkout-completion.md`

### Code-derived evidence to cite

- Workflow module: `app::checkout_completion` in [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs), Rustdoc source view [`target/doc/src/app/checkout_completion.rs.html`](../../target/doc/src/app/checkout_completion.rs.html).
- Key types: `Request`, `Packet`, `CompletionStatus`, `StaffHandoff`, `SafeAgentAction`, `BlockedAction`, `AuditEventDraft`, `Workflow`.
- Source status evidence: `Request` carries `source::Provenance` and observed `source::reservation::Status`; `Packet` preserves provenance and audit-event drafts.
- Domain evidence: normalized reservation status in `domain::entities::reservation::Status`, source reservation status in `domain::source::reservation::Status`, `domain::policy::ReviewGate`, and checkout-related data-quality kinds in `domain::data_quality::Kind`.
- Tests: [`app/tests/checkout_completion_workflow_contracts.rs`](../../app/tests/checkout_completion_workflow_contracts.rs).
- Provider evidence: Gingr reservation endpoints in [`integrations/gingr/src/endpoint/reservations.rs`](../../integrations/gingr/src/endpoint/reservations.rs) and generated Rustdoc pages under [`target/doc/gingr/endpoint/reservations/index.html`](../../target/doc/gingr/endpoint/reservations/index.html).

### Source data facts and boundaries

- Source inputs include reservation id, provider checkout/PMS status and provenance, care summary, belongings status, departure-note review, staff handoff, and payment/care/source exceptions.
- Provider/PMS checkout state is source evidence; the app packet can suggest staff-verified status and draft audit events, but does not execute final status mutation.

### Agent packet/spec/tool-port evidence

- Safe actions support summarizing checkout evidence, drafting internal handoff tasks, drafting audit events, and preparing follow-up only when checkout evidence is safe.
- Checkout packets are reused by manager daily brief and CRM retention, making checkout completion a gate before retention outreach.

### Human approval gates and blocked live actions

- Gates: manager approval for incomplete handoff/source-not-checked-out states; customer-message approval for customer follow-up paths.
- Blocked: sending customer messages, mutating provider/PMS records, moving refunds/discounts/payments, and suggesting checked-out status when the source is not checked out or handoff review is still needed.

### Gaps / planned wording

- App workflow and tests exist. No live PMS mutation, payment/refund movement, or final operator UI is proven. Say staff-reviewed checkout packet, not autonomous checkout close.

## Grooming Rebooking / Retention

Intended page path: `docs/workflows/operator/grooming-rebooking-retention.md`

### Code-derived evidence to cite

- Workflow module: `app::crm_retention` in [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs), Rustdoc source view [`target/doc/src/app/crm_retention.rs.html`](../../target/doc/src/app/crm_retention.rs.html).
- Key types: `Request`, `Packet`, `RetentionOpportunity`, `OpportunityEvidence`, `SourceGroundedReasonCode`, `OpportunityKind`, `ContactPermission`, `ConsentStatus`, `FollowUpEligibility`, `StaffReviewPacket`, `SafeAgentAction`, `BlockedAction`, `OutcomeRecord`, `FollowUpOutcome`.
- Checkout prerequisite: `Request` embeds `checkout_completion::Packet`, and eligibility rejects non-staff-verified checkout.
- Grooming domain evidence: `domain::grooming::{history, rebooking, appointment}` in [`domain/src/grooming/mod.rs`](../../domain/src/grooming/mod.rs) and Rustdoc source view [`target/doc/src/domain/grooming/mod.rs.html`](../../target/doc/src/domain/grooming/mod.rs.html). This includes grooming service history, cadence recommendations, approval state, outreach planning, and review gates for schedule/rebooking work.
- Message/contact evidence: `domain::message::Channel` in [`domain/src/message.rs`](../../domain/src/message.rs) and customer/pet/reservation entities in `domain/src/entities.rs`.
- Agent spec: `baseline_agent_specs()` includes `grooming-rebooking`, grooming-history/availability reads, draft-message tools, and forbidden booking/discount/send actions.
- Tests: [`app/tests/crm_retention_workflow_contracts.rs`](../../app/tests/crm_retention_workflow_contracts.rs).
- Existing workflow docs: [`docs/workflows/crm-retention-agent.md`](../workflows/crm-retention-agent.md), [`docs/workflows/crm-retention-parts/rebooking-workflow.md`](../workflows/crm-retention-parts/rebooking-workflow.md), and [`docs/workflows/crm-retention-parts/inputs.md`](../workflows/crm-retention-parts/inputs.md).

### Source data facts and boundaries

- Source inputs include staff-verified checkout, customer/pet ids, completed visit/service history, grooming cadence, contact permission/consent, preferred/allowed channel, source-grounded reason codes, and suppression/contact risks.
- Completed stays, grooming history, and contact permission are evidence; live booking calendar, discounts, messages, and PMS/POS writes remain external authority.

### Agent packet/spec/tool-port evidence

- Safe actions support summarizing retention evidence, creating internal staff review tasks, drafting customer follow-up for review, and recording follow-up outcome evidence.
- Eligibility requires staff-verified checkout, at least one source-grounded opportunity, a permitted draft channel, and source evidence for contact permission.

### Human approval gates and blocked live actions

- Gates: customer-message approval for eligible follow-up drafts; manager approval for suppressed/ineligible/risky packets.
- Blocked: customer sends, provider/PMS mutations, refunds/discounts/payments, auto discounts, booking slot changes, and consent/DNC overreach.

### Gaps / planned wording

- App contract and grooming domain support exist; no live campaign send, offer/discount engine, calendar mutation, or measured conversion pipeline is proven. Outcome record exists in app but storage persistence for CRM retention outcomes is not present in `storage::operations` yet, so conversion/revenue claims should be future/planned unless separately backed.

## Daily Updates / Pawgress drafts

Intended page path: `docs/workflows/operator/daily-updates-pawgress-drafts.md`

### Code-derived evidence to cite

- Workflow module: `app::daily_update` in [`app/src/daily_update.rs`](../../app/src/daily_update.rs), Rustdoc source view [`target/doc/src/app/daily_update.rs.html`](../../target/doc/src/app/daily_update.rs.html).
- Key types: `MvpPreviewRequest`, `MvpPreview`, `daily_care_update::{Input, Output, Agent}`, `CustomerMessageDraft`, `ReviewDisposition`, `InternalFlag`, `InternalFlagCode`, `InternalFlagSeverity`, `RecommendedFlagAction`, `IncludedFact`, `OmittedFact`, `OmissionReason`, `SendStub`, `SendMode`.
- Agent packet evidence: `daily_care_update::Agent` implements `WorkflowAgent`, builds `AgentPromptPacket`, sets goal/policy/output schema, and validates output through the app boundary.
- Source event evidence: request/event validation requires `workflow::EventType::DailyNoteCreated` or `DailyUpdateNeeded`, staff care notes, and allowed actions `SummarizeCareNotes` or `DraftCustomerMessage`.
- Care/message evidence: `domain::entities::CareNote`, `domain::message::{BodyRef, Channel, Direction, Status}`, `domain::policy::ReviewGate`, and approval/audit entities in `domain/src/entities.rs`.
- Agent spec: `baseline_agent_specs()` includes `daily-care-update`, care-note read and draft-message tools, forbidden diagnose/hide/auto-send actions, and customer-message approval.
- Tests: [`app/tests/daily_care_update_mvp.rs`](../../app/tests/daily_care_update_mvp.rs).
- Existing workflow docs: [`docs/workflows/daily-care-update-agent.md`](../workflows/daily-care-update-agent.md), [`docs/workflows/daily-care-update-agent-parts/inputs.md`](../workflows/daily-care-update-agent-parts/inputs.md), [`docs/workflows/daily-care-update-agent-parts/output-schema.md`](../workflows/daily-care-update-agent-parts/output-schema.md), [`docs/workflows/daily-care-update-agent-parts/safety-rules.md`](../workflows/daily-care-update-agent-parts/safety-rules.md), [`docs/workflows/daily-care-update-agent-parts/tone-guide.md`](../workflows/daily-care-update-agent-parts/tone-guide.md), and [`docs/workflows/daily-care-update-agent-parts/example-transformations.md`](../workflows/daily-care-update-agent-parts/example-transformations.md).

### Source data facts and boundaries

- Source inputs include care notes, note ids, note visibility/kind/body, pet and owner display names, policy snapshot id, workflow event, allowed actions, redaction/tone/channel policy, and approval/audit records.
- Staff notes and approved media/evidence are the source basis; the agent draft is not authority for care-task completion, diagnosis, incident disposition, or customer send.

### Agent packet/spec/tool-port evidence

- `build_mvp_preview()` packages typed source notes into an agent packet and creates a draft-only `CustomerMessageDraft`, approval record, blocked send stub, internal flags, included facts, omitted facts, and audit log.
- `ReviewDisposition::allows_live_send()` returns false and `requires_human_review()` returns true; `SendStub::is_blocked_until_human_approval()` requires approval.

### Human approval gates and blocked live actions

- Gate: customer-message approval for every draft; manager/behavior/medical review when notes are sensitive or internal-only.
- Blocked: live customer sends, raw internal note exposure, diagnosis/medical advice, hiding concerning facts, media publication without approval, care-task completion, incident disposition, provider writes.

### Gaps / planned wording

- MVP preview and draft-only safety path exist. Media approval/source refs, live send integration, production template/channel policy, and outcome storage for saved writing time are not proven by this map. Say Pawgress drafts are review-required previews, not automatic sends.

## Regional Labor Exceptions / future portfolio view

Intended page path: `docs/workflows/operator/regional-labor-exceptions.md`

### Code-derived evidence to cite

- Current operations taxonomy: `domain::operations::{OperatingFunction, AiUseCase, pet_resort::*}` in [`domain/src/operations.rs`](../../domain/src/operations.rs) and Rustdoc source view [`target/doc/src/domain/operations.rs.html`](../../target/doc/src/domain/operations.rs.html). Relevant use cases include regional operations exception reporting and regional performance benchmarking.
- Manager and data-quality inputs: `domain::daily_brief`, `app::manager_daily_brief`, `storage::operations::ManagerDailyBriefOutcomeRecord`, `domain::data_quality::Issue`, and `storage::operations::DataQualityHygieneOutcomeRecord`.
- Analytics inputs: `domain::analytics::{stay, service_demand}` in [`domain/src/analytics.rs`](../../domain/src/analytics.rs), including service-demand facts with `source::RecordRef`, demand units, data-quality status, and data-quality issue attachments.
- Reputation/incident signals: `domain::reputation::Signal` in [`domain/src/reputation.rs`](../../domain/src/reputation.rs) and incident/care modules for possible regional exception context.
- Provider labor source: Gingr `endpoint::labor_ops::TimeclockReport` in [`integrations/gingr/src/endpoint/labor_ops.rs`](../../integrations/gingr/src/endpoint/labor_ops.rs) and generated Rustdoc [`target/doc/gingr/endpoint/labor_ops/index.html`](../../target/doc/gingr/endpoint/labor_ops/index.html).
- Portfolio storage: `storage::operations::{PetResortPortfolioRecord, PetResortBrandRecord, OperatorCode, PortfolioStructureCode, BusinessLineCode}` in [`storage/src/operations.rs`](../../storage/src/operations.rs).
- Design/audit evidence: [`docs/design/labor-cost-reduction-crosswalk.md`](labor-cost-reduction-crosswalk.md) and [`docs/audits/2026-06-18-labor-cost-platform-readiness.md`](../audits/2026-06-18-labor-cost-platform-readiness.md).

### Source data facts and boundaries

- Source inputs should be location/period/metric grouped read models, labor-risk and demand/staffing variance, utilization/capacity facts, incidents/reviews, data-quality caveats, manager brief outcomes, source refs, and peer/trend context.
- BI/timeclock/payroll/labor scheduling/provider facts remain their own source-of-record lanes. A regional view should cite read-model provenance and data-quality caveats instead of overwriting local systems.

### Agent packet/spec/tool-port evidence

- There is no dedicated `app::regional_labor_exceptions` module or baseline `regional-labor-exceptions` agent spec yet. The nearest evidence is manager daily brief/data-quality outcomes, domain operations taxonomy, analytics facts, and crosswalk/audit planned loop.
- Future agent work should be described as rank/summarize/draft-review-queue only, using the same packet/review/outcome pattern as manager daily brief and data-quality hygiene.

### Human approval gates and blocked live actions

- Gates: regional/human approval for GM follow-up, staffing plan changes, policy exceptions, pricing, customer communication, discipline/personnel action, and BI/source corrections.
- Blocked: schedule mutation, personnel/discipline decisions, provider/PMS writes, hiding BI/data-quality caveats, pricing changes, customer messages, or automated policy exceptions.

### Gaps / planned wording

- This page is future/planned. There is domain/storage/design evidence for the need and supporting source facts, but no dedicated app packet, service, tests, Rustdoc module, or persisted regional outcome record yet. Operator copy must say planned/future portfolio view and cite manager daily brief/data-quality contracts as feeders, not as a shipped regional automation product.

## Inspected source areas

- Parent inventory: [`docs/design/operator-workflow-page-inventory.md`](operator-workflow-page-inventory.md).
- Source/provenance/data-quality/domain boundaries: `domain/src/source.rs`, `domain/src/data_quality.rs`, `domain/src/policy.rs`, `domain/src/workflow.rs`, `domain/src/entities.rs`, `domain/src/analytics.rs`, `domain/src/operations.rs`, `domain/src/daily_brief.rs`, `domain/src/grooming/mod.rs`, `domain/src/message.rs`, `domain/src/reputation.rs`, `domain/src/lead.rs`, `domain/src/reservation/mod.rs`.
- App workflow contracts: `app/src/agents.rs`, `app/src/manager_daily_brief.rs`, `app/src/booking_triage.rs`, `app/src/data_quality_hygiene.rs`, `app/src/checkout_completion.rs`, `app/src/crm_retention.rs`, `app/src/daily_update.rs`, `app/src/tools.rs`.
- Storage/outcome contracts: `storage/src/operations.rs`.
- Gingr/provider evidence: `integrations/gingr/src/endpoint/reservations.rs`, `integrations/gingr/src/endpoint/owners_animals.rs`, `integrations/gingr/src/endpoint/labor_ops.rs`, `integrations/gingr/src/endpoint/commerce_retail.rs`, `integrations/gingr/src/mapping/pet.rs`, `integrations/gingr/src/mapping/customer.rs`, generated Rustdoc under `target/doc/gingr/`.
- Executable coverage: `app/tests/manager_daily_brief_workflow_contracts.rs`, `app/tests/booking_triage_mvp.rs`, `app/tests/data_quality_hygiene_workflow_contracts.rs`, `app/tests/checkout_completion_workflow_contracts.rs`, `app/tests/crm_retention_workflow_contracts.rs`, `app/tests/daily_care_update_mvp.rs`, `app/tests/workflow_service_composition_contracts.rs`.
- Design/workflow docs: `docs/design/labor-cost-reduction-crosswalk.md`, `docs/design/manager-daily-brief-measurable-labor-loop.md`, `docs/design/data-quality-hygiene-labor-loop.md`, `docs/audits/2026-06-18-labor-cost-platform-readiness.md`, `docs/workflows/booking-triage-agent.md`, `docs/workflows/daily-care-update-agent.md`, `docs/workflows/crm-retention-agent.md`, and relevant `docs/workflows/*-parts/` files.

## Unresolved evidence gaps to preserve in later operator pages

- `app` public item Rustdoc pages are not present in the current generated docs tree; app evidence is source file plus generated Rustdoc source view (`target/doc/src/app/*.rs.html`). Regenerate full crate docs before publishing if public item-page links are required.
- No dedicated regional labor exceptions app module, agent spec, tests, or outcome storage exists yet; regional page must be marked future/planned.
- CRM retention has an app-level outcome record, but no storage `CrmRetentionOutcomeRecord` was found in `storage::operations`; conversion/revenue outcome language should be planned unless a later storage/read-model artifact is added.
- Daily updates have MVP preview/draft-only safety tests, but no proven live channel integration, media approval workflow, or writing-time outcome storage in this map.
- Checkout completion suggests/reviews checkout state and drafts audit events; it does not execute provider/PMS checkout mutations or money movement.
- Booking triage produces deterministic packets/drafts; it does not hold inventory, confirm/deny bookings, mutate provider records, or move payment/deposit state.
- Data-quality hygiene validates and drafts cleanup work; it does not repair provider/source records automatically.
- Manager daily brief has source-grounded packet and outcome storage, but production/live NVA data connectors, UI, and verified production labor-savings metrics are outside the evidence inspected here.
