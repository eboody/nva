use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cadence {
    DailyRoomReset,
    TwiceDailyForExtendedStay,
    TurnoverOnly,
}
