//! Resort-location values used to scope policies, schedules, labor, and source data.
//!
//! Location labels and time zones are source-of-truth facts for multi-site automation. They keep
//! manager briefings, reservation windows, and labor-cost reports tied to the correct resort rather
//! than relying on loose strings from individual systems.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

/// Display name for a resort location or brand-specific site.
///
/// This is the human-readable location label used in manager briefings, audit events, and staff
/// workflows across the 170-location portfolio.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct Name(String);

/// IANA-style or provider-supplied timezone identifier for a resort location.
///
/// Timezone is part of the safety boundary for reminders, check-in windows, daily briefings, and
/// labor reporting; adapters should validate provider-specific semantics before relying on it for
/// live scheduling actions.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct Timezone(String);
