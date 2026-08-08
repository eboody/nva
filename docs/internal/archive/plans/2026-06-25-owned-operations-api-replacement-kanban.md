# Owned Operations API Replacement Kanban Plan

## Strategic correction

The goal is **not** to mirror Gingr exactly. Gingr is the incumbent source/system with painful operational and BI gaps. The goal is to design and implement an **owned Pet Resorts operations API** that does the things NVA actually needs: source-backed operational records, useful workflow APIs, review gates, logging, metrics, audit/outbox posture, and BI-ready read models. Gingr becomes one possible legacy/source adapter during migration, not the product authority.

Current business context from the user: the BI team is already pulling data out of Gingr into their own database because Gingr is not sufficient. That is the opening. A useful owned API should make that extract-and-patch workaround unnecessary over time by giving operations, BI, agents, and future integrations a product-owned contract.

## Board objective

Create and begin a comprehensive serialized Kanban board for `/home/eran/code/nva` that turns the existing access-constrained prototype into a credible plan and first implementation path for an owned operations API that could eventually replace Gingr for the workflows NVA cares about.

## Non-goals and safety boundaries

- Do **not** claim live replacement of Gingr today.
- Do **not** require live NVA/Gingr credentials.
- Do **not** perform provider/PMS writes, customer sends, payments/refunds/discounts, schedule changes, or production deployment.
- Do **not** mirror raw Gingr DTOs as the owned API. Provider DTOs remain evidence/adapters only.
- Do **not** introduce a broad ORM/storage rewrite as a prerequisite.
- Shared checkout is `/home/eran/code/nva`; serialize mutating work unless explicit isolated worktrees are created later.

## Desired end state

A reviewer can see:

1. A documented owned API thesis: why it is not a Gingr clone, which jobs it owns, and how it reduces BI/operator work.
2. A source-to-owned-contract migration map: Gingr/provider data enters as source evidence, then promotes into owned entities/workflows/read models.
3. A concrete v0 API surface with OpenAPI/schema direction and product-owned DTOs.
4. A logging/metrics/audit design that answers operational and BI questions directly.
5. A first vertical slice that extends the existing Data-Quality Hygiene proof toward the owned API replacement story.
6. A demo/runbook showing how this API makes the current BI workaround less necessary.

## Proposed serialized graph

1. Strategy/source inventory: owned API replacement thesis and BI pain mapping.
2. Domain/product API contract: entities, resources, workflows, and non-Gingr boundaries.
3. Observability/metrics/audit contract: logs, correlation, business metrics, BI-readiness.
4. OpenAPI/schema and route plan: publishable API contract for v0.
5. Storage/read-model plan: operational write model plus BI-friendly read projections.
6. Implementation slice A: API schema/DTO scaffold for owned operations API.
7. Implementation slice B: persistence/repository adapter for one workflow/read-model proof.
8. Implementation slice C: logging/metrics/correlation improvements tied to the API contract.
9. Migration/adapter slice: Gingr-as-source adapter map, import boundaries, replacement roadmap.
10. Demo/runbook package: how to present “we can replace the BI workaround / Gingr pain over time.”
11. Final review: gates, commit, push, and presentation handoff.

## Verification standards

At minimum for code-changing cards:

```sh
cargo fmt --check
cargo test --workspace
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
```

When schema/API docs are added, also run any local schema validation or generation command introduced by the board. Secret scans must verify changed diffs do not include tokens, signed URLs, raw credentials, or live customer/provider data.

## Board creation note

Cards should preserve the user's strategic clarification: build what NVA needs so Gingr becomes unnecessary, rather than cloning Gingr. The BI workaround is evidence that NVA needs an owned operational data/API layer, not merely better provider DTO coverage.
