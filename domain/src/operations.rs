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

pub use crate::service::retail::{
    Partner as RetailPartner, ProductCategory as RetailProductCategory,
};

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
    // Temporary compatibility shim; canonical path is `crate::service::retail`.
    // Remove after storage/app/test call sites migrate off `domain::operations::retail`.
    pub use crate::service::retail::*;
}
