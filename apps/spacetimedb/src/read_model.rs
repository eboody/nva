//! Public subscription-oriented read models.
//!
//! Public rows here are client contracts for realtime dashboards. They are not
//! canonical domain aggregates and should remain derivable from private storage
//! plus app/domain rules.

pub mod manager_queue_item;
pub mod staff_queue_item;

pub use manager_queue_item::ManagerQueueItemRow;
pub use staff_queue_item::{BlockedActionNoticeRow, HygieneOutcomeCardRow, StaffQueueItemRow};
