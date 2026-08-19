use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Boarding accommodation vocabulary used for room capacity and housekeeping-labor math.
pub enum Accommodation {
    /// Standard boarding suite option used as baseline capacity and housekeeping labor.
    ClassicSuite,
    /// Premium boarding suite option that may affect capacity, add-on value, and service expectations.
    LuxurySuite,
    /// Cat lodging option kept distinct from dog suites for capacity and care-labor planning.
    CatCondo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Boarding care-feature vocabulary used to explain included labor and customer-update obligations.
pub enum CareFeature {
    /// Daily housekeeping care feature that contributes predictable kennel labor.
    DailyHousekeeping,
    /// Potty-walk care feature that affects labor scheduling and owner expectations.
    PottyWalks,
    /// Bedding care feature that affects room setup and cleaning labor.
    Bedding,
    /// Progress report shared with the customer during care.
    PawgressReport,
    /// Feeding-support feature that can require staff instructions and care-note evidence.
    FeedingSupport,
    /// Medication-support feature that stays safety-sensitive and must not be changed by automation.
    MedicationSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Boarding add-on vocabulary used for reviewed upsell, capacity, and labor signals.
pub enum AddOn {
    /// Playtime add-on that creates extra yard or staff time before it can be offered or scheduled.
    Playtime,
    /// Bath offered before departure from boarding.
    ExitBath,
    /// Premium-suite add-on that changes room value and capacity expectations.
    PremiumSuite,
    /// Grooming service line or care-note category.
    Grooming,
    /// Training-session add-on that requires trainer availability and human-reviewed scheduling.
    TrainingSession,
}
