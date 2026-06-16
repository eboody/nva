# PetSuites service-domain implementation review

Final review scope: service-domain maps, implication specs, and Rust increments created by the PetSuites/NVA service-domain board.

Review date: 2026-06-10

## Verdict

GO for the NVA AI-engineer conversation and for continued foundation work.

NO-GO for live autonomous/member-facing PetSuites operations until the remaining adapter, approval-event, repository, and execution-gate follow-ups below are implemented and separately reviewed.

The board successfully produced a semantic service-domain foundation: every requested core service line has a map, every operational implication has a document artifact, and the Rust domain now has typed contracts/tests for all five service lines. The strongest near-term use is as an interview/conversation artifact showing how to translate PetSuites/NVA operations into typed, auditable AI-assistive workflows without pretending the system is already production automation.

## Evidence reviewed

- Service maps present:
  - `docs/domain/petsuites/boarding/service-domain-map.md`
  - `docs/domain/petsuites/daycare/service-domain-map.md`
  - `docs/domain/petsuites/grooming/service-domain-map.md`
  - `docs/domain/petsuites/training/service-domain-map.md`
  - `docs/domain/petsuites/retail/service-domain-map.md`
- Implication artifacts present:
  - Boarding: 9 docs covering capacity, room/suite availability, holiday spikes, check-in/out, pet profile requirements, medication/feeding/behavior notes, staff handoffs, payment/deposit handling, and upsells.
  - Daycare: 8 docs covering temperament eligibility, group assignment, staff ratios, incident tracking, health/behavior notes, recurring attendance, package opportunities, and front-desk throughput.
  - Grooming: 7 docs covering groomer calendar optimization, breed/coat estimates, no-show/cancellation, 2-8 week rebooking cadence, cross-sell after daycare/boarding, reminders, and service history.
  - Training: 7 docs covering high-value upsells, progress reporting, trainer availability, curriculum tracking, parent follow-up, outcome documentation, and packages/recurring engagement.
  - Retail/partner products: 5 docs covering POS sales, inventory, recommendation workflows, personalized upsells, and supply-chain/reorder tracking.
- Rust domain implementation reviewed:
  - `domain/src/operations.rs`
  - `domain/tests/petsuites_core_service_contracts.rs`
  - `domain/tests/domain_quality_patterns.rs`
- Storage/integration evidence reviewed:
  - `storage/src/operations.rs`
  - `storage/tests/core_service_contract_storage.rs`
  - `storage/tests/operations_storage_contracts.rs`
  - `integrations/gingr/src/endpoint/catalog.rs`
  - `integrations/gingr/tests/endpoint_contracts.rs`
- Verification commands run locally:
  - `rustup default stable` because this worker environment had no default toolchain configured.
  - `cargo fmt --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

All Rust verification gates passed. The full workspace test run reported 8 domain unit tests, 33 domain quality-pattern tests, 36 PetSuites core-service contract tests, 24 Gingr integration tests, 7 storage tests, and doc-tests all passing. Clippy completed with `-D warnings` successfully.

`git status`/diff evidence is unavailable because `/home/eran/code/nva` is not a git repository in this runtime.

## Requirement-by-requirement review

### 1. Every core service has a service-domain map

Pass.

The five requested service lines have dedicated service-domain maps under `docs/domain/petsuites/`:

| Service | Map | Review note |
| --- | --- | --- |
| Boarding | `boarding/service-domain-map.md` | Strongest and broadest map. Clearly separates overnight stay, accommodation, capacity, care, deposit/cancellation, handoff, upsell, Pawgress/report, and approval boundaries. |
| Daycare | `daycare/service-domain-map.md` | Good semantic separation between dog group play, individual day boarding, hybrid play+room, and cat enrichment. Avoids collapsing eligibility into booleans. |
| Grooming | `grooming/service-domain-map.md` | Covers menu, calendar, breed/coat estimates, no-show, rebooking cadence, reminders, history, and cross-sell with explicit automation limits. |
| Training | `training/service-domain-map.md` | Correctly treats training as a core service line rather than only a boarding/daycare add-on. Models curriculum, trainer availability, progress, outcomes, and parent follow-up. |
| Retail / partner products | `retail/service-domain-map.md` | Correctly models Virbac/Purina/in-house diet as review-gated product/inventory/POS/recommendation contracts, not as free-text upsells. |

### 2. Every listed operational implication has an artifact and contract support or documented follow-up

Pass with expected foundation-stage follow-ups.

Every listed implication has a doc artifact. The Rust implementation covers a representative deterministic contract slice for each service line and documents wider follow-ups in the maps/implication specs. No implication is missing outright.

Implemented Rust contract support now includes:

- Boarding:
  - `operations::boarding::Contract` in `CoreServiceContracts`.
  - Typed accommodation/capacity modules: `boarding::accommodation::{Kind, Preference}` and `boarding::capacity::{Snapshot, NightlySegmentSnapshot, Request, Decision, Policy}`.
  - Deposit readiness policy with review gate for due-at-booking collection.
  - Care policy that blocks missing feeding instructions and medication-review requirements.
  - Upsell policy that recommends exit bath only when eligibility and care-safety boundaries allow it.
- Daycare:
  - `operations::daycare::Contract`, `ServiceVariant`, `CareMode`, staff/pet ratio scalars, incident policy, coverage policy, eligibility evidence/decisions, assignment policy, recurring attendance, package opportunity, and front-desk readiness.
- Grooming:
  - `operations::grooming::Contract`, calendar/no-show/rebooking/reminder/history fields, duration-estimate policy, ordinary 2-8 week cadence, no-show/rebooking gates, consent-gated reminder plan, and separated style notes vs care/medical handling references.
- Training:
  - `operations::training::Contract`, enrollment/progress/outcome/package/follow-up vocabulary, trainer availability decisions, parent-facing approval gates, outcome evidence requirements, package ledger, and follow-up planning.
- Retail/partner products:
  - `operations::retail::Contract`, SKU/product/location offering, zero-capable on-hand/reserved/available inventory, POS policy, sale-line decisions, reorder policy, recommendation policy, and medical-claim-safe customer-copy policy.
- Cross-service/storage:
  - `operations::CoreServiceContracts` groups boarding, daycare, grooming, training, and retail by `LocationId`.
  - Storage JSON codecs round-trip `CoreServiceContracts` and reject invalid validated scalars.
  - Service-offering storage tests reject cross-variant shapes.

Intentional follow-ups remain documented for production-grade behavior:

- Boarding needs deeper season/holiday/waitlist/cancellation audit aggregates and per-pet medication task schedules before live stay planning.
- Boarding care/Pawgress/report generation still needs source-provenance and approval-state integration before customer-facing use.
- Daycare needs persistence/repository surfaces for roster, attendance, incident history, and eligibility evidence before operational scheduling.
- Grooming needs full appointment/calendar repositories and schedule-window conflict resolution before booking automation.
- Training needs richer enrollment/program duration constraints, trainer qualification models, and progress-report persistence before live training workflows.
- Retail needs provider/POS/vendor adapters, product-family seed data, inventory movement audit trails, and stronger recommendation evidence before checkout or reorder automation.
- Gingr provider endpoint mapping still intentionally records semantic mapping gaps for provider retail/training/grooming domains; those are integration gaps, not missing service-domain maps.

### 3. Semantic fidelity of domain types

Pass.

The implementation generally follows the semantic-code doctrine:

- Service-line behavior lives under meaningful paths such as `operations::boarding`, `operations::daycare`, `operations::grooming`, `operations::training`, and `operations::retail`.
- The cross-service aggregate is explicit: `CoreServiceContracts { boarding, daycare, grooming, training, retail }` rather than raw flags or stringly service maps.
- Meaningful values use enums/newtypes/builders/policies:
  - `CoreServiceLine`, `ServiceOffering`, `BoardingAccommodation`, `DaycareFormat`, `GroomingService`, `TrainingProgram`, `RetailPartner`, `RetailProductCategory` for catalog-level meaning.
  - Positive scalars such as boarding stay nights, notice hours, daycare staff/pet counts, grooming appointment minutes, training duration/session counts, retail sale quantities.
  - Zero-capable retail inventory values where zero stock must be representable.
  - Nutype-backed validated IDs/notes/copy/rationale values in training and retail.
  - `bon` builders for multi-field contracts and evidence objects.
- Relationship contracts are visible at call sites:
  - Boarding capacity will not put a dog request into a cat condo segment.
  - Daycare distinguishes service variant from operational care mode.
  - Grooming separates customer-facing style notes from care/medical handling references.
  - Training separates evidence-bearing progress/outcome claims from parent-facing approval.
  - Retail separates in-house/sellable usage, reserved/on-hand/available units, POS policy, and customer-safe copy.
- Approval gates are first-class and conservative:
  - Customer messages are drafts/approval-gated.
  - Deposit/refund/waiver/manager exception paths carry review gates.
  - Medical/care/behavior ambiguity routes to staff/manager/medical review rather than silent eligibility.
  - Retail medical claims are blocked from customer drafts.

Semantic caveats to keep in view:

- Some root-level catalog enums remain intentionally broad (`operations::ServiceOffering`, `DaycareFormat`, `GroomingService`, `TrainingProgram`, `RetailPartner`). That is acceptable for the current foundation, but future behavior should migrate richer rules into service-owned modules or re-export one canonical public surface.
- Grooming still has both top-level `operations::CadenceWeeks` and `operations::grooming::CadenceWeeks`/`OrdinaryCadenceWeeks`. The implementation adds the correct 2-8 week ordinary policy, but future code should avoid duplicating cadence semantics.
- Several contracts use `Vec` fields for rule/menu collections. That is fine at this stage, but production code should add non-empty/deduplicated catalog/menu types once behavior depends on them.
- Domain error types exist in training/retail and scalar modules, but boarding/daycare/grooming can still benefit from more module-local `Error`/`Result` surfaces as policies grow.

### 4. Compile/test status

Pass.

Commands run from `/home/eran/code/nva`:

```text
rustup default stable
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Result: all passed.

Important scope note: because the workspace is not a git repository in this runtime, I could not inspect a diff, verify commit state, or attribute changed lines by commit. The review is based on current file contents, parent-card handoffs, and fresh local gates.

### 5. Live/member-facing/destructive actions

Pass.

No live/member-facing/destructive action was performed during this review. I ran local repository inspection, file reads/searches, Rust formatter/test/clippy gates, and wrote this review document. I did not call external PetSuites/Gingr/NVA systems, send messages, alter reservations, charge/refund/waive money, change care instructions, or perform destructive file actions.

The implementation itself remains contract/test oriented. It contains HTTP transport abstractions and provider endpoint builders, but the reviewed PetSuites domain work is not presented as live automation.

## Remaining gaps before production automation

These are not blockers for the NVA AI-engineer conversation; they are blockers for claiming live-operational readiness.

1. Approval-event/audit layer
   - Need durable approval records with actor, policy version, evidence IDs, review gate, approval state, and execution tool IDs.
   - Current contracts model gates well, but live execution needs an auditable gate-clearance protocol.

2. Repository and adapter contracts
   - Need service-owned repositories for capacity snapshots, daycare rosters, grooming calendars/history, training enrollments/progress, retail inventory/POS/vendor state.
   - Provider DTOs must convert into semantic domain values before policy behavior sees them.

3. Source provenance and sensitivity
   - Care/medication/behavior, grooming history, training progress, and retail recommendation evidence need explicit source provenance and redaction policy before report/message generation.

4. Live execution boundary
   - Need tool contracts that distinguish read-only, draft-only, internal-task creation, approved customer send, approved reservation mutation, approved payment/refund/waiver, and prohibited actions.
   - Tests should prove disallowed actions cannot be represented as executable without approvals.

5. Service-specific production rules
   - Boarding: holiday/season policy, waitlist, overbooking/manager-hold, medication schedules, Pawgress report approval.
   - Daycare: roster/capacity persistence, recurring attendance materialization, incident history enforcement.
   - Grooming: calendar conflict resolution, groomer qualification, appointment lifecycle, no-show/cancellation history.
   - Training: Stay-and-Study duration constraints, enrollment readiness, trainer qualification, milestone evidence, parent homework copy.
   - Retail: product-family seed data, inventory movement ledger, POS sale/refund/comp gates, vendor reorder workflow, customer-safe recommendation copy.

6. Integration mapping gaps
   - Gingr endpoint catalog still documents provider-domain semantic mapping gaps for retail/training/grooming. That is appropriate transparency, but should become concrete provider adapters before operational claims.

## Strongest NVA AI-engineer talking points

1. Semantic contracts before agents
   - The work does not start by prompting an AI to operate a pet resort. It first names the operational domain: boarding capacity, daycare eligibility, grooming calendar/history, training progress, retail inventory/POS, and approval gates.

2. AI as assistant, not unsafe executor
   - The contracts explicitly allow AI to draft, summarize, recommend, classify, and create internal review work while blocking live booking changes, payment actions, medical/behavior approvals, and customer sends without typed human approval.

3. Cross-service upsell with care-safety constraints
   - Boarding, daycare, grooming, training, and retail are connected commercially, but the types preserve safety boundaries. For example, exit-bath/grooming/training/retail recommendations are suppressed or gated when care/medical/behavior evidence is ambiguous.

4. Operational truth is typed and auditable
   - Instead of raw notes and flags, the foundation models capacity decisions, eligibility decisions, review gates, no-show policy, progress evidence, inventory positions, POS policy, and customer-safe copy as domain values that can be tested.

5. Location-specific policy without hard-coded resort folklore
   - The maps intentionally treat prices, room counts, holiday rules, local menus, trainer staffing, product inventory, and cancellation/deposit rules as location policy data. The Rust contracts are ready to consume those inputs without baking them into brittle constants.

6. Production humility
   - The current state is not overclaimed. It is a green, tested domain foundation plus a clear production gap list: repositories, adapters, durable approvals, audit events, and live execution gates.

7. Engineering maturity signal
   - The workspace passes formatter, full Rust tests, and clippy with warnings denied. Tests read like operational truths, not incidental implementation checks.

## Recommended next board slice

If the next goal is to move from conversation-ready foundation to pilot-ready implementation, create a serialized lane for:

1. `policy::approval` / workflow audit events: durable approval records and executable-action gates.
2. Boarding capacity/care pilot adapter: read-only capacity/care snapshots plus internal task drafts only.
3. Grooming calendar/history repository: read-only schedule/history plus draft rebooking/reminder plans.
4. Retail inventory/recommendation repository: inventory snapshots plus internal reorder/recommendation tasks, no POS execution.
5. Integration smoke tests using synthetic fixtures and no live/member-facing writes.

Do not wire live reservation mutation, payment/refund/waiver, or customer-message send paths until the approval/audit layer proves those actions require authorized gate clearance.
