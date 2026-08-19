//! Portfolio and cross-service operating values for pet-resort automation.
//!
//! This module models the external source-of-truth chain at the broad operations layer:
//! portfolio facts, provider/adjacent-system access patterns, service-line offerings,
//! pain areas, and labor/capacity optimization levers become validated domain vocabulary
//! before analytics, daily briefs, staff tasks, or agent workflows can use them.
//!
//! Service-specific daily brief, lead, reputation, staff, grooming, training, and retail
//! vocabulary lives in those owner modules; this module keeps the shared operations
//! namespace visible without flattening source facts into vague strings.

use bon::Builder;
use chrono::{DateTime, NaiveDate, Utc};
use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{entities, entities::LocationId, location, money, policy};

/// Labor coverage quantities and scheduled-coverage facts for operations recommendations.
///
/// Canonical owner for labor concepts previously introduced by `strategic_ai_ops::labor`.
/// These values support review-gated manager recommendations and never mutate staffing by
/// themselves.
pub mod labor;

/// Capacity and labor optimization contracts for review-gated operations recommendations.
///
/// Canonical owner for capacity concepts previously introduced by `strategic_ai_ops::capacity`.
/// Recommendations produced here remain manager-review inputs, not schedule-change authority.
pub mod capacity;

/// UTC time buckets used for SLA, staffing, demand, and attribution.
///
/// Canonical owner for bridge time-window concepts previously introduced by
/// `strategic_ai_ops::time`; new local reporting windows should prefer the existing
/// `operating_window` and `reporting_period` modules when local business-date semantics matter.
pub mod time_bucket;

/// Operating-day key used to group service-line demand, staffing, and reporting.
pub mod operating_day;

/// Local operating windows scoped by resort timezone and business date.
pub mod operating_window;

/// Reporting periods that deliberately distinguish local business dates from UTC instants.
///
/// ```
/// use chrono::{NaiveDate, TimeZone, Utc};
/// use domain::{entities, location, operations};
///
/// let location_id = entities::LocationId::new(uuid::Uuid::from_u128(1));
/// let timezone = location::Timezone::try_new("America/Los_Angeles").unwrap();
/// let local = operations::reporting_period::Period::local_operating_dates(
///     location_id,
///     timezone,
///     NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
///     NaiveDate::from_ymd_opt(2026, 8, 13).unwrap(),
/// ).unwrap();
/// let utc = operations::reporting_period::Period::utc_instants(
///     location_id,
///     Utc.with_ymd_and_hms(2026, 8, 12, 0, 0, 0).unwrap(),
///     Utc.with_ymd_and_hms(2026, 8, 13, 0, 0, 0).unwrap(),
/// ).unwrap();
/// assert!(matches!(local, operations::reporting_period::Period::LocalOperatingDates { .. }));
/// assert!(matches!(utc, operations::reporting_period::Period::UtcInstants { .. }));
/// ```
pub mod reporting_period;

/// Operational observations and recommendations produced from validated source facts.
pub mod operational;

/// Portfolio facts for the NVA Pet Resorts operating context.
pub mod pet_resort;

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
pub mod lodging_offer;

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

/// Core service-line vocabulary joining source systems to resort operating models.
pub mod service_core;
