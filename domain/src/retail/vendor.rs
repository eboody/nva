//! Vendor models for partner products and external catalog-management flags.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Retail partner product line used for recommendation and catalog relationship documentation.
pub enum Partner {
    /// Virbac calming-care line that may appear in care-sensitive recommendation and catalog reviews.
    VirbacCalmCare,
    /// Purina Pro Plan veterinary supplement line that requires care-aware recommendation review.
    PurinaProPlanVeterinarySupplements,
    /// Purina EN boarding diet line used for in-house diet continuity and low-stock planning.
    PurinaEnBoardingDiet,
}
