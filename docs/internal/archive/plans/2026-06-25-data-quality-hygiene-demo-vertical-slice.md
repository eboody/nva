# Data-Quality Hygiene Demo Vertical Slice Implementation Plan

> **For Hermes:** Historical board plan artifact. The implemented presentation slice below is the local/fixture-safe proof path; Postgres-backed adapter work remains future scope unless a later card explicitly reopens it.

**Chosen workflow:** Data-Quality Hygiene.

**Delivered scope:** Local/demo Data-Quality Hygiene workflow from API request through typed app/domain packet, storage-shaped review/audit/outbox candidate projections, fake/disabled worker proof, and readiness/log/metric evidence. The API still uses in-memory state; no production database, provider write, customer send, payment action, or deployment is enabled.

**Original goal:** Wire one local/demo Data-Quality Hygiene workflow end to end from API request through typed app/domain packet, durable storage/review/audit/outbox candidate rows, fake/disabled worker result, and readiness/log/metric proof.

**Why this slice:** Repository discovery supports the task default. Data-Quality Hygiene already has a strong labor-reduction story, explicit source ambiguity/review gates, app/API/storage contracts, local smoke evidence, and outcome/labor projections. Manager Daily Brief is also strong, but Data-Quality Hygiene is the cleaner proof that provider/source data is evidence to review rather than blind business truth.

**Architecture:** Keep the current API route family and `WorkflowRepository` seam, but add one local Postgres-backed adapter path for the Data-Quality Hygiene route family. API handlers continue to expose product-owned DTOs, promote only through `app::data_quality_hygiene`, persist semantic workflow/review/audit/outcome/outbox-candidate projections, and run the worker in fake deterministic or disabled mode. The demo must prove the processing lineage without enabling live customer/provider/payment side effects.

**Tech stack / proof surfaces:** Rust workspace, Axum API (`apps/api/src/http.rs`), app/domain workflow contracts (`app/src/data_quality_hygiene.rs`, `domain/src/data_quality.rs`, `domain/src/source.rs`), SQL migration (`migrations/0001_mvp_foundation.sql`), storage codecs (`storage/src/operations.rs`), fake/disabled worker runtime (`apps/worker/src/runtime.rs`), local smoke scripts and contract tests.

---

## Current proof paths read

- `README.md` — canonical labor-cost operating model, proof-chain lens, and blocked-action boundaries.
- `docs/presentation/job-presentation-walkthrough.md` — presentation story and recommended next safe vertical slice shape.
- `docs/architecture/runtime-contract-boundaries.md` — provider evidence -> domain/app -> API DTO -> review -> DB/audit/outbox -> logs/metrics boundary.
- `docs/audits/dto-api-db-observability-readiness-gap-map.md` — P0 gaps: durable API persistence and request/job correlation.
- `docs/workflows/operator/data-quality-hygiene.md` — workflow-specific labor story, contracts, blocked actions, evidence paths, and smoke proof.
- `app/src/data_quality_hygiene.rs` — typed workflow packet, actions, blocked actions, draft validation, outcome record.
- `apps/api/src/http.rs` — current safe route family and in-memory repository seam.
- `migrations/0001_mvp_foundation.sql` — `workflow_events`, `workflow_results`, `review_packets`, `approval_records`, `data_quality_hygiene_outcomes`, `outbox_records`, `audit_events`.
- `storage/src/operations.rs` — data-quality outcome storage codes, source refs, labor-minute wrappers, reporting groups.
- `storage/tests/mvp_migration_contract.rs` — migration contract proof for workflow/review/outbox/audit/outcome invariants.

---

## End-to-end target flow

1. API request: `GET /agent/context/data-quality-hygiene?location_id=...&operating_day=...` builds the source-grounded local context packet.
2. Typed app/domain packet: `app::data_quality_hygiene::Workflow::evaluate` emits `Packet`, `Action`, source refs, issue refs, review gates, blocked actions, redaction boundary, and labor estimate from `domain::data_quality` and `domain::source` facts.
3. Storage/review/audit rows: Postgres adapter inserts a `workflow_events` row for the request, `review_packets` rows for manager/front-desk review work, and `audit_events` rows for context creation and draft validation.
4. Draft review: `POST /agent/drafts/data-quality-hygiene` accepts only internal cleanup drafts that cite packet/source/issue refs and request no live side effects; it writes `workflow_results` with `needs_review` or `succeeded` for the fake deterministic result.
5. Outcome capture: `POST /data-quality-hygiene/actions/{action_id}/outcome` writes `approval_records` for reviewed disposition plus `data_quality_hygiene_outcomes` with labor minutes, source refs, issue refs, correlation id, owner persona, and resolution status.
6. Outbox candidate only: if a reviewed internal handoff is needed, insert an `outbox_records` row with a local/internal topic only after the matching approved `approval_records` row exists. It is a handoff candidate, not a send/write permission.
7. Disabled/fake worker result: `apps/worker` remains `FakeDeterministic` or `Disabled`, claims only local demo work if a worker loop is added, and writes a safe `workflow_results`/audit record without publishing side effects.
8. Readiness/log/metric proof: `GET /readyz` and `GET /ops/metrics/summary` continue to state live side effects/provider writes/customer messaging disabled; route logs include request id/correlation evidence; outcome summary shows aggregate reviewed labor proof.

---

## Files and functions to touch

### API / runtime

- Modify `apps/api/src/http.rs`
  - Keep routes:
    - `GET /agent/context/data-quality-hygiene` -> `data_quality_hygiene_agent_context`
    - `POST /agent/drafts/data-quality-hygiene` -> `submit_data_quality_hygiene_agent_draft`
    - `POST /data-quality-hygiene/actions/{action_id}/outcome` -> `capture_data_quality_hygiene_action_outcome`
    - `GET /data-quality-hygiene/outcomes/summary` -> `data_quality_hygiene_outcome_summary`
    - `GET /readyz` -> `readyz`
    - `GET /ops/metrics/summary` -> `ops_metrics_summary`
  - Extend `WorkflowRepository` so Data-Quality Hygiene has semantic methods for recording a context workflow event, draft validation result, review packet, approval record, outcome record, audit event, optional internal outbox candidate, and summary readback.
  - Add/route a Postgres-backed implementation behind the same trait while preserving the current in-memory implementation for deterministic tests.
  - Add request/correlation id propagation to Data-Quality Hygiene context, draft, outcome, and summary payloads without logging raw provider payloads.

- Modify or add `apps/api/tests/data_quality_hygiene_agent_contract.rs`
  - Prove context payload contains API contract marker, source refs, issue refs, blocked actions, correlation/request evidence, and `live_side_effects_allowed=false`.
  - Prove draft validation rejects `send_customer_message`, provider/PMS mutation, ambiguity hiding, and unknown side effects.
  - Prove outcome capture rejects empty source refs, empty issue refs, zero actual minutes, unknown action id, and any requested side effect.
  - Prove successful outcome increments summary/metrics while preserving disabled live side-effect posture.

- Add focused integration test, e.g. `apps/api/tests/data_quality_hygiene_postgres_vertical_slice.rs`
  - Use local/test Postgres or the repo's established database-test harness if present.
  - Exercise context -> draft -> outcome -> summary through the API/repository seam.
  - Assert rows exist in `workflow_events`, `workflow_results`, `review_packets`, `approval_records`, `audit_events`, `data_quality_hygiene_outcomes`, and, only for approved internal handoff candidates, `outbox_records`.
  - Assert no customer/provider/payment topics or payloads are created.

### App/domain contracts

- Modify `app/src/data_quality_hygiene.rs` only if the API/storage adapter needs additional semantic accessors.
  - Preserve `WORKFLOW_NAME = "data-quality-hygiene"` and `SCHEMA_VERSION`.
  - Do not add storage-specific or API-specific types here.
  - Prefer semantic newtypes/accessors over raw strings if a new field becomes necessary.

- Modify `app/tests/data_quality_hygiene_workflow_contracts.rs`
  - Add contract assertions only for new app-level invariants, such as required correlation/workflow id fields or new fake-worker result classification.
  - Do not test SQL shape here.

- Modify `domain/src/data_quality.rs` or `domain/src/source.rs` only for genuine missing business concepts.
  - Do not add API DTO, SQL row, or Toasty abstractions to domain.

### Storage / migration

- Modify `storage/src/operations.rs`
  - Add or reuse record shapes/codecs for the durable Data-Quality Hygiene outcome and source refs.
  - Add storage-level conversion helpers only where they preserve semantic projection boundaries.

- Modify `storage/tests/data_quality_hygiene_outcome_storage.rs`
  - Prove storage codecs preserve outcome, source refs, issue refs, labor minutes, correlation id, reporting group, owner persona, action kind, and resolution status.

- Modify `migrations/0001_mvp_foundation.sql` only if existing tables lack a needed local-demo column or constraint.
  - Expected tables to use as-is: `workflow_events`, `workflow_results`, `review_packets`, `approval_records`, `data_quality_hygiene_outcomes`, `outbox_records`, `audit_events`.
  - Preserve outbox approval constraint and append-only audit triggers.
  - Do not add auth/session/role/location authorization, real provider snapshots, durable general job leasing, or infra dashboards in this slice unless required by tests for the local proof.

- Modify `storage/tests/mvp_migration_contract.rs`
  - Add assertions for any new migration invariant.
  - Keep existing deferred-surface assertions honest.

### Worker / fake result

- Modify `apps/worker/src/runtime.rs` only if the demo needs a named fake Data-Quality Hygiene result.
  - Preserve `FakeDeterministic` and `Disabled` behavior.
  - Preserve `SideEffectMode::Stubbed`.
  - Do not add live adapter calls.

- Add or modify `apps/worker/tests/runtime_mode_contract.rs`
  - Prove the worker/result mode remains fake deterministic or disabled and cannot publish live side effects.

### Smoke / docs

- Modify `scripts/smoke_data_quality_hygiene_local_loop.sh`
  - Extend current marker checks only after the API/Postgres path exists.
  - Required markers should include context, draft validation, blocked side-effect rejection, outcome, row-count/readiness/metrics proof, and `live_side_effects_allowed=false`.

- Modify `docs/ops/data-quality-hygiene-local-smoke.md`
  - Update expected output and troubleshooting for the durable local/demo slice.

- This plan artifact is `docs/plans/2026-06-25-data-quality-hygiene-demo-vertical-slice.md`.

---

## Bite-sized implementation tasks

### Task 1: Add failing API contract tests for the current demo packet and disabled side-effect posture

**Objective:** Lock the existing route contract before persistence changes.

**Files:**
- Modify: `apps/api/tests/data_quality_hygiene_agent_contract.rs`
- Read: `apps/api/src/http.rs`

**Steps:**
1. Add tests for context, draft validation, side-effect rejection, outcome capture, and summary route.
2. Run: `cargo test -p pet-resort-api --test data_quality_hygiene_agent_contract -- --nocapture`
3. Expected before implementation: failures only for newly asserted durable/correlation fields not present yet; existing disabled-side-effect behavior should still pass.

### Task 2: Introduce semantic repository methods for Data-Quality Hygiene

**Objective:** Make the API handler describe workflow/review/audit/outcome persistence in domain terms instead of pushing raw vectors directly.

**Files:**
- Modify: `apps/api/src/http.rs`

**Steps:**
1. Extend `WorkflowRepository` with Data-Quality Hygiene-specific methods.
2. Update `VaccineDocumentStore` in-memory implementation to satisfy the new trait without changing API behavior.
3. Run: `cargo test -p pet-resort-api --test data_quality_hygiene_agent_contract -- --nocapture`
4. Expected: current behavior remains deterministic; new tests should progress to persistence/correlation gaps.

### Task 3: Add request/correlation evidence to Data-Quality Hygiene route payloads

**Objective:** Prove traceability from HTTP request to workflow/action/outcome without logging payload bodies.

**Files:**
- Modify: `apps/api/src/http.rs`
- Modify: `apps/api/tests/data_quality_hygiene_agent_contract.rs`

**Steps:**
1. Ensure `RequestTraceEvidence` is included in context/draft/outcome audit payloads.
2. Add or propagate a `correlation_id` through context packet, draft submission, outcome capture, summary query, and audit metadata.
3. Keep `payload_logging="disabled"` in route spans.
4. Run: `cargo test -p pet-resort-api --test data_quality_hygiene_agent_contract -- --nocapture`
5. Expected: API contract tests prove request/correlation evidence appears and raw provider payloads do not.

### Task 4: Add durable Postgres adapter for this one workflow

**Objective:** Persist the Data-Quality Hygiene processing lineage to the existing MVP tables.

**Files:**
- Modify: `apps/api/src/http.rs` or create a local module such as `apps/api/src/workflow_repository.rs` if the file is too large.
- Modify: `apps/api/Cargo.toml` only if a test/adapter dependency is missing.
- Do not modify migration unless a test proves the existing schema cannot represent the slice.

**Steps:**
1. Implement inserts for `workflow_events`, `review_packets`, `audit_events`, `workflow_results`, `approval_records`, `data_quality_hygiene_outcomes`, and optional approved internal `outbox_records` candidate.
2. Use the existing `data_quality_hygiene_outcomes.workflow_event_id`, `approval_record_id`, `source_refs`, `issue_refs`, `correlation_id`, and labor-minute fields.
3. Keep outbox topic internal/local, e.g. `internal.data_quality_hygiene.reviewed_handoff`, and only insert it after a matching approved approval record.
4. Do not create topics for customer messages, provider writes, payments/refunds/discounts, schedule changes, or profile merges/deletes.
5. Run focused API/storage tests.

### Task 5: Add integration test for API -> Postgres -> fake/disabled result -> metrics proof

**Objective:** Give downstream reviewers one command that proves the local vertical slice.

**Files:**
- Create: `apps/api/tests/data_quality_hygiene_postgres_vertical_slice.rs`
- Modify: `scripts/smoke_data_quality_hygiene_local_loop.sh`
- Modify: `docs/ops/data-quality-hygiene-local-smoke.md`

**Steps:**
1. Test the full sequence through public routes where practical: context, draft, outcome, summary, readiness/metrics.
2. Assert row counts and key fields in the durable tables.
3. Assert readiness/metrics still report disabled live side effects and aggregate-only metrics.
4. Extend the smoke script to print markers such as `postgres_rows_ok`, `fake_worker_result_ok`, `readiness_ok`, and `metrics_ok`.
5. Run: `./scripts/smoke_data_quality_hygiene_local_loop.sh`
6. Expected: smoke passes on local fake data only and prints positive estimated/actual labor minutes saved.

### Task 6: Refresh migration/storage contract tests only if schema changes

**Objective:** Keep schema proof aligned without overbuilding.

**Files:**
- Maybe modify: `migrations/0001_mvp_foundation.sql`
- Maybe modify: `storage/tests/mvp_migration_contract.rs`
- Maybe modify: `storage/tests/data_quality_hygiene_outcome_storage.rs`

**Steps:**
1. If no schema change is needed, leave migration untouched.
2. If a schema change is needed, add the smallest invariant and matching contract assertion.
3. Run: `cargo test -p storage --test mvp_migration_contract -- --nocapture`
4. Run: `cargo test -p storage --test data_quality_hygiene_outcome_storage -- --nocapture`

### Task 7: Final readiness and presentation proof

**Objective:** Produce the acceptance evidence for the presentation demo slice.

**Files:**
- Modify: `docs/ops/data-quality-hygiene-local-smoke.md`
- Maybe modify: `docs/presentation/job-presentation-walkthrough.md` only to point at the new proof after it exists.

**Steps:**
1. Run focused route/storage/worker tests.
2. Run smoke script.
3. Run docs check if docs changed.
4. Capture command outputs in the downstream task handoff.

---

## Acceptance checks and smoke commands

Primary local smoke:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
```

Focused tests to run as the implementation grows:

```sh
cargo test -p app --test data_quality_hygiene_workflow_contracts -- --nocapture
cargo test -p storage --test data_quality_hygiene_outcome_storage -- --nocapture
cargo test -p storage --test mvp_migration_contract -- --nocapture
cargo test -p pet-resort-api --test data_quality_hygiene_agent_contract -- --nocapture
cargo test -p pet-resort-api --test data_quality_hygiene_postgres_vertical_slice -- --nocapture
cargo test -p pet-resort-worker --test runtime_mode_contract -- --nocapture
```

Runtime proof commands after implementation:

```sh
# Terminal 1
RUST_LOG=pet_resort_api=info,tower_http=info cargo run -p pet-resort-api

# Terminal 2
curl -sS http://127.0.0.1:3000/readyz | python -m json.tool
curl -sS 'http://127.0.0.1:3000/agent/context/data-quality-hygiene?location_id=00000000-0000-0000-0000-000000000001&operating_day=2026-06-25' | python -m json.tool
curl -sS http://127.0.0.1:3000/ops/metrics/summary | python -m json.tool
```

Docs verification after docs edits:

```sh
./scripts/check_docs.sh
```

Acceptance criteria for the implemented slice:

- API routes expose product-owned Data-Quality Hygiene DTOs and never pass raw provider DTOs through as API contracts.
- Context creation records source refs, issue refs, review gates, blocked actions, labor estimate, request id, and correlation id.
- Draft validation accepts internal cleanup work and rejects customer sends, provider/PMS writes, payment/refund/discount movement, schedule changes, destructive merge/delete, sensitive payload exposure, and ambiguity hiding.
- Durable rows exist for `workflow_events`, `workflow_results`, `review_packets`, `approval_records`, `audit_events`, `data_quality_hygiene_outcomes`, and optional approved internal `outbox_records` candidate.
- Outbox remains an approved handoff candidate only; no live adapter publishes or sends.
- Worker/runtime proof is fake deterministic or disabled and side-effect stubbed.
- Readiness and metrics proof states live side effects, customer messaging, and provider writes are disabled.
- Smoke output shows positive estimated and actual labor minutes saved from reviewed local/fake data.

---

## Non-goals / explicit boundaries

- No live NVA/Gingr/provider credentials.
- No production data.
- No provider/PMS/POS writes.
- No live customer email, SMS, portal, or review-request sends.
- No payment, refund, discount, deposit, invoice, or checkout money movement.
- No staff schedule, reservation, capacity, vaccine acceptance, incident closure, eligibility, profile merge/delete, or source-system repair side effects.
- No hidden source cleanup or automatic ambiguity resolution.
- No Toasty migration now. If storage boilerplate becomes painful, add a separate future storage-only Toasty spike; do not add Toasty types to domain/app/API for this slice.
- No production-readiness claim. This slice is local/demo/presentation proof of the contract shape and safety posture.
- No broad API schema/OpenAPI publishing unless separately scoped after this vertical slice is green.
