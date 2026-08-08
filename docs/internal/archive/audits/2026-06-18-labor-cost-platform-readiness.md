# Labor-cost platform readiness memo — 2026-06-18

## Scope and evidence

This memo answers how the repository looks as a labor-cost-reduction platform for the NVA Pet Resorts context: a 170-site pet-resort portfolio where the valuable AI work is reducing manager/front-desk rework, source reconciliation, handoff churn, and repetitive drafting without letting an agent own operational truth.

Evidence reviewed:

- `nva-pet-resorts-ai-context.md` for the labor-cost acceptance lens.
- `README.md` for crate boundaries, docs-as-contracts policy, and repo navigation.
- `docs/architecture/agent-app-infrastructure.md` for deterministic app-owned facts, policy, review gates, persistence, audit, and side-effect control.
- `docs/audits/2026-06-18-agent-app-infrastructure-readiness.md` for the previous go/no-go posture.
- `docs/plans/2026-06-18-labor-cost-docs-as-contracts-kanban.md` for the board plan and final acceptance criteria.
- `docs/design/labor-cost-reduction-crosswalk.md` for the driver-to-workflow mapping.
- `docs/ops/data-quality-hygiene-local-smoke.md`, `app::data_quality_hygiene`, `storage::operations::DataQualityHygieneOutcomeRecord`, and their tests for the second workflow proof.
- Parent handoffs from `t_15839424` and `t_cf966cb0`, including the fan-in commit `30060fc` and the data-quality smoke commit `d9a5b61`.

## Executive verdict

Score: 7.5 / 10 as a labor-cost-reduction platform foundation.

The repo is now credible as a local/demo and sandbox foundation for review-gated labor workflows. It is no longer just a pile of pet-resort domain types or a generic agent demo: the current shape has app-owned context packets, deterministic draft validation, source refs, blocked side effects, outcome capture, doctest-backed API contracts, and a labor-cost crosswalk that keeps future work anchored to measurable operations work.

It is not pilot/live ready. The missing pieces are not “more AI”; they are production source access, real NVA data/read-model proof, permissioning, monitoring, durable audit/retention, rollback, operator authorization, and explicit app-owned write contracts for any side effect. Until those exist, the platform should not claim readiness for customer sends, provider/PMS mutation, schedule changes, payment/refund/discount movement, or safety-sensitive autonomous decisions.

## Scorecard against the labor-cost objective

| Dimension | Score | Evidence | Gap |
| --- | ---: | --- | --- |
| Labor-cost lens and workflow focus | 9/10 | README states labor-cost reduction, not generic chat; the crosswalk maps scheduling, front-desk load, checkout bottlenecks, source hygiene, regional visibility, retention, SOP/incident, and post-stay/reputation work to repo surfaces. | Real NVA metric definitions and accepted outcome measures still need operator confirmation. |
| Deterministic app/agent boundary | 9/10 | Architecture guide and app contracts keep facts, policy, review gates, persistence, audit, and side effects in deterministic app code; agents consume context and submit drafts. | The boundary is proven locally, not against production identity/permissions and not yet packaged as stable MCP/tool schemas. |
| Measurable outcome capture | 8/10 | Manager Daily Brief and Data-Quality Hygiene both model before/after or estimated/actual labor minutes; storage records persist source refs, actor/persona, outcome, issue/action kind, and correlation fields. | Outcome evidence is fixture/local so far; no longitudinal adoption, payroll variance, or portfolio KPI rollups. |
| Source grounding and data quality | 8/10 | `domain::source`, `domain::data_quality`, data-quality hygiene context/actions, source refs, freshness, sensitivity, and ambiguity-preservation are first-class. | Real provider/source-system coverage beyond fixture/Gingr-shaped contracts is still unknown. |
| Safety gates | 8/10 | Draft validation rejects blocked and unsupported side effects, data-quality ambiguity hiding, missing source refs, wrong context/correlation, and action mismatches. Crosswalk repeats no live sends/PMS writes/schedule/payment movement. | Pilot/live requires production authz, audit retention, policy versioning, monitoring, and explicit approval records around any future write path. |
| Workflow breadth | 7/10 | Manager Daily Brief is first loop; Data-Quality Hygiene is second local loop; crosswalk names regional exception and grooming/retention follow-up next. | API/runtime shells and operator UI are not yet built for Data-Quality Hygiene, regional exceptions, or grooming/retention. |
| Docs-as-contracts and maintainability | 8/10 | READMEs are navigation/wiki; Rustdoc examples are executable contracts; `scripts/check_docs.sh`, Markdown link checks, and README coverage gates protect navigation and API examples. | The repo is documentation-heavy; continued discipline is needed to prevent README prose from becoming duplicate non-executable specs. |
| Runtime/product readiness | 4/10 | Local fake-data smokes and app/storage contracts exist; no live side effects are enabled. | No production deployment, source credentials, NVA data model validation, monitoring, rollback, staff UI hardening, or customer/provider write approval path. |

## Readiness by stage

### Local/demo readiness: ready

The repo is ready to demonstrate the platform pattern locally with fake data. The strongest proof is no longer just prose:

- Manager Daily Brief has app/domain/storage contracts and prior local smoke evidence for context -> draft -> reviewed outcome with estimated and actual minutes.
- Data-Quality Hygiene has a fixture-only local smoke: `./scripts/smoke_data_quality_hygiene_local_loop.sh` exercises source-grounded context creation, draft acceptance, blocked customer-send validation, and reviewed outcome capture.
- Docs-as-contracts gates are present: Rust doctests for contract crates plus offline Markdown link/README coverage checks.
- The root README points maintainers to the labor-cost lens, crosswalk, architecture guide, smoke runbook, and readiness reviews.

Local/demo claim allowed: “This repo can show how a deterministic pet-resort app gives an agent source-grounded context, validates a reviewable draft, blocks side effects, and records labor-minute outcomes.”

Local/demo claim not allowed: “This saves labor at NVA already” or “the agent can operate against Gingr/live customers.”

### Sandbox readiness: ready for continuation

The repo is ready for sandbox continuation of review-gated workflows, especially Data-Quality Hygiene and a read-only regional exception loop, because the app-owned contract pattern is clear:

1. Build typed context packets with source refs, location/day/persona, data-quality flags, allowed actions, blocked actions, and correlation IDs.
2. Let agents summarize, rank, draft internal tasks, preserve ambiguity, and estimate labor minutes.
3. Validate every draft deterministically before it reaches staff.
4. Capture reviewed outcomes with actual minutes and source-fact corrections.

Sandbox continuation should remain fixture-safe or non-production read-only unless an operator explicitly provides sandbox systems, sandbox credentials, and sandbox retention/audit expectations.

### Pilot readiness: not ready

Pilot readiness needs proof beyond local contracts:

- real NVA source/read-model access with source refs and data freshness semantics;
- identity, role, and approval records for GMs, front-desk leads, regional operators, and analysts;
- durable persistence and audit retention for context packets, drafts, validation results, review decisions, and outcomes;
- monitoring/alerting for rejected drafts, wrong-source outcomes, stale packets, and side-effect attempts;
- rollback/disable paths per workflow, location, and action kind;
- operator-approved metric definitions for labor savings, including how estimated minutes, actual minutes, rework reduction, payroll variance, and adoption are interpreted;
- a staff/reviewer UI or equivalent operational surface that does not rely on developer-only scripts.

Pilot claim allowed after those are proven: “A review-gated sandbox/pilot workflow can reduce a defined class of manager/front-desk reconciliation work and measure reviewed outcomes.”

### Live readiness: blocked

Live/member-facing readiness remains blocked. The repo must not authorize:

- customer email/SMS/portal/review sends;
- provider/PMS/Gingr writes;
- schedule/staffing mutations;
- package/invoice/payment/refund/discount movement;
- hidden data-quality resolution, duplicate merging, or source ambiguity suppression;
- autonomous decisions around incidents, aggression, medical/vaccine ambiguity, safety, eligibility, or personnel action.

Those capabilities would need separate app-owned write contracts, explicit policy/versioning, production authz, human approval, audit, monitoring, and rollback. MCP packaging or enterprise Claude access would not change this boundary by itself.

## Next best workflow recommendation

### 1. Finish Data-Quality Hygiene from local smoke to sandbox app/runtime workflow

Recommended next workflow: Data-Quality Hygiene.

Why it is first:

- It directly reduces repeated source reconciliation by managers/front-desk staff.
- It improves every later labor loop by keeping missing/ambiguous source facts visible instead of buried in agent prose.
- It is naturally internal and review-gated; no customer send, provider write, schedule mutation, or payment movement is needed.
- It already has source-grounded app contracts, storage outcome records, contract tests, and a local smoke runbook.

Measurable labor metric:

- Primary: actual minutes avoided in manual source reconciliation and downstream rework per location/day, captured by reviewed outcomes.
- Supporting: issue/action count by kind, wrong-source rate, deferred/suppressed rate, repeated issue recurrence by source system/field path, and time from issue surfaced to reviewed disposition.

Safety gates:

- Required source refs for every accepted action.
- Data-quality ambiguity must remain visible until reviewed; no hiding or auto-resolution.
- Draft validation rejects known blocked side effects and unknown non-empty side-effect requests.
- Outcomes record actor/persona, issue refs, source refs, resolution status after review, correlation ID, estimated minutes, and actual minutes.
- Still no customer sends, provider/PMS writes, schedule mutations, payment/refund/discount movement, or source-record merging.

Smallest next implementation slice:

- Add API/runtime endpoints or CLI/operator surfaces for Data-Quality Hygiene context, draft validation, and outcome capture using the existing `app::data_quality_hygiene` contract.
- Keep it local/sandbox and fixture/read-only until production source/authz proof exists.

### 2. Regional labor exception loop

Why second:

Manager Daily Brief works at a location/day level. A 170-site operator also needs regional leaders to see which sites are off plan and why without manually scanning dashboards or asking each GM for a narrative.

Measurable labor metric:

- Regional leader minutes avoided in dashboard scanning and follow-up prioritization.
- Count of off-plan sites reviewed with source-backed next action.
- Wrong-source/deferred/completed outcomes by reporting group.

Safety gates:

- Context packets must group sites by metric/period/source refs/data-quality caveats/existing outcomes.
- Drafts recommend review queues only: ask GM to review staffing plan, inspect checkout exception pattern, validate data hygiene, etc.
- No staff discipline, schedule mutation, pricing decision, customer communication, provider/PMS write, or policy exception from agent output.

### 3. Grooming rebooking / retention follow-up loop

Why third:

It can save front-desk drafting/prioritization time and may support revenue, but it is closer to customer messaging, discounts/offers, scheduling, and consent. It should wait until data-quality and regional exception loops prove source refs, consent/suppression, review outcomes, and safety gates at portfolio scale.

Measurable labor metric:

- Front-desk minutes avoided in finding safe follow-up candidates and drafting outreach.
- Optional separate business metrics: filled grooming capacity, rebooking conversion, retained customers.

Safety gates:

- Context packets include grooming cadence, service history, contact permission, channel, source refs, suppression reasons, and slot-utilization/readiness signals.
- Drafts propose staff-reviewed follow-up language and internal tasks only.
- No live customer send, discount/refund/payment movement, schedule/provider mutation, or unsupported offer claim without explicit app-owned approval and human review.

## What to say about the repo now

Accurate positioning:

> The repo is a Rust-first deterministic app foundation for safe pet-resort labor workflows. It has local/demo and sandbox-ready contracts for review-gated agent loops: source-grounded context packets, deterministic draft validation, blocked side effects, and outcome capture for labor minutes. It is ready to continue building internal review workflows, not ready for live/member-facing or provider-writing automation.

Avoid saying:

- “AI agents automate NVA operations” without naming the app-owned context/draft/outcome contracts.
- “Ready for production” without production source access, authz, monitoring, rollback, retention, and operator approval.
- “Labor savings proven” beyond local fake-data and reviewed outcome contract capability.
- “Gingr integration complete” while source-system coverage and real instance/API availability remain discovery questions.

## Follow-up gates before any pilot/live posture

1. Convert Data-Quality Hygiene from local app/storage contract into an API/runtime/operator path with the same safety gates.
2. Define the production/sandbox source packet contract with NVA: systems of record, source refs, freshness, retention, metric definitions, and BI/read-model ownership.
3. Add durable audit storage for context packets, draft submissions, validations, review decisions, side-effect attempts, and outcomes.
4. Add workflow/location/action kill switches and monitoring for rejected drafts, wrong-source outcomes, missing refs, stale packets, and attempted side effects.
5. Build or integrate a staff review surface that records actor/persona, approval/deferral/suppression/correction, and actual minutes.
6. Keep READMEs as navigation and move executable examples into Rustdoc/doctests; keep the docs gate in the default test path.

## Final recommendation

Continue with Data-Quality Hygiene as the next implementation workflow, then a read-only Regional Labor Exception loop, then Grooming Rebooking / Retention follow-up. Do not broaden into live customer/provider/schedule/payment side effects yet. The repo is strongest when it treats agents as draft/reasoning workers behind deterministic pet-resort app contracts and measures labor savings through reviewed outcomes rather than AI claims.
