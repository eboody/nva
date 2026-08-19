use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        AsRef,
        Serialize,
        Deserialize
    )
)]
/// Provider or source identifier retained as the stable join key.
pub struct Id(String);

impl Id {
    /// Provider/read-model record id exposed for reconciliation joins.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl std::fmt::Debug for Id {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("source::record::Id([REDACTED])")
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[strum(serialize_all = "snake_case")]
/// Kinds of related records that may be attached to source-data lineage.
pub enum Role {
    /// Customer record participating in the workflow.
    Customer,
    /// Pet record participating in the workflow.
    Pet,
    /// Resort location record participating in the workflow.
    Location,
    /// Provider reservation-type identifier used for service reconciliation.
    ReservationType,
    /// Provider invoice identifier tied to reservation/payment reconciliation.
    Invoice,
    /// Provider payment identifier tied to deposit or checkout reconciliation.
    Payment,
    /// Provider service identifier used when mapping provider service types.
    Service,
    /// Staff provider id retained for labor-source reconciliation.
    Staff,
    /// Related record role is unknown, so reconciliation should not assume the link's business meaning.
    Unknown,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Link from a source record to another related provider record.
pub struct RelatedId {
    role: Role,
    id: Id,
}

impl RelatedId {
    /// Related-record role explaining how this source id participates in reconciliation.
    pub const fn role(&self) -> Role {
        self.role
    }

    /// Provider/read-model identifier retained for reconciliation.
    pub const fn id(&self) -> &Id {
        &self.id
    }
}

impl std::fmt::Debug for RelatedId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("source::record::RelatedId([REDACTED])")
    }
}
