# Contract crosswalk closeout

This package is the source-backed proof layer for the Entity Atlas. Use it when a reader has already found a business entity, workflow, or safety boundary in the [root README](../../../README.md#entity-navigation-map), [central entity index](../../design/entity-index.md), [operator workflow index](../../workflows/operator/README.md), or [public docs landing](../../public/index.html), and now needs to verify where that concept enters, normalizes, gets used by workflows, persists, is exposed, and remains review-gated.

Plain-English boundary: these files prove traceable repo contracts; they do not authorize live customer sends, Gingr/PMS/provider writes, schedule or capacity changes, payment/refund/discount movement, safety/medical/behavior approval, policy exceptions, source hiding, or sensitive data exposure without app-owned approval and human/system-of-record review.

## Canonical crosswalk files

Read the package in this order when auditing a claim end to end:

| File | Canonical role | Use it to answer |
| --- | --- | --- |
| [crosswalk-schema.md](crosswalk-schema.md) | Row contract and evidence rules | What must every entity/workflow/source/storage row say about meaning, authority, automation, blocked actions, value, proof, and caveats? |
| [surface-inventory.md](surface-inventory.md) | Source/Rustdoc/test/docs inventory | Where does the entity or workflow surface in repo source, tests, generated Rustdocs, README/docs, or known gaps? |
| [source-provider-flows.md](source-provider-flows.md) | Source-entry and normalization proof | Where do Gingr/provider/staff/import facts enter, which mapper or source contract normalizes them, and what remains provider evidence or an explicit gap? |
| [workflow-packets.md](workflow-packets.md) | Workflow-use proof | Which app packets, review queues, agent prompt packets, drafts, validators, blocked actions, and outcome fields use the entity? |
| [storage-persistence.md](storage-persistence.md) | Persistence and projection proof | Which entities have storage records, SQL schema artifacts, API-local state, outcome projections, source refs, or intentionally no persistence yet? |
| [runtime-exposure.md](runtime-exposure.md) | API/worker/CLI/web/script exposure proof | Which runtime shells expose packets, drafts, outcomes, source evidence, readiness, or generated docs, and which side effects remain disabled or blocked? |
| [relationship-adjacency.md](relationship-adjacency.md) | Bidirectional relationship and flow proof | How does evidence move enter -> normalize -> use -> persist -> expose, and which adjacent entities depend on or feed each other? |

Review evidence created for this board lives at [docs/qa-contract-crosswalk-source-evidence-usability-2026-06-19.md](../../qa-contract-crosswalk-source-evidence-usability-2026-06-19.md). It records the source-evidence/non-coder usability pass and the corrections made during review.

## Coverage summary

The crosswalk package now covers the main proof paths needed by the Entity Atlas:

- Source/provider boundary: Gingr endpoints, DTOs, response envelopes, webhook verification, customer/pet/retail mapping candidates, reservation source snapshots, provider-surface gaps, provenance, raw payload/source refs, and data-quality issues.
- Core pet-resort entities: customer, pet, reservation/stay, source system, provenance, data-quality issue, care/vaccine/document/incident facts, service-line contracts, message/customer draft, review gate, blocked action, outcome record, money/payment evidence, operations/analytics context, and storage/runtime shells.
- Workflow packets and agent-safe use: Booking Triage, Checkout Completion, CRM Retention/Grooming Rebooking, Daily Updates/Pawgress drafts, Manager Daily Brief, Data-Quality Hygiene, regional labor exception patterns, tool-port boundaries, and local smoke/bridge proof.
- Persistence and exposure: MVP SQL schema artifact readiness, current Rust storage projection contracts, API in-memory/demo state, storage-shaped outcome records, worker/CLI/web shells, bridge scripts, smoke scripts, rendered Rustdoc/source paths, and docs-public navigation.
- Relationship proof: bidirectional adjacency rows and Mermaid flows connect source evidence to provenance/domain candidates, app packets, drafts/review gates, reviewed outcomes, storage projections, API/CLI/web/script exposure, and reporting proof.

The strongest current evidence is local/demo and contract-test proof for source-grounded context, deterministic draft validation, fail-closed blocked side effects, and reviewed outcome capture. The package deliberately separates that from live operational authority.

## Model-depth proof chain added by this board

The final model-depth pass should be read as a labor proof chain, not as isolated structs. The currently inspected diff strengthens these end-to-end links:

| Role/workflow labor reduced | Source/domain model depth | App packet or draft boundary | Review gate and blocked actions | Outcome / proof surface |
| --- | --- | --- | --- | --- |
| Front-desk booking triage: reduce repeated missing-info, vaccine, care, behavior, payment, and policy checks. | `app::booking_triage::{MissingInfoReason, BlockerKind, BlockerEvidence}` keeps source-backed blockers attached to deterministic readiness. | `StaffEvaluationPacket` may carry a `MissingInfoDraft`, but the draft is only customer-copy awaiting approval. | Confirmation, customer message, provider/PMS mutation, schedule/capacity movement, and payment movement remain blocked until staff/system-of-record action. | `app/tests/booking_triage_mvp.rs`, booking operator docs, and workflow-packets crosswalk prove deterministic routing and draft-only behavior. |
| Checkout/retention/daily-update staff: reduce handoff, rebooking, and customer-copy drafting time after source-backed review. | Checkout packets, CRM retention opportunities, contact permission, message draft state, included/omitted facts, and safe send stubs carry source facts and suppression reasons. | Agents may summarize, rank, and draft only inside `app::checkout_completion`, `app::crm_retention`, or `app::daily_update` packet contracts. | Customer sends, booking/calendar writes, payment/discount movement, sensitive care/incident wording, and provider writes stay human-reviewed. | App workflow tests and operator pages prove draft/review surfaces; durable outcome storage remains strongest for manager daily brief/data-quality hygiene today. |
| Grooming/training/retail/daycare opportunity review: reduce manual opportunity, package/session, reorder, and package-fit triage. | `domain::training`, `domain::retail::reorder`, and `domain::daycare::package_opportunity` name source evidence, blocked actions, review gates, and labor-minute outcomes. | Opportunities become internal tasks, manager review, vendor notice drafts, or customer-copy drafts only after source evidence exists. | Package enrollment, billing/session balance, POS/inventory, vendor purchase order, provider write, customer send, and outcome approval remain blocked. | Domain tests, storage service-line projections, Gingr retail mapping tests, and revenue-opportunity atlas rows prove the review-safe model depth. |
| Manager/data-quality outcome capture: reduce repeated reconciliation and make labor claims auditable. | `domain::source::Provenance`/`RecordRef`, app outcome records, and storage outcome records keep source refs and issue refs visible. | API routes accept reviewed outcome payloads only when source/issue evidence is present for the relevant workflow. | API responses keep live side effects disallowed and record blocked actions instead of repairing source/provider records. | `apps/api/tests/*outcome*`, `storage/tests/*outcome_storage.rs`, and `storage::operations` prove source-backed reviewed labor outcome capture. |

Use this table as the fan-in lens when updating public docs: every claim should travel source evidence -> domain/app contract -> agent-safe packet/draft -> human/system-of-record review -> outcome/labor evidence -> storage/runtime/Rustdoc proof.

## Known evidence gaps and caveats

Keep these caveats visible when reusing crosswalk claims in public or non-coder docs:

1. Runtime DB wiring is not proven for API/worker routes. `migrations/0001_mvp_foundation.sql` and `storage/tests/mvp_migration_contract.rs` prove schema readiness, while current API route state is still process-local/in-memory unless a later adapter wires durable repositories.
2. Gingr reservation normalization is source-contract based; this repo does not currently prove a direct `integrations/gingr::response::ReservationRecord` mapper into the domain snapshot.
3. Grooming and training provider service DTOs remain explicit `ProviderSurface::NoDocumentedServiceDto` gaps. Domain grooming/training contracts exist, but provider-service payload mapping is not proven.
4. Retail product mapping now rejects missing provider category or active status instead of silently defaulting, but retail quantity-on-hand is still preserved only on the Gingr DTO and is not yet part of the retail product candidate contract; POS, vendor, reorder, and inventory movements remain future/review-gated evidence.
5. Webhook verification is modeled, but no live webhook receiver or webhook-payload-to-domain-workflow mapper is wired in this package.
6. Worker runtime is a safe shell with fake/disabled agent runtime and stubbed side-effect posture; no durable queue/scheduler consumer is implemented yet.
7. Staff web and API surfaces are local/demo contract surfaces. They may show review gates, drafts, outcome forms, and audit-visible concepts, but they are not live systems of record.
8. Labor-savings claims must stay tied to reviewed outcome records, actual/estimated minutes, disposition, and reporting group fields. Hypotheses should remain labeled as planned metrics until outcome proof exists.
9. Many atlas/docs/source changes were produced by parallel board work in a shared directory workspace. Treat this README as the closeout/navigation artifact for the crosswalk package, not as a clean diff claim over every modified file in the checkout.

## How this feeds the Entity Atlas and public docs

The crosswalk package is the proof spine behind the non-coder atlas:

- Central Entity Atlas: [docs/design/entity-index.md](../../design/entity-index.md#contract-crosswalk-proof-paths) uses these files as the source/Rustdoc/test proof path after the reader starts from a business entity family.
- Relationship and workflow navigation: [docs/design/entity-atlas-relationships.md](../../design/entity-atlas-relationships.md), [docs/design/workflow-to-entity-navigation-map.md](../../design/workflow-to-entity-navigation-map.md), and [docs/workflows/operator/README.md](../../workflows/operator/README.md) should point here when a workflow claim needs source-entry, normalization, persistence, runtime, or adjacency proof.
- Public/non-coder landing: [docs/public/index.html](../../public/index.html) remains the front door for entity-first reading. It should summarize meaning and safety first, then route deeper evidence requests to the entity index and this crosswalk package instead of sending readers directly to crate names.
- Future page writers: use [crosswalk-schema.md](crosswalk-schema.md) as the row contract and link one or more of the canonical crosswalk files above whenever adding a public claim about automation authority, data source, storage readiness, runtime exposure, labor value, or evidence gaps.

## Closeout checklist for future maintainers

Before treating a future Entity Atlas or public-docs claim as crosswalk-backed, verify:

- It links to the relevant crosswalk file and at least one source/test/Rustdoc/module path.
- It distinguishes source evidence, domain/app meaning, storage projection, runtime exposure, and human/system-of-record authority.
- It names allowed automation verbs narrowly: read, validate, summarize, rank, draft, prepare internal task, or record reviewed outcome.
- It names blocked or human-reviewed actions when touching customers, pets, safety, schedules, money, provider records, source cleanup, or sensitive exposure.
- It marks fixture/local/demo/runtime-shell evidence separately from production/live/readiness claims.
- Markdown links and public-doc landing checks pass after the change.
