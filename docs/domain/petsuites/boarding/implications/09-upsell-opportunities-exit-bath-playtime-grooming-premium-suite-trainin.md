# Boarding implication 09: upsell opportunities — exit bath, playtime, grooming, premium suite, training

Purpose: model how Boarding should surface safe, useful upsell opportunities without turning revenue hints into unreviewed customer offers, booking changes, care decisions, or payment actions. This is a domain-contract artifact for later Rust implementation cards; it does not authorize live customer messaging, reservation modification, payment collection, or member-facing offers.

Assumptions:

- PetSuites boarding upsells are location-scoped catalog items. Availability, price, staff capacity, and local exclusions are provider/location data, not constants in the domain core.
- Upsells can be generated at reservation request, pre-arrival review, check-in, during stay, check-out prep, and post-stay follow-up, but the safest default is internal recommendation until staff approve the customer-facing offer.
- “Premium suite” is a Boarding accommodation upgrade, not a generic add-on. It should be modeled as a requested/available accommodation transition with capacity rules.
- “Exit bath” may be a boarding add-on and may also map to the grooming service line. Boarding owns why it is suggested for a stay; grooming owns grooming-calendar/service fulfillment once accepted.
- Training and full grooming upsells should create cross-service handoff opportunities unless the customer has explicitly requested them. Boarding should not reserve trainer/groomer time automatically.

## 1. Operational story

### Trigger

Upsell opportunity evaluation is triggered by one of these Boarding lifecycle events:

- `workflow::WorkflowEventType::BookingRequested` or `BookingTriageNeeded` for a stay request with requested dates, pets, accommodation preference, and requested add-ons.
- Pre-arrival checklist generation for a confirmed/held boarding reservation.
- Check-in review after staff verify pet profile, care instructions, behavior notes, deposit status, belongings, and accommodation assignment.
- Daily care evidence or Pawgress Report drafting when staff notes reveal a care-safe enrichment/grooming/training opportunity.
- Check-out prep when staff prepare belongings, payment, report artifact, and departure tasks.
- Manager daily brief generation for revenue opportunities across upcoming stays.

### Actors

- Customer/member: the human responsible for the reservation. They may accept or decline an offer, but agents do not contact them without approval.
- Pet: the subject of care, eligibility, behavior, coat/cleanliness, training, and accommodation fit.
- Front desk staff: reviews customer-facing offer drafts, collects customer preference, and creates approved follow-up tasks.
- Kennel technician / care staff: supplies care/behavior/cleanliness evidence and may recommend or suppress playtime/exit-bath opportunities.
- Groomer: reviews grooming or exit-bath feasibility and owns grooming service fulfillment details.
- Trainer: reviews training fit and owns training consult/session fulfillment details.
- Manager / lead staff: approves sensitive recommendations, capacity upgrades, exceptions, discounts/waivers, or any recommendation linked to incident, behavior, medical, complaint, or capacity risk.
- AI agent: summarizes evidence and drafts internal `upsell::Recommendation` values, staff tasks, and manager brief items under deterministic policy.
- Deterministic Rust policy: evaluates eligibility, suppression reasons, review gates, and allowed automation level.

### Inputs

- `entities::Reservation`: location, customer, pets, service `Boarding`, status, dates, requested add-ons, deposit/payment state, and hard stops.
- `entities::Customer`: contact preference and portal/customer refs, used only for drafts and audit context.
- `entities::Pet`: species, spay/neuter status, temperament profile, care profile, medication/allergy/medical facts.
- `operations::boarding::StayPlan` / `StayContract`: accommodation, date range, care plan, capacity decision, and add-ons already accepted.
- `operations::boarding::Contract`: local upsell catalog, accommodation inventory posture, housekeeping, handoff, payment, and approval defaults.
- `operations::boarding::capacity::Snapshot`: segmented classic/luxury/cat inventory and holds for the stay dates.
- `operations::boarding::care::Plan`: medication/feeding/behavior review state, playtime eligibility, cleaning/exit-bath signals, and incomplete tasks.
- Grooming and training catalog summaries: service availability windows, required staff review, and coarse feasibility, not direct booking authority.
- Staff evidence: completed tasks, care notes, coat/cleanliness observations, playgroup notes, Pawgress Report source evidence, and incident/safety flags.
- Prior offer state: previously suggested, accepted, declined, suppressed, or completed opportunities for this stay.

### Decisions

The Boarding upsell policy answers:

1. Is the opportunity in the location’s Boarding catalog and active for this stay phase?
2. Is the pet/customer/stay context eligible, ineligible, review-required, or unknown?
3. Is the opportunity care-safe and not insensitive given medical, behavior, incident, complaint, stress, or payment context?
4. Is staff capacity available or is a cross-service review needed?
5. Is the customer-facing offer already accepted/declined, recently sent, or duplicate with an existing add-on?
6. Is the recommendation internal-only, draft-only, staff-review, manager-review, or never-automate?
7. If accepted, which truthful owner fulfills it: Boarding care task, Boarding accommodation upgrade, Grooming service handoff, Training service handoff, or customer follow-up task?

### Outputs

- `operations::boarding::upsell::Recommendation` values with typed opportunity kind, evidence, eligibility, suppression/review state, phase, and rationale.
- `operations::StaffTask` drafts for front desk review, groomer/trainer feasibility review, kennel-tech evidence collection, or manager approval.
- `workflow::RecommendedAction::InternalTask` or `RequestHumanReview`, never direct booking/payment/member-facing execution.
- `operations::RevenueOpportunity` / `DailyBriefSection::RevenueOpportunities` summary preserving Boarding source context.
- Optional `workflow::RecommendedAction::DraftMessage` only after policy marks the offer draftable and the draft carries `policy::ReviewGate::CustomerMessageApproval` until staff approval.
- Audit events recording evaluation, suppression, review, approval, offer sent, accepted, declined, and fulfillment handoff.

### Success state

The success state is not “sell more at any cost.” It is:

- eligible opportunities are surfaced internally with enough evidence for staff to trust or reject them;
- unsafe, insensitive, duplicate, unavailable, or non-catalog opportunities are suppressed with typed reasons;
- customer-facing offer text is draft-only until reviewed;
- accepted opportunities produce handoffs to the owner that can fulfill them;
- all recommendations and approvals are auditably tied to reservation, pet, actor, policy version, and evidence.

### Failure and exception states

- Missing pet profile, species, care instructions, temperament review, or spay/neuter status: route to `upsell::Eligibility::UnknownNeedsReview` or suppress playtime; create profile/care review tasks instead of guessing.
- Medical, medication, allergy, injury, incident, stress, aggression, complaint, or safety signal: suppress non-essential upsells unless a manager explicitly approves a sensitive follow-up.
- Deposit/payment/cancellation dispute: suppress marketing-like offer drafts and route to manager/front-desk review.
- Premium suite unavailable or inventory limited: do not promise upgrade; emit a `capacity::Decision::Waitlist`/`ManagerReview`-backed opportunity only for internal review.
- Groomer/trainer capacity unknown: emit cross-service feasibility task, not a bookable offer.
- Customer previously declined or recently received a similar offer: suppress duplicate with audit reason.
- Customer-facing draft contains medical, behavior, incident, denial, pricing exception, or policy explanation: manager/customer-message approval required.
- Accepted add-on cannot be fulfilled after approval: create a service-recovery/manager task and preserve audit trail; do not silently remove it.

## 2. Domain types to add or refine

### Root upsell module

- `operations::boarding::upsell::Opportunity`
  - Variants:
    - `ExitBath(exit_bath::Opportunity)`
    - `Playtime(playtime::Opportunity)`
    - `Grooming(grooming_handoff::Opportunity)`
    - `PremiumSuite(upgrade::PremiumSuiteOpportunity)`
    - `Training(training_handoff::Opportunity)`
  - Invariant: each variant carries the semantic detail needed by the owner that evaluates/fulfills it; do not collapse to a label string.

- `operations::boarding::upsell::Recommendation`
  - Fields: `reservation_id`, `customer_id`, `pet_id`, `stay_phase`, `opportunity`, `eligibility`, `approval_state`, `rationale`, `evidence`, `source`, `audit_ref`.
  - Invariant: cannot be constructed without at least one typed evidence item or explicit customer request; an agent “hunch” is not sufficient.

- `operations::boarding::upsell::Eligibility`
  - `Eligible { fulfillment_owner: FulfillmentOwner }`
  - `NeedsStaffReview { gate: policy::ReviewGate, reason: ReviewReason }`
  - `NeedsManagerReview { reason: ManagerReviewReason }`
  - `Suppressed { reason: SuppressionReason }`
  - `Unavailable { reason: AvailabilityReason }`
  - `AlreadyAccepted`
  - `AlreadyDeclined`
  - Invariant: only `Eligible` may be converted to a customer-offer draft, and still only under approval policy.

- `operations::boarding::upsell::ApprovalState`
  - `InternalDraft`, `StaffReviewRequired`, `ManagerReviewRequired`, `ApprovedToOffer { approved_by: entities::StaffId }`, `Offered`, `Accepted`, `Declined`, `FulfillmentHandedOff`, `Completed`, `Cancelled`.
  - Invariant: `Offered` requires `ApprovedToOffer`; `Accepted` requires a customer response or staff-entered consent; `Completed` requires fulfillment evidence.

- `operations::boarding::upsell::StayPhase`
  - `BookingTriage`, `PreArrival`, `CheckIn`, `DuringStay`, `CheckoutPrep`, `PostStayFollowUp`, `ManagerDailyBrief`.

- `operations::boarding::upsell::Evidence`
  - `CustomerRequested(entities::ReservationId)`, `AcceptedCatalogAddOn(entities::AddOn)`, `CareTaskEvidence(operations::TaskCompletionEvidence)`, `StaffObservation(operations::OperationalObservation)`, `PawgressSource(report::SourceEvidence)`, `CapacitySnapshot(capacity::SnapshotId)`, `GroomingHistory(grooming_handoff::HistorySummary)`, `TrainingSignal(training_handoff::BehaviorSkillSignal)`, `LocationCatalog(catalog::Version)`.
  - Invariant: customer-facing recommendations must preserve evidence references rather than inventing unsupported claims.

- `operations::boarding::upsell::Rationale`
  - Non-empty bounded staff-facing rationale. This is not customer copy.

- `operations::boarding::upsell::SuppressionReason`
  - `CareOrMedicalReviewOpen`, `BehaviorOrIncidentRisk`, `PetStressOrAnxietySignal`, `PaymentOrComplaintContext`, `DuplicateOffer`, `CustomerRecentlyDeclined`, `NotInLocationCatalog`, `SpeciesIncompatible`, `AgeOrPolicyHardStop`, `InsufficientEvidence`, `CustomerContactNotApproved`.

- `operations::boarding::upsell::AvailabilityReason`
  - `PremiumSuiteUnavailable`, `GroomerCapacityUnknown`, `TrainerCapacityUnknown`, `PlaygroupCapacityUnavailable`, `NotAvailableForStayDates`, `LocationDoesNotOffer`.

- `operations::boarding::upsell::FulfillmentOwner`
  - `BoardingCare`, `BoardingAccommodation`, `GroomingService`, `TrainingService`, `FrontDeskFollowUp`, `ManagerReview`.

### Exit bath

- `operations::boarding::exit_bath::Opportunity`
  - Fields: `trigger`, `timing`, `requires_groomer_review`, optional `grooming_service_ref`.
  - Triggers: `CheckoutCleanliness`, `CustomerRequested`, `LongStay`, `PlayOrOutdoorActivity`, `StaffRecommended`.
  - Invariant: never eligible when allergies/medical/skin condition evidence requires review; route to groomer/manager instead.

- `operations::boarding::exit_bath::Timing`
  - `BeforeCheckout`, `DuringCheckoutWindow`, `PostStayAppointmentNeeded`.

### Playtime

- `operations::boarding::playtime::Opportunity`
  - Fields: `kind`, `eligibility_decision`, `capacity_hint`, `care_notes_required`.
  - Reuse/refine `playtime::Kind` from the service-domain map: `DogGroupPlay`, `DogIndividualPlay`, `CatIndividualPlay`, `PrivateEnrichment`.
  - Invariant: dog group play must incorporate `policy::PlayEligibilityDecision`; cats never inherit dog group-play rules.

- `operations::boarding::playtime::CapacityHint`
  - `Available`, `LimitedNeedsStaffReview`, `Unavailable`, `Unknown`.

### Grooming handoff

- `operations::boarding::grooming_handoff::Opportunity`
  - Fields: `service_interest`, `reason`, `history_summary`, `review_requirement`.
  - Reason variants: `ExitBathCandidate`, `RebookingDue`, `CoatConditionObserved`, `CustomerRequested`, `FirstTimeOffer`.
  - Invariant: Boarding can recommend a grooming handoff; grooming owns time estimate, calendar, no-show/deposit, groomer qualification, and style-history details.

- `operations::boarding::grooming_handoff::HistorySummary`
  - `NoKnownHistory`, `LastGroomingDateKnown`, `CadenceDue`, `NeedsGroomerReview`.

### Premium suite upgrade

- `operations::boarding::upgrade::PremiumSuiteOpportunity`
  - Fields: `current_accommodation`, `target_accommodation`, `capacity_decision`, `reason`.
  - Invariant: target must be a dog luxury/premium accommodation where supported; no cat/dog mismatch; cannot be offered without capacity evidence.

- `operations::boarding::upgrade::UpgradeReason`
  - `CustomerPreference`, `ExtendedStayComfort`, `HolidayCapacityOptimization`, `ManagerSuggested`, `ServiceRecoveryOnly`.
  - `ServiceRecoveryOnly` must require manager approval and should not be treated as revenue upsell.

### Training handoff

- `operations::boarding::training_handoff::Opportunity`
  - Fields: `signal`, `program_hint`, `review_requirement`.
  - Signals: `PuppyManners`, `LeashWalking`, `Recall`, `AnxietyConfidenceBuilding`, `BehaviorReviewRequired`, `CustomerRequested`.
  - Invariant: behavior-risk signals do not become customer-facing “your pet needs training” messages without manager/staff copy review.

- `operations::boarding::training_handoff::BehaviorSkillSignal`
  - Staff/agent-safe summary of observed trainable skills, distinct from incident/diagnosis labels.

### Catalog and offer state

- `operations::boarding::upsell::Catalog`
  - Location-scoped set of active opportunities, cross-service handoff rules, default review gates, and duplicate suppression windows.

- `operations::boarding::upsell::CatalogItem`
  - Fields: `opportunity_kind`, `availability_policy`, `default_automation_level`, `fulfillment_owner`, optional cross-service catalog reference.

- `operations::boarding::upsell::OfferState`
  - Per-reservation/pet/customer state: `NotEvaluated`, `Recommended`, `ApprovedToOffer`, `Offered`, `Accepted`, `Declined`, `Suppressed`, `Fulfilled`, `Cancelled`.

- `operations::boarding::upsell::DuplicateWindow`
  - Positive duration or stay-phase scoped rule used to suppress repeated offers.

## 3. Relationship map

### Entities and value objects

- `entities::Reservation` anchors the stay, requested add-ons, status, deposit, hard stops, and date window.
- `entities::Customer` provides identity/contact preference; it does not authorize sends.
- `entities::Pet` provides species, spay/neuter, temperament, and care profile used by eligibility.
- `operations::boarding::StayPlan` and `StayContract` supply accommodation, care plan, and accepted add-ons.
- `operations::boarding::upsell::Recommendation` is the central Boarding-owned aggregate for a recommendation.
- `operations::boarding::upsell::Opportunity` preserves the kind-specific meaning.
- `operations::boarding::upsell::Eligibility`, `ApprovalState`, `SuppressionReason`, and `AvailabilityReason` encode safe state transitions.

### Policies

- `operations::boarding::upsell::Policy` owns recommendation eligibility/suppression and review gates.
- `operations::boarding::playtime::Policy` owns playtime enrichment eligibility in Boarding context and composes `policy::PlayEligibilityPolicy`.
- `operations::boarding::capacity::Policy` owns premium-suite availability and upgrade feasibility.
- `operations::boarding::care::Policy` owns care/medical/behavior suppressions.
- `operations::boarding::agent::ApprovalPolicy` maps recommendations, offer drafts, and fulfillment handoffs to `policy::AutomationLevel` and `policy::ReviewGate`.
- Grooming/training policies own fulfillment-specific feasibility after Boarding creates a handoff.

### Repositories and stores

- `operations::boarding::upsell::Repository` stores recommendation/offer state, suppression reasons, approvals, and evidence refs.
- `operations::boarding::catalog::Repository` reads location catalog items and cross-service refs.
- `operations::boarding::reservation::Repository` reads Boarding reservation projections and requested add-ons.
- `operations::boarding::care::Repository` reads care/profile/task evidence and writes internal care-review task drafts.
- `operations::boarding::capacity::Repository` reads premium-suite inventory snapshots and holds.
- `operations::boarding::grooming_handoff::Repository` reads coarse grooming history/availability projections and creates review handoff drafts.
- `operations::boarding::training_handoff::Repository` reads coarse training program availability/projections and creates review handoff drafts.

### Workflow events, staff tasks, and audit

- Workflow events: `BookingRequested`, `BookingTriageNeeded`, `DailyNoteCreated`, `DailyUpdateNeeded`, `CheckoutCompleted`, `ReviewRequestEligible`, plus future Boarding-specific events such as `UpsellEvaluationRequested`, `UpsellOfferApproved`, `UpsellAccepted`, and `UpsellFulfillmentFailed` if the workflow module grows.
- Staff tasks:
  - Front desk: review offer draft, contact customer after approval, record accept/decline.
  - Kennel technician: add care/cleanliness/play evidence, review playtime eligibility.
  - Groomer: review exit bath/grooming feasibility.
  - Trainer: review training consult/session fit.
  - Manager: approve sensitive, capacity-limited, service-recovery, exception, or complaint-adjacent offers.
- Audit subjects: reservation, pet, customer, workflow event, external provider references.
- Audit actions: policy decision recorded, workflow event recorded, reservation status suggested, extension labels for upsell evaluated/offered/accepted/declined/fulfilled/suppressed.

### Agent specs and tools

- Existing `agents::baseline_agent_specs()` covers `booking-triage`, `daily-care-update`, `manager-daily-brief`, `lead-conversion`, and `grooming-rebooking` as adjacent agent specs.
- Add/refine `boarding-upsell-recommender` agent spec:
  - allowed tools: `reservation-read`, `care-note-read`, `availability-read`, `catalog-read`, `task-create`, `draft-message` only after policy permits draft creation;
  - forbidden actions: confirm/modify booking, charge/refund/waive money, promise availability, book groomer/trainer, send customer message, hide care concerns;
  - default review gates: `CustomerMessageApproval`, `ManagerApproval` when sensitive/capacity-limited.
- Tool adapters execute only approved drafts; `tools::ReservationUpdateDraft`, messaging tools, payment tools, and calendar tools are outside automatic execution.

## 4. Interaction contract

Rust-like pseudo-signatures below describe ownership and behavior. Names are intentionally semantic and may be split across files in implementation.

```rust
pub trait operations::boarding::upsell::Repository {
    fn recommendations_for_stay(
        &self,
        stay: boarding::stay::Id,
    ) -> upsell::Result<Vec<upsell::Recommendation>>;

    fn save_recommendation(
        &mut self,
        recommendation: upsell::Recommendation,
    ) -> upsell::Result<upsell::RecommendationId>;

    fn offer_state(
        &self,
        reservation_id: entities::ReservationId,
        pet_id: entities::PetId,
        opportunity: upsell::OpportunityDiscriminant,
    ) -> upsell::Result<upsell::OfferState>;

    fn record_offer_transition(
        &mut self,
        transition: upsell::OfferTransition,
    ) -> upsell::Result<upsell::OfferState>;
}
```

```rust
impl operations::boarding::upsell::Policy {
    pub fn evaluate(
        &self,
        context: upsell::EvaluationContext<'_>,
    ) -> Vec<upsell::Recommendation>;

    pub fn evaluate_one(
        &self,
        opportunity: upsell::Opportunity,
        context: upsell::EvaluationContext<'_>,
    ) -> upsell::Recommendation;

    pub fn approval_for(
        &self,
        recommendation: &upsell::Recommendation,
    ) -> upsell::ApprovalDecision;
}
```

`upsell::Policy` owns cross-opportunity suppression rules because it sees the whole stay, prior offers, care review state, and duplicate windows. It may delegate kind-specific checks to `exit_bath::Policy`, `playtime::Policy`, `upgrade::Policy`, `grooming_handoff::Policy`, and `training_handoff::Policy`, but those are semantic child policies, not free-floating helpers.

```rust
pub struct operations::boarding::upsell::EvaluationContext<'a> {
    pub contract: &'a boarding::Contract,
    pub reservation: &'a entities::Reservation,
    pub customer: &'a entities::Customer,
    pub pet: &'a entities::Pet,
    pub stay_plan: &'a boarding::StayPlan,
    pub care_plan: &'a boarding::care::Plan,
    pub capacity: &'a boarding::capacity::Snapshot,
    pub catalog: &'a boarding::upsell::Catalog,
    pub prior_offers: &'a [boarding::upsell::OfferState],
    pub evidence: &'a [boarding::upsell::Evidence],
    pub phase: boarding::upsell::StayPhase,
}
```

```rust
impl operations::boarding::playtime::Policy {
    pub fn opportunity_for(
        &self,
        pet: &entities::Pet,
        stay: &boarding::StayPlan,
        care: &boarding::care::Plan,
        play_policy: &dyn policy::PlayEligibilityPolicy,
    ) -> playtime::OpportunityDecision;
}
```

`playtime::Policy` owns playtime safety. It composes the existing `policy::PlayEligibilityPolicy` and adds Boarding-specific capacity/care/stay-phase details. It must return typed suppression/review for behavior, species, stress, and staff-capacity cases.

```rust
impl operations::boarding::exit_bath::Policy {
    pub fn decide(
        &self,
        pet: &entities::Pet,
        care: &boarding::care::Plan,
        evidence: &[upsell::Evidence],
        phase: upsell::StayPhase,
    ) -> exit_bath::Decision;
}
```

`exit_bath::Policy` owns exit-bath safety and timing. It should reject or review when allergies, medical/skin conditions, medication, stress, or checkout-window capacity makes the offer unsafe.

```rust
impl operations::boarding::upgrade::Policy {
    pub fn premium_suite_opportunity(
        &self,
        stay: &boarding::StayPlan,
        capacity: &boarding::capacity::Snapshot,
        catalog: &upsell::Catalog,
    ) -> upgrade::Decision;
}
```

`upgrade::Policy` owns premium-suite feasibility but delegates inventory truth to `boarding::capacity::Policy`/`Repository`. It must not construct an offer without a capacity decision for the requested date range.

```rust
impl operations::boarding::grooming_handoff::Policy {
    pub fn grooming_opportunity(
        &self,
        pet: &entities::Pet,
        reservation: &entities::Reservation,
        care: &boarding::care::Plan,
        history: grooming_handoff::HistorySummary,
    ) -> grooming_handoff::Decision;
}

impl operations::boarding::training_handoff::Policy {
    pub fn training_opportunity(
        &self,
        pet: &entities::Pet,
        evidence: &[upsell::Evidence],
        care: &boarding::care::Plan,
    ) -> training_handoff::Decision;
}
```

These handoff policies own Boarding-side identification of opportunities. Grooming/training service modules own actual appointment/program availability, quote, booking, and fulfillment contracts.

```rust
impl operations::boarding::workflow::Planner {
    pub fn plan_upsell_review_tasks(
        &self,
        recommendations: &[upsell::Recommendation],
    ) -> Vec<operations::StaffTask>;

    pub fn summarize_revenue_opportunities(
        &self,
        recommendations: &[upsell::Recommendation],
    ) -> Vec<operations::RevenueOpportunity>;
}
```

The workflow planner owns conversion from Boarding recommendations into generic operations artifacts. It should preserve source context through typed fields, not by flattening everything into task titles.

```rust
impl operations::boarding::agent::ApprovalPolicy {
    pub fn upsell_action_level(
        &self,
        recommendation: &upsell::Recommendation,
        proposed_action: upsell::ProposedAction,
    ) -> policy::AutomationLevel;

    pub fn required_review_gates(
        &self,
        recommendation: &upsell::Recommendation,
        proposed_action: upsell::ProposedAction,
    ) -> Vec<policy::ReviewGate>;
}
```

Approval policy owns automation boundaries. Absence of a review gate is not approval; approval must be represented by an explicit typed transition.

## 5. Review and approval contract

### Automation level

- Safe to automate:
  - read-only evaluation of existing reservation/care/capacity/catalog data;
  - creation of internal recommendation values;
  - suppression of unsafe/duplicate/unavailable opportunities;
  - generation of internal staff-task drafts;
  - manager daily brief summaries marked as recommendations.
- Internal task only:
  - groomer/trainer feasibility review;
  - kennel-tech evidence collection;
  - front-desk review of a proposed offer;
  - manager review for capacity-limited or sensitive opportunities.
- Draft only:
  - customer-facing offer copy after deterministic eligibility permits drafting and no sensitive context exists.
- Staff approval required:
  - sending ordinary offer messages;
  - marking customer accepted/declined when response came through staff-mediated channel;
  - adding accepted non-sensitive add-on tasks to the stay plan.
- Manager approval required:
  - premium-suite upgrades under limited inventory or manager hold;
  - service recovery offers, discounts, waivers, refunds, or payment exceptions;
  - any offer adjacent to complaint, incident, injury, behavior risk, medical concern, or customer dissatisfaction;
  - customer-facing copy that explains policy, denial, safety, behavior, medical, or payment issues.
- Never automate:
  - booking a room/grooming/training slot;
  - charging, refunding, waiving, or forfeiting money;
  - modifying or confirming a live reservation;
  - sending customer messages without approval;
  - hiding negative care facts to make an offer sound better.

### Review gates

- `policy::ReviewGate::CustomerMessageApproval`: any customer-facing offer draft.
- `policy::ReviewGate::BehaviorReview`: playtime/training opportunities tied to temperament, spay/neuter ambiguity, anxiety, aggression, group-play stress, or behavior observations.
- `policy::ReviewGate::MedicalDocumentReview`: exit bath/grooming/playtime recommendation with allergies, medical conditions, medication, injury, or skin/coat health ambiguity.
- `policy::ReviewGate::ManagerApproval`: premium suite capacity exceptions, service recovery, sensitive customer context, complaint/incident adjacency, cross-service capacity uncertainty, or offer copy requiring policy explanation.
- `policy::ReviewGate::RefundOrDepositException`: any discount, deposit, refund, waiver, or payment-adjacent offer.

### Audit trail

Every recommendation evaluation should produce or be able to produce an audit record with:

- actor: agent, staff, manager, or system policy;
- subject: reservation, pet, customer, workflow event;
- policy/catalog version;
- opportunity kind and stay phase;
- evidence refs used;
- eligibility decision and suppression/review reason;
- approval state transition;
- customer-response source if offered/accepted/declined;
- fulfillment handoff and completion evidence if accepted.

### Customer/member-facing boundaries

- A recommendation is not an offer.
- An approved offer draft is not sent until a staff/manager actor clears the required review gate.
- An accepted offer is not fulfillment. It must become a Boarding care task, accommodation upgrade workflow, grooming handoff, or training handoff with its own owner and evidence.
- Customer copy must not imply diagnosis, blame, safety judgment, availability promise, discount/payment exception, or guaranteed outcome unless the owning policy and human review explicitly allow it.

## 6. Test contracts

Future implementation cards should add semantic tests like these:

- `boarding_upsell_recommendation_requires_evidence_or_customer_request`
  - A recommendation cannot be built from an empty evidence list unless the opportunity was explicitly requested by the customer/reservation.
- `boarding_upsell_policy_suppresses_duplicate_recent_offer_for_same_pet_and_stay`
  - Prior offered/declined state blocks repeated offer spam.
- `boarding_exit_bath_recommendation_is_suppressed_when_medical_or_allergy_review_is_open`
  - Exit bath does not surface as eligible when care profile contains unresolved medical/allergy review.
- `boarding_exit_bath_after_long_stay_creates_internal_front_desk_review_before_customer_message`
  - Long-stay evidence can create a front-desk review task, not an auto-sent offer.
- `boarding_playtime_recommendation_uses_policy_play_eligibility_decision`
  - Existing `policy::PlayEligibilityPolicy` gates dog group play recommendations.
- `boarding_playtime_for_cat_uses_cat_individual_play_not_dog_group_play`
  - Species semantics are preserved.
- `boarding_behavior_or_anxiety_flag_blocks_unsupervised_playtime_and_training_offer_drafts`
  - Behavior/anxiety evidence routes to review/suppression, not customer-facing upsell copy.
- `boarding_premium_suite_upgrade_requires_capacity_snapshot_for_requested_dates`
  - Premium-suite recommendations cannot be eligible without date-scoped capacity evidence.
- `boarding_premium_suite_upgrade_never_assigns_cat_to_dog_luxury_suite`
  - Accommodation/species mismatch returns typed denial or is unrepresentable.
- `boarding_limited_premium_suite_inventory_requires_manager_review_before_offer`
  - Limited/held inventory does not produce an automatic customer offer.
- `boarding_grooming_handoff_preserves_boarding_reason_but_leaves_calendar_ownership_to_grooming`
  - Boarding recommendation maps to a grooming review task rather than booking a slot.
- `boarding_training_handoff_from_skill_signal_requires_staff_review_before_customer_copy`
  - Training recommendation remains internal until staff review.
- `boarding_service_recovery_upgrade_is_manager_review_not_revenue_opportunity`
  - Service recovery is not counted as normal upsell revenue.
- `boarding_customer_message_draft_for_upsell_carries_customer_message_review_gate`
  - Draft messages are review-gated.
- `boarding_upsell_acceptance_creates_fulfillment_handoff_to_truthful_owner`
  - Accepted exit bath/playtime/premium-suite/grooming/training opportunities route to Boarding care, Boarding accommodation, grooming, or training owner as appropriate.
- `boarding_revenue_opportunity_daily_brief_preserves_reservation_pet_and_boarding_opportunity_context`
  - Generic `RevenueOpportunity` summaries do not lose Boarding source details.
- `boarding_upsell_policy_records_suppression_reason_for_insensitive_payment_or_complaint_context`
  - Suppression is audit-visible, not silent.
- `boarding_agent_can_create_internal_upsell_task_but_cannot_send_offer_or_modify_reservation`
  - Approval policy enforces automation boundaries.

## 7. Integration notes for later serialized Rust code card

### Files likely touched

- `domain/src/operations.rs`
  - Current `operations::boarding::Upsell::{ExitBath, TrainingSession, EnrichmentPlay, PremiumBedding}` is an early flat enum. Preserve/re-export during migration, but add child modules before behavior grows: `boarding::upsell`, `boarding::exit_bath`, `boarding::playtime`, `boarding::upgrade`, `boarding::grooming_handoff`, `boarding::training_handoff`.
  - Current `operations::BoardingAddOn::{Playtime, ExitBath, PremiumSuite, Grooming, TrainingSession}` should map into Boarding-owned opportunity/add-on semantics. Avoid adding more behavior to the broad root enum.
  - `RevenueOpportunityKind` already has `ExitBathAfterBoarding`, `GroomingRebookingDue`, `TrainingConsultCandidate`, and `HolidayBoardingWaitlistFill`; add Boarding source wrappers or mapping methods rather than relying on the generic enum as the core upsell model.
- `domain/src/policy.rs`
  - Reuse `PlayEligibilityPolicy`, `PlayEligibilityDecision`, `AutomationLevel`, and `ReviewGate`. Add only review reasons/gates that are truly missing.
- `domain/src/workflow.rs`
  - Existing `RecommendedAction`, `AllowedAction`, and `WorkflowEventType` may need upsell-specific variants or may carry upsell data through typed bodies initially. Prefer typed extensions when behavior depends on the distinction.
- `domain/src/agents.rs`
  - Add/refine a `boarding-upsell-recommender` agent spec or extend `manager-daily-brief`/`booking-triage` constraints to include upsell boundaries.
- `domain/src/entities.rs`
  - Reuse `Reservation.requested_add_ons`, `HardStop`, `AuditEvent`, `AuditAction::Extension`, `ActorRef`, `Customer`, `Pet`, and `ContactChannel`; do not duplicate customer/pet facts in Boarding types.
- `domain/tests/petsuites_core_service_contracts.rs`
  - Add tests for catalog/add-on semantics and standard PetSuites upsell defaults.
- `domain/tests/domain_quality_patterns.rs`
  - Add quality tests that ensure no new raw string/boolean helper soup appears in the upsell policy surface if the project has such pattern checks.
- Future storage module (likely `storage::operations` if present in the app/workspace)
  - Add codecs for `upsell::Recommendation`, `OfferState`, and catalog items after the domain model stabilizes.

### Migration/refactor risks

- Flat enum compatibility: existing serialization may expect `operations::boarding::Upsell` and root `BoardingAddOn`. Add conversion/re-export paths before replacing stored shapes.
- Grooming/training boundary leak: do not let Boarding book appointments or own grooming/training fulfillment. Boarding owns opportunity identification and handoff.
- Premium-suite confusion: model as accommodation/capacity upgrade, not as an arbitrary add-on. It requires date-scoped capacity evidence.
- Care-safety leakage: exit bath/playtime/training suggestions must consult care and behavior review state before producing offer drafts.
- Generic revenue summaries: `operations::RevenueOpportunity` is useful for daily brief output but too flat to own Boarding eligibility, suppression, and approval state.
- Customer-message safety: avoid helper functions that turn recommendation directly into message text; message drafts belong behind approval policy and review gates.
- Audit/versioning: recommendations should record catalog and policy version; otherwise later catalog changes make historical recommendation reasons ambiguous.

### Dependencies on other implications

- Accommodation/capacity segmentation must exist before premium-suite upgrade recommendations can be more than manager-review hints.
- Care/playtime/handoff implication must exist before playtime and exit-bath eligibility can be fully deterministic.
- Pawgress Report/source-evidence implication helps provide trustworthy stay evidence for during-stay and checkout upsells.
- Deposit/payment/cancellation implication is needed to suppress insensitive offers during disputes or payment exceptions.
- Grooming and training service maps are needed for cross-service handoff contracts, but Boarding can first model coarse review tasks and not direct bookings.

### Suggested implementation sequence

1. Add `boarding::upsell` core enums/newtypes and tests for evidence, eligibility, approval state, and suppression reasons.
2. Add conversion from existing `boarding::Upsell` and `BoardingAddOn` into typed opportunity discriminants while preserving serialization compatibility.
3. Add `upsell::Policy` with conservative suppression and `agent::ApprovalPolicy` tests; return internal recommendations only.
4. Add kind-specific policies for exit bath, playtime, premium suite, grooming handoff, and training handoff.
5. Add workflow planner mapping to staff tasks and manager daily brief revenue opportunities.
6. Add persistence/storage codecs only after domain tests lock down state transitions and conversion behavior.
