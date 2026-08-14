# Enterprise Semantic Integration Remediation Implementation Plan

> **For Hermes:** Use subagent-driven-development to implement this plan one semantic boundary at a time, with RED/GREEN evidence and spec review before code-quality review.

**Goal:** Remove the eight confirmed production-readiness gaps while making the API, provenance, reservation, capacity, approval/outbox, telemetry, browser, and CI models tightly integrated and type-driven.

**Architecture:** Boundary wire records remain explicit and strict, then promote into semantic values before orchestration. Canonical domain identifiers retain ownership across layers. Contradictory source evidence is represented as a typed anomaly instead of collapsed into an ordinary state. Executable outbox work requires validated internal topics plus opaque one-shot authority derived from matching approval evidence; SQL independently rechecks the persisted relationship. Production readiness is represented as typed configuration evidence, not optimistic booleans.

**Tech stack:** Rust 2024, Serde, Axum, Statum where legal method availability changes, PostgreSQL migrations, tracing/OpenTelemetry-compatible export, Prometheus/OpenMetrics, Next.js, Playwright, GitHub Actions, cargo-audit, cargo-deny.

---

## Non-negotiable invariants

1. `public_contract.rs` owns public request DTOs; HTTP handlers do not define parallel versions.
2. Unknown public request fields fail closed with `serde(deny_unknown_fields)`.
3. Required provenance is all-or-nothing and never synthesized by an adapter.
4. Canonical reservation identity remains `domain::entities::reservation::Id` through booking triage.
5. `occupied > total` is not ordinary zero availability; it yields typed reconciliation evidence and never yields confirmation authority.
6. Approval evidence is not executable authority. Only an opaque, non-Serde, one-shot capability created after role, gate, target, topic, and status validation may construct a pending outbox row.
7. PostgreSQL independently enforces approved status, matching gate/target, and the internal-only topic class. Existing matching trigger logic is retained and tested rather than duplicated.
8. Production readiness claims are derived from validated telemetry configuration and exporter/metrics health.
9. Browser tests exercise rendered behavior, not source text.
10. Remote CI evidence applies to the exact pushed remediation commit.

## Task 1: Establish red contract tests for the canonical API request

**Files**
- Modify: `apps/api/tests/public_contract_identity_regression.rs`
- Modify: `apps/api/tests/data_quality_hygiene_outcome_capture_contract.rs`
- Modify: `apps/api/tests/owned_api_openapi_contract.rs`

**RED**
1. Add a compile-time construction test proving `public_contract::DataQualityHygieneOutcomeCaptureRequest` uses closed outcome/persona/resolution enums and contains `actor_role` and a semantic idempotency key.
2. Add HTTP tests proving an unknown field returns `422`, omitted required `source_refs`, `issue_refs`, and `requested_side_effects` fail deserialization, and documented `idempotency_key` is consumed.
3. Add an OpenAPI parity test comparing the required fields and enum values to the Rust-owned request contract.
4. Run targeted tests and verify expected failures.

**GREEN**
1. Rehome the storage-owned public enums or define API-owned closed enums in `public_contract.rs` with explicit exhaustive conversions into storage values.
2. Add `#[serde(deny_unknown_fields)]` to the public request and nested public actor/audit records.
3. Remove `#[serde(default)]` from arrays required by OpenAPI.
4. Delete the private parallel request/actor/audit structs from `http.rs`; deserialize the public contract directly.
5. Validate `actor_role` against authenticated role and feed `idempotency_key` into the outcome repository path.
6. Run targeted API tests, OpenAPI tests, and strict API clippy.

## Task 2: Make outcome idempotency a semantic repository contract

**Files**
- Modify: `apps/api/src/public_contract.rs`
- Modify: `apps/api/src/http.rs`
- Modify: `storage/src/operations.rs`
- Modify/add tests under `apps/api/tests/` and `storage/tests/`

**RED**
1. Add tests for identical replay returning the existing record without incrementing persistence counts.
2. Add tests for a reused key with changed payload returning conflict.
3. Add tests for missing/blank/oversized keys failing promotion.

**GREEN**
1. Add a redacted validated `IdempotencyKey` newtype at the API boundary.
2. Add a stable request fingerprint covering every consequential field.
3. Store `(key, fingerprint, result)` atomically in the local outcome repository abstraction.
4. Return explicit `Created`, `IdempotentReplay`, or `PayloadDrift` outcomes.
5. Never log the raw key.

## Task 3: Stop manufacturing inquiry provenance and complete replay comparison

**Files**
- Modify: `apps/api/src/http.rs`
- Modify: `apps/api/tests/api_dto_contracts.rs`
- Modify: `apps/api/tests/health_contract.rs`

**RED**
1. Add tests proving partial provenance tuples are rejected, no provenance fields produce `None`, and a complete tuple is preserved byte-for-byte semantically.
2. Add replay-drift tests for `source_system`, `provider_model_path`, `raw_payload_ref`, `received_at`, contact attempts, and simulated conversion.

**GREEN**
1. Replace the independent optional provenance fields with a private raw shape promoted into `Option<InquiryProvenance>` by an all-or-none constructor.
2. Delete `local_inquiry_fixture`, provider-model-path, and synthesized `api://` defaults.
3. Build accepted provenance only from complete supplied evidence.
4. Compare every consequential field through a canonical request fingerprint instead of hand-maintained partial equality.

## Task 4: Preserve canonical reservation identity through booking triage

**Files**
- Modify: `app/src/booking_triage.rs`
- Modify: booking-triage tests and UI compile tests under `app/tests/`

**RED**
1. Change wished-for tests to construct typestate requests and staff packets with `domain::entities::reservation::Id`.
2. Add a compile-time/API test proving arbitrary strings cannot become booking-triage reservation identity.
3. Verify failures against the current string wrapper.

**GREEN**
1. Delete `booking_triage::Reservation(String)`.
2. Store `reservation_entity::Id` in the Statum machine and `StaffEvaluationPacket`.
3. Make production `Service::evaluate` consume a `Request<ReadyForPolicyDecision>` or explicitly rebuild one from validated repository evidence before evaluation; no ceremonial side path.
4. Remove UUID-to-string-to-wrapper conversion and update call sites/docs.

## Task 5: Model capacity contradiction as reconciliation-required evidence

**Files**
- Modify: `domain/src/boarding/capacity.rs`
- Modify: `domain/tests/petsuites_core_service_contracts.rs`
- Modify: `domain/tests/constructor_equivalent_serde.rs`

**RED**
1. Add a test that `occupied > total` does not equal ordinary `EligibleSegmentFull`.
2. Add a test that no `Available` decision can arise from contradictory inventory.
3. Add Serde/rehydration tests proving the contradiction remains explicit.

**GREEN**
1. Introduce `CapacityState::{Available(RoomCount), Full, Contradictory(OverOccupancy)}` or an equivalent exhaustive semantic enum.
2. Add typed `OverOccupancy { total, occupied, excess }` evidence.
3. Replace `available_rooms()` with an exhaustive `capacity_state()`; keep a separately named fail-safe count accessor only if a reporting adapter needs it.
4. Add `Decision::ReconciliationRequired { anomaly, review_gate }` or a denial reason carrying the typed anomaly.
5. Make policy exhaustive over the capacity state.

## Task 6: Make approval authority opaque and outbox topics closed

**Files**
- Modify: `storage/src/operations.rs`
- Modify: `storage/src/persistence.rs`
- Modify: `storage/tests/approval_outbox_infrastructure.rs`
- Modify: `app/tests/ui/` compile-fail fixtures

**RED**
1. Add compile-fail tests proving callers cannot construct execution authority, serialize it, clone it, or create a pending outbox candidate from `ApprovalReviewDisposition` alone.
2. Add runtime tests for mismatched role, approval ID, gate, target, and non-internal topic.
3. Verify current public builder path fails the new contract.

**GREEN**
1. Replace raw `outbox_topic: String` with a closed `InternalHandoffTopic` enum or validated `persistence::Topic<InternalHandoff>`.
2. Separate persistence evidence (`ApprovalReviewDisposition`) from current authorization.
3. Introduce private-field `ApprovedInternalHandoffAuthority`, issued only by a policy function that consumes current authenticated reviewer capability plus matching decision evidence.
4. Make authority one-shot and non-Serde/non-Clone.
5. Require authority to construct `PendingOutboxRecord`; pending status and approval binding remain private.
6. Keep publisher execution unavailable.

## Task 7: Enforce the approval/outbox relationship in PostgreSQL

**Files**
- Modify: `migrations/0001_mvp_foundation.sql` only if this migration is still mutable; otherwise create the next numbered migration.
- Modify: `storage/tests/postgres_migration_contract.rs`
- Modify: `.github/workflows/ci.yml`

**RED**
1. Add database tests that direct SQL inserts fail for non-internal topics, non-approved records, and mismatched gate/target.
2. Add update tests proving approval or outbox fields cannot drift while work is pending/claimed.

**GREEN**
1. Preserve the existing approved-status/gate/target trigger, which current source already implements.
2. Add a deterministic internal-topic constraint such as `topic LIKE 'internal.%'` plus a bounded validated topic grammar.
3. Ensure trigger checks also run when `topic` or relevant approval fields change.
4. Run migration tests against PostgreSQL.

## Task 8: Add typed production telemetry composition

**Files**
- Modify: `apps/api/Cargo.toml`
- Create: `apps/api/src/observability.rs`
- Modify: `apps/api/src/lib.rs`
- Modify: `apps/api/src/main.rs`
- Modify: `apps/api/src/http.rs`
- Modify/add: `apps/api/tests/health_contract.rs`, `apps/api/tests/observability_contract.rs`

**RED**
1. Add config tests for local JSON-only mode, production OTLP mode, invalid endpoints, and missing production telemetry configuration.
2. Add readiness tests proving production is not ready unless trace export and durable metrics are configured/healthy.
3. Add `/metrics` tests for bounded labels and request counters; assert no actor IDs, tenant IDs, source payloads, or idempotency keys appear.

**GREEN**
1. Model `TelemetryMode::{LocalEphemeral, ProductionExport(ProductionTelemetry)}` with validated endpoint/service/resource values.
2. Compose tracing once in `observability.rs`; install JSON formatting plus an OTLP/OpenTelemetry layer in production.
3. Add a Prometheus/OpenMetrics recorder and `/metrics` endpoint with bounded route/method/status labels.
4. Replace hard-coded `not_configured` claims with a typed readiness snapshot reporting configured, healthy, or unavailable components.
5. Preserve payload logging as disabled/redacted and add graceful exporter shutdown.

## Task 9: Add rendered browser E2E coverage

**Files**
- Modify: `apps/staff-web/package.json`
- Modify: root `package.json`
- Create: `apps/staff-web/playwright.config.ts`
- Create: `apps/staff-web/e2e/staff-dashboard.spec.ts`
- Create: `apps/staff-web/e2e/standalone-startup.mjs` if a separate artifact harness is needed
- Modify: `package-lock.json`

**RED**
1. Add Playwright tests for desktop/mobile navigation and primary landmarks.
2. Add keyboard and accessibility checks, including no serious/critical axe violations.
3. Mock successful and failing API responses and assert fail-safe user-visible behavior.
4. Add responsive overflow checks at phone/tablet/desktop widths.
5. Add a test that builds and boots `.next/standalone` and serves the page.
6. Run before configuration and confirm failure.

**GREEN**
1. Add `@playwright/test` and `@axe-core/playwright` as dev dependencies; keep runtime Playwright out of production dependencies.
2. Configure deterministic `webServer`, retries, traces/screenshots on failure, and Chromium installation.
3. Add stable accessible selectors only where semantic roles/names are insufficient.
4. Add `test:e2e` and `test:standalone` scripts.
5. Run all browser tests locally.

## Task 10: Harden reproducible CI and security policy

**Files**
- Modify: `.github/workflows/ci.yml`
- Create: `deny.toml`
- Create: `.github/dependabot.yml`
- Modify: `rust-toolchain.toml`
- Modify: `Cargo.lock`
- Modify: `apps/staff-web/package.json`, `package-lock.json`

**RED**
1. Add/update repository quality-contract tests to require locked/all-target Rust tests, debug and release strict clippy, Rustdoc, cargo audit/deny, clean npm install, typecheck/lint/unit/E2E/build/standalone, migration tests, explicit permissions, concurrency, and timeouts.
2. Run and observe current workflow contract failure.

**GREEN**
1. Pin a supported Rust release.
2. Update `anyhow` to a RustSec-safe release.
3. Add reviewed cargo-deny advisory/license/source/bans policy.
4. Upgrade Next.js and matching eslint config to the non-vulnerable stable release supported by the app; regenerate lockfile and rerun production audit.
5. Use `npm ci --include=dev` with `NODE_ENV=development` for frontend gates.
6. Add Playwright browser installation/cache strategy and all frontend gates.
7. Add least-privilege `permissions: contents: read`, concurrency cancellation, and job timeouts.
8. Run full local CI-equivalent gates.

## Task 11: Obtain exact-commit CI evidence

**Files**
- No production code beyond reviewed remediation.

**Steps**
1. Run `cargo fmt`, debug/release clippy, locked all-target tests, docs, migration tests, cargo-audit/deny, frontend clean install/typecheck/lint/unit/E2E/build/standalone/audit, and workspace quality scripts.
2. Run RepoWise change-risk analysis and resolve high-risk findings.
3. Dispatch independent spec-compliance, secure-cross-layer, semantic-model, and code-quality reviews; fix and re-review until approved.
4. Confirm only intended files changed and `.vscode/` remains untouched.
5. Commit the complete verified remediation with an auditable message.
6. Push the current branch using the repository's 1Password-backed GitHub SSH workflow.
7. Observe GitHub Actions for the exact commit; fix failures and repeat until green.
8. Report the commit SHA and workflow URL. Do not claim remote attestation before the run completes successfully.

## Acceptance gate

The remediation is complete only when:

- all targeted RED tests were observed failing before implementation;
- all narrow and full local gates pass;
- direct source and SQL tests prove the eight invariant classes;
- independent spec and quality reviewers approve;
- cargo-audit/cargo-deny and production npm audit pass under checked-in policy;
- Playwright tests pass against development and standalone production servers;
- GitHub Actions is green for the exact remediation commit;
- live publisher/provider/payment/customer-message side effects remain disabled.
