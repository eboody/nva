/// Provider DTO surface for Gingr grooming payloads before semantic promotion.
pub mod grooming;
/// Provider DTO surface for Gingr retail payloads before semantic promotion.
pub mod retail;
/// Provider DTO surface for Gingr training payloads before semantic promotion.
pub mod training;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Documents provider surfaces where Gingr has endpoints but no stable DTO contract in this crate.
pub enum ProviderSurface {
    /// Endpoint is known, but this crate intentionally has no DTO mapping yet.
    NoDocumentedServiceDto {
        /// Gingr endpoint name whose service DTO is not modeled here.
        endpoint: &'static str,
    },
}
