//! SpacetimeDB realtime runtime adapter for NVA operational review loops.
//!
//! This crate is intentionally an adapter crate, not a domain crate. SpacetimeDB
//! table structs are storage/read-model rows, reducers are command-boundary
//! entrypoints, and app/domain crates remain the source of business rules.

pub mod adapter;
pub mod authz;
pub mod fixture_seed;
pub mod read_model;
pub mod reducers;
pub mod runtime;
pub mod storage;
pub mod tables;

/// Links the bare-metal profiler runtime for the separately built disposable coverage harness.
///
/// This hidden feature-gated symbol is not a reducer, table, authority, or supported client API.
/// Default production builds do not compile it, and it cannot alter the deployable module schema.
#[cfg(feature = "wasm-coverage-runtime")]
#[doc(hidden)]
pub fn __link_wasm_coverage_runtime() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        minicov::coverage_enabled()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

#[cfg(test)]
mod realtime_queue_tests;

#[cfg(all(test, feature = "wasm-coverage-runtime"))]
mod coverage_runtime_contract {
    #[test]
    fn native_all_features_build_has_no_active_wasm_profiler() {
        assert!(!super::__link_wasm_coverage_runtime());
    }
}

pub use reducers::*;
