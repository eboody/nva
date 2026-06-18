//! Pet identity values shared by reservations, care notes, and safety checks.
//!
//! Pet names are human-facing labels used in staff handoffs, customer messages, and manager
//! briefings. Safety-critical facts such as species, temperament, vaccines, and medications live
//! in the richer domain records that reference this value.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

/// Pet display name from the customer portal, staff intake, or imported operating record.
///
/// A non-empty bounded name keeps generated care packets and customer drafts intelligible while
/// avoiding blank labels that would force staff to reconcile records manually.
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
pub struct Name(String);
