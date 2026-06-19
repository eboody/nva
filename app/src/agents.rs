//! Agent specs and prompt packet rules.
//!
//! # Operator framing
//!
//! Use this page when you want to understand what an AI workflow is allowed to
//! see, draft, or summarize before a person reviews it. It is for operators,
//! managers, and implementers checking the safety gate between helpful
//! automation and live resort actions.
//!
//! The next step is to read `baseline_agent_specs` for the built-in workflow
//! list, then open `AgentPromptPacket` to see the evidence bundle each run
//! receives. The Rust API details below remain the implementer rules; the
//! framing here explains how to interpret those rules operationally.
//!
//! This module is the app-layer rules for safe workflow automation. Agent
//! specs describe narrow read/draft capabilities for resort workflows, and
//! prompt packets carry the source event, typed workflow input, policy language,
//! and expected output schema an agent may use to prepare a draft or evidence
//! bundle. They are not live authority to mutate bookings, customer messages,
//! schedules, deposits, incident records, or policy decisions.
//!
//! Specs make the app gate visible to the runtime: allowed tools are narrow
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

/// App-facing alias for the domain agent specification used by workflow automation.
///
/// A spec is the stable rules an agent runner receives before it builds a
/// prompt packet: the workflow identity, business purpose, read/draft tools it
/// may use, actions it must never take directly, and deterministic review gates
/// that keep resort staff in control of bookings, messages, schedules, and
/// safety-sensitive decisions.
pub type AgentSpec = agent::Spec;

/// Rules implemented by app workflow agents that prepare safe prompt packets.
///
/// Implementors expose their immutable [`AgentSpec`], package an event and typed
/// input into an [`AgentPromptPacket`], then validate model/tool output before it
/// is accepted as a draft or evidence bundle. The trait is intentionally about
/// preparing and checking workflow artifacts; it does not grant authority to
/// write back to Gingr, send pet-parent messages, change labor schedules, or
/// mutate reservations.
pub trait WorkflowAgent<Input, Output> {
    /// Returns the agent's operational specification.
    ///
    /// The spec names the workflow, states its labor/customer-service purpose,
    /// lists only the read or draft tools the runner may expose, and records the
    /// review gates that must remain outside model control.
    fn spec(&self) -> AgentSpec;
    /// Builds the prompt packet for one workflow event and typed input payload.
    ///
    /// The returned packet should contain source event context, workflow input,
    /// policy instructions, and output schema expectations sufficient for an
    /// agent to draft or summarize evidence without taking live operational
    /// action.
    fn build_prompt_packet(
        &self,
        event: &workflow::Event,
        input: Input,
    ) -> AgentPromptPacket<Input>;
    /// Validates a proposed workflow output before downstream app code can use it.
    ///
    /// Implementations should preserve deterministic policy failures and reject
    /// unsafe output rather than treating agent text as authority to mutate
    /// bookings, messages, schedules, incident records, or customer commitments.
    fn validate_output(&self, output: workflow::Result<Output>) -> workflow::Result<Output>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Safe prompt-and-evidence packet exchanged with an automation agent.
///
/// Operator framing: this is the checklist packet an agent receives for one
/// reviewable workflow run. It matters to managers because every suggested
/// action, brief, classification, or customer-message draft should trace back to
/// the workflow name, source event, approved input facts, policy instructions,
/// and expected output shape recorded here.
///
/// Next step: compare the packet fields with the workflow's `AgentSpec` and
/// review gates before treating any generated output as usable evidence. The
/// field-level Rustdoc below documents the API shape; it does not grant live
/// authority to book, charge, message, schedule, or override policy.
///
/// An agent prompt packet bundles the workflow identity, triggering source event,
/// typed app input, policy instructions, and expected output schema sent to an
/// agent runner. It is a draft/evidence gate: agents can use it to prepare a
/// briefing, follow-up draft, classification, or manager-review packet, but the
/// packet itself is not permission to mutate live resort systems.
pub struct AgentPromptPacket<T> {
    /// Workflow identifier that ties the packet to a specific agent spec.
    ///
    /// Examples include `manager-daily-brief`, `booking-triage`, and
    /// `grooming-rebooking`; the name lets audits connect a packet back to the
    /// workflow rules that defined its allowed tools and forbidden actions.
    pub workflow_name: agent::Name,
    /// Business goal the agent should pursue while preparing only draft output.
    ///
    /// This describes the labor, customer-service, safety, or data-quality
    /// outcome for the workflow, such as summarizing labor risk or drafting a
    /// customer follow-up, without granting authority to perform the action.
    pub goal: agent::Purpose,
    /// Source workflow event that caused the packet to be built.
    ///
    /// The event provides audit correlation for the triggering reservation,
    /// intake, document, review, incident, or scheduled briefing path so a human
    /// reviewer can trace why this packet exists.
    pub event: workflow::Event,
    /// Typed app-layer input facts available to the agent for this run.
    ///
    /// The payload should contain the workflow-specific request or evidence the
    /// app has already promoted from source systems; it is context for drafting,
    /// not authorization to repair or overwrite those source records.
    pub input: T,
    /// Policy instructions the runner must include in the agent context.
    ///
    /// These instructions state review gates, safety limits, escalation rules,
    /// and source-grounding requirements that constrain agent drafts and make
    /// policy compliance reviewable after the run.
    pub policies: Vec<agent::PolicyInstruction>,
    /// Name of the output schema expected from the agent.
    ///
    /// The schema name tells the runner and validator which structured draft,
    /// classification, briefing, or evidence bundle shape to expect before any
    /// downstream workflow code accepts the output.
    pub output_schema_name: agent::OutputSchemaName,
}

/// Returns the baseline app agent specifications for pet-resort automation.
///
/// The list covers bounded workflows such as inquiry intake, booking triage,
/// vaccine document review, manager briefings, lead conversion, grooming
/// rebooking, reputation triage, and SOP assistance. Each spec deliberately
/// exposes read/draft tools and review gates while forbidding direct actions
/// such as confirming bookings, changing schedules, waiving deposits, diagnosing
/// pets, or sending customer messages without approval.
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
