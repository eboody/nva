//! Manager Daily Brief workflow rules for internal review and nonclaimable reported labor evidence.
//!
//! The workflow starts from app-owned, source-grounded context and produces a
//! deterministic packet that an agent may summarize or rank, but not use as
//! authority to mutate schedules, provider/PMS records, customer channels, or
//! money movement. Outcome records retain caller-reported baseline and workflow-minute
//! labels as nonclaimable evidence; they prove no review, measurement, completion, or labor reduction.
//!
//! Crosswalk navigation: operator docs link this workflow to the
//! Manager Daily Brief packet row in
//! `docs/entity-atlas/contract-crosswalk/workflow-packets.md`; storage outcome
//! projection evidence lives in `storage-persistence.md`, runtime/API exposure
//! in `runtime-exposure.md`, and executable proof in
//! `app/tests/manager_daily_brief_workflow_contracts.rs` plus API/storage tests.
//!
//! ```
//! use app::manager_daily_brief as brief;
//! use chrono::NaiveDate;
//! use domain::{analytics, entities, operations, source};
//! use uuid::Uuid;
//!
//! let location_id = entities::LocationId::new(Uuid::from_u128(0x170));
//! let operating_day = operations::operating_day::Date::try_new(
//!     NaiveDate::from_ymd_opt(2026, 6, 18).expect("fixture date is valid"),
//! )?;
//! let source_ref = source::RecordRef::new(
//!     source::System::BusinessIntelligence,
//!     source::record::Id::try_new("labor-read-model:boarding-demand:2026-06-18")?,
//! );
//! let demand_fact = analytics::service_demand::Fact::try_new(
//!     analytics::service_demand::Id::try_new("boarding-demand-risk")?,
//!     operations::operating_day::Key::new(
//!         location_id,
//!         operations::service_core::ServiceLine::Boarding,
//!         operating_day,
//!     ),
//!     analytics::service_demand::DemandUnits::try_new(42)?,
//!     vec![source_ref.clone()],
//!     analytics::ProjectionVersion::try_new("manager-brief-fixture-v1")?,
//!     vec![],
//! )?;
//!
//! let request = brief::Request::builder()
//!     .location_id(location_id)
//!     .operating_day(operating_day)
//!     .prepared_for(brief::ManagerBriefPersona::GeneralManager)
//!     .demand_attention_threshold(brief::DemandThresholdUnits::try_new(25)?)
//!     .service_demand_facts(vec![demand_fact])
//!     .build();
//!
//! let packet = brief::Workflow::evaluate(request);
//!
//! assert_eq!(packet.actions().len(), 1);
//! assert!(packet.all_actions_are_source_grounded());
//! assert!(packet.safe_agent_actions().contains(&brief::SafeAgentAction::RankManagerActions));
//! assert!(packet.blocked_actions().contains(&brief::BlockedAction::ChangeStaffSchedule));
//! assert!(packet.blocked_actions().contains(&brief::BlockedAction::MutateProviderOrPmsRecord));
//! assert!(packet.reported_estimated_minutes_difference() > 0);
//!
//! let outcome = brief::OutcomeRecord::builder()
//!     .action_id(packet.actions()[0].id().clone())
//!     .recorded_by(entities::ActorRef::Manager {
//!         manager_id: entities::ManagerId::try_new("gm-fixture")?,
//!     })
//!     .outcome(brief::FeedbackOutcome::Completed)
//!     .before_minutes(brief::LaborMinutes::try_new(45)?)
//!     .actual_minutes(brief::LaborMinutes::try_new(12)?)
//!     .source_record_refs(vec![source_ref])
//!     .build();
//!
//! assert!(outcome.records_feedback_without_external_mutation());
//! assert!(outcome.blocked_actions().contains(&brief::BlockedAction::SendCustomerMessage));
//! assert!(!outcome.counts_as_labor_savings());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use domain::{analytics, entities, operations, policy, source};
use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{checkout_completion, crm_retention};

mod outcome;
mod workflow;

pub use outcome::{
    FeedbackOutcome, LaborSavingsClaim, LaborSavingsNotClaimedReason, OutcomeRecord,
    ReportedDisposition,
};
pub use workflow::{
    ActionId, ActionRationale, AggregateLaborMinutes, BlockedAction, BriefAction, BriefActionKind,
    BriefActionPriority, BriefSummary, DemandThresholdUnits, Error, LaborImpactEstimate,
    LaborMinutes, ManagerBriefPersona, ManagerFeedback, Packet, RemovedManualWork, Request, Result,
    SafeAgentAction, ScopedCheckoutPacket, ScopedRetentionPacket, SourceFact, SourceFactKind,
    Workflow, requested_side_effect_rejection_reason,
};
