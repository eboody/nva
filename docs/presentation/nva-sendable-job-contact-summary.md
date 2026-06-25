# NVA job-contact summary

I built a safe local proof of the operations layer NVA Pet Resorts would want above provider systems like Gingr: an NVA-owned API/read-model/workflow seam that turns source evidence into reviewable work, records labor outcomes, and keeps risky live actions disabled. It is designed to be easy to forward to a job contact, hiring manager, product/operations leader, or skeptical engineer.

## What is this?

This repo is a working local demonstration of an owned operations API for pet-resort workflows. The current runnable slice is Data-Quality Hygiene: source-quality issues become internal review packets, safe recommendations pass validation, unsafe side effects are blocked, and reviewed outcomes capture labor evidence.

Gingr is treated as source evidence, not product authority. The point is not to clone a provider system; it is to show how NVA could own the operational contracts, approval boundaries, BI-friendly read models, and audit trail around provider data.

## Why it matters to NVA / Pet Resorts operations

Provider exports and reporting databases can show what happened, but they usually do not own the workflow meaning: which facts are trustworthy, which cleanup work needs review, which actions are unsafe, how staff time was saved, or what BI should count. An owned operations layer gives operations and BI cleaner concepts while preserving source refs, caveats, review status, and outcomes.

That matters because the valuable work is not just "pull data from Gingr." It is making resort work safer and easier to measure: fewer hidden cleanup loops, clearer review queues, better source-quality signals, and more honest labor-savings evidence.

## What is real and runnable now

From the repo root, the local demo wrapper is:

```sh
./scripts/demo_owned_operations_api.sh
```

It checks the OpenAPI contract, runs the Data-Quality Hygiene local loop, proves unsafe actions such as customer sends stay blocked, records reviewed labor evidence, and exercises the disabled worker/outbox posture. The expected closeout anchor is:

```text
demo_owned_operations_api_ok local_fixture_only=true live_side_effects_allowed=false
```

The checked OpenAPI artifact is at [`apps/api/openapi/owned-operations-v0.openapi.json`](../../apps/api/openapi/owned-operations-v0.openapi.json), and the presentation run sheet is [`docs/presentation/nva-presentation-checklist.md`](nva-presentation-checklist.md).

## What is intentionally not claimed

This is not live NVA/Gingr access, not production data, not a production deployment, and not a replacement of Gingr. It does not perform provider/PMS writes, customer/member sends, payment/refund/discount actions, schedule/capacity changes, medical/safety decisions, or any live operational side effect.

The honest claim is stronger: the local proof shows the safe seam NVA would need before connecting anything live, with review gates and disabled side effects visible instead of hand-waved.

## What job/conversation capability it demonstrates

This demonstrates product and engineering judgment under access constraints: define the owned domain boundary, build a runnable contract proof, keep live-risk actions out of scope, explain the value in operator language, and name the next validation step without overclaiming.

It is useful for conversations about product engineering, internal platforms, operations systems, data/BI contracts, AI-assisted workflow safety, and legacy/provider-system modernization.

## Narrow read-only next step

The next useful ask is not production credentials or write access. It is approved read-only validation material: endpoint/report docs, redacted exports or sample source snapshots, provider ID/status/service-line mapping examples, BI query inventory, and owner-approved KPI definitions.

With that, the local read models can be compared against real source shape, mapping gaps can become visible source-quality issues, and one safe pilot workflow can be scoped while live writes and sends remain disabled.

## Sendable intro

I built a safe local proof of an NVA-owned operations API/read-model layer above provider systems like Gingr. It demonstrates how source evidence can become reviewable cleanup work, BI-ready concepts, labor-outcome records, and audited/disabled side-effect posture without pretending to have live access. The right next step is narrow read-only validation against approved docs, exports, sample data, or BI query inventory — not production credentials or live writes.
