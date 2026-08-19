use super::{Date, LocationId, Method, Request};

fn push_optional<T: core::fmt::Display>(
    params: &mut Vec<(String, String)>,
    key: &str,
    value: Option<T>,
) {
    if let Some(value) = value {
        params.push((key.to_owned(), value.to_string()));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Display)]
/// Gingr user identifier accepted by labor-operation endpoints.
pub struct UserId(u64);

impl UserId {}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Typed request for Gingr timeclock reports used as labor workflow evidence.
pub struct TimeclockReport {
    start_date: Date,
    end_date: Date,
    location_id: LocationId,
    include_deleted: Option<bool>,
    include_clocked_in: Option<bool>,
    user_ids: Vec<UserId>,
}

impl TimeclockReport {}

impl Request for TimeclockReport {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/timeclock_report"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("start_date".to_owned(), self.start_date.to_string()),
            ("end_date".to_owned(), self.end_date.to_string()),
            ("location_id".to_owned(), self.location_id.to_string()),
        ];
        push_optional(&mut params, "include_deleted", self.include_deleted);
        push_optional(&mut params, "include_clocked_in", self.include_clocked_in);
        params.extend(
            self.user_ids
                .iter()
                .map(|user_id| ("user_ids[]".to_owned(), user_id.to_string())),
        );
        params
    }
}
