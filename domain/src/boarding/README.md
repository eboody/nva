# `domain::boarding`

Operator translation: boarding pages describe how the system helps staff review overnight stays, room/accommodation fit, care readiness, deposits, cancellations, housekeeping, handoffs, minimum-stay rules, and exit-bath opportunities without letting a provider record or automation draft book a room or override policy. In code, that business meaning lives in `domain::boarding`, where `Contract` means a source-backed boarding rule bundle, not a legal/customer contract.

`domain::boarding` is the domain crate's model for overnight pet-resort boarding. It owns boarding concepts that should not be flattened into provider payloads or storage codes: accommodation preferences, room-capacity decisions, care-readiness gates, cancellation and deposit policy, housekeeping cadence, staff handoff requirements, minimum-stay rules, and boarding-specific upsell opportunities.

Start at [`mod.rs`](./mod.rs). It declares the module surface, defines shared validated scalars such as [`domain::boarding::RoomInventory`](./mod.rs), [`StayNights`](./mod.rs), [`NoticeHours`](./mod.rs), and [`HourOfDay`](./mod.rs), and collects the per-topic policies into [`domain::boarding::Contract`](./mod.rs). `Contract::standard_petsuites` is a fixture-like standard contract for core-service storage and tests; it is not a complete catalog of every boarding package.

## Module navigation

- [`accommodation.rs`](./accommodation.rs) defines accommodation vocabulary: [`domain::boarding::accommodation::Kind`](./accommodation.rs) and [`Preference`](./accommodation.rs). `Kind::supports_species` keeps dog-suite and cat-condo compatibility close to the domain type, while `Preference::acceptable_kinds` gives capacity policy a normalized list to evaluate.
- [`capacity.rs`](./capacity.rs) defines nightly inventory snapshots and capacity decisions: [`RoomCount`](./capacity.rs), [`SegmentCounts`](./capacity.rs), [`NightlySegmentSnapshot`](./capacity.rs), [`Snapshot`](./capacity.rs), [`Request`](./capacity.rs), [`Decision`](./capacity.rs), [`DenialReason`](./capacity.rs), [`WaitlistReason`](./capacity.rs), and [`Policy`](./capacity.rs). `Policy::evaluate` checks species/accommodation compatibility and room availability before returning available, waitlist, or deny-with-review-gate outcomes.
- [`care.rs`](./care.rs) defines boarding care-readiness review: [`domain::boarding::care::Policy`](./care.rs), [`Plan`](./care.rs), [`Readiness`](./care.rs), [`ReviewGate`](./care.rs), and [`GateReason`](./care.rs). `Policy::plan_for_pet` reads a `domain::entities::CareProfile` and blocks check-in when feeding instructions are missing or medication review is required.
- [`deposit.rs`](./deposit.rs) defines confirmation-time deposit policy: [`domain::boarding::deposit::Policy`](./deposit.rs), [`ConfirmationReadiness`](./deposit.rs), and [`Blocker`](./deposit.rs). It combines [`domain::boarding::DepositRule`](./mod.rs), [`PaymentTiming`](./mod.rs), and `domain::payment::Deposit` status to decide whether reservation confirmation is ready or needs a refund/deposit exception review gate.
- [`cancellation.rs`](./cancellation.rs) defines [`domain::boarding::cancellation::Policy`](./cancellation.rs) and [`Penalty`](./cancellation.rs), pairing required [`NoticeHours`](./mod.rs) with no-penalty, forfeit-deposit, or manager-review outcomes.
- [`housekeeping.rs`](./housekeeping.rs) defines [`domain::boarding::housekeeping::Cadence`](./housekeeping.rs): daily room reset, twice-daily extended-stay work, or turnover-only housekeeping.
- [`handoff.rs`](./handoff.rs) defines [`domain::boarding::handoff::Requirement`](./handoff.rs) for arrival care review, medication double-check, and departure belongings review.
- [`minimum_stay.rs`](./minimum_stay.rs) defines [`domain::boarding::minimum_stay::Policy`](./minimum_stay.rs) and [`Reason`](./minimum_stay.rs), keeping the required [`StayNights`](./mod.rs) tied to standard, holiday-peak, or multi-pet operational-buffer reasons.
- [`upsell.rs`](./upsell.rs) defines boarding upsell recommendations: [`domain::boarding::upsell::Policy`](./upsell.rs), [`Recommendation`](./upsell.rs), [`Opportunity`](./upsell.rs), [`Eligibility`](./upsell.rs), [`SuppressionReason`](./upsell.rs), and [`ReviewReason`](./upsell.rs). `Policy::evaluate_exit_bath` can produce an exit-bath opportunity, but care-sensitive profiles route to staff review before customer messaging.

## Type/module map

| Concept | Public type/module path | Defined in |
| --- | --- | --- |
| Boarding module contract | `domain::boarding::Contract` | [`mod.rs`](./mod.rs) |
| Room inventory and stay/cancellation scalars | `domain::boarding::RoomInventory`, `StayNights`, `NoticeHours`, `HourOfDay` | [`mod.rs`](./mod.rs) |
| Capacity plan and room availability state | `domain::boarding::CapacityPlan`, `RoomAvailability` | [`mod.rs`](./mod.rs) |
| Arrival/departure service windows | `domain::boarding::ServiceWindow`, `ServiceWindowError` | [`mod.rs`](./mod.rs) |
| Deposit and payment timing contract values | `domain::boarding::DepositRule`, `PaymentTiming` | [`mod.rs`](./mod.rs) |
| Contract-level upsell list | `domain::boarding::Upsell` | [`mod.rs`](./mod.rs) |
| Accommodation kind and preference | `domain::boarding::accommodation::Kind`, `Preference` | [`accommodation.rs`](./accommodation.rs) |
| Capacity snapshot/request | `domain::boarding::capacity::Snapshot`, `NightlySegmentSnapshot`, `SegmentCounts`, `Request` | [`capacity.rs`](./capacity.rs) |
| Capacity policy/outcome | `domain::boarding::capacity::Policy`, `Decision`, `DenialReason`, `WaitlistReason` | [`capacity.rs`](./capacity.rs) |
| Care readiness plan | `domain::boarding::care::Policy`, `Plan`, `Readiness` | [`care.rs`](./care.rs) |
| Care review gates | `domain::boarding::care::ReviewGate`, `GateReason` | [`care.rs`](./care.rs) |
| Deposit confirmation policy | `domain::boarding::deposit::Policy`, `ConfirmationReadiness`, `Blocker` | [`deposit.rs`](./deposit.rs) |
| Cancellation policy | `domain::boarding::cancellation::Policy`, `Penalty` | [`cancellation.rs`](./cancellation.rs) |
| Housekeeping cadence | `domain::boarding::housekeeping::Cadence` | [`housekeeping.rs`](./housekeeping.rs) |
| Staff handoff requirement | `domain::boarding::handoff::Requirement` | [`handoff.rs`](./handoff.rs) |
| Minimum stay policy | `domain::boarding::minimum_stay::Policy`, `Reason` | [`minimum_stay.rs`](./minimum_stay.rs) |
| Boarding upsell recommendation | `domain::boarding::upsell::Policy`, `Recommendation`, `Opportunity`, `Eligibility` | [`upsell.rs`](./upsell.rs) |
| Upsell suppression/review reasons | `domain::boarding::upsell::SuppressionReason`, `ReviewReason` | [`upsell.rs`](./upsell.rs) |

## Operator summary

Boarding supports the overnight-stay decision queue: front-desk staff and managers need to know whether a requested stay can be confirmed, should be waitlisted, must be denied pending manager approval, or needs care/payment/customer-message review before staff promise anything to a customer. The module reduces labor by turning room-capacity evidence, care-profile readiness, deposit state, cancellation notice, housekeeping cadence, handoff requirements, minimum-stay rules, and exit-bath opportunities into named decisions instead of making staff reread scattered provider notes and free-text reservation context.

It is not allowed to automate live boarding operations on its own. `domain::boarding` does not book or cancel reservations, mutate room inventory, collect deposits, issue refunds, send customer messages, make medical judgments, or override staff. It supplies semantic policy inputs and outcomes that application, storage, and integration layers can compose behind their own approval and side-effect gates.

The authoritative facts stay outside the prose: room counts and reservation state must come from the provider/read model or promoted source snapshots; payments and deposit status must come from `domain::payment`; feeding, allergy, medication, medical-condition, and temperament facts must come from the pet care/profile records; and boarding policy values must come from `domain::boarding::Contract` and the linked policy modules. If those facts are missing or contradictory, the safe output is a review gate or data-quality question, not an automated promise.

Review gates protect pets, customers, and staff at the high-risk edges: manager approval for denied/exception capacity decisions, medical-document review for missing feeding instructions or medication/care ambiguity, refund/deposit exception review for payment edge cases, and customer-message approval before any upsell recommendation becomes customer-facing.

## Boarding workflow surface

The workflow surface is mostly review triage:

1. A requested accommodation becomes `domain::boarding::accommodation::Preference` and a `domain::boarding::capacity::Request`. [`domain::boarding::capacity::Policy`](./capacity.rs) evaluates that request against a [`Snapshot`](./capacity.rs), returning an available accommodation, a waitlist reason, or a denial with `domain::policy::ReviewGate::ManagerApproval`.
2. A pet's care profile becomes a [`domain::boarding::care::Plan`](./care.rs). Missing feeding instructions or medication review requirements become [`care::ReviewGate`](./care.rs) values, so check-in readiness can be shown as `ReadyForCheckIn` or `Blocked` without ad hoc staff judgment.
3. Deposit and cancellation rules stay in domain policy. [`domain::boarding::deposit::Policy`](./deposit.rs) decides whether confirmation is blocked by a required booking deposit, while [`domain::boarding::cancellation::Policy`](./cancellation.rs) names the notice and penalty rule that downstream workflows can apply.
4. Housekeeping and handoff values in [`domain::boarding::Contract`](./mod.rs) keep operational expectations attached to the boarding service contract: room resets, arrival care review, medication double-checks, departure belongings review, minimum stays, payment timing, and allowed upsells.
5. [`domain::boarding::upsell::Policy`](./upsell.rs) demonstrates safe upsell triage for exit baths. Care-clear pets can be eligible; care-sensitive profiles require medical-document review, and every recommendation exposes a customer-message approval gate before an offer is sent.

The module does not book reservations, send customer messages, charge deposits, or mutate room inventory itself. It supplies semantic policy inputs and outcomes that application, storage, and integration layers can compose.

## Cross-crate relationships

- The domain crate exposes this module from [`domain/src/lib.rs`](../lib.rs) as `pub mod boarding`.
- `domain::operations::ServiceOffering::Boarding` links customer-facing lodging offers to broader service-offering records in [`domain/src/operations.rs`](../operations.rs). Its `lodging_offer` values name accommodation, included care, and add-ons for storage/service catalog records.
- `domain::operations::service_core::ServiceContracts` includes `boarding: domain::boarding::Contract` in [`domain/src/operations.rs`](../operations.rs), alongside daycare, grooming, training, and retail contracts.
- `app` composes boarding deposit policy into booking-triage evaluation in [`app/tests/workflow_service_composition_contracts.rs`](../../../app/tests/workflow_service_composition_contracts.rs). The test explicitly keeps `booking_triage` from owning the boarding deposit decision: it maps `domain::boarding::deposit::ConfirmationReadiness` into app-level readiness and approval gates.
- `storage::service_line::boarding` persists migrated boarding contracts and service-offering codes in [`storage/src/service_line/boarding.rs`](../../../storage/src/service_line/boarding.rs). `ContractRecord` wraps `domain::boarding::Contract`; `AccommodationCode`, `CareFeatureCode`, and `AddOnCode` convert to and from `domain::operations::lodging_offer` values.
- `storage::operations::ServiceOfferingRecord` stores boarding-specific fields in [`storage/src/operations.rs`](../../../storage/src/operations.rs): `boarding_accommodation`, `boarding_included_care`, and `boarding_add_ons`. Its shape checks keep those fields off daycare/grooming/training/retail variants.
- `storage::operations::CoreServiceContractsRecord` stores a boarding contract with the other core service contracts in [`storage/src/operations.rs`](../../../storage/src/operations.rs). Storage coverage in [`storage/tests/core_service_contract_storage.rs`](../../../storage/tests/core_service_contract_storage.rs) round-trips `domain::boarding::Contract::standard_petsuites` and rejects invalid validated boarding scalars such as zero room inventory.
- App/API demo and smoke surfaces use boarding as a service-line example in files such as [`app/src/local_smoke.rs`](../../../app/src/local_smoke.rs), [`apps/api/src/http.rs`](../../../apps/api/src/http.rs), and [`apps/staff-web/app/page.tsx`](../../../apps/staff-web/app/page.tsx), but those files do not own the core boarding policy types.
- No Gingr boarding DTO or mapper is currently present under [`integrations/gingr/src`](../../../integrations/gingr/src). If provider-specific boarding payloads are added later, keep raw provider fields there and promote only validated values into `domain::boarding`.

## Maintainer notes

- Add new boarding policy where the domain concept owns the decision: room availability in [`capacity.rs`](./capacity.rs), check-in readiness in [`care.rs`](./care.rs), confirmation gating in [`deposit.rs`](./deposit.rs), customer-safe offer triage in [`upsell.rs`](./upsell.rs), and service-contract configuration in [`mod.rs`](./mod.rs).
- Preserve module-qualified names in prose and code. `domain::boarding::capacity::Decision`, `domain::boarding::deposit::ConfirmationReadiness`, and `domain::boarding::upsell::Eligibility` are intentionally different outcomes even though all can affect reservation readiness.
- Keep storage-code conversions explicit in [`storage/src/service_line/boarding.rs`](../../../storage/src/service_line/boarding.rs) and [`storage/src/operations.rs`](../../../storage/src/operations.rs) so snake_case database values do not leak into domain call sites.
- Keep application workflows thin: app code can translate boarding decisions into UI/API readiness buckets, staff tasks, or approval gates, but the underlying policy should remain in `domain::boarding`.
