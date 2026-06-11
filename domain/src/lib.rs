//! Typed foundation for a 170-location pet-resort workflow/agent platform.
//!
//! This crate intentionally models *business contracts* before implementation details:
//! entities, workflow events, agent boundaries, tool traits, and policy decisions.

pub mod agent;
pub mod agents;
pub mod audit;
pub mod booking_triage;
pub mod care;
pub mod customer;
pub mod daily_update;
pub mod document;
pub mod entities;
pub mod incident;
pub mod location;
pub mod message;
pub mod money;
pub mod operations;
pub mod payment;
pub mod pet;
pub mod policy;
pub mod portal;
pub mod reservation;
pub mod temperament;
pub mod tools;
pub mod vaccine;
pub mod workflow;

pub mod prelude {
    pub use crate::agent::{
        ForbiddenAction, Name as AgentName, OutputSchemaName, PolicyInstruction,
        Purpose as AgentPurpose, Spec as AgentSpec, ToolName,
    };
    pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
    pub use crate::entities::{
        ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
        AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
        Location, LocationId, LocationPolicyRefs, ManagerId, PaymentStatus, Pet, PetId,
        PortalAccountRef, PortalProvider, Reservation, ReservationId, ReservationSource,
        ReservationStatus, ServiceKind, Sex, SpayNeuterStatus, Species, StaffId,
        TemperamentProfile,
    };
    pub use crate::operations::{
        StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority,
        StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
    };
    pub use crate::policy::{AutomationLevel, ReviewGate};
    pub use crate::tools::{
        AvailabilityDecision, AvailabilityDenialReason, AvailabilityRequest, AvailabilityResult,
        AvailabilityServiceNotes, AvailabilitySuccessReason, CapacitySnapshotId,
        ReservationUpdateDraft, StatusSuggestionReason, ToolError,
    };
    pub use crate::workflow::{
        AllowedAction, PolicyContext, RecommendedAction, ReviewReason, RiskFlag, Summary,
        VerificationNote, WorkflowEvent, WorkflowEventId, WorkflowEventType, WorkflowResult,
        WorkflowStatus, WorkflowSubject,
    };
}
