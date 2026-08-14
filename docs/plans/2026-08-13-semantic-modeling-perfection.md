# NVA Semantic Modeling Perfection Implementation Plan

> **For Hermes:** Execute this plan through the `nva-semantic-modeling-perfection` Kanban board using strict RED-GREEN-REFACTOR, one mutating worker at a time in `/home/eran/code/nva`, with independent review and a single final verified commit.

**Goal:** Close every material modeling weakness identified by the 2026-08-13 domain, workflow/typestate, and cross-layer audits so ordinary Rust construction, rehydration, authorization, storage, API, and provider paths preserve the same domain truths.

**Architecture:** Domain aggregates become closed values constructed only through invariant-preserving APIs. Runtime identities and review facts promote into typed evidence or capabilities before mutation. Application crates own use-case ports; HTTP, storage, SpacetimeDB, and provider integrations remain explicit adapters with versioned, fallible conversions. Typestate is used only when legal method availability changes by phase; persisted outcomes use evidence-bearing enums.

**Tech stack:** Rust 2024, `bon`, `nutype`, `nonempty`, Serde, Statum where justified, Axum, SpacetimeDB, Postgres/storage adapters, trybuild, Cargo/Clippy/Rustdoc, and Modum as a non-authoritative semantic discovery tool.

---

## Binding semantic proof chain

```text
untrusted request/provider/storage representation
→ authenticated or source-attributed context
→ validated semantic values
→ relationship-checked aggregate
→ explicit policy/review decision
→ target-bound evidence or capability
→ legal application transition
→ versioned adapter projection
→ evidence-bearing outcome
```

## Non-negotiable rules

- Write a failing semantic test before every behavioral or invariant change and record RED/GREEN commands.
- A valid deserialized value and a valid ordinarily constructed Rust value must have identical invariants.
- Request-body actor identifiers are claims, never authorization.
- Evidence quality, workflow completion, review approval, and executable authority are distinct types.
- Authority-bearing capabilities are opaque, target-bound, non-Serde unless a boundary representation is explicitly required, and consumed where one-shot use is intended.
- Aggregate fields are private when arbitrary field products can violate relationships.
- Builders returning an infallible value are allowed only when every field combination is legal.
- Provider and storage DTOs remain untrusted and convert explicitly into domain/app values.
- No live customer messages, provider/PMS/CRM writes, scheduling/capacity mutation, payment/refund/discount action, deployment, release, merge, or push.
- Preserve NVA's deliberate latest-tracking policy for `statum`.
- Do not chase Modum counts mechanically; classify every remaining diagnostic by semantic value.

## Work program

### 1. Regression contract and construction-surface inventory

Create executable tests proving the currently known bypasses before changing production code. Cover direct/generated construction, Serde, lifecycle evidence, review authority, repeated decisions, API authorization, stable DTO identity, lossy realtime encoding, and provenance attachment. Save a concise machine-readable/current audit manifest so final closeout can prove each finding resolved or intentionally superseded.

### 2. Authenticated mutating HTTP boundary

Introduce an app-owned authenticated actor context and authorization seam. Mutating routes must reject absent/invalid authentication, resolve actor role/location from trusted runtime context, and invoke app-owned authorization. Body actor IDs may be validated against that context but cannot grant authority. Keep deterministic local proof possible without pretending a production identity provider exists.

### 3. Vaccine review decision protocol

Replace `approved: bool` with a typed approve/reject decision carrying required evidence. Validate source state, make repeated/conflicting decisions typed conflicts, remove panic-prone lookups, and update review packet/document/vaccine/eligibility/approval projections atomically through an app-owned transition service.

### 4. Approval/outbox authority model

Replace `completed: bool` with `ReviewDisposition` or equivalent evidence-bearing variants. Only actual approval evidence may create approved review rows and outbox candidates. Pending/rejected paths must not manufacture authority. Make approval row decision fields structurally coherent.

### 5. Separate finance attribution from approval

Remove the mapping from `can_support_value_claim` to workflow approval/completion. Represent value attribution evidence, workflow completion, and manager approval independently. Add mismatch tests and explicit projections.

### 6. Close Reservation construction

Make reservation aggregate fields private, introduce semantic stay interval and non-empty/unique pet party as justified, expose checked constructors/accessors, and ensure builders and Serde use the same invariant function. Add compile-fail contracts for struct-literal bypass if stable enough.

### 7. Close Message construction and lifecycle

Replace the direction/status/optional-gate Cartesian product with an evidence-bearing lifecycle or phase-specific construction surface. Ensure outbound queue/delivery states require matching approval evidence and inbound states cannot carry outbound evidence. Separate historical approval evidence from executable queue capability.

### 8. Close WorkflowEvent and WorkflowOutcome construction

Make event type/subject agreement universal. Replace generic workflow result's independent status/reason/output fields with an outcome enum carrying exactly the evidence legal for each variant. Preserve stable boundary conversion explicitly.

### 9. Close Document, VaccineRecord, Incident, ApprovalRecord, and StaffTask aggregates

Add private fields and checked construction/rehydration where relationships exist. Encode task completion evidence in the completed state, vaccine temporal/review coherence, document scan/redaction/review coherence, incident review requirements, and approval decision time/target/gate agreement.

### 10. Complete constructor-equivalent Serde coverage

Audit every invariant-bearing domain value that derives `Deserialize`. Fix at least reporting periods, local operating windows, and training outcome claims, plus any equivalent findings. Raw DTO/row types may derive freely; promoted domain values may not bypass validation.

### 11. Cardinality, identity, and typed rehydration errors

Use `NonEmpty`, sets, or named collection wrappers where emptiness/duplicates/order are lies. Close public UUID tuple constructors where nil or arbitrary construction is invalid. Replace `&'static str` rehydration errors with module-local typed errors.

### 12. Canonical public API contract adoption

Make handlers consume and return `apps/api::public_contract` DTOs directly. Remove duplicate metadata/DTO definitions, replace known-schema `Value` fields with typed/versioned DTOs, and protect serialized shapes with round-trip and snapshot tests. Replace boolean protocol metadata with semantic modes.

### 13. Move repository authority out of the API shell

Move workflow/outcome/read-model ports into owning `app` modules. Implement storage adapters and keep `apps/api` responsible only for authentication, DTO conversion, routing, and dependency wiring. Remove direct SQL/storage-record vocabulary from handlers where an app port exists.

### 14. SpacetimeDB semantic transitions and codecs

Replace `requires_manager_approval: bool`, debug-formatted IDs/outcomes, and comma-joined references with stable typed/versioned representations. Add legal source-state checks for claim, staff disposition, manager outcome, recommendation attachment, and outcome capture. Exhaustively round-trip every supported review gate/status/source/actor variant.

### 15. Atomic provider promotion with provenance

Return or require a source-backed promotion value that binds candidate, record reference/provenance, and mapping version. Apply it to existing Gingr customer, pet, and retail mappings. Keep grooming/training gaps explicit until verified provider fixtures exist.

### 16. Typed persistence conversion boundary

Introduce explicit fallible conversions for money/currency, timestamps, periods, IDs, source refs, idempotency keys, workflow names/topics, and versioned JSON payloads. Keep raw SQL rows private. Ensure decode rejects invalid combinations before app/domain use.

### 17. Unified API error taxonomy

Use one public error envelope and typed conversion from authentication, authorization, validation, review conflict, idempotency, source ambiguity, unavailable dependency, and internal failures. Eliminate ad hoc handler JSON errors and preserve safe redaction.

### 18. Capability and compile-time protocol hardening

Separate serializable evidence from executable authority. Add opaque consumed capabilities where queue/outbox/payment/provider-write boundaries warrant them. Expand trybuild coverage only for real compile-time guarantees: inaccessible aggregate fields, unavailable pre-approval actions, and one-shot/phase-specific methods.

### 19. Semantic ownership and module concentration

Rehome mixed aggregate ownership from `entities.rs` and split oversized API/storage modules only where paths become more truthful. Preserve one canonical public path and avoid aliases that hide incomplete movement. Replace raw extension strings with validated labels where domain-owned.

### 20. Independent semantic/security/adoption review and remediation

Run fresh reviews for domain invariants, workflow/capabilities, HTTP auth/security, and cross-layer conversion fidelity. Fix all security or logic findings. Classify Modum diagnostics with explicit rationale rather than requiring a cosmetic zero.

### 21. Final verification and commit

Run all focused suites and:

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
python scripts/check_rustdoc_completeness.py
./scripts/check_docs.sh
modum check --format json
```

Inspect the complete diff, scan added content for secrets and sensitive Debug leaks, verify the working tree contains only intentional artifacts, obtain independent approval, and create exactly one final commit named:

```text
[verified] perfect semantic modeling across NVA boundaries
```

Do not push.

## Completion criteria

The work is complete only when:

- all 21 work-program items are implemented or a final independent reviewer proves a named item inapplicable with source evidence;
- ordinary Rust construction cannot bypass the aggregate invariants protected at Serde boundaries;
- mutating HTTP authorization is fail-closed and body identities cannot grant authority;
- review approval cannot be manufactured from completion/value booleans;
- stable API/storage/realtime/provider conversions are explicit, versioned where needed, and round-trip tested;
- dangerous actions require typed, target-bound authority;
- full gates pass;
- the repository is committed once and clean afterward.
