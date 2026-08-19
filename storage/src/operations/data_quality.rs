use super::*;

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
    /// Caller-reported completion label retained as nonclaimable disposition evidence.
    Completed,
    /// Caller-reported deferral retained as nonclaimable disposition evidence.
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
/// Persisted caller-reported lifecycle-status label for a data-quality issue; it proves no review, acknowledgement, repair, or supersession.
pub enum DataQualityResolutionStatusCode {
    /// Caller reports an open-status label without proving review.
    Open,
    /// Caller reports an acknowledged-status label without proving acceptance or a repair commitment.
    Acknowledged,
    /// Caller reports an ignored-status label without proving review or an intentional decision.
    Ignored,
    /// Caller reports a repaired-status label without proving correction or review.
    Repaired,
    /// Caller reports a superseded-status label without proving replacement or fresher evidence.
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
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

impl DataQualityIssueRecord {}

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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
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

impl DataQualitySourceImportRunRecord {}

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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
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

impl DataQualitySyncGapRecord {}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Latest caller-reported hygiene outcome evidence linked to the issue, if known; it proves no review or resolution.
    pub latest_outcome_id: Option<String>,
    /// Stable projection contract version.
    pub projection_version: String,
    /// Caveats such as `raw_payload_redacted` or `live_side_effects_disabled`.
    pub caveats: Vec<String>,
}

impl SourceQualityBacklogRow {}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl ImportFreshnessRow {}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Dimensions used to group data-quality hygiene outcomes by location, day, issue type, and owner role.
pub struct DataQualityHygieneReportingGroup {
    /// Location whose operating day or service rules is described.
    pub location_id: String,
    /// Caller-reported business date used to group retained evidence.
    pub operating_day: String,
    /// Caller-reported action-kind label used to group retained evidence.
    pub action_kind: DataQualityHygieneActionKindCode,
    /// Caller-reported owner-persona label; it does not prove ownership or review.
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Version tag for the durable data-quality hygiene outcome row contract.
pub enum DataQualityHygieneOutcomeSchemaVersion {
    /// Initial persisted Data Quality Hygiene outcome row schema.
    #[serde(rename = "data_quality_hygiene_outcome.v1")]
    #[default]
    V1,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Stored evidence for a data-quality hygiene action, including labor deltas, issue references, and resolution state.
pub struct DataQualityHygieneOutcomeRecord {
    #[builder(default)]
    /// Stable schema version that makes durable row evolution explicit.
    pub schema_version: DataQualityHygieneOutcomeSchemaVersion,
    /// Stable action correlation identifier used for idempotent reported evidence.
    pub action_id: String,
    /// Caller-reported disposition label; not proof of review or completion.
    pub outcome: DataQualityHygieneOutcomeCode,
    /// Caller-reported baseline estimate retained as nonclaimable evidence.
    pub before_minutes: StoredDataQualityHygieneLaborMinutes,
    /// Caller-reported minutes spent; not proof that the workflow completed or was reviewed.
    pub actual_minutes: StoredDataQualityHygieneLaborMinutes,
    /// Caller-reported actor label; this row does not authenticate identity.
    pub actor_id: String,
    /// Caller-reported persona label; this row does not prove completion or review.
    pub actor_persona: DataQualityHygienePersonaCode,
    /// Caller-reported feedback retained as nonclaimable history.
    pub feedback: String,
    #[builder(default)]
    /// Source refs correlated to the report; they do not prove the reported action occurred.
    pub source_refs: Vec<StoredSourceRecordRef>,
    #[builder(default)]
    /// Caller-provided issue references correlated to the report.
    pub issue_refs: Vec<String>,
    /// Caller-reported resolution-status label; not proof that review or repair completed.
    pub reported_resolution_status: DataQualityResolutionStatusCode,
    /// Server-issued durable recording timestamp.
    pub recorded_at: String,
    /// Cross-system identifier tying the record to a workflow run or request.
    pub correlation_id: String,
    /// Location whose operating day or service rules is described.
    pub location_id: String,
    /// Caller-reported business date used to group retained evidence.
    pub operating_day: String,
    /// Caller-reported action-kind label used to group retained evidence.
    pub action_kind: DataQualityHygieneActionKindCode,
    /// Caller-reported owner-persona label; it does not prove ownership or review.
    pub owner_persona: DataQualityHygienePersonaCode,
    /// Caller-reported estimate difference retained as nonclaimable evidence.
    pub reported_estimated_minutes_difference: u16,
}

impl DataQualityHygieneOutcomeRecord {
    /// Returns dimensions used to group caller-reported, nonclaimable time evidence.
    pub fn reporting_group(&self) -> DataQualityHygieneReportingGroup {
        DataQualityHygieneReportingGroup {
            location_id: self.location_id.clone(),
            operating_day: self.operating_day.clone(),
            action_kind: self.action_kind,
            owner_persona: self.owner_persona,
        }
    }
}

impl fmt::Debug for DataQualityHygieneOutcomeRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DataQualityHygieneOutcomeRecord([REDACTED])")
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Caller-reported data-quality hygiene outcome rollup for location/day/correlation reporting.
///
/// The summary keeps source refs and issue refs visible while aggregating caller-reported
/// time evidence and dispositions. It is a reporting readout only; it does not
/// authorize provider writes, customer sends, schedule changes, or payment movement.
pub struct DataQualityHygieneOutcomeSummary {
    /// Location whose reported hygiene outcomes are summarized.
    pub location_id: String,
    /// Business date whose reported hygiene outcomes are summarized.
    pub operating_day: String,
    /// Optional workflow correlation filter used to compare one run or request.
    pub correlation_id: Option<String>,
    /// Count of stored caller-reported outcome records in scope.
    pub reported_outcome_count: usize,
    /// Outcomes whose serialized disposition reports completion; not accepted completion authority.
    pub reported_completed_outcome_count: usize,
    /// Deferred outcomes kept visible but excluded from completed-savings proof.
    pub deferred_count: usize,
    /// Outcomes where source evidence was wrong and must not be hidden.
    pub wrong_source_count: usize,
    /// Outcomes reviewed as not actionable.
    pub not_actionable_count: usize,
    /// Outcomes suppressed by a manager or reviewer.
    pub suppressed_by_manager_count: usize,
    /// Sum of caller-reported estimates retained as nonclaimable evidence.
    pub total_reported_estimated_minutes_difference: u16,
    /// Sum of caller-reported minute fields retained as nonclaimable evidence.
    pub total_actual_minutes_spent: u16,

    /// Source-record evidence retained for audit and reconciliation.
    pub source_refs: Vec<StoredSourceRecordRef>,
    /// Data-quality issue identifiers retained for audit and reconciliation.
    pub issue_refs: Vec<String>,
}

impl DataQualityHygieneOutcomeSummary {
    /// Aggregates caller-reported outcome records by location, day, and optional correlation.
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
            reported_outcome_count: 0,
            reported_completed_outcome_count: 0,
            deferred_count: 0,
            wrong_source_count: 0,
            not_actionable_count: 0,
            suppressed_by_manager_count: 0,
            total_reported_estimated_minutes_difference: 0,
            total_actual_minutes_spent: 0,
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
            summary.reported_outcome_count += 1;
            match record.outcome {
                DataQualityHygieneOutcomeCode::Completed => {
                    summary.reported_completed_outcome_count += 1;
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
