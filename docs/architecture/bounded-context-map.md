# Bounded-context ownership map

This map is the short architecture index for current code. It assigns each kind of meaning to one owner so storage rows, provider payloads, HTTP DTOs, and runtime shells cannot become parallel domain authorities.

## Dependency direction

```text
integrations/gingr ─┐
                    ├─> domain -> app -> apps/api
storage ────────────┘                 -> apps/worker
                                      -> apps/spacetimedb
```

The diagram is an ownership summary, not permission for reverse dependencies. `domain` depends on no storage, integration, or runtime crate. `app` owns use-case contracts and ports; storage and runtime crates implement or consume those contracts. Runtime shells do not depend on one another.

The trust path remains `Observed<T> -> Candidate<T> -> Accepted<T>`. Executable authority is a separate opaque capability. A serialized label, provider role, historical approval, persisted evidence row, or accepted-looking DTO cannot cross either promotion boundary by itself.

## Context responsibilities

| Context | Concept owner | Error owner | Conversion owner | Compatibility codec owner | Repository owner | Authority issuer owner |
| --- | --- | --- | --- | --- | --- | --- |
| `domain` | Business identities, aggregates, evidence, policies, service-line rules, accepted facts | The narrow concept module whose invariant failed | Fallible source/evidence promotion into domain truth | None | Repository ports only when the contract is domain-shaped and not use-case-specific | Opaque domain capabilities; production issuers remain absent until an authenticated root can bind exact actor, subject, scope, gate, and evidence |
| `app` | Workflow packets, drafts, recommendations, review gates, blocked actions, outcomes, and use-case ports | The owning workflow or application port | Domain facts into use-case packets and reported outcomes | None as a general dependency | Application-owned repository traits such as `app::workflow_repository`; no storage rows in the port contract | Trusted application authorization boundary when one exists; otherwise no production issuer |
| `storage` | PostgreSQL rows, projections, SQL admission predicates, idempotency, audit/outbox persistence | Storage projection, codec, repository, or SQL-admission owner | Explicit row-to-domain/app promotion and domain/app-to-row projection | None; the clean-slate schema has one current row shape | Concrete PostgreSQL repositories and persistence adapters | `ApprovedInternalHandoffAuthority` only after exact current-reviewer and persisted-binding checks; storage status alone cannot issue it |
| `integrations/gingr` | Gingr endpoint grammar, transport, webhook, provider DTO, and mapping quirks | The endpoint, transport, webhook, DTO, or mapping module | `integrations/gingr/src/mapping/` promotes provider evidence fallibly into source/domain values | None; one current provider wire contract is quarantined inside the adapter | Provider client/read adapter only, never an owned-domain repository | None; a provider DTO or provider role cannot issue NVA authority |
| `apps/api` | Axum routing, authentication seam, product-owned HTTP DTOs, safe error mapping, and OpenAPI exposure | API error and route-family mapping modules | HTTP requests into app contracts and app results into canonical public DTOs | None; `/v1` is the sole product API | Runtime adapter implementations of app ports | Authentication may supply current actor evidence; it does not bypass app/domain issuance rules |
| `apps/worker` | Safe worker configuration, leasing/processing shell, telemetry, and stubbed side-effect adapters | Worker runtime boundary | App workflow evidence into explicit worker processing contracts and safe worker results | None | Runtime adapter implementations of app ports | None in the current local worker; claimed work and persisted status do not issue execution authority |
| `apps/spacetimedb` | Realtime reducer commands, private rows, subscription read models, and authorization adapter | Reducer, authorization, transition, or storage-codec owner | Explicit reducer/input and row/read-model codecs around app/domain contracts | None; `apps/spacetimedb/src/storage/review_queue/codec.rs` owns the sole current row projection | SpacetimeDB table/reducer adapter | Authenticated reducer boundary only where current identity, role, and scoped policy are checked |

## Quarantine and contract rules

1. A provider DTO is evidence owned by `integrations/gingr`; `gingr::dto` and `gingr::response` types do not appear in domain, app, storage, API, worker, or SpacetimeDB production code. Provider-crate dependency aliases are rejected outside the adapter so `extern crate` aliases, `use` aliases, and grouped imports cannot bypass quarantine. Named mapping functions perform the promotion.
2. PostgreSQL and SpacetimeDB each expose one current clean-slate row model. Active code has no historical translation namespace or alternate deployed-shape codec.
3. `apps/api` and `apps/worker` each depend on `app` contracts and never on each other. API JSON/OpenAPI and worker processing shapes are runtime adaptations, not shared shell types.
4. OpenAPI and runtime serde have one product-owned API owner and structural parity tests. Storage and provider DTOs are not reused as public API payloads.
5. Sensitive diagnostics remain redacted. A new context does not gain `Debug`, `Display`, or telemetry permission for raw customer, care, medical, message, payment, credential, or provider payload data.

## Authority inventory

The executable architecture gate inventories authority-shaped structs whose names end in `Authority`, `Authorization`, or `Capability`, across public and private visibility, brace, tuple and unit forms, generic declarations, `where` clauses, and generic inherent implementations. It rejects `Serialize`/`Deserialize`, plain public fields, and public unit construction. Balanced signature scanning covers free and inherent functions, including generic methods, nested parameter delimiters, qualified or generic owners, `where` clauses, inherent `Self`, equivalent explicit authority returns, and direct or chained crate-local type aliases across separate module files and nested inline-module scopes (including generic, explicit-item, glob-imported, module-aliased, and path-qualified uses); every issuer must be listed with a rationale. Authority-shaped unresolved, ambiguous, or cyclic aliases fail closed rather than suppressing an issuer. A policy-shaped name or `*Policy` implementation never suppresses this inventory. The sole policy-only exception is an exact, rationalized manifest entry whose checked type is a fieldless, non-serializable unit struct, implements the exact documented trait, and has no inherent or free manufacture function through either its canonical name or any same-module or cross-module crate-local alias. Classification enums and ordinary functions returning non-authority values are not executable capabilities.

Current scarce capabilities are:

- `domain::customer::intelligence::NoteAcceptanceAuthority`: opaque and non-serializable; no production issuer.
- `domain::entities::incident::IncidentClosureAuthority`: opaque and non-serializable; test-only issuer while authenticated reviewer infrastructure is unavailable.
- `domain::entities::message_record::{ReviewerAuthority, QueueAuthorization}`: reviewer issuance is test-only; queue authorization consumes exact current reviewer authority plus matching historical approval.
- `domain::lead::response::QueueContactAuthority`: private opaque unit proof nested in `QueueableContact`; no production issuer exists.
- `storage::operations::CurrentApprovalReviewerCapability`: opaque and non-serializable; its `try_new` issuer is test-only until a trusted authentication adapter owns current identity and role validation.
- `storage::operations::ApprovedInternalHandoffAuthority`: opaque, non-cloneable, non-serializable, one-shot admission for one exact internal handoff after consuming the current-reviewer capability and exact persisted binding.

`PayloadAuthority` and similarly named enums are classifications, not executable capabilities. They remain serializable only when they explicitly describe evidence posture and grant no operation.

## Where a new item belongs

- Put a concept and its semantic error beside the invariant owner.
- Put a conversion at the boundary that understands both representations; make trust-changing promotion fallible and named.
- Do not add historical translation to the prelaunch tree; revise the one current boundary model when evidence changes.
- Put use-case repository traits in `app`, concrete persistence in `storage` or the runtime adapter, and provider reads in the integration adapter.
- Put authority issuance at the narrowest authenticated boundary that can recheck current identity, role, scope, target, gate, evidence version, and chronology. If that root is unavailable, model no production issuer and fail closed.

Executable enforcement lives in `scripts/check_architecture_quality.py`, its fixture tests in `scripts/tests/test_architecture_quality.py`, and compile-fail/behavior tests in the owning Rust crates.
