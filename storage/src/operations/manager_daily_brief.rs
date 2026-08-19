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
/// Persisted outcome states for manager daily-brief actions.
pub enum ManagerDailyBriefOutcomeCode {
    /// Caller-reported completion label retained as nonclaimable disposition evidence.
    Completed,
    /// Caller-reported deferral retained as nonclaimable disposition evidence.
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
/// Persisted current manager daily-brief action codes.
///
/// A stable code is not authority or proof that the current workflow can emit the action. In
/// Each code names a current review-safe action family.
pub enum ManagerDailyBriefActionKindCode {
    /// Stable storage code for review demand against staffing plan.
    ReviewDemandAgainstStaffingPlan,
    /// Stable storage code for resolve checkout exception.
    ResolveCheckoutException,

    /// Stable storage code for investigate source data quality issue.
    InvestigateSourceDataQualityIssue,
    /// Stable storage code for reviewed capacity/labor recommendations.
    ReviewCapacityLaborRecommendation,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Dimensions used to aggregate manager daily-brief labor outcomes by location, day, action, and owner role.
pub struct ManagerDailyBriefReportingGroup {
    /// Location whose operating day or service rules is described.
    pub location_id: String,
    /// Caller-reported business date used to group retained evidence.
    pub operating_day: String,
    /// Caller-reported action-kind label used to group retained evidence.
    pub action_kind: ManagerDailyBriefActionKindCode,
    /// Caller-reported owner-persona label; it does not prove ownership or review.
    pub owner_persona: ManagerDailyBriefPersonaCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
/// Non-zero minute quantity persisted for manager daily-brief labor evidence.
pub struct StoredManagerDailyBriefLaborMinutes(u16);

impl StoredManagerDailyBriefLaborMinutes {
    /// Validates and wraps a non-zero labor-minute quantity before persistence.
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Stored evidence for a manager daily-brief action, including before/after labor minutes and source references.
pub struct ManagerDailyBriefOutcomeRecord {
    /// Stable action correlation identifier used for idempotent reported evidence.
    pub action_id: String,
    /// Caller-reported disposition label; not proof of review or completion.
    pub outcome: ManagerDailyBriefOutcomeCode,
    /// Caller-reported baseline estimate retained as nonclaimable evidence.
    pub before_minutes: StoredManagerDailyBriefLaborMinutes,
    /// Caller-reported minutes spent; not proof that the workflow completed or was reviewed.
    pub actual_minutes: StoredManagerDailyBriefLaborMinutes,
    /// Caller-reported actor label; this row does not authenticate identity.
    pub actor_id: String,
    /// Caller-reported persona label; this row does not prove completion or review.
    pub actor_persona: ManagerDailyBriefPersonaCode,
    /// Caller-reported feedback retained as nonclaimable history.
    pub feedback: String,
    #[builder(default)]
    /// Source refs correlated to the report; they do not prove the reported action occurred.
    pub source_refs: Vec<StoredSourceRecordRef>,
    /// Server-issued durable recording timestamp.
    pub recorded_at: String,
    /// Cross-system identifier tying the record to a workflow run or request.
    pub correlation_id: String,
    /// Location whose operating day or service rules is described.
    pub location_id: String,
    /// Caller-reported business date used to group retained evidence.
    pub operating_day: String,
    /// Caller-reported action-kind label used to group retained evidence.
    pub action_kind: ManagerDailyBriefActionKindCode,
    /// Caller-reported owner-persona label; it does not prove ownership or review.
    pub owner_persona: ManagerDailyBriefPersonaCode,
    /// Caller-reported estimate difference retained as nonclaimable evidence.
    pub reported_estimated_minutes_difference: u16,
}

impl ManagerDailyBriefOutcomeRecord {
    /// Returns dimensions used to group caller-reported, nonclaimable time evidence.
    pub fn reporting_group(&self) -> ManagerDailyBriefReportingGroup {
        ManagerDailyBriefReportingGroup {
            location_id: self.location_id.clone(),
            operating_day: self.operating_day.clone(),
            action_kind: self.action_kind,
            owner_persona: self.owner_persona,
        }
    }
}

impl fmt::Debug for ManagerDailyBriefOutcomeRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ManagerDailyBriefOutcomeRecord([REDACTED])")
    }
}
