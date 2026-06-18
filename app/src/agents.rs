//! Agent specs and prompt packet contracts.
//!
//! Specs make the app boundary visible to the runtime: allowed tools are narrow
//! read/draft surfaces, forbidden actions name unsafe authority, and default
//! review gates remain deterministic app policy.
//!
//! ```
//! use app::agents;
//!
//! let specs = agents::baseline_agent_specs();
//! let manager_brief = specs
//!     .iter()
//!     .find(|spec| spec.name.clone().into_inner() == "manager-daily-brief")
//!     .expect("baseline manager brief spec exists");
//!
//! assert!(manager_brief
//!     .allowed_tools
//!     .iter()
//!     .any(|tool| tool.clone().into_inner() == "reservation-read"));
//! assert!(manager_brief
//!     .forbidden_actions
//!     .iter()
//!     .any(|action| action.clone().into_inner() == "change schedule"));
//! assert!(manager_brief
//!     .forbidden_actions
//!     .iter()
//!     .any(|action| action.clone().into_inner() == "send customer message without approval"));
//! assert!(!manager_brief.default_review_gates.is_empty());
//! ```
use bon::Builder;
use serde::{Deserialize, Serialize};

use domain::{agent, policy, workflow};

pub use domain::agent::{OutputSchemaName, PolicyInstruction};

/// Shared agent spec type used across the agents boundary.
pub type AgentSpec = agent::Spec;

/// Defines the behavior required from a workflow agent participant in the agents workflow.
pub trait WorkflowAgent<Input, Output> {
    /// Runs the spec step while preserving the agent packet boundary safety boundary.
    fn spec(&self) -> AgentSpec;
    /// Runs the build prompt packet step while preserving the agent packet boundary safety boundary.
    fn build_prompt_packet(
        &self,
        event: &workflow::Event,
        input: Input,
    ) -> AgentPromptPacket<Input>;
    /// Runs the output step while preserving the agent packet boundary safety boundary.
    fn validate_output(&self, output: workflow::Result<Output>) -> workflow::Result<Output>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Agent prompt packet carried by the agent packet boundary; it defines packet contracts exchanged with agents across safe drafting boundaries.
pub struct AgentPromptPacket<T> {
    /// Workflow name preserved as evidence for audit, review, or agent context.
    pub workflow_name: agent::Name,
    /// Goal preserved as evidence for audit, review, or agent context.
    pub goal: agent::Purpose,
    /// Event preserved as evidence for audit, review, or agent context.
    pub event: workflow::Event,
    /// Input preserved as evidence for audit, review, or agent context.
    pub input: T,
    /// Policies preserved as evidence for audit, review, or agent context.
    pub policies: Vec<agent::PolicyInstruction>,
    /// Output schema name preserved as evidence for audit, review, or agent context.
    pub output_schema_name: agent::OutputSchemaName,
}

/// Produces the baseline agent specs contract for the agents workflow.
pub fn baseline_agent_specs() -> Vec<AgentSpec> {
    vec![
        spec(
            "inquiry-intake",
            "Extract new customer/pet/service/date details, identify missing info, and draft safe follow-up replies.",
            ["portal-read", "crm-read", "task-create"],
            ["confirm booking", "send sensitive message without approval"],
            [policy::ReviewGate::CustomerMessageApproval],
        ),
        spec(
            "booking-triage",
            "Evaluate booking requests against deterministic availability, eligibility, vaccine, deposit, and policy context.",
            ["availability-read", "policy-read", "draft-message"],
            [
                "invent availability",
                "override hard policy",
                "waive deposit",
            ],
            [policy::ReviewGate::ManagerApproval],
        ),
        spec(
            "vaccine-document",
            "Extract vaccine names/dates from uploaded proof and route ambiguity to human review.",
            ["document-read", "ocr", "vaccine-policy-read"],
            ["final approve uncertain medical document"],
            [policy::ReviewGate::MedicalDocumentReview],
        ),
        spec(
            "daily-care-update",
            "Turn staff notes/photos into warm customer-safe update drafts with risk flags.",
            ["care-note-read", "draft-message"],
            [
                "diagnose",
                "hide concerning facts",
                "auto-send health concern",
            ],
            [policy::ReviewGate::CustomerMessageApproval],
        ),
        spec(
            "incident-escalation",
            "Summarize incident facts, classify possible severity, identify missing fields, and draft manager/owner review packets.",
            ["incident-read", "task-create", "draft-message"],
            [
                "close incident",
                "diagnose",
                "send owner message without manager approval",
            ],
            [
                policy::ReviewGate::ManagerApproval,
                policy::ReviewGate::CustomerMessageApproval,
            ],
        ),
        spec(
            "manager-daily-brief",
            "Summarize occupancy, arrivals, labor risk, pet-care watchlist, customer follow-ups, and revenue opportunities for resort leaders.",
            [
                "reservation-read",
                "labor-schedule-read",
                "care-note-read",
                "task-create",
            ],
            [
                "invent occupancy",
                "change schedule",
                "send customer message without approval",
            ],
            [policy::ReviewGate::ManagerApproval],
        ),
        spec(
            "lead-conversion",
            "Classify inquiry intent, identify missing intake requirements, and draft next-best follow-up for boarding, daycare, grooming, or training leads.",
            ["lead-read", "customer-read", "portal-read", "draft-message"],
            [
                "book reservation",
                "promise availability",
                "override requirements",
            ],
            [policy::ReviewGate::CustomerMessageApproval],
        ),
        spec(
            "grooming-rebooking",
            "Find grooming cadence opportunities, low-utilization slots, and safe customer follow-up drafts without changing calendars automatically.",
            [
                "grooming-history-read",
                "availability-read",
                "draft-message",
            ],
            [
                "book grooming slot",
                "apply discount",
                "send message without approval",
            ],
            [policy::ReviewGate::CustomerMessageApproval],
        ),
        spec(
            "reputation-triage",
            "Classify review themes, identify safety/legal escalations, summarize location trends, and draft public-response packets.",
            ["review-read", "task-create", "draft-message"],
            [
                "delete review",
                "deny incident facts",
                "publish response without approval",
            ],
            [
                policy::ReviewGate::ManagerApproval,
                policy::ReviewGate::CustomerMessageApproval,
            ],
        ),
        spec(
            "sop-policy-assistant",
            "Answer staff policy questions from approved SOP context and route medical, refund, safety, or incident ambiguity to human review.",
            ["policy-read", "sop-read", "task-create"],
            ["diagnose", "approve refund", "override safety policy"],
            [policy::ReviewGate::ManagerApproval],
        ),
    ]
}

fn spec<const TOOLS: usize, const FORBIDDEN: usize, const GATES: usize>(
    name: &str,
    purpose: &str,
    allowed_tools: [&str; TOOLS],
    forbidden_actions: [&str; FORBIDDEN],
    default_review_gates: [policy::ReviewGate; GATES],
) -> AgentSpec {
    AgentSpec::builder()
        .name(agent::Name::try_new(name).expect("baseline agent names are non-empty"))
        .purpose(agent::Purpose::try_new(purpose).expect("baseline purposes are non-empty"))
        .allowed_tools(
            allowed_tools
                .into_iter()
                .map(|tool| {
                    agent::ToolName::try_new(tool).expect("baseline tool names are non-empty")
                })
                .collect(),
        )
        .forbidden_actions(
            forbidden_actions
                .into_iter()
                .map(|action| {
                    agent::ForbiddenAction::try_new(action)
                        .expect("baseline forbidden actions are non-empty")
                })
                .collect(),
        )
        .default_review_gates(default_review_gates.into_iter().collect())
        .build()
}
