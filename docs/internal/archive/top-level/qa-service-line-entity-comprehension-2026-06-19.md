# QA: service-line entity comprehension

Kanban task: `t_0c4e35bb`

Question tested: can a non-coder understand service-line entities and their operational role using the public/README-style docs, without reading source code first?

Scope selected from the entity matrix:

- Boarding contract
- Daycare contract
- Grooming contract / grooming rebooking cadence
- Training contract / training package-session opportunity

Primary docs used:

- `README.md`
- `docs/design/entity-index.md`
- `docs/design/entity-atlas-petsuites-core-entities.md`
- `docs/design/entity-atlas-revenue-opportunity-entities.md`
- `docs/design/entity-atlas-relationships.md`
- `docs/design/workflow-to-entity-navigation-map.md`
- `docs/design/entity-atlas-inventory.md`
- `domain/src/boarding/README.md`
- `domain/src/daycare/README.md`
- `domain/src/grooming/README.md`
- `domain/src/training/README.md`

Result summary: PASS overall, with documentation follow-ups. A non-coder can answer the README entity questions for all four sampled service-line entities, especially by starting at `README.md` -> `docs/design/entity-index.md` -> the relevant family page -> the service-line README. The main gap is not correctness; it is that grooming and training rely more on a combined revenue-family page plus source/module README than on a full per-entity table like boarding/daycare have in the core entity atlas.

## README entity-question answers by service-line entity

### Boarding contract

| README question | Answer from non-coder docs | Evidence links |
| --- | --- | --- |
| What is this in pet-resort language? | The overnight-stay rule set for accommodation, room/suite capacity, deposits, cancellation, care readiness, housekeeping, handoff, minimum stays, and boarding-specific upsell opportunities. | `docs/design/entity-atlas-petsuites-core-entities.md` Boarding contract; `domain/src/boarding/README.md` Operator summary. |
| Why does it exist? | To replace manual policy lookup for “can we promise this boarding stay?” with named capacity, care, payment, and handoff decisions. | Core atlas Boarding contract; `domain/src/boarding/README.md` lines 42-50. |
| Which labor, safety, or customer-trust problem does it help with? | Reduces front-desk/manager time spent reconciling room capacity, care profile readiness, deposit state, cancellation rules, housekeeping, handoff requirements, and exit-bath upsell review; protects against unsafe promises. | `domain/src/boarding/README.md` Operator summary and workflow surface. |
| Which workflows and contracts feature it? | Booking Triage, Checkout Completion, Daily Care Update, Manager Daily Brief, Data Quality Hygiene; `domain::boarding::Contract`; storage service-line contract records. | Entity index service-line row; core atlas; relationship map service-line table. |
| What other entities does it depend on or feed? | Depends on reservation/stay, pet, care profile, location policy, capacity/source snapshots, payment/deposit evidence, storage projections; feeds booking/readiness packets, review gates, outcome evidence, storage projections. | Core atlas Boarding contract relationship row; `domain/src/boarding/README.md` Cross-crate relationships. |
| Who or what is authoritative? | `domain::boarding::Contract` and policy modules define semantics; provider/read-model evidence supplies inventory/stay facts; storage persists projections; boarding/front-desk lead, GM, or care staff handle exceptions. | Core atlas Boarding contract source-of-record and human-role rows. |
| What can an agent draft or recommend around it? | Evaluate readiness from source snapshots, surface waitlist/denial/review reasons, draft internal tasks or customer-safe scripts for review, record outcomes. | Core atlas Boarding contract allowed-actions row; entity index service-line row. |
| What must stay human-reviewed or blocked? | Booking/cancellation/overbooking, room assignment, capacity holds/releases, deposit/refund/fee decisions, medical/care acceptance, customer upsell sends. | Core atlas Boarding contract blocked-actions row; `domain/src/boarding/README.md` safety boundary. |
| Where is the code/Rustdoc/test evidence? | `domain/src/boarding/*`, `storage/src/service_line/boarding.rs`, `storage/src/operations.rs`, `storage/tests/core_service_contract_storage.rs`, app workflow composition tests. | Core atlas source/Rustdoc rows; `domain/src/boarding/README.md` Cross-crate relationships. |

Verdict: PASS. Boarding has the strongest individual entry shape because the core atlas provides a full non-coder table and the module README reinforces operational role, blocked actions, relationships, and evidence links.

### Daycare contract

| README question | Answer from non-coder docs | Evidence links |
| --- | --- | --- |
| What is this in pet-resort language? | The daytime-care rule set for service variant, attendance, package policy, staff-to-pet ratio, group-play eligibility, playgroup assignment, incident restrictions, front-desk lanes, and package-opportunity review. | `docs/design/entity-atlas-petsuites-core-entities.md` Daycare contract; `domain/src/daycare/README.md` Operator summary. |
| Why does it exist? | To keep busy check-in and playgroup decisions from becoming ad hoc manual judgment by naming fast lane, collection, care-team review, manager review, policy block, waitlist, and package-review outcomes. | Core atlas Daycare contract; `domain/src/daycare/README.md` lines 7-15 and 50-61. |
| Which labor, safety, or customer-trust problem does it help with? | Reduces repeated lookup of attendance policy, package state, staff ratios, temperament/vaccine readiness, incident restrictions, and customer-message readiness while preserving safety gates. | `domain/src/daycare/README.md` Operator summary. |
| Which workflows and contracts feature it? | Booking Triage, Daily Care Update, Incident Escalation, Manager Daily Brief, Data Quality Hygiene, daycare staff operations; `domain::daycare::Contract`; storage daycare contract records. | Entity index; workflow-to-entity map; relationship map service-line table. |
| What other entities does it depend on or feed? | Depends on pet species/spay-neuter/temperament/care facts, vaccine status, incidents, reservation/attendance, staff coverage, customer-message policy; feeds queue tickets, review gates, manager/daycare packets, outcome records, storage projections. | Core atlas Daycare contract relationship row; `domain/src/daycare/README.md` Cross-crate relationships. |
| Who or what is authoritative? | `domain::daycare::Contract` and child policy modules define semantics; provider/reservation/attendance/source evidence supplies day context; roster/read model supplies coverage; daycare lead/care reviewer/manager/front desk handle exceptions. | Core atlas Daycare contract source-of-record and human-role rows. |
| What can an agent draft or recommend around it? | Materialize attendance candidates, evaluate ratio/eligibility/readiness, produce queue tickets, draft package/review tasks, record review and outcome evidence. | Core atlas allowed-actions row; entity index service-line row. |
| What must stay human-reviewed or blocked? | Admission, playgroup placement, group-play override, incident restriction clearance, package sale/enrollment, billing action, provider write, customer update/send, manager override. | Core atlas blocked-actions row; `domain/src/daycare/README.md` safety boundary. |
| Where is the code/Rustdoc/test evidence? | `domain/src/daycare/*`, `storage/src/service_line/daycare.rs`, `storage/src/operations.rs`, `storage/tests/core_service_contract_storage.rs`, `storage/tests/operations_storage_contracts.rs`, `domain/tests/petsuites_core_service_contracts.rs`. | Core atlas source/Rustdoc rows; `domain/src/daycare/README.md` Cross-crate relationships. |

Verdict: PASS. Daycare is understandable to a non-coder and has strong safety boundaries; its only readability cost is many module names in the module README, but the operator summary and core atlas table come first enough to orient the reader.

### Grooming contract / grooming rebooking cadence

| README question | Answer from non-coder docs | Evidence links |
| --- | --- | --- |
| What is this in pet-resort language? | A location’s grooming rules for services, appointment-duration estimates, no-show handling, service history, rebooking cadence, reminders, and review requirements. Rebooking cadence is the timing that turns completed groom history into due-later/due-now/overdue/needs-review follow-up. | `docs/design/entity-atlas-revenue-opportunity-entities.md` definitions table; `domain/src/grooming/README.md` Operator summary. |
| Why does it exist? | To reduce staff time checking prior groomer notes, estimating appointment length, checking no-show history, and remembering rebooking/reminder cadence. | Revenue opportunity atlas definitions and purpose; `domain/src/grooming/README.md` lines 36-44. |
| Which labor, safety, or customer-trust problem does it help with? | Finds reviewable rebooking opportunities and appointment-duration/review requirements without overpromising customer sends, bookings, deposits, or handling/medical judgments. | Revenue opportunity atlas purpose/blocked sections; grooming README operator summary. |
| Which workflows and contracts feature it? | Grooming Rebooking / Retention, CRM Retention agent, Manager Daily Brief, Data Quality Hygiene; `domain::grooming::Contract`; storage grooming service-line records; provider catalog gap markers. | Workflow-to-entity map Grooming Rebooking row; revenue atlas workflow table; grooming README cross-crate relationships. |
| What other entities does it depend on or feed? | Depends on completed service history, customer/pet/location/staff identity, source refs, consent/contact permission, no-show/deposit evidence, provider catalog evidence if mapped; feeds retention opportunity, staff review packet, customer follow-up draft, follow-up outcome, storage projection. | Revenue atlas relationships; workflow-to-entity map; `domain/src/grooming/README.md` cross-crate relationships. |
| Who or what is authoritative? | Grooming domain contract plus approved service history and source refs; groomer or grooming manager for sensitive/exception decisions; provider catalog discovery is not policy by itself. | Revenue atlas source-of-record/human-role rows; grooming README authority paragraph. |
| What can an agent draft or recommend around it? | Validate cadence/history/no-show/reminder gates, draft customer follow-up or staff review tasks for approval, rank rebooking opportunities, record follow-up outcome. | Revenue atlas allowed-actions section; workflow-to-entity map. |
| What must stay human-reviewed or blocked? | Appointment booking/moving, groomer assignment, customer sends, discounts/credits/payment/deposits, provider/calendar writes, service promises, medical/handling judgments. | Revenue atlas blocked actions; grooming README blocked boundary. |
| Where is the code/Rustdoc/test evidence? | `domain/src/grooming/mod.rs`, `domain/src/grooming/README.md`, `storage/src/service_line/grooming.rs`, `storage/src/operations.rs`, `app/src/crm_retention.rs`, `domain/tests/petsuites_core_service_contracts.rs`, `domain/tests/domain_quality_patterns.rs`, `domain/tests/service_module_architecture.rs`, `storage/tests/*`. | Revenue atlas contracts table; grooming README cross-crate relationships. |

Verdict: PASS WITH DOC FOLLOW-UP. Grooming is understandable, but it is less self-contained than boarding/daycare because the revenue atlas gives grouped definitions rather than a full per-entity writer-value row with source-of-record, allowed, blocked, and evidence fields for Grooming Contract specifically. The module README fills the gap, but a non-coder has to combine two pages.

### Training contract / training package-session opportunity

| README question | Answer from non-coder docs | Evidence links |
| --- | --- | --- |
| What is this in pet-resort language? | A location’s training rules for program enrollment, trainer assignment/capacity, curriculum/progress, outcome documentation, package/session balances, and parent follow-up. Package/session opportunity is a reviewable chance to reserve, consume, reconcile, or follow up on reusable training sessions. | `docs/design/entity-atlas-revenue-opportunity-entities.md` definitions table; `domain/src/training/README.md` Operator summary. |
| Why does it exist? | To reduce repeated manual reconciliation around trainer capacity, session packages, missing progress evidence, outcome claims, and graduation/re-enrollment follow-up. | `domain/src/training/README.md` opening/operator summary; revenue atlas purpose. |
| Which labor, safety, or customer-trust problem does it help with? | Prevents staff from promising training assignments, progress/outcome claims, session use, or parent follow-up when care/behavior/payment/trainer/source evidence is incomplete. | Training README safety boundary; revenue atlas blocked-action section. |
| Which workflows and contracts feature it? | Booking Triage, Retention/Grooming Rebooking, Manager Daily Brief, Data Quality Hygiene; `domain::training::Contract`; storage training service-line records. | Entity index grooming/training/retail row; relationship map service-line table; revenue atlas workflow table. |
| What other entities does it depend on or feed? | Depends on customer, pet, location, staff/trainer, reservation, care, temperament, payment, approval, provider/catalog evidence after mapping, storage package/service records; feeds assignment/waitlist/review decisions, progress/outcome drafts, package-ledger reconciliation, follow-up plan, outcome evidence. | Training README cross-crate relationships; revenue atlas relationships. |
| Who or what is authoritative? | Training domain contract plus package/payment/source evidence; trainer/front-desk lead/GM for exceptions; provider/catalog evidence stays in `integrations::gingr` until mapped. | Revenue atlas authority table; training README authority paragraph. |
| What can an agent draft or recommend around it? | Validate enrollment, trainer availability, package/session balance, progress/outcome evidence, follow-up readiness; draft internal decisions, review packets, progress/follow-up text for review; record dispositions and outcomes. | Training README operator summary; revenue atlas allowed-actions section. |
| What must stay human-reviewed or blocked? | Live trainer assignments, waitlist movement, provider/PMS writes, customer messages, payment/package adjustments, credits/refunds, outcome/graduation claims, safety-sensitive behavior/care interpretations. | Training README safety boundary; revenue atlas blocked actions. |
| Where is the code/Rustdoc/test evidence? | `domain/src/training/mod.rs`, `domain/src/training/README.md`, `storage/src/service_line/training.rs`, `storage/src/operations.rs`, `domain/tests/petsuites_core_service_contracts.rs`, `domain/tests/domain_quality_patterns.rs`, `domain/tests/service_module_architecture.rs`, storage tests; Gingr DTO gap markers in `integrations/gingr/src/dto/training.rs` and `endpoint/catalog.rs`. | Revenue atlas contracts table; training README cross-crate relationships. |

Verdict: PASS WITH DOC FOLLOW-UP. Training is understandable to a non-coder, but like grooming it is split across the grouped revenue atlas and a source-adjacent module README. A future standalone or per-entry training table would make authority, allowed actions, blocked actions, and outcome fields faster to verify.

## Pass/fail table and required doc follow-ups

| Entity | Result | Missing or weak explanation flags | Required doc follow-up |
| --- | --- | --- | --- |
| Boarding contract | PASS | No blocking gaps. Value framing is present as policy lookup avoided and safe queue lanes. Evidence links are strong. | Optional polish only: add a one-line cross-link from `domain/src/boarding/README.md` back to the exact `entity-index.md` service-line row and core atlas Boarding contract anchor when anchors are stable. |
| Daycare contract | PASS | No blocking gaps. Value framing is strong around check-in throughput, ratio/eligibility review, and package opportunities. Evidence links are strong. | Optional polish only: add a one-line cross-link from `domain/src/daycare/README.md` back to the exact `entity-index.md` service-line row and core atlas Daycare contract anchor when anchors are stable. |
| Grooming contract / rebooking cadence | PASS WITH DOC FOLLOW-UP | Non-coder meaning is present, but the revenue atlas is grouped; grooming does not get the same full per-entity row shape that boarding/daycare get in the core atlas. Value framing is good for rebooking/search labor, but outcome fields are described at family level rather than directly beside the grooming entity. | Add a per-entry grooming table or subsection in `docs/design/entity-atlas-revenue-opportunity-entities.md` with the same fields as core atlas entries: source of record, human role, allowed actions, blocked actions, safe-use evidence/outcome fields, examples/non-examples. |
| Training contract / package-session opportunity | PASS WITH DOC FOLLOW-UP | Non-coder meaning is present, but the revenue atlas is grouped; training does not get the same full per-entity row shape that boarding/daycare get. Package/session value is clear, but labor/outcome proof is easier to find in the family-level evidence table than beside the training entity. | Add a per-entry training table or subsection in `docs/design/entity-atlas-revenue-opportunity-entities.md` with source of record, human role, allowed actions, blocked actions, safe-use evidence/outcome fields, examples/non-examples. |

## Cross-service findings

| Check | Finding |
| --- | --- |
| Missing non-coder explanations | No blocking gaps. Each sampled entity has plain pet-resort language before or near module/API details when read through the README/entity-index path. |
| Coder-only explanations | Boarding/daycare/grooming/training module READMEs still include many `domain::*` names and source paths, but each has an operator summary first. The entity index and family pages are the right non-coder entrypoints. |
| Weak value or labor-cost framing | Boarding/daycare are strong. Grooming/training are good but should move outcome/labor fields closer to the individual entity entries in the revenue atlas. |
| Missing relationships | No blocking gaps. Relationship map plus family pages show source evidence -> domain/service-line contract -> app packet/review gate -> outcome/storage. |
| Missing evidence links | No blocking gaps. All sampled entities link source/module paths and storage/test evidence. Follow-up is mostly anchor-level convenience, not missing proof. |
| Unsafe automation implication | No blocking gaps found. All sampled docs preserve the boundary that automation may draft/rank/summarize/validate/route/record, but not send, book, assign, mutate provider/PMS, move money, approve safety, or override policy. |

## Recommended next docs work, not implementation work

1. Extend `docs/design/entity-atlas-revenue-opportunity-entities.md` with individual Grooming Contract and Training Contract rows that mirror the detailed Boarding/Daycare rows in `docs/design/entity-atlas-petsuites-core-entities.md`.
2. When stable anchors exist, add direct back-links from the four service-line READMEs to their entity-index/family-atlas entries so non-coders can jump from source-adjacent docs back to the entity-first spine.
3. Keep this as documentation follow-up only. No Rust/API/storage implementation changes are implied by this QA pass.

## Verification performed

- Manually answered README/entity-index questions 1-9 for the four sampled service-line entities using the docs listed above.
- Checked the entity index, relationship map, workflow-to-entity matrix, service-line READMEs, and core/revenue family atlas pages for plain-English purpose, labor/value framing, relationships, authority, allowed automation, blocked actions, and evidence links.
- No source code behavior was changed or tested; this is a documentation comprehension QA artifact.
