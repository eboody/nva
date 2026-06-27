# NVA job-contact summary

I built a safe local proof of the operations layer NVA Pet Resorts would want above provider systems like Gingr: an NVA-owned API/read-model/workflow seam that turns source evidence into reviewable work, records labor outcomes, and keeps risky live actions disabled. It is designed to be easy to forward to a job contact, hiring manager, product/operations leader, or skeptical engineer.

## What is this?

This repo is a working local demonstration of an owned operations layer for pet-resort workflows. The current runnable slice is Data-Quality Hygiene: source-quality issues become internal review packets, safe recommendations pass validation, unsafe side effects are blocked, and reviewed outcomes capture labor evidence. The newest architecture spike adds a SpacetimeDB realtime runtime adapter so staff and managers can see role/location-scoped queue updates instead of refreshing static reports.

Gingr is treated as source evidence, not product authority. SpacetimeDB is also not a business-logic rewrite: domain and app contracts still own vocabulary, rules, review gates, and blocked-action policy. The runtime adapter's job is to project safe queue/read-model state, enforce actor/location scope for the demo, and make the operating loop feel live for a portfolio that could span roughly 170 locations.

## Why it matters to NVA / Pet Resorts operations

Provider exports and reporting databases can show what happened, but they usually do not own the workflow meaning: which facts are trustworthy, which cleanup work needs review, which actions are unsafe, how staff time was saved, which manager/location can act, or what BI should count. An owned operations layer gives operations and BI cleaner concepts while preserving source refs, caveats, review status, role/location scope, and outcomes.

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

The checked OpenAPI artifact is at [`apps/api/openapi/owned-operations-v0.openapi.json`](../../apps/api/openapi/owned-operations-v0.openapi.json), and the presentation run sheet is [`docs/presentation/nva-presentation-checklist.md`](nva-presentation-checklist.md). The realtime queue presenter script is:

```sh
scripts/spacetimedb_realtime_queue_demo.sh --self-test
scripts/spacetimedb_realtime_queue_demo.sh --force-fallback
```

That script currently documents a local SpacetimeDB ABI mismatch and falls back to a deterministic event-stream demo. Treat the fallback as evidence of the intended reducer/subscription story and scope discipline, not as proof that a live SpacetimeDB module was published.

## What is intentionally not claimed

This is not live NVA/Gingr access, not production data, not a production deployment, not production SSO/authz, and not a replacement of Gingr. It uses fixture auth/actors for the realtime slice and local-only demo data. It does not perform provider/PMS writes, customer/member sends, payment/refund/discount actions, schedule/capacity changes, medical/safety decisions, live side effects, or any live operational side effect.

The honest claim is stronger: the local proof shows the safe seam NVA would need before connecting anything live, with review gates and disabled side effects visible instead of hand-waved.

## What job/conversation capability it demonstrates

This demonstrates product and engineering judgment under access constraints: define the owned domain boundary, keep runtime adapters from owning business truth, build a runnable contract proof, add a realtime role/location-scoped operating substrate, keep live-risk actions out of scope, explain the value in operator language, and name the next validation step without overclaiming.

It is useful for conversations about product engineering, internal platforms, operations systems, data/BI contracts, AI-assisted workflow safety, and legacy/provider-system modernization.

## Narrow read-only next step

The next useful ask is not production credentials or write access. It is approved read-only validation material: endpoint/report docs, redacted exports or sample source snapshots, provider ID/status/service-line mapping examples, BI query inventory, and owner-approved KPI definitions.

With that, the local read models can be compared against real source shape, mapping gaps can become visible source-quality issues, actor/location scope can be validated against real operating roles, and one safe pilot workflow can be scoped while live writes and sends remain disabled.

## Concise Postgres/S3 answer

SpacetimeDB is the realtime operations runtime: reducers, subscriptions, and scoped staff/manager queue views. Postgres and S3-compatible storage still have distinct enterprise jobs: Postgres keeps durable audit/history/reporting/reconciliation/export facts, while S3/MinIO keeps immutable source/document/media evidence objects and manifests. I would remove Postgres only after SpacetimeDB proves years-scale audit retention, point-in-time historical queries, BI-safe export contracts, source reconciliation, object-evidence handling, backup/restore, and operational tooling.

## Sendable intro

I built a safe local proof of an NVA-owned operations layer above provider systems like Gingr. It demonstrates how source evidence can become reviewable cleanup work, BI-ready concepts, labor-outcome records, audited/disabled side-effect posture, and a realtime role/location-scoped queue substrate without pretending to have live access. SpacetimeDB is the runtime adapter for the live operating loop, not a rewrite of business logic or a claim of production readiness. The right next step is narrow read-only validation against approved docs, exports, sample data, role/location mappings, or BI query inventory — not production credentials or live writes.
