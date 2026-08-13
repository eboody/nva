# Data Quality Hygiene

Data Quality Hygiene saves managers, front-desk leads, and operations analysts from re-solving the same source-data problems every shift. It turns missing evidence, duplicate or contradictory records, stale facts, inconsistent service names, and sensitive-source exceptions into ranked internal cleanup work with clear source proof and human review.

Example: if one pet profile has a stale vaccine date, a duplicate customer/pet candidate, an unmapped service name, and checkout evidence that does not match the current reservation state, the workflow can group those findings, draft internal cleanup tasks, and preserve the ambiguity for staff instead of letting every downstream workflow trip over the same bad fact.

Status: supported app/API/storage contract and local smoke evidence. The workflow ranks, drafts, validates, and records reviewed cleanup outcomes; it does not automatically repair Gingr/PMS records, merge profiles, approve vaccines, change payments, or hide source ambiguity.

Navigation: start with the [operator workflow index](README.md). Entity-first backlinks: [source/provenance/data quality](../../design/source-provenance-data-quality-atlas.md), [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [runtime/storage/API surfaces](../../design/entity-atlas-runtime-storage-api-surfaces.md), and [outcome/labor atlas](../../design/entity-atlas-outcomes-operations-money.md).

## 1. What problem does this solve?

Bad source facts create repeated resort rework and make automation unsafe. Missing required fields, duplicate records, stale provider status, incomplete pet/customer profiles, inconsistent service names, payment/source conflicts, checkout gaps, ambiguous owner-pet relationships, missing vaccination records, and quarantined sensitive payloads should be surfaced as reviewable cleanup work, not silently normalized away.

The workflow keeps the suspect fact, affected entity family, issue category, field path, provenance, source record reference, severity, freshness, sensitivity/redaction boundary, and workflow-blocking status visible. That gives staff a precise queue: what looks wrong, which source record produced it, why it blocks or degrades another workflow, who must review the cleanup, and which raw payloads remain hidden from agent-visible drafts.

## 2. Whose time does it save?

- Front-desk leads and agents who otherwise re-check the same pet, client, booking, vaccination, service, checkout, and payment records across screens.
- General managers and assistant GMs who need a trustworthy cleanup queue before acting on manager briefs, booking triage, checkout completion, or retention recommendations.
- Operations analysts and regional operators who need recurring source-quality patterns separated from true labor, demand, or revenue exceptions.

## 3. Source data and featured entities

It needs [data-quality issue](../../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue) candidates, [source refs](../../glossary-architecture-terms.md#source-ref-domainsourcerecordref), [provenance](../../glossary-architecture-terms.md#provenance-domainsourceprovenance), location, operating day, affected entity family, issue category, entity/field path, severity, resolution status, source freshness, sensitivity/redaction metadata, workflow-blocking state, BI visibility, issue refs, action refs, reviewer role, cleanup-action family, and labor-minute estimates.

| Entity or source fact | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| Data-quality issue and field path | Names the bad or ambiguous fact: missing required field, duplicate record, unmapped service type, payment conflict, checkout gap, incomplete pet profile, missing vaccine, quarantined payload, and similar defects. | Domain data-quality contract owns the issue vocabulary; provider/source evidence proves where it came from; humans decide the repair. | [`domain/src/data_quality.rs`](../../../domain/src/data_quality.rs) (`domain::data_quality::{Issue, Kind, FieldPath, Severity, ResolutionStatus}`); [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../../app/tests/data_quality_hygiene_workflow_contracts.rs). |
| Source ref and provenance | Keeps cleanup work tied to a concrete provider/read-model/import record instead of model memory or a vague summary. | Source system evidence and `domain::source` are authority for lineage; the app packet only carries the evidence forward. | [`domain/src/source.rs`](../../../domain/src/source.rs) (`domain::source::{RecordRef, Provenance, System}`); [workflow backing map](../../design/workflow-source-rustdoc-backing-map.md#shared-source-of-record-and-safety-boundaries). |
| Hygiene candidate/action | Groups issue evidence into staff-reviewable work and assigns affected entity, issue category, action kind, cleanup-action family, priority, owner persona, reviewer role, review gates, redaction boundary, and labor estimate. | `app::data_quality_hygiene` derives the review packet from source-grounded candidates; staff owns the cleanup decision. | [`app/src/data_quality_hygiene.rs`](../../../app/src/data_quality_hygiene.rs) (`Candidate`, `AffectedEntity`, `IssueCategory`, `RedactionPolicy`, `Action`, `ActionKind`, `CleanupAction`, `ActionPriority`, `HygienePersona`, `ReviewerRole`, `RemovedManualWork`, `LaborImpactEstimate`). |
| Pet, customer, booking, service, vaccination, checkout, and region/policy context | Explains what the bad source fact affects: profile completion, owner-pet relationship, booking readiness, service mapping, vaccine review, checkout completion, manager brief quality, or regional rollup accuracy. | Provider/domain records are evidence; policy and human review decide whether the record is acceptable. | [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md); [workflow-to-entity navigation map](../../design/workflow-to-entity-navigation-map.md#workflow---entity-matrix); [`domain/src/entities.rs`](../../../domain/src/entities.rs), [`domain/src/policy.rs`](../../../domain/src/policy.rs), [`domain/src/operations.rs`](../../../domain/src/operations.rs). |
| Outcome record and labor minutes | Records reviewed disposition, issue refs, source refs, before/actual minutes, owner persona, resolution status after review, and estimated/actual minutes saved. | App outcome and storage projection are evidence of reviewed cleanup labor; they are not proof that the provider source was repaired. | [`app/src/data_quality_hygiene.rs`](../../../app/src/data_quality_hygiene.rs) (`OutcomeRecord`, `FeedbackOutcome`); [`storage/src/operations.rs`](../../../storage/src/operations.rs) (`DataQualityHygieneOutcomeRecord`); [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../../storage/tests/data_quality_hygiene_outcome_storage.rs). |

Related entities to mention without making them the page's center:

- Manager Daily Brief: can include unresolved/high-impact data-quality issues as source-risk context, but the brief does not repair the source record.
- Booking Triage: depends on clean customer, pet, reservation, service, vaccine, deposit/payment, and policy facts; hygiene work can prepare internal cleanup but cannot confirm/waitlist/deny bookings.
- Checkout Completion: checkout evidence gaps or unclosed reservation facts can become hygiene actions; checkout/PMS state remains provider/staff authority.
- Grooming retention and Daily Updates: rely on trustworthy pet/customer/contact/care facts; hygiene can flag missing/stale inputs before customer-facing drafts.
- Regional Labor Exceptions: can consume aggregate data-quality outcomes as a future portfolio signal, but there is no dedicated regional exception app module or durable regional outcome record yet.

## Entity-first navigation

Start with the [workflow-to-entity navigation map](../../design/workflow-to-entity-navigation-map.md#workflow---entity-matrix), then read [Source, provenance, and data-quality atlas](../../design/source-provenance-data-quality-atlas.md) for data-quality issues, field paths, source refs, and provenance; [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md) for the customer/pet/reservation/service facts being cleaned up; [Runtime/storage/API surfaces](../../design/entity-atlas-runtime-storage-api-surfaces.md) for storage/API proof; and [Outcome/labor atlas](../../design/entity-atlas-outcomes-operations-money.md) for labor-minute and outcome evidence.

Entity flow: source refs/provenance + entity field path + issue severity/sensitivity -> hygiene candidate/action packet -> internal cleanup draft and draft validation -> human cleanup/review disposition -> `DataQualityHygieneOutcomeRecord`. The entity is the suspect fact and its source proof; the workflow only ranks and drafts review work around it.

## 4. Featured contracts

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | `app::data_quality_hygiene::{Request, Packet, Candidate, Action, DraftSubmission, DraftValidation, SafeAgentAction, BlockedAction, OutcomeRecord, Workflow}` | Source-grounded packet building, ranked hygiene actions, internal cleanup drafts, draft validation, labor estimates, and reviewed outcome records. | Live provider/PMS repair, customer sends, payment/refund/discount movement, schedule changes, source hiding, destructive merges/deletes, or sensitive payload release. |
| `domain` | `domain::data_quality::{Issue, Kind, FieldPath, Severity, ResolutionStatus}` and `domain::source::{RecordRef, Provenance}` | Names the defect, severity, source field path, resolution lifecycle, and source lineage. | Provider-specific mutation, deciding that an ambiguous fact is acceptable, or approving policy/safety exceptions. |
| `domain` / policy | `domain::policy::ReviewGate` plus pet/customer/reservation/service/vaccine/operation domain facts | Names manager, medical/document, behavior, message, and refund/deposit review gates that staff must satisfy when a cleanup touches policy-sensitive facts. | Replacing the named human/system-of-record approval. |
| `storage` | `storage::operations::DataQualityHygieneOutcomeRecord`, `DataQualityHygieneOutcomeSummary`, outcome/persona/action/resolution codes, stored source refs, labor-minute scalar | Durable evidence that reviewed cleanup actions were completed/deferred/suppressed/source-wrong/not-actionable, how many minutes were saved or spent, and which source/issue refs support a location/day/correlation rollup. | Proof that the underlying provider record was changed or that production labor savings were independently audited. |
| `apps/api` | `/agent/context/data-quality-hygiene`, `/agent/drafts/data-quality-hygiene`, `/data-quality-hygiene/actions/{action_id}/outcome`, `/data-quality-hygiene/outcomes/summary` contract tests | Local/API shell evidence for context, draft rejection, outcome capture, and reviewed outcome summary reporting without live side effects. | Production connector behavior, live writeback, or unrestricted API authorization claims. |
| `integrations/gingr` | Gingr read endpoint, DTO, response, webhook, and mapping boundaries | Provider evidence and mapping candidates for source normalization. | Domain truth by itself or permission to write/merge/delete provider records. |

## 5. What does the agent draft, rank, or recommend?

The agent may:

- Summarize source evidence and quote the relevant source refs/provenance.
- Group candidates by affected entity, issue category, issue, field path, severity, freshness, sensitivity/redaction status, workflow-blocking status, owner persona, reviewer role, and manual work removed.
- Rank hygiene actions by priority and labor impact.
- Draft internal cleanup tasks for staff review.
- Preserve ambiguity for review instead of rewriting it as certainty.
- Validate a draft submission against the packet id, correlation id, source refs, issue refs, review gates, and blocked side effects.
- Estimate before/after reconciliation minutes and record reviewed outcomes after a human disposition.

The executable contract covers missing vaccination evidence and ambiguous service-line examples: the packet produced source-grounded actions, kept `PreserveAmbiguityForReview` in allowed agent actions, and blocked `MutateProviderOrPmsRecord` plus `HideOrAutoResolveSourceAmbiguity` in [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../../app/tests/data_quality_hygiene_workflow_contracts.rs).

## 6. What must a human approve?

A manager, front-desk lead, operations analyst, or appropriate policy/document reviewer must approve:

- Duplicate customer/pet reconciliation, merge/delete decisions, and owner-pet relationship corrections.
- Missing or stale pet/customer/profile facts, including vaccine or medical/document evidence.
- Booking/reservation/service-line/checkout mapping cleanup that affects downstream workflows.
- Payment, deposit, refund, discount, POS, or checkout/payment-state conflicts.
- Region-policy or site-policy interpretation when a source defect affects portfolio reporting or a policy exception.
- Sensitive/quarantined payload handling and any redaction or release decision.
- Final provider/PMS/source-system edits, if a separate approved operational path exists.

## 7. Blocked or human-reviewed actions

Blocked by default:

- Sending customer messages.
- Mutating Gingr/PMS/POS/provider records.
- Changing staff schedules or live booking/checkout state.
- Moving refunds, discounts, deposits, or payments.
- Hiding, auto-resolving, or overwriting source ambiguity.
- Exposing quarantined sensitive payloads.
- Performing destructive profile merges/deletes without the approved staff workflow.
- Treating stale/unknown source context as accepted truth.

Draft validation is intentionally fail-closed: the API/app tests reject attempts to request `send_customer_message`, reject ambiguity hiding, and reject unsupported side effects such as direct source repair.

## 8. Outcome and labor value

- Estimated labor value: the packet can compare before/after reconciliation minutes for the candidate queue. Local API contract evidence shows a sample context with 55 before minutes, 22 after minutes, and 33 estimated minutes saved.
- Measured outcome record or field: `OutcomeRecord` and `DataQualityHygieneOutcomeRecord` capture action id, outcome, before minutes, actual minutes, estimated/actual minutes saved, actor/persona, feedback, source refs, issue refs, location, operating day, correlation id, action kind, owner persona, and resolution status after review. `DataQualityHygieneOutcomeSummary` aggregates reviewed outcomes for a location/day/correlation while preserving source refs, issue refs, disposition counts, estimated minutes, actual minutes spent, and completed actual minutes saved.
- Current evidence status: supported app/API/storage contract and local smoke/spec evidence. Storage codecs preserve labor/provenance, reject zero-minute evidence, and API tests prove the summary route reports reviewed minutes without enabling live side effects.
- Caveat: outcome capture records reviewed cleanup labor; it does not prove that a source system was repaired, that production NVA labor savings have been audited, or that a live provider writeback exists.

## Contract crosswalk links

Use the [workflow packet row](../../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map) for the bidirectional path from this workflow page back to the entities it consumes and produces. Use [surface inventory](../../entity-atlas/contract-crosswalk/surface-inventory.md) for source/Rustdoc/test proof, [source/provider flows](../../entity-atlas/contract-crosswalk/source-provider-flows.md) for source-entry and normalization evidence, [storage/persistence](../../entity-atlas/contract-crosswalk/storage-persistence.md) for `data-quality hygiene outcome record`, and [runtime exposure](../../entity-atlas/contract-crosswalk/runtime-exposure.md) for API/worker/CLI/web/script exposure. Rustdoc/module path: `app::data_quality_hygiene::{Candidate, Action, OutcomeRecord}`; operator-facing entity family: `Data-quality hygiene candidate/action`.

## 9. Evidence citations

Operator evidence and design:

- [Entity-driven workflow page template and evidence matrix](../../design/entity-driven-workflow-page-template.md#seven-workflow-pages-expected-featured-entities-and-contracts)
- [Workflow source and Rustdoc backing map](../../design/workflow-source-rustdoc-backing-map.md#data-quality-hygiene)
- [Data-quality hygiene labor loop](../../design/data-quality-hygiene-labor-loop.md)
- [Data-quality hygiene local smoke](../../ops/data-quality-hygiene-local-smoke.md)
- [Source evidence map](../../safety/source-evidence-map.md)
- [Review boundaries matrix](../../safety/review-boundaries-matrix.md)
- [Evidence policy, blocked actions, and outcomes](../../safety/evidence-policy-blocked-actions-outcomes.md)

Source and test evidence:

- Source: [`app/src/data_quality_hygiene.rs`](../../../app/src/data_quality_hygiene.rs) (`app::data_quality_hygiene::{Request, Packet, Candidate, CandidateKind, SourceFreshness, Sensitivity, Action, ActionKind, ActionPriority, SafeAgentAction, BlockedAction, DraftSubmission, DraftValidation, DraftRejectionReason, OutcomeRecord, FeedbackOutcome, LaborImpactEstimate, Workflow}`).
- Source: [`domain/src/data_quality.rs`](../../../domain/src/data_quality.rs) (`domain::data_quality::{Issue, Kind, FieldPath, Severity, ResolutionStatus}`) and [`domain/src/source.rs`](../../../domain/src/source.rs) (`domain::source::{RecordRef, Provenance}`).
- Source: [`storage/src/operations.rs`](../../../storage/src/operations.rs) (`storage::operations::{DataQualityHygieneOutcomeRecord, DataQualityHygieneOutcomeSummary, DataQualityHygieneReportingGroup, DataQualityHygieneOutcomeCode, DataQualityResolutionStatusCode, StoredSourceRecordRef}`).
- Source/API shell: [`apps/api/src/http.rs`](../../../apps/api/src/http.rs) for context, draft-validation, outcome capture, and outcome-summary routes.
- Tests: [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../../app/tests/data_quality_hygiene_workflow_contracts.rs), [`apps/api/tests/data_quality_hygiene_agent_contract.rs`](../../../apps/api/tests/data_quality_hygiene_agent_contract.rs), and [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../../storage/tests/data_quality_hygiene_outcome_storage.rs).
- Local smoke: [`scripts/smoke_data_quality_hygiene_local_loop.sh`](../../../scripts/smoke_data_quality_hygiene_local_loop.sh), [`app/examples/data_quality_hygiene_local_smoke.rs`](../../../app/examples/data_quality_hygiene_local_smoke.rs), and [Data-quality hygiene local smoke](../../ops/data-quality-hygiene-local-smoke.md).
- Rustdoc: generated locally under `target/doc/app/data_quality_hygiene/index.html`, `target/doc/domain/data_quality/index.html`, `target/doc/domain/source/index.html`, and `target/doc/storage/operations/index.html` after running `cargo doc --no-deps --workspace`.

Evidence status: supported local/app/API/storage contract for ranking, drafting, validation, blocked side effects, and outcome capture. Do not claim autonomous source cleanup, live provider/PMS mutation, production labor savings, or dedicated regional exception automation from this page.
