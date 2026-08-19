//! Daycare playgroup assignment decisions after eligibility and coverage are known.
//!
//! ```
//! use domain::{daycare, entities};
//! use uuid::Uuid;
//!
//! let request = daycare::assignment::Request::builder()
//!     .pet_id(entities::PetId::new(uuid::Uuid::from_u128(1)))
//!     .service(daycare::ServiceVariant::AllDayPlay)
//!     .eligibility(daycare::eligibility::GroupPlayDecision::Eligible {
//!         basis: daycare::eligibility::EligibleBasis::CurrentEvidence,
//!     })
//!     .coverage(daycare::coverage::Decision::Sufficient)
//!     .playgroup(daycare::assignment::PlaygroupId::try_new("small-dogs-am").unwrap())
//!     .build();
//!
//! assert!(matches!(
//!     daycare::assignment::Service.assign(request),
//!     daycare::assignment::Decision::Blocked {
//!         gate: domain::policy::ReviewGate::ManagerApproval,
//!         ..
//!     }
//! ));
//! ```

use super::*;

pub use playgroup_id::Id as PlaygroupId;

/// Playgroup identifier chosen from scheduling/source data for daycare assignment review.
pub mod playgroup_id {
    use super::*;

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
    pub struct Id(String);
}

#[derive(Debug, Clone, Default)]
/// Deterministic daycare assignment service for playgroup placement decisions.
pub struct Service;

impl Service {}
