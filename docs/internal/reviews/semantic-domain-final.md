<!-- archived-historical-record -->
# Archived semantic architecture security and adoption review

This is a historical review of a superseded tree. It is not current architecture authority.

Date: 2026-08-13
Task: `t_921e8ad6`
Repository: `/home/eran/code/nva`
Branch: `nva-pilot-clean-20260807`

## Verdict

`approved=true` after closeout remediation on 2026-08-13.

The repository is substantially stronger than the earlier baseline and now demonstrates real data-modeling-first behavior across several pet-resort vertical slices. The best current proof is the lead-response chain: source event and SLA evidence, exact consent, ordered reviewed attempt, message-approval proof, queue-only legal action, API idempotency drift rejection, and no live-send/provider-write capability.

Closeout remediation moved the strategic AI operations concepts into their canonical owner modules and shrank `domain/src/strategic_ai_ops_bridge.rs` to deprecated compatibility re-exports from those owners. The former alias-hidden ownership failure is resolved: `domain::access`, `domain::consent`, `domain::identity`, `domain::lead::response`, `domain::customer::intelligence`, `domain::operations::{labor,capacity,time_bucket}`, `domain::agent::{assistant,knowledge}`, and `domain::analytics::{finance,outcome}` now own the definitions directly.

## Blocking finding

### B1: Canonical owner modules still hide bridge-owned definitions behind re-exports

Severity: blocking architecture/data-integrity review finding. Status: resolved by closeout remediation.

Evidence paths:

- `domain/src/lib.rs:25` maps `strategic_ai_ops_bridge.rs` into a private module named `canonical_ai_ops_contracts`.
- `domain/src/access.rs:7` re-exports `crate::canonical_ai_ops_contracts::access::*`.
- `domain/src/consent.rs:7` re-exports `crate::canonical_ai_ops_contracts::communication::*`.
- `domain/src/identity.rs:7` re-exports `crate::canonical_ai_ops_contracts::identity::*`.
- `domain/src/lead.rs:90-92` exposes `lead::response` via `canonical_ai_ops_contracts::lead_response::*`.
- `domain/src/customer.rs:17-19` exposes `customer::intelligence` via `canonical_ai_ops_contracts::crm::*`.
- `domain/src/operations.rs:24-43` exposes `labor`, `capacity`, and `time_bucket` via `canonical_ai_ops_contracts`.
- `domain/src/agent.rs` and `domain/src/analytics.rs` follow the same bridge-re-export pattern for assistant/knowledge and finance/outcome.

Why it blocks approval:

1. The public path says the concepts are owned by `domain::access`, `domain::lead::response`, `domain::customer::intelligence`, `domain::operations::{labor,capacity}`, `domain::agent::{assistant,knowledge}`, and `domain::analytics::{finance,outcome}`, but the definitions still live in one large bridge file.
2. Rustdoc, search, review, and future diffs still center a catch-all implementation surface rather than concept-owned files.
3. A future change can mutate strategic definitions by editing the bridge file without touching the advertised canonical owner module.
4. This weakens mechanical review against the atlas and makes it harder to prove that source evidence, consent, review, action, and outcome concepts have one true owner.

RED proof added by this review:

- Changed file: `domain/tests/semantic_rehome_contracts.rs`
- Test: `canonical_semantic_owner_modules_do_not_hide_renamed_bridge_reexports`
- Command: `cargo test -p domain --test semantic_rehome_contracts canonical_semantic_owner_modules_do_not_hide_renamed_bridge_reexports -- --nocapture`
- Observed failure: `domain/src/access.rs still re-exports renamed bridge definitions instead of owning canonical definitions`.

Required remediation:

Move definitions into their canonical owner modules, then make any deprecated compatibility facade re-export from those canonical modules. After remediation, `domain/src/strategic_ai_ops_bridge.rs` should disappear or shrink to compatibility glue with no primary definitions. The already-added direct-bridge test and the new renamed-bridge test should both pass.

## Strategic maturity table

| Strategic workstream | Final rating | Evidence paths | Residual risk / remediation |
| --- | --- | --- | --- |
| Lead response source to reviewed queue-only action | L3, with partial L4 adoption | `domain/src/lead.rs`, `domain/src/strategic_ai_ops_bridge.rs`, `domain/tests/strategic_ai_ops_model_contracts.rs`, `domain/tests/semantic_rehome_contracts.rs`, `apps/api/src/http.rs`, `apps/api/tests/api_dto_contracts.rs`, `domain/src/lead.rs` doctest | Strong invariant chain, but canonical definitions still bridge-owned. Move lead-response definitions into `domain/src/lead.rs` or a `domain/src/lead/response.rs` owner module. |
| CRM retention and customer intelligence | L3 app/domain behavior, partial L4 through manager brief/storage; ownership blocker remains | `domain/src/customer.rs`, `app/src/crm_retention.rs`, `app/tests/crm_retention_workflow_contracts.rs`, `app/src/manager_daily_brief.rs`, `storage/src/operations.rs` | Accepted/expired/operations-only/consent distinctions are present, but customer-intelligence definitions are still bridge-owned. |
| Capacity/labor to Manager Daily Brief | L3/L4 slice behavior, ownership blocker remains | `domain/src/operations.rs`, `domain/tests/strategic_ai_ops_model_contracts.rs`, `app/src/manager_daily_brief.rs`, `app/tests/manager_daily_brief_workflow_contracts.rs`, `storage/tests/manager_daily_brief_outcome_storage.rs` | Recommendation checks source evidence, same grain, feasibility, role, manager gate, and blocks schedule mutation. Definitions still need canonical module ownership. |
| Permissioned knowledge retrieval to assistant packet | L3 app/domain behavior, partial L4 API exposure | `domain/src/agent.rs`, `app/src/permissioned_knowledge.rs`, `app/tests/permissioned_knowledge_workflow_contracts.rs`, `apps/api/src/http.rs`, `apps/api/tests/health_contract.rs` | Authorization/citation/escalation are fail-closed, but knowledge/assistant definitions still originate in the bridge file. |
| Site finance recommendation and reviewed outcome | L3 app/domain behavior, partial L4 API/storage proof | `domain/src/analytics.rs`, `app/src/site_finance.rs`, `app/tests/site_finance_workflow_contracts.rs`, `apps/api/src/http.rs`, `storage/src/operations.rs` | Checked money, period/service/source/review/action/outcome proof are present. Finance/outcome definitions remain bridge-owned and source provenance is stronger at app slice than at generalized domain fact. |
| Approval/outbox/read-model infrastructure | L3 storage infrastructure, partial L4 across proven slices | `storage/src/operations.rs`, `storage/tests/approval_outbox_infrastructure.rs`, `apps/api/src/http.rs` | Durable records preserve review/outbox/source/action fields, but this is infrastructure proof, not live provider write authority. |
| Data-quality/source provenance spine | L3/L4 for data-quality and source slices | `domain/src/source.rs`, `domain/tests/source_code_contracts.rs`, `app/src/data_quality_hygiene.rs`, `storage/tests/data_quality_read_model_storage.rs`, `apps/api/tests/data_quality_hygiene_agent_contract.rs` | Strongest existing source/provenance adoption; remaining docs-navigation debt is separate from data-integrity proof. |

## Cross-layer adoption and security review

### Strengths

- Source/provider facts remain quarantined. API docs and DTO markers explicitly say provider payloads are not product-owned contracts (`apps/api/src/http.rs:1-8`, `apps/api/src/http.rs:395-399`).
- Live side effects remain blocked in inspected slices: customer messages, provider/PMS writes, schedule/capacity mutation, and payment/refund/discount actions are represented as blocked actions or disabled booleans, not executable capabilities.
- Builders that matter for invariants are not blindly trusted. The important lead-response, SLA, finance insight, site-finance projection, and outcome paths use `try_new`, custom serde, or promotion methods for cross-field checks.
- Sensitive text and identifiers have several redaction protections: lead idempotency key debug redaction, CRM note body redaction, assistant answer packet debug redaction, and no raw provider payload passthrough in the API surface.
- Outcome/value claims are mostly fail-closed: manager daily brief outcomes require matching action/source refs for labor-savings support, finance/strategic outcomes distinguish reviewed attribution from correlated-only evidence, and local safety docs still say no measured NVA labor-savings claim exists yet.

### Nonblocking risks

- Several app workflow modules still use fixture/static `unwrap`/`expect` in deterministic fixture construction. I did not classify those as production panic blockers because they are static fixtures or post-validation conversions, but new runtime ingestion paths should not copy that style.
- `./scripts/check_docs.sh` was already reported by the previous card as failing on broad docs/navigation debt. That is a follow-up docs lane, not a reason to hide the architecture blocker.
- Modum remains noisy repo-wide. The previous pass reported 209 diagnostics under `report.diagnostics`, classified as semantic-design prompts rather than regressions. This review did not mechanically appease Modum.

## RED/GREEN and verification evidence

RED added in this review:

```sh
cargo test -p domain --test semantic_rehome_contracts canonical_semantic_owner_modules_do_not_hide_renamed_bridge_reexports -- --nocapture
```

Observed result: failed as expected, 0 passed / 1 failed, because `domain/src/access.rs` re-exports renamed bridge definitions.

Focused re-run after adding the review test:

```sh
cargo test -p domain --test semantic_rehome_contracts -- --nocapture
```

Observed result: failed as expected, 2 passed / 1 failed. The pre-existing direct bridge test passes; the new renamed bridge test fails.

Fresh passing checks during this review:

```sh
cargo fmt --all -- --check
python scripts/tests/test_semantic_domain_contract_atlas.py
git diff --check
```

Observed result: all passed.

Recent upstream full-gate evidence from parent card `t_dbbd57a6` remains relevant but is not sufficient to approve this final review because the new RED review test intentionally fails:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
python scripts/check_rustdoc_completeness.py
git diff --check
```

Parent observed result: all passed before this final review test was added. `./scripts/check_docs.sh` remained a known repo-wide docs/navigation debt.

Secret scan:

No secret-shaped literals were found in the final review diff for `docs/internal/reviews/semantic-domain-final.md`, `docs/architecture/semantic-domain-contract-atlas.md`, and `domain/tests/semantic_rehome_contracts.rs` using the review regex scan recorded in the Kanban handoff.

## Closeout remediation update

The blocking ownership finding is resolved. Definitions now live in canonical owner files, and the deprecated `strategic_ai_ops` facade re-exports only from those canonical owners. Verification evidence:

```sh
cargo test -p domain --test semantic_rehome_contracts -- --nocapture
cargo test -p domain --test strategic_ai_ops_model_contracts -- --nocapture
```

Observed result: `semantic_rehome_contracts` passed 3/3, including the renamed-bridge ownership test, and `strategic_ai_ops_model_contracts` passed 34/34.

## Approval state

`approved=true` for the final semantic architecture/security/adoption review after the closeout remediation above. This approval remains bounded to the local, no-live-side-effect repository proof; it does not authorize production deployment, provider/PMS/CRM writes, customer sends, scheduling/staffing mutation, money actions, or measured NVA value claims without future source/outcome evidence.

## Next card assumptions

- No live customer/provider/PMS/CRM/schedule/capacity/staffing/payment/deployment action is authorized.
- The next remediation should be a pure move/ownership refactor, not a semantic behavior redesign.
- Keep the public `domain::strategic_ai_ops` compatibility facade only as deprecated re-exports from canonical owner modules.
- Preserve all existing serde, redaction, review-gate, source-evidence, outcome-attribution, and no-live-side-effect tests while moving definitions.
