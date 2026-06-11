# Retail / partner products implication: Inventory

## Scope and modeling posture

This implication turns the retail service-domain map into a concrete inventory contract for PetSuites partner products. It is a modeling artifact for later serialized Rust work, not an implementation patch.

Inventory belongs to `operations::retail` because stock position, movement, threshold, forecast, and reorder decisions are retail/product operations. Neighboring modules provide identities and facts:

- `entities` supplies `LocationId`, `CustomerId`, `PetId`, `ReservationId`, and staff/manager identities.
- `care` owns feeding instructions, allergies, medication, medical-condition, veterinarian, and other sensitive care facts.
- `reservation` owns stay/check-in/check-out lifecycle and any customer authorization for add-ons.
- `money` / `payment` own price, charges, refunds, payment capture, and payment references.
- `workflow`, `agents`, and `operations::StaffTask` own routed review work; retail supplies typed inventory payloads and decisions.

Assumption: the first supported inventory surface must handle Virbac CalmCare, Purina Pro Plan Veterinary Supplements, and Purina EN boarding diet. The model should allow future products, locations, vendors, and POS systems without raw-string branching in the domain core.

## 1. Operational story

### Trigger

Inventory evaluation can be triggered by any of these events:

1. POS sale or reservation-checkout sale consumes sellable units.
2. Boarding check-in/prep reserves expected in-house Purina EN units for a stay.
3. Boarding care execution consumes reserved in-house diet units during the stay.
4. Vendor receipt increases on-hand units.
5. Staff inventory count, waste/expiry report, transfer, or return creates an adjustment candidate.
6. Scheduled inventory watcher compares current positions against thresholds, vendor status, and upcoming reservations.
7. Product recommendation workflow asks whether a SKU is currently safe to suggest or promise.

### Actors

- Front desk: sees stock warnings, reserves/sells retail products, attaches reviewed sale drafts to checkout.
- Kennel technician: consumes in-house diet during boarding care and flags diet stock discrepancies.
- Lead staff / manager: approves manual adjustments, reorder tasks, vendor-order drafts, substitutions, comps, and policy threshold changes.
- POS/vendor/storage adapters: translate external SKU, receipt, sale, and inventory snapshots into semantic retail movements.
- `retail-inventory-watch` agent: reads inventory snapshots and emits deterministic reorder/review candidates, never mutates stock by itself.
- `boarding-diet-forecast` agent: reads future reservations and Purina EN usage expectations, emits projected depletion risk.
- `retail-recommendation-drafter` agent: reads inventory availability before drafting product recommendations.

### Inputs

- `operations::retail::Sku` and product/catalog identity.
- `entities::LocationId`, because inventory is location-scoped.
- Current inventory position: on-hand, reserved, available-for-sale, reorder threshold, optional reorder target, and vendor state.
- Requested inventory movement: receipt, sale, reservation hold, consumption, release, adjustment, expiry/waste, return, or transfer.
- Movement source: POS transaction, vendor receipt, reservation, staff count, workflow event, or agent-drafted review packet.
- Future reservation demand for in-house diet SKUs, especially Purina EN.
- Review context: staff role, manager approval, idempotency key, and audit actor.
- Optional recommendation context: customer, pet, reservation, care-profile review flags, and product promise audience.

### Decisions

- Is this SKU active and stocked at the requested location?
- Is the product inventory-tracked, not tracked, or vendor-managed?
- Does the movement preserve inventory invariants: no negative on-hand, no reserved units greater than on-hand, no sale promise above available units, no consumption without a reservation/source?
- Does the movement require staff or manager review before commit?
- Does the position produce a reorder signal: below threshold, projected depletion, vendor-managed review, backorder, discontinued, or healthy?
- Should a reorder signal create an internal task, draft a vendor order, or only appear in a manager brief?
- Is inventory availability sufficient for a product recommendation or customer-facing draft?
- Does in-house diet usage reserve/consume operational inventory rather than create a customer sale line?

### Outputs

- Updated `operations::retail::InventoryPosition` after an approved and idempotent movement.
- Immutable `operations::retail::StockMovement` audit record.
- `operations::retail::InventoryAvailability` for recommendation/POS policy.
- `operations::retail::ReorderSignal` and `operations::retail::ReorderDecision`.
- Internal `operations::StaffTask` draft for reorder review, inventory adjustment review, stock discrepancy, or projected depletion.
- Workflow event for downstream manager brief, recommendation review, or POS/vendor reconciliation.
- Denial/review result when a movement would violate invariants or approval gates.

### Success state

A successful inventory operation leaves the location/SKU position truthful and auditable:

- on-hand is zero or positive;
- reserved is zero or positive and never greater than on-hand;
- available-for-sale is derived, not stored as mutable truth;
- every committed stock change has a semantic movement kind, source, actor, timestamp, and idempotency key where sourced externally;
- customer-facing recommendations and sale-line drafts only see inventory states that can be promised under policy;
- reorder/review work is routed to staff/manager without creating vendor orders or payment actions autonomously.

### Failure and exception states

- `UnknownSku`: external SKU cannot be promoted to `operations::retail::Sku` or no catalog item exists.
- `InactiveLocationOffering`: SKU exists but is inactive/unavailable at the location.
- `InventoryNotTracked`: movement attempted for a product whose policy says not tracked; adapter should either no-op with audit or route data-quality review.
- `InsufficientAvailableUnits`: sale, reservation hold, or recommendation promise exceeds available units.
- `ReservedUnitsExceedOnHand`: invalid position or movement would make reserved greater than on-hand.
- `NegativeStockRejected`: external POS/vendor payload would drive on-hand below zero.
- `DuplicateMovement`: idempotency key already applied; return existing movement/position.
- `MovementRequiresReview`: manual adjustment, discrepancy, substitution, comp-linked movement, or vendor-order action needs staff/manager approval.
- `VendorBackordered` / `Discontinued`: stock cannot be replenished through normal reorder policy.
- `CareSensitivePromiseRejected`: recommendation flow tried to promise a diet/supplement when care profile review or stock state requires staff review.
- `AuditActorMissing`: stock mutation lacks a staff/system actor and must be rejected at the boundary.

## 2. Domain types to add or refine

### Refine existing retail inventory types

- `operations::retail::UnitCount`
  - Keep as a positive count for quantities that must be at least one: reorder thresholds, order quantities, sale quantities, receipt quantities.
  - Do not use it for on-hand when zero stock is a valid state.

- `operations::retail::InventoryPolicy`
  - Current shape: `NotTracked | Tracked { on_hand: UnitCount, reorder_at: UnitCount }`.
  - Refine toward policy, not live state:
    - `NotTracked`
    - `Tracked { reorder_at: ReorderThreshold, reorder_target: Option<OrderQuantity> }`
    - `VendorManaged { review_cadence: VendorReviewCadence }`
  - Live stock should move to `InventoryPosition`.

- `operations::retail::Contract::should_reorder()`
  - Current boolean method is useful but too lossy for inventory operations.
  - Replace/refine with `InventoryPosition::reorder_signal(&self, policy, forecast, vendor_status) -> ReorderSignal` or `ReorderPolicy::evaluate(...) -> ReorderDecision`.

### New value objects and invariants

- `operations::retail::OnHandUnits`
  - Zero-capable `u32` newtype.
  - Invariant: cannot be negative; represents physical/location-owned units before subtracting reservations.

- `operations::retail::ReservedUnits`
  - Zero-capable `u32` newtype.
  - Invariant: cannot be negative; must not exceed `OnHandUnits` in an `InventoryPosition`.

- `operations::retail::AvailableUnits`
  - Derived zero-capable `u32` newtype.
  - Invariant: `on_hand - reserved`; never directly deserialized as authoritative domain truth.

- `operations::retail::ReorderThreshold`
  - Positive `u32` newtype.
  - Invariant: threshold greater than zero; semantically distinct from on-hand count.

- `operations::retail::OrderQuantity`
  - Positive `u32` newtype.
  - Invariant: order/receipt/sale movement quantities are at least one.

- `operations::retail::InventoryMovementId`
  - Stable domain identity for committed stock movement records.

- `operations::retail::InventorySyncKey`
  - Validated non-empty idempotency key from POS/vendor/reservation/tool boundary.
  - Invariant: trimmed, non-empty, scoped by source system and location/SKU.

- `operations::retail::VendorLeadTimeDays`
  - Positive bounded value if forecast policy needs vendor lead-time math.

- `operations::retail::InventoryAdjustmentReason`
  - Validated internal note or enum-backed reason; should not be a raw free-text hole for sensitive customer/pet details.

### New enums

- `operations::retail::StockMovementKind`
  - `Received`
  - `Sold`
  - `ReservedForStay`
  - `ConsumedDuringStay`
  - `ReleasedFromReservation`
  - `AdjustedAfterCount`
  - `ExpiredOrWasted`
  - `Returned`
  - `TransferredOut`
  - `TransferredIn`

- `operations::retail::StockMovementSource`
  - `PosTransaction { external_id }`
  - `VendorReceipt { external_id }`
  - `Reservation { reservation_id: entities::ReservationId }`
  - `StaffCount { staff_id: entities::StaffId }`
  - `WorkflowEvent { event_id: workflow::WorkflowEventId }`
  - `AgentDraft { spec: agent::Name }`

- `operations::retail::InventoryAvailability`
  - `InStock { available: AvailableUnits }`
  - `LowStock { available: AvailableUnits, threshold: ReorderThreshold }`
  - `OutOfStock`
  - `ReservedOnly`
  - `Backordered`
  - `Discontinued`
  - `Unknown`

- `operations::retail::ReorderSignal`
  - `Healthy`
  - `BelowThreshold { available, threshold }`
  - `ProjectedDepletion { forecast }`
  - `Backordered`
  - `VendorManagedReview`
  - `Discontinued`

- `operations::retail::ReorderDecision`
  - `NoAction`
  - `CreateStaffTask { task: operations::StaffTask }`
  - `DraftVendorOrder { draft: VendorOrderDraft, review: policy::ReviewGate }`
  - `ManagerReviewRequired { reason: ReorderReviewReason }`
  - `VendorManaged`

- `operations::retail::InventoryApprovalDecision`
  - `Allowed`
  - `StaffReviewRequired { reason }`
  - `ManagerReviewRequired { reason }`
  - `Rejected { reason }`

- `operations::retail::InventoryError`
  - Module-local semantic error enum for the exception states listed above.

### New entities / aggregates

- `operations::retail::InventoryPosition`
  - Fields: `location_id`, `sku`, `on_hand`, `reserved`, `reorder_threshold`, `vendor_status`, `updated_at`.
  - Invariants: location/SKU required; reserved <= on_hand; available derived; threshold positive.
  - Behavior owner for local stock math and availability classification.

- `operations::retail::StockMovement`
  - Fields: movement id, location, SKU, kind, quantity, source, actor, occurred_at, optional reservation/customer/pet references, optional idempotency key.
  - Invariants: movement quantity positive; kind/source pair must be coherent; committed movement is immutable.

- `operations::retail::InventoryForecast`
  - Fields: location, SKU, forecast window, starting position, expected reservation holds/consumption, expected receipts, depletion date/risk.
  - Invariants: forecast derives from typed demand and receipts; does not mutate inventory.

- `operations::retail::VendorOrderDraft`
  - Fields: location, SKU(s), quantities, vendor, rationale, source signal, review gate.
  - Invariant: draft is not a submitted purchase order.

- `operations::retail::InventoryReviewPacket`
  - Staff-facing explanation bundle for low stock, discrepancy, adjustment, substitution, or projected depletion.
  - Invariant: safe for internal staff/manager review; customer-facing content must be separately drafted and approved.

## 3. Relationship map between types

### Entities

- `operations::retail::PartnerProduct` / `CatalogItem` identifies what the product is.
- `operations::retail::LocationOffering` identifies whether a product can be stocked, sold, recommended, or consumed at one `entities::LocationId`.
- `operations::retail::InventoryPosition` is the live location/SKU stock aggregate.
- `operations::retail::StockMovement` is the immutable audit event that changes position.
- `operations::retail::InventoryForecast` is a computed view over future demand, not an aggregate that owns stock.
- `operations::retail::VendorOrderDraft` is a reviewable draft, not a vendor-side order.

### Value objects

- Identity: `retail::Sku`, `retail::InventoryMovementId`, `retail::InventorySyncKey`, `retail::VendorCatalogId`.
- Counts: `retail::OnHandUnits`, `retail::ReservedUnits`, `retail::AvailableUnits`, `retail::ReorderThreshold`, `retail::OrderQuantity`.
- Explanation: `retail::InventoryAdjustmentReason`, `retail::ReorderReviewReason`, `retail::InventoryReviewRationale`.

### Policies

- `retail::InventoryPolicy` owns whether a product is tracked and what threshold/target applies.
- `retail::ReorderPolicy` owns how a signal routes: no action, staff task, manager review, vendor-managed, or vendor-order draft.
- `retail::PointOfSalePolicy` owns whether sale/checkout/comp movement may be drafted.
- `retail::RecommendationPolicy` consumes `InventoryAvailability` so it cannot promise unavailable products.

### Repositories and stores

- `retail::InventoryRepository` stores positions, movements, idempotency keys, and inventory review packets.
- `retail::CatalogRepository` or parent `retail::Repository` stores catalog items and location offerings.
- `retail::ForecastRepository` is optional if forecasts need persistence; otherwise forecast can be computed from reservation and inventory repositories.
- `retail::RecommendationRepository` reads inventory availability but does not own stock mutation.

### Workflow events

- `RetailStockMovementRecorded`
- `RetailInventoryBelowThreshold`
- `RetailInventoryProjectedDepletion`
- `RetailInventoryAdjustmentReviewRequested`
- `RetailVendorOrderDrafted`
- `RetailInventoryAvailabilityChanged`

These events should carry typed location/SKU/product references and avoid generic JSON blobs.

### Staff tasks

Suggested `operations::StaffTaskKind` extensions:

- `RetailReorderReview { location_id, sku }`
- `RetailInventoryAdjustmentReview { location_id, sku }`
- `RetailStockDiscrepancyReview { location_id, sku }`
- `RetailSubstitutionReview { reservation_id, sku }`
- `RetailVendorOrderReview { location_id }`

Task assignment defaults:

- reorder review: `StaffRole::Manager` or `StaffRole::LeadStaff` by location policy;
- count discrepancy: `StaffRole::LeadStaff`;
- reservation substitution: `StaffRole::FrontDesk` plus manager review if customer-facing or diet-sensitive;
- in-house diet depletion risk: `StaffRole::Manager` with boarding context.

### Agent specs and tools

- `retail-inventory-watch`
  - Reads: catalog/location offerings, inventory positions, stock movements, POS/vendor snapshots.
  - Emits: `ReorderSignal`, `InventoryReviewPacket`, internal staff task drafts.
  - Forbidden: stock mutation, customer messages, vendor purchase orders, payment actions.

- `boarding-diet-forecast`
  - Reads: future boarding reservations with in-house diet needs and Purina EN inventory position.
  - Emits: `InventoryForecast`, `ProjectedDepletion` signals, manager-review tasks.
  - Forbidden: changing feeding instructions, substituting diets, customer messages.

- `retail-data-quality`
  - Reads: unknown SKUs, external product names, category/partner mismatches.
  - Emits: cleanup tasks and proposed semantic mappings.
  - Forbidden: automatically adding active sellable products without review.

- `retail-recommendation-drafter`
  - Reads: availability classification from inventory policy.
  - Emits: staff-review recommendation drafts only when inventory is safe to discuss.
  - Forbidden: customer send, charge, sale, medical claim, diet change.

## 4. Interaction contract

Rust-like pseudo-signatures below are intended as domain contracts, not exact implementation code.

### Inventory position owns stock math

```rust
impl operations::retail::InventoryPosition {
    pub fn new(
        location_id: entities::LocationId,
        sku: operations::retail::Sku,
        on_hand: operations::retail::OnHandUnits,
        reserved: operations::retail::ReservedUnits,
        reorder_threshold: operations::retail::ReorderThreshold,
        vendor_status: operations::retail::SupplyChainStatus,
    ) -> operations::retail::Result<Self>;

    pub fn available_for_sale(&self) -> operations::retail::AvailableUnits;

    pub fn availability(&self) -> operations::retail::InventoryAvailability;

    pub fn can_reserve(&self, quantity: operations::retail::OrderQuantity) -> bool;

    pub fn apply(
        self,
        movement: operations::retail::StockMovement,
    ) -> operations::retail::Result<Self>;
}
```

Behavior ownership: `InventoryPosition` performs stock arithmetic because it owns the invariant `reserved <= on_hand` and derives availability. Call sites should not subtract raw counts.

### Stock movement owns movement coherence

```rust
impl operations::retail::StockMovement {
    pub fn receipt(
        location_id: entities::LocationId,
        sku: operations::retail::Sku,
        quantity: operations::retail::OrderQuantity,
        source: operations::retail::StockMovementSource,
        actor: entities::ActorRef,
        sync_key: Option<operations::retail::InventorySyncKey>,
    ) -> operations::retail::Result<Self>;

    pub fn reservation_hold(
        reservation_id: entities::ReservationId,
        location_id: entities::LocationId,
        sku: operations::retail::Sku,
        quantity: operations::retail::OrderQuantity,
        actor: entities::ActorRef,
    ) -> operations::retail::Result<Self>;

    pub fn sale(
        location_id: entities::LocationId,
        sku: operations::retail::Sku,
        quantity: operations::retail::OrderQuantity,
        source: operations::retail::SaleSource,
        actor: entities::ActorRef,
        sync_key: operations::retail::InventorySyncKey,
    ) -> operations::retail::Result<Self>;

    pub fn adjustment_request(
        location_id: entities::LocationId,
        sku: operations::retail::Sku,
        quantity: operations::retail::OrderQuantity,
        direction: operations::retail::AdjustmentDirection,
        reason: operations::retail::InventoryAdjustmentReason,
        actor: entities::ActorRef,
    ) -> operations::retail::Result<operations::retail::InventoryReviewPacket>;
}
```

Behavior ownership: movement constructors validate that the kind/source/quantity combination makes semantic sense. Manual adjustments produce review packets unless approval policy says otherwise.

### Repository owns persistence and idempotency

```rust
#[async_trait]
pub trait operations::retail::InventoryRepository {
    async fn position(
        &self,
        location_id: entities::LocationId,
        sku: operations::retail::Sku,
    ) -> operations::retail::Result<operations::retail::InventoryPosition>;

    async fn movement_by_sync_key(
        &self,
        key: operations::retail::InventorySyncKey,
    ) -> operations::retail::Result<Option<operations::retail::StockMovement>>;

    async fn commit_movement(
        &self,
        expected: operations::retail::InventoryPositionVersion,
        movement: operations::retail::StockMovement,
    ) -> operations::retail::Result<operations::retail::InventoryPosition>;

    async fn save_review_packet(
        &self,
        packet: operations::retail::InventoryReviewPacket,
    ) -> operations::retail::Result<workflow::RecommendedAction>;
}
```

Behavior ownership: the repository does not decide business policy, but it does protect storage-level consistency, idempotency, and optimistic concurrency.

### Reorder policy owns reorder decisions

```rust
impl operations::retail::ReorderPolicy {
    pub fn evaluate(
        &self,
        offering: &operations::retail::LocationOffering,
        position: &operations::retail::InventoryPosition,
        forecast: Option<&operations::retail::InventoryForecast>,
    ) -> operations::retail::ReorderDecision;
}

impl operations::retail::ReorderDecision {
    pub fn into_staff_task(
        self,
        due_at: chrono::DateTime<chrono::Utc>,
    ) -> Option<operations::StaffTask>;
}
```

Behavior ownership: `ReorderPolicy` decides routing. `StaffTask` remains the task aggregate; retail only supplies a typed task kind/source/title/priority.

### Forecast service owns future demand projection

```rust
pub struct operations::retail::InventoryForecastService<R, I> {
    reservations: R,
    inventory: I,
}

impl<R, I> operations::retail::InventoryForecastService<R, I>
where
    R: reservation::Repository,
    I: operations::retail::InventoryRepository,
{
    pub async fn forecast_in_house_diet(
        &self,
        location_id: entities::LocationId,
        sku: operations::retail::Sku,
        window: operations::retail::ForecastWindow,
    ) -> operations::retail::Result<operations::retail::InventoryForecast>;
}
```

Behavior ownership: the service coordinates reservation demand and inventory state. It does not own reservation lifecycle or feeding-instruction truth.

### Recommendation and POS consume availability; they do not mutate stock directly

```rust
impl operations::retail::RecommendationPolicy {
    pub fn evaluate_inventory_gate(
        &self,
        product: &operations::retail::PartnerProduct,
        availability: operations::retail::InventoryAvailability,
        audience: operations::retail::RecommendationAudience,
    ) -> operations::retail::RecommendationDecision;
}

impl operations::retail::PointOfSalePolicy {
    pub fn authorize_sale_draft(
        &self,
        source: operations::retail::SaleSource,
        availability: operations::retail::InventoryAvailability,
        quantity: operations::retail::OrderQuantity,
        staff_role: operations::StaffRole,
        approval: Option<policy::ApprovalRecord>,
    ) -> operations::retail::SaleDecision;
}
```

Behavior ownership: recommendation and POS policies can block or route a draft based on inventory, but actual stock mutation occurs through approved `StockMovement` committed by the inventory repository.

## 5. Review / approval contract

### Automation level

Allowed without human approval:

- Read inventory/catalog/position snapshots.
- Classify availability from already-promoted semantic inventory data.
- Detect below-threshold or projected-depletion candidates.
- Create internal staff-task drafts for review.
- Produce manager-brief summaries of stock risk.
- Deduplicate external POS/vendor movements by idempotency key.

Staff review required:

- Manual inventory adjustments from counts or discrepancies.
- Reservation holds/releases that affect a customer stay when source data is ambiguous.
- Product substitutions or “similar product” suggestions.
- Customer-facing recommendation drafts that mention current stock.
- Sale-line drafts attached to checkout.
- Any inventory-derived recommendation based on care, diet, medication, allergy, medical, anxiety, or behavior context.

Manager review required:

- Vendor order submission or vendor managed-order policy change.
- Reorder threshold / target quantity / active offering policy changes.
- Comps, discounts, refunds, write-offs, and waste/expiry adjustment above configured limits.
- Backorder/discontinued substitution decisions.
- Any product recommendation that could be interpreted as veterinary/medical advice.
- Any destructive stock correction not backed by POS/vendor receipt data.

Never autonomous:

- Sending a customer message.
- Charging a customer or capturing/refunding payment.
- Submitting a vendor purchase order.
- Changing feeding instructions or care plans.
- Promising out-of-stock inventory.
- Hiding stock risk from staff/manager review.

### Audit trail

Every committed stock mutation must record:

- location and SKU;
- movement kind and quantity;
- before/after or position version;
- source system / reservation / staff / workflow / agent source;
- actor reference;
- occurred_at and committed_at;
- idempotency key for external source events;
- approval record when review was required;
- optional review packet/rationale ID for manager decisions.

Every denied or review-routed mutation should also be auditable as a decision, but it must not update `InventoryPosition`.

### Customer/member-facing boundaries

Inventory status may inform an approved customer message only after staff review. The domain should distinguish:

- internal availability: exact operational stock numbers visible to staff;
- customer-safe availability: approved wording such as “available at checkout” or “we can discuss options”; and
- forbidden promises: “guaranteed”, “reserved for you”, or medical/diet claims without approval and confirmed stock.

Exact stock counts should default to internal-only unless a product/location policy explicitly allows customer display.

## 6. Test contracts

### Domain-quality tests

- `retail_inventory_position_allows_zero_on_hand_without_using_positive_unit_count`
  - Proves out-of-stock is representable as a valid position while thresholds/order quantities remain positive.

- `retail_inventory_position_derives_available_units_without_raw_math_at_call_sites`
  - Proves available units are derived by `InventoryPosition` and reserved cannot exceed on-hand.

- `retail_stock_movement_constructors_reject_incoherent_kind_source_pairs`
  - Proves sale movements need POS/reservation sale source, reservation holds need reservation source, and receipts need vendor/source context.

- `retail_stock_movement_apply_rejects_negative_on_hand_and_over_reservation`
  - Proves movement application preserves core inventory invariants.

- `retail_inventory_sync_key_deduplicates_external_pos_and_vendor_events`
  - Proves idempotent replay returns the existing movement/position rather than double-counting stock.

- `retail_inventory_policy_classifies_in_stock_low_stock_out_of_stock_and_reserved_only`
  - Proves availability is semantic enum output, not boolean/string status.

- `retail_reorder_policy_returns_projected_depletion_for_purina_en_boarding_demand`
  - Proves future boarding diet reservations can trigger review before threshold is crossed.

- `retail_reorder_policy_creates_internal_task_not_vendor_order_for_auto_create_manager_task`
  - Proves agent/policy automation creates staff review work, not external orders.

- `retail_recommendation_policy_blocks_customer_promise_when_inventory_is_out_of_stock`
  - Proves recommendation flows consume inventory availability and cannot promise unavailable products.

- `retail_pos_policy_requires_approval_before_checkout_sale_consumes_reserved_inventory`
  - Proves POS drafts and inventory movement are separate and review-gated.

### Storage/boundary tests

- `retail_inventory_position_records_roundtrip_zero_on_hand_reserved_threshold_and_vendor_state`
  - Proves storage codecs preserve semantic stock state.

- `retail_inventory_movement_records_roundtrip_kind_source_actor_sync_key_and_approval`
  - Proves audit semantics survive persistence.

- `retail_inventory_records_reject_negative_or_shape_mismatched_external_stock_fields`
  - Proves provider data cannot create impossible domain state.

- `retail_unknown_external_sku_routes_data_quality_review_without_creating_product`
  - Proves boundary adapters do not silently create active products from raw strings.

- `retail_inventory_commit_uses_expected_position_version_to_prevent_lost_updates`
  - Proves concurrent POS/vendor/staff writes cannot overwrite each other silently.

### Workflow/agent tests

- `retail_inventory_watch_agent_can_emit_reorder_review_task_but_not_submit_vendor_order`
  - Proves agent spec forbidden actions and output schema enforce review boundaries.

- `retail_boarding_diet_forecast_agent_surfaces_purina_en_depletion_to_manager_brief`
  - Proves forecast outputs are manager-review artifacts.

- `retail_data_quality_agent_routes_unknown_partner_sku_to_cleanup_task`
  - Proves extensibility without raw-string branching in `operations::retail`.

- `retail_recommendation_drafter_uses_customer_safe_availability_after_staff_review_only`
  - Proves exact internal stock does not leak into member-facing drafts without approval.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Refactor current `pub mod retail` inventory surface.
  - Add newtypes/enums/entities/policies under `operations::retail` or split into nested `operations::retail::{inventory, reorder, stock_movement}` modules if the file grows too large.
  - Add semantic `retail::Error` / `retail::Result<T>`.
  - Extend `StaffTaskKind`, `StaffTaskSource`, `RevenueOpportunityKind`, and possibly `OperationsAction` with typed retail variants only when behavior needs them.

- `domain/src/entities.rs`
  - Usually no inventory ownership changes; may need `ActorRef` reuse or audit subject variants if stock movements require explicit audit subject identity.

- `domain/src/workflow.rs` / `domain/src/agents.rs` / `domain/src/agent.rs`
  - Add typed workflow events/recommended action payloads or agent specs for inventory watch and boarding diet forecast.

- `domain/src/tools.rs`
  - Add boundary request/result contracts for inventory snapshot reads, stock movement draft/commit, and vendor/POS sync if tools are modeled in-domain.

- `domain/tests/petsuites_core_service_contracts.rs`
  - Update current retail contract tests away from live stock embedded in `InventoryPolicy` if refined.

- `domain/tests/domain_quality_patterns.rs`
  - Add semantic tests for inventory position, movements, zero-capable stock, review gates, and agent boundaries.

- `storage/tests/core_service_contract_storage.rs`
  - Preserve existing retail contract JSON roundtrip while adding storage coverage for positions/movements if storage card owns it.

- `storage/tests/operations_storage_contracts.rs`
  - Extend service-offering/retail variants and reject malformed inventory payloads.

### Migration and refactor risks

- Existing `InventoryPolicy::Tracked { on_hand, reorder_at }` conflates policy with live stock. Moving `on_hand` into `InventoryPosition` may require transitional codecs or compatibility fixtures.
- Existing `UnitCount` rejects zero, which is correct for thresholds and quantities but wrong for live on-hand. Introduce `OnHandUnits` instead of weakening `UnitCount` semantics globally.
- Existing `Contract::should_reorder() -> bool` is too coarse. Keep temporarily for compatibility or reimplement through `ReorderSignal` while tests migrate.
- `RetailPartner` and `RetailProductCategory` currently live at `operations` parent level. Retail inventory behavior should prefer `operations::retail::{Partner, Category}` or one canonical re-export path to avoid split ownership.
- POS sale and inventory consumption are related but not identical. Do not let payment/POS adapters mutate stock without producing an approved `StockMovement`.
- In-house diet inventory and sellable retail inventory can share SKU/product concepts, but movement sources must distinguish operational consumption from customer sale.
- Exact stock counts and care-sensitive context can leak through debug/customer-message surfaces if review packets are not carefully separated from customer-safe drafts.

### Dependencies on other implications

- Recommendation / personalized upsell implication: consumes `InventoryAvailability` and must honor out-of-stock/low-stock/review decisions before creating customer-facing drafts.
- POS / checkout implication: creates sale-line drafts and, after approval/payment boundary success, commits `Sold` movements.
- Boarding diet / Purina EN implication: supplies future reservation demand and in-house consumption events for forecast/depletion logic.
- Vendor / reorder implication: may expand `VendorOrderDraft`, lead time, backorder, and receipt flows.
- Data-quality implication: handles unknown SKU, partner/category mismatches, and location offering activation review.
- Audit/workflow implication: supplies durable approval records, workflow events, staff task routing, and manager brief surfaces.

## Acceptance summary

This inventory model keeps the domain honest by separating product catalog, location offering, live inventory position, immutable stock movement, forecast, reorder decision, and staff/customer-facing review. The safest extensible default is: agents may detect and draft; staff/managers approve; only approved, idempotent stock movements mutate inventory; and customer/member-facing actions never proceed directly from inventory automation.
