# Workflow page source and Rustdoc backing map

Purpose: cite the concrete source, generated Rustdoc, workflow specs, and known gaps that later operator-facing workflow pages may use. This is an evidence map only; it is not final page copy and it must not widen code-derived claims into shipped live automation.

Generated Rustdoc checked with:

- `cargo doc --no-deps --workspace`
- Local Rustdoc root: `target/doc/`

Rustdoc generation completed, but emitted pre-existing broken intra-doc link warnings in `domain/src/grooming/mod.rs`. Those warnings do not invalidate the concrete pages below, but grooming workflow copy should avoid citing the broken bare intra-doc links until they are fixed.

## Shared source-of-record and authority rules

Code-derived evidence:

- `domain/src/source.rs` / `target/doc/domain/source/index.html`
  - `source::System` (`target/doc/domain/source/enum.System.html`) names upstream systems including Gingr.
  - `source::RecordRef` (`target/doc/domain/source/struct.RecordRef.html`) is the stable pointer to an upstream record and owning system.
  - `source::Provenance` (`target/doc/domain/source/struct.Provenance.html`) ties normalized data back to provider records.
  - `source::ObservedStatus` (`target/doc/domain/source/struct.ObservedStatus.html`) preserves provider status text before normalization.
- `domain/src/policy.rs` / `target/doc/domain/policy/index.html`
  - `policy::ReviewGate` (`target/doc/domain/policy/enum.ReviewGate.html`) is the shared human-review gate vocabulary.
- `domain/src/workflow.rs` / `target/doc/domain/workflow/index.html`
  - `workflow::Event`, `PolicyContext`, `RecommendedAction`, and `Status` carry workflow evidence, allowed actions, review gates, and lifecycle state.
- `app/src/agents.rs` / `target/doc/app/agents/index.html`
  - `AgentSpec`, `WorkflowAgent`, and `AgentPromptPacket` are the shared app-layer agent packet/spec surfaces.
- `app/src/tools.rs` and `app/src/tools/README.md`
  - Tool ports are app-owned candidates/interfaces; they are not proof of autonomous live side effects.
- Gingr provider/read-model surfaces:
  - `integrations/gingr/src/endpoint/reservations.rs` / `target/doc/gingr/endpoint/reservations/index.html` (`Reservations`, `BackOfHouse`, `GetServicesByType`).
  - `integrations/gingr/src/endpoint/labor_ops.rs` / `target/doc/gingr/endpoint/labor_ops/index.html` (`TimeclockReport`).
  - `integrations/gingr/src/mapping/mod.rs` and `integrations/gingr/src/mapping/README.md` for adapter mapping boundaries.
  - `docs/integrations/gingr/source-inventory.md`, `docs/integrations/gingr/adapter-boundary-and-labor-source-expansion.md`, and `docs/integrations/gingr/bi-read-model-contract.md` for source-of-record explanations.

Operator-facing explanation boundary:

- Source facts must be normalized app/domain packets with `RecordRef`/`Provenance`, not raw provider payloads, model memory, or prior AI summaries.
- Agents may summarize, rank, draft, validate, and prepare review packets.
- Humans or approved systems of record retain authority for customer sends, booking confirmations, schedule changes, payment/refund/discount movement, provider/PMS writes, policy exceptions, source hiding, and sensitive data release.

## Manager Daily Brief

Source/Rustdoc evidence:

- `app/src/manager_daily_brief.rs` / `target/doc/app/manager_daily_brief/index.html`
  - `Request` (`struct.Request.html`): input contract for source-grounded records.
  - `Packet` (`struct.Packet.html`): reviewable packet for staff/agents after deterministic gates.
  - `BriefAction` and `BriefActionKind` (`struct.BriefAction.html`, `enum.BriefActionKind.html`): ranked manager actions.
  - `SafeAgentAction` (`enum.SafeAgentAction.html`): allowed summarization/ranking/task-draft work.
  - `BlockedAction` (`enum.BlockedAction.html`) and `requested_side_effect_rejection_reason` (`fn.requested_side_effect_rejection_reason.html`): side-effect rejection evidence.
  - `LaborImpactEstimate` (`struct.LaborImpactEstimate.html`) and `OutcomeRecord` (`struct.OutcomeRecord.html`): labor estimate/outcome capture.
  - `Workflow` (`struct.Workflow.html`): app workflow assembly.
- `domain/src/daily_brief.rs` / `target/doc/domain/daily_brief/index.html`
  - `ResortOperatingDay`, `OccupancySnapshot`, `LaborRisk`, `CustomerFollowUp`, `RevenueOpportunity`, and `Action` support the daily operating facts and draft action model.
- `domain/src/analytics.rs` / `target/doc/domain/analytics/index.html`
  - `analytics::service_demand::Fact` is an evidence candidate for demand facts.
- `storage/src/operations.rs` / `target/doc/storage/operations/index.html`
  - `ManagerDailyBriefOutcomeRecord` (`struct.ManagerDailyBriefOutcomeRecord.html`) stores before/after labor-minute and source-reference evidence.
- Tests: `app/tests/manager_daily_brief_workflow_contracts.rs`.
- Design docs: `docs/design/manager-daily-brief-measurable-labor-loop.md`, `docs/design/labor-cost-reduction-crosswalk.md`, `docs/ops/manager-daily-brief-local-smoke.md`, `docs/ops/hermes-manager-daily-brief-bridge.md`.

Source data facts and source-of-record boundaries:

- Needs location/operating day, demand/occupancy/labor facts, scoped checkout and retention packets, data-quality caveats, source refs, and prior outcomes.
- Gingr/provider records can supply reservations/back-of-house/timeclock source evidence through the Gingr endpoint surfaces, but the page should cite normalized domain/app packets as the truth the agent receives.

Agent packet/spec/tool-port evidence:

- `manager_daily_brief::Packet` is the reviewable context packet.
- `BriefAction` ranks/summarizes next work.
- Hermes bridge docs/scripts may draft operational brief payloads, but source authority remains in app packets and outcome records.

Human approval gates and blocked live actions:

- Manager approval is required for staffing, schedule, policy, and data-quality actions.
- Customer-message approval is required for retention follow-up drafts.
- Blocked actions include schedule/staffing mutation, provider/PMS mutation, customer sends, payment/refund/discount movement, and hiding source data-quality issues.

Gaps/planned wording:

- The page may claim local contract and outcome-record support, not production-verified NVA labor savings or live provider write access.
- Regional aggregation from daily brief outcomes belongs to the regional/future page unless a dedicated regional read model lands.

## Booking Triage

Source/Rustdoc evidence:

- `app/src/booking_triage.rs` / `target/doc/app/booking_triage/index.html`
  - `Request`/`Service` (`struct.Service.html`) shape the app service boundary.
  - `DeterministicResult` (`struct.DeterministicResult.html`) keeps readiness decisions in deterministic policy evidence.
  - `StaffEvaluationPacket` (`struct.StaffEvaluationPacket.html`) is the staff review packet.
  - `ConfirmationDraft` (`struct.ConfirmationDraft.html`) and `AuditEventDraft` (`enum.AuditEventDraft.html`) are draft artifacts, not live confirmation.
  - `SafeAgentAction` (`enum.SafeAgentAction.html`) and `BlockedAction` (`enum.BlockedAction.html`) define allowed vs forbidden actions.
- Shared gates: `domain/src/policy.rs` `ReviewGate`; `domain/src/workflow.rs` `PolicyContext` and `RecommendedAction`.
- Source/provider support: `integrations/gingr/src/endpoint/reservations.rs` (`Reservations`, `BackOfHouse`, `GetServicesByType`) and mapping docs.
- Tests: `app/tests/booking_triage_mvp.rs`, `app/tests/workflow_service_composition_contracts.rs`.
- Workflow spec: `docs/workflows/booking-triage-agent.md`.

Source data facts and source-of-record boundaries:

- Needs booking request, customer/pet profile, reservation/provider state, service catalog, availability/capacity, vaccine/document evidence, payment/deposit state, behavior/care notes, staffing/policy snapshots, and source refs.
- Provider statuses and services must be normalized into app/domain facts before agent use; raw Gingr payloads do not become customer-safe truth by themselves.

Agent packet/spec/tool-port evidence:

- Agent work is explanation/ranking/drafting around deterministic results and staff packets.
- `ConfirmationDraft` is safe only as a reviewed draft; it is not a booking confirmation.

Human approval gates and blocked live actions:

- Staff/manager approval is required for offers, confirmations, waitlist movement, denials, behavior/care/vaccine exceptions, payment actions, and customer-facing messages.
- Blocked actions include confirming/mutating booking/provider records, sending messages, bypassing review gates, and moving money.

Gaps/planned wording:

- Existing evidence supports an MVP contract and tests. Do not imply live PMS booking mutation, autonomous waitlist moves, or production vaccine/payment authority.

## Data Quality Hygiene

Source/Rustdoc evidence:

- `app/src/data_quality_hygiene.rs` / `target/doc/app/data_quality_hygiene/index.html`
  - `Request` (`struct.Request.html`) and `Packet` (`struct.Packet.html`) build the source-grounded review packet.
  - `Candidate` (`struct.Candidate.html`) and `Action` (`struct.Action.html`) represent issue candidates and proposed cleanup work.
  - `DraftSubmission` and `DraftValidation` (`struct.DraftSubmission.html`, `struct.DraftValidation.html`) validate drafts and reject unsafe requests.
  - `SafeAgentAction`/`BlockedAction` (`enum.SafeAgentAction.html`, `enum.BlockedAction.html`) constrain agent work.
  - `OutcomeRecord` (`struct.OutcomeRecord.html`) captures feedback and labor deltas.
- `domain/src/data_quality.rs` / `target/doc/domain/data_quality/index.html`
  - `Issue`, `Kind`, `FieldPath`, `Severity`, and `ResolutionStatus` define source-evidence-backed hygiene problems.
- `domain/src/source.rs` (`RecordRef`, `Provenance`) anchors issue evidence.
- `storage/src/operations.rs` / `target/doc/storage/operations/index.html`
  - `DataQualityHygieneOutcomeRecord` (`struct.DataQualityHygieneOutcomeRecord.html`) is durable outcome evidence.
- Tests: `app/tests/data_quality_hygiene_workflow_contracts.rs`.
- Design/ops docs: `docs/design/data-quality-hygiene-labor-loop.md`, `docs/ops/data-quality-hygiene-local-smoke.md`, `docs/safety/source-evidence-map.md`.

Source data facts and source-of-record boundaries:

- Needs source refs/provenance, location/operating day, entity/field path, severity, freshness, workflow-blocking status, BI visibility, and sensitivity/redaction metadata.
- Source ambiguity is part of the packet; the page must not imply the agent repairs Gingr/provider records or hides conflicts.

Agent packet/spec/tool-port evidence:

- Agent can group/rank candidates, summarize source evidence, draft internal cleanup tasks, and estimate reconciliation minutes avoided.
- Draft validation preserves ambiguity and blocked actions.

Human approval gates and blocked live actions:

- Manager/front-desk-lead review is required for ambiguity, duplicates, stale vaccines, service-line mapping, payment conflicts, and sensitive/quarantined payloads.
- Blocked actions include provider/PMS writes, source hiding, destructive merge/delete, sensitive data exposure, and policy/payment changes.

Gaps/planned wording:

- App/storage outcome contracts exist. Any API/runtime shell beyond local smoke/specs should be described as planned unless a concrete endpoint is added.

## Checkout Completion

Source/Rustdoc evidence:

- `app/src/checkout_completion.rs` / `target/doc/app/checkout_completion/index.html`
  - `Request` (`struct.Request.html`) and `Packet` (`struct.Packet.html`) are source-grounded checkout workflow contracts.
  - `CompletionStatus` (`enum.CompletionStatus.html`) classifies checkout state.
  - `StaffHandoff` (`struct.StaffHandoff.html`) makes unresolved handoff/payment/care notes explicit.
  - `SafeAgentAction` and `BlockedAction` (`enum.SafeAgentAction.html`, `enum.BlockedAction.html`) define safe prep vs forbidden live actions.
  - `AuditEventDraft` (`enum.AuditEventDraft.html`) is a draft audit artifact.
  - `Workflow` (`struct.Workflow.html`) assembles the packet.
- Shared source/gates: `domain/src/source.rs`, `domain/src/workflow.rs`, `domain/src/policy.rs`.
- Provider source evidence: `integrations/gingr/src/endpoint/reservations.rs` and reservation mapping/read-model docs.
- Tests: `app/tests/checkout_completion_workflow_contracts.rs`.
- Crosswalk: checkout bottleneck rows in `docs/design/labor-cost-reduction-crosswalk.md`.

Source data facts and source-of-record boundaries:

- Needs reservation id, source checkout/PMS status plus provenance, staff handoff, care summary, belongings status, departure note review, payment/care/source exceptions, and policy gates.
- Source checkout status must be cited as provider evidence, not overwritten by an agent-generated summary.

Agent packet/spec/tool-port evidence:

- Agent/app may suggest staff verification readiness, draft internal handoff tasks, create audit-event drafts, and route incomplete handoffs to review.
- Retention follow-up should only be prepared after safe checkout evidence exists.

Human approval gates and blocked live actions:

- Staff/manager approval is required for unresolved handoffs, departure note concerns, payment/refund/discount moves, customer messages, provider/PMS mutation, and final status execution.

Gaps/planned wording:

- There is no separate durable checkout outcome record named in storage yet. Use app contract/test evidence and mark durable outcome persistence as planned unless later code adds it.

## Grooming Rebooking / Retention

Source/Rustdoc evidence:

- `app/src/crm_retention.rs` / `target/doc/app/crm_retention/index.html`
  - `Request` (`struct.Request.html`) and `Packet` (`struct.Packet.html`) build the retention review workflow.
  - `RetentionOpportunity` (`struct.RetentionOpportunity.html`), `OpportunityKind` (`enum.OpportunityKind.html`), and `SourceGroundedReasonCode` (`enum.SourceGroundedReasonCode.html`) classify opportunities such as `GroomingRebook`.
  - `ContactPermission` (`struct.ContactPermission.html`) and `FollowUpEligibility` (`enum.FollowUpEligibility.html`) enforce contact safety.
  - `StaffReviewPacket` (`struct.StaffReviewPacket.html`) packages evidence and draft limits for staff.
  - `SafeAgentAction` and `BlockedAction` (`enum.SafeAgentAction.html`, `enum.BlockedAction.html`) prevent sends/provider/payment mutation.
  - `OutcomeRecord` (`struct.OutcomeRecord.html`) captures disposition/outcome.
- `domain/src/grooming/mod.rs` / `target/doc/domain/grooming/index.html`
  - `Contract`, `DurationEstimate`, `ReviewRequirement`, service/history/rebooking modules support grooming context and review gates.
- `domain/src/message.rs` (`Direction`, `Channel`, `Status`, `BodyRef`) supports message state vocabulary.
- `domain/src/lead.rs` and `domain/src/reputation.rs` are optional supporting signals for follow-up/reputation context, not booking authority.
- Tests: `app/tests/crm_retention_workflow_contracts.rs`.
- Workflow specs: `docs/workflows/crm-retention-agent.md`, `docs/workflows/crm-retention-parts/rebooking-workflow.md`.

Source data facts and source-of-record boundaries:

- Needs completed checkout/stay evidence, customer/pet ids, grooming history/cadence, service history, package/membership or slot-utilization signals, contact permission/consent, suppression flags, preferred channel, and source refs.
- Grooming provider/calendar state must remain source-of-record evidence unless promoted into domain/app packets.

Agent packet/spec/tool-port evidence:

- Agent can classify eligible/suppressed opportunities, prioritize staff review packets, draft personalized follow-up for review, and record staff disposition.

Human approval gates and blocked live actions:

- Human approval is required for customer sends, offers, discounts, refunds, payment movement, booking/provider/calendar mutation, DNC/consent handling, and complaint/incident-sensitive outreach.

Gaps/planned wording:

- Rustdoc generated but has broken intra-doc links in grooming module prose. Cite concrete Rustdoc pages/files, not the broken bare links, until fixed.
- Do not imply automatic grooming appointment creation or autonomous customer outreach.

## Daily Updates / Pawgress Drafts

Source/Rustdoc evidence:

- `app/src/daily_update.rs` / `target/doc/app/daily_update/index.html`
  - `MvpPreviewRequest` (`struct.MvpPreviewRequest.html`) and `MvpPreview` (`struct.MvpPreview.html`) define the preview packet.
  - `CustomerMessageDraft` (`struct.CustomerMessageDraft.html`) is the customer-safe draft artifact.
  - `ReviewDisposition` (`enum.ReviewDisposition.html`) records review outcome.
  - `InternalFlag` (`struct.InternalFlag.html`), `IncludedFact` (`struct.IncludedFact.html`), and `OmittedFact` (`struct.OmittedFact.html`) preserve source/safety decisions.
  - `SendStub` (`struct.SendStub.html`) is a stub, not an autonomous send.
  - `build_mvp_preview` (`fn.build_mvp_preview.html`) builds the local preview.
- Nested module: `app::daily_update::daily_care_update` in `app/src/daily_update.rs` and `target/doc/app/daily_update/daily_care_update/index.html` (`Input`, `Output`, `Agent`).
- `domain/src/workflow.rs` `Event` and `domain/src/message.rs` message state types support source event/message vocabulary.
- Tests: `app/tests/daily_care_update_mvp.rs`.
- Workflow specs: `docs/workflows/daily-care-update-agent.md`, `docs/workflows/daily-care-update-agent-parts/output-schema.md`, `safety-rules.md`, `staff-note-capture.md`, `example-transformations.md`.

Source data facts and source-of-record boundaries:

- Needs staff care notes, task evidence, reservation/pet/update window, approved media refs, note provenance/version, visibility/review state, redaction/sensitivity flags, customer-message policy, and tone/brand rules.
- Staff notes/media approval state are the evidence; the model cannot invent care facts or decide incident/medical outcomes.

Agent packet/spec/tool-port evidence:

- Agent can transform source-backed notes into customer-safe draft copy, list included/omitted facts, flag missing/conflicting/sensitive evidence, and draft internal missing-evidence tasks.

Human approval gates and blocked live actions:

- Staff/customer-message approval is required before send.
- Manager/medical/behavior/media/privacy review is required for concerns.
- Blocked actions include autonomous sends, media publication, care-task completion, medical advice, incident disposition, and provider writes.

Gaps/planned wording:

- Current evidence supports MVP preview/draft contracts and tests. Do not imply production Pawgress delivery, media publishing, or live messaging integration.

## Regional Labor Exceptions / future portfolio view

Source/Rustdoc evidence:

- `domain/src/operations.rs` / `target/doc/domain/operations/index.html`
  - `AiUseCase::RegionalOpsExceptionReporting` and `AiUseCase::RegionalPerformanceBenchmarking` in `enum.AiUseCase.html` are the strongest code-derived proof that regional exception workflows are planned concepts.
  - `OperatingFunction` (`enum.OperatingFunction.html`), `DataQualityIssue` (`enum.DataQualityIssue.html`), and `OptimizationOpportunity` (`enum.OptimizationOpportunity.html`) support portfolio operations vocabulary.
- `domain/src/daily_brief.rs` supports operating-day facts that can feed aggregated manager/regional reporting.
- `domain/src/reputation.rs` supports review/reputation signals as optional regional caveats.
- `storage/src/operations.rs` / `target/doc/storage/operations/index.html`
  - `PetResortPortfolioRecord` (`struct.PetResortPortfolioRecord.html`) and existing manager/data-quality outcome records can seed future portfolio views.
- Provider/read-model source evidence: `integrations/gingr/src/endpoint/labor_ops.rs` `TimeclockReport`, `integrations/gingr/src/endpoint/reservations.rs` `BackOfHouse`, and BI read-model docs.
- Design/audit docs: `docs/design/labor-cost-reduction-crosswalk.md`, `docs/audits/2026-06-18-labor-cost-platform-readiness.md`.

Source data facts and source-of-record boundaries:

- Needs portfolio/regional read models grouped by location/period/metric, labor risk, demand/staffing variance, utilization/capacity, incidents/reviews, data-quality caveats, manager daily brief outcomes, source refs, and peer/trend context.
- Regional page should cite derived read models and outcome records, not raw BI screenshots or AI-generated summaries.

Agent packet/spec/tool-port evidence:

- Existing app evidence is indirect: manager daily brief outcome records, data-quality hygiene outcomes, and operations enums support a future aggregated queue.
- No dedicated `app::regional_labor_exceptions` module or storage projection was found.

Human approval gates and blocked live actions:

- Regional/human approval is required for GM follow-up, staffing plan changes, discipline/personnel action, policy exceptions, pricing, customer communications, provider/PMS writes, schedule mutation, and BI hiding.

Gaps/planned wording:

- This page must be explicitly labeled planned/future. There is no dedicated regional exception app module, packet, API endpoint, or durable regional exception outcome record yet.
- It may say the domain vocabulary and existing outcome records point toward the portfolio view, not that a shipped regional automation exists.

## Evidence gaps to carry forward

- Regional labor exceptions are planned/future: no dedicated app module, packet, API endpoint, or storage outcome record was found.
- Checkout completion has an app workflow and tests, but no dedicated durable checkout outcome projection was found in `storage/src/operations.rs`.
- Daily Updates/Pawgress has MVP preview/draft contracts and tests, but no proof of production send/media publication integration.
- Booking triage has MVP/app service evidence, but no proof of autonomous live booking/provider mutation, payment capture, vaccine approval, or waitlist movement.
- Grooming rebooking/retention has retention packet/outcome support and grooming domain vocabulary, but no proof of autonomous grooming appointment creation or customer outreach.
- `cargo doc --no-deps --workspace` currently emits broken intra-doc link warnings in `domain/src/grooming/mod.rs`; later docs should either fix those links or cite concrete Rustdoc pages/files directly.

## Source areas inspected

- Parent inventory: `docs/design/operator-workflow-page-inventory.md`.
- Navigation/source docs: `README.md`, `app/README.md`, `domain/README.md`, `storage/README.md`, `integrations/gingr/README.md`.
- Labor/design/audit docs: `docs/design/labor-cost-reduction-crosswalk.md`, `docs/design/manager-daily-brief-measurable-labor-loop.md`, `docs/design/data-quality-hygiene-labor-loop.md`, `docs/audits/2026-06-18-labor-cost-platform-readiness.md`.
- Workflow specs: `docs/workflows/booking-triage-agent.md`, `docs/workflows/daily-care-update-agent.md`, `docs/workflows/daily-care-update-agent-parts/*`, `docs/workflows/crm-retention-agent.md`, `docs/workflows/crm-retention-parts/rebooking-workflow.md`.
- App modules: `app/src/manager_daily_brief.rs`, `booking_triage.rs`, `data_quality_hygiene.rs`, `checkout_completion.rs`, `crm_retention.rs`, `daily_update.rs`, `agents.rs`, `tools.rs`.
- Domain modules: `domain/src/source.rs`, `data_quality.rs`, `daily_brief.rs`, `operations.rs`, `grooming/mod.rs`, `message.rs`, `workflow.rs`, `policy.rs`, `reputation.rs`, `lead.rs`.
- Storage/integration modules: `storage/src/operations.rs`, `integrations/gingr/src/endpoint/labor_ops.rs`, `integrations/gingr/src/endpoint/reservations.rs`, `integrations/gingr/src/mapping/mod.rs`.
- Tests: required app workflow tests under `app/tests/`.
