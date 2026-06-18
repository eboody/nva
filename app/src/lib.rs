//! Shared application/workflow orchestration for the pet-resort agent platform.
//!
//! This crate composes semantic domain contracts from `domain` into executable
//! workflow packets, agent prompts, tool-port contracts, and MVP orchestration
//! previews shared by API, worker, and CLI shells.

pub mod agents;
pub mod booking_triage;
/// Checkout-closeout packets, gates, and safe handoff drafts.
pub mod checkout_completion;
/// Source-grounded retention follow-up packets and review-only customer drafts.
pub mod crm_retention;
/// Daily operational update packets built from deterministic source context.
pub mod daily_update;
/// Duplicate/stale-record hygiene workflows that stop before provider mutation.
pub mod data_quality_hygiene;
/// Local deterministic fixtures that exercise agent and tool boundaries.
pub mod local_smoke;
pub mod manager_daily_brief;
pub mod tools;

/// Common app contracts for shells that need agent specs and tool catalogs.
pub mod prelude {
    pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
    pub use crate::tools::{availability, draft_update};
}
