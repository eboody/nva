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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroomingService {
    MiniGroom,
    FullGroom,
    ExitBath,
    FullBath,
    PremiumBath,
    NailTrim,
    NailDremel,
    EarCleaning,
    CoatSkinSpecificProduct,
    FirstTimeGroomingOffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroomingCadence {
    EveryWeeks(CadenceWeeks),
    AsNeeded,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct CadenceWeeks(u8);

impl CadenceWeeks {
    pub const fn try_new(value: u8) -> Result<Self, CadenceWeeksError> {
        if value == 0 {
            return Err(CadenceWeeksError::ZeroWeeks);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for CadenceWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CadenceWeeksError {
    #[error("grooming cadence requires at least one week")]
    ZeroWeeks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainingProgram {
    StayAndStudy {
        duration: TrainingProgramDurationWeeks,
    },
    TutorSession,
    GroupClass,
    PuppyKindergarten,
    PrivateLesson,
    AkcCanineGoodCitizenPrep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct TrainingProgramDurationWeeks(u8);

impl TrainingProgramDurationWeeks {
    pub const fn try_new(value: u8) -> Result<Self, TrainingProgramDurationWeeksError> {
        if value == 0 {
            return Err(TrainingProgramDurationWeeksError::ZeroWeeks);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for TrainingProgramDurationWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TrainingProgramDurationWeeksError {
    #[error("training program duration requires at least one week")]
    ZeroWeeks,
}

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
    use super::*;
    positive_scalar!(
        AppointmentMinutes,
        u16,
        AppointmentMinutesError,
        "grooming appointment estimate requires at least one minute"
    );
    positive_scalar!(
        CadenceWeeks,
        u8,
        GroomingCadenceWeeksError,
        "grooming rebooking cadence requires at least one week"
    );

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CalendarPolicy {
        AnyQualifiedGroomer,
        GroomerSpecific,
        FirstAvailableWithManagerOverride,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum BreedCategory {
        ShortCoat,
        DoubleCoat,
        Doodle,
        Cat,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CoatCondition {
        Maintained,
        ThickUndercoat,
        Matted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct BreedCoatTimeEstimate {
        pub breed: BreedCategory,
        pub coat: CoatCondition,
        minutes: AppointmentMinutes,
    }
    impl BreedCoatTimeEstimate {
        pub const fn new(
            breed: BreedCategory,
            coat: CoatCondition,
            minutes: AppointmentMinutes,
        ) -> Self {
            Self {
                breed,
                coat,
                minutes,
            }
        }
        pub const fn minutes(&self) -> AppointmentMinutes {
            self.minutes
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum NoShowPolicy {
        NoteHistoryOnly,
        RequireDepositForRebooking,
        ManagerReviewBeforeRebooking,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RebookingCadence {
        EveryWeeks(CadenceWeeks),
        AsNeeded,
        GroomerRecommended,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReminderRule {
        OneWeekBefore,
        FortyEightHoursBefore,
        MorningOf,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum HistoryRequirement {
        KeepServiceNotes,
        KeepStyleNotesAndPhotos,
        KeepMedicalHandlingNotes,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    pub struct OrdinaryCadenceWeeks(u8);

    impl OrdinaryCadenceWeeks {
        pub const fn try_new(value: u8) -> std::result::Result<Self, OrdinaryCadenceWeeksError> {
            if value < 2 || value > 8 {
                return Err(OrdinaryCadenceWeeksError::OutsideOrdinaryGroomingBand);
            }
            Ok(Self(value))
        }

        pub const fn get(self) -> u8 {
            self.0
        }
    }

    impl<'de> Deserialize<'de> for OrdinaryCadenceWeeks {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub enum OrdinaryCadenceWeeksError {
        #[error("ordinary grooming rebooking cadence must be between 2 and 8 weeks")]
        OutsideOrdinaryGroomingBand,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct EstimationRequest {
        pub pet_id: PetId,
        pub service: super::GroomingService,
        pub breed: BreedCategory,
        pub coat: CoatCondition,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum EstimateBasis {
        BreedCoatPolicy,
        GroomerHistory,
        LocationDefault,
        ProviderDefault,
        ManualStaffOverride,
        AiSuggestedPendingReview,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum EstimateConfidence {
        High,
        Medium,
        Low,
        UnknownRequiresReview,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReviewRequirement {
        None,
        StaffReview,
        GroomerReview,
        ManagerReview,
        CareReview,
    }

    impl ReviewRequirement {
        pub const fn calendar_execution_gate(self) -> Option<crate::policy::ReviewGate> {
            match self {
                Self::None => None,
                Self::StaffReview | Self::GroomerReview | Self::ManagerReview => {
                    Some(crate::policy::ReviewGate::ManagerApproval)
                }
                Self::CareReview => Some(crate::policy::ReviewGate::MedicalDocumentReview),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DurationEstimate {
        minutes: AppointmentMinutes,
        basis: EstimateBasis,
        confidence: EstimateConfidence,
        review: ReviewRequirement,
    }

    impl DurationEstimate {
        pub const fn new(
            minutes: AppointmentMinutes,
            basis: EstimateBasis,
            confidence: EstimateConfidence,
            review: ReviewRequirement,
        ) -> Self {
            Self {
                minutes,
                basis,
                confidence,
                review,
            }
        }

        pub const fn minutes(&self) -> AppointmentMinutes {
            self.minutes
        }

        pub const fn basis(&self) -> EstimateBasis {
            self.basis
        }

        pub const fn confidence(&self) -> EstimateConfidence {
            self.confidence
        }

        pub const fn review(&self) -> ReviewRequirement {
            self.review
        }

        pub const fn calendar_execution_gate(&self) -> Option<crate::policy::ReviewGate> {
            self.review.calendar_execution_gate()
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct EstimationPolicy;

    impl EstimationPolicy {
        pub fn estimate(
            &self,
            request: EstimationRequest,
            history: &[history::ServiceHistoryEntry],
            contract: &Contract,
        ) -> DurationEstimate {
            if let Some(entry) = history
                .iter()
                .rev()
                .find(|entry| entry.pet_id == request.pet_id && entry.duration().is_some())
            {
                return DurationEstimate::new(
                    entry.duration().expect("checked above"),
                    EstimateBasis::GroomerHistory,
                    EstimateConfidence::Medium,
                    if entry.requires_review() {
                        ReviewRequirement::GroomerReview
                    } else {
                        ReviewRequirement::None
                    },
                );
            }

            let minutes = contract
                .time_estimates
                .iter()
                .find(|estimate| estimate.breed == request.breed && estimate.coat == request.coat)
                .or_else(|| {
                    contract
                        .time_estimates
                        .iter()
                        .find(|estimate| estimate.breed == request.breed)
                })
                .map(BreedCoatTimeEstimate::minutes)
                .unwrap_or_else(|| {
                    AppointmentMinutes::try_new(60).expect("default estimate is positive")
                });

            let review = match request.coat {
                CoatCondition::Matted => ReviewRequirement::GroomerReview,
                CoatCondition::Maintained | CoatCondition::ThickUndercoat => {
                    ReviewRequirement::None
                }
            };
            let confidence = if matches!(review, ReviewRequirement::None) {
                EstimateConfidence::High
            } else {
                EstimateConfidence::Medium
            };

            DurationEstimate::new(minutes, EstimateBasis::BreedCoatPolicy, confidence, review)
        }
    }

    pub mod no_show {
        use super::*;

        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct NoShowCount(u16);

        impl NoShowCount {
            pub const fn try_new(
                value: u16,
            ) -> std::result::Result<Self, std::convert::Infallible> {
                Ok(Self(value))
            }

            pub const fn get(self) -> u16 {
                self.0
            }
        }

        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct LateCancelCount(u16);

        impl LateCancelCount {
            pub const fn try_new(
                value: u16,
            ) -> std::result::Result<Self, std::convert::Infallible> {
                Ok(Self(value))
            }

            pub const fn get(self) -> u16 {
                self.0
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub struct History {
            pub no_shows: NoShowCount,
            pub late_cancels: LateCancelCount,
        }

        impl History {
            pub const fn new(no_shows: NoShowCount, late_cancels: LateCancelCount) -> Self {
                Self {
                    no_shows,
                    late_cancels,
                }
            }

            pub const fn repeat_behavior_count(&self) -> u16 {
                self.no_shows.get().saturating_add(self.late_cancels.get())
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Decision {
            ClearToRebook,
            DepositRequired { gate: crate::policy::ReviewGate },
            ManagerReviewRequired { gate: crate::policy::ReviewGate },
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Evaluation {
            pub customer_id: CustomerId,
            pub pet_id: PetId,
            pub history: History,
        }

        #[derive(Debug, Clone)]
        pub struct Policy {
            rule: NoShowPolicy,
        }

        impl Policy {
            pub const fn new(rule: NoShowPolicy) -> Self {
                Self { rule }
            }

            pub fn evaluate(
                &self,
                customer_id: CustomerId,
                pet_id: PetId,
                history: History,
            ) -> Decision {
                let _evaluation = Evaluation {
                    customer_id,
                    pet_id,
                    history,
                };
                match self.rule {
                    NoShowPolicy::NoteHistoryOnly => Decision::ClearToRebook,
                    NoShowPolicy::RequireDepositForRebooking
                        if history.repeat_behavior_count() > 0 =>
                    {
                        Decision::DepositRequired {
                            gate: crate::policy::ReviewGate::RefundOrDepositException,
                        }
                    }
                    NoShowPolicy::RequireDepositForRebooking => Decision::ClearToRebook,
                    NoShowPolicy::ManagerReviewBeforeRebooking => Decision::ManagerReviewRequired {
                        gate: crate::policy::ReviewGate::ManagerApproval,
                    },
                }
            }
        }
    }

    pub mod history {
        use super::*;

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
        pub struct StyleNote(String);

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum CareReference {
            SensitiveSkinProduct,
            MedicatedProductRequiresReview,
            HandlingOrMedicalConcern,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ServiceOutcome {
            Completed,
            NoShow,
            LateCancelled,
            NeedsFollowUp,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ApprovalState {
            Draft,
            ReviewRequired { gate: crate::policy::ReviewGate },
            ApprovedByGroomer { groomer_id: StaffId },
            Rejected { gate: crate::policy::ReviewGate },
        }

        impl ApprovalState {
            pub const fn requires_review(&self) -> bool {
                matches!(self, Self::Draft | Self::ReviewRequired { .. })
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
        pub struct ServiceHistoryEntry {
            pub pet_id: PetId,
            pub location_id: LocationId,
            pub service: super::super::GroomingService,
            pub completed_on: NaiveDate,
            pub outcome: ServiceOutcome,
            pub approval: ApprovalState,
            #[builder(default)]
            style_notes: Vec<StyleNote>,
            #[builder(default)]
            care_refs: Vec<CareReference>,
            duration: Option<AppointmentMinutes>,
        }

        impl ServiceHistoryEntry {
            pub fn style_notes(&self) -> &[StyleNote] {
                &self.style_notes
            }

            pub fn care_refs(&self) -> &[CareReference] {
                &self.care_refs
            }

            pub const fn duration(&self) -> Option<AppointmentMinutes> {
                self.duration
            }

            pub const fn requires_review(&self) -> bool {
                self.approval.requires_review() || !self.care_refs.is_empty()
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RebookingStatus {
        DueLater,
        DueNow,
        Overdue,
        NeedsGroomerRecommendation,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RebookingRationale {
        LastCompletedServiceCadence,
        NoCompletedHistory,
        GroomerRecommendedCadenceRequired,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct RebookingRecommendation {
        pub pet_id: PetId,
        pub due_on: Option<NaiveDate>,
        pub status: RebookingStatus,
        pub rationale: RebookingRationale,
    }

    #[derive(Debug, Clone, Default)]
    pub struct RebookingPolicy;

    impl RebookingPolicy {
        pub fn recommend_from_history(
            &self,
            pet_id: PetId,
            history: &[history::ServiceHistoryEntry],
            cadence: RebookingCadence,
            today: NaiveDate,
        ) -> RebookingRecommendation {
            let Some(last_completed) = history
                .iter()
                .filter(|entry| entry.pet_id == pet_id)
                .filter(|entry| matches!(entry.outcome, history::ServiceOutcome::Completed))
                .max_by_key(|entry| entry.completed_on)
            else {
                return RebookingRecommendation {
                    pet_id,
                    due_on: None,
                    status: RebookingStatus::NeedsGroomerRecommendation,
                    rationale: RebookingRationale::NoCompletedHistory,
                };
            };

            let RebookingCadence::EveryWeeks(weeks) = cadence else {
                return RebookingRecommendation {
                    pet_id,
                    due_on: None,
                    status: RebookingStatus::NeedsGroomerRecommendation,
                    rationale: RebookingRationale::GroomerRecommendedCadenceRequired,
                };
            };

            let due_on = last_completed
                .completed_on
                .checked_add_days(chrono::Days::new(u64::from(weeks.get()) * 7))
                .expect("bounded grooming cadence should fit chrono date range");
            let status = if today > due_on {
                RebookingStatus::Overdue
            } else if today == due_on {
                RebookingStatus::DueNow
            } else {
                RebookingStatus::DueLater
            };

            RebookingRecommendation {
                pet_id,
                due_on: Some(due_on),
                status,
                rationale: RebookingRationale::LastCompletedServiceCadence,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReminderKind {
        AppointmentConfirmation,
        PrepInstructions,
        MorningOf,
        RebookingDue,
        LapsedCadenceWinback,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CommunicationConsent {
        Granted,
        NotGranted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ReminderSendBoundary {
        DraftRequiresApproval,
        ReadyForApprovedSend,
        SuppressedUntilConsent,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ReminderPlan {
        pub customer_id: CustomerId,
        pub kind: ReminderKind,
        boundary: ReminderSendBoundary,
    }

    impl ReminderPlan {
        pub const fn send_boundary(&self) -> ReminderSendBoundary {
            self.boundary
        }

        pub const fn customer_message_gate(&self) -> Option<crate::policy::ReviewGate> {
            match self.boundary {
                ReminderSendBoundary::DraftRequiresApproval => {
                    Some(crate::policy::ReviewGate::CustomerMessageApproval)
                }
                ReminderSendBoundary::ReadyForApprovedSend
                | ReminderSendBoundary::SuppressedUntilConsent => None,
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct ReminderPolicy;

    impl ReminderPolicy {
        pub const fn plan(
            &self,
            customer_id: CustomerId,
            kind: ReminderKind,
            consent: CommunicationConsent,
        ) -> ReminderPlan {
            let boundary = match consent {
                CommunicationConsent::Granted => ReminderSendBoundary::DraftRequiresApproval,
                CommunicationConsent::NotGranted => ReminderSendBoundary::SuppressedUntilConsent,
            };
            ReminderPlan {
                customer_id,
                kind,
                boundary,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct Contract {
        pub calendar: CalendarPolicy,
        #[builder(default)]
        pub time_estimates: Vec<BreedCoatTimeEstimate>,
        pub no_show: NoShowPolicy,
        pub rebooking: RebookingCadence,
        #[builder(default)]
        pub reminders: Vec<ReminderRule>,
        pub history: HistoryRequirement,
    }

    impl Contract {
        pub fn requires_deposit_after_no_show(&self) -> bool {
            matches!(
                self.no_show,
                NoShowPolicy::RequireDepositForRebooking
                    | NoShowPolicy::ManagerReviewBeforeRebooking
            )
        }
        pub fn standard_petsuites() -> Self {
            Self::builder()
                .calendar(CalendarPolicy::GroomerSpecific)
                .time_estimates(vec![BreedCoatTimeEstimate::new(
                    BreedCategory::Doodle,
                    CoatCondition::Matted,
                    AppointmentMinutes::try_new(180).unwrap(),
                )])
                .no_show(NoShowPolicy::RequireDepositForRebooking)
                .rebooking(RebookingCadence::EveryWeeks(
                    CadenceWeeks::try_new(6).unwrap(),
                ))
                .reminders(vec![
                    ReminderRule::FortyEightHoursBefore,
                    ReminderRule::MorningOf,
                ])
                .history(HistoryRequirement::KeepStyleNotesAndPhotos)
                .build()
        }
    }
}

pub mod training {
    use super::*;
    use crate::policy;

    positive_scalar!(
        DurationWeeks,
        u8,
        DurationWeeksError,
        "training program duration requires at least one week"
    );
    positive_scalar!(
        SessionCount,
        u16,
        SessionCountError,
        "training package requires at least one session"
    );

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
    pub struct EnrollmentId(String);

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
    pub struct TrainingSessionId(String);

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
    pub struct SessionRef(String);

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
    pub struct ProgressReportId(String);

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
    pub struct EvidenceId(String);

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
    pub struct MilestoneId(String);

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
    pub struct OutcomeDocumentationId(String);

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
    pub struct ProgressNote(String);

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum Error {
        #[error("training progress report requires evidence before it can be reviewed")]
        ProgressEvidenceRequired,
        #[error("training outcome claim requires evidence for achieved/readiness claims")]
        OutcomeEvidenceRequired,
        #[error("training outcome documentation requires at least one claim")]
        OutcomeClaimRequired,
        #[error("training package policy does not define a reusable session balance")]
        PackageHasNoReusableBalance,
    }

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ProgramDuration {
        SingleSession,
        Weeks(DurationWeeks),
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CurriculumUnit {
        PuppyManners,
        LooseLeashWalking,
        Recall,
        ConfidenceBuilding,
        CanineGoodCitizenPrep,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ProgressTracking {
        AttendanceOnly,
        SessionNotesAndMilestones,
        TrainerScorecard,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Outcome {
        BasicManners,
        ReducedReactivity,
        CanineGoodCitizenReadiness,
        OwnerHandlingPlan,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TrainerAvailability {
        AnyCertifiedTrainer,
        NamedTrainerRequired,
        WaitlistUntilTrainerAvailable,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PackagePolicy {
        PayPerSession,
        MultiSessionPackage { sessions: SessionCount },
        BoardAndTrainBundle,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum FollowUpCadence {
        None,
        AfterEachSession,
        AfterProgramCompletion,
        ThirtyDaysAfterCompletion,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum EnrollmentReadiness {
        Ready,
        TrainerReviewRequired { gate: policy::ReviewGate },
        BehaviorOrCareReviewRequired { gate: policy::ReviewGate },
        PackageOrPaymentReviewRequired { gate: policy::ReviewGate },
    }

    impl EnrollmentReadiness {
        pub fn blocking_gate(&self) -> Option<policy::ReviewGate> {
            match self {
                Self::Ready => None,
                Self::TrainerReviewRequired { gate }
                | Self::BehaviorOrCareReviewRequired { gate }
                | Self::PackageOrPaymentReviewRequired { gate } => Some(gate.clone()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TrainerRequirement {
        AnyCertifiedTrainer,
        NamedTrainer { trainer_id: StaffId },
        ProgramQualified { program: super::TrainingProgram },
    }

    impl TrainerRequirement {
        pub const fn requires_named_trainer(&self) -> bool {
            matches!(self, Self::NamedTrainer { .. })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum MilestoneStatus {
        NotStarted,
        Introduced,
        Practicing,
        Generalized,
        Completed,
        DeferredNeedsTrainerNote,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct CurriculumProgress {
        pub milestone_id: MilestoneId,
        pub status: MilestoneStatus,
    }

    impl CurriculumProgress {
        pub const fn new(milestone_id: MilestoneId, status: MilestoneStatus) -> Self {
            Self {
                milestone_id,
                status,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ProgressEvidence {
        TrainerNote {
            evidence_id: EvidenceId,
            note: ProgressNote,
        },
        MilestoneObserved {
            evidence_id: EvidenceId,
            milestone_id: MilestoneId,
            status: MilestoneStatus,
        },
        SessionCompleted {
            evidence_id: EvidenceId,
            session_id: TrainingSessionId,
        },
        OutcomeCandidate {
            evidence_id: EvidenceId,
            outcome: Outcome,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ApprovalState {
        Draft,
        TrainerApproved {
            trainer_id: StaffId,
        },
        ManagerApproved {
            manager_id: crate::entities::ManagerId,
        },
        Rejected {
            gate: policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum OutcomeReviewState {
        Draft,
        TrainerApproved { trainer_id: StaffId },
        ApprovedForMemberFacingUse { approved_by: StaffId },
        Rejected { gate: policy::ReviewGate },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum MemberFacingBoundary {
        InternalOnly,
        DraftRequiresApproval { gate: policy::ReviewGate },
        ApprovedForMemberFacingUse,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct SessionBalance(u16);

    impl SessionBalance {
        pub const fn new(value: u16) -> Self {
            Self(value)
        }
        pub const fn get(self) -> u16 {
            self.0
        }
        pub const fn remaining(self) -> Self {
            self
        }
        pub const fn reserve_one(self) -> Self {
            Self(self.0.saturating_sub(1))
        }
    }

    pub mod availability {
        use super::*;

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum CapacityDecision {
            Available,
            Unavailable,
            UnknownRequiresReview,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
        pub struct Request {
            pub enrollment_id: EnrollmentId,
            pub pet_id: PetId,
            pub program: super::super::TrainingProgram,
            pub requirement: TrainerRequirement,
            pub capacity: CapacityDecision,
            pub readiness: EnrollmentReadiness,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Decision {
            AssignmentDrafted,
            Waitlist {
                reason: WaitlistReason,
                gate: policy::ReviewGate,
            },
            ReviewRequired {
                reason: ReviewReason,
                gate: policy::ReviewGate,
            },
        }

        impl Decision {
            pub fn provider_mutation_gate(&self) -> Option<policy::ReviewGate> {
                match self {
                    Self::AssignmentDrafted => None,
                    Self::Waitlist { gate, .. } | Self::ReviewRequired { gate, .. } => {
                        Some(gate.clone())
                    }
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum WaitlistReason {
            RequestedTrainerUnavailable,
            CapacitySnapshotUnavailable,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ReviewReason {
            EnrollmentNotReady,
            CapacityUnknown,
        }

        #[derive(Debug, Clone, Default)]
        pub struct Policy;

        impl Policy {
            pub fn evaluate(&self, request: &Request) -> Decision {
                if let Some(gate) = request.readiness.blocking_gate() {
                    return Decision::ReviewRequired {
                        reason: ReviewReason::EnrollmentNotReady,
                        gate,
                    };
                }
                match request.capacity {
                    CapacityDecision::Available => Decision::AssignmentDrafted,
                    CapacityDecision::Unavailable => Decision::Waitlist {
                        reason: if request.requirement.requires_named_trainer() {
                            WaitlistReason::RequestedTrainerUnavailable
                        } else {
                            WaitlistReason::CapacitySnapshotUnavailable
                        },
                        gate: policy::ReviewGate::ManagerApproval,
                    },
                    CapacityDecision::UnknownRequiresReview => Decision::ReviewRequired {
                        reason: ReviewReason::CapacityUnknown,
                        gate: policy::ReviewGate::ManagerApproval,
                    },
                }
            }
        }
    }

    pub mod progress {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Report {
            pub report_id: ProgressReportId,
            pub enrollment_id: EnrollmentId,
            pub session_ref: SessionRef,
            evidence: Vec<ProgressEvidence>,
            milestones: Vec<CurriculumProgress>,
            approval: ApprovalState,
        }

        impl Report {
            pub fn builder() -> ReportBuilder {
                ReportBuilder::default()
            }
            pub fn has_evidence(&self) -> bool {
                !self.evidence.is_empty()
            }
            pub fn milestones(&self) -> &[CurriculumProgress] {
                &self.milestones
            }
            pub fn approval(&self) -> &ApprovalState {
                &self.approval
            }
            pub fn parent_facing_boundary(&self) -> MemberFacingBoundary {
                match &self.approval {
                    ApprovalState::Draft | ApprovalState::TrainerApproved { .. } => {
                        MemberFacingBoundary::DraftRequiresApproval {
                            gate: policy::ReviewGate::CustomerMessageApproval,
                        }
                    }
                    ApprovalState::ManagerApproved { .. } => {
                        MemberFacingBoundary::ApprovedForMemberFacingUse
                    }
                    ApprovalState::Rejected { .. } => MemberFacingBoundary::InternalOnly,
                }
            }
        }

        #[derive(Default)]
        pub struct ReportBuilder {
            report_id: Option<ProgressReportId>,
            enrollment_id: Option<EnrollmentId>,
            session_ref: Option<SessionRef>,
            evidence: Vec<ProgressEvidence>,
            milestones: Vec<CurriculumProgress>,
            approval: Option<ApprovalState>,
        }

        impl ReportBuilder {
            pub fn report_id(mut self, value: ProgressReportId) -> Self {
                self.report_id = Some(value);
                self
            }
            pub fn enrollment_id(mut self, value: EnrollmentId) -> Self {
                self.enrollment_id = Some(value);
                self
            }
            pub fn session_ref(mut self, value: SessionRef) -> Self {
                self.session_ref = Some(value);
                self
            }
            pub fn evidence(mut self, value: Vec<ProgressEvidence>) -> Self {
                self.evidence = value;
                self
            }
            pub fn milestones(mut self, value: Vec<CurriculumProgress>) -> Self {
                self.milestones = value;
                self
            }
            pub fn approval(mut self, value: ApprovalState) -> Self {
                self.approval = Some(value);
                self
            }
            pub fn build(self) -> Result<Report> {
                if self.evidence.is_empty() {
                    return Err(Error::ProgressEvidenceRequired);
                }
                Ok(Report {
                    report_id: self.report_id.expect("report_id is required"),
                    enrollment_id: self.enrollment_id.expect("enrollment_id is required"),
                    session_ref: self.session_ref.expect("session_ref is required"),
                    evidence: self.evidence,
                    milestones: self.milestones,
                    approval: self.approval.unwrap_or(ApprovalState::Draft),
                })
            }
        }
    }

    pub mod outcome {
        use super::*;

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ClaimStatus {
            Achieved,
            Readiness,
            Deferred,
            NotAssessed,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Claim {
            pub outcome: Outcome,
            pub status: ClaimStatus,
            evidence: Vec<EvidenceId>,
            milestones: Vec<MilestoneId>,
        }

        impl Claim {
            pub fn new(
                outcome: Outcome,
                status: ClaimStatus,
                evidence: Vec<EvidenceId>,
                milestones: Vec<MilestoneId>,
            ) -> Result<Self> {
                if matches!(status, ClaimStatus::Achieved | ClaimStatus::Readiness)
                    && evidence.is_empty()
                {
                    return Err(Error::OutcomeEvidenceRequired);
                }
                Ok(Self {
                    outcome,
                    status,
                    evidence,
                    milestones,
                })
            }
            pub fn evidence(&self) -> &[EvidenceId] {
                &self.evidence
            }
            pub fn milestones(&self) -> &[MilestoneId] {
                &self.milestones
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Documentation {
            pub documentation_id: OutcomeDocumentationId,
            pub enrollment_id: EnrollmentId,
            pub pet_id: PetId,
            pub location_id: LocationId,
            claims: Vec<Claim>,
            review: OutcomeReviewState,
        }

        impl Documentation {
            pub fn builder() -> DocumentationBuilder {
                DocumentationBuilder::default()
            }
            pub fn claims(&self) -> &[Claim] {
                &self.claims
            }
            pub fn review(&self) -> &OutcomeReviewState {
                &self.review
            }
            pub fn member_facing_boundary(&self) -> MemberFacingBoundary {
                match &self.review {
                    OutcomeReviewState::ApprovedForMemberFacingUse { .. } => {
                        MemberFacingBoundary::ApprovedForMemberFacingUse
                    }
                    OutcomeReviewState::Draft | OutcomeReviewState::TrainerApproved { .. } => {
                        MemberFacingBoundary::DraftRequiresApproval {
                            gate: policy::ReviewGate::CustomerMessageApproval,
                        }
                    }
                    OutcomeReviewState::Rejected { .. } => MemberFacingBoundary::InternalOnly,
                }
            }
        }

        #[derive(Default)]
        pub struct DocumentationBuilder {
            documentation_id: Option<OutcomeDocumentationId>,
            enrollment_id: Option<EnrollmentId>,
            pet_id: Option<PetId>,
            location_id: Option<LocationId>,
            claims: Vec<Claim>,
            review: Option<OutcomeReviewState>,
        }

        impl DocumentationBuilder {
            pub fn documentation_id(mut self, value: OutcomeDocumentationId) -> Self {
                self.documentation_id = Some(value);
                self
            }
            pub fn enrollment_id(mut self, value: EnrollmentId) -> Self {
                self.enrollment_id = Some(value);
                self
            }
            pub fn pet_id(mut self, value: PetId) -> Self {
                self.pet_id = Some(value);
                self
            }
            pub fn location_id(mut self, value: LocationId) -> Self {
                self.location_id = Some(value);
                self
            }
            pub fn claims(mut self, value: Vec<Claim>) -> Self {
                self.claims = value;
                self
            }
            pub fn review(mut self, value: OutcomeReviewState) -> Self {
                self.review = Some(value);
                self
            }
            pub fn build(self) -> Result<Documentation> {
                if self.claims.is_empty() {
                    return Err(Error::OutcomeClaimRequired);
                }
                Ok(Documentation {
                    documentation_id: self.documentation_id.expect("documentation_id is required"),
                    enrollment_id: self.enrollment_id.expect("enrollment_id is required"),
                    pet_id: self.pet_id.expect("pet_id is required"),
                    location_id: self.location_id.expect("location_id is required"),
                    claims: self.claims,
                    review: self.review.unwrap_or(OutcomeReviewState::Draft),
                })
            }
        }
    }

    pub mod package {
        use super::*;

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
        pub struct Id(String);

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum LedgerEntry {
            Purchased { sessions: SessionCount },
            Reserved { session_id: TrainingSessionId },
            Consumed { session_id: TrainingSessionId },
            Released { session_id: TrainingSessionId },
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Ledger {
            package_id: Id,
            pub customer_id: CustomerId,
            pub pet_id: PetId,
            policy: PackagePolicy,
            entries: Vec<LedgerEntry>,
        }

        impl Ledger {
            pub fn new(
                package_id: Id,
                customer_id: CustomerId,
                pet_id: PetId,
                policy: PackagePolicy,
                entries: Vec<LedgerEntry>,
            ) -> Result<Self> {
                if !matches!(policy, PackagePolicy::MultiSessionPackage { .. }) {
                    return Err(Error::PackageHasNoReusableBalance);
                }
                Ok(Self {
                    package_id,
                    customer_id,
                    pet_id,
                    policy,
                    entries,
                })
            }
            pub fn package_id(&self) -> &Id {
                &self.package_id
            }
            pub fn entries(&self) -> &[LedgerEntry] {
                &self.entries
            }
            pub fn balance(&self) -> SessionBalance {
                let PackagePolicy::MultiSessionPackage { sessions } = self.policy else {
                    return SessionBalance::new(0);
                };
                let used = self.entries.iter().fold(0u16, |used, entry| match entry {
                    LedgerEntry::Reserved { .. } | LedgerEntry::Consumed { .. } => {
                        used.saturating_add(1)
                    }
                    LedgerEntry::Released { .. } => used.saturating_sub(1),
                    LedgerEntry::Purchased { .. } => used,
                });
                SessionBalance::new(sessions.get().saturating_sub(used))
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum UsageDecision {
            ReserveNextSession {
                package_id: Id,
                remaining_after_reservation: SessionBalance,
            },
            NoRemainingSessions {
                package_id: Id,
                gate: policy::ReviewGate,
            },
            ReconciliationRequired {
                package_id: Id,
                gate: policy::ReviewGate,
            },
        }

        #[derive(Debug, Clone, Default)]
        pub struct Policy;

        impl Policy {
            pub fn decide_usage(&self, ledger: &Ledger) -> UsageDecision {
                let balance = ledger.balance();
                if balance.get() == 0 {
                    UsageDecision::NoRemainingSessions {
                        package_id: ledger.package_id().clone(),
                        gate: policy::ReviewGate::RefundOrDepositException,
                    }
                } else {
                    UsageDecision::ReserveNextSession {
                        package_id: ledger.package_id().clone(),
                        remaining_after_reservation: balance.reserve_one(),
                    }
                }
            }
        }
    }

    pub mod follow_up {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Trigger {
            SessionCompleted { session_id: TrainingSessionId },
            ProgramCompleted { enrollment_id: EnrollmentId },
            LaterCadenceCheckpoint { enrollment_id: EnrollmentId },
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Purpose {
            ProgressUpdate,
            HomeworkCoaching,
            ProgramCompletionSummary,
            ReEnrollmentPrompt,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum EvidenceReadiness {
            ProgressAndHomeworkReady,
            NeedsTrainerEvidence,
            OutcomeDisputedOrAmbiguous,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum State {
            NotDue,
            TrainerEvidenceRequired { gate: policy::ReviewGate },
            DraftRequiresApproval { gate: policy::ReviewGate },
            Suppressed,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Plan {
            pub trigger: Trigger,
            purpose: Purpose,
            state: State,
        }

        impl Plan {
            pub const fn purpose(&self) -> Purpose {
                self.purpose
            }
            pub fn state(&self) -> State {
                self.state.clone()
            }
        }

        #[derive(Debug, Clone, Default)]
        pub struct Policy;

        impl Policy {
            pub const fn plan(
                &self,
                trigger: Trigger,
                cadence: FollowUpCadence,
                evidence: EvidenceReadiness,
            ) -> Plan {
                let purpose = match trigger {
                    Trigger::SessionCompleted { .. } => Purpose::ProgressUpdate,
                    Trigger::ProgramCompleted { .. } => Purpose::ProgramCompletionSummary,
                    Trigger::LaterCadenceCheckpoint { .. } => Purpose::ReEnrollmentPrompt,
                };
                let cadence_matches = matches!(
                    (&trigger, cadence),
                    (
                        Trigger::SessionCompleted { .. },
                        FollowUpCadence::AfterEachSession
                    ) | (
                        Trigger::ProgramCompleted { .. },
                        FollowUpCadence::AfterProgramCompletion
                    ) | (
                        Trigger::LaterCadenceCheckpoint { .. },
                        FollowUpCadence::ThirtyDaysAfterCompletion
                    )
                );
                let state = if !cadence_matches || matches!(cadence, FollowUpCadence::None) {
                    State::NotDue
                } else {
                    match evidence {
                        EvidenceReadiness::ProgressAndHomeworkReady => {
                            State::DraftRequiresApproval {
                                gate: policy::ReviewGate::CustomerMessageApproval,
                            }
                        }
                        EvidenceReadiness::NeedsTrainerEvidence => State::TrainerEvidenceRequired {
                            gate: policy::ReviewGate::ManagerApproval,
                        },
                        EvidenceReadiness::OutcomeDisputedOrAmbiguous => {
                            State::TrainerEvidenceRequired {
                                gate: policy::ReviewGate::ManagerApproval,
                            }
                        }
                    }
                };
                Plan {
                    trigger,
                    purpose,
                    state,
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct Contract {
        pub program_duration: ProgramDuration,
        #[builder(default)]
        pub curriculum: Vec<CurriculumUnit>,
        pub progress: ProgressTracking,
        #[builder(default)]
        pub outcomes: Vec<Outcome>,
        pub trainer_availability: TrainerAvailability,
        pub package: PackagePolicy,
        pub follow_up: FollowUpCadence,
    }

    impl Contract {
        pub fn requires_named_trainer(&self) -> bool {
            matches!(
                self.trainer_availability,
                TrainerAvailability::NamedTrainerRequired
                    | TrainerAvailability::WaitlistUntilTrainerAvailable
            )
        }
        pub fn has_outcome(&self, outcome: &Outcome) -> bool {
            self.outcomes.contains(outcome)
        }
        pub fn standard_petsuites() -> Self {
            Self::builder()
                .program_duration(ProgramDuration::Weeks(DurationWeeks::try_new(3).unwrap()))
                .curriculum(vec![
                    CurriculumUnit::LooseLeashWalking,
                    CurriculumUnit::Recall,
                ])
                .progress(ProgressTracking::SessionNotesAndMilestones)
                .outcomes(vec![Outcome::CanineGoodCitizenReadiness])
                .trainer_availability(TrainerAvailability::NamedTrainerRequired)
                .package(PackagePolicy::MultiSessionPackage {
                    sessions: SessionCount::try_new(6).unwrap(),
                })
                .follow_up(FollowUpCadence::AfterProgramCompletion)
                .build()
        }
    }
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
