use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::LocationId;

pub use crate::daily_brief::{
    Action, ArrivalDepartureSnapshot, CapacityBooked, CapacityLimit, CapacityLimitError,
    CapacityMetric, CapacitySaturationBasisPoints, CustomerFollowUp, DailyBriefSection,
    FollowUpReason, LaborRisk, LaborSnapshot, OccupancySnapshot, PetCareWatch, PetCareWatchReason,
    ResortDailyBrief, ResortOperatingDay, RevenueOpportunity, RevenueOpportunityKind, Risk,
    ScheduledStaffCount, SnapshotId,
};
pub use crate::lead::{
    CampaignName, Lead, LeadConversionStage, LeadIntent, LeadNextAction, LeadSource, LeadSourceName,
};
pub use crate::reputation::{
    ReputationSignal, ReviewEscalation, ReviewId, ReviewPlatformName, ReviewSentiment, ReviewTheme,
};
pub use crate::staff::{
    StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority, StaffTaskSource,
    StaffTaskStatus, TaskCompletionEvidence,
};

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
