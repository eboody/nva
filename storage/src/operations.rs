//! Persistence records for app/domain operational contracts.
//!
//! Storage code is allowed to speak in stable record codes, but promotion back into
//! `domain` values is explicit and source-grounded. This keeps labor-saving
//! workflows from treating database vocabulary as the business model.
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use storage::operations::{
//!     ServiceOfferingKindCode, ServiceOfferingRecord, StoredSourceRecordRef,
//! };
//!
//! let source_ref = StoredSourceRecordRef {
//!     system: "gingr".to_owned(),
//!     record_type: "reservation_type".to_owned(),
//!     record_id: "reservation-type-42".to_owned(),
//!     observed_at: "2026-06-18T14:00:00Z".to_owned(),
//!     adapter_version: "gingr-fixture-v1".to_owned(),
//! };
//!
//! let promoted_service = domain::operations::ServiceOffering::Daycare {
//!     format: domain::operations::DaycareFormat::AllDayPlay,
//!     eligibility_rules: vec![
//!         domain::operations::DaycareEligibilityRule::TemperamentReviewRequired,
//!         domain::operations::DaycareEligibilityRule::StaffToPetRatioRequired,
//!     ],
//! };
//!
//! let stored = ServiceOfferingRecord::try_from(promoted_service.clone())?;
//! assert_eq!(stored.service_kind, ServiceOfferingKindCode::Daycare);
//! assert_eq!(source_ref.record_id, "reservation-type-42");
//!
//! let encoded = stored.encode_json()?;
//! let decoded = ServiceOfferingRecord::decode_json(&encoded)?;
//! let demoted: domain::operations::ServiceOffering = decoded.try_into()?;
//! assert_eq!(demoted, promoted_service);
//! # Ok(())
//! # }
//! ```

use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::service_line::{boarding, daycare, grooming, retail, training};
use domain::operations::{pet_resort, service_core};

pub use crate::service_line::{
    grooming::StoredCadenceWeeksError,
    training::{
        StoredProgramDurationWeeks as StoredTrainingProgramDurationWeeks,
        StoredProgramDurationWeeksError as StoredTrainingProgramDurationWeeksError,
    },
};

/// Result type returned by fallible operations operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
/// Errors raised while validating Gingr configuration, request parameters, or DTO mappings.
pub enum Error {
    #[error("storage codec error")]
    /// Wraps a storage JSON codec failure without losing the underlying source error.
    Codec(#[from] CodecError),
    #[error("{record:?} storage shape mismatch: {reason:?}")]
    /// Signals that a flattened record populated fields inconsistent with its discriminator.
    StorageShapeMismatch {
        /// Record family whose flattened storage shape failed validation.
        record: RecordKind,
        /// Human-readable or typed reason explaining why storage conversion failed.
        reason: ShapeMismatchReason,
    },
    #[error("domain value rejected storage field {field:?}: {reason}")]
    /// Signals that a domain value cannot be represented safely in storage.
    InvalidDomainValue {
        /// Field attached to this Gingr error or DTO.
        field: StorageField,
        /// Provider-facing reason explaining why request construction failed.
        reason: String,
    },
}

#[derive(Debug, thiserror::Error)]
/// JSON codec failures at the storage boundary.
pub enum CodecError {
    #[error("failed to decode json: {source}")]
    /// JSON could not be decoded into the expected storage record.
    JsonDecode {
        /// Source attached to this Gingr error or DTO.
        source: serde_json::Error,
    },
    #[error("failed to encode json: {source}")]
    /// Storage record could not be serialized as JSON.
    JsonEncode {
        /// Source attached to this Gingr error or DTO.
        source: serde_json::Error,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Storage record families used in shape-validation diagnostics.
pub enum RecordKind {
    /// Stable storage code for pet resort portfolio.
    PetResortPortfolio,
    /// Stable storage code for service offering.
    ServiceOffering,
    /// Stable storage code for core service contracts.
    CoreServiceContracts,
    /// Stable storage code for data quality hygiene outcome.
    DataQualityHygieneOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Reasons a flattened record cannot represent the requested domain variant.
pub enum ShapeMismatchReason {
    /// A field required by the selected discriminator was absent.
    RequiredFieldMissing,
    /// A field from another flattened variant was populated.
    FieldBelongsToDifferentVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Persisted fields that can reject invalid domain values during storage conversion.
pub enum StorageField {
    /// Stable storage code for resort count.
    ResortCount,
    /// Stable storage code for brand name.
    BrandName,
    /// Stable storage code for grooming cadence weeks.
    GroomingCadenceWeeks,
    /// Stable storage code for training program duration weeks.
    TrainingProgramDurationWeeks,
    /// Stable storage code for manager daily brief labor minutes.
    ManagerDailyBriefLaborMinutes,
    /// Stable storage code for data quality hygiene labor minutes.
    DataQualityHygieneLaborMinutes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Provider provenance attached to stored evidence so facts can be audited back to Gingr or another source system.
pub struct StoredSourceRecordRef {
    /// Source system name, for example `gingr`, used to keep provider facts quarantined by origin.
    pub system: String,
    /// Provider record collection or endpoint that produced the evidence.
    pub record_type: String,
    /// Provider-native identifier for the source record.
    pub record_id: String,
    /// Timestamp when the adapter observed this provider fact.
    pub observed_at: String,
    /// Adapter or fixture version that interpreted the source record.
    pub adapter_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Persisted outcome states for manager daily-brief actions.
pub enum ManagerDailyBriefOutcomeCode {
    /// Workflow completed and can contribute final labor evidence.
    Completed,
    /// Workflow was postponed and should not be counted as completed savings.
    Deferred,
    /// Manager intentionally hid or skipped the suggested workflow action.
    SuppressedByManager,
    /// Provider evidence was incorrect, so the action is excluded or corrected.
    SourceFactWasWrong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Persisted staff personas accountable for manager daily-brief work.
pub enum ManagerDailyBriefPersonaCode {
    /// Stable storage code for general manager.
    GeneralManager,
    /// Stable storage code for assistant general manager.
    AssistantGeneralManager,
    /// Stable storage code for front desk lead.
    FrontDeskLead,
    /// Stable storage code for front desk agent.
    FrontDeskAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Persisted manager daily-brief actions that can produce labor-minute evidence.
pub enum ManagerDailyBriefActionKindCode {
    /// Stable storage code for review demand against staffing plan.
    ReviewDemandAgainstStaffingPlan,
    /// Stable storage code for resolve checkout exception.
    ResolveCheckoutException,
    /// Stable storage code for approve retention follow up draft.
    ApproveRetentionFollowUpDraft,
    /// Stable storage code for investigate source data quality issue.
    InvestigateSourceDataQualityIssue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Dimensions used to aggregate manager daily-brief labor outcomes by location, day, action, and owner role.
pub struct ManagerDailyBriefReportingGroup {
    /// Location whose operating day or service contract is described.
    pub location_id: String,
    /// Business date used for labor and reporting aggregation.
    pub operating_day: String,
    /// Workflow action that generated the labor evidence.
    pub action_kind: ManagerDailyBriefActionKindCode,
    /// Role expected to own or review the workflow item.
    pub owner_persona: ManagerDailyBriefPersonaCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
/// Non-zero minute quantity persisted for manager daily-brief labor evidence.
pub struct StoredManagerDailyBriefLaborMinutes(u16);

impl StoredManagerDailyBriefLaborMinutes {
    /// Validates and wraps a positive storage quantity before persistence.
    pub fn try_new(value: u16) -> Result<Self> {
        if value == 0 {
            return Err(Error::InvalidDomainValue {
                field: StorageField::ManagerDailyBriefLaborMinutes,
                reason: "must be greater than zero".to_owned(),
            });
        }

        Ok(Self(value))
    }

    /// Returns the provider numeric identifier carried by this wrapper.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for StoredManagerDailyBriefLaborMinutes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Stored evidence for a manager daily-brief action, including before/after labor minutes and source references.
pub struct ManagerDailyBriefOutcomeRecord {
    /// Stable workflow action identifier used for idempotent labor evidence.
    pub action_id: String,
    /// Final disposition recorded for the workflow action.
    pub outcome: ManagerDailyBriefOutcomeCode,
    /// Estimated manual minutes before automation or assisted workflow execution.
    pub before_minutes: StoredManagerDailyBriefLaborMinutes,
    /// Observed minutes spent after the workflow was completed or reviewed.
    pub actual_minutes: StoredManagerDailyBriefLaborMinutes,
    /// User, worker, or system actor that recorded the outcome.
    pub actor_id: String,
    /// Role of the actor that completed or reviewed the action.
    pub actor_persona: ManagerDailyBriefPersonaCode,
    /// Optional operator feedback explaining the decision or correction.
    pub feedback: String,
    #[builder(default)]
    /// Provider evidence records used to justify the workflow action.
    pub source_refs: Vec<StoredSourceRecordRef>,
    /// Timestamp when the labor evidence was written.
    pub recorded_at: String,
    /// Cross-system identifier tying the record to a workflow run or request.
    pub correlation_id: String,
    /// Location whose operating day or service contract is described.
    pub location_id: String,
    /// Business date used for labor and reporting aggregation.
    pub operating_day: String,
    /// Workflow action that generated the labor evidence.
    pub action_kind: ManagerDailyBriefActionKindCode,
    /// Role expected to own or review the workflow item.
    pub owner_persona: ManagerDailyBriefPersonaCode,
    /// Derived labor savings based on before and actual minute evidence.
    pub estimated_minutes_saved: u16,
}

impl ManagerDailyBriefOutcomeRecord {
    /// Decodes a JSON storage payload into its typed record shape.
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| CodecError::JsonDecode { source }.into())
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| CodecError::JsonEncode { source }.into())
    }

    /// Returns or constructs the Gingr actual minutes saved value.
    pub const fn actual_minutes_saved(&self) -> u16 {
        self.before_minutes
            .get()
            .saturating_sub(self.actual_minutes.get())
    }

    /// Returns the aggregation dimensions used for labor reporting.
    pub fn reporting_group(&self) -> ManagerDailyBriefReportingGroup {
        ManagerDailyBriefReportingGroup {
            location_id: self.location_id.clone(),
            operating_day: self.operating_day.clone(),
            action_kind: self.action_kind,
            owner_persona: self.owner_persona,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Persisted outcome states for data-quality hygiene actions.
pub enum DataQualityHygieneOutcomeCode {
    /// Workflow completed and can contribute final labor evidence.
    Completed,
    /// Workflow was postponed and should not be counted as completed savings.
    Deferred,
    /// Manager intentionally hid or skipped the suggested workflow action.
    SuppressedByManager,
    /// Provider evidence was incorrect, so the action is excluded or corrected.
    SourceFactWasWrong,
    /// Issue was reviewed but did not require an operational repair.
    NotActionable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Persisted personas accountable for data-quality hygiene work.
pub enum DataQualityHygienePersonaCode {
    /// Stable storage code for general manager.
    GeneralManager,
    /// Stable storage code for assistant general manager.
    AssistantGeneralManager,
    /// Stable storage code for front desk lead.
    FrontDeskLead,
    /// Stable storage code for front desk agent.
    FrontDeskAgent,
    /// Stable storage code for regional operator.
    RegionalOperator,
    /// Stable storage code for operations analyst.
    OperationsAnalyst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Persisted data-quality actions used to quarantine, repair, or reconcile source evidence.
pub enum DataQualityHygieneActionKindCode {
    /// Stable storage code for investigate missing source evidence.
    InvestigateMissingSourceEvidence,
    /// Stable storage code for reconcile duplicate customer or pet candidate.
    ReconcileDuplicateCustomerOrPetCandidate,
    /// Stable storage code for complete missing pet or customer profile fields.
    CompleteMissingPetOrCustomerProfileFields,
    /// Stable storage code for review stale vaccination source freshness.
    ReviewStaleVaccinationSourceFreshness,
    /// Stable storage code for normalize ambiguous service line naming.
    NormalizeAmbiguousServiceLineNaming,
    /// Stable storage code for review checkout or unclosed reservation evidence.
    ReviewCheckoutOrUnclosedReservationEvidence,
    /// Stable storage code for escalate sensitive or quarantined payload.
    EscalateSensitiveOrQuarantinedPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Persisted lifecycle status for a data-quality issue after review.
pub enum DataQualityResolutionStatusCode {
    /// Issue remains open after review.
    Open,
    /// Issue was accepted for later repair or monitoring.
    Acknowledged,
    /// Issue was intentionally ignored after review.
    Ignored,
    /// Issue was corrected during or after review.
    Repaired,
    /// Issue was replaced by fresher evidence or another issue record.
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Dimensions used to group data-quality hygiene outcomes by location, day, issue type, and owner role.
pub struct DataQualityHygieneReportingGroup {
    /// Location whose operating day or service contract is described.
    pub location_id: String,
    /// Business date used for labor and reporting aggregation.
    pub operating_day: String,
    /// Workflow action that generated the labor evidence.
    pub action_kind: DataQualityHygieneActionKindCode,
    /// Role expected to own or review the workflow item.
    pub owner_persona: DataQualityHygienePersonaCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
/// Non-zero minute quantity persisted for data-quality hygiene labor evidence.
pub struct StoredDataQualityHygieneLaborMinutes(u16);

impl StoredDataQualityHygieneLaborMinutes {
    /// Validates and wraps a positive storage quantity before persistence.
    pub fn try_new(value: u16) -> Result<Self> {
        if value == 0 {
            return Err(Error::InvalidDomainValue {
                field: StorageField::DataQualityHygieneLaborMinutes,
                reason: "must be greater than zero".to_owned(),
            });
        }

        Ok(Self(value))
    }

    /// Returns the provider numeric identifier carried by this wrapper.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for StoredDataQualityHygieneLaborMinutes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Stored evidence for a data-quality hygiene action, including labor deltas, issue references, and resolution state.
pub struct DataQualityHygieneOutcomeRecord {
    /// Stable workflow action identifier used for idempotent labor evidence.
    pub action_id: String,
    /// Final disposition recorded for the workflow action.
    pub outcome: DataQualityHygieneOutcomeCode,
    /// Estimated manual minutes before automation or assisted workflow execution.
    pub before_minutes: StoredDataQualityHygieneLaborMinutes,
    /// Observed minutes spent after the workflow was completed or reviewed.
    pub actual_minutes: StoredDataQualityHygieneLaborMinutes,
    /// User, worker, or system actor that recorded the outcome.
    pub actor_id: String,
    /// Role of the actor that completed or reviewed the action.
    pub actor_persona: DataQualityHygienePersonaCode,
    /// Optional operator feedback explaining the decision or correction.
    pub feedback: String,
    #[builder(default)]
    /// Provider evidence records used to justify the workflow action.
    pub source_refs: Vec<StoredSourceRecordRef>,
    #[builder(default)]
    /// Data-quality issue identifiers reviewed by the hygiene workflow.
    pub issue_refs: Vec<String>,
    /// Issue lifecycle state after the hygiene review completed.
    pub resolution_status_after_review: DataQualityResolutionStatusCode,
    /// Timestamp when the labor evidence was written.
    pub recorded_at: String,
    /// Cross-system identifier tying the record to a workflow run or request.
    pub correlation_id: String,
    /// Location whose operating day or service contract is described.
    pub location_id: String,
    /// Business date used for labor and reporting aggregation.
    pub operating_day: String,
    /// Workflow action that generated the labor evidence.
    pub action_kind: DataQualityHygieneActionKindCode,
    /// Role expected to own or review the workflow item.
    pub owner_persona: DataQualityHygienePersonaCode,
    /// Derived labor savings based on before and actual minute evidence.
    pub estimated_minutes_saved: u16,
}

impl DataQualityHygieneOutcomeRecord {
    /// Decodes a JSON storage payload into its typed record shape.
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| CodecError::JsonDecode { source }.into())
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| CodecError::JsonEncode { source }.into())
    }

    /// Returns or constructs the Gingr actual minutes saved value.
    pub const fn actual_minutes_saved(&self) -> u16 {
        self.before_minutes
            .get()
            .saturating_sub(self.actual_minutes.get())
    }

    /// Returns the aggregation dimensions used for labor reporting.
    pub fn reporting_group(&self) -> DataQualityHygieneReportingGroup {
        DataQualityHygieneReportingGroup {
            location_id: self.location_id.clone(),
            operating_day: self.operating_day.clone(),
            action_kind: self.action_kind,
            owner_persona: self.owner_persona,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Storage shape for the pet-resort portfolio facts used to seed operating assumptions.
pub struct PetResortPortfolioRecord {
    /// Portfolio operator represented by the seed record.
    pub operator: OperatorCode,
    /// Number of resorts represented by the portfolio fact.
    pub resort_count: StoredResortCount,
    /// Portfolio organization model used in operating assumptions.
    pub structure: PortfolioStructureCode,
    /// Business lines included in the portfolio fact.
    pub business_lines: Vec<BusinessLineCode>,
    /// Pet-resort brands included in the portfolio fact.
    pub brands: Vec<PetResortBrandRecord>,
}

impl PetResortPortfolioRecord {
    /// Decodes a JSON storage payload into its typed record shape.
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| CodecError::JsonDecode { source }.into())
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| CodecError::JsonEncode { source }.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable operator code used in portfolio seed records.
pub enum OperatorCode {
    #[serde(rename = "nva")]
    /// Stable storage code for national veterinary associates.
    NationalVeterinaryAssociates,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable portfolio-structure codes for pet-resort operating assumptions.
pub enum PortfolioStructureCode {
    /// Stable storage code for federated multi brand.
    FederatedMultiBrand,
    /// Stable storage code for single brand.
    SingleBrand,
    /// Provider supplied an unrecognized value; preserve it for audit instead of failing closed.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable business-line codes for NVA portfolio membership.
pub enum BusinessLineCode {
    /// Stable storage code for general practice veterinary hospitals.
    GeneralPracticeVeterinaryHospitals,
    /// Stable storage code for pet resorts.
    PetResorts,
    /// Stable storage code for equine.
    Equine,
    /// Stable storage code for specialty emergency hospitals.
    SpecialtyEmergencyHospitals,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
/// Stored pet-resort brand descriptor with code plus display name.
pub enum PetResortBrandRecord {
    /// Stable storage code for known.
    Known {
        /// Code attached to this Gingr error or DTO.
        code: PetResortBrandCode,
    },
    /// Stable storage code for other.
    Other {
        /// Name attached to this Gingr error or DTO.
        name: StoredBrandName,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable brand codes for NVA pet-resort banners.
pub enum PetResortBrandCode {
    /// Stable storage code for nva pet resorts.
    NvaPetResorts,
    /// Stable storage code for pet suites.
    PetSuites,
    /// Stable storage code for pooch hotel.
    PoochHotel,
    /// Stable storage code for elite suites.
    EliteSuites,
    /// Stable storage code for the bark side.
    TheBarkSide,
    /// Stable storage code for woofdorf astoria.
    WoofdorfAstoria,
    /// Stable storage code for doggie district.
    DoggieDistrict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive resort count persisted for portfolio seed facts.
pub struct StoredResortCount(u16);

impl StoredResortCount {
    /// Validates and wraps a positive quantity before it is persisted.
    pub const fn try_new(value: u16) -> std::result::Result<Self, StoredResortCountError> {
        if value == 0 {
            return Err(StoredResortCountError::ZeroResorts);
        }
        Ok(Self(value))
    }

    /// Returns the provider numeric identifier carried by this wrapper.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for StoredResortCount {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u16::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures for persisted resort-count quantities.
pub enum StoredResortCountError {
    #[error("stored pet resort portfolios require at least one resort")]
    /// Stable storage code for zero resorts.
    ZeroResorts,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Non-empty pet-resort brand display name persisted beside the stable brand code.
pub struct StoredBrandName(String);

impl StoredBrandName {
    /// Validates and wraps a positive storage quantity before persistence.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(Error::InvalidDomainValue {
                field: StorageField::BrandName,
                reason: "brand name cannot be empty".to_owned(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the normalized provider or storage string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<PetResortPortfolioRecord> for pet_resort::Portfolio {
    type Error = Error;

    fn try_from(record: PetResortPortfolioRecord) -> Result<Self> {
        Ok(Self::builder()
            .operator(record.operator.into())
            .resort_count(record.resort_count.try_into()?)
            .structure(record.structure.into())
            .business_lines(record.business_lines.into_iter().map(Into::into).collect())
            .brands(
                record
                    .brands
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>>>()?,
            )
            .build())
    }
}

impl TryFrom<pet_resort::Portfolio> for PetResortPortfolioRecord {
    type Error = Error;

    fn try_from(domain_portfolio: pet_resort::Portfolio) -> Result<Self> {
        Ok(Self::builder()
            .operator(domain_portfolio.operator.into())
            .resort_count(domain_portfolio.resort_count.try_into()?)
            .structure(domain_portfolio.structure.into())
            .business_lines(
                domain_portfolio
                    .business_lines
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            )
            .brands(
                domain_portfolio
                    .brands
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>>>()?,
            )
            .build())
    }
}

impl From<OperatorCode> for pet_resort::Operator {
    fn from(value: OperatorCode) -> Self {
        match value {
            OperatorCode::NationalVeterinaryAssociates => Self::NationalVeterinaryAssociates,
        }
    }
}

impl From<pet_resort::Operator> for OperatorCode {
    fn from(value: pet_resort::Operator) -> Self {
        match value {
            pet_resort::Operator::NationalVeterinaryAssociates => {
                Self::NationalVeterinaryAssociates
            }
        }
    }
}

impl From<PortfolioStructureCode> for pet_resort::PortfolioStructure {
    fn from(value: PortfolioStructureCode) -> Self {
        match value {
            PortfolioStructureCode::FederatedMultiBrand => Self::FederatedMultiBrand,
            PortfolioStructureCode::SingleBrand => Self::SingleBrand,
            PortfolioStructureCode::Unknown => Self::Unknown,
        }
    }
}

impl From<pet_resort::PortfolioStructure> for PortfolioStructureCode {
    fn from(value: pet_resort::PortfolioStructure) -> Self {
        match value {
            pet_resort::PortfolioStructure::FederatedMultiBrand => Self::FederatedMultiBrand,
            pet_resort::PortfolioStructure::SingleBrand => Self::SingleBrand,
            pet_resort::PortfolioStructure::Unknown => Self::Unknown,
        }
    }
}

impl From<BusinessLineCode> for pet_resort::BusinessLine {
    fn from(value: BusinessLineCode) -> Self {
        match value {
            BusinessLineCode::GeneralPracticeVeterinaryHospitals => {
                Self::GeneralPracticeVeterinaryHospitals
            }
            BusinessLineCode::PetResorts => Self::PetResorts,
            BusinessLineCode::Equine => Self::Equine,
            BusinessLineCode::SpecialtyEmergencyHospitals => Self::SpecialtyEmergencyHospitals,
        }
    }
}

impl From<pet_resort::BusinessLine> for BusinessLineCode {
    fn from(value: pet_resort::BusinessLine) -> Self {
        match value {
            pet_resort::BusinessLine::GeneralPracticeVeterinaryHospitals => {
                Self::GeneralPracticeVeterinaryHospitals
            }
            pet_resort::BusinessLine::PetResorts => Self::PetResorts,
            pet_resort::BusinessLine::Equine => Self::Equine,
            pet_resort::BusinessLine::SpecialtyEmergencyHospitals => {
                Self::SpecialtyEmergencyHospitals
            }
        }
    }
}

impl TryFrom<StoredResortCount> for domain::operations::ResortCount {
    type Error = Error;

    fn try_from(value: StoredResortCount) -> Result<Self> {
        domain::operations::ResortCount::try_new(value.get()).map_err(|err| {
            Error::InvalidDomainValue {
                field: StorageField::ResortCount,
                reason: err.to_string(),
            }
        })
    }
}

impl TryFrom<domain::operations::ResortCount> for StoredResortCount {
    type Error = Error;

    fn try_from(value: domain::operations::ResortCount) -> Result<Self> {
        Self::try_new(value.get()).map_err(|err| Error::InvalidDomainValue {
            field: StorageField::ResortCount,
            reason: err.to_string(),
        })
    }
}

impl TryFrom<PetResortBrandRecord> for pet_resort::Brand {
    type Error = Error;

    fn try_from(value: PetResortBrandRecord) -> Result<Self> {
        Ok(match value {
            PetResortBrandRecord::Known { code } => code.into(),
            PetResortBrandRecord::Other { name } => Self::Other {
                name: ::domain::location::Name::try_new(name.as_str()).map_err(|err| {
                    Error::InvalidDomainValue {
                        field: StorageField::BrandName,
                        reason: err.to_string(),
                    }
                })?,
            },
        })
    }
}

impl TryFrom<pet_resort::Brand> for PetResortBrandRecord {
    type Error = Error;

    fn try_from(value: pet_resort::Brand) -> Result<Self> {
        Ok(match value {
            pet_resort::Brand::NvaPetResorts => Self::Known {
                code: PetResortBrandCode::NvaPetResorts,
            },
            pet_resort::Brand::PetSuites => Self::Known {
                code: PetResortBrandCode::PetSuites,
            },
            pet_resort::Brand::PoochHotel => Self::Known {
                code: PetResortBrandCode::PoochHotel,
            },
            pet_resort::Brand::EliteSuites => Self::Known {
                code: PetResortBrandCode::EliteSuites,
            },
            pet_resort::Brand::TheBarkSide => Self::Known {
                code: PetResortBrandCode::TheBarkSide,
            },
            pet_resort::Brand::WoofdorfAstoria => Self::Known {
                code: PetResortBrandCode::WoofdorfAstoria,
            },
            pet_resort::Brand::DoggieDistrict => Self::Known {
                code: PetResortBrandCode::DoggieDistrict,
            },
            pet_resort::Brand::Other { name } => Self::Other {
                name: StoredBrandName::try_new(name.into_inner())?,
            },
        })
    }
}

impl From<PetResortBrandCode> for pet_resort::Brand {
    fn from(value: PetResortBrandCode) -> Self {
        match value {
            PetResortBrandCode::NvaPetResorts => Self::NvaPetResorts,
            PetResortBrandCode::PetSuites => Self::PetSuites,
            PetResortBrandCode::PoochHotel => Self::PoochHotel,
            PetResortBrandCode::EliteSuites => Self::EliteSuites,
            PetResortBrandCode::TheBarkSide => Self::TheBarkSide,
            PetResortBrandCode::WoofdorfAstoria => Self::WoofdorfAstoria,
            PetResortBrandCode::DoggieDistrict => Self::DoggieDistrict,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Flattened storage shape for one service-line offering; only fields valid for its `service_kind` may be populated.
pub struct ServiceOfferingRecord {
    /// Discriminator indicating which service-line fields are meaningful.
    pub service_kind: ServiceOfferingKindCode,
    /// Boarding room or suite type for a boarding offering.
    pub boarding_accommodation: Option<boarding::AccommodationCode>,
    #[builder(default)]
    /// Included care features bundled with a boarding offering.
    pub boarding_included_care: Vec<boarding::CareFeatureCode>,
    #[builder(default)]
    /// Optional boarding add-ons available for the offering.
    pub boarding_add_ons: Vec<boarding::AddOnCode>,
    /// Daycare play or day-boarding format represented by the offering.
    pub daycare_format: Option<daycare::FormatCode>,
    #[builder(default)]
    /// Eligibility requirements that must be satisfied before daycare use.
    pub daycare_eligibility_rules: Vec<daycare::EligibilityRuleCode>,
    /// Grooming service represented by the offering.
    pub grooming_service: Option<grooming::ServiceCode>,
    /// Recommended grooming repeat cadence in weeks.
    pub grooming_cadence_weeks: Option<grooming::StoredCadenceWeeks>,
    /// Training program represented by the offering.
    pub training_program: Option<training::ProgramRecord>,
    /// Retail partner product represented by the offering.
    pub retail_partner: Option<retail::PartnerCode>,
    /// Retail category used for merchandising and upsell logic.
    pub retail_product_category: Option<retail::ProductCategoryCode>,
}

impl ServiceOfferingRecord {
    /// Decodes a JSON storage payload into its typed record shape.
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| CodecError::JsonDecode { source }.into())
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| CodecError::JsonEncode { source }.into())
    }

    fn mismatch(reason: ShapeMismatchReason) -> Error {
        Error::StorageShapeMismatch {
            record: RecordKind::ServiceOffering,
            reason,
        }
    }

    fn ensure_empty_cross_variant_fields(&self, allowed: ServiceOfferingKindCode) -> Result<()> {
        let invalid = match allowed {
            ServiceOfferingKindCode::Boarding => {
                self.daycare_format.is_some()
                    || !self.daycare_eligibility_rules.is_empty()
                    || self.grooming_service.is_some()
                    || self.grooming_cadence_weeks.is_some()
                    || self.training_program.is_some()
                    || self.retail_partner.is_some()
                    || self.retail_product_category.is_some()
            }
            ServiceOfferingKindCode::Daycare => {
                self.boarding_accommodation.is_some()
                    || !self.boarding_included_care.is_empty()
                    || !self.boarding_add_ons.is_empty()
                    || self.grooming_service.is_some()
                    || self.grooming_cadence_weeks.is_some()
                    || self.training_program.is_some()
                    || self.retail_partner.is_some()
                    || self.retail_product_category.is_some()
            }
            ServiceOfferingKindCode::Grooming => {
                self.boarding_accommodation.is_some()
                    || !self.boarding_included_care.is_empty()
                    || !self.boarding_add_ons.is_empty()
                    || self.daycare_format.is_some()
                    || !self.daycare_eligibility_rules.is_empty()
                    || self.training_program.is_some()
                    || self.retail_partner.is_some()
                    || self.retail_product_category.is_some()
            }
            ServiceOfferingKindCode::Training => {
                self.boarding_accommodation.is_some()
                    || !self.boarding_included_care.is_empty()
                    || !self.boarding_add_ons.is_empty()
                    || self.daycare_format.is_some()
                    || !self.daycare_eligibility_rules.is_empty()
                    || self.grooming_service.is_some()
                    || self.grooming_cadence_weeks.is_some()
                    || self.retail_partner.is_some()
                    || self.retail_product_category.is_some()
            }
            ServiceOfferingKindCode::RetailPartnerProduct => {
                self.boarding_accommodation.is_some()
                    || !self.boarding_included_care.is_empty()
                    || !self.boarding_add_ons.is_empty()
                    || self.daycare_format.is_some()
                    || !self.daycare_eligibility_rules.is_empty()
                    || self.grooming_service.is_some()
                    || self.grooming_cadence_weeks.is_some()
                    || self.training_program.is_some()
            }
        };

        if invalid {
            Err(Self::mismatch(
                ShapeMismatchReason::FieldBelongsToDifferentVariant,
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Discriminator for the service-line variant represented by a flattened offering record.
pub enum ServiceOfferingKindCode {
    /// Stable storage code for boarding.
    Boarding,
    /// Stable storage code for daycare.
    Daycare,
    /// Stable storage code for grooming.
    Grooming,
    /// Stable storage code for training.
    Training,
    /// Stable storage code for retail partner product.
    RetailPartnerProduct,
}

impl TryFrom<domain::operations::ServiceOffering> for ServiceOfferingRecord {
    type Error = Error;

    fn try_from(value: domain::operations::ServiceOffering) -> Result<Self> {
        Ok(match value {
            domain::operations::ServiceOffering::Boarding {
                accommodation,
                included_care,
                add_ons,
            } => Self::builder()
                .service_kind(ServiceOfferingKindCode::Boarding)
                .boarding_accommodation(accommodation.into())
                .boarding_included_care(included_care.into_iter().map(Into::into).collect())
                .boarding_add_ons(add_ons.into_iter().map(Into::into).collect())
                .build(),
            domain::operations::ServiceOffering::Daycare {
                format,
                eligibility_rules,
            } => Self::builder()
                .service_kind(ServiceOfferingKindCode::Daycare)
                .daycare_format(format.into())
                .daycare_eligibility_rules(eligibility_rules.into_iter().map(Into::into).collect())
                .build(),
            domain::operations::ServiceOffering::Grooming { service, cadence } => {
                let cadence_weeks = match cadence {
                    domain::grooming::rebooking::Cadence::EveryWeeks(weeks) => {
                        Some(weeks.try_into()?)
                    }
                    domain::grooming::rebooking::Cadence::AsNeeded
                    | domain::grooming::rebooking::Cadence::GroomerRecommended
                    | domain::grooming::rebooking::Cadence::Unknown => None,
                };
                let builder = Self::builder()
                    .service_kind(ServiceOfferingKindCode::Grooming)
                    .grooming_service(service.into());
                match cadence_weeks {
                    Some(weeks) => builder.grooming_cadence_weeks(weeks).build(),
                    None => builder.build(),
                }
            }
            domain::operations::ServiceOffering::Training { program } => Self::builder()
                .service_kind(ServiceOfferingKindCode::Training)
                .training_program(program.try_into()?)
                .build(),
            domain::operations::ServiceOffering::RetailPartnerProduct { partner, category } => {
                Self::builder()
                    .service_kind(ServiceOfferingKindCode::RetailPartnerProduct)
                    .retail_partner(partner.into())
                    .retail_product_category(category.into())
                    .build()
            }
        })
    }
}

impl TryFrom<ServiceOfferingRecord> for domain::operations::ServiceOffering {
    type Error = Error;

    fn try_from(record: ServiceOfferingRecord) -> Result<Self> {
        match record.service_kind {
            ServiceOfferingKindCode::Boarding => {
                record.ensure_empty_cross_variant_fields(ServiceOfferingKindCode::Boarding)?;
                Ok(Self::Boarding {
                    accommodation: record
                        .boarding_accommodation
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .into(),
                    included_care: record
                        .boarding_included_care
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                    add_ons: record
                        .boarding_add_ons
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                })
            }
            ServiceOfferingKindCode::Daycare => {
                record.ensure_empty_cross_variant_fields(ServiceOfferingKindCode::Daycare)?;
                Ok(Self::Daycare {
                    format: record
                        .daycare_format
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .into(),
                    eligibility_rules: record
                        .daycare_eligibility_rules
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                })
            }
            ServiceOfferingKindCode::Grooming => {
                record.ensure_empty_cross_variant_fields(ServiceOfferingKindCode::Grooming)?;
                let service = record
                    .grooming_service
                    .ok_or_else(|| {
                        ServiceOfferingRecord::mismatch(ShapeMismatchReason::RequiredFieldMissing)
                    })?
                    .into();
                let cadence = match record.grooming_cadence_weeks {
                    Some(weeks) => {
                        domain::grooming::rebooking::Cadence::EveryWeeks(weeks.try_into()?)
                    }
                    None => domain::grooming::rebooking::Cadence::Unknown,
                };
                Ok(Self::Grooming { service, cadence })
            }
            ServiceOfferingKindCode::Training => {
                record.ensure_empty_cross_variant_fields(ServiceOfferingKindCode::Training)?;
                Ok(Self::Training {
                    program: record
                        .training_program
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .try_into()?,
                })
            }
            ServiceOfferingKindCode::RetailPartnerProduct => {
                record.ensure_empty_cross_variant_fields(
                    ServiceOfferingKindCode::RetailPartnerProduct,
                )?;
                Ok(Self::RetailPartnerProduct {
                    partner: record
                        .retail_partner
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .into(),
                    category: record
                        .retail_product_category
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .into(),
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage snapshot of the service-line contracts enabled for a location.
pub struct CoreServiceContractsRecord {
    /// Location whose operating day or service contract is described.
    pub location_id: domain::entities::LocationId,
    /// Boarding contract capabilities for the location.
    pub boarding: boarding::ContractRecord,
    /// Daycare contract capabilities for the location.
    pub daycare: daycare::ContractRecord,
    /// Grooming contract capabilities for the location.
    pub grooming: grooming::ContractRecord,
    /// Training contract capabilities for the location.
    pub training: training::ContractRecord,
    /// Retail contract capabilities for the location.
    pub retail: retail::ContractRecord,
}

impl CoreServiceContractsRecord {
    /// Returns or constructs the Gingr record kind value.
    pub const fn record_kind(&self) -> RecordKind {
        RecordKind::CoreServiceContracts
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|source| Error::Codec(CodecError::JsonEncode { source }))
    }

    /// Decodes a JSON storage payload into its typed record shape.
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| Error::Codec(CodecError::JsonDecode { source }))
    }
}

impl From<service_core::ServiceContracts> for CoreServiceContractsRecord {
    fn from(contracts: service_core::ServiceContracts) -> Self {
        Self {
            location_id: contracts.location_id,
            boarding: contracts.boarding.into(),
            daycare: contracts.daycare.into(),
            grooming: contracts.grooming.into(),
            training: contracts.training.into(),
            retail: contracts.retail.into(),
        }
    }
}

impl From<CoreServiceContractsRecord> for service_core::ServiceContracts {
    fn from(record: CoreServiceContractsRecord) -> Self {
        Self::builder()
            .location_id(record.location_id)
            .boarding(record.boarding.into())
            .daycare(record.daycare.into())
            .grooming(record.grooming.into())
            .training(record.training.into())
            .retail(record.retail.into())
            .build()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Stored view of the systems that produce operational data and adjacent labor signals.
pub struct TechnologyEcosystemRecord {
    /// Primary operating portal expected to originate pet-resort facts.
    pub core_portal: CoreOperatingSystemCode,
    /// Access paths available for extracting source evidence.
    pub data_access: Vec<DataAccessPatternCode>,
    /// Nearby systems that may corroborate or enrich operational evidence.
    pub adjacent_systems: Vec<AdjacentSystemCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable codes for operational source systems that may feed NVA workflows.
pub enum CoreOperatingSystemCode {
    /// Stable storage code for gingr.
    Gingr,
    /// Stable storage code for mixed systems.
    MixedSystems,
    /// Provider supplied an unrecognized value; preserve it for audit instead of failing closed.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable codes for how operational facts are accessed from source systems.
pub enum DataAccessPatternCode {
    /// Stable storage code for api.
    Api,
    /// Stable storage code for webhook.
    Webhook,
    /// Stable storage code for data export.
    DataExport,
    /// Stable storage code for warehouse.
    Warehouse,
    /// Stable storage code for business intelligence dashboard.
    BusinessIntelligenceDashboard,
    /// Provider supplied an unrecognized value; preserve it for audit instead of failing closed.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable codes for adjacent systems that provide labor, recruiting, marketing, or analytics evidence.
pub enum AdjacentSystemCode {
    /// Stable storage code for avature recruiting.
    AvatureRecruiting,
    /// Stable storage code for ga4.
    Ga4,
    /// Stable storage code for amplitude.
    Amplitude,
    /// Stable storage code for google tag manager.
    GoogleTagManager,
    /// Stable storage code for hris.
    Hris,
    /// Stable storage code for labor scheduling.
    LaborScheduling,
    /// Stable storage code for payroll.
    Payroll,
    /// Stable storage code for marketing automation.
    MarketingAutomation,
    /// Stable storage code for ticketing.
    Ticketing,
    /// Stable storage code for call center telephony.
    CallCenterTelephony,
    /// Stable storage code for reviews.
    Reviews,
    /// Stable storage code for email sms marketing.
    EmailSmsMarketing,
    /// Stable storage code for business intelligence.
    BusinessIntelligence,
    /// Stable storage code for data lake.
    DataLake,
}

impl From<domain::operations::TechnologyEcosystem> for TechnologyEcosystemRecord {
    fn from(value: domain::operations::TechnologyEcosystem) -> Self {
        Self::builder()
            .core_portal(value.core_portal.into())
            .data_access(value.data_access.into_iter().map(Into::into).collect())
            .adjacent_systems(value.adjacent_systems.into_iter().map(Into::into).collect())
            .build()
    }
}

impl From<TechnologyEcosystemRecord> for domain::operations::TechnologyEcosystem {
    fn from(value: TechnologyEcosystemRecord) -> Self {
        Self::builder()
            .core_portal(value.core_portal.into())
            .data_access(value.data_access.into_iter().map(Into::into).collect())
            .adjacent_systems(value.adjacent_systems.into_iter().map(Into::into).collect())
            .build()
    }
}

macro_rules! bidirectional_code_map {
    ($storage:ty, $domain:ty, { $($storage_variant:ident => $domain_variant:ident),+ $(,)? }) => {
        impl From<$storage> for $domain {
            fn from(value: $storage) -> Self {
                match value {
                    $(<$storage>::$storage_variant => Self::$domain_variant,)+
                }
            }
        }

        impl From<$domain> for $storage {
            fn from(value: $domain) -> Self {
                match value {
                    $(<$domain>::$domain_variant => Self::$storage_variant,)+
                }
            }
        }
    };
}

bidirectional_code_map!(CoreOperatingSystemCode, service_core::OperatingSystem, {
    Gingr => Gingr,
    MixedSystems => MixedSystems,
    Unknown => Unknown,
});

bidirectional_code_map!(DataAccessPatternCode, domain::operations::DataAccessPattern, {
    Api => Api,
    Webhook => Webhook,
    DataExport => DataExport,
    Warehouse => Warehouse,
    BusinessIntelligenceDashboard => BusinessIntelligenceDashboard,
    Unknown => Unknown,
});

bidirectional_code_map!(AdjacentSystemCode, domain::operations::AdjacentSystem, {
    AvatureRecruiting => AvatureRecruiting,
    Ga4 => Ga4,
    Amplitude => Amplitude,
    GoogleTagManager => GoogleTagManager,
    Hris => Hris,
    LaborScheduling => LaborScheduling,
    Payroll => Payroll,
    MarketingAutomation => MarketingAutomation,
    Ticketing => Ticketing,
    CallCenterTelephony => CallCenterTelephony,
    Reviews => Reviews,
    EmailSmsMarketing => EmailSmsMarketing,
    BusinessIntelligence => BusinessIntelligence,
    DataLake => DataLake,
});
