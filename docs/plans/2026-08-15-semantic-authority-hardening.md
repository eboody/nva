# Semantic Authority Hardening Implementation Plan

> **For Hermes:** Use subagent-driven-development and strict RED-GREEN-REFACTOR to implement this plan task-by-task. The parent session owns integration, formatting, commits, and final verification.

**Goal:** Make the pre-data NVA model a beautiful, tightly integrated, expressive enterprise foundation in which observed evidence cannot masquerade as accepted truth or executable authority, validated states rehydrate safely, identities remain exact, and domain/API/storage/SQL contracts do not drift or lose meaning.

**Architecture:** Preserve the explicit uncertainty gradient `Observed -> Candidate -> Accepted` while keeping executable `Authority<Action, Subject, Scope>` opaque and separately issued. Historical evidence remains serializable; authority is non-serializable, subject/target/gate/scope-bound, and issued only from trusted application capability. Boundary records deserialize through raw forms and promote fallibly; storage round trips are exhaustive and lossless; published API and SQL contracts have one semantic owner plus parity tests.

**Stage constraint:** Real provider data/schema is not yet available. Do not invent provider cardinalities, service tables, identity mappings, or business policy. Harden universal software invariants and leave unknown source relationships explicit.

**Tech stack:** Rust 2024 workspace, Serde, bon/nutype/statum where semantically appropriate, trybuild compile-fail tests, PostgreSQL migrations/contracts, checked OpenAPI JSON, Cargo workspace verification.

---

## Global acceptance contract

1. No serializable or caller-constructible evidence directly mints executable authority.
2. Every type named or semantically equivalent to accepted/approved/authorized/verified/eligible/reviewed/confirmed/completed/current/trusted either revalidates on deserialization or is intentionally non-deserializable.
3. Consent and other authorization-bearing evidence bind exact subject, purpose, channel, source/version, and effective lifecycle required by the current model without inventing provider facts.
4. Semantic domain-storage-domain round trips preserve every known variant or fail explicitly.
5. Runtime DTO and OpenAPI field names, requiredness, nullability, enums, and nested shapes are structurally equivalent.
6. Rust stable codes and SQL CHECK values are parity-tested.
7. Nil/sentinel IDs do not cross production constructors or substitute for subject mismatch.
8. Authorization rehydration rejects duplicate/conflicting identity-role authority and unsupported schema versions.
9. Full workspace tests, strict debug/release Clippy, formatting, Rustdoc, migration/live SQL gates available locally, independent reviews, diff/secret scan, and exact-SHA risk review pass.
10. Real-data unknowns remain explicit hypotheses/candidates, not guessed constraints.

---

### Task 1: Build the authority and validated-rehydration regression harness

**Objective:** Establish failing tests that enumerate every current manufacture site before production changes.

**Files:**
- Create: `domain/tests/authority_provenance_contracts.rs`
- Create: `domain/tests/validated_rehydration_contracts.rs`
- Modify: `domain/tests/ui.rs`
- Create UI fixtures under: `domain/tests/ui/`
- Inspect all domain/storage/app types matching accepted, approved, authorized, verified, eligible, reviewed, confirmed, completed, current, trusted.

**RED:** Add tests proving the current defects:
- candidate/reviewer-less structured note cannot deserialize as accepted;
- customer-scoped permission rejects consent for a different customer;
- customer/agent/system historical approval cannot mint customer-message queue authority without reviewer authority;
- high/critical incident cannot close with only a required-gate label;
- contradictory play eligibility/review combinations cannot deserialize;
- ambiguous identity requires at least two unique candidates;
- queue authority cannot be constructed, cloned, serialized, reused, or transferred.

Run each exact test target and record the expected semantic failure.

**GREEN:** No production code in this task. The task is complete when tests fail for the intended missing contracts rather than compile/setup errors.

---

### Task 2: Make accepted intelligence opaque at persistence boundaries

**Objective:** Ensure `AcceptedNote` cannot exist unless acceptance invariants and reviewer authority hold, and cannot be cloned or rehydrated from historical fields.

**Files:**
- Modify: `domain/src/customer.rs`
- Test: `domain/tests/authority_provenance_contracts.rs`
- Test: `domain/tests/validated_rehydration_contracts.rs`
- Update relevant Rustdocs.

**RED:** Run the accepted-note adversarial tests from Task 1.

**GREEN:** Keep `StructuredNote` as serializable historical observation, but require opaque one-use `NoteAcceptanceAuthority` for promotion. Keep `AcceptedNote` and `SegmentMembership` non-cloneable and non-serializable so persisted fields cannot recreate accepted state.

**Verification:** Focused domain tests, then `cargo test -p domain --locked`.

---

### Task 3: Bind consent to the exact customer and evidence lifecycle

**Objective:** Make customer-specific permissions impossible to issue from transferable channel/purpose flags.

**Files:**
- Modify: `domain/src/consent.rs`
- Modify: `domain/src/customer.rs`
- Modify: `domain/src/lead.rs` only where existing transactional consent consumers require subject binding.
- Modify tests/fixtures under `domain/tests/` and affected app tests.
- Update Rustdocs.

**RED:** Add/execute mismatch, revoked/superseded/effective-time, and correct-subject tests.

**GREEN:** Introduce semantic consent subject/source identity and checked permission issuance without guessing undocumented provider schema. Bind the current evidence to `CustomerId`, purpose, channel, status, and explicit source reference/version supported by existing data. Preserve `Observed`/candidate posture where confirmation is unavailable.

**Verification:** Domain plus app focused tests.

---

### Task 4: Separate approval history from trusted reviewer authority

**Objective:** Prevent historical approval records from minting queue authority without a trusted, target-scoped reviewer capability.

**Files:**
- Modify: `domain/src/entities.rs` approval and `message_record` modules, splitting modules if needed for truthful ownership.
- Modify: `app/src/` authority issuance owner and call sites.
- Modify: domain/app compile-fail and runtime tests.
- Preserve storage outbox exact-binding semantics.
- Update Rustdocs for every public authority/evidence surface.

**RED:** Prove unauthorized actors and caller-built approval history cannot call the issuance API; prove matching trusted reviewer authority can issue exactly one target-bound queue capability.

**GREEN:** Keep serializable approval evidence historical. Require opaque non-serializable reviewer authority bound to gate, target, scope, and deciding actor before issuing `QueueAuthorization`. Do not encode speculative NVA role policy in the domain; application authorization owns role/location policy and capability issuance.

**Verification:** Compile-fail tests, domain/app focused suites, and storage outbox regressions.

---

### Task 5: Make review-sensitive lifecycle transitions consume evidence

**Objective:** Replace gate-label-as-proof patterns and contradictory policy product states.

**Files:**
- Modify: `domain/src/entities.rs` incident lifecycle.
- Modify: `domain/src/policy.rs` play eligibility decision.
- Modify: `domain/src/identity.rs` ambiguity cardinality.
- Modify affected app/storage projections and tests.

**RED:** Use Task 1 tests for incident closure, contradictory eligibility, and malformed ambiguity.

**GREEN:**
- Model incident closure as a checked transition requiring exact closure evidence/authority while allowing observed provider status to remain quarantined as source evidence.
- Replace `Eligibility + Option<ReviewGate>` product with state-shaped variants.
- Use a minimum-two, unique candidate collection for ambiguity and custom serde.

**Verification:** Domain focused and full crate suite.

---

### Task 6: Eliminate sentinel identity fallbacks

**Objective:** Ensure subject mismatch and malformed external IDs fail explicitly rather than becoming nil IDs.

**Files:**
- Modify domain ID ownership/constructors where production construction remains public.
- Modify: `app/src/daily_update.rs`
- Modify: `apps/api/src/http.rs` malformed location authorization path.
- Update tests to use deterministic non-nil fixture IDs except explicit boundary rejection tests.

**RED:** Add tests for non-reservation daily-update subjects, malformed API location IDs, and nil ID promotion.

**GREEN:** Return typed errors or preserve explicit unknown external identity; do not use `Uuid::nil()` as fallback. Keep fixture ergonomics through test-only helpers, not production constructors.

**Verification:** Domain/app/API focused suites.

---

### Task 7: Make provenance codecs checked and service-policy round trips lossless

**Objective:** Ensure typed storage records cannot contain malformed provenance and every known service-policy variant survives persistence.

**Files:**
- Modify: `storage/src/operations.rs`
- Modify: `storage/src/persistence.rs`
- Modify: `storage/src/service_line/grooming.rs`
- Add/modify tests under `storage/tests/`.
- Version service contract records if required to avoid direct durable dependence on unstable domain serde.

**RED:** Add malformed source system/record/timestamp/adapter version JSON tests and all-variant grooming cadence round trips, including `AsNeeded`, `GroomerRecommended`, and `Unknown`.

**GREEN:** Deserialize raw storage fields through checked promotion. Add a cadence-kind representation instead of overloading absent weeks. Use exhaustive conversions and explicit unknown-version errors.

**Verification:** `cargo test -p storage --locked` and all affected domain/storage contract tests.

---

### Task 8: Establish Rust-SQL stable-code and relationship parity

**Objective:** Prevent Rust-valid records from failing SQL and prove approval applicability for outcome evidence where current schema claims review lineage.

**Files:**
- Modify: `migrations/0001_mvp_foundation.sql` or add the next forward migration according to repository migration policy.
- Modify: `storage/tests/mvp_migration_contract.rs`
- Create parity tests/scripts under `storage/tests/` or existing migration contract owner.
- Modify outcome relationship schema only to encode known workflow identity, not speculative provider relationships.

**RED:** Prove `ReviewCapacityLaborRecommendation` Rust/SQL drift and unrelated approval/outcome acceptance.

**GREEN:** Create one canonical stable-code parity gate. Bind outcome approval evidence to the exact known workflow/review/action relationship. Explicitly model attempt/history semantics where multiple payment/workflow results are currently legal; do not force one-to-one if retries are conceptually valid.

**Verification:** Source contract tests plus migration execution against a fresh local PostgreSQL instance when available; each expected rejection uses savepoints/separate transactions.

---

### Task 9: Make OpenAPI and runtime DTO ownership structurally identical

**Objective:** Remove duplicate boundary semantics and guarantee generated clients match runtime serde.

**Files:**
- Modify: `apps/api/src/public_contract.rs`
- Modify/generate: `apps/api/openapi/owned-operations-v1.openapi.json`
- Modify: `apps/api/tests/owned_api_openapi_contract.rs`
- Modify Manager Daily Brief route DTO ownership in `apps/api/src/http.rs` where needed.

**RED:** Add structural parity tests for field names, requiredness, nullability, enums, nested refs, unknown-field behavior, idempotency key, actor identity, source references, and route inventory.

**GREEN:** Select one canonical owner and generate/map the other exhaustively. Runtime must require exactly what the contract requires; no undocumented serde defaults.

**Verification:** API focused suite and schema validation tests.

---

### Task 10: Harden authorization rehydration cardinality and version checks

**Objective:** Remove iteration-order authority from SpacetimeDB identity/role promotion and enforce persisted compatibility.

**Files:**
- Modify: `apps/spacetimedb/src/tables.rs`
- Modify: `apps/spacetimedb/src/authz.rs`
- Modify: `apps/spacetimedb/src/storage/review_queue/codec.rs`
- Modify: `apps/spacetimedb/src/realtime_queue_tests.rs` and authorization tests.

**RED:** Add duplicate identity, conflicting role, malformed scope, and unsupported schema-version tests.

**GREEN:** Enforce or fallibly validate identity uniqueness; represent multi-role semantics explicitly or reject conflicts; do not take first match; reject malformed scope and unsupported authority schema versions.

**Verification:** `cargo test -p pet-resort-spacetimedb --locked` using the actual package name from the workspace.

---

### Task 11: Make current in-memory outcome recording idempotent

**Objective:** Prevent retries from duplicating Manager Daily Brief outcomes and labor-value claims.

**Files:**
- Modify: `apps/api/src/http.rs`
- Modify: `storage/src/workflow_repository.rs`
- Modify affected public contract/storage tests.

**RED:** Add same-key/same-payload replay and same-key/different-payload conflict tests.

**GREEN:** Use semantic operation fingerprinting and atomic replay/conflict/record behavior matching the existing Data Quality pattern without copy-paste ownership.

**Verification:** API/storage focused tests.

---

### Task 12: Add generalized architecture prevention and uncertainty documentation

**Objective:** Make recurrence mechanically difficult and explain the pre-data semantic posture from code-derived truth.

**Files:**
- Add architecture tests under `domain/tests/`, `storage/tests/`, and `apps/api/tests/`.
- Update `docs/architecture/semantic-domain-contract-atlas.md` and ownership ADR.
- Update Rustdocs on all changed public surfaces.
- Add a concise observed/candidate/accepted/authority doctrine to the canonical architecture documentation.

**Guards:**
- compile-fail capability construction/clone/serde/reuse/transfer;
- checked serde fixtures for validated wrappers;
- exhaustive semantic round-trip matrix;
- OpenAPI/runtime parity;
- Rust/SQL stable-code parity;
- no sentinel ID fallback in production code;
- documented classification of universal invariant, hypothesis, observation, migration assumption, owned policy, and unknown.

---

### Task 13: Independent adversarial review and release evidence ladder

**Objective:** Prove the completion bar against the integrated diff.

**Commands/gates:**
1. `cargo fmt --all -- --check`
2. Focused semantic, authority, codec, schema, migration, and compile-fail tests
3. `cargo test --workspace --locked`
4. `cargo clippy --workspace --all-targets --locked -- -D warnings`
5. `cargo clippy --workspace --all-targets --release --locked -- -D warnings`
6. Strict workspace Rustdoc according to repository CI policy
7. Dependency advisory/license gates available in the repository
8. Fresh-database migration and live SQL rejection tests when local prerequisites exist
9. Independent specification review
10. Independent secure cross-layer review
11. Independent code-quality/semantic-doctrine review
12. Final diff, generated-artifact classification, secret/privacy scan, and `git status`
13. Repowise change-risk assessment and exact impacted-test review
14. Commit only after all local blockers are zero; push/remote CI only if requested and credentials are available

**Terminal gate:** No unresolved security or logic blockers in the scoped pre-data abstraction framework. Deferred production identity, worker execution, authorized realtime views, live providers, and unknown provider-data relationships remain explicitly labeled rather than falsely claimed complete.
