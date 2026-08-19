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
//! use storage::operations::{ServiceOfferingRecord, StoredSourceRecordRef};
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
//! assert!(matches!(stored, ServiceOfferingRecord::Daycare { .. }));
//! assert_eq!(source_ref.record_id, "reservation-type-42");
//!
//! let encoded = stored.encode_json()?;
//! let decoded = ServiceOfferingRecord::decode_json(&encoded)?;
//! let demoted: domain::operations::ServiceOffering = decoded.try_into()?;
//! assert_eq!(demoted, promoted_service);
//! # Ok(())
//! # }
//! ```

#[cfg(test)]
mod approval_outbox_authority_tests;

mod approval_outbox;
mod data_quality;
mod manager_daily_brief;
mod service_catalog;
mod site_finance;
mod workflow;

pub use crate::projection::{
    ApprovalOutboxAuthorityMismatch, CodecError, Error, Result, ShapeMismatchReason, StorageField,
};
pub use approval_outbox::{
    ApprovalOutboxBindingRecord, ApprovalOutboxLineageIds, ApprovalOutboxProjection,
    ApprovalOutboxProjectionInput, ApprovedInternalHandoffAuthority,
    CurrentApprovalReviewerCapability, DataQualityHygieneLineageIds,
    DataQualityHygieneLocalPersistenceRecords, InternalHandoff, InternalHandoffTopic,
    PendingOutboxRecord, SiteFinanceLocalPersistenceRecords,
};
pub use data_quality::{
    AffectedEntityKindCode, DataQualityFreshnessCode, DataQualityHygieneActionKindCode,
    DataQualityHygieneOutcomeCode, DataQualityHygieneOutcomeRecord,
    DataQualityHygieneOutcomeSchemaVersion, DataQualityHygieneOutcomeSummary,
    DataQualityHygienePersonaCode, DataQualityHygieneReportingGroup, DataQualityIssueKindCode,
    DataQualityIssueRecord, DataQualityResolutionStatusCode, DataQualitySensitivityCode,
    DataQualitySeverityCode, DataQualitySourceImportModeCode, DataQualitySourceImportRunRecord,
    DataQualitySourceImportStatusCode, DataQualitySyncGapKindCode, DataQualitySyncGapRecord,
    DataQualitySyncGapStatusCode, DataQualityWorkflowBlockingCode, ImportFreshnessRow,
    SourceQualityBacklogRow, StoredDataQualityHygieneLaborMinutes,
};
pub use manager_daily_brief::{
    ManagerDailyBriefActionKindCode, ManagerDailyBriefOutcomeCode, ManagerDailyBriefOutcomeRecord,
    ManagerDailyBriefPersonaCode, ManagerDailyBriefReportingGroup,
    StoredManagerDailyBriefLaborMinutes,
};

pub use service_catalog::{
    AdjacentSystemCode, BusinessLineCode, CoreOperatingSystemCode, CoreServiceContractsRecord,
    DataAccessPatternCode, OperatorCode, PetResortBrandCode, PetResortBrandRecord,
    PetResortPortfolioRecord, PortfolioStructureCode, ServiceOfferingRecord, StoredBrandName,
    StoredResortCount, StoredResortCountError, TechnologyEcosystemRecord,
};
pub use site_finance::{
    SiteFinanceOutcomeRecord, SiteFinanceValueAttribution, SiteFinanceWorkflowCompletion,
};
pub use workflow::{
    ActorKindCode, ApprovalRecordRow, AuditEventRecord, DataQualityHygieneOutcomeRow,
    OutboxStatusCode, ReviewGateCode, ReviewPacketRecord, ReviewPacketStatusCode,
    SiteFinanceOutcomeRow, WorkflowEventRecord, WorkflowResultRecord, WorkflowResultStatusCode,
};

use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use std::{collections::BTreeSet, fmt};

use crate::service_line::{boarding, daycare, grooming, retail, training};
use domain::operations::{pet_resort, service_core};

#[derive(Clone, PartialEq, Eq, Serialize, Builder)]
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

impl fmt::Debug for StoredSourceRecordRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("StoredSourceRecordRef([REDACTED])")
    }
}

impl StoredSourceRecordRef {
    /// Promotes raw persistence fields only after validating the complete provenance relationship.
    ///
    /// Source identity, record family and id, observation time, and adapter version are checked
    /// together so no partially plausible source reference can cross the storage boundary.
    pub fn try_new(
        system: impl AsRef<str>,
        record_type: impl Into<String>,
        record_id: impl Into<String>,
        observed_at: impl AsRef<str>,
        adapter_version: impl Into<String>,
    ) -> std::result::Result<Self, crate::persistence::Error> {
        let columns = crate::persistence::SourceRefColumns::try_new(
            system,
            record_type,
            record_id,
            observed_at,
            adapter_version,
        )?;
        Ok(Self {
            system: columns.record_ref().system().to_string(),
            record_type: columns.record_type().as_str().to_owned(),
            record_id: columns.record_id().as_str().to_owned(),
            observed_at: columns
                .observed_at()
                .get()
                .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
            adapter_version: columns.adapter_version().as_str().to_owned(),
        })
    }
}

impl<'de> Deserialize<'de> for StoredSourceRecordRef {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawStoredSourceRecordRef {
            system: String,
            record_type: String,
            record_id: String,
            observed_at: String,
            adapter_version: String,
        }

        let raw = RawStoredSourceRecordRef::deserialize(deserializer)?;
        Self::try_new(
            raw.system,
            raw.record_type,
            raw.record_id,
            raw.observed_at,
            raw.adapter_version,
        )
        .map_err(serde::de::Error::custom)
    }
}
