---
title: "Runtime, storage, API, worker, CLI, and contract-test surfaces"
slug: "runtime-storage-api-surfaces"
family: "storage-api-runtime-shells"
status: "draft"
audience: ["general-manager", "regional-ops", "docs-writer", "engineering-maintainer"]
plain_english_definition: "The safe plumbing that stores review evidence, exposes typed API requests and responses, runs future background work, prints local inspection JSON, and proves those contracts with tests."
primary_labor_problem: "Reduces manager/front-desk reconciliation and engineering handoff time by keeping source facts, drafts, review gates, outcome records, and blocked side effects visible at every runtime boundary."
source_of_record: "domain and app workflow contracts, storage projections, API/worker/CLI source, local smoke fixtures, and executable tests linked from this page"
authoritative_human_role: "engineering maintainer for runtime contracts; manager/front-desk reviewer for workflow decisions and outcome records"
workflow_links: ["manager-daily-brief", "data-quality-hygiene", "vaccine-document", "booking-triage", "checkout-completion", "runtime-api-operations"]
source_paths:
  - "storage/src"
  - "apps/api"
  - "apps/worker"
  - "apps/cli"
  - "app/src/tools.rs"
  - "app/src/tools/error.rs"
  - "app/src/local_smoke.rs"
  - "docs/architecture/agent-app-infrastructure.md"
rustdoc_contracts:
  - "storage::operations"
  - "pet_resort_api::http::{VaccineDocumentState, router, router_with_state}"
  - "pet_resort_worker::runtime::{Config, AgentRuntimeMode, SideEffectMode}"
  - "app::tools::{CustomerStore, ReservationSystem, AgentRuntime, ExternalToolCandidate}"
  - "app::local_smoke::{run_fixture, FullChainEvidence, Stage, SmokeBoundaries}"
glossary_links:
  - "../glossary-architecture-terms.md"
  - "../glossary-source-data-terms.md"
  - "../glossary-workflow-state-terms.md"
allowed_action_summary: "read typed context, validate drafts, record review/outcome evidence, run deterministic local smoke examples, and expose safe inspection surfaces"
blocked_action_summary: "no autonomous customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, source hiding, safety approvals, or live side effects from these shells by default"
outcome_fields: ["source refs", "review gate", "draft validation result", "actual labor minutes", "outcome code", "reporting group", "audit event", "blocked side-effect reason"]
---

# Runtime, storage, API, worker, CLI, and contract-test surfaces

Purpose: give non-coder atlas writers a family page for the runtime and storage entities that support pet-resort workflow pages. These entities are not pets, reservations, or customers. They are the safe plumbing that turns source facts into reviewable packets, draft validations, outcome records, and contract tests without letting an agent or shell silently perform live customer, provider, payment, or schedule actions.

Use this page with the [entity atlas inventory](entity-atlas-inventory.md), the [entity atlas page template](entity-atlas-page-template.md), the [documentation style guide](../quality/nva-documentation-style-guide.md), and the [Agent-App Infrastructure Guide](../architecture/agent-app-infrastructure.md). Markdown is orientation; the source and tests linked below are the behavior contract.

## Plain-English pet-resort definition

Runtime and storage surfaces are the behind-the-scenes counters, inboxes, drafts, receipts, and safety switches for workflow automation:

- Storage projections are durable-looking records for evidence, outcomes, service offerings, and code values. They preserve what was reviewed or measured; they do not become the business rulebook.
- API request/response contracts are the staff-facing or agent-facing doors into workflows. They shape context packets, draft submissions, review decisions, and outcome capture.
- Worker runtime modes are the background-process safety switches. Today they are fake/deterministic or disabled, with side effects stubbed.
- CLI and local smoke surfaces are manual inspection/demo paths. They print source-controlled JSON or run fixtures so maintainers can inspect contracts without live systems.
- Tool errors are typed “why this could not safely complete” results, not vague logs.
- Contract tests are the executable proof that the source facts, drafts, review gates, blocked actions, and outcomes still match the docs.

## Purpose: labor-cost or safety problem

This page helps docs writers, maintainers, and reviewers avoid confusing safe automation support with live authority. A manager daily brief can save time only if source facts, draft validation, and outcome capture are traceable. A data-quality queue can reduce rework only if missing, stale, conflicting, or sensitive facts stay visible. A runtime shell can support agents only if it cannot bypass app policy, human review, or audit evidence.

## Relationship map

```text
provider/source evidence or local fixture
  -> domain source refs, policy, and workflow facts
  -> app workflow packet or app tool request
  -> API/CLI/worker runtime shell
  -> draft validation or read-only context
  -> staff review gate
  -> storage outcome/projection record
  -> contract test proving blocked actions still fail closed
```

The important distinction is authority:

- Source systems and domain/app contracts own operational meaning.
- Storage records preserve normalized projection and outcome evidence.
- API/worker/CLI shells expose or run contracts; they do not invent new pet-resort policy.
- Hermes/agents may summarize, rank, draft, route, and propose through typed tools.
- Human review and deterministic app validation decide whether a draft can be accepted or a live side effect can ever be attempted.

## Atlas entry: Storage operations boundary

### Plain-English definition

The storage operations boundary is the filing cabinet for reviewable records, stable codes, JSON codecs, and storage-specific validation. It stores shapes such as manager daily brief outcomes, data-quality hygiene outcomes, service offerings, core service contracts, technology ecosystem records, and source-record references.

### What it reduces

It reduces repeated reconciliation by giving managers and maintainers a stable place to inspect what was projected, what source refs were attached, what labor minutes were recorded, and why invalid persisted values were rejected.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Storage README | [`storage/README.md`](../../storage/README.md) | Storage owns records/codes/codecs and conversion boundaries, not domain truth. |
| Crate root | [`storage/src/lib.rs`](../../storage/src/lib.rs) | Public crate surface and re-exports. |
| Operations source | [`storage/src/operations.rs`](../../storage/src/operations.rs) | `Error`, `CodecError`, `RecordKind`, `StorageField`, `StoredSourceRecordRef`, outcome records, service records, technology records, and conversions. |
| Service-line storage modules | [`storage/src/service_line/mod.rs`](../../storage/src/service_line/mod.rs) | Boarding/daycare/grooming/training/retail storage wrappers and code tables. |
| Storage service-line guide | [`storage/src/service_line/README.md`](../../storage/src/service_line/README.md) | How service-line records wrap domain contracts without replacing policy. |
| Storage tests | [`storage/tests`](../../storage/tests) | Executable proof for codecs, conversions, shape validation, and outcome storage. |
| Rustdoc/module paths | `storage::operations`; `storage::service_line` | Exact compiled contract once rendered Rustdoc is generated or published. |

### Source of record and human role

| Fact or decision | Source of record | Human role when incomplete or sensitive |
| --- | --- | --- |
| Domain business meaning | `domain::*` types that storage converts to/from | Engineering maintainer verifies semantic boundary. |
| Persisted shape/code | `storage::operations` and `storage::service_line` records/codes | Engineering maintainer reviews migrations and storage compatibility. |
| Outcome minutes and dispositions | App workflow outcome plus storage outcome record | Manager/front-desk reviewer records or audits the result. |
| Source traceability | `StoredSourceRecordRef` plus domain provenance/source refs | Reviewer checks whether evidence is current enough for a recommendation. |
| Bad persisted value | `Error`, `CodecError`, `ShapeMismatchReason`, `StorageField` | Maintainer/data-quality reviewer decides repair or quarantine path. |

### Allowed actions

Storage may encode/decode JSON records, validate storage scalars, map stable code values to semantic domain values, reject malformed shapes, preserve source refs, and record reviewed outcome evidence. Docs may say storage supports reporting and replay evidence.

### Blocked actions and review gates

Storage does not send customer messages, mutate Gingr or another PMS, book reservations, assign staff, change schedules, approve vaccines or safety exceptions, move money, hide source issues, or create domain policy. A storage record can prove a reviewed outcome existed; it cannot by itself authorize the next live action.

### Safe-use evidence and outcome fields

Safe storage use needs source refs, record kind, code values, validation outcome, review disposition, actual labor minutes when present, and reporting group. Contract tests should prove codecs preserve those values and reject zero/invalid labor-minute quantities.

### Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | `ManagerDailyBriefOutcomeRecord` | Stores source refs, action kind, persona, reporting group, and labor evidence for a reviewed manager action. |
| Example | `StoredSourceRecordRef` | Keeps projected records traceable to source evidence. |
| Non-example | `ServiceOfferingKindCode` by itself | It is a storage classifier; explain it under the service-offering/storage page, not as standalone business truth. |
| Non-example | JSON codec success | It proves serialization shape, not that a workflow action was safe or reviewed. |

## Atlas entry: API request/response contracts

### Plain-English definition

The API shell is the HTTP front door for deterministic local workflow contracts. It accepts request bodies, returns response packets, and holds in-memory state for demos/tests. It is a runtime shell around app/domain/storage contracts, not a place to invent provider facts or hidden business decisions.

### What it reduces

It reduces manager and engineering handoff time by making context packets, draft submissions, review decisions, and outcome capture reachable over repeatable HTTP routes. Staff and agents can work from explicit request/response shapes instead of copying raw provider screens or free-form prompts.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| API README | [`apps/api/README.md`](../../apps/api/README.md) | Current route list, shell boundary, and route-to-test map. |
| HTTP source | [`apps/api/src/http.rs`](../../apps/api/src/http.rs) | `VaccineDocumentState`, `router`, `router_with_state`, handlers, DTOs, local store, and validation helpers. |
| API crate root | [`apps/api/src/lib.rs`](../../apps/api/src/lib.rs) | Exposes `pet_resort_api::http`. |
| API binary | [`apps/api/src/main.rs`](../../apps/api/src/main.rs) | Address binding and Axum serving; no business policy. |
| Manager context/drafts/outcomes tests | [`apps/api/tests`](../../apps/api/tests) | Context is read-only, drafts fail closed, live side effects are rejected, outcomes record evidence. |
| Architecture guide | [`docs/architecture/agent-app-infrastructure.md`](../architecture/agent-app-infrastructure.md) | Context-packet, draft-validation, human-review, audit/replay, and memory boundaries. |
| Rustdoc/module paths | `pet_resort_api::http::{VaccineDocumentState, router, router_with_state}` | Exact compiled contract once rendered Rustdoc exists. |

### API workflow surfaces

| Surface | Non-coder meaning | Safe result |
| --- | --- | --- |
| `/healthz` and `/readyz` | Runtime posture check | Reports service status and disabled/stubbed dependencies. |
| Inquiry intake | A local staff queue entry with normalized lead, draft reply, task, and audit events | Draft reply and staff-review task; no live customer send. |
| Manager daily brief context | Source-grounded packet for manager action review | Read-only context with source facts and blocked actions. |
| Manager daily brief draft submission | Agent proposed manager actions | Accepted/rejected draft validation; blocked side effects fail closed. |
| Manager daily brief outcome capture | Staff feedback after action review | Storage-shaped outcome record with labor evidence. |
| Data-quality hygiene context/draft/outcome | Source-issue review queue and cleanup evidence | Reviewable cleanup actions and outcome capture; no source hiding. |
| Vaccine document upload/review | Local medical-document review packet | Approval/rejection/audit status; no hidden medical acceptance. |

### Source of record and human role

| Fact or decision | Source of record | Human role when incomplete or sensitive |
| --- | --- | --- |
| HTTP route shape | `apps/api/src/http.rs` and API tests | Engineering maintainer keeps docs aligned with router. |
| Workflow meaning | `app::*` workflow modules and `domain::*` contracts | Workflow owner/maintainer validates policy and packet semantics. |
| Outcome projection | `storage::operations` record built by handler | Manager/front-desk reviewer records actual outcome and minutes. |
| Vaccine decision | Local API review packet and domain document/vaccine concepts | Trained staff or manager approves/rejects. |
| Customer-facing draft | API-local draft plus app validation | Approved sender or front desk reviews before any send. |

### Allowed actions

The API may deserialize requests, build deterministic context packets from local fixtures/app workflows, validate drafts, reject side-effect requests, record review decisions, and store in-memory or storage-shaped outcome projections for tests/demos.

### Blocked actions and review gates

The API shell must not become a shortcut for provider/PMS writes, live customer sends, payment/refund/discount moves, schedule changes, medical/vaccine approvals without review, or source-data hiding. Every sensitive route should expose a review packet, approval/rejection status, blocked action, or audit event.

### Safe-use evidence and outcome fields

Safe API evidence includes request id or context id, source refs, required review gate, accepted/rejected draft status, blocked side-effect reason, audit events, staff decision, actual labor minutes, and storage outcome record fields.

## Atlas entry: Worker runtime shell

### Plain-English definition

The worker runtime shell is the future background process for scheduled or queued app-owned work. Today it mainly proves the safety posture: agent execution is fake deterministic or disabled, and side effects are stubbed.

### What it reduces

It creates a safe place to develop future background manager briefs, queue consumers, task/schedule drafts, and agent-runtime adapters without accidentally sending messages, mutating providers, changing schedules, or moving money.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Worker README | [`apps/worker/README.md`](../../apps/worker/README.md) | Thin shell role, future background work, and side-effect boundary. |
| Runtime source | [`apps/worker/src/runtime.rs`](../../apps/worker/src/runtime.rs) | `AgentRuntimeMode`, `SideEffectMode`, `Config`, env-default behavior. |
| Worker crate root | [`apps/worker/src/lib.rs`](../../apps/worker/src/lib.rs) | Exposes worker runtime module. |
| Worker binary | [`apps/worker/src/main.rs`](../../apps/worker/src/main.rs) | Tracing, mode logging, and shutdown plumbing. |
| Runtime test | [`apps/worker/tests/runtime_mode_contract.rs`](../../apps/worker/tests/runtime_mode_contract.rs) | Default remains fake deterministic and side-effect safe. |
| App tool ports | [`app/src/tools.rs`](../../app/src/tools.rs) | Future worker adapters should satisfy app-owned tool contracts. |
| Rustdoc/module paths | `pet_resort_worker::runtime::{Config, AgentRuntimeMode, SideEffectMode}` | Exact compiled contract once rendered Rustdoc exists. |

### Source of record and human role

| Fact or decision | Source of record | Human role when incomplete or sensitive |
| --- | --- | --- |
| Runtime mode | `PET_RESORT_AGENT_RUNTIME_MODE` parsed by `Config::from_env_defaults` | Engineering/operator chooses deployment mode. |
| Side-effect posture | `SideEffectMode::Stubbed` in runtime source | Engineering/security reviewer approves any future live mode. |
| Background workflow semantics | `app` workflow and tool contracts | Workflow owner reviews policy and review gates. |
| Queue/schedule execution | Future worker adapter code and tests | Operator/maintainer reviews reliability and replay evidence. |

### Allowed actions

The worker may boot, log runtime posture, select fake deterministic or disabled agent mode, keep side effects stubbed, and later wire app-owned workflows through typed ports when those adapters exist.

### Blocked actions and review gates

No live side-effect mode exists today. Do not document the worker as sending customer messages, writing Gingr/PMS records, executing payments/refunds/discounts, changing schedules, or approving safety-sensitive decisions. Any future live mode needs app validation, review-gate evidence, idempotency/replay controls, redaction/evidence hygiene, and tests proving blocked actions fail closed.

## Atlas entry: CLI and local smoke surfaces

### Plain-English definition

The CLI is a local/manual command shell. Local smoke surfaces are deterministic examples and tests that run workflow chains without live systems. Together they let maintainers inspect agent specs, tool candidates, and workflow evidence before building real adapters.

### What it reduces

They reduce coordination cost by making source-controlled automation vocabulary and local workflow examples visible as JSON or tests. A maintainer can inspect “what agents/tools exist?” and “does the full chain preserve review boundaries?” without asking a manager to interpret raw provider screens.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| CLI README | [`apps/cli/README.md`](../../apps/cli/README.md) | Current command list and thin-shell boundary. |
| CLI source | [`apps/cli/src/main.rs`](../../apps/cli/src/main.rs) | `pet-resort agents` and `pet-resort tools` only. |
| Local smoke source | [`app/src/local_smoke.rs`](../../app/src/local_smoke.rs) | `run_fixture`, stage taxonomy, smoke boundaries, review evidence, and full-chain evidence. |
| Local smoke test | [`app/tests/full_chain_local_smoke.rs`](../../app/tests/full_chain_local_smoke.rs) | Full-chain fixture behavior and no-external-side-effect boundaries. |
| Data-quality local example | [`app/examples/data_quality_hygiene_local_smoke.rs`](../../app/examples/data_quality_hygiene_local_smoke.rs) | Reviewable data-quality queue, blocked provider/customer effects, and outcome capture. |
| App tool README | [`app/src/tools/README.md`](../../app/src/tools/README.md) | Tool ports, typed outcomes, and review/error posture exposed to CLI/runtime planning. |
| Rustdoc/module paths | `app::local_smoke::{run_fixture, FullChainEvidence, Stage, SmokeBoundaries}` | Exact compiled contract once rendered Rustdoc exists. |

### CLI commands and local examples

| Surface | Non-coder meaning | Safe result |
| --- | --- | --- |
| `pet-resort agents` | Prints baseline agent specs assembled from app/domain contracts | Reviewable JSON of allowed tools, forbidden actions, and review gates. |
| `pet-resort tools` | Prints candidate external systems named by `app::tools` | Reviewable JSON inventory; no provider calls. |
| `app::local_smoke::run_fixture` | Runs a deterministic chain from source event through reviewable evidence | Full-chain evidence packet with explicit boundaries. |
| `data_quality_hygiene_local_smoke` example | Demonstrates stale/source issue queue, draft validation, blocked side effects, and outcome record | Local proof that cleanup drafts and outcomes stay review-gated. |

### Allowed actions

The CLI may print source-controlled JSON for agents and tool candidates. Local smoke fixtures may run deterministic app workflows, produce review evidence, and prove side-effect boundaries in tests/examples.

### Blocked actions and review gates

The CLI and local smoke examples do not call live providers, send customer messages, mutate storage, execute payments, alter schedules, or approve safety/medical decisions. Do not document them as operational automation; they are inspection and contract-demo surfaces.

## Atlas entry: App tool ports and tool errors

### Plain-English definition

App tool ports are typed promises between deterministic workflows and concrete capabilities. They say “the app may ask for this kind of lookup, draft, OCR, payment review, media snapshot, or Hermes task draft” without choosing a provider or granting live authority.

Tool errors are the typed explanations returned when a tool cannot safely complete: missing resource, policy denied, or external system unavailable.

### What it reduces

Tool ports reduce handoff time by replacing raw provider calls and vague agent instructions with typed requests/outcomes. Staff and maintainers can see whether a result was unavailable, draft-only, requires review, or rejected by policy.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| App tool README | [`app/src/tools/README.md`](../../app/src/tools/README.md) | Plain-language tool port map and safety posture. |
| Tool source | [`app/src/tools.rs`](../../app/src/tools.rs) | `CustomerStore`, `ReservationSystem`, `AgentRuntime`, modules for availability, draft update, portal, payment, messaging, documents, media, Hermes, and `ExternalToolCandidate`. |
| Tool error source | [`app/src/tools/error.rs`](../../app/src/tools/error.rs) | `Error`, `ExternalFailure`, `Resource`, `ResourceId`, and `Result`. |
| App README | [`app/README.md`](../../app/README.md) | How app workflows compose domain truth and tool ports. |
| Architecture guide | [`docs/architecture/agent-app-infrastructure.md`](../architecture/agent-app-infrastructure.md) | Agents consume typed context and return drafts/recommendations for app validation. |
| CLI tools command | [`apps/cli/src/main.rs`](../../apps/cli/src/main.rs) | Tool inventory is inspectable without live provider calls. |
| Rustdoc/module paths | `app::tools::{CustomerStore, ReservationSystem, AgentRuntime, ExternalToolCandidate}`; `app::tools::error::{Error, ExternalFailure, Resource, ResourceId}` | Exact compiled contract once rendered Rustdoc exists. |

### Tool families

| Tool family | Non-coder meaning | Safe result |
| --- | --- | --- |
| Customer store | Read source-backed customer, pet, and reservation records | Facts for review packets; not a write path. |
| Reservation system | Check availability and draft reservation updates | Review-held drafts; no PMS mutation by default. |
| Agent runtime | Run a model/tool runner over typed workflow input | Typed workflow result that app validates. |
| Portal lookup | Find provider/account matches | Match/not-found/ambiguous outcome, not source truth by itself. |
| Payment | Authorization/refund/deposit helper vocabulary | Provider result or human-review requirement; no hidden money movement. |
| Messaging | Draft customer/staff/manager messages | Draft id/status requiring review; no send. |
| Documents/OCR | Intake document and extract text | Reviewable document/OCR evidence. |
| Media | Request camera/media snapshot | Media ref or unavailable reason; no device authority. |
| Hermes hooks | Draft task or schedule | Drafted task/schedule status requiring review. |

### Source of record and human role

| Fact or decision | Source of record | Human role when incomplete or sensitive |
| --- | --- | --- |
| Tool request/outcome shape | `app::tools` source and README | Engineering maintainer reviews port semantics. |
| Policy denial | `domain::policy::denial::Reason` through `app::tools::Error::PolicyDenied` | Workflow owner/manager interprets operational impact. |
| Missing resource | `Resource` and `ResourceId` | Front desk/data-quality reviewer resolves missing source record. |
| External failure | `ExternalFailure` classifier plus adapter logs | Operator/maintainer investigates provider/system outage. |
| Draft approval | App workflow review gate and approval record | Staff/manager or approved sender decides. |

### Allowed actions

Tool ports may read typed records, check availability, create draft-only updates/messages/tasks/schedules, return OCR/media evidence, classify external failures, and expose policy denial as a first-class result.

### Blocked actions and review gates

A tool result must not imply a customer was messaged, a provider/PMS record was changed, money moved, a schedule changed, or a safety-sensitive approval happened unless a separate app-owned policy/review path executes that side effect. Provider-specific errors should not be flattened until enough detail is preserved for audit/review.

### Safe-use evidence and outcome fields

Safe tool use needs typed ids, source refs, request/outcome type, review policy/status, unavailable or denial reason, draft id when created, idempotency/correlation where relevant, and downstream outcome capture after staff review.

## Atlas entry: App packets and outcome records used by runtime surfaces

### Plain-English definition

App packets are the work envelopes a workflow produces for staff or agents: source facts, recommended actions, allowed/blocked actions, review gates, draft text, and outcome fields. Runtime shells expose packets; storage records preserve reviewed outcomes.

### What it reduces

Packets reduce labor by grouping the facts a manager/front-desk lead would otherwise reconcile manually, while keeping the review decision and labor result measurable.

### Contract map

| Packet or record family | Link or path | What the writer should verify |
| --- | --- | --- |
| Manager daily brief packet | [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs) | Source facts, brief actions, safe/blocked actions, labor estimates, and outcome record. |
| Manager daily brief storage outcome | [`storage/tests/manager_daily_brief_outcome_storage.rs`](../../storage/tests/manager_daily_brief_outcome_storage.rs) | Outcome codecs preserve labor savings and reject zero minutes. |
| Manager daily brief API contracts | [`apps/api/tests/manager_daily_brief_agent_context_contract.rs`](../../apps/api/tests/manager_daily_brief_agent_context_contract.rs), [`apps/api/tests/manager_daily_brief_agent_drafts_contract.rs`](../../apps/api/tests/manager_daily_brief_agent_drafts_contract.rs), [`apps/api/tests/manager_daily_brief_outcome_capture_contract.rs`](../../apps/api/tests/manager_daily_brief_outcome_capture_contract.rs) | Context is source-grounded, drafts are validated, outcomes record evidence. |
| Data-quality hygiene packet | [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs) | Candidate/action/outcome shape, source freshness, sensitivity, blocked effects. |
| Data-quality API contract | [`apps/api/tests/data_quality_hygiene_agent_contract.rs`](../../apps/api/tests/data_quality_hygiene_agent_contract.rs) | API context/draft/outcome route behavior. |
| Data-quality storage outcome | [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs) | Outcome record preserves labor/provenance and rejects zero minutes. |
| Vaccine document local packet | [`apps/api/tests/vaccine_document_workflow_contract.rs`](../../apps/api/tests/vaccine_document_workflow_contract.rs) | Upload/review/approval/rejection/audit lineage in API-local workflow. |
| Booking/checkout/retention/daily update tests | [`app/tests`](../../app/tests) | Workflow packets preserve review gates, source evidence, blocked actions, and draft boundaries. |

### Allowed actions

App packets may rank work, summarize source evidence, draft internal/customer-facing text for review, estimate labor minutes, validate draft submissions, record outcome evidence, and preserve data-quality issues.

### Blocked actions and review gates

Packets do not grant live authority. They must carry blocked actions such as provider/PMS mutation, customer sends, schedule changes, payment/refund/discount moves, source hiding, and safety/policy approvals. Accepted drafts still need app validation and human review when sensitive.

### Safe-use evidence and outcome fields

Safe packet use needs workflow name/version, context/correlation ids when available, source refs/provenance, source fact kinds, data-quality issues, allowed/blocked actions, required review gates, draft validation status, audit events, outcome disposition, actual minutes, and reporting group.

## Atlas entry: Contract tests as documentation authority

### Plain-English definition

Contract tests are executable documentation. They prove the repository still behaves the way the atlas page claims: side effects are blocked, drafts are validated, storage values round-trip, and outcome evidence stays measurable.

### What it reduces

They reduce stale-doc and unsafe-regression work. A writer can link the exact test file that proves a runtime or storage boundary instead of relying on prose alone.

### Test surface map

| Test surface | Link or path | What it proves for non-coders |
| --- | --- | --- |
| Storage operations contracts | [`storage/tests/operations_storage_contracts.rs`](../../storage/tests/operations_storage_contracts.rs) | Storage conversions/codecs reject invalid shapes and preserve semantic boundaries. |
| Core service contract storage | [`storage/tests/core_service_contract_storage.rs`](../../storage/tests/core_service_contract_storage.rs) | Service-line contract records can round-trip without replacing domain policy. |
| Manager outcome storage | [`storage/tests/manager_daily_brief_outcome_storage.rs`](../../storage/tests/manager_daily_brief_outcome_storage.rs) | Manager outcome records preserve labor/source evidence and reject zero minutes. |
| Data-quality outcome storage | [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs) | Data-quality outcomes preserve provenance, disposition, and labor evidence. |
| API health/readiness | [`apps/api/tests/health_contract.rs`](../../apps/api/tests/health_contract.rs) | Runtime reports disabled/stubbed live side effects. |
| Manager API context/draft/outcome | [`apps/api/tests`](../../apps/api/tests) | Context/draft/outcome routes enforce source refs, review gates, and blocked effects. |
| Worker runtime mode | [`apps/worker/tests/runtime_mode_contract.rs`](../../apps/worker/tests/runtime_mode_contract.rs) | Default worker is fake deterministic and side-effect safe. |
| Full-chain local smoke | [`app/tests/full_chain_local_smoke.rs`](../../app/tests/full_chain_local_smoke.rs) | Local deterministic chain preserves review evidence and boundaries. |
| App workflow contracts | [`app/tests`](../../app/tests) | Booking, checkout, retention, daily update, manager brief, and data-quality packets remain review-gated. |

### How writers should use tests

- Link a test file when claiming a runtime, packet, storage record, or blocked action is enforced.
- Say “contract test” or “executable contract,” not “business proof,” unless the test includes real reviewed outcomes.
- Do not claim a rendered Rustdoc URL exists unless `target/doc` or a published docs site exists for this workspace.
- If a test uses deterministic/local fixtures, describe it as local proof of shape and safety boundaries, not live operational evidence.

## Cross-family notes for later atlas pages

1. Storage projections and outcome records belong beside workflow pages because they prove measurement and source traceability; they do not own business policy.
2. API request/response DTOs should be merged into the API runtime entry unless a DTO becomes a reusable app/domain concept.
3. Worker runtime config belongs in a runtime-shell entry until a concrete queue/scheduler/agent adapter has its own operator-facing workflow.
4. CLI commands belong in the local/manual shell entry unless a command begins executing a real app workflow with reviewable inputs/outputs.
5. App tool ports deserve relationship entries because they are the safety contract between deterministic workflows and external capabilities.
6. Contract tests should appear under every page section that claims a blocked action, review gate, storage codec, or outcome field.

## Glossary cross-links

Use the repo glossary pages when introducing boundary terms in final public docs: [architecture terms](../glossary-architecture-terms.md), [source/data terms](../glossary-source-data-terms.md), and [workflow-state terms](../glossary-workflow-state-terms.md). The most important terms for this family are storage, read model/projection, source ref, provenance, app, tool port, DTO, review gate, blocked action, draft, workflow packet, and outcome capture.
