# Owned operations API visual guide

Status: presenter companion for the standalone visual artifact. This guide explains the diagram and how to narrate it in a job/networking conversation. It does not claim live NVA/Gingr access, production data, provider/PMS writes, customer/member sends, payment movement, schedule changes, medical/safety decisions, or production deployment.

Open the visual artifact here: [owned-operations-api-replacement.html](assets/owned-operations-api-replacement.html).

## What the diagram shows

The diagram compares two operating models:

1. **Today: Gingr-centered extraction workaround.** Gingr or another provider PMS is treated as the operational source. BI extracts and a separate reporting database can answer some questions, but NVA still lacks a product-owned workflow gate. Labor outcomes, data-quality cleanup, and audit lineage are inferred downstream rather than captured as reviewed operations evidence.
2. **Proposed: NVA-owned operations API and read-model layer.** Gingr becomes a source adapter with provenance, source refs, observed timestamps, adapter versions, and visible data-quality issues. The NVA-owned operations API owns reviewable workflow packets, audit/logging/metrics/events, and BI/read-model projections.
3. **Safe demo slice: Data-Quality Hygiene.** The local proof demonstrates one narrow vertical slice: questionable source facts become reviewable cleanup work, draft recommendations are validated, unsafe side effects are rejected, reviewed outcomes record labor evidence, and BI can consume clearer read-model concepts later.

The core thesis is: **do not clone Gingr; build the operations API NVA needs, with Gingr as source evidence during migration.**

## How to narrate it in 60 seconds

"This picture is the whole project in one frame. On the left is the current pain: Gingr or a provider PMS is the source, BI pulls extracts into a separate database, but the actual workflow authority is still not owned by NVA. Operators do cleanup and exception handling, but review gates, labor outcomes, and audit lineage are inferred after the fact.

On the right is the product I would build toward. Gingr is not copied into a new public model; it is narrowed into a source adapter with provenance and caveats. NVA owns the operations API, the review-gated workflow packets, the audit and metrics events, and the BI/read-model projections. The local demo proves the safest first slice: Data-Quality Hygiene. It turns source-quality issues into reviewable internal cleanup work, records reviewed labor evidence, and keeps live side effects disabled.

The important safety line is at the bottom: no live customer sends, no provider/PMS writes, no payment, schedule, or medical decisions, and no production claim until read-only access validates the real mappings."

## Legend

| Visual element | Meaning in the presentation |
| --- | --- |
| Slate source boxes | External/provider evidence such as Gingr records, provider IDs, raw responses, reports, or fixtures. These are evidence, not product authority. |
| Emerald API box | NVA-owned operations API contracts: staff/reviewer routes, product-owned resources, and workflow boundaries. |
| Rose review/safety boxes | Required review gates and explicit blocked-action boundaries before any side effect can happen. |
| Orange/amber event boxes | Audit, logging, metrics, request/workflow/review/outbox IDs, and labor-outcome evidence. |
| Violet read-model boxes | BI/reporting projections over reviewed owned meaning, not raw provider-table mirroring. |
| Dashed boundaries | Separation between today's extraction workaround and tomorrow's owned operations layer. |

## Caveats and next access ask

What runs locally now:

- checked OpenAPI artifact and local demo wrapper;
- Data-Quality Hygiene workflow context, draft validation, blocked-action proof, reviewed outcome capture, and disabled worker/outbox posture;
- local/fixture-only evidence with live side effects disabled.

What this does **not** claim:

- no live NVA/Gingr credentials or production data;
- no provider/PMS writes;
- no customer/member sends;
- no payment, schedule, capacity, or medical/safety decisions;
- no production deployment or full Gingr replacement.

The next access ask is narrow and read-only: approved docs, exports, sample data, source snapshots, or BI query inventory that validate the source mappings and show which owned read models should be piloted first.

## Related presentation path

- [NVA demo executive brief](nva-demo-executive-brief.md)
- [Job presentation walkthrough](job-presentation-walkthrough.md)
- [Owned operations API replacement talk track](owned-operations-api-replacement-talk-track.md)
- [Owned operations API replacement thesis](../architecture/owned-operations-api-replacement.md)
- [Gingr adapter to owned API migration map](../integrations/gingr/owned-api-migration-map.md)
