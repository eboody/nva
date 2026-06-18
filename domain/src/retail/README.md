# `domain::retail`

`domain::retail` is the domain crate's model for retail products inside pet-resort operations. It owns the semantic retail concepts that are not just provider payloads or database records: product identity and categories, per-location offerings, POS sale policy, inventory positions, recommendation review policy, reorder decisions, and vendor/catalog relationships.

Start at [`mod.rs`](./mod.rs). It declares the module surface, re-exports the common product/vendor types, defines the module-local [`domain::retail::Error`](./mod.rs), and collects the retail sub-policies into [`domain::retail::Contract`](./mod.rs). `Contract::standard_petsuites` is a fixture-like standard contract, not an exhaustive catalog: it wires one `Product`, POS policy, inventory policy, recommendation rule, and reorder policy together.

## Module navigation

- [`product.rs`](./product.rs) defines product identity and location-specific retail offerings: [`domain::retail::product::Sku`](./product.rs), [`Name`](./product.rs), [`Category`](./product.rs), [`Product`](./product.rs), [`OfferingStatus`](./product.rs), [`Usage`](./product.rs), and [`LocationOffering`](./product.rs). `LocationOffering::can_be_sold_to_customer` and `LocationOffering::has_available_sale_units` are the saleability checks used by POS policy.
- [`inventory.rs`](./inventory.rs) defines count/value objects and stock positions: [`UnitCount`](./inventory.rs), [`OnHandUnits`](./inventory.rs), [`ReservedUnits`](./inventory.rs), [`AvailableUnits`](./inventory.rs), [`Stock`](./inventory.rs), [`Position`](./inventory.rs), [`Policy`](./inventory.rs), and [`Availability`](./inventory.rs). `Position::record` rejects reserved units greater than on-hand units before computing availability.
- [`pos.rs`](./pos.rs) defines checkout policy: [`domain::retail::pos::Policy`](./pos.rs), [`Quantity`](./pos.rs), [`Request`](./pos.rs), [`Source`](./pos.rs), [`PriceAdjustment`](./pos.rs), and [`Decision`](./pos.rs). It distinguishes standalone staff sales, reservation-checkout attachments, external reconciliation, inventory denial, and manager/customer-message review gates.
- [`recommendation.rs`](./recommendation.rs) defines internal recommendation candidates and review/suppression policy: [`Rule`](./recommendation.rs), [`Candidate`](./recommendation.rs), [`Reason`](./recommendation.rs), [`CareSensitivity`](./recommendation.rs), [`Preference`](./recommendation.rs), [`Policy`](./recommendation.rs), [`Decision`](./recommendation.rs), and nested [`rationale::Text`](./recommendation.rs) / [`customer_copy::SafeCopy`](./recommendation.rs). Customer-facing copy is separately checked for medical-claim language before it can become an approved message draft.
- [`reorder.rs`](./reorder.rs) defines reorder automation: [`domain::retail::reorder::Policy`](./reorder.rs), [`Decision`](./reorder.rs), and [`Reason`](./reorder.rs). `Policy::evaluate` only acts when an [`inventory::Position`](./inventory.rs) is at or below its reorder threshold; it then produces either no action, a manager review, a staff task, or a vendor-managed notice.
- [`vendor.rs`](./vendor.rs) defines the current partner catalog vocabulary: [`domain::retail::vendor::Partner`](./vendor.rs) and [`CatalogRelationship`](./vendor.rs).

## Type/module map

| Concept | Public type/module path | Defined in |
| --- | --- | --- |
| Retail module contract | `domain::retail::Contract` | [`mod.rs`](./mod.rs) |
| Retail module error/result | `domain::retail::Error`, `domain::retail::Result` | [`mod.rs`](./mod.rs) |
| Product identity | `domain::retail::Product`, `domain::retail::Sku`, `domain::retail::product::Name` | [`product.rs`](./product.rs) |
| Product category/status/usage | `domain::retail::product::Category`, `domain::retail::OfferingStatus`, `domain::retail::product::Usage` | [`product.rs`](./product.rs) |
| Per-location retail offering | `domain::retail::LocationOffering` | [`product.rs`](./product.rs) |
| Inventory counts | `domain::retail::inventory::UnitCount`, `OnHandUnits`, `ReservedUnits`, `AvailableUnits` | [`inventory.rs`](./inventory.rs) |
| Inventory stock/position | `domain::retail::inventory::Stock`, `domain::retail::inventory::Position` | [`inventory.rs`](./inventory.rs) |
| Inventory policy/availability | `domain::retail::inventory::Policy`, `domain::retail::inventory::Availability` | [`inventory.rs`](./inventory.rs) |
| POS sale request | `domain::retail::pos::Request`, `Quantity`, `Source`, `PriceAdjustment` | [`pos.rs`](./pos.rs) |
| POS sale outcome | `domain::retail::pos::Decision`, `ReviewReason`, `DenialReason` | [`pos.rs`](./pos.rs) |
| Recommendation candidate | `domain::retail::recommendation::Candidate` | [`recommendation.rs`](./recommendation.rs) |
| Recommendation policy/outcome | `domain::retail::recommendation::Policy`, `Decision`, `ReviewReason`, `SuppressionReason` | [`recommendation.rs`](./recommendation.rs) |
| Recommendation text values | `domain::retail::recommendation::rationale::Text`, `customer_copy::SafeCopy` | [`recommendation.rs`](./recommendation.rs) |
| Reorder policy/outcome | `domain::retail::reorder::Policy`, `Decision`, `Reason` | [`reorder.rs`](./reorder.rs) |
| Vendor/catalog relationship | `domain::retail::Partner`, `domain::retail::vendor::CatalogRelationship` | [`vendor.rs`](./vendor.rs) |

## Retail opportunity workflow

The labor-cost-reduction surface is split across recommendations and reorders:

1. A provider retail item or internal catalog entry becomes a semantic [`Product`](./product.rs) / [`LocationOffering`](./product.rs). The domain type keeps saleability, usage, POS policy, inventory policy, and reorder policy in one local offering rather than scattering those decisions through checkout code.
2. [`domain::retail::recommendation::Policy`](./recommendation.rs) evaluates a [`Candidate`](./recommendation.rs) using customer preference, inventory availability, and care sensitivity. Safe candidates can become internal drafts; care-sensitive products route to staff or manager review gates instead of relying on ad hoc staff judgment.
3. [`domain::retail::recommendation::customer_copy::Policy`](./recommendation.rs) rejects customer-facing copy that contains medical-claim language and otherwise requires customer-message approval. That supports safer review before any upsell message reaches a customer.
4. [`domain::retail::reorder::Policy`](./reorder.rs) evaluates an [`inventory::Position`](./inventory.rs). At-threshold inventory can automatically create a staff task, require manager review, or emit a vendor-managed notice. This turns stock exceptions into explicit work items instead of requiring managers to manually scan inventory reports.

The module does not claim to send customer messages, place vendor orders, or reconcile payments itself. It supplies typed decisions and review gates that application/storage/integration layers can execute.

## Cross-crate relationships

- The domain crate exposes this module from [`domain/src/lib.rs`](../lib.rs) as `pub mod retail`.
- `domain::operations::ServiceOffering::RetailPartnerProduct` links retail partner/category values into the broader service-offering model in [`domain/src/operations.rs`](../operations.rs). That variant carries [`domain::retail::Partner`](./vendor.rs) and [`domain::retail::product::Category`](./product.rs).
- `storage::service_line::retail` persists migrated retail contracts and code values in [`storage/src/service_line/retail.rs`](../../../storage/src/service_line/retail.rs). It wraps [`domain::retail::Contract`](./mod.rs) as `ContractRecord` and converts between storage codes and `domain::retail::Partner` / `domain::retail::product::Category`.
- `storage::operations::ServiceOfferingRecord` stores retail service offerings as `retail_partner` and `retail_product_category` in [`storage/src/operations.rs`](../../../storage/src/operations.rs), with shape checks that keep retail fields off boarding/daycare/grooming/training variants.
- Gingr retail DTOs live in [`integrations/gingr/src/dto/retail.rs`](../../../integrations/gingr/src/dto/retail.rs). [`dto::retail::Item`](../../../integrations/gingr/src/dto/retail.rs) preserves provider fields such as `id`, `name`, `sku`, `category`, `active`, `quantity_on_hand`, and unknown fields.
- Gingr retail endpoint wrappers live in [`integrations/gingr/src/endpoint/commerce_retail.rs`](../../../integrations/gingr/src/endpoint/commerce_retail.rs). They cover `get_all_retail_items`, subscriptions, legacy transaction lookup, invoice listing, and payment-sensitive transaction lookup; they are provider boundary requests, not domain policy.
- Gingr retail mapping lives in [`integrations/gingr/src/mapping/retail.rs`](../../../integrations/gingr/src/mapping/retail.rs). `mapping::retail::product_candidate` promotes a provider [`dto::retail::Item`](../../../integrations/gingr/src/dto/retail.rs) into a `ProductCandidate` with `domain::retail::product::Name`, `domain::retail::Sku`, `domain::retail::Product`, and `domain::retail::OfferingStatus`.
- Contract coverage exists in storage tests such as [`storage/tests/core_service_contract_storage.rs`](../../../storage/tests/core_service_contract_storage.rs) and [`storage/tests/operations_storage_contracts.rs`](../../../storage/tests/operations_storage_contracts.rs), and Gingr endpoint/mapping coverage lives in [`integrations/gingr/tests/endpoint_contracts.rs`](../../../integrations/gingr/tests/endpoint_contracts.rs) and [`integrations/gingr/tests/expanded_endpoint_contracts.rs`](../../../integrations/gingr/tests/expanded_endpoint_contracts.rs).

## Maintainer notes

- Keep provider-specific retail payload details in `integrations/gingr` DTOs/mappers. Promote only validated, semantically named values into `domain::retail`.
- Add new retail rules where the domain concept owns the decision: POS sale gating in [`pos.rs`](./pos.rs), stock availability and thresholds in [`inventory.rs`](./inventory.rs), customer/internal upsell review in [`recommendation.rs`](./recommendation.rs), and reorder actions in [`reorder.rs`](./reorder.rs).
- When adding storage variants, keep conversions explicit in [`storage/src/service_line/retail.rs`](../../../storage/src/service_line/retail.rs) or [`storage/src/operations.rs`](../../../storage/src/operations.rs) so database codes do not leak into domain call sites.
- Preserve the module-qualified names in prose and code. `domain::retail::recommendation::Decision` and `domain::retail::reorder::Decision` are intentionally different decisions even though both use the leaf name `Decision`.
