---
title: "Outcome, Labor, Operations, Analytics, Money, and Safety Evidence Atlas"
slug: "outcomes-operations-money"
family: "outcomes-labor-operations-analytics-money"
status: "draft"
audience: ["general-manager", "front-desk-lead", "regional-ops", "docs-writer", "engineer"]
plain_english_definition: "A family of evidence entities that show whether resort automation saved staff time, preserved review gates, completed safe internal work, or correctly blocked unsafe work."
primary_labor_problem: "Turns source-backed workflow help into measurable proof instead of unverified claims about AI productivity."
source_of_record: "App workflow outcome records, storage outcome records, domain operations/analytics/money/payment contracts, and source refs/provenance from provider facts."
authoritative_human_role: "general manager, front-desk lead, regional operator, or the reviewer who performed the work"
workflow_links: ["manager-daily-brief", "data-quality-hygiene", "checkout-completion", "retention-grooming-rebooking", "daily-updates"]
source_paths:
  - "domain/src/operations.rs"
  - "domain/src/daily_brief.rs"
  - "domain/src/analytics.rs"
  - "domain/src/money/mod.rs"
  - "domain/src/payment/mod.rs"
  - "app/src/manager_daily_brief.rs"
  - "app/src/data_quality_hygiene.rs"
  - "app/src/checkout_completion.rs"
  - "app/src/crm_retention.rs"
  - "app/src/daily_update.rs"
  - "storage/src/operations.rs"
rustdoc_contracts:
  - "domain::operations::*"
  - "domain::daily_brief::*"
  - "domain::analytics::{ProjectionVersion, stay::Fact, service_demand::Fact}"
  - "domain::money::{Money, MinorUnits, Currency}"
  - "domain::payment::{Deposit, DepositStatus, Reference}"
  - "app::manager_daily_brief::{Packet, BriefAction, LaborImpactEstimate, OutcomeRecord}"
  - "app::data_quality_hygiene::{Packet, Action, Candidate, OutcomeRecord}"
  - "app::checkout_completion::{Packet, CompletionStatus, SafeAgentAction, BlockedAction}"
  - "app::crm_retention::{RetentionOpportunity, FollowUpOutcome, OutcomeRecord}"
  - "storage::operations::{ManagerDailyBriefOutcomeRecord, DataQualityHygieneOutcomeRecord}"
glossary_links:
  - "../glossary-source-data-terms.md"
  - "../glossary-workflow-state-terms.md"
  - "../glossary-architecture-terms.md"
allowed_action_summary: "summarize, draft, rank, validate, preserve ambiguity, record reviewed outcomes, and calculate labor minutes saved only through app-owned contracts"
blocked_action_summary: "no customer sends, provider/PMS writes, schedule mutations, payment/refund/discount movement, source hiding, or safety/policy approvals without a separate approved workflow and human review"
outcome_fields: ["drafted vs sent", "reviewed vs blocked", "completed checkout", "hygiene issue resolved", "rebooking opportunity captured", "outcome record stored", "labor minutes", "source refs", "correlation id"]
---

# Outcome, labor, operations, analytics, money, and safety evidence atlas

Purpose: give non-coder documentation writers a family atlas page for the entities that prove the system saved time or prevented unsafe work. These entries sit across the [labor-cost reduction crosswalk](labor-cost-reduction-crosswalk.md), [Manager Daily Brief measurable labor loop](manager-daily-brief-measurable-labor-loop.md), and [Data-quality Hygiene measurable labor loop](data-quality-hygiene-labor-loop.md). They should be read with the [entity atlas page template](entity-atlas-page-template.md) and [entity atlas inventory](entity-atlas-inventory.md).

This page does not authorize new product behavior. It explains how the existing domain, app, and storage contracts make labor evidence traceable back to source facts and human review.

## 1. Family definition in pet-resort language

This family answers one operator question: "How do we know the workflow actually helped without doing something unsafe?"

The answer is not "an agent said it helped." The answer is a chain of evidence:

```text
Provider/source fact or staff evidence
  -> provenance / source ref / data-quality status
  -> app workflow packet with allowed and blocked actions
  -> reviewed draft, ranked queue, or internal task
  -> human disposition or blocked result
  -> outcome record with before/actual minutes and source refs
  -> storage record / reporting group for later measurement
```

The family includes:

- outcome records: final reviewed dispositions such as completed, deferred, suppressed, wrong-source, not-actionable, converted, or human-sent;
- labor measurement: before minutes, after/actual minutes, minutes saved, cost or payment-adjacent fields, and reporting groups;
- operational evidence: daily brief actions, checkout completion evidence, hygiene actions, retention opportunities, and safe-result envelopes;
- analytics facts: source-derived stay and service-demand facts used to make manager/regional actions explainable;
- operations context: portfolio, service offering, operating day, service line, and daily brief entities that scope where the labor claim applies;
- money/payment evidence: payment references, deposit status, and typed money fields used for reconciliation while blocking payment movement.

## 2. Purpose: labor-cost or safety problem reduced

The family prevents two bad outcomes:

1. Unmeasured labor claims: a workflow may look helpful because it drafted a message or ranked a queue, but there is no proof unless staff records what happened and how many minutes were actually saved.
2. Unsafe automation claims: a workflow may know about checkout status, payment status, customer messaging, or data-quality ambiguity, but it must prove whether the result stayed drafted, reviewed, blocked, or completed internally before anyone treats it as operational action.

The safe outcome is measurable and reviewable: "front desk completed checkout review in 8 minutes instead of 20," "manager blocked a draft because source facts were wrong," "hygiene issue stayed visible until reviewed," or "retention opportunity was captured but no customer message was sent automatically."

## 3. Workflows where this family appears

| Workflow | How these entities appear | Safe workflow result |
| --- | --- | --- |
| [Manager Daily Brief](manager-daily-brief-measurable-labor-loop.md) | `BriefAction`, `LaborImpactEstimate`, `OutcomeRecord`, `SourceFact`, and blocked actions show what the manager reviewed and why. | Ranked manager/front-desk action plus recorded actual minutes; no schedule, provider, customer-message, payment, refund, or discount mutation. |
| [Data-quality Hygiene](data-quality-hygiene-labor-loop.md) | `Candidate`, `Action`, `DraftValidation`, `OutcomeRecord`, source refs, issue refs, and resolution status keep ambiguity visible. | Internal cleanup/review task and reviewed outcome; no hiding ambiguity or provider/PMS repair from the agent loop. |
| Checkout Completion | `Packet`, `CompletionStatus`, `StaffHandoff`, care summary, source provenance, safe actions, and blocked actions separate completed checkout proof from provider mutation. | Staff-reviewed checkout/completion evidence; agent may summarize or draft an internal task, not write checked-out status. |
| Retention / Grooming Rebooking | `RetentionOpportunity`, contact permission, `FollowUpEligibility`, draft-only safe actions, `FollowUpOutcome`, and `OutcomeRecord` capture opportunity and disposition. | Safe rebooking opportunity or follow-up draft for review; no customer send, discount/payment movement, or schedule/provider update. |
| Daily Updates / Pawgress Drafts | Customer message drafts, included/omitted facts, internal flags, review disposition, and send stubs show drafted vs sent. | Customer-safe draft and review outcome; no unreviewed customer send. |
| Regional Exceptions | Aggregated outcome records, reporting groups, operations context, analytics facts, and data-quality status explain off-plan sites. | Regional review queue with source-backed next actions; no personnel action, pricing change, or schedule mutation. |

## 4. Relationships and adjacency

```text
Source system / staff evidence
  -> source::Provenance and source::RecordRef
  -> analytics::stay::Fact or analytics::service_demand::Fact
  -> daily_brief::Resort / operations::operating_day::Key
  -> app workflow Packet
  -> SafeAgentAction + BlockedAction
  -> draft/review/block/completion disposition
  -> app OutcomeRecord
  -> storage outcome record + reporting group
```

Do not confuse these boundaries:

- App outcome record vs storage outcome record: the app record is the reviewed workflow contract; the storage record is durable evidence/reporting shape.
- Labor estimate vs actual labor evidence: estimates support prioritization; actual minutes in a reviewed outcome prove or disprove the savings claim.
- Draft vs send: drafts save writing time, but a draft is not customer contact. Sent-by-human or approval evidence must be separate.
- Completed checkout vs provider status mutation: checkout evidence can be staff-verified without the agent writing a PMS/provider status.
- Hygiene resolution vs source repair: a hygiene outcome may record that staff reviewed or repaired an issue, but the workflow itself does not hide ambiguity or write provider records.
- Payment/deposit evidence vs money movement: payment references and deposit status help staff reconcile, but refunds, discounts, deposits, or charges remain blocked side effects unless a separate app-owned payment workflow exists.

## 5. Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Operations domain | [`domain/src/operations.rs`](../../domain/src/operations.rs) | Portfolio, operating day, service line, operating function, service offering, and AI use-case vocabulary scope the work. |
| Daily brief domain | [`domain/src/daily_brief.rs`](../../domain/src/daily_brief.rs) | Manager-facing sections, labor risk, risks, and recommended actions are read-model results, not side effects. |
| Analytics domain | [`domain/src/analytics.rs`](../../domain/src/analytics.rs) | Stay and service-demand facts keep projection version, source refs, and data-quality status attached. |
| Money domain | [`domain/src/money/mod.rs`](../../domain/src/money/mod.rs) | Money uses typed minor units and currency so cost/payment facts do not become vague strings. |
| Payment domain | [`domain/src/payment/mod.rs`](../../domain/src/payment/mod.rs) | Deposits record payment readiness, references, refund windows, status, and collection need without executing payment movement. |
| Manager Daily Brief app workflow | [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs) | Packet, action, labor estimate, allowed/blocked actions, and outcome record capture manager review and minutes saved. |
| Data-quality Hygiene app workflow | [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs) | Candidate, action, draft validation, outcome record, issue refs, and resolution state keep ambiguity visible. |
| Checkout Completion app workflow | [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs) | Completion status, staff handoff, source evidence, safe actions, and blocked actions keep provider mutation blocked. |
| CRM Retention app workflow | [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs) | Retention opportunities, follow-up eligibility, safe draft actions, blocked sends, and follow-up outcomes separate opportunity capture from customer contact. |
| Daily Update app workflow | [`app/src/daily_update.rs`](../../app/src/daily_update.rs) | Drafts, included facts, omitted facts, review disposition, internal flags, and send stub preserve drafted-vs-sent evidence. |
| Storage operations boundary | [`storage/src/operations.rs`](../../storage/src/operations.rs) | Persisted outcome records, labor-minute wrappers, source refs, reporting groups, and storage codes keep evidence durable. |
| Crosswalk docs | [`docs/design/labor-cost-reduction-crosswalk.md`](labor-cost-reduction-crosswalk.md) | Every labor claim must have typed context packet, source refs, blocked actions, review gates, outcome capture, and labor metric. |

Rustdoc/module paths to cite in future rendered docs:

- `domain::operations::{ServiceOffering, service_core::ServiceContracts, service_core::ServiceLine, pet_resort::Portfolio, TechnologyEcosystem, AiUseCase, OperatingFunction}`
- `domain::daily_brief::{ResortOperatingDay, Resort, Section, OccupancySnapshot, LaborSnapshot, LaborRisk, Action, Risk}`
- `domain::analytics::{ProjectionVersion, stay::Fact, service_demand::Fact, service_demand::DemandUnits}`
- `domain::money::{Money, MinorUnits, Currency}`
- `domain::payment::{Deposit, DepositStatus, Reference}`
- `app::manager_daily_brief::{Packet, BriefAction, SourceFact, LaborImpactEstimate, OutcomeRecord}`
- `app::data_quality_hygiene::{Packet, Candidate, Action, DraftSubmission, DraftValidation, OutcomeRecord}`
- `app::checkout_completion::{Packet, CompletionStatus, StaffHandoff, SafeAgentAction, BlockedAction, AuditEventDraft}`
- `app::crm_retention::{RetentionOpportunity, OpportunityEvidence, ContactPermission, FollowUpEligibility, FollowUpOutcome, OutcomeRecord}`
- `storage::operations::{ManagerDailyBriefOutcomeRecord, DataQualityHygieneOutcomeRecord, StoredManagerDailyBriefLaborMinutes, StoredDataQualityHygieneLaborMinutes}`

Rendered Rustdoc is not assumed here; source paths are the authority until docs are generated or published.

## 6. Authoritative source system or human role

| Fact or decision | Source of record | Human role when source is incomplete or sensitive |
| --- | --- | --- |
| Provider reservation, checkout, owner, pet, service, or payment reference fact | Provider/source evidence plus `source::Provenance` or `source::RecordRef` | Front-desk lead or general manager verifies exceptions. |
| Analytics stay or service-demand projection | `domain::analytics` fact with projection version, source refs, and data-quality status | General manager or operations analyst reviews nonblocking/blocked data-quality status. |
| Manager daily brief action disposition | `app::manager_daily_brief::OutcomeRecord` and storage `ManagerDailyBriefOutcomeRecord` | General manager, assistant GM, or front-desk lead. |
| Data-quality hygiene disposition | `app::data_quality_hygiene::OutcomeRecord` and storage `DataQualityHygieneOutcomeRecord` | General manager, front-desk lead, regional operator, or operations analyst. |
| Customer-message draft vs send | App workflow draft/review disposition and explicit approval/send evidence | Approved sender or manager. |
| Checkout completed vs provider written | Staff handoff/completion evidence and checkout packet | Front-desk lead or manager; provider/PMS write requires separate approved workflow. |
| Rebooking opportunity captured vs customer contacted | CRM retention opportunity/outcome and contact permission | Front-desk lead or approved sender. |
| Payment/deposit readiness | `domain::payment::Deposit` status and payment reference | Manager/front desk handles real collection, refund, discount, waiver, or payment movement outside this atlas entry. |

## 7. Allowed actions

Automation or agent workflows may safely use this family to:

- read source-backed evidence and display it with provenance;
- map provider facts into explicit analytics, operations, or workflow candidates;
- rank manager/front-desk work queues;
- draft internal cleanup tasks, checkout summaries, daily-update messages, or retention follow-up language for review;
- validate whether draft submissions cite source refs and request no blocked side effects;
- preserve data-quality ambiguity as visible work;
- record reviewed dispositions, source refs considered, labor minutes, actor/persona, correlation id, and feedback;
- calculate minutes saved from reviewed before/actual minutes.

Use narrow language: "drafted," "ranked," "reviewed," "blocked," "captured," "recorded," or "completed internally." Do not write "sent," "updated Gingr," "collected payment," "changed schedule," or "approved safety decision" unless a linked app-owned contract explicitly proves that side effect and human gate.

## 8. Blocked actions and review gates

Default no-go boundaries for this family:

- unreviewed customer/member sends;
- PMS/provider writes, check-in/out status mutation, source-data deletion, or source hiding;
- booking, room, yard, schedule, staffing, payroll, or service mutations;
- payment, refund, discount, deposit collection, deposit waiver, package/session-balance movement, or charge movement;
- vaccine, medical, incident, temperament, safety, behavior, or policy approvals;
- personnel actions or staff discipline;
- raw sensitive payload exposure to an agent runtime;
- treating missing or stale provenance as a completed labor-saving result.

A blocked result is still useful evidence. It proves unsafe work was prevented, such as "draft rejected because source refs were missing," "checkout status not suggested because staff handoff was absent," or "payment-related ambiguity routed to manager review instead of moving money."

## 9. Safe-use evidence and outcome fields

For this family, safe-use evidence means the row, packet, or page can answer all of these questions:

| Evidence question | Field or contract to look for | Why it matters |
| --- | --- | --- |
| What source fact caused the recommendation? | `source_refs`, `provenance`, provider endpoint/raw payload refs, projection version | Prevents free-text recommendations from becoming operational truth. |
| What work was removed? | removed manual work, action kind, before minutes | Shows which dashboard scan, reconciliation step, writing task, or review queue was reduced. |
| What actually happened? | outcome/disposition, review status, follow-up outcome, completion status | Separates completed work from deferred, blocked, suppressed, or wrong-source outcomes. |
| Who reviewed or recorded it? | actor id, actor persona, owner persona, reviewer role | Makes labor evidence accountable to staff/manager review. |
| How many minutes were saved? | before minutes, actual/after minutes, `actual_minutes_saved()` / derived saved minutes | Turns estimated productivity into measurable evidence. |
| Was unsafe work blocked? | blocked actions, required review gates, draft validation rejection reason | Proves prevention of unsafe sends, source writes, schedule changes, payment movement, or ambiguity hiding. |
| Can reports group it later? | location id, operating day, action kind, reporting group, correlation id | Makes regional and portfolio reporting reproducible. |

Required outcome fields for new pages in this family:

- workflow/action id;
- action kind;
- location and operating day where applicable;
- owner persona and actor persona;
- final disposition/outcome;
- before minutes and actual minutes;
- derived minutes saved;
- source refs considered;
- issue refs or opportunity refs where applicable;
- review gate or blocked-action reason;
- correlation id and recorded timestamp;
- feedback/correction reason when source facts were wrong.

## 10. Family entries

### 10.1 Labor Minutes

Plain-English definition: Labor Minutes are the before-and-after time values that show whether a workflow reduced manager, front-desk, or operations analyst work.

Source paths and contracts:

- [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs): `LaborMinutes`, `AggregateLaborMinutes`, `LaborImpactEstimate`.
- [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs): `LaborMinutes`, `AggregateLaborMinutes`.
- [`storage/src/operations.rs`](../../storage/src/operations.rs): `StoredManagerDailyBriefLaborMinutes`, `StoredDataQualityHygieneLaborMinutes`.

Labor proof rule: estimated minutes are useful for ranking, but labor savings are proven only when a reviewed outcome records actual minutes. The calculation is the before value minus actual/after value, using typed minute fields rather than prose.

Allowed: estimate, total, persist, and report minutes tied to source-backed workflow actions.

Blocked: counting a draft, recommendation, or agent response as savings without a reviewed outcome record.

Examples:

| Type | Item | Why |
| --- | --- | --- |
| Example | 45 minutes before staffing scan, 15 actual minutes after brief review | Shows manager time avoided for demand-versus-staffing review. |
| Example | 25 minutes before source hygiene investigation, 9 actual minutes after staff review | Shows repeated source reconciliation was reduced. |
| Non-example | "The AI saved time" | No actor, source refs, outcome, or actual minutes. |
| Non-example | Zero or missing minute value in storage | Storage labor-minute wrappers require positive values for persisted evidence. |

### 10.2 Manager Daily Brief Outcome Record

Plain-English definition: A Manager Daily Brief Outcome Record is the manager/front-desk feedback row that says whether a brief action was completed, deferred, suppressed, or found wrong-source, and how many minutes the reviewed work actually took.

Source paths and contracts:

- [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs): `Packet`, `BriefAction`, `LaborImpactEstimate`, `OutcomeRecord`.
- [`storage/src/operations.rs`](../../storage/src/operations.rs): `ManagerDailyBriefOutcomeRecord`, outcome/persona/action codes, reporting group.
- [`docs/design/manager-daily-brief-measurable-labor-loop.md`](manager-daily-brief-measurable-labor-loop.md): reviewed feedback and metric contract.

Traceability chain:

```text
SourceFact with source refs
  -> BriefAction kind and owner persona
  -> LaborImpactEstimate before/after estimate
  -> manager/front-desk review
  -> OutcomeRecord with actual minutes and source refs
  -> storage ManagerDailyBriefOutcomeRecord
```

How it proves savings or safety:

- completed result proves reviewed work happened;
- deferred/suppressed result prevents overclaiming labor savings;
- source-fact-was-wrong result prevents bad evidence from becoming a success metric;
- blocked actions prove the brief did not change schedule, provider status, customer messaging, refunds, discounts, payments, or data-quality visibility.

### 10.3 Data-quality Hygiene Outcome Record

Plain-English definition: A Data-quality Hygiene Outcome Record is the reviewed result of an internal cleanup/reconciliation action, including whether the issue was completed, deferred, suppressed, wrong-source, or not actionable.

Source paths and contracts:

- [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs): `Packet`, `Candidate`, `Action`, `DraftValidation`, `OutcomeRecord`.
- [`storage/src/operations.rs`](../../storage/src/operations.rs): `DataQualityHygieneOutcomeRecord`, issue refs, resolution status, labor-minute fields, reporting group.
- [`docs/design/data-quality-hygiene-labor-loop.md`](data-quality-hygiene-labor-loop.md): action kinds, draft validation, outcome endpoint, blocked actions.

Traceability chain:

```text
data_quality::Issue + source::RecordRef
  -> hygiene Candidate
  -> app Action with review gate and estimated minutes
  -> draft validation rejects unsafe requests
  -> staff/manager outcome with source refs considered
  -> storage DataQualityHygieneOutcomeRecord
```

How it proves savings or safety:

- completed/internal review proves source reconciliation work was done;
- issue refs and resolution status show which ambiguity was reviewed;
- deferred/suppressed/not-actionable states keep labor reports honest;
- source-fact-was-wrong records the correction instead of hiding bad evidence;
- blocked actions prevent provider/PMS repair, customer sends, schedule changes, payment movement, and ambiguity hiding from the agent loop.

### 10.4 Analytics Facts

Plain-English definition: Analytics Facts are source-derived stay and service-demand records used to explain occupancy, staffing, service demand, and regional exceptions without treating raw provider data as already-safe truth.

Source paths and contracts:

- [`domain/src/analytics.rs`](../../domain/src/analytics.rs): `ProjectionVersion`, `stay::Fact`, `stay::DataQualityStatus`, `service_demand::Fact`, `service_demand::DemandUnits`.
- [`domain/src/source.rs`](../../domain/src/source.rs): provenance and source refs that analytics facts carry forward.
- [`domain/src/data_quality.rs`](../../domain/src/data_quality.rs): data-quality issues that can block or qualify projections.

Safe-use evidence:

- projection version identifies which deterministic read-model logic produced the fact;
- source refs/provenance explain where the provider facts came from;
- data-quality status marks complete, manager-review-required, or blocking issues;
- demand units and operating-day keys connect analytics to labor planning and daily briefs.

Allowed: feed manager brief actions and regional exception reporting with source-backed facts.

Blocked: using raw provider DTO fields directly as manager truth, ignoring blocking issues, or removing nonblocking caveats from reports.

### 10.5 Operations Context and Service Offering

Plain-English definition: Operations Context tells which resort, operating day, service line, portfolio, and service offering a labor claim belongs to.

Source paths and contracts:

- [`domain/src/operations.rs`](../../domain/src/operations.rs): `operating_day::Key`, `pet_resort::Portfolio`, `ServiceOffering`, `service_core::ServiceLine`, `TechnologyEcosystem`, `AiUseCase`, `OperatingFunction`.
- [`domain/src/daily_brief.rs`](../../domain/src/daily_brief.rs): `ResortOperatingDay`, `Resort`, `Section`, `LaborSnapshot`, `Action`, `Risk`.
- [`storage/src/operations.rs`](../../storage/src/operations.rs): service offering records, portfolio records, outcome reporting groups.

How it proves savings or safety:

- location and operating day keep labor evidence scoped to the site/day where work happened;
- service line prevents boarding, daycare, grooming, training, retail, and payment facts from being flattened into vague operations text;
- reporting group lets regional leaders compare like with like;
- daily brief actions require manager approval for risky labor recommendations.

Allowed: scope, group, and explain operational work.

Blocked: treating the operations context as authority to mutate schedules, staffing, payroll, pricing, provider records, or customer messages.

### 10.6 Money and Payment Evidence

Plain-English definition: Money and Payment Evidence are typed amounts, currencies, deposit statuses, refund windows, and payment references used to help staff reconcile work without moving money automatically.

Source paths and contracts:

- [`domain/src/money/mod.rs`](../../domain/src/money/mod.rs): `Money`, `MinorUnits`, `Currency`.
- [`domain/src/payment/mod.rs`](../../domain/src/payment/mod.rs): `Deposit`, `DepositStatus`, `Reference`, `requires_collection()`.
- [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs): checkout packets and blocked actions for payment exceptions.
- [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs): data-quality handling for payment-state conflicts and sensitive/quarantined payloads.

How it proves savings or safety:

- typed money prevents ambiguous strings such as "$50ish" from driving checkout/payment review;
- deposit status distinguishes required, paid, refunded, failed, and waived-by-manager states;
- payment references support reconciliation and audit;
- payment-state conflicts become data-quality or checkout review work, not automatic money movement.

Allowed: summarize deposit readiness, route collection/reconciliation review, display payment references, and record outcome evidence.

Blocked: collecting deposits, refunding, discounting, waiving, charging, moving package/session balances, or changing payment status through an agent loop.

### 10.7 Safe Result Envelope / Draft-Review-Blocked State

Plain-English definition: A Safe Result Envelope is the workflow result shape that packages source evidence, allowed actions, blocked actions, review gates, draft state, and outcome/disposition so a reviewer can see what did and did not happen.

Source paths and contracts:

- [`domain/src/workflow.rs`](../../domain/src/workflow.rs): workflow event/result/status vocabulary.
- [`app/src/booking_triage.rs`](../../app/src/booking_triage.rs): booking packet, confirmation draft, and audit event draft.
- [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs): checkout packet and completion status.
- [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs): follow-up opportunity/outcome and blocked customer sends.
- [`app/src/daily_update.rs`](../../app/src/daily_update.rs): customer message draft, review disposition, internal flags, send stub.
- [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs): manager actions and blocked actions.
- [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs): draft validation and outcome capture.

Safe-result proof examples:

| Evidence phrase | What it proves | What it does not prove |
| --- | --- | --- |
| Drafted confirmation or Pawgress message | Staff writing time may have been reduced. | It was not sent unless send approval/evidence exists. |
| Reviewed and blocked checkout completion | Unsafe provider status mutation was prevented. | Checkout is not complete in the provider/PMS. |
| Completed checkout review | Staff verified completion evidence in the app workflow. | The agent wrote the provider status. |
| Hygiene issue resolved or acknowledged | A human reviewed ambiguity and recorded a disposition. | The provider record was repaired by the agent. |
| Rebooking opportunity captured | A safe candidate was identified with source evidence and contact constraints. | A customer was contacted or appointment booked. |
| Outcome record stored | A durable labor/safety evidence row exists. | The result was necessarily positive; check disposition and actual minutes. |

## 11. Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | `ManagerDailyBriefOutcomeRecord` with `Completed`, before minutes, actual minutes, actor persona, source refs, location, operating day, and action kind | Proves reviewed work and lets reports calculate actual minutes saved. |
| Example | `DataQualityHygieneOutcomeRecord` with issue refs, source refs, `resolution_status_after_review`, and `SourceFactWasWrong` | Proves unsafe or stale source evidence was caught rather than hidden. |
| Example | `crm_retention::RetentionOpportunity` with contact permission and `FollowUpOutcome` | Captures a rebooking opportunity while separating review/send/convert outcomes. |
| Example | `checkout_completion::Packet` with staff handoff and blocked provider mutation | Shows completed checkout review can be safe without autonomous PMS writes. |
| Example | `payment::Deposit` with `DepositStatus::Required` and `requires_collection()` | Supports front-desk reconciliation review without collecting money. |
| Non-example | Raw provider reservation id by itself | Needs provenance/source refs and workflow context before it can support a labor claim. |
| Non-example | A customer-message string | It may be a draft; it is not proof of send, approval, or labor saved. |
| Non-example | A storage code by itself | Codes support durable persistence but are not standalone business proof without the surrounding record. |
| Non-example | A claimed percent reduction without source refs and outcome rows | Labor savings must come from reviewed outcome capture, not prose. |

## 12. Glossary cross-links

Use these glossary families when splitting this family page into standalone atlas pages later:

- [domain, app, storage, adapter, DTO, and tool-port terms](../glossary-architecture-terms.md);
- [Gingr, source-of-record, provider record, provenance, source refs, and data-quality terms](../glossary-source-data-terms.md);
- [draft, review gate, blocked action, workflow packet, and outcome capture terms](../glossary-workflow-state-terms.md).

## 13. Writer checklist for future standalone pages

When turning any entry above into a standalone page, preserve these requirements:

1. Name the labor or safety problem reduced.
2. Link source paths and Rustdoc/module paths; do not invent rendered Rustdoc URLs.
3. Identify source-of-record for each fact, not one blanket authority.
4. Separate draft, review, blocked, completed, sent, stored, and converted outcomes.
5. Include blocked actions even when the page is about positive results.
6. Include exact outcome fields needed to prove actual minutes saved.
7. Explain how source refs, issue refs, opportunity refs, or payment references trace back to evidence.
8. Say explicitly when the entity is evidence only and cannot mutate provider, payment, schedule, safety, or customer-message state.
