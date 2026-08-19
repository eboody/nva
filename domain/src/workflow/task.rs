use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

/// Staff-visible task title summarizing the operational work item.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
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
pub struct Title(String);

/// Task or message body text that carries source evidence and review instructions.
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
