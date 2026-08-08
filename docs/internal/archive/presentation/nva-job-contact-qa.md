# NVA job-contact Q&A

Use this as a skim-friendly objection-response sheet for a recruiter, job contact, hiring manager, product/ops leader, or skeptical technical stakeholder. It describes a safe local proof only. It does not claim live NVA/Gingr access, production data, production deployment, provider/PMS writes, customer/member sends, payment/refund/discount actions, schedule/capacity changes, medical/safety decisions, or a completed Gingr replacement.

## Quick answers

### How was this possible without access?

I did not fake live access. I built the safer first step: a local, fixture-backed proof of the NVA-owned operations API/read-model/workflow seam that should exist before connecting to live systems. It proves the contract shape, review gates, blocked side effects, labor-outcome records, BI-friendly read models, and a fixture-backed role/location-scoped realtime queue; approved read-only source material would validate real field mappings and operating-scope mappings later.

### Is it production-ready?

No. It is presentation-ready architecture and a runnable local contract/realtime proof. Production still needs approved source access, durable Postgres-backed routes and/or reducer-to-archive flow, production SSO/auth and location scope, monitoring and rollback, worker leasing/dead-letter handling, owner-approved KPI definitions, retention/redaction rules, and explicit approval for any live side effect.

### Why is this better than BI reports alone?

BI reports are still valuable, but reports often infer business meaning after the fact from provider-shaped data. The owned API makes source quality, review status, labor outcomes, audit lineage, import freshness, and caveats first-class upstream concepts, so BI consumes cleaner NVA-owned read models instead of rediscovering the same cleanup logic downstream.

### Why does realtime/authz matter?

For a portfolio with roughly 170 locations, the useful product is not just another report someone refreshes. Staff should see only the work relevant to their location and role, managers should see manager-gated items immediately, and unsafe or out-of-scope attempts should fail closed with audit evidence. The SpacetimeDB spike demonstrates that operating substrate with fixture actors: Alice at Location 101 sees her queue update, Sam at Location 202 does not see or mutate Location 101 work, and Morgan sees manager-scoped work. That is the reason realtime and authorization scope belong in the story.

### Why not just clone Gingr?

Because Gingr is source evidence, not the product authority NVA needs to own. A clone would copy provider screens and tables; this proof models the operating contracts around them: review queues, allowed/blocked actions, labor evidence, audit/outbox posture, BI read models, and realtime scoped operating views. Gingr or another PMS can remain a source/system of record while NVA owns the workflow meaning around it.

### Is SpacetimeDB replacing the business logic?

No. The boundary discipline is that `domain` and `app` still own vocabulary, invariants, workflow policy, review gates, and blocked-action rules. SpacetimeDB is a storage/runtime adapter: it holds private rows and public subscription read models, runs thin reducer adapters, and publishes safe realtime views. If a reducer needs business meaning, it should call or mirror app-owned policy rather than inventing a second source of truth.

### Do you still need Postgres and S3?

Yes for the enterprise shape, unless future proof gates show otherwise. SpacetimeDB is for live reducers/subscriptions and low-latency queue coordination. Postgres is still the durable audit/history/reporting/reconciliation/export ledger. S3 or MinIO stores immutable source snapshots, documents, media, export bundles, hashes, and manifests. This is not a sunk-cost answer; each retained layer has a distinct job.

### What access is needed next?

Read-only validation material, not production credentials or write access: endpoint/report docs, redacted exports or sample source snapshots, provider ID/status/service-line mapping examples, BI query inventory, owner-approved KPI definitions, and location/retention/redaction expectations. That is enough to compare the local read models against real source shape and scope a safe dual-run pilot.

### What job capability does it demonstrate?

It demonstrates product-engineering judgment under constrained access: separating source evidence from product authority, turning ambiguous legacy/provider data into owned contracts, building a runnable proof instead of slideware, preserving safety boundaries, explaining the business value in operator language, and naming the next validation step without overclaiming.

### What are the safety/risk boundaries?

The current package is local and fixture-backed. Realtime auth/role/location scope uses demo actors, not production SSO. It uses no live NVA/Gingr credentials, no production data, and no production deployment. It does not perform provider/PMS writes, customer/member sends, payment/refund/discount movement, schedule/capacity changes, medical/safety decisions, or live operational automation. Outbox-shaped work is disabled/review-gated until a separate approved adapter path exists.

### What should the job contact forward or say?

Forward the job-contact summary and, if useful, this Q&A. A short intro is:

> This is a safe local proof of an NVA-owned operations API/read-model layer above provider systems like Gingr. It shows how source evidence can become reviewable cleanup work, BI-ready concepts, labor-outcome records, audited/disabled side-effect posture, and a realtime role/location-scoped queue without pretending to have live access. The right next step is narrow read-only validation against approved docs, exports, sample data, or BI query inventory — not production credentials or live writes.

## Best supporting links

- [Sendable job-contact summary](nva-sendable-job-contact-summary.md)
- [Three-minute presentation script](nva-3-minute-presentation-script.md)
- [Final presentation checklist](nva-presentation-checklist.md)
- [Skeptical review and objection scan](nva-demo-skeptical-review.md)
- [Owned operations API replacement talk track](owned-operations-api-replacement-talk-track.md)
- [Checked OpenAPI artifact](../../apps/api/openapi/owned-operations-v0.openapi.json)
- [SpacetimeDB realtime queue demo runbook](../ops/spacetimedb-realtime-queue-demo.md)
- [Audit/reporting/evidence backbone](../architecture/audit-reporting-evidence-backbone.md)
