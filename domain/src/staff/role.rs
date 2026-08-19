use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Resort labor role that can own or be assigned a staff task.
pub enum Role {
    /// Front desk team handling check-in, checkout, customer, or document work.
    FrontDesk,
    /// Kennel technician team handling pet care, feeding, medication, or cleanup work.
    KennelTechnician,
    /// Groomer handling grooming preparation, service, or follow-up work.
    Groomer,
    /// Trainer handling training assignment, progress, package, or follow-up work.
    Trainer,
    /// Lead staff member triaging work before manager escalation.
    LeadStaff,
    /// Manager accountable for approvals, exceptions, and queue escalation.
    Manager,
}
