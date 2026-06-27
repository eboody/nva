# HTTP runtime responsibility cutline after SpacetimeDB

Status: architecture cutline for the owned Pet Resorts operations API after SpacetimeDB owns live command/subscription state. This page is intentionally a planning artifact; it does not remove current demo routes, change the OpenAPI artifact, publish a SpacetimeDB module, enable live provider/customer side effects, or claim production readiness.

Source context: [API HTTP shell](../../apps/api/src/http.rs), [API public contract DTOs](../../apps/api/src/public_contract.rs), [API README](../../apps/api/README.md), [owned operations OpenAPI contract artifact](../../apps/api/openapi/owned-operations-v0.openapi.json), [owned API storage/read-model cutline](owned-api-storage-read-model-cutline.md), [owned operations API contract](owned-operations-api-contract.md), [runtime contract boundaries](runtime-contract-boundaries.md), and the completed upstream SpacetimeDB/Postgres architecture handoffs recorded on the Kanban parent tasks.

## Cutline thesis

After SpacetimeDB owns realtime operational state, the HTTP crate should stop acting like the live operations runtime and become the edge/runtime shell for request/response responsibilities.

The authority split is:

```text
external systems and users
  - provider webhooks, staff/admin HTTP clients, file upload/download clients,
    presentation/demo clients, health probes, export/reporting readers
        |
        v
apps/api HTTP shell
  - authenticate/authorize HTTP entry, verify webhook signatures, issue upload/download
    tickets, serialize stable OpenAPI-compatible DTOs, mediate exports/reports,
    report readiness, and route compatibility traffic
        |
        v
app/domain/storage/integration ports and SpacetimeDB adapters
  - app/domain own policy; integrations own provider DTOs; Postgres/S3 own durable
    archive/reporting/evidence; SpacetimeDB owns live commands, queue state,
    reducer transactions, and subscription read models
        |
        v
staff/manager realtime clients
  - subscribe to SpacetimeDB public read models rather than polling broad HTTP queues
```

In one sentence: HTTP remains the stable ingress/egress contract and compatibility edge; SpacetimeDB becomes the live operations runtime for staff/manager command and subscription state.

## What HTTP keeps

HTTP keeps responsibilities that are naturally request/response, externally addressable, or compatibility/reporting oriented.

### Provider webhook ingress

HTTP is still the right first hop for provider/PMS webhooks because webhooks are HTTP deliveries and need an edge contract before any internal runtime sees them.

HTTP may own:

- route binding, TLS/front-door integration, body size limits, and safe request logging;
- `WebhookSignatureVerifier`-style verification for provider signatures and timestamp/replay checks;
- normalization into app/integration-owned event envelopes or source evidence refs;
- idempotency-key extraction and handoff to the app/storage/realtime command boundary;
- safe rejection envelopes for bad signatures, stale timestamps, unsupported provider events, or disabled live modes.

HTTP must not turn provider webhook payloads into canonical operations DTOs. Provider DTOs remain integration/source evidence and should be promoted through app/domain/storage boundaries before affecting live state.

### Upload/download mediation

HTTP is still the natural surface for browser/mobile upload and download flows, but it should mediate object access rather than owning document business state.

HTTP may own:

- upload/download ticket issuance and validation;
- MIME/size/header checks, request correlation, and safe error envelopes;
- handoff to object storage/S3/MinIO and durable evidence metadata;
- callback/complete endpoints that tell app/storage/realtime boundaries that an object is available.

HTTP may define an `UploadTicketIssuer` trait for presigned URL or upload-session issuance. That trait should only cover the HTTP/object-access concern: ticket subject, object key/bucket/ref, expiry, allowed MIME/size, and correlation. It should not become a vaccine-document, eligibility, or review-policy port.

### Admin, export, and reporting

HTTP keeps admin/read-only surfaces that are not live queue coordination:

- admin inspection pages or JSON for support/operator diagnostics;
- export bundle generation/streaming for audit/reporting/BI handoff;
- report endpoints over durable Postgres/S3 read models;
- low-frequency compatibility views for external presentation or BI consumers.

HTTP may define an `ExportBundleReader` trait for reading already-authorized durable export bundles or report artifacts. It should not own the business logic that decides what a workflow outcome means; it only reads/streams durable artifacts with audit/correlation metadata.

### Health and readiness

HTTP keeps `/healthz`, `/readyz`, and request-observability surfaces because they describe the edge process and its dependencies.

HTTP may define an `ApiReadinessProbe` trait for dependency posture such as:

- HTTP process status;
- SpacetimeDB client connectivity/subscription posture;
- Postgres/S3 archive/reporting availability;
- webhook verification configuration;
- upload/download ticket backend availability;
- live-side-effect disabled/enabled posture.

Readiness should distinguish `configured`, `connected`, `degraded`, and intentionally unavailable dependencies where possible. It should not imply that live operations are HTTP-owned.

### Compatibility and OpenAPI

HTTP keeps the stable OpenAPI contract for request/response clients, presentation demos, and integration consumers that cannot speak SpacetimeDB subscriptions.

The existing `apps/api/openapi/owned-operations-v0.openapi.json` is safe as a demo/compatibility contract artifact because it explicitly says provider DTOs are evidence only and `live_side_effects_allowed` is false. The migration path below narrows the meaning of its live-looking routes without breaking the presentation demo.

## What HTTP should not own once SpacetimeDB owns realtime operations

### Broad review queue polling

HTTP should not be the broad polling surface for staff/manager review queues once SpacetimeDB public read models exist.

Current HTTP examples that become compatibility/demo surfaces instead of canonical live surfaces:

- `GET /staff/inquiries` over in-memory state;
- `GET /v0/agent/context/data-quality-hygiene` when used as a live queue/context feed;
- `GET /v0/read-models/source-quality-backlog` if treated as operator queue state rather than a durable reporting/read-model endpoint.

The canonical live queue path should be SpacetimeDB public subscription rows such as `StaffQueueItemRow`, `ManagerQueueItemRow`, `BlockedActionNoticeRow`, and `HygieneOutcomeCardRow` from `apps/spacetimedb/src/read_model`. HTTP can keep low-frequency reporting snapshots, but staff dashboards should subscribe to SpacetimeDB for live queue changes.

### Custom WebSocket fanout

HTTP should not grow a parallel Axum WebSocket hub for the same staff/manager queue state that SpacetimeDB already publishes.

If a browser needs live queue updates, prefer generated SpacetimeDB clients or a narrow gateway that speaks SpacetimeDB subscriptions. Do not invent a second queue fanout protocol in `apps/api` unless a specific compatibility client cannot use SpacetimeDB; if that exception appears, document it as a compatibility adapter with no mutation authority.

### Duplicated command endpoints for live staff operations

HTTP should not duplicate live reducer commands for staff operations unless a compatibility route is explicitly required.

Examples of command-like HTTP routes that should move behind SpacetimeDB reducers for live operations:

- staff/manager claim, assign, resolve, approve, reject, disposition, and outcome capture for realtime review queues;
- agent draft submission when the draft becomes live staff-visible queue state;
- blocked side-effect recording for live operational workflows.

Compatibility HTTP routes may remain only if they are clearly labeled and implemented as adapters into the same authoritative reducer/app path. They must not write a separate in-memory/Postgres/HTTP-owned copy of queue state.

## HTTP-specific traits it may define

HTTP-specific traits are allowed when they model edge concerns. They should be small, named after the edge responsibility, and should call app/domain/storage/integration ports rather than duplicate them.

Recommended HTTP-local traits:

| Trait | Responsibility | Must not own |
| --- | --- | --- |
| `UploadTicketIssuer` | Issue/validate upload/download tickets or presigned object-storage access for HTTP clients. | Document/vaccine eligibility policy, review decisions, durable workflow state. |
| `WebhookSignatureVerifier` | Verify provider webhook signatures, timestamps, replay windows, and safe failure classes. | Provider DTO mapping, source import semantics, workflow policy. |
| `ApiReadinessProbe` | Report API edge dependency posture: SpacetimeDB, Postgres/S3, webhook config, object access, live-side-effect posture. | Health of business workflows as if HTTP owns them. |
| `ExportBundleReader` | Locate and stream already-authorized export/report bundles from durable storage. | BI semantics, workflow outcome meaning, provider payload parsing. |

Avoid app-business-port duplicates such as `ReviewQueueRepository`, `DataQualityHygieneCommandService`, `ManagerDailyBriefWorkflow`, `ProviderMutationPort`, or `StaffQueueFanout` inside `apps/api`. Those concepts belong in app/domain/storage/integration or SpacetimeDB adapter crates.

## Current route classification

| Current route family | Current role | Post-SpacetimeDB classification |
| --- | --- | --- |
| `/healthz`, `/v0/healthz`, `/readyz`, `/v0/readyz` | Edge health/readiness with disabled live side effects. | Keep in HTTP; expand readiness to mention SpacetimeDB/Postgres/S3 posture honestly. |
| `/ops/metrics/summary`, `/v0/ops/metrics/summary` | Local aggregate demo metrics and labor rollups. | Keep as aggregate/admin/reporting; do not make it a live queue API. |
| `/inquiries`, `/staff/inquiries` | In-memory inquiry intake and staff queue demo surface. | Keep only as demo/compatibility until promoted; live inquiry queue should become reducer/subscription state if it remains a realtime workflow. |
| `/agent/context/manager-daily-brief`, `/agent/drafts/manager-daily-brief`, `/manager-daily-brief/actions/{action_id}/outcome` | HTTP demo loop for context, draft validation, and outcome capture. | Move live command/queue parts to SpacetimeDB reducers/read models; keep HTTP compatibility/reporting routes if the presentation demo still needs request/response calls. |
| `/v0/agent/context/data-quality-hygiene`, `/v0/agent/drafts/data-quality-hygiene`, `/v0/data-quality-hygiene/actions/{action_id}/outcome` | Data-Quality Hygiene context/draft/outcome demo with review gates and blocked side effects. | SpacetimeDB owns live review queue commands/subscriptions; HTTP keeps compatibility adapter or export/report endpoints. |
| `/v0/data-quality-hygiene/outcomes/summary` | Reviewed local outcome rollup. | Keep as reporting/admin HTTP over durable archive/read model, not live queue truth. |
| `/v0/read-models/source-quality-backlog` | Postgres/synthetic backlog read-model endpoint. | Keep as BI/reporting/read-model HTTP surface; do not use as broad staff queue polling when SpacetimeDB subscriptions are available. |
| `/vaccine-documents/uploads` | Local upload + document/review demo surface. | Keep upload mediation; move medical document/review state to domain/storage/realtime boundaries before live use. |
| `/vaccine-documents/review-packets/{id}/approve|reject` | Local review decision demo surface. | Do not keep as duplicated live review commands; either adapt to authoritative reducer/app path or retire to demo-only compatibility. |

## Migration path for existing demo OpenAPI claims

Do not break the presentation demo by deleting routes first. Narrow the claims in stages.

1. Preserve the OpenAPI artifact as the `owned_operations_api_v0` compatibility/demo contract while live side effects remain disabled.
   - Keep `provider_payload_passthrough: false` and `live_side_effects_allowed: false`.
   - Keep current route tests as presentation-demo drift guards.
2. Add explicit language to route descriptions and README prose that command-like Data-Quality Hygiene and manager-brief routes are local/demo compatibility until wired through SpacetimeDB reducers.
   - `operationId`s can stay stable for demo clients.
   - Descriptions should say whether the route is `edge_compatibility`, `reporting_read_model`, or `live_reducer_adapter`.
3. Introduce SpacetimeDB reducer/read-model names as the canonical live contract for queue operations.
   - Queue context/listing shifts from HTTP polling to public subscription rows.
   - Live mutation shifts from HTTP handler-local state to reducer entrypoints that call app/domain policy.
4. Convert compatible HTTP commands into thin reducer adapters only if required by existing clients.
   - The HTTP handler verifies/authenticates/deserializes, calls the reducer/app adapter, and serializes the result.
   - The handler must not persist an independent queue copy.
5. Split OpenAPI tags or vendor extensions before claiming production readiness.
   - Suggested tags: `edge-health`, `provider-webhooks`, `uploads`, `reporting`, `compatibility-demo`, `spacetimedb-reducer-adapter`.
   - Routes still backed by in-memory state should stay tagged `compatibility-demo`.
6. Retire demo-only routes after the presentation/demo replacement path exists.
   - Replacement path can be a SpacetimeDB demo client, a small compatibility adapter, or a documented curl-to-reducer bridge.
   - Until then, route removal is unnecessary risk.

## Replacement path for the current presentation demo

The current demo can keep using HTTP safely if the story is honest:

- Health/readiness shows the HTTP shell and disabled live side effects.
- Data-Quality Hygiene HTTP routes show DTO/review-gate shape and compatibility behavior.
- SpacetimeDB demo should show live queue authority: staff/manager public subscription rows, blocked-action notices without sensitive payloads, and outcome cards after reducer transitions.
- Reporting/read-model HTTP endpoints show durable or synthetic read-model snapshots, not live staff coordination.

The demo script should say: "HTTP proves edge contracts and compatibility; SpacetimeDB proves live operations." It should not say the HTTP Data-Quality Hygiene routes are the future live queue API once SpacetimeDB is in place.

## Implementation guidance

No code change is required by this cutline yet. The low-risk next implementation card should be documentation/OpenAPI narrowing plus optional route tagging, not deletion.

Recommended follow-up implementation slice:

1. Update `apps/api/README.md` route descriptions to label each route as keep, compatibility/demo, reporting, or future reducer adapter.
2. Add OpenAPI tags/descriptions for `compatibility-demo`, `reporting`, and `edge-health` without changing schemas or paths.
3. If a reducer adapter exists, add one narrow HTTP compatibility route that calls the authoritative SpacetimeDB/app path and proves it does not write API-local queue state.
4. Keep existing API tests green; add a drift test that fails if a route description claims live queue authority while backed by in-memory state.

## Verification checklist for future changes

Before modifying `apps/api/src/http.rs` after this cutline, verify:

- [ ] The change is an HTTP edge concern, reporting/export concern, upload/download concern, provider webhook concern, health/readiness concern, or explicit compatibility adapter.
- [ ] Live queue commands/subscriptions remain authoritative in SpacetimeDB, not HTTP-local state.
- [ ] HTTP-specific traits are edge traits (`UploadTicketIssuer`, `WebhookSignatureVerifier`, `ApiReadinessProbe`, `ExportBundleReader`) rather than app-business ports.
- [ ] Provider DTOs remain integration/source evidence and are not exposed as canonical API resources.
- [ ] Postgres/S3 remain durable archive/reporting/evidence surfaces; SpacetimeDB remains live operations state.
- [ ] Existing presentation demo routes are either preserved or have a documented replacement path.
- [ ] OpenAPI descriptions match the actual backing authority and do not overclaim production/live behavior.
