use super::ProviderSurface;

pub fn provider_surface() -> ProviderSurface {
    ProviderSurface::NoDocumentedServiceDto {
        endpoint: "get_services_by_type",
    }
}
