//! Checkout-completion workflow rules for staff departure handoff and closeout review.
//!
//! ## Operator summary
//!
//! Staff use this workflow to route a departure to manager-gated handoff review or
//! source-status reconciliation. It compares observed source reservation status with caller-reported
//! front-desk handoff evidence such as belongings return, care summary, and departure-note review.
//! Serialized handoff evidence never creates verified-checkout or retention authority.
//!
//! The workflow may summarize checkout evidence, create an internal handoff task, and produce
//! audit-event drafts. It never grants the retention-follow-up draft action from serialized packet
//! state. It is not
//! allowed to close a live PMS/provider record, send a customer message, apply a checkout status
//! without staff/source agreement, release capacity, waive/discount/refund, collect payment, or
//! move money. Payment and closeout surfaces remain review queues, not autonomous execution.
//!
//! Source facts remain authoritative in their own systems: `domain::source::Provenance` and
//! `domain::source::reservation::Status` for observed provider state, staff-submitted handoff
//! evidence for belongings and departure notes, `domain::entities::reservation::Status` for the
//! normalized lifecycle suggestion, and approved payment/ledger records for balances, refunds,
//! discounts, or waivers. Review gates protect pets, customers, and staff by requiring manager
//! approval when source or handoff evidence is incomplete and customer-message approval before any
//! departure or retention copy leaves draft form.

use chrono::{DateTime, Utc};
use domain::{entities, policy, source};
use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::fmt;

pub use domain::boarding::handoff::DepartureTaskDraft as StaffTaskDraft;
pub use domain::payment::CheckoutException as PaymentException;
pub use domain::reservation::CheckoutSourceException as SourceException;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Staff minutes used to compare manual checkout audit effort with packet-review effort.
pub struct LaborMinutes(u16);

impl LaborMinutes {
    /// Validates non-zero minutes before a checkout labor estimate can appear in a packet.
    pub const fn try_new(value: u16) -> Result<Self, &'static str> {
        if value == 0 {
            return Err("checkout labor minutes must be greater than zero");
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Evidence-backed reasons that a departure remains under staff or manager review.
pub enum ReviewReason {
    /// Caller-reported departure evidence always requires authenticated staff review.
    DepartureEvidenceRequiresReview,
    /// The caller reports that belongings still need staff follow-up.
    BelongingsFollowUpReported,
    /// The caller reports that care or departure notes still need manager review.
    CareReviewReported,
    /// The observed source record does not report a checked-out reservation.
    SourceNotCheckedOut,
    /// Payment, refund, discount, waiver, or balance issue retained for ledger/PMS review.
    Payment(PaymentException),
    /// Source/PMS checkout state or provider record conflict retained for reconciliation.
    Source(SourceException),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reviewable labor estimate for the open-stay audit workflow.
pub struct LaborImpact {
    manual_audit_minutes: LaborMinutes,
    packet_review_minutes: LaborMinutes,
}

impl LaborImpact {
    /// Captures the expected manual audit effort and packet-review effort without claiming realized savings.
    pub const fn new(
        manual_audit_minutes: LaborMinutes,
        packet_review_minutes: LaborMinutes,
    ) -> Self {
        Self {
            manual_audit_minutes,
            packet_review_minutes,
        }
    }

    /// Returns the manual checkout audit effort estimated for front-desk staff.
    pub const fn manual_audit_minutes(&self) -> LaborMinutes {
        self.manual_audit_minutes
    }

    /// Returns the packet review effort estimated for front-desk staff.
    pub const fn packet_review_minutes(&self) -> LaborMinutes {
        self.packet_review_minutes
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1200),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct CareSummary(String);

impl fmt::Debug for CareSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CareSummary(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Current review-safe checkout tasks.
pub enum SafeAgentAction {
    /// Allows agents to summarize checkout evidence for staff review without mutating records or contacting customers.
    SummarizeCheckoutEvidence,
    /// Allows agents to create internal handoff task for staff review without mutating records or contacting customers.
    CreateInternalHandoffTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Actions the agent must never perform without a human/operator system of record.
pub enum BlockedAction {
    /// Blocks agents from suggest checked out status until staff or the system of record performs the action.
    SuggestCheckedOutStatus,
    /// Blocks agents from send customer message until staff or the system of record performs the action.
    SendCustomerMessage,
    /// Blocks agents from mutate provider or pms record until staff or the system of record performs the action.
    MutateProviderOrPmsRecord,
    /// Blocks agents from move refund discount or payment until staff or the system of record performs the action.
    MoveRefundDiscountOrPayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Internal audit-draft labels describe pending evidence only; they prove no review, event, action, queue admission, or checkout completion.
pub enum AuditEventDraft {
    /// Selects source checkout observed for the checkout completion decision model so the app can choose a review, evidence, or draft path without taking live action.
    SourceCheckoutObserved,
    /// Records the staff-submitted handoff payload as received, even when the source status prevents
    /// treating it as checkout-completion evidence.
    DepartureObservationRecorded,
    /// Selects staff handoff review requested for the checkout completion decision model so the app can choose a review, evidence, or draft path without taking live action.
    DepartureReviewRequested,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Caller-reported departure evidence; its actor, time, and boolean reports prove no authenticated identity, review, action, queue admission, or checkout completion.
pub struct DepartureObservation {
    reported_by: entities::ActorRef,
    reported_at: DateTime<Utc>,
    reported_belongings_returned: bool,
    care_summary: CareSummary,
    reported_care_summary_reviewed: bool,
}

impl fmt::Debug for DepartureObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DepartureObservation([REDACTED])")
    }
}

impl DepartureObservation {
    /// Returns the caller-reported actor label; it authenticates no actor, completion, or review.
    pub const fn reported_by(&self) -> &entities::ActorRef {
        &self.reported_by
    }

    /// Returns the caller-reported observation time; it proves no completion, review, or action.
    pub const fn reported_at(&self) -> DateTime<Utc> {
        self.reported_at
    }

    /// Reports whether the caller says belongings were returned; this is evidence, not verified completion.
    pub const fn reported_belongings_returned(&self) -> bool {
        self.reported_belongings_returned
    }

    /// Returns the care summary evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn care_summary(&self) -> &CareSummary {
        &self.care_summary
    }

    /// Reports whether the caller says the care summary was reviewed; this authenticates no review.
    pub const fn reported_care_summary_reviewed(&self) -> bool {
        self.reported_care_summary_reviewed
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Input rules for building the workflow packet from source-grounded records.
pub struct Request {
    reservation_id: entities::reservation::Id,
    source_provenance: source::Provenance,
    observed_source_status: source::reservation::Status,
    departure_observation: DepartureObservation,
    payment_exception: Option<PaymentException>,
    source_exception: Option<SourceException>,
    #[builder(default = LaborMinutes::try_new(15).expect("default checkout audit minutes are non-zero"))]
    estimated_manual_audit_minutes: LaborMinutes,
    #[builder(default = LaborMinutes::try_new(5).expect("default checkout packet minutes are non-zero"))]
    estimated_packet_review_minutes: LaborMinutes,
}

impl fmt::Debug for Request {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Request([REDACTED])")
    }
}

impl Request {
    /// Returns the reservation id evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the source provenance evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn source_provenance(&self) -> &source::Provenance {
        &self.source_provenance
    }

    /// Returns the observed source status evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn observed_source_status(&self) -> source::reservation::Status {
        self.observed_source_status.clone()
    }

    /// Returns caller-reported departure evidence while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn departure_observation(&self) -> &DepartureObservation {
        &self.departure_observation
    }

    /// Returns retained payment exception evidence; agents may route it, not move money.
    pub const fn payment_exception(&self) -> Option<PaymentException> {
        self.payment_exception
    }

    /// Returns retained source exception evidence; agents may route it, not mutate providers.
    pub const fn source_exception(&self) -> Option<SourceException> {
        self.source_exception
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
/// Workflow-issued review packet. It is intentionally not deserializable because serialized evidence cannot recreate workflow review output.
pub struct ReviewPacket {
    reservation_id: entities::reservation::Id,
    provenance: source::Provenance,
    departure_observation: DepartureObservation,
    review_reasons: Vec<ReviewReason>,
    required_review_gates: Vec<policy::ReviewGate>,
    safe_agent_actions: Vec<SafeAgentAction>,
    blocked_actions: Vec<BlockedAction>,
    audit_event_drafts: Vec<AuditEventDraft>,
    staff_task_drafts: Vec<StaffTaskDraft>,
    labor_impact: LaborImpact,
}

impl fmt::Debug for ReviewPacket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ReviewPacket([REDACTED])")
    }
}

impl ReviewPacket {
    /// Returns the reservation id evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn reservation_id(&self) -> entities::reservation::Id {
        self.reservation_id
    }

    /// Returns the provenance evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn provenance(&self) -> &source::Provenance {
        &self.provenance
    }

    /// Returns caller-reported departure evidence while leaving provider, customer, payment, and schedule systems unchanged.
    pub const fn departure_observation(&self) -> &DepartureObservation {
        &self.departure_observation
    }

    /// Returns the evidence-backed reasons this departure remains under review.
    pub fn review_reasons(&self) -> &[ReviewReason] {
        &self.review_reasons
    }

    /// Returns the required review gates evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }

    /// Returns the safe agent actions evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn safe_agent_actions(&self) -> &[SafeAgentAction] {
        &self.safe_agent_actions
    }

    /// Returns the blocked actions evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }

    /// Returns the audit event drafts evidence available to checkout completion review while leaving provider, customer, payment, and schedule systems unchanged.
    pub fn audit_event_drafts(&self) -> &[AuditEventDraft] {
        &self.audit_event_drafts
    }

    /// Returns draft-only staff task recommendations; agents may prepare these but not complete live checkout work.
    pub fn staff_task_drafts(&self) -> &[StaffTaskDraft] {
        &self.staff_task_drafts
    }

    /// Returns estimated labor impact for open-stay audit packet review; this is not a realized savings claim.
    pub const fn labor_impact(&self) -> &LaborImpact {
        &self.labor_impact
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Workflow used by the checkout completion workflow; it keeps checkout tasks, payment exceptions, and handoff notes explicit for staff review.
pub struct Workflow;

impl Workflow {
    /// Builds the evaluate result for the checkout completion workflow from reviewed source facts while preserving human review gates and draft-only side effects.
    pub fn evaluate(request: Request) -> ReviewPacket {
        let review_reasons = review_reasons_for(&request);
        let staff_task_drafts = staff_task_drafts_for(&review_reasons);
        let labor_impact = LaborImpact::new(
            request.estimated_manual_audit_minutes,
            request.estimated_packet_review_minutes,
        );

        ReviewPacket {
            reservation_id: request.reservation_id,
            provenance: request.source_provenance,
            departure_observation: request.departure_observation,
            review_reasons,
            required_review_gates: vec![policy::ReviewGate::ManagerApproval],
            safe_agent_actions: vec![
                SafeAgentAction::SummarizeCheckoutEvidence,
                SafeAgentAction::CreateInternalHandoffTask,
            ],
            blocked_actions: blocked_actions_for_review(),
            audit_event_drafts: audit_event_drafts_for(&request.observed_source_status),
            staff_task_drafts,
            labor_impact,
        }
    }
}

fn blocked_actions_for_review() -> Vec<BlockedAction> {
    let mut blocked_actions = vec![
        BlockedAction::SendCustomerMessage,
        BlockedAction::MutateProviderOrPmsRecord,
        BlockedAction::MoveRefundDiscountOrPayment,
    ];
    blocked_actions.push(BlockedAction::SuggestCheckedOutStatus);
    blocked_actions.sort_unstable();
    blocked_actions.dedup();
    blocked_actions
}

fn audit_event_drafts_for(
    observed_source_status: &source::reservation::Status,
) -> Vec<AuditEventDraft> {
    let mut drafts = vec![
        AuditEventDraft::DepartureObservationRecorded,
        AuditEventDraft::DepartureReviewRequested,
    ];
    if matches!(
        observed_source_status,
        source::reservation::Status::CheckedOut
    ) {
        drafts.push(AuditEventDraft::SourceCheckoutObserved);
    }
    drafts.sort_unstable();
    drafts.dedup();
    drafts
}

fn review_reasons_for(request: &Request) -> Vec<ReviewReason> {
    let mut reasons = vec![ReviewReason::DepartureEvidenceRequiresReview];

    if !request.departure_observation.reported_belongings_returned() {
        reasons.push(ReviewReason::BelongingsFollowUpReported);
    }

    if !request
        .departure_observation
        .reported_care_summary_reviewed()
    {
        reasons.push(ReviewReason::CareReviewReported);
    }

    if let Some(payment_exception) = request.payment_exception() {
        reasons.push(ReviewReason::Payment(payment_exception));
    }

    if let Some(source_exception) = request.source_exception() {
        reasons.push(ReviewReason::Source(source_exception));
    }

    if !matches!(
        request.observed_source_status,
        source::reservation::Status::CheckedOut
    ) {
        reasons.push(ReviewReason::SourceNotCheckedOut);
    }

    reasons.sort_unstable();
    reasons.dedup();
    reasons
}

fn staff_task_drafts_for(reasons: &[ReviewReason]) -> Vec<StaffTaskDraft> {
    let mut drafts = Vec::new();
    for reason in reasons {
        match reason {
            ReviewReason::DepartureEvidenceRequiresReview => {}
            ReviewReason::BelongingsFollowUpReported => {
                drafts.push(StaffTaskDraft::VerifyBelongingsReturn);
            }
            ReviewReason::CareReviewReported => {
                drafts.push(StaffTaskDraft::ReviewCareAndDepartureNotes);
            }
            ReviewReason::Payment(_) => {
                drafts.push(StaffTaskDraft::ResolvePaymentException);
            }
            ReviewReason::SourceNotCheckedOut | ReviewReason::Source(_) => {
                drafts.push(StaffTaskDraft::ReconcileSourceStatus);
            }
        }
    }
    drafts.sort_unstable();
    drafts.dedup();
    drafts
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn departure_observation_accessors_preserve_reported_actor_and_time() {
        let actor = entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("coverage-front-desk").unwrap(),
        };
        let observation = DepartureObservation::builder()
            .reported_by(actor.clone())
            .reported_at(DateTime::<Utc>::UNIX_EPOCH)
            .reported_belongings_returned(true)
            .care_summary(CareSummary::try_new("All belongings returned.").unwrap())
            .reported_care_summary_reviewed(true)
            .build();

        assert_eq!(observation.reported_by(), &actor);
        assert_eq!(observation.reported_at(), DateTime::<Utc>::UNIX_EPOCH);
    }
}
