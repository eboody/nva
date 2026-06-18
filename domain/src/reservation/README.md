# `domain::reservation`

`domain::reservation` is the domain crate's small shared vocabulary for reservation policy facts that do not belong to one service line. It owns reusable reservation inputs such as minimum-age thresholds, customer-facing add-on labels, and transition reasons. The canonical reservation entity, identifier, source, and lifecycle status currently live in [`domain::entities::Reservation`](../entities.rs) and its nested [`domain::entities::reservation`](../entities.rs) module; this module complements that entity rather than replacing it.

Start at [`mod.rs`](./mod.rs). It declares the module surface, re-exports [`domain::reservation::Error`](./error.rs) and [`Result`](./error.rs), validates [`MinimumAgeWeeks`](./mod.rs) and [`AddOnLabel`](./mod.rs), and names policy/transition reasons used by booking and provider-update workflows.

## Module navigation

- [`mod.rs`](./mod.rs) defines the public reservation support surface: [`domain::reservation::MinimumAgeWeeks`](./mod.rs), [`AgePolicyReason`](./mod.rs), [`AgeThreshold`](./mod.rs), [`AddOnLabel`](./mod.rs), and [`TransitionReason`](./mod.rs). These are intentionally service-neutral so boarding, daycare/day-play, grooming, and training flows can share them without copying provider strings or free-text notes.
- [`error.rs`](./error.rs) defines [`domain::reservation::Error`](./error.rs) and [`domain::reservation::Result`](./error.rs). The current errors cover invalid minimum-age and add-on-label construction: `EmptyMinimumAge`, `EmptyAddOnLabel`, and `AddOnLabelTooLong`.
- [`domain/src/entities.rs`](../entities.rs) owns the broader reservation entity and lifecycle vocabulary: [`domain::entities::reservation::Id`](../entities.rs), [`Status`](../entities.rs), [`Source`](../entities.rs), [`domain::entities::Reservation`](../entities.rs), [`AddOn`](../entities.rs), and [`HardStop`](../entities.rs). `domain::entities::AddOn::Other` carries `domain::reservation::AddOnLabel`, while `domain::entities::HardStop::AgeBelowMinimumWeeks` carries `domain::reservation::AgeThreshold`.

## Type/module map

| Concept | Public type/module path | Defined in |
| --- | --- | --- |
| Reservation support module | `domain::reservation` | [`mod.rs`](./mod.rs) |
| Minimum age scalar | `domain::reservation::MinimumAgeWeeks` | [`mod.rs`](./mod.rs) |
| Minimum-age policy reason | `domain::reservation::AgePolicyReason` | [`mod.rs`](./mod.rs) |
| Minimum-age threshold | `domain::reservation::AgeThreshold` | [`mod.rs`](./mod.rs) |
| Custom add-on label | `domain::reservation::AddOnLabel` | [`mod.rs`](./mod.rs) |
| Reservation transition reason | `domain::reservation::TransitionReason` | [`mod.rs`](./mod.rs) |
| Reservation support errors | `domain::reservation::Error`, `domain::reservation::Result` | [`error.rs`](./error.rs) |
| Canonical reservation id | `domain::entities::reservation::Id` | [`../entities.rs`](../entities.rs) |
| Canonical reservation lifecycle status | `domain::entities::reservation::Status` | [`../entities.rs`](../entities.rs) |
| Canonical reservation source | `domain::entities::reservation::Source` | [`../entities.rs`](../entities.rs) |
| Canonical reservation record | `domain::entities::Reservation` | [`../entities.rs`](../entities.rs) |
| Reservation add-ons and hard stops | `domain::entities::AddOn`, `domain::entities::HardStop` | [`../entities.rs`](../entities.rs) |

## Reservation workflow surface

The labor-cost-reduction surface is source-data normalization and exception triage. `domain::reservation` keeps recurring reservation facts as validated values so app workflows can route obvious cases and review exceptions without asking managers to re-interpret provider fields.

1. `domain::reservation::MinimumAgeWeeks` rejects zero-week minimums during construction/deserialization. `domain::reservation::AgeThreshold` pairs that scalar with `domain::reservation::AgePolicyReason` variants such as `BoardingMinimum`, `DayPlayMinimum`, `DaycareMinimum`, or `ServiceSpecificMinimum`.
2. `domain::entities::HardStop::AgeBelowMinimumWeeks` in [`entities.rs`](../entities.rs) embeds `domain::reservation::AgeThreshold`, letting `app::booking_triage` treat age policy as a typed hard stop instead of a provider-specific status note.
3. `domain::reservation::AddOnLabel` trims and length-checks custom add-ons before they enter `domain::entities::AddOn::Other`; standard add-ons such as `GroupPlay`, `ExitBath`, and `MedicationAdministration` remain explicit enum variants in [`domain::entities::AddOn`](../entities.rs).
4. `domain::reservation::TransitionReason` names common status-change rationales (`CustomerRequested`, `CapacityUnavailable`, `PolicyHardStop`, `MissingRequiredInformation`, and `StaffOverride`) for reservation-update and audit language. The current source defines the vocabulary only; status mutation is performed by application/tool/provider layers.
5. `domain::entities::reservation::Status` in [`entities.rs`](../entities.rs) is the canonical domain lifecycle enum used by booking triage, checkout completion, workflow status updates, source snapshots, and provider-facing draft updates.

The module does not book reservations, hold capacity, mutate Gingr, send customer messages, or decide service-line-specific policy by itself. Those decisions live in service modules such as [`domain::boarding`](../boarding/README.md), [`domain::daycare`](../daycare/README.md), [`domain::grooming`](../grooming/mod.rs), and [`domain::training`](../training/mod.rs), then app workflows translate their outcomes into staff packets or provider drafts.

## Cross-crate relationships

- The domain crate exposes this module from [`domain/src/lib.rs`](../lib.rs) as `pub mod reservation`.
- [`domain::entities::Reservation`](../entities.rs) is the aggregate record that carries `domain::entities::reservation::Id`, `Status`, `Source`, `domain::payment::Deposit`, requested add-ons, and hard stops. Keep the aggregate there; add only reusable reservation-support vocabulary here.
- [`domain::source::reservation`](../source.rs) models source/provider reservation snapshots, statuses, assumptions, and data-quality projection inputs. Tests in [`domain/tests/reservation_source_contracts.rs`](../../tests/reservation_source_contracts.rs) show Gingr source facts promoting to source-agnostic reservation snapshots before analytics/stay projections consume them.
- [`domain::boarding`](../boarding/README.md) uses reservation/payment policy during confirmation: [`domain::boarding::deposit::Policy`](../boarding/deposit.rs) evaluates `domain::payment::Deposit` against boarding `DepositRule` and `PaymentTiming`. Boarding capacity, care, cancellation, handoff, and upsell policies can all produce reservation-readiness facts.
- [`domain::daycare`](../daycare/README.md) uses reservation identity and readiness in high-volume check-in and group-play flows. Daycare front-desk decisions reference `domain::entities::reservation::Id` in [`domain/src/daycare/front_desk.rs`](../daycare/front_desk.rs) while keeping daycare-specific routing in `domain::daycare`.
- [`domain::grooming`](../grooming/mod.rs) and [`domain::training`](../training/mod.rs) are service-line modules that can feed reservation scheduling, deposit/review, rebooking, package, and customer-message review decisions without redefining the core reservation identifier or status vocabulary.
- [`app::booking_triage`](../../../app/src/booking_triage.rs) reads `domain::entities::Reservation` through a repository, evaluates hard stops/deposit readiness, maps outcomes to `domain::entities::reservation::Status`, and blocks unsafe actions such as provider mutation, customer send, or payment movement until review gates are satisfied.
- [`app::checkout_completion`](../../../app/src/checkout_completion.rs) consumes `domain::entities::reservation::Id` plus source reservation status and staff handoff evidence, then suggests `domain::entities::reservation::Status::CheckedOut` only when staff and source evidence agree.
- [`app::tools::ReservationSystem`](../../../app/src/tools.rs) exposes app-level availability and draft-update ports. `tools::availability::Request` carries an optional `domain::entities::reservation::Id`, and `tools::draft_update::Request` carries a proposed `domain::entities::reservation::Status` plus app-level rationale.
- Gingr reservation endpoint wrappers live at [`integrations/gingr/src/endpoint/reservations.rs`](../../../integrations/gingr/src/endpoint/reservations.rs). They model provider-boundary requests such as reservation types, reservation searches, reservations by animal/owner, back-of-house lookups, and services by type. Keep Gingr endpoint ids, filters, and location-scope caveats there; promote validated source facts into `domain::source`/`domain::entities` before domain workflows use them.
- Gingr module exports and shared endpoint primitives live in [`integrations/gingr/src/lib.rs`](../../../integrations/gingr/src/lib.rs) and [`integrations/gingr/src/endpoint/mod.rs`](../../../integrations/gingr/src/endpoint/mod.rs). Provider-specific date/range/id errors should not become `domain::reservation::Error` unless the invariant is truly domain-owned.

## Maintainer notes

- Preserve the split between `domain::reservation` support vocabulary and `domain::entities::reservation` entity lifecycle vocabulary. If a type identifies or stores the reservation aggregate, it probably belongs under `domain::entities`; if it is a reusable policy reason or validated reservation fact, it may belong here.
- Keep service-line policy in the service-line modules. For example, boarding minimum-stay and deposit decisions belong in [`domain::boarding`](../boarding/README.md), daycare group-play and front-desk decisions belong in [`domain::daycare`](../daycare/README.md), grooming no-show/rebooking decisions belong in [`domain::grooming`](../grooming/mod.rs), and training package/progress decisions belong in [`domain::training`](../training/mod.rs).
- Keep provider mutation outside the domain crate. `domain::reservation::TransitionReason` can explain why a transition is proposed, but app/provider ports such as [`app::tools::draft_update`](../../../app/src/tools.rs) and Gingr endpoint wrappers own draft/update boundaries.
- Add error variants to [`error.rs`](./error.rs) only for invariants this module validates directly. Do not turn provider parser failures or app workflow repository errors into reservation-domain errors.
