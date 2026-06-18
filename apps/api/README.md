# `apps/api`

`apps/api` is the HTTP runtime shell for the pet-resort workspace. It boots an Axum server in [`src/main.rs`](./src/main.rs), exposes the crate surface from [`src/lib.rs`](./src/lib.rs), and keeps the current route/context/draft-validation/outcome-capture wiring in [`src/http.rs`](./src/http.rs).

This shell should stay thin. It may bind sockets, build routers, deserialize HTTP request bodies, serialize response payloads, hold temporary local-dev state, and call typed `app`, `domain`, or `storage` contracts. It should not become the place where pet-resort policy, provider DTOs, persistence schemas, or live side effects are invented.

## Entry points and route surface

Start at [`src/main.rs`](./src/main.rs). The binary reads `PET_RESORT_API_ADDR`, defaults to `127.0.0.1:3001`, initializes JSON tracing, binds a Tokio listener, and serves [`http::router`](./src/http.rs). [`src/lib.rs`](./src/lib.rs) is intentionally small: it documents the API shell role and exposes [`pet_resort_api::http`](./src/http.rs).

The current router is built by [`router_with_state`](./src/http.rs) and uses [`VaccineDocumentState`](./src/http.rs) as local in-memory state. The routes in [`src/http.rs`](./src/http.rs) are:

- `GET /healthz` via `healthz`: reports `pet-resort-api`, `ok`, and `live_side_effects: disabled`. See [`tests/health_contract.rs`](./tests/health_contract.rs).
- `GET /readyz` via `readyz`: makes stubbed dependencies explicit (`database`, `object_storage`, fake deterministic `agent_runtime`, and disabled live customer/provider writes). See [`tests/health_contract.rs`](./tests/health_contract.rs).
- `POST /inquiries` via `submit_inquiry`: normalizes an inquiry into a review-gated [`InquiryIntakeRecord`](./src/http.rs) with a draft reply, staff-review task, fake deterministic agent marker, and audit events.
- `GET /staff/inquiries` via `staff_inquiries`: returns the in-memory staff queue of [`InquiryIntakeRecord`](./src/http.rs) values.
- `GET /agent/context/manager-daily-brief` via `manager_daily_brief_agent_context`: returns a source-grounded, read-only manager daily brief context packet built from `app::manager_daily_brief`, `domain::analytics`, `domain::source`, and local fixtures.
- `POST /agent/drafts/manager-daily-brief` via `submit_manager_daily_brief_agent_draft`: validates agent-submitted recommended actions before they can be shown to a manager.
- `POST /manager-daily-brief/actions/{action_id}/outcome` via `capture_manager_daily_brief_action_outcome`: captures staff feedback and labor-savings evidence as a `storage::operations::ManagerDailyBriefOutcomeRecord` in local state.
- `POST /vaccine-documents/uploads` via `upload_vaccine_document`: creates a local vaccine-document workflow payload with document, extraction, vaccine record, review packet, eligibility, and audit evidence.
- `POST /vaccine-documents/review-packets/{review_packet_id}/approve` and `/reject` via `approve_vaccine_document`, `reject_vaccine_document`, and `decide_vaccine_document`: records staff approval/rejection, updates document/vaccine/eligibility status, and preserves audit lineage.

## Context, draft-validation, and outcome-capture surfaces

The manager daily brief route group is the clearest current API-to-app boundary:

1. Context assembly starts in [`manager_daily_brief_agent_context`](./src/http.rs). It builds an `app::manager_daily_brief::Request` with `domain::entities::LocationId`, `domain::operations::operating_day::Date`, service-demand facts, checkout exception packets, and CRM retention opportunities, then evaluates `app::manager_daily_brief::Workflow::evaluate`.
2. Local context fixtures live in helpers such as [`local_manager_daily_brief_service_demand_facts`](./src/http.rs), [`local_manager_daily_brief_checkout_packets`](./src/http.rs), [`local_manager_daily_brief_retention_packets`](./src/http.rs), and provenance helpers like [`manager_brief_source_provenance`](./src/http.rs). These helpers are local smoke/demo scaffolding, not a substitute for storage or integration adapters.
3. Draft validation enters through [`ManagerDailyBriefAgentDraftSubmissionRequest`](./src/http.rs), [`ManagerDailyBriefSubmittedAction`](./src/http.rs), and [`submit_manager_daily_brief_agent_draft`](./src/http.rs). [`validate_manager_daily_brief_submitted_action`](./src/http.rs) requires known action kinds, source references, the required review gate from [`required_manager_daily_brief_review_gate`](./src/http.rs), and rejects every requested side effect through [`manager_daily_brief_requested_side_effect_rejection_reason`](./src/http.rs).
4. Outcome capture enters through [`ManagerDailyBriefOutcomeCaptureRequest`](./src/http.rs) and [`capture_manager_daily_brief_action_outcome`](./src/http.rs). It validates side effects, labor minutes with `storage::operations::StoredManagerDailyBriefLaborMinutes`, reporting scope with `domain::entities::LocationId` and `domain::operations::operating_day::Date`, and action ids against the app-evaluated packet before building `storage::operations::ManagerDailyBriefOutcomeRecord`.

Those surfaces are covered by [`tests/manager_daily_brief_agent_context_contract.rs`](./tests/manager_daily_brief_agent_context_contract.rs), [`tests/manager_daily_brief_agent_drafts_contract.rs`](./tests/manager_daily_brief_agent_drafts_contract.rs), and [`tests/manager_daily_brief_outcome_capture_contract.rs`](./tests/manager_daily_brief_outcome_capture_contract.rs). The tests are the best executable map of the contract: source-grounded context is read-only, draft submissions fail closed without source refs/review gates, live side effects are rejected, and outcomes record labor evidence without mutating provider systems.

## Inquiry and vaccine-document local workflows

The inquiry routes use API-local request/response structs because no durable inquiry app service exists in this shell yet. [`InquirySubmissionRequest`](./src/http.rs) is normalized by [`build_inquiry_intake_record`](./src/http.rs) into [`InquiryIntakeRecord`](./src/http.rs), which includes [`ParsedInquiryLead`](./src/http.rs), [`InquiryDraftReply`](./src/http.rs), [`InquiryTask`](./src/http.rs), and [`InquiryAuditEvent`](./src/http.rs). The important boundary is explicit in the payload: the draft reply is not live-sent, and the record says staff review is required.

The vaccine-document routes are also local-dev workflow scaffolding. [`VaccineDocumentUploadRequest`](./src/http.rs) creates [`DocumentRecord`](./src/http.rs), [`VaccineExtractionRecord`](./src/http.rs), [`VaccineRecord`](./src/http.rs), [`ReviewPacket`](./src/http.rs), [`PetEligibility`](./src/http.rs), and [`AuditEvent`](./src/http.rs) values inside [`VaccineDocumentStore`](./src/http.rs). Staff decisions use [`VaccineReviewDecisionRequest`](./src/http.rs), [`ApprovalRecord`](./src/http.rs), and [`decide_vaccine_document`](./src/http.rs). This models the shape of review-gated medical-document automation, but it remains an in-memory API contract fixture until storage/domain promotion cards move these records into the right crates.

See [`tests/vaccine_document_workflow_contract.rs`](./tests/vaccine_document_workflow_contract.rs) for the current assertions around extraction persistence, review packets, approval/rejection, eligibility status, and audit lineage.

## Type/module map

| Concept | Public or local type/module path | Defined in | Current role |
| --- | --- | --- | --- |
| API crate root | `pet_resort_api` | [`src/lib.rs`](./src/lib.rs) | Thin library surface that exposes the HTTP module. |
| HTTP module | `pet_resort_api::http` | [`src/http.rs`](./src/http.rs) | Router, handlers, request/response payloads, local state, and fixture adapters. |
| Binary entry point | `pet-resort-api` binary `main` | [`src/main.rs`](./src/main.rs) | Tracing, address parsing, listener binding, Axum serving, graceful shutdown. |
| Router constructor | `pet_resort_api::http::router` | [`src/http.rs`](./src/http.rs) | Builds the default router around process-global local state. |
| Testable router constructor | `pet_resort_api::http::router_with_state` | [`src/http.rs`](./src/http.rs) | Builds a router with explicit [`VaccineDocumentState`](./src/http.rs) for tests. |
| Local API state | `pet_resort_api::http::VaccineDocumentState` | [`src/http.rs`](./src/http.rs) | Cloneable wrapper around an in-memory `VaccineDocumentStore`. |
| Local store | private `VaccineDocumentStore` | [`src/http.rs`](./src/http.rs) | Holds inquiry records, vaccine workflow records, manager brief outcomes, and audit events during local/test runs. |
| Health/readiness payloads | private `HealthPayload`, `ReadinessPayload` | [`src/http.rs`](./src/http.rs) | Keep service identity and disabled/stubbed dependency posture explicit. |
| Inquiry request | private `InquirySubmissionRequest` | [`src/http.rs`](./src/http.rs) | HTTP boundary shape for inquiry intake. |
| Inquiry review packet | private `InquiryIntakeRecord` | [`src/http.rs`](./src/http.rs) | Review-gated staff queue record with normalized lead, draft reply, task, and audit events. |
| Inquiry draft reply | private `InquiryDraftReply` | [`src/http.rs`](./src/http.rs) | Draft-only customer reply with `live_send_allowed: false`. |
| Manager context query | private `ManagerDailyBriefAgentContextQuery` | [`src/http.rs`](./src/http.rs) | Query shape for location/operating-day context packets. |
| Manager draft submission | private `ManagerDailyBriefAgentDraftSubmissionRequest` | [`src/http.rs`](./src/http.rs) | HTTP boundary for agent-proposed manager brief actions. |
| Manager submitted action | private `ManagerDailyBriefSubmittedAction` | [`src/http.rs`](./src/http.rs) | Draft action payload validated for kind, source refs, review gates, and side effects. |
| Manager outcome request | private `ManagerDailyBriefOutcomeCaptureRequest` | [`src/http.rs`](./src/http.rs) | HTTP boundary for staff feedback and actual labor-minute capture. |
| Manager outcome projection | `storage::operations::ManagerDailyBriefOutcomeRecord` | [`../../storage/src/operations.rs`](../../storage/src/operations.rs) | Persistable/storage-shaped labor-savings evidence created by the API handler. |
| Manager app workflow | `app::manager_daily_brief::{Request, Workflow, Packet, BriefAction}` | [`../../app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs) | Source-grounded app evaluation consumed by context and outcome handlers. |
| Manager source facts | `domain::analytics::service_demand::Fact`, `domain::source::RecordRef` | [`../../domain/src/analytics.rs`](../../domain/src/analytics.rs), [`../../domain/src/source.rs`](../../domain/src/source.rs) | Domain evidence included in context packets and outcome source refs. |
| Vaccine upload request | private `VaccineDocumentUploadRequest` | [`src/http.rs`](./src/http.rs) | HTTP boundary for local vaccine-document intake. |
| Vaccine workflow payload | private `VaccineDocumentWorkflowPayload` | [`src/http.rs`](./src/http.rs) | Response envelope containing local document/extraction/review/eligibility/audit records. |
| Vaccine local records | private `DocumentRecord`, `VaccineExtractionRecord`, `VaccineRecord`, `ReviewPacket`, `ApprovalRecord`, `PetEligibility`, `AuditEvent` | [`src/http.rs`](./src/http.rs) | In-memory API contract fixture for medical-document review automation. |
| Vaccine decision request | private `VaccineReviewDecisionRequest` | [`src/http.rs`](./src/http.rs) | Staff approve/reject boundary for review packets. |

## Cross-crate relationships

- [`domain`](../../domain/README.md) owns semantic truth: identities, source refs, analytics facts, data-quality issues, policies, review gates, workflow contracts, document/vaccine concepts, and audit references. The API uses paths such as `domain::entities::LocationId`, `domain::operations::operating_day::Date`, `domain::analytics::service_demand::Fact`, `domain::data_quality::Issue`, `domain::source::Provenance`, and `domain::policy::ReviewGate` from [`../../domain/src`](../../domain/src).
- [`app`](../../app/README.md) owns use-case workflows. The manager brief handlers call [`app::manager_daily_brief`](../../app/src/manager_daily_brief.rs) and local fixture helpers also compose [`app::checkout_completion`](../../app/src/checkout_completion.rs) and [`app::crm_retention`](../../app/src/crm_retention.rs). New API behavior should call app services/workflows rather than embedding business decisions in handlers.
- [`storage`](../../storage/README.md) owns persistence-shaped records and codecs. The API currently stores manager brief outcomes as `storage::operations::ManagerDailyBriefOutcomeRecord` from [`../../storage/src/operations.rs`](../../storage/src/operations.rs); the rest of the in-memory API-local records should be promoted to storage/domain only by explicit future cards.
- [`integrations/gingr`](../../integrations/gingr/README.md) owns provider DTOs, endpoint requests, response parsing, webhooks, and mapping. This API crate should not parse raw Gingr payloads or mutate provider state directly; future routes should go through app ports and integration adapters.
- [`apps/worker`](../worker/README.md) is the background runtime shell. Scheduled manager briefs, queue consumers, and agent-runtime implementations belong there once implemented; the API should expose HTTP contracts and hand off long-running/background execution.
- [`apps/cli`](../cli/README.md) is the local/manual operator shell. Use it for source-controlled JSON inspection and local smoke commands rather than adding operator-only command behavior to HTTP handlers.
- [`docs/architecture/agent-app-infrastructure.md`](../../docs/architecture/agent-app-infrastructure.md) documents the agent/app boundary: deterministic app code owns source facts, policy, validation, audit, and side-effect authorization; agents consume typed context and return drafts/recommendations for validation.

## Labor-cost-reduction contribution

`apps/api` reduces labor cost by making reviewable automation surfaces reachable over HTTP while keeping the deterministic guardrails visible. The manager daily brief context route packages source-grounded demand, checkout, retention, data-quality, and labor-impact facts so a manager does not manually reconcile dashboards before deciding what needs attention. Draft validation rejects unsupported actions, missing source refs, missing review gates, and live side-effect requests before agent output can become a manager-visible recommendation. Outcome capture stores actual labor-minute feedback and source refs so the team can measure whether automation reduced manager/admin time.

The inquiry and vaccine-document routes show the same pattern at local-dev scale: normalize incoming work, draft but do not send customer communication, create staff-review tasks or medical-document review packets, and preserve audit evidence. Those flows save time only if they keep exception triage safer than manual handoffs; that is why every current live customer message, provider/PMS mutation, schedule change, payment/refund/discount move, and unsupported side effect fails closed.

## Maintainer notes

- Keep HTTP handlers as adapters. Deserialize, validate boundary shape, call `app`/`domain`/`storage` contracts, and serialize responses; move reusable workflow semantics out of [`src/http.rs`](./src/http.rs) as soon as they become app or domain concepts.
- Preserve semantic paths in prose and code: prefer `app::manager_daily_brief::Packet`, `domain::analytics::service_demand::Fact`, `domain::policy::ReviewGate`, and `storage::operations::ManagerDailyBriefOutcomeRecord` when those boundaries matter.
- Treat the API-local `VaccineDocumentStore` and private inquiry/vaccine structs as contract scaffolding, not canonical models. If they become durable product concepts, promote them into `domain`, `app`, or `storage` with explicit conversion boundaries.
- Do not add live side effects as route shortcuts. Customer sends, provider/PMS writes, schedule mutations, payment/refund/discount moves, and hidden data-quality decisions need app-owned validation, review gates, audit evidence, idempotency/replay controls, and integration adapters.
- Update this README when [`router_with_state`](./src/http.rs) changes. The route list and type/module map should match the current router and tests.
