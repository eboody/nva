# Agent-app infrastructure readiness review — 2026-06-18

## Scope

Final review of the first NVA Pet Resorts agent-app infrastructure loop after the Manager Daily Brief local smoke path landed. Review lens: labor-cost reduction for a 170-location pet-resort portfolio, not generic chatbot capability.

Evidence reviewed:

- `nva-pet-resorts-ai-context.md`
- `README.md` section `Where we are: labor-cost reduction platform`
- `docs/architecture/agent-app-infrastructure.md`
- `docs/design/manager-daily-brief-measurable-labor-loop.md`
- `docs/ops/hermes-manager-daily-brief-bridge.md`
- `docs/ops/manager-daily-brief-local-smoke.md`
- `docs/ops/openviking-local-memory.md`
- `docker-compose.yml`
- Manager Daily Brief API/domain/storage/UI contracts and smoke evidence from parent task `t_1474a95e`

Verification run in this review:

- `./scripts/test.sh` — pass
- `git diff --check` — pass
- `docker compose --profile agent-infra config >/tmp/nva-compose-agent-infra-review.yml` — pass
- lightweight repository secret scan — one expected fake-token redaction test fixture only (`scripts/tests/test_hermes_agent_bridge.py`)

## Executive verdict

Go for starting a second workflow on the same deterministic app/agent rails, with a narrow safety hardening follow-up already carded before any live/pilot use.

The current repo is sandbox/local-demo ready for a second review-gated labor workflow. It is not production-operational and should not be presented as ready for live customer sends, provider/PMS mutation, staff schedule mutation, payments/refunds/discounts, or safety-sensitive autonomous decisions.

## Readiness checklist

| Check | Verdict | Evidence |
| --- | --- | --- |
| Deterministic app owns facts, policy, review gates, persistence, audit, and side effects | Pass | Architecture guide names the boundary; API contract tests validate source refs, review gates, and blocked side effects; outcome capture persists app-owned labor evidence. |
| Hermes only consumes typed context and submits drafts/recommendations | Pass | `scripts/hermes-tools/*` call app-owned context, draft, and outcome endpoints; docs explicitly forbid raw Postgres/object-store/provider access from Hermes. |
| OpenViking is memory/knowledge infra, not operational truth | Pass with preflight caveat | `docs/ops/openviking-local-memory.md` and smoke docs state OpenViking can enrich reasoning but cannot authorize source facts or side effects. Local container now has a scripted preflight/remediation path; provider-backed health still requires operator-supplied `ov.conf`. |
| Docker stack reproducible locally | Mostly pass | Compose config validates and the parent smoke ran `PET_RESORT_POSTGRES_HOST_PORT=55432 ./scripts/smoke_manager_daily_brief_local_loop.sh`. App/API/worker/Postgres/MinIO are reproducible; OpenViking has a deterministic preflight that exits with remediation when the local volume lacks config. |
| Manager Daily Brief labor loop visible and measurable | Pass | Domain/API/storage/UI tests cover context packets, draft validation, outcome capture, estimated vs actual minutes saved, and staff-web smoke surface. Parent smoke reported context actions=3, estimated minutes saved=62, accepted draft actions=1, and actual minutes saved=8 for a fake reviewed outcome. |
| Secrets are not committed | Pass | `.env.example` uses local placeholders; Docker service values are local/dev only or redacted; bridge tests prove bearer tokens are not printed on errors. |
| README/docs tell future workers where to start | Pass after this review note and README status refresh | README points to the NVA context, app/agent infrastructure guide, Manager Daily Brief loop design, OpenViking runbook, bridge runbook, smoke runbook, and this readiness review. |
| `./scripts/test.sh` passes | Pass | Full Rust, Python bridge, and staff-web gates passed in this review. |

## Go/no-go for a second agent workflow

Go, but keep the second workflow draft/review/outcome-only until its own smoke proves the same rails.

Recommended next workflow: data-quality hygiene.

Why data-quality hygiene is the best second workflow after Manager Daily Brief:

1. It directly amplifies labor-cost reduction by reducing repeated manager/front-desk re-checking of missing vaccines, ambiguous checkout/completion facts, incomplete pet/customer profiles, duplicate customers, vague notes, and inconsistent service naming.
2. It is naturally internal/review-gated: recommendations can become manager or front-desk tasks without customer sends, PMS mutation, schedule changes, or payment movement.
3. It strengthens every later workflow. Lead conversion, grooming rebooking, SOP assistance, and regional exception reporting all get safer when source ambiguity is first-class and measurable.
4. It reuses the exact pattern already proven by Manager Daily Brief: typed context packet -> source refs/data-quality issues -> ranked internal actions -> app validation -> staff outcome capture -> actual minutes saved.

Suggested second-workflow slice:

- `GET /agent/context/data-quality-hygiene?location_id=...&operating_day=...`
- `POST /agent/drafts/data-quality-hygiene`
- `POST /data-quality-hygiene/actions/{action_id}/outcome`
- Action kinds: investigate missing source evidence, reconcile duplicate customer/pet candidate, complete missing pet profile fields, review stale vaccination/source freshness, normalize ambiguous service-line naming.
- Blocked actions: customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, hiding or auto-resolving source ambiguity.
- Labor metric: minutes avoided in manual source reconciliation and repeated staff rework per location/day.

## Remaining blockers / future cards

The remaining blockers are explicit follow-up cards rather than vague prose:

1. `t_60324496` — fail-closed validation for unknown Manager Daily Brief draft side effects.
   - Finding: draft validation rejects known blocked side effects but should also reject any unknown non-empty requested side effect as unsupported. Outcome capture already behaves fail-closed; draft validation should match it.
   - Impact: blocks live/pilot posture, does not block starting a second sandbox workflow if the same follow-up is applied before pilot.

2. `t_026ec84f` — make local OpenViking agent-infra initialization reproducible without secrets. (Resolved by follow-up preflight/remediation script.)
   - Finding: OpenViking is correctly documented as optional agent-side context infrastructure, and a fresh local volume now fails with a scripted remediation when `ov.conf` is missing instead of pretending local agent-infra is turnkey.
   - Impact: provider-backed OpenViking health still requires operator-supplied config/secrets, but the local repo path is reproducible and secret-safe.

## Caveats

- This review did not run the full Docker smoke again; it relied on the parent task's smoke evidence and reran Compose config plus the canonical local gate.
- The repo is currently local/demo and sandbox-proven, not pilot/live. Real NVA data access, permissions, retention, monitoring, and rollback need separate proof.
- Any future MCP packaging should preserve the same app-owned contract. MCP is a tool schema/packaging upgrade, not a permission expansion.
