//! Typed domain contracts for NVA Pet Resorts labor-cost automation.
//!
//! The crate is the code-derived source of truth for a 170-location pet-resort operating model:
//! customers, pets, reservations, source-system provenance, safety policies, payment/deposit
//! states, documents, messages, workflow events, and audit evidence. Types in this crate describe
//! what an agent or adapter may know and what must remain draft/review-gated before any live staff,
//! customer, payment, medical, or capacity action.
//!
//! Crosswalk navigation for docs readers: start with the entity index and contract
//! crosswalks before treating a type as operational authority. `domain` is where
//! provider/staff evidence is normalized into semantic values; workflow use,
//! persistence, runtime exposure, and tests are linked from
//! `docs/entity-atlas/contract-crosswalk/{surface-inventory,source-provider-flows,workflow-packets,storage-persistence,runtime-exposure}.md`.

/// Agent prompt and packet contracts for review-gated operating workflows.
pub mod agent;
/// Analytics contracts for labor, revenue, occupancy, and exception reporting.
pub mod analytics;
/// Audit contracts that preserve evidence for source facts and automation decisions.
pub mod audit;
/// Boarding contracts for accommodation, deposits, handoffs, housekeeping, and upsells.
pub mod boarding;
/// Care-profile contracts for feeding, medication, handling, and pet-safety notes.
pub mod care;
/// Customer identity/contact value contracts for portal and messaging workflows.
pub mod customer;
/// Manager daily-briefing contracts for occupancy, staffing, risk, and action summaries.
pub mod daily_brief;
/// Data-quality contracts for repairing uncertain source records before automation uses them.
pub mod data_quality;
/// Daycare contracts for eligibility, attendance, yard assignment, coverage, and throughput.
pub mod daycare;
/// Document intake, verification, storage, and retention contracts.
pub mod document;
/// Normalized core entity contracts for locations, customers, pets, reservations, and records.
pub mod entities;
/// Grooming contracts for appointments, services, estimates, rebooking, and history.
pub mod grooming;
/// Incident contracts for safety events, evidence, escalation, and follow-up.
pub mod incident;
/// Lead intake and follow-up contracts for prospective customer conversion workflows.
pub mod lead;
/// Location contracts that scope local resort capability, policy, and timezone facts.
pub mod location;
/// Message contracts for draft, approval, queueing, delivery, and suppression state.
pub mod message;
/// Money contracts for typed resort charges and deposits.
pub mod money;
/// Operations contracts for service offerings, capacity, labor, and location execution.
pub mod operations;
/// Payment contracts for deposits, references, collection, waiver, and refund state.
pub mod payment;
/// Pet identity and care-label contracts used across reservations and safety workflows.
pub mod pet;
/// Policy contracts that decide what is safe to automate and what must be reviewed.
pub mod policy;
/// Portal account contracts for provider/customer access boundaries.
pub mod portal;
/// Reputation contracts for review monitoring, response drafting, and escalation workflows.
pub mod reputation;
/// Reservation contracts for age rules, add-ons, transitions, and service eligibility.
pub mod reservation;
/// Retail contracts for POS, inventory, recommendations, vendors, and reorder signals.
pub mod retail;
/// Source-system provenance contracts for Gingr/import facts and normalization assumptions.
pub mod source;
/// Staff contracts for scheduling, roles, training, and shift/labor context.
pub mod staff;
/// Temperament contracts for group-play safety and behavior review evidence.
pub mod temperament;
/// Training contracts for programs, progress, trainer availability, and upsell workflows.
pub mod training;
/// Vaccine contracts for compliance, proof, expiry, and review requirements.
pub mod vaccine;
/// Workflow contracts for tasks, events, reviews, and recommended actions.
pub mod workflow;
