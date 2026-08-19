use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Validated portfolio context used to scope cross-resort automation and reporting.
pub struct Portfolio {
    /// Operator whose portfolio context explains why the same labor and source-governance contracts apply across resorts.
    pub operator: Operator,
    /// Number of resorts used to size portfolio rollups; zero resorts is rejected before outcome metrics are reported.
    pub resort_count: ResortCount,
    /// Portfolio structure used to decide whether a brief is local, brand-level, or cross-brand comparison context.
    pub structure: PortfolioStructure,
    /// Business lines that keep pet-resort automation scoped away from veterinary or equine assumptions unless explicitly modeled.
    pub business_lines: Vec<BusinessLine>,
    /// Pet-resort brands used for navigation and reporting filters, not as automatic permission to change local policy.
    pub brands: Vec<Brand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Portfolio operator vocabulary used to scope source evidence and labor-value claims.
pub enum Operator {
    /// NVA portfolio context for cross-resort reporting; it does not override local manager approval gates.
    NationalVeterinaryAssociates,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Portfolio structure vocabulary that tells reports whether comparisons are single-brand or federated.
pub enum PortfolioStructure {
    /// Multi-brand portfolio context where regional reports compare patterns without assuming one brand policy fits every site.
    FederatedMultiBrand,
    /// Single-brand context where comparisons can use a narrower policy and vocabulary set.
    SingleBrand,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// NVA business-line vocabulary used to keep pet-resort labor claims separate from other NVA operating models.
pub enum BusinessLine {
    /// Veterinary-hospital line of business retained as adjacent context, not a source for pet-resort policy.
    GeneralPracticeVeterinaryHospitals,
    /// Pet-resort line of business where boarding, daycare, grooming, training, and retail workflows are in scope.
    PetResorts,
    /// Equine line of business retained as out-of-scope portfolio context unless a source contract models it directly.
    Equine,
    /// Specialty/emergency hospital context retained so reports do not confuse medical operations with resort labor loops.
    SpecialtyEmergencyHospitals,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Pet-resort brand vocabulary used for portfolio filtering and source reconciliation.
pub enum Brand {
    /// NVA Pet Resorts portfolio label for rollups and navigation across resort brands.
    NvaPetResorts,
    /// PetSuites brand label used when comparing resort workflows that may carry brand-specific naming.
    PetSuites,
    /// Pooch Hotel brand label used for portfolio reports without inventing local policy authority.
    PoochHotel,
    /// Elite Suites brand label for source and reporting filters.
    EliteSuites,
    /// The Bark Side brand label for source and reporting filters.
    TheBarkSide,
    /// Woofdorf Astoria brand label for source and reporting filters.
    WoofdorfAstoria,
    /// Doggie District brand label for source and reporting filters.
    DoggieDistrict,
    /// Local or acquired brand name that staff recognize but the domain cannot classify into a known portfolio brand.
    Other {
        /// Display name retained so a reviewer can map the local brand before it appears in portfolio reporting.
        name: crate::location::Name,
    },
}
