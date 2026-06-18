//! Portfolio and cross-service operating contracts for pet-resort automation.
//!
//! This module models the external source-of-truth chain at the broad operations layer:
//! portfolio facts, Gingr/adjacent-system access patterns, service-line offerings,
//! pain areas, and labor/capacity optimization levers become validated domain vocabulary
//! before analytics, daily briefs, staff tasks, or agent workflows can use them.
//!
//! Service-specific daily brief, lead, reputation, staff, grooming, training, and retail
//! vocabulary lives in those owner modules; this module keeps the shared operations
//! namespace visible without flattening source facts into vague strings.

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
/// Validated operations metric label used for KPI/read-model dimensions.
///
/// Metric names label source-derived facts such as labor-to-revenue risk,
/// occupancy, utilization, or conversion measures without treating free text as
/// authoritative workflow state.
pub struct MetricName(String);

/// Operating day boundary for operations contracts.
pub mod operating_day {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Typed date domain value that keeps raw primitives out of operations workflows.
    pub struct Date(NaiveDate);

    impl Date {
        /// Promotes boundary input into a validated operations domain value.
        pub const fn try_new(value: NaiveDate) -> Result<Self> {
            Ok(Self(value))
        }

        /// Exposes the validated scalar for serialization and adapter boundaries.
        pub const fn get(self) -> NaiveDate {
            self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Typed key domain value that keeps raw primitives out of operations workflows.
    pub struct Key {
        location_id: LocationId,
        service_line: super::service_core::ServiceLine,
        date: Date,
    }

    impl Key {
        /// Assembles this operations value from already-validated domain parts.
        pub const fn new(
            location_id: LocationId,
            service_line: super::service_core::ServiceLine,
            date: Date,
        ) -> Self {
            Self {
                location_id,
                service_line,
                date,
            }
        }

        /// Returns this operations value's location id.
        pub const fn location_id(&self) -> LocationId {
            self.location_id
        }

        /// Returns this operations value's service line.
        pub const fn service_line(&self) -> super::service_core::ServiceLine {
            self.service_line
        }

        /// Returns this operations value's date.
        pub const fn date(&self) -> Date {
            self.date
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Validation failures returned by operations domain constructors.
    pub enum Error {}

    /// Result type returned by fallible operations operations.
    pub type Result<T> = std::result::Result<T, Error>;
}

/// Operational observations and recommendations produced from validated source facts.
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
    /// Human-readable operational observation attached to evidence-backed workflows.
    ///
    /// Observations describe what a source/read-model chain found—such as labor
    /// mismatch, customer-experience risk, or revenue leakage—without granting
    /// an agent authority to act without the target workflow gate.
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
    /// Human-readable recommendation proposed for staff or manager review.
    ///
    /// Recommendations are labor-cost levers only after the surrounding workflow
    /// decides whether they remain drafts, become staff tasks, or require manager
    /// approval.
    pub struct Recommendation(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Portfolio pain area that can become a bounded automation or labor-improvement lane.
    pub enum PainArea {
        /// Labor efficiency operations signal for labor, capacity, or task planning.
        LaborEfficiency,
        /// Customer communication load operations signal for labor, capacity, or task planning.
        CustomerCommunicationLoad,
        /// Reservation capacity optimization operations signal for labor, capacity, or task planning.
        ReservationCapacityOptimization,
        /// Data fragmentation operations signal for labor, capacity, or task planning.
        DataFragmentation,
        /// Sales retention marketing operations signal for labor, capacity, or task planning.
        SalesRetentionMarketing,
        /// Training and standards operations signal for labor, capacity, or task planning.
        TrainingAndStandards,
    }
}

/// Portfolio facts for the NVA Pet Resorts operating context.
pub mod pet_resort {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Validated portfolio context used to scope cross-resort automation and reporting.
    pub struct Portfolio {
        /// Operator fact promoted into this operations contract.
        pub operator: Operator,
        /// Resort count fact promoted into this operations contract.
        pub resort_count: ResortCount,
        /// Structure fact promoted into this operations contract.
        pub structure: PortfolioStructure,
        /// Business lines fact promoted into this operations contract.
        pub business_lines: Vec<BusinessLine>,
        /// Brands fact promoted into this operations contract.
        pub brands: Vec<Brand>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for operator decisions in operations workflows.
    pub enum Operator {
        /// National veterinary associates operations signal for labor, capacity, or task planning.
        NationalVeterinaryAssociates,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for portfolio structure decisions in operations workflows.
    pub enum PortfolioStructure {
        /// Federated multi brand operations signal for labor, capacity, or task planning.
        FederatedMultiBrand,
        /// Single brand operations signal for labor, capacity, or task planning.
        SingleBrand,
        /// Provider role or status could not be mapped confidently.
        Unknown,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for business line decisions in operations workflows.
    pub enum BusinessLine {
        /// General practice veterinary hospitals operations signal for labor, capacity, or task planning.
        GeneralPracticeVeterinaryHospitals,
        /// Pet resorts operations signal for labor, capacity, or task planning.
        PetResorts,
        /// Equine operations signal for labor, capacity, or task planning.
        Equine,
        /// Specialty emergency hospitals operations signal for labor, capacity, or task planning.
        SpecialtyEmergencyHospitals,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for brand decisions in operations workflows.
    pub enum Brand {
        /// Nva pet resorts operations signal for labor, capacity, or task planning.
        NvaPetResorts,
        /// Pet suites operations signal for labor, capacity, or task planning.
        PetSuites,
        /// Pooch hotel operations signal for labor, capacity, or task planning.
        PoochHotel,
        /// Elite suites operations signal for labor, capacity, or task planning.
        EliteSuites,
        /// The bark side operations signal for labor, capacity, or task planning.
        TheBarkSide,
        /// Woofdorf astoria operations signal for labor, capacity, or task planning.
        WoofdorfAstoria,
        /// Doggie district operations signal for labor, capacity, or task planning.
        DoggieDistrict,
        /// Contact or display name used by staff.
        Other {
            /// Name carried by this variant.
            name: crate::location::Name,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for operating term decisions in operations workflows.
    pub enum OperatingTerm {
        /// Pawgress reports operations signal for labor, capacity, or task planning.
        PawgressReports,
        /// Boarding reservations operations signal for labor, capacity, or task planning.
        BoardingReservations,
        /// Daycare packages operations signal for labor, capacity, or task planning.
        DaycarePackages,
        /// Pet points rewards operations signal for labor, capacity, or task planning.
        PetPointsRewards,
        /// Gingr customer portal operations signal for labor, capacity, or task planning.
        GingrCustomerPortal,
        /// Lead capture and conversion operations signal for labor, capacity, or task planning.
        LeadCaptureAndConversion,
        /// Website email social outreach operations signal for labor, capacity, or task planning.
        WebsiteEmailSocialOutreach,
        /// Local market plans operations signal for labor, capacity, or task planning.
        LocalMarketPlans,
        /// Sales labor expenses customer satisfaction kpis operations signal for labor, capacity, or task planning.
        SalesLaborExpensesCustomerSatisfactionKpis,
        /// Osha cash handling operational compliance operations signal for labor, capacity, or task planning.
        OshaCashHandlingOperationalCompliance,
        /// Training certification completion operations signal for labor, capacity, or task planning.
        TrainingCertificationCompletion,
        /// Resort level ebitda profitability operations signal for labor, capacity, or task planning.
        ResortLevelEbitdaProfitability,
        /// Grooming cadence operations signal for labor, capacity, or task planning.
        GroomingCadence,
        /// Daycare eligibility rules operations signal for labor, capacity, or task planning.
        DaycareEligibilityRules,
        /// Guest experience operations signal for labor, capacity, or task planning.
        GuestExperience,
        /// Team member engagement retention operations signal for labor, capacity, or task planning.
        TeamMemberEngagementRetention,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Nonzero count of resorts in the operating portfolio.
pub struct ResortCount(u16);

impl ResortCount {
    /// Promotes boundary input into a validated operations domain value.
    pub const fn try_new(value: u16) -> Result<Self, ResortCountError> {
        if value == 0 {
            return Err(ResortCountError::ZeroResorts);
        }
        Ok(Self(value))
    }

    /// Exposes the validated scalar for serialization and adapter boundaries.
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
/// Domain vocabulary for resort count error decisions in operations workflows.
pub enum ResortCountError {
    #[error("pet resort portfolios require at least one resort")]
    /// Zero resorts operations signal for labor, capacity, or task planning.
    ZeroResorts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Service offering whose source facts drive capacity, labor, upsell, and care workflows.
pub enum ServiceOffering {
    /// Overnight stay service line.
    Boarding {
        /// Accommodation fact promoted into this operations contract.
        accommodation: lodging_offer::Accommodation,
        /// Included care fact promoted into this operations contract.
        included_care: Vec<lodging_offer::CareFeature>,
        /// Add ons fact promoted into this operations contract.
        add_ons: Vec<lodging_offer::AddOn>,
    },
    /// Daycare operations signal for labor, capacity, or task planning.
    Daycare {
        /// Format fact promoted into this operations contract.
        format: DaycareFormat,
        /// Eligibility rules fact promoted into this operations contract.
        eligibility_rules: Vec<DaycareEligibilityRule>,
    },
    /// Grooming service line or care-note category.
    Grooming {
        /// Requested service that drives scheduling and labor estimates.
        service: crate::grooming::Service,
        /// Cadence fact promoted into this operations contract.
        cadence: crate::grooming::rebooking::Cadence,
    },
    /// Training service line or care-note category.
    Training {
        /// Program fact promoted into this operations contract.
        program: crate::training::Program,
    },
    /// Retail partner product operations signal for labor, capacity, or task planning.
    RetailPartnerProduct {
        /// Partner fact promoted into this operations contract.
        partner: crate::retail::Partner,
        /// Category fact promoted into this operations contract.
        category: crate::retail::product::Category,
    },
}

/// Boarding/lodging offer vocabulary that affects room capacity and care labor.
pub mod lodging_offer {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for accommodation decisions in operations workflows.
    pub enum Accommodation {
        /// Classic suite operations signal for labor, capacity, or task planning.
        ClassicSuite,
        /// Luxury suite operations signal for labor, capacity, or task planning.
        LuxurySuite,
        /// Cat condo operations signal for labor, capacity, or task planning.
        CatCondo,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for care feature decisions in operations workflows.
    pub enum CareFeature {
        /// Daily housekeeping operations signal for labor, capacity, or task planning.
        DailyHousekeeping,
        /// Potty walks operations signal for labor, capacity, or task planning.
        PottyWalks,
        /// Bedding operations signal for labor, capacity, or task planning.
        Bedding,
        /// Progress report shared with the customer during care.
        PawgressReport,
        /// Feeding support operations signal for labor, capacity, or task planning.
        FeedingSupport,
        /// Medication support operations signal for labor, capacity, or task planning.
        MedicationSupport,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for add on decisions in operations workflows.
    pub enum AddOn {
        /// Playtime operations signal for labor, capacity, or task planning.
        Playtime,
        /// Bath offered before departure from boarding.
        ExitBath,
        /// Premium suite operations signal for labor, capacity, or task planning.
        PremiumSuite,
        /// Grooming service line or care-note category.
        Grooming,
        /// Training session operations signal for labor, capacity, or task planning.
        TrainingSession,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Daycare format whose eligibility and supervision needs affect staffing.
pub enum DaycareFormat {
    /// All day play operations signal for labor, capacity, or task planning.
    AllDayPlay,
    /// Half day play operations signal for labor, capacity, or task planning.
    HalfDayPlay,
    /// Daytime boarding care with lodging-style supervision.
    DayBoarding,
    /// Day play plus room operations signal for labor, capacity, or task planning.
    DayPlayPlusRoom,
    /// Cat individual playtime operations signal for labor, capacity, or task planning.
    CatIndividualPlaytime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Daycare rule that gates group-play workflow and protects staffing/safety decisions.
pub enum DaycareEligibilityRule {
    /// Temperament review required operations signal for labor, capacity, or task planning.
    TemperamentReviewRequired,
    /// Spay neuter required for group play operations signal for labor, capacity, or task planning.
    SpayNeuterRequiredForGroupPlay,
    /// Vaccine proof required operations signal for labor, capacity, or task planning.
    VaccineProofRequired,
    /// Staff to pet ratio required operations signal for labor, capacity, or task planning.
    StaffToPetRatioRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Validated technology/source-system context for integrations and read models.
pub struct TechnologyEcosystem {
    /// Core portal fact promoted into this operations contract.
    pub core_portal: service_core::OperatingSystem,
    /// Data access fact promoted into this operations contract.
    pub data_access: Vec<DataAccessPattern>,
    /// Adjacent systems fact promoted into this operations contract.
    pub adjacent_systems: Vec<AdjacentSystem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Way operational source facts can enter the platform before validation.
pub enum DataAccessPattern {
    /// Api operations signal for labor, capacity, or task planning.
    Api,
    /// Webhook operations signal for labor, capacity, or task planning.
    Webhook,
    /// Data export operations signal for labor, capacity, or task planning.
    DataExport,
    /// Warehouse operations signal for labor, capacity, or task planning.
    Warehouse,
    /// Business intelligence dashboard operations signal for labor, capacity, or task planning.
    BusinessIntelligenceDashboard,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Adjacent enterprise system that can provide labor, revenue, marketing, or review evidence.
pub enum AdjacentSystem {
    /// Avature recruiting operations signal for labor, capacity, or task planning.
    AvatureRecruiting,
    /// Ga4 operations signal for labor, capacity, or task planning.
    Ga4,
    /// Amplitude operations signal for labor, capacity, or task planning.
    Amplitude,
    /// Google tag manager operations signal for labor, capacity, or task planning.
    GoogleTagManager,
    /// Hris operations signal for labor, capacity, or task planning.
    Hris,
    /// Labor scheduling source for staffing plans.
    LaborScheduling,
    /// Payroll source for labor-cost reconciliation.
    Payroll,
    /// Marketing automation operations signal for labor, capacity, or task planning.
    MarketingAutomation,
    /// Ticketing operations signal for labor, capacity, or task planning.
    Ticketing,
    /// Call center telephony operations signal for labor, capacity, or task planning.
    CallCenterTelephony,
    /// Reviews operations signal for labor, capacity, or task planning.
    Reviews,
    /// Email sms marketing operations signal for labor, capacity, or task planning.
    EmailSmsMarketing,
    /// Reporting or BI data source.
    BusinessIntelligence,
    /// Data lake operations signal for labor, capacity, or task planning.
    DataLake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Bounded AI use case mapped to a measurable workflow and human-approval boundary.
pub enum AiUseCase {
    /// Resort manager daily briefing operations signal for labor, capacity, or task planning.
    ResortManagerDailyBriefing,
    /// Regional ops exception reporting operations signal for labor, capacity, or task planning.
    RegionalOpsExceptionReporting,
    /// Customer inbox and call deflection operations signal for labor, capacity, or task planning.
    CustomerInboxAndCallDeflection,
    /// Lead conversion operations signal for labor, capacity, or task planning.
    LeadConversion,
    /// Grooming rebooking operations signal for labor, capacity, or task planning.
    GroomingRebooking,
    /// Post stay pawgress report assistant operations signal for labor, capacity, or task planning.
    PostStayPawgressReportAssistant,
    /// Review reputation triage operations signal for labor, capacity, or task planning.
    ReviewReputationTriage,
    /// Sop knowledge assistant operations signal for labor, capacity, or task planning.
    SopKnowledgeAssistant,
    /// Data quality ops hygiene operations signal for labor, capacity, or task planning.
    DataQualityOpsHygiene,
    /// Incident report drafting operations signal for labor, capacity, or task planning.
    IncidentReportDrafting,
    /// Training onboarding assistant operations signal for labor, capacity, or task planning.
    TrainingOnboardingAssistant,
    /// Lapsed customer winback operations signal for labor, capacity, or task planning.
    LapsedCustomerWinback,
    /// Boarding pre arrival checklist automation operations signal for labor, capacity, or task planning.
    BoardingPreArrivalChecklistAutomation,
    /// Capacity alerts operations signal for labor, capacity, or task planning.
    CapacityAlerts,
    /// Labor revenue anomaly detection operations signal for labor, capacity, or task planning.
    LaborRevenueAnomalyDetection,
    /// Website reservation assistant operations signal for labor, capacity, or task planning.
    WebsiteReservationAssistant,
    /// Vaccination document collection operations signal for labor, capacity, or task planning.
    VaccinationDocumentCollection,
    /// Demand forecasting operations signal for labor, capacity, or task planning.
    DemandForecasting,
    /// Staffing recommendations operations signal for labor, capacity, or task planning.
    StaffingRecommendations,
    /// Regional performance benchmarking operations signal for labor, capacity, or task planning.
    RegionalPerformanceBenchmarking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Portfolio-level hygiene issue type that can explain unreliable labor/read-model signals.
pub enum DataQualityIssue {
    /// Missing pet vaccination records operations signal for labor, capacity, or task planning.
    MissingPetVaccinationRecords,
    /// Incomplete pet profiles operations signal for labor, capacity, or task planning.
    IncompletePetProfiles,
    /// Duplicate customers operations signal for labor, capacity, or task planning.
    DuplicateCustomers,
    /// Missing temperament notes operations signal for labor, capacity, or task planning.
    MissingTemperamentNotes,
    /// Open invoices operations signal for labor, capacity, or task planning.
    OpenInvoices,
    /// Unclosed reservations operations signal for labor, capacity, or task planning.
    UnclosedReservations,
    /// Unused packages operations signal for labor, capacity, or task planning.
    UnusedPackages,
    /// Staff notes too vague operations signal for labor, capacity, or task planning.
    StaffNotesTooVague,
    /// Inconsistent service naming across sites operations signal for labor, capacity, or task planning.
    InconsistentServiceNamingAcrossSites,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Resort operating function whose workload can be reduced or coordinated by automation.
pub enum OperatingFunction {
    /// Front desk operations signal for labor, capacity, or task planning.
    FrontDesk,
    /// Call center operations signal for labor, capacity, or task planning.
    CallCenter,
    /// General managers operations signal for labor, capacity, or task planning.
    GeneralManagers,
    /// Assistant general managers operations signal for labor, capacity, or task planning.
    AssistantGeneralManagers,
    /// Regional operations operations signal for labor, capacity, or task planning.
    RegionalOperations,
    /// Grooming service line or care-note category.
    Grooming,
    /// Training service line or care-note category.
    Training,
    /// Marketing operations signal for labor, capacity, or task planning.
    Marketing,
    /// Information technology operations signal for labor, capacity, or task planning.
    InformationTechnology,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Training/standards workflow where AI can reduce lookup or documentation labor.
pub enum StaffTrainingWorkflow {
    /// New hire onboarding operations signal for labor, capacity, or task planning.
    NewHireOnboarding,
    /// Sop lookup operations signal for labor, capacity, or task planning.
    SopLookup,
    /// Incident documentation operations signal for labor, capacity, or task planning.
    IncidentDocumentation,
    /// Pet behavior note consistency operations signal for labor, capacity, or task planning.
    PetBehaviorNoteConsistency,
    /// Manager coaching operations signal for labor, capacity, or task planning.
    ManagerCoaching,
    /// Regulatory safety policy operations signal for labor, capacity, or task planning.
    RegulatorySafetyPolicy,
    /// Customer complaint handling operations signal for labor, capacity, or task planning.
    CustomerComplaintHandling,
    /// Training quiz generation operations signal for labor, capacity, or task planning.
    TrainingQuizGeneration,
    /// Shift lead copilot operations signal for labor, capacity, or task planning.
    ShiftLeadCopilot,
    /// Shift summary operations signal for labor, capacity, or task planning.
    ShiftSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// High-volume customer communication workflow suitable for drafting or triage.
pub enum CustomerCommunicationWorkflow {
    /// Availability question operations signal for labor, capacity, or task planning.
    AvailabilityQuestion,
    /// Vaccine requirement question operations signal for labor, capacity, or task planning.
    VaccineRequirementQuestion,
    /// Multi pet boarding question operations signal for labor, capacity, or task planning.
    MultiPetBoardingQuestion,
    /// Group play eligibility question operations signal for labor, capacity, or task planning.
    GroupPlayEligibilityQuestion,
    /// Daycare readiness question operations signal for labor, capacity, or task planning.
    DaycareReadinessQuestion,
    /// Add bath request operations signal for labor, capacity, or task planning.
    AddBathRequest,
    /// Pet update request operations signal for labor, capacity, or task planning.
    PetUpdateRequest,
    /// Checkout time question operations signal for labor, capacity, or task planning.
    CheckoutTimeQuestion,
    /// Cancel or change reservation operations signal for labor, capacity, or task planning.
    CancelOrChangeReservation,
    /// Loyalty points question operations signal for labor, capacity, or task planning.
    LoyaltyPointsQuestion,
    /// Training options question operations signal for labor, capacity, or task planning.
    TrainingOptionsQuestion,
    /// Anxiety or special handling question operations signal for labor, capacity, or task planning.
    AnxietyOrSpecialHandlingQuestion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Constraint that limits capacity utilization or creates labor mismatch risk.
pub enum CapacityConstraintKind {
    /// Room or suite availability operations signal for labor, capacity, or task planning.
    RoomOrSuiteAvailability,
    /// Play yard availability operations signal for labor, capacity, or task planning.
    PlayYardAvailability,
    /// Groomer slot availability operations signal for labor, capacity, or task planning.
    GroomerSlotAvailability,
    /// Trainer availability operations signal for labor, capacity, or task planning.
    TrainerAvailability,
    /// Staff ratio operations signal for labor, capacity, or task planning.
    StaffRatio,
    /// Pet temperament operations signal for labor, capacity, or task planning.
    PetTemperament,
    /// Holiday peak operations signal for labor, capacity, or task planning.
    HolidayPeak,
    /// Check in checkout bottleneck operations signal for labor, capacity, or task planning.
    CheckInCheckoutBottleneck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Labor, capacity, or revenue optimization lever supported by validated source facts.
pub enum OptimizationOpportunity {
    /// Demand forecasting operations signal for labor, capacity, or task planning.
    DemandForecasting,
    /// No show prediction operations signal for labor, capacity, or task planning.
    NoShowPrediction,
    /// Dynamic waitlist filling operations signal for labor, capacity, or task planning.
    DynamicWaitlistFilling,
    /// Capacity recommendation operations signal for labor, capacity, or task planning.
    CapacityRecommendation,
    /// Add on recommendation operations signal for labor, capacity, or task planning.
    AddOnRecommendation,
    /// Holiday planning operations signal for labor, capacity, or task planning.
    HolidayPlanning,
    /// Over under staffing alert operations signal for labor, capacity, or task planning.
    OverUnderStaffingAlert,
    /// Revenue optimization without care degradation operations signal for labor, capacity, or task planning.
    RevenueOptimizationWithoutCareDegradation,
}

/// Core service-line boundary joining source systems to service contracts.
pub mod service_core {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for operating system decisions in operations workflows.
    pub enum OperatingSystem {
        /// Gingr reservation and pet-care operating system.
        Gingr,
        /// Mixed systems operations signal for labor, capacity, or task planning.
        MixedSystems,
        /// Provider role or status could not be mapped confidently.
        Unknown,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Service-line contract bundle for one resort/location's operating model.
    pub struct ServiceContracts {
        /// Location id fact promoted into this operations contract.
        pub location_id: LocationId,
        /// Boarding fact promoted into this operations contract.
        pub boarding: crate::boarding::Contract,
        /// Daycare fact promoted into this operations contract.
        pub daycare: crate::daycare::Contract,
        /// Grooming fact promoted into this operations contract.
        pub grooming: crate::grooming::Contract,
        /// Training fact promoted into this operations contract.
        pub training: crate::training::Contract,
        /// Retail fact promoted into this operations contract.
        pub retail: crate::retail::Contract,
    }

    impl ServiceContracts {
        /// Returns the five service lines whose demand and staffing drive daily labor plans.
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
    /// Core pet-resort service line used to partition demand, capacity, and labor metrics.
    pub enum ServiceLine {
        /// Overnight stay service line.
        Boarding,
        /// Daycare operations signal for labor, capacity, or task planning.
        Daycare,
        /// Grooming service line or care-note category.
        Grooming,
        /// Training service line or care-note category.
        Training,
        /// Retail operations signal for labor, capacity, or task planning.
        Retail,
    }
}
