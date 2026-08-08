# Owned API OpenAPI and DTO contract plan

Status: concrete v0 schema plan for a publishable owned Pet Resorts operations API contract. This is a planning artifact for downstream code cards; it does not add schema-generation dependencies, expose provider DTOs as public API contracts, use live NVA/Gingr credentials, perform provider/PMS writes, send customer/member messages, move payments/refunds/discounts, change schedules/capacity, make medical/safety decisions, or deploy production paths.

Source context: [owned operations API contract families](owned-operations-api-contract.md), [owned API observability/audit/metrics contract](owned-api-observability-metrics-contract.md), [runtime contract boundaries](runtime-contract-boundaries.md), [DTO/API/DB/observability readiness gap map](../audits/dto-api-db-observability-readiness-gap-map.md), [Pet Resort MVP stack](../roadmap/pet-resort-mvp-stack.md), [Data Quality Hygiene workflow](../workflows/operator/data-quality-hygiene.md), [API HTTP shell](../../apps/api/src/http.rs), [current API tests](../../apps/api/tests/), [foundation migration](../../migrations/0001_mvp_foundation.sql), and [storage operations projections](../../storage/src/operations.rs).

## Recommendation

Use a manually checked OpenAPI/JSON-schema artifact first, then adopt a Rust schema-generation crate after the v0 owned DTO module and route contract have stabilized.

Immediate recommendation for the first code card:

1. Add product-owned public API DTOs in `apps/api/src/public_contract.rs` or `apps/api/src/http/public_contract.rs` and re-export them from `apps/api/src/lib.rs`.
2. Add a checked static OpenAPI document at `apps/api/openapi/owned-operations-v0.openapi.json` or `docs/api/owned-operations-v0.openapi.json` that covers only safe v0 routes.
3. Add a tiny validation test that parses the OpenAPI JSON, checks route/schema names, verifies the version and side-effect-disallowed posture, and prevents drift in the Data-Quality Hygiene slice.
4. Defer `utoipa`, `aide`, or `paperclip` until the API has public DTO structs with stable derives and enough routes to justify generation maintenance.

Reasoning:

- The workspace currently has Axum, Tower, Serde, Chrono, UUID, Tracing, and related API/runtime dependencies, but no existing OpenAPI/schema crate. A grep over workspace manifests found no `utoipa`, `aide`, `paperclip`, `schemars`, `okapi`, or OpenAPI dependency.
- Current API DTOs live as private Rust structs in `apps/api/src/http.rs`, with deterministic in-memory state. Publishing a schema directly from those private handler structs would freeze local/demo names and internal shapes too early.
- The strategic contract says provider DTOs remain source evidence. A manual artifact forces the first public contract to be named around NVA operations, review gates, outcomes, audit, and BI/read-model needs rather than copying Gingr/provider payloads or local private handler names.
- Manual first does not mean untyped forever. The first code slice should isolate stable owned DTOs so `utoipa` or `aide` can later derive/generate from those DTOs without rewriting every handler.

Generation crate posture for later:

| Option | Adopt now? | Fit later | Notes |
| --- | --- | --- | --- |
| Manual checked OpenAPI JSON/YAML | Yes | Good bootstrap | Lowest dependency risk; makes route and schema names explicit; pair with parse/check tests and docs link checks. |
| `utoipa` | Defer | Strong candidate after DTO extraction | Common Rust OpenAPI derive flow; useful once public DTO structs are stable. Avoid deriving from provider DTOs or broad storage rows. |
| `aide` | Defer | Possible if route-integrated Axum generation becomes valuable | Good Axum-oriented design, but adds framework coupling before the boundary is settled. |
| `paperclip` | No for now | Unlikely default | Older ecosystem fit and less aligned with current Axum 0.8 workspace than manual/utoipa/aide. |
| `schemars` only | Defer | Useful companion if JSON Schema per DTO is needed | Does not by itself define the OpenAPI route contract; still needs route metadata and error/version conventions. |

## Versioning and path policy

Publish v0 as an explicit contract version without pretending it is production-stable:

- Base path: `/v0` for the publishable contract, even if the current local routes remain unversioned during transition.
- OpenAPI metadata: `info.version = "0.1.0"`, `info.title = "NVA Pet Resorts Owned Operations API"`.
- Schema namespace: prefix public DTO names with stable product concepts rather than handler names, for example `DataQualityHygieneContextResponse`, not `local_data_quality_context_payload`.
- Compatibility rule: within v0, additive response fields are allowed only when optional or nullable; enum additions require release notes because staff UI and BI clients may branch on them.
- Deprecation rule: never remove a field or change enum semantics in-place; add a new field/schema version and mark the old field deprecated in the OpenAPI description.
- Provider rule: provider/source ids appear only as `SourceRef`, `ExternalRecordRef`, or `ProviderEvidenceRef` evidence fields. Public resources are not named after Gingr endpoints or raw provider DTOs.

## Public DTO boundary

The public API DTO module should be a boundary layer over app/domain/storage evidence, not a direct serialization of private handler structs, raw domain internals, database rows, or provider DTOs.

Recommended module shape for the first implementation card:

```text
apps/api/src/
  lib.rs
  http.rs
  public_contract.rs        # product-owned public DTOs and error envelope
apps/api/openapi/
  owned-operations-v0.openapi.json
apps/api/tests/
  owned_api_openapi_contract.rs
```

Alternative if the repo wants API docs under docs:

```text
docs/api/owned-operations-v0.openapi.json
apps/api/tests/owned_api_openapi_contract.rs
```

Use explicit conversion functions from current app/storage/domain values into public DTOs. Do not put `Serialize`-only public schema derives directly on provider adapter DTOs or broad SQL records.

## Cross-cutting schemas

These schemas should be shared by every v0 route family.

| Schema | Required fields | Notes |
| --- | --- | --- |
| `ApiContractMetadata` | `owner`, `boundary`, `version`, `provider_payload_passthrough`, `live_side_effects_allowed` | Values should make the owned/non-provider boundary visible in responses. |
| `RequestMetadata` | `request_id`, `correlation_id`, `payload_logging`, optional `actor`, optional `location_id`, optional `tenant_id` | Align with the observability contract. `request_id` is echoed from/generated for `x-request-id`; `correlation_id` is the workflow/business spine. |
| `ActorRef` | `actor_kind`, optional `actor_id`, optional `actor_role` | Current shell may use payload/staff ids; future auth should supply it. No secrets or raw emails required. |
| `SourceRef` | `source_system`, `external_record_ref`, optional `observed_at`, optional `adapter_version`, optional `source_visibility` | Evidence only; never canonical owned entity identity. |
| `ReviewGateRef` | `gate`, `required`, optional `reviewer_role`, optional `reason` | Mirrors domain review gates without exposing raw policy internals. |
| `BlockedAction` | `action`, `blocked_reason`, optional `review_gate` | Use exact unsafe action family: customer send, provider write, payment move, schedule/capacity mutation, medical/safety decision, raw payload export. |
| `AuditRef` | `audit_event_id`, `event_name`, optional `workflow_event_id`, optional `review_packet_id`, optional `approval_record_id`, optional `outbox_record_id` | Safe refs only; no raw payloads. |
| `Pagination` | `limit`, `cursor`, `next_cursor`, `has_more` | Cursor-based for lists. Offset can be omitted in v0. |
| `FilterMetadata` | `applied_filters`, `projection_version`, `generated_at`, optional `freshness`, optional `caveats` | Required on BI/read-model responses. |
| `ErrorEnvelope` | `error.code`, `error.message`, `error.safe_error_class`, `request_id`, optional `correlation_id`, optional `details`, `live_side_effects_allowed=false` | Stable low-cardinality `safe_error_class`; message safe for logs/UI; details must contain refs/field paths, not raw provider/customer/payment/document payloads. |

### Error envelope policy

All non-2xx responses in v0 should use `ErrorEnvelope`:

```json
{
  "error": {
    "code": "validation_failed",
    "message": "The draft requested a blocked live side effect.",
    "safe_error_class": "validation_failed",
    "details": [
      {
        "field": "actions[0].requested_side_effects",
        "reason": "customer_send_requires_review_and_live_sends_are_disabled"
      }
    ]
  },
  "request_id": "req_...",
  "correlation_id": "corr_...",
  "live_side_effects_allowed": false
}
```

Initial status-code mapping:

| Status | Error `code` | `safe_error_class` | Use |
| --- | --- | --- | --- |
| 400 | `invalid_request` | `validation_failed` | Malformed JSON, missing required fields, invalid query values. |
| 401 | `authentication_required` | `auth_required` | Reserved future auth error only; do not implement broad auth in the schema card. |
| 403 | `forbidden` | `authorization_failed` | Future role/location/tenant scope failures. |
| 404 | `not_found` | `not_found` | Unknown action, packet, issue, or read-model cursor. |
| 409 | `idempotency_conflict` | `idempotency_conflict` | Duplicate idempotency key with different payload or stale workflow state. |
| 422 | `workflow_validation_failed` | `validation_failed` | Draft/action violates review gate, blocked action, or source evidence invariants. |
| 503 | `dependency_unavailable` | `dependency_unavailable` | Future durable DB/object storage/worker dependency unavailable. |

## Filtering, pagination, and sorting conventions

Use boring list conventions now so the staff UI and BI/export clients do not invent per-route semantics.

- `limit`: optional integer, default 50, max 200.
- `cursor`: opaque string returned by the previous page.
- `sort`: optional low-cardinality enum per route, e.g. `priority_desc`, `oldest_first`, `generated_at_desc`.
- Date filters use ISO strings: `operating_day=YYYY-MM-DD`, `from=YYYY-MM-DD`, `to=YYYY-MM-DD`.
- IDs and refs stay strings in the public schema, even if Rust uses UUID/newtype wrappers internally.
- List responses include `items`, `pagination`, and `filters`.
- Read-model responses include `projection_name`, `projection_version`, `generated_at`, `freshness` or `source_cutoff_at`, `caveats`, and lineage refs.
- No list response should return raw provider payloads, raw document text, payment payloads, secrets, hidden prompts, signed URLs, or broad customer/member PII.

## Auth reserved seam

Do not block the schema contract on full auth/session implementation. Do include the reserved auth seam so future widening does not break the contract:

- OpenAPI security scheme: `StaffSessionCookie` with `type: apiKey`, `in: cookie`, `name: nva_staff_session`.
- Route descriptions should say current local runtime may run unauthenticated for deterministic tests, while production/staging must supply actor/location/role context.
- `ActorRef`, `location_id`, and future `tenant_id` are part of request/response metadata now; current local routes may set them from payload/query or leave future-auth fields nullable.
- Auth never authorizes live provider writes, customer sends, payment moves, schedule/capacity changes, medical/safety decisions, or production deployment by itself. Those still require workflow-specific review/outbox gates.

## v0 route plan

The first published schema should include Data-Quality Hygiene as the runnable vertical slice and name adjacent future surfaces without requiring them all to be implemented at once.

### Slice A: Data-Quality Hygiene, runnable first

| Method/path | Public operation id | Request schema | Response schema | Implement now? | Notes |
| --- | --- | --- | --- | --- | --- |
| `GET /v0/agent/context/data-quality-hygiene` | `getDataQualityHygieneContext` | Query: `location_id`, `operating_day` | `DataQualityHygieneContextResponse` | Yes | Current unversioned route exists. Response should contain ranked candidates, source refs, issue refs, actions, labor estimate, allowed actions, blocked actions, and audit/request metadata. |
| `POST /v0/agent/drafts/data-quality-hygiene` | `submitDataQualityHygieneDraft` | `DataQualityHygieneDraftSubmissionRequest` | `DataQualityHygieneDraftSubmissionResponse` or `ErrorEnvelope` | Yes | Validate context packet, correlation id, action ids, source refs, issue refs, review gates, requested side effects, and ambiguity-hiding flag. |
| `POST /v0/data-quality-hygiene/actions/{action_id}/outcome` | `captureDataQualityHygieneOutcome` | `DataQualityHygieneOutcomeCaptureRequest` | `DataQualityHygieneOutcomeCaptureResponse` or `ErrorEnvelope` | Yes | Records reviewed outcome/labor evidence only. Does not prove provider repair. |
| `GET /v0/data-quality-hygiene/outcomes/summary` | `getDataQualityHygieneOutcomeSummary` | Query: `location_id`, `operating_day`, optional `correlation_id` | `DataQualityHygieneOutcomeSummaryResponse` | Yes | Current unversioned route exists. Add filter/projection metadata and caveats. |
| `GET /v0/read-models/source-quality-backlog` | `listSourceQualityBacklog` | Query: `location_id`, optional `operating_day`, `severity`, `workflow_blocking`, `owner_role`, `limit`, `cursor` | `SourceQualityBacklogListResponse` | Name now, implement after durable source-quality table/read model | BI-facing target for replacing raw Gingr/source scraping. Current local proof may document it as reserved until durable wiring exists. |

Minimum Data-Quality Hygiene schemas:

| Schema | Fields |
| --- | --- |
| `DataQualityHygieneContextResponse` | `metadata: RequestMetadata`, `api_contract: ApiContractMetadata`, `workflow`, `location_id`, `operating_day`, `prepared_for`, `candidates`, `hygiene_actions`, `labor_savings_estimate`, `allowed_agent_actions`, `blocked_actions`, `audit`, `live_side_effects_allowed=false` |
| `DataQualityCandidate` | `candidate_id`, `affected_entity_ref`, `issue_category`, `issue_kind`, `field_path`, `severity`, `freshness`, `sensitivity`, `workflow_blocking`, `source_refs`, `issue_refs`, `review_gates`, `redaction_policy` |
| `DataQualityHygieneAction` | `action_id`, `kind`, `priority`, `owner_persona`, `reviewer_role`, `source_refs`, `issue_refs`, `review_gates`, `cleanup_action`, `estimated_minutes`, `safe_agent_actions`, `blocked_actions` |
| `DataQualityHygieneDraftSubmissionRequest` | `context_packet_id`, `correlation_id`, `actions`, optional `idempotency_key` |
| `DataQualityHygieneSubmittedAction` | `action_id`, `kind`, `source_refs`, `issue_refs`, `review_gates`, `requested_side_effects`, `attempted_ambiguity_resolution` |
| `DataQualityHygieneDraftSubmissionResponse` | `metadata`, `validation`, `accepted_actions`, `rejected_actions`, optional `workflow_event_id`, optional `review_packet_id`, optional `outbox_candidate`, `audit_refs`, `live_side_effects_allowed=false` |
| `DraftValidationResult` | `status`, `safe_error_class`, `reasons`, `blocked_actions`, `review_required` |
| `DataQualityHygieneOutcomeCaptureRequest` | `outcome`, `actual_minutes`, `actor`, `feedback`, `source_refs`, `issue_refs`, `resolution_status_after_review`, `timestamp`, `audit`, `requested_side_effects`, optional `idempotency_key` |
| `DataQualityHygieneOutcomeCaptureResponse` | `metadata`, `outcome_record`, `audit_refs`, `summary_ref`, `live_side_effects_allowed=false` |
| `DataQualityHygieneOutcomeRecord` | `outcome_record_id`, `action_id`, `outcome`, `actual_minutes`, `estimated_minutes_saved`, `actual_minutes_saved`, `actor`, `feedback_summary`, `source_refs`, `issue_refs`, `resolution_status_after_review`, `location_id`, `operating_day`, `correlation_id`, `recorded_at` |
| `DataQualityHygieneOutcomeSummaryResponse` | `metadata`, `filters`, `summary`, `source_refs`, `issue_refs`, `caveats`, `live_side_effects_allowed=false` |

Data-Quality Hygiene invariants for tests:

- Context response includes `live_side_effects_allowed=false`, source refs, issue refs, review gates, labor estimate, and blocked actions including provider/PMS mutation and ambiguity hiding.
- Draft submission rejects `send_customer_message`, provider/PMS mutation, and ambiguity hiding with `422 ErrorEnvelope` or a typed rejected draft response carrying `safe_error_class="validation_failed"`.
- Outcome capture accepts reviewed labor evidence when no blocked side effect is requested and returns request/correlation/audit refs.
- Outcome summary returns aggregate/review evidence and caveats, not raw provider rows.

### Slice B: runtime/readiness and metrics, safe now

| Method/path | Public operation id | Response schema | Implement now? | Notes |
| --- | --- | --- | --- | --- |
| `GET /v0/healthz` | `getHealth` | `HealthResponse` | Yes | Current route exists unversioned. Schema must state side effects disabled. |
| `GET /v0/readyz` | `getReadiness` | `ReadinessResponse` | Yes | Include `workflow_repository`, `observability`, disabled customer/provider posture, and production gaps. |
| `GET /v0/ops/metrics/summary` | `getOpsMetricsSummary` | `OpsMetricsSummaryResponse` | Yes | Aggregate-only local runtime/business metrics; not a raw row export or production observability claim. |

Schemas to publish: `HealthResponse`, `ReadinessResponse`, `WorkflowRepositoryReadiness`, `ObservabilityReadiness`, `OpsMetricsSummaryResponse`, `ProductLaborMetrics`, `LocalRuntimeCounters`, `MetricsSafety`.

### Slice C: future Manager Daily Brief and review queue surfaces

Name these routes in the plan/OpenAPI roadmap, but do not force the first implementation to wire all of them.

| Method/path | Public operation id | Schema names | First implementation posture |
| --- | --- | --- | --- |
| `GET /v0/agent/context/manager-daily-brief` | `getManagerDailyBriefContext` | `ManagerDailyBriefContextResponse` | Current unversioned local route exists; add after Data-Quality Hygiene schemas if time permits. |
| `POST /v0/agent/drafts/manager-daily-brief` | `submitManagerDailyBriefDraft` | `ManagerDailyBriefDraftSubmissionRequest`, `ManagerDailyBriefDraftSubmissionResponse` | Future same validation envelope and blocked side-effect policy. |
| `POST /v0/manager-daily-brief/actions/{action_id}/outcome` | `captureManagerDailyBriefOutcome` | `ManagerDailyBriefOutcomeCaptureRequest`, `ManagerDailyBriefOutcomeCaptureResponse` | Future outcome/labor route. |
| `GET /v0/review-queues` | `listReviewQueues` | `ReviewQueueListResponse` | Future durable queue/read model. |
| `GET /v0/review-packets/{review_packet_id}` | `getReviewPacket` | `ReviewPacketResponse` | Future shared review packet detail. |
| `POST /v0/review-packets/{review_packet_id}/decision` | `recordReviewPacketDecision` | `ReviewDecisionRequest`, `ReviewDecisionResponse` | Future approval/rejection path, no live effects by itself. |
| `GET /v0/read-models/review-queue-aging` | `listReviewQueueAging` | `ReviewQueueAgingListResponse` | Future BI/operator read model. |
| `GET /v0/read-models/labor-outcomes` | `listLaborOutcomes` | `LaborOutcomeListResponse` | Future BI/operator read model across workflows. |

### Slice D: future source/import, audit, outbox, and BI read models

Route names to reserve without overbuilding v0:

| Method/path | Public operation id | Schema names | Notes |
| --- | --- | --- | --- |
| `POST /v0/operations/source-imports` | `createSourceImport` | `SourceImportRequest`, `SourceImportResponse` | Future approved import/snapshot path; not live provider writes. |
| `GET /v0/audit-events` | `listAuditEvents` | `AuditEventListResponse` | Safe metadata and redacted summaries only. |
| `GET /v0/correlations/{correlation_id}` | `getCorrelationLineage` | `CorrelationLineageResponse` | Request -> workflow -> review -> approval -> outcome/outbox -> audit. |
| `GET /v0/outbox-records` | `listOutboxRecords` | `OutboxRecordListResponse` | Shows approved/stubbed posture; does not execute live adapters. |
| `GET /v0/read-models/outbox-posture` | `listOutboxPosture` | `OutboxPostureListResponse` | BI/operator queue/outbox posture. |
| `GET /v0/read-models/audit-lineage` | `listAuditLineage` | `AuditLineageListResponse` | BI audit lookup with safe refs. |
| `GET /v0/read-models/occupancy-service-demand` | `listOccupancyServiceDemand` | `OccupancyServiceDemandListResponse` | Future normalized demand read model with source caveats. |

## Observability alignment

The OpenAPI contract should encode the observability contract rather than leaving it as a logging-only concern:

- Every response schema includes `metadata.request_id` and, for workflow/read-model routes, `metadata.correlation_id`.
- Every command response names generated or touched `workflow_event_id`, `review_packet_id`, `approval_record_id`, `outbox_record_id`, and `audit_event_id` when applicable.
- Every error uses `safe_error_class` and the same class appears in route logs/audit failure metadata.
- `payload_logging` is always `disabled` or `redacted_summary_only`; no route describes raw customer/provider/payment/document payload logging.
- Business read models are separate from infrastructure metrics: `GET /v0/ops/metrics/summary` is aggregate operational proof, while `/v0/read-models/...` routes carry BI projection metadata and caveats.

## Downstream code-card acceptance checklist

A downstream code worker should be able to implement the contract without broad API design questions if it follows this checklist.

Files to create/change:

- Create `docs/architecture/owned-api-openapi-plan.md` from this plan if not already present.
- Create `apps/api/src/public_contract.rs` with the shared DTOs and Data-Quality Hygiene v0 DTOs listed above.
- Change `apps/api/src/lib.rs` to expose the public contract module.
- Change `apps/api/src/http.rs` only enough to convert existing handler responses into public DTOs or mount `/v0` aliases for the safe routes; do not rewrite persistence in the schema card.
- Create either `apps/api/openapi/owned-operations-v0.openapi.json` or `docs/api/owned-operations-v0.openapi.json`.
- Create `apps/api/tests/owned_api_openapi_contract.rs` validating the static OpenAPI artifact and the Data-Quality Hygiene contract shape.
- Optionally add docs navigation/link references after the schema artifact exists.

Commands to run:

```sh
cargo fmt --check
cargo test -p pet-resort-api --test owned_api_openapi_contract
cargo test -p pet-resort-api --test data_quality_hygiene_agent_contract
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
```

Do not add `utoipa`, `aide`, `paperclip`, `schemars`, or another schema crate in the first contract pass unless the code card also commits to public DTO extraction, generation tests, and maintenance of the generated artifact. If a crate is adopted later, prefer `utoipa` or `aide` after the manual artifact proves the owned contract names.

## Non-goals and safety reminders

- No raw Gingr/provider DTOs as public schemas.
- No raw provider/customer/payment/document payload examples in OpenAPI examples.
- No production data, credentials, signed URLs, or real customer/member examples.
- No live sends, provider/PMS writes, payment/refund/discount movement, schedule/capacity mutation, medical/safety decisions, or production deployment paths.
- No claim that the current in-memory API shell is durable production persistence.
- No broad auth implementation in the schema card; include only the reserved auth seam and actor/location fields so future auth can attach safely.
