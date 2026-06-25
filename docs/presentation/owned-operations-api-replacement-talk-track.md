# Owned operations API replacement talk track

Status: presentation packet for explaining the current repo as an access-constrained proof of an owned Pet Resorts operations API, not a Gingr clone and not a production replacement claim. This page uses only local/fixture-safe commands. It does not claim live NVA/Gingr credentials, production data, provider/PMS writes, customer/member sends, payment/refund/discount movement, schedule/capacity changes, medical/safety decisions, or production deployment.

Use this when a job contact, operator, or BI stakeholder asks: "If Gingr is the incumbent system and BI already has a separate database, what would this API own?"

Short answer: Gingr is source evidence. BI's workaround proves NVA needs cleaner operational contracts. The owned API should own source refs, review queues, workflow events, outcomes, audit, outbox posture, metrics, and BI-ready read models so operations and analytics stop reverse-engineering product meaning from raw provider tables.

## One-minute thesis

"I would not replace Gingr by cloning Gingr. Gingr is useful source evidence during migration, but provider tables are not the operating model NVA actually needs. The repo is building an owned Pet Resorts operations API: source-backed workflow packets, review gates, outcome/labor records, append-only audit, disabled outbox posture, request/workflow correlation, and BI-friendly read models."

"The business clue is that BI is already pulling data from Gingr into a separate database. That means reporting and operations need normalized answers Gingr does not provide cleanly enough. The product move is to push those answers upstream into a supported owned API/read-model layer, with provenance and caveats, instead of making every downstream analyst repair raw provider data again."

## What the owned API needs to provide

| Need | Owned API answer | Current proof path | Honest caveat |
| --- | --- | --- | --- |
| Why Gingr is not enough | Provider payloads are source evidence, not workflow authority; unsupported or dirty facts stay visible as gaps. | [Owned API replacement thesis](../architecture/owned-operations-api-replacement.md), [runtime boundaries](../architecture/runtime-contract-boundaries.md), [Gingr migration map](../integrations/gingr/owned-api-migration-map.md). | No live Gingr/NVA access is used, and this does not prove all provider surfaces. |
| What API NVA needs | Product-owned contracts for sources, data quality, customers/pets/reservations, review queues, outcomes/labor, audit/outbox, BI read models, and readiness metrics. | [Owned API contract families](../architecture/owned-operations-api-contract.md), [OpenAPI plan](../architecture/owned-api-openapi-plan.md), [API shell](../../apps/api/src/http.rs). | DTOs are still mostly Rust/private/local-demo contracts; published client/OpenAPI hardening is ongoing. |
| What runs now | Data-Quality Hygiene local loop and disabled worker/outbox proof run against fixture/local state. | [Data Quality Hygiene workflow](../workflows/operator/data-quality-hygiene.md), [job walkthrough](job-presentation-walkthrough.md). | Local/in-memory API state remains separate from production Postgres runtime wiring. |
| What metrics/logging prove | Request/correlation fields, safe readiness posture, local aggregate labor/outbox metrics, reviewed outcomes, and storage/read-model shape. | [Observability contract](../architecture/owned-api-observability-metrics-contract.md), [readiness gap map](../audits/dto-api-db-observability-readiness-gap-map.md), [storage/read-model cutline](../architecture/owned-api-storage-read-model-cutline.md). | Durable traces, worker leasing, dead-letter views, alerting, and production dashboards are gaps. |
| What real access unlocks next | Approved read-only imports, source snapshot validation, BI query inventory, dual-run comparison, and scoped workflow replacement. | [Gingr migration map](../integrations/gingr/owned-api-migration-map.md), [owned API plan](../plans/2026-06-25-owned-operations-api-replacement-kanban.md). | Real credentials, production data handling, KPI ownership, retention/redaction, and live action approvals remain human gates. |

## First runnable demo: Data-Quality Hygiene

Keep Data-Quality Hygiene as the first live-at-keyboard demo because it connects the BI workaround to the owned API story:

- Bad provider/source facts become visible data-quality issues instead of hidden BI cleanup.
- Source refs, issue refs, freshness, sensitivity, review gates, and blocked actions stay attached to each candidate.
- Draft cleanup work is validated and unsafe side effects are rejected.
- Reviewed outcomes record estimated/actual labor minutes and source-fact correctness.
- Metrics/readiness expose aggregate local proof while live delivery remains disabled.

From the repo root:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
```

Expected output shape:

```text
[data-quality-hygiene-smoke] ...
context_ok
draft_validation_ok
blocked_draft_validation_ok
outcome_ok
live_side_effects_allowed=false
smoke_assertions_ok estimated_minutes_saved=... actual_minutes_saved=...
[data-quality-hygiene-worker-outbox-smoke] disabled worker/outbox proof passed as local internal handoff only
```

What those markers mean in plain English:

1. `context_ok`: the API/app can assemble a source-backed cleanup packet.
2. `draft_validation_ok`: an internal cleanup recommendation can pass review-safe validation.
3. `blocked_draft_validation_ok`: unsafe requests such as source repair, customer sends, or ambiguity hiding are rejected.
4. `outcome_ok`: reviewed staff disposition and labor evidence can be recorded.
5. `live_side_effects_allowed=false`: the demo did not send messages, mutate Gingr/PMS, move money, change schedules, or deploy production code.
6. Worker/outbox smoke: any outbox-shaped work is a disabled/review-gated handoff, not a live sender.

## Suggested five-minute flow

### 0:00-0:45 — Start from the business gap

"Gingr may be the incumbent source system, but BI already needs a separate data path because raw provider shape is not enough. That is the product signal. NVA needs an operations API that speaks resort work: source provenance, review queues, outcomes, labor metrics, audit, and read models."

Point to:

- [Owned operations API replacement thesis](../architecture/owned-operations-api-replacement.md)
- [Owned API contract families](../architecture/owned-operations-api-contract.md)
- [Gingr adapter to owned API migration map](../integrations/gingr/owned-api-migration-map.md)

### 0:45-1:45 — Separate provider evidence from product authority

"A Gingr DTO or report says what a source emitted. It does not define NVA's canonical workflow, review gate, labor metric, or BI projection. The API boundary keeps provider IDs and raw/unknown fields quarantined as source evidence. Product-owned resources carry semantic IDs, source refs, review requirements, outcome records, and audit lineage."

Memorize:

"Provider DTOs are adapter evidence; owned API DTOs are NVA operating contracts."

Point to:

- [Runtime contract boundaries](../architecture/runtime-contract-boundaries.md)
- [Gingr provider DTO boundary](../../integrations/gingr/src/dto/README.md)
- [Storage/read-model cutline](../architecture/owned-api-storage-read-model-cutline.md)

### 1:45-3:15 — Run the Data-Quality Hygiene demo

Run:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
```

Narrate:

"This is the smallest replacement-shaped loop. The API does not repair Gingr. It turns questionable source facts into reviewable cleanup work, validates the safe draft, rejects forbidden side effects, records a reviewed outcome, and keeps worker/outbox execution disabled. That is how you reduce the BI cleanup burden without pretending raw provider data is business truth."

### 3:15-4:15 — Explain metrics, logging, and BI proof

"The proof is not only route existence. The useful signals are source-quality backlog fields, import freshness, outcome/labor minutes, request/workflow/review/outbox correlation, append-only audit posture, and aggregate metrics that say live side effects are disabled. Those are the kinds of read models BI should consume instead of reverse-engineering provider tables."

Point to:

- [Observability/metrics/audit contract](../architecture/owned-api-observability-metrics-contract.md)
- [DTO/API/DB/observability gap map](../audits/dto-api-db-observability-readiness-gap-map.md)
- [Data Quality Hygiene workflow](../workflows/operator/data-quality-hygiene.md)

### 4:15-5:00 — Close with replacement ladder and caveats

"The current repo is architecture/demo-ready, not production replacement. The ladder is: local owned contracts, then durable Postgres-backed v0 proof, then BI read-model pilot, then adapter dual-run with approved read-only access, then scoped workflow replacement, and only much later provider shrinkage. Anything involving live writes, customer sends, payments, scheduling, medical/safety decisions, or production data needs explicit owner approval."

Point to:

- [Replacement readiness ladder](../architecture/owned-operations-api-replacement.md#replacement-readiness-ladder)
- [Gingr migration phases](../integrations/gingr/owned-api-migration-map.md#migration-phases)

## Likely questions and answers

### "Are you replacing Gingr today?"

"No. Today this is an honest local/demo and architecture proof. It shows how an owned API would reduce dependence on Gingr over time by moving source refs, review queues, outcome/labor evidence, audit, and read models into product-owned contracts. Real replacement requires approved access, durable persistence, auth, monitoring, owner-defined BI metrics, dual-run comparison, and explicit live-action gates."

### "Why is this better than BI just pulling Gingr data?"

"BI pulls can answer some reporting questions, but they usually have to infer meaning after the fact: which source fields are stale, which statuses map to operations, which records were reviewed, and which labor was actually saved. The owned API makes those things first-class: source-quality issues, review status, outcomes, labor minutes, audit lineage, import freshness, and caveats. BI still matters, but it gets a cleaner upstream contract."

### "What API routes or read models would NVA call?"

"For v0, I would show Data-Quality Hygiene and read models such as source-quality backlog, labor outcomes, review queue aging, import freshness, outbox posture, and audit lineage. The API contract page sketches routes like `GET /read-models/source-quality-backlog`, `GET /read-models/labor-outcomes`, `GET /read-models/audit-lineage`, plus existing local Data-Quality Hygiene context/draft/outcome/summary routes."

### "What proves logging and metrics?"

"Current proof includes JSON tracing startup, request-id/correlation fields in the API shell, readiness and local metrics payloads, Data-Quality Hygiene outcome summaries, storage/read-model fields, and worker/outbox telemetry posture. It does not yet claim durable traces, dashboards, worker leasing, dead-letter metrics, or alerting."

### "What would real NVA access unlock?"

"First, read-only validation: what exact Gingr endpoints/reports feed BI, source grain, location scope, provider ID stability, status mappings, service-line meanings, payment/care/document sensitivity, and webhook freshness. Then a dual-run: compare owned read models against the existing BI path, with gaps becoming explicit source-quality issues rather than guesses. Live writes or sends remain separate approval-controlled adapter work."

## Claims to avoid

- Do not say "we replaced Gingr." Say "we have an owned API replacement path and a local first slice."
- Do not say "BI can turn off its database now." Say "BI could consume owned read models after durable wiring, approved source imports, and dual-run validation."
- Do not say "the API is production-ready." Say "the owned contracts and local proof are presentation-ready; durable runtime, auth, monitoring, and access remain gaps."
- Do not say "the worker sends/repairs anything." Say "outbox-shaped work is disabled/review-gated in the current proof."
- Do not say "Gingr fields are the API schema." Say "Gingr fields are adapter/source evidence promoted only through validated owned contracts."

## Verification commands for this packet

Use these when updating the presentation docs:

```sh
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
./scripts/smoke_data_quality_hygiene_local_loop.sh
./scripts/smoke_data_quality_hygiene_disabled_worker_outbox.sh
git status --short
```
