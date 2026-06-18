use super::{LocationId, Method, Request};

fn push_optional<T: core::fmt::Display>(
    params: &mut Vec<(String, String)>,
    key: &str,
    value: Option<T>,
) {
    if let Some(value) = value {
        params.push((key.to_owned(), value.to_string()));
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Request descriptor for Gingr report-card file metadata used as source material for Pawgress-style customer updates.
pub struct ReportCardFiles {
    number_days: Option<u64>,
    limit: Option<u64>,
    location_id: Option<LocationId>,
}

impl ReportCardFiles {
    /// Starts a builder that makes each provider parameter explicit before request capture.
    pub fn builder() -> ReportCardFilesBuilder {
        ReportCardFilesBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
/// Builder for report-card file lookup filters by owner, animal, reservation, and location.
pub struct ReportCardFilesBuilder {
    number_days: Option<u64>,
    limit: Option<u64>,
    location_id: Option<LocationId>,
}

impl ReportCardFilesBuilder {
    /// Limits report-card file lookup to a recent day window.
    pub fn number_days(mut self, number_days: u64) -> Self {
        self.number_days = Some(number_days);
        self
    }

    /// Sets the provider result limit so automation does not imply unbounded source coverage.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Scopes the Gingr endpoint request to a location.
    pub fn location_id(mut self, location_id: LocationId) -> Self {
        self.location_id = Some(location_id);
        self
    }

    /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
    pub fn build(self) -> ReportCardFiles {
        ReportCardFiles {
            number_days: self.number_days,
            limit: self.limit,
            location_id: self.location_id,
        }
    }
}

impl Request for ReportCardFiles {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/report_card_files"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        push_optional(&mut params, "number_days", self.number_days);
        push_optional(&mut params, "limit", self.limit);
        push_optional(&mut params, "location_id", self.location_id);
        params
    }
}
