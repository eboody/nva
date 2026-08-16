//! Small SpacetimeDB value columns for review-queue storage rows.
//!
//! These enums are adapter storage values, not domain enums. Codecs convert them
//! explicitly at the boundary before app/domain logic runs.

/// Caller-reported feedback label encoded in review-queue storage.
///
/// These compatibility values do not prove review, completion, suppression, or resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum FeedbackOutcomeColumn {
    /// Caller reported a completed label; cleanup completion is not proven.
    Completed,
    /// Caller reported a deferred label; reviewer action is not proven.
    Deferred,
    /// Caller reported a manager-suppressed label; manager action is not proven.
    SuppressedByManager,
    /// Caller reported a wrong-source label; reviewer action is not proven.
    SourceFactWasWrong,
    /// Caller reported a not-actionable label; reviewer action is not proven.
    NotActionable,
}

impl FeedbackOutcomeColumn {
    /// Every supported persisted outcome, used by exhaustive codec contracts.
    pub const ALL: [Self; 5] = [
        Self::Completed,
        Self::Deferred,
        Self::SuppressedByManager,
        Self::SourceFactWasWrong,
        Self::NotActionable,
    ];
}

/// Caller-reported resolution-status label encoded in review-queue storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum ResolutionStatusColumn {
    /// Open issue.
    Open,
    /// Acknowledged issue.
    Acknowledged,
    /// Ignored issue.
    Ignored,
    /// Repaired issue.
    Repaired,
    /// Superseded issue.
    Superseded,
}

/// Blocked capture reason encoded in review-queue storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum BlockedActionReasonColumn {
    /// Actor id did not resolve.
    ActorNotFound,
    /// Review queue item did not resolve.
    ReviewQueueItemNotFound,
    /// Actor lacked the required review gate.
    ActorLacksReviewGate,
}

/// Stable upstream-system discriminator for source record references.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum SourceSystemColumn {
    /// Represents the `Gingr` semantic case.
    Gingr,
    /// Represents the `Telephony` semantic case.
    Telephony,
    /// Represents the `SmsProvider` semantic case.
    SmsProvider,
    /// Represents the `Email` semantic case.
    Email,
    /// Represents the `WebChat` semantic case.
    WebChat,
    /// Represents the `WebsiteForms` semantic case.
    WebsiteForms,
    /// Represents the `MarketingAutomation` semantic case.
    MarketingAutomation,
    /// Represents the `Crm` semantic case.
    Crm,
    /// Represents the `FinanceAccounting` semantic case.
    FinanceAccounting,
    /// Represents the `WorkforceManagement` semantic case.
    WorkforceManagement,
    /// Represents the `KnowledgeBase` semantic case.
    KnowledgeBase,
    /// Represents the `ProviderOrPms` semantic case.
    ProviderOrPms,
    /// Represents the `BusinessIntelligence` semantic case.
    BusinessIntelligence,
    /// Represents the `LaborScheduling` semantic case.
    LaborScheduling,
    /// Represents the `Timeclock` semantic case.
    Timeclock,
    /// Represents the `Payroll` semantic case.
    Payroll,
    /// Represents the `CapacityInventory` semantic case.
    CapacityInventory,
    /// Represents the `PointOfSale` semantic case.
    PointOfSale,
    /// Represents the `ManualImport` semantic case.
    ManualImport,
}

impl SourceSystemColumn {
    /// Every supported variant value for exhaustive codec contracts.
    pub const ALL: [Self; 19] = [
        Self::Gingr,
        Self::Telephony,
        Self::SmsProvider,
        Self::Email,
        Self::WebChat,
        Self::WebsiteForms,
        Self::MarketingAutomation,
        Self::Crm,
        Self::FinanceAccounting,
        Self::WorkforceManagement,
        Self::KnowledgeBase,
        Self::ProviderOrPms,
        Self::BusinessIntelligence,
        Self::LaborScheduling,
        Self::Timeclock,
        Self::Payroll,
        Self::CapacityInventory,
        Self::PointOfSale,
        Self::ManualImport,
    ];
}

/// Stable review-gate discriminator; unlike a boolean this preserves every gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum ReviewGateColumn {
    /// Represents the `ManagerApproval` semantic case.
    ManagerApproval,
    /// Represents the `MedicalDocumentReview` semantic case.
    MedicalDocumentReview,
    /// Represents the `BehaviorReview` semantic case.
    BehaviorReview,
    /// Represents the `CustomerMessageApproval` semantic case.
    CustomerMessageApproval,
    /// Represents the `RefundOrDepositException` semantic case.
    RefundOrDepositException,
}

impl ReviewGateColumn {
    /// Every supported variant value for exhaustive codec contracts.
    pub const ALL: [Self; 5] = [
        Self::ManagerApproval,
        Self::MedicalDocumentReview,
        Self::BehaviorReview,
        Self::CustomerMessageApproval,
        Self::RefundOrDepositException,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
/// Stable staff disposition column representation used at this boundary.
pub enum StaffDispositionColumn {
    /// Represents the `RecommendForManagerApproval` semantic case.
    RecommendForManagerApproval,
    /// Represents the `CompleteWithoutManagerApproval` semantic case.
    CompleteWithoutManagerApproval,
    /// Represents the `Defer` semantic case.
    Defer,
}

impl StaffDispositionColumn {
    /// Every supported variant value for exhaustive codec contracts.
    pub const ALL: [Self; 3] = [
        Self::RecommendForManagerApproval,
        Self::CompleteWithoutManagerApproval,
        Self::Defer,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
/// Stable manager outcome column representation used at this boundary.
pub enum ManagerOutcomeColumn {
    /// Represents the `Approved` semantic case.
    Approved,
    /// Represents the `Rejected` semantic case.
    Rejected,
    /// Represents the `Deferred` semantic case.
    Deferred,
}

impl ManagerOutcomeColumn {
    /// Every supported variant value for exhaustive codec contracts.
    pub const ALL: [Self; 3] = [Self::Approved, Self::Rejected, Self::Deferred];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
/// Stable blocked action column representation used at this boundary.
pub enum BlockedActionColumn {
    /// Represents the `SendCustomerMessage` semantic case.
    SendCustomerMessage,
    /// Represents the `MutateProviderOrPmsRecord` semantic case.
    MutateProviderOrPmsRecord,
    /// Represents the `ChangeStaffSchedule` semantic case.
    ChangeStaffSchedule,
    /// Represents the `MoveRefundDiscountOrPayment` semantic case.
    MoveRefundDiscountOrPayment,
    /// Represents the `HideOrAutoResolveSourceAmbiguity` semantic case.
    HideOrAutoResolveSourceAmbiguity,
    /// Represents the `ExposeQuarantinedSensitivePayload` semantic case.
    ExposeQuarantinedSensitivePayload,
    /// Represents the `RecordReviewedOutcome` semantic case.
    RecordReviewedOutcome,
    /// Represents the `UnauthorizedQueueWork` semantic case.
    UnauthorizedQueueWork,
    /// Represents the `UnsafeSideEffect` semantic case.
    UnsafeSideEffect,
}

impl BlockedActionColumn {
    /// Every supported variant value for exhaustive codec contracts.
    pub const ALL: [Self; 9] = [
        Self::SendCustomerMessage,
        Self::MutateProviderOrPmsRecord,
        Self::ChangeStaffSchedule,
        Self::MoveRefundDiscountOrPayment,
        Self::HideOrAutoResolveSourceAmbiguity,
        Self::ExposeQuarantinedSensitivePayload,
        Self::RecordReviewedOutcome,
        Self::UnauthorizedQueueWork,
        Self::UnsafeSideEffect,
    ];
}

#[derive(Clone, PartialEq, Eq, spacetimedb::SpacetimeType)]
/// Stable actor ref column representation used at this boundary.
pub enum ActorRefColumn {
    /// Represents the `Customer` semantic case.
    Customer(String),
    /// Represents the `Staff` semantic case.
    Staff(String),
    /// Represents the `Manager` semantic case.
    Manager(String),
    /// Represents the `System` semantic case.
    System,
    /// Represents the `Agent` semantic case.
    Agent(String),
}

impl std::fmt::Debug for ActorRefColumn {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ActorRefColumn([REDACTED])")
    }
}

#[derive(Clone, PartialEq, Eq, spacetimedb::SpacetimeType)]
/// Relationship-checked source record ref column used at this boundary.
pub struct SourceRecordRefColumn {
    /// Stores the system component of this boundary value.
    pub system: SourceSystemColumn,
    /// Stores the record id component of this boundary value.
    pub record_id: String,
}

impl std::fmt::Debug for SourceRecordRefColumn {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SourceRecordRefColumn([REDACTED])")
    }
}

/// Canonical issue ref column used by this module.
pub type IssueRefColumn = String;
/// Canonical recommendation column used by this module.
pub type RecommendationColumn = String;
