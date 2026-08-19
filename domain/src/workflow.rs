//! Workflow events and outcomes for reviewable resort operations.
//!
//! # Operator framing
//!
//! Use this page to understand how a source fact turns into a staff-visible task,
//! review reason, draft message, or recommended next action. It matters to
//! operators because workflow values preserve why something is being suggested,
//! what evidence supports it, and which human review gate still controls the live
//! care, labor, payment, or customer-communication step.
//!
//! The next step is to follow the type that matches the queue you are explaining:
//! events identify why work started, task/message modules describe staff-facing
//! drafts, review values explain why automation stopped, and outcomes record the
//! evidence trail. The Rust API details below are the generated implementation surface for
//! implementers; this framing is the business reading guide.
//!
//! Workflows connect provider/read-model facts to staff-visible tasks, customer-message drafts, policy
//! context, and recommended next actions. They preserve evidence and review reasons so AI agents can
//! reduce manual triage while keeping live care, labor, payment, and customer communications inside
//! explicit approval boundaries.

use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::{entities, policy};

/// External workflow-provider vocabulary retained before promotion into domain tasks or messages.
pub mod external;
/// Provider message fields used before normalization into customer-message workflows.
pub mod message;
/// Provider status-update fields used to reconcile external task or message progress.
pub mod status_update;
/// Provider task fields used to create staff work without losing source evidence.
pub mod task;

mod event;
pub use event::{
    AllowedAction, Event, EventError, EventId, EventType, PolicyContext, ReviewReason, RiskFlag,
    Subject, Summary, VerificationNote,
};

mod outcome;
pub use outcome::{Error, Outcome, RecommendedAction, Result, Status};
