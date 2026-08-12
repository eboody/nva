# NVA strategic AI operations Kanban

Created: 2026-08-12

This board turns the requested NVA Pet Resorts AI opportunities into a staged discovery/design roadmap. It is intentionally parked for planning first. No worker should claim live NVA access, send customer messages, change schedules, write to PMS/CRM systems, or make staffing decisions without explicit approval and a review-gated system-of-record path.

## Strategic objective

Build an owned Pet Resorts operations layer that converts source evidence into review-gated workflow packets, measurable labor/outcome records, and safe AI assistance. The common platform should support lead response, capacity and labor optimization, operator knowledge assistance, retention actions, financial site insights, and CRM/customer note enrichment without hiding provenance or bypassing human/system gates.

## Safety and access boundaries

Current repo framing still applies:

- Fixture/local/read-only evidence first.
- Customer-visible messages are drafts until human-approved.
- Provider/PMS/CRM writes are blocked until explicitly approved through a system-of-record path.
- Schedule, capacity, staffing, discount, payment, and policy decisions remain reviewed recommendations, not autonomous mandates.
- Financial claims and labor-savings claims require outcome/provenance records.
- Granular customer profile enrichment must respect access boundaries, consent, retention policy, role scope, and marketing-vs-operations separation.

## Board graph

Parallel discovery lanes:

1. Lead callback/text automation.
2. Capacity-based scheduling and labor-hour optimization.
3. Pet Resorts GPT knowledge/context assistant.
4. Customer retention behavioral insights.
5. Site-level financial insight/actions.
6. CRM note generation, organization, segmentation, and granular customer signal capture.
7. Cross-workstream data/source/access inventory.

Fan-in lanes:

8. Unified owned operations data model and safety policy.
9. MVP sequencing and pilot roadmap.
10. Review/closeout: verify board outputs are actionable, safe, and aligned to labor-cost / booking-conversion objectives.

## Implemented domain coverage

The initial strategic model contract is implemented in `domain/src/strategic_ai_ops.rs` with coverage tests in `domain/tests/strategic_ai_ops_model_contracts.rs`.

The model adds shared primitives and packet types for:

- Realtime lead response: event id, idempotency key, event kind, received time, source system, identity match, response SLA, consent evidence, contact attempt, review gate, and conversion attribution.
- CRM/customer intelligence: structured notes, note kind, visibility, allowed use, confidence, review state, segmentation basis, and marketing-allowed separation.
- Capacity and labor optimization: time windows, service demand units, constraints, labor standards, scheduled coverage, loaded labor cost, optimization objective, recommended action, labor delta, and manager review gate.
- Pet Resorts GPT: actor id/title/role/location/purpose, allowed uses, knowledge document metadata, document kind, approval status, applicability, citations, answer packet, confidence, and escalation reason.
- Site financial insight: site period, service-line revenue fact, gross/discount/refund/labor costs, source system, net revenue calculation, insight kind, expected impact, recommendation, and a hard block on autonomous financial mutation.
- Cross-workstream outcomes: workstream, metric, before/after value, attribution strength, source system, recorded time, and value-claim eligibility.
- Source/access foundation: expanded source-system vocabulary, actor roles, visibility scopes, allowed uses, and customer identity-match confidence.

These additions intentionally model the strategic gaps without claiming live operational readiness. They create the semantic contracts downstream importers, agents, optimizers, dashboards, and review workflows can bind to.

## Key design questions to answer next

- Which source systems hold the earliest lead events, missed calls, texts, web forms, reservations, capacity, staffing schedules, invoices, POS/revenue, discounts, campaign touchpoints, customer notes, service history, pet details, incidents, and vendor/SOP/pricing knowledge?
- What is available as export/read-only access versus API versus webhook versus manual report?
- Which site roles should be allowed to view, approve, or act on each recommendation?
- Which outputs can be drafts, which can be internal recommendations, and which require strict human approval?
- What measurable outcome proves value for each lane: faster lead response, booking conversion, kennel utilization, labor percentage, staff schedule quality, retained customers, margin/revenue improvement, handle time, or personalization quality?
- What sensitive customer/pet data must not be exposed to marketing, generalized LLM prompts, or broad staff contexts?

## Expected board status

The board is `nva-strategic-ai-ops`. It should stay parked until the user explicitly asks to begin discovery/design execution. Once execution starts, docs/discovery workers can run in parallel. Code-mutating or repo-writing implementation should be serialized or isolated into worktrees.
