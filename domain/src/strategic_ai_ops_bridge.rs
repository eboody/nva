//! Deprecated compatibility facade for strategic AI-operations vocabulary.
//!
//! Canonical callers should use semantic owner modules directly. This file deliberately contains
//! only compatibility re-exports, so ownership, Rustdoc, and review diffs center the canonical
//! modules instead of a catch-all bridge surface.

/// Source-system vocabulary for strategic AI operations beyond the first Gingr/local proof.
pub mod source {
    /// Canonical enterprise source system that can provide evidence for AI-ops recommendations.
    pub use crate::source::System;
}

/// Time primitives used by optimization, financial, and lead-response models.
pub mod time {
    pub use crate::operations::time_bucket::*;
}

/// Shared access, role, visibility, and allowed-use contracts.
pub mod access {
    pub use crate::access::*;
}

/// Identity matching contracts for customer, pet, and household reconciliation.
pub mod identity {
    pub use crate::identity::*;
}

/// Communication consent and purpose contracts shared by leads, retention, and CRM.
pub mod communication {
    pub use crate::consent::*;
}

/// Real-time lead response contracts.
pub mod lead_response {
    pub use crate::lead::response::*;
}

/// CRM/customer-intelligence contracts.
pub mod crm {
    pub use crate::customer::intelligence::*;
}

/// Labor modeling for scheduled coverage and optimization recommendations.
pub mod labor {
    pub use crate::operations::labor::*;
}

/// Capacity and demand optimization contracts.
pub mod capacity {
    pub use crate::operations::capacity::*;
}

/// Knowledge document/citation contracts for permissioned retrieval.
pub mod knowledge {
    pub use crate::agent::knowledge::*;
}

/// Assistant packet contracts for cited/escalated answers.
pub mod assistant {
    pub use crate::agent::assistant::*;
}

/// Site finance facts and review-gated insight contracts.
pub mod financial {
    pub use crate::analytics::finance::*;
}

/// Outcome attribution contracts used before making value claims.
pub mod outcome {
    pub use crate::analytics::outcome::*;
}
