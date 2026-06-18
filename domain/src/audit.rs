use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed event id domain value that keeps raw primitives out of audit workflows.
pub struct EventId(pub Uuid);
