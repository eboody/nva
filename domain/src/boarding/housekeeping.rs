use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for cadence decisions in boarding workflows.
pub enum Cadence {
    /// Daily room reset boarding policy, stay, capacity, or upsell signal.
    DailyRoomReset,
    /// Twice daily for extended stay boarding policy, stay, capacity, or upsell signal.
    TwiceDailyForExtendedStay,
    /// Turnover only boarding policy, stay, capacity, or upsell signal.
    TurnoverOnly,
}
