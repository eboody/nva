# SpacetimeDB stale-state and adapter consistency audit

Status: quality-gate evidence for Kanban task `t_d0f78b6d`. This audit is scoped to the SpacetimeDB Data-Quality Hygiene slice and the docs/demo/API artifacts touched by the enterprise-pivot board. It does not claim production deployment, live NVA/Gingr/PMS access, live customer/provider side effects, or full-domain SpacetimeDB persistence.

## Gate conclusion

The scoped SpacetimeDB slice has no remaining blocker-class stale or inconsistent adapter state after this pass and the follow-up denied-command rollback audit.

The concrete drift found by the parent audit has been fixed: app-service blocked outcome-capture rows emitted through `BlockedActionLogAdapter` now keep review-queue location context for known actions, are persisted by `HygieneCaptureRuntime`, and are projected into public `BlockedActionNoticeRow` rows just like direct reducer blocked paths. The follow-up rollback audit also fixed the reducer return boundary: denied commands that need blocked-attempt evidence now return `Ok(())` after projecting notice rows, because returning `Err` would roll back the SpacetimeDB transaction. The regression test `realtime_queue_tests::app_service_blocked_capture_rows_keep_review_location_for_public_notices` covers the private-row/public-notice contract; the reducer module docs capture the transaction-return invariant.

Deferred areas remain explicit architecture/product choices, not orphaned local surfaces: enterprise audit/export backbone, production identity/tenant integration, raw evidence object storage, provider/PMS mutation adapters, and full customer/pet/reservation/payment/schedule persistence are either documented elsewhere or intentionally out of the scoped slice.

## Workspace and changed-file classification

Observed `git status --short` during this gate: the checkout is intentionally dirty from the broader board. This task did not revert or destructively clean shared work.

Classification:

- Source/runtime adapter: `apps/spacetimedb/src/adapter.rs`, `apps/spacetimedb/src/runtime.rs`, `apps/spacetimedb/src/realtime_queue_tests.rs` were changed by this gate to resolve the blocked-action projection drift and add focused coverage.
- Existing SpacetimeDB slice source already present from parent work: `apps/spacetimedb/src/read_model.rs`, `apps/spacetimedb/src/reducers.rs`, `apps/spacetimedb/src/storage/review_queue/codec.rs`, plus new crate/read-model/storage/table files under `apps/spacetimedb/`.
- Dedicated audit evidence: `docs/architecture/spacetimedb-domain-surface-coverage.md` was updated to mark the blocked-action drift resolved, and this file was added as final stale-state evidence.
- Other board artifacts left in place: local demo docs, demo seed fixtures, Dockerfiles/compose updates, API/read-model tests, OpenAPI JSON, staff-web smoke/UI files, and package/script changes are broader board outputs rather than stale-state cleanup targets for this task.
- Generated/API artifact disposition: `apps/api/openapi/owned-operations-v0.openapi.json` remains an intentional touched API artifact from the board. No generated auxiliary cache/report directory was found or cleaned by this gate.
- Untracked architecture docs: `audit-reporting-evidence-backbone.md`, `http-runtime-responsibility-cutline.md`, and `spacetimedb-domain-surface-coverage.md` are intentional evidence/decision artifacts and should be staged or reviewed with the feature-board closeout, not discarded as noise.

## Storage/read-model/reducer role ledger

Every SpacetimeDB struct inspected has a deliberate adapter role:

| Surface | Role | Evidence |
| --- | --- | --- |
| `ReviewQueueItemRow` | Private persisted/queryable row for live Data-Quality Hygiene review work. | Projected to staff/manager read models; promoted to app `ReviewQueueItem` through `codec::review_queue_item`. |
| `DataQualityIssueRow` | Private source-quality issue summary feeding the review queue. | Seed reducer inserts issue refs/summaries; full source/provenance remains outside the live row. |
| `WorkflowEventRow` | Private append-only transition/event row for reducer activity. | Written by reducer helper `append_workflow_event`. |
| `WorkflowOutcomeRow` | Private manager/staff disposition summary row. | Written by `record_manager_outcome`. |
| `HygieneOutcomeRow` | Private accepted outcome fact emitted by app-owned `OutcomeRecorder`. | Runtime persists adapter-emitted rows and projects outcome cards. |
| `HygieneAuditEventRow` | Private accepted-capture audit proof. | Runtime persists accepted app audit rows; enterprise audit backbone remains deferred. |
| `BlockedActionAttemptRow` | Private fail-closed audit row for denied capture/side-effect attempts. | Direct reducers and app-service blocked path write it; now also projected to public notice from both paths and committed via `Ok(())` denied-command returns. |
| `StaffActorRow` | Private identity-to-app-actor adapter row. | Used by `authz` and `ActorDirectoryAdapter`; not a domain actor entity. |
| `RoleAssignmentRow` | Private review-role assignment row. | Used to rebuild `ActorAssignment` for app authorization. |
| `LocationScopeRow` | Private location-scope row for review authorization. | Used to rebuild location-scoped app assignment. |
| `StaffQueueItemRow` | Public subscription read model for staff dashboard queue state. | Projected by `codec::staff_queue_item` from private queue rows. |
| `ManagerQueueItemRow` | Public subscription read model for manager-gated queue state. | Projected only when `requires_manager_approval` is true. |
| `BlockedActionNoticeRow` | Public subscription read model for denied actions/blocked side effects. | Projected by `codec::blocked_action_notice` from private blocked attempts. |
| `HygieneOutcomeCardRow` | Public subscription read model for reviewed hygiene outcomes. | Projected by `codec::staff_outcome_card`; hard-codes `live_delivery_allowed=false`. |
| `ReviewQueueStatusColumn`, `FeedbackOutcomeColumn`, `ResolutionStatusColumn`, `BlockedActionReasonColumn`, `ActorKindColumn`, `ReviewerRoleColumn` | Nested `SpacetimeType` adapter columns/reducer arguments. | Used as storage/reducer boundary vocabulary; app/domain semantics remain canonical. |
| `ActorDirectoryAdapter`, `ReviewQueueAdapter`, `OutcomeRecorderAdapter`, `AuditLogAdapter`, `BlockedActionLogAdapter` | App-port adapters around table snapshots and emitted rows. | They store/retrieve rows only and promote through app/domain types before decisions. |
| `HygieneCaptureRuntime` | Reducer runtime assembly/persistence boundary. | Loads table snapshots, calls app service, persists emitted outcome/audit/blocked rows, and projects public read models. |
| Reducer functions in `reducers.rs` | Thin command boundaries. | Parse primitive reducer args, resolve sender, call app policy/service, upsert private rows, and project public rows. |

## Drift found and fixed

Issue: the app-service blocked outcome-capture path and direct reducer blocked paths were inconsistent. Direct reducer blocked paths wrote `BlockedActionAttemptRow` and projected `BlockedActionNoticeRow`; the app-service path wrote only private blocked rows and used an empty `location_id`.

Fix:

- `BlockedActionLogAdapter` now carries review-queue snapshots and resolves `location_id` from the submitted action id, using `unknown` only when the app service blocks an action with no queue row.
- `HygieneCaptureRuntime::record_reviewed_outcome` now projects `BlockedActionNoticeRow` for each app-service blocked row it persists.
- Added a focused regression test that proves the app-service blocked row keeps location `101` and projects the same location into the public notice.
- Updated `docs/architecture/spacetimedb-domain-surface-coverage.md` so it no longer carries stale debt language for the resolved issue.

## Documentation consistency scan

Docs inspected or cross-checked:

- `apps/spacetimedb/README.md`
- `docs/architecture/spacetimedb-domain-surface-coverage.md`
- `docs/architecture/audit-reporting-evidence-backbone.md`
- `docs/architecture/http-runtime-responsibility-cutline.md`
- `docs/demo/local-demo-walkthrough.md`
- `docs/ops/local-demo-compose.md`
- `docs/presentation/nva-demo-executive-brief.md`
- root `README.md`

Consistency result:

- SpacetimeDB docs consistently frame rows as adapter storage/read models, not canonical domain entities.
- Postgres/S3 docs consistently keep durable audit/reporting/evidence outside the SpacetimeDB live runtime and do not contradict current code.
- HTTP cutline docs consistently keep HTTP as edge/compatibility/reporting after SpacetimeDB owns live queue commands/subscriptions.
- Demo/presentation docs consistently caveat local fixture/demo posture and `live_side_effects_allowed=false`.
- The stale statement about app-service blocked rows lacking public notice projection was corrected in the domain-surface audit.

## Explicit deferrals that are not stale debt

- Production SpacetimeDB publish/smoke remains caveated because `spacetime` CLI is installed but prior local probing documented a module/host ABI mismatch (`module abi 10.4`, host `10.0`).
- Enterprise audit/export/reporting remains delegated to the durable Postgres/S3-compatible backbone documented in `audit-reporting-evidence-backbone.md`.
- Full customer/pet/reservation/payment/schedule/vaccination/provider persistence remains outside the scoped realtime demo slice.
- Raw provider/document/media evidence remains outside public subscription rows and belongs in object storage/durable evidence refs.
- Production identity, tenanting, and provider mutation/live customer side-effect adapters require separate authorization and implementation lanes.

## Verification commands

Commands run from `/home/eran/code/nva`:

```sh
cargo fmt --all
cargo test -p nva-spacetimedb realtime_queue -- --nocapture
cargo check -p nva-spacetimedb
cargo check --workspace
python scripts/check_markdown_links.py --repo-root .
```

Results:

- `cargo fmt --all -- --check`: passed after formatting the new imports with `cargo fmt --all`.
- `cargo test -p nva-spacetimedb realtime_queue -- --nocapture`: passed, 8 tests.
- `cargo check -p nva-spacetimedb`: passed.
- `cargo check --workspace`: passed.
- `python scripts/check_markdown_links.py --repo-root .`: passed, 357 markdown files scanned and 22 required README entries checked. The first run exposed the missing root README link to `apps/spacetimedb/README.md`; this gate added that link and reran the check successfully.

Updated caveat from the follow-up audit: the local `spacetime` CLI is installed on `PATH`, but compatible publish/call/sql/subscription smoke remains blocked by the previously documented module/host ABI mismatch until the local host is upgraded or otherwise matched to the Rust SDK.
