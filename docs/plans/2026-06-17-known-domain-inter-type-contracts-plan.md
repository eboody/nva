# Known Domain and Inter-Type Contracts Implementation Plan

> **For Hermes:** Use subagent-driven-development or a serialized Kanban board to implement this plan task-by-task. The `/home/eran/code/nva` checkout is shared; dispatch at most one mutating worker at a time unless explicit isolated worktrees are created and verified.

**Goal:** Model the stable NVA/pet-resort concepts we already know will exist, and define the inter-type contracts that make source evidence, analytics/read models, operational workflows, policies, staff actions, and future labor-cost validators compose without provider leakage or primitive soup.

**Architecture:** Keep `domain` as the semantic truth layer. External systems enter through source/adapters, promote into source-agnostic evidence, project into analytics/read models, and then feed deterministic policy/workflow validators. The plan deliberately models contract surfaces and relationships before adding automation: unknown BI/source facts become assumptions or `data_quality::Issue`s; known service and workflow concepts become module-owned semantic types; inter-type contracts are explicit bridge structs/functions rather than hidden `.into()` chains.

**Tech Stack:** Rust workspace; `domain` crate first; `app` only after domain contracts are stable; existing `bon`, `serde`, `thiserror`, `chrono`, `nutype`, and `statum` where phase legality needs compile-time API guidance. Existing docs and tests remain the authority surface: `docs/architecture/domain-contract-skeleton.md`, `docs/integrations/gingr/adapter-boundary-and-labor-source-expansion.md`, `docs/integrations/gingr/bi-read-model-contract.md`, and `docs/discovery/bi-question-decision-rubric.md`.

---

## 1. Current inspected baseline

The repository already has a strong semantic foundation:

- `domain/src/source.rs` owns source systems, provenance, source record identity, reservation snapshots, Gingr adapter snapshots, and promotion into source-agnostic reservation facts.
- `domain/src/data_quality.rs` owns source-agnostic field paths and data-quality issues.
- `domain/src/analytics.rs` owns `analytics::stay::Fact::project_from_source_reservation(...)`.
- `domain/src/operations.rs` owns portfolio/service/metric/labor context primitives and operating vocabulary.
- `domain/src/staff.rs`, `daily_brief.rs`, `workflow.rs`, `policy.rs`, `entities.rs`, and service modules already contain early contracts for tasks, briefings, status updates, automation/review gates, reservations/customers/pets/locations, boarding/daycare/grooming/training/retail.
- Source-boundary docs now preserve the desired chain:

```text
Gingr DTO / endpoint response
-> source::gingr::reservation::Snapshot
-> source::reservation::Snapshot
-> analytics::stay::Fact
-> future workflow-validator evidence
```

The next work should not add random fields. It should produce a coherent type atlas and bridge contracts for the things we know will exist.

## 2. Design principles for this plan

1. **Known concepts become named types, not comments.** If NVA will definitely need it—staff shift, paid labor, capacity unit, operational day, service demand, manager review, workflow evidence—it gets a semantic module/type once a first test needs it.
2. **Inter-type contracts are first-class.** A projection from source evidence to analytics, from analytics to validator input, or from validator decision to staff action should have a named API and tests.
3. **Sources do not become the domain.** Gingr, BI exports, timeclock, payroll, scheduling, POS, capacity inventory, and manual imports are evidence systems. They feed contracts; they do not own business truth.
4. **Analytics facts are regenerable read models.** They may denormalize for reporting and comparison, but must point back to source provenance and data-quality issues.
5. **Workflow validators consume evidence bundles.** They should not inspect raw provider statuses or BI table names.
6. **Policy decides automation safety.** Agents may draft/extract/summarize/recommend; deterministic policy validators decide whether action is allowed, blocked, or manager-reviewed.
7. **Statum only where phase-specific APIs matter.** Use runtime enums for early contracts; introduce typestate only when method legality changes by phase.
8. **BI answers drive expansion priority.** Unknown scheduling/timeclock/payroll/capacity details are named as future source families and assumptions until artifacts prove grain, identity, and provenance.

## 3. Contract atlas: things we know will exist

### 3.1 Source evidence layer

Current implemented roots:

```rust
source::System
source::Provenance
source::RecordRef
source::record::{Id, RelatedId, Role}
source::reservation::{Snapshot, Status, Assumption}
source::gingr::{Provenance, ProviderRecordId, ProviderStatus}
source::gingr::reservation::Snapshot
```

Known future source families, already represented in `source::System`:

```rust
source::System::{
    Gingr,
    BusinessIntelligence,
    LaborScheduling,
    Timeclock,
    Payroll,
    CapacityInventory,
    PointOfSale,
    ManualImport,
}
```

Do **not** create modules for every source family immediately. Add a concrete source submodule only when a fixture/artifact proves:

- row/event grain;
- stable source record identity;
- related-record roles;
- provenance fields;
- first projection need;
- data-quality issue behavior.

Known inter-type contracts:

```text
source-specific snapshot -> source-agnostic snapshot
source snapshot -> data_quality::Issue
source snapshot -> analytics fact
source snapshot + analytics fact + data_quality issues -> workflow evidence
```

### 3.2 Data-quality layer

Current implemented roots:

```rust
data_quality::FieldPath
data_quality::FieldSegment
data_quality::ReservationField
data_quality::StayField
data_quality::SourceField
data_quality::Kind
data_quality::Severity
data_quality::ResolutionStatus
data_quality::Issue
```

Known expansion needs:

- `data_quality::FieldPath` should grow only when a new modeled evidence family exists: labor shift, time punch, payroll cost, capacity unit, POS/payment, customer/pet identity, staff identity.
- `data_quality::Kind` should include issue classes only when tests prove real behavior: missing source field, ambiguous identity, conflicting timestamps, unknown source status, stale source evidence, impossible join, disputed BI metric, untrusted manual import, sensitive payload quarantined.
- Issues must be able to block workflows, warn analytics, or route to manager/BI review.

Inter-type contract:

```rust
source::reservation::Snapshot::data_quality_issues(...) -> Vec<data_quality::Issue>
analytics::projection(...) -> Result<Fact, Vec<data_quality::Issue>>
workflow_validator.evaluate(EvidenceBundle) -> Decision carrying issue refs
```

### 3.3 Analytics/read-model layer

Current implemented root:

```rust
analytics::stay::Fact
analytics::ProjectionVersion
```

Known analytics facts we will likely need, in safe modeling order:

1. `analytics::stay::Fact` — implemented first, source reservation -> stay read model.
2. `analytics::demand::Fact` or `analytics::service_demand::Fact` — service-line/location/day demand derived from stay/reservation/service facts.
3. `analytics::capacity::Fact` — capacity inventory or occupancy pressure once capacity artifacts exist.
4. `analytics::labor::scheduled::Fact` — scheduled coverage by operational day/location/role once scheduling artifacts exist.
5. `analytics::labor::actual::Fact` — timeclock/actual paid coverage once timeclock artifacts exist.
6. `analytics::labor::cost::Fact` — wage/payroll cost attribution once payroll or wage-rate artifacts exist.
7. `analytics::variance::Fact` — scheduled-vs-actual, demand-vs-coverage, capacity-vs-demand, or cost-vs-revenue comparisons.

Do not add all modules now. The first concrete next analytics module should be the one that can be fixture-proven from available artifacts.

Inter-type contracts:

```text
source::reservation::Snapshot -> analytics::stay::Fact
analytics::stay::Fact + source::capacity::* -> analytics::capacity::Fact
analytics::stay::Fact + source::labor_schedule::* -> analytics::labor::scheduled::Fact
analytics::labor::scheduled::Fact + source::timeclock::* -> analytics::labor::actual::Fact
analytics::labor::actual::Fact + source::payroll::* -> analytics::labor::cost::Fact
analytics facts + data_quality issues -> workflow validator evidence
```

### 3.4 Operational day/location/service layer

Known cross-cutting concepts:

```rust
location::Name / entities::LocationId
operations::pet_resort::Brand
operations::ServiceOffering
boarding::*
daycare::*
grooming::*
training::*
retail::*
daily_brief::ResortOperatingDay
daily_brief::Resort
```

Missing contract pressure: source evidence and analytics facts need a shared operational index, but this should not become a generic `String` tuple. Candidate future module:

```rust
operations::operating_day::Key {
    location: entities::LocationId or source::record::Id,
    service_line: operations::ServiceLine,
    date: operations::OperatingDate,
}
```

Only add it when a test needs cross-fact joins by day/location/service.

Inter-type contract:

```text
source location/service/date evidence -> operations operating key
analytics facts carry operating key or typed unresolved-key issue
workflow/staff tasks reference operating key instead of raw report filters
```

### 3.5 Staff, scheduling, timeclock, and payroll layer

Current surfaces:

```rust
staff::Role
staff::Task
staff::completion_evidence::*
daily_brief::ScheduledStaffCount
daily_brief::LaborSnapshot
operations::operational::PainArea::LaborEfficiency
source::System::{LaborScheduling, Timeclock, Payroll}
```

Known future contract families:

```rust
source::labor_schedule::ShiftSnapshot
source::timeclock::PunchSnapshot
source::payroll::CostSnapshot
analytics::labor::scheduled::CoverageFact
analytics::labor::actual::WorkedTimeFact
analytics::labor::cost::LaborCostFact
```

Do not create these until discovery artifacts exist. But the plan should reserve the inter-type shape:

```text
scheduled shift evidence -> scheduled coverage fact
clock punch evidence -> actual worked coverage fact
payroll/wage evidence -> labor cost fact
coverage/cost facts + stay/demand facts -> labor pressure evidence
labor pressure evidence -> staff/manager workflow decision
```

Important distinction:

- `staff::Role` is domain/team-member capability language.
- `source::record::Role::Staff` is source record identity language.
- Future `labor::Role` or `staffing::CoverageRole` should be added only if schedules/payroll distinguish labor buckets that do not map cleanly to `staff::Role`.

### 3.6 Capacity and inventory layer

Current surfaces:

```rust
boarding::capacity::*
daycare::coverage::*
daily_brief::capacity::*
operations::CapacityMetric / capacity-related service contracts
source::System::CapacityInventory
```

Known future contracts:

```rust
source::capacity::UnitSnapshot       // kennel/run/room/group-play slot/service capacity evidence
analytics::capacity::OccupancyFact   // stay/demand joined to known capacity
analytics::capacity::PressureFact    // saturation, overbooking risk, staffing pressure
```

Inter-type contract:

```text
capacity source evidence + stay/demand fact -> occupancy/pressure fact
pressure fact + policy thresholds -> workflow/staff recommendation
```

Guardrail: Gingr reservation counts are demand/occupancy evidence, not capacity truth. Capacity truth needs capacity inventory artifacts or explicit assumptions.

### 3.7 Customer, pet, care, temperament, vaccine, and safety layer

Current surfaces:

```rust
entities::{Customer, Pet, CareProfile, TemperamentProfile}
care::*
temperament::*
vaccine::*
policy::{ReviewGate, PlayEligibility, automation::*}
daycare::eligibility::*
```

Known inter-type contracts:

```text
source customer/pet/profile evidence -> semantic customer/pet/care/temperament evidence
care/temperament/vaccine evidence -> policy decision/review gate
daycare eligibility evidence -> daycare assignment/front-desk decision
policy/review decision -> workflow recommended action or staff task
```

Guardrail: safety decisions cannot be inferred from missing data. Missing vaccine/care/temperament evidence creates review/blocking issues, not happy-path eligibility.

### 3.8 Workflow, policy, agent, and tool-action layer

Current surfaces:

```rust
workflow::{Event, Result, RecommendedAction, status_update::*}
policy::{ReviewGate, automation::*}
agent::Spec
tools::*
staff::Task
```

Known future contract shape:

```rust
workflow::EvidenceBundle       // named bundle of analytics facts, source refs, issue refs, policy context
workflow::Validator            // deterministic evaluator per workflow family
workflow::Decision             // allow / block / manager review / BI review / staff task
policy::AutomationDecision     // explicit action-safety decision
```

Do not add a generic bundle immediately if one specific validator will do. Add narrowly named evidence bundles first, for example:

```rust
workflow::labor::StaffingPressureEvidence
workflow::reservation::ReadinessEvidence
workflow::daycare::GroupPlayReadinessEvidence
```

Inter-type contract:

```text
analytics/source/data-quality evidence -> validator evidence bundle
validator evidence bundle -> workflow decision
workflow decision -> staff task, message draft, status update draft, or manager review gate
tool/action ports accept drafts only after policy decision permits them
```

## 4. Target public path map

Preserve semantic module paths at call sites. Preferred public surfaces:

```rust
source::reservation::Snapshot
analytics::stay::Fact
analytics::labor::scheduled::CoverageFact      // future
analytics::labor::actual::WorkedTimeFact       // future
analytics::labor::cost::LaborCostFact          // future
analytics::capacity::PressureFact              // future
operations::operating_day::Key                 // future if cross-fact joins need it
workflow::labor::StaffingPressureEvidence      // future
workflow::labor::Decision                      // future
staff::Task
policy::ReviewGate
```

Avoid flat aliases like:

```rust
StayFact
LaborFact
CapacityFact
WorkflowEvidence
```

unless the importing module already supplies the missing semantic context.

## 5. Implementation sequence

### Phase 0: Plan acceptance and stale-board reconciliation

**Objective:** Preserve this plan as the new canonical architecture target before new code work.

**Files:**
- Create: `docs/plans/2026-06-17-known-domain-inter-type-contracts-plan.md`
- Later update: `docs/architecture/domain-contract-skeleton.md`
- Later update: `docs/integrations/gingr/adapter-boundary-and-labor-source-expansion.md`

**Verification:**

```bash
git diff --check
git status --short --branch
```

Expected: docs-only diff; no code gates required unless examples are changed.

### Phase 1: Type atlas / contract map artifact

**Objective:** Add a durable architecture document that maps known entities, evidence types, analytics facts, validators, and action outputs.

**Files:**
- Create: `docs/architecture/known-domain-inter-type-contracts.md`
- Modify: `docs/architecture/domain-contract-skeleton.md` to link it.

**Acceptance criteria:**

- Names every currently implemented type surface that is part of the source -> analytics -> workflow path.
- Names future type families without pretending they are implemented.
- For each future family, records its first required proof artifact before code may be added.
- Distinguishes `source::record::Role`, `staff::Role`, service-line roles, and future labor coverage roles.

**Verification:**

```bash
git diff --check
```

### Phase 2: Inter-type bridge tests for existing implemented path

**Objective:** Lock the current chain with tests that read like contract documentation.

**Files:**
- Modify: `domain/tests/reservation_source_contracts.rs`
- Possibly create: `domain/tests/inter_type_contracts.rs` if the existing file becomes too broad.

**Test cases:**

1. `gingr_snapshot_promotes_to_source_reservation_without_leaking_provider_types`
2. `source_reservation_projects_to_stay_fact_with_provenance_and_projection_version`
3. `blocking_data_quality_issues_prevent_stay_fact_projection`
4. `stay_fact_remains_regenerable_from_source_reservation_snapshot`

**Commands:**

```bash
cargo test -p domain --test reservation_source_contracts
cargo fmt --check
cargo clippy -p domain --all-targets -- -D warnings
```

Expected: all pass; no new production concepts beyond current source/stay path.

### Phase 3: Operating-day/location/service key, only if cross-fact joins need it

**Objective:** Introduce the smallest shared key for day/location/service joins if the next model needs to compare demand, capacity, and labor.

**Files:**
- Create or modify: `domain/src/operations.rs` or `domain/src/operations/operating_day.rs` if `operations.rs` becomes too large.
- Test: `domain/tests/domain_quality_patterns.rs` or new `domain/tests/operations_inter_type_contracts.rs`.

**Candidate surface:**

```rust
operations::operating_day::Date
operations::operating_day::Key
operations::service_line::Line // only if existing service-line enums do not cover the join semantics
```

**Inter-type contract:**

```text
source reservation/stay evidence -> operating-day key candidate
analytics facts carry key or typed issue explaining why key cannot be formed
```

**Do not add:** capacity/labor calculations in this phase.

### Phase 4: Capacity evidence contract, gated by artifact/fixture

**Objective:** Model capacity source evidence once a capacity inventory artifact, policy doc, screenshot, or redacted fixture exists.

**Files:**
- Candidate code: `domain/src/source.rs` nested `source::capacity` or split file later.
- Candidate analytics: `domain/src/analytics.rs` nested `analytics::capacity` or split later.
- Tests: `domain/tests/capacity_source_contracts.rs`.

**Required precondition:** a non-secret artifact summary identifying capacity grain, such as room/run/unit, service line, location, date/effective period, and availability/constraint shape.

**Inter-type contract:**

```text
source::capacity::UnitSnapshot
+ analytics::stay::Fact / analytics::demand::Fact
-> analytics::capacity::OccupancyFact or PressureFact
```

**Data-quality examples:** missing location mapping, ambiguous service line, effective-date gap, capacity unit retired but referenced.

### Phase 5: Scheduling/timeclock/payroll source contracts, artifact-first

**Objective:** Model labor evidence only after at least one scheduling/timeclock/payroll artifact exists.

**Files:**
- Candidate source modules: `source::labor_schedule`, `source::timeclock`, `source::payroll`.
- Candidate analytics modules: `analytics::labor::scheduled`, `analytics::labor::actual`, `analytics::labor::cost`.
- Tests: `domain/tests/labor_source_contracts.rs`.

**Required preconditions:**

- Scheduling: shift grain, staff/role/location mapping, scheduled start/end, status/callout/swap shape.
- Timeclock: punch grain, staff identity, location/role context, paid/unpaid break behavior, edit/approval provenance.
- Payroll: pay period grain, wage/cost attribution, role/location mapping, what can legally/ethically be modeled.

**Inter-type contracts:**

```text
labor_schedule shift -> scheduled coverage fact
timeclock punch -> actual worked time fact
payroll cost -> labor cost fact
scheduled + actual -> coverage variance fact
labor cost + stay/demand/capacity -> labor pressure fact
```

**Guardrail:** no wage/cost inference from schedules alone unless explicitly modeled as estimate/projection with confidence and review status.

### Phase 6: Workflow validator evidence bundles

**Objective:** Define the first validator input bundle that consumes existing facts without raw source/provider leakage.

**Candidate first validators:**

1. `workflow::reservation::ReadinessEvidence` — can reservation/stay facts safely drive an operational workflow?
2. `workflow::labor::StaffingPressureEvidence` — later, once labor/capacity facts exist.
3. `workflow::daycare::GroupPlayReadinessEvidence` — links care/temperament/vaccine facts to policy and front-desk decisions.

**Files:**
- Candidate: `domain/src/workflow.rs` nested module first.
- Tests: `domain/tests/workflow_evidence_contracts.rs`.

**Inter-type contract:**

```text
EvidenceBundle {
    source_refs,
    analytics_facts,
    data_quality_issues,
    policy_context,
}
-> workflow::Decision::{Allowed, Blocked, ManagerReviewRequired, BiReviewRequired, StaffTaskDrafted}
```

**Guardrail:** validators must not import `source::gingr::*` or BI table names.

### Phase 7: App-layer use-case contracts

**Objective:** Once domain evidence/decision contracts exist, define app-layer use cases that orchestrate repositories/adapters/tools.

**Files:**
- Candidate app modules under `app/src/`.
- Tests under `app/tests/`.

**Inter-type contract:**

```text
app use case pulls source snapshots/read-model facts
-> asks domain validator for decision
-> emits staff task/message/status-update draft
-> tool port executes only after policy allows
```

**Guardrail:** app layer wires use cases; it does not invent domain decisions.

### Phase 8: Final integration and documentation closeout

**Objective:** Keep the architecture understandable after code lands.

**Files:**
- Update `docs/architecture/known-domain-inter-type-contracts.md` with implemented vs planned status.
- Update `docs/integrations/gingr/adapter-boundary-and-labor-source-expansion.md` if source boundaries change.
- Update `docs/discovery/bi-question-decision-rubric.md` if new artifacts alter task priority.

**Verification:**

```bash
cargo fmt --check
cargo test -p domain
cargo clippy -p domain --all-targets -- -D warnings
git diff --check
```

## 6. Kanban execution shape

Create a new board only after this plan is accepted. Suggested board slug:

```text
nva-known-domain-contract-map
```

Suggested serialized card graph:

1. **docs:** write `known-domain-inter-type-contracts.md` atlas from this plan.
2. **code:** strengthen existing reservation/source/stay bridge tests.
3. **code:** add operating-day/service key only if tests prove it is needed for the next cross-fact contract.
4. **docs:** create artifact checklist templates for capacity, schedule, timeclock, payroll, and BI/manual import sources.
5. **blocked/gated:** capacity source contract, blocked until artifact exists.
6. **blocked/gated:** labor scheduling source contract, blocked until artifact exists.
7. **blocked/gated:** timeclock source contract, blocked until artifact exists.
8. **blocked/gated:** payroll source contract, blocked until artifact exists.
9. **review:** semantic inter-type contract review and final docs closeout.

Keep artifact-gated cards blocked/scheduled rather than letting workers invent schemas.

## 7. Acceptance criteria for the whole plan

The plan is successful when:

- There is a canonical type atlas distinguishing implemented, planned, and artifact-gated contracts.
- Existing source -> data quality -> analytics bridge tests prove no Gingr leakage into analytics/workflow-facing surfaces.
- Every future known family has an explicit precondition before code can be added.
- Inter-type contracts are named and tested before automation consumes them.
- The next BI/source artifact can be classified quickly into: inventory, source contract, analytics projection, workflow validator, or labor/capacity model.

## 8. Explicit anti-goals

- Do not model all future source modules now without artifacts.
- Do not create placeholder crates for timeclock/payroll/scheduling/capacity.
- Do not add labor optimization fields to `analytics::stay::Fact`.
- Do not let `operations` become a catch-all for source facts, staff decisions, and workflow evidence.
- Do not hide source-family uncertainty behind `String`, `bool`, or untyped metadata.
- Do not build customer/member-facing or staff-action automation from raw source statuses.

## 9. Immediate next recommendation

Start with a docs-first board that produces `docs/architecture/known-domain-inter-type-contracts.md` and then one focused code card to strengthen the existing reservation/source/stay inter-type tests. Park capacity/labor/payroll/timeclock cards behind artifact gates until BI/source answers exist.
