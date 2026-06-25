//! Persistence records for app/domain operational rules.
//!
//! This module documents the storage/public projection gate for the
//! pet-resort AI program: portfolio seed facts, service-line offerings, core
//! service rules, manager daily-brief labor outcomes, data-quality hygiene
//! outcomes, and source-system ecosystem records. Storage code is allowed to
//! speak in stable record codes, flattened optional fields, and JSON payloads,
//! but promotion back into `domain` values is explicit and source-grounded.
//!
//! The gate is deliberately narrow:
//!
//! - `domain` owns business meaning and invariants such as daycare eligibility,
//!   grooming cadence, training duration, source evidence, and review gates.
//! - `storage` owns durable representations, discriminator checks, codec errors,
//!   and idempotent evidence records suitable for Postgres or fixtures.
//! - `app` and runtime crates decide when a workflow may read or write records;
//!   storage records never authorize live provider writes or customer messaging.
//! - `integration` adapters attach `StoredSourceRecordRef` values so a derived
//!   record can be audited back to Gingr, a warehouse export, or another source
//!   instead of becoming an invented operational fact.
//!
//! Crosswalk navigation: this module backs the storage/persistence rows for
//! outcome records, source refs, service offerings, portfolio records, and
//! reporting groups. Use
//! `docs/entity-atlas/contract-crosswalk/storage-persistence.md` from entity
//! pages, `workflow-packets.md` from workflow pages, and the storage/API tests
//! named there as the executable proof.
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
use serde_json::json;
use std::collections::BTreeSet;

use crate::service_line::{boarding, daycare, grooming, retail, training};
use domain::operations::{pet_resort, service_core};

pub use crate::service_line::{
    grooming::StoredCadenceWeeksError,
    training::{
        StoredProgramDurationWeeks as StoredTrainingProgramDurationWeeks,
        StoredProgramDurationWeeksError as StoredTrainingProgramDurationWeeksError,
    },
};

/// Result type returned by fallible storage projection and codec operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
/// Errors raised while validating storage records, codecs, or domain-to-storage projection.
pub enum Error {
    #[error("storage codec error: {0}")]
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
        /// Storage field whose value failed projection or validation.
        field: StorageField,
        /// Human-readable reason explaining why the storage projection was unsafe.
        reason: String,
    },
}

#[derive(Debug, thiserror::Error)]
/// JSON codec failures at the storage gate.
pub enum CodecError {
    #[error("failed to decode {record:?} json: {source}")]
    /// JSON could not be decoded into the expected storage record family.
    JsonDecode {
        /// Storage record family expected at this decode boundary.
        record: RecordKind,
        /// Underlying serde error raised while decoding the stored payload.
        source: serde_json::Error,
    },
    #[error("failed to encode {record:?} json: {source}")]
    /// Storage record could not be serialized as JSON for its record family.
    JsonEncode {
        /// Storage record family being encoded.
        record: RecordKind,
        /// Underlying serde error raised while encoding the stored payload.
        source: serde_json::Error,
    },
}

impl CodecError {
    fn decode(record: RecordKind, source: serde_json::Error) -> Self {
        Self::JsonDecode { record, source }
    }

    fn encode(record: RecordKind, source: serde_json::Error) -> Self {
        Self::JsonEncode { record, source }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Storage record families used in shape-validation diagnostics.
pub enum RecordKind {
    /// Portfolio seed facts used to orient multi-brand NVA pet-resort assumptions.
    PetResortPortfolio,
    /// Flattened record for one boarding, daycare, grooming, training, or retail offering.
    ServiceOffering,
    /// Location-level snapshot of enabled service-line rules.
    CoreServiceContracts,
    /// Manager-facing daily-brief labor evidence emitted by the booking triage workflow.
    ManagerDailyBriefOutcome,
    /// Labor-evidence record for a data-quality hygiene workflow outcome.
    DataQualityHygieneOutcome,
    /// Source-quality backlog issue backing Data-Quality Hygiene BI read models.
    DataQualityIssue,
    /// Read-only source import/freshness run for data-quality caveats.
    DataQualitySourceImportRun,
    /// Source synchronization gap row for import/read-model freshness caveats.
    DataQualitySyncGap,
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
    /// Resort-count field promoted into a positive domain count.
    ResortCount,
    /// Freeform brand-name field preserved for non-enumerated pet-resort banners.
    BrandName,
    /// Grooming cadence quantity persisted in weeks when cadence is known.
    GroomingCadenceWeeks,
    /// Training program duration quantity persisted in weeks.
    TrainingProgramDurationWeeks,
    /// Manager daily-brief labor-minute field used for before/after evidence.
    ManagerDailyBriefLaborMinutes,
    /// Data-quality hygiene labor-minute field used for before/after evidence.
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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
    /// Location whose operating day or service rules is described.
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
    /// Validates and wraps a non-empty brand name before persistence.
    pub fn try_new(value: u16) -> Result<Self> {
        if value == 0 {
            return Err(Error::InvalidDomainValue {
                field: StorageField::ManagerDailyBriefLaborMinutes,
                reason: "must be greater than zero".to_owned(),
            });
        }

        Ok(Self(value))
    }

    /// Returns the validated numeric quantity kept on this storage wrapper.
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
    /// Location whose operating day or service rules is described.
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
        serde_json::from_str(raw).map_err(|source| {
            CodecError::decode(RecordKind::ManagerDailyBriefOutcome, source).into()
        })
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| {
            CodecError::encode(RecordKind::ManagerDailyBriefOutcome, source).into()
        })
    }

    /// Returns the derived minutes saved from before/after labor evidence.
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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
    /// Stable storage code for protected payment conflict review without money movement.
    ReviewPaymentStateConflict,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Durable issue categories backing the Data-Quality Hygiene source-quality backlog.
pub enum DataQualityIssueKindCode {
    /// Expected source evidence is absent or not linked to an operational entity.
    MissingSourceEvidence,
    /// Source records appear to describe the same customer or pet more than once.
    DuplicateEntityCandidate,
    /// Source record lacks a field needed for safe operations or review.
    MissingRequiredField,
    /// Source record is older than the workflow's freshness posture allows.
    StaleSourceFreshness,
    /// Source vocabulary does not map cleanly to an owned operations concept.
    AmbiguousServiceLineNaming,
    /// Checkout/reservation source facts are incomplete or not closed out.
    UnclosedReservationEvidence,
    /// Payload requires quarantine or narrower review before ordinary operations use.
    SensitivePayloadQuarantine,
    /// Payment state conflict is tracked for review without authorizing money movement.
    PaymentStateConflict,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Severity code for operator/BI prioritization of source-quality issues and sync gaps.
pub enum DataQualitySeverityCode {
    /// Low-priority issue that can be batched.
    Low,
    /// Medium-priority issue that needs normal operational review.
    Medium,
    /// High-priority issue that blocks or materially degrades workflow confidence.
    High,
    /// Critical issue requiring immediate review before relying on derived output.
    Critical,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Freshness posture for source evidence represented in read models.
pub enum DataQualityFreshnessCode {
    /// Source evidence is current enough for ordinary review.
    Current,
    /// Source evidence is stale and should be caveated before BI/operator use.
    Stale,
    /// Freshness could not be determined from the available import evidence.
    Unknown,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Sensitivity posture for data-quality issue evidence and read-model caveats.
pub enum DataQualitySensitivityCode {
    /// Operational metadata with no customer/provider payload exposure.
    OperationalMetadata,
    /// Ordinary customer or pet profile details may be implicated.
    CustomerOrPetProfile,
    /// Medical/vaccination evidence needs narrower review and redaction.
    MedicalOrVaccination,
    /// Payment evidence may be implicated; no money movement is authorized here.
    PaymentState,
    /// Payload is sensitive or quarantined and ordinary BI rows must stay redacted.
    Quarantined,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Owned entity kind affected by a source-quality issue.
pub enum AffectedEntityKindCode {
    /// Customer profile or identity evidence.
    Customer,
    /// Pet profile, vaccination, eligibility, or care evidence.
    Pet,
    /// Reservation/checkout/stay evidence.
    Reservation,
    /// Location-level operating/source coverage evidence.
    Location,
    /// Source-only record that has not mapped safely to an owned entity yet.
    SourceRecord,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Whether a source-quality issue blocks an owned workflow or remains advisory.
pub enum DataQualityWorkflowBlockingCode {
    /// Issue blocks or gates a workflow decision until reviewed.
    Blocking,
    /// Issue is visible for repair/BI but does not stop the workflow.
    NonBlocking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Durable source-quality issue row that backs the Data-Quality Hygiene backlog read model.
pub struct DataQualityIssueRecord {
    /// Stable issue identifier used in workflow outcomes and BI lineage.
    pub issue_ref: String,
    /// Location whose source evidence is affected.
    pub location_id: String,
    /// Optional tenant namespace when multi-tenant persistence is enabled.
    pub tenant_id: Option<String>,
    /// Owned entity family affected by the source issue.
    pub affected_entity_kind: AffectedEntityKindCode,
    /// Owned entity identifier or source-only identifier affected by the issue.
    pub affected_entity_id: String,
    /// Domain/source field path needing review.
    pub field_path: String,
    /// Semantic issue category.
    pub issue_kind: DataQualityIssueKindCode,
    /// Operator/BI prioritization severity.
    pub severity: DataQualitySeverityCode,
    /// Source freshness posture.
    pub freshness: DataQualityFreshnessCode,
    /// Sensitivity/redaction posture.
    pub sensitivity: DataQualitySensitivityCode,
    /// Whether the issue blocks the workflow until review.
    pub workflow_blocking: DataQualityWorkflowBlockingCode,
    /// Staff/operating persona accountable for review or repair.
    pub owner_persona: String,
    /// Review gate required before operational handoff.
    pub review_gate: ReviewGateCode,
    /// Current reviewed lifecycle state.
    pub resolution_status: DataQualityResolutionStatusCode,
    #[builder(default)]
    /// Source records supporting this issue; no raw provider payload is embedded.
    pub source_refs: Vec<StoredSourceRecordRef>,
    /// Linked workflow event when the issue is being processed or reviewed.
    pub workflow_event_id: Option<String>,
    /// Issue creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
    /// Resolution timestamp when the issue is no longer open.
    pub resolved_at: Option<String>,
}

impl DataQualityIssueRecord {
    /// Decodes a JSON storage payload into its typed source-quality issue shape.
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw)
            .map_err(|source| CodecError::decode(RecordKind::DataQualityIssue, source).into())
    }

    /// Encodes the source-quality issue as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|source| CodecError::encode(RecordKind::DataQualityIssue, source).into())
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Read-only import mode used by source freshness rows.
pub enum DataQualitySourceImportModeCode {
    /// Adapter collected a safe snapshot without writing to the provider.
    ReadOnlySnapshot,
    /// Adapter executed a dry-run mapping pass without live provider mutation.
    DryRunMapping,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Import run status used to caveat source-quality read models.
pub enum DataQualitySourceImportStatusCode {
    /// Import run is still pending.
    Pending,
    /// Import completed without known row rejections.
    Completed,
    /// Import completed but rejected some source rows.
    CompletedWithRejections,
    /// Import failed before producing trustworthy coverage.
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Read-only source import run used to prove freshness and coverage caveats.
pub struct DataQualitySourceImportRunRecord {
    /// Import run primary key.
    pub id: String,
    /// Source system observed by the adapter.
    pub source_system: String,
    /// Adapter version that interpreted the source payloads.
    pub adapter_version: String,
    /// Location whose records were imported or checked.
    pub location_id: String,
    /// Optional tenant namespace when multi-tenant persistence is enabled.
    pub tenant_id: Option<String>,
    /// Read-only import mode.
    pub mode: DataQualitySourceImportModeCode,
    /// Import completion status.
    pub status: DataQualitySourceImportStatusCode,
    /// Run start timestamp.
    pub started_at: String,
    /// Run completion timestamp when available.
    pub completed_at: Option<String>,
    /// Number of source records observed.
    pub record_count: u32,
    /// Number of records rejected after safe validation/mapping.
    pub rejected_count: u32,
    /// Redacted failure class; never raw provider payload or credentials.
    pub safe_error_class: Option<String>,
    /// Redaction posture for payload storage/references.
    pub redaction_posture: String,
    /// Record creation timestamp.
    pub created_at: String,
}

impl DataQualitySourceImportRunRecord {
    /// Decodes a JSON storage payload into its typed import-run shape.
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| {
            CodecError::decode(RecordKind::DataQualitySourceImportRun, source).into()
        })
    }

    /// Encodes the import-run record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| {
            CodecError::encode(RecordKind::DataQualitySourceImportRun, source).into()
        })
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Sync/import gap kind used to caveat source-quality backlog and freshness rows.
pub enum DataQualitySyncGapKindCode {
    /// A source record expected by coverage checks was missing.
    MissingExpectedRecord,
    /// A source record was stale relative to the operating date.
    StaleExpectedRecord,
    /// A source record could not map safely to an owned entity.
    MappingUncertain,
    /// Adapter/source failure prevented freshness proof.
    AdapterFailure,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Lifecycle status for sync/import gaps.
pub enum DataQualitySyncGapStatusCode {
    /// Gap is open and must caveat read models.
    Open,
    /// Gap is acknowledged but not repaired.
    Acknowledged,
    /// Gap was resolved by fresher import or repair.
    Resolved,
    /// Gap was superseded by another issue or import run.
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Durable sync/import gap row used by import freshness and backlog caveats.
pub struct DataQualitySyncGapRecord {
    /// Sync gap primary key.
    pub id: String,
    /// Source system whose coverage/freshness is affected.
    pub source_system: String,
    /// Optional source ref associated with the gap; raw payload is not embedded.
    pub source_ref: Option<StoredSourceRecordRef>,
    /// Location whose import/read model is affected.
    pub location_id: String,
    /// Optional tenant namespace when multi-tenant persistence is enabled.
    pub tenant_id: Option<String>,
    /// Gap category.
    pub gap_kind: DataQualitySyncGapKindCode,
    /// Gap severity.
    pub severity: DataQualitySeverityCode,
    /// Timestamp when the gap was detected.
    pub detected_at: String,
    /// Age of the gap at projection time.
    pub age_seconds: u64,
    /// Gap lifecycle status.
    pub status: DataQualitySyncGapStatusCode,
    /// Linked workflow event when review/repair is underway.
    pub workflow_event_id: Option<String>,
    /// Redacted failure class; never raw provider payload or credentials.
    pub safe_error_class: Option<String>,
    /// Record creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

impl DataQualitySyncGapRecord {
    /// Decodes a JSON storage payload into its typed sync-gap shape.
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw)
            .map_err(|source| CodecError::decode(RecordKind::DataQualitySyncGap, source).into())
    }

    /// Encodes the sync-gap record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|source| CodecError::encode(RecordKind::DataQualitySyncGap, source).into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// BI-safe source-quality backlog projection; it exposes dimensions and lineage, not raw provider payloads.
pub struct SourceQualityBacklogRow {
    /// Stable issue identifier used for drill-through.
    pub issue_ref: String,
    /// Location dimension.
    pub location_id: String,
    /// Optional tenant dimension.
    pub tenant_id: Option<String>,
    /// Affected owned entity kind.
    pub affected_entity_kind: AffectedEntityKindCode,
    /// Affected owned entity id or safe source-only id.
    pub affected_entity_id: String,
    /// Field path needing review.
    pub field_path: String,
    /// Issue category dimension.
    pub issue_kind: DataQualityIssueKindCode,
    /// Severity dimension.
    pub severity: DataQualitySeverityCode,
    /// Freshness dimension.
    pub freshness: DataQualityFreshnessCode,
    /// Sensitivity/redaction dimension.
    pub sensitivity: DataQualitySensitivityCode,
    /// Workflow blocking dimension.
    pub workflow_blocking: DataQualityWorkflowBlockingCode,
    /// Staff/operating owner persona.
    pub owner_persona: String,
    /// Review gate dimension.
    pub review_gate: ReviewGateCode,
    /// Current resolution state.
    pub resolution_status: DataQualityResolutionStatusCode,
    /// Source lineage references used by BI or operators to audit the row.
    pub source_refs: Vec<StoredSourceRecordRef>,
    /// Workflow event currently linked to the issue, if any.
    pub workflow_event_id: Option<String>,
    /// Latest reviewed hygiene outcome linked to the issue, if known.
    pub latest_outcome_id: Option<String>,
    /// Stable projection contract version.
    pub projection_version: String,
    /// Caveats such as `raw_payload_redacted` or `live_side_effects_disabled`.
    pub caveats: Vec<String>,
}

impl SourceQualityBacklogRow {
    /// Builds a BI-safe backlog row from a durable issue record and optional latest outcome lineage.
    pub fn from_issue(
        issue: DataQualityIssueRecord,
        latest_outcome_id: Option<String>,
        projection_version: String,
        caveats: Vec<String>,
    ) -> Self {
        Self {
            issue_ref: issue.issue_ref,
            location_id: issue.location_id,
            tenant_id: issue.tenant_id,
            affected_entity_kind: issue.affected_entity_kind,
            affected_entity_id: issue.affected_entity_id,
            field_path: issue.field_path,
            issue_kind: issue.issue_kind,
            severity: issue.severity,
            freshness: issue.freshness,
            sensitivity: issue.sensitivity,
            workflow_blocking: issue.workflow_blocking,
            owner_persona: issue.owner_persona,
            review_gate: issue.review_gate,
            resolution_status: issue.resolution_status,
            source_refs: issue.source_refs,
            workflow_event_id: issue.workflow_event_id,
            latest_outcome_id,
            projection_version,
            caveats,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// BI-safe import freshness projection for caveating backlog rows.
pub struct ImportFreshnessRow {
    /// Source system dimension.
    pub source_system: String,
    /// Location dimension.
    pub location_id: String,
    /// Timestamp of the latest completed or completed-with-rejections import.
    pub last_completed_at: Option<String>,
    /// Most recent adapter version observed for this source/location.
    pub adapter_version: Option<String>,
    /// Total records observed in matching import runs.
    pub record_count: u32,
    /// Total rejected rows in matching import runs.
    pub rejected_count: u32,
    /// Count of failed import runs.
    pub failed_import_count: usize,
    /// Count of open sync gaps.
    pub open_gap_count: usize,
    /// Stable projection contract version.
    pub projection_version: String,
    /// BI caveats derived from rejected imports, failed imports, or open gaps.
    pub caveats: Vec<String>,
}

impl ImportFreshnessRow {
    /// Builds a BI-safe import freshness projection from import runs and sync gaps.
    pub fn from_import_runs_and_sync_gaps(
        source_system: &str,
        location_id: &str,
        import_runs: &[DataQualitySourceImportRunRecord],
        sync_gaps: &[DataQualitySyncGapRecord],
        projection_version: String,
    ) -> Self {
        let matching_runs: Vec<_> = import_runs
            .iter()
            .filter(|run| run.source_system == source_system && run.location_id == location_id)
            .collect();
        let record_count = matching_runs
            .iter()
            .fold(0_u32, |total, run| total.saturating_add(run.record_count));
        let rejected_count = matching_runs
            .iter()
            .fold(0_u32, |total, run| total.saturating_add(run.rejected_count));
        let failed_import_count = matching_runs
            .iter()
            .filter(|run| run.status == DataQualitySourceImportStatusCode::Failed)
            .count();
        let latest_completed = matching_runs
            .iter()
            .filter(|run| {
                matches!(
                    run.status,
                    DataQualitySourceImportStatusCode::Completed
                        | DataQualitySourceImportStatusCode::CompletedWithRejections
                )
            })
            .filter_map(|run| {
                run.completed_at
                    .as_ref()
                    .map(|completed_at| (completed_at, run))
            })
            .max_by(|(left, _), (right, _)| left.cmp(right));
        let open_gap_count = sync_gaps
            .iter()
            .filter(|gap| {
                gap.source_system == source_system
                    && gap.location_id == location_id
                    && gap.status == DataQualitySyncGapStatusCode::Open
            })
            .count();

        let mut caveats = Vec::new();
        if rejected_count > 0 {
            caveats.push("source_import_had_rejections".to_owned());
        }
        if failed_import_count > 0 {
            caveats.push("source_import_failed".to_owned());
        }
        if open_gap_count > 0 {
            caveats.push("open_sync_gaps".to_owned());
        }

        Self {
            source_system: source_system.to_owned(),
            location_id: location_id.to_owned(),
            last_completed_at: latest_completed.map(|(completed_at, _)| (*completed_at).clone()),
            adapter_version: latest_completed.map(|(_, run)| run.adapter_version.clone()),
            record_count,
            rejected_count,
            failed_import_count,
            open_gap_count,
            projection_version,
            caveats,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Dimensions used to group data-quality hygiene outcomes by location, day, issue type, and owner role.
pub struct DataQualityHygieneReportingGroup {
    /// Location whose operating day or service rules is described.
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

    /// Returns the validated resort count kept on this storage wrapper.
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
    /// Location whose operating day or service rules is described.
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
        serde_json::from_str(raw).map_err(|source| {
            CodecError::decode(RecordKind::DataQualityHygieneOutcome, source).into()
        })
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| {
            CodecError::encode(RecordKind::DataQualityHygieneOutcome, source).into()
        })
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Reviewed data-quality hygiene outcome rollup for location/day/correlation reporting.
///
/// The summary keeps source refs and issue refs visible while aggregating reviewed
/// labor evidence and dispositions. It is a reporting readout only; it does not
/// authorize provider writes, customer sends, schedule changes, or payment movement.
pub struct DataQualityHygieneOutcomeSummary {
    /// Location whose reviewed hygiene outcomes are summarized.
    pub location_id: String,
    /// Business date whose reviewed hygiene outcomes are summarized.
    pub operating_day: String,
    /// Optional workflow correlation filter used to compare one run or request.
    pub correlation_id: Option<String>,
    /// Count of stored reviewed outcome records in scope.
    pub reviewed_outcome_count: usize,
    /// Completed outcomes that may support actual labor-savings evidence.
    pub completed_count: usize,
    /// Deferred outcomes kept visible but excluded from completed-savings proof.
    pub deferred_count: usize,
    /// Outcomes where source evidence was wrong and must not be hidden.
    pub wrong_source_count: usize,
    /// Outcomes reviewed as not actionable.
    pub not_actionable_count: usize,
    /// Outcomes suppressed by a manager or reviewer.
    pub suppressed_by_manager_count: usize,
    /// Sum of action-level estimated saved minutes in scope.
    pub total_estimated_minutes_saved: u16,
    /// Sum of actual reviewed minutes spent in scope.
    pub total_actual_minutes_spent: u16,
    /// Sum of actual saved minutes for completed outcomes only.
    pub completed_actual_minutes_saved: u16,
    /// Source-record evidence retained for audit and reconciliation.
    pub source_refs: Vec<StoredSourceRecordRef>,
    /// Data-quality issue identifiers retained for audit and reconciliation.
    pub issue_refs: Vec<String>,
}

impl DataQualityHygieneOutcomeSummary {
    /// Aggregates reviewed outcome records by location, operating day, and optional correlation.
    pub fn from_records(
        records: &[DataQualityHygieneOutcomeRecord],
        location_id: &str,
        operating_day: &str,
        correlation_id: Option<&str>,
    ) -> Self {
        let mut summary = Self {
            location_id: location_id.to_owned(),
            operating_day: operating_day.to_owned(),
            correlation_id: correlation_id.map(str::to_owned),
            reviewed_outcome_count: 0,
            completed_count: 0,
            deferred_count: 0,
            wrong_source_count: 0,
            not_actionable_count: 0,
            suppressed_by_manager_count: 0,
            total_estimated_minutes_saved: 0,
            total_actual_minutes_spent: 0,
            completed_actual_minutes_saved: 0,
            source_refs: Vec::new(),
            issue_refs: Vec::new(),
        };
        let mut issue_refs = BTreeSet::new();

        for record in records.iter().filter(|record| {
            record.location_id == location_id
                && record.operating_day == operating_day
                && correlation_id
                    .is_none_or(|correlation_id| record.correlation_id == correlation_id)
        }) {
            summary.reviewed_outcome_count += 1;
            match record.outcome {
                DataQualityHygieneOutcomeCode::Completed => {
                    summary.completed_count += 1;
                    summary.completed_actual_minutes_saved = summary
                        .completed_actual_minutes_saved
                        .saturating_add(record.actual_minutes_saved());
                }
                DataQualityHygieneOutcomeCode::Deferred => summary.deferred_count += 1,
                DataQualityHygieneOutcomeCode::SuppressedByManager => {
                    summary.suppressed_by_manager_count += 1;
                }
                DataQualityHygieneOutcomeCode::SourceFactWasWrong => {
                    summary.wrong_source_count += 1;
                }
                DataQualityHygieneOutcomeCode::NotActionable => summary.not_actionable_count += 1,
            }
            summary.total_estimated_minutes_saved = summary
                .total_estimated_minutes_saved
                .saturating_add(record.estimated_minutes_saved);
            summary.total_actual_minutes_spent = summary
                .total_actual_minutes_spent
                .saturating_add(record.actual_minutes.get());
            summary.source_refs.extend(record.source_refs.clone());
            issue_refs.extend(record.issue_refs.iter().cloned());
        }

        summary.issue_refs = issue_refs.into_iter().collect();
        summary
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Caller-supplied identifiers used to project one reviewed Data-Quality Hygiene outcome into durable local-demo rows.
pub struct DataQualityHygieneLineageIds {
    /// Durable workflow event id supplied by the repository adapter before insert.
    pub workflow_event_id: String,
    /// Durable review packet id supplied by the repository adapter before insert.
    pub review_packet_id: String,
    /// Durable approval record id supplied by the repository adapter before insert.
    pub approval_record_id: String,
    /// Durable outbox record id supplied by the repository adapter before insert.
    pub outbox_record_id: String,
    /// Location or other subject id used by the local-demo workflow event row.
    pub subject_id: String,
    /// Idempotency key used by the workflow event row and derived outbox candidate.
    pub idempotency_key: String,
    /// Stable timestamp copied into row projections for deterministic tests and replay.
    pub recorded_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Actor kinds accepted by MVP workflow, approval, audit, and outbox tables.
pub enum ActorKindCode {
    /// Customer actor persisted at the storage boundary.
    Customer,
    /// Staff actor persisted at the storage boundary.
    Staff,
    /// Manager actor persisted at the storage boundary.
    Manager,
    /// System actor persisted at the storage boundary.
    System,
    /// Agent actor persisted at the storage boundary.
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Review gates accepted by the MVP review/approval/outbox migration.
pub enum ReviewGateCode {
    /// Manager approval gate for internal handoff and data-quality review.
    ManagerApproval,
    /// Medical document review gate.
    MedicalDocumentReview,
    /// Behavior review gate.
    BehaviorReview,
    /// Customer message approval gate.
    CustomerMessageApproval,
    /// Refund or deposit exception gate.
    RefundOrDepositException,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Review packet statuses accepted by the MVP review packet table.
pub enum ReviewPacketStatusCode {
    /// Draft packet not yet ready for review.
    Draft,
    /// Packet prepared for review.
    ReadyForReview,
    /// Packet under review.
    InReview,
    /// Packet approved by the appropriate review gate.
    Approved,
    /// Packet rejected by review.
    Rejected,
    /// Packet cancelled before completion.
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Workflow result statuses accepted by the MVP workflow results table.
pub enum WorkflowResultStatusCode {
    /// Workflow completed locally without enabling live side effects.
    Succeeded,
    /// Workflow failed before reviewable output.
    Failed,
    /// Workflow produced reviewable output that still needs review.
    NeedsReview,
    /// Workflow was deferred.
    Deferred,
    /// Workflow was cancelled.
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Outbox statuses accepted by the MVP outbox table.
pub enum OutboxStatusCode {
    /// Candidate is available for local/internal review processing only.
    Pending,
    /// Candidate was claimed by a worker.
    Claimed,
    /// Candidate was published by a future approved adapter.
    Published,
    /// Candidate failed but may retry.
    Failed,
    /// Candidate failed permanently.
    DeadLetter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped workflow event row for the local Data-Quality Hygiene demo slice.
pub struct WorkflowEventRecord {
    /// Workflow event primary key.
    pub id: String,
    /// Semantic workflow name persisted in `workflow_events.workflow_name`.
    pub workflow_name: String,
    /// Event kind persisted in `workflow_events.event_kind`.
    pub event_kind: String,
    /// Subject family persisted in `workflow_events.subject_kind`.
    pub subject_kind: String,
    /// Subject id persisted in `workflow_events.subject_id`.
    pub subject_id: String,
    /// Idempotency key persisted in `workflow_events.idempotency_key`.
    pub idempotency_key: String,
    /// JSON payload for source refs, issue refs, correlation evidence, and safety posture.
    pub payload: serde_json::Value,
    /// Event occurrence timestamp.
    pub occurred_at: String,
    /// Storage record timestamp.
    pub recorded_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped workflow result row for reviewable fake/deterministic output.
pub struct WorkflowResultRecord {
    /// Workflow result primary key or derived local identifier.
    pub id: String,
    /// Parent workflow event id.
    pub workflow_event_id: String,
    /// Result status accepted by `workflow_results.status`.
    pub status: WorkflowResultStatusCode,
    /// Reviewable result payload; never execution proof for live side effects.
    pub result: serde_json::Value,
    /// Optional local error code for failed results.
    pub error_code: Option<String>,
    /// Result creation timestamp.
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped review packet row for the manager/front-desk review gate.
pub struct ReviewPacketRecord {
    /// Review packet primary key.
    pub id: String,
    /// Subject family persisted for review.
    pub subject_kind: String,
    /// Subject id persisted for review.
    pub subject_id: String,
    /// Review gate required before handoff.
    pub gate: ReviewGateCode,
    /// Review packet status.
    pub status: ReviewPacketStatusCode,
    /// Linked workflow event id.
    pub workflow_event_id: String,
    /// Actor kind that prepared the packet.
    pub created_by_actor_kind: ActorKindCode,
    /// Actor id that prepared the packet.
    pub created_by_actor_id: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Update timestamp.
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped approval row for a reviewed local handoff candidate.
pub struct ApprovalRecordRow {
    /// Approval primary key.
    pub id: String,
    /// Target aggregate kind matching the outbox candidate.
    pub target_kind: String,
    /// Target aggregate id matching the outbox candidate.
    pub target_id: String,
    /// Review gate used for the decision.
    pub gate: ReviewGateCode,
    /// Approval status string accepted by the migration.
    pub status: String,
    /// Actor kind that requested approval.
    pub requested_by_actor_kind: ActorKindCode,
    /// Actor id that requested approval.
    pub requested_by_actor_id: String,
    /// Request timestamp.
    pub requested_at: String,
    /// Actor kind that decided approval, present only for approved/rejected rows.
    pub decided_by_actor_kind: Option<ActorKindCode>,
    /// Actor id that decided approval, present only for approved/rejected rows.
    pub decided_by_actor_id: Option<String>,
    /// Decision timestamp, present only for approved/rejected rows.
    pub decided_at: Option<String>,
    /// Linked review packet id.
    pub review_packet_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped Data-Quality Hygiene outcome row with workflow and approval foreign keys.
pub struct DataQualityHygieneOutcomeRow {
    /// Parent workflow event id.
    pub workflow_event_id: String,
    /// Parent approval record id.
    pub approval_record_id: String,
    /// Typed storage outcome payload.
    pub record: DataQualityHygieneOutcomeRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped audit event row for append-only local proof.
pub struct AuditEventRecord {
    /// Actor kind that produced the audit event.
    pub actor_kind: ActorKindCode,
    /// Actor id that produced the audit event.
    pub actor_id: String,
    /// Audited subject family.
    pub subject_kind: String,
    /// Audited subject id.
    pub subject_id: String,
    /// Audit action.
    pub action: String,
    /// Linked workflow event id.
    pub workflow_event_id: String,
    /// Metadata proving source refs, issue refs, and side-effect posture.
    pub metadata: serde_json::Value,
    /// Event occurrence timestamp.
    pub occurred_at: String,
    /// Storage record timestamp.
    pub recorded_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped outbox candidate row; it is an approved local/internal handoff candidate, not a send.
pub struct OutboxRecord {
    /// Outbox primary key.
    pub id: String,
    /// Idempotency key for the candidate.
    pub idempotency_key: String,
    /// Matching approved approval record id.
    pub approval_record_id: String,
    /// Internal topic only; no customer/provider/payment/schedule topics are produced here.
    pub topic: String,
    /// Review gate matching the approval row.
    pub review_gate: ReviewGateCode,
    /// Aggregate kind matching the approval row.
    pub aggregate_kind: String,
    /// Aggregate id matching the approval row.
    pub aggregate_id: String,
    /// Candidate payload for local/internal handoff.
    pub payload: serde_json::Value,
    /// Candidate status.
    pub status: OutboxStatusCode,
    /// Availability timestamp.
    pub available_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Complete storage projection for one reviewed local Data-Quality Hygiene workflow outcome.
pub struct DataQualityHygieneLocalPersistenceRecords {
    /// Workflow event row.
    pub workflow_event: WorkflowEventRecord,
    /// Workflow result row.
    pub workflow_result: WorkflowResultRecord,
    /// Review packet row.
    pub review_packet: ReviewPacketRecord,
    /// Approval record row.
    pub approval_record: ApprovalRecordRow,
    /// Outcome row linked to workflow and approval rows.
    pub outcome: DataQualityHygieneOutcomeRow,
    /// Append-only audit rows for context creation and reviewed outcome capture.
    pub audit_events: Vec<AuditEventRecord>,
    /// Optional approved internal handoff candidate.
    pub outbox_candidate: Option<OutboxRecord>,
}

impl DataQualityHygieneLocalPersistenceRecords {
    /// Projects a reviewed Data-Quality Hygiene outcome into storage-shaped MVP rows without enabling live side effects.
    pub fn from_reviewed_outcome(
        ids: DataQualityHygieneLineageIds,
        outcome: DataQualityHygieneOutcomeRecord,
    ) -> Self {
        let completed = outcome.outcome == DataQualityHygieneOutcomeCode::Completed;
        let workflow_payload = json!({
            "correlation_id": outcome.correlation_id,
            "location_id": outcome.location_id,
            "operating_day": outcome.operating_day,
            "action_id": outcome.action_id,
            "source_refs": outcome.source_refs,
            "issue_refs": outcome.issue_refs,
            "live_side_effects_allowed": false,
            "provider_writes_allowed": false,
            "customer_messages_allowed": false,
        });

        let workflow_event = WorkflowEventRecord {
            id: ids.workflow_event_id.clone(),
            workflow_name: "data-quality-hygiene".to_owned(),
            event_kind: "context_created".to_owned(),
            subject_kind: "location".to_owned(),
            subject_id: ids.subject_id.clone(),
            idempotency_key: ids.idempotency_key.clone(),
            payload: workflow_payload.clone(),
            occurred_at: ids.recorded_at.clone(),
            recorded_at: ids.recorded_at.clone(),
        };

        let workflow_result = WorkflowResultRecord {
            id: format!("{}:result", ids.workflow_event_id),
            workflow_event_id: ids.workflow_event_id.clone(),
            status: if completed {
                WorkflowResultStatusCode::Succeeded
            } else {
                WorkflowResultStatusCode::NeedsReview
            },
            result: json!({
                "mode": "fake_deterministic_or_disabled",
                "reviewable_output_only": true,
                "action_id": outcome.action_id,
                "outcome": outcome.outcome,
                "live_side_effects_allowed": false,
            }),
            error_code: None,
            created_at: ids.recorded_at.clone(),
        };

        let review_packet = ReviewPacketRecord {
            id: ids.review_packet_id.clone(),
            subject_kind: "location".to_owned(),
            subject_id: ids.subject_id.clone(),
            gate: ReviewGateCode::ManagerApproval,
            status: if completed {
                ReviewPacketStatusCode::Approved
            } else {
                ReviewPacketStatusCode::ReadyForReview
            },
            workflow_event_id: ids.workflow_event_id.clone(),
            created_by_actor_kind: ActorKindCode::Agent,
            created_by_actor_id: "data-quality-hygiene-agent".to_owned(),
            created_at: ids.recorded_at.clone(),
            updated_at: ids.recorded_at.clone(),
        };

        let approval_record = ApprovalRecordRow {
            id: ids.approval_record_id.clone(),
            target_kind: "message".to_owned(),
            target_id: ids.subject_id.clone(),
            gate: ReviewGateCode::ManagerApproval,
            status: if completed {
                "approved"
            } else {
                "approval_requested"
            }
            .to_owned(),
            requested_by_actor_kind: ActorKindCode::Agent,
            requested_by_actor_id: "data-quality-hygiene-agent".to_owned(),
            requested_at: ids.recorded_at.clone(),
            decided_by_actor_kind: completed.then_some(ActorKindCode::Staff),
            decided_by_actor_id: completed.then(|| outcome.actor_id.clone()),
            decided_at: completed.then(|| ids.recorded_at.clone()),
            review_packet_id: ids.review_packet_id.clone(),
        };

        let audit_events = vec![
            AuditEventRecord {
                actor_kind: ActorKindCode::Agent,
                actor_id: "data-quality-hygiene-agent".to_owned(),
                subject_kind: "workflow_event".to_owned(),
                subject_id: ids.workflow_event_id.clone(),
                action: "data_quality_hygiene.context_created".to_owned(),
                workflow_event_id: ids.workflow_event_id.clone(),
                metadata: workflow_payload,
                occurred_at: ids.recorded_at.clone(),
                recorded_at: ids.recorded_at.clone(),
            },
            AuditEventRecord {
                actor_kind: ActorKindCode::Staff,
                actor_id: outcome.actor_id.clone(),
                subject_kind: "approval".to_owned(),
                subject_id: ids.approval_record_id.clone(),
                action: "data_quality_hygiene.reviewed_outcome_recorded".to_owned(),
                workflow_event_id: ids.workflow_event_id.clone(),
                metadata: json!({
                    "action_id": outcome.action_id,
                    "outcome": outcome.outcome,
                    "resolution_status_after_review": outcome.resolution_status_after_review,
                    "estimated_minutes_saved": outcome.estimated_minutes_saved,
                    "actual_minutes_saved": outcome.actual_minutes_saved(),
                    "live_side_effects_allowed": false,
                }),
                occurred_at: ids.recorded_at.clone(),
                recorded_at: ids.recorded_at.clone(),
            },
        ];

        let outbox_candidate = completed.then(|| OutboxRecord {
            id: ids.outbox_record_id.clone(),
            idempotency_key: format!("{}:internal-reviewed-handoff", ids.idempotency_key),
            approval_record_id: ids.approval_record_id.clone(),
            topic: "internal.data_quality_hygiene.reviewed_handoff".to_owned(),
            review_gate: ReviewGateCode::ManagerApproval,
            aggregate_kind: "message".to_owned(),
            aggregate_id: ids.subject_id.clone(),
            payload: json!({
                "action_id": outcome.action_id,
                "correlation_id": outcome.correlation_id,
                "issue_refs": outcome.issue_refs,
                "source_refs": outcome.source_refs,
                "internal_handoff_only": true,
                "live_delivery_allowed": false,
            }),
            status: OutboxStatusCode::Pending,
            available_at: ids.recorded_at.clone(),
        });

        Self {
            workflow_event,
            workflow_result,
            review_packet,
            approval_record,
            outcome: DataQualityHygieneOutcomeRow {
                workflow_event_id: ids.workflow_event_id,
                approval_record_id: ids.approval_record_id,
                record: outcome,
            },
            audit_events,
            outbox_candidate,
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
        serde_json::from_str(raw)
            .map_err(|source| CodecError::decode(RecordKind::PetResortPortfolio, source).into())
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|source| CodecError::encode(RecordKind::PetResortPortfolio, source).into())
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Stable operator code used in portfolio seed records.
pub enum OperatorCode {
    #[serde(rename = "nva")]
    #[strum(serialize = "nva")]
    /// Stable storage code for national veterinary associates.
    NationalVeterinaryAssociates,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Stable portfolio-structure codes for pet-resort operating assumptions.
pub enum PortfolioStructureCode {
    /// Stable storage code for federated multi brand.
    FederatedMultiBrand,
    /// Stable storage code for single brand.
    SingleBrand,
    /// Provider supplied an unrecognized value; preserve it for audit instead of failing closed.
    Unknown,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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
    /// Enumerated brand known to the pet-resort context pack.
    Known {
        /// Stable brand code promoted into a domain brand.
        code: PetResortBrandCode,
    },
    /// Non-enumerated brand preserved with a validated display name.
    Other {
        /// Validated display name for a brand not yet represented by a stable code.
        name: StoredBrandName,
    },
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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

    /// Returns the provider numeric identifier kept on this wrapper.
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
        serde_json::from_str(raw)
            .map_err(|source| CodecError::decode(RecordKind::ServiceOffering, source).into())
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|source| CodecError::encode(RecordKind::ServiceOffering, source).into())
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
/// Storage snapshot of the service-line rules enabled for a location.
pub struct CoreServiceContractsRecord {
    /// Location whose operating day or service rules is described.
    pub location_id: domain::entities::LocationId,
    /// Boarding rules capabilities for the location.
    pub boarding: boarding::ContractRecord,
    /// Daycare rules capabilities for the location.
    pub daycare: daycare::ContractRecord,
    /// Grooming rules capabilities for the location.
    pub grooming: grooming::ContractRecord,
    /// Training rules capabilities for the location.
    pub training: training::ContractRecord,
    /// Retail rules capabilities for the location.
    pub retail: retail::ContractRecord,
}

impl CoreServiceContractsRecord {
    /// Returns the stable record family represented by this storage snapshot.
    pub const fn record_kind(&self) -> RecordKind {
        RecordKind::CoreServiceContracts
    }

    /// Encodes the storage record as JSON for persistence or fixture comparison.
    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| {
            Error::Codec(CodecError::encode(RecordKind::CoreServiceContracts, source))
        })
    }

    /// Decodes a JSON storage payload into its typed record shape.
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| {
            Error::Codec(CodecError::decode(RecordKind::CoreServiceContracts, source))
        })
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
