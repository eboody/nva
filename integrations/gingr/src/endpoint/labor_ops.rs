use super::{Date, Error, LocationId, Method, Request, Result};

fn push_optional<T: core::fmt::Display>(
    params: &mut Vec<(String, String)>,
    key: &str,
    value: Option<T>,
) {
    if let Some(value) = value {
        params.push((key.to_owned(), value.to_string()));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserId(u64);

impl UserId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

impl core::fmt::Display for UserId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeclockReport {
    start_date: Date,
    end_date: Date,
    location_id: LocationId,
    include_deleted: Option<bool>,
    include_clocked_in: Option<bool>,
    user_ids: Vec<UserId>,
}

impl TimeclockReport {
    pub fn builder() -> TimeclockReportBuilder {
        TimeclockReportBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct TimeclockReportBuilder {
    start_date: Option<Date>,
    end_date: Option<Date>,
    location_id: Option<LocationId>,
    include_deleted: Option<bool>,
    include_clocked_in: Option<bool>,
    user_ids: Vec<UserId>,
}

impl TimeclockReportBuilder {
    pub fn date_range(mut self, start_date: Date, end_date: Date) -> Self {
        self.start_date = Some(start_date);
        self.end_date = Some(end_date);
        self
    }

    pub fn location_id(mut self, location_id: LocationId) -> Self {
        self.location_id = Some(location_id);
        self
    }

    pub fn include_deleted(mut self, include_deleted: bool) -> Self {
        self.include_deleted = Some(include_deleted);
        self
    }

    pub fn include_clocked_in(mut self, include_clocked_in: bool) -> Self {
        self.include_clocked_in = Some(include_clocked_in);
        self
    }

    pub fn user_id(mut self, user_id: UserId) -> Self {
        self.user_ids.push(user_id);
        self
    }

    pub fn build(self) -> Result<TimeclockReport> {
        Ok(TimeclockReport {
            start_date: self.start_date.ok_or(Error::MissingRequiredParameter {
                parameter: "start_date",
            })?,
            end_date: self.end_date.ok_or(Error::MissingRequiredParameter {
                parameter: "end_date",
            })?,
            location_id: self.location_id.ok_or(Error::MissingRequiredParameter {
                parameter: "location_id",
            })?,
            include_deleted: self.include_deleted,
            include_clocked_in: self.include_clocked_in,
            user_ids: self.user_ids,
        })
    }
}

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
