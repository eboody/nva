//! Data-quality hygiene workflow rules for source-grounded internal cleanup.
//!
//! Crosswalk navigation: this module is the workflow-use surface for data-quality
//! issues, source refs, hygiene candidates/actions, draft validation, and
//! caller-reported outcome capture. The bidirectional docs path is
//! `docs/entity-atlas/contract-crosswalk/workflow-packets.md` for workflow use,
//! `source-provider-flows.md` for source entry and normalization,
//! `storage-persistence.md` for `DataQualityHygieneOutcomeRecord`,
//! `runtime-exposure.md` for API/smoke exposure, and
//! `app/tests/data_quality_hygiene_workflow_contracts.rs` plus API/storage tests
//! for executable proof.

use nonempty::NonEmpty;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use domain::{data_quality, entities, operations, policy, source};

const WORKFLOW_NAME: &str = "data-quality-hygiene";
const SCHEMA_VERSION: &str = "data-quality-hygiene-context-v1";

mod outcome;
mod outcome_capture;
mod workflow;

use workflow::blocked_actions_for;

pub use outcome::{
    FeedbackOutcome, OutcomeIssueRefs, OutcomeRecord, OutcomeRecordBuilder, OutcomeSourceRecordRefs,
};
pub use outcome_capture::{
    ActorAssignment, ActorDirectory, ActorId, AuditLog, AuditRecord, AuthorizationPolicy,
    BlockedActionLog, BlockedActionReason, BlockedActionRecord, OutcomeCaptureRequest,
    OutcomeCaptureService, OutcomeReceipt, OutcomeRecorder, ReviewQueueItem, ReviewQueueStore,
    RoleLocationAuthorization,
};
pub use workflow::{
    Action, ActionId, ActionKind, ActionPriority, ActionRationale, AffectedEntity,
    AggregateLaborMinutes, BlockedAction, Candidate, CandidateKind, CleanupAction, ContextPacketId,
    CorrelationId, DraftAction, DraftRejectionReason, DraftSubmission, DraftValidation, Error,
    HygienePersona, IssueCategory, IssueRef, LaborImpactEstimate, LaborMinutes, Packet,
    RedactionPolicy, RemovedManualWork, Request, Result, ReviewerRole, SafeAgentAction,
    Sensitivity, SourceFreshness, Workflow,
};
