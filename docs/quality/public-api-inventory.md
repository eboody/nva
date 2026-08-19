# Public API inventory

This inventory records the intentional Rust public surfaces at the clean-slate quality boundary. It distinguishes canonical concept ownership from boundary/runtime exposure. The complete symbol, constructor, method, associated-constant, and enum-variant inventory generated from current Rust documentation is `docs/quality/generated-public-api-inventory.md`; generate it with `scripts/generate_public_api_inventory.py` after the locked workspace documentation build. The executable sources of truth for reviewed high-risk external contracts and re-export exceptions are `docs/quality/public-api-inventory.json` and `docs/quality/architecture-quality-baseline.json`; `scripts/check_architecture_quality.py` rejects unrecorded high-risk declarations, rationale-free entries, and stale entries after removal.

## Crate-root ownership

| Crate | Intentional root surface | Ownership rule |
| --- | --- | --- |
| `domain` | Semantic modules from `access` through `workflow` | Canonical callers name the concept-owning module; the historical `strategic_ai_ops` facade is retired. |
| `app` | Workflow modules, tool ports, repository ports, and the small `prelude` | App owns orchestration packets and ports, not domain truth or runtime authority. The prelude exposes only agent-spec and tool-catalog essentials. |
| `storage` | `operations`, `persistence`, `service_line`, `workflow_repository`; `CodecError` and module-local `Result` | Storage exposes records, codecs, and adapters at the storage boundary. |
| `gingr` | `config`, `dto`, `endpoint`, `mapping`, `response`, `transport`, `webhook`; validated config values | Provider DTOs remain quarantined. Root config re-exports are the supported construction seam, not domain values. |
| `nva-api` | `http`, `error`, `observability`, `public_contract` | Public JSON/OpenAPI DTOs and safe error conversion are runtime contracts. Authentication internals remain private. |
| `nva-spacetimedb` | Adapter/runtime/storage/read-model/table modules and reducer ABI exports | Rows are storage or subscription contracts, never domain authority. Reducer root exports are retained for SpacetimeDB ABI registration. |
| `nva-worker` | `runtime` | Worker shell exposes only its runtime composition seam. |
| `nva-cli` | Binary only | No library API. |

## Re-export inventory

| Owner | Retained re-export | Rationale |
| --- | --- | --- |
| `integrations/gingr/src/lib.rs` | `config::{ApiKey, BaseUrl, Provider, Subdomain}` | Canonical validated Gingr configuration seam. |
| `integrations/gingr/src/endpoint/mod.rs` | `error::Error` | Canonical endpoint error seam with current cutover-date vocabulary. |
| `integrations/gingr/src/endpoint/mod.rs` | `reservations::Reservations` | Canonical endpoint family handle under `gingr::endpoint`. |
| `app/src/lib.rs::prelude` | Agent prompt/spec traits and `tools::{availability, draft_update}` | Deliberately small shell integration prelude; it does not flatten workflow/domain types. |
| `app/src/agents.rs` | `domain::agent::{OutputSchemaName, PolicyInstruction}` | App agent specifications use these domain-owned policy values directly; the redundant `AgentSpec` alias is retired in favor of `domain::agent::Spec`. |
| `app/src/booking_triage.rs` | Request typestate stages | The workflow module owns the guided construction protocol and keeps phase markers together. |
| `app/src/checkout_completion.rs` | `StaffTaskDraft`, `PaymentException`, `ReportedDisposition`, `SourceException` | App-owned workflow role names for the checkout packet; exact aliases are checked and cannot grow silently. |
| `app/src/tools.rs` | `ExternalToolCandidate` | Current executable policy classifies external tool candidates without exposing unused speculative tool ports. |
| `storage/src/lib.rs` | `operations::{CodecError, Result}` | Canonical crate-level storage codec/error seam. |
| `storage/src/operations.rs` | Service-line record families | Existing storage aggregation surface pending Task 8 decomposition; it is concept-bound, not a glob. |
| `apps/spacetimedb/src/storage/review_queue/mod.rs` | Row and status-column families | Canonical private adapter storage boundary. |
| `apps/spacetimedb/src/tables.rs` | Review-queue rows/accessors | Canonical SpacetimeDB table registration surface. |
| `apps/spacetimedb/src/read_model.rs` | Manager/staff/blocked subscription rows | Canonical current subscription surface. |
| `apps/spacetimedb/src/lib.rs` | `reducers::*` | Retained broad export for SpacetimeDB reducer ABI registration; exact declaration is executable-policy allowlisted. |
| `domain/src/consent.rs` | `message::Channel` | Consent owns channel permission semantics at this boundary. |
| `domain/src/reservation/mod.rs`, `domain/src/payment/mod.rs` | Module-local `Error` and `Result` | Canonical error seams for the owning semantic modules. |
| `domain/src/retail/mod.rs` | Product vocabulary and `vendor::Partner` | Retail is the canonical aggregate boundary for its product/vendor concepts. |
| `domain/src/daily_brief.rs` | `snapshot::Id as Snapshot` | Canonical daily-brief snapshot identity; exact alias is checked. |
| `domain/src/daycare/assignment.rs` | `playgroup_id::Id as PlaygroupId` | Canonical public builder vocabulary; exact alias is checked. |
| `domain/src/grooming/mod.rs` | Appointment `Request` and duration-estimate `Policy` grouped aliases | Canonical short leaves only under the semantic `grooming::appointment` and `grooming::duration_estimate` paths. |

Module-local `Result<T>` aliases are intentional error ownership, not convenience flattening. Boundary aliases such as SpacetimeDB `IssueRefColumn`/`RecommendationColumn` preserve explicit storage-column meaning. The unused `domain::entities::{Deposit, PaymentStatus}`, `app::agents::AgentSpec`, and operations-level service-line aliases are retired; callers use `domain::payment` and concept-owned storage paths.

## Structural authority denials

- `app::crm_retention::FollowUpEligibility` is an opaque, non-serializable value issued only by the workflow. Packets expose one canonical serialize-only evidence projection and deliberately implement no deserialization path, so serialized claims cannot rehydrate eligibility, review, queue, task, or draft authority.
- The unissuable public eligible variant and future eligibility reason are removed. Current packets structurally contain only an ineligibility reason, manager review gate, safe evidence actions, and blocked live actions.
- Constant-false/constant-true and duplicate denial getters were removed: `can_drive_marketing_draft`, `marketable_opportunities`, `records_staff_evidence_only`, outcome-level `blocked_actions`, and `matches_reported_packet_evidence`.
- Architecture and behavioral fixtures prove callers cannot deserialize retention packets into authority, use removed stale getters, or recover removed convenience aliases.

## Hygiene outcome card decision

The clean-slate SpacetimeDB schema exposes one canonical `HygieneOutcomeCardRow`. Its `reported_actual_minutes_spent` field is evidence-only, the row has one canonical read-model path, and the compile contract rejects parallel historical/current table surfaces.

## Executable guards

1. `scripts/check_architecture_quality.py` scans production Rust after removing comments and literals.
2. Every broad `pub use ...::*` and renamed `pub use ... as ...` must match an exact checked declaration with a non-empty rationale.
3. A new declaration fails closed. Removing or changing an approved declaration also fails until the inventory is ratcheted.
4. SpacetimeDB trybuild fixtures compile the canonical schema path.
5. Existing authority compile-fail suites continue to prove opaque, non-serializable, non-reusable issuance boundaries; this inventory does not authorize constructors or side effects.
6. `scripts/generate_public_api_inventory.py` reads the exact generated Rust documentation and records every exported item and associated surface together with concrete non-test workspace references.

## Successor boundaries

- Remaining re-export aliases are retained only where the semantic owner path and inventory rationale establish the canonical caller vocabulary.
- SpacetimeDB table changes must preserve the one canonical current schema and its compile contracts.
