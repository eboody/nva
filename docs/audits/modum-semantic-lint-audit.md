# Modum semantic lint audit

Workspace: `/home/eran/code/pet-resort-agent-foundation`  
Command: `modum check --format json`  
Scope: audit only; no source fixes applied.  

## Executive summary

- Diagnostics audited: **192**
- good lint: **79**
- directionally right / prelude audit: **34**
- directionally right / design prompt: **31**
- directionally right / different solution: **18**
- directionally right / manual review: **15**
- good lint, but service boundary needs review: **8**
- bad lint: **6**
- directionally right / maybe not this fix: **1**

### By lint code

- `api_redundant_leaf_context`: 51
- `api_candidate_semantic_module`: 31
- `namespace_flat_pub_use`: 22
- `namespace_flat_use`: 15
- `api_candidate_semantic_module_unsupported_construct`: 13
- `namespace_flat_pub_use_redundant_leaf_context`: 10
- `api_builder_candidate`: 6
- `namespace_overqualified_callsite_path`: 6
- `api_candidate_child_facet_module`: 5
- `api_raw_id_surface`: 5
- `api_integer_protocol_parameter`: 4
- `internal_organizational_submodule_flatten`: 3
- `namespace_flat_use_preserve_module`: 3
- `namespace_parent_surface`: 3
- `namespace_flat_pub_use_preserve_module`: 2
- `api_semantic_string_scalar`: 2
- `namespace_family_unsupported_construct`: 2
- `api_catch_all_module`: 1
- `api_missing_parent_surface_export`: 1
- `api_redundant_category_suffix`: 1
- `api_repeated_parameter_cluster`: 1
- `api_weak_module_generic_leaf`: 1
- `api_boolean_flag_cluster`: 1
- `api_manual_enum_string_helper`: 1
- `api_string_error_surface`: 1
- `api_stringly_protocol_parameter`: 1

### Highest-impact themes

- `domain::prelude` and root re-exports flatten too many domain values; canonical module paths should remain the truth, with any prelude narrowed or quarantined.
- `domain::policy` and `domain::service` are weak/broad buckets. Several modum suggestions are directionally right but would be better solved by moving concepts to stronger owners (`play::eligibility`, service-line modules, owned facets) rather than chopping suffixes in place.
- Many Gingr integration DTOs expose raw ids/status/path/error strings. Those are good lints: boundary code may speak provider language, but the public boundary should promote provider concepts into typed values early.
- Macro/unsupported-construct warnings are not refactor instructions; they are analysis limits and should become manual inspection tasks.
- Some lints are bad in this repo doctrine: `error.rs` implementation modules and external derive names such as `thiserror::Error` should not be flattened into domain parent surfaces.

## Complete item-by-item audit

### M001 — `api_builder_candidate` — good lint
Location: `domain/src/operations.rs:1305`  
Modum: public entrypoint `new` takes 4 positional parameters (`minutes`, `basis`, `confidence`, `review`); prefer a builder or typed options struct when setup is this configuration-heavy

- **What currently exists:** `new` / public entrypoint `new` takes 4 positional parameters (`minutes`, `basis`, `confidence`, `review`); prefer a builder or typed options struct when setup is this configuration-heavy
- **What should exist:** A named builder or cohesive typed options/configuration value so the call site states field meaning instead of relying on positional order.
- **Why:** Four or more positional public constructor parameters hide meaning at call sites; use a builder or cohesive options/domain type.

```rust
 1303| 
 1304|     impl DurationEstimate {
 1305|         pub const fn new(
 1306|             minutes: AppointmentMinutes,
 1307|             basis: EstimateBasis,
```

### M002 — `api_builder_candidate` — good lint
Location: `domain/src/operations.rs:2313`  
Modum: public entrypoint `new` takes 4 positional parameters (`outcome`, `status`, `evidence`, `milestones`); prefer a builder or typed options struct when setup is this configuration-heavy

- **What currently exists:** `new` / public entrypoint `new` takes 4 positional parameters (`outcome`, `status`, `evidence`, `milestones`); prefer a builder or typed options struct when setup is this configuration-heavy
- **What should exist:** A named builder or cohesive typed options/configuration value so the call site states field meaning instead of relying on positional order.
- **Why:** Four or more positional public constructor parameters hide meaning at call sites; use a builder or cohesive options/domain type.

```rust
 2311| 
 2312|         impl Claim {
 2313|             pub fn new(
 2314|                 outcome: Outcome,
 2315|                 status: ClaimStatus,
```

### M003 — `api_builder_candidate` — good lint
Location: `domain/src/operations.rs:2463`  
Modum: public entrypoint `new` takes 5 positional parameters (`package_id`, `customer_id`, `pet_id`, `policy`, `entries`); prefer a builder or typed options struct when setup is this configuration-heavy

- **What currently exists:** `new` / public entrypoint `new` takes 5 positional parameters (`package_id`, `customer_id`, `pet_id`, `policy`, `entries`); prefer a builder or typed options struct when setup is this configuration-heavy
- **What should exist:** A named builder or cohesive typed options/configuration value so the call site states field meaning instead of relying on positional order.
- **Why:** Four or more positional public constructor parameters hide meaning at call sites; use a builder or cohesive options/domain type.

```rust
 2461| 
 2462|         impl Ledger {
 2463|             pub fn new(
 2464|                 package_id: Id,
 2465|                 customer_id: CustomerId,
```

### M004 — `api_builder_candidate` — good lint
Location: `domain/src/operations.rs:2895`  
Modum: public entrypoint `new` takes 5 positional parameters (`location_id`, `sku`, `on_hand`, `reserved`, `reorder_at`); prefer a builder or typed options struct when setup is this configuration-heavy

- **What currently exists:** `new` / public entrypoint `new` takes 5 positional parameters (`location_id`, `sku`, `on_hand`, `reserved`, `reorder_at`); prefer a builder or typed options struct when setup is this configuration-heavy
- **What should exist:** A named builder or cohesive typed options/configuration value so the call site states field meaning instead of relying on positional order.
- **Why:** Four or more positional public constructor parameters hide meaning at call sites; use a builder or cohesive options/domain type.

```rust
 2893| 
 2894|     impl InventoryPosition {
 2895|         pub fn new(
 2896|             location_id: LocationId,
 2897|             sku: Sku,
```

### M005 — `api_builder_candidate` — directionally right / maybe not this fix
Location: `domain/src/service/boarding/capacity.rs:28`  
Modum: public entrypoint `new` takes 3 positional parameters (`accommodation`, `total`, `occupied`); prefer a builder or typed options struct when setup is this configuration-heavy

- **What currently exists:** `new` / public entrypoint `new` takes 3 positional parameters (`accommodation`, `total`, `occupied`); prefer a builder or typed options struct when setup is this configuration-heavy
- **What should exist:** A named builder or cohesive typed options/configuration value so the call site states field meaning instead of relying on positional order.
- **Why:** Three typed parameters can be acceptable, but this constructor is public and configuration-shaped; review call sites before choosing builder vs typed aggregate.

```rust
   26| 
   27| impl NightlySegmentSnapshot {
   28|     pub const fn new(
   29|         accommodation: accommodation::Kind,
   30|         total: RoomCount,
```

### M006 — `api_builder_candidate` — good lint
Location: `integrations/gingr/src/transport.rs:23`  
Modum: public entrypoint `new` takes 4 positional parameters (`method`, `path`, `parameters`, `sensitive_parameter_names`); prefer a builder or typed options struct when setup is this configuration-heavy

- **What currently exists:** `new` / public entrypoint `new` takes 4 positional parameters (`method`, `path`, `parameters`, `sensitive_parameter_names`); prefer a builder or typed options struct when setup is this configuration-heavy
- **What should exist:** A named builder or cohesive typed options/configuration value so the call site states field meaning instead of relying on positional order.
- **Why:** Four or more positional public constructor parameters hide meaning at call sites; use a builder or cohesive options/domain type.

```rust
   21| 
   22| impl RequestParts {
   23|     pub fn new(
   24|         method: endpoint::Method,
   25|         path: impl Into<String>,
```

### M007 — `api_catch_all_module` — directionally right / different solution
Location: `domain/src/lib.rs:25`  
Modum: `service` is a catch-all public module; prefer a stable domain or facet

- **What currently exists:** `service` / `service` is a catch-all public module; prefer a stable domain or facet
- **What should exist:** Top-level modules named for actual service-line/facet owners (`boarding`, `daycare`, `grooming`, `training`, `retail`, or a deliberately named `service_line` family), not a weak catch-all `service` bucket.
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
   23| pub mod portal;
   24| pub mod reservation;
   25| pub mod service;
   26| pub mod temperament;
   27| pub mod tools;
```

### M008 — `api_missing_parent_surface_export` — good lint
Location: `integrations/gingr/src/endpoint/mod.rs:7`  
Modum: parent surface is missing `endpoint::Reservations`; re-export it so callers do not have to use `endpoint::reservations::Reservations`

- **What currently exists:** `endpoint::Reservations` / parent surface is missing `endpoint::Reservations`; re-export it so callers do not have to use `endpoint::reservations::Reservations`
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    5| pub mod reference_data;
    6| pub mod report_cards_files;
    7| pub mod reservations;
    8| 
    9| use crate::transport;
```

### M009 — `api_redundant_category_suffix` — directionally right / different solution
Location: `domain/src/tools/error.rs:7`  
Modum: `tools::error::ToolError` repeats the `error` category; prefer `tools::error::Tool`

- **What currently exists:** `tools::error::ToolError` / `tools::error::ToolError` repeats the `error` category; prefer `tools::error::Tool`
- **What should exist:** Module-local `Error`/`Result` plus parent re-export (`tools::Error`, `tools::Result`); not `tools::error::Tool` as a new public leaf.
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
    5| pub type Result<T> = std::result::Result<T, ToolError>;
    6| 
    7| #[derive(Debug, Error, Clone, PartialEq, Eq)]
    8| pub enum ToolError {
    9|     #[error("not found: {resource} {id}")]
```

### M010 — `api_redundant_leaf_context` — directionally right / different solution
Location: `domain/src/operations.rs:373`  
Modum: `operations::OperationsRisk` repeats the `operations` context; prefer `operations::Risk`

- **What currently exists:** `operations::OperationsRisk` / `operations::OperationsRisk` repeats the `operations` context; prefer `operations::Risk`
- **What should exist:** operations::Risk
- **Why:** `operations` is broad, so shortening to `operations::Risk`/`Action` may be too vague. The better fix is likely an owned facet such as `operations::risk::Kind`/`operations::action::Kind` or a more specific module.

```rust
  371| }
  372| 
  373| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  374| pub enum OperationsRisk {
  375|     CapacityConstraint { service: ServiceKind },
```

### M011 — `api_redundant_leaf_context` — directionally right / different solution
Location: `domain/src/operations.rs:396`  
Modum: `operations::OperationsAction` repeats the `operations` context; prefer `operations::Action`

- **What currently exists:** `operations::OperationsAction` / `operations::OperationsAction` repeats the `operations` context; prefer `operations::Action`
- **What should exist:** operations::Action
- **Why:** `operations` is broad, so shortening to `operations::Risk`/`Action` may be too vague. The better fix is likely an owned facet such as `operations::risk::Kind`/`operations::action::Kind` or a more specific module.

```rust
  394| }
  395| 
  396| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  397| pub enum OperationsAction {
  398|     CreateInternalTask {
```

### M012 — `api_redundant_leaf_context` — good lint
Location: `domain/src/operations.rs:1193`  
Modum: public API already exposes `operations::grooming::no_show::Policy`; prefer it over `operations::grooming::NoShowPolicy`

- **What currently exists:** `operations::grooming::no_show::Policy` / public API already exposes `operations::grooming::no_show::Policy`; prefer it over `operations::grooming::NoShowPolicy`
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
 1191|     }
 1192| 
 1193|     #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
 1194|     pub enum NoShowPolicy {
 1195|         NoteHistoryOnly,
```

### M013 — `api_redundant_leaf_context` — good lint
Location: `domain/src/operations.rs:1401`  
Modum: `operations::grooming::no_show::NoShowCount` repeats the `no_show` context; prefer `operations::grooming::no_show::Count`

- **What currently exists:** `operations::grooming::no_show::NoShowCount` / `operations::grooming::no_show::NoShowCount` repeats the `no_show` context; prefer `operations::grooming::no_show::Count`
- **What should exist:** operations::grooming::no_show::Count
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
 1399|         use super::*;
 1400| 
 1401|         #[derive(
 1402|             Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
 1403|         )]
```

### M014 — `api_redundant_leaf_context` — good lint
Location: `domain/src/operations.rs:1811`  
Modum: `operations::training::TrainingSessionId` repeats the `training` context; prefer `operations::training::SessionId`

- **What currently exists:** `operations::training::TrainingSessionId` / `operations::training::TrainingSessionId` repeats the `training` context; prefer `operations::training::SessionId`
- **What should exist:** operations::training::SessionId
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
 1809|     pub struct EnrollmentId(String);
 1810| 
 1811|     #[nutype(
 1812|         sanitize(trim),
 1813|         validate(not_empty, len_char_max = 120),
```

### M015 — `api_redundant_leaf_context` — good lint
Location: `domain/src/operations.rs:1976`  
Modum: public API already exposes `operations::training::package::Policy`; prefer it over `operations::training::PackagePolicy`

- **What currently exists:** `operations::training::package::Policy` / public API already exposes `operations::training::package::Policy`; prefer it over `operations::training::PackagePolicy`
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
 1974|         WaitlistUntilTrainerAvailable,
 1975|     }
 1976|     #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
 1977|     pub enum PackagePolicy {
 1978|         PayPerSession,
```

### M016 — `api_redundant_leaf_context` — good lint
Location: `domain/src/payment/mod.rs:10`  
Modum: `payment::PaymentReference` repeats the `payment` context; prefer `payment::Reference`

- **What currently exists:** `payment::PaymentReference` / `payment::PaymentReference` repeats the `payment` context; prefer `payment::Reference`
- **What should exist:** payment::Reference
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
    8| pub use error::{Error, Result};
    9| 
   10| #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
   11| pub struct PaymentReference(String);
   12| 
```

### M017 — `api_redundant_leaf_context` — directionally right / different solution
Location: `domain/src/policy.rs:113`  
Modum: `policy::PolicyDenialReason` repeats the `policy` context; prefer `policy::DenialReason`

- **What currently exists:** `policy::PolicyDenialReason` / `policy::PolicyDenialReason` repeats the `policy` context; prefer `policy::DenialReason`
- **What should exist:** A denial reason under the concept that owns denial: e.g. `play::eligibility::DenialReason` or `eligibility_policy::play::DenialReason`, not a generic `policy::DenialReason` if multiple policy families will exist.
- **Why:** `policy` is too generic here. The smell is real, but `policy::PlayEligibility` is still under a weak bucket; prefer a stronger path such as `play::eligibility::Policy`, `play::eligibility::DenialReason`, or `eligibility_policy::Play` depending on ownership.

```rust
  111| }
  112| 
  113| #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  114| pub enum PolicyDenialReason {
  115|     ManagerApprovalRequired,
```

### M018 — `api_redundant_leaf_context` — directionally right / different solution
Location: `domain/src/policy.rs:176`  
Modum: `policy::PlayEligibilityPolicy` repeats the `policy` context; prefer `policy::PlayEligibility`

- **What currently exists:** `policy::PlayEligibilityPolicy` / `policy::PlayEligibilityPolicy` repeats the `policy` context; prefer `policy::PlayEligibility`
- **What should exist:** A stronger owner path: likely `play::eligibility::Policy` / `play::eligibility::ConservativePolicy`, or `eligibility_policy::Play` if policies are the family. Avoid the weak bucket `policy::{PlayEligibility, ConservativePlayEligibility}` as the final design.
- **Why:** `policy` is too generic here. The smell is real, but `policy::PlayEligibility` is still under a weak bucket; prefer a stronger path such as `play::eligibility::Policy`, `play::eligibility::DenialReason`, or `eligibility_policy::Play` depending on ownership.

```rust
  174| }
  175| 
  176| pub trait PlayEligibilityPolicy {
  177|     fn decide(&self, pet: &Pet, service: &ServiceKind) -> PlayEligibilityDecision;
  178| }
```

### M019 — `api_redundant_leaf_context` — directionally right / different solution
Location: `domain/src/policy.rs:180`  
Modum: `policy::ConservativePlayEligibilityPolicy` repeats the `policy` context; prefer `policy::ConservativePlayEligibility`

- **What currently exists:** `policy::ConservativePlayEligibilityPolicy` / `policy::ConservativePlayEligibilityPolicy` repeats the `policy` context; prefer `policy::ConservativePlayEligibility`
- **What should exist:** A stronger owner path: likely `play::eligibility::Policy` / `play::eligibility::ConservativePolicy`, or `eligibility_policy::Play` if policies are the family. Avoid the weak bucket `policy::{PlayEligibility, ConservativePlayEligibility}` as the final design.
- **Why:** `policy` is too generic here. The smell is real, but `policy::PlayEligibility` is still under a weak bucket; prefer a stronger path such as `play::eligibility::Policy`, `play::eligibility::DenialReason`, or `eligibility_policy::Play` depending on ownership.

```rust
  178| }
  179| 
  180| /// PetSuites/NVA-inspired default from public policy pages.
  181| ///
  182| /// This is intentionally conservative: it can route to day boarding / review, but it
```

### M020 — `api_redundant_leaf_context` — good lint, but service boundary needs review
Location: `domain/src/service/boarding/mod.rs:174`  
Modum: public API already exposes `service::boarding::minimum_stay::Reason`; prefer it over `service::boarding::MinimumStayReason`

- **What currently exists:** `service::boarding::minimum_stay::Reason` / public API already exposes `service::boarding::minimum_stay::Reason`; prefer it over `service::boarding::MinimumStayReason`
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** The child facet already carries meaning, so the flattened parent alias is likely a duplicate surface. Also reassess whether `service` itself should remain the root boundary.

```rust
  172| }
  173| 
  174| #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  175| pub enum MinimumStayReason {
  176|     StandardPolicy,
```

### M021 — `api_redundant_leaf_context` — good lint, but service boundary needs review
Location: `domain/src/service/boarding/mod.rs:181`  
Modum: public API already exposes `service::boarding::cancellation::Policy`; prefer it over `service::boarding::CancellationPolicy`

- **What currently exists:** `service::boarding::cancellation::Policy` / public API already exposes `service::boarding::cancellation::Policy`; prefer it over `service::boarding::CancellationPolicy`
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** The child facet already carries meaning, so the flattened parent alias is likely a duplicate surface. Also reassess whether `service` itself should remain the root boundary.

```rust
  179| }
  180| 
  181| #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  182| pub struct CancellationPolicy {
  183|     pub notice: NoticeHours,
```

### M022 — `api_redundant_leaf_context` — good lint, but service boundary needs review
Location: `domain/src/service/boarding/mod.rs:193`  
Modum: public API already exposes `service::boarding::cancellation::Penalty`; prefer it over `service::boarding::CancellationPenalty`

- **What currently exists:** `service::boarding::cancellation::Penalty` / public API already exposes `service::boarding::cancellation::Penalty`; prefer it over `service::boarding::CancellationPenalty`
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** The child facet already carries meaning, so the flattened parent alias is likely a duplicate surface. Also reassess whether `service` itself should remain the root boundary.

```rust
  191| }
  192| 
  193| #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  194| pub enum CancellationPenalty {
  195|     None,
```

### M023 — `api_redundant_leaf_context` — good lint, but service boundary needs review
Location: `domain/src/service/boarding/mod.rs:213`  
Modum: public API already exposes `service::boarding::housekeeping::Cadence`; prefer it over `service::boarding::HousekeepingCadence`

- **What currently exists:** `service::boarding::housekeeping::Cadence` / public API already exposes `service::boarding::housekeeping::Cadence`; prefer it over `service::boarding::HousekeepingCadence`
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** The child facet already carries meaning, so the flattened parent alias is likely a duplicate surface. Also reassess whether `service` itself should remain the root boundary.

```rust
  211| }
  212| 
  213| #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  214| pub enum HousekeepingCadence {
  215|     DailyRoomReset,
```

### M024 — `api_redundant_leaf_context` — good lint, but service boundary needs review
Location: `domain/src/service/boarding/mod.rs:220`  
Modum: public API already exposes `service::boarding::handoff::Requirement`; prefer it over `service::boarding::HandoffRequirement`

- **What currently exists:** `service::boarding::handoff::Requirement` / public API already exposes `service::boarding::handoff::Requirement`; prefer it over `service::boarding::HandoffRequirement`
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** The child facet already carries meaning, so the flattened parent alias is likely a duplicate surface. Also reassess whether `service` itself should remain the root boundary.

```rust
  218| }
  219| 
  220| #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  221| pub enum HandoffRequirement {
  222|     ArrivalCareReview,
```

### M025 — `api_redundant_leaf_context` — good lint, but service boundary needs review
Location: `domain/src/service/daycare/attendance.rs:25`  
Modum: `service::daycare::attendance::AttendanceDays` repeats the `attendance` context; prefer `service::daycare::attendance::Days`

- **What currently exists:** `service::daycare::attendance::AttendanceDays` / `service::daycare::attendance::AttendanceDays` repeats the `attendance` context; prefer `service::daycare::attendance::Days`
- **What should exist:** service::daycare::attendance::Days
- **Why:** The child facet already carries meaning, so the flattened parent alias is likely a duplicate surface. Also reassess whether `service` itself should remain the root boundary.

```rust
   23| }
   24| 
   25| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   26| pub struct AttendanceDays(Vec<chrono::Weekday>);
   27| 
```

### M026 — `api_redundant_leaf_context` — good lint, but service boundary needs review
Location: `domain/src/service/daycare/attendance.rs:41`  
Modum: `service::daycare::attendance::AttendanceDaysError` repeats the `attendance` context; prefer `service::daycare::attendance::DaysError`

- **What currently exists:** `service::daycare::attendance::AttendanceDaysError` / `service::daycare::attendance::AttendanceDaysError` repeats the `attendance` context; prefer `service::daycare::attendance::DaysError`
- **What should exist:** service::daycare::attendance::DaysError
- **Why:** The child facet already carries meaning, so the flattened parent alias is likely a duplicate surface. Also reassess whether `service` itself should remain the root boundary.

```rust
   39| }
   40| 
   41| #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
   42| pub enum AttendanceDaysError {
   43|     #[error("daycare attendance recurrence requires at least one weekday")]
```

### M027 — `api_redundant_leaf_context` — good lint, but service boundary needs review
Location: `domain/src/service/daycare/mod.rs:143`  
Modum: public API already exposes `service::daycare::incident::Policy`; prefer it over `service::daycare::IncidentPolicy`

- **What currently exists:** `service::daycare::incident::Policy` / public API already exposes `service::daycare::incident::Policy`; prefer it over `service::daycare::IncidentPolicy`
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** The child facet already carries meaning, so the flattened parent alias is likely a duplicate surface. Also reassess whether `service` itself should remain the root boundary.

```rust
  141| }
  142| 
  143| #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  144| pub enum IncidentPolicy {
  145|     StaffNoteOnly,
```

### M028 — `api_redundant_leaf_context` — good lint
Location: `domain/src/temperament.rs:55`  
Modum: `temperament::TemperamentRating` repeats the `temperament` context; prefer `temperament::Rating`

- **What currently exists:** `temperament::TemperamentRating` / `temperament::TemperamentRating` repeats the `temperament` context; prefer `temperament::Rating`
- **What should exist:** temperament::Rating
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
   53| }
   54| 
   55| #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
   56| pub enum TemperamentRating {
   57|     Easygoing,
```

### M029 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:155`  
Modum: `tools::portal::PortalLookup` repeats the `portal` context; prefer `tools::portal::Lookup`

- **What currently exists:** `tools::portal::PortalLookup` / `tools::portal::PortalLookup` repeats the `portal` context; prefer `tools::portal::Lookup`
- **What should exist:** tools::portal::Lookup
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  153|     use super::*;
  154| 
  155|     #[async_trait]
  156|     pub trait PortalLookup: Send + Sync {
  157|         async fn lookup(&self, request: LookupRequest) -> Result<LookupResult>;
```

### M030 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:566`  
Modum: `tools::media::MediaCapture` repeats the `media` context; prefer `tools::media::Capture`

- **What currently exists:** `tools::media::MediaCapture` / `tools::media::MediaCapture` repeats the `media` context; prefer `tools::media::Capture`
- **What should exist:** tools::media::Capture
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  564|     use super::*;
  565| 
  566|     #[async_trait]
  567|     pub trait MediaCapture: Send + Sync {
  568|         async fn request_snapshot(
```

### M031 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:574`  
Modum: `tools::media::MediaSnapshotRequest` repeats the `media` context; prefer `tools::media::SnapshotRequest`

- **What currently exists:** `tools::media::MediaSnapshotRequest` / `tools::media::MediaSnapshotRequest` repeats the `media` context; prefer `tools::media::SnapshotRequest`
- **What should exist:** tools::media::SnapshotRequest
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  572|     }
  573| 
  574|     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  575|     pub struct MediaSnapshotRequest {
  576|         pub location_id: LocationId,
```

### M032 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:581`  
Modum: `tools::media::MediaSnapshotResult` repeats the `media` context; prefer `tools::media::SnapshotResult`

- **What currently exists:** `tools::media::MediaSnapshotResult` / `tools::media::MediaSnapshotResult` repeats the `media` context; prefer `tools::media::SnapshotResult`
- **What should exist:** tools::media::SnapshotResult
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  579|     }
  580| 
  581|     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  582|     pub enum MediaSnapshotResult {
  583|         Captured { media_ref: MediaRef },
```

### M033 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:594`  
Modum: `tools::media::MediaUnavailableReason` repeats the `media` context; prefer `tools::media::UnavailableReason`

- **What currently exists:** `tools::media::MediaUnavailableReason` / `tools::media::MediaUnavailableReason` repeats the `media` context; prefer `tools::media::UnavailableReason`
- **What should exist:** tools::media::UnavailableReason
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  592|     }
  593| 
  594|     #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  595|     pub enum MediaUnavailableReason {
  596|         CameraOffline,
```

### M034 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:618`  
Modum: `tools::media::MediaRef` repeats the `media` context; prefer `tools::media::Ref`

- **What currently exists:** `tools::media::MediaRef` / `tools::media::MediaRef` repeats the `media` context; prefer `tools::media::Ref`
- **What should exist:** tools::media::Ref
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  616|     pub struct CameraId(String);
  617| 
  618|     #[nutype(
  619|         sanitize(trim),
  620|         validate(not_empty, len_char_max = 240),
```

### M035 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:639`  
Modum: `tools::hermes::HermesAutomationHooks` repeats the `hermes` context; prefer `tools::hermes::AutomationHooks`

- **What currently exists:** `tools::hermes::HermesAutomationHooks` / `tools::hermes::HermesAutomationHooks` repeats the `hermes` context; prefer `tools::hermes::AutomationHooks`
- **What should exist:** tools::hermes::AutomationHooks
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  637|     use super::*;
  638| 
  639|     #[async_trait]
  640|     pub trait HermesAutomationHooks: Send + Sync {
  641|         async fn draft_task(&self, request: TaskDraftRequest) -> Result<TaskDraftResult>;
```

### M036 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:682`  
Modum: `tools::hermes::HermesDraftStatus` repeats the `hermes` context; prefer `tools::hermes::DraftStatus`

- **What currently exists:** `tools::hermes::HermesDraftStatus` / `tools::hermes::HermesDraftStatus` repeats the `hermes` context; prefer `tools::hermes::DraftStatus`
- **What should exist:** tools::hermes::DraftStatus
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  680|     }
  681| 
  682|     #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  683|     pub enum HermesDraftStatus {
  684|         Drafted,
```

### M037 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:729`  
Modum: `tools::hermes::HermesTaskId` repeats the `hermes` context; prefer `tools::hermes::TaskId`

- **What currently exists:** `tools::hermes::HermesTaskId` / `tools::hermes::HermesTaskId` repeats the `hermes` context; prefer `tools::hermes::TaskId`
- **What should exist:** tools::hermes::TaskId
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  727|     pub struct ScheduleName(String);
  728| 
  729|     #[nutype(
  730|         sanitize(trim),
  731|         validate(not_empty, len_char_max = 160),
```

### M038 — `api_redundant_leaf_context` — good lint
Location: `domain/src/tools.rs:746`  
Modum: `tools::hermes::HermesScheduleId` repeats the `hermes` context; prefer `tools::hermes::ScheduleId`

- **What currently exists:** `tools::hermes::HermesScheduleId` / `tools::hermes::HermesScheduleId` repeats the `hermes` context; prefer `tools::hermes::ScheduleId`
- **What should exist:** tools::hermes::ScheduleId
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  744|     pub struct HermesTaskId(String);
  745| 
  746|     #[nutype(
  747|         sanitize(trim),
  748|         validate(not_empty, len_char_max = 160),
```

### M039 — `api_redundant_leaf_context` — good lint
Location: `domain/src/workflow.rs:10`  
Modum: `workflow::WorkflowEventId` repeats the `workflow` context; prefer `workflow::EventId`

- **What currently exists:** `workflow::WorkflowEventId` / `workflow::WorkflowEventId` repeats the `workflow` context; prefer `workflow::EventId`
- **What should exist:** workflow::EventId
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
    8| use crate::policy::{AutomationLevel, ReviewGate};
    9| 
   10| #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
   11| pub struct WorkflowEventId(pub Uuid);
   12| 
```

### M040 — `api_redundant_leaf_context` — good lint
Location: `domain/src/workflow.rs:235`  
Modum: `workflow::status_update::ReservationStatusUpdate` repeats the `status_update` context; prefer `workflow::status_update::Reservation`

- **What currently exists:** `workflow::status_update::ReservationStatusUpdate` / `workflow::status_update::ReservationStatusUpdate` repeats the `status_update` context; prefer `workflow::status_update::Reservation`
- **What should exist:** workflow::status_update::Reservation
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  233|     }
  234| 
  235|     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  236|     pub struct ReservationStatusUpdate {
  237|         pub status: ReservationStatus,
```

### M041 — `api_redundant_leaf_context` — good lint
Location: `domain/src/workflow.rs:248`  
Modum: `workflow::WorkflowEvent` repeats the `workflow` context; prefer `workflow::Event`

- **What currently exists:** `workflow::WorkflowEvent` / `workflow::WorkflowEvent` repeats the `workflow` context; prefer `workflow::Event`
- **What should exist:** workflow::Event
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  246| }
  247| 
  248| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  249| pub struct WorkflowEvent {
  250|     pub event_id: WorkflowEventId,
```

### M042 — `api_redundant_leaf_context` — good lint
Location: `domain/src/workflow.rs:259`  
Modum: `workflow::WorkflowEventType` repeats the `workflow` context; prefer `workflow::EventType`

- **What currently exists:** `workflow::WorkflowEventType` / `workflow::WorkflowEventType` repeats the `workflow` context; prefer `workflow::EventType`
- **What should exist:** workflow::EventType
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  257| }
  258| 
  259| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  260| pub enum WorkflowEventType {
  261|     InquiryReceived,
```

### M043 — `api_redundant_leaf_context` — good lint
Location: `domain/src/workflow.rs:277`  
Modum: `workflow::WorkflowSubject` repeats the `workflow` context; prefer `workflow::Subject`

- **What currently exists:** `workflow::WorkflowSubject` / `workflow::WorkflowSubject` repeats the `workflow` context; prefer `workflow::Subject`
- **What should exist:** workflow::Subject
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  275| }
  276| 
  277| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  278| pub enum WorkflowSubject {
  279|     Customer(CustomerId),
```

### M044 — `api_redundant_leaf_context` — good lint
Location: `domain/src/workflow.rs:307`  
Modum: `workflow::WorkflowResult` repeats the `workflow` context; prefer `workflow::Result`

- **What currently exists:** `workflow::WorkflowResult` / `workflow::WorkflowResult` repeats the `workflow` context; prefer `workflow::Result`
- **What should exist:** workflow::Result
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  305| }
  306| 
  307| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  308| pub struct WorkflowResult<T> {
  309|     pub status: WorkflowStatus,
```

### M045 — `api_redundant_leaf_context` — good lint
Location: `domain/src/workflow.rs:318`  
Modum: `workflow::WorkflowStatus` repeats the `workflow` context; prefer `workflow::Status`

- **What currently exists:** `workflow::WorkflowStatus` / `workflow::WorkflowStatus` repeats the `workflow` context; prefer `workflow::Status`
- **What should exist:** workflow::Status
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  316| }
  317| 
  318| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  319| pub enum WorkflowStatus {
  320|     Completed,
```

### M046 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/config.rs:159`  
Modum: `config::ClientConfig` repeats the `config` context; prefer `config::Client`

- **What currently exists:** `config::ClientConfig` / `config::ClientConfig` repeats the `config` context; prefer `config::Client`
- **What should exist:** config::Client
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  157| }
  158| 
  159| #[derive(Clone)]
  160| pub struct ClientConfig {
  161|     base_url: BaseUrl,
```

### M047 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/endpoint/reservations.rs:146`  
Modum: `endpoint::reservations::ReservationsBuilder` repeats the `reservations` context; prefer `endpoint::reservations::Builder`

- **What currently exists:** `endpoint::reservations::ReservationsBuilder` / `endpoint::reservations::ReservationsBuilder` repeats the `reservations` context; prefer `endpoint::reservations::Builder`
- **What should exist:** endpoint::reservations::Builder
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  144| }
  145| 
  146| #[derive(Clone, Debug, PartialEq, Eq)]
  147| pub struct ReservationsBuilder {
  148|     checked_in: bool,
```

### M048 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/endpoint/reservations.rs:310`  
Modum: `endpoint::reservations::ReservationsByAnimal` repeats the `reservations` context; prefer `endpoint::reservations::ByAnimal`

- **What currently exists:** `endpoint::reservations::ReservationsByAnimal` / `endpoint::reservations::ReservationsByAnimal` repeats the `reservations` context; prefer `endpoint::reservations::ByAnimal`
- **What should exist:** endpoint::reservations::ByAnimal
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  308| }
  309| 
  310| #[derive(Clone, Debug, PartialEq, Eq)]
  311| pub struct ReservationsByAnimal {
  312|     animal_id: AnimalId,
```

### M049 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/endpoint/reservations.rs:325`  
Modum: `endpoint::reservations::ReservationsByAnimalBuilder` repeats the `reservations` context; prefer `endpoint::reservations::ByAnimalBuilder`

- **What currently exists:** `endpoint::reservations::ReservationsByAnimalBuilder` / `endpoint::reservations::ReservationsByAnimalBuilder` repeats the `reservations` context; prefer `endpoint::reservations::ByAnimalBuilder`
- **What should exist:** endpoint::reservations::ByAnimalBuilder
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  323| }
  324| 
  325| #[derive(Clone, Debug, Default)]
  326| pub struct ReservationsByAnimalBuilder {
  327|     animal_id: Option<AnimalId>,
```

### M050 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/endpoint/reservations.rs:380`  
Modum: `endpoint::reservations::ReservationsByOwner` repeats the `reservations` context; prefer `endpoint::reservations::ByOwner`

- **What currently exists:** `endpoint::reservations::ReservationsByOwner` / `endpoint::reservations::ReservationsByOwner` repeats the `reservations` context; prefer `endpoint::reservations::ByOwner`
- **What should exist:** endpoint::reservations::ByOwner
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  378| }
  379| 
  380| #[derive(Clone, Debug, PartialEq, Eq)]
  381| pub struct ReservationsByOwner {
  382|     owner_id: OwnerId,
```

### M051 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/endpoint/reservations.rs:395`  
Modum: `endpoint::reservations::ReservationsByOwnerBuilder` repeats the `reservations` context; prefer `endpoint::reservations::ByOwnerBuilder`

- **What currently exists:** `endpoint::reservations::ReservationsByOwnerBuilder` / `endpoint::reservations::ReservationsByOwnerBuilder` repeats the `reservations` context; prefer `endpoint::reservations::ByOwnerBuilder`
- **What should exist:** endpoint::reservations::ByOwnerBuilder
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  393| }
  394| 
  395| #[derive(Clone, Debug, Default)]
  396| pub struct ReservationsByOwnerBuilder {
  397|     owner_id: Option<OwnerId>,
```

### M052 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/webhook.rs:12`  
Modum: `webhook::WebhookSignatureKey` repeats the `webhook` context; prefer `webhook::SignatureKey`

- **What currently exists:** `webhook::WebhookSignatureKey` / `webhook::WebhookSignatureKey` repeats the `webhook` context; prefer `webhook::SignatureKey`
- **What should exist:** webhook::SignatureKey
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
   10| type HmacSha256 = Hmac<Sha256>;
   11| 
   12| #[derive(Clone)]
   13| pub struct WebhookSignatureKey(SecretString);
   14| 
```

### M053 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/webhook.rs:38`  
Modum: `webhook::WebhookEventType` repeats the `webhook` context; prefer `webhook::EventType`

- **What currently exists:** `webhook::WebhookEventType` / `webhook::WebhookEventType` repeats the `webhook` context; prefer `webhook::EventType`
- **What should exist:** webhook::EventType
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
   36| }
   37| 
   38| #[derive(Clone, Debug, PartialEq, Eq)]
   39| pub enum WebhookEventType {
   40|     CheckIn,
```

### M054 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/webhook.rs:93`  
Modum: `webhook::WebhookEntityType` repeats the `webhook` context; prefer `webhook::EntityType`

- **What currently exists:** `webhook::WebhookEntityType` / `webhook::WebhookEntityType` repeats the `webhook` context; prefer `webhook::EntityType`
- **What should exist:** webhook::EntityType
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
   91| }
   92| 
   93| #[derive(Clone, Debug, PartialEq, Eq)]
   94| pub enum WebhookEntityType {
   95|     Reservation,
```

### M055 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/webhook.rs:127`  
Modum: `webhook::WebhookEntityId` repeats the `webhook` context; prefer `webhook::EntityId`

- **What currently exists:** `webhook::WebhookEntityId` / `webhook::WebhookEntityId` repeats the `webhook` context; prefer `webhook::EntityId`
- **What should exist:** webhook::EntityId
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  125| }
  126| 
  127| #[derive(Clone, Debug, PartialEq, Eq)]
  128| pub struct WebhookEntityId(String);
  129| 
```

### M056 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/webhook.rs:154`  
Modum: `webhook::WebhookEnvelope` repeats the `webhook` context; prefer `webhook::Envelope`

- **What currently exists:** `webhook::WebhookEnvelope` / `webhook::WebhookEnvelope` repeats the `webhook` context; prefer `webhook::Envelope`
- **What should exist:** webhook::Envelope
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  152| }
  153| 
  154| #[derive(Clone, PartialEq)]
  155| pub struct WebhookEnvelope {
  156|     wire: WireWebhookEnvelope,
```

### M057 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/webhook.rs:236`  
Modum: `webhook::VerifiedWebhook` repeats the `webhook` context; prefer `webhook::Verified`

- **What currently exists:** `webhook::VerifiedWebhook` / `webhook::VerifiedWebhook` repeats the `webhook` context; prefer `webhook::Verified`
- **What should exist:** webhook::Verified
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  234| }
  235| 
  236| #[derive(Clone, Debug, PartialEq)]
  237| pub struct VerifiedWebhook {
  238|     event_type: WebhookEventType,
```

### M058 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/webhook.rs:315`  
Modum: `webhook::WebhookParseError` repeats the `webhook` context; prefer `webhook::ParseError`

- **What currently exists:** `webhook::WebhookParseError` / `webhook::WebhookParseError` repeats the `webhook` context; prefer `webhook::ParseError`
- **What should exist:** webhook::ParseError
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  313| }
  314| 
  315| #[derive(Debug, thiserror::Error)]
  316| pub enum WebhookParseError {
  317|     #[error("invalid Gingr webhook JSON: {0}")]
```

### M059 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/webhook.rs:321`  
Modum: `webhook::WebhookVerificationError` repeats the `webhook` context; prefer `webhook::VerificationError`

- **What currently exists:** `webhook::WebhookVerificationError` / `webhook::WebhookVerificationError` repeats the `webhook` context; prefer `webhook::VerificationError`
- **What should exist:** webhook::VerificationError
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  319| }
  320| 
  321| #[derive(Debug, thiserror::Error, PartialEq, Eq)]
  322| pub enum WebhookVerificationError {
  323|     #[error("Gingr webhook is missing required field {field}")]
```

### M060 — `api_redundant_leaf_context` — good lint
Location: `integrations/gingr/src/webhook.rs:333`  
Modum: `webhook::WebhookAck` repeats the `webhook` context; prefer `webhook::Ack`

- **What currently exists:** `webhook::WebhookAck` / `webhook::WebhookAck` repeats the `webhook` context; prefer `webhook::Ack`
- **What should exist:** webhook::Ack
- **Why:** The parent path carries the repeated context; shorten the leaf or keep the semantic child module visible at call sites.

```rust
  331| }
  332| 
  333| #[derive(Clone, Debug, PartialEq, Eq)]
  334| pub enum WebhookAck {
  335|     Processed,
```

### M061 — `api_repeated_parameter_cluster` — directionally right / different solution
Location: `domain/src/booking_triage.rs:304`  
Modum: public entrypoints `RuleEvaluation::unknown`, `RuleEvaluation::needs_human_approval`, `RuleEvaluation::hard_block` repeat the same positional parameter cluster (`rule_id`, `failure_code`, `readiness_bucket`, `human_approval_required`, `evidence_refs`); prefer a shared options type or `bon` builder instead of duplicating the call shape

- **What currently exists:** `RuleEvaluation::unknown` / public entrypoints `RuleEvaluation::unknown`, `RuleEvaluation::needs_human_approval`, `RuleEvaluation::hard_block` repeat the same positional parameter cluster (`rule_id`, `failure_code`, `readiness_bucket`, `human_approval_required`, `evidence_refs`); prefer a shared options type or `bon` builder instead of duplicating the call shape
- **What should exist:** A shared typed evidence/evaluation input or builder for the repeated rule-evaluation cluster; not three public constructors with the same parameter train.
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
  302|     }
  303| 
  304|     pub fn unknown(
  305|         rule_id: RuleId,
  306|         failure_code: FailureCode,
```

### M062 — `api_weak_module_generic_leaf` — directionally right / different solution
Location: `integrations/gingr/src/transport.rs:6`  
Modum: `transport::Error` is too generic for weak module `transport`; keep the domain in the leaf or choose a stronger module

- **What currently exists:** `transport::Error` / `transport::Error` is too generic for weak module `transport`; keep the domain in the leaf or choose a stronger module
- **What should exist:** Either keep `transport::Error` if `transport` is an intentional boundary, or make the boundary stronger (`http::transport::Error`, `gingr::transport::Error`) rather than inventing a vague longer leaf.
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
    4| pub type Result<T> = core::result::Result<T, Error>;
    5| 
    6| #[derive(Debug, thiserror::Error)]
    7| pub enum Error {
    8|     #[error("failed to construct Gingr URL: {0}")]
```

### M063 — `internal_organizational_submodule_flatten` — bad lint
Location: `domain/src/money/mod.rs:1`  
Modum: internal organizational module `error` should usually be flattened or renamed so the category does not carry the naming burden

- **What currently exists:** `error` / internal organizational module `error` should usually be flattened or renamed so the category does not carry the naming burden
- **What should exist:** Keep `error.rs` as an implementation module when it owns the module-local `Error` and `Result`; parent modules should re-export the canonical error surface.
- **Why:** This conflicts with the repo doctrine: module-local `error.rs` plus `Error`/`Result` re-exports are good, and derive-macro paths like `thiserror::Error` are not semantic domain call sites.

```rust
    1| mod error;
    2| 
    3| use serde::{Deserialize, Deserializer, Serialize};
```

### M064 — `internal_organizational_submodule_flatten` — bad lint
Location: `domain/src/payment/mod.rs:1`  
Modum: internal organizational module `error` should usually be flattened or renamed so the category does not carry the naming burden

- **What currently exists:** `error` / internal organizational module `error` should usually be flattened or renamed so the category does not carry the naming burden
- **What should exist:** Keep `error.rs` as an implementation module when it owns the module-local `Error` and `Result`; parent modules should re-export the canonical error surface.
- **Why:** This conflicts with the repo doctrine: module-local `error.rs` plus `Error`/`Result` re-exports are good, and derive-macro paths like `thiserror::Error` are not semantic domain call sites.

```rust
    1| mod error;
    2| 
    3| use chrono::{DateTime, Utc};
```

### M065 — `internal_organizational_submodule_flatten` — bad lint
Location: `domain/src/reservation/mod.rs:1`  
Modum: internal organizational module `error` should usually be flattened or renamed so the category does not carry the naming burden

- **What currently exists:** `error` / internal organizational module `error` should usually be flattened or renamed so the category does not carry the naming burden
- **What should exist:** Keep `error.rs` as an implementation module when it owns the module-local `Error` and `Result`; parent modules should re-export the canonical error surface.
- **Why:** This conflicts with the repo doctrine: module-local `error.rs` plus `Error`/`Result` re-exports are good, and derive-macro paths like `thiserror::Error` are not semantic domain call sites.

```rust
    1| mod error;
    2| 
    3| use serde::{Deserialize, Deserializer, Serialize};
```

### M066 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:37`  
Modum: flattened re-export hides namespace context for `AuditAction`; prefer `entities::AuditAction`

- **What currently exists:** `AuditAction` / flattened re-export hides namespace context for `AuditAction`; prefer `entities::AuditAction`
- **What should exist:** entities::AuditAction
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M067 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:37`  
Modum: flattened re-export hides namespace context for `AuditEvent`; prefer `entities::AuditEvent`

- **What currently exists:** `AuditEvent` / flattened re-export hides namespace context for `AuditEvent`; prefer `entities::AuditEvent`
- **What should exist:** entities::AuditEvent
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M068 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:37`  
Modum: flattened re-export hides namespace context for `AuditMetadataKey`; prefer `entities::AuditMetadataKey`

- **What currently exists:** `AuditMetadataKey` / flattened re-export hides namespace context for `AuditMetadataKey`; prefer `entities::AuditMetadataKey`
- **What should exist:** entities::AuditMetadataKey
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M069 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:37`  
Modum: flattened re-export hides namespace context for `AuditMetadataValue`; prefer `entities::AuditMetadataValue`

- **What currently exists:** `AuditMetadataValue` / flattened re-export hides namespace context for `AuditMetadataValue`; prefer `entities::AuditMetadataValue`
- **What should exist:** entities::AuditMetadataValue
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M070 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:37`  
Modum: flattened re-export hides namespace context for `AuditSubject`; prefer `entities::AuditSubject`

- **What currently exists:** `AuditSubject` / flattened re-export hides namespace context for `AuditSubject`; prefer `entities::AuditSubject`
- **What should exist:** entities::AuditSubject
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M071 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:37`  
Modum: flattened re-export hides namespace context for `CareProfile`; prefer `entities::CareProfile`

- **What currently exists:** `CareProfile` / flattened re-export hides namespace context for `CareProfile`; prefer `entities::CareProfile`
- **What should exist:** entities::CareProfile
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M072 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:37`  
Modum: flattened re-export hides namespace context for `ReservationId`; prefer `entities::ReservationId`

- **What currently exists:** `ReservationId` / flattened re-export hides namespace context for `ReservationId`; prefer `entities::ReservationId`
- **What should exist:** entities::ReservationId
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M073 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:37`  
Modum: flattened re-export hides namespace context for `ReservationSource`; prefer `entities::ReservationSource`

- **What currently exists:** `ReservationSource` / flattened re-export hides namespace context for `ReservationSource`; prefer `entities::ReservationSource`
- **What should exist:** entities::ReservationSource
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M074 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:37`  
Modum: flattened re-export hides namespace context for `ReservationStatus`; prefer `entities::ReservationStatus`

- **What currently exists:** `ReservationStatus` / flattened re-export hides namespace context for `ReservationStatus`; prefer `entities::ReservationStatus`
- **What should exist:** entities::ReservationStatus
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M075 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:45`  
Modum: flattened re-export hides namespace context for `StaffRole`; prefer `operations::StaffRole`

- **What currently exists:** `StaffRole` / flattened re-export hides namespace context for `StaffRole`; prefer `operations::StaffRole`
- **What should exist:** operations::StaffRole
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   43|         TemperamentProfile,
   44|     };
   45|     pub use crate::operations::{
   46|         StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority,
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
```

### M076 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:45`  
Modum: flattened re-export hides namespace context for `StaffTaskAssignment`; prefer `operations::StaffTaskAssignment`

- **What currently exists:** `StaffTaskAssignment` / flattened re-export hides namespace context for `StaffTaskAssignment`; prefer `operations::StaffTaskAssignment`
- **What should exist:** operations::StaffTaskAssignment
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   43|         TemperamentProfile,
   44|     };
   45|     pub use crate::operations::{
   46|         StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority,
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
```

### M077 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:45`  
Modum: flattened re-export hides namespace context for `StaffTaskKind`; prefer `operations::StaffTaskKind`

- **What currently exists:** `StaffTaskKind` / flattened re-export hides namespace context for `StaffTaskKind`; prefer `operations::StaffTaskKind`
- **What should exist:** operations::StaffTaskKind
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   43|         TemperamentProfile,
   44|     };
   45|     pub use crate::operations::{
   46|         StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority,
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
```

### M078 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:45`  
Modum: flattened re-export hides namespace context for `StaffTaskPriority`; prefer `operations::StaffTaskPriority`

- **What currently exists:** `StaffTaskPriority` / flattened re-export hides namespace context for `StaffTaskPriority`; prefer `operations::StaffTaskPriority`
- **What should exist:** operations::StaffTaskPriority
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   43|         TemperamentProfile,
   44|     };
   45|     pub use crate::operations::{
   46|         StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority,
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
```

### M079 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:45`  
Modum: flattened re-export hides namespace context for `StaffTaskSource`; prefer `operations::StaffTaskSource`

- **What currently exists:** `StaffTaskSource` / flattened re-export hides namespace context for `StaffTaskSource`; prefer `operations::StaffTaskSource`
- **What should exist:** operations::StaffTaskSource
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   43|         TemperamentProfile,
   44|     };
   45|     pub use crate::operations::{
   46|         StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority,
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
```

### M080 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:45`  
Modum: flattened re-export hides namespace context for `StaffTaskStatus`; prefer `operations::StaffTaskStatus`

- **What currently exists:** `StaffTaskStatus` / flattened re-export hides namespace context for `StaffTaskStatus`; prefer `operations::StaffTaskStatus`
- **What should exist:** operations::StaffTaskStatus
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   43|         TemperamentProfile,
   44|     };
   45|     pub use crate::operations::{
   46|         StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority,
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
```

### M081 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:45`  
Modum: flattened re-export hides namespace context for `StaffTask`; prefer `operations::StaffTask`

- **What currently exists:** `StaffTask` / flattened re-export hides namespace context for `StaffTask`; prefer `operations::StaffTask`
- **What should exist:** operations::StaffTask
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   43|         TemperamentProfile,
   44|     };
   45|     pub use crate::operations::{
   46|         StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority,
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
```

### M082 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:49`  
Modum: flattened re-export hides namespace context for `AutomationLevel`; prefer `policy::AutomationLevel`

- **What currently exists:** `AutomationLevel` / flattened re-export hides namespace context for `AutomationLevel`; prefer `policy::AutomationLevel`
- **What should exist:** policy::AutomationLevel
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
   48|     };
   49|     pub use crate::policy::{AutomationLevel, ReviewGate};
   50|     pub use crate::tools::{
   51|         AvailabilityDecision, AvailabilityDenialReason, AvailabilityRequest, AvailabilityResult,
```

### M083 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:50`  
Modum: flattened re-export hides namespace context for `AvailabilityDecision`; prefer `tools::AvailabilityDecision`

- **What currently exists:** `AvailabilityDecision` / flattened re-export hides namespace context for `AvailabilityDecision`; prefer `tools::AvailabilityDecision`
- **What should exist:** tools::AvailabilityDecision
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   48|     };
   49|     pub use crate::policy::{AutomationLevel, ReviewGate};
   50|     pub use crate::tools::{
   51|         AvailabilityDecision, AvailabilityDenialReason, AvailabilityRequest, AvailabilityResult,
   52|         AvailabilityServiceNotes, AvailabilitySuccessReason, CapacitySnapshotId,
```

### M084 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:50`  
Modum: flattened re-export hides namespace context for `AvailabilityRequest`; prefer `tools::AvailabilityRequest`

- **What currently exists:** `AvailabilityRequest` / flattened re-export hides namespace context for `AvailabilityRequest`; prefer `tools::AvailabilityRequest`
- **What should exist:** tools::AvailabilityRequest
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   48|     };
   49|     pub use crate::policy::{AutomationLevel, ReviewGate};
   50|     pub use crate::tools::{
   51|         AvailabilityDecision, AvailabilityDenialReason, AvailabilityRequest, AvailabilityResult,
   52|         AvailabilityServiceNotes, AvailabilitySuccessReason, CapacitySnapshotId,
```

### M085 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:50`  
Modum: flattened re-export hides namespace context for `AvailabilityResult`; prefer `tools::AvailabilityResult`

- **What currently exists:** `AvailabilityResult` / flattened re-export hides namespace context for `AvailabilityResult`; prefer `tools::AvailabilityResult`
- **What should exist:** tools::AvailabilityResult
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   48|     };
   49|     pub use crate::policy::{AutomationLevel, ReviewGate};
   50|     pub use crate::tools::{
   51|         AvailabilityDecision, AvailabilityDenialReason, AvailabilityRequest, AvailabilityResult,
   52|         AvailabilityServiceNotes, AvailabilitySuccessReason, CapacitySnapshotId,
```

### M086 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `domain/src/lib.rs:50`  
Modum: flattened re-export hides namespace context for `ToolError`; prefer `tools::ToolError`

- **What currently exists:** `ToolError` / flattened re-export hides namespace context for `ToolError`; prefer `tools::ToolError`
- **What should exist:** tools::ToolError
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   48|     };
   49|     pub use crate::policy::{AutomationLevel, ReviewGate};
   50|     pub use crate::tools::{
   51|         AvailabilityDecision, AvailabilityDenialReason, AvailabilityRequest, AvailabilityResult,
   52|         AvailabilityServiceNotes, AvailabilitySuccessReason, CapacitySnapshotId,
```

### M087 — `namespace_flat_pub_use` — directionally right / prelude audit
Location: `storage/src/lib.rs:9`  
Modum: flattened re-export hides namespace context for `Error`; prefer `operations::Error`

- **What currently exists:** `Error` / flattened re-export hides namespace context for `Error`; prefer `operations::Error`
- **What should exist:** operations::Error
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
    7| pub mod operations;
    8| 
    9| pub use operations::{CodecError, Error, RecordKind, Result};
```

### M088 — `namespace_flat_pub_use_preserve_module` — directionally right / prelude audit
Location: `domain/src/lib.rs:49`  
Modum: flattened re-export hides configured namespace context for `ReviewGate`; prefer `policy::ReviewGate`

- **What currently exists:** `ReviewGate` / flattened re-export hides configured namespace context for `ReviewGate`; prefer `policy::ReviewGate`
- **What should exist:** policy::ReviewGate
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
   48|     };
   49|     pub use crate::policy::{AutomationLevel, ReviewGate};
   50|     pub use crate::tools::{
   51|         AvailabilityDecision, AvailabilityDenialReason, AvailabilityRequest, AvailabilityResult,
```

### M089 — `namespace_flat_pub_use_preserve_module` — directionally right / prelude audit
Location: `integrations/gingr/src/lib.rs:9`  
Modum: flattened re-export hides configured namespace context for `Client`; prefer `transport::Client`

- **What currently exists:** `Client` / flattened re-export hides configured namespace context for `Client`; prefer `transport::Client`
- **What should exist:** transport::Client
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
    7| 
    8| pub use config::{ApiKey, BaseUrl, ClientConfig, Provider, Subdomain};
    9| pub use transport::Client;
```

### M090 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `domain/src/lib.rs:32`  
Modum: flattened re-export keeps redundant `agent` context in `AgentName`; prefer `agent::Name`

- **What currently exists:** `agent` / flattened re-export keeps redundant `agent` context in `AgentName`; prefer `agent::Name`
- **What should exist:** agent::Name
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   30| 
   31| pub mod prelude {
   32|     pub use crate::agent::{
   33|         ForbiddenAction, Name as AgentName, OutputSchemaName, PolicyInstruction,
   34|         Purpose as AgentPurpose, Spec as AgentSpec, ToolName,
```

### M091 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `domain/src/lib.rs:32`  
Modum: flattened re-export keeps redundant `agent` context in `AgentPurpose`; prefer `agent::Purpose`

- **What currently exists:** `agent` / flattened re-export keeps redundant `agent` context in `AgentPurpose`; prefer `agent::Purpose`
- **What should exist:** agent::Purpose
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   30| 
   31| pub mod prelude {
   32|     pub use crate::agent::{
   33|         ForbiddenAction, Name as AgentName, OutputSchemaName, PolicyInstruction,
   34|         Purpose as AgentPurpose, Spec as AgentSpec, ToolName,
```

### M092 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `domain/src/lib.rs:32`  
Modum: flattened re-export keeps redundant `agent` context in `AgentSpec`; prefer `agent::Spec`

- **What currently exists:** `agent` / flattened re-export keeps redundant `agent` context in `AgentSpec`; prefer `agent::Spec`
- **What should exist:** agent::Spec
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   30| 
   31| pub mod prelude {
   32|     pub use crate::agent::{
   33|         ForbiddenAction, Name as AgentName, OutputSchemaName, PolicyInstruction,
   34|         Purpose as AgentPurpose, Spec as AgentSpec, ToolName,
```

### M093 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `domain/src/lib.rs:55`  
Modum: flattened re-export keeps redundant `workflow` context in `WorkflowEventId`; prefer `workflow::EventId`

- **What currently exists:** `workflow` / flattened re-export keeps redundant `workflow` context in `WorkflowEventId`; prefer `workflow::EventId`
- **What should exist:** workflow::EventId
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   53|         ReservationUpdateDraft, StatusSuggestionReason, ToolError,
   54|     };
   55|     pub use crate::workflow::{
   56|         AllowedAction, PolicyContext, RecommendedAction, ReviewReason, RiskFlag, Summary,
   57|         VerificationNote, WorkflowEvent, WorkflowEventId, WorkflowEventType, WorkflowResult,
```

### M094 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `domain/src/lib.rs:55`  
Modum: flattened re-export keeps redundant `workflow` context in `WorkflowEventType`; prefer `workflow::EventType`

- **What currently exists:** `workflow` / flattened re-export keeps redundant `workflow` context in `WorkflowEventType`; prefer `workflow::EventType`
- **What should exist:** workflow::EventType
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   53|         ReservationUpdateDraft, StatusSuggestionReason, ToolError,
   54|     };
   55|     pub use crate::workflow::{
   56|         AllowedAction, PolicyContext, RecommendedAction, ReviewReason, RiskFlag, Summary,
   57|         VerificationNote, WorkflowEvent, WorkflowEventId, WorkflowEventType, WorkflowResult,
```

### M095 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `domain/src/lib.rs:55`  
Modum: flattened re-export keeps redundant `workflow` context in `WorkflowEvent`; prefer `workflow::Event`

- **What currently exists:** `workflow` / flattened re-export keeps redundant `workflow` context in `WorkflowEvent`; prefer `workflow::Event`
- **What should exist:** workflow::Event
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   53|         ReservationUpdateDraft, StatusSuggestionReason, ToolError,
   54|     };
   55|     pub use crate::workflow::{
   56|         AllowedAction, PolicyContext, RecommendedAction, ReviewReason, RiskFlag, Summary,
   57|         VerificationNote, WorkflowEvent, WorkflowEventId, WorkflowEventType, WorkflowResult,
```

### M096 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `domain/src/lib.rs:55`  
Modum: flattened re-export keeps redundant `workflow` context in `WorkflowResult`; prefer `workflow::Result`

- **What currently exists:** `workflow` / flattened re-export keeps redundant `workflow` context in `WorkflowResult`; prefer `workflow::Result`
- **What should exist:** workflow::Result
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   53|         ReservationUpdateDraft, StatusSuggestionReason, ToolError,
   54|     };
   55|     pub use crate::workflow::{
   56|         AllowedAction, PolicyContext, RecommendedAction, ReviewReason, RiskFlag, Summary,
   57|         VerificationNote, WorkflowEvent, WorkflowEventId, WorkflowEventType, WorkflowResult,
```

### M097 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `domain/src/lib.rs:55`  
Modum: flattened re-export keeps redundant `workflow` context in `WorkflowStatus`; prefer `workflow::Status`

- **What currently exists:** `workflow` / flattened re-export keeps redundant `workflow` context in `WorkflowStatus`; prefer `workflow::Status`
- **What should exist:** workflow::Status
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   53|         ReservationUpdateDraft, StatusSuggestionReason, ToolError,
   54|     };
   55|     pub use crate::workflow::{
   56|         AllowedAction, PolicyContext, RecommendedAction, ReviewReason, RiskFlag, Summary,
   57|         VerificationNote, WorkflowEvent, WorkflowEventId, WorkflowEventType, WorkflowResult,
```

### M098 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `domain/src/lib.rs:55`  
Modum: flattened re-export keeps redundant `workflow` context in `WorkflowSubject`; prefer `workflow::Subject`

- **What currently exists:** `workflow` / flattened re-export keeps redundant `workflow` context in `WorkflowSubject`; prefer `workflow::Subject`
- **What should exist:** workflow::Subject
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
   53|         ReservationUpdateDraft, StatusSuggestionReason, ToolError,
   54|     };
   55|     pub use crate::workflow::{
   56|         AllowedAction, PolicyContext, RecommendedAction, ReviewReason, RiskFlag, Summary,
   57|         VerificationNote, WorkflowEvent, WorkflowEventId, WorkflowEventType, WorkflowResult,
```

### M099 — `namespace_flat_pub_use_redundant_leaf_context` — directionally right / prelude audit
Location: `integrations/gingr/src/lib.rs:8`  
Modum: flattened re-export keeps redundant `config` context in `ClientConfig`; prefer `config::Client`

- **What currently exists:** `config` / flattened re-export keeps redundant `config` context in `ClientConfig`; prefer `config::Client`
- **What should exist:** config::Client
- **Why:** A broad prelude intentionally flattens names, but this one hides too much domain context. Keep canonical module paths and narrow any prelude to ergonomics-only use.

```rust
    6| pub mod webhook;
    7| 
    8| pub use config::{ApiKey, BaseUrl, ClientConfig, Provider, Subdomain};
    9| pub use transport::Client;
```

### M100 — `namespace_flat_use` — good lint
Location: `domain/src/agents.rs:6`  
Modum: flattened import hides namespace context for `WorkflowEvent`; prefer `workflow::WorkflowEvent`

- **What currently exists:** `WorkflowEvent` / flattened import hides namespace context for `WorkflowEvent`; prefer `workflow::WorkflowEvent`
- **What should exist:** workflow::WorkflowEvent
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    4| use crate::agent;
    5| use crate::policy::ReviewGate;
    6| use crate::workflow::{WorkflowEvent, WorkflowResult};
    7| 
    8| pub use crate::agent::{OutputSchemaName, PolicyInstruction};
```

### M101 — `namespace_flat_use` — good lint
Location: `domain/src/agents.rs:6`  
Modum: flattened import hides namespace context for `WorkflowResult`; prefer `workflow::WorkflowResult`

- **What currently exists:** `WorkflowResult` / flattened import hides namespace context for `WorkflowResult`; prefer `workflow::WorkflowResult`
- **What should exist:** workflow::WorkflowResult
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    4| use crate::agent;
    5| use crate::policy::ReviewGate;
    6| use crate::workflow::{WorkflowEvent, WorkflowResult};
    7| 
    8| pub use crate::agent::{OutputSchemaName, PolicyInstruction};
```

### M102 — `namespace_flat_use` — good lint
Location: `domain/src/daily_update.rs:6`  
Modum: flattened import hides namespace context for `Error`; prefer `thiserror::Error`

- **What currently exists:** `Error` / flattened import hides namespace context for `Error`; prefer `thiserror::Error`
- **What should exist:** thiserror::Error
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    4| use serde::{Deserialize, Serialize};
    5| use std::collections::BTreeMap;
    6| use thiserror::Error;
    7| use uuid::Uuid;
    8| 
```

### M103 — `namespace_flat_use` — good lint
Location: `domain/src/operations.rs:6`  
Modum: flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`

- **What currently exists:** `ReservationId` / flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`
- **What should exist:** entities::ReservationId
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    4| use serde::{Deserialize, Deserializer, Serialize};
    5| 
    6| use crate::entities::{CustomerId, LocationId, PetId, ReservationId, ServiceKind, StaffId};
    7| use crate::workflow::task;
    8| 
```

### M104 — `namespace_flat_use` — good lint
Location: `domain/src/policy.rs:250`  
Modum: flattened import hides namespace context for `CareProfile`; prefer `entities::CareProfile`

- **What currently exists:** `CareProfile` / flattened import hides namespace context for `CareProfile`; prefer `entities::CareProfile`
- **What should exist:** entities::CareProfile
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  248| mod tests {
  249|     use super::*;
  250|     use crate::entities::{CareProfile, CustomerId, PetId, TemperamentProfile};
  251|     use crate::temperament::{BehaviorObservation, GroupPlayObservation, TemperamentRating};
  252|     use uuid::Uuid;
```

### M105 — `namespace_flat_use` — good lint
Location: `domain/src/service/boarding/mod.rs:4`  
Modum: flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`

- **What currently exists:** `ReservationId` / flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`
- **What should exist:** entities::ReservationId
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    2| use serde::{Deserialize, Deserializer, Serialize};
    3| 
    4| use crate::entities::{LocationId, PetId, ReservationId};
    5| use crate::money;
    6| 
```

### M106 — `namespace_flat_use` — good lint
Location: `domain/src/service/daycare/mod.rs:6`  
Modum: flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`

- **What currently exists:** `ReservationId` / flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`
- **What should exist:** entities::ReservationId
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    4| use serde::{Deserialize, Deserializer, Serialize};
    5| 
    6| use crate::entities::{CustomerId, PetId, ReservationId};
    7| 
    8| macro_rules! positive_scalar {
```

### M107 — `namespace_flat_use` — good lint
Location: `domain/src/tools/error.rs:1`  
Modum: flattened import hides namespace context for `Error`; prefer `thiserror::Error`

- **What currently exists:** `Error` / flattened import hides namespace context for `Error`; prefer `thiserror::Error`
- **What should exist:** thiserror::Error
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    1| use thiserror::Error;
    2| 
    3| use crate::policy;
```

### M108 — `namespace_flat_use` — good lint
Location: `domain/src/tools.rs:5`  
Modum: flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`

- **What currently exists:** `ReservationId` / flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`
- **What should exist:** entities::ReservationId
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    3| use serde::{Deserialize, Serialize};
    4| 
    5| use crate::entities::{
    6|     Customer, CustomerId, LocationId, Pet, PetId, Reservation, ReservationId, ReservationStatus,
    7| };
```

### M109 — `namespace_flat_use` — good lint
Location: `domain/src/tools.rs:5`  
Modum: flattened import hides namespace context for `ReservationStatus`; prefer `entities::ReservationStatus`

- **What currently exists:** `ReservationStatus` / flattened import hides namespace context for `ReservationStatus`; prefer `entities::ReservationStatus`
- **What should exist:** entities::ReservationStatus
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    3| use serde::{Deserialize, Serialize};
    4| 
    5| use crate::entities::{
    6|     Customer, CustomerId, LocationId, Pet, PetId, Reservation, ReservationId, ReservationStatus,
    7| };
```

### M110 — `namespace_flat_use` — good lint
Location: `domain/src/tools.rs:9`  
Modum: flattened import hides namespace context for `WorkflowEvent`; prefer `workflow::WorkflowEvent`

- **What currently exists:** `WorkflowEvent` / flattened import hides namespace context for `WorkflowEvent`; prefer `workflow::WorkflowEvent`
- **What should exist:** workflow::WorkflowEvent
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    7| };
    8| use crate::money::Money;
    9| use crate::workflow::{self, WorkflowEvent, WorkflowResult};
   10| 
   11| pub mod error;
```

### M111 — `namespace_flat_use` — good lint
Location: `domain/src/tools.rs:9`  
Modum: flattened import hides namespace context for `WorkflowResult`; prefer `workflow::WorkflowResult`

- **What currently exists:** `WorkflowResult` / flattened import hides namespace context for `WorkflowResult`; prefer `workflow::WorkflowResult`
- **What should exist:** workflow::WorkflowResult
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    7| };
    8| use crate::money::Money;
    9| use crate::workflow::{self, WorkflowEvent, WorkflowResult};
   10| 
   11| pub mod error;
```

### M112 — `namespace_flat_use` — good lint
Location: `domain/src/workflow.rs:7`  
Modum: flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`

- **What currently exists:** `ReservationId` / flattened import hides namespace context for `ReservationId`; prefer `entities::ReservationId`
- **What should exist:** entities::ReservationId
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    5| use uuid::Uuid;
    6| 
    7| use crate::entities::{ActorRef, CustomerId, LocationId, PetId, ReservationId};
    8| use crate::policy::{AutomationLevel, ReviewGate};
    9| 
```

### M113 — `namespace_flat_use` — good lint
Location: `domain/src/workflow.rs:8`  
Modum: flattened import hides namespace context for `AutomationLevel`; prefer `policy::AutomationLevel`

- **What currently exists:** `AutomationLevel` / flattened import hides namespace context for `AutomationLevel`; prefer `policy::AutomationLevel`
- **What should exist:** policy::AutomationLevel
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    6| 
    7| use crate::entities::{ActorRef, CustomerId, LocationId, PetId, ReservationId};
    8| use crate::policy::{AutomationLevel, ReviewGate};
    9| 
   10| #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
```

### M114 — `namespace_flat_use` — good lint
Location: `domain/src/workflow.rs:206`  
Modum: flattened import hides namespace context for `ReservationStatus`; prefer `entities::ReservationStatus`

- **What currently exists:** `ReservationStatus` / flattened import hides namespace context for `ReservationStatus`; prefer `entities::ReservationStatus`
- **What should exist:** entities::ReservationStatus
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  204|     use serde::{Deserialize, Serialize};
  205| 
  206|     use crate::entities::ReservationStatus;
  207| 
  208|     #[nutype(
```

### M115 — `namespace_flat_use_preserve_module` — good lint
Location: `domain/src/agent.rs:5`  
Modum: flattened import hides configured namespace context for `ReviewGate`; prefer `policy::ReviewGate`

- **What currently exists:** `ReviewGate` / flattened import hides configured namespace context for `ReviewGate`; prefer `policy::ReviewGate`
- **What should exist:** policy::ReviewGate
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    3| use serde::{Deserialize, Serialize};
    4| 
    5| use crate::policy::ReviewGate;
    6| 
    7| #[nutype(
```

### M116 — `namespace_flat_use_preserve_module` — good lint
Location: `domain/src/agents.rs:5`  
Modum: flattened import hides configured namespace context for `ReviewGate`; prefer `policy::ReviewGate`

- **What currently exists:** `ReviewGate` / flattened import hides configured namespace context for `ReviewGate`; prefer `policy::ReviewGate`
- **What should exist:** policy::ReviewGate
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    3| 
    4| use crate::agent;
    5| use crate::policy::ReviewGate;
    6| use crate::workflow::{WorkflowEvent, WorkflowResult};
    7| 
```

### M117 — `namespace_flat_use_preserve_module` — good lint
Location: `domain/src/workflow.rs:8`  
Modum: flattened import hides configured namespace context for `ReviewGate`; prefer `policy::ReviewGate`

- **What currently exists:** `ReviewGate` / flattened import hides configured namespace context for `ReviewGate`; prefer `policy::ReviewGate`
- **What should exist:** policy::ReviewGate
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    6| 
    7| use crate::entities::{ActorRef, CustomerId, LocationId, PetId, ReservationId};
    8| use crate::policy::{AutomationLevel, ReviewGate};
    9| 
   10| #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
```

### M118 — `namespace_parent_surface` — bad lint
Location: `domain/src/money/error.rs:1`  
Modum: `thiserror::Error` bypasses the canonical parent surface for `Error`; prefer `money::Error`

- **What currently exists:** `thiserror::Error` / `thiserror::Error` bypasses the canonical parent surface for `Error`; prefer `money::Error`
- **What should exist:** Leave external derive/import paths such as `thiserror::Error` alone; canonical domain surfaces apply to domain items, not proc-macro names.
- **Why:** This conflicts with the repo doctrine: module-local `error.rs` plus `Error`/`Result` re-exports are good, and derive-macro paths like `thiserror::Error` are not semantic domain call sites.

```rust
    1| use thiserror::Error;
    2| 
    3| #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
```

### M119 — `namespace_parent_surface` — bad lint
Location: `domain/src/payment/error.rs:1`  
Modum: `thiserror::Error` bypasses the canonical parent surface for `Error`; prefer `payment::Error`

- **What currently exists:** `thiserror::Error` / `thiserror::Error` bypasses the canonical parent surface for `Error`; prefer `payment::Error`
- **What should exist:** Leave external derive/import paths such as `thiserror::Error` alone; canonical domain surfaces apply to domain items, not proc-macro names.
- **Why:** This conflicts with the repo doctrine: module-local `error.rs` plus `Error`/`Result` re-exports are good, and derive-macro paths like `thiserror::Error` are not semantic domain call sites.

```rust
    1| use thiserror::Error;
    2| 
    3| #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
```

### M120 — `namespace_parent_surface` — bad lint
Location: `domain/src/reservation/error.rs:1`  
Modum: `thiserror::Error` bypasses the canonical parent surface for `Error`; prefer `reservation::Error`

- **What currently exists:** `thiserror::Error` / `thiserror::Error` bypasses the canonical parent surface for `Error`; prefer `reservation::Error`
- **What should exist:** Leave external derive/import paths such as `thiserror::Error` alone; canonical domain surfaces apply to domain items, not proc-macro names.
- **Why:** This conflicts with the repo doctrine: module-local `error.rs` plus `Error`/`Result` re-exports are good, and derive-macro paths like `thiserror::Error` are not semantic domain call sites.

```rust
    1| use thiserror::Error;
    2| 
    3| #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
```

### M121 — `api_boolean_flag_cluster` — directionally right / different solution
Location: `domain/src/daily_update.rs:53`  
Modum: public struct `DailyCareUpdateOutput` carries several boolean flags (`should_send`, `requires_review`); prefer a typed options or mode surface when those flags jointly shape behavior

- **What currently exists:** `DailyCareUpdateOutput` / public struct `DailyCareUpdateOutput` carries several boolean flags (`should_send`, `requires_review`); prefer a typed options or mode surface when those flags jointly shape behavior
- **What should exist:** A semantic output state/mode enum or decision object that names combinations like send/review, instead of separate booleans.
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
   51| }
   52| 
   53| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   54| pub struct DailyCareUpdateOutput {
   55|     pub customer_message: CustomerMessageDraft,
```

### M122 — `api_candidate_child_facet_module` — directionally right / different solution
Location: `domain/src/operations.rs:1510`  
Modum: public module `operations::grooming::history` mixes broader surface `ServiceHistoryEntry` with validated leaf `operations::grooming::history::StyleNote`; consider an owned child facet like `operations::grooming::history::style_note` so the leaf value and failure surface can live together

- **What currently exists:** `operations::grooming::history` / public module `operations::grooming::history` mixes broader surface `ServiceHistoryEntry` with validated leaf `operations::grooming::history::StyleNote`; consider an owned child facet like `operations::grooming::history::style_note` so the leaf value and failure surface can live together
- **What should exist:** operations::grooming::history::style_note
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
 1508|         use super::*;
 1509| 
 1510|         #[nutype(
 1511|             sanitize(trim),
 1512|             validate(not_empty, len_char_max = 500),
```

### M123 — `api_candidate_child_facet_module` — directionally right / different solution
Location: `domain/src/operations.rs:2428`  
Modum: public module `operations::training::package` mixes broader surface `Ledger`, `UsageDecision` with validated leaf `operations::training::package::Id`; consider an owned child facet like `operations::training::package::id` so the leaf value and failure surface can live together

- **What currently exists:** `operations::training::package` / public module `operations::training::package` mixes broader surface `Ledger`, `UsageDecision` with validated leaf `operations::training::package::Id`; consider an owned child facet like `operations::training::package::id` so the leaf value and failure surface can live together
- **What should exist:** operations::training::package::id
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
 2426|         use super::*;
 2427| 
 2428|         #[nutype(
 2429|             sanitize(trim),
 2430|             validate(not_empty, len_char_max = 120),
```

### M124 — `api_candidate_child_facet_module` — directionally right / different solution
Location: `domain/src/service/daycare/assignment.rs:4`  
Modum: public module `service::daycare::assignment` mixes broader surface `Decision`, `Request` with validated leaf `service::daycare::assignment::PlaygroupId`; consider an owned child facet like `service::daycare::assignment::playgroup_id` so the leaf value and failure surface can live together

- **What currently exists:** `service::daycare::assignment` / public module `service::daycare::assignment` mixes broader surface `Decision`, `Request` with validated leaf `service::daycare::assignment::PlaygroupId`; consider an owned child facet like `service::daycare::assignment::playgroup_id` so the leaf value and failure surface can live together
- **What should exist:** service::daycare::assignment::playgroup_id
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
    2| use crate::policy;
    3| 
    4| #[nutype(
    5|     sanitize(trim),
    6|     validate(not_empty, len_char_max = 120),
```

### M125 — `api_candidate_child_facet_module` — directionally right / different solution
Location: `domain/src/tools.rs:444`  
Modum: public module `tools::messaging` mixes broader surface `DraftMessageRequest` with validated leaf `tools::messaging::MessageBody`; consider an owned child facet like `tools::messaging::message_body` so the leaf value and failure surface can live together

- **What currently exists:** `tools::messaging` / public module `tools::messaging` mixes broader surface `DraftMessageRequest` with validated leaf `tools::messaging::MessageBody`; consider an owned child facet like `tools::messaging::message_body` so the leaf value and failure surface can live together
- **What should exist:** tools::messaging::message_body
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
  442|     }
  443| 
  444|     #[nutype(
  445|         sanitize(trim),
  446|         validate(not_empty, len_char_max = 4000),
```

### M126 — `api_candidate_child_facet_module` — directionally right / different solution
Location: `domain/src/workflow.rs:208`  
Modum: public module `workflow::status_update` mixes broader surface `ReservationStatusUpdate` with validated leaf `workflow::status_update::Reason`; consider an owned child facet like `workflow::status_update::reason` so the leaf value and failure surface can live together

- **What currently exists:** `workflow::status_update` / public module `workflow::status_update` mixes broader surface `ReservationStatusUpdate` with validated leaf `workflow::status_update::Reason`; consider an owned child facet like `workflow::status_update::reason` so the leaf value and failure surface can live together
- **What should exist:** workflow::status_update::reason
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
  206|     use crate::entities::ReservationStatus;
  207| 
  208|     #[nutype(
  209|         sanitize(trim),
  210|         validate(not_empty, len_char_max = 500),
```

### M127 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/booking_triage.rs:164`  
Modum: public siblings `RuleId`, `RuleDecision`, `RuleEvaluation` share the `Rule` head; consider a semantic `rule::{Decision, Evaluation, Id}` surface

- **What currently exists:** `RuleId` / public siblings `RuleId`, `RuleDecision`, `RuleEvaluation` share the `Rule` head; consider a semantic `rule::{Decision, Evaluation, Id}` surface
- **What should exist:** rule::{Decision, Evaluation, Id}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  162| pub struct CustomerMessageDraft(String);
  163| 
  164| #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
  165| pub enum RuleId {
  166|     DateRangeAndServiceSupported,
```

### M128 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/daily_update.rs:45`  
Modum: public siblings `DailyCareUpdateInput`, `DailyCareUpdateOutput`, `DailyCareUpdateAgent` share the `DailyCareUpdate` head; consider a semantic `daily_care_update::{Agent, Input, Output}` surface

- **What currently exists:** `DailyCareUpdateInput` / public siblings `DailyCareUpdateInput`, `DailyCareUpdateOutput`, `DailyCareUpdateAgent` share the `DailyCareUpdate` head; consider a semantic `daily_care_update::{Agent, Input, Output}` surface
- **What should exist:** daily_care_update::{Agent, Input, Output}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   43| }
   44| 
   45| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   46| pub struct DailyCareUpdateInput {
   47|     pub pet_name: pet::Name,
```

### M129 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/entities.rs:24`  
Modum: public siblings `ReservationId`, `ReservationStatus`, `ReservationSource` share the `Reservation` head; consider a semantic `reservation::{Id, Source, Status}` surface

- **What currently exists:** `ReservationId` / public siblings `ReservationId`, `ReservationStatus`, `ReservationSource` share the `Reservation` head; consider a semantic `reservation::{Id, Source, Status}` surface
- **What should exist:** reservation::{Id, Source, Status}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   22| pub struct PetId(pub Uuid);
   23| 
   24| #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
   25| pub struct ReservationId(pub Uuid);
   26| 
```

### M130 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/entities.rs:276`  
Modum: public siblings `CareNoteId`, `CareNoteSubject`, `CareNoteKind`, `CareNoteVisibility`, `CareNoteBody` share the `CareNote` head; consider a semantic `care_note::{Body, Id, Kind, Subject, Visibility}` surface

- **What currently exists:** `CareNoteId` / public siblings `CareNoteId`, `CareNoteSubject`, `CareNoteKind`, `CareNoteVisibility`, `CareNoteBody` share the `CareNote` head; consider a semantic `care_note::{Body, Id, Kind, Subject, Visibility}` surface
- **What should exist:** care_note::{Body, Id, Kind, Subject, Visibility}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  274| pub struct VaccineRecordId(pub Uuid);
  275| 
  276| #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
  277| pub struct CareNoteId(pub Uuid);
  278| 
```

### M131 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/entities.rs:285`  
Modum: public siblings `ApprovalId`, `ApprovalRecord`, `ApprovalTarget`, `ApprovalLifecycle`, `ApprovalStatus` share the `Approval` head; consider a semantic `approval::{Id, Lifecycle, Record, Status, Target}` surface

- **What currently exists:** `ApprovalId` / public siblings `ApprovalId`, `ApprovalRecord`, `ApprovalTarget`, `ApprovalLifecycle`, `ApprovalStatus` share the `Approval` head; consider a semantic `approval::{Id, Lifecycle, Record, Status, Target}` surface
- **What should exist:** approval::{Id, Lifecycle, Record, Status, Target}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  283| pub struct MessageId(pub Uuid);
  284| 
  285| #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
  286| pub struct ApprovalId(pub Uuid);
  287| 
```

### M132 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/entities.rs:580`  
Modum: public siblings `AuditEvent`, `AuditSubject`, `AuditAction`, `AuditActionLabel`, `AuditMetadataKey`, `AuditMetadataValue` share the `Audit` head; consider a semantic `audit::{Action, ActionLabel, Event, MetadataKey, MetadataValue, Subject}` surface

- **What currently exists:** `AuditEvent` / public siblings `AuditEvent`, `AuditSubject`, `AuditAction`, `AuditActionLabel`, `AuditMetadataKey`, `AuditMetadataValue` share the `Audit` head; consider a semantic `audit::{Action, ActionLabel, Event, MetadataKey, MetadataValue, Subject}` surface
- **What should exist:** audit::{Action, ActionLabel, Event, MetadataKey, MetadataValue, Subject}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  578| }
  579| 
  580| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  581| pub struct AuditEvent {
  582|     pub at: DateTime<Utc>,
```

### M133 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/lib.rs:32`  
Modum: public siblings `AgentName`, `AgentPurpose`, `AgentSpec`, `AgentPromptPacket` share the `Agent` head; consider a semantic `agent::{Name, PromptPacket, Purpose, Spec}` surface

- **What currently exists:** `AgentName` / public siblings `AgentName`, `AgentPurpose`, `AgentSpec`, `AgentPromptPacket` share the `Agent` head; consider a semantic `agent::{Name, PromptPacket, Purpose, Spec}` surface
- **What should exist:** agent::{Name, PromptPacket, Purpose, Spec}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   30| 
   31| pub mod prelude {
   32|     pub use crate::agent::{
   33|         ForbiddenAction, Name as AgentName, OutputSchemaName, PolicyInstruction,
   34|         Purpose as AgentPurpose, Spec as AgentSpec, ToolName,
```

### M134 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/lib.rs:36`  
Modum: public siblings `WorkflowAgent`, `WorkflowEvent`, `WorkflowEventId`, `WorkflowEventType`, `WorkflowResult`, `WorkflowStatus`, `WorkflowSubject` share the `Workflow` head; consider a semantic `workflow::{Agent, Event, EventId, EventType, Result, Status, Subject}` surface

- **What currently exists:** `WorkflowAgent` / public siblings `WorkflowAgent`, `WorkflowEvent`, `WorkflowEventId`, `WorkflowEventType`, `WorkflowResult`, `WorkflowStatus`, `WorkflowSubject` share the `Workflow` head; consider a semantic `workflow::{Agent, Event, EventId, EventType, Result, Status, Subject}` surface
- **What should exist:** workflow::{Agent, Event, EventId, EventType, Result, Status, Subject}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   34|         Purpose as AgentPurpose, Spec as AgentSpec, ToolName,
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
```

### M135 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/lib.rs:37`  
Modum: public siblings `AuditAction`, `AuditEvent`, `AuditMetadataKey`, `AuditMetadataValue`, `AuditSubject` share the `Audit` head; consider a semantic `audit::{Action, Event, MetadataKey, MetadataValue, Subject}` surface

- **What currently exists:** `AuditAction` / public siblings `AuditAction`, `AuditEvent`, `AuditMetadataKey`, `AuditMetadataValue`, `AuditSubject` share the `Audit` head; consider a semantic `audit::{Action, Event, MetadataKey, MetadataValue, Subject}` surface
- **What should exist:** audit::{Action, Event, MetadataKey, MetadataValue, Subject}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M136 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/lib.rs:37`  
Modum: public siblings `ReservationId`, `ReservationSource`, `ReservationStatus`, `ReservationUpdateDraft` share the `Reservation` head; consider a semantic `reservation::{Id, Source, Status, UpdateDraft}` surface

- **What currently exists:** `ReservationId` / public siblings `ReservationId`, `ReservationSource`, `ReservationStatus`, `ReservationUpdateDraft` share the `Reservation` head; consider a semantic `reservation::{Id, Source, Status, UpdateDraft}` surface
- **What should exist:** reservation::{Id, Source, Status, UpdateDraft}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M137 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/lib.rs:37`  
Modum: public siblings `StaffId`, `StaffRole`, `StaffTask`, `StaffTaskAssignment`, `StaffTaskKind`, `StaffTaskPriority`, `StaffTaskSource`, `StaffTaskStatus` share the `Staff` head; consider a semantic `staff::{Id, Role, Task, TaskAssignment, TaskKind, TaskPriority, TaskSource, TaskStatus}` surface

- **What currently exists:** `StaffId` / public siblings `StaffId`, `StaffRole`, `StaffTask`, `StaffTaskAssignment`, `StaffTaskKind`, `StaffTaskPriority`, `StaffTaskSource`, `StaffTaskStatus` share the `Staff` head; consider a semantic `staff::{Id, Role, Task, TaskAssignment, TaskKind, TaskPriority, TaskSource, TaskStatus}` surface
- **What should exist:** staff::{Id, Role, Task, TaskAssignment, TaskKind, TaskPriority, TaskSource, TaskStatus}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   35|     };
   36|     pub use crate::agents::{AgentPromptPacket, WorkflowAgent, baseline_agent_specs};
   37|     pub use crate::entities::{
   38|         ActorRef, AddOn, AuditAction, AuditEvent, AuditMetadataKey, AuditMetadataValue,
   39|         AuditSubject, Brand, CareProfile, ContactChannel, Customer, CustomerId, Deposit, HardStop,
```

### M138 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/lib.rs:50`  
Modum: public siblings `AvailabilityDecision`, `AvailabilityDenialReason`, `AvailabilityRequest`, `AvailabilityResult`, `AvailabilityServiceNotes`, `AvailabilitySuccessReason` share the `Availability` head; consider a semantic `availability::{Decision, DenialReason, Request, Result, ServiceNotes, SuccessReason}` surface

- **What currently exists:** `AvailabilityDecision` / public siblings `AvailabilityDecision`, `AvailabilityDenialReason`, `AvailabilityRequest`, `AvailabilityResult`, `AvailabilityServiceNotes`, `AvailabilitySuccessReason` share the `Availability` head; consider a semantic `availability::{Decision, DenialReason, Request, Result, ServiceNotes, SuccessReason}` surface
- **What should exist:** availability::{Decision, DenialReason, Request, Result, ServiceNotes, SuccessReason}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   48|     };
   49|     pub use crate::policy::{AutomationLevel, ReviewGate};
   50|     pub use crate::tools::{
   51|         AvailabilityDecision, AvailabilityDenialReason, AvailabilityRequest, AvailabilityResult,
   52|         AvailabilityServiceNotes, AvailabilitySuccessReason, CapacitySnapshotId,
```

### M139 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/policy.rs:57`  
Modum: public siblings `AutomationRationale`, `AutomationLevel`, `AutomationRule` share the `Automation` head; consider a semantic `automation::{Level, Rationale, Rule}` surface

- **What currently exists:** `AutomationRationale` / public siblings `AutomationRationale`, `AutomationLevel`, `AutomationRule` share the `Automation` head; consider a semantic `automation::{Level, Rationale, Rule}` surface
- **What should exist:** automation::{Level, Rationale, Rule}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   55| pub struct WorkflowName(String);
   56| 
   57| #[nutype(
   58|     sanitize(trim),
   59|     validate(not_empty, len_char_max = 400),
```

### M140 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/policy.rs:82`  
Modum: public siblings `PlayEligibilityDecision`, `PlayEligibility`, `PlayEligibilityReason`, `PlayIneligibilityReason`, `PlayEligibilityPolicy` share the `Play` head; consider a semantic `play::{Eligibility, EligibilityDecision, EligibilityPolicy, EligibilityReason, IneligibilityReason}` surface

- **What currently exists:** `PlayEligibilityDecision` / public siblings `PlayEligibilityDecision`, `PlayEligibility`, `PlayEligibilityReason`, `PlayIneligibilityReason`, `PlayEligibilityPolicy` share the `Play` head; consider a semantic `play::{Eligibility, EligibilityDecision, EligibilityPolicy, EligibilityReason, IneligibilityReason}` surface
- **What should exist:** play::{Eligibility, EligibilityDecision, EligibilityPolicy, EligibilityReason, IneligibilityReason}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   80| }
   81| 
   82| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   83| pub struct PlayEligibilityDecision {
   84|     pub eligibility: PlayEligibility,
```

### M141 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools/error.rs:7`  
Modum: public siblings `ToolError`, `ToolResource`, `ToolResourceId` share the `Tool` head; consider a semantic `tool::{Error, Resource, ResourceId}` surface

- **What currently exists:** `ToolError` / public siblings `ToolError`, `ToolResource`, `ToolResourceId` share the `Tool` head; consider a semantic `tool::{Error, Resource, ResourceId}` surface
- **What should exist:** tool::{Error, Resource, ResourceId}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
    5| pub type Result<T> = std::result::Result<T, ToolError>;
    6| 
    7| #[derive(Debug, Error, Clone, PartialEq, Eq)]
    8| pub enum ToolError {
    9|     #[error("not found: {resource} {id}")]
```

### M142 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:13`  
Modum: public siblings `ToolError`, `ToolResource`, `ToolResourceId` share the `Tool` head; consider a semantic `tool::{Error, Resource, ResourceId}` surface

- **What currently exists:** `ToolError` / public siblings `ToolError`, `ToolResource`, `ToolResourceId` share the `Tool` head; consider a semantic `tool::{Error, Resource, ResourceId}` surface
- **What should exist:** tool::{Error, Resource, ResourceId}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   11| pub mod error;
   12| 
   13| pub use error::{ExternalFailure, Result, ToolError, ToolResource, ToolResourceId};
   14| 
   15| #[async_trait]
```

### M143 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:40`  
Modum: public siblings `AvailabilityRequest`, `AvailabilityServiceNotes`, `AvailabilityResult`, `AvailabilityDecision`, `AvailabilitySuccessReason`, `AvailabilityDenialReason` share the `Availability` head; consider a semantic `availability::{Decision, DenialReason, Request, Result, ServiceNotes, SuccessReason}` surface

- **What currently exists:** `AvailabilityRequest` / public siblings `AvailabilityRequest`, `AvailabilityServiceNotes`, `AvailabilityResult`, `AvailabilityDecision`, `AvailabilitySuccessReason`, `AvailabilityDenialReason` share the `Availability` head; consider a semantic `availability::{Decision, DenialReason, Request, Result, ServiceNotes, SuccessReason}` surface
- **What should exist:** availability::{Decision, DenialReason, Request, Result, ServiceNotes, SuccessReason}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   38| }
   39| 
   40| #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   41| pub struct AvailabilityRequest {
   42|     pub location_id: LocationId,
```

### M144 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:160`  
Modum: public siblings `LookupRequest`, `LookupResult`, `LookupMatch`, `LookupCriteria` share the `Lookup` head; consider a semantic `lookup::{Criteria, Match, Request, Result}` surface

- **What currently exists:** `LookupRequest` / public siblings `LookupRequest`, `LookupResult`, `LookupMatch`, `LookupCriteria` share the `Lookup` head; consider a semantic `lookup::{Criteria, Match, Request, Result}` surface
- **What should exist:** lookup::{Criteria, Match, Request, Result}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  158|     }
  159| 
  160|     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  161|     pub struct LookupRequest {
  162|         pub provider: Provider,
```

### M145 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:242`  
Modum: public siblings `PaymentGateway`, `PaymentSubject`, `PaymentReviewReason` share the `Payment` head; consider a semantic `payment::{Gateway, ReviewReason, Subject}` surface

- **What currently exists:** `PaymentGateway` / public siblings `PaymentGateway`, `PaymentSubject`, `PaymentReviewReason` share the `Payment` head; consider a semantic `payment::{Gateway, ReviewReason, Subject}` surface
- **What should exist:** payment::{Gateway, ReviewReason, Subject}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  240|     use super::*;
  241| 
  242|     #[async_trait]
  243|     pub trait PaymentGateway: Send + Sync {
  244|         async fn authorize(&self, request: AuthorizationRequest) -> Result<AuthorizationResult>;
```

### M146 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:252`  
Modum: public siblings `AuthorizationRequest`, `AuthorizationResult`, `AuthorizationId` share the `Authorization` head; consider a semantic `authorization::{Id, Request, Result}` surface

- **What currently exists:** `AuthorizationRequest` / public siblings `AuthorizationRequest`, `AuthorizationResult`, `AuthorizationId` share the `Authorization` head; consider a semantic `authorization::{Id, Request, Result}` surface
- **What should exist:** authorization::{Id, Request, Result}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  250|     }
  251| 
  252|     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  253|     pub struct AuthorizationRequest {
  254|         pub subject: PaymentSubject,
```

### M147 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:274`  
Modum: public siblings `RefundRequest`, `RefundResult`, `RefundReason`, `RefundRejectionReason`, `RefundId` share the `Refund` head; consider a semantic `refund::{Id, Reason, RejectionReason, Request, Result}` surface

- **What currently exists:** `RefundRequest` / public siblings `RefundRequest`, `RefundResult`, `RefundReason`, `RefundRejectionReason`, `RefundId` share the `Refund` head; consider a semantic `refund::{Id, Reason, RejectionReason, Request, Result}` surface
- **What should exist:** refund::{Id, Reason, RejectionReason, Request, Result}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  272|     }
  273| 
  274|     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  275|     pub struct RefundRequest {
  276|         pub payment_reference: crate::payment::PaymentReference,
```

### M148 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:399`  
Modum: public siblings `MessageDrafting`, `MessageReviewPolicy`, `MessageBody` share the `Message` head; consider a semantic `message::{Body, Drafting, ReviewPolicy}` surface

- **What currently exists:** `MessageDrafting` / public siblings `MessageDrafting`, `MessageReviewPolicy`, `MessageBody` share the `Message` head; consider a semantic `message::{Body, Drafting, ReviewPolicy}` surface
- **What should exist:** message::{Body, Drafting, ReviewPolicy}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  397|     use super::*;
  398| 
  399|     #[async_trait]
  400|     pub trait MessageDrafting: Send + Sync {
  401|         async fn draft_message(&self, request: DraftMessageRequest) -> Result<DraftMessageResult>;
```

### M149 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:404`  
Modum: public siblings `DraftMessageRequest`, `DraftMessageResult`, `DraftMessageStatus` share the `DraftMessage` head; consider a semantic `draft_message::{Request, Result, Status}` surface

- **What currently exists:** `DraftMessageRequest` / public siblings `DraftMessageRequest`, `DraftMessageResult`, `DraftMessageStatus` share the `DraftMessage` head; consider a semantic `draft_message::{Request, Result, Status}` surface
- **What should exist:** draft_message::{Request, Result, Status}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  402|     }
  403| 
  404|     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  405|     pub struct DraftMessageRequest {
  406|         pub channel: DeliveryChannel,
```

### M150 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:465`  
Modum: public siblings `DocumentIntake`, `DocumentIntakeRequest`, `DocumentIntakeResult`, `DocumentSource`, `DocumentClassification`, `DocumentRef` share the `Document` head; consider a semantic `document::{Classification, Intake, IntakeRequest, IntakeResult, Ref, Source}` surface

- **What currently exists:** `DocumentIntake` / public siblings `DocumentIntake`, `DocumentIntakeRequest`, `DocumentIntakeResult`, `DocumentSource`, `DocumentClassification`, `DocumentRef` share the `Document` head; consider a semantic `document::{Classification, Intake, IntakeRequest, IntakeResult, Ref, Source}` surface
- **What should exist:** document::{Classification, Intake, IntakeRequest, IntakeResult, Ref, Source}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  463|     use super::*;
  464| 
  465|     #[async_trait]
  466|     pub trait DocumentIntake: Send + Sync {
  467|         async fn intake_document(
```

### M151 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:487`  
Modum: public siblings `OcrRequest`, `OcrResult`, `OcrReviewReason` share the `Ocr` head; consider a semantic `ocr::{Request, Result, ReviewReason}` surface

- **What currently exists:** `OcrRequest` / public siblings `OcrRequest`, `OcrResult`, `OcrReviewReason` share the `Ocr` head; consider a semantic `ocr::{Request, Result, ReviewReason}` surface
- **What should exist:** ocr::{Request, Result, ReviewReason}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  485|     }
  486| 
  487|     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  488|     pub struct OcrRequest {
  489|         pub document: DocumentRef,
```

### M152 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `domain/src/tools.rs:662`  
Modum: public siblings `ScheduleDraftRequest`, `ScheduleDraftResult`, `ScheduleCadence`, `ScheduleName` share the `Schedule` head; consider a semantic `schedule::{Cadence, DraftRequest, DraftResult, Name}` surface

- **What currently exists:** `ScheduleDraftRequest` / public siblings `ScheduleDraftRequest`, `ScheduleDraftResult`, `ScheduleCadence`, `ScheduleName` share the `Schedule` head; consider a semantic `schedule::{Cadence, DraftRequest, DraftResult, Name}` surface
- **What should exist:** schedule::{Cadence, DraftRequest, DraftResult, Name}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  660|     }
  661| 
  662|     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  663|     pub struct ScheduleDraftRequest {
  664|         pub name: ScheduleName,
```

### M153 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `integrations/gingr/src/endpoint/commerce_retail.rs:17`  
Modum: public siblings `GetAllRetailItems`, `GetSubscription`, `GetSubscriptions`, `GetSubscriptionsBuilder` share the `Get` head; consider a semantic `get::{AllRetailItems, Subscription, Subscriptions, SubscriptionsBuilder}` surface

- **What currently exists:** `GetAllRetailItems` / public siblings `GetAllRetailItems`, `GetSubscription`, `GetSubscriptions`, `GetSubscriptionsBuilder` share the `Get` head; consider a semantic `get::{AllRetailItems, Subscription, Subscriptions, SubscriptionsBuilder}` surface
- **What should exist:** get::{AllRetailItems, Subscription, Subscriptions, SubscriptionsBuilder}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   15| }
   16| 
   17| #[derive(Clone, Debug, Default, PartialEq, Eq)]
   18| pub struct GetAllRetailItems;
   19| 
```

### M154 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `integrations/gingr/src/endpoint/commerce_retail.rs:34`  
Modum: public siblings `ListTransactions`, `ListTransactionsBuilder`, `ListInvoices`, `ListInvoicesBuilder` share the `List` head; consider a semantic `list::{Invoices, InvoicesBuilder, Transactions, TransactionsBuilder}` surface

- **What currently exists:** `ListTransactions` / public siblings `ListTransactions`, `ListTransactionsBuilder`, `ListInvoices`, `ListInvoicesBuilder` share the `List` head; consider a semantic `list::{Invoices, InvoicesBuilder, Transactions, TransactionsBuilder}` surface
- **What should exist:** list::{Invoices, InvoicesBuilder, Transactions, TransactionsBuilder}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
   32| }
   33| 
   34| #[derive(Clone, Debug, PartialEq, Eq)]
   35| pub struct ListTransactions {
   36|     from_date: Date,
```

### M155 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `integrations/gingr/src/endpoint/owners_animals.rs:251`  
Modum: public siblings `CustomFieldName`, `CustomFieldSearch`, `CustomFieldSearchBuilder` share the `CustomField` head; consider a semantic `custom_field::{Name, Search, SearchBuilder}` surface

- **What currently exists:** `CustomFieldName` / public siblings `CustomFieldName`, `CustomFieldSearch`, `CustomFieldSearchBuilder` share the `CustomField` head; consider a semantic `custom_field::{Name, Search, SearchBuilder}` surface
- **What should exist:** custom_field::{Name, Search, SearchBuilder}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  249| }
  250| 
  251| #[derive(Clone, Debug, PartialEq, Eq)]
  252| pub struct CustomFieldName(String);
  253| 
```

### M156 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `integrations/gingr/src/endpoint/reservations.rs:3`  
Modum: public siblings `ReservationTypeId`, `ReservationTypes`, `ReservationTypesBuilder`, `ReservationWidgetData`, `ReservationWidgetDataBuilder`, `ReservationSearchFilters`, `ReservationSearchFiltersBuilder` share the `Reservation` head; consider a semantic `reservation::{SearchFilters, SearchFiltersBuilder, TypeId, Types, TypesBuilder, WidgetData, WidgetDataBuilder}` surface

- **What currently exists:** `ReservationTypeId` / public siblings `ReservationTypeId`, `ReservationTypes`, `ReservationTypesBuilder`, `ReservationWidgetData`, `ReservationWidgetDataBuilder`, `ReservationSearchFilters`, `ReservationSearchFiltersBuilder` share the `Reservation` head; consider a semantic `reservation::{SearchFilters, SearchFiltersBuilder, TypeId, Types, TypesBuilder, WidgetData, WidgetDataBuilder}` surface
- **What should exist:** reservation::{SearchFilters, SearchFiltersBuilder, TypeId, Types, TypesBuilder, WidgetData, WidgetDataBuilder}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
    1| use super::{AnimalId, Date, DateRange, IsoDate, Limit, LocationId, Method, OwnerId, Request};
    2| 
    3| #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    4| pub struct ReservationTypeId(u64);
    5| 
```

### M157 — `api_candidate_semantic_module` — directionally right / design prompt
Location: `integrations/gingr/src/endpoint/reservations.rs:310`  
Modum: public siblings `ReservationsByAnimal`, `ReservationsByAnimalBuilder`, `ReservationsByOwner`, `ReservationsByOwnerBuilder` share the `ReservationsBy` head; consider a semantic `reservations_by::{Animal, AnimalBuilder, Owner, OwnerBuilder}` surface

- **What currently exists:** `ReservationsByAnimal` / public siblings `ReservationsByAnimal`, `ReservationsByAnimalBuilder`, `ReservationsByOwner`, `ReservationsByOwnerBuilder` share the `ReservationsBy` head; consider a semantic `reservations_by::{Animal, AnimalBuilder, Owner, OwnerBuilder}` surface
- **What should exist:** reservations_by::{Animal, AnimalBuilder, Owner, OwnerBuilder}
- **Why:** The repeated prefix is real semantic pressure, but extract a module only when it becomes the canonical owner; do not mechanically nest every sibling family.

```rust
  308| }
  309| 
  310| #[derive(Clone, Debug, PartialEq, Eq)]
  311| pub struct ReservationsByAnimal {
  312|     animal_id: AnimalId,
```

### M158 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `domain/src/care.rs:69`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
   67| pub struct ReviewReason(String);
   68| 
   69| macro_rules! redacted_debug {
   70|     ($type:ident, $label:literal) => {
   71|         impl fmt::Debug for $type {
```

### M159 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `domain/src/operations.rs:1088`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
 1086| }
 1087| 
 1088| macro_rules! positive_scalar {
 1089|     ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
 1090|         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
```

### M160 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `domain/src/operations.rs:1137`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
 1135| pub mod grooming {
 1136|     use super::*;
 1137|     positive_scalar!(
 1138|         AppointmentMinutes,
 1139|         u16,
```

### M161 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `domain/src/operations.rs:1781`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
 1779|     use crate::policy;
 1780| 
 1781|     positive_scalar!(
 1782|         DurationWeeks,
 1783|         u8,
```

### M162 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `domain/src/operations.rs:2693`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
 2691|     pub type Result<T> = std::result::Result<T, Error>;
 2692| 
 2693|     positive_scalar!(
 2694|         UnitCount,
 2695|         u32,
```

### M163 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `domain/src/service/boarding/mod.rs:7`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
    5| use crate::money;
    6| 
    7| macro_rules! positive_scalar {
    8|     ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
    9|         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
```

### M164 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `domain/src/service/daycare/front_desk.rs:4`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
    2| use crate::policy;
    3| 
    4| positive_scalar!(
    5|     QueuePosition,
    6|     u16,
```

### M165 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `domain/src/service/daycare/mod.rs:8`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
    6| use crate::entities::{CustomerId, PetId, ReservationId};
    7| 
    8| macro_rules! positive_scalar {
    9|     ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
   10|         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
```

### M166 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `integrations/gingr/src/endpoint/mod.rs:5`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
    3| pub mod labor_ops;
    4| pub mod owners_animals;
    5| pub mod reference_data;
    6| pub mod report_cards_files;
    7| pub mod reservations;
```

### M167 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `integrations/gingr/src/endpoint/reference_data.rs:20`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
   18| }
   19| 
   20| macro_rules! simple_reference_endpoint {
   21|     ($name:ident, $path:literal) => {
   22|         #[derive(Clone, Debug, Default, PartialEq, Eq)]
```

### M168 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `integrations/gingr/src/lib.rs:2`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
    1| pub mod config;
    2| pub mod endpoint;
    3| pub mod mapping;
    4| pub mod response;
```

### M169 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `storage/src/lib.rs:7`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
    5| //! between storage records and core domain types.
    6| 
    7| pub mod operations;
    8| 
    9| pub use operations::{CodecError, Error, RecordKind, Result};
```

### M170 — `api_candidate_semantic_module_unsupported_construct` — directionally right / manual review
Location: `storage/src/operations.rs:988`  
Modum: skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion

- **What currently exists:** `item macro` / skipped semantic module family inference in this scope because source-level analysis saw `item macro`, `macro_rules!`; `api_candidate_semantic_module` only runs on direct parsed items without cfg pruning or macro expansion
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
  986| }
  987| 
  988| macro_rules! bidirectional_code_map {
  989|     ($storage:ty, $domain:ty, { $($storage_variant:ident => $domain_variant:ident),+ $(,)? }) => {
  990|         impl From<$storage> for $domain {
```

### M171 — `api_integer_protocol_parameter` — good lint
Location: `integrations/gingr/src/response.rs:11`  
Modum: public boundary `Raw::new` uses protocol parameter(s) `status` as raw integers; prefer typed enums or newtypes for boundary-facing protocol concepts

- **What currently exists:** `Raw::new` / public boundary `Raw::new` uses protocol parameter(s) `status` as raw integers; prefer typed enums or newtypes for boundary-facing protocol concepts
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
    9| 
   10| impl Raw {
   11|     pub fn new(status: u16, body: impl Into<Bytes>) -> Self {
   12|         Self {
   13|             status,
```

### M172 — `api_integer_protocol_parameter` — good lint
Location: `integrations/gingr/src/response.rs:18`  
Modum: public boundary `Raw::status` returns a protocol concept as a raw integer; prefer a typed enum or newtype

- **What currently exists:** `Raw::status` / public boundary `Raw::status` returns a protocol concept as a raw integer; prefer a typed enum or newtype
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
   16|     }
   17| 
   18|     pub fn status(&self) -> u16 {
   19|         self.status
   20|     }
```

### M173 — `api_integer_protocol_parameter` — good lint
Location: `integrations/gingr/src/webhook.rs:342`  
Modum: public boundary `WebhookAck::retryable_status` uses protocol parameter(s) `status` as raw integers; prefer typed enums or newtypes for boundary-facing protocol concepts

- **What currently exists:** `WebhookAck::retryable_status` / public boundary `WebhookAck::retryable_status` uses protocol parameter(s) `status` as raw integers; prefer typed enums or newtypes for boundary-facing protocol concepts
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  340| 
  341| impl WebhookAck {
  342|     pub fn retryable_status(status: u16) -> Self {
  343|         if status == 200 || status == 403 || !(100..=599).contains(&status) {
  344|             Self::RetryableFailure
```

### M174 — `api_integer_protocol_parameter` — good lint
Location: `integrations/gingr/src/webhook.rs:350`  
Modum: public boundary `WebhookAck::http_status` returns a protocol concept as a raw integer; prefer a typed enum or newtype

- **What currently exists:** `WebhookAck::http_status` / public boundary `WebhookAck::http_status` returns a protocol concept as a raw integer; prefer a typed enum or newtype
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  348|     }
  349| 
  350|     pub fn http_status(&self) -> u16 {
  351|         match self {
  352|             Self::Processed => 200,
```

### M175 — `api_manual_enum_string_helper` — directionally right / different solution
Location: `integrations/gingr/src/mapping/mod.rs:14`  
Modum: public enum `ProviderField` has a manual `Display` impl that only maps variants to string literals; prefer `strum::Display` when that string surface is canonical

- **What currently exists:** `ProviderField` / public enum `ProviderField` has a manual `Display` impl that only maps variants to string literals; prefer `strum::Display` when that string surface is canonical
- **What should exist:** strum::Display
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
   12| }
   13| 
   14| impl fmt::Display for ProviderField {
   15|     fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
   16|         match self {
```

### M176 — `api_raw_id_surface` — good lint
Location: `integrations/gingr/src/endpoint/owners_animals.rs:208`  
Modum: public boundary `FormKind::form_id` returns a raw id value; prefer a typed id newtype at the boundary

- **What currently exists:** `FormKind::form_id` / public boundary `FormKind::form_id` returns a raw id value; prefer a typed id newtype at the boundary
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  206| 
  207| impl FormKind {
  208|     pub const fn form_id(self) -> u64 {
  209|         match self {
  210|             Self::Owner => 1,
```

### M177 — `api_raw_id_surface` — good lint
Location: `integrations/gingr/src/response.rs:45`  
Modum: public struct `OwnerRecord` keeps raw id field(s) `id` as strings or primitive integers; prefer id newtypes at the boundary

- **What currently exists:** `OwnerRecord` / public struct `OwnerRecord` keeps raw id field(s) `id` as strings or primitive integers; prefer id newtypes at the boundary
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
   43| }
   44| 
   45| #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
   46| pub struct OwnerRecord {
   47|     pub id: u64,
```

### M178 — `api_raw_id_surface` — good lint
Location: `integrations/gingr/src/response.rs:77`  
Modum: public struct `AnimalRecord` keeps raw id field(s) `id` as strings or primitive integers; prefer id newtypes at the boundary

- **What currently exists:** `AnimalRecord` / public struct `AnimalRecord` keeps raw id field(s) `id` as strings or primitive integers; prefer id newtypes at the boundary
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
   75| }
   76| 
   77| #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
   78| pub struct AnimalRecord {
   79|     pub id: u64,
```

### M179 — `api_raw_id_surface` — good lint
Location: `integrations/gingr/src/response.rs:92`  
Modum: public struct `ReservationRecord` keeps raw id field(s) `id` as strings or primitive integers; prefer id newtypes at the boundary

- **What currently exists:** `ReservationRecord` / public struct `ReservationRecord` keeps raw id field(s) `id` as strings or primitive integers; prefer id newtypes at the boundary
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
   90| }
   91| 
   92| #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
   93| pub struct ReservationRecord {
   94|     pub id: u64,
```

### M180 — `api_raw_id_surface` — good lint
Location: `integrations/gingr/src/response.rs:105`  
Modum: public struct `ReferenceRecord` keeps raw id field(s) `id` as strings or primitive integers; prefer id newtypes at the boundary

- **What currently exists:** `ReferenceRecord` / public struct `ReferenceRecord` keeps raw id field(s) `id` as strings or primitive integers; prefer id newtypes at the boundary
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  103| }
  104| 
  105| #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
  106| pub struct ReferenceRecord {
  107|     pub id: u64,
```

### M181 — `api_semantic_string_scalar` — good lint
Location: `integrations/gingr/src/response.rs:45`  
Modum: public struct `OwnerRecord` carries semantic scalar field(s) `email` as raw strings; prefer typed boundary values or focused newtypes

- **What currently exists:** `OwnerRecord` / public struct `OwnerRecord` carries semantic scalar field(s) `email` as raw strings; prefer typed boundary values or focused newtypes
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
   43| }
   44| 
   45| #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
   46| pub struct OwnerRecord {
   47|     pub id: u64,
```

### M182 — `api_semantic_string_scalar` — good lint
Location: `integrations/gingr/src/transport.rs:51`  
Modum: public boundary `RequestParts::path` returns a semantic scalar as a raw string; prefer a typed boundary value or focused newtype

- **What currently exists:** `RequestParts::path` / public boundary `RequestParts::path` returns a semantic scalar as a raw string; prefer a typed boundary value or focused newtype
- **What should exist:** A canonical semantic module/call-site shape chosen after ownership review.
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
   49|     }
   50| 
   51|     pub fn path(&self) -> &str {
   52|         &self.path
   53|     }
```

### M183 — `api_string_error_surface` — directionally right / different solution
Location: `integrations/gingr/src/response.rs:27`  
Modum: public struct `Envelope` stores boundary error text as raw string field(s) `error`; prefer a typed error surface with named variants or focused error data

- **What currently exists:** `Envelope` / public struct `Envelope` stores boundary error text as raw string field(s) `error`; prefer a typed error surface with named variants or focused error data
- **What should exist:** A typed provider error surface with variants/data for the known envelope failure modes, while preserving raw text only as optional provider detail.
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
   25| }
   26| 
   27| #[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
   28| pub struct Envelope<T> {
   29|     pub success: Option<bool>,
```

### M184 — `api_stringly_protocol_parameter` — directionally right / different solution
Location: `integrations/gingr/src/config.rs:87`  
Modum: public boundary `BaseUrl::join_path` takes stringly protocol or state parameter(s) `path`; prefer typed enums or newtypes at the boundary

- **What currently exists:** `BaseUrl::join_path` / public boundary `BaseUrl::join_path` takes stringly protocol or state parameter(s) `path`; prefer typed enums or newtypes at the boundary
- **What should exist:** A typed endpoint/path/request target value at the boundary; `BaseUrl` should join a path type, not arbitrary text.
- **Why:** The smell is real, but the linter suggestion should be treated as a prompt rather than an automatic rewrite; choose the owner/path that preserves semantic context at call sites.

```rust
   85|     }
   86| 
   87|     pub(crate) fn join_path(&self, path: &str) -> core::result::Result<Url, url::ParseError> {
   88|         self.0.join(path.trim_start_matches('/'))
   89|     }
```

### M185 — `namespace_family_unsupported_construct` — directionally right / manual review
Location: `domain/src/lib.rs:45`  
Modum: skipped namespace-family inference for `TaskCompletionEvidence` in this re-export because source-level analysis saw `macro_rules!`; verify the owning family manually before changing the visible path

- **What currently exists:** `TaskCompletionEvidence` / skipped namespace-family inference for `TaskCompletionEvidence` in this re-export because source-level analysis saw `macro_rules!`; verify the owning family manually before changing the visible path
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
   43|         TemperamentProfile,
   44|     };
   45|     pub use crate::operations::{
   46|         StaffRole, StaffTask, StaffTaskAssignment, StaffTaskKind, StaffTaskPriority,
   47|         StaffTaskSource, StaffTaskStatus, TaskCompletionEvidence,
```

### M186 — `namespace_family_unsupported_construct` — directionally right / manual review
Location: `storage/src/lib.rs:9`  
Modum: skipped namespace-family inference for `CodecError`, `RecordKind` in this re-export because source-level analysis saw `item macro`, `macro_rules!`; verify the owning family manually before changing the visible path

- **What currently exists:** `CodecError` / skipped namespace-family inference for `CodecError`, `RecordKind` in this re-export because source-level analysis saw `item macro`, `macro_rules!`; verify the owning family manually before changing the visible path
- **What should exist:** No immediate code shape implied. First inspect the macro-expanded/cfg-resolved public surface, then decide whether a semantic module family exists.
- **Why:** This is not a fix request; it marks a source-analysis boundary caused by macros/cfg/includes. Inspect the real expanded/public surface before making structural changes.

```rust
    7| pub mod operations;
    8| 
    9| pub use operations::{CodecError, Error, RecordKind, Result};
```

### M187 — `namespace_overqualified_callsite_path` — good lint
Location: `domain/src/daily_update.rs:324`  
Modum: `crate::agents::baseline_agent_specs` keeps too much module scaffolding at the call site; call through existing `agents` namespace and prefer `agents::baseline_agent_specs`

- **What currently exists:** `crate::agents::baseline_agent_specs` / `crate::agents::baseline_agent_specs` keeps too much module scaffolding at the call site; call through existing `agents` namespace and prefer `agents::baseline_agent_specs`
- **What should exist:** agents::baseline_agent_specs
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  322| impl agents::WorkflowAgent<DailyCareUpdateInput, DailyCareUpdateOutput> for DailyCareUpdateAgent {
  323|     fn spec(&self) -> agents::AgentSpec {
  324|         crate::agents::baseline_agent_specs()
  325|             .into_iter()
  326|             .find(|spec| spec.name.clone().into_inner() == "daily-care-update")
```

### M188 — `namespace_overqualified_callsite_path` — good lint
Location: `storage/src/operations.rs:875`  
Modum: `domain::service::boarding::Contract` keeps too much module scaffolding at the call site; import `domain::service::boarding` and prefer `boarding::Contract`

- **What currently exists:** `domain::service::boarding::Contract` / `domain::service::boarding::Contract` keeps too much module scaffolding at the call site; import `domain::service::boarding` and prefer `boarding::Contract`
- **What should exist:** boarding::Contract
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  873| pub struct CoreServiceContractsRecord {
  874|     pub location_id: domain::entities::LocationId,
  875|     pub boarding: domain::service::boarding::Contract,
  876|     pub daycare: domain::service::daycare::Contract,
  877|     pub grooming: domain::operations::grooming::Contract,
```

### M189 — `namespace_overqualified_callsite_path` — good lint
Location: `storage/src/operations.rs:876`  
Modum: `domain::service::daycare::Contract` keeps too much module scaffolding at the call site; import `domain::service::daycare` and prefer `daycare::Contract`

- **What currently exists:** `domain::service::daycare::Contract` / `domain::service::daycare::Contract` keeps too much module scaffolding at the call site; import `domain::service::daycare` and prefer `daycare::Contract`
- **What should exist:** daycare::Contract
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  874|     pub location_id: domain::entities::LocationId,
  875|     pub boarding: domain::service::boarding::Contract,
  876|     pub daycare: domain::service::daycare::Contract,
  877|     pub grooming: domain::operations::grooming::Contract,
  878|     pub training: domain::operations::training::Contract,
```

### M190 — `namespace_overqualified_callsite_path` — good lint
Location: `storage/src/operations.rs:877`  
Modum: `domain::operations::grooming::Contract` keeps too much module scaffolding at the call site; import `domain::operations::grooming` and prefer `grooming::Contract`

- **What currently exists:** `domain::operations::grooming::Contract` / `domain::operations::grooming::Contract` keeps too much module scaffolding at the call site; import `domain::operations::grooming` and prefer `grooming::Contract`
- **What should exist:** grooming::Contract
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  875|     pub boarding: domain::service::boarding::Contract,
  876|     pub daycare: domain::service::daycare::Contract,
  877|     pub grooming: domain::operations::grooming::Contract,
  878|     pub training: domain::operations::training::Contract,
  879|     pub retail: domain::operations::retail::Contract,
```

### M191 — `namespace_overqualified_callsite_path` — good lint
Location: `storage/src/operations.rs:878`  
Modum: `domain::operations::training::Contract` keeps too much module scaffolding at the call site; import `domain::operations::training` and prefer `training::Contract`

- **What currently exists:** `domain::operations::training::Contract` / `domain::operations::training::Contract` keeps too much module scaffolding at the call site; import `domain::operations::training` and prefer `training::Contract`
- **What should exist:** training::Contract
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  876|     pub daycare: domain::service::daycare::Contract,
  877|     pub grooming: domain::operations::grooming::Contract,
  878|     pub training: domain::operations::training::Contract,
  879|     pub retail: domain::operations::retail::Contract,
  880| }
```

### M192 — `namespace_overqualified_callsite_path` — good lint
Location: `storage/src/operations.rs:879`  
Modum: `domain::operations::retail::Contract` keeps too much module scaffolding at the call site; import `domain::operations::retail` and prefer `retail::Contract`

- **What currently exists:** `domain::operations::retail::Contract` / `domain::operations::retail::Contract` keeps too much module scaffolding at the call site; import `domain::operations::retail` and prefer `retail::Contract`
- **What should exist:** retail::Contract
- **Why:** This matches the semantic-module doctrine: preserve enough namespace context without over- or under-qualifying the call site.

```rust
  877|     pub grooming: domain::operations::grooming::Contract,
  878|     pub training: domain::operations::training::Contract,
  879|     pub retail: domain::operations::retail::Contract,
  880| }
  881| 
```
