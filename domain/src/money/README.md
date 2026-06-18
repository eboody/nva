# `domain::money`

`domain::money` is the domain crate's small value module for non-zero monetary amounts. It owns the semantic difference between a raw integer and an amount in currency: [`domain::money::MinorUnits`](./mod.rs) rejects zero, [`domain::money::Currency`](./mod.rs) currently models USD, and [`domain::money::Money`](./mod.rs) pairs the amount with its currency.

Start at [`mod.rs`](./mod.rs). This module intentionally has no service-line policy or provider DTO shape of its own; other modules use it when a service contract or workflow needs a typed amount instead of a naked `u32`.

## Module navigation

- [`mod.rs`](./mod.rs) defines [`domain::money::MinorUnits`](./mod.rs), [`Currency`](./mod.rs), [`Money`](./mod.rs), [`Error`](./mod.rs), and [`Result`](./mod.rs). `MinorUnits::try_new` is the validation boundary for positive minor-unit amounts, and `Money::new` stores a validated amount with a currency.

## Type/module map

| Concept | Public type/module path | Defined in |
| --- | --- | --- |
| Non-zero minor-unit amount | `domain::money::MinorUnits` | [`mod.rs`](./mod.rs) |
| Supported currency code | `domain::money::Currency` | [`mod.rs`](./mod.rs) |
| Monetary value | `domain::money::Money` | [`mod.rs`](./mod.rs) |
| Money validation error/result | `domain::money::Error`, `domain::money::Result` | [`mod.rs`](./mod.rs) |

## Workflow surface

`domain::money` contributes to labor-cost reduction by normalizing source amounts before they enter contracts, deposits, payment prompts, or application tools. A validated `domain::money::Money` value makes review safer because a maintainer can see when a field is money, what unit it uses, and where the zero-amount guard lives.

Current in-repo uses are deliberately narrow:

1. [`domain::boarding::DepositRule`](../boarding/mod.rs) stores required deposits as `domain::money::Money`, and [`domain::boarding::deposit::Policy`](../boarding/deposit.rs) evaluates those rules with payment timing/status, so boarding policy can talk about deposits without inventing a boarding-local amount primitive.
2. [`domain::payment`](../payment/README.md) uses [`domain::money::Money`](./mod.rs) for payment amounts in its request/decision surface.
3. App-level tool and workflow tests create `domain::money::Money` values before composing booking/payment flows, keeping application code on domain values rather than raw cents.

## Cross-crate relationships

- The domain crate exposes this module from [`domain/src/lib.rs`](../lib.rs) as `pub mod money`.
- [`domain::payment`](../payment/README.md) is the closest domain consumer: [`domain/src/payment/mod.rs`](../payment/mod.rs) imports `domain::money::Money` for payment requests and outcomes.
- [`domain::boarding`](../boarding/README.md) uses `domain::money::Money` for deposit policy in [`domain/src/boarding/mod.rs`](../boarding/mod.rs).
- App workflow and tool code reference `domain::money::Money` in [`app/src/tools.rs`](../../../app/src/tools.rs), with composition coverage in [`app/tests/app_service_contracts.rs`](../../../app/tests/app_service_contracts.rs), [`app/tests/workflow_service_composition_contracts.rs`](../../../app/tests/workflow_service_composition_contracts.rs), and [`app/tests/application_quality_patterns.rs`](../../../app/tests/application_quality_patterns.rs).
- Domain quality and service-contract tests cover construction and zero-value rejection in [`domain/tests/domain_quality_patterns.rs`](../../tests/domain_quality_patterns.rs), [`domain/tests/petsuites_core_service_contracts.rs`](../../tests/petsuites_core_service_contracts.rs), and [`domain/tests/service_module_architecture.rs`](../../tests/service_module_architecture.rs).
- `storage::service_line` modules do not currently define a money-specific record; service-line records persist the domain contracts that already contain typed `domain::money::Money` values.
- `integrations/gingr` currently exposes catalog/reservation/commerce surfaces in [`integrations/gingr/src/endpoint`](../../../integrations/gingr/src/endpoint/mod.rs) and retail DTOs in [`integrations/gingr/src/dto/retail.rs`](../../../integrations/gingr/src/dto/retail.rs), but there is no Gingr money DTO or mapper module that promotes provider amounts into `domain::money`.

## Maintainer notes

- Keep `domain::money` as a small value module. Add currency or amount semantics here only when they are shared domain truths, not because one provider payload happens to have a numeric field.
- Preserve the `domain::money::MinorUnits` path in prose and code when the unit matters; it is the source of the positive-amount invariant.
- Provider or storage code should promote raw amount fields into `domain::money::MinorUnits` at the boundary and carry errors with the owning boundary module rather than loosening `domain::money`.
