//! Portfolio and cross-service operating values for pet-resort automation.
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
/// Metric names label provider/read-model facts such as labor-to-revenue risk,
/// occupancy, utilization, or conversion measures without treating free text as
/// authoritative workflow state.
pub struct MetricName(String);

/// Operating-day key used to group service-line demand, staffing, and reporting.
pub mod operating_day {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Operating-day date used when manager briefs compare booked demand against staffing and room capacity.
    pub struct Date(NaiveDate);

    impl Date {
        /// Accepts a source/read-model operating date after the adapter has already chosen the resort business day.
        pub const fn try_new(value: NaiveDate) -> Result<Self> {
            Ok(Self(value))
        }

        /// Returns the operating-day date for storage records, analytics projections, or adapter output.
        pub const fn get(self) -> NaiveDate {
            self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Location/service/date key that groups the demand and staffing facts a manager brief can rank.
    pub struct Key {
        location_id: LocationId,
        service_line: super::service_core::ServiceLine,
        date: Date,
    }

    impl Key {
        /// Assembles the resort, service line, and operating day used before analytics can compare labor to demand.
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

        /// Returns the resort/location whose staffing or capacity queue is being evaluated.
        pub const fn location_id(&self) -> LocationId {
            self.location_id
        }

        /// Returns the service line whose boarding, daycare, grooming, training, or retail demand is being grouped.
        pub const fn service_line(&self) -> super::service_core::ServiceLine {
            self.service_line
        }

        /// Returns the business day for the manager or regional reporting workflow.
        pub const fn date(&self) -> Date {
            self.date
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Validation failures returned by operations domain constructors.
    pub enum Error {}

    /// Result type for operations values that must reject impossible reporting keys before automation sees them.
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
        /// Labor-efficiency pain area where automation should reduce manual staffing and demand reconciliation.
        LaborEfficiency,
        /// Customer-communication pain area where drafting or triage may reduce phone and inbox load.
        CustomerCommunicationLoad,
        /// Reservation-capacity pain area where recommendations must respect room, yard, staff, and policy gates.
        ReservationCapacityOptimization,
        /// Data-fragmentation pain area where duplicate or missing source facts make workflows slower and less safe.
        DataFragmentation,
        /// Sales/retention pain area where outreach candidates can be ranked but customer contact remains reviewed.
        SalesRetentionMarketing,
        /// Training/standards pain area where assistants reduce lookup and documentation labor without replacing manager approval.
        TrainingAndStandards,
    }
}

/// Portfolio facts for the NVA Pet Resorts operating context.
pub mod pet_resort {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Validated portfolio context used to scope cross-resort automation and reporting.
    pub struct Portfolio {
        /// Operator whose portfolio context explains why the same labor and source-governance contracts apply across resorts.
        pub operator: Operator,
        /// Number of resorts used to size portfolio rollups; zero resorts is rejected before outcome metrics are reported.
        pub resort_count: ResortCount,
        /// Portfolio structure used to decide whether a brief is local, brand-level, or cross-brand comparison context.
        pub structure: PortfolioStructure,
        /// Business lines that keep pet-resort automation scoped away from veterinary or equine assumptions unless explicitly modeled.
        pub business_lines: Vec<BusinessLine>,
        /// Pet-resort brands used for navigation and reporting filters, not as automatic permission to change local policy.
        pub brands: Vec<Brand>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Portfolio operator vocabulary used to scope source evidence and labor-value claims.
    pub enum Operator {
        /// NVA portfolio context for cross-resort reporting; it does not override local manager approval gates.
        NationalVeterinaryAssociates,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Portfolio structure vocabulary that tells reports whether comparisons are single-brand or federated.
    pub enum PortfolioStructure {
        /// Multi-brand portfolio context where regional reports compare patterns without assuming one brand policy fits every site.
        FederatedMultiBrand,
        /// Single-brand context where comparisons can use a narrower policy and vocabulary set.
        SingleBrand,
        /// Provider role or status could not be mapped confidently.
        Unknown,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// NVA business-line vocabulary used to keep pet-resort labor claims separate from other NVA operating models.
    pub enum BusinessLine {
        /// Veterinary-hospital line of business retained as adjacent context, not a source for pet-resort policy.
        GeneralPracticeVeterinaryHospitals,
        /// Pet-resort line of business where boarding, daycare, grooming, training, and retail workflows are in scope.
        PetResorts,
        /// Equine line of business retained as out-of-scope portfolio context unless a source contract models it directly.
        Equine,
        /// Specialty/emergency hospital context retained so reports do not confuse medical operations with resort labor loops.
        SpecialtyEmergencyHospitals,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Pet-resort brand vocabulary used for portfolio filtering and source reconciliation.
    pub enum Brand {
        /// NVA Pet Resorts portfolio label for rollups and navigation across resort brands.
        NvaPetResorts,
        /// PetSuites brand label used when comparing resort workflows that may carry brand-specific naming.
        PetSuites,
        /// Pooch Hotel brand label used for portfolio reports without inventing local policy authority.
        PoochHotel,
        /// Elite Suites brand label for source and reporting filters.
        EliteSuites,
        /// The Bark Side brand label for source and reporting filters.
        TheBarkSide,
        /// Woofdorf Astoria brand label for source and reporting filters.
        WoofdorfAstoria,
        /// Doggie District brand label for source and reporting filters.
        DoggieDistrict,
        /// Local or acquired brand name that staff recognize but the domain cannot classify into a known portfolio brand.
        Other {
            /// Display name retained so a reviewer can map the local brand before it appears in portfolio reporting.
            name: crate::location::Name,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Operating terms from public/provider context that become labels for labor, source, and workflow discovery.
    pub enum OperatingTerm {
        /// Customer care-report workflow where automation may draft narrative updates but staff approve sends.
        PawgressReports,
        /// Boarding reservation workflow whose demand, suites, and stay dates drive capacity and labor planning.
        BoardingReservations,
        /// Daycare package workflow where eligibility, unused sessions, and staffed playgroups affect value and safety.
        DaycarePackages,
        /// Loyalty/rewards context useful for customer questions; it does not authorize point or billing changes by automation.
        PetPointsRewards,
        /// Gingr portal context for customer self-service evidence before source promotion.
        GingrCustomerPortal,
        /// Lead conversion workflow where automation may rank follow-up work but cannot book or message without approval.
        LeadCaptureAndConversion,
        /// Marketing/outreach source context for demand discovery and drafted responses.
        WebsiteEmailSocialOutreach,
        /// Local-market planning context for human-reviewed growth work, not an automatic pricing or staffing decision.
        LocalMarketPlans,
        /// KPI bundle used to compare revenue, labor expense, and satisfaction without treating any one metric as final authority.
        SalesLaborExpensesCustomerSatisfactionKpis,
        /// Compliance context that must stay human-reviewed before safety, cash-handling, or personnel actions occur.
        OshaCashHandlingOperationalCompliance,
        /// Staff training completion context for standards follow-up and manager coaching queues.
        TrainingCertificationCompletion,
        /// Resort profitability context for regional reporting; automation may summarize variance, not change budgets.
        ResortLevelEbitdaProfitability,
        /// Grooming rebooking cadence context for staff-reviewed outreach and schedule-fill opportunities.
        GroomingCadence,
        /// Daycare eligibility context where temperament, vaccine, and ratio rules remain safety gates.
        DaycareEligibilityRules,
        /// Guest-experience context for reputation and follow-up queues where customer contact stays review-gated.
        GuestExperience,
        /// Team-member engagement context for manager coaching and labor risk summaries, not personnel action automation.
        TeamMemberEngagementRetention,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Nonzero count of resorts used when portfolio metrics claim regional or cross-brand labor impact.
pub struct ResortCount(u16);

impl ResortCount {
    /// Accepts a source/read-model operating date after the adapter has already chosen the resort business day.
    pub const fn try_new(value: u16) -> Result<Self, ResortCountError> {
        if value == 0 {
            return Err(ResortCountError::ZeroResorts);
        }
        Ok(Self(value))
    }

    /// Returns the operating-day date for storage records, analytics projections, or adapter output.
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
/// Resort-count validation failure that prevents meaningless portfolio reports.
pub enum ResortCountError {
    #[error("pet resort portfolios require at least one resort")]
    /// Zero resorts would make labor-value and portfolio comparisons fictitious, so construction fails.
    ZeroResorts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Service offering whose source facts drive capacity, labor, upsell, and care workflows.
pub enum ServiceOffering {
    /// Overnight stay service line.
    Boarding {
        /// Boarding room/suite choice that drives capacity checks and room-labor expectations.
        accommodation: lodging_offer::Accommodation,
        /// Included boarding care features that explain kennel labor before any upsell or customer copy is drafted.
        included_care: Vec<lodging_offer::CareFeature>,
        /// Optional boarding add-ons that can become reviewed upsell or staffing signals.
        add_ons: Vec<lodging_offer::AddOn>,
    },
    /// Daycare service offering where group-play eligibility and staffing ratios gate automation suggestions.
    Daycare {
        /// Daycare format used to estimate play-yard, room, and supervision needs.
        format: DaycareFormat,
        /// Daycare rules that must be satisfied before group-play recommendations or package value claims are shown.
        eligibility_rules: Vec<DaycareEligibilityRule>,
    },
    /// Grooming service line or care-note category.
    Grooming {
        /// Requested service that drives scheduling and labor estimates.
        service: crate::grooming::Service,
        /// Grooming rebooking cadence used to explain follow-up timing; it does not send customer outreach by itself.
        cadence: crate::grooming::rebooking::Cadence,
    },
    /// Training service line or care-note category.
    Training {
        /// Training program context used for package progress, trainer handoff, and graduation/follow-up tasks.
        program: crate::training::Program,
    },
    /// Retail partner product context for inventory and recommendation workflows; purchasing and discounts stay gated.
    RetailPartnerProduct {
        /// Retail partner whose catalog evidence can explain recommendations but cannot override local inventory policy.
        partner: crate::retail::Partner,
        /// Retail category used to match checkout, inventory, and care-sensitive recommendation rules.
        category: crate::retail::product::Category,
    },
}

/// Boarding/lodging offer vocabulary that affects room capacity and care labor.
pub mod lodging_offer {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Boarding accommodation vocabulary used for room capacity and housekeeping-labor math.
    pub enum Accommodation {
        /// Standard boarding suite option used as baseline capacity and housekeeping labor.
        ClassicSuite,
        /// Premium boarding suite option that may affect capacity, add-on value, and service expectations.
        LuxurySuite,
        /// Cat lodging option kept distinct from dog suites for capacity and care-labor planning.
        CatCondo,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Boarding care-feature vocabulary used to explain included labor and customer-update obligations.
    pub enum CareFeature {
        /// Daily housekeeping care feature that contributes predictable kennel labor.
        DailyHousekeeping,
        /// Potty-walk care feature that affects labor scheduling and owner expectations.
        PottyWalks,
        /// Bedding care feature that affects room setup and cleaning labor.
        Bedding,
        /// Progress report shared with the customer during care.
        PawgressReport,
        /// Feeding-support feature that can require staff instructions and care-note evidence.
        FeedingSupport,
        /// Medication-support feature that stays safety-sensitive and must not be changed by automation.
        MedicationSupport,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Boarding add-on vocabulary used for reviewed upsell, capacity, and labor signals.
    pub enum AddOn {
        /// Playtime add-on that creates extra yard or staff time before it can be offered or scheduled.
        Playtime,
        /// Bath offered before departure from boarding.
        ExitBath,
        /// Premium-suite add-on that changes room value and capacity expectations.
        PremiumSuite,
        /// Grooming service line or care-note category.
        Grooming,
        /// Training-session add-on that requires trainer availability and human-reviewed scheduling.
        TrainingSession,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Daycare format whose eligibility and supervision needs affect staffing.
pub enum DaycareFormat {
    /// Full-day daycare format with the largest playgroup labor and ratio exposure.
    AllDayPlay,
    /// Half-day daycare format that changes demand units and staffing windows.
    HalfDayPlay,
    /// Daytime boarding care with lodging-style supervision.
    DayBoarding,
    /// Daycare format with both playgroup and room capacity implications.
    DayPlayPlusRoom,
    /// Cat playtime format kept separate from dog group-play eligibility and staffing assumptions.
    CatIndividualPlaytime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Daycare rule that gates group-play workflow and protects staffing/safety decisions.
pub enum DaycareEligibilityRule {
    /// Temperament review gate that blocks group-play automation until a human/source record supports it.
    TemperamentReviewRequired,
    /// Spay/neuter rule that can explain a daycare hold but cannot be bypassed by an agent recommendation.
    SpayNeuterRequiredForGroupPlay,
    /// Vaccine-proof rule that keeps daycare safety review ahead of package or playgroup recommendations.
    VaccineProofRequired,
    /// Staff-to-pet ratio rule that turns demand into a labor-capacity constraint.
    StaffToPetRatioRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Validated technology/source-system context for integrations and read models.
pub struct TechnologyEcosystem {
    /// Primary operating system whose records may feed workflows after DTO quarantine and domain promotion.
    pub core_portal: service_core::OperatingSystem,
    /// Access patterns that describe how source facts arrive before validation and redaction.
    pub data_access: Vec<DataAccessPattern>,
    /// Adjacent systems that can corroborate labor, revenue, marketing, or review evidence without becoming domain policy.
    pub adjacent_systems: Vec<AdjacentSystem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Way operational source facts can enter the platform before validation.
pub enum DataAccessPattern {
    /// Direct API access path for source facts, subject to transport redaction and mapping contracts.
    Api,
    /// Webhook access path for event-driven evidence that still requires idempotent mapping and review gates.
    Webhook,
    /// Batch export path used for reconciliation when live API authority is unavailable or inappropriate.
    DataExport,
    /// Warehouse path used for aggregate reporting rather than live provider writes.
    Warehouse,
    /// BI dashboard source used as reporting evidence, not as a workflow authority.
    BusinessIntelligenceDashboard,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Adjacent enterprise system that can provide labor, revenue, marketing, or review evidence.
pub enum AdjacentSystem {
    /// Recruiting system context for staffing risk and hiring pipeline evidence.
    AvatureRecruiting,
    /// GA4 marketing/traffic evidence for demand and lead-funnel context.
    Ga4,
    /// Amplitude product analytics evidence for portal or app behavior.
    Amplitude,
    /// Google Tag Manager context for instrumentation evidence, not customer-contact authority.
    GoogleTagManager,
    /// HRIS context for staffing evidence; personnel actions remain outside automation authority.
    Hris,
    /// Labor scheduling source for staffing plans.
    LaborScheduling,
    /// Payroll source for labor-cost reconciliation.
    Payroll,
    /// Marketing automation context for campaign evidence; customer sends stay approval-gated.
    MarketingAutomation,
    /// Ticketing context for support workload and unresolved exception queues.
    Ticketing,
    /// Call-center telephony evidence for repeat-question volume and deflection opportunities.
    CallCenterTelephony,
    /// Review-platform evidence for reputation triage and human-approved responses.
    Reviews,
    /// Email/SMS marketing evidence for retention outreach; sends remain human-approved.
    EmailSmsMarketing,
    /// Reporting or BI data source.
    BusinessIntelligence,
    /// Data-lake context for aggregate evidence and historical reconciliation.
    DataLake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Bounded AI use case mapped to a measurable workflow and human-approval gate.
pub enum AiUseCase {
    /// Daily manager brief ranks source-backed exceptions so managers can avoid morning spreadsheet reconciliation.
    ResortManagerDailyBriefing,
    /// Regional exception reports summarize cross-site risks for review without changing local workflows automatically.
    RegionalOpsExceptionReporting,
    /// Inbox/call deflection drafts answers for repeat questions while customer sends remain gated.
    CustomerInboxAndCallDeflection,
    /// Lead conversion ranks follow-up opportunities; booking, pricing, and messages remain approval-gated.
    LeadConversion,
    /// Grooming rebooking identifies cadence gaps and drafts follow-up for staff review.
    GroomingRebooking,
    /// Post-stay Pawgress assistant drafts care summaries from evidence that staff approve before sending.
    PostStayPawgressReportAssistant,
    /// Reputation triage classifies review risk and drafts responses for human approval.
    ReviewReputationTriage,
    /// SOP knowledge assistant reduces lookup labor but does not replace manager judgment or safety policy.
    SopKnowledgeAssistant,
    /// Data-quality hygiene queues duplicate, stale, or missing-source records for review.
    DataQualityOpsHygiene,
    /// Incident report drafting organizes facts for safety review; it does not finalize incident determinations.
    IncidentReportDrafting,
    /// Training onboarding assistant drafts learning paths and checklists for manager review.
    TrainingOnboardingAssistant,
    /// Lapsed-customer winback ranks outreach candidates while customer contact remains approval-gated.
    LapsedCustomerWinback,
    /// Boarding pre-arrival checklist surfaces vaccine, feeding, and accommodation gaps before check-in.
    BoardingPreArrivalChecklistAutomation,
    /// Capacity alerts warn staff about room, yard, or service-slot pressure before accepting more demand.
    CapacityAlerts,
    /// Labor/revenue anomaly detection flags variance for manager review before staffing or pricing changes.
    LaborRevenueAnomalyDetection,
    /// Website reservation assistant can draft intake help, not commit bookings or provider writes.
    WebsiteReservationAssistant,
    /// Vaccination document collection reduces chase-down labor while medical/safety acceptance remains reviewed.
    VaccinationDocumentCollection,
    /// Demand forecasting projects service-line workload so staffing plans can be reviewed earlier.
    DemandForecasting,
    /// Staffing recommendations compare forecast demand with labor signals but remain manager-reviewed.
    StaffingRecommendations,
    /// Regional benchmarking compares sites for coaching and investigation, not automatic enforcement.
    RegionalPerformanceBenchmarking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Portfolio-level hygiene issue type that can explain unreliable labor/read-model signals.
pub enum DataQualityIssue {
    /// Missing vaccine record issue queues document follow-up and blocks unsafe eligibility assumptions.
    MissingPetVaccinationRecords,
    /// Incomplete pet profile issue explains why care, eligibility, or communication drafts need staff review.
    IncompletePetProfiles,
    /// Duplicate customer issue reduces lookup and billing confusion after a human verifies merge safety.
    DuplicateCustomers,
    /// Missing temperament note issue blocks group-play confidence until staff/source evidence exists.
    MissingTemperamentNotes,
    /// Open invoice issue surfaces checkout or payment follow-up without authorizing payment movement.
    OpenInvoices,
    /// Unclosed reservation issue helps staff finish stays before occupancy, billing, or labor reports drift.
    UnclosedReservations,
    /// Unused package issue can explain retention opportunities while outreach and account changes stay reviewed.
    UnusedPackages,
    /// Vague staff-note issue queues cleanup because weak evidence makes summaries and safety handoffs unreliable.
    StaffNotesTooVague,
    /// Inconsistent service-name issue prevents cross-site reporting from merging unlike offerings by accident.
    InconsistentServiceNamingAcrossSites,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Resort operating function whose workload can be reduced or coordinated by automation.
pub enum OperatingFunction {
    /// Front desk workload includes check-in, checkout, phone, inbox, and source-correction queues.
    FrontDesk,
    /// Call-center workload covers repeat questions and lead routing that automation may draft or classify.
    CallCenter,
    /// General managers own daily review gates for staffing, capacity, exceptions, and customer-impacting actions.
    GeneralManagers,
    /// Assistant general managers share local exception review and handoff cleanup work.
    AssistantGeneralManagers,
    /// Regional operations reviews portfolio variance and coaching opportunities without bypassing site authority.
    RegionalOperations,
    /// Grooming service line or care-note category.
    Grooming,
    /// Training service line or care-note category.
    Training,
    /// Marketing workload includes reviewed outreach, campaign evidence, and retention queues.
    Marketing,
    /// IT workload includes integration, access, and source-system hygiene needed before automation can trust evidence.
    InformationTechnology,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Training/standards workflow where AI can reduce lookup or documentation labor.
pub enum StaffTrainingWorkflow {
    /// New-hire onboarding workflow can draft checklists and quizzes but manager certification remains authoritative.
    NewHireOnboarding,
    /// SOP lookup workflow reduces policy-search time while safety-sensitive interpretation stays reviewed.
    SopLookup,
    /// Incident documentation workflow drafts chronology and evidence packets for safety review.
    IncidentDocumentation,
    /// Pet-behavior note consistency workflow helps staff rewrite vague notes into source-backed handoffs.
    PetBehaviorNoteConsistency,
    /// Manager coaching workflow summarizes patterns for human coaching, not personnel action automation.
    ManagerCoaching,
    /// Regulatory/safety policy workflow keeps compliance answers review-gated and source-cited.
    RegulatorySafetyPolicy,
    /// Customer complaint workflow drafts summaries and responses for manager approval.
    CustomerComplaintHandling,
    /// Training quiz workflow drafts knowledge checks that managers review before using for certification.
    TrainingQuizGeneration,
    /// Shift-lead copilot workflow summarizes tasks and risks but does not assign safety-sensitive work unsupervised.
    ShiftLeadCopilot,
    /// Shift summary workflow converts source-backed notes into handoffs that supervisors can verify.
    ShiftSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// High-volume customer communication workflow suitable for drafting or triage.
pub enum CustomerCommunicationWorkflow {
    /// Availability questions can be answered from source-backed capacity only when booking remains review-gated.
    AvailabilityQuestion,
    /// Vaccine requirement questions need policy/source citations and cannot approve medical compliance automatically.
    VaccineRequirementQuestion,
    /// Multi-pet boarding questions combine room capacity, household context, and staff-reviewed booking rules.
    MultiPetBoardingQuestion,
    /// Group-play eligibility questions depend on temperament, vaccine, and ratio evidence before recommendations.
    GroupPlayEligibilityQuestion,
    /// Daycare readiness questions turn profile evidence into staff-reviewed eligibility guidance.
    DaycareReadinessQuestion,
    /// Add-bath requests affect grooming/exit-bath capacity and require schedule confirmation.
    AddBathRequest,
    /// Pet-update requests may draft from care notes, but customer-visible messages remain staff-approved.
    PetUpdateRequest,
    /// Checkout-time questions use reservation policy evidence and do not change stay or fee state automatically.
    CheckoutTimeQuestion,
    /// Cancel/change questions require human approval before schedule mutation, fee, or provider write.
    CancelOrChangeReservation,
    /// Loyalty-points questions can summarize account evidence but cannot adjust balances automatically.
    LoyaltyPointsQuestion,
    /// Training-options questions can rank programs while enrollment and scheduling remain reviewed.
    TrainingOptionsQuestion,
    /// Anxiety/special-handling questions stay safety-sensitive and require staff review before care commitments.
    AnxietyOrSpecialHandlingQuestion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Constraint that limits capacity utilization or creates labor mismatch risk.
pub enum CapacityConstraintKind {
    /// Room/suite availability constraint blocks overbooking and informs waitlist or staffing review.
    RoomOrSuiteAvailability,
    /// Play-yard availability constraint links daycare demand to space and supervision limits.
    PlayYardAvailability,
    /// Groomer-slot availability constraint protects schedule promises and grooming labor plans.
    GroomerSlotAvailability,
    /// Trainer availability constraint protects package scheduling and trainer handoff promises.
    TrainerAvailability,
    /// Staff-ratio constraint turns pet counts into safety and labor review requirements.
    StaffRatio,
    /// Pet-temperament constraint can block group play or require manager review before recommendations.
    PetTemperament,
    /// Holiday-peak constraint flags demand periods where minimum stays, staffing, and capacity need review.
    HolidayPeak,
    /// Check-in/checkout bottleneck constraint explains front-desk labor pressure and queue risk.
    CheckInCheckoutBottleneck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Labor, capacity, or revenue optimization lever supported by validated source facts.
pub enum OptimizationOpportunity {
    /// Demand forecasting projects service-line workload so staffing plans can be reviewed earlier.
    DemandForecasting,
    /// No-show prediction can rank follow-up or waitlist work but cannot cancel or rebook automatically.
    NoShowPrediction,
    /// Dynamic waitlist filling recommends candidates for human-approved booking outreach.
    DynamicWaitlistFilling,
    /// Capacity recommendation summarizes source-backed pressure while booking decisions stay reviewed.
    CapacityRecommendation,
    /// Add-on recommendation can surface relevant services but customer offers remain approved by staff.
    AddOnRecommendation,
    /// Holiday planning uses forecast demand to prepare staffing and capacity reviews ahead of peak periods.
    HolidayPlanning,
    /// Over/under-staffing alert compares demand with schedules so managers can review labor changes.
    OverUnderStaffingAlert,
    /// Revenue optimization is constrained by care and safety evidence before any pricing or offer change is considered.
    RevenueOptimizationWithoutCareDegradation,
}

/// Core service-line vocabulary joining source systems to resort operating models.
pub mod service_core {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Operating-system vocabulary that tells source adapters which provider facts need quarantine before domain promotion.
    pub enum OperatingSystem {
        /// Gingr reservation and pet-care operating system.
        Gingr,
        /// Mixed operating systems require source reconciliation before automation trusts cross-system facts.
        MixedSystems,
        /// Provider role or status could not be mapped confidently.
        Unknown,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Service-line operating bundle for one resort/location.
    pub struct ServiceContracts {
        /// Location whose service contracts and outcomes are being compared in local or regional reports.
        pub location_id: LocationId,
        /// Boarding contract that owns stay, suite, minimum-stay, and checkout-exception rules.
        pub boarding: crate::boarding::Contract,
        /// Daycare contract that owns group-play eligibility, package, and ratio rules.
        pub daycare: crate::daycare::Contract,
        /// Grooming contract that owns service duration, rebooking cadence, add-on, and no-show rules.
        pub grooming: crate::grooming::Contract,
        /// Training contract that owns program progress, package, graduation, and trainer handoff rules.
        pub training: crate::training::Contract,
        /// Retail contract that owns catalog, inventory, POS, recommendation, and reorder gates.
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
        /// Daycare service offering where group-play eligibility and staffing ratios gate automation suggestions.
        Daycare,
        /// Grooming service line or care-note category.
        Grooming,
        /// Training service line or care-note category.
        Training,
        /// Retail service line partitions inventory, checkout, and recommendation work from care-service capacity.
        Retail,
    }
}
