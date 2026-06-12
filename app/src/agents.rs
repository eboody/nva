use bon::Builder;
use serde::{Deserialize, Serialize};

use domain::agent;
use domain::policy::ReviewGate;
use domain::workflow::{WorkflowEvent, WorkflowResult};

pub use domain::agent::{OutputSchemaName, PolicyInstruction};

pub type AgentSpec = agent::Spec;

pub trait WorkflowAgent<Input, Output> {
    fn spec(&self) -> AgentSpec;
    fn build_prompt_packet(&self, event: &WorkflowEvent, input: Input) -> AgentPromptPacket<Input>;
    fn validate_output(&self, output: WorkflowResult<Output>) -> WorkflowResult<Output>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct AgentPromptPacket<T> {
    pub workflow_name: agent::Name,
    pub goal: agent::Purpose,
    pub event: WorkflowEvent,
    pub input: T,
    pub policies: Vec<agent::PolicyInstruction>,
    pub output_schema_name: agent::OutputSchemaName,
}

pub fn baseline_agent_specs() -> Vec<AgentSpec> {
    vec![
        spec(
            "inquiry-intake",
            "Extract new customer/pet/service/date details, identify missing info, and draft safe follow-up replies.",
            ["portal-read", "crm-read", "task-create"],
            ["confirm booking", "send sensitive message without approval"],
            [ReviewGate::CustomerMessageApproval],
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
            [ReviewGate::ManagerApproval],
        ),
        spec(
            "vaccine-document",
            "Extract vaccine names/dates from uploaded proof and route ambiguity to human review.",
            ["document-read", "ocr", "vaccine-policy-read"],
            ["final approve uncertain medical document"],
            [ReviewGate::MedicalDocumentReview],
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
            [ReviewGate::CustomerMessageApproval],
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
                ReviewGate::ManagerApproval,
                ReviewGate::CustomerMessageApproval,
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
            [ReviewGate::ManagerApproval],
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
            [ReviewGate::CustomerMessageApproval],
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
            [ReviewGate::CustomerMessageApproval],
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
                ReviewGate::ManagerApproval,
                ReviewGate::CustomerMessageApproval,
            ],
        ),
        spec(
            "sop-policy-assistant",
            "Answer staff policy questions from approved SOP context and route medical, refund, safety, or incident ambiguity to human review.",
            ["policy-read", "sop-read", "task-create"],
            ["diagnose", "approve refund", "override safety policy"],
            [ReviewGate::ManagerApproval],
        ),
    ]
}

fn spec<const TOOLS: usize, const FORBIDDEN: usize, const GATES: usize>(
    name: &str,
    purpose: &str,
    allowed_tools: [&str; TOOLS],
    forbidden_actions: [&str; FORBIDDEN],
    default_review_gates: [ReviewGate; GATES],
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
