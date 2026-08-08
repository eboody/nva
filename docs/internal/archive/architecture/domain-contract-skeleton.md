# Domain contract skeleton map

Purpose: this is the contract-first expansion map for the pet-resort domain. It preserves the repository doctrine of semantic fidelity: write failing semantic API tests first, verify RED, then add the smallest module-owned type/trait/typestate surface needed to make the contract true. Do not add behavior to raw `String`, `bool`, integer, or UUID fields while expanding the domain.

For the canonical atlas of known source evidence, analytics/read-model, workflow validator, policy, staff-action, labor, capacity, POS, and BI-export bridge contracts, see [`known-domain-inter-type-contracts.md`](known-domain-inter-type-contracts.md).

Status after final skeleton review: the first contract skeleton is implemented and review-approved. The foundation now has module-owned semantic contracts for care/medical, temperament, money/deposit, reservation add-ons and age thresholds, workflow/action payloads, tool boundaries, operations metrics, agent prompts, and baseline entity identity surfaces. The remaining items in this document are intentional entry rules for future features, not blockers for the current skeleton.

## Implemented semantic contract surface

Current modules:

- `entities`: aggregate/entity shell for locations, customers, pets, reservations, deposits, hard stops, audit events, and actor references. Aggregates now mostly compose module-owned semantic values instead of raw primitives.
- `pet`, `customer`, `location`, `portal`, `agent`: validated scalar modules for currently extracted semantic values.
- `care`: feeding instructions, medication names/doses/schedules, allergy and medical-condition names, contact references, medication review requirements, and sensitive debug redaction.
- `temperament`: group-play observations, people orientation, temperament ratings, behavior observations, extension labels, staff notes, and redacted debug surfaces.
- `money`, `payment`, `reservation`: validated minor units/currency, deposit/payment semantics, payment references, add-on labels, minimum-age thresholds, age policy reasons, and module-local error/`Result<T>` surfaces where construction can fail.
- `policy`: policy IDs, vaccine names, play eligibility decisions/reasons, review gates, automation levels/rules, automation rationales, and conservative play eligibility policy.
- `workflow`: workflow event IDs, external subject values, policy context, result summaries, review/risk/verification text, draft task/message payloads, and reservation-specific typed status updates.
- `tools` and `tools::error`: external system traits, availability/reservation draft contracts, typed portal/payment/messaging/document/media/Hermes request contracts, external tool candidate list, and module-local `ToolError`/`Result<T>`.
- `operations`: NVA-style resort operations contracts for daily briefs, lead conversion, grooming/rebooking/reputation signals, operations recommendations/risks, capacity metrics, labor snapshots, and validated operational text values.
- `agents`: workflow-agent trait, prompt packet boundary, semantic prompt scalar re-exports, and baseline agent specs.
- `booking_triage`: statum typestate example for request readiness before policy decision.
- `prelude`: ergonomic re-export surface for common agent/entity/tool/workflow boundary types while preserving module-qualified ownership.

Implemented test coverage is concentrated in `crates/domain/tests/domain_quality_patterns.rs`. It covers semantic scalars, bon builders, booking-triage typestate, care/temperament/payment/reservation/workflow/tool/operations semantics, debug redaction for sensitive profile notes, `prelude` re-exports, and CLI-facing baseline agent/tool surfaces.

## Remaining intentional debt / entry gates

The skeleton is intentionally not a complete product domain. Future work must retire the relevant debt before adding behavior that depends on it:

1. Public tuple ID constructors.
   - `entities::{LocationId, CustomerId, PetId, ReservationId}` and workflow IDs are semantic by type, but some tuple constructors remain public.
   - Before persistence, reconciliation, or provider identity behavior depends on construction rules, decide whether IDs stay transparent boundary values or move behind module-owned smart constructors.

2. Extension-label escape hatches.
   - `entities::{PortalProvider::Other, Species::Other, AddOn::Other}` and tool external/other failure variants are acceptable while their payloads are opaque labels.
   - If business rules branch on an `Other` payload, migrate that payload to a validated extension-label type or promote concrete enum variants.

3. Audit/action metadata shape.
   - `entities::AuditEvent.metadata` uses typed key/value values and is safe for append-only logging.
   - If filtering, policy, reconciliation, workflow routing, or reporting behavior depends on a metadata key/value, promote that key/value into a typed audit field or dedicated module.

4. Actor identity ownership.
   - `entities::ActorRef` carries typed staff/manager IDs and agent workflow names, but ownership remains in `entities`/`agent` rather than a dedicated actor module.
   - Add `actor.rs` only when staff, manager, system, or workflow identities gain behavior outside attribution/audit.

5. Policy configuration granularity.
   - `VaccineRequirement.source_must_be_licensed_vet: bool` remains the main known policy boolean.
   - Before vaccine-source verification behavior branches on it, replace it with an explicit source-verification enum/reason vocabulary owned by `policy`.

6. Lifecycle typestate.
   - `booking_triage` has real phased typestate today. Reservation lifecycle remains enum/runtime-data oriented.
   - Introduce `statum` for reservation phases only when method legality should change by phase, for example requested -> offered -> confirmed -> checked-in -> active -> checked-out.

7. Operations metric policy depth.
   - `operations::CapacityMetric` now uses `CapacityBooked`, non-zero `CapacityLimit`, and `CapacitySaturationBasisPoints`; `LaborSnapshot` uses `ScheduledStaffCount`.
   - If scheduling/capacity rules become behavior rather than reporting signals, add explicit policy vocabulary for overbooking, staff ratios, and saturation thresholds before branching on raw numeric thresholds.

## Next feature entry rule

Before any feature touches a domain surface, classify the surface:

- Already semantic: use the module-owned type directly and keep behavior in deterministic Rust policy/contract code.
- Quarantined boundary/raw payload: convert it immediately at the boundary before policy, workflow, customer messaging, tool execution, or reporting behavior sees it.
- Debt item listed above: write a failing semantic API test first, verify RED, migrate the touched surface to an invariant-bearing type/enum/typestate, then implement the feature behavior.

Do not create feature logic that branches on raw `String`, `bool`, integer, or provider IDs just because a field already exists. A touched raw field is an entry gate, not permission to pile behavior onto primitives.

## Ownership map for future expansion

### Care and medical profile contracts

Current state:

- `care.rs` owns the implemented non-medical daily-care and medical-profile values used by `entities::CareProfile` and `entities::MedicationInstruction`.
- `MedicationInstruction` has a `bon` builder and a semantic `MedicationReviewRequirement` rather than a raw review boolean.
- Sensitive profile/debug output is covered by tests so medication, allergy, condition, contact, and staff-note details are not leaked through `Debug`.

Future expansion rule: if health documents, veterinary source proof, OCR ambiguity, or medical approval behavior grows beyond the current care profile values, introduce the smallest medical/source-owned semantic values first. `policy` owns review gates and decisions; care/medical facts should not be hidden inside `policy`.

Boundary rule: raw OCR/provider/customer text may enter through a boundary DTO, but must be converted into semantic care/medical values before any policy decision, review gate, or customer message uses it.

### Temperament and group-play observation contracts

Current state:

- `temperament.rs` owns group-play observation, people orientation, ratings, behavior observations, extension labels, and staff notes.
- `entities::TemperamentProfile` composes those values through a `bon` builder with safe defaults.
- `policy::ConservativePlayEligibilityPolicy` consumes typed temperament/profile state and returns `policy::PlayEligibility::{Eligible, Ineligible}` with typed reasons.

Future expansion rule: observations are not policy decisions. If playgroup matching, incident handling, or enrichment routing needs more nuance, add observation enums/reasons first and only then bridge them into policy decisions.

Boundary rule: avoid representing group-play candidacy as `Option<bool>` or ad hoc strings. Unknown, candidate, not-candidate, and review-required states must be explicit enum cases when behavior branches on them.

### Reservation and payment contracts

Current state:

- `money.rs`, `payment.rs`, and `reservation.rs` own money/deposit/payment-reference/add-on/minimum-age semantics and construction errors.
- `entities::Reservation`, `entities::Deposit`, and `entities::HardStop` carry semantic payment/reservation/policy values instead of bare amounts or text hard stops.
- Tool reservation/payment interactions remain draft/authorization oriented and carry typed IDs/reasons.

Future expansion rule: use runtime status enums until code needs phase-specific legal method surfaces. Add `statum` reservation typestate only when lifecycle behavior cannot be honestly represented by typed runtime validation.

Boundary rule: payment provider IDs and raw payloads belong in adapters or boundary DTOs. Domain-facing contracts should accept and return semantic amounts, statuses, drafts, references, reasons, and review gates.

### Workflow and status-transition contracts

Current state:

- `workflow::WorkflowEvent` uses `WorkflowEventId`.
- `workflow::RecommendedAction::UpdateStatus` carries a `status_update::Target::Reservation` payload with `entities::ReservationStatus`, `TransitionIntent`, and a validated transition reason.
- Arbitrary entity/status string pairs are rejected by serialization tests.

Future expansion rule: add entity-specific status-update targets only when that entity has a real status vocabulary. Do not reintroduce generic `{ entity, status: String }` updates.

Validator evidence rule: future workflow validators should consume named source-agnostic evidence bundles, not provider DTOs, BI table names, raw customer/staff prose, or LLM summaries. The canonical bridge shape is maintained in [`known-domain-inter-type-contracts.md`](known-domain-inter-type-contracts.md): deterministic evidence and policy decisions first, then separate staff/manager/customer copy drafts that cite the decision and remain review-gated as needed.

Boundary rule: agents may recommend actions; deterministic Rust validators decide whether an action is allowed and whether it needs human review before a tool sees it.

### External tool trait contracts

Current state:

- `tools` contains draft/read-oriented boundary contracts for availability, reservation updates, portal lookups, payment authorizations, message drafts, document intake, media snapshots, and Hermes task drafts.
- `tools::error` owns external failure categories and policy-denied errors.
- Availability notes and result surfaces are typed; reservation updates are drafts with typed status/reason data.

Future expansion rule: split traits by capability as integrations grow, for example `CustomerStore`, `AvailabilityPort`, `ReservationDraftPort`, `PaymentPort`, `MessagingPort`, `DocumentPort`, and `AuditSink`. Split when capability ownership or review semantics would otherwise blur.

Boundary rule: concrete provider IDs and raw payloads belong in integration adapters or boundary DTOs. Domain-facing traits should accept and return semantic IDs, decisions, drafts, and review gates.

### Audit and actor identity contracts

Current state:

- `entities::AuditEvent` uses typed subject/action/metadata key/value contracts.
- `entities::ActorRef` uses typed staff IDs, manager IDs, and agent workflow names.

Future expansion rule: create `audit.rs` and/or `actor.rs` only when audit search, actor permissions, staff/manager routing, or system attribution gains behavior outside passive logging.

Boundary rule: audit metadata can remain a typed map only while it is write-only/read-only logging. If behavior branches on a metadata key/value, promote that key/value into a typed field.

### Safety and automation boundary contracts

Current state:

- `policy::AutomationLevel` and `policy::ReviewGate` are the primary safety vocabulary.
- Agent prompt packets and workflow results use semantic instructions, schemas, summaries, risk flags, verification notes, and review reasons.
- Tool traits preserve draft/review semantics for customer-facing and provider-facing operations.

Future expansion rule: before automated execution exists, add explicit `policy::AutomationDecision`/denial/rationale values for the action being automated. Do not infer approval from a raw bool or from the absence of a review gate.

Boundary rule: an agent can draft, extract, summarize, recommend, and flag risk. It cannot independently confirm bookings, waive/refund deposits, approve ambiguous medical documents, hide concerning facts, close incidents, or send sensitive customer messages unless a typed policy decision explicitly allows it.

## Test-first order for new slices

Use one focused integration/API test file per slice under `crates/domain/tests/` unless a module-local unit test better captures private helpers. For every slice: write the failing semantic API test, run the focused command and verify RED, implement only the semantic contract, run the focused test to GREEN, then run `cargo test --workspace`. Run full format/clippy/CLI gates for code-changing cards.

Recommended next migration order:

1. Policy vaccine-source verification enum before vaccine-source verification behavior grows.
2. Audit/actor module extraction only when audit search, staff/manager attribution, or actor-scoped permissions become behavior rather than passive data.
3. Extension-label cleanup (`Other` payloads, tool external/other failures) when code starts branching on those payloads.
4. ID constructor policy before persistence/reconciliation adapters depend on unrestricted tuple constructors.
5. Reservation lifecycle typestate only when workflow code starts enforcing legal phase-specific operations.
6. Operations capacity/labor policy values only when reports become scheduling/capacity decisions.

## Documentation and gate expectations

- Contract-map/doc-only changes: run `cargo test --workspace` if examples or API references changed; full gates are optional unless code changed.
- Code-changing semantic slices: run:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo run -q -p cli -- agents | python -m json.tool >/dev/null`
  - `cargo run -q -p cli -- tools | python -m json.tool >/dev/null`
- Never batch unrelated contract migrations. The shared workspace is serialized; each mutating card should retire one ownership cluster and leave a clear test-first trail.
