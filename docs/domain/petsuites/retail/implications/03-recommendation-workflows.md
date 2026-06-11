# Retail / partner products implication 03: Recommendation workflows

## Purpose

This implication turns the retail service-map concept of `Recommendation workflows` into an implementation contract for later Rust/domain cards. It is modeling-only: no customer message, POS sale, feeding-plan change, supplement instruction, or vendor/customer side effect should occur from this workflow without the review gates named below.

Safe extensibility assumption: a retail recommendation is an internal, evidence-backed candidate that may become a staff task, manager review packet, customer-message draft, reservation-checkout prompt, or POS sale draft only after deterministic policy review. The model must support Virbac CalmCare, Purina Pro Plan Veterinary Supplements, Purina EN boarding diet, and future partner products without encoding veterinary diagnosis or raw partner/SKU strings in policy logic.

## Operational story

### Trigger

A recommendation workflow starts when one of these typed events or scheduled scans finds product-relevant context:

- A boarding, daycare, training, grooming, or checkout workflow produces a `workflow::WorkflowEvent` with a customer, pet, reservation, or external POS subject.
- A `operations::PetCareWatch` or care-profile snapshot indicates a staff-observed anxiety/stress flag, feeding exception, special diet instruction, allergy, medication, condition, or staff review requirement.
- A reservation lifecycle event reaches pre-arrival, in-stay review, check-out prep, or post-stay follow-up.
- A prior purchase, in-house use, or abandoned checkout suggests replenishment or continuation.
- A staff member or manager explicitly asks for product support options for a customer/pet/reservation.
- An agent scan produces a candidate from approved internal data and asks deterministic retail policy to classify it.

The trigger creates or updates a `operations::retail::RecommendationCase`; it does not directly send a message, attach a charge, alter feeding instructions, or promise stock.

### Actors

- Front desk staff: reviews customer-facing copy, discusses optional products, and may attach approved sale drafts to checkout.
- Kennel technician / care staff: supplies factual care observations and confirms in-house diet use; does not authorize supplement claims.
- Trainer / groomer: supplies service-specific observations that can support a candidate, such as anxiety/stress context or coat-care context.
- Manager / lead staff: approves high-risk recommendations, comps/discounts, substitutions, incident/complaint follow-up, vendor/order escalation, and any recommendation that could be read as medical advice.
- Customer: receives only staff-approved customer-facing recommendations or sale prompts.
- AI agent: drafts candidates, rationale, and staff review packets within `DraftOnly` or `HumanReviewRequired` automation levels.
- Retail/POS/vendor adapters: provide catalog, price, inventory, purchase-history, and availability facts after boundary translation into semantic retail types.

### Inputs

- Product facts: `operations::retail::PartnerProduct`, `Sku`, `ProductName`, `ProductFamily`, `Category`, `UsageMode`, approved display claims, partner/vendor identifiers, location offering status.
- Location facts: `entities::LocationId`, active/inactive availability, price/taxability, POS policy, inventory policy, recommendation policy, customer-message template version.
- Customer and pet facts: `entities::CustomerId`, `entities::PetId`, communication preferences, opt-outs, purchase history, species/age/size if relevant.
- Reservation facts: `entities::ReservationId`, service kind, stay dates, check-in/check-out state, requested add-ons, in-house diet use, checkout context.
- Care facts owned by `care` / `entities::CareProfile`: feeding instructions, allergies, medications, conditions, veterinarian contact, review requirements. Retail consumes these as policy inputs only.
- Service observations: `operations::PetCareWatchReason`, temperament/training/daycare/grooming notes, staff-entered observation references.
- Inventory and sale facts: `InventoryPosition`, `InventoryAvailability`, reserved units, available-for-sale units, price estimate, prior sale line source.
- Workflow context: `workflow::PolicyContext`, `policy::AutomationLevel`, `policy::ReviewGate`, actor, event id, audit correlation id.

### Decisions

The truthful owner of the decision is `operations::retail::RecommendationPolicy`. It decides:

- whether a product is eligible for the location, customer, pet, reservation, and audience;
- whether the rationale is allowed, needs staff review, needs manager review, or is forbidden;
- whether stock state permits an internal prompt, customer-facing draft, waitlist/substitution review, or no action;
- whether care-sensitive evidence requires staff/manager review before any customer-facing content;
- whether a sale draft may be built by `CheckoutService` or must remain a recommendation only;
- whether the recommendation can be merged with an existing open case to avoid duplicate staff tasks.

Policy should return a semantic decision enum such as `RecommendationDecision`, not booleans like `requires_review` or raw status strings.

### Outputs

Allowed outputs are internal and draft-shaped until approval:

- `operations::retail::RecommendationCase` with state, reason, source context, candidate products, evidence, and review requirements.
- `operations::retail::RecommendationCandidate` values for one or more products.
- `operations::StaffTask` or `workflow::RecommendedAction::InternalTask` for staff review.
- `workflow::RecommendedAction::RequestHumanReview(ReviewGate)` when policy requires approval.
- `operations::retail::UpsellDraft` or `workflow::RecommendedAction::DraftMessage` only as a staff-review draft.
- `operations::retail::SaleLineDraft` only after POS policy allows a draft; payment capture remains outside retail.
- `entities::AuditEvent` / workflow audit trail entries for every policy decision, draft, approval, rejection, and customer-facing send.

### Success state

A successful recommendation workflow ends in one of these explicit states:

- `NoRecommendation`: policy found no safe/product-relevant candidate and recorded the reason.
- `InternalPromptCreated`: a staff-facing candidate or task was created with typed product, context, and rationale.
- `DraftPendingReview`: a customer-message or checkout prompt draft exists but is not customer-visible.
- `ApprovedForStaffPresentation`: staff/manager approved the recommendation for discussion or message send.
- `AttachedToCheckoutDraft`: an approved sale-line draft is ready for checkout but not paid/captured.
- `DeclinedOrDismissed`: staff/customer declined and the case carries an audit reason.
- `CompletedWithSaleReference`: an external POS/payment boundary reports a completed sale; retail records a reference, not payment truth.

### Failure and exception states

- `ProductUnavailable`: SKU inactive, not stocked at location, out of stock, backordered, discontinued, or category mismatch.
- `CareReviewRequired`: allergies, medications, medical conditions, feeding exceptions, or ambiguous notes require staff/manager review.
- `UnsafeClaimRejected`: rationale or customer draft implies diagnosis, treatment, cure, prevention, or medical advice.
- `CustomerContactBlocked`: customer opt-out/preference or missing consent blocks message draft/send.
- `DuplicateCandidateMerged`: open case already exists for same customer/pet/product/context and should be updated instead of duplicated.
- `InventoryUncertain`: provider/POS/vendor data is stale, missing, or shape-mismatched.
- `PriceOrTaxUnavailable`: sale draft cannot be created because money/POS facts are absent.
- `PolicyDenied`: deterministic policy forbids the recommendation and records a typed denial reason.
- `NeedsHumanReview`: policy cannot decide safely from available facts.
- `BoundarySyncFailed`: POS, portal, vendor, or messaging adapter failed after domain policy approved a draft; domain state remains safely unexecuted.

## Domain types to add or refine

### Product and offering types

- `operations::retail::Partner`
  - Variants: `Virbac`, `Purina`, later `Other { name: partner::Name }` only when partner expansion needs runtime names.
  - Invariant: policies branch on semantic partner/product family, not raw strings.
- `operations::retail::ProductFamily`
  - Variants: `VirbacCalmCare`, `PurinaProPlanVeterinarySupplements`, `PurinaEnBoardingDiet`, future approved families.
  - Invariant: product-family behavior is explicit and testable.
- `operations::retail::Category`
  - Refines or re-exports current `RetailProductCategory` as retail-owned vocabulary.
  - Initial variants: `Supplement`, `InHouseDiet`, `PersonalizedUpsell`.
- `operations::retail::PartnerProduct`
  - Fields: `sku`, `name`, `partner`, `family`, `category`, `usage_mode`, optional `vendor_catalog_id`, approved internal/customer claim policy.
  - Invariant: SKU/name/partner/family/category are present before recommendation policy can evaluate.
- `operations::retail::LocationOffering`
  - Fields: `location_id`, product, active status, `PointOfSalePolicy`, `InventoryPolicy`, `RecommendationPolicyRef` or inline policy, price/taxability reference.
  - Invariant: a recommendation candidate must reference a location offering, not only a catalog item.

### Recommendation case types

- `operations::retail::RecommendationCaseId`
  - Stable case identity for dedupe, review, audit, and persistence.
- `operations::retail::RecommendationCase`
  - Aggregate root for workflow state.
  - Fields: case id, location, customer, pet, optional reservation, source trigger, candidates, status, review trail, audit references.
  - Invariants: at least one semantic subject (`customer_id` and/or `pet_id`) and exactly one source trigger; cannot transition to customer-facing states without approval.
- `operations::retail::RecommendationCandidate`
  - Fields: offering/product, reason, source context, inventory snapshot, rationale, confidence, decision, review gate, optional message/checkout draft refs.
  - Invariants: candidate product must be active at the location; care-sensitive candidates cannot be `ApprovedForCustomerDraft` without review.
- `operations::retail::RecommendationReason`
  - Suggested variants: `AnxietyOrStressSupport`, `BoardingDietContinuity`, `StaffRecommendedCareSupport`, `CheckoutContinuation`, `PriorPurchaseReplenishment`, `ManagerInitiated`, `ServiceAddOnCompanion`.
  - Invariant: reason is structured; free-text rationale supplements but does not replace it.
- `operations::retail::RecommendationSource`
  - Variants: `WorkflowEvent(workflow::WorkflowEventId)`, `PetCareWatch { reason }`, `ReservationCheckout { reservation_id }`, `PriorPurchase { sale_ref }`, `StaffEntered { staff_id }`, `ManagerInitiated { manager_id }`, `AgentDraft { workflow: agent::Name }`.
  - Invariant: every candidate can be traced to a source.
- `operations::retail::RecommendationRationale`
  - Trimmed, non-empty, bounded internal rationale.
  - Invariant: internal rationale may cite evidence but must be redacted/summarized before customer copy.
- `operations::retail::CustomerSafeRationale`
  - Trimmed, non-empty, bounded customer-facing rationale.
  - Invariant: rejects or routes diagnosis/treatment/cure/prevention wording to manager review.
- `operations::retail::RecommendationConfidence`
  - Prefer enum: `ObservedFit`, `PolicyMatched`, `WeakSignal`, `ManagerEntered`; avoid false precision decimals unless calibrated.
- `operations::retail::RecommendationDecision`
  - Variants: `NoAction { reason }`, `CreateInternalPrompt`, `StaffReviewRequired { gate }`, `ManagerReviewRequired { reason }`, `ApprovedForDraft { audience }`, `Forbidden { reason }`.
- `operations::retail::RecommendationStatus`
  - Variants: `Candidate`, `MergedDuplicate`, `StaffReviewOpen`, `ManagerReviewOpen`, `ApprovedForStaffPresentation`, `CustomerDraftReady`, `AttachedToCheckoutDraft`, `Rejected`, `Declined`, `Expired`, `Completed`.
- `operations::retail::RecommendationDenialReason`
  - Variants: `ProductUnavailable`, `OutOfStock`, `CareProfileRequiresReview`, `UnsafeMedicalClaim`, `CustomerOptOut`, `NoRelevantEvidence`, `PriceUnavailable`, `PolicyMismatch`, `DuplicateSuppressed`.

### Review, approval, and audit types

- `operations::retail::RecommendationReviewGate`
  - Retail-specific facade over `policy::ReviewGate`, with variants such as `StaffProductReview`, `ManagerApproval`, `CustomerMessageApproval`, `CareSensitiveReview`, `PosAttachmentApproval`.
- `operations::retail::ReviewOutcome`
  - `Approved`, `Rejected { reason }`, `NeedsMoreInformation { reason }`, `EditedAndApproved { edit_summary }`, `Escalated { gate }`.
- `operations::retail::ApprovalTrail`
  - Ordered review events with actor, timestamp, gate, outcome, and audit event id.
- `operations::retail::RecommendationAuditRef`
  - Links case/candidate transitions to `entities::AuditEvent` and/or `workflow::WorkflowEventId`.

### Draft, sale, and inventory-support types

- `operations::retail::UpsellDraft`
  - Staff-review customer copy or checkout prompt, not a sent message.
  - Invariants: references candidate/case; carries audience and review state; uses `CustomerSafeRationale` for customer copy.
- `operations::retail::SaleLineDraft`
  - Product/SKU, quantity, source, price reference, approval status.
  - Invariant: cannot represent captured payment; payment/POS adapters own execution.
- `operations::retail::SaleSource`
  - `StandaloneRetail`, `ReservationCheckout { reservation_id }`, `BoardingInHouseDiet { reservation_id }`, `StaffEntered`, `AgentDrafted`.
- `operations::retail::InventoryPosition`
  - Should use zero-capable `OnHandUnits` and `ReservedUnits`, plus positive `ReorderThreshold`.
  - Invariant: reserved units cannot exceed on-hand; available-for-sale is derived.
- `operations::retail::InventoryAvailability`
  - `InStock`, `LowStock`, `OutOfStock`, `Backordered`, `Discontinued`, `Unknown`.

## Relationship map

### Entities and value objects

- `entities::CustomerId`, `entities::PetId`, `entities::ReservationId`, `entities::LocationId`, `entities::StaffId`, and manager identity types identify subjects and actors.
- `operations::retail::RecommendationCase` is the aggregate root for the workflow.
- `operations::retail::RecommendationCandidate` is a child entity/value object inside a case; each candidate references one `LocationOffering` and one `RecommendationReason`.
- `operations::retail::PartnerProduct` and `LocationOffering` provide product/availability/POS context.
- `care` and `temperament` provide fact inputs; retail stores only typed evidence references or summarized rationale, not ownership of medical/care facts.
- `money` and `payment` provide price/payment semantics; retail stores only sale-line draft references and source state.

### Policies

- `operations::retail::RecommendationPolicy` owns recommendation eligibility, audience, review gate, unsafe-claim, stock-state, and duplicate-case decisions.
- `operations::retail::PointOfSalePolicy` owns whether an approved recommendation may become a sale-line draft, reservation-checkout prompt, standalone-sale prompt, or manager-only comp request.
- `operations::retail::InventoryPolicy` owns stock availability classification used by recommendation policy.
- `policy::AutomationLevel` and `policy::ReviewGate` remain cross-cutting policy vocabulary consumed by workflow/agent surfaces.

### Repositories and stores

- `operations::retail::RecommendationRepository` stores cases, candidate state, review outcomes, draft refs, and audit links.
- `operations::retail::CatalogRepository` or broader `Repository` loads products and location offerings by `Sku`, location, and product family.
- `operations::retail::InventoryRepository` loads inventory snapshots and records movement references, but recommendation workflow should not mutate stock directly.
- `operations::retail::PurchaseHistoryRepository` can be a read model over POS/customer purchases; it should return semantic purchase summaries, not raw receipts.
- Storage adapters translate Gingr/POS/vendor rows into semantic records before repository methods return domain values.

### Workflow events

- Existing `workflow::WorkflowEventType` may need retail variants such as `RetailRecommendationCandidateDetected`, `RetailRecommendationReviewNeeded`, `RetailRecommendationApproved`, `RetailRecommendationRejected`, `RetailCheckoutPromptRequested`, and `RetailRecommendationCompleted`.
- `workflow::WorkflowResult<operations::retail::RecommendationCase>` should carry safe structured output, recommended actions, risk flags, verification notes, and human-review reason.
- `workflow::RecommendedAction` may remain generic for internal tasks/draft messages, but typed retail payloads should live in structured output or a new typed action variant rather than `{ sku: String }` blobs.

### Staff tasks

- Extend `operations::StaffTaskKind` when code needs task specificity:
  - `RetailRecommendationReview { customer_id, pet_id, sku }`
  - `RetailCustomerDraftReview { customer_id, pet_id, sku }`
  - `RetailCheckoutPromptReview { reservation_id, sku }`
  - `RetailSubstitutionReview { reservation_id, requested_sku, substitute_sku }`
- Review tasks should use `StaffTaskStatus::NeedsManagerReview` or assignment to `StaffRole::Manager` for manager-gated cases.

### Agent specs and tools

- `retail-recommendation-drafter`
  - Reads: customer/pet/reservation/care summaries, catalog/location offerings, inventory, approved templates.
  - Writes: `RecommendationCase`/candidate structured output, staff-review task drafts, customer-message drafts only under review.
  - Forbidden: send customer message, attach charge, diagnose, alter care plan, promise unavailable inventory.
- `retail-checkout-upsell-assistant`
  - Reads: reservation checkout context, approved recommendation cases, POS price/stock snapshot.
  - Writes: checkout prompt draft or staff task.
  - Forbidden: create/capture payment, force add-on, apply discount/comp.
- `retail-care-sensitive-review-router`
  - Reads: candidate and care-sensitive flags.
  - Writes: review gate and manager/staff task.
  - Forbidden: expose sensitive care details in customer-facing copy.

Tool contracts should be boundary-specific and semantic, for example `tools::pos::PriceLookupRequest { sku, location_id }` and `tools::messaging::DraftMessageRequest` with `MessageReviewPolicy::ManagerApprovalRequired` or customer-message approval.

## Interaction contract

Rust-like pseudo-signatures below name desired ownership and call-site shape. Exact generics/lifetimes can be decided during implementation.

```rust
pub mod operations::retail {
    pub struct RecommendationCase { /* aggregate */ }
    pub struct RecommendationCandidate { /* child candidate */ }
    pub struct RecommendationPolicy { /* deterministic policy owner */ }
    pub struct RecommendationService<R> { repo: R, policy: RecommendationPolicy }

    pub trait RecommendationRepository {
        fn load_case(&self, id: RecommendationCaseId) -> Result<Option<RecommendationCase>>;
        fn find_open_case(
            &self,
            key: RecommendationDedupeKey,
        ) -> Result<Option<RecommendationCase>>;
        fn save_case(&mut self, case: &RecommendationCase) -> Result<()>;
        fn append_review(&mut self, id: RecommendationCaseId, outcome: ReviewOutcome) -> Result<()>;
    }

    pub trait CatalogRepository {
        fn product(&self, sku: &Sku) -> Result<PartnerProduct>;
        fn location_offering(&self, location: entities::LocationId, sku: &Sku)
            -> Result<LocationOffering>;
        fn active_offerings_for_family(
            &self,
            location: entities::LocationId,
            family: ProductFamily,
        ) -> Result<Vec<LocationOffering>>;
    }

    pub trait InventoryRepository {
        fn position(&self, location: entities::LocationId, sku: &Sku)
            -> Result<InventoryPosition>;
    }
}
```

### Recommendation service

```rust
impl<R> operations::retail::RecommendationService<R>
where
    R: RecommendationRepository + CatalogRepository + InventoryRepository,
{
    pub fn evaluate_trigger(
        &mut self,
        trigger: RecommendationTrigger,
        context: RecommendationContext,
    ) -> Result<RecommendationCase>;

    pub fn draft_customer_copy(
        &mut self,
        case_id: RecommendationCaseId,
        candidate_id: RecommendationCandidateId,
        template: CustomerTemplateRef,
    ) -> Result<UpsellDraft>;

    pub fn approve_review(
        &mut self,
        case_id: RecommendationCaseId,
        outcome: ReviewOutcome,
        actor: entities::ActorRef,
    ) -> Result<RecommendationCase>;
}
```

Behavior ownership:

- `RecommendationService::evaluate_trigger` coordinates repository reads, duplicate-case merging, policy evaluation, and case persistence.
- `RecommendationCase` owns legal state transitions such as `open_staff_review`, `attach_draft`, `approve_for_customer_presentation`, `reject`, and `expire`.
- `RecommendationCandidate` owns candidate-local facts and derived flags such as `is_care_sensitive`, `is_customer_visible`, or `requires_stock_confirmation` when those derive only from candidate state.
- `RecommendationPolicy` owns policy decisions and unsafe-claim validation; it should not persist or send anything.
- `PointOfSalePolicy` owns conversion from approved candidate to sale-line draft.

### Policy contracts

```rust
impl operations::retail::RecommendationPolicy {
    pub fn decide(
        &self,
        offering: &LocationOffering,
        context: &RecommendationContext,
        evidence: &RecommendationEvidence,
        audience: RecommendationAudience,
    ) -> RecommendationDecision;

    pub fn validate_customer_rationale(
        &self,
        rationale: &CustomerSafeRationale,
        evidence: &RecommendationEvidence,
    ) -> CustomerRationaleDecision;
}

impl operations::retail::PointOfSalePolicy {
    pub fn draft_sale_line(
        &self,
        candidate: &RecommendationCandidate,
        quantity: OrderQuantity,
        price: money::Money,
        source: SaleSource,
        actor: entities::ActorRef,
    ) -> Result<SaleLineDraft>;
}
```

`RecommendationPolicy::decide` must not create tasks or write storage. It returns typed decisions such as `StaffReviewRequired`, `ManagerReviewRequired`, `Forbidden`, or `ApprovedForDraft`; services/workflows translate those decisions into tasks/results.

### Workflow and agent contract

```rust
pub struct RetailRecommendationInput {
    pub trigger: operations::retail::RecommendationTrigger,
    pub context: operations::retail::RecommendationContext,
}

pub type RetailRecommendationOutput = operations::retail::RecommendationCase;

impl agents::WorkflowAgent<RetailRecommendationInput, RetailRecommendationOutput>
    for RetailRecommendationDrafter
{
    fn spec(&self) -> agents::AgentSpec;
    fn build_prompt_packet(
        &self,
        event: &workflow::WorkflowEvent,
        input: RetailRecommendationInput,
    ) -> agents::AgentPromptPacket<RetailRecommendationInput>;
    fn validate_output(
        &self,
        output: workflow::WorkflowResult<RetailRecommendationOutput>,
    ) -> workflow::WorkflowResult<RetailRecommendationOutput>;
}
```

Validation obligations:

- output schema must include case id, subjects, SKU/family, reason, rationale, evidence refs, decision, review gate, and draft/customer-facing boundary flags;
- output cannot contain a sent-message marker, captured payment, care-plan mutation, or vendor-order submission;
- customer-facing text must either be absent or attached to a required `CustomerMessageApproval` / manager review gate.

## Review and approval contract

### Automation level

Default automation is `policy::AutomationLevel::DraftOnly` for all recommendation workflows. Limited `CreateInternalTask` automation is acceptable for staff-review tasks when policy returns `CreateInternalPrompt` or `StaffReviewRequired`. No recommendation workflow should run at an autonomous member-facing or payment-executing level.

### Review gates

- Staff review required:
  - every customer-facing product recommendation or checkout prompt;
  - every recommendation based on behavior/anxiety/stress, diet, feeding instructions, medications, allergies, medical conditions, ambiguous care notes, or staff observations;
  - every substitution or out-of-stock alternative;
  - every agent-drafted customer message.
- Manager review required:
  - any medical-adjacent claim or phrasing that could imply diagnosis, treatment, cure, prevention, or veterinary advice;
  - incident, complaint, safety, dissatisfaction, recovery, refund/comp/discount, or legal-sensitive context;
  - product comps/discounts/refunds/charge reversals;
  - policy/template/catalog changes;
  - attaching a paid item to checkout when POS policy requires manager authorization.
- Customer-message approval required:
  - before any draft leaves internal systems by email/SMS/portal/app/printed checkout summary.
- POS/payment approval required:
  - before attaching a retail line item to a reservation checkout, standalone sale, payment authorization, capture, refund, or comp.

### Audit trail

Every case should append audit entries for:

- trigger observed and input snapshot references;
- policy decision with typed decision and denial/review reason;
- candidate creation, merge, update, expiry, rejection, and approval;
- customer-facing draft creation and edits;
- staff/manager review outcome with actor and timestamp;
- checkout sale-line draft creation;
- external POS/payment/messaging references after boundary execution.

Audit metadata should use typed keys/values such as `entities::AuditMetadataKey` and `AuditMetadataValue`; avoid storing raw private care notes in durable audit metadata. Store evidence references or redacted summaries instead.

### Customer/member-facing boundaries

Automation may draft but must not:

- send product/supplement/diet recommendations to customers;
- state or imply a product treats, cures, diagnoses, or prevents a condition;
- change feeding instructions, medication instructions, or care plans;
- attach charges, capture payment, refund, discount, comp, or submit POS sale;
- create vendor orders;
- promise stock or hide out-of-stock facts.

## Test contracts

Domain tests:

- `retail_recommendation_case_requires_customer_or_pet_subject`
  - A case cannot be built with no customer, pet, or reservation subject.
- `retail_recommendation_candidate_requires_active_location_offering`
  - A candidate cannot be created from a catalog product that is inactive/unavailable at the location.
- `retail_recommendation_reason_is_semantic_not_raw_rationale_text`
  - Recommendation reason remains an enum while rationale is supplemental text.
- `retail_recommendation_policy_routes_calmcare_anxiety_support_to_staff_review`
  - Virbac CalmCare recommendation from anxiety/stress evidence yields staff review and no customer send.
- `retail_recommendation_policy_routes_purina_en_diet_context_to_care_sensitive_review`
  - Purina EN diet continuity based on feeding/care context requires staff/manager review as configured.
- `retail_recommendation_policy_forbids_customer_copy_with_medical_claims`
  - Customer-safe rationale rejects diagnosis/treat/cure/prevent wording or escalates to manager review.
- `retail_recommendation_policy_blocks_customer_prompt_when_inventory_is_out_of_stock`
  - Out-of-stock offering cannot become customer-facing draft or checkout prompt.
- `retail_recommendation_policy_merges_duplicate_open_cases_for_same_context`
  - Duplicate trigger updates an existing open case instead of creating duplicate staff tasks.
- `retail_recommendation_case_transitions_require_review_before_customer_visibility`
  - State transitions prevent `CustomerDraftReady` / `ApprovedForStaffPresentation` without review outcome.
- `retail_recommendation_rationale_redacts_care_sensitive_details_for_customer_copy`
  - Internal evidence can exist without leaking medication/allergy/condition details into customer-facing text.

Workflow and agent tests:

- `retail_recommendation_agent_outputs_structured_candidate_not_free_form_sku_blob`
  - Agent output validates into `RecommendationCase`/candidate with typed SKU, reason, decision, and review gate.
- `retail_recommendation_agent_can_create_internal_review_task_but_not_send_message`
  - Allowed actions include internal task/draft only; sent-message markers fail validation.
- `retail_recommendation_workflow_result_carries_human_review_reason_for_care_sensitive_cases`
  - `WorkflowResult` uses `NeedsHumanReview` and semantic review reason for sensitive inputs.
- `retail_checkout_upsell_agent_cannot_capture_payment_or_apply_comp`
  - Agent/tool output rejects payment capture, discount, comp, or sale completion.

Storage/boundary tests:

- `retail_recommendation_records_roundtrip_review_gate_reason_and_status`
  - Persistence preserves case state, candidate reason, decision, and review gate.
- `retail_recommendation_storage_rejects_unknown_status_or_boolean_review_shape`
  - Boundary codecs reject raw status strings/booleans that erase review semantics.
- `retail_recommendation_audit_refs_roundtrip_without_raw_sensitive_care_notes`
  - Stored audit references preserve traceability without leaking sensitive notes.
- `retail_pos_purchase_history_promotes_raw_receipts_to_semantic_purchase_summary`
  - Purchase history repository returns typed product/customer/location facts, not raw receipt maps.

Integration tests:

- `retail_calmcare_recommendation_from_boarding_stress_watch_creates_staff_review_task`
- `retail_purina_en_recommendation_from_boarding_diet_continuity_creates_manager_review_when_care_profile_is_sensitive`
- `retail_prior_purchase_replenishment_creates_customer_draft_only_after_inventory_and_contact_policy_pass`
- `retail_recommendation_case_approved_for_checkout_builds_sale_line_draft_without_payment_capture`

## Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Add or split `operations::retail` types for recommendation cases, candidates, reasons, decisions, statuses, review outcomes, inventory availability, sale drafts, policies, and repository traits.
  - Consider moving current parent-level `RetailPartner` / `RetailProductCategory` behind retail-owned paths or re-exporting through `operations::retail` with one canonical public surface.
  - Extend `StaffTaskKind`, `OperationsAction`, `RevenueOpportunityKind`, and `DailyBriefSection` only when a typed retail surface is needed by workflow outputs.
- `domain/src/workflow.rs`
  - Add retail workflow event variants and/or typed retail recommended-action targets if generic actions cannot preserve semantics.
  - Ensure `PolicyContext` supports retail recommendation review gates and allowed action names.
- `domain/src/agents.rs`
  - Add baseline specs for `retail-recommendation-drafter`, `retail-checkout-upsell-assistant`, and possibly `retail-care-sensitive-review-router`.
- `domain/src/policy.rs`
  - Add retail-specific review gates only if existing `ManagerApproval` and `CustomerMessageApproval` are not precise enough.
- `domain/src/tools.rs` / `domain/src/tools/*`
  - Add semantic POS/catalog/inventory/messaging requests if recommendation workflow needs tool-boundary contracts.
- `domain/tests/domain_quality_patterns.rs`
  - Add semantic domain and workflow tests named above.
- Storage crate/tests, likely `storage/tests/core_service_contract_storage.rs` and `storage/tests/operations_storage_contracts.rs`
  - Add codecs for retail recommendation cases, location offerings, inventory snapshots, and review/audit records once persisted.

### Migration and refactor risks

- Current `operations::retail::UnitCount` is positive-only, while inventory recommendations need zero on-hand. Introduce zero-capable `OnHandUnits`/`ReservedUnits` rather than weakening positive threshold semantics.
- Current `operations::retail::Product` has SKU and category but not partner/family/name. Recommendation policy needs product identity and approved claim boundaries; do not keep those as raw strings in adapters.
- Current `RecommendationRule` is coarse and contract-level. Do not keep adding variants as workflow state; introduce `RecommendationCase`, `RecommendationCandidate`, `RecommendationReason`, and `RecommendationDecision`.
- Current `OperationsAction::SuggestRevenueFollowUp` carries only `RevenueOpportunityKind`; retail recommendations need typed SKU/product/review context, so use structured workflow output or new retail-specific action types.
- Staff tasks currently lack retail-specific variants. Adding variants is safer than overloading `CustomerFollowUp` with opaque recommendation text.
- Customer-safe rationale must not accidentally serialize raw `care` details or temperament notes. Prefer evidence refs and redacted summary values.
- Review gates should remain explicit enums, not booleans like `approved`, `requires_manager`, or `customer_visible` without state-transition protection.
- Boundary adapters may have POS/vendor-specific SKU/status names; promote them immediately into `Sku`, `InventoryAvailability`, `ProductFamily`, and semantic denial errors.

### Dependencies on other implications

- Inventory/reorder implication: recommendation policy depends on `InventoryPosition`, `InventoryAvailability`, reorder thresholds, and backorder/vendor status but should not create vendor orders.
- POS/checkout implication: checkout prompts and `SaleLineDraft` depend on money/POS/tax/payment approval contracts.
- Partner catalog/data-quality implication: product family/category/approved claims and SKU mappings must be normalized before recommendation policy runs.
- Boarding/Purina EN diet implication: in-house diet continuity and consumption forecast should feed candidates without changing feeding instructions.
- Customer communication implication: customer-message drafts need contact preferences, opt-outs, template approval, and message review gates.
- Care/medical sensitivity implication: allergies, medications, conditions, and ambiguous care notes remain owned by care modules and only drive retail review decisions.

## Acceptance checklist

- Recommendation workflow is modeled as a review-gated internal case, not direct upsell execution.
- Domain paths preserve `operations::retail` ownership of recommendation vocabulary and policy behavior.
- Product, reason, decision, review gate, inventory state, and sale source are typed domain concepts.
- Staff/customer/manager boundaries are explicit and auditable.
- Agent contracts can draft and classify but cannot send, sell, diagnose, alter care, or promise stock.
- Tests state semantic truths and protect the review/customer-facing boundary.
