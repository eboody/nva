use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

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
pub struct Channel(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 2000),
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
pub struct Body(String);
