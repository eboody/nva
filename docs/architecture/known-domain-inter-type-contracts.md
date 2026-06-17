# Known domain inter-type contract atlas

Purpose: this atlas is the canonical map for known NVA pet-resort domain contracts that cross type families. It keeps `domain` as the semantic truth layer while external systems remain evidence sources. New code should use this document to decide where a concept belongs, which bridge contract must be named, and which artifact gate must be satisfied before adding scheduling, timeclock, payroll, capacity, POS, or BI-export models.

North star: reduce labor cost and operational drag through deterministic, reviewable contracts. Source payloads and BI exports may supply evidence; they do not own workflow truth, staff actions, or policy decisions.

## End-to-end contract chain

The current implemented chain is:

```text
source::gingr::reservation::Snapshot
-> source::reservation::Snapshot
-> Vec<data_quality::Issue>
-> analytics::stay::Fact
-> future workflow validator evidence bundle
-> policy/workflow decision
-> staff::Task, workflow::RecommendedAction, message/status-update draft, or manager/BI review
```

Every arrow should become a named bridge API with semantic tests before downstream automation consumes it. Avoid casual `.into()` chains when a conversion promotes source evidence, crosses a trust boundary, applies data-quality rules, or changes action-safety state.

Future workflow validators should treat the chain as evidence-in, decision-out. They consume already-promoted source snapshots, analytics/read-model facts, data-quality issues, and policy context; they do not read provider DTOs, BI table names, raw payroll exports, raw customer/staff prose, or LLM-written summaries as authority. A validator may assemble a review packet or draftable action request, but its deterministic output must stay separate from any natural-language copy later written for staff, managers, or customers.

## Stable type families NVA already knows it needs

### Source evidence

Implemented roots:

- `source::System`
- `source::Provenance`
- `source::RecordRef`
- `source::record::{Id, RelatedId, Role}`
- `source::reservation::{Snapshot, Status, OwnerPetRelationship, Assumption}`
- `source::gingr::{Provenance, ProviderRecordId, ProviderStatus}`
- `source::gingr::reservation::Snapshot`

Reserved source families in `source::System`:

- `BusinessIntelligence`
- `LaborScheduling`
- `Timeclock`
- `Payroll`
- `CapacityInventory`
- `PointOfSale`
- `ManualImport`

Contract rule: source-specific snapshots may translate provider vocabulary into source-agnostic snapshots. Domain, analytics, workflow, and policy validators must consume the source-agnostic shape, provenance, record refs, and data-quality issues rather than provider DTO/status/table names.

### Data quality

Implemented roots:

- `data_quality::FieldPath`
- `data_quality::{FieldSegment, ReservationField, StayField, SourceField}`
- `data_quality::{Kind, Severity, ResolutionStatus}`
- `data_quality::Issue`

Contract rule: missing, ambiguous, conflicting, unknown, stale, quarantined, or assumption-backed evidence becomes `data_quality::Issue`. Issues must be able to block workflows, warn analytics, route BI/manager review, and remain traceable to `source::Provenance` and `source::RecordRef`.

Field paths expand only after a modeled evidence family exists. Do not add generic string paths for future labor, capacity, POS, or BI facts.

### Analytics/read models

Implemented roots:

- `analytics::ProjectionVersion`
- `analytics::stay::{Id, Fact, DataQualityStatus}`

Known future fact families, artifact-gated:

- `analytics::demand::Fact` or `analytics::service_demand::Fact`
- `analytics::capacity::{OccupancyFact, PressureFact}`
- `analytics::labor::scheduled::CoverageFact`
- `analytics::labor::actual::WorkedTimeFact`
- `analytics::labor::cost::LaborCostFact`
- `analytics::variance::Fact`

Contract rule: analytics facts are regenerable read models. They can denormalize for comparison/reporting, but they must carry projection version, provenance/source refs, and quality status or linked issue refs. They should not become the place where provider vocabulary, policy approval, or staff actions are invented.

### Operations, service, location, and operating day

Implemented/current surfaces:

- `entities::{LocationId, CustomerId, PetId, ReservationId}`
- `operations::ServiceOffering` and related NVA operating vocabulary
- `daily_brief::{ResortOperatingDay, Resort, ScheduledStaffCount, LaborSnapshot}`
- service modules such as `boarding`, `daycare`, `grooming`, `training`, and `retail`

Known future pressure:

- `operations::operating_day::Key` for typed day/location/service joins once cross-fact joins need it.
- A service-line or coverage-line semantic type only if existing service modules cannot honestly express the join.

Contract rule: source location/service/date evidence should map to a typed operating key or produce an explicit issue explaining why it cannot. Workflow and staff tasks should reference the semantic key rather than raw report filters.

### Staff, scheduling, timeclock, and payroll

Implemented/current surfaces:

- `staff::{Role, Task}`
- `staff::completion_evidence::*`
- `daily_brief::ScheduledStaffCount`
- `daily_brief::LaborSnapshot`
- `operations::operational::PainArea::LaborEfficiency`
- `source::System::{LaborScheduling, Timeclock, Payroll}`

Known future families:

- `source::labor_schedule::ShiftSnapshot`
- `source::timeclock::PunchSnapshot`
- `source::payroll::CostSnapshot`
- `analytics::labor::scheduled::CoverageFact`
- `analytics::labor::actual::WorkedTimeFact`
- `analytics::labor::cost::LaborCostFact`

Contract rule: schedules prove planned coverage, timeclock proves actual paid work, and payroll proves cost attribution. Do not infer actual labor or wage cost from schedules alone unless the type says estimate/projection and carries confidence/review state.

Role distinction:

- `staff::Role` is domain/team-member capability language.
- `source::record::Role::Staff` is source-record identity language.
- A future `labor::Role` or `staffing::CoverageRole` is allowed only if scheduling/payroll labor buckets do not map cleanly to `staff::Role`.

### Capacity and inventory

Implemented/current surfaces:

- `boarding::capacity::*`
- `daycare::coverage::*`
- `daily_brief::capacity::*`
- `source::System::CapacityInventory`

Known future families:

- `source::capacity::UnitSnapshot`
- `analytics::capacity::OccupancyFact`
- `analytics::capacity::PressureFact`

Contract rule: reservations and stays are demand/occupancy evidence, not capacity truth. Capacity truth requires capacity-inventory artifacts or explicit assumptions. Capacity pressure can feed workflow recommendations only after policy thresholds are typed.

### Customer, pet, care, temperament, vaccine, and safety

Implemented/current surfaces:

- `entities::{Customer, Pet, CareProfile, TemperamentProfile}`
- `care::*`
- `temperament::*`
- `vaccine::*`
- `policy::{ReviewGate, PlayEligibility, automation::*}`
- `daycare::eligibility::*`

Contract rule: customer/pet/profile source evidence promotes into semantic care, temperament, vaccine, and safety evidence before policy sees it. Missing safety evidence creates blocking/review issues; it must not default to eligibility.

### Workflow, policy, agents, tools, and actions

Implemented/current surfaces:

- `workflow::{Event, EventId, Result, RecommendedAction, status_update::*}`
- `policy::{ReviewGate, automation::*}`
- `agent::Spec`
- `tools::*`
- `staff::Task`

Known future families:

- `workflow::reservation::ReadinessEvidence`
- `workflow::daycare::GroupPlayReadinessEvidence`
- `workflow::labor::StaffingPressureEvidence`
- `workflow::*::Decision` where each workflow family owns its decision vocabulary
- `policy::AutomationDecision` for action safety

Contract rule: validators consume named evidence bundles of analytics facts, source refs, issue refs, and policy context. Validators emit workflow decisions. Tool/action ports accept drafts only after policy permits them.

Evidence-bundle rule: start with narrowly owned bundles such as `workflow::reservation::ReadinessEvidence`, `workflow::daycare::GroupPlayReadinessEvidence`, or `workflow::labor::StaffingPressureEvidence` instead of a generic `WorkflowEvidence` bag. Each bundle should name the facts it accepts, the issue severities that can block or route review, the policy snapshot it evaluated, and the exact output decision vocabulary for that workflow family.

Decision/output rule: workflow decisions are deterministic domain values. They can say `allowed`, `blocked`, `manager_review_required`, `bi_review_required`, `staff_task_draft_required`, or a more specific workflow-owned variant. They must not contain final customer-facing prose, provider-write payloads, payment operations, or unreviewed staff-completion claims. LLM-written text belongs in a downstream draft artifact that cites the decision and evidence refs and remains review-gated according to `policy`.

## Bridge contracts to name and test

| Bridge | Owner | Allowed input | Output | Forbidden shortcut |
| --- | --- | --- | --- | --- |
| Provider snapshot promotion | `source::<provider>` | Provider DTO/snapshot vocabulary | `source::<family>::Snapshot` plus assumptions | Importing provider types into analytics/workflow |
| Source data-quality detection | `source::<family>` with `data_quality` values | Source-agnostic snapshot + detected timestamp | `Vec<data_quality::Issue>` | Treating missing/unknown evidence as happy path |
| Analytics projection | `analytics::<family>` | Source snapshot/read-model prerequisites + projection version | Regenerable fact or blocking issues | BI/report table names as fact schema |
| Operating-key formation | `operations::operating_day` when needed | Location/service/date evidence | Typed key or issue | Raw `(String, String, date)` tuples |
| Validator evidence assembly | `workflow::<family>` | Analytics facts + source refs + issue refs + policy context | Named evidence bundle | Generic `WorkflowEvidence` bag or provider status checks |
| Policy/action decision | `policy` + `workflow::<family>` | Evidence bundle and action context | Allow/block/manager-review/BI-review/staff-task decision | Agents/tools deciding safety from raw data |
| Staff/manager action draft | `workflow`/`staff`/`tools` | Policy-approved decision | Task, message draft, status-update draft, or tool request | Direct member-facing/provider mutation from analytics |

## Ownership boundaries and dependencies

Allowed dependency direction:

```text
source-specific adapter/module
-> source-agnostic source evidence
-> data_quality issues
-> analytics/read-model facts
-> workflow validator evidence
-> policy decisions
-> staff/workflow action drafts
-> app/tool ports
```

Boundary ownership:

- `source` owns source systems, provenance, record identity, source-specific quarantine, source-agnostic snapshots, and assumptions.
- `data_quality` owns issue vocabulary, field paths, severities, workflow-blocking state, and resolution state.
- `analytics` owns regenerable read models/projections and versioned facts.
- `operations` owns resort operating concepts, service/location/day join keys, capacity/labor metric language, and operational pain vocabulary.
- `entities`, service modules, `care`, `temperament`, and `vaccine` own business entities and care/safety facts.
- `workflow` owns deterministic validator input bundles, workflow decisions, result envelopes, status-update drafts, and recommended actions.
- `policy` owns review gates, automation safety vocabulary, denial/approval reasons, and thresholds once behavior depends on them.
- `staff` owns team-member role/task language and completion evidence.
- `tools` and the app layer own external execution ports and orchestration after the domain decision exists.

Forbidden dependency directions:

- `analytics` must not import `source::gingr::*`, BI table names, POS export names, or provider status values.
- Workflow validators must not inspect provider DTOs, raw BI filters, raw payroll fields, or source-specific statuses.
- `policy` must not infer safety from missing data, raw booleans, or absence of a review gate.
- `operations` must not become a junk drawer for source facts, staff decisions, workflow evidence, and analytics projections.
- `staff::Role`, `source::record::Role`, service-line roles, and future labor coverage roles must not be flattened into one enum without a proven shared concept.
- App/tool code must not invent domain decisions to make integration easier.

## Vocabulary leak guardrails

Gingr, BI, provider, export, POS, scheduling, timeclock, and payroll words are allowed in boundary modules and docs that describe artifact provenance. They are not allowed to become core validator language unless promoted into a source-agnostic semantic type.

Use this quarantine pattern:

```text
provider field/status/table/export name
-> source-specific validated value
-> source-agnostic snapshot/fact/issue
-> analytics/workflow/policy value
```

Examples:

- Gingr `ProviderStatus` may promote to `source::reservation::Status`; workflow readiness must not branch on the original provider string.
- BI export columns may identify a future artifact gate; they must not define analytics fact names until grain, identity, provenance, and issue behavior are known.
- Scheduling role names may map to `staff::Role` only when semantics align. Otherwise create an artifact-proven coverage-role type.
- Payroll cost fields may feed labor-cost facts only after legal/ethical modeling boundaries and cost-attribution grain are explicit.

## Workflow validator contract shape

Validators are deterministic contract code, not copywriting or tool-execution code. A future validator should have an explicit contract shaped like:

```text
workflow::<family>::Evidence {
    source_refs,
    analytics_facts,
    data_quality_issue_refs,
    policy_context,
    optional human-entered/reviewed facts,
}
-> workflow::<family>::Decision
-> staff/manager action draft or message/status/tool draft request
```

Evidence bundles may reference raw artifacts by governed source refs, but the validator-facing fields should be source-agnostic semantic facts. If a workflow needs a human-entered fact, OCR extraction, BI export row, or LLM summary, that item must first become reviewed evidence or a `data_quality::Issue`/assumption with provenance. LLM output can assist extraction, summarization, or copy drafting; it is not a source of policy truth.

Policy decisions should be small, auditable values with reasons and issue refs. Preferred output shape:

- decision kind: allowed, blocked, manager review, BI review, staff task draft, or workflow-specific denial/review variant;
- reasons: typed policy/data-quality reasons, not free-form copy as authority;
- evidence refs: stable links to source records, projection versions, issue ids, and policy snapshot ids;
- permitted draft classes: internal task, manager review packet, customer-message draft, status-update draft, provider/tool request draft;
- blocked actions: exact side effects that must not execute without approval.

LLM-written copy is a downstream draft artifact. It may render the deterministic decision for a human, but it must cite the decision/evidence refs, preserve redactions, avoid making final promises, and remain gated by `policy` for customer-facing, provider-write, payment, medical, behavior, incident, and sensitive staff/manager actions.

### Labor-cost reduction decision levers

The labor-cost north star should appear in validator design as explicit decision levers, not as hidden optimization prose. Useful levers include:

- demand-vs-scheduled-coverage variance: route manager review when scheduled labor materially exceeds/under-runs source-backed service demand;
- actual-vs-scheduled variance: flag missed punches, callouts, overtime risk, or schedule adherence only after timeclock evidence exists;
- labor-cost-per-stay/service/day: compare payroll/cost facts to demand/revenue/service facts only after payroll or approved cost artifacts exist;
- capacity pressure: distinguish low-demand labor trimming opportunities from high-pressure service/care risk;
- task deflection: draft staff/manager tasks that remove manual reconciliation, duplicate data entry, follow-up chasing, and status-check labor;
- review targeting: route only ambiguous/high-risk exceptions to managers while allowing routine internal summaries and safe draft preparation.

These levers should produce staff or manager action drafts such as "review tomorrow's daycare coverage variance," "resolve missing timeclock evidence before trusting actual labor cost," or "prepare customer follow-up draft for missing vaccine proof." They should not claim an automatic schedule change, wage decision, provider write, customer send, or live optimization exists until source artifacts, policy thresholds, and approved execution ports are implemented.

## Open artifact gates

Do not add concrete modules for these families until a non-secret artifact or fixture proves the listed facts.

### Scheduling

Required proof:

- shift grain and stable identity;
- staff/role/location mapping;
- scheduled start/end and timezone behavior;
- shift status, callout, swap, or cancellation shape;
- provenance fields and source record refs;
- first data-quality issue behavior.

First likely bridge: `source::labor_schedule::ShiftSnapshot -> analytics::labor::scheduled::CoverageFact`.

### Timeclock

Required proof:

- punch/event grain and stable identity;
- staff identity mapping;
- location/role context;
- paid/unpaid break behavior;
- edit/approval provenance;
- first data-quality issue behavior.

First likely bridge: `source::timeclock::PunchSnapshot -> analytics::labor::actual::WorkedTimeFact`.

### Payroll

Required proof:

- pay-period and cost-attribution grain;
- wage/rate/cost fields that may legally and ethically be modeled;
- staff/role/location mapping;
- adjustment/overtime/tip/commission behavior if present;
- provenance and sensitive-payload quarantine rules.

First likely bridge: `source::payroll::CostSnapshot -> analytics::labor::cost::LaborCostFact`.

### Capacity

Required proof:

- unit or inventory grain: kennel/run/room/group-play slot/service capacity;
- location and service-line mapping;
- effective dates, retirement, restrictions, or availability shape;
- relationship to reservation/stay demand;
- first data-quality issue behavior.

First likely bridge: `source::capacity::UnitSnapshot + analytics::stay::Fact -> analytics::capacity::OccupancyFact/PressureFact`.

### POS

Required proof:

- transaction/line/payment grain and identity;
- service/product/category mapping;
- refunds/voids/discounts/taxes/tips behavior;
- relationship to reservation/customer/pet/location evidence;
- provenance and financial sensitivity boundaries.

First likely bridge: `source::pos::*Snapshot -> analytics::revenue::*Fact` or a more specific service/product fact once reporting need is known.

### BI exports and manual imports

Required proof:

- export/report purpose and grain;
- stable row identity or derivation key;
- source systems behind the BI layer if known;
- refresh cadence and staleness behavior;
- whether the export is evidence, assumption, reconciliation aid, or generated read model;
- fields that should create `data_quality::Issue` instead of being trusted.

First likely bridge: BI/manual evidence becomes a source-family snapshot or an assumption-bearing reconciliation artifact. It should not bypass source/data-quality contracts into workflow validators.

## Implementation posture

- Add the smallest type family only when a test or artifact needs it.
- Prefer runtime enums for early contracts; introduce `statum` only when phase-specific method legality matters.
- Keep public paths semantic: `analytics::stay::Fact`, `workflow::labor::StaffingPressureEvidence`, `operations::operating_day::Key`.
- Avoid flat aliases such as `StayFact`, `LaborFact`, `CapacityFact`, or generic `WorkflowEvidence` unless the importing module already supplies the missing context.
- Keep README-level docs concise; update this atlas when a contract graduates from planned to implemented.
