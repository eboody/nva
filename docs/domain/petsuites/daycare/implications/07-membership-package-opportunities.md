# Daycare implication 07: Membership/package opportunities

Purpose: model daycare membership and package opportunities as approval-gated operational recommendations. The domain should recognize attendance and payment patterns, suggest a truthful next-best commercial conversation, and create staff-review artifacts without enrolling a customer, changing price, consuming visits, charging payment, or sending a promise autonomously.

Scope assumptions:

- A daycare package is a prepaid or preauthorized visit bundle tied to a customer, pet, location, service/care-mode eligibility, expiration/term, and consumption rules.
- A daycare membership is a recurring commercial relationship with location-specific terms, not merely a boolean flag on a customer. It may include recurring visits, discounts, priority booking, or billing cadence, but those details stay policy-driven.
- Unknown pricing, billing cadence, tax, refund, discount, or local offer details route to manager review. This model intentionally records opportunity and review state without hard-coding price math.
- Package/membership opportunity detection is safe as read/draft/recommend automation only. Staff/manager approval is required before offer language, enrollment, payment capture, discounts, credits, or customer-facing commitment.

## 1. Operational story

Trigger:

- A daycare attendance pattern crosses a configured opportunity threshold, such as repeated drop-in/pay-per-visit attendance, recurring weekly attendance, frequent half-day/full-day use, upcoming recurrence requests without a package, or package visits approaching exhaustion/expiration.
- A front-desk or manager daily brief asks for daycare revenue opportunities for a location/day/week.
- A customer asks about recurring daycare, a package, membership, or "best value" for frequent daycare.
- Checkout detects that a customer is repeatedly paying per visit while a configured package/membership might fit the same service/care mode.

Actors:

- Customer/member: receives only approved offer language and never sees internal scoring or sensitive pet-care details.
- Pet: the opportunity is constrained by the pet's service/care-mode eligibility and attendance history.
- Front desk: reviews recommendations, confirms customer intent, resolves package/eligibility/payment blockers, and may draft follow-up.
- Manager: approves pricing-sensitive, discount, exception, refund/credit, membership-enrollment, or policy-override decisions.
- Daycare staff: supplies attendance/care/behavior facts; does not own commercial approval.
- Agent workflows: `manager-daily-brief`, `lead-conversion`, `booking-triage`, and a later daycare package-opportunity agent can summarize and draft, but cannot sell or modify financial state.
- Payment/money adapters: execute approved payment/enrollment/visit consumption only after a separate payment contract authorizes it.

Inputs:

- `entities::CustomerId`, `entities::PetId`, `entities::LocationId`, and relevant reservation/attendance IDs.
- Daycare service contract: `operations::daycare::Contract`, `PackagePolicy`, `AttendancePolicy`, `StaffPetRatio`, and future location-specific `PackageCatalog`/`MembershipPlan` policy refs.
- Attendance history: dated visits, no-shows/cancellations if modeled, care mode/service variant, package visit consumption, and recurrence intent.
- Eligibility state: current `GroupPlayEligibilityDecision`, care mode, unresolved incident restrictions, vaccination/spay-neuter/temperament/care-note review state.
- Payment/package state: pay-per-visit history, active package/membership, remaining visits, expiration, billing status, prior discounts/credits where available.
- Customer communication preferences and consent boundaries from `customer`/`portal` contexts.
- Location policy: which packages/memberships are available, thresholds for recommendation, staff/manager review gates, and approved offer templates.

Decisions:

1. Is the customer/pet already covered by an active compatible package or membership?
2. Is the pet eligible for the care mode that the package/membership implies, or does the offer need staff review first?
3. Does attendance history satisfy a configured opportunity rule without overstating savings or availability?
4. Is the package/membership available at this location and compatible with species, service variant, care mode, recurrence, expiration, and capacity policy?
5. Is this a low-risk staff-reviewed follow-up, a manager-priced offer, a payment/enrollment action, or a no-op?
6. Which audit trail and review gate must attach before any customer-facing draft or financial action?

Outputs:

- `operations::daycare::PackageOpportunityDecision` with an explicit outcome: no opportunity, existing coverage, draft staff follow-up, manager review required, eligibility review required, payment/enrollment blocked, or policy unavailable.
- `operations::RevenueOpportunity { service: ServiceKind::Daycare, opportunity: RevenueOpportunityKind::DaycarePackageCandidate, ... }` for existing daily-brief surfaces, plus a richer daycare-owned detail object for later implementation.
- Optional `operations::StaffTaskKind::CustomerFollowUp`, `DocumentReview`, `PlaygroupAssessment`, or manager-review task when blockers exist.
- Optional customer-message draft requiring `policy::ReviewGate::CustomerMessageApproval` and, when pricing/enrollment is involved, `policy::ReviewGate::ManagerApproval` or a future money/payment approval gate.
- Audit event describing the recommendation source, evidence snapshot, review gate, reviewer, and final disposition.

Success state:

- The system records a typed recommendation that a staff member can review locally: who, which pet, which care mode/service pattern, which package/membership family, why now, what evidence was considered, and what cannot be promised yet.
- If approved, the staff/manager can send approved language or initiate a separate payment/enrollment workflow. The daycare domain never silently creates a membership, discounts a reservation, captures payment, or marks visits consumed.

Failure/exception states:

- `AlreadyCovered`: active compatible package/membership exists; surface remaining visits/term if safe, not another sales prompt.
- `EligibilityReviewRequired`: package implies group play or a care mode that lacks current approval; create/reuse staff review instead of offering.
- `PolicyUnavailable`: location/service/species/care-mode has no configured package/membership; do not invent offers.
- `InsufficientEvidence`: attendance/payment history is too sparse, stale, or ambiguous; route to staff review or no-op.
- `CapacityOrStaffingRisk`: recurring/member volume may exceed daycare capacity or ratio; manager review before selling priority/recurrence.
- `PaymentOrBillingReviewRequired`: open balance, payment failure, refund/credit ambiguity, tax/discount uncertainty, or package consumption mismatch.
- `SensitiveIncidentOrCareRestriction`: unresolved incident/medical/care note prevents offer until manager/staff review clears safe handling.
- `CustomerContactBlocked`: consent/preference/do-not-contact/portal constraints prevent customer-facing outreach.

## 2. Domain types to add/refine

Recommended semantic paths:

```rust
operations::daycare::PackageOpportunityService
operations::daycare::PackageOpportunityPolicy
operations::daycare::PackageOpportunityDecision
operations::daycare::PackageOpportunityEvidence
operations::daycare::PackageOpportunityScore
operations::daycare::PackageOpportunityReason
operations::daycare::PackageOpportunityBlocker
operations::daycare::PackageOpportunityDisposition
operations::daycare::PackageCatalog
operations::daycare::PackageOffer
operations::daycare::PackageOfferId
operations::daycare::PackageOfferName
operations::daycare::PackageVisitBalance
operations::daycare::PackageExpiration
operations::daycare::PackageConsumptionRule
operations::daycare::MembershipPlan
operations::daycare::MembershipPlanId
operations::daycare::MembershipTerm
operations::daycare::MembershipCadence
operations::daycare::MembershipEligibility
operations::daycare::MembershipEnrollmentIntent
operations::daycare::RecurringAttendanceSignal
operations::daycare::AttendanceFrequencyWindow
operations::daycare::OpportunityReviewPacket
operations::daycare::OpportunityAuditTrail
operations::daycare::OpportunityRepository
operations::daycare::PackageCatalogRepository
operations::daycare::MembershipRepository
```

Types/invariants:

- `PackageOfferId` / `MembershipPlanId`: non-empty provider-neutral IDs; external POS/portal IDs convert at adapter boundaries.
- `PackageOfferName`: trimmed, non-empty display-safe name; still not a customer-facing promise without approved template.
- `PackageVisitBalance`: non-negative visit count; consumption cannot underflow; unknown balance is a review blocker, not zero.
- `PackageVisits`: existing non-zero bundle size remains the primitive for configured packages.
- `PackageExpiration`: `NoExpiration`, `ExpiresOn(date)`, or `ExpiresAfter(MembershipTerm)`; expired packages cannot be recommended as active coverage.
- `MembershipTerm`: explicit start/end or start/cadence model; no zero/negative duration; open-ended terms require policy support.
- `MembershipCadence`: weekly, monthly, annual, or location-defined; never raw text in core policy.
- `AttendanceFrequencyWindow`: start/end date plus counted eligible visits; non-empty window and no future-only evidence.
- `RecurringAttendanceSignal`: captures attendance pattern (`WeeklyRecurring`, `FrequentDropIn`, `PackageDepleting`, `PackageExpiring`, `CustomerAskedAboutRecurringCare`, `NoSignal`) without mixing it with final recommendation.
- `PackageOpportunityEvidence`: customer, pet, location, attendance window, service variant/care mode, eligibility decision, active package/membership state, payment status summary, and source snapshot IDs.
- `PackageOpportunityScore`: bounded ordinal or enum (`Low`, `Moderate`, `Strong`) configured by policy; it is not a hidden numeric sales score in customer-visible output.
- `PackageOpportunityReason`: typed reasons such as `FrequentPayPerVisit`, `RecurringScheduleRequested`, `PackageVisitsLow`, `PackageNearingExpiration`, `CustomerAskedForBestValue`, `MembershipMayReduceFrontDeskFriction`.
- `PackageOpportunityBlocker`: typed blockers such as `AlreadyCovered`, `EligibilityReviewRequired`, `LocationPolicyUnavailable`, `InsufficientAttendanceEvidence`, `PaymentReviewRequired`, `CapacityReviewRequired`, `CustomerContactNotAllowed`.
- `PackageOpportunityDecision`: semantic enum; no `eligible: bool`/`should_offer: bool` pair.

Suggested decision enum:

```rust
pub enum PackageOpportunityDecision {
    NoOpportunity {
        evidence: PackageOpportunityEvidence,
        reasons: Vec<PackageOpportunityReason>,
    },
    ExistingCoverage {
        coverage: ExistingPackageCoverage,
        evidence: PackageOpportunityEvidence,
    },
    RecommendStaffReview {
        offer: PackageOfferCandidate,
        reasons: Vec<PackageOpportunityReason>,
        evidence: PackageOpportunityEvidence,
        review: OpportunityReviewPacket,
    },
    RequiresManagerReview {
        offer: PackageOfferCandidate,
        blockers: Vec<PackageOpportunityBlocker>,
        evidence: PackageOpportunityEvidence,
        review: OpportunityReviewPacket,
    },
    Blocked {
        blockers: Vec<PackageOpportunityBlocker>,
        evidence: PackageOpportunityEvidence,
    },
}
```

Refinements to existing types:

- Refine `operations::daycare::PackagePolicy` from `PayPerVisit | PrepaidPasses | Membership` into a contract that can reference `PackageCatalog`, `MembershipPlan`, consumption rules, expiration rules, and review gates.
- Keep `RevenueOpportunityKind::DaycarePackageCandidate` as the cross-service summary enum, but add a daycare-owned detail payload before logic branches on opportunity details.
- Add `FollowUpReason::DaycarePackageOpportunity` or a daycare-owned customer-follow-up reason that converts into existing task/message surfaces.
- Consider splitting `MembershipEligibility` into commercial eligibility (customer/location/payment/plan fit) and daycare care eligibility (pet/service/care-mode safety). Do not use one flag for both.

## 3. Relationship map between types

Entities and value objects:

- `entities::CustomerId` owns customer identity; `operations::daycare::PackageOpportunityEvidence` references it but does not duplicate customer profile data.
- `entities::PetId` owns pet identity; package opportunity is per customer/pet/care-mode because daycare eligibility and attendance are pet-specific.
- `entities::LocationId` scopes package catalogs, membership plan availability, package expiration rules, local thresholds, and manager review policy.
- `operations::daycare::ServiceVariant` and `CareMode` constrain which package/membership can truthfully apply.
- `operations::daycare::PackageOffer`, `MembershipPlan`, `MembershipTerm`, and `PackageVisitBalance` are daycare/money-adjacent value objects; they do not execute payment.

Policies:

- `PackageOpportunityPolicy` owns recommendation thresholds, review gates, compatible services/care modes, reason/blocker classification, and safe ranking between package vs membership candidates.
- `MembershipEligibility` owns commercial-plan fit, while `GroupPlayEligibilityPolicy`/care-mode eligibility own pet safety. A positive commercial opportunity cannot override safety policy.
- `StaffCoveragePolicy` and capacity policy can block or manager-gate recurring membership offers when package uptake would stress ratios/capacity.
- `payment`/`money` policy owns billing authorization, payment capture, refunds, tax, credits, and discounts.

Repositories/stores:

- `AttendanceRepository` provides semantic attendance history and package consumption facts.
- `PackageCatalogRepository` loads location-specific packages and membership plans, approved templates, terms, expiration, and review gates.
- `MembershipRepository` reads active/past memberships and writes only approved enrollment intents/dispositions when called by an approved workflow.
- `OpportunityRepository` stores recommendations, evidence snapshots, dispositions, suppressions, reviewer decisions, and audit links.
- Payment repositories/stores remain outside daycare and are called only by approved payment workflows.

Workflow events:

- `workflow::WorkflowEventType::BookingTriageNeeded`: customer asks about recurring daycare or package fit.
- `ManagerDailyBriefNeeded` or future `DaycareRevenueReviewNeeded`: daily/weekly opportunity scan.
- Future events: `DaycarePackageOpportunityDetected`, `DaycarePackageOpportunityReviewed`, `MembershipEnrollmentApproved`, `PackageBalanceLow`, `PackageExpirationApproaching`.

Staff tasks:

- `StaffTaskKind::CustomerFollowUp`: staff-reviewed package/membership conversation.
- `StaffTaskKind::DocumentReview` or `PlaygroupAssessment`: blockers that must resolve before a care-mode-specific offer.
- `StaffTaskKind::IncidentFollowUp`: unresolved incident/eligibility blocker.
- Future daycare-specific task kind: `PackageOpportunityReview { customer_id, pet_id, opportunity_id }` if generic customer follow-up becomes too vague.

Agent specs/tools:

- `manager-daily-brief`: reads attendance and catalog summaries; outputs opportunities in manager-facing brief.
- `lead-conversion`: drafts safe follow-up when a lead asks about recurring daycare/package value.
- `booking-triage`: identifies package/membership state as a readiness/payment issue, not a booking confirmation.
- Future `daycare-package-opportunity` spec: allowed tools `attendance-read`, `policy-read`, `package-catalog-read`, `customer-read`, `task-create`, `draft-message`; forbidden actions `enroll membership`, `capture payment`, `apply discount`, `promise savings`, `send message without approval`.

## 4. Interaction contract

Rust-like pseudo-signatures:

```rust
impl operations::daycare::PackageOpportunityPolicy {
    pub fn classify(
        &self,
        evidence: &operations::daycare::PackageOpportunityEvidence,
        catalog: &operations::daycare::PackageCatalog,
    ) -> operations::daycare::PackageOpportunityDecision;
}

impl operations::daycare::PackageOpportunityService {
    pub fn evaluate_for_pet(
        &self,
        customer_id: entities::CustomerId,
        pet_id: entities::PetId,
        location_id: entities::LocationId,
        as_of: chrono::NaiveDate,
    ) -> operations::daycare::Result<operations::daycare::PackageOpportunityDecision>;

    pub fn build_review_packet(
        &self,
        decision: &operations::daycare::PackageOpportunityDecision,
    ) -> operations::daycare::Result<operations::daycare::OpportunityReviewPacket>;

    pub fn record_disposition(
        &self,
        opportunity_id: operations::daycare::OpportunityId,
        disposition: operations::daycare::PackageOpportunityDisposition,
        reviewer: entities::ActorRef,
    ) -> operations::daycare::Result<operations::daycare::OpportunityAuditTrail>;
}

pub trait operations::daycare::PackageCatalogRepository {
    fn load_for_location(
        &self,
        location_id: entities::LocationId,
    ) -> operations::daycare::Result<operations::daycare::PackageCatalog>;
}

pub trait operations::daycare::OpportunityRepository {
    fn find_recent_for_pet(
        &self,
        pet_id: entities::PetId,
        location_id: entities::LocationId,
        window: operations::daycare::AttendanceFrequencyWindow,
    ) -> operations::daycare::Result<Vec<operations::daycare::StoredPackageOpportunity>>;

    fn save_recommendation(
        &self,
        decision: operations::daycare::PackageOpportunityDecision,
    ) -> operations::daycare::Result<operations::daycare::StoredPackageOpportunity>;
}

pub trait operations::daycare::MembershipRepository {
    fn active_coverage_for_pet(
        &self,
        customer_id: entities::CustomerId,
        pet_id: entities::PetId,
        location_id: entities::LocationId,
    ) -> operations::daycare::Result<operations::daycare::MembershipCoverageState>;
}
```

Behavior ownership:

- `PackageOpportunityPolicy::classify` owns threshold/reason/blocker classification because it is a policy decision over evidence and catalog. Do not bury this in `utils::detect_package_candidate`.
- `PackageOpportunityEvidence::builder()` owns completeness and snapshot invariants; missing safety/payment/catalog facts create typed blockers or review states.
- `PackageCatalog` owns offer compatibility checks such as `catalog.compatible_offers_for(care_mode, service_variant, location_id)` because catalog terms define what can be sold.
- `MembershipPlan` owns plan-term compatibility (cadence, term, service family). It does not inspect temperament or payment history directly.
- `PackageOpportunityService` orchestrates repositories and policies; it does not invent pricing, mutate payment, send messages, or clear eligibility.
- `OpportunityReviewPacket` owns customer-safe/staff-safe redaction boundaries. Sensitive incident/care/temperament details can be summarized for staff/manager but not included in customer offer drafts.

Precise behavior rules:

1. Unknown eligibility, payment state, package catalog, attendance history, or customer contact permission never produces `RecommendStaffReview` without blockers; it produces `Blocked` or `RequiresManagerReview`.
2. Existing active compatible coverage suppresses duplicate sales prompts and may produce a front-desk note about remaining visits/term only if source data is trustworthy.
3. Package/membership recommendation must include `PackageOpportunityReason`; recommendations with empty reasons are invalid.
4. Customer-facing draft generation requires an approved template or review packet. It must not state exact savings, discount, availability, priority, or enrollment unless those facts came from policy/catalog and passed review.
5. Package visit consumption belongs to checkout/attendance/payment workflows, not opportunity detection.
6. `ServiceKind::Daycare` summary values may carry coarse `RevenueOpportunityKind::DaycarePackageCandidate`, but downstream behavior must use daycare-owned detail types before action.

## 5. Review/approval contract

Automation level:

- Evidence gathering, pattern classification, blocker detection, and internal review-packet drafting can be `policy::AutomationLevel::DraftOnly` or recommendation-only.
- Creating an internal staff task from a deterministic recommendation can be allowed where location policy permits.
- Customer-message draft generation is allowed only as draft output with `policy::ReviewGate::CustomerMessageApproval`.

Staff review gates:

- Staff/front-desk may review ordinary `RecommendStaffReview` outcomes where package/membership details are policy-approved and no pricing/payment/eligibility blocker exists.
- Staff must verify customer intent, preferred contact, offer wording, pet/service fit, and current package state before customer-facing outreach.
- Staff may not clear manager-only pricing, discount, refund, credit, capacity, or incident restrictions.

Manager approval gates:

- Required for discounts, waivers, refunds/credits, exception pricing, enrollment commitments, priority/recurrence promises, suppressing capacity/staffing blockers, and unresolved incident/care/eligibility concerns.
- Required when the system cannot distinguish package opportunity from a billing correction or package-consumption error.
- Required before changing local package catalog, thresholds, expiration rules, or membership terms.

Audit trail:

- Record the triggering event, evidence snapshot IDs, policy/catalog version, attendance window, reasons, blockers, recommended offer/plan ID, review gate, reviewer identity, disposition, and any customer-message draft ID.
- Audit subjects should include customer, pet, location, and any reservation/package/membership IDs; metadata uses `entities::AuditMetadataKey`/`AuditMetadataValue`, not raw maps in domain core.
- Dispositions: `CreatedStaffTask`, `DraftedCustomerMessage`, `DismissedAlreadyCovered`, `SuppressedByEligibility`, `EscalatedToManager`, `ApprovedForPaymentWorkflow`, `DeclinedByCustomer`, `Deferred`.

Customer/member-facing boundaries:

- Agents and automatic policy evaluation must not send offers, quote unapproved savings, promise availability/priority booking, enroll a membership, consume package visits, capture payment, waive fees, apply discounts, or modify billing.
- Customer-visible text must not expose internal scores, sensitive care/behavior/incident details, or unreviewed medical/temperament observations.
- If a package opportunity is blocked by safety/eligibility/care facts, the customer-safe message should ask for approved next steps or missing documentation, not imply the pet is unsafe or blame the pet.

## 6. Test contracts

Named semantic tests for later implementation:

1. `frequent_pay_per_visit_daycare_attendance_creates_package_candidate_review`
   - Given repeated eligible daycare visits without active coverage, policy returns `RecommendStaffReview` with `PackageOpportunityReason::FrequentPayPerVisit` and no payment mutation.

2. `active_compatible_package_suppresses_duplicate_offer`
   - Given active package coverage for the same pet/location/care mode, policy returns `ExistingCoverage` rather than another sales prompt.

3. `active_membership_suppresses_package_candidate_unless_catalog_policy_allows_upgrade_review`
   - Existing membership blocks ordinary package offers; any upgrade/cross-sell is manager/staff review with explicit policy support.

4. `package_candidate_is_bound_to_pet_care_mode_and_location`
   - A dog group-play package candidate cannot be reused for cat individual playtime, day boarding, another pet, or another location without explicit catalog compatibility.

5. `group_play_package_requires_current_group_play_eligibility`
   - Missing/stale temperament, vaccine, spay/neuter, or unresolved incident evidence yields `EligibilityReviewRequired` before a group-play package can be offered.

6. `day_boarding_package_does_not_require_dog_group_play_eligibility`
   - A dog ineligible for group play may still be a day-boarding package candidate if individual care eligibility, room capacity, and local package policy allow it.

7. `unknown_package_catalog_blocks_offer_instead_of_inventing_terms`
   - Missing package/membership catalog produces `PolicyUnavailable` or `InsufficientEvidence`, not a generic membership message.

8. `package_balance_low_creates_staff_review_not_auto_renewal`
   - Low remaining visits can create a review packet/task, but no renewal, payment capture, or package purchase occurs.

9. `package_expiration_near_term_creates_safe_follow_up_with_approved_template`
   - Near-expiration evidence can draft customer-safe follow-up only when a reviewed template/catalog rule exists.

10. `payment_or_billing_ambiguity_requires_manager_review`
    - Open balance, failed payment, refund/credit ambiguity, or package-consumption mismatch returns `RequiresManagerReview` and does not offer pricing.

11. `capacity_or_staffing_risk_manager_gates_recurring_membership_offer`
    - If projected recurring daycare volume conflicts with ratio/capacity policy, recommendation escalates before promising recurring slots or priority.

12. `customer_contact_preference_blocks_unapproved_outreach`
    - Do-not-contact or unsupported channel state prevents customer-message draft/send and creates a staff task only if policy permits.

13. `customer_message_draft_for_package_opportunity_carries_review_gate`
    - All customer-facing package/membership drafts include `ReviewGate::CustomerMessageApproval`; pricing/enrollment drafts also carry manager/payment approval where required.

14. `package_opportunity_audit_trail_records_evidence_policy_and_disposition`
    - Saved recommendations include evidence snapshot, policy/catalog version, reasons/blockers, reviewer, and final disposition.

15. `revenue_opportunity_summary_does_not_replace_daycare_detail_contract`
    - `RevenueOpportunityKind::DaycarePackageCandidate` is emitted for briefs, but action logic requires the daycare-owned `PackageOpportunityDecision` detail.

16. `customer_safe_draft_redacts_incident_temperament_and_care_details`
    - Review packets may preserve staff/manager facts, but customer drafts do not leak sensitive behavior, medical, incident, or internal scoring details.

## 7. Integration notes for later serialized Rust code card

Likely files touched:

- `domain/src/operations.rs`: add/refine `operations::daycare` package/membership opportunity types, policies, repositories, service contracts, errors, and perhaps a future nested file split if the module grows.
- `domain/src/entities.rs`: add audit subjects/actions or package/membership identifiers only if they are core entities rather than daycare-local refs.
- `domain/src/workflow.rs`: add event types for package opportunity detection/review if current workflow events are too generic.
- `domain/src/agents.rs`: add or refine baseline agent spec for daycare package opportunity review; ensure forbidden actions cover enrollment/payment/discount/send boundaries.
- `domain/src/tools.rs`: add read-only package catalog/attendance/membership tools and draft-only task/message tools; keep payment tools gated.
- `domain/src/payment/mod.rs` and `domain/src/money/mod.rs`: integrate only for approved payment/enrollment workflows, not recommendation detection.
- `domain/tests/petsuites_core_service_contracts.rs`: add contract-level tests for `PackagePolicy`/catalog/refined daycare contracts.
- `domain/tests/domain_quality_patterns.rs`: add semantic tests for opportunity decisions, review gates, audit metadata, and redaction.
- Future storage adapter tests: verify provider package codes and membership terms convert into semantic IDs/plans/terms without raw-string branching in the domain core.

Migration/refactor risks:

- Existing `PackagePolicy::Membership` is too coarse; replacing it may break current simple tests unless re-exported or migrated incrementally.
- `RevenueOpportunityKind::DaycarePackageCandidate` is intentionally coarse. Do not overload it with package details; introduce a daycare-owned detail type and keep the summary enum for briefs.
- Payment/package consumption can easily bleed into opportunity detection. Keep recommendation, enrollment, payment, and consumption as separate domain operations.
- Avoid boolean traps: `is_member`, `has_package`, `should_offer`, `eligible_for_membership`, and `auto_renew` hide separate coverage, commercial fit, safety eligibility, customer consent, and payment authorization concepts.
- Customer-visible copy risks leaking sensitive behavior/care facts. Review packets need separate staff/manager and customer-safe projections.
- Membership/package catalogs may vary by location and source system. Use semantic catalog repositories and provider-code conversions; do not branch on raw service/package names inside policy.
- Capacity/staffing impact of membership/recurrence should be manager-gated until deterministic forecasting exists.

Dependencies on other daycare implications/domain work:

- Depends on the Daycare service map's `PackageOpportunityService`, `MembershipEligibility`, `DailyRecurringAttendance`, `FrontDeskReadiness`, `GroupPlayEligibilityDecision`, and repository contracts.
- Should compose with group-play eligibility, staff coverage/capacity, daily recurring attendance, front-desk readiness, incident policy, and customer-message review implications when those are written.
- Later serialized Rust cards should implement the semantic decision/core types before wiring agents/tools, so review gates and forbidden actions can be tested against concrete domain outcomes.

Doc-only status: this artifact changes no code, storage schemas, live systems, customer/member data, package catalogs, payment state, or operational policy.
