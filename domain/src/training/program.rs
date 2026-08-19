use core::num::NonZeroU8;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive number of weeks in a Stay-and-Study or other multi-week training program.
pub struct DurationWeeks(NonZeroU8);

impl DurationWeeks {
    /// Rejects zero or unsupported training values before they affect package balances, trainer scheduling, progress reports, or parent summaries.
    pub const fn try_new(value: u8) -> std::result::Result<Self, DurationWeeksError> {
        match NonZeroU8::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(DurationWeeksError::ZeroWeeks),
        }
    }

    /// Promotes an already-proven positive week count without repeating validation.
    pub const fn from_nonzero(value: NonZeroU8) -> Self {
        Self(value)
    }

    /// Preserves the proof that this duration is positive across trusted adapters.
    pub const fn into_nonzero(self) -> NonZeroU8 {
        self.0
    }

    /// Returns the training number used by package balances, scheduling, progress reports, or parent summaries.
    pub const fn get(self) -> u8 {
        self.0.get()
    }
}

impl<'de> Deserialize<'de> for DurationWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Duration validation error for multi-week training programs that cannot use zero weeks.
pub enum DurationWeeksError {
    #[error("training program duration requires at least one week")]
    /// Staff can see the zero weeks training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    ZeroWeeks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Program duration shape used to plan trainer labor and customer expectations.
pub enum Duration {
    /// Staff can see the single session training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    SingleSession,
    /// Staff can see the weeks training state during training enrollment, curriculum, progress, package, trainer-capacity, or follow-up review.
    Weeks(DurationWeeks),
}
