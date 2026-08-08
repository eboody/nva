# SpacetimeDB domain surface coverage and adapter debt register

Status: audit artifact for the Data-Quality Hygiene SpacetimeDB slice. This page verifies the chosen operational slice is mapped cleanly without pretending the whole pet-resort domain has been ported to SpacetimeDB. It does not request storage annotations in the domain crate, does not require every domain type to become a SpacetimeDB row, and does not claim live NVA/Gingr/PMS access, provider writes, customer sends, payment movement, schedule changes, or production deployment.

Source files inspected:

- [`domain/src/data_quality.rs`](../../domain/src/data_quality.rs)
- [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs)
- [`apps/spacetimedb/src/storage/review_queue/row.rs`](../../apps/spacetimedb/src/storage/review_queue/row.rs)
- [`apps/spacetimedb/src/storage/review_queue/status_column.rs`](../../apps/spacetimedb/src/storage/review_queue/status_column.rs)
- [`apps/spacetimedb/src/storage/review_queue/codec.rs`](../../apps/spacetimedb/src/storage/review_queue/codec.rs)
- [`apps/spacetimedb/src/tables.rs`](../../apps/spacetimedb/src/tables.rs)
- [`apps/spacetimedb/src/read_model/`](../../apps/spacetimedb/src/read_model/)
- [`apps/spacetimedb/src/adapter.rs`](../../apps/spacetimedb/src/adapter.rs)
- [`apps/spacetimedb/src/runtime.rs`](../../apps/spacetimedb/src/runtime.rs)
- [`apps/spacetimedb/src/reducers.rs`](../../apps/spacetimedb/src/reducers.rs)
- [`apps/spacetimedb/src/realtime_queue_tests.rs`](../../apps/spacetimedb/src/realtime_queue_tests.rs)
- [`app/tests/data_quality_hygiene_workflow_contracts.rs`](../../app/tests/data_quality_hygiene_workflow_contracts.rs)
- [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs)

## Scope conclusion

The SpacetimeDB implementation covers one operational vertical slice: live review of source-grounded Data-Quality Hygiene work, manager/staff queue coordination, reviewed outcome capture, audit rows, and fail-closed blocked-action notices. The implementation does not attempt to persist the full `domain` crate or every pet-resort concept in SpacetimeDB. That is the right cutline for this pivot.

The demo slice is clean enough to describe as extensible because:

1. app/domain types remain canonical in `domain` and `app`;
2. SpacetimeDB structs are adapter storage rows, public read models, or small adapter value columns;
3. reducers parse primitives, resolve sender identity, call app-owned authorization/capture services, and project rows;
4. query/subscription fields needed by the live queue are flattened on rows/read models; and
5. unimplemented domain areas are deliberate deferrals, not accidental hidden storage gaps.

It is not honest to claim that customer/pet/reservation/payments/scheduling/vaccination/provider-ingestion have all been implemented in SpacetimeDB. They are represented only as source refs, issue categories, affected-entity labels, review gates, actor/location scopes, and blocked-action families needed by this slice.

## Touched concept inventory

| Domain/app concept | Source owner | SpacetimeDB representation | Coverage decision |
| --- | --- | --- | --- |
| Data-quality issue kind, severity, field path, source provenance, resolution status | `domain::data_quality` | Private `DataQualityIssueRow` stores `issue_ref`, `location_id`, `source_ref_id`, summary, timestamps, schema version. `ResolutionStatusColumn` is a nested `SpacetimeType` used on outcomes. | Persist only the queue-summary/ref facts needed by live review. Full issue/provenance semantics stay in domain/app and durable Postgres evidence. |
| Source record refs and provenance | `domain::source` | `source_ref_id` flattened on queue/read-model rows; `source_record_refs` encoded string on outcome rows/cards; reducer argument `source_record_id`. | Live queue needs filter/display traceability, not raw provider payloads. Full source/evidence backbone remains outside SpacetimeDB live rows. |
| Hygiene candidate/request/packet/action/labor estimate | `app::data_quality_hygiene` | Not persisted as a large packet. Seed/demo reducers insert `ReviewQueueItemRow`; read models expose queue state. Labor estimate is persisted on accepted `HygieneOutcomeRow`. | Correctly not stored opaquely. The live slice persists action/review/outcome facts, while app workflow remains the canonical packet builder. |
| Review queue item | `app::data_quality_hygiene::ReviewQueueItem` | Private `ReviewQueueItemRow`; public `StaffQueueItemRow` and `ManagerQueueItemRow`. | Fully represented for the slice. Key fields are flat: `action_id`, `location_id`, `actor_id`, `claimed_by_actor_id`, status label/column, `source_ref_id`, `issue_ref`, timestamps, manager gate. |
| Review queue status | app workflow status semantics | Nested `ReviewQueueStatusColumn` on private queue rows; public `status_label` strings on subscription rows. | Acceptable adapter column. It is storage/read-model vocabulary, not a domain enum replacement. |
| Staff/manager/system actor identity | `domain::entities::ActorRef` plus app `ActorId`/`ActorAssignment` | Private `StaffActorRow` keyed by `actor_id`, indexed by SpacetimeDB `identity`; `ActorKindColumn` nested value. | Covered for demo/local authz. Production identity provider integration is deferred; live slice has a deterministic adapter boundary. |
| Review role and location scope | `app::data_quality_hygiene::ReviewerRole`; `domain::entities::LocationId` | Private `RoleAssignmentRow`, `LocationScopeRow`, `ReviewerRoleColumn`; adapter promotion through `authz.rs`. | Covered for location-scoped reducer authorization. Composite identity/scope uniqueness is not modeled because SpacetimeDB lacks composite primary keys; synthetic ids plus indexes are acceptable for the slice. |
| Authorization policy | `app::data_quality_hygiene::RoleLocationAuthorization` | Not persisted as policy. Reducers load adapter rows and call app policy. | Correct: policy remains app-owned and test-covered. |
| Reducer command inputs | SpacetimeDB reducer boundary | Primitive reducer arguments (`String`, `u32`, `bool`, small `SpacetimeType` columns) promoted through codecs/newtypes. | Correct boundary shape. The primitives are not allowed past the adapter without promotion. |
| Outcome capture request, reviewed outcome, source/issue proof | `app::data_quality_hygiene::OutcomeRecord` and `OutcomeCaptureService` | Reducer arguments plus private `HygieneOutcomeRow`; public `HygieneOutcomeCardRow`. `FeedbackOutcomeColumn` and `ResolutionStatusColumn` are nested values. | Covered for accepted outcomes. Source refs and issue refs are required by app builders before the adapter persists rows. |
| Accepted audit event | `app::data_quality_hygiene::AuditLog` | Private `HygieneAuditEventRow` with `action_id`, actor id/label, blocked actions, timestamp, schema version. | Covered for realtime proof. Enterprise audit history needs richer durable shape in the audit/reporting backbone. |
| Denied command / blocked side effect | `app::data_quality_hygiene::BlockedActionLog` and reducer fail-closed branches | Private `BlockedActionAttemptRow`; public `BlockedActionNoticeRow`; `BlockedActionReasonColumn` nested value. | Covered for the demo slice. Explicit `attempt_blocked_side_effect`, direct reducer authz denial, and app-service blocked outcome-capture rows all project public notices and then return `Ok(())` so SpacetimeDB does not roll back the audit transaction; app-service rows derive `location_id` from review-queue context and fall back to `unknown` only when the submitted action has no queue row. |
| Public staff dashboard subscription | SpacetimeDB public read model | `StaffQueueItemRow`, `HygieneOutcomeCardRow`, `BlockedActionNoticeRow`. | Covered for demo. Subscription-critical fields are flattened/indexable where clients filter or route. |
| Public manager dashboard subscription | SpacetimeDB public read model | `ManagerQueueItemRow`, emitted only for manager-gated work. | Covered for demo. Manager-specific read model is separate from staff queue, so the public contract can evolve independently. |
| Customer, pet, reservation, service line, vaccination, payment, checkout, sensitive payload | Large domain/app surface named by `AffectedEntity`, `IssueCategory`, `BlockedAction`, `source::RecordRef` | Not persisted as domain objects. Only refs/categories/review gates/blocked action families are stored. | Deliberately deferred/not needed for this slice. The demo reviews source-quality work about these areas; it does not become their system of record. |

## Flattening and subscription audit

The live queue/subscription paths have indexable flat fields for the queries this slice needs:

- reducer lookup: `ReviewQueueItemRow.action_id` primary key;
- location routing/filtering: `location_id` indexes on private queue rows and public staff/manager rows;
- actor ownership/filtering: `actor_id` / `claimed_by_actor_id` indexes on private/public queue rows;
- source traceability/filtering: `source_ref_id` indexes on private/public queue rows;
- issue traceability: `issue_ref` is a direct queue/read-model field;
- queue ordering/backlog: `created_at` indexes on queue/read-model rows;
- role/scope lookup: `StaffActorRow.identity`, `RoleAssignmentRow.actor_id`, `LocationScopeRow.actor_id`, and `LocationScopeRow.location_id` indexes;
- blocked notices: `action_id`, `actor_id`, and `location_id` indexes;
- audit/outcome lookup: accepted audit rows and workflow outcome rows index `action_id` and actor id.

No large domain aggregate is stored opaquely where a reducer or subscription needs to query inside it. The only compact encoded strings are `source_record_refs`, `issue_refs`, `blocked_actions`, and display labels on audit/outcome cards. For the realtime demo they are display/evidence summaries, not subscription predicates. If future clients need to filter by each individual source ref, issue ref, blocked action, or audit decision, split those into child rows before production rather than parsing strings in clients.

## App-port implementation audit

Reducers use these app-owned capabilities:

| Capability | Implementation in SpacetimeDB adapter | Status |
| --- | --- | --- |
| `ActorDirectory` | `ActorDirectoryAdapter` promotes `StaffActorRow` + role/scope rows into `ActorAssignment`. | Implemented for demo/local identity rows. |
| `AuthorizationPolicy` | `RoleLocationAuthorization` remains in app; reducers call it for queue work and outcome capture. | Implemented and test-covered. |
| `ReviewQueueStore` | `ReviewQueueAdapter` promotes `ReviewQueueItemRow` through `codec::review_queue_item`. | Implemented. |
| `OutcomeRecorder` | `OutcomeRecorderAdapter` collects `HygieneOutcomeRow`; `HygieneCaptureRuntime` persists emitted rows. | Implemented. |
| `AuditLog` | `AuditLogAdapter` collects `HygieneAuditEventRow`; runtime persists emitted rows. | Implemented for accepted captures; enterprise audit details are deferred to the durable backbone. |
| `BlockedActionLog` | `BlockedActionLogAdapter` collects `BlockedActionAttemptRow`; runtime persists emitted rows and projects `BlockedActionNoticeRow`. | Implemented for the scoped demo path. The adapter carries review-queue context so blocked outcome-capture rows keep location-scoped public notice evidence. |

The reducer layer also has direct fail-closed paths (`record_blocked_attempt`, `attempt_blocked_side_effect`) that write blocked attempt rows and project public notices. That is useful proof, but it creates two blocked-action paths. The final implementation should consolidate or verify both paths produce the same private and public evidence.

## Accepted/denied command coverage

Accepted commands:

- `claim_review_item`, `attach_recommendation`, `record_staff_disposition`, and `record_manager_outcome` update private queue state, append workflow events, and project staff/manager read models.
- `record_manager_outcome` writes a `WorkflowOutcomeRow` and `HygieneAuditEventRow` with blocked live side effects preserved.
- `record_reviewed_hygiene_outcome` builds an app `OutcomeRecord` requiring source and issue proof, calls app-owned authorization/capture, persists accepted outcome/audit rows, updates the queue to `OutcomeRecorded`, and projects outcome cards.

Denied commands:

- Pre-evidence validation failures such as unknown sender identity, missing queue rows, or invalid primitive/domain inputs still return reducer errors because there is no safe action/location evidence row to commit yet.
- Actor/location/review-gate failure records `BlockedActionAttemptRow` in direct reducer authz paths and then returns `Ok(())` through the reducer so that evidence is committed.
- `attempt_blocked_side_effect` stores a blocked attempt, projects `BlockedActionNoticeRow`, marks the queue blocked, appends a workflow event, and returns `Ok(())` without performing the side effect so the denied-attempt evidence is committed.
- App-service outcome-capture denial records blocked rows through `BlockedActionLogAdapter`; `HygieneCaptureRuntime` persists the private row and projects the public `BlockedActionNoticeRow` from the same row, then the reducer returns `Ok(())` so the evidence is committed.

The app-service blocked path is now aligned with the direct reducer blocked paths for this scoped demo. These denied reducers intentionally use committed notice rows, not reducer `Err` returns, because SpacetimeDB rolls back reducer transactions on errors. Production hardening should still replace synthetic timestamps and enrich the durable audit/export backbone before making enterprise audit claims.

## Deferred domain areas

| Area | Decision | Rationale |
| --- | --- | --- |
| Full customer profile and messaging | Acceptable deferred / not needed | Data-Quality Hygiene blocks customer sends and stores only source/action refs. A future messaging/outbox lane must add reviewed delivery contracts before any live send. |
| Pet/vaccination care eligibility | Acceptable deferred / not needed | This slice can flag missing/stale vaccination evidence; it must not decide medical/safety eligibility. Future care/vaccine workflows should own those read models. |
| Reservation/stay/checkout lifecycle | Acceptable deferred | The slice references reservation/stay/checkout source issues but does not own bookings, occupancy, checkout, or billing state. Future realtime operations may add dedicated rows if subscriptions need them. |
| Payment/refund/discount state | Acceptable deferred with stricter gate | Payment conflict review is represented as protected review work only. Any live payment movement remains blocked and would require a separate approval/audit/outbox implementation. |
| Provider/PMS mutation adapters | Not needed for this slice | The slice explicitly proves no provider/PMS mutation. Future live adapters must be approval-gated and disabled/stubbed until separately authorized. |
| Durable audit/reporting/export | Safe future work, already documented | SpacetimeDB live rows are not the enterprise ledger. See [audit/reporting/evidence backbone](audit-reporting-evidence-backbone.md). |
| Production identity provider / tenant model | Safe future work | Demo seed rows prove the adapter boundary. Production authn/authz needs a dedicated identity/tenant integration lane. |
| Raw source payload/document/media evidence | Not needed in SpacetimeDB live rows | Live rows should carry safe refs/summaries only. Raw or redacted large evidence belongs in object storage and durable audit rows. |

## Debt register

| Severity | Item | Evidence | Disposition |
| --- | --- | --- | --- |
| blocker | None found for the scoped local Data-Quality Hygiene demo after the denied-command rollback audit. | Domain crate remains unannotated; rows/read models are adapter-owned; reducers are thin enough for the slice; live side effects remain blocked; denied commands that need evidence now commit blocked rows by returning `Ok(())`. | Final review can evaluate the slice without requiring full-domain SpacetimeDB coverage. |
| resolved | App-service blocked outcome-capture path now persists `BlockedActionAttemptRow` and projects `BlockedActionNoticeRow`; `BlockedActionLogAdapter` no longer emits empty `location_id` for known queued actions. | `adapter.rs` builds blocked rows from `BlockedActionRecord` with review-queue context; `runtime.rs` persists blocked rows and projects notices; `realtime_queue_tests::app_service_blocked_capture_rows_keep_review_location_for_public_notices` covers the contract. | Resolved by quality gate `t_d0f78b6d`; no separate remediation card required for this adapter consistency issue. |
| near-term | Accepted audit rows are sufficient for realtime proof but too thin for enterprise audit/export: no correlation/causation id, command label, decision reason, role/scope snapshot, source/object refs, or idempotency key. | `HygieneAuditEventRow` stores action, actor, blocked actions, timestamp, schema version. | Covered by [audit/reporting/evidence backbone](audit-reporting-evidence-backbone.md); do not represent current SpacetimeDB audit rows as production-complete audit ledger. |
| acceptable deferred | Encoded comma-separated source refs / issue refs / blocked actions are display summaries, not filterable child rows. | `HygieneOutcomeRow.source_record_refs`, `issue_refs`, and audit `blocked_actions` are strings. | Acceptable for demo display. Split to child rows if future subscriptions/export require filtering by individual ref/action. |
| acceptable deferred | Production identity/tenant/auth provider integration is not implemented. | Demo actor rows are seeded through reducers and `StaffActorRow.identity`. | Honest demo boundary. Future identity lane required for production. |
| not needed | Full customer/pet/reservation/payment/schedule domain persistence in SpacetimeDB. | These concepts appear as affected entity/source issue families and blocked action categories only. | Not technical debt for this slice; implementing them would over-scope the pivot. |

## Verification notes

Focused verification expected for this artifact and the code it audits:

```sh
cargo test -p nva-spacetimedb realtime_queue -- --nocapture
cargo check -p nva-spacetimedb
cargo check --workspace
python scripts/check_markdown_links.py --repo-root .
```

Current caveat from this audit: `spacetime` CLI is installed on PATH, but prior local publish probing documented an ABI mismatch (`module abi 10.4`, host `10.0`). Treat Cargo checks and script self-tests as compile/presenter evidence until a compatible SpacetimeDB host can run publish/call/sql/subscription smoke.
