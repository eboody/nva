use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
/// Half-open UTC time window used for SLA, staffing, demand, and reporting buckets.
pub struct Window {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl Window {
    /// Creates a time window only when the end follows the start.
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, Error> {
        if end <= start {
            return Err(Error::EndMustFollowStart);
        }
        Ok(Self { start, end })
    }

    /// Start instant of the window.
    pub const fn start(&self) -> DateTime<Utc> {
        self.start
    }

    /// End instant of the window.
    pub const fn end(&self) -> DateTime<Utc> {
        self.end
    }
}

impl<'de> Deserialize<'de> for Window {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawWindow {
            start: DateTime<Utc>,
            end: DateTime<Utc>,
        }

        let raw = RawWindow::deserialize(deserializer)?;
        Self::new(raw.start, raw.end).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Time-window validation failures.
pub enum Error {
    #[error("time window end must follow start")]
    /// The end timestamp did not follow the start timestamp.
    EndMustFollowStart,
}
