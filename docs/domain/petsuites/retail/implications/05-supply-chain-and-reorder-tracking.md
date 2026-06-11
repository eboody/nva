# Retail / partner products implication 05: Supply chain and reorder tracking

## Scope and source context

This implication expands the Retail / partner-products service map into the supply-chain and reorder slice. It is a modeling artifact for later Rust/domain implementation, not a live purchasing workflow.

Source surfaces read:

- `docs/domain/petsuites/retail/service-domain-map.md`
- `domain/src/operations.rs`, especially `operations::ServiceOffering::RetailPartnerProduct`, `operations::RetailPartner`, `operations::RetailProductCategory`, and `operations::retail::{Contract, Product, Sku, InventoryPolicy, UnitCount, ReorderPolicy}`.
- `domain/tests/petsuites_core_service_contracts.rs`, especially `retail_contract_encodes_product_pos_inventory_recommendation_and_reorder_rules`.

Safe modeling assumptions:

- Inventory is location-scoped. The same partner product can be in stock, backordered, vendor-managed, or inactive at one PetSuites location while available at another.
- Reorder tracking covers both retail sale stock and in-house consumable stock such as Purina EN used during boarding stays.
- Automation may detect reorder risk, draft an internal task, summarize vendor status, and prepare a purchase-order draft. It does not submit vendor orders, change thresholds, hide stockouts, promise customer availability, or charge customers without review.
- Unknown vendor-specific API details should stay in boundary adapters. The domain core should model typed supply-chain status and reorder decisions, not raw vendor payloads.

## 1. Operational story

### Trigger

A supply-chain/reorder evaluation begins from one or more deterministic facts:

- POS or inventory sync records a `StockMovement` such as a sale, receipt, reservation consumption, adjustment, return, expiry/waste, or transfer.
- A scheduled inventory watch runs for each active `LocationOffering` with tracked inventory.
- A boarding forecast sees upcoming reservations that will consume an in-house diet such as Purina EN.
- A vendor feed or manager entry changes a product's availability, lead time, case pack, minimum order quantity, backorder state, or discontinuation state.
- A staff member or manager asks for a reorder review after observing shelf stock, spoilage, damaged items, or a location-level merchandising change.

### Actors

- Front desk / retail staff: record sales, attach approved sale lines to checkout, report shelf counts, and complete staff review tasks.
- Boarding / kennel staff: record in-house diet consumption and future boarding diet needs.
- Manager: approves reorder tasks, purchase-order drafts, threshold changes, substitutions, comps, and vendor exceptions.
- Vendor / partner adapter: imports catalog, availability, receipt, and order-status facts from Virbac, Purina, POS, or distributor systems.
- Retail inventory agent: reads approved inventory/POS/vendor snapshots, computes typed reorder signals, and creates internal review tasks.
- Retail data-quality agent: detects SKU/category/vendor mismatches and routes cleanup work.

### Inputs

- `entities::LocationId` for the location being evaluated.
- `operations::retail::Sku` and a product/offering identity such as `PartnerProduct` or `LocationOffering`.
- Inventory position: on-hand units, reserved units, reorder threshold, available-for-sale, and optional safety stock.
- Recent `StockMovement` facts with source and idempotency key.
- Reorder policy: manual review, manager-task automation, vendor-managed restock, or future auto-draft policy.
- Supply-chain status: vendor availability, lead time, order state, backorder/discontinued state, and receipt state.
- Forecast demand: upcoming boarding/in-house-diet reservations and expected units consumed per stay or service interval.
- Staff/manager approvals and prior audit entries.

### Decisions

The reorder service or policy decides:

1. Is the offering inventory-tracked at this location?
2. Is the product active, sellable, in-house consumable, or both?
3. Does on-hand or available-for-sale fall below threshold?
4. Will future reservation consumption deplete stock before the next expected receipt date?
5. Is the product vendor-managed, backordered, discontinued, or missing vendor data?
6. Does the action only need a staff task, or does it require manager review?
7. Should the system create no action, create an internal reorder task, draft a vendor order for review, create a data-quality task, or route an exception?
8. Are any customer-facing recommendations or substitutions currently unsafe because the product is unavailable?

### Outputs

- `operations::retail::ReorderSignal`: a typed explanation of the current inventory/supply-chain state.
- `operations::retail::ReorderDecision`: no action, staff task, manager review, purchase-order draft, vendor-managed notice, or blocked exception.
- `operations::StaffTask` / future `StaffTaskKind::RetailReorderReview { location_id, sku }` for reviewable action.
- `operations::retail::PurchaseOrderDraft` for manager/vendor review, never a submitted vendor order.
- `operations::retail::SupplyChainAuditEntry` preserving the facts, policy version, decision, actor/agent, and idempotency key.
- Optional manager brief item warning about projected depletion or vendor risk.
- Optional recommendation/checkout block reason: out-of-stock, backordered, discontinued, or manager substitution required.

### Success state

The location has an auditable, review-gated reorder posture:

- Stock movements reconcile to an `InventoryPosition` whose available units are derived, not hand-maintained.
- Products below threshold or projected to deplete before planned use have internal review tasks or manager-approved order drafts.
- Vendor order state and receipt state can be tracked without pretending inventory arrived before receipt.
- Customer-facing recommendation and checkout flows know when inventory is unavailable and cannot promise unavailable stock.
- Managers can explain why a reorder was proposed, approved, deferred, or blocked.

### Failure and exception states

- Inventory is not tracked for the offering: return `ReorderDecision::NoAction { reason: NotTracked }` and do not create noisy tasks.
- On-hand is zero or below threshold: create a review task or purchase-order draft according to policy, but do not mutate stock or submit an order.
- Reserved units exceed on-hand: reject the position as impossible and create a data-quality task.
- POS/vendor movement has already been applied: ignore via idempotency key and preserve a duplicate-event audit note.
- Vendor reports backorder/discontinuation: route manager review and block customer promises or auto-recommendations.
- Future Purina EN boarding consumption exceeds projected available stock: create manager attention before the stay rather than waiting for threshold crossing.
- Unknown SKU/partner/category mapping: quarantine at the adapter boundary and create retail data-quality cleanup, not a generic product branch.
- Reorder threshold, case pack, or lead time is missing: use manual review as the safe fallback and document the missing fact in the task payload.
- Receipt quantity does not match the draft/order: create partial-receipt or mismatch review, not silent adjustment.
- Any customer/member-facing substitution or product recommendation depends on unavailable stock: require staff/manager approval before customer contact.

## 2. Domain types to add or refine

### Keep and refine existing surface

- `operations::retail::Sku`
  - Existing invariant: trimmed, non-empty.
  - Refine later with max length / allowed character policy only if external POS/vendor reconciliation needs it.
- `operations::retail::Product`
  - Current fields: SKU and parent-level `RetailProductCategory`.
  - Refine into or wrap with `PartnerProduct` before supply-chain rules depend on partner or vendor catalog identity.
- `operations::retail::InventoryPolicy`
  - Current shape: `NotTracked` or `Tracked { on_hand: UnitCount, reorder_at: UnitCount }`.
  - Refine because `UnitCount` is positive-only and cannot express true zero on-hand.
- `operations::retail::ReorderPolicy`
  - Current variants are useful routing seeds: `ManualReview`, `AutoCreateManagerTask`, `VendorManaged`.
  - Refine into a policy object once decisions need lead time, case pack, minimum order quantity, forecast demand, or vendor status.
- `operations::retail::Contract::should_reorder()`
  - Keep as a simple threshold compatibility method while introducing a richer `ReorderService::evaluate(...)` for supply-chain posture.

### New value objects and invariants

- `operations::retail::OnHandUnits`
  - Zero-capable count for stock physically believed to be present.
  - Invariant: `u32` or wider, never negative.
- `operations::retail::ReservedUnits`
  - Zero-capable count allocated to reservations, pending checkout, or in-house consumption.
  - Invariant: never negative; cannot exceed on-hand inside a valid `InventoryPosition`.
- `operations::retail::AvailableUnits`
  - Derived value from `InventoryPosition`, not a persisted mutable input.
  - Invariant: `on_hand - reserved`, never negative in a valid position.
- `operations::retail::ReorderThreshold`
  - Positive threshold distinct from on-hand.
  - Invariant: at least one unit; may be location/offering-specific.
- `operations::retail::SafetyStockUnits`
  - Optional positive or zero floor reserved for operations continuity.
  - Invariant: does not itself prove order quantity.
- `operations::retail::LeadTimeDays`
  - Positive vendor lead-time estimate.
  - Invariant: at least one day when present; unknown lead time is an enum state, not zero.
- `operations::retail::CasePackUnits`
  - Positive vendor order multiple.
  - Invariant: order quantities must be compatible when the vendor requires case packs.
- `operations::retail::OrderQuantity`
  - Positive requested order amount.
  - Invariant: never zero; may be rounded by reorder policy to case-pack multiples.
- `operations::retail::VendorCatalogId`
  - Trimmed non-empty identifier promoted from vendor/POS payloads.
  - Invariant: adapter-specific raw IDs stay outside the domain until parsed.
- `operations::retail::StockMovementId` and `operations::retail::StockMovementIdempotencyKey`
  - Stable IDs for audit and duplicate suppression.
  - Invariant: movement application is idempotent per source/key.
- `operations::retail::PurchaseOrderDraftId`
  - Internal draft identity, not proof of vendor submission.

### New enums and aggregate types

- `operations::retail::PartnerProduct`
  - Fields: SKU, product name, partner/family, category, usage mode, optional vendor catalog ID.
  - Invariant: a product family such as Purina EN must be compatible with its category and usage mode.
- `operations::retail::LocationOffering`
  - Fields: location, partner product, active status, POS policy, inventory policy, reorder policy, optional vendor contract.
  - Invariant: inventory/reorder decisions are scoped to a location offering, not a global SKU alone.
- `operations::retail::UsageMode`
  - Variants: `Sellable`, `InHouseConsumable`, `SellableAndInHouseConsumable`.
  - Invariant: in-house consumption creates stock movements even when no customer sale exists.
- `operations::retail::InventoryPosition`
  - Fields: location, SKU, on-hand, reserved, threshold, optional safety stock, as-of timestamp/source.
  - Invariant: reserved cannot exceed on-hand; available units are derived.
- `operations::retail::StockMovement`
  - Fields: movement ID, location, SKU, kind, quantity, source, occurred-at, idempotency key, optional reservation/customer/order reference.
  - Invariant: every movement has a semantic `StockMovementKind`; raw provider event names are boundary-only.
- `operations::retail::StockMovementKind`
  - Variants: `Received`, `Sold`, `ConsumedDuringStay`, `ReservedForStay`, `ReleasedFromReservation`, `Adjusted`, `ExpiredOrWasted`, `Returned`, `TransferredOut`, `TransferredIn`.
- `operations::retail::SupplyChainStatus`
  - Variants: `Available`, `OrderNeeded`, `OrderDrafted`, `Ordered`, `PartiallyReceived`, `Received`, `Backordered`, `Discontinued`, `VendorManaged`, `Unknown`.
- `operations::retail::ReorderSignal`
  - Variants: `Healthy`, `BelowThreshold`, `ProjectedDepletion`, `OutOfStock`, `Backordered`, `Discontinued`, `VendorManagedReview`, `DataQualityException`.
  - Invariant: includes typed location/SKU/product context and human-readable rationale safe for internal audit.
- `operations::retail::ReorderDecision`
  - Variants: `NoAction`, `CreateStaffTask`, `ManagerReviewRequired`, `DraftPurchaseOrder`, `VendorManagedNotice`, `Blocked`.
  - Invariant: decision distinguishes a draft from an approved/submitted order.
- `operations::retail::PurchaseOrderDraft`
  - Fields: location, vendor/partner, line items, reason, source signal, approval state.
  - Invariant: cannot be treated as submitted unless a later boundary workflow records approved vendor submission.
- `operations::retail::SupplyChainAuditEntry`
  - Fields: actor/agent, policy version, input snapshot references, decision, review gate, timestamps.
  - Invariant: enough context to explain why stock changed, why reorder was proposed, or why action was blocked.

## 3. Relationship map

### Entities and value objects

- `entities::LocationId` owns the location identity. Retail uses it to scope offerings, inventory positions, reorder thresholds, vendor settings, and purchase-order drafts.
- `operations::retail::Sku` identifies the product in retail/POS/vendor context after adapter promotion.
- `operations::retail::PartnerProduct` names the product and partner/family; it should not own location-specific stock.
- `operations::retail::LocationOffering` joins product identity, location, POS policy, inventory policy, reorder policy, and active status.
- `operations::retail::InventoryPosition` is the current stock posture for one location offering.
- `operations::retail::StockMovement` is the auditable event stream that changes or reconciles stock posture.
- `entities::ReservationId` appears on future/actual in-house diet consumption, not on every retail sale.
- `entities::CustomerId` and `entities::PetId` are optional context for sold/reserved/consumed movements and recommendation blocking.

### Policies

- `operations::retail::InventoryPolicy` owns whether stock is tracked and what threshold applies.
- `operations::retail::ReorderPolicy` owns routing: manual review, auto-created manager task, vendor-managed, and future purchase-order drafting rules.
- `operations::retail::SupplyChainPolicy` should own vendor status interpretation, lead-time/case-pack constraints, and whether a signal is blocked or reviewable.
- `operations::retail::PointOfSalePolicy` owns whether stock can become a sale-line draft; it must respect unavailable inventory.
- `operations::retail::RecommendationPolicy` consumes supply-chain status to prevent customer-facing promises for unavailable products.

### Repositories and stores

- `operations::retail::InventoryRepository`
  - Loads inventory positions and movement history.
  - Applies stock movements idempotently.
  - Stores audit entries for adjustments and reconciliations.
- `operations::retail::LocationOfferingRepository`
  - Loads active location offerings, thresholds, vendor settings, and POS/reorder policies.
- `operations::retail::SupplyChainRepository`
  - Loads vendor status, order drafts, order status, receipt state, and lead-time facts.
- `operations::retail::PurchaseOrderDraftRepository`
  - Saves manager-reviewable drafts and records approval/submission linkage after boundary workflows complete.
- Storage adapters may use database tables, POS APIs, vendor APIs, or snapshots, but domain behavior should depend on semantic repository contracts.

### Workflow events and staff tasks

- `operations::retail::StockMovementRecorded` triggers reorder evaluation.
- `operations::retail::ReorderSignalRaised` records why the policy saw risk.
- `operations::retail::PurchaseOrderDrafted` records an internal draft only.
- `operations::retail::ReceiptReconciled` records stock receipt and closes/reduces outstanding risk.
- `operations::StaffTaskKind::RetailReorderReview { location_id, sku }` should be added when implementation needs task-specific routing.
- `operations::StaffTaskKind::RetailInventoryAdjustmentReview { location_id, sku }` should cover impossible positions and manual adjustments.
- Manager brief integration should read the typed signal/decision rather than screen-scraping task text.

### Agent specs and tools

- `retail-inventory-watch`
  - Reads inventory/POS/vendor snapshots and future in-house diet demand.
  - Emits `ReorderSignal`, `ReorderDecision`, internal staff tasks, and manager brief drafts.
  - Forbidden: vendor order submission, threshold mutation, stock mutation without source movement, customer messaging.
- `boarding-diet-forecast`
  - Reads upcoming boarding reservations and Purina EN usage assumptions.
  - Emits `ProjectedDepletion` signals when expected consumption risks stockout.
- `retail-vendor-sync`
  - Reads vendor/distributor order status and availability.
  - Emits typed `SupplyChainStatus` updates and receipt reconciliation candidates.
- `retail-data-quality`
  - Detects unknown SKUs, category mismatches, duplicate vendor IDs, impossible stock, and missing thresholds.
  - Emits cleanup staff tasks, not guessed product mappings.

## 4. Interaction contract

Rust-like pseudo-signatures below intentionally put behavior on truthful owners: positions derive availability, policies decide, repositories persist, services orchestrate.

```rust
pub mod operations::retail {
    pub struct InventoryPosition { /* location, sku, on_hand, reserved, threshold */ }

    impl InventoryPosition {
        pub fn new(
            location_id: entities::LocationId,
            sku: Sku,
            on_hand: OnHandUnits,
            reserved: ReservedUnits,
            reorder_threshold: ReorderThreshold,
        ) -> Result<Self>;

        pub fn available_units(&self) -> AvailableUnits;
        pub fn is_below_threshold(&self) -> bool;
        pub fn with_movement(&self, movement: StockMovement) -> Result<InventoryPosition>;
    }

    pub trait InventoryRepository {
        fn inventory_position(
            &self,
            location_id: entities::LocationId,
            sku: &Sku,
        ) -> Result<Option<InventoryPosition>>;

        fn apply_stock_movement(&mut self, movement: StockMovement) -> Result<InventoryPosition>;
        fn movement_seen(&self, key: &StockMovementIdempotencyKey) -> Result<bool>;
        fn save_audit_entry(&mut self, entry: SupplyChainAuditEntry) -> Result<()>;
    }

    pub trait LocationOfferingRepository {
        fn active_location_offering(
            &self,
            location_id: entities::LocationId,
            sku: &Sku,
        ) -> Result<Option<LocationOffering>>;

        fn active_tracked_offerings(
            &self,
            location_id: entities::LocationId,
        ) -> Result<Vec<LocationOffering>>;
    }

    pub trait SupplyChainRepository {
        fn supply_chain_status(
            &self,
            location_id: entities::LocationId,
            sku: &Sku,
        ) -> Result<SupplyChainStatus>;

        fn save_purchase_order_draft(
            &mut self,
            draft: PurchaseOrderDraft,
        ) -> Result<PurchaseOrderDraftId>;
    }

    pub struct ReorderPolicy;

    impl ReorderPolicy {
        pub fn evaluate(
            &self,
            offering: &LocationOffering,
            position: &InventoryPosition,
            forecast: &DemandForecast,
            vendor: &SupplyChainStatus,
        ) -> ReorderDecision;
    }

    pub struct ReorderService<Offerings, Inventory, SupplyChain> { /* repos */ }

    impl<Offerings, Inventory, SupplyChain> ReorderService<Offerings, Inventory, SupplyChain>
    where
        Offerings: LocationOfferingRepository,
        Inventory: InventoryRepository,
        SupplyChain: SupplyChainRepository,
    {
        pub fn evaluate_location_offering(
            &mut self,
            location_id: entities::LocationId,
            sku: Sku,
            forecast: DemandForecast,
            actor: SupplyChainActor,
        ) -> Result<ReorderEvaluation>;

        pub fn record_stock_movement_and_evaluate(
            &mut self,
            movement: StockMovement,
            forecast: DemandForecast,
            actor: SupplyChainActor,
        ) -> Result<ReorderEvaluation>;
    }
}
```

Expected behavior:

- `InventoryPosition::new` rejects reserved units greater than on-hand units and should return a retail-local semantic error.
- `InventoryPosition::available_units` is the only owner of available-stock arithmetic; call sites should not compute `on_hand - reserved` directly.
- `InventoryRepository::apply_stock_movement` is idempotent by movement key. Duplicate POS/vendor events should not double-apply stock changes.
- `ReorderPolicy::evaluate` returns typed decisions; it does not write staff tasks, vendor orders, or stock movements.
- `ReorderService` orchestrates repository reads/writes, creates audit entries, and asks workflow/task owners to create tasks. It does not bypass policy.
- Purchase-order submission belongs to a future reviewed boundary workflow, not this service.

Example decision mapping:

```rust
match (position.is_below_threshold(), forecast.depletes_before_next_receipt(), vendor) {
    (_, _, SupplyChainStatus::Discontinued) => ReorderDecision::Blocked {
        reason: ReorderBlockReason::Discontinued,
        review_gate: ReviewGate::Manager,
    },
    (_, _, SupplyChainStatus::Backordered { .. }) => ReorderDecision::ManagerReviewRequired {
        signal: ReorderSignal::Backordered,
    },
    (true, _, _) => ReorderDecision::CreateStaffTask {
        task: RetailReorderTaskDraft::from_threshold_breach(...),
    },
    (_, true, _) => ReorderDecision::ManagerReviewRequired {
        signal: ReorderSignal::ProjectedDepletion,
    },
    _ => ReorderDecision::NoAction { reason: ReorderNoActionReason::Healthy },
}
```

## 5. Review and approval contract

### Automation level

Allowed automation:

- Import POS/vendor/inventory snapshots into semantic boundary records.
- Classify stock movements into typed `StockMovementKind` when source mapping is known.
- Calculate inventory position, threshold breach, future in-house diet depletion, and vendor-risk signals.
- Create internal reorder-review or data-quality staff tasks.
- Draft purchase-order line items for manager review.
- Draft manager brief summaries and internal recommendations.

Not allowed without review:

- Submit vendor purchase orders.
- Change reorder thresholds, case packs, lead times, location offering active/inactive status, POS policy, or vendor-managed status.
- Apply manual inventory adjustments that are not backed by POS/vendor receipt facts.
- Substitute products for a customer or boarding guest.
- Send customer-facing availability/recommendation messages.
- Attach a sale line, charge payment, issue refund, or apply comp.

### Review gates

- Staff review gate:
  - Shelf count discrepancy, routine below-threshold reorder task, unknown movement source that can be reconciled with local evidence.
- Manager review gate:
  - Purchase-order draft approval, threshold/policy changes, backorder/discontinued substitutions, comp/discount/refund impact, partial receipt mismatch, customer complaint context, or anything that could look like veterinary/dietary advice.
- Vendor/boundary review gate:
  - Submitted order acknowledgement, receipt confirmation, external order cancellation, and vendor-managed restock status must come from explicit vendor/POS/manager facts.

### Audit trail

Every reorder evaluation that creates a task, draft, block, or inventory change should record:

- Location, SKU, product/partner, and offering version.
- Inventory snapshot and forecast snapshot used.
- Vendor status and source timestamp.
- Policy version and decision variant.
- Actor: staff, manager, adapter, or named agent.
- Review gate and approval outcome.
- Idempotency key for source movements or vendor events.
- Links to staff task, purchase-order draft, receipt, or stock movement.

### Customer/member-facing boundaries

- Customers should not see low-stock promises, substitutions, or recommendations until staff/manager approval.
- A product being reorderable does not mean it is safe to recommend or sell.
- A boarding guest's in-house diet need may influence internal stock planning, but customer messaging about diet or supplement changes must remain reviewed and care-profile-safe.
- Out-of-stock/backordered/discontinued states should block customer-facing upsell drafts or force explicit staff copy review.

## 6. Test contracts

Domain tests:

- `retail_inventory_position_allows_zero_on_hand_but_rejects_reserved_units_above_on_hand`
  - Proves zero stock is representable and impossible availability is rejected.
- `retail_inventory_position_derives_available_units_without_call_site_arithmetic`
  - Proves available units are owned by `InventoryPosition`.
- `retail_stock_movement_application_is_idempotent_by_source_key`
  - Proves duplicate POS/vendor events do not double decrement or increment stock.
- `retail_stock_movement_kind_distinguishes_sale_from_boarding_diet_consumption`
  - Proves in-house Purina EN consumption is not modeled as a customer sale.
- `retail_reorder_policy_routes_below_threshold_stock_to_reviewable_staff_task`
  - Proves a threshold breach creates typed internal action rather than direct vendor order.
- `retail_reorder_policy_projects_purina_en_depletion_from_future_boarding_stays`
  - Proves future boarding demand can trigger manager attention before current stock is below threshold.
- `retail_reorder_policy_blocks_customer_promises_when_vendor_reports_backorder`
  - Proves vendor risk affects recommendation/checkout boundaries.
- `retail_purchase_order_draft_is_not_a_submitted_vendor_order`
  - Proves draft, approval, and submission are distinct states.
- `retail_supply_chain_status_discontinued_requires_manager_review_and_blocks_auto_reorder`
  - Proves discontinuation is a semantic exception, not a threshold reorder.
- `retail_reorder_threshold_and_order_quantity_are_distinct_domain_values`
  - Proves threshold, on-hand count, case pack, and order quantity cannot be swapped accidentally.

Storage/boundary tests:

- `retail_inventory_position_records_roundtrip_zero_on_hand_and_positive_threshold`
  - Proves storage codecs preserve zero stock and threshold invariants.
- `retail_stock_movement_records_reject_negative_quantities_and_unknown_kind`
  - Proves raw provider payloads cannot create impossible movements.
- `retail_vendor_status_records_promote_raw_payloads_to_supply_chain_status`
  - Proves adapters translate external statuses before domain policies run.
- `retail_purchase_order_drafts_persist_review_state_without_vendor_submission_claim`
  - Proves persistence does not erase approval boundaries.
- `retail_location_offering_records_scope_thresholds_by_location_and_sku`
  - Proves reorder thresholds are not global SKU facts.

Workflow/agent tests:

- `retail_inventory_watch_agent_can_create_reorder_review_task_but_not_submit_order`
  - Proves agent permissions and output schema enforce safe automation.
- `boarding_diet_forecast_agent_emits_projected_depletion_signal_for_purina_en`
  - Proves future in-house diet use creates internal manager attention.
- `retail_vendor_sync_agent_records_backorder_without_customer_message`
  - Proves vendor facts do not leak into unreviewed customer communication.
- `retail_data_quality_agent_quarantines_unknown_sku_partner_mapping`
  - Proves extensibility without raw string branching in domain policy.

Regression tests to preserve:

- Existing `retail_contract_encodes_product_pos_inventory_recommendation_and_reorder_rules` should continue to pass or be split into more semantic tests once `InventoryPosition` and `ReorderDecision` land.
- Existing `operations::retail::Contract::should_reorder()` compatibility can remain for simple threshold checks while richer service tests cover forecast/vendor states.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Existing retail module lives here. A later card may either extend this module first or split it into `domain/src/operations/retail/*.rs` if the file becomes too large.
  - Add/refine `OnHandUnits`, `ReservedUnits`, `AvailableUnits`, `ReorderThreshold`, `OrderQuantity`, `InventoryPosition`, `StockMovement`, `SupplyChainStatus`, `ReorderSignal`, `ReorderDecision`, and `PurchaseOrderDraft`.
- `domain/src/entities.rs`
  - Reuse existing `LocationId`, `ReservationId`, `CustomerId`, and `PetId`; avoid retail-specific duplicate IDs unless the concept is truly new.
- `domain/src/workflow.rs` and/or `domain/src/agents.rs`
  - Add typed output/review contracts for retail inventory watch and vendor sync agents only after the domain payloads exist.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Preserve current contract tests and add semantic inventory/reorder tests.
- `domain/tests/domain_quality_patterns.rs`
  - Add doctrine-oriented tests around newtypes, enum decisions, agent boundaries, and customer-facing review gates.
- Storage crate/tests, if present in the implementation card:
  - Add inventory position, stock movement, vendor status, and purchase-order draft roundtrip tests.

### Migration and refactor risks

- `UnitCount` currently rejects zero. Reusing it for `on_hand` will make out-of-stock state unrepresentable; introduce zero-capable `OnHandUnits` instead of weakening `UnitCount` globally.
- `RetailPartner` currently encodes product families (`VirbacCalmCare`, `PurinaProPlanVeterinarySupplements`, `PurinaEnBoardingDiet`) rather than partner companies. A code card should decide whether to keep it as a service-offering enum and add `operations::retail::{Partner, ProductFamily}` for behavior.
- `RetailProductCategory` lives at the parent `operations` level. Behavior-heavy retail code should prefer `operations::retail::Category` or a canonical re-export rather than scattering parent-level category checks.
- `Contract::should_reorder()` returns `bool`, which is too lossy for forecast/vendor workflows. Keep it as compatibility sugar while adding `ReorderDecision` tests and call sites.
- Purchase-order drafts must not be serialized or named as submitted orders. State names should make approval/submission boundaries impossible to miss.
- Vendor/POS payload adapters may be tempted to persist raw status strings. Force promotion to `SupplyChainStatus` and semantic errors at the boundary.
- Inventory reconciliation can affect money/POS, reservation, and care workflows. Keep the core event/domain contracts deterministic before tool integration.

### Dependencies on other implications and service maps

- Depends on the retail service-domain-map vocabulary for partner product, inventory position, reorder signal, recommendation boundary, POS policy, and agent contracts.
- Interacts with boarding implications for Purina EN in-house diet forecasting and reservation consumption.
- Interacts with recommendation/personalized-upsell retail implications because supply-chain state must block or review customer-facing promises.
- Interacts with POS/checkout retail implications because sale-line drafts and inventory movements are related but not the same concept.
- Interacts with care-profile / medical-sensitivity rules because diet/supplement substitutions can be customer/member-facing and must remain reviewed.

### Suggested implementation order

1. Add semantic count/value types and `InventoryPosition` with tests for zero on-hand, reserved <= on-hand, and derived availability.
2. Add `StockMovement` and idempotency contracts without external adapters.
3. Add `SupplyChainStatus`, `ReorderSignal`, and `ReorderDecision` enum contracts.
4. Replace or supplement `Contract::should_reorder()` with `ReorderPolicy::evaluate(...)` while keeping existing tests green.
5. Add repository traits after the domain values are stable.
6. Add workflow/staff-task/agent payloads after policy decisions are typed.
7. Add storage/boundary codecs and vendor/POS adapters last.
