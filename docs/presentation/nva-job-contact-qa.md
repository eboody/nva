# NVA job-contact Q&A

Use this as a skim-friendly objection-response sheet for a recruiter, job contact, hiring manager, product/ops leader, or skeptical technical stakeholder. It describes a safe local proof only. It does not claim live NVA/Gingr access, production data, production deployment, provider/PMS writes, customer/member sends, payment/refund/discount actions, schedule/capacity changes, medical/safety decisions, or a completed Gingr replacement.

## Quick answers

### How was this possible without access?

I did not fake live access. I built the safer first step: a local, fixture-backed proof of the NVA-owned operations API/read-model/workflow seam that should exist before connecting to live systems. It proves the contract shape, review gates, blocked side effects, labor-outcome records, and BI-friendly read models; approved read-only source material would validate real field mappings later.

### Is it production-ready?

No. It is presentation-ready architecture and a runnable local contract proof. Production still needs approved source access, durable Postgres-backed routes, auth/location scope, monitoring and rollback, worker leasing/dead-letter handling, owner-approved KPI definitions, retention/redaction rules, and explicit approval for any live side effect.

### Why is this better than BI reports alone?

BI reports are still valuable, but reports often infer business meaning after the fact from provider-shaped data. The owned API makes source quality, review status, labor outcomes, audit lineage, import freshness, and caveats first-class upstream concepts, so BI consumes cleaner NVA-owned read models instead of rediscovering the same cleanup logic downstream.

### Why not just clone Gingr?

Because Gingr is source evidence, not the product authority NVA needs to own. A clone would copy provider screens and tables; this proof models the operating contracts around them: review queues, allowed/blocked actions, labor evidence, audit/outbox posture, and BI read models. Gingr or another PMS can remain a source/system of record while NVA owns the workflow meaning around it.

### What access is needed next?

Read-only validation material, not production credentials or write access: endpoint/report docs, redacted exports or sample source snapshots, provider ID/status/service-line mapping examples, BI query inventory, owner-approved KPI definitions, and location/retention/redaction expectations. That is enough to compare the local read models against real source shape and scope a safe dual-run pilot.

### What job capability does it demonstrate?

It demonstrates product-engineering judgment under constrained access: separating source evidence from product authority, turning ambiguous legacy/provider data into owned contracts, building a runnable proof instead of slideware, preserving safety boundaries, explaining the business value in operator language, and naming the next validation step without overclaiming.

### What are the safety/risk boundaries?

The current package is local and fixture-backed. It uses no live NVA/Gingr credentials, no production data, and no production deployment. It does not perform provider/PMS writes, customer/member sends, payment/refund/discount movement, schedule/capacity changes, medical/safety decisions, or live operational automation. Outbox-shaped work is disabled/review-gated until a separate approved adapter path exists.

### What should the job contact forward or say?

Forward the job-contact summary and, if useful, this Q&A. A short intro is:

> This is a safe local proof of an NVA-owned operations API/read-model layer above provider systems like Gingr. It shows how source evidence can become reviewable cleanup work, BI-ready concepts, labor-outcome records, and audited/disabled side-effect posture without pretending to have live access. The right next step is narrow read-only validation against approved docs, exports, sample data, or BI query inventory — not production credentials or live writes.

## Best supporting links

- [Sendable job-contact summary](nva-sendable-job-contact-summary.md)
- [Three-minute presentation script](nva-3-minute-presentation-script.md)
- [Final presentation checklist](nva-presentation-checklist.md)
- [Skeptical review and objection scan](nva-demo-skeptical-review.md)
- [Owned operations API replacement talk track](owned-operations-api-replacement-talk-track.md)
- [Checked OpenAPI artifact](../../apps/api/openapi/owned-operations-v0.openapi.json)
