//! Private SpacetimeDB storage rows and codecs.
//!
//! Storage modules own persisted/queryable facts for the realtime adapter. They
//! are allowed to carry SpacetimeDB metadata, indexes, schema versions, and
//! denormalized lookup columns because they are not domain/app objects.

pub mod review_queue;
