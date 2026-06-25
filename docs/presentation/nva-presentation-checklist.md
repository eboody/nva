# NVA presentation checklist

Use this as the live run sheet for a job or networking conversation. The story is: no live NVA/Gingr access was available, so the repo proves the safer first step — a local NVA-owned operations API/read-model/workflow layer with source evidence, review gates, labor outcomes, audit/outbox posture, and disabled live side effects.

## 1. Pre-flight from repo root

Run these before the conversation:

```sh
./scripts/demo_owned_operations_api.sh
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
git status --short
```

Optional if you want to rehearse the separate lanes:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
```

## 2. Open these first, in order

1. `docs/presentation/nva-demo-executive-brief.md` — start here for the pitch.
2. `docs/presentation/owned-operations-api-visual-guide.md` or `docs/presentation/assets/owned-operations-api-replacement.html` — show the one-frame thesis.
3. `docs/presentation/job-presentation-walkthrough.md` — use for the five-minute path and demo anchors.
4. `docs/presentation/owned-operations-api-replacement-talk-track.md` — use when asked why this is not just Gingr or BI.
5. `docs/presentation/nva-demo-skeptical-review.md` — keep nearby for objections and caveats.
6. `apps/api/openapi/owned-operations-v0.openapi.json` — open only if the audience asks for contract evidence.

## 3. 30-second pitch

"I did not have live NVA or Gingr access, so I did not fake a production integration. I built the safe seam NVA would need first: an owned operations API proof that turns source evidence into reviewable cleanup work, records labor outcomes, keeps live side effects disabled, and gives BI cleaner read-model concepts. Gingr is treated as source evidence, not product authority. The next useful step is read-only validation against real docs, exports, or sample data."

## 4. 2-minute pitch

"I would frame this as: do not clone Gingr. Gingr may be the incumbent source system, but NVA's product need is bigger than provider tables. Operations and BI need source provenance, review queues, labor outcomes, audit lineage, and read models that answer resort-work questions directly.

Because I did not have live credentials or production data, I built a local proof instead of overclaiming. The Data-Quality Hygiene slice shows the shape: questionable source facts become reviewable cleanup candidates, safe internal recommendations pass validation, unsafe side effects are rejected, reviewed outcomes record actual labor evidence, and outbox-shaped work remains disabled.

That matters because it moves cleanup and reporting meaning upstream. BI should not have to reverse-engineer NVA business meaning from raw provider exports forever. The owned API gives BI and operations a cleaner contract while keeping source refs and caveats visible.

The right next step is not production access or live writes. It is read-only validation: docs, exports, sample data, source snapshots, and BI query inventory. Then we can compare the owned read models against real source shape, find gaps, and pilot one durable workflow with review and audit still in front of every risky action."

## 5. Demo command and what to watch for

Default live command:

```sh
./scripts/demo_owned_operations_api.sh
```

Expected anchors:

- `openapi_title=NVA Pet Resorts Owned Operations API`
- `openapi_paths=8`
- `contract_lane_ok live_side_effects_allowed=false`
- `context_ok workflow=data-quality-hygiene actions=1 estimated_minutes_saved=15 live_side_effects_allowed=false`
- `draft_validation_ok accepted_actions=1 requested_side_effects=0`
- `blocked_draft_validation_ok blocked_side_effect=send_customer_message`
- `outcome_ok estimated_minutes_saved=15 actual_minutes_saved=17 live_side_effects_allowed=false`
- `smoke_assertions_ok estimated_minutes_saved=15 actual_minutes_saved=17`
- `test result: ok. 5 passed; 0 failed`
- `[data-quality-hygiene-worker-outbox-smoke] disabled worker/outbox proof passed as local internal handoff only`
- `demo_owned_operations_api_ok local_fixture_only=true live_side_effects_allowed=false`

Narrate the three lanes:

1. Contract lane: checked OpenAPI boundary and owned routes.
2. Workflow lane: source-quality issue -> reviewable cleanup packet -> safe draft validation -> blocked unsafe action -> reviewed outcome and labor evidence.
3. Operations lane: worker/outbox posture is disabled/local, not a live sender.

## 6. If the demo fails or is too slow

Do not troubleshoot live for more than a minute. Switch to this fallback:

1. Say: "The command passed in pre-flight; if the local shell is slow, I can show the expected anchors and the files behind them."
2. Open `docs/presentation/job-presentation-walkthrough.md#local-demo-slice-data-quality-hygiene-in-five-minutes`.
3. Point to the checked contract at `apps/api/openapi/owned-operations-v0.openapi.json`.
4. Point to the smoke scripts: `scripts/smoke_data_quality_hygiene_local_loop.sh` and `scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh`.
5. Continue the story: local proof, disabled side effects, read-only validation next.

## 7. What not to claim

Avoid these exact overclaims:

- Do not say "integrated with live NVA/Gingr" or "we replaced Gingr."
- Do not say "production-ready" or "production deployed."
- Do not say "the API is backed by production Postgres today"; say the local API is in-memory for the demo and Postgres migration/storage proof is present.
- Do not say "BI can turn off its database now"; say BI could consume cleaner owned read models after durable wiring and dual-run validation.
- Do not say "agents can contact customers" or "the worker sends/repairs anything"; outbox-shaped work is disabled/review-gated.
- Do not claim provider/PMS writes, customer/member sends, payments/refunds/discounts, schedule/capacity changes, medical/safety decisions, production data, or production deployment.
- Do not claim observability is complete; local tracing/correlation/metrics exist, durable traces, queue/dead-letter views, alerting, auth/location scope, worker leasing, object storage, and rollback remain future work.

## 8. Read-only/sample-data access to ask for

Ask for approved, narrow, read-only validation material only:

- endpoint/report docs for the current Gingr/NVA data paths;
- redacted exports or sample source snapshots;
- provider ID/status/service-line mapping examples;
- BI query inventory and current reporting database assumptions;
- owner-approved KPI definitions, retention/redaction expectations, and location scope;
- permission to compare owned read models against current reports in a dual-run, still with live writes and sends disabled.

Phrase it as: "I am not asking for production credentials or write access. I am asking for enough read-only shape to validate mappings and pick the first safe pilot."

## 9. Final evidence list

Use this closeout list when asked "what proves it?"

- Presentation path: `README.md#presentation-path-safe-local-owned-api-proof`.
- Checklist: `docs/presentation/nva-presentation-checklist.md`.
- Executive brief: `docs/presentation/nva-demo-executive-brief.md`.
- Visual guide/diagram: `docs/presentation/owned-operations-api-visual-guide.md` and `docs/presentation/assets/owned-operations-api-replacement.html`.
- Walkthrough and talk track: `docs/presentation/job-presentation-walkthrough.md`, `docs/presentation/owned-operations-api-replacement-talk-track.md`.
- Skeptical review: `docs/presentation/nva-demo-skeptical-review.md`.
- Checked OpenAPI artifact: `apps/api/openapi/owned-operations-v0.openapi.json`.
- Demo command: `./scripts/demo_owned_operations_api.sh`.
- Verification gates: `./scripts/check_docs.sh` and `python scripts/check_markdown_links.py --repo-root .`.
- Current commit/status: run `git status --short` and cite the final commit after the presentation-polish work is committed.

## 10. Closing line

"This is credible because it says exactly where automation stops: local contract proof now, read-only validation next, and no live side effects until the approval path and system-of-record authority are real."
