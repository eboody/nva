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

#[derive(Clone, Debug, Default, PartialEq, Eq, bon::Builder)]
/// Request descriptor for Gingr report-card file metadata used as source material for Pawgress-style customer updates.
pub struct ReportCardFiles {
    number_days: Option<u64>,
    limit: Option<u64>,
    location_id: Option<LocationId>,
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
