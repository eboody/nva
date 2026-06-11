# PetSuites retail / partner-products service domain map

## Purpose and assumptions

This document models the Retail / partner-products core service line for the NVA/PetSuites AI-operations foundation. It is a contract map for later Rust/domain cards, not an implementation patch.

Source service surface:

- Virbac CalmCare: behavior/anxiety-support supplement opportunity, usually connected to boarding, daycare, training, and anxiety/stress care notes.
- Purina Pro Plan Veterinary Supplements: broader supplement opportunity, usually connected to care-profile facts, staff/vet-recommended support, and retail POS inventory.
- Purina EN as in-house diet for boarding guests: operational diet inventory used during stays and potentially sold or recommended when a pet has compatible feeding needs.

Assumptions:

- PetSuites retail is a service line inside operations, but specific products are location-stocked offerings. A product may be partner-branded, location-specific, inventory-tracked, and eligible for POS sale, recommendation, or in-house use.
- The domain should not encode veterinary diagnosis. It may encode facts such as staff-observed anxiety/stress flags, care-profile diet needs, existing customer-provided instructions, and recommendation approval state.
- Recommendations, reorder suggestions, and customer-facing upsell drafts are separate from execution. Staff or manager approval gates control sends, sales, substitutions, comps, and medical/diet-sensitive recommendations.
- Partner/vendor names are known seed variants, but the contract should allow future partner products without scattering raw strings.

## Domain vocabulary and bounded-context summary

Retail / partner products is the PetSuites operations context that turns partner product availability into safe, inventory-aware, POS-ready, and review-gated recommendations.

Core vocabulary:

- Partner product: a sellable or in-house consumable product sourced from a named partner/vendor, such as Virbac or Purina.
- Product offering: the location-specific retail contract for a SKU, including partner, category, POS policy, inventory policy, recommendation policy, and reorder policy.
- In-house diet: inventory consumed as part of boarding care rather than only sold over the counter; Purina EN is the initial modeled example.
- Recommendation: an internal, explainable suggestion that a product may fit a pet/customer/reservation context.
- Personalized upsell: a staff-reviewed customer-facing sale opportunity derived from care profile, reservation context, purchase history, and current inventory.
- Inventory position: on-hand, reserved, reorder threshold, backorder/vendor state, and audit trail for stock movement.
- Reorder signal: a deterministic inventory/supply-chain outcome such as below threshold, depletion risk before future reservations, or vendor-managed restock candidate.
- Point-of-sale / POS sale: a checkout- or standalone-sale line item that must integrate with money/payment and staff/customer permissions.
- Approval boundary: a typed domain decision that says whether automation may only classify/draft, may create an internal task, or may proceed after staff/manager approval.

Bounded-context role:

- `operations::retail` should own retail product contract language, inventory thresholds, recommendation/reorder policies, and retail-specific staff tasks.
- `care` should own medical/care facts such as feeding instructions, allergies, conditions, medication review, and sensitive debug behavior.
- `reservation` should own stays/check-in/check-out lifecycle and add-ons; retail only references reservations when a product is attached to a stay, checkout, or pre-arrival workflow.
- `money` and `payment` should own price, tax/charge, authorization, and payment-reference semantics.
- `workflow` and `agents` should own recommended actions, review gates, and agent contracts; retail supplies domain-specific payloads and policy decisions.
- Integrations/adapters should translate Gingr/POS/vendor payloads into semantic retail contracts before policy logic sees them.

## Domain type inventory using semantic paths

Current truthful paths from `domain/src/operations.rs`:

- `operations::ServiceOffering::RetailPartnerProduct { partner, category }`
- `operations::RetailPartner::{VirbacCalmCare, PurinaProPlanVeterinarySupplements, PurinaEnBoardingDiet}`
- `operations::RetailProductCategory::{Supplement, InHouseDiet, PersonalizedUpsell}`
- `operations::retail::Contract`
- `operations::retail::Product`
- `operations::retail::Sku`
- `operations::retail::PointOfSalePolicy::{StandaloneSale, IntegratedWithReservationCheckout, ManagerOnlyComp}`
- `operations::retail::InventoryPolicy::{NotTracked, Tracked { on_hand, reorder_at }}`
- `operations::retail::UnitCount`
- `operations::retail::RecommendationRule::{None, AnxietySupportAfterBoarding, DietSupportAfterBoarding, CoatCareAfterGrooming}`
- `operations::retail::ReorderPolicy::{ManualReview, AutoCreateManagerTask, VendorManaged}`

Recommended next retail-specific domain surface:

- `operations::retail::Partner`
  - Prefer moving or re-exporting the current `operations::RetailPartner` into/through `operations::retail` if future behavior belongs to the retail context.
  - Variants: `Virbac`, `Purina`, `Other { name: partner::Name }` if partner expansion becomes necessary.
- `operations::retail::PartnerProduct`
  - Product identity with `Sku`, `ProductName`, `Partner`, category, and optional vendor catalog reference.
- `operations::retail::Category`
  - Refined replacement or re-export for `RetailProductCategory`.
  - Candidate variants: `Supplement`, `InHouseDiet`, `CheckoutUpsell`, `CareSupportConsumable`.
- `operations::retail::Sku`
  - Already implemented; should remain a validated non-empty retail identity value.
- `operations::retail::ProductName`
  - Validated trimmed non-empty display name; never use bare `String` in domain code once customer/staff display depends on it.
- `operations::retail::CatalogItem`
  - Stable product catalog contract independent of a location's inventory.
- `operations::retail::LocationOffering`
  - Location-specific SKU, price, availability, POS, inventory, recommendation, and reorder contract.
- `operations::retail::InventoryPosition`
  - On-hand, reserved-for-stay, available-for-sale, reorder threshold, and optional depletion risk.
- `operations::retail::StockMovement`
  - Receipt, sale, reservation consumption, adjustment, waste/expiry, transfer, and return events.
- `operations::retail::ReorderSignal`
  - `Healthy`, `BelowThreshold`, `ProjectedDepletion`, `Backordered`, `VendorManagedReview`.
- `operations::retail::RecommendationCandidate`
  - Candidate product with customer, pet, location, reservation context, reason, confidence/rationale, and required review gate.
- `operations::retail::RecommendationReason`
  - `AnxietyOrStressSupport`, `BoardingDietContinuity`, `StaffRecommendedCareSupport`, `CheckoutContinuation`, `PriorPurchaseReplenishment`, `ManagerInitiated`.
- `operations::retail::RecommendationPolicy`
  - Deterministic policy deciding whether a candidate is allowed, staff-review-only, manager-review-required, or forbidden.
- `operations::retail::UpsellDraft`
  - Staff/customer-safe proposed message or checkout prompt, not an authorized send/sale.
- `operations::retail::SaleLine`
  - Semantic POS line item containing SKU, quantity, price, taxability, discount/comp policy, and source.
- `operations::retail::SupplyChainStatus`
  - Vendor availability and reorder state: `OrderNeeded`, `OrderDrafted`, `Ordered`, `PartiallyReceived`, `Received`, `Backordered`, `Discontinued`.
- `operations::retail::Repository`
  - Domain-facing catalog, offering, inventory, and recommendation persistence interface once behavior needs persistence.
- `operations::retail::ReorderService`
  - Domain service for converting inventory positions and future reservations into reorder signals/tasks.
- `operations::retail::RecommendationService`
  - Domain service for converting care/reservation/customer context into review-gated recommendation candidates.
- `operations::retail::PosPolicy`
  - Behavior owner for allowed checkout/standalone sale/comp flows.

Avoid generic paths such as `operations::Product`, `helpers::recommend_product`, `retail_utils`, or booleans like `requires_review: bool`. The retail path should carry product/inventory/POS/recommendation meaning at call sites.

## Existing Rust/domain surface to reuse or refactor

Reuse directly:

- `operations::retail::Contract` as the current retail service-line contract shape.
- `operations::retail::Sku` and `operations::retail::Product` as the seed product identity surface.
- `operations::retail::UnitCount` for positive inventory thresholds/counts. Later add a separate zero-capable count if true zero on-hand inventory must be represented.
- `operations::retail::InventoryPolicy` and `Contract::should_reorder()` as the current deterministic inventory threshold check.
- `operations::retail::PointOfSalePolicy` for standalone sale, reservation-checkout integration, and manager-only comp distinction.
- `operations::retail::RecommendationRule` as initial coarse workflow triggers.
- `operations::retail::ReorderPolicy` for manual/manager-task/vendor-managed reorder routing.
- `operations::ServiceOffering::RetailPartnerProduct` for service catalog discovery where retail is a peer service line.
- `operations::RevenueOpportunity` and `RevenueOpportunityKind` for existing manager-brief revenue opportunity integration.
- `operations::StaffTask`, `StaffTaskKind::CustomerFollowUp`, `StaffTaskSource`, and `StaffRole` for internal reorder/recommendation/review tasks.
- `operations::OperationsAction::SuggestRevenueFollowUp` and `OperationsAction::CreateInternalTask` for safe agent outputs.

Refactor or extend when implementing code cards:

- `operations::RetailPartner` and `operations::RetailProductCategory` currently live at the parent operations level. That is acceptable for broad service catalog variants, but retail behavior should prefer `operations::retail::{Partner, Category}` or parent re-exports if rules become product-specific.
- `operations::retail::Product` currently carries SKU and category but not partner or display name. Partner/source identity should move into the retail product contract before recommendation/reorder logic depends on partner-specific workflows.
- `operations::retail::InventoryPolicy::Tracked { on_hand, reorder_at }` uses positive `UnitCount`, which makes `on_hand = 0` unrepresentable. If zero-stock state matters, introduce `operations::retail::OnHandUnits` as a zero-capable newtype and reserve `UnitCount` or `ReorderThreshold` for positive thresholds.
- `operations::retail::RecommendationRule` is a coarse enum. As soon as personalized upsells depend on pet/reservation/care/customer facts, introduce `RecommendationCandidate`, `RecommendationReason`, and `RecommendationPolicy` rather than adding more ad hoc enum variants.
- `operations::retail::Contract::standard_petsuites()` currently returns a generic placeholder SKU/category. Product-specific standard contracts should be seed fixtures or builders such as `Contract::virbac_calmcare(...)`, `Contract::purina_pro_plan_supplement(...)`, and `Contract::purina_en_boarding_diet(...)` only if they remain deterministic test fixtures rather than hard-coded production catalog.

Related existing tests to preserve/extend:

- `storage/tests/core_service_contract_storage.rs` verifies `CoreServiceContracts` roundtrip, retail contract JSON codec, and `retail.should_reorder()`.
- `storage/tests/operations_storage_contracts.rs` verifies service-offering variant codecs and cross-variant shape rejection.
- `domain/tests/domain_quality_patterns.rs` verifies operations semantics, builders, newtypes, workflow/agent boundaries, and sensitive care/temperament redaction.

## Required newtypes, enums, builders, policies, repositories, services

Newtypes and value objects:

- `operations::retail::ProductName`
  - Invariant: trimmed, non-empty, bounded display length.
- `operations::retail::VendorCatalogId`
  - Invariant: trimmed, non-empty; adapter-specific IDs stay at the boundary unless a vendor catalog workflow depends on them.
- `operations::retail::Sku`
  - Existing invariant: trimmed non-empty. Consider max length and allowed character policy before external POS reconciliation.
- `operations::retail::OnHandUnits`
  - Invariant: zero or positive; use when stock can be depleted.
- `operations::retail::ReservedUnits`
  - Invariant: zero or positive; must never exceed on-hand when calculating available-for-sale.
- `operations::retail::ReorderThreshold`
  - Invariant: positive; semantically distinct from on-hand.
- `operations::retail::OrderQuantity`
  - Invariant: positive; separate from threshold and on-hand.
- `operations::retail::RecommendationRationale`
  - Invariant: trimmed, non-empty, bounded; should not contain sensitive medical detail intended only for internal review.
- `operations::retail::RecommendationConfidence`
  - Invariant: bounded score or discrete enum; prefer enum if the business uses bands rather than decimals.

Enums:

- `operations::retail::Partner`
  - `Virbac`, `Purina`, future `Other { name }` only when necessary.
- `operations::retail::ProductFamily`
  - `VirbacCalmCare`, `PurinaProPlanVeterinarySupplements`, `PurinaEnBoardingDiet`, future family variants.
- `operations::retail::Category`
  - `Supplement`, `InHouseDiet`, `PersonalizedUpsell`, plus future categories after they carry behavior.
- `operations::retail::UsageMode`
  - `Sellable`, `InHouseConsumable`, `SellableAndInHouseConsumable`.
- `operations::retail::InventoryAvailability`
  - `InStock`, `LowStock`, `OutOfStock`, `Backordered`, `Discontinued`, `Unknown`.
- `operations::retail::RecommendationReason`
  - Explicit reasons listed above; should replace raw rationale categories.
- `operations::retail::RecommendationDecision`
  - `AllowedInternalPrompt`, `StaffReviewRequired`, `ManagerReviewRequired`, `Forbidden { reason }`.
- `operations::retail::SaleSource`
  - `StandaloneRetail`, `ReservationCheckout { reservation_id }`, `BoardingInHouseDiet { reservation_id }`, `StaffEntered`, `AgentDrafted`.
- `operations::retail::CompPolicy`
  - `NotAllowed`, `ManagerApprovalRequired`, `AllowedByPolicy { reason }`.
- `operations::retail::StockMovementKind`
  - `Received`, `Sold`, `ConsumedDuringStay`, `ReservedForStay`, `ReleasedFromReservation`, `Adjusted`, `ExpiredOrWasted`, `Returned`, `Transferred`.
- `operations::retail::ReorderDecision`
  - `NoAction`, `CreateStaffTask`, `DraftVendorOrder`, `ManagerReviewRequired`, `VendorManaged`.

Builders and aggregate contracts:

- `operations::retail::PartnerProduct::builder()`
  - Required: SKU, product name, partner, product family, category, usage mode.
  - Optional: vendor catalog ID, default recommendation policy, default reorder policy.
- `operations::retail::LocationOffering::builder()`
  - Required: location, product, POS policy, inventory policy, recommendation policy, reorder policy.
  - Optional: price/money contract, taxability, active/inactive status.
- `operations::retail::InventoryPosition::new(on_hand, reserved, reorder_threshold)`
  - Invariant: reserved units cannot exceed on-hand units; available-for-sale is derived, not stored as a mutable raw number.
- `operations::retail::RecommendationCandidate::builder()`
  - Required: customer, pet, location, product, reason, source context, rationale, decision/review gate.
  - Optional: reservation, care-profile evidence, inventory snapshot, price estimate.
- `operations::retail::SaleLine::builder()`
  - Required: product/SKU, quantity, money amount, source, approval state.
  - Optional: reservation/customer/pet references and discount/comp details.

Policies:

- `operations::retail::RecommendationPolicy`
  - Inputs: product, inventory availability, care profile, reservation context, pet watch/review flags, customer preference/purchase history, and proposed audience.
  - Outputs: `RecommendationDecision` and review gate.
  - Invariants: no diagnosis, no unsafe diet/supplement claim, no member-facing send without approval, out-of-stock products cannot be customer-promised.
- `operations::retail::InventoryPolicy` / `ReorderPolicy`
  - Inputs: inventory position, future reservations using in-house diet, lead time/vendor status, reorder thresholds.
  - Outputs: `ReorderDecision` and staff task draft.
  - Invariants: reorder recommendation never mutates stock; vendor order creation needs manager/vendor approval unless a separate policy explicitly allows it.
- `operations::retail::PointOfSalePolicy`
  - Inputs: sale source, customer/reservation/payment context, staff role, comp/discount request.
  - Outputs: allowed sale draft or denial/review gate.
  - Invariants: comp/discount policy is explicit; payment capture remains in money/payment/tool boundary.

Repositories and domain services:

- `operations::retail::Repository`
  - `catalog_item(sku)`, `location_offering(location_id, sku)`, `inventory_position(location_id, sku)`, `record_stock_movement(...)`, `save_recommendation_candidate(...)`.
- `operations::retail::RecommendationRepository`
  - Stores candidate state, review outcome, customer-facing draft linkage, and audit evidence.
- `operations::retail::InventoryRepository`
  - Stores stock positions/movements and idempotency keys for POS/vendor sync.
- `operations::retail::RecommendationService`
  - Creates candidates from care/reservation/customer context and policy.
- `operations::retail::ReorderService`
  - Computes reorder signals and staff/vendor draft tasks.
- `operations::retail::CheckoutService`
  - Builds POS sale-line drafts that payment/POS tools may later authorize under review policy.

## Relationships to neighboring modules

Customer:

- Retail recommendations reference `entities::CustomerId` for purchase history, preferred contact, portal account, and customer-facing draft ownership.
- Customer preferences and opt-outs should be represented before automated upsell messages. Do not infer consent from purchase history alone.

Pet:

- Recommendations reference `entities::PetId` and pet profile facts.
- Virbac CalmCare candidates may be triggered by `operations::PetCareWatchReason::AnxietyOrStressFlag`, temperament observations, training/daycare notes, or staff-entered concern, but must not state diagnosis.
- Product candidates should account for species, age/size if relevant, allergies, diet restrictions, and current medical/care review requirements.

Reservation:

- Purina EN in-house diet connects to `entities::ReservationId`, boarding stays, check-in prep, feeding support, and checkout summaries.
- Reservation checkout is a sale source, not proof that the customer approved a retail purchase.
- Future code should distinguish reservation-consumed inventory from customer-sold inventory with `StockMovementKind` and `SaleSource`.

Care profile:

- `care` owns feeding instructions, allergies, medical conditions, medication instructions, contacts, and sensitive review details.
- Retail may consume care facts as policy inputs but should not own medical facts.
- Diet/supplement recommendations need staff review when care profile contains allergies, conditions, new medications, special feeding instructions, or ambiguous notes.

Location:

- `entities::LocationId` scopes product availability, inventory, price, POS policy, active/inactive status, reorder thresholds, and vendor workflows.
- A product may be active at one location and unavailable or vendor-managed at another.

Staff task:

- Reorder signals may create `operations::StaffTask` for front desk/manager/vendor-order review.
- Recommendation candidates may create staff review tasks before a draft can be shown to a customer.
- Suggested extensions to `StaffTaskKind`: `RetailReorderReview { sku, location_id }`, `RetailRecommendationReview { customer_id, pet_id, sku }`, and `RetailInventoryAdjustmentReview { sku, location_id }` when implementation needs task specificity.

Money/payment:

- Price, discounts, comps, taxes, payment capture, and refunds belong to `money`, `payment`, and external tool adapters.
- Retail should produce semantic sale-line drafts and approval decisions; payment tools authorize/capture only after policy gates pass.

Workflow/agent modules:

- Agents may create `workflow::RecommendedAction` values or `operations::OperationsAction` drafts that route to staff review.
- Retail-specific action payloads should remain typed; avoid generic `{ action: "upsell", sku: String }` blobs.
- Agent specs must declare allowed tools, forbidden actions, output schema, and review gates.

## AI-agent opportunities and approval boundaries

Safe automation:

- Classify product catalog entries into semantic retail categories from approved internal source data.
- Detect low-stock/reorder candidates and create internal review tasks.
- Summarize inventory risk for manager daily briefs.
- Draft staff-facing recommendation candidates with rationale and evidence links.
- Draft customer-facing upsell copy for staff review without sending.
- Match future boarding reservations that indicate in-house Purina EN consumption against inventory projections.
- Flag inconsistent SKU/category/partner mappings across locations for data-quality cleanup.

Staff review required:

- Any customer-facing product recommendation or upsell message.
- Any recommendation based on care profile, anxiety/stress behavior, diet instructions, medication, allergy, or medical-condition context.
- Any attachment of a retail sale line to a reservation checkout.
- Any substitution when a product is out of stock or unavailable.
- Any inventory adjustment that is not directly reconciled from POS/vendor receipt data.

Manager review required:

- Complimentary products, discounts, refunds, or charge reversals.
- Vendor order approval unless a future policy explicitly allows auto-ordering.
- Recommendations involving incident follow-up, safety concern, complaint recovery, or customer dissatisfaction.
- Policy changes to reorder thresholds, partner availability, POS rules, or customer-message templates.
- Any recommendation that could look like veterinary medical advice.

Member-facing or unsafe without explicit approval:

- Sending a retail/supplement/diet recommendation directly to a customer.
- Claiming a product treats, cures, diagnoses, or prevents a condition.
- Changing feeding instructions or care plans based solely on AI inference.
- Selling/charging a product, creating a payment, issuing a refund, or applying a comp.
- Creating or submitting a vendor purchase order.
- Hiding low-stock/out-of-stock facts from staff or promising unavailable inventory to a customer.

Recommended agent contracts:

- `retail-inventory-watch`: reads inventory/POS/vendor snapshots, emits reorder signals and staff tasks.
- `retail-recommendation-drafter`: reads approved customer/pet/reservation/care context, emits recommendation candidates and staff-review drafts.
- `boarding-diet-forecast`: reads upcoming boarding reservations and in-house diet flags, emits projected Purina EN depletion risk.
- `retail-data-quality`: detects unknown partner/product/category/SKU mismatches and routes cleanup tasks.

Each agent should output typed contracts such as `operations::retail::RecommendationCandidate`, `operations::retail::ReorderSignal`, or `workflow::RecommendedAction` rather than free-form instructions.

## Acceptance tests and contracts for later code cards

Domain-quality tests:

- `retail_skus_and_product_names_are_trimmed_non_empty_domain_values`
  - Proves `Sku` and `ProductName` validate/sanitize and reject blank values.
- `retail_inventory_position_derives_available_units_without_raw_math_at_call_sites`
  - Proves on-hand minus reserved is owned by `InventoryPosition`, and reserved cannot exceed on-hand.
- `retail_reorder_policy_creates_manager_task_when_stock_is_below_threshold`
  - Proves reorder decisions are semantic enum values and staff task drafts carry typed location/SKU context.
- `retail_reorder_policy_accounts_for_boarding_diet_reservations_before_depletion`
  - Proves Purina EN in-house consumption forecast can trigger projected-depletion review before on-hand falls below threshold.
- `retail_recommendation_policy_routes_care_sensitive_supplement_candidates_to_staff_review`
  - Proves anxiety/diet/supplement suggestions do not become autonomous customer messages.
- `retail_recommendation_policy_forbids_medical_claims_in_customer_drafts`
  - Proves unsafe wording or diagnosis-like rationale is rejected or manager-reviewed.
- `retail_pos_policy_requires_manager_approval_for_comps_and_discounts`
  - Proves comp/discount decisions are explicit enum outcomes, not booleans.
- `retail_sale_line_preserves_money_quantity_source_and_review_state`
  - Proves POS draft line items carry `money::Money`, quantity, sale source, and approval status.

Storage/boundary tests:

- `retail_location_offering_records_roundtrip_between_storage_and_domain`
  - Proves catalog/offering/inventory fields codec cleanly and preserve semantic newtype invariants.
- `retail_inventory_records_reject_negative_or_shape_mismatched_stock_fields`
  - Proves raw provider data cannot create impossible domain state.
- `retail_service_offering_records_preserve_partner_product_variants`
  - Extends existing service-offering codec coverage for `RetailPartnerProduct` variants.
- `retail_recommendation_records_preserve_review_gate_and_reason`
  - Proves candidate persistence does not erase approval boundary or rationale semantics.

Workflow/agent tests:

- `retail_inventory_agent_can_create_internal_reorder_task_but_not_vendor_order`
  - Proves allowed/forbidden actions are encoded in agent spec and output validation.
- `retail_recommendation_agent_drafts_staff_review_packet_not_customer_send`
  - Proves customer-facing messages remain drafts until approval.
- `retail_boarding_diet_forecast_surfaces_manager_attention_when_purina_en_stock_will_deplete`
  - Proves manager daily brief integration.
- `retail_data_quality_agent_routes_unknown_partner_product_to_cleanup_task`
  - Proves extensibility without raw-string branching in the domain core.

Contracts to preserve:

- Retail behavior must remain deterministic in Rust policies/services after an agent drafts or classifies.
- Raw provider/POS/vendor strings must be promoted to semantic values at the boundary.
- Customer-facing actions, payment actions, product substitutions, medical/diet-sensitive recommendations, and vendor orders require typed approval decisions.
- Product inventory and POS sale concepts should remain separate even when a product is both in-house consumable and sellable.
- `operations::retail` should be the truthful owner of retail product, recommendation, inventory, reorder, and POS policy vocabulary; neighboring modules supply identities and facts but should not own retail policy behavior.
