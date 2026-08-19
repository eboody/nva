//! Staff-facing API/runtime DTO contracts.
//!
//! Types in this module are product-owned HTTP payloads for the local runtime shell.
//! They intentionally sit on our side of the boundary: provider DTOs may contribute
//! source evidence and record references, but provider response shapes are not passed
//! through as API contracts. Every exposed workflow DTO should preserve review gates,
//! audit/correlation evidence, labor/outcome fields, and disabled live-side-effect
//! status until a future approved adapter crosses the customer/provider boundary.

mod auth;
mod checkout;
mod contract_observation;
mod data_quality;
mod dto;
mod error_mapping;
mod health;
mod information_lifespan_fixture;
mod inquiry;
mod manager_brief;
mod router;
mod site_finance;
mod state;
mod vaccine;
mod workflow;

use crate::{
    authentication,
    error::{AuthenticationFailure, AuthorizationFailure, ErrorContext, ErrorKind, PublicApiError},
    observability::ObservabilityRuntime,
    public_contract,
};
use app::workflow_repository::OutcomeRepository as _;
use app::workflow_repository::Repository as _;
use app::workflow_repository::source_quality_backlog::Repository as _;
use app::{
    checkout_completion, crm_retention, data_quality_hygiene, information_lifespan,
    manager_daily_brief, site_finance as app_site_finance, workflow_repository,
};
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Extension, MatchedPath, Path, Query, State},
    http::{HeaderName, HeaderValue, Request, Response, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use contract_observation::Protected;
use domain::{
    access, agent, analytics, data_quality as domain_data_quality, entities, message, operations,
    policy, source,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::{Level, Span};
use uuid::Uuid;

use auth::*;
use checkout::*;
use data_quality::*;
use dto::*;
use error_mapping::*;
use health::*;
use inquiry::*;
use manager_brief::*;
use router::*;
use site_finance::*;
use state::*;
use vaccine::*;
use workflow::*;

#[doc(hidden)]
pub use contract_observation::{DeniedLiveEffectIntent, DeniedLiveEffectWorkflow};
pub use router::{router, router_with_state, router_with_test_auth_state};
pub use state::VaccineDocumentState;
