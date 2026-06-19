//! Customer identity and contact values used by reservation and messaging workflows.
//!
//! These newtypes promote portal/import strings into validated customer identifiers and
//! contact values before inbox drafting, reservation triage, or follow-up can use them.
//! They do not prove identity or consent by themselves; adapters must still attach provenance and
//! any required approval gates before live customer communication.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

/// Customer display name as staff, portal records, and customer-facing drafts should show it.
///
/// The value is trimmed and bounded so generated messages, manager briefings, and search indexes
/// can carry a stable customer label without accepting blank source data.
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

/// Customer email address captured from a portal, staff entry, import, or message source.
///
/// This type only enforces the storage/display envelope; workflows must still respect channel
/// consent, approval state, and resort policy before sending outbound email.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 254),
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
pub struct Email(String);

/// Customer phone number text used for call, SMS, and staff-note correlation.
///
/// The type preserves a normalized non-empty contact string for source-derived records; it is not
/// a permission to text or call without the message-policy and review gates required upstream.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 40),
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
pub struct Phone(String);
