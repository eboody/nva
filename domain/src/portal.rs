//! Customer-portal identifiers imported from external systems such as Gingr.
//!
//! Portal ids are source facts, not internal authority by themselves: downstream domain
//! workflows must validate and join them to customer/pet/location records before using
//! them for labor planning, customer communication, or read models.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

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
/// External customer-portal identifier retained for source lineage and joins.
pub struct CustomerId(String);
