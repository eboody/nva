# NVA static demo fallback packet

Status: presenter fallback for a job or networking conversation when the local shell is unavailable, slow, or not worth troubleshooting live. This packet is intentionally static: it does not claim fresh command output, live NVA/Gingr access, production data, provider/PMS writes, customer/member sends, payment/refund/discount actions, schedule/capacity changes, medical/safety decisions, or production deployment.

Use this only after running the normal pre-flight before the conversation. If the terminal is not available during the meeting, say that pre-flight was run and show the expected anchors plus the source files behind them instead of pretending stale output is fresh live proof.

## Exact fallback sentence

"Pre-flight was run before the conversation; if this shell is slow or unavailable, I won’t pretend stale output is fresh proof — I’ll show the static diagram, the checked OpenAPI artifact, and the expected local-demo anchors instead."

## What to open if terminal is unavailable

Open these in order:

1. `docs/presentation/assets/owned-operations-api-replacement.html` — the static HTML/SVG one-frame thesis.
2. `docs/presentation/owned-operations-api-visual-guide.md` — narration for the diagram and safety caveats.
3. `docs/presentation/job-presentation-walkthrough.md#local-demo-slice-data-quality-hygiene-in-five-minutes` — the demo section with expected anchors.
4. `apps/api/openapi/owned-operations-v0.openapi.json` — checked OpenAPI contract artifact.
5. `docs/presentation/nva-presentation-checklist.md#5-demo-command-and-what-to-watch-for` — live-demo anchors and claims-to-avoid context.

If the audience needs only the business story, start from `docs/presentation/nva-sendable-job-contact-summary.md` and then show the visual artifact.

## HTML/SVG visual path

The visual artifact is:

```text
docs/presentation/assets/owned-operations-api-replacement.html
```

It is a standalone HTML/SVG page with no JavaScript. It shows the shift from Gingr/provider extraction as source evidence to an NVA-owned operations API/read-model layer with provenance, review-gated workflow packets, audit/metrics/events, BI projections, and explicit safety boundaries.

Use this narration:

"This is the project in one frame: Gingr or another provider PMS is evidence, not product authority. The owned layer is where NVA can preserve provenance, review cleanup work, capture labor outcomes, and give BI cleaner read-model concepts without letting automation perform live customer, provider, payment, schedule, or medical actions."

## Expected demo anchors

These are the anchors expected from `./scripts/demo_owned_operations_api.sh` after pre-flight. Treat them as expected local-demo markers, not as fresh proof unless the command is actually run in the meeting.

Contract lane:

- `openapi_title=NVA Pet Resorts Owned Operations API`
- `openapi_version=0.1.0`
- `openapi_paths=8`
- `owned_route=/v0/agent/context/data-quality-hygiene`
- `owned_route=/v0/agent/drafts/data-quality-hygiene`
- `owned_route=/v0/data-quality-hygiene/actions/{action_id}/outcome`
- `owned_route=/v0/data-quality-hygiene/outcomes/summary`
- `owned_route=/v0/ops/metrics/summary`
- `owned_route=/v0/read-models/source-quality-backlog`
- `contract_lane_ok live_side_effects_allowed=false`

Workflow lane:

- `context_ok workflow=data-quality-hygiene actions=1 estimated_minutes_saved=15 live_side_effects_allowed=false`
- `draft_validation_ok accepted_actions=1 requested_side_effects=0`
- `blocked_draft_validation_ok blocked_side_effect=send_customer_message`
- `outcome_ok estimated_minutes_saved=15 actual_minutes_saved=17 live_side_effects_allowed=false`
- `smoke_assertions_ok estimated_minutes_saved=15 actual_minutes_saved=17`

Operations lane:

- `test result: ok. 5 passed; 0 failed`
- `[data-quality-hygiene-worker-outbox-smoke] disabled worker/outbox proof passed as local internal handoff only`

Wrapper close:

- `demo_owned_operations_api_ok local_fixture_only=true live_side_effects_allowed=false`

## Checked OpenAPI artifact path

Use this file when someone asks for contract evidence beyond prose:

```text
apps/api/openapi/owned-operations-v0.openapi.json
```

The pre-flight expectation is title `NVA Pet Resorts Owned Operations API`, version `0.1.0`, and 8 paths, including Data-Quality Hygiene context/draft/outcome routes, local ops metrics, and the source-quality backlog read model.

## Smoke script paths

Primary wrapper:

```sh
./scripts/demo_owned_operations_api.sh
```

Separate lane scripts:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
```

Docs verification gates:

```sh
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
```

## Final commit/status check instructions

Before presenting, run this from the repo root:

```sh
./scripts/demo_owned_operations_api.sh
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
grep -qi '<script' docs/presentation/assets/owned-operations-api-replacement.html && echo 'unexpected script tag' && exit 1 || echo 'html_static_no_script=true'
git status --short
```

If the status is clean, cite the final commit hash. If there are expected uncommitted presentation files, say that the local pre-flight passed on the current checkout and name the changed files rather than implying a committed release.

## Safety boundaries to keep explicit

What is real now:

- local/fixture-only Data-Quality Hygiene proof;
- checked OpenAPI artifact and local demo wrapper;
- review-gated cleanup packet, blocked unsafe action, labor-outcome evidence, and disabled worker/outbox posture.

What not to claim:

- no live NVA/Gingr credentials or production data;
- no provider/PMS writes;
- no customer/member sends;
- no payment, refund, discount, schedule, capacity, or medical/safety decisions;
- no production deployment, complete Gingr replacement, or live BI cutover.

Close the fallback with:

"The credible part is not that I can make a shell scroll; it is that the repo names where automation stops, proves the local contract shape, and asks for read-only validation before any live integration claim."
