# Labor-cost reduction crosswalk

Purpose: keep repo work anchored to labor-cost reduction across the 170-site NVA Pet Resorts operating context. This is a product/architecture crosswalk, not a generic AI backlog: it maps the major labor-cost drivers from `nva-pet-resorts-ai-context.md` to current repo surfaces, future source/read-model needs, workflow automation loops, and review-gated agent loops.

The operating thesis is simple: labor cost falls when managers and front-desk teams stop rediscovering source facts, reconciling dashboards by hand, rewriting repetitive drafts, and reworking ambiguous records. The deterministic app still owns facts, policy, workflow state, review gates, audit, outcome capture, and every external side effect. Agents only consume app-owned context packets and submit reviewable drafts/recommendations.

## Boundary legend

| Boundary | What belongs there | Current repo surfaces |
| --- | --- | --- |
| Operational source systems | Systems of record and provider-specific facts. These answer “what did Gingr / a scheduling tool / BI export / HR system say?” but do not become the domain model. | `integrations/gingr/src/endpoint/*`, `integrations/gingr/src/dto/*`, `integrations/gingr/src/mapping/*`, `domain::source`, `domain::analytics` |
| BI / read models | Normalized, source-grounded projections for daily operations, staffing, utilization, exceptions, and portfolio comparison. These should keep source refs and data-quality status. | `domain::analytics::{stay, service_demand}`, `domain::operations::operating_day`, `storage::operations`, future data-quality/read-model storage |
| Workflow automation | Deterministic app use cases that rank work, validate drafts, preserve policy gates, and record outcomes without live side effects by default. | `app::manager_daily_brief`, `app::booking_triage`, `app::checkout_completion`, `app::crm_retention`, `app::daily_update`, `app::tools` |
| Review-gated agent loops | Agent-readable context packets and draft submissions where every claim cites source evidence and every unsafe action stays blocked or human-approved. | `app::agents`, `app::manager_daily_brief::*`, `scripts/hermes-tools/*`, `docs/architecture/agent-app-infrastructure.md` |
| Outcome evidence | Staff/manager feedback that compares estimated vs actual work saved and turns labor savings into measured evidence. | `app::manager_daily_brief::OutcomeRecord`, `storage::operations::ManagerDailyBriefOutcomeRecord`, `docs/design/manager-daily-brief-measurable-labor-loop.md` |

## Driver-to-surface crosswalk

| Labor-cost driver from context pack | Operational source systems to read | BI / read-model need | Current domain/app/storage surfaces | Workflow automation target | Review-gated agent loop | Measurement and safety boundary |
| --- | --- | --- | --- | --- | --- | --- |
| Scheduling, payroll, and labor-to-revenue risk | Gingr reservations/service demand, timeclock reports, scheduling/labor tool, payroll/labor cost export, BI revenue projection. Current concrete adapter clue: `integrations/gingr::endpoint::labor_ops::TimeclockReport`. | Location + service-line + operating-day projection joining demand units, scheduled labor, staff roles, occupancy/capacity, revenue or labor-budget target, and data-quality flags. | `domain::operations::operating_day::Key`, `domain::analytics::service_demand::Fact`, `domain::daily_brief::{LaborSnapshot, LaborRisk}`, `app::manager_daily_brief::{BriefActionKind::ReviewDemandAgainstStaffingPlan, LaborImpactEstimate}`. | Manager Daily Brief demand-versus-staffing action queue; future regional exception report for sites off labor plan. | Agent summarizes evidence and ranks internal actions only. It cannot change staff schedules or payroll records. | Minutes avoided in morning dashboard reconciliation; actual manager minutes captured in outcome. Schedule edits remain blocked unless a separate app-owned approval/write contract exists. |
| Front-desk call/inbox load and repetitive customer questions | Customer portal/reservation facts, policy/SOP source, contact permissions, messages/call logs, website lead forms. | Queue of safe response opportunities and policy-answer coverage with source refs, consent/channel status, and escalation flags. | `app::booking_triage`, `app::daily_update`, `app::tools::messaging`, `domain::policy::ReviewGate`, `domain::message`, `domain::workflow`. | Draft-only customer response / booking-triage packets that pre-fill staff-visible rationale and missing-information requests. | Agent drafts or classifies only. Customer sends, medical/safety advice, refunds, incidents, and aggressive-behavior judgments stay review-gated. | Handle-time reduction and deflected repeat lookups; no customer send without deterministic app validation and required human approval. |
| Reservation, capacity, check-in/check-out bottlenecks | Reservations, stays, occupancy, room/suite inventory, check-in/out status, staff handoff notes, add-on requests. | Operating-day stay/checkout exception projection with unresolved handoffs, source status, care notes, and source ambiguity. | `domain::source::reservation::Snapshot`, `domain::analytics::stay::Fact`, `app::checkout_completion`, `app::manager_daily_brief::BriefActionKind::ResolveCheckoutException`, `domain::boarding`, `domain::daycare`. | Checkout completion and daily brief exception queue that reduces open-stay audits and handoff rediscovery. | Agent ranks exceptions and drafts internal tasks with evidence. It cannot mutate provider/PMS status or send customer updates. | Minutes avoided in checkout audit; source-fact-wrong outcomes tracked. Provider/PMS mutation remains blocked. |
| Data fragmentation and source hygiene across brands/sites | Gingr, local exports, BI warehouse, duplicate customer/pet records, vaccination/document status, service catalogs, notes. | Data-quality hygiene queue grouped by location, entity, field path, severity, source system, and repeated rework impact. | `domain::data_quality::{Issue, Kind, FieldPath, Severity}`, `domain::source::{RecordRef, Provenance}`, `domain::analytics::*`, readiness review’s recommended second workflow. | Data-quality hygiene workflow: investigate missing evidence, reconcile duplicate candidates, complete pet profile fields, review stale vaccine/source freshness, normalize ambiguous service-line names. | Agent can summarize issue clusters and draft internal correction tasks only; it cannot hide/auto-resolve ambiguity or write back to provider records. | Minutes avoided in repeated source reconciliation and downstream rework; data-quality ambiguity remains manager-visible until reviewed. |
| Manager coaching, daily/weekly reporting, and regional visibility gaps | BI dashboards, operations KPIs, incidents, reviews, staffing data, completed workflow outcomes. | Portfolio/regional exception read model: off-plan sites with evidence, trend, recommended next action, and comparable peers. | Current: `domain::daily_brief`, `domain::operations`, `domain::reputation`, `app::manager_daily_brief`; future: regional exception workflow over outcome/read models. | Regional Ops Exception loop: “which locations need attention, why, and what action should a regional leader review?” | Agent turns app-produced exception packets into narrative summaries and recommended review queues. It cannot decide discipline, staffing changes, or policy exceptions. | Regional leader review minutes avoided; follow-up outcomes recorded. Personnel/safety actions stay human-owned. |
| Sales, lead conversion, retention, and grooming rebooking | Leads/forms/calls, completed stays, grooming history, package/membership history, contact consent, campaign source, reservations. | Customer/pet opportunity projection with source-grounded reason code, consent, due cadence, slot utilization, and suppression/eligibility status. | `domain::lead`, `app::crm_retention`, `domain::grooming::{rebooking, appointment}`, `domain::retail::recommendation`, `app::manager_daily_brief::BriefActionKind::ApproveRetentionFollowUpDraft`. | Retention/grooming rebooking queue that prioritizes safe, consented follow-up candidates and records staff disposition. | Agent drafts personalized follow-up language for review. No live message, discount, payment movement, or provider update without app-owned approval. | Minutes saved in queue prioritization and draft writing; revenue lift can be measured separately, but labor claim requires outcome capture. |
| Training consistency, SOP lookup, incident/safety documentation | SOP repository, incident records, staff training records, care/behavior notes, policy versions. | Policy/SOP context packets with versioned source refs and escalation categories. | `domain::agent`, `domain::policy`, `domain::incident`, `domain::care`, `domain::temperament`, `app::agents`, future SOP-assistant app contract. | SOP assistant and incident-drafting loop for internal guidance and documentation completeness. | Agent may answer “what should staff review/document?” using versioned SOP context; incidents, injuries, aggression, medical/vaccine ambiguity, and compliance issues stay review-gated. | Reduced manager lookup/coaching time and better documentation completeness; no autonomous safety decision or customer-facing incident response. |
| Post-stay updates, Pawgress reports, and reputation/review work | Staff care notes, photos/media refs, stay outcomes, review platforms, customer history, brand tone rules. | Draftable report/reputation queue with omitted sensitive facts, internal flags, review severity, and contact/channel status. | `app::daily_update`, `domain::reputation`, `domain::message`, `domain::document`, `app::tools::{media, messaging}`. | Daily update / Pawgress draft and review response draft workflow. | Agent transforms terse notes into staff-reviewed drafts and flags sensitive/negative notes. No unreviewed customer send. | Draft-writing minutes saved; review outcomes and suppression reasons recorded. Sensitive facts remain internal unless approved. |

## Current strongest repo support

1. Manager Daily Brief is the most complete measurable labor loop.
   - Source/read model: `domain::analytics::service_demand::Fact` and source refs/data-quality issues.
   - Workflow: `app::manager_daily_brief` ranks demand/staffing, checkout, retention, and data-quality actions.
   - Outcome: `app::manager_daily_brief::OutcomeRecord` plus `storage::operations` outcome records capture estimated vs actual minutes.
   - Runtime/agent bridge: `scripts/hermes-tools/get_manager_daily_brief_context`, `submit_manager_daily_brief_draft`, and `record_manager_daily_brief_outcome` exercise app-owned surfaces.

2. Source hygiene is already modeled as a reusable safety rail.
   - `domain::data_quality::Issue` and `domain::source::RecordRef` prevent missing/ambiguous data from being hidden.
   - The readiness review recommends data-quality hygiene as the second workflow because it reduces repeated re-checking and strengthens every later loop.

3. Customer/retention/daily-update draft workflows already have the right review posture.
   - `app::crm_retention` and `app::daily_update` draft and evaluate; they do not own live sends.
   - `app::tools` keeps external providers, messaging, payments, documents, media, Hermes, and reservation systems behind ports.

## Top 3 next labor loops

### 1. Data-quality hygiene labor loop

Measurable outcome: minutes avoided in manual source reconciliation and repeated downstream rework per location/day.

Why next: The 170-site portfolio will not get reliable labor automation if missing customer/pet/reservation/service facts, duplicate records, stale vaccine evidence, vague notes, and inconsistent service names remain invisible until a manager hits them manually.

Minimum contract:

- `GET /agent/context/data-quality-hygiene?location_id=...&operating_day=...`
- `POST /agent/drafts/data-quality-hygiene`
- `POST /data-quality-hygiene/actions/{action_id}/outcome`
- Action kinds: investigate missing source evidence, reconcile duplicate customer/pet candidate, complete missing pet profile fields, review stale vaccination/source freshness, normalize ambiguous service-line naming.
- Blocked actions: customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, hiding or auto-resolving source ambiguity.

Current surfaces to extend: `domain::data_quality`, `domain::source`, `domain::analytics`, new `app::data_quality_hygiene`, `storage::operations`, API/CLI/bridge shells after the app contract is proven.

### 2. Regional labor exception loop

Measurable outcome: regional leader minutes avoided in dashboard scanning and follow-up prioritization; number of off-plan sites reviewed with source-backed next action.

Why next after data quality: Manager Daily Brief operates at one location/day. Regional leaders need portfolio-level exception triage across labor risk, demand softness, utilization, incidents, reviews, and data hygiene without asking each GM to manually narrate the problem.

Minimum contract:

- Context packet groups sites by operating period, metric, variance, source refs, data-quality caveats, and existing manager outcomes.
- Draft packet recommends review queues only: “ask GM to review staffing plan,” “inspect checkout exception pattern,” “validate source hygiene,” etc.
- Outcome captures reviewed/deferred/wrong-source/completed plus actual minutes.

Current surfaces to extend: `domain::operations`, `domain::daily_brief`, `domain::reputation`, `storage::operations`, manager daily brief outcome storage, future BI/read models. Safety boundary: no staff discipline, schedule mutation, pricing, customer communication, or provider/PMS writes from the agent loop.

### 3. Grooming rebooking / retention follow-up loop

Measurable outcome: front-desk minutes avoided in finding safe follow-up candidates and drafting outreach; optional separate revenue metric for filled grooming capacity or retained customers.

Why third: It is labor-saving and commercially valuable, but it is closer to customer messaging, discounts/offers, and scheduling side effects. It should come after data-quality and regional exception patterns reinforce source refs, consent, and review outcomes.

Minimum contract:

- Context packet lists eligible pet/customer opportunities with grooming cadence, completed stay/grooming history, contact permission, channel, source refs, and suppression reasons.
- Draft packet proposes staff-reviewed follow-up language and internal tasks.
- Outcome captures sent-by-human/deferred/suppressed/wrong-source/converted plus actual minutes.

Current surfaces to extend: `app::crm_retention`, `domain::grooming::rebooking`, `domain::lead`, `domain::message`, `app::tools::messaging`, future slot-utilization read model. Safety boundary: no live customer send, discount/refund/payment movement, or schedule/provider mutation without explicit app-owned approval and human review.

## Source-of-record questions to keep explicit

These are discovery questions for NVA or fixture-safe implementation notes, not assumptions the repo should silently bake in:

- Do all 170 resorts use Gingr, and are they on common instances/configurations?
- Which system owns staff schedule, timeclock, payroll, and labor budget targets?
- Which system owns BI/read models and portfolio KPI definitions?
- Which customer messaging channels have consent/opt-out data available to app contracts?
- Which policy/SOP repository is versioned enough to produce source refs for an SOP assistant?
- Which outcome metrics will NVA accept as labor-cost evidence: estimated minutes, actual minutes, manager adoption, rework reduction, payroll variance, or a combination?

Until those are answered by source packets or fixtures, workflow docs should say “future source/read model” rather than inventing provider authority.

## Guardrails for future crosswalk updates

- Keep READMEs as navigation/wiki. Put executable API examples in Rustdoc/doctests.
- Promote only stable business concepts into `domain`; keep provider DTOs, storage records, HTTP shapes, and runtime shells at their boundary.
- Every agent loop needs: typed context packet, source refs, allowed/blocked actions, review gates, draft validation, outcome capture, and a labor metric.
- Labor-savings claims must be measured with outcome capture, not just asserted in prose.
- No live/member-facing actions, PMS/provider writes, schedule mutations, customer sends, payment/refund/discount movement, or secret-dependent behavior are authorized by this crosswalk.
