# Entity-driven workflow page template and evidence matrix

Purpose: give docs writers a reusable page contract for the seven operator workflow pages. Each page should read like an operator workflow first and an entity atlas entry second: name the labor problem, show the source-backed entities and app contracts that make the workflow safe, and cite Rustdoc/source/tests for every behavior claim.

Scope: documentation-only. This template does not authorize live customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, safety approvals, personnel actions, or production claims beyond linked source/Rustdoc/test evidence.

## Existing conventions to read first

These repo surfaces already define the naming and authority conventions this template uses:

| Convention source | What it defines for writers |
| --- | --- |
| [`README.md`](../../README.md#how-to-read-any-entity) and [`README.md`](../../README.md#documentation-contracts) | Entity-first docs should answer what the concept means, why it exists, related workflows/contracts, authority, allowed automation, blocked actions, outcomes, and proof. READMEs/operator pages are navigation; source/Rustdoc/tests are authority. |
| [`docs/quality/nva-documentation-style-guide.md`](../quality/nva-documentation-style-guide.md) | Page order: labor saved, operator English, source facts, agent/app boundary, human approval boundary, outcome measurement, source/Rustdoc evidence. |
| [`docs/design/entity-atlas-page-template.md`](entity-atlas-page-template.md) | Entity-page frontmatter/body shape, source-of-record language, allowed/blocked action defaults, outcome fields, and examples/non-examples. |
| [`docs/design/operator-workflow-page-inventory.md`](operator-workflow-page-inventory.md) | The seven required workflow pages, audiences, source data, draft/rank actions, approval gates, measured outcomes, glossary links, and inspection targets. |
| [`docs/design/workflow-page-source-rustdoc-map.md`](workflow-page-source-rustdoc-map.md) | Current evidence map for app/domain/storage/Gingr Rustdoc, source files, tests, workflow specs, and known gaps for each workflow page. |
| [`docs/safety/source-evidence-map.md`](../safety/source-evidence-map.md), [`docs/safety/review-boundaries-matrix.md`](../safety/review-boundaries-matrix.md), and [`docs/safety/evidence-policy-blocked-actions-outcomes.md`](../safety/evidence-policy-blocked-actions-outcomes.md) | Source evidence, review-gate, blocked-action, and outcome-capture language that keeps prose from becoming unauthorized behavior. |
| [`docs/glossary-workflow-state-terms.md`](../glossary-workflow-state-terms.md), [`docs/glossary-source-data-terms.md`](../glossary-source-data-terms.md), and [`docs/glossary-architecture-terms.md`](../glossary-architecture-terms.md) | Non-coder explanations for workflow packet, draft, review gate, blocked action, outcome capture, source-of-record, provider record, source ref, provenance, read model, domain, app, storage, and integration terms. |
| Source modules such as [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs), [`app/src/booking_triage.rs`](../../app/src/booking_triage.rs), [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs), [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs), [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs), [`app/src/daily_update.rs`](../../app/src/daily_update.rs), [`domain/src/source.rs`](../../domain/src/source.rs), [`domain/src/policy.rs`](../../domain/src/policy.rs), [`domain/src/workflow.rs`](../../domain/src/workflow.rs), and [`storage/src/operations.rs`](../../storage/src/operations.rs) | Compiled contract vocabulary: `Request`, `Packet`, review packet, draft artifact, `SafeAgentAction`, `BlockedAction`, `OutcomeRecord`, `RecordRef`, `Provenance`, `ReviewGate`, workflow events/status, and storage outcome projections. |
| App tests such as [`app/tests/manager_daily_brief_workflow_contracts.rs`](../../app/tests/manager_daily_brief_workflow_contracts.rs), [`app/tests/booking_triage_mvp.rs`](../../app/tests/booking_triage_mvp.rs), [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../app/tests/data_quality_hygiene_workflow_contracts.rs), [`app/tests/checkout_completion_workflow_contracts.rs`](../../app/tests/checkout_completion_workflow_contracts.rs), [`app/tests/crm_retention_workflow_contracts.rs`](../../app/tests/crm_retention_workflow_contracts.rs), and [`app/tests/daily_care_update_mvp.rs`](../../app/tests/daily_care_update_mvp.rs) | Executable evidence that app workflows keep source facts, review gates, allowed agent actions, blocked actions, and outcome/draft contracts separate. |

## Reusable workflow-page contract

Every operator workflow page must answer these questions, in this order. Use page-local headings that read naturally, but keep the evidence complete.

1. Problem solved: what repeated resort task, error cost, safety risk, or manager review burden does this workflow reduce?
2. Whose time saved: which role benefits first: front desk, general manager, assistant GM, groomer, care staff, regional ops, marketing/retention, or operations analyst?
3. Source data needed: which provider/read-model/domain facts are required, and what source refs/provenance must accompany them?
4. Featured entities: which pet-resort entities are central to the page, and which ones are only related context?
5. Related entities: which upstream/downstream entities, source evidence, review gates, storage records, or outcome records does the workflow touch?
6. Featured contracts: which `app`, `domain`, `storage`, and `integrations/gingr` modules/types/functions are the authoritative contract vocabulary?
7. Authority/source of truth: for each important fact or decision, who owns it: source system/provider evidence, domain policy, app workflow packet, storage projection, or human role?
8. Agent draft/rank/recommend boundary: what may the agent safely summarize, rank, classify, validate, draft, or record?
9. Human approvals: what must staff, managers, regional operators, or approved systems of record approve before action?
10. Blocked or human-reviewed actions: explicitly name forbidden live actions: customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, vaccine/medical/safety/policy approvals, personnel action, source hiding, destructive merges/deletes, or secret-dependent actions.
11. Measured outcomes/labor value: what outcome record, disposition, labor-minute estimate/actual, conversion, suppression, wrong-source finding, or review result proves value? If not implemented, say it is planned/future.
12. Code/Rustdoc/test backing: cite source paths, Rustdoc item paths, tests, workflow specs, generated-docs command, and evidence gaps.

## Copyable page skeleton

```md
# <Workflow name>

This page helps <role> avoid <manual work/error/safety risk> by using <source-backed packet/entity> to <rank/draft/summarize/validate/record> while <human role> keeps approval over <live/sensitive actions>.

Status: <implemented local contract | MVP preview | planned/future>

## Problem solved and time saved

- Problem solved:
- First role whose time is saved:
- Secondary reviewers/operators:
- Pet-resort example:

## Source data and featured entities

| Entity or source fact | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| <Reservation / customer / pet / issue / outcome> | <routing/review/draft reason> | <provider evidence + domain/app contract + human role> | <source + Rustdoc + test> |

Related entities to mention without making them the page's center:

- <related entity>: <adjacency and boundary>

## Featured contracts

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | `<module>::Request` / `<module>::Packet` | source-grounded packet building and review queue | live source mutation or customer send |
| `domain` | `<domain module/type>` | business vocabulary, policy, source evidence, review gate | provider-specific payload authority |
| `storage` | `<operation outcome record>` | durable outcome/labor evidence | policy decision by itself |
| `integrations/gingr` | `<endpoint/dto/mapping>` | provider evidence and mapping boundary | domain truth or approved side effect |

## Agent work, approvals, and blocked actions

- Agent may:
- Human must approve:
- Blocked by default:

## Outcome and labor value

- Estimated labor value:
- Measured outcome record or field:
- Current evidence status:
- Gap/future source need:

## Evidence citations

- Source: `<path>` (`<module::Type>` or `<function>`)
- Rustdoc: `target/doc/<crate>/<module>/...` generated by `<command>`; if not generated, cite the module/type path only.
- Tests: `<path>::<test name or file-level contract>`
- Supporting docs: `<path>#<heading>`
- Evidence status: `supported | local/MVP only | planned/future | gap`
```

## Evidence citation format

Use citations that let a later reviewer verify the claim without guessing which artifact was authority.

Preferred compact format:

```text
Evidence: Source `app/src/booking_triage.rs` (`app::booking_triage::{StaffEvaluationPacket, DeterministicResult, ConfirmationDraft, SafeAgentAction, BlockedAction}`); Rustdoc `target/doc/app/booking_triage/index.html` generated by `cargo doc --no-deps --workspace`; tests `app/tests/booking_triage_mvp.rs` and `app/tests/workflow_service_composition_contracts.rs`; supporting spec `docs/workflows/booking-triage-agent.md`; status: supported local/MVP contract, no autonomous booking/provider/payment action.
```

Citation rules:

- Source paths are required for every behavioral claim.
- Rustdoc item/module paths are required when the claim names a Rust type, enum, function, or module. Use `target/doc/...` only after `cargo doc --no-deps --workspace` has generated it; otherwise cite the Rust module path and source file.
- Tests are required for claims about allowed/blocked actions, outcome capture, deterministic policy gates, or draft validation when tests exist.
- Supporting docs can explain operator meaning, but they do not override source/Rustdoc/tests.
- The docs command must be named once near the evidence section. Current evidence maps use `cargo doc --no-deps --workspace` and local root `target/doc/`.
- Known warnings or gaps must stay attached to the page. Current evidence map notes pre-existing broken intra-doc link warnings in `domain/src/grooming/mod.rs`; grooming pages should cite concrete files/Rustdoc pages rather than broken bare links until fixed.

## Seven workflow pages: expected featured entities and contracts

| Workflow page | Expected featured entities | Related entities / source evidence | Featured app contracts | Domain/storage/Gingr contracts | Evidence status and gap to preserve |
| --- | --- | --- | --- | --- | --- |
| [Manager Daily Brief](../workflows/operator/manager-daily-brief.md) | Operating day/location, demand/occupancy/labor facts, brief action, labor-impact estimate, outcome record | Checkout packets, retention opportunities, data-quality issues, source refs, prior outcomes | `app::manager_daily_brief::{Request, Packet, BriefAction, BriefActionKind, SafeAgentAction, BlockedAction, LaborImpactEstimate, OutcomeRecord}` | `domain::daily_brief::{ResortOperatingDay, OccupancySnapshot, LaborRisk, CustomerFollowUp, RevenueOpportunity, Action}`; `domain::analytics::service_demand::Fact`; `storage::operations::ManagerDailyBriefOutcomeRecord`; Gingr reservations/back-of-house/timeclock evidence | Supported local contract and storage outcome evidence. Do not claim production-verified NVA labor savings or live staffing/schedule/customer side effects. |
| [Booking Triage](../workflows/operator/booking-triage.md) | Booking request, reservation, customer, pet, service, deterministic result, staff evaluation packet, confirmation draft | Vaccine/document evidence, deposit/payment state, care/behavior notes, availability/capacity, policy gates, source refs | `app::booking_triage::{Request, Service, DeterministicResult, StaffEvaluationPacket, ConfirmationDraft, AuditEventDraft, SafeAgentAction, BlockedAction}` | `domain::policy::ReviewGate`; `domain::workflow::{PolicyContext, RecommendedAction}`; `domain::entities::Reservation`; Gingr reservation/back-of-house/service endpoints and mapping docs | Supported MVP/app contract. No autonomous booking confirmation, waitlist movement, vaccine/payment approval, customer send, or provider mutation. |
| [Data Quality Hygiene](../workflows/operator/data-quality-hygiene.md) | Data-quality issue, field path, severity, resolution status, hygiene candidate/action, draft submission/validation, outcome record | Source refs/provenance, provider records, location/operating day, workflow-blocking state, BI visibility, sensitivity/redaction metadata | `app::data_quality_hygiene::{Request, Packet, Candidate, Action, DraftSubmission, DraftValidation, SafeAgentAction, BlockedAction, OutcomeRecord}` | `domain::data_quality::{Issue, Kind, FieldPath, Severity, ResolutionStatus}`; `domain::source::{RecordRef, Provenance}`; `storage::operations::DataQualityHygieneOutcomeRecord` | Supported app/storage outcome contract. Runtime/API shells beyond local smoke/specs should be marked planned unless cited. |
| [Checkout Completion](../workflows/operator/checkout-completion.md) | Reservation id, checkout completion status, checkout packet, staff handoff, audit-event draft | Source checkout/PMS status, provenance, care summary, belongings status, departure notes, payment/care/source exceptions, review gates | `app::checkout_completion::{Request, Packet, CompletionStatus, StaffHandoff, SafeAgentAction, BlockedAction, AuditEventDraft, Workflow}` | `domain::source`; `domain::workflow`; `domain::policy::ReviewGate`; Gingr reservations endpoint/mapping evidence | Supported app workflow/tests. Dedicated durable checkout outcome projection is not yet identified; describe persistence as planned unless later source adds it. |
| [Grooming Rebooking / Retention](../workflows/operator/grooming-rebooking-retention.md) | Retention opportunity, grooming rebook opportunity kind, contact permission, follow-up eligibility, staff review packet, outcome record | Completed checkout/stay evidence, customer/pet ids, grooming history/cadence, service history, consent/suppression flags, preferred channel, source refs | `app::crm_retention::{Request, Packet, RetentionOpportunity, OpportunityKind, SourceGroundedReasonCode, ContactPermission, FollowUpEligibility, StaffReviewPacket, SafeAgentAction, BlockedAction, OutcomeRecord}` | `domain::grooming::{Contract, DurationEstimate, ReviewRequirement}` plus grooming service/history/rebooking modules; `domain::message`; `domain::lead`; `domain::reputation` | Supported retention packet/outcome evidence and grooming vocabulary. No automatic grooming appointment creation, discount/payment action, or customer outreach. Preserve current grooming Rustdoc warning caveat. |
| [Daily Updates / Pawgress Drafts](../workflows/operator/daily-updates-pawgress-drafts.md) | Daily update preview request/preview, customer message draft, included/omitted facts, internal flags, review disposition, send stub | Staff care notes, task evidence, reservation/pet/update window, approved media refs, note provenance/version, redaction/sensitivity flags, tone policy | `app::daily_update::{MvpPreviewRequest, MvpPreview, CustomerMessageDraft, ReviewDisposition, InternalFlag, IncludedFact, OmittedFact, SendStub, build_mvp_preview}` and `app::daily_update::daily_care_update::{Input, Output, Agent}` | `domain::workflow::Event`; `domain::message` state vocabulary; daily-care workflow specs/parts | Supported MVP preview/draft contracts and tests. No production Pawgress delivery, media publication, care-task completion, incident/medical decision, or live messaging integration. |
| [Regional Labor Exceptions / Future Portfolio View](../workflows/operator/regional-labor-exceptions.md) | Regional/portfolio exception, off-plan site/metric, reporting group, manager daily brief outcomes, data-quality outcomes | Portfolio/regional read models, labor risk, demand/staffing variance, utilization/capacity, incidents/reviews, peer/trend context, source refs | No dedicated `app::regional_labor_exceptions` module found; reuse `app::manager_daily_brief` and data-quality outcomes only as seed evidence | `domain::operations::{AiUseCase::RegionalOpsExceptionReporting, AiUseCase::RegionalPerformanceBenchmarking, OperatingFunction::RegionalOperations}`; `domain::daily_brief`; `domain::reputation`; `storage::operations::PetResortPortfolioRecord`; Gingr labor ops/reservation source evidence | Planned/future. Must say there is no dedicated app module, packet, API endpoint, or durable regional exception outcome record yet. |

## Final writer checklist

Before handing off a workflow page, verify:

- The first paragraph names the role and manual work reduced before module/type names appear.
- Source facts are provider/read-model/domain facts with `RecordRef`/`Provenance` or equivalent evidence, not model memory or prior summaries.
- Featured entities are distinct from related context; provider DTOs/storage codes are not promoted into business authority by prose.
- App contracts name the review packet/draft/recommendation/outcome, plus `SafeAgentAction` and `BlockedAction` where present.
- Human approval and blocked actions are explicit for customer/provider/payment/schedule/safety/personnel/source-hiding actions.
- Outcome/labor-value claims cite an implemented outcome field/record or are marked planned/future.
- The evidence section includes source path, Rustdoc module/type/function path, tests, docs command, and known gaps.
- Local links should pass `./scripts/check_markdown_links.py`; when Rustdoc/source links or executable docs change, use `./scripts/check_docs.sh` or the narrower command named by the task.

## Known evidence gaps to preserve

- Regional labor exceptions remain planned/future until a dedicated app module/packet/API endpoint/storage outcome exists.
- Checkout completion has app workflow/test evidence but no dedicated durable checkout outcome projection identified in `storage/src/operations.rs`.
- Daily Updates/Pawgress pages may cite MVP preview/draft/test evidence, not production send/media-publication integration.
- Booking triage pages may cite MVP/app service evidence, not live booking/provider mutation, payment capture, vaccine approval, or waitlist movement.
- Grooming rebooking/retention pages may cite CRM retention and grooming vocabulary, not autonomous appointment creation or customer outreach; cite concrete files/Rustdoc pages until grooming intra-doc link warnings are fixed.
