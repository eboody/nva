# Source-evidence and non-coder usability review — 2026-06-19

Scope reviewed: the contract crosswalk set under [`docs/entity-atlas/contract-crosswalk/`](entity-atlas/contract-crosswalk/), with spot checks against source, tests, and migration evidence.

## Verdict

The crosswalk is usable for a non-coder who starts from a representative entity and wants to trace entry, normalization, workflow use, persistence, exposure, authority, automation boundaries, human-review gates, value measures, and proof links. The strongest reader path is to start with [`crosswalk-schema.md`](entity-atlas/contract-crosswalk/crosswalk-schema.md), then follow the row-specific links into source/provider flows, workflow packets, storage/persistence, runtime exposure, and surface inventory.

One factual consistency issue was found and patched during review: [`storage-persistence.md`](entity-atlas/contract-crosswalk/storage-persistence.md) said no SQL migration existed even though [`migrations/0001_mvp_foundation.sql`](../migrations/0001_mvp_foundation.sql) is present and already cited by [`surface-inventory.md`](entity-atlas/contract-crosswalk/surface-inventory.md). The patched wording now separates schema readiness from runtime DB wiring, so readers do not infer that API/worker routes are durable simply because the migration exists.

## Representative entity walkthroughs

### Manager Daily Brief packet

A non-coder can answer the acceptance questions:

- Where it enters the system: source facts, labor/timeclock/POS/read-model evidence, checkout/retention packets, source refs, and data-quality issues enter through the Manager Daily Brief row in [`crosswalk-schema.md`](entity-atlas/contract-crosswalk/crosswalk-schema.md#entity-manager-daily-brief-packet-appmanager_daily_briefpacket), the workflow row in [`workflow-packets.md`](entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map), and API context in [`runtime-exposure.md`](entity-atlas/contract-crosswalk/runtime-exposure.md#exposure-table).
- Where it is normalized: `domain::daily_brief`, `domain::analytics`, and `app::manager_daily_brief::{Request, SourceFact, LaborImpactEstimate}` are named in the row and backed by [`app/src/manager_daily_brief.rs`](../app/src/manager_daily_brief.rs), [`domain/src/daily_brief.rs`](../domain/src/daily_brief.rs), and [`domain/src/analytics.rs`](../domain/src/analytics.rs).
- Which workflows use it: daily brief itself is the aggregator workflow; checkout completion, CRM retention, and data-quality hygiene are linked as inputs in [`workflow-packets.md`](entity-atlas/contract-crosswalk/workflow-packets.md#packet-relationships-that-matter).
- Where it is persisted: packet state is derived/API-local, while reviewed outcome evidence is represented by `storage::operations::ManagerDailyBriefOutcomeRecord`; this is clear in [`storage-persistence.md`](entity-atlas/contract-crosswalk/storage-persistence.md#workflow-packets-agents-and-app-runtime-entities) and [`storage-persistence.md`](entity-atlas/contract-crosswalk/storage-persistence.md#review-gates-outcomes-analytics-and-runtime-shells).
- Where it is exposed: local API context/draft/outcome routes, Hermes bridge scripts, local smoke scripts, operator docs, Rustdoc/module paths, and API tests are named in [`runtime-exposure.md`](entity-atlas/contract-crosswalk/runtime-exposure.md#exposure-table).
- Who/what is authoritative: source systems own observed facts; manager/regional/operator roles own management decisions; approved sender owns customer-message decisions; storage/outcome records own reviewed measurement evidence.
- What automation may do: summarize source evidence, rank internal work, draft review/customer follow-up for approval, estimate labor impact, validate drafts, and record reviewed outcomes.
- What needs human review: staffing/capacity/source-quality/policy-sensitive actions, customer messages, provider/PMS writes, payments/discounts/refunds, source ambiguity, and safety/policy exceptions remain blocked without the named review gate.
- How value is measured: estimated/actual labor minutes, feedback outcome, action kind, staff persona/reporting group, source refs, reviewed disposition, and correlation/audit ids.
- Where source/Rustdoc/test evidence lives: rows cite `app::manager_daily_brief`, `storage::operations::ManagerDailyBriefOutcomeRecord`, `apps/api/tests/manager_daily_brief_*`, storage outcome tests, and bridge/smoke scripts.

### Gingr retail product candidate

A non-coder can answer most acceptance questions and the caveats prevent overclaiming:

- Where it enters: Gingr commerce/retail endpoint descriptors and `gingr::dto::retail::Item` in [`source-provider-flows.md`](entity-atlas/contract-crosswalk/source-provider-flows.md#crosswalk-where-provider-entities-enter-and-normalize).
- Where it is normalized: `gingr::mapping::retail::product_candidate` maps provider fields into `ProductCandidate` and domain retail values; missing/unsupported fields become mapping errors rather than silent truth.
- Which workflows use it: current workflow use is limited to data-quality review and future retail/reorder opportunities unless a workflow test links it to an app packet.
- Where it is persisted: no direct candidate table; storage records hold service-line/retail projections only after reviewed domain promotion. The patched storage crosswalk now also clarifies that SQL schema presence is separate from runtime persistence.
- Where it is exposed: Gingr provider docs, integration README/Rustdoc/module paths, retail entity docs, storage/domain docs, and mapping tests.
- Who/what is authoritative: Gingr owns observed item fields; integration owns mapping correctness; retail/service-line managers own product approval, reorder, POS, vendor, price, and inventory decisions.
- What automation may do: read/map provider evidence, flag missing/invalid fields, and draft review/reorder recommendations only where workflow contracts allow it.
- What needs human review: POS transaction, inventory adjustment, vendor order, discount/price movement, product approval, provider write, and source hiding.
- How value is measured: the crosswalk correctly keeps value hypothetical until outcome or inventory/reorder dispositions exist.
- Where evidence lives: [`integrations/gingr/src/dto/retail.rs`](../integrations/gingr/src/dto/retail.rs), [`integrations/gingr/src/mapping/retail.rs`](../integrations/gingr/src/mapping/retail.rs), [`domain/src/retail/mod.rs`](../domain/src/retail/mod.rs), and `integrations/gingr/tests/expanded_endpoint_contracts.rs`.

## Evidence-quality notes

- Strong: the schema requires every row to preserve source-entry, normalization, workflow use, persistence, exposure, authority, allowed automation, blocked/human-reviewed actions, value measures, source/test/Rustdoc evidence, and caveats in a stable order.
- Strong: runtime and storage pages repeatedly distinguish local/demo/in-memory shell behavior from durable persistence and live provider/customer/payment authority.
- Strong: provider flows explicitly quarantine Gingr DTOs and webhooks until mapper/source contracts promote them.
- Strong: evidence links prefer repo-local source/test files and module paths rather than unverified rendered Rustdoc URLs.
- Patched: storage persistence now acknowledges the MVP SQL migration while preserving the caveat that API/worker runtime persistence is not proven.
- Patched: five glossary links now point to the actual `promotion / demotion` anchor (`#promotion-demotion`), restoring repo-wide Markdown link validation.
- Caveat: after rebuilding `target/doc`, source/module/test evidence and rendered Rustdoc smoke checks pass. The aggregate `scripts/check_docs.sh` command still returned `127` at the script tail even after its visible doctest/rustdoc subgates passed, so the review records the independently rerun subgate results below rather than calling the wrapper fully green.

## Verification run

- Passed: `python scripts/check_markdown_links.py --repo-root .`
- Passed: `python scripts/check_public_docs_landing.py`
- Passed: `cargo doc --workspace --no-deps` to rebuild rendered Rustdoc pages under `target/doc`
- Passed: `python scripts/check_rustdoc_completeness.py`
  - Strict missing-docs gate passed.
  - Rendered Rustdoc smoke check passed.
- Partially failed wrapper: `bash scripts/check_docs.sh`
  - Visible subgates passed before the wrapper failure: domain/app/storage/gingr doctests, strict Rustdoc missing-docs gate, and rendered Rustdoc smoke check.
  - Wrapper exit: `127`, with shell message `scripts/check_docs.sh: line 16: ness.py: command not found` at the tail. I did not patch the wrapper because this review task is scoped to crosswalk evidence/usability, and the same subgates were rerun independently.

## Follow-up suggestions

1. Add a short "How to read these six files" index under `docs/entity-atlas/contract-crosswalk/README.md` if this crosswalk becomes a primary non-coder entry point.
2. Add per-entity anchors or appendix rows for the highest-value entities once final entity pages exist, so readers do not need to scan broad matrices for common examples.
3. Investigate the domain doctest dependency-resolution failure separately; it blocks full docs closeout confidence even though Markdown links pass.
