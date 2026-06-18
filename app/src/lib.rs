//! Shared application/workflow orchestration for the pet-resort agent platform.
//!
//! This crate composes semantic domain contracts from `domain` into executable
//! workflow packets, agent prompts, tool-port contracts, and MVP orchestration
//! previews shared by API, worker, and CLI shells.

pub mod agents;
pub mod booking_triage;
pub mod checkout_completion;
pub mod crm_retention;
pub mod daily_update;
pub mod data_quality_hygiene;
pub mod local_smoke;
pub mod manager_daily_brief;
pub mod tools;

pub mod prelude {
    pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
    pub use crate::tools::{availability, draft_update};
}
