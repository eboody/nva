//! Operations owns portfolio, technology, KPI, and cross-service operating-context
//! contracts. Service-specific daily brief, lead, reputation, staff, grooming,
//! training, and retail vocabulary lives in those owner modules; this module keeps
//! those namespaces visible instead of re-exporting compatibility synonyms.

use bon::Builder;
use chrono::NaiveDate;
use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::LocationId;

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

pub mod operating_day {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Date(NaiveDate);

    impl Date {
        pub const fn try_new(value: NaiveDate) -> Result<Self> {
            Ok(Self(value))
        }

        pub const fn get(self) -> NaiveDate {
            self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Key {
        location_record_id: crate::source::record::Id,
        service_line: super::service_core::ServiceLine,
        date: Date,
    }

    impl Key {
        pub const fn new(
            location_record_id: crate::source::record::Id,
            service_line: super::service_core::ServiceLine,
            date: Date,
        ) -> Self {
            Self {
                location_record_id,
                service_line,
                date,
            }
        }

        pub const fn location_record_id(&self) -> &crate::source::record::Id {
            &self.location_record_id
        }

        pub const fn service_line(&self) -> super::service_core::ServiceLine {
            self.service_line
        }

        pub const fn date(&self) -> Date {
            self.date
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub enum Error {}

    pub type Result<T> = std::result::Result<T, Error>;
}

pub mod operational {
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
    pub struct Observation(String);

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
    pub struct Recommendation(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PainArea {
        LaborEfficiency,
        CustomerCommunicationLoad,
        ReservationCapacityOptimization,
        DataFragmentation,
        SalesRetentionMarketing,
        TrainingAndStandards,
    }
}

pub mod pet_resort {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct Portfolio {
        pub operator: Operator,
        pub resort_count: ResortCount,
        pub structure: PortfolioStructure,
        pub business_lines: Vec<BusinessLine>,
        pub brands: Vec<Brand>,
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
    pub enum Brand {
        NvaPetResorts,
        PetSuites,
        PoochHotel,
        EliteSuites,
        TheBarkSide,
        WoofdorfAstoria,
        DoggieDistrict,
        Other { name: crate::location::Name },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum OperatingTerm {
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
        accommodation: lodging_offer::Accommodation,
        included_care: Vec<lodging_offer::CareFeature>,
        add_ons: Vec<lodging_offer::AddOn>,
    },
    Daycare {
        format: DaycareFormat,
        eligibility_rules: Vec<DaycareEligibilityRule>,
    },
    Grooming {
        service: crate::grooming::Service,
        cadence: crate::grooming::rebooking::Cadence,
    },
    Training {
        program: crate::training::Program,
    },
    RetailPartnerProduct {
        partner: crate::retail::Partner,
        category: crate::retail::product::Category,
    },
}

pub mod lodging_offer {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Accommodation {
        ClassicSuite,
        LuxurySuite,
        CatCondo,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CareFeature {
        DailyHousekeeping,
        PottyWalks,
        Bedding,
        PawgressReport,
        FeedingSupport,
        MedicationSupport,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum AddOn {
        Playtime,
        ExitBath,
        PremiumSuite,
        Grooming,
        TrainingSession,
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct TechnologyEcosystem {
    pub core_portal: service_core::OperatingSystem,
    pub data_access: Vec<DataAccessPattern>,
    pub adjacent_systems: Vec<AdjacentSystem>,
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

pub mod service_core {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum OperatingSystem {
        Gingr,
        MixedSystems,
        Unknown,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct ServiceContracts {
        pub location_id: LocationId,
        pub boarding: crate::boarding::Contract,
        pub daycare: crate::daycare::Contract,
        pub grooming: crate::grooming::Contract,
        pub training: crate::training::Contract,
        pub retail: crate::retail::Contract,
    }

    impl ServiceContracts {
        pub fn core_services(&self) -> [ServiceLine; 5] {
            [
                ServiceLine::Boarding,
                ServiceLine::Daycare,
                ServiceLine::Grooming,
                ServiceLine::Training,
                ServiceLine::Retail,
            ]
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub enum ServiceLine {
        Boarding,
        Daycare,
        Grooming,
        Training,
        Retail,
    }
}
