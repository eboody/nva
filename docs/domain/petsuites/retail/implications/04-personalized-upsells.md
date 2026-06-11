# Retail / partner products implication 04: Personalized upsells

## Purpose and modeling assumptions

This document fleshes out the Retail / partner-products operational implication `Personalized upsells` for later Rust/domain implementation. It is a modeling artifact, not an implementation patch.

Source context:

- `docs/domain/petsuites/retail/service-domain-map.md`
- Current `domain/src/operations.rs` retail surface:
  - `operations::ServiceOffering::RetailPartnerProduct { partner, category }`
  - `operations::RetailPartner::{VirbacCalmCare, PurinaProPlanVeterinarySupplements, PurinaEnBoardingDiet}`
  - `operations::RetailProductCategory::{Supplement, InHouseDiet, PersonalizedUpsell}`
  - `operations::retail::{Contract, Product, Sku, PointOfSalePolicy, InventoryPolicy, UnitCount, RecommendationRule, ReorderPolicy}`
  - `operations::{OperationsAction, StaffTask, RevenueOpportunity}` for staff-review and manager-brief integration.
- Current domain tests under `domain/tests/`, especially `petsuites_core_service_contracts.rs`.

Safe assumptions:

- A personalized upsell is a reviewed sales opportunity, not an automatic sale, charge, product promise, or customer send.
- Recommendations may use care-profile, pet, reservation, inventory, customer preference, and purchase-history facts, but retail does not own medical diagnosis, feeding instructions, medication instructions, or payment capture.
- The safest extensible default is `staff review required` for any customer-facing product recommendation, and `manager review required` when a product claim, substitution, comp/discount, complaint recovery, incident follow-up, or care-sensitive ambiguity is involved.
- Out-of-stock products may be suggested internally as alternatives/reorder prompts, but must not be promised to customers.

## Operational story

### Trigger

A personalized upsell opportunity is considered when one or more domain events or workflows supply enough context to evaluate a location-specific product offering:

- A reservation is created, modified, checked in, checked out, or summarized.
- Boarding/daycare/training/grooming notes create a care-support opportunity, such as stress/anxiety observation, diet-continuity support, coat/skin care context, or post-service follow-up.
- A customer has prior retail purchase history suggesting replenishment or continuation.
- A staff member manually asks for a retail recommendation packet for a specific pet/reservation.
- A manager daily brief requests revenue opportunities that are inventory-aware and safe to review.

The trigger creates an internal `operations::retail::UpsellOpportunityContext`; it does not create a sale line, payment, vendor order, or customer message by itself.

### Actors

- Customer: receives only staff-approved customer-facing copy or checkout prompts.
- Pet: provides species, profile, reservation, care, and behavior context through neighboring modules.
- Front desk / counselor / service staff: reviews ordinary recommendation packets, edits customer-safe copy, and records acceptance/decline.
- Manager: approves high-risk recommendations, substitutions, discounts/comps, complaint/incident recovery recommendations, and policy/template changes.
- Retail/inventory coordinator: verifies SKU availability, substitutions, and reorder context.
- Retail recommendation agent: drafts internal candidates and staff-review packets under an explicit agent spec.
- POS/payment adapters: can later execute approved sale-line drafts, but do not own retail recommendation policy.

### Inputs

Inputs should be promoted into semantic domain values before policy evaluation:

- `entities::LocationId`, `entities::CustomerId`, `entities::PetId`, and optional `entities::ReservationId`.
- `operations::retail::LocationOffering` with product, SKU, category/family, POS policy, inventory snapshot, price estimate, and active/inactive status.
- `operations::retail::InventoryPosition` and `operations::retail::InventoryAvailability`.
- `operations::retail::RecommendationReason` and typed evidence references from care/reservation/service history.
- Customer retail preferences: opt-out state, prior purchase summary, preferred contact boundary, and do-not-contact constraints.
- Care-profile facts supplied by `care`: allergies, diet restrictions, medication/medical review flags, feeding instructions, and sensitive notes.
- Reservation/service context supplied by boarding/daycare/training/grooming modules.
- Staff role and intended audience: internal-only, staff-review packet, manager-review packet, customer-facing draft, checkout prompt.

### Decisions

Personalized upsell policy makes separate decisions instead of collapsing everything into a boolean:

1. Candidate eligibility: Is this product offering relevant to this pet/customer/reservation context?
2. Inventory promise boundary: Is the product in stock, low stock, unavailable, backordered, discontinued, or in-house-only?
3. Care-sensitivity boundary: Does the rationale depend on allergy, diet, medication, medical condition, stress/anxiety observation, incident, or ambiguous staff notes?
4. Customer-contact boundary: Does the customer allow retail recommendations through the proposed channel?
5. Approval gate: Is the candidate internal-only, staff-review-required, manager-review-required, or forbidden?
6. POS boundary: If the customer accepts, can the product become a sale-line draft, reservation-checkout attachment, or manager-only comp request?
7. Audit boundary: What evidence, policy version, reviewer, and final disposition must be recorded?

### Outputs

Allowed outputs:

- `operations::retail::UpsellCandidate`: internal candidate with typed reason, evidence, product, location, inventory state, and review gate.
- `operations::retail::UpsellReviewPacket`: staff/manager review packet with customer-safe copy draft, internal rationale, policy decision, inventory notes, and audit requirements.
- `operations::StaffTask` with `RetailUpsellReview { customer_id, pet_id, sku, reservation_id }` or equivalent future task kind.
- `operations::OperationsAction::SuggestRevenueFollowUp` or a future typed retail action for manager briefs.
- `operations::retail::SaleLineDraft` only after customer interest is recorded and POS policy allows a reviewed sale draft.
- `operations::retail::UpsellAuditEvent` recording candidate creation, review, edit, approval, rejection, customer response, sale-line draft creation, and exception handling.

Forbidden outputs without explicit approval:

- Sending a customer message.
- Adding a product charge, capturing payment, applying a comp/discount, or issuing a refund.
- Substituting a product when the selected product is unavailable.
- Creating or submitting a vendor order.
- Changing a care plan, feeding instruction, or medical/diet instruction.

### Success state

A successful personalized upsell workflow reaches one of these terminal states:

- `DeclinedByPolicy`: policy forbids the candidate before staff time is spent.
- `DismissedByStaff`: staff reviews and decides not to contact the customer.
- `CustomerDraftApproved`: customer-facing draft is approved and ready for an external messaging/checkout workflow.
- `AcceptedSaleDrafted`: customer accepts and an approved `SaleLineDraft` is available for POS/payment integration.
- `ManagerResolved`: manager approves, rejects, or edits a high-risk candidate and the audit trail captures the outcome.

Success never means autonomous customer outreach or autonomous sale execution.

### Failure and exception states

- `MissingRequiredContext`: no customer, pet, product offering, location, or evidence reference.
- `CustomerOptedOut`: customer preferences or channel rules block outreach.
- `NoActiveLocationOffering`: SKU is not active or available at the location.
- `OutOfStockOrUnavailable`: inventory state blocks customer promise; may create internal reorder/substitution review.
- `CareReviewRequired`: allergy, medication, diet, condition, stress/anxiety, incident, or ambiguous note requires staff/manager review.
- `UnsafeClaimRejected`: rationale or customer copy implies diagnosis, treatment, cure, prevention, or unsupported veterinary advice.
- `ConflictingEvidence`: care/reservation/customer facts disagree and need human review.
- `PriceOrTaxUnavailable`: retail can draft a candidate but cannot form a sale-line draft until money/POS data is available.
- `PolicyVersionChanged`: candidate must be re-evaluated if approval policy changed after the draft was created.
- `StaleInventory`: inventory snapshot is too old to promise or checkout.

## Domain types to add or refine

### Value objects and newtypes

- `operations::retail::ProductName`
  - Invariant: trimmed, non-empty, bounded display length; customer-safe display must not be a raw vendor string.
- `operations::retail::ProductFamily`
  - Enum candidates: `VirbacCalmCare`, `PurinaProPlanVeterinarySupplements`, `PurinaEnBoardingDiet`, future variants.
- `operations::retail::Partner`
  - Prefer a retail-owned enum or re-export around current `operations::RetailPartner` when behavior moves into `operations::retail`.
- `operations::retail::OfferingStatus`
  - Enum: `Active`, `Inactive`, `InHouseOnly`, `Discontinued`, `PendingDataQualityReview`.
- `operations::retail::UpsellCandidateId`
  - Invariant: stable identity for audit/review; adapter IDs stay at the boundary unless they become workflow identity.
- `operations::retail::RecommendationRationale`
  - Invariant: trimmed, non-empty, bounded; internal rationale can cite evidence but must separate customer-safe phrasing.
- `operations::retail::CustomerSafeCopy`
  - Invariant: trimmed, non-empty, bounded; rejects or escalates medical claims, diagnosis language, hidden urgency, and unsupported promises.
- `operations::retail::PolicyVersion`
  - Invariant: non-empty immutable policy identifier used for re-evaluation and audit.
- `operations::retail::InventorySnapshotAt`
  - Timestamp wrapper proving when the inventory state was evaluated.
- `operations::retail::OnHandUnits`, `ReservedUnits`, `AvailableUnits`, `ReorderThreshold`
  - Refine current positive-only `UnitCount` so zero stock is representable and available units are derived, not stored as mutable raw math.

### Entities and aggregate contracts

- `operations::retail::PartnerProduct`
  - Product identity independent of location inventory.
  - Required fields: `Sku`, `ProductName`, `Partner`, `ProductFamily`, `Category`, `UsageMode`.
- `operations::retail::LocationOffering`
  - Location-specific product contract.
  - Required fields: `LocationId`, `PartnerProduct`, `OfferingStatus`, POS policy, inventory policy, recommendation policy, price/taxability reference if saleable.
- `operations::retail::UpsellOpportunityContext`
  - Typed input packet for policy evaluation.
  - Required fields: customer, pet, location, intended audience, trigger, evidence references.
  - Optional fields: reservation, service line, prior purchases, care-profile summary, price estimate.
- `operations::retail::UpsellCandidate`
  - Internal recommendation candidate.
  - Required fields: ID, product/offering, customer, pet, location, reason, rationale, inventory state, decision/review gate, policy version.
- `operations::retail::UpsellReviewPacket`
  - Staff/manager review aggregate containing the candidate, internal rationale, customer-safe draft, inventory/POS notes, and required actions.
- `operations::retail::SaleLineDraft`
  - POS-ready but not payment-executed line draft containing SKU, quantity, price/money reference, source, customer/reservation references, and approval state.
- `operations::retail::UpsellAuditEvent`
  - Append-only workflow event capturing who/what created, reviewed, edited, approved, rejected, sent, accepted, declined, or converted the candidate.

### Enums and policies

- `operations::retail::UpsellTrigger`
  - `ReservationBooked`, `CheckInPrep`, `CheckoutPrep`, `PostStayFollowUp`, `ServiceNoteCreated`, `PriorPurchaseReplenishment`, `StaffRequested`, `ManagerBriefRequested`.
- `operations::retail::RecommendationReason`
  - Refine current coarse `RecommendationRule` into specific reasons: `AnxietyOrStressSupport`, `BoardingDietContinuity`, `PriorPurchaseReplenishment`, `StaffRecommendedCareSupport`, `CheckoutContinuation`, `CoatOrSkinCareSupport`, `ManagerInitiated`.
- `operations::retail::InventoryAvailability`
  - `InStock`, `LowStock`, `OutOfStock`, `Backordered`, `Discontinued`, `Unknown`.
- `operations::retail::UpsellDecision`
  - `InternalOnly`, `StaffReviewRequired`, `ManagerReviewRequired`, `Forbidden { reason: UpsellDenialReason }`.
- `operations::retail::UpsellDenialReason`
  - `CustomerOptedOut`, `NoActiveOffering`, `InventoryUnavailable`, `UnsafeMedicalClaim`, `CarePlanConflict`, `UnsupportedSpeciesOrPetProfile`, `MissingRequiredEvidence`, `PriceUnavailable`, `PolicyChanged`.
- `operations::retail::ReviewGate`
  - `NoCustomerAction`, `StaffApproval`, `ManagerApproval`, `CareTeamApproval`, `PaymentOrPosApproval`.
- `operations::retail::CustomerDisposition`
  - `NotContacted`, `Sent`, `Accepted`, `Declined`, `Deferred`, `NoResponse`.
- `operations::retail::SaleDraftState`
  - `NotRequested`, `DraftedForStaff`, `ApprovedForCheckout`, `Rejected`, `ConvertedToPosLine`.

## Relationship map

### Entities

- `entities::CustomerId`
  - Owns customer identity and links to preferences/opt-outs and purchase history.
- `entities::PetId`
  - Owns pet identity and links to profile/care/reservation facts.
- `entities::ReservationId`
  - Optional context for boarding/daycare/grooming/training stay/service lifecycle and checkout source.
- `entities::LocationId`
  - Scopes product availability, POS policy, price, inventory, templates, and staff review routing.
- `operations::retail::PartnerProduct`
  - Owns stable product identity.
- `operations::retail::LocationOffering`
  - Owns location-specific product contract.
- `operations::retail::UpsellCandidate`
  - Owns an evaluated recommendation opportunity.
- `operations::retail::UpsellReviewPacket`
  - Owns the review-ready representation of a candidate.

### Value objects

- `operations::retail::{Sku, ProductName, RecommendationRationale, CustomerSafeCopy, PolicyVersion}` protect retail-specific semantics.
- `operations::retail::{OnHandUnits, ReservedUnits, AvailableUnits, InventorySnapshotAt}` protect inventory math and staleness boundaries.
- `money::{Money, TaxPolicy}` or equivalent future money types remain outside retail; retail references them in `SaleLineDraft`.

### Policies

- `operations::retail::UpsellPolicy`
  - Owns recommendation eligibility, care-sensitivity, customer-contact, and review-gate decisions.
- `operations::retail::InventoryPromisePolicy`
  - Owns whether inventory can be shown, promised, reserved, or must become an internal review/reorder task.
- `operations::retail::CustomerCopyPolicy`
  - Owns copy safety checks and medical-claim escalation.
- `operations::retail::PointOfSalePolicy`
  - Existing owner of standalone / reservation checkout / manager-only comp distinctions; later refine to produce sale-line decisions.

### Repositories and stores

- `operations::retail::CatalogRepository`
  - Loads partner products and location offerings by semantic SKU/location.
- `operations::retail::InventoryRepository`
  - Loads stock positions, records inventory snapshot references, and supports stale-inventory checks.
- `operations::retail::UpsellRepository`
  - Saves candidates, review packets, approval outcomes, dispositions, sale-draft linkage, and audit events.
- `operations::retail::CustomerRetailPreferenceRepository`
  - Reads opt-outs/preferences through a retail-facing contract while customer remains the semantic owner of customer identity.
- `operations::retail::PurchaseHistoryRepository`
  - Reads normalized purchase facts without leaking raw POS rows into policy code.

### Workflow events

- `operations::retail::UpsellCandidateCreated`
- `operations::retail::UpsellReviewTaskCreated`
- `operations::retail::UpsellCustomerCopyEdited`
- `operations::retail::UpsellApproved`
- `operations::retail::UpsellRejected`
- `operations::retail::UpsellSentToCustomer`
- `operations::retail::UpsellAcceptedByCustomer`
- `operations::retail::SaleLineDrafted`
- `operations::retail::UpsellExpiredDueToInventoryOrPolicyChange`

### Staff tasks

Suggested future `operations::StaffTaskKind` variants:

- `RetailUpsellReview { customer_id, pet_id, sku, reservation_id }`
- `RetailManagerUpsellReview { customer_id, pet_id, sku, reason }`
- `RetailCustomerCopyReview { customer_id, pet_id, candidate_id }`
- `RetailSaleLineApproval { customer_id, pet_id, sku, source }`
- `RetailInventoryExceptionReview { sku, location_id, availability }`

Until those variants exist, use existing `StaffTaskKind::CustomerFollowUp` or `OperationsAction::CreateInternalTask` only as a bridge; do not hide retail semantics in untyped task titles long term.

### Agent specs and tools

- `retail-recommendation-drafter`
  - Reads approved semantic customer/pet/reservation/care/product inputs.
  - Emits `UpsellCandidate` and `UpsellReviewPacket` drafts only.
  - Forbidden: sends customer messages, charges customers, changes care plans, submits vendor orders.
- `retail-copy-safety-reviewer`
  - Checks `CustomerSafeCopy` for medical claims, unsupported promises, manipulative urgency, and hidden low-stock claims.
  - Emits `CustomerCopyDecision` and suggested edits.
- `retail-checkout-assistant`
  - After staff/customer approval, creates `SaleLineDraft` for POS integration.
  - Forbidden: captures payment or applies comps unless manager approval state is already present.

## Interaction contract

Rust-like pseudo-signatures intentionally name behavior owners instead of free-floating helpers.

```rust
impl operations::retail::UpsellOpportunityContext {
    pub fn builder() -> UpsellOpportunityContextBuilder<MissingCustomer, MissingPet, MissingLocation, MissingTrigger>;
    pub fn intended_audience(&self) -> operations::retail::IntendedAudience;
    pub fn contains_care_sensitive_evidence(&self) -> bool;
}
```

`UpsellOpportunityContext` owns input completeness and simple evidence classification. It must not decide policy outcomes by itself.

```rust
impl operations::retail::LocationOffering {
    pub fn can_be_recommended_for(&self, audience: IntendedAudience) -> OfferingRecommendationBoundary;
    pub fn inventory_availability(&self, position: InventoryPosition) -> InventoryAvailability;
    pub fn can_form_sale_line(&self, source: SaleSource) -> SaleLineBoundary;
}
```

`LocationOffering` owns product availability/POS capability for the location. It should not inspect medical facts or customer preferences.

```rust
pub trait operations::retail::CatalogRepository {
    fn location_offering(
        &self,
        location_id: entities::LocationId,
        sku: &operations::retail::Sku,
    ) -> operations::retail::Result<Option<operations::retail::LocationOffering>>;

    fn active_recommendable_offerings(
        &self,
        location_id: entities::LocationId,
        reason: operations::retail::RecommendationReason,
    ) -> operations::retail::Result<Vec<operations::retail::LocationOffering>>;
}
```

The catalog repository owns product/offering lookup, not recommendation scoring.

```rust
pub trait operations::retail::InventoryRepository {
    fn inventory_position(
        &self,
        location_id: entities::LocationId,
        sku: &operations::retail::Sku,
    ) -> operations::retail::Result<operations::retail::InventoryPosition>;

    fn is_snapshot_fresh(
        &self,
        snapshot_at: operations::retail::InventorySnapshotAt,
    ) -> bool;
}
```

Inventory repository owns current stock facts and snapshot freshness. It must not promise customer availability.

```rust
impl operations::retail::UpsellPolicy {
    pub fn evaluate(
        &self,
        context: &operations::retail::UpsellOpportunityContext,
        offering: &operations::retail::LocationOffering,
        inventory: &operations::retail::InventoryPosition,
        preferences: &operations::retail::CustomerRetailPreferences,
    ) -> operations::retail::UpsellDecision;
}
```

`UpsellPolicy` owns eligibility and review-gate decisions. It should return semantic outcomes, never booleans such as `approved: bool`.

```rust
impl operations::retail::CustomerCopyPolicy {
    pub fn draft_customer_copy(
        &self,
        candidate: &operations::retail::UpsellCandidate,
        template: &operations::retail::CustomerCopyTemplate,
    ) -> operations::retail::Result<operations::retail::CustomerCopyDecision>;
}

pub enum operations::retail::CustomerCopyDecision {
    ApprovedDraft { copy: CustomerSafeCopy },
    NeedsStaffEdit { reasons: Vec<CustomerCopyConcern> },
    ManagerReviewRequired { reasons: Vec<CustomerCopyConcern> },
    Forbidden { reason: UpsellDenialReason },
}
```

`CustomerCopyPolicy` owns customer-safe wording. It rejects medical claims and unsupported promises before staff can approve a draft.

```rust
impl operations::retail::RecommendationService {
    pub fn create_candidate(
        &self,
        context: operations::retail::UpsellOpportunityContext,
        sku: operations::retail::Sku,
    ) -> operations::retail::Result<operations::retail::UpsellCandidate>;

    pub fn create_review_packet(
        &self,
        candidate_id: operations::retail::UpsellCandidateId,
    ) -> operations::retail::Result<operations::retail::UpsellReviewPacket>;
}
```

The service coordinates repositories and policies. It does not send messages or capture payment.

```rust
pub trait operations::retail::UpsellRepository {
    fn save_candidate(&self, candidate: operations::retail::UpsellCandidate) -> operations::retail::Result<()>;
    fn save_review_packet(&self, packet: operations::retail::UpsellReviewPacket) -> operations::retail::Result<()>;
    fn record_audit_event(&self, event: operations::retail::UpsellAuditEvent) -> operations::retail::Result<()>;
    fn candidate(&self, id: operations::retail::UpsellCandidateId) -> operations::retail::Result<Option<operations::retail::UpsellCandidate>>;
}
```

`UpsellRepository` owns durable state and audit events, not policy evaluation.

```rust
impl operations::retail::PointOfSalePolicy {
    pub fn evaluate_sale_line(
        &self,
        candidate: &operations::retail::UpsellCandidate,
        customer_disposition: operations::retail::CustomerDisposition,
        staff_role: operations::StaffRole,
        price: money::Money,
    ) -> operations::retail::SaleLineDecision;
}

pub enum operations::retail::SaleLineDecision {
    DraftAllowed { draft: SaleLineDraft },
    StaffReviewRequired { reason: SaleLineReviewReason },
    ManagerReviewRequired { reason: SaleLineReviewReason },
    Forbidden { reason: UpsellDenialReason },
}
```

The POS policy owns sale-line draft permission. Payment capture remains outside retail.

## Review and approval contract

### Automation level

- Agent/autonomous workflows may classify, draft, score, and route internal candidates.
- Agents may create staff/manager review tasks with typed evidence.
- Agents may generate customer-safe copy drafts only through `CustomerCopyPolicy`.
- Agents may not send messages, attach checkout sale lines, charge payment methods, apply discounts/comps, change care plans, or submit vendor orders.

### Staff review gates

Staff approval is required before:

- Any customer-facing product recommendation is sent or shown.
- Any reservation-checkout prompt is attached to a customer flow.
- Any care-profile, anxiety/stress, diet, medication, allergy, medical-condition, or ambiguous note influences the rationale.
- Any staff-edited copy replaces generated copy.
- Any product is recommended while inventory is low or stale but still possibly available.

### Manager review gates

Manager approval is required before:

- A recommendation references incident follow-up, safety concern, customer complaint, dissatisfaction, or recovery gesture.
- Copy could be interpreted as veterinary medical advice or as treating/curing/preventing a condition.
- A product substitution is proposed because a selected product is unavailable.
- A discount, comp, refund, charge reversal, or exception price is requested.
- A policy, threshold, template, or partner-product availability rule changes.

### Audit trail

Every candidate must record:

- Candidate ID, policy version, creation timestamp, trigger, location, customer, pet, optional reservation.
- Product/offering identity: SKU, product family, category, partner, offering status.
- Evidence references, with sensitive details redacted from customer-facing copy.
- Inventory snapshot and staleness decision.
- Decision/review gate and denial/escalation reason when applicable.
- Generated draft, human edits, reviewer identity/role, approval/rejection timestamp.
- Customer disposition and sale-line draft linkage if the workflow proceeds.

### Customer/member-facing boundaries

Customer-facing content must:

- Be explicitly approved by staff or manager.
- Avoid diagnosis, treatment, cure, prevention, urgency pressure, or unsupported claims.
- Reflect inventory truth without promising unavailable stock.
- Respect opt-outs and channel preferences.
- Separate recommendation acceptance from payment authorization.

## Test contracts

Domain-quality tests:

- `retail_personalized_upsell_context_requires_customer_pet_location_trigger_and_evidence`
  - Proves the builder prevents incomplete recommendation contexts.
- `retail_upsell_policy_routes_care_sensitive_supplement_to_staff_review`
  - Proves anxiety/diet/supplement recommendations cannot become autonomous customer outreach.
- `retail_upsell_policy_forbids_customer_contact_when_customer_has_retail_opt_out`
  - Proves customer preferences are policy inputs, not afterthought flags.
- `retail_upsell_policy_requires_manager_review_for_incident_recovery_or_comps`
  - Proves high-risk revenue recovery does not route to ordinary staff approval.
- `retail_customer_safe_copy_rejects_medical_claims_and_unsupported_promises`
  - Proves draft copy policy owns claim safety.
- `retail_location_offering_blocks_customer_promise_for_out_of_stock_products`
  - Proves unavailable products cannot be promised even when relevant.
- `retail_inventory_position_represents_zero_stock_and_derives_available_units`
  - Proves the future inventory model fixes positive-only `UnitCount` limitations.
- `retail_sale_line_draft_requires_customer_acceptance_and_pos_policy_approval`
  - Proves a recommendation candidate is not a sale.
- `retail_upsell_audit_event_records_policy_version_review_gate_and_human_outcome`
  - Proves audit state preserves the approval boundary.

Storage/boundary tests:

- `retail_upsell_candidate_records_roundtrip_without_erasing_review_gate`
  - Candidate persistence preserves reason, decision, evidence references, and policy version.
- `retail_customer_copy_records_preserve_internal_rationale_separate_from_customer_copy`
  - Sensitive/internal rationale cannot leak into customer-facing payload by shape accident.
- `retail_location_offering_records_reject_unknown_or_inactive_product_for_customer_promise`
  - Raw POS/vendor rows must promote into active semantic offerings before recommendation.
- `retail_inventory_snapshot_records_expire_stale_customer_promises`
  - Stale inventory cannot be used for approved checkout prompts.

Workflow/agent tests:

- `retail_recommendation_agent_outputs_review_packet_not_customer_send_action`
  - Agent contract forbids direct message/send actions.
- `retail_recommendation_agent_can_create_internal_staff_task_for_review`
  - Agent may route review but not execute the upsell.
- `retail_copy_safety_agent_escalates_veterinary_claims_to_manager_review`
  - Copy safety contract protects customer/member-facing boundaries.
- `retail_checkout_assistant_creates_sale_line_draft_only_after_customer_acceptance`
  - Checkout helper respects customer disposition and POS approval state.

Existing tests to preserve and extend:

- `retail_contract_encodes_product_pos_inventory_recommendation_and_reorder_rules`
- `core_service_contract_groups_all_petsuites_lines_without_raw_field_flags`
- Storage codec tests for retail contracts and `RetailPartnerProduct` service-offering variants.

## Integration notes for later serialized Rust code cards

### Files likely touched

- `domain/src/operations.rs`
  - Either extend the existing `pub mod retail` or split into `domain/src/operations/retail.rs` if module size justifies it.
  - Add semantic retail types, policies, service contracts, and domain-local errors.
  - Consider retail-owned re-exports for current `operations::RetailPartner` and `operations::RetailProductCategory`.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add domain-quality tests for upsell context, policy gates, copy safety, inventory availability, and sale-line draft boundaries.
- `domain/tests/domain_quality_patterns.rs`
  - Add architecture tests if this file already carries workflow/agent boundary assertions.
- `storage/tests/core_service_contract_storage.rs`
  - Extend retail JSON/record roundtrip tests if candidates/offering records are persisted.
- `storage/tests/operations_storage_contracts.rs`
  - Extend codec coverage for future retail-specific workflow/action/task variants.
- Future storage module files for `retail_location_offerings`, `retail_upsell_candidates`, `retail_upsell_audit_events`, and inventory snapshots if persistence is implemented.

### Migration and refactor risks

- Current `operations::retail::Product` only contains `Sku` and parent-level category. Personalized upsells need product family, partner, display name, active offering, and usage mode before recommendation policy can be truthful.
- Current `operations::retail::UnitCount` is positive-only, making zero on-hand impossible. Upsell and inventory-promise logic need zero-capable on-hand/available counts while keeping thresholds/order quantities positive.
- Current `operations::retail::RecommendationRule` is coarse. Avoid adding many ad hoc variants; introduce `RecommendationReason`, `UpsellOpportunityContext`, and `UpsellPolicy` instead.
- Current `OperationsAction::DraftCustomerMessage` is generic. Retail customer messages need a review packet/candidate boundary so agents cannot accidentally produce send-ready customer actions.
- Existing `StaffTaskKind::CustomerFollowUp` can bridge review routing, but long-term retail task variants should carry typed SKU/customer/pet/reservation context.
- Money/payment/POS integration must not be pulled into retail policy. Retail should produce `SaleLineDraft` and sale-line decisions; payment adapters execute only after approval.
- Care-profile facts must remain owned by `care`; retail consumes summarized evidence and review flags rather than duplicating medical/diet semantics.

### Dependencies on other implications and service lines

- Inventory/reorder implication: `InventoryPosition`, zero-capable stock counts, stale snapshot behavior, and reorder/substitution review feed personalized upsell safety.
- POS/checkout implication: `SaleLineDraft`, comp/discount approval, reservation-checkout integration, and payment boundaries determine how accepted upsells become sale drafts.
- Partner-product catalog implication: `PartnerProduct`, `ProductFamily`, `LocationOffering`, active/inactive status, and SKU data quality are prerequisites for reliable candidates.
- Boarding and daycare: reservation/check-in/check-out context can trigger diet, supplement, and stress-support candidates.
- Grooming: service history can trigger coat/skin product candidates, but copy safety must avoid unsupported medical claims.
- Training: trainer notes can trigger staff-reviewed anxiety/stress support candidates, but retail must avoid diagnosing behavior.
- Customer/contact preferences: personalized upsells depend on opt-out/channel preferences before customer-facing drafts leave staff review.

## Acceptance summary

A later implementation satisfies this implication when `operations::retail` can model personalized upsells as typed, review-gated, inventory-aware candidates with separate customer-safe copy, audit trail, and sale-line draft boundaries. The code should make it impossible to confuse internal recommendation, staff-approved customer draft, customer acceptance, POS sale-line draft, and payment execution.
