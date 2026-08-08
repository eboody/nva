# Semantic code doctrine inventory

Scope refreshed on 2026-06-10 after final skeleton review and operations metric remediation: `crates/domain/src/{lib,entities,workflow,tools,policy,agent,agents,booking_triage,care,temperament,money,payment,reservation,customer,location,pet,portal,operations}.rs`, `crates/domain/tests/domain_quality_patterns.rs`, and the `cli agents/tools` JSON surface.

Quality-pause rule: future feature work must not add behavior on top of a raw primitive domain surface it touches. If a feature touches one of the intentional debt items below, first add a failing semantic API test, verify RED, migrate the touched surface to the owning semantic type/enumeration/typestate, run the focused test, then run full gates for any code-changing card.

Next feature entry rule: a raw or weakly typed field is not permission to add behavior. Treat it as an entry gate. Convert boundary payloads immediately, or create the missing semantic contract before policy, workflow, tool execution, customer messaging, reporting, scheduling, or automation branches on the value.

## Completed / already semantic baseline

- `entities::{LocationId, CustomerId, PetId, ReservationId}` wrap UUIDs as semantic identifiers. Future ID policy can still decide whether tuple constructors should remain public.
- `entities::Pet.name` uses `pet::Name`; `entities::Pet` has a compile-checked `bon` builder with semantic defaults for optional/profile fields.
- `entities::Location::{name, timezone}` and `entities::Brand::NeighborhoodPetResort.name` use `location::{Name, Timezone}`.
- `entities::Customer::{full_name, email, mobile_phone}` uses `customer::{Name, Email, Phone}`; `entities::Customer` has a `bon` builder.
- `entities::PortalAccountRef.external_customer_id` uses `portal::CustomerId`.
- `entities::LocationPolicyRefs::{vaccine_policy_id, deposit_policy_id, playgroup_policy_id}` use `policy::Id`.
- `entities::CareProfile` and `entities::MedicationInstruction` use `care::*` semantic values for feeding, allergy, medical condition, medication, contact, and review-reason fields. `MedicationInstruction` has a `bon` builder and sensitive debug output is redacted by tests.
- `entities::TemperamentProfile` uses `temperament::*` semantic observation values and a `bon` builder with safe defaults. Staff notes and extension labels are redacted from debug output.
- `entities::Reservation` carries semantic payment/deposit, add-on, age-threshold, source, and hard-stop values and has a `bon` builder with defaulted optional/collection fields.
- `entities::HardStop::{MissingRequiredVaccine, IneligibleForGroupPlay, AgeBelowMinimumWeeks}` carries policy/reservation semantic values rather than raw strings or bare numbers.
- `entities::ActorRef` carries typed staff IDs, manager IDs, and agent workflow names. `entities::AuditEvent` carries typed subject, action, and metadata key/value values.
- `agent.rs` defines validated `nutype` scalars for agent names, purposes, tool names, forbidden actions, policy instructions, and output schema names.
- `agents.rs` uses semantic agent types in `AgentPromptPacket` and re-exports prompt-packet semantic scalars at the agent boundary.
- `booking_triage.rs` uses `nutype` request/policy snapshot values and `statum` typestate for booking triage readiness.
- `money.rs`, `payment.rs`, and `reservation.rs` own deposit/money/age/add-on semantics, with module-local `error.rs`/`Result<T>` where validation can fail.
- `operations.rs` owns daily-brief, lead, review/reputation, metric, and operations recommendation semantics for NVA-style resort operations. Capacity/labor metrics now use `CapacityBooked`, non-zero `CapacityLimit`, `CapacitySaturationBasisPoints`, and `ScheduledStaffCount` instead of exposing raw public numeric fields.
- `policy.rs` uses `Id`, `VaccineName`, `WorkflowName`, `AutomationRationale`, enum-centered `PlayEligibility`, and typed `PolicyDenialReason` instead of vaccine/result rationale strings.
- `tools.rs` uses module-local `tools::error::{ToolError, Result}` plus enum-centered availability decisions, typed portal/payment/messaging/document/media/Hermes request contracts, `CapacitySnapshotId`, `AvailabilityServiceNotes`, `ReservationStatus`, `StatusSuggestionReason`, and `DraftUpdateId` for tool result surfaces.
- `workflow.rs` uses `WorkflowEventId`, semantic external subject values, summary/risk/verification/review text, task/message/status-update payload values, and enum-centered recommended actions. Reservation status updates carry `entities::ReservationStatus` plus transition intent/reason.
- `lib.rs` exposes `prelude` as an ergonomic consolidation surface for common agent/entity/tool/workflow boundary types while preserving module-qualified semantic ownership.
- `domain_quality_patterns.rs` covers migrated/consolidated surfaces: semantic scalars, bon builders, booking-triage typestate, care/temperament/payment/reservation/workflow/tool/operations semantics, debug redaction, `prelude` re-exports, and CLI-facing baseline specs.
- CLI output is JSON-valid and reflects the typed agent/tool surfaces: baseline agent specs and external tool candidates.

## Domain-core debt: migrate before adding behavior here

1. Public tuple ID constructors.
   - UUID wrapper IDs are semantic by type, but tuple constructors are still public.
   - Before persistence or external reconciliation behavior grows, decide whether IDs should be constructed through module-owned smart constructors or remain transparent boundary values.

2. Extension-label escape hatches.
   - `entities::{PortalProvider::Other, Species::Other, AddOn::Other}` and similar extension variants are acceptable while quarantined as opaque labels.
   - `ToolResourceId::External(String)` and `ExternalFailure::Other(String)` remain quarantined external-system escape hatches.
   - If business rules branch on the payload, migrate to a validated extension-label type or promote concrete enum/provider variants.

3. Audit/action metadata shape.
   - `entities::AuditEvent.metadata` is a typed key/value map and safe for append-only logging.
   - If filtering, policy, reconciliation, workflow routing, or reporting behavior depends on a key/value, promote that metadata into a typed audit field or dedicated module.

4. Actor identity ownership.
   - `entities::ActorRef` carries typed staff/manager IDs and agent workflow names, but ownership is still in `entities`/`agent` rather than a dedicated actor module.
   - Add `actor.rs` only when staff/manager/workflow identities gain behavior outside attribution/audit.

5. Policy configuration granularity.
   - `VaccineRequirement.source_must_be_licensed_vet: bool` should become a source-verification enum before vaccine-source behavior branches on it.
   - Ownership: `policy` owns vaccine/automation rule semantics; workflow identity may reuse `agent::Name` only if an agent spec and workflow are intentionally the same concept.

6. Lifecycle typestate.
   - `booking_triage` has real phased typestate today. Reservation lifecycle remains enum/runtime-data oriented.
   - Introduce `statum` for reservation phases only when method legality should change by phase, for example requested -> offered -> confirmed -> checked-in -> active -> checked-out.

7. Operations policy depth.
   - Capacity/labor reporting now uses semantic values, but overbooking rules, labor-ratio rules, and saturation thresholds are not yet policy contracts.
   - Before operations reports become scheduling/capacity decisions, add typed threshold/reason/decision values instead of branching directly on numeric basis points or staff counts.

## Recommended next migration order

1. Policy vaccine-source verification enum before vaccine-source verification behavior grows.
2. Audit/actor module extraction only when audit search, staff/manager attribution, or actor-scoped permissions become behavior rather than passive data.
3. Extension-label cleanup (`Other` payloads, tool external/other failures) when code starts branching on those payloads.
4. ID constructor policy before persistence/reconciliation adapters depend on unrestricted tuple constructors.
5. Reservation lifecycle typestate only when workflow code starts enforcing legal phase-specific operations.
6. Operations capacity/labor policy values only when reports become scheduling/capacity decisions.

## Follow-up test coverage suggestions

- Add semantic API tests for any feature that consumes audit metadata as behavior rather than log context.
- Add construction tests for future policy source-verification enum values before vaccine document automation depends on them.
- Add compile/API tests around `prelude` whenever new commonly consumed boundary contracts are added, while keeping module-local types authoritative.
- Add focused operations policy tests before capacity/labor metrics drive automated scheduling, staffing, or booking decisions.
