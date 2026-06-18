use super::ProviderSurface;

/// Reports that Gingr training service discovery is endpoint-only until a stable DTO is modeled.
pub fn provider_surface() -> ProviderSurface {
    ProviderSurface::NoDocumentedServiceDto {
        endpoint: "get_services_by_type",
    }
}
