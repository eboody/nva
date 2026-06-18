use bon::Builder;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::policy;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
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
pub struct Name(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 400),
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
pub struct Purpose(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
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
pub struct ToolName(String);

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
pub struct ForbiddenAction(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 400),
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
pub struct PolicyInstruction(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
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
pub struct OutputSchemaName(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Domain contract for one bounded automation agent.
///
/// A spec describes what an agent is meant to help with, which narrow tools may
/// be exposed to it, which live operational actions are outside its authority,
/// and which deterministic review gates must remain in the app workflow.
pub struct Spec {
    /// Stable workflow-facing agent name.
    ///
    /// This is an identifier such as `manager-daily-brief` or `booking-triage`,
    /// not a human display label. It connects prompt packets, outputs, and audit
    /// evidence back to the spec that constrained the agent run.
    pub name: Name,
    /// Business purpose for the agent's draft or evidence work.
    ///
    /// The purpose should state the resort operation being supported, such as
    /// summarizing labor risk, drafting customer-safe follow-up, or routing
    /// vaccine-document ambiguity; it does not grant live-action authority.
    pub purpose: Purpose,
    /// Tool names the runtime may expose to this agent.
    ///
    /// These should be read-only, draft-only, or task-creation surfaces scoped to
    /// the workflow. They are the positive capability list for context building,
    /// not permission to bypass review gates or mutate source systems.
    pub allowed_tools: Vec<ToolName>,
    /// Live or unsafe actions the agent must not perform directly.
    ///
    /// Examples include confirming bookings, promising availability, changing
    /// labor schedules, waiving deposits, diagnosing pets, or sending customer
    /// messages without approval.
    pub forbidden_actions: Vec<ForbiddenAction>,
    /// Human or deterministic app review gates required for the workflow.
    ///
    /// These gates keep manager approval, customer-message approval, medical
    /// document review, and similar authority outside the model-generated output.
    pub default_review_gates: Vec<policy::ReviewGate>,
}
