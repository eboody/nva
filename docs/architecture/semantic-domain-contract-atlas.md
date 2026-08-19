# Semantic domain contract atlas

Status: active architecture inventory for the current clean-slate tree.

Purpose: this atlas names the single current owner for the NVA pilot concept families that exist in source. It does not authorize historical facades, compatibility codecs, provider-owned app contracts, or unimplemented production issuers.

Safety baseline: source/provider systems provide evidence, not automatic authority. Customer sends, PMS/CRM/provider writes, schedule/capacity/staffing mutation, money/discount/refund/payment actions, production release, and NVA value claims remain blocked until review and system-of-record proof exist.

The trust path is `Observed<T> -> Candidate<T> -> Accepted<T>`. App contracts are provider-neutral. Serialized evidence uses validated rehydration and must fail closed when an invariant, relationship, or current authorization fact cannot be re-established. Executable authority is opaque and non-serializable; private issuance requires a trusted boundary that can bind the exact action, subject, scope, actor, gate, evidence, and chronology.

## Binding proof chain

```text
source evidence
→ source provenance and data-quality classification
→ validated semantic fact
→ relationship-checked aggregate
→ review/authorization proof
→ legal application action or draft-only recommendation
→ observed outcome and attribution evidence
```

The owner of a later step may consume earlier steps but must not silently re-own them. For example, `app::manager_daily_brief` may rank a staffing review action, but the demand fact remains `domain::analytics::service_demand::Fact`, review gates remain `domain::policy::ReviewGate`, and persisted outcomes remain storage projections until a canonical domain outcome is introduced.

## Maturity key

- L1: concept named.
- L2: concept structurally represented.
- L3: invariants and relationships enforced by types or fallible constructors.
- L4: adopted across domain, application, storage, API, and provider boundaries.

## Canonical concept inventory

| Concept family | Canonical owner/module | Boundary rule | Current adoption | Proof |
| --- | --- | --- | --- | --- |
| Location, customer, pet, reservation, document, message, approval, incident | `domain::entities` plus concept modules such as `domain::customer`, `domain::reservation`, and `domain::document` | Provider and storage identifiers remain evidence until fallible promotion establishes the owned identity and relationships. | Domain entities feed app workflows; storage and API adapt rather than re-own them. | Domain entity, aggregate, source, app, and storage contract tests. |
| Source, provenance, identity matching, data quality | `domain::source` and its `domain::source::System`, plus `domain::identity` and `domain::data_quality` | Raw/provider facts remain observed evidence. Ambiguity or missing provenance blocks promotion. | Gingr mapping is quarantined in `integrations/gingr`; provider-neutral app requests consume owned source/domain values. | `domain/tests/source_code_contracts.rs` and app data-quality tests. |
| Consent, access, and allowed use | `domain::consent`, `domain::access`, `domain::policy` | Serializable status labels do not establish accepted consent, authenticated identity, or executable permission. | App workflows consume scoped facts and explicit review gates. | Consent, authority, CRM-retention, and message compile/runtime tests. |
| Lead response and customer intelligence | `domain::lead::response`, `domain::customer::intelligence` | Source events, notes, visibility, consent, and review evidence are relationship-checked; queue/contact authority remains opaque. | Booking, retention, manager-brief, and API consumers use canonical owners directly. | `domain/tests/semantic_rehome_contracts.rs` plus app/API workflow tests. |
| Demand, labor, capacity, and time buckets | `domain::analytics::service_demand`, `domain::operations::labor`, `domain::operations::capacity`, `domain::operations::time_bucket` | Facts and recommendations cite source evidence and manager gates; they cannot mutate staffing or schedules. | Manager Daily Brief consumes canonical facts and emits reviewable actions. | Domain operations and manager-brief app/storage tests. |
| Knowledge and assistant packets | `domain::agent::knowledge`, `domain::agent::assistant`, core documents in `domain::entities` and `domain::document` | Answers cite approved applicable knowledge or escalate; packets are decision support, not tool authority. | Permissioned-knowledge app/API paths use canonical contracts. | Domain model, permissioned-knowledge, and API tests. |
| Finance, money, and outcomes | `domain::money`, `domain::payment`, `domain::analytics::finance`, `domain::analytics::outcome`; workflow-specific reports remain in `app` | Financial recommendations need review; caller-created outcome labels prove no completion, measured labor, realized savings, payment, or value. | Site-finance and outcome evidence cross app/storage/API through explicit projections. | Money, finance, site-finance, outcome, and storage tests. |
| Executable action authority | Opaque concept-owned capabilities and narrow trusted issuers | `Authority<Action, Subject, Scope>` is separate from evidence. No serde label, historical approval, caller actor field, or persisted status can construct it. | Most issuers are absent or test-only; current storage handoff admission consumes exact current-reviewer capability and persisted binding. | Authority inventory gate and compile-fail/runtime authority suites. |

## Required relationship matrix

| Relationship | Canonical owner | Required proof | Cardinality and gate | Current implementation | Gaps / migration target |
| --- | --- | --- | --- | --- | --- |
| lead ↔ customer/pet/site/service | `domain::lead::response`, with source links in `domain::source::reservation::Snapshot` and validated reservation relationships in `domain::entities::Reservation` | source provenance, related provider IDs, identity match, data-quality status | one lead event binds one site and intended service; unresolved customer/pet identity remains none/candidate/ambiguous until review | canonical lead response values consume source and identity evidence directly | outbound contact remains queue-only and authority-gated |
| event ↔ SLA ↔ attempt ↔ message ↔ conversion | `domain::lead::response`; stored messages remain `domain::entities::Message` | event provenance, SLA target/due status, attempt review gate, message approval, conversion attribution with source refs | one event has one SLA and many ordered attempts; conversion stays reported evidence | `Event`, `ResponseSla`, `ContactAttempt`, `ResponsePacket`, and `ConversionAttribution` live under the lead owner | no live send or conversion authority is inferred |
| note ↔ evidence/visibility/consent | `domain::customer::intelligence` with consent in `domain::consent` | source/audit refs, visibility, allowed use, consent evidence, exact customer relationship | marketing use requires accepted note and accepted consent authority | app CRM retention remains suppressed when current opaque acceptance is unavailable | serialized accepted-looking labels remain observations only |
| demand ↔ coverage ↔ shift/skill | `domain::analytics::service_demand`, `domain::operations::labor`, `domain::operations::capacity`, and `domain::staff` | source refs, location/day/service, role/skill, time bucket, manager gate | recommendations consume relationship-compatible demand and coverage | Manager Daily Brief ranks reviewable actions from canonical facts | no schedule or staffing mutation authority exists |
| document ↔ applicability ↔ citation | `domain::entities::Document`, `domain::document`, `domain::agent::knowledge`, and `domain::agent::assistant` | document status, storage/source ref, applicability scope, approved citation section | an answer cites one-or-more approved applicable documents or escalates | permissioned knowledge uses the canonical document identity | citations do not authorize tools or side effects |
| recommendation ↔ reviewed action ↔ reported outcome | `domain::workflow`, `domain::entities::approval`, workflow-specific app outcomes, and `domain::analytics::outcome` for generalized observations | review gate, source evidence, action relationship, caller-reported outcome correlation | serializable outcome labels remain nonclaimable even when identifiers match | manager brief `OutcomeRecord::labor_savings_claim_for_action` unconditionally returns `NotClaimed` | any future value claim requires a new opaque, non-serializable measurement authority |

## Serde and boundary rules

- Domain/app values that deserialize from stored packets use validated rehydration; builder-required fields are not sufficient validation.
- Provider DTOs and storage records may preserve provider/storage wire vocabulary, but promotion into domain facts must be explicit and fallible when validation, normalization, authorization, or trust changes.
- `From` is allowed only for total, lossless, authority-neutral conversion; use `TryFrom` or named promotion for source/provider/storage/domain crossing.
- Sensitive note/body/message text must avoid casual `Debug`/`Display` leakage and should expose disclosure only through approved renderers or review packets.

## Runtime/storage adoption snapshot

- L4-ish adopted slices today: service-line contracts through `domain::operations` ↔ `storage::service_line`, data-quality hygiene source/read-model/outcome path, manager daily brief app/storage/API proof, and local full-chain smoke draft workflow.
- Canonical access, consent, identity, lead-response, customer-intelligence, labor, capacity, time-bucket, assistant, knowledge, finance, and outcome definitions live under their concept owners; no catch-all facade remains.
- Gingr integration owns provider endpoints, DTOs, transport, webhook parsing, and mapping. It must keep provider IDs and quirks at the boundary until promoted into `domain::source` or canonical domain values.

## Current adoption evidence

| Workstream | Current model | Evidence paths | Fail-closed boundary |
| --- | --- | --- | --- |
| Lead response source to queue-only review | `domain::lead::response` | `domain/src/lead.rs`, `domain/tests/semantic_rehome_contracts.rs`, app/API lead consumers | No live contact; opaque queue authority is private. |
| CRM retention and customer intelligence | `domain::customer::intelligence`, `domain::consent`, `app::crm_retention` | `domain/src/customer.rs`, `app/src/crm_retention.rs`, manager-brief/storage consumers | Accepted-looking serialized evidence cannot create eligibility, drafting, contact, conversion, completion, or value. |
| Capacity/labor to Manager Daily Brief | canonical analytics and operations modules | `domain/src/operations.rs`, `app/src/manager_daily_brief.rs`, storage tests | Recommendations require source and manager gates and cannot mutate staffing or schedules. |
| Permissioned knowledge | canonical agent/document owners | `domain/src/agent.rs`, `app/src/permissioned_knowledge.rs`, API tests | Missing approval, applicability, or citation forces escalation. |
| Site finance and outcomes | canonical analytics, money, app, and storage owners | `domain/src/analytics.rs`, `app/src/site_finance.rs`, `storage/src/operations.rs` | Caller-created completion labels remain nonclaimable observations. |
| Approval/outbox infrastructure | `domain::entities::approval`, app ports, and storage adapters | `storage/src/operations/approval_outbox.rs` and authority tests | Only current trusted identity plus exact persisted binding can issue one-shot handoff authority. |

## Residual owner decisions safely defaulted

No human product-owner decision is blocking this atlas. Defaults used here:

1. Keep one canonical domain owner for each concept; do not reintroduce catch-all or parallel facades.
2. Treat consent/visibility/allowed-use as shared authorization vocabulary, not app-local booleans.
3. Treat generic outcomes as a future domain module only after at least one more vertical slice proves common shape.
4. Keep live execution out of scope; legal application actions remain reviewed/draft artifacts until a separate side-effect authorization card creates capabilities.
