//! Boarding housekeeping cadence policies for room resets and turnover planning.
//!
//! Cadences make the labor commitment of a boarding stay visible to planners and manager briefs.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Housekeeping cadence required by a boarding room or suite stay.
pub enum Cadence {
    /// Room should be reset daily during the stay.
    DailyRoomReset,
    /// Extended stay needs twice-daily cleaning attention for labor planning.
    TwiceDailyForExtendedStay,
    /// Cleaning work is limited to post-departure room turnover.
    TurnoverOnly,
}
