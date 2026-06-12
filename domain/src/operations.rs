use bon::Builder;
use chrono::{DateTime, NaiveDate, Utc};
use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{CustomerId, LocationId, PetId, ReservationId, ServiceKind, StaffId};
use crate::workflow::task;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct MetricName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct SnapshotId(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct OperationalObservation(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct OperationalRecommendation(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct TaskCompletionEvidence(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct LeadSourceName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct CampaignName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct ReviewPlatformName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct ReviewId(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResortOperatingDay {
    pub location_id: LocationId,
    pub date: NaiveDate,
    pub snapshot_id: SnapshotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResortDailyBrief {
    pub operating_day: ResortOperatingDay,
    pub sections: Vec<DailyBriefSection>,
    pub recommended_actions: Vec<OperationsAction>,
    pub risks: Vec<OperationsRisk>,
}

impl ResortDailyBrief {
    pub fn has_manager_attention_required(&self) -> bool {
        self.risks
            .iter()
            .any(OperationsRisk::requires_manager_attention)
            || self
                .recommended_actions
                .iter()
                .any(OperationsAction::requires_manager_approval)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DailyBriefSection {
    Occupancy(OccupancySnapshot),
    ArrivalsAndDepartures(ArrivalDepartureSnapshot),
    Labor(LaborSnapshot),
    CustomerFollowUps(Vec<CustomerFollowUp>),
    PetCareWatchlist(Vec<PetCareWatch>),
    RevenueOpportunities(Vec<RevenueOpportunity>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccupancySnapshot {
    pub boarding_capacity: CapacityMetric,
    pub daycare_capacity: CapacityMetric,
    pub grooming_utilization: CapacityMetric,
    pub training_utilization: CapacityMetric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CapacityBooked(u32);

impl CapacityBooked {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct CapacityLimit(u32);

impl CapacityLimit {
    pub const fn try_new(value: u32) -> Result<Self, CapacityLimitError> {
        if value == 0 {
            return Err(CapacityLimitError::ZeroCapacity);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for CapacityLimit {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CapacityLimitError {
    #[error("capacity metrics require an explicit non-zero capacity limit")]
    ZeroCapacity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CapacitySaturationBasisPoints(u32);

impl CapacitySaturationBasisPoints {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ScheduledStaffCount(u16);

impl ScheduledStaffCount {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityMetric {
    booked: CapacityBooked,
    capacity: CapacityLimit,
}

impl CapacityMetric {
    pub const fn new(booked: CapacityBooked, capacity: CapacityLimit) -> Self {
        Self { booked, capacity }
    }

    pub const fn booked(&self) -> CapacityBooked {
        self.booked
    }

    pub const fn capacity(&self) -> CapacityLimit {
        self.capacity
    }

    pub fn saturation_basis_points(&self) -> CapacitySaturationBasisPoints {
        CapacitySaturationBasisPoints::new(
            self.booked.get().saturating_mul(10_000) / self.capacity.get(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrivalDepartureSnapshot {
    pub check_ins: Vec<ReservationId>,
    pub check_outs: Vec<ReservationId>,
    pub late_departure_risk: Vec<ReservationId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaborSnapshot {
    pub scheduled_staff_count: ScheduledStaffCount,
    pub labor_risk: LaborRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaborRisk {
    Understaffed,
    OnPlan,
    Overstaffed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomerFollowUp {
    pub customer_id: CustomerId,
    pub reason: FollowUpReason,
    pub due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowUpReason {
    MissingVaccineProof,
    DepositNotPaid,
    ReservationChangeRequested,
    LeadNeedsResponse,
    PostStayCheckIn,
    ReviewResponseNeeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PetCareWatch {
    pub pet_id: PetId,
    pub reason: PetCareWatchReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetCareWatchReason {
    MedicationDue,
    FeedingException,
    AnxietyOrStressFlag,
    BehaviorReview,
    IncidentFollowUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueOpportunity {
    pub customer_id: Option<CustomerId>,
    pub pet_id: Option<PetId>,
    pub service: ServiceKind,
    pub opportunity: RevenueOpportunityKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevenueOpportunityKind {
    ExitBathAfterBoarding,
    GroomingRebookingDue,
    DaycarePackageCandidate,
    TrainingConsultCandidate,
    HolidayBoardingWaitlistFill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationsRisk {
    CapacityConstraint { service: ServiceKind },
    LaborMismatch { risk: LaborRisk },
    CustomerExperienceRisk { observation: OperationalObservation },
    PetSafetyOrCareRisk { observation: OperationalObservation },
    RevenueLeakage { observation: OperationalObservation },
}

impl OperationsRisk {
    pub fn requires_manager_attention(&self) -> bool {
        matches!(
            self,
            Self::CapacityConstraint { .. }
                | Self::LaborMismatch {
                    risk: LaborRisk::Understaffed
                }
                | Self::CustomerExperienceRisk { .. }
                | Self::PetSafetyOrCareRisk { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationsAction {
    CreateInternalTask {
        recommendation: OperationalRecommendation,
    },
    DraftCustomerMessage {
        customer_id: CustomerId,
        reason: FollowUpReason,
    },
    EscalateToManager {
        reason: OperationalObservation,
    },
    SuggestScheduleReview {
        risk: LaborRisk,
    },
    SuggestRevenueFollowUp {
        opportunity: RevenueOpportunityKind,
    },
}

impl OperationsAction {
    pub fn requires_manager_approval(&self) -> bool {
        matches!(
            self,
            Self::EscalateToManager { .. } | Self::SuggestScheduleReview { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct StaffTask {
    pub location_id: LocationId,
    pub kind: StaffTaskKind,
    pub title: task::Title,
    pub status: StaffTaskStatus,
    pub priority: StaffTaskPriority,
    pub due_at: DateTime<Utc>,
    pub assignment: StaffTaskAssignment,
    pub source: StaffTaskSource,
    pub completion_evidence: Option<TaskCompletionEvidence>,
}

impl StaffTask {
    pub fn requires_manager_attention(&self) -> bool {
        matches!(
            self.status,
            StaffTaskStatus::Blocked | StaffTaskStatus::NeedsManagerReview
        ) || matches!(
            self.priority,
            StaffTaskPriority::High | StaffTaskPriority::Critical
        ) || matches!(
            self.kind,
            StaffTaskKind::IncidentFollowUp { .. }
                | StaffTaskKind::MedicationAdministration { .. }
                | StaffTaskKind::DocumentReview { .. }
        )
    }

    pub fn complete_with(mut self, evidence: TaskCompletionEvidence) -> Self {
        self.status = StaffTaskStatus::Completed;
        self.completion_evidence = Some(evidence);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffTaskKind {
    CheckInPrep {
        reservation_id: ReservationId,
    },
    CheckOutPrep {
        reservation_id: ReservationId,
    },
    Feeding {
        pet_id: PetId,
    },
    MedicationAdministration {
        pet_id: PetId,
    },
    PlaygroupAssessment {
        pet_id: PetId,
    },
    CleaningTurnover {
        reservation_id: ReservationId,
    },
    DailyUpdateDraft {
        reservation_id: ReservationId,
    },
    DocumentReview {
        pet_id: PetId,
    },
    IncidentFollowUp {
        pet_id: PetId,
    },
    CustomerFollowUp {
        customer_id: CustomerId,
        reason: FollowUpReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffTaskStatus {
    Open,
    InProgress,
    Blocked,
    NeedsManagerReview,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StaffTaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffTaskAssignment {
    Unassigned,
    Staff(StaffId),
    Role(StaffRole),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffRole {
    FrontDesk,
    KennelTechnician,
    Groomer,
    Trainer,
    LeadStaff,
    Manager,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffTaskSource {
    Reservation(ReservationId),
    Pet(PetId),
    Customer(CustomerId),
    DailyBrief(SnapshotId),
    WorkflowEvent(crate::workflow::WorkflowEventId),
    StaffCreated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lead {
    pub customer_id: Option<CustomerId>,
    pub source: LeadSource,
    pub intent: LeadIntent,
    pub stage: LeadConversionStage,
    pub requested_service: Option<ServiceKind>,
    pub next_action: LeadNextAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadSource {
    WebsiteForm,
    Phone,
    Sms,
    Email,
    SocialMedia,
    LocalReferral { source_name: LeadSourceName },
    Campaign { name: CampaignName },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadIntent {
    NewCustomerIntake,
    BoardingQuote,
    DaycareTrial,
    GroomingAppointment,
    TrainingConsult,
    ExistingCustomerChange,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadConversionStage {
    New,
    ContactAttempted,
    WaitingOnCustomer,
    MissingRequirements,
    ReadyToBook,
    Converted,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadNextAction {
    DraftReply,
    RequestMissingPetProfile,
    RequestVaccineProof,
    OfferReservationTimes,
    RouteToHuman { reason: OperationalObservation },
    NoAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReputationSignal {
    pub location_id: LocationId,
    pub platform: ReviewPlatformName,
    pub review_id: ReviewId,
    pub sentiment: ReviewSentiment,
    pub themes: Vec<ReviewTheme>,
    pub escalation: ReviewEscalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSentiment {
    Positive,
    Neutral,
    Negative,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewTheme {
    StaffExperience,
    Cleanliness,
    PricingOrBilling,
    BookingExperience,
    GroomingOutcome,
    PetInjuryOrSafety,
    Communication,
    WaitTime,
    Other(OperationalObservation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewEscalation {
    None,
    DraftPublicResponse,
    ManagerReviewRequired,
    SafetyOrLegalReviewRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct PetResortPortfolio {
    pub operator: Operator,
    pub resort_count: ResortCount,
    pub structure: PortfolioStructure,
    pub business_lines: Vec<BusinessLine>,
    pub brands: Vec<PetResortBrand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    NationalVeterinaryAssociates,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortfolioStructure {
    FederatedMultiBrand,
    SingleBrand,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusinessLine {
    GeneralPracticeVeterinaryHospitals,
    PetResorts,
    Equine,
    SpecialtyEmergencyHospitals,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetResortBrand {
    NvaPetResorts,
    PetSuites,
    PoochHotel,
    EliteSuites,
    TheBarkSide,
    WoofdorfAstoria,
    DoggieDistrict,
    Other { name: crate::location::Name },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ResortCount(u16);

impl ResortCount {
    pub const fn try_new(value: u16) -> Result<Self, ResortCountError> {
        if value == 0 {
            return Err(ResortCountError::ZeroResorts);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ResortCount {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u16::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ResortCountError {
    #[error("pet resort portfolios require at least one resort")]
    ZeroResorts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceOffering {
    Boarding {
        accommodation: BoardingAccommodation,
        included_care: Vec<BoardingCareFeature>,
        add_ons: Vec<BoardingAddOn>,
    },
    Daycare {
        format: DaycareFormat,
        eligibility_rules: Vec<DaycareEligibilityRule>,
    },
    Grooming {
        service: GroomingService,
        cadence: GroomingCadence,
    },
    Training {
        program: TrainingProgram,
    },
    RetailPartnerProduct {
        partner: RetailPartner,
        category: RetailProductCategory,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardingAccommodation {
    ClassicSuite,
    LuxurySuite,
    CatCondo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardingCareFeature {
    DailyHousekeeping,
    PottyWalks,
    Bedding,
    PawgressReport,
    FeedingSupport,
    MedicationSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardingAddOn {
    Playtime,
    ExitBath,
    PremiumSuite,
    Grooming,
    TrainingSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DaycareFormat {
    AllDayPlay,
    HalfDayPlay,
    DayBoarding,
    DayPlayPlusRoom,
    CatIndividualPlaytime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DaycareEligibilityRule {
    TemperamentReviewRequired,
    SpayNeuterRequiredForGroupPlay,
    VaccineProofRequired,
    StaffToPetRatioRequired,
}

pub use crate::service::grooming::RebookingCadence as GroomingCadence;
pub use crate::service::grooming::Service as GroomingService;
pub use crate::service::grooming::{CadenceWeeks, GroomingCadenceWeeksError as CadenceWeeksError};

pub use crate::service::training::{
    DurationWeeks as TrainingProgramDurationWeeks,
    DurationWeeksError as TrainingProgramDurationWeeksError, Program as TrainingProgram,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetailPartner {
    VirbacCalmCare,
    PurinaProPlanVeterinarySupplements,
    PurinaEnBoardingDiet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetailProductCategory {
    Supplement,
    InHouseDiet,
    PersonalizedUpsell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct TechnologyEcosystem {
    pub core_portal: CoreOperatingSystem,
    pub data_access: Vec<DataAccessPattern>,
    pub adjacent_systems: Vec<AdjacentSystem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreOperatingSystem {
    Gingr,
    MixedSystems,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataAccessPattern {
    Api,
    Webhook,
    DataExport,
    Warehouse,
    BusinessIntelligenceDashboard,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjacentSystem {
    AvatureRecruiting,
    Ga4,
    Amplitude,
    GoogleTagManager,
    Hris,
    LaborScheduling,
    Payroll,
    MarketingAutomation,
    Ticketing,
    CallCenterTelephony,
    Reviews,
    EmailSmsMarketing,
    BusinessIntelligence,
    DataLake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalPainArea {
    LaborEfficiency,
    CustomerCommunicationLoad,
    ReservationCapacityOptimization,
    DataFragmentation,
    SalesRetentionMarketing,
    TrainingAndStandards,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiUseCase {
    ResortManagerDailyBriefing,
    RegionalOpsExceptionReporting,
    CustomerInboxAndCallDeflection,
    LeadConversion,
    GroomingRebooking,
    PostStayPawgressReportAssistant,
    ReviewReputationTriage,
    SopKnowledgeAssistant,
    DataQualityOpsHygiene,
    IncidentReportDrafting,
    TrainingOnboardingAssistant,
    LapsedCustomerWinback,
    BoardingPreArrivalChecklistAutomation,
    CapacityAlerts,
    LaborRevenueAnomalyDetection,
    WebsiteReservationAssistant,
    VaccinationDocumentCollection,
    DemandForecasting,
    StaffingRecommendations,
    RegionalPerformanceBenchmarking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetResortOperatingTerm {
    PawgressReports,
    BoardingReservations,
    DaycarePackages,
    PetPointsRewards,
    GingrCustomerPortal,
    LeadCaptureAndConversion,
    WebsiteEmailSocialOutreach,
    LocalMarketPlans,
    SalesLaborExpensesCustomerSatisfactionKpis,
    OshaCashHandlingOperationalCompliance,
    TrainingCertificationCompletion,
    ResortLevelEbitdaProfitability,
    GroomingCadence,
    DaycareEligibilityRules,
    GuestExperience,
    TeamMemberEngagementRetention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataQualityIssue {
    MissingPetVaccinationRecords,
    IncompletePetProfiles,
    DuplicateCustomers,
    MissingTemperamentNotes,
    OpenInvoices,
    UnclosedReservations,
    UnusedPackages,
    StaffNotesTooVague,
    InconsistentServiceNamingAcrossSites,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingFunction {
    FrontDesk,
    CallCenter,
    GeneralManagers,
    AssistantGeneralManagers,
    RegionalOperations,
    Grooming,
    Training,
    Marketing,
    InformationTechnology,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffTrainingWorkflow {
    NewHireOnboarding,
    SopLookup,
    IncidentDocumentation,
    PetBehaviorNoteConsistency,
    ManagerCoaching,
    RegulatorySafetyPolicy,
    CustomerComplaintHandling,
    TrainingQuizGeneration,
    ShiftLeadCopilot,
    ShiftSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerCommunicationWorkflow {
    AvailabilityQuestion,
    VaccineRequirementQuestion,
    MultiPetBoardingQuestion,
    GroupPlayEligibilityQuestion,
    DaycareReadinessQuestion,
    AddBathRequest,
    PetUpdateRequest,
    CheckoutTimeQuestion,
    CancelOrChangeReservation,
    LoyaltyPointsQuestion,
    TrainingOptionsQuestion,
    AnxietyOrSpecialHandlingQuestion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapacityConstraintKind {
    RoomOrSuiteAvailability,
    PlayYardAvailability,
    GroomerSlotAvailability,
    TrainerAvailability,
    StaffRatio,
    PetTemperament,
    HolidayPeak,
    CheckInCheckoutBottleneck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationOpportunity {
    DemandForecasting,
    NoShowPrediction,
    DynamicWaitlistFilling,
    CapacityRecommendation,
    AddOnRecommendation,
    HolidayPlanning,
    OverUnderStaffingAlert,
    RevenueOptimizationWithoutCareDegradation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct CoreServiceContracts {
    pub location_id: LocationId,
    pub boarding: crate::service::boarding::Contract,
    pub daycare: crate::service::daycare::Contract,
    pub grooming: grooming::Contract,
    pub training: training::Contract,
    pub retail: retail::Contract,
}

impl CoreServiceContracts {
    pub fn core_services(&self) -> [CoreServiceLine; 5] {
        [
            CoreServiceLine::Boarding,
            CoreServiceLine::Daycare,
            CoreServiceLine::Grooming,
            CoreServiceLine::Training,
            CoreServiceLine::Retail,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreServiceLine {
    Boarding,
    Daycare,
    Grooming,
    Training,
    Retail,
}

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        pub struct $name($primitive);

        impl $name {
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            pub const fn get(self) -> $primitive {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::try_new(<$primitive>::deserialize(deserializer)?)
                    .map_err(serde::de::Error::custom)
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
        pub enum $error {
            #[error($message)]
            Zero,
        }
    };
}

pub mod boarding {
    // Temporary compatibility shim; canonical path is `crate::service::boarding`.
    // Remove after storage/app/test call sites migrate off `domain::operations::boarding`.
    pub use crate::service::boarding::*;
}

pub mod daycare {
    // Temporary compatibility shim; canonical path is `crate::service::daycare`.
    // Remove after storage/app/test call sites migrate off `domain::operations::daycare`.
    pub use crate::service::daycare::*;
}
pub mod grooming {
    // Temporary compatibility shim; canonical path is `crate::service::grooming`.
    // Remove after storage/app/test call sites migrate off `domain::operations::grooming`.
    pub use crate::service::grooming::*;
}

pub mod training {
    // Temporary compatibility shim; canonical path is `crate::service::training`.
    // Remove after storage/app/test call sites migrate off `domain::operations::training`.
    pub use crate::service::training::*;
}
pub mod retail {
    use super::*;
    use crate::policy;

    pub type Result<T> = std::result::Result<T, Error>;

    positive_scalar!(
        UnitCount,
        u32,
        UnitCountError,
        "retail inventory count requires at least one unit"
    );
    positive_scalar!(
        SaleQuantity,
        u32,
        SaleQuantityError,
        "retail sale quantity requires at least one unit"
    );

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct OnHandUnits(u32);
    impl OnHandUnits {
        pub const fn new(value: u32) -> Self {
            Self(value)
        }
        pub const fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ReservedUnits(u32);
    impl ReservedUnits {
        pub const fn new(value: u32) -> Self {
            Self(value)
        }
        pub const fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct AvailableUnits(u32);
    impl AvailableUnits {
        pub const fn new(value: u32) -> Self {
            Self(value)
        }
        pub const fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum Error {
        #[error("retail inventory position cannot reserve more units than are on hand")]
        ReservedUnitsExceedOnHand,
        #[error("retail recommendation rationale is required")]
        MissingRecommendationRationale,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    pub struct Sku(String);
    impl Sku {
        pub fn try_new(value: impl Into<String>) -> std::result::Result<Self, SkuError> {
            let value = value.into().trim().to_owned();
            if value.is_empty() {
                return Err(SkuError::Empty);
            }
            Ok(Self(value))
        }
        pub fn into_inner(self) -> String {
            self.0
        }
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }
    impl<'de> Deserialize<'de> for Sku {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub enum SkuError {
        #[error("retail SKU cannot be empty")]
        Empty,
    }

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
        derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize
        )
    )]
    pub struct ProductName(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 500),
        derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize
        )
    )]
    pub struct RecommendationRationale(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 500),
        derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize
        )
    )]
    pub struct CustomerSafeCopy(String);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Product {
        sku: Sku,
        pub category: super::RetailProductCategory,
    }
    impl Product {
        pub fn new(sku: Sku, category: super::RetailProductCategory) -> Self {
            Self { sku, category }
        }
        pub fn sku(&self) -> &Sku {
            &self.sku
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum OfferingStatus {
        Active,
        Inactive,
        Discontinued,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ProductUsage {
        CustomerSellable,
        InHouseConsumable,
        SellableAndInHouseConsumable,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct LocationOffering {
        pub location_id: LocationId,
        pub product: Product,
        pub status: OfferingStatus,
        pub usage: ProductUsage,
        pub pos: PointOfSalePolicy,
        pub inventory: InventoryPolicy,
        pub reorder: ReorderPolicy,
    }

    impl LocationOffering {
        pub fn can_be_sold_to_customer(&self) -> bool {
            matches!(self.status, OfferingStatus::Active)
                && matches!(
                    self.usage,
                    ProductUsage::CustomerSellable | ProductUsage::SellableAndInHouseConsumable
                )
        }

        pub fn has_available_sale_units(&self, quantity: SaleQuantity) -> bool {
            match self.inventory {
                InventoryPolicy::NotTracked => true,
                InventoryPolicy::Tracked { on_hand, .. } => on_hand.get() >= quantity.get(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct InventoryPosition {
        pub location_id: LocationId,
        sku: Sku,
        on_hand: OnHandUnits,
        reserved: ReservedUnits,
        reorder_at: UnitCount,
    }

    impl InventoryPosition {
        pub fn new(
            location_id: LocationId,
            sku: Sku,
            on_hand: OnHandUnits,
            reserved: ReservedUnits,
            reorder_at: UnitCount,
        ) -> Result<Self> {
            if reserved.get() > on_hand.get() {
                return Err(Error::ReservedUnitsExceedOnHand);
            }
            Ok(Self {
                location_id,
                sku,
                on_hand,
                reserved,
                reorder_at,
            })
        }

        pub fn sku(&self) -> &Sku {
            &self.sku
        }
        pub const fn on_hand(&self) -> OnHandUnits {
            self.on_hand
        }
        pub const fn reserved(&self) -> ReservedUnits {
            self.reserved
        }
        pub const fn reorder_at(&self) -> UnitCount {
            self.reorder_at
        }
        pub const fn available_units(&self) -> AvailableUnits {
            AvailableUnits(self.on_hand.get() - self.reserved.get())
        }
        pub const fn is_at_or_below_reorder_threshold(&self) -> bool {
            self.available_units().get() <= self.reorder_at.get()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PointOfSalePolicy {
        StandaloneSale,
        IntegratedWithReservationCheckout,
        ManagerOnlyComp,
    }

    impl PointOfSalePolicy {
        pub fn evaluate(&self, request: &SaleRequest) -> SaleLineDecision {
            if !request.offering.can_be_sold_to_customer() {
                return SaleLineDecision::Denied {
                    reason: SaleDenialReason::OfferingNotSellable,
                };
            }
            if !request.offering.has_available_sale_units(request.quantity) {
                return SaleLineDecision::Denied {
                    reason: SaleDenialReason::InventoryUnavailable,
                };
            }
            if request.price_adjustment.requires_manager_approval()
                || matches!(self, Self::ManagerOnlyComp)
            {
                return SaleLineDecision::ReviewRequired {
                    reason: SaleReviewReason::PriceException,
                    gate: policy::ReviewGate::ManagerApproval,
                };
            }
            match (self, &request.source) {
                (Self::StandaloneSale, SaleSource::StandaloneStaffSale { .. }) => {
                    SaleLineDecision::DraftAllowed
                }
                (
                    Self::IntegratedWithReservationCheckout,
                    SaleSource::ReservationCheckout { .. },
                ) => SaleLineDecision::ReviewRequired {
                    reason: SaleReviewReason::ReservationCheckoutAttachment,
                    gate: policy::ReviewGate::CustomerMessageApproval,
                },
                _ => SaleLineDecision::Denied {
                    reason: SaleDenialReason::SourceNotAllowed,
                },
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum InventoryPolicy {
        NotTracked,
        Tracked {
            on_hand: UnitCount,
            reorder_at: UnitCount,
        },
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RecommendationRule {
        None,
        AnxietySupportAfterBoarding,
        DietSupportAfterBoarding,
        CoatCareAfterGrooming,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReorderPolicy {
        ManualReview,
        AutoCreateManagerTask,
        VendorManaged,
    }

    impl ReorderPolicy {
        pub fn evaluate(&self, position: &InventoryPosition) -> ReorderDecision {
            if !position.is_at_or_below_reorder_threshold() {
                return ReorderDecision::NoAction;
            }
            match self {
                Self::ManualReview => ReorderDecision::ManagerReviewRequired {
                    reason: ReorderReason::AtOrBelowThreshold,
                    gate: policy::ReviewGate::ManagerApproval,
                },
                Self::AutoCreateManagerTask => ReorderDecision::CreateStaffTask {
                    location_id: position.location_id,
                    sku: position.sku().clone(),
                    reason: ReorderReason::AtOrBelowThreshold,
                    gate: policy::ReviewGate::ManagerApproval,
                },
                Self::VendorManaged => ReorderDecision::VendorManagedNotice {
                    sku: position.sku().clone(),
                    reason: ReorderReason::AtOrBelowThreshold,
                },
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReorderDecision {
        NoAction,
        CreateStaffTask {
            location_id: LocationId,
            sku: Sku,
            reason: ReorderReason,
            gate: policy::ReviewGate,
        },
        ManagerReviewRequired {
            reason: ReorderReason,
            gate: policy::ReviewGate,
        },
        VendorManagedNotice {
            sku: Sku,
            reason: ReorderReason,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReorderReason {
        AtOrBelowThreshold,
        VendorBackorder,
        ForecastedBoardingDietDepletion,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct SaleRequest {
        pub offering: LocationOffering,
        pub quantity: SaleQuantity,
        pub source: SaleSource,
        pub price_adjustment: PriceAdjustment,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum SaleSource {
        StandaloneStaffSale { staff_id: StaffId },
        ReservationCheckout { reservation_id: ReservationId },
        ExternalPosReconciliation,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PriceAdjustment {
        None,
        PolicyDiscount { reason: PriceExceptionReason },
        ManagerComp { reason: PriceExceptionReason },
        RefundOrReversal { reason: PriceExceptionReason },
    }

    impl PriceAdjustment {
        pub const fn requires_manager_approval(self) -> bool {
            !matches!(self, Self::None)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PriceExceptionReason {
        ComplaintRecovery,
        StaffCourtesy,
        RefundCorrection,
        ManagerOverride,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum SaleLineDecision {
        DraftAllowed,
        ReviewRequired {
            reason: SaleReviewReason,
            gate: policy::ReviewGate,
        },
        Denied {
            reason: SaleDenialReason,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum SaleReviewReason {
        PriceException,
        ReservationCheckoutAttachment,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum SaleDenialReason {
        OfferingNotSellable,
        InventoryUnavailable,
        SourceNotAllowed,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct RecommendationCandidate {
        pub customer_id: CustomerId,
        pub pet_id: PetId,
        pub location_id: LocationId,
        pub product: Product,
        pub reason: RecommendationReason,
        pub rationale: RecommendationRationale,
        pub care_sensitivity: CareSensitivity,
        pub inventory: InventoryAvailability,
        pub customer_preference: CustomerRetailPreference,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RecommendationReason {
        AnxietyOrStressSupport,
        BoardingDietContinuity,
        CoatOrSkinCareAfterGrooming,
        PriorPurchaseReplenishment,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CareSensitivity {
        NoKnownCareConflict,
        SupplementOrDietReviewRequired,
        CarePlanConflict,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum InventoryAvailability {
        Available,
        OutOfStock,
        Backordered,
        Unknown,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CustomerRetailPreference {
        AllowsRetailRecommendations,
        OptedOut,
        UnknownRequiresReview,
    }

    #[derive(Debug, Clone, Default)]
    pub struct RecommendationPolicy;

    impl RecommendationPolicy {
        pub fn evaluate(&self, candidate: &RecommendationCandidate) -> RecommendationDecision {
            if matches!(
                candidate.customer_preference,
                CustomerRetailPreference::OptedOut
            ) {
                return RecommendationDecision::Suppressed {
                    reason: RecommendationSuppressionReason::CustomerOptedOut,
                };
            }
            if !matches!(candidate.inventory, InventoryAvailability::Available) {
                return RecommendationDecision::Suppressed {
                    reason: RecommendationSuppressionReason::InventoryUnavailable,
                };
            }
            match candidate.care_sensitivity {
                CareSensitivity::NoKnownCareConflict => {
                    RecommendationDecision::DraftInternalCandidate
                }
                CareSensitivity::SupplementOrDietReviewRequired => {
                    RecommendationDecision::StaffReviewRequired {
                        reason: RecommendationReviewReason::CareSensitiveProduct,
                        gate: policy::ReviewGate::MedicalDocumentReview,
                    }
                }
                CareSensitivity::CarePlanConflict => {
                    RecommendationDecision::ManagerReviewRequired {
                        reason: RecommendationReviewReason::CarePlanConflict,
                        gate: policy::ReviewGate::ManagerApproval,
                    }
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RecommendationDecision {
        DraftInternalCandidate,
        StaffReviewRequired {
            reason: RecommendationReviewReason,
            gate: policy::ReviewGate,
        },
        ManagerReviewRequired {
            reason: RecommendationReviewReason,
            gate: policy::ReviewGate,
        },
        Suppressed {
            reason: RecommendationSuppressionReason,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RecommendationReviewReason {
        CareSensitiveProduct,
        CarePlanConflict,
        UnknownCustomerPreference,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RecommendationSuppressionReason {
        CustomerOptedOut,
        InventoryUnavailable,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CustomerCopyPolicy;

    impl CustomerCopyPolicy {
        pub fn evaluate(&self, copy: &CustomerSafeCopy) -> CustomerCopyDecision {
            let normalized = copy.clone().into_inner().to_lowercase();
            if ["treat", "diagnos", "cure", "prescrib", "medical"]
                .iter()
                .any(|term| normalized.contains(term))
            {
                CustomerCopyDecision::Rejected {
                    reason: CustomerCopyRejectionReason::MedicalClaim,
                    gate: policy::ReviewGate::CustomerMessageApproval,
                }
            } else {
                CustomerCopyDecision::DraftRequiresApproval {
                    gate: policy::ReviewGate::CustomerMessageApproval,
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CustomerCopyDecision {
        DraftRequiresApproval {
            gate: policy::ReviewGate,
        },
        Rejected {
            reason: CustomerCopyRejectionReason,
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CustomerCopyRejectionReason {
        MedicalClaim,
        UnsupportedPromise,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct Contract {
        pub product: Product,
        pub pos: PointOfSalePolicy,
        pub inventory: InventoryPolicy,
        pub recommendation: RecommendationRule,
        pub reorder: ReorderPolicy,
    }

    impl Contract {
        pub fn should_reorder(&self) -> bool {
            matches!(self.inventory, InventoryPolicy::Tracked { on_hand, reorder_at } if on_hand.get() <= reorder_at.get())
        }
        pub fn standard_petsuites() -> Self {
            Self::builder()
                .product(Product::new(
                    Sku::try_new("PETSUITES-RETAIL").unwrap(),
                    super::RetailProductCategory::PersonalizedUpsell,
                ))
                .pos(PointOfSalePolicy::IntegratedWithReservationCheckout)
                .inventory(InventoryPolicy::Tracked {
                    on_hand: UnitCount::try_new(1).unwrap(),
                    reorder_at: UnitCount::try_new(10).unwrap(),
                })
                .recommendation(RecommendationRule::AnxietySupportAfterBoarding)
                .reorder(ReorderPolicy::AutoCreateManagerTask)
                .build()
        }
    }
}
