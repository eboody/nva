//! Daycare package-opportunity policy for review-gated membership/pass recommendations.
//!
//! ```
//! use domain::{daycare, entities, policy};
//! use uuid::Uuid;
//!
//! let evidence = daycare::package_opportunity::Evidence::builder()
//!     .customer_id(entities::CustomerId(Uuid::nil()))
//!     .pet_id(entities::PetId(Uuid::nil()))
//!     .attendance_visits(daycare::package_opportunity::AttendanceVisitCount::new(8))
//!     .eligibility(daycare::package_opportunity::CareEligibility::Cleared)
//!     .package_state(daycare::package_opportunity::PackageState::PayPerVisit)
//!     .payment_state(daycare::package_opportunity::PaymentState::Current)
//!     .build();
//!
//! assert_eq!(
//!     daycare::package_opportunity::Policy.classify(&evidence),
//!     daycare::package_opportunity::Decision::RecommendStaffReview {
//!         score: daycare::package_opportunity::OpportunityScore::Strong,
//!         gate: policy::ReviewGate::CustomerMessageApproval,
//!     },
//! );
//! ```

use super::*;
use crate::policy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Count of recent daycare visits used to score pass or membership opportunities.
pub struct AttendanceVisitCount(u16);

impl AttendanceVisitCount {
    /// Creates an attendance visit count from prior daycare check-in history.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the raw visit count for reporting, scoring, and serialization.
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Care eligibility state used before recommending daycare packages.
pub enum CareEligibility {
    /// Care and safety gates are clear enough to consider package recommendations.
    Cleared,
    /// Safety or care review blocks sales recommendations until staff clear it.
    BlockedBySafetyReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Existing package coverage state for a customer and pet.
pub enum PackageState {
    /// Customer currently pays per visit and may benefit from a package recommendation.
    PayPerVisit,
    /// Existing package or membership already covers the daycare need.
    AlreadyCovered,
    /// Package coverage could not be mapped confidently, so recommendations should not assume need.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Billing state used to suppress recommendations when collection or review is needed.
pub enum PaymentState {
    /// Payment status is current enough to allow staff-reviewed recommendations.
    Current,
    /// Billing state requires review before staff suggest another package or membership.
    NeedsBillingReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Source evidence used to classify daycare package or membership opportunities.
pub struct Evidence {
    /// Customer account that would receive the package recommendation.
    pub customer_id: CustomerId,
    /// Pet whose attendance history and care eligibility drive the recommendation.
    pub pet_id: PetId,
    /// Recent visit count used as the demand signal for package scoring.
    pub attendance_visits: AttendanceVisitCount,
    /// Care/safety eligibility that can suppress recommendations.
    pub eligibility: CareEligibility,
    /// Existing package coverage used to avoid duplicate sales prompts.
    pub package_state: PackageState,
    /// Billing readiness used to suppress recommendations needing collection review.
    pub payment_state: PaymentState,
    #[builder(default)]
    /// Source records behind attendance, package, eligibility, and billing evidence.
    pub source_record_refs: Vec<crate::source::RecordRef>,
}

impl Evidence {
    /// Returns source records staff can inspect before reviewing a package or membership opportunity.
    pub fn source_record_refs(&self) -> &[crate::source::RecordRef] {
        &self.source_record_refs
    }

    /// Reports whether this opportunity input is source-backed rather than model-only rationale.
    pub fn has_source_evidence(&self) -> bool {
        !self.source_record_refs.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Package-opportunity decision staff may review before contacting a customer.
pub enum Decision {
    /// Recommendation is promising enough to show staff, but customer messaging remains approval-gated.
    RecommendStaffReview {
        /// Opportunity strength derived from attendance history.
        score: OpportunityScore,
        /// Review gate required before staff use or send a recommendation.
        gate: policy::ReviewGate,
    },
    /// Recommendation is intentionally hidden because safety, care, or billing review comes first.
    Suppressed {
        /// Reason evidence does not allow a direct package recommendation.
        reason: SuppressionReason,
        /// Review gate required before staff use or send a recommendation.
        gate: policy::ReviewGate,
    },
    /// Evidence does not justify a package recommendation.
    NoOpportunity {
        /// Reason evidence does not allow a direct package recommendation.
        reason: NoOpportunityReason,
    },
}

impl Decision {
    /// Returns the human review gate required before customer messaging, billing, or package action.
    pub fn review_gate(&self) -> Option<policy::ReviewGate> {
        match self {
            Self::RecommendStaffReview { gate, .. } | Self::Suppressed { gate, .. } => {
                Some(gate.clone())
            }
            Self::NoOpportunity { .. } => None,
        }
    }

    /// Returns package, payment, provider, and send actions blocked by this decision.
    pub const fn blocked_actions(&self) -> &'static [BlockedAction] {
        match self {
            Self::RecommendStaffReview { .. } | Self::Suppressed { .. } => {
                PACKAGE_OPPORTUNITY_BLOCKED_ACTIONS
            }
            Self::NoOpportunity { .. } => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Live side effects blocked by daycare package opportunities until staff/system-of-record approval.
pub enum BlockedAction {
    /// Do not enroll, modify, renew, or cancel a package or membership autonomously.
    EnrollPackageOrMembership,
    /// Do not collect, refund, discount, credit, or invoice from attendance evidence alone.
    MutatePaymentOrInvoice,
    /// Do not write package decisions to Gingr/PMS/provider records.
    MutateProviderRecord,
    /// Do not send customer package or membership copy without approval.
    SendCustomerMessage,
}

const PACKAGE_OPPORTUNITY_BLOCKED_ACTIONS: &[BlockedAction] = &[
    BlockedAction::EnrollPackageOrMembership,
    BlockedAction::MutatePaymentOrInvoice,
    BlockedAction::MutateProviderRecord,
    BlockedAction::SendCustomerMessage,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Strength of a daycare package opportunity from recent attendance evidence.
pub enum OpportunityScore {
    /// Attendance history suggests a possible package fit, but not enough for the strongest score.
    Moderate,
    /// Attendance history strongly suggests staff should review a package or membership offer.
    Strong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons daycare package recommendations are suppressed before staff review.
pub enum SuppressionReason {
    /// Care or behavior review must be handled before sales recommendations.
    SafetyOrCareReviewRequired,
    /// Billing issue must be handled before sales recommendations.
    PaymentOrBillingReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons evidence does not indicate a new daycare package opportunity.
pub enum NoOpportunityReason {
    /// Existing package or membership already covers the daycare need.
    AlreadyCovered,
    /// Recent visits are too low to justify a package recommendation.
    NotEnoughAttendanceHistory,
}

#[derive(Debug, Clone, Default)]
/// Deterministic policy that scores daycare package opportunities from evidence.
pub struct Policy;

impl Policy {
    /// Classifies package opportunity evidence into recommend, suppress, or no-opportunity outcomes.
    pub fn classify(&self, evidence: &Evidence) -> Decision {
        if matches!(evidence.eligibility, CareEligibility::BlockedBySafetyReview) {
            return Decision::Suppressed {
                reason: SuppressionReason::SafetyOrCareReviewRequired,
                gate: policy::ReviewGate::BehaviorReview,
            };
        }
        if matches!(evidence.payment_state, PaymentState::NeedsBillingReview) {
            return Decision::Suppressed {
                reason: SuppressionReason::PaymentOrBillingReviewRequired,
                gate: policy::ReviewGate::RefundOrDepositException,
            };
        }
        if matches!(evidence.package_state, PackageState::AlreadyCovered) {
            return Decision::NoOpportunity {
                reason: NoOpportunityReason::AlreadyCovered,
            };
        }
        if evidence.attendance_visits.get() >= 8 {
            Decision::RecommendStaffReview {
                score: OpportunityScore::Strong,
                gate: policy::ReviewGate::CustomerMessageApproval,
            }
        } else {
            Decision::NoOpportunity {
                reason: NoOpportunityReason::NotEnoughAttendanceHistory,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Estimated staff minutes saved by a source-backed daycare package opportunity packet.
pub struct EstimatedLaborMinutes(u16);

impl EstimatedLaborMinutes {
    /// Accepts a positive labor-minute estimate for package review outcome comparison.
    pub const fn try_new(value: u16) -> std::result::Result<Self, LaborMinutesError> {
        if value == 0 {
            return Err(LaborMinutesError::Zero);
        }
        Ok(Self(value))
    }

    /// Returns the minute count used for labor-savings outcome records.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for EstimatedLaborMinutes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_new(u16::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Actual staff minutes spent after daycare package opportunity review.
pub struct ActualLaborMinutes(u16);

impl ActualLaborMinutes {
    /// Accepts a positive actual-minute count for package review outcome measurement.
    pub const fn try_new(value: u16) -> std::result::Result<Self, LaborMinutesError> {
        if value == 0 {
            return Err(LaborMinutesError::Zero);
        }
        Ok(Self(value))
    }

    /// Returns the minute count used for labor-savings outcome records.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ActualLaborMinutes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_new(u16::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Labor-minute validation failures for daycare package opportunity outcomes.
pub enum LaborMinutesError {
    #[error("daycare package opportunity labor minutes must be positive")]
    /// Rejects zero-minute estimates so labor-reduction reports stay measurable.
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Staff disposition for a daycare package opportunity after human review.
pub enum Disposition {
    /// Staff reviewed a package or membership offer draft.
    StaffReviewedOffer,
    /// Staff deferred the recommendation because timing, billing, or customer context was wrong.
    DeferredByStaff,
    /// Staff suppressed the recommendation because source or care evidence was wrong.
    SuppressedOrWrongSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Outcome record proving whether daycare package opportunity review reduced lookup/recommendation labor.
pub struct OutcomeRecord {
    disposition: Disposition,
    before_minutes: EstimatedLaborMinutes,
    actual_minutes: ActualLaborMinutes,
    source_record_refs: Vec<crate::source::RecordRef>,
}

impl OutcomeRecord {
    /// Creates a labor outcome without authorizing package enrollment, billing, provider writes, or customer sends.
    pub fn new(
        disposition: Disposition,
        before_minutes: EstimatedLaborMinutes,
        actual_minutes: ActualLaborMinutes,
        source_record_refs: Vec<crate::source::RecordRef>,
    ) -> Self {
        Self {
            disposition,
            before_minutes,
            actual_minutes,
            source_record_refs,
        }
    }

    /// Returns reviewer disposition for reporting and quality loops.
    pub const fn disposition(&self) -> Disposition {
        self.disposition
    }

    /// Returns the estimated manual lookup minutes before the opportunity packet.
    pub const fn before_minutes(&self) -> EstimatedLaborMinutes {
        self.before_minutes
    }

    /// Returns the actual staff minutes spent after the opportunity packet.
    pub const fn actual_minutes(&self) -> ActualLaborMinutes {
        self.actual_minutes
    }

    /// Returns source records used by the reviewer when measuring labor impact.
    pub fn source_record_refs(&self) -> &[crate::source::RecordRef] {
        &self.source_record_refs
    }

    /// Reports whether outcome measurement remains tied to source evidence.
    pub fn has_source_evidence(&self) -> bool {
        !self.source_record_refs.is_empty()
    }

    /// Computes saved staff minutes without allowing negative labor-savings claims.
    pub const fn minutes_saved(&self) -> u16 {
        self.before_minutes
            .get()
            .saturating_sub(self.actual_minutes.get())
    }
}
