# NVA demo executive brief

Status: one-page speaking note for an access-constrained job or networking conversation. This brief describes a safe local proof. It does not claim live NVA/Gingr access, production data, provider/PMS writes, customer/member sends, payment movement, schedule changes, medical/safety decisions, or production deployment.

## One-sentence pitch

I built a safe local proof of the operations API NVA would want above Gingr: source-backed cleanup work, review gates, labor-outcome records, audit/outbox posture, and BI-ready read models without pretending to have live access.

## What I built without access

I did not have live NVA/Gingr credentials, production data, or permission to touch provider systems. Instead of faking that, I built the contract shape NVA would need before connecting anything live:

- provider facts stay as source evidence with provenance;
- product-owned workflow packets decide what staff can review;
- unsafe actions stay blocked;
- outcomes record reviewed labor evidence;
- audit, metrics, and outbox posture make the next integration step inspectable.

That boundary is the point. The demo shows judgment as much as code: build the useful seam first, then ask for narrow read-only validation.

## Why Gingr is source evidence, not product authority

Gingr can say what a provider system emitted. It should not define NVA's workflow authority, labor metrics, BI projections, or approval gates. The owned API turns provider/source evidence into NVA operating contracts with source refs, review status, caveats, outcomes, and audit lineage.

This is not a Gingr clone and not an AI dashboard standing beside Gingr. It is a piece-meal migration path for NVA to reduce dependence on provider-shaped data over time while BI and operations get cleaner upstream answers.

## Migration spine

The practical sequence is:

1. read-only source evidence;
2. owned workflow authority;
3. BI/read-model replacement;
4. controlled outbox/writeback;
5. workflow-by-workflow replacement.

The current no-access boundary is safe judgment, not a gap to hide. The demo proves the owned seam with fixture evidence before asking anyone for credentials. Real access should start with approved read-only docs, exports, sample rows, source snapshots, field dictionaries, and BI query inventory so the mappings and read models can be validated without live writes.

## What runs locally now

The current runnable slice is Data-Quality Hygiene. From the repo root, run the shortest local demo wrapper:

```sh
./scripts/demo_owned_operations_api.sh
```

Expected anchors include `openapi_title=NVA Pet Resorts Owned Operations API`, `openapi_paths=8`, `contract_lane_ok live_side_effects_allowed=false`, `context_ok workflow=data-quality-hygiene actions=1 estimated_minutes_saved=15 live_side_effects_allowed=false`, `draft_validation_ok accepted_actions=1 requested_side_effects=0`, `blocked_draft_validation_ok blocked_side_effect=send_customer_message`, `outcome_ok estimated_minutes_saved=15 actual_minutes_saved=17 live_side_effects_allowed=false`, `smoke_assertions_ok estimated_minutes_saved=15 actual_minutes_saved=17`, the worker test summary `test result: ok. 5 passed; 0 failed`, and `demo_owned_operations_api_ok local_fixture_only=true live_side_effects_allowed=false`.

If you want to run the workflow and operations lanes separately:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
```

The checked OpenAPI artifact is at [`apps/api/openapi/owned-operations-v0.openapi.json`](../../apps/api/openapi/owned-operations-v0.openapi.json).

## What the demo proves

The demo proves a small but important operating loop:

1. source-quality problems become reviewable internal work instead of hidden BI cleanup;
2. draft recommendations are validated against allowed and blocked actions;
3. unsafe requests such as provider repair, customer sends, schedule/payment movement, medical/safety decisions, and ambiguity hiding are rejected;
4. reviewed outcomes capture labor evidence and source-fact correctness;
5. outbox-shaped work stays disabled until a real approval-controlled adapter exists.

It is presentation-ready architecture and local contract proof, not production replacement.

## What real access unlocks next

The next ask is narrow and read-only: approved docs, exports, sample data, or source snapshots that show the actual Gingr/NVA fields BI and operations rely on. With that, the local contract could become an integration pilot:

- validate source mappings and provider ID stability;
- compare owned read models against current BI queries;
- turn mapping gaps into visible source-quality issues;
- define owner-approved KPI and retention/redaction rules;
- keep live writes, sends, payment, schedule, and medical/safety actions out of scope until separately approved.

## CEO close: cautious pilot scaler

For a 170-location portfolio, I would frame the value as an illustrative scaler to validate, not as a savings claim. A conservative first model is:

| Scenario | Per-location assumption | 170-location illustration | Caveat |
|---|---:|---:|---|
| Read-only validation target | 1–2 staff hours/week | 170–340 staff hours/week | Illustrative only; confirm against real volume, wage bands, and BI definitions. |
| Two repeatable workflows | 3–5 staff hours/week | 510–850 staff hours/week | Still a sizing model, not production ROI; use it to decide whether to instrument a broader pilot. |

The next ask I would make of a job contact or senior operator is intentionally safe:

1. read-only exports or source snapshots for one or two workflows;
2. sample rows with field dictionaries and provenance notes;
3. the current BI query inventory for labor, exceptions, cleanup, conversion, and follow-up metrics;
4. owner-approved KPI definitions plus retention/redaction rules.

A 2–3 week validation frame is enough for the next proof: week 1 maps source fields and BI queries, week 2 compares owned read models against real sample rows, and week 3 runs a reviewed pilot lane if the first comparisons are useful. Live writes, customer messaging, payment/refund actions, schedule/capacity changes, and medical or safety decisions remain out of scope until separately approved.

## 30-second pitch

"I did not have live NVA or Gingr access, so I did not fake a production integration. I built the safe seam NVA would need first: an owned operations API proof that turns source evidence into reviewable cleanup work, records labor outcomes, keeps live side effects disabled, and gives BI cleaner read-model concepts. Gingr is treated as source evidence, not product authority. The next useful step is read-only validation against real docs, exports, or sample data."

## 2-minute pitch

"The way I would frame this is: don't clone Gingr. Gingr may be the incumbent source system, but NVA's product need is bigger than provider tables. Operations and BI need source provenance, review queues, labor outcomes, audit lineage, and read models that answer resort-work questions directly.

Because I did not have live credentials or production data, I built a local proof instead of overclaiming. The Data-Quality Hygiene slice shows the shape: questionable source facts become reviewable cleanup candidates, safe internal recommendations pass validation, unsafe side effects are rejected, reviewed outcomes record actual labor evidence, and outbox-shaped work remains disabled.

That matters because it moves cleanup and reporting meaning upstream. BI should not have to reverse-engineer NVA business meaning from raw provider exports forever. The owned API gives BI and operations a cleaner contract while keeping source refs and caveats visible.

The right next step is not production access or live writes. It is read-only validation: docs, exports, sample data, and BI query inventory. Then we can compare the owned read models against real source shape, find gaps, and pilot one durable workflow with review and audit still in front of every risky action."

## Quick links

- [Manager brief demo script](intelligible-manager-brief-demo-script.md)
- [Migration spine note](owned-backend-migration-spine.md)
- [README presentation path](../../README.md#presentation-path-safe-local-owned-api-proof)
- [Final presentation checklist](nva-presentation-checklist.md)
- [Local demo walkthrough](../demo/local-demo-walkthrough.md)
- [Owned operations API visual guide](owned-operations-api-visual-guide.md)
- [Standalone HTML/SVG visual](assets/owned-operations-api-replacement.html)
- [Job presentation walkthrough](job-presentation-walkthrough.md)
- [Owned operations API replacement talk track](owned-operations-api-replacement-talk-track.md)
- [Skeptical review and objection scan](nva-demo-skeptical-review.md)
- [Owned operations API replacement thesis](../architecture/owned-operations-api-replacement.md)
- [Owned API contract families](../architecture/owned-operations-api-contract.md)
