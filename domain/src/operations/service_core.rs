use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Operating-system vocabulary that tells source adapters which provider facts need quarantine before domain promotion.
pub enum OperatingSystem {
    /// Provider-neutral reservation and pet-care management system.
    ProviderManagementSystem,
    /// Mixed operating systems require source reconciliation before automation trusts cross-system facts.
    MixedSystems,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Service-line operating bundle for one resort/location.
pub struct ServiceContracts {
    /// Location whose service contracts and outcomes are being compared in local or regional reports.
    pub location_id: LocationId,
    /// Boarding contract that owns stay, suite, minimum-stay, and checkout-exception rules.
    pub boarding: crate::boarding::Contract,
    /// Daycare contract that owns group-play eligibility, package, and ratio rules.
    pub daycare: crate::daycare::Contract,
    /// Grooming contract that owns service duration, rebooking cadence, add-on, and no-show rules.
    pub grooming: crate::grooming::Contract,
    /// Training contract that owns program progress, package, graduation, and trainer handoff rules.
    pub training: crate::training::Contract,
    /// Retail contract that owns catalog, inventory, POS, recommendation, and reorder gates.
    pub retail: crate::retail::Contract,
}

impl ServiceContracts {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Core pet-resort service line used to partition demand, capacity, and labor metrics.
pub enum ServiceLine {
    /// Overnight stay service line.
    Boarding,
    /// Daycare service offering where group-play eligibility and staffing ratios gate automation suggestions.
    Daycare,
    /// Grooming service line or care-note category.
    Grooming,
    /// Training service line or care-note category.
    Training,
    /// Retail service line partitions inventory, checkout, and recommendation work from care-service capacity.
    Retail,
}
