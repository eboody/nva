//! Shared application/workflow orchestration for the pet-resort agent platform.
//!
//! # Operator framing
//!
//! Use this crate page when you want the app-level map of how domain facts become
//! reviewable workflow packets, tool-port calls, smoke fixtures, and shell-facing
//! orchestration. It matters to operators because this layer is where safe drafts
//! and evidence bundles are assembled before an API, worker, or CLI presents them
//! for review.
//!
//! The next step is to open `agents` for agent safety rules, then the
//! workflow module that matches the queue being reviewed, such as manager daily
//! brief, booking triage, checkout completion, CRM retention, daily update, or
//! data-quality hygiene. The Rustdoc items below remain implementer API detail;
//! this overview explains which operational page to read first.
//!
//! This crate composes semantic domain rules from `domain` into executable
//! workflow packets, agent prompts, tool-port rules, and MVP orchestration
//! previews shared by API, worker, and CLI shells.
//!
//! Crosswalk navigation: app modules are the workflow-use step in the entity
//! path. Use `docs/entity-atlas/contract-crosswalk/workflow-packets.md` to
//! move from an operator workflow to consumed/produced entities, then follow
//! `storage-persistence.md`, `runtime-exposure.md`, and the listed tests for
//! persistence, API/worker/CLI exposure, and executable proof.

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
/// Local deterministic fixtures that exercise agent and tool gates.
pub mod local_smoke;
pub mod manager_daily_brief;
pub mod tools;

/// Common app rules for shells that need agent specs and tool catalogs.
pub mod prelude {
    pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
    pub use crate::tools::{availability, draft_update};
}
