# `domain::payment`

`domain::payment` is the domain crate's shared reservation-payment vocabulary. It owns validated payment references and deposit state used by booking, boarding confirmation, checkout, and payment-provider ports. It does not authorize cards, issue refunds, reconcile ledgers, or mutate a provider record; those actions are represented by app/tool interfaces and external integrations.

Start at [`mod.rs`](./mod.rs). It declares the module surface, re-exports [`domain::payment::Error`](./error.rs) and [`Result`](./error.rs), validates [`Reference`](./mod.rs), and defines [`DepositStatus`](./mod.rs) plus [`Deposit`](./mod.rs). `Deposit::requires_collection` is the current domain helper that says whether a `Required` or `Failed` deposit still needs payment handling before a workflow treats the reservation as financially satisfied.

## Module navigation

- [`mod.rs`](./mod.rs) defines [`domain::payment::Reference`](./mod.rs), [`DepositStatus`](./mod.rs), and [`Deposit`](./mod.rs). A `Deposit` carries a [`domain::money::Money`](../money/mod.rs) amount, an optional refund deadline, the current deposit status, and an optional provider/payment reference.
- [`error.rs`](./error.rs) defines [`domain::payment::Error`](./error.rs) and [`domain::payment::Result`](./error.rs). The current errors are `EmptyReference` and `ReferenceTooLong`; provider declines, refund failures, transport failures, and app repository failures are intentionally not payment-domain construction errors.
- [`domain/src/money/mod.rs`](../money/mod.rs) owns the amount type used by `Deposit`: [`domain::money::Money`](../money/mod.rs), [`MinorUnits`](../money/mod.rs), and [`Currency`](../money/mod.rs). Keep amount validation there instead of duplicating payment-local integer rules.

## Type/module map

| Concept | Public type/module path | Defined in |
| --- | --- | --- |
| Payment support module | `domain::payment` | [`mod.rs`](./mod.rs) |
| Provider/payment reference scalar | `domain::payment::Reference` | [`mod.rs`](./mod.rs) |
| Deposit lifecycle status | `domain::payment::DepositStatus` | [`mod.rs`](./mod.rs) |
| Reservation deposit value | `domain::payment::Deposit` | [`mod.rs`](./mod.rs) |
| Payment construction errors | `domain::payment::Error`, `domain::payment::Result` | [`error.rs`](./error.rs) |
| Monetary amount and currency | `domain::money::Money`, `domain::money::MinorUnits`, `domain::money::Currency` | [`../money/mod.rs`](../money/mod.rs) |
| Reservation aggregate deposit alias | `domain::entities::Deposit`, `domain::entities::PaymentStatus` | [`../entities.rs`](../entities.rs) |
| Boarding deposit confirmation policy | `domain::boarding::deposit::Policy`, `ConfirmationReadiness`, `Blocker` | [`../boarding/deposit.rs`](../boarding/deposit.rs) |
| App payment gateway port | `app::tools::payment::Gateway` | [`../../../app/src/tools.rs`](../../../app/src/tools.rs) |

## Payment policy surface

The labor-cost-reduction surface is payment exception triage: the domain gives booking and checkout workflows a small, typed answer to “is the deposit settled enough to proceed?” so staff do not reconcile free-text payment notes before every confirmation.

1. `domain::payment::Reference` trims and validates provider/payment references up to 160 characters. It is suitable for storing references such as payment-intent, PMS, or portal ids after a boundary layer has accepted them; it is not a provider authorization object.
2. `domain::payment::DepositStatus` names the deposit state: `NotRequired`, `Required`, `Paid`, `Refunded`, `Failed`, and `WaivedByManager`.
3. `domain::payment::Deposit::required` creates a required deposit with no provider reference. `Deposit::paid` and `Deposit::mark_paid` attach a validated `domain::payment::Reference` and set the status to `Paid`. `Deposit::waived` records manager-waived deposits with an amount but no reference.
4. `Deposit::with_refundable_until` records the refund deadline as `chrono::DateTime<Utc>`; the domain value stores that deadline but does not issue refunds.
5. `Deposit::requires_collection` returns `true` for `Required` and `Failed` deposits. App workflows can use that as a deterministic precondition before drafting confirmations or escalating payment exceptions.

The module does not decide boarding cancellation penalties, capture payments, refund money, or move POS balances. Boarding policy, app payment ports, and external payment providers compose this value when they need those actions.

## Cross-crate relationships

- The domain crate exposes this module from [`domain/src/lib.rs`](../lib.rs) as `pub mod payment`.
- [`domain::entities::Reservation`](../entities.rs) stores `deposit: Option<domain::entities::Deposit>`, where [`domain::entities::Deposit`](../entities.rs) is a type alias for `domain::payment::Deposit`. `domain::entities::PaymentStatus` aliases `domain::payment::DepositStatus`.
- [`domain::money`](../money/mod.rs) supplies the `Money` amount embedded in `domain::payment::Deposit`; tests in [`domain/tests/domain_quality_patterns.rs`](../../tests/domain_quality_patterns.rs) cover money/deposit/reference validation and deserialization failures for invalid primitives.
- [`domain::boarding::deposit::Policy`](../boarding/deposit.rs) evaluates `domain::payment::Deposit` against boarding [`DepositRule`](../boarding/mod.rs) and [`PaymentTiming`](../boarding/mod.rs). Boarding tests in [`domain/tests/petsuites_core_service_contracts.rs`](../../tests/petsuites_core_service_contracts.rs) show required booking deposits blocking confirmation and paid deposits with references satisfying readiness.
- [`domain::boarding`](../boarding/README.md) documents how deposit rules sit beside capacity, care, cancellation, housekeeping, handoff, and upsell policies. Boarding owns confirmation-time deposit policy; `domain::payment` owns the reusable deposit state value.
- [`domain::reservation`](../reservation/README.md) documents the reservation side of the same aggregate. Payment deposits appear in `domain::entities::Reservation`; reservation statuses and transition reasons remain separate from payment status.
- [`domain::daycare`](../daycare/README.md), [`domain::grooming`](../grooming/mod.rs), [`domain::training`](../training/mod.rs), and [`domain::retail`](../retail/README.md) can all feed payment/review needs without redefining payment primitives. For example, grooming no-show policy and retail POS review can require refund/deposit or manager gates, while payment movement remains outside those domain modules.
- [`app::booking_triage`](../../../app/src/booking_triage.rs) checks `domain::entities::Reservation::deposit` through its local `ReservationDepositReadiness` helper. Paid, not-required, and manager-waived deposits are treated as satisfied; missing, required, or failed deposits produce `PaymentManagerApproval` and block provider/customer/payment actions.
- [`app::checkout_completion`](../../../app/src/checkout_completion.rs) explicitly blocks `MoveRefundDiscountOrPayment` in checkout-completion packets. Even when checkout evidence is clean, the app workflow may draft follow-up work but does not move payment without a separate approved surface.
- [`app::tools::payment`](../../../app/src/tools.rs) is the app-level payment-provider port. It defines `Gateway`, `Subject`, `CapturePolicy`, `ReviewReason`, `IdempotencyKey`, `authorization::Request`, `refund::Request`, and `deposit::RecordRequest`/`RecordResult`. Those types wrap provider interaction and idempotency; they use `domain::payment::Reference` for already-known payment references and return `domain::payment::DepositStatus` when recording deposits.
- Gingr reservation and payment-ish provider boundaries currently appear as reservation/search/service endpoint wrappers in [`integrations/gingr/src/endpoint/reservations.rs`](../../../integrations/gingr/src/endpoint/reservations.rs), shared endpoint primitives/errors in [`integrations/gingr/src/endpoint/mod.rs`](../../../integrations/gingr/src/endpoint/mod.rs), and transport/config surfaces in [`integrations/gingr/src/transport.rs`](../../../integrations/gingr/src/transport.rs) and [`integrations/gingr/src/config.rs`](../../../integrations/gingr/src/config.rs). There is no dedicated Gingr payment DTO/mapper under [`integrations/gingr/src`](../../../integrations/gingr/src) in the current source.
- Provider-facing portal lookup and reservation-ledger inclusion are represented in [`app::tools::portal`](../../../app/src/tools.rs), especially `tools::portal::Include::ReservationLedger`. Keep provider ledger lookup at the app/integration boundary and promote only validated references/statuses into `domain::payment`.

## Maintainer notes

- Add payment-domain errors only for invariants checked by this module, such as reference shape. Authorization declines, duplicate risk, refund rejection, provider ambiguity, and transport failures belong in [`app::tools::payment`](../../../app/src/tools.rs), Gingr/payment-provider integrations, or app workflow errors.
- Preserve semantic paths in call sites and docs. Prefer `domain::payment::DepositStatus::Paid` and `domain::payment::Reference` over flattened aliases when the payment context matters; `domain::entities::Deposit` is useful only when documenting the reservation aggregate field.
- Do not let provider ids leak into domain as raw `String`s. Boundary code should validate or normalize them into `domain::payment::Reference` when they become durable domain evidence.
- Keep money rules in [`domain::money`](../money/mod.rs), deposit-confirmation rules in [`domain::boarding::deposit`](../boarding/deposit.rs), provider execution in [`app::tools::payment`](../../../app/src/tools.rs), and reusable payment state in this module.
