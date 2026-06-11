# Retail / partner products implication 01: Point-of-sale sales

## Purpose

This implication models how a PetSuites retail or partner-product item becomes a safe point-of-sale sale. It is a domain contract for later Rust code, not a POS integration implementation.

A POS sale is not merely "a SKU was recommended." It is a staff- or checkout-owned commercial event that must preserve product identity, location availability, quantity, money/tax semantics, customer or reservation context, inventory effect, approval gates, and an audit trail. Retail may draft the sale line and record retail-specific decisions; payment capture, tax calculation, receipt issuance, and external Gingr/POS synchronization remain boundary or neighboring-module responsibilities.

Assumptions used here:

- PetSuites POS sales may be standalone retail purchases, reservation checkout attachments, or manager-approved comps/discounted lines.
- Retail products can be partner-branded supplements, in-house diet products, or future retail offerings. The model should not hard-code Virbac/Purina as the only possible vendors, but current seed fixtures should cover them.
- Staff approval is always required before any sale line is charged or shown as customer-approved. Agent automation may draft or validate, not sell.
- If a product is both sellable and consumed in-house, sale inventory and care-consumption inventory must remain semantically distinct even when they reduce the same location stock position.

## 1. Operational story

### Trigger

A point-of-sale sale starts from one of these truthful triggers:

1. Front-desk or retail staff scans/selects a stocked product during a standalone purchase.
2. Staff adds a retail product to a reservation checkout after customer approval.
3. Staff accepts a reviewed recommendation candidate and converts it into a sale-line draft.
4. Manager approves a comp, discount, return/reversal, or service-recovery retail line.
5. POS/vendor reconciliation reports a completed sale that must be promoted into domain stock movement and audit history.

Agent-created upsell drafts and recommendation candidates are not triggers for an executed sale by themselves. They can only create a staff-review packet that a staff actor may convert into a sale-line draft.

### Actors

- Customer or account owner: approves the purchase and receives any receipt/customer-facing message.
- Pet or reservation context: optional domain context when the sale relates to a pet stay, boarding diet, supplement recommendation, or checkout continuation.
- Front-desk staff: creates standard sale-line drafts and attaches approved retail items to checkout.
- Resort manager: approves comps, discounts, refunds/reversals, medical-claim-sensitive recommendations, substitutions, and policy exceptions.
- Retail inventory role or vendor/POS adapter: supplies stock snapshots and completed external sale events.
- Retail recommendation agent: may draft internal recommendation/sale packets but cannot charge, send, substitute, or promise stock.
- POS/payment adapter: owns external charge, tax, receipt, and sync mechanics after domain gates allow the sale.

### Inputs

Required inputs for a sale-line draft:

- `operations::retail::LocationOffering` or current `operations::retail::Contract` identifying the location-specific SKU/product and POS policy.
- `operations::retail::SaleQuantity` / positive quantity.
- `entities::LocationId`.
- `operations::retail::SaleSource` such as standalone retail, reservation checkout, approved recommendation, or manager comp.
- Staff actor and role.
- Current inventory snapshot or explicit `NotTracked` policy.
- Price/taxability reference from `money`/pricing boundary or a typed `retail::PriceSnapshot` if later introduced.

Optional but semantically important inputs:

- `entities::CustomerId`.
- `entities::PetId`.
- `entities::ReservationId`.
- Approved recommendation candidate ID.
- Manager approval token/decision for discounts, comps, or sensitive substitutions.
- External POS idempotency key for reconciled completed sales.

### Decisions

The POS workflow makes these decisions in order:

1. Product/offering eligibility: is this SKU active and sellable at this location for the requested sale source?
2. Inventory eligibility: is the requested quantity available for sale, or is a manager-reviewed backorder/substitution path required?
3. Context eligibility: can this product be attached to the reservation/customer/pet context without crossing care-profile or medical/diet review boundaries?
4. Staff permission: may the staff role draft the sale, or does the POS policy require manager approval?
5. Price/discount/comp policy: is this a normal priced sale, discount, manager-only comp, refund/reversal, or external reconciliation event?
6. Customer-facing boundary: has a human approved any message, prompt, checkout attachment, or charge before it reaches the customer?
7. Inventory effect: should the sale reserve inventory, immediately record sold stock movement, or wait for external POS completion?

### Outputs

Successful domain outputs:

- `operations::retail::SaleLineDraft` or `SaleLine` with typed SKU/product, quantity, sale source, price snapshot, review state, and audit context.
- `operations::retail::PointOfSaleDecision::AllowedDraft` for staff/POS adapter routing, or a typed review/denial decision.
- `operations::retail::StockMovementDraft { kind: Sold | ReservedForSale, ... }` when inventory tracking applies.
- `operations::StaffTask` for manager review, substitution review, stock adjustment, or data-quality cleanup when gates do not allow a normal sale.
- `workflow::RecommendedAction` only for internal staff review, never direct charging or customer send.
- Audit event linking staff actor, customer/reservation context, recommendation candidate, POS adapter reference, and policy decisions.

Boundary outputs owned outside retail:

- Payment authorization/capture.
- Tax calculation.
- Receipt rendering/sending.
- External POS mutation.
- Refund settlement.

### Success state

A POS sale is successful in the retail domain when:

- The product/offering is active and sellable for the source.
- Quantity is positive and inventory is available or explicitly review-approved.
- Price/discount/comp policy has an allowed decision.
- Any customer-facing or charge-producing boundary has staff/manager approval as required.
- The sale line carries customer/reservation/pet context when applicable without raw IDs or stringly state.
- Inventory movement is either recorded idempotently or intentionally deferred until external POS confirmation.
- The audit trail can explain who/what created the line, which policy allowed it, and which external reference finalized it.

### Failure and exception states

Represent failures as semantic decisions/errors rather than booleans or strings:

- `InactiveOffering`: SKU exists but is not active at this location.
- `NotSellableForSource`: product is in-house-only or POS policy forbids the requested source.
- `OutOfStock`: available-for-sale is below requested quantity.
- `InventoryUnknownRequiresReview`: inventory cannot be trusted enough for customer promise.
- `CareSensitiveRecommendationRequiresReview`: diet/supplement context touches care-profile facts.
- `ManagerApprovalRequired`: comp, discount, refund, substitution, complaint recovery, or policy exception.
- `CustomerApprovalMissing`: sale is attached to a reservation/customer context without explicit staff-recorded customer approval.
- `PaymentBoundaryNotAuthorized`: retail draft is valid but payment/POS adapter has not completed charge.
- `ExternalPosDuplicate`: idempotency key or POS reference already recorded.
- `StockMovementConflict`: POS sale and stock snapshot disagree; create inventory adjustment review rather than silently correcting.
- `UnsafeCustomerClaim`: customer-facing copy suggests diagnosis/treatment/cure/prevention or hides inventory limitation.

## 2. Domain types to add or refine

### Product and offering identity

- `operations::retail::PartnerProduct`
  - Required fields: `Sku`, `ProductName`, `Partner`, `ProductFamily`, `Category`, `UsageMode`.
  - Invariant: a product has one stable SKU and semantic product family; display name is trimmed, non-empty, bounded.
- `operations::retail::LocationOffering`
  - Required fields: `entities::LocationId`, `PartnerProduct`, `OfferingStatus`, `PointOfSalePolicy`, `InventoryPolicy`, optional price/taxability snapshot.
  - Invariant: an inactive offering cannot produce an allowed sale-line draft.
- `operations::retail::OfferingStatus`
  - Variants: `Active`, `TemporarilyUnavailable`, `Discontinued`, `DataQualityReviewRequired`.
- `operations::retail::UsageMode`
  - Variants: `Sellable`, `InHouseConsumable`, `SellableAndInHouseConsumable`.
  - Invariant: `InHouseConsumable` alone cannot be sold through POS without manager policy exception.

### POS sale values

- `operations::retail::SaleLineId`
  - Domain identity for an internal sale-line draft/executed line.
- `operations::retail::SaleQuantity`
  - Positive unit count for a sale, distinct from `OnHandUnits` and `ReorderThreshold`.
- `operations::retail::SaleLineDraft`
  - A not-yet-charged line with product, quantity, source, staff actor, context, price snapshot, review gate, and idempotency key.
  - Invariant: a draft is never a payment capture or customer receipt.
- `operations::retail::SaleLine`
  - A finalized retail-domain sale record after POS/payment confirmation or trusted reconciliation.
  - Invariant: must reference a prior allowed draft or trusted external completed-sale event.
- `operations::retail::SaleSource`
  - Variants: `StandaloneRetail`, `ReservationCheckout { reservation_id }`, `ApprovedRecommendation { candidate_id }`, `ManagerComp { reason }`, `ExternalPosReconciliation { external_sale_id }`.
- `operations::retail::SaleContext`
  - Optional typed customer/pet/reservation context. Prefer an enum/struct over many nullable fields.
- `operations::retail::CustomerApprovalEvidence`
  - Staff-recorded proof that customer approved a sale/checkout attachment; could wrap timestamp, staff actor, and channel/reference.
- `operations::retail::ExternalPosSaleId` and `operations::retail::SaleIdempotencyKey`
  - Trimmed, non-empty values used only at the POS boundary/reconciliation seam.

### POS decisions and policy states

- `operations::retail::PointOfSaleDecision`
  - Variants: `AllowedDraft { review_state }`, `StaffReviewRequired { reason }`, `ManagerReviewRequired { reason }`, `Denied { reason }`.
- `operations::retail::PointOfSaleDenialReason`
  - Variants for inactive offering, source mismatch, out of stock, missing customer approval, unsafe claim, duplicate external sale, and payment-boundary mismatch.
- `operations::retail::SaleReviewState`
  - Variants: `StaffDraft`, `StaffApproved`, `ManagerApproved`, `ExternallyCompleted`, `Rejected`.
- `operations::retail::DiscountPolicy` or refine current `CompPolicy`
  - Variants: `NoDiscount`, `ManagerApprovalRequired`, `ManagerApproved { reason }`, `PolicyAllowed { reason }`.
- Refine existing `operations::retail::PointOfSalePolicy`
  - Keep variants `StandaloneSale`, `IntegratedWithReservationCheckout`, `ManagerOnlyComp` as seed cases, but allow behavior methods to answer `allows_source(source, staff_role)` and `requires_manager_approval(request)`.

### Inventory effect values

- `operations::retail::InventoryPosition`
  - Zero-capable `OnHandUnits`, `ReservedUnits`, and derived `available_for_sale()`.
  - Invariant: reserved cannot exceed on-hand; available is derived, not stored.
- `operations::retail::StockMovement`
  - Includes `Sold`, `ReservedForSale`, `ReleasedFromSale`, `Returned`, `Adjusted`, `ConsumedDuringStay`.
  - Invariant: sale movement references a sale line or external POS reconciliation id.
- `operations::retail::InventoryEffect`
  - Variants: `NoTracking`, `ReserveForSale`, `RecordSoldMovement`, `RequiresAdjustmentReview`.

### Errors

Add module-local semantic errors/results when implementation grows beyond current inline types:

```rust
operations::retail::Result<T> = core::result::Result<T, operations::retail::Error>;

operations::retail::Error::InvalidSku { source: SkuError }
operations::retail::Error::InvalidSaleQuantity { value: u32 }
operations::retail::Error::ReservedUnitsExceedOnHand { on_hand, reserved }
operations::retail::Error::DuplicateExternalSale { external_sale_id }
operations::retail::Error::SaleLineMissingApproval { source }
```

## 3. Relationship map between types

### Entities and value objects

- `entities::LocationId` scopes `retail::LocationOffering`, `retail::InventoryPosition`, price/taxability, and POS rules.
- `entities::CustomerId` owns purchase approval and receipt/message ownership.
- `entities::PetId` is optional sale context for recommendations or reservation checkout, but retail does not own pet care facts.
- `entities::ReservationId` links checkout sale lines to boarding/daycare/grooming/training reservations.
- `retail::Sku`, `retail::ProductName`, `retail::PartnerProduct`, and `retail::LocationOffering` identify what can be sold.
- `money::Money` or a future pricing value owns price; retail should keep only a `retail::PriceSnapshot` if the sale-line draft must preserve the presented price.

### Policies

- `retail::PointOfSalePolicy` decides whether a sale source/staff role/context can produce a sale-line draft.
- `retail::InventoryPolicy` and `retail::InventoryPosition` decide availability and inventory effect.
- `retail::RecommendationPolicy` decides whether an approved recommendation can be converted to staff/customer-facing sales flow.
- `retail::DiscountPolicy` / `retail::CompPolicy` decides review gates for price exceptions.
- `payment`/`money` policies own charge/refund/tax outcomes after retail allows a draft.

### Repositories and stores

- `retail::CatalogRepository`: loads `PartnerProduct` by SKU and validates product identity.
- `retail::OfferingRepository`: loads active `LocationOffering` by location/SKU.
- `retail::InventoryRepository`: loads `InventoryPosition` and records stock movements idempotently.
- `retail::SaleRepository`: saves sale-line drafts, finalized sale lines, external POS references, and audit events.
- `retail::RecommendationRepository`: loads approved recommendation candidates that may become sale sources.
- Storage adapters translate raw Gingr/POS/vendor strings into domain values before repositories expose them.

### Workflow events, staff tasks, and audit

- `retail::SaleLineDrafted` event: created when policy allows an internal draft.
- `retail::SaleLineApproved` event: staff/manager approval before customer-facing or charge boundary.
- `retail::SaleCompleted` event: external POS/payment confirms sale.
- `retail::InventoryMovementRecorded` event: stock movement created from completed sale.
- `operations::StaffTaskKind::RetailSaleReview { location_id, sku, source }`: staff review gate for sale-line conversion.
- `operations::StaffTaskKind::RetailManagerApproval { location_id, sku, reason }`: manager gate for comp/discount/substitution/exceptions.
- `operations::StaffTaskKind::RetailInventoryConflictReview { location_id, sku, external_sale_id }`: reconciliation conflict.

### Agent specs and tools

- `retail-pos-sale-drafter`
  - Allowed: read approved offerings, inventory snapshots, recommendation candidates; emit `SaleLineDraft` proposals and staff tasks.
  - Forbidden: payment capture, receipt send, external POS mutation, refund, comp approval.
- `retail-pos-reconciliation`
  - Allowed: ingest trusted external completed sale events, match idempotency keys, draft stock movements/conflict tasks.
  - Forbidden: silently correcting stock conflicts or creating customer-facing claims.
- `retail-checkout-review-assistant`
  - Allowed: assemble staff review packet with product, inventory, approval evidence, and checkout context.
  - Forbidden: attaching retail line to customer checkout without staff approval.

## 4. Interaction contract

Use behavior-owning methods/services rather than free-floating helpers.

### Policy contract

```rust
impl operations::retail::PointOfSalePolicy {
    pub fn evaluate_sale_request(
        &self,
        request: &operations::retail::SaleRequest,
        offering: &operations::retail::LocationOffering,
        inventory: operations::retail::InventorySnapshot,
        staff: &operations::StaffActor,
    ) -> operations::retail::PointOfSaleDecision;
}
```

Behavior ownership:

- `PointOfSalePolicy` owns source/staff/comp/checkout permission decisions.
- It must not calculate tax or capture payment.
- It must return review/denial reasons, not `bool`.

### Sale request / draft construction

```rust
impl operations::retail::SaleRequest {
    pub fn builder() -> operations::retail::sale_request::Builder<MissingRequiredFields>;
}

impl operations::retail::SaleLineDraft {
    pub fn from_allowed_request(
        request: operations::retail::SaleRequest,
        offering: operations::retail::LocationOffering,
        price: money::PriceSnapshot,
        decision: operations::retail::AllowedPointOfSaleDraft,
    ) -> operations::retail::Result<Self>;
}
```

Invariants:

- Builder requires location, SKU/product, quantity, source, staff actor, and review state.
- Draft construction fails if the decision is review-required or denied.
- Draft construction never creates a payment.

### Inventory contract

```rust
impl operations::retail::InventoryPosition {
    pub fn available_for_sale(&self) -> operations::retail::AvailableUnits;

    pub fn can_sell(
        &self,
        quantity: operations::retail::SaleQuantity,
    ) -> operations::retail::InventorySaleDecision;

    pub fn sold_movement_for(
        &self,
        sale_line: &operations::retail::SaleLine,
    ) -> operations::retail::Result<operations::retail::StockMovement>;
}
```

Behavior ownership:

- Inventory math belongs on `InventoryPosition`.
- Stock movement creation belongs on `InventoryPosition` or an inventory policy/service, not caller-side arithmetic.
- Zero on-hand must be representable with a zero-capable `OnHandUnits`; do not reuse current positive-only `UnitCount` for depleted stock.

### Repository contracts

```rust
pub trait operations::retail::OfferingRepository {
    fn offering_for_sale(
        &self,
        location_id: entities::LocationId,
        sku: operations::retail::Sku,
    ) -> operations::retail::Result<Option<operations::retail::LocationOffering>>;
}

pub trait operations::retail::InventoryRepository {
    fn inventory_position(
        &self,
        location_id: entities::LocationId,
        sku: operations::retail::Sku,
    ) -> operations::retail::Result<operations::retail::InventoryPosition>;

    fn record_stock_movement(
        &mut self,
        movement: operations::retail::StockMovement,
        idempotency: operations::retail::SaleIdempotencyKey,
    ) -> operations::retail::Result<operations::retail::RecordedStockMovement>;
}

pub trait operations::retail::SaleRepository {
    fn save_draft(
        &mut self,
        draft: operations::retail::SaleLineDraft,
    ) -> operations::retail::Result<operations::retail::SaleLineId>;

    fn mark_completed(
        &mut self,
        sale_line_id: operations::retail::SaleLineId,
        external_sale_id: operations::retail::ExternalPosSaleId,
        payment_reference: payment::Reference,
    ) -> operations::retail::Result<operations::retail::SaleLine>;
}
```

### Domain service contract

```rust
pub struct operations::retail::CheckoutService<P, O, I, S> {
    pos_policy: P,
    offerings: O,
    inventory: I,
    sales: S,
}

impl<P, O, I, S> operations::retail::CheckoutService<P, O, I, S> {
    pub fn draft_sale_line(
        &mut self,
        request: operations::retail::SaleRequest,
    ) -> operations::retail::Result<operations::retail::CheckoutDraftOutcome>;

    pub fn reconcile_completed_external_sale(
        &mut self,
        event: operations::retail::ExternalCompletedSale,
    ) -> operations::retail::Result<operations::retail::ReconciliationOutcome>;
}
```

Outcome variants:

- `CheckoutDraftOutcome::DraftSaved { sale_line_id, staff_task: Option<StaffTask> }`.
- `CheckoutDraftOutcome::StaffReviewRequired { task }`.
- `CheckoutDraftOutcome::ManagerReviewRequired { task }`.
- `CheckoutDraftOutcome::Denied { reason }`.
- `ReconciliationOutcome::SaleRecorded { sale_line, movement }`.
- `ReconciliationOutcome::DuplicateIgnored { external_sale_id }`.
- `ReconciliationOutcome::InventoryConflictReviewCreated { task }`.

## 5. Review and approval contract

### Automation level

Safe automation:

- Classify whether an approved location offering can theoretically be sold through a requested source.
- Build internal `SaleLineDraft` proposals when all policy gates pass and a staff actor remains responsible for final action.
- Detect inventory conflicts or duplicate external POS events.
- Create internal staff tasks for review.
- Prepare staff-facing explanation packets.

Staff approval required:

- Attaching a retail product to reservation checkout.
- Turning an approved recommendation into a customer-facing checkout prompt.
- Any sale connected to diet/supplement/care-profile rationale, even if the product is common.
- Any customer-facing copy, receipt note, or upsell message.

Manager approval required:

- Comps, discounts, refunds, charge reversals, service recovery, or complaint-driven product gifts.
- Substitutions when requested SKU is unavailable.
- Selling in-house-only inventory as retail.
- Any medical/diet-sensitive recommendation that could be read as veterinary advice.
- Policy changes to POS eligibility, discount permissions, or inventory exception handling.

Forbidden for agents without explicit human/API approval:

- Capturing payment or creating a charge.
- Sending receipt or product recommendation to customer.
- Mutating external POS sale state.
- Applying comp/discount/refund.
- Promising unavailable inventory.
- Modifying care instructions or feeding plans.

### Audit trail

Every sale-line draft or reconciliation outcome should preserve:

- Product/SKU and location.
- Sale source and context IDs.
- Staff actor or agent spec that produced the draft.
- Policy decision and review gate.
- Customer approval evidence where relevant.
- Manager approval evidence where relevant.
- External POS sale id/payment reference when completed.
- Inventory movement id or conflict task id.
- Recommendation candidate id/rationale if the sale came from a recommendation.

### Customer/member-facing boundaries

The retail domain may produce a customer-safe draft only after review policy allows it. The draft must avoid diagnosis/treatment claims and should phrase products as optional support or staff recommendation, not medical advice. Actual sending belongs to workflow/customer communication tools after explicit staff approval.

## 6. Test contracts

Domain tests that should pass after implementation:

- `retail_pos_sale_request_requires_location_sku_quantity_source_and_staff_actor`
  - Proves sale requests cannot be built from loose SKU strings or missing context.
- `retail_pos_policy_allows_standalone_sale_for_active_sellable_offering_with_available_stock`
  - Proves normal staff retail purchase can produce an allowed draft decision.
- `retail_pos_policy_requires_staff_approval_before_reservation_checkout_attachment`
  - Proves reservation checkout source is not equivalent to customer approval.
- `retail_pos_policy_requires_manager_approval_for_comps_discounts_and_refunds`
  - Proves exceptions are semantic review outcomes, not boolean flags.
- `retail_pos_policy_denies_in_house_only_product_for_standard_standalone_sale`
  - Proves usage mode protects Purina EN or other in-house inventory from accidental sale.
- `retail_sale_line_draft_does_not_capture_payment_or_send_receipt`
  - Proves retail draft state remains distinct from payment/POS side effects.
- `retail_sale_quantity_and_sku_are_semantic_validated_values`
  - Extends existing SKU/unit tests with sale-specific positive quantity and trimmed idempotency values.
- `retail_inventory_position_derives_available_for_sale_and_blocks_oversell`
  - Proves no caller-side raw subtraction and reserved units cannot exceed on-hand.
- `retail_completed_sale_records_idempotent_sold_stock_movement`
  - Proves external POS reconciliation does not double-decrement stock.
- `retail_external_pos_duplicate_is_ignored_with_audit_not_replayed`
  - Proves idempotency key/external ID protects against duplicate provider events.
- `retail_pos_reconciliation_creates_inventory_conflict_review_when_provider_stock_disagrees`
  - Proves conflicts route to staff task rather than silent correction.
- `retail_recommendation_to_sale_requires_review_for_care_sensitive_supplement_or_diet_context`
  - Proves anxiety/diet/supplement recommendations do not become autonomous sales.
- `retail_customer_facing_sale_prompt_rejects_medical_claim_language`
  - Proves unsafe claims are denied or manager-reviewed before customer exposure.
- `retail_sale_line_preserves_customer_pet_reservation_and_recommendation_context`
  - Proves sale context survives persistence without raw blobs.

Storage/boundary tests:

- `retail_sale_line_records_roundtrip_between_storage_and_domain`
  - Preserves SKU, location, source, quantity, review state, price snapshot, external POS reference, and audit fields.
- `retail_pos_adapter_promotes_raw_sku_and_external_sale_id_into_semantic_values`
  - Boundary conversion rejects blank SKU/id values before domain policy sees them.
- `retail_sale_repository_rejects_completed_sale_without_allowed_draft_or_trusted_external_event`
  - Prevents fabricated sale completion in storage.

Workflow/agent tests:

- `retail_pos_sale_drafter_agent_outputs_staff_review_packet_not_payment_action`
  - Agent spec disallows charge/receipt/POS mutation tools.
- `retail_pos_reconciliation_agent_can_create_inventory_conflict_task_but_not_adjust_stock_silently`
  - Reconciliation automation remains review-gated.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Existing `pub mod retail` should gain sale request/draft/decision, refined POS policy behavior, inventory position, stock movement, review state, and semantic errors.
  - Consider splitting `operations::retail` into child modules if the file grows: `operations/retail/product.rs`, `offering.rs`, `pos.rs`, `inventory.rs`, `sale.rs`, `error.rs`, then re-export through `operations::retail`.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Extend current retail contract test beyond SKU/POS/inventory/reorder to sale-line draft and policy decisions.
- `domain/tests/domain_quality_patterns.rs`
  - Add semantic-path, approval-boundary, and no-helper/no-raw-blob assertions if that suite remains the domain-quality umbrella.
- `storage/tests/core_service_contract_storage.rs`
  - Existing retail contract codec tests should continue to pass; add storage tests only when sale-line records are introduced.
- `storage/tests/operations_storage_contracts.rs`
  - Extend POS sale and retail service-offering shape validation when storage records exist.
- Future storage modules/migrations for `retail_sale_lines`, `retail_stock_movements`, and `retail_sale_audit_events` if persistence is introduced.

### Migration/refactor risks

- Current `operations::retail::UnitCount` is positive-only and cannot represent zero on-hand. POS sale work needs `OnHandUnits` or similar zero-capable inventory counts.
- Current `Product` contains SKU and parent-level `RetailProductCategory`, but no partner, product family, display name, usage mode, location status, or price/taxability. Sale behavior should use `PartnerProduct`/`LocationOffering` rather than overloading this seed type.
- Current `PointOfSalePolicy` is a coarse enum. Do not add booleans such as `requires_manager_approval`; introduce decision enums and behavior methods.
- Payment and receipt side effects must stay out of `operations::retail`; use typed references/outcomes to integrate with payment/POS adapters.
- Recommendation-derived sales can accidentally blur staff suggestions with customer approval. Keep `RecommendationCandidate`, `CustomerApprovalEvidence`, and `SaleLineDraft` distinct.
- In-house diet consumption and customer-sold units may share inventory, but their stock movements have different semantics and audit requirements.
- External POS reconciliation must be idempotent; duplicate external events should be audit outcomes, not repeated stock movement.

### Dependencies on other implications

- Depends on the retail service map for product/offering/inventory/recommendation vocabulary.
- Interacts with future recommendation/upsell implications: approved recommendation candidates may be sale sources, but POS sales own sale-line and charge boundary decisions.
- Interacts with reorder/inventory implications: completed sales and reservations affect stock position and reorder signals.
- Interacts with boarding/in-house diet implications: Purina EN may be consumed during stay or sold/recommended at checkout; these paths must remain distinct.
- Interacts with money/payment work: price/tax/payment/receipt types should be imported as semantic references, not recreated in retail.
- Interacts with workflow/agent work: agents may draft staff tasks and sale packets but require review gates before customer-facing or payment actions.

## Implementation posture

Start with deterministic domain policy and type tests before wiring storage or agents. The smallest useful code slice is:

1. Add semantic sale request, quantity, source, review state, and POS decision types.
2. Add behavior on `PointOfSalePolicy` to evaluate active sellable offerings and manager-only exceptions.
3. Add inventory position availability checks with zero-capable on-hand units.
4. Add `SaleLineDraft` construction only from an allowed decision.
5. Add tests proving agents/staff tasks stop at review packets and do not cross payment/customer-facing boundaries.

This keeps POS sales modeled as a truthful retail-domain workflow while preserving payment, customer communication, and external POS mutation as explicit boundary integrations.
