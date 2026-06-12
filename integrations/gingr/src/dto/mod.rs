pub mod grooming;
pub mod retail;
pub mod training;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSurface {
    NoDocumentedServiceDto { endpoint: &'static str },
}
