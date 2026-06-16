# BI Question-Driven Decision Rubric

Date: 2026-06-16

## Purpose

The BI discovery questions are not just background research. They are the decision mechanism for what NVA builds next.

Until the answers are known, NVA should prefer reversible source contracts, provenance capture, typed uncertainty, and fixture-driven projections over premature workflow automation or hard-coded Gingr/BI assumptions.

## Operating rule

For each BI answer, classify it into one or more decision buckets:

1. **Source surface** — what systems/tables/reports exist.
2. **Grain and identity** — what one row means and which IDs can be trusted.
3. **Refresh and mutation behavior** — how records change, backfill, duplicate, merge, or disappear.
4. **Reliability and semantic ambiguity** — what fields/statuses mean, and where they mislead.
5. **Provenance and raw retention** — whether source payloads, batches, endpoints, and transformation versions exist.
6. **Metric trust** — which reports/definitions are operationally trusted or distrusted.
7. **Artifact availability** — what we can inspect next without formal access.

The next engineering task should be selected by the highest-risk unanswered bucket, not by implementation convenience.

## Decision map

| If BI answer reveals... | Then prioritize next... | Avoid until later... |
| --- | --- | --- |
| Tables/sources are unknown or fragmented | Source inventory and adapter boundary docs | Domain workflow machines |
| Table grain is unclear | Grain/identity fixtures and `StayFact` projection tests | Aggregate metrics or dashboards |
| Stable IDs are unclear | Identity-resolution types, join assumptions, duplicate/merge data-quality issues | Cross-source optimization |
| Refresh/backfill behavior is unclear | Snapshot/provenance model, batch IDs, pulled-at timestamps, mutation policy | Treating BI rows as immutable facts |
| Raw payloads/import batches exist | Raw snapshot preservation and replay fixtures | Only modeling cleaned BI rows |
| Raw payloads do not exist | BI-row-as-source adapter with explicit confidence limits | Pretending source truth is recoverable |
| Statuses/columns are overloaded | Typed data-quality issues and semantic normalization tests | Workflow validators that trust statuses |
| Trusted dashboards exist | Reconciliation tests against trusted metric definitions | Inventing metric names in isolation |
| Distrusted dashboards exist | Failure-case fixtures and anti-reconciliation tests | Copying BI definitions blindly |
| Payroll/timeclock/scheduling data is available | Labor-cost projection and staffing optimization planning | Gingr-only product framing |
| Only screenshots/table names are available | Lightweight schema glossary and fixture guesses marked provisional | Live DB integration |
| Formal access is granted | Read-only connector spike and schema introspection | Production writes or member-facing actions |

## Practical task-selection algorithm

When new BI input arrives:

1. Add the raw non-secret answer or artifact summary under `docs/discovery/` or `docs/integrations/gingr/`.
2. Tag each answer with the relevant decision buckets above.
3. Update current assumptions in the source contract/read-model docs.
4. Choose the next card by this priority order:
   1. **Safety blockers**: unstable IDs, ambiguous grain, destructive refreshes, missing provenance.
   2. **Projection blockers**: fields needed to produce a deterministic `StayFact`.
   3. **Labor-cost blockers**: schedule/timeclock/payroll/capacity signals needed to connect occupancy/stays to labor spend.
   4. **Workflow blockers**: status/event meanings needed before Statum workflow validators can be trusted.
   5. **Optimization opportunities**: reports or metrics trusted enough to compare against proposed automations.
5. If an answer contradicts an existing contract, create a remediation card before adding new capability.
6. If an answer is unknown, encode uncertainty as a typed assumption or data-quality issue rather than blocking all progress.

## Current implications for the active board

The active `nva-gingr-source-bi` board should proceed in this order:

1. **Clear the reservation/stay source-contract review gate** because provenance, source snapshots, and data-quality issue types are prerequisites for using BI answers safely.
2. **Build the deterministic reservation/stay to `StayFact` projection** only with assumptions explicitly tied back to the question set.
3. **Design the Statum/workflow bridge** after projection tests show which source facts are reliable enough to become workflow inputs.
4. **Do not broaden into labor optimization implementation** until at least one scheduling/timeclock/payroll/capacity answer or artifact exists.

## What counts as a decision-ready answer

An answer does not have to be complete. Any of these are enough to drive a next task:

- table list or schema screenshot;
- redacted sample rows;
- dashboard/report names;
- trusted vs distrusted metric notes;
- refresh cadence description;
- known messy-field warning;
- rough transformation code or pseudo-code;
- "we do not retain raw payloads";
- "this ID is not stable";
- "payroll/timeclock data exists but lives elsewhere".

The standard is not certainty. The standard is enough factual shape to decide whether the next safest move is inventory, contract refinement, projection, workflow validation, or labor-cost modeling.

## Anti-goals

- Do not let Gingr table names become the domain model.
- Do not let BI decide NVA's labor-cost strategy.
- Do not build automation on ambiguous statuses without typed uncertainty.
- Do not wait for perfect access before modeling safe assumptions.
- Do not ignore scheduling, timeclock, payroll, capacity, SOP, or task-system signals when they appear; they are central to the labor-cost goal.
