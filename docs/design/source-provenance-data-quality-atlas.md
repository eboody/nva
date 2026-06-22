# Source, provenance, documents, audit, and data-quality atlas entries

Purpose: give non-coder writers and reviewers source-backed atlas entries for the entities that decide whether a pet-resort fact is safe to use. These entries are orientation pages; the linked Rust source, tests, and safety docs remain the behavior contract.

Use with the [entity atlas page template](entity-atlas-page-template.md), the [entity atlas inventory](entity-atlas-inventory.md), the [source evidence map](../safety/source-evidence-map.md), and the [Gingr source inventory](../integrations/gingr/source-inventory.md).

## Family summary

Source facts answer “where did this operational fact come from?” Provenance and source refs preserve the receipt. Data-quality issues make missing, stale, conflicting, ambiguous, duplicate, or sensitive facts visible. Documents and audit ids provide supporting evidence and traceability. None of these entities grant automation authority by themselves.

```text
Gingr / BI / labor / payroll / POS / manual import / document upload
  -> provider record, raw payload, document object, or import row
  -> source ref + provenance + payload/document hash
  -> source snapshot or extracted fact
  -> data-quality issues for missing/uncertain/stale/conflicting/sensitive facts
  -> workflow packet with allowed draft/rank/summarize actions and blocked actions
  -> staff review gate + audit/outcome evidence
```

## Entry: Source system and source fact

---
title: "Source system and source fact"
slug: "source-system-and-source-fact"
family: "source-provenance-data-quality"
status: "draft"
audience: ["general-manager", "regional-ops", "operations-analyst", "docs-writer", "compliance"]
plain_english_definition: "A source system is where an operational fact came from; a source fact is the provider, import, document, or staff-entered evidence before a workflow treats it as trusted business context."
primary_labor_problem: "Prevents staff and agents from reconciling dashboards by memory or treating every provider field as clean domain truth."
source_of_record: "domain::source::System, RecordRef, Provenance, source snapshots, documents, and workflow source facts; Gingr and other providers are evidence sources, not automatic domain truth."
authoritative_human_role: "front desk lead, general manager, operations analyst, or regional operator depending on the fact family"
workflow_links: ["manager-daily-brief", "data-quality-hygiene", "booking-triage", "checkout-completion", "gingr-source-normalization"]
source_paths:
  - "domain/src/source.rs"
  - "app/src/manager_daily_brief.rs"
  - "app/src/data_quality_hygiene.rs"
  - "storage/src/operations.rs"
  - "integrations/gingr/src/*"
rustdoc_contracts:
  - "domain::source::{System, RecordRef, Provenance}"
  - "domain::source::reservation::{Snapshot, Status, Assumption}"
  - "app::manager_daily_brief::SourceFact"
  - "storage::operations::StoredSourceRecordRef"
glossary_links:
  - "../glossary-source-data-terms.md#domainsourcesystemgingr-gingr"
  - "../glossary-source-data-terms.md#source-of-record"
  - "../glossary-source-data-terms.md#provider-record"
allowed_action_summary: "read, cite, summarize, compare, rank review work, and preserve source facts in packets/outcomes"
blocked_action_summary: "do not overwrite provider/PMS records, hide source ambiguity, treat provider ids as canonical customer/pet truth, or make customer/payment/schedule/safety decisions from source facts alone"
outcome_fields: ["source refs", "provenance", "data-quality issue", "review gate", "audit event", "outcome disposition"]
---

### Plain-English pet-resort definition

A source system is the place a fact came from: Gingr, a BI export, timeclock, payroll, capacity inventory, POS, a manual import, or a document pipeline. A source fact is the evidence from that place before NVA workflows decide how much to trust it.

### Purpose: labor-cost or safety problem

This entry helps managers, front desk leads, and operations analysts avoid repeated dashboard reconciliation and unsafe assumptions. It tells writers to say “Gingr showed this reservation status at this pull time” rather than “the resort has definitively approved this booking.”

### Workflows where it appears

| Workflow | How the entity appears | Safe workflow result |
| --- | --- | --- |
| [Manager Daily Brief](manager-daily-brief-measurable-labor-loop.md) | actions carry source facts and data-quality visibility | ranked internal action queue with source refs and blocked actions |
| [Data Quality Hygiene](data-quality-hygiene-labor-loop.md) | candidates are source issues, duplicates, profile gaps, service-line mappings, or freshness problems | cleanup draft or review disposition, not provider mutation |
| [Booking Triage](../workflows/booking-triage-agent.md) | reservation, pet, customer, vaccine, policy, payment, and provider evidence are evaluated before a staff packet | readiness packet and confirmation draft for review |
| Checkout Completion | checkout/source status and staff handoff are compared | review packet and audit draft, not automatic checkout mutation |
| Gingr Source Normalization | provider DTOs/endpoints/records are preserved before mapping | source snapshot and explicit gaps/issues |

### Relationships and adjacency

```text
Provider/source record
  -> System + RecordRef
  -> Provenance receipt
  -> optional source snapshot
  -> optional DataQualityIssue
  -> workflow SourceFact or Candidate
  -> ReviewGate + AuditEvent + OutcomeRecord
```

Source facts are adjacent to provider records, source snapshots, provenance, documents, data-quality issues, review gates, blocked actions, audit ids, and outcome records. They are not the same as domain truth, staff approval, customer permission, medical/safety clearance, or provider-write authority.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Domain source contract | [`domain/src/source.rs`](../../domain/src/source.rs) | `System` variants, `RecordRef`, `Provenance`, source reservation snapshots, assumptions, and Gingr adapter boundary |
| Source quality contract | [`domain/src/data_quality.rs`](../../domain/src/data_quality.rs) | issue kinds, severity, resolution status, BI visibility, workflow-blocking flag |
| Data hygiene app workflow | [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs) | candidate/action packet, safe actions, blocked actions, draft validation, outcome record |
| Safety citation map | [Source evidence map](../safety/source-evidence-map.md) | which safety claims can cite source/provenance, blocked actions, and outcome records |
| Gingr source inventory | [Gingr source inventory](../integrations/gingr/source-inventory.md) | provider boundary, read endpoint facts, documented gaps, and why Gingr is evidence not clean domain truth |
| Storage projection | [`storage/src/operations.rs`](../../storage/src/operations.rs) | stored source refs and outcome/source-code codecs |
| Rustdoc/module paths | `domain::source::{System, RecordRef, Provenance}`; `domain::source::reservation::Snapshot`; `app::data_quality_hygiene::*` | exact compiled contract once generated or published |

### Authoritative source system or human role

Ask “source of record for what?” instead of assigning one blanket owner.

| Fact or decision | Source of record | Human role when source is incomplete or sensitive |
| --- | --- | --- |
| Provider reservation/customer/pet facts | Gingr provider record plus provenance and source ref | front desk lead or general manager verifies exceptions |
| BI, timeclock, payroll, POS, capacity, or manual import facts | named source system plus extraction/import receipt | operations analyst or regional operator validates interpretation |
| Source mapping from provider field to domain meaning | domain source contract and adapter mapping tests | docs/engineering reviewer; operations confirms business meaning |
| Sensitive source evidence | quarantined payload/document status plus review gate | manager, compliance/security, or trained staff depending on sensitivity |
| Final operational decision | domain/app workflow contract plus staff review/approval | front desk lead, general manager, or regional operator |

### Allowed actions

Workflows may read source-backed evidence, preserve source refs, summarize what a source says, compare sources, surface ambiguity, rank review work, draft internal cleanup tasks, and record review outcomes when the app contract provides those fields.

### Blocked actions and review gates

A source fact must not authorize customer sends, booking/check-in/checkout changes, schedule changes, provider/PMS writes, payment/refund/discount movement, source-data deletion, source ambiguity hiding, vaccine/medical/safety clearance, policy exceptions, or personnel/staffing decisions. Those remain blocked unless a later source contract names an approved deterministic write path and human gate.

### Safe-use evidence and outcome fields

Safe use requires at least the source system, record id/source ref, provenance receipt where available, source freshness or pull/import time, issue status when quality is poor, review gate/reviewer role for sensitive decisions, audit id for operational decisions, and outcome record when claiming labor savings.

### Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | “Gingr reservation 123, pulled from `/api/v1/reservations` in batch `batch-2026-06-18`, says checked in.” | It names the system, endpoint, record, and extraction context. |
| Example | “Manual import row has missing pet id; data-quality issue blocks booking triage.” | It preserves source quality instead of guessing. |
| Non-example | “OwnerId 123 is the canonical customer.” | Provider ids are source identifiers until mapped and reviewed. |
| Non-example | “The source had a payment amount, so issue a refund.” | Payment movement requires separate policy and human authority. |

## Entry: Provenance and record ref

---
title: "Provenance and record ref"
slug: "provenance-and-record-ref"
family: "source-provenance-data-quality"
status: "draft"
audience: ["general-manager", "regional-ops", "operations-analyst", "docs-writer", "compliance", "it"]
plain_english_definition: "Provenance is the receipt for a source-backed fact; a record ref is the stable pointer to the upstream record."
primary_labor_problem: "Lets reviewers trace why a recommendation exists without re-opening every dashboard or raw export."
source_of_record: "domain::source::Provenance and domain::source::RecordRef; storage mirrors source refs for durable outcome records."
authoritative_human_role: "operations analyst or compliance reviewer for traceability; workflow owner for operational disposition"
workflow_links: ["manager-daily-brief", "data-quality-hygiene", "booking-triage", "checkout-completion", "source-normalization"]
source_paths:
  - "domain/src/source.rs"
  - "storage/src/operations.rs"
  - "storage/tests/data_quality_hygiene_outcome_storage.rs"
  - "storage/tests/manager_daily_brief_outcome_storage.rs"
rustdoc_contracts:
  - "domain::source::{Provenance, RecordRef, Endpoint, ExtractionBatchId, RequestScope, SchemaVersion, PayloadHash, RawPayloadRef}"
  - "storage::operations::StoredSourceRecordRef"
glossary_links:
  - "../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence"
allowed_action_summary: "cite, compare, store, and audit source receipts"
blocked_action_summary: "do not use provenance as proof that the fact is clean, current, complete, approved, or safe for live side effects"
outcome_fields: ["system", "endpoint", "record id", "related record ids", "extraction batch", "pulled at", "request scope", "schema version", "payload hash", "raw payload ref"]
---

### Plain-English pet-resort definition

Provenance is the receipt that says exactly where a fact came from: source system, endpoint or import route, record id, extraction batch, pull time, request scope, schema version, payload hash, and raw payload reference. A record ref is the short pointer derived from that receipt: source system plus record id. Persisted provenance value objects must re-run their smart-constructor validation on rehydration, so blank stored endpoints, payload hashes, raw payload refs, request scopes, schema versions, observed statuses, or Gingr provider ids/statuses cannot bypass the source boundary.

### Purpose: labor-cost or safety problem

This entry helps reviewers avoid re-checking raw dashboards by hand. If a manager action, hygiene candidate, or outcome cites provenance, a reviewer can trace it back to the exact source evidence and decide whether it is current, stale, ambiguous, or sensitive.

### Workflows where it appears

| Workflow | How provenance appears | Safe workflow result |
| --- | --- | --- |
| Manager Daily Brief | actions and outcomes cite source refs | measurable action queue with traceable evidence |
| Data Quality Hygiene | candidates carry issue provenance and source refs; outcomes preserve source refs | cleanup review with audit trail |
| Booking/checkout workflows | source facts and audit drafts name evidence | staff packet, not automatic mutation |
| Storage outcomes | stored source refs preserve proof after review | durable reporting/audit evidence |

### Relationships and adjacency

```text
Provenance
  -> RecordRef(system, record_id)
  -> SourceFact / DataQualityIssue / Candidate / Action
  -> Draft validation or staff review
  -> Audit event / Outcome record / Storage projection
```

Payload hashes and raw payload refs belong inside provenance unless a separate source-integrity page needs them. Related record ids help explain owner/pet/location/reservation relationships but do not resolve identity by themselves.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Domain provenance | [`domain/src/source.rs`](../../domain/src/source.rs) | required receipt fields and `RecordRef::from_provenance` |
| Storage source refs | [`storage/src/operations.rs`](../../storage/src/operations.rs) | durable source-ref projection and code mapping |
| Data-quality outcome test | [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs) | source refs survive encode/decode with cleanup outcome records |
| Manager outcome test | [`storage/tests/manager_daily_brief_outcome_storage.rs`](../../storage/tests/manager_daily_brief_outcome_storage.rs) | source refs are required for outcome evidence |
| Rustdoc/module paths | `domain::source::Provenance`; `domain::source::RecordRef`; `storage::operations::StoredSourceRecordRef` | exact compiled contract once generated or published |

### Authoritative source system or human role

| Fact or decision | Source of record | Human role when incomplete or sensitive |
| --- | --- | --- |
| Where the fact came from | `Provenance` fields | operations analyst checks traceability gaps |
| Which upstream record is being cited | `RecordRef` | workflow reviewer confirms it is the right customer/pet/reservation/location |
| Whether the raw payload is safe to inspect | raw payload ref plus sensitivity/document status | compliance/security or manager |
| Whether the fact can drive an action | app workflow contract plus review gate | front desk lead, manager, or regional operator |

### Allowed actions

Automation may attach provenance to source facts, create record refs, preserve raw payload references, compare payload hashes for drift, and store source refs on outcome records.

### Blocked actions and review gates

Provenance does not mean the source is correct. It must not be used to auto-resolve duplicate customers, ambiguous owner/pet relationships, unknown provider status, stale vaccine facts, quarantined sensitive payloads, payment decisions, provider writes, or customer sends.

### Safe-use evidence and outcome fields

A safe recommendation should show the source ref, pull/import time, endpoint or document source, payload/document hash when available, schema/version context, data-quality issue status if present, review gate, and outcome/audit id after staff action.

### Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | `domain::source::Provenance` with system `Gingr`, endpoint `/reservations`, record id, batch, pulled-at time, request scope, schema version, payload hash, raw payload ref. | This is a complete source receipt. |
| Example | `StoredSourceRecordRef` inside a data-quality outcome. | It proves the reviewed cleanup outcome remained traceable. |
| Non-example | Payload hash alone. | Hash proves drift/idempotency support, not operational meaning. |
| Non-example | “Source says so.” | Missing endpoint, record id, time, and schema context. |

## Entry: Source reservation snapshot

---
title: "Source reservation snapshot"
slug: "source-reservation-snapshot"
family: "source-provenance-data-quality"
status: "draft"
audience: ["front-desk", "general-manager", "operations-analyst", "docs-writer"]
plain_english_definition: "A point-in-time provider/source view of a reservation before it becomes trusted domain or workflow input."
primary_labor_problem: "Reduces manual reconciliation of reservation status, owner/pet links, service type, location, and checkout facts while preserving source uncertainty."
source_of_record: "domain::source::reservation::Snapshot plus Gingr provider endpoint/response evidence; not a confirmed domain reservation by itself."
authoritative_human_role: "front desk lead or general manager for operational exceptions; operations analyst for source-model gaps"
workflow_links: ["booking-triage", "checkout-completion", "manager-daily-brief", "data-quality-hygiene", "gingr-source-normalization"]
source_paths:
  - "domain/src/source.rs"
  - "integrations/gingr/src/endpoint/reservations.rs"
  - "integrations/gingr/src/response.rs"
  - "docs/integrations/gingr/source-inventory.md"
rustdoc_contracts:
  - "domain::source::reservation::{Snapshot, SnapshotBuilder, Status, OwnerPetRelationship, Assumption}"
  - "domain::source::gingr::reservation::Snapshot"
glossary_links:
  - "../glossary-source-data-terms.md#provider-record"
  - "../glossary-source-data-terms.md#source-of-record"
allowed_action_summary: "build point-in-time source snapshots, expose missing fields and assumptions, and emit data-quality issues"
blocked_action_summary: "do not treat source snapshots as approval to confirm bookings, check in/out, alter schedules, collect/refund money, or resolve owner/pet identity"
outcome_fields: ["provenance", "customer record id", "pet record id", "location record id", "service type record id", "source status", "owner-pet relationship", "assumptions", "data-quality issues"]
---

### Plain-English pet-resort definition

A source reservation snapshot is what the provider/source appeared to say about a booking or stay at one moment. It can include a customer record id, pet record id, location, service type, lifecycle status, owner/pet relationship confidence, assumptions, and provenance.

### Purpose: labor-cost or safety problem

This entry helps staff avoid cross-checking multiple Gingr screens or exports by hand while making source caveats visible. Missing IDs, unknown statuses, ambiguous owner/pet links, raw-payload retention gaps, and refresh-policy gaps become data-quality issues instead of quiet guesses.

### Workflows where it appears

| Workflow | How the snapshot appears | Safe workflow result |
| --- | --- | --- |
| Booking Triage | pre-domain reservation/customer/pet/source status evidence | readiness packet and review gate |
| Checkout Completion | source checkout status and missing/unknown evidence | completion packet, not automatic provider update |
| Manager Daily Brief | reservation/stay facts feed action ranking when source quality allows | source-backed manager action |
| Data Quality Hygiene | missing/stale/conflicting snapshot fields become cleanup candidates | review queue for data repair |
| Gingr Source Normalization | provider DTO -> Gingr snapshot -> source-agnostic snapshot | explicit mapping and gap documentation |

### Relationships and adjacency

```text
Gingr reservation endpoint / provider response
  -> Gingr adapter snapshot
  -> source-agnostic reservation snapshot
  -> data-quality issues for missing fields, unknown status, ambiguity, or assumptions
  -> booking/checkout/manager/data-quality workflow packet
```

The same promotion rule applies to other Gingr mapping candidates: provider evidence may become a mapped candidate only after required provider fields are present and domain constructors accept them. Missing retail category or active status remains a mapping error/review item instead of defaulting into a product category or offering status.

The snapshot is adjacent to provider records, domain reservations, analytics stay facts, data-quality issues, booking/checkout packets, and audit outcomes. It must not be confused with a confirmed booking, checkout authorization, payment state, or staff-reviewed identity match.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Domain source snapshot | [`domain/src/source.rs`](../../domain/src/source.rs) | snapshot fields, status enum, relationship confidence, assumptions, issue generation |
| Gingr reservation endpoints | [`integrations/gingr/src/endpoint/reservations.rs`](../../integrations/gingr/src/endpoint/reservations.rs) | provider request shapes and endpoint scope |
| Gingr response wrapper | [`integrations/gingr/src/response.rs`](../../integrations/gingr/src/response.rs) | raw/provider record shape and unknown fields |
| Source inventory | [Gingr source inventory](../integrations/gingr/source-inventory.md) | current endpoint facts and integration gaps |
| Adapter boundary doc | [Adapter boundary and labor source expansion](../integrations/gingr/adapter-boundary-and-labor-source-expansion.md) | DTO -> snapshot -> analytics flow and source-vs-domain boundary |
| Rustdoc/module paths | `domain::source::reservation::Snapshot`; `domain::source::reservation::Assumption` | exact compiled contract once generated or published |

### Authoritative source system or human role

| Fact or decision | Source of record | Human role when incomplete or sensitive |
| --- | --- | --- |
| Provider reservation status | provider endpoint response plus provenance | front desk lead verifies exceptions before acting |
| Customer/pet/location/service type source ids | source snapshot fields plus provider records | front desk lead or operations analyst resolves ambiguous ids |
| Owner/pet relationship confidence | `OwnerPetRelationship` and related record evidence | staff reviewer resolves ambiguous relationship |
| Source assumptions | `Assumption` variants and data-quality issues | operations analyst decides whether projection is safe |
| Operational booking/checkout decision | app workflow packet plus domain policy/review gate | front desk lead or general manager |

### Allowed actions

Workflows may build source snapshots from validated source parts, carry unknown provider status as `Status::Unknown`, preserve assumptions, generate data-quality issues, and use clean snapshots as review evidence.

### Blocked actions and review gates

A source snapshot blocks rather than authorizes action when required ids are missing, status is absent or unknown, owner/pet relationship is ambiguous, raw payload retention is unknown, refresh mutation policy is unknown, or sensitive evidence is quarantined. It must not auto-confirm bookings, mutate Gingr/PMS records, check pets in/out, change schedules, move payments, or resolve identity conflicts.

### Safe-use evidence and outcome fields

Safe use requires complete provenance, required source ids, known status, resolved owner/pet relationship, no workflow-blocking issues, explicit assumptions if any, and a workflow review gate when the source feeds a customer/pet/schedule/payment/safety decision.

### Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | Snapshot with provenance, customer/pet/location/service ids, `CheckedIn` status, resolved owner/pet link, and no blocking issues. | Usable as source evidence for a reviewed packet. |
| Example | Snapshot with `Status::Unknown { observed }` and a blocking data-quality issue. | Preserves provider text without pretending it is safe. |
| Non-example | Treating missing pet id as “unknown pet is fine.” | Missing required ids block workflow projection. |
| Non-example | Treating `quick_checkin` provider docs as read-safe source evidence. | Gingr inventory excludes side-effecting functions from the v0 read SDK surface. |

## Entry: Data-quality issue and hygiene packet

---
title: "Data-quality issue and hygiene packet"
slug: "data-quality-issue-and-hygiene-packet"
family: "source-provenance-data-quality"
status: "draft"
audience: ["front-desk", "general-manager", "regional-ops", "operations-analyst", "docs-writer"]
plain_english_definition: "A data-quality issue is a tracked source-data defect; a hygiene packet turns those defects into reviewable cleanup work."
primary_labor_problem: "Makes missing, stale, duplicate, incomplete, conflicting, ambiguous, unmapped, unclosed, and sensitive source facts visible before staff or agents waste time or take unsafe action."
source_of_record: "domain::data_quality::Issue for the defect; app::data_quality_hygiene::Packet/Candidate/Action for review work; storage outcome records for measured cleanup results."
authoritative_human_role: "front desk lead, general manager, operations analyst, or regional operator depending on issue sensitivity and workflow"
workflow_links: ["data-quality-hygiene", "manager-daily-brief", "regional-exceptions", "booking-triage", "vaccine-document"]
source_paths:
  - "domain/src/data_quality.rs"
  - "app/src/data_quality_hygiene.rs"
  - "storage/src/operations.rs"
  - "storage/tests/data_quality_hygiene_outcome_storage.rs"
  - "app/examples/data_quality_hygiene_local_smoke.rs"
rustdoc_contracts:
  - "domain::data_quality::{Issue, Kind, Severity, ResolutionStatus, FieldPath}"
  - "app::data_quality_hygiene::{Request, Packet, Candidate, Action, DraftSubmission, DraftValidation, OutcomeRecord}"
  - "storage::operations::DataQualityHygieneOutcomeRecord"
glossary_links:
  - "../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue"
  - "../glossary-workflow-state-terms.md#review-gate"
allowed_action_summary: "surface, rank, summarize, draft internal cleanup tasks, preserve ambiguity, estimate reconciliation minutes, and record reviewed outcomes"
blocked_action_summary: "no autonomous source repair, provider/PMS mutation, ambiguity hiding, sensitive payload exposure, customer sends, payment movement, or schedule/staffing changes"
outcome_fields: ["issue kind", "severity", "resolution status", "workflow blocking", "visible to BI", "source refs", "review gates", "actor persona", "before minutes", "actual minutes", "outcome", "notes"]
---

### Plain-English pet-resort definition

A data-quality issue is a named problem with source evidence: missing required field, unknown provider status, duplicate source record, ambiguous owner/pet relationship, unmapped service type, stale vaccine source, payment conflict, checkout evidence missing, unclosed reservation, incomplete pet profile, or quarantined sensitive payload. A hygiene packet groups those issues into reviewable work for a manager, front desk lead, operations analyst, or regional operator.

### Purpose: labor-cost or safety problem

This entry helps staff stop spending time rediscovering the same messy source facts and prevents unsafe automation from depending on them. Instead of hiding uncertainty, the app shows the issue, severity, source ref, resolution status, and whether the issue blocks workflow use.

### Workflows where it appears

| Workflow | How the issue or packet appears | Safe workflow result |
| --- | --- | --- |
| Data Quality Hygiene | candidates and actions are built from issues, freshness, sensitivity, and source refs | ranked cleanup queue and draft validation |
| Manager Daily Brief | data-quality issues remain visible in action source facts | manager sees caveat rather than bad recommendation |
| Regional Exceptions | visible-to-BI issues explain exception trends caused by source hygiene | reporting caveat and cleanup follow-up |
| Booking Triage / Checkout Completion | blocking issues stop unsafe readiness/completion suggestions | staff review gate instead of mutation |
| Vaccine/document workflows | stale/missing vaccine or unreviewed document facts stay blocked | trained-staff review |

### Relationships and adjacency

```text
Source snapshot / document / provider record
  -> DataQualityIssue(kind, severity, provenance, source ref, detected_at)
  -> HygieneCandidate(freshness, sensitivity, issue)
  -> HygieneAction(source refs, issue refs, review gates, labor estimate)
  -> DraftValidation(no stale packet, no missing refs, no blocked side effect)
  -> OutcomeRecord(actual minutes, reviewed resolution status, notes)
```

Data-quality issues are adjacent to field paths, provenance, source refs, source snapshots, documents, review gates, blocked actions, labor minutes, and storage outcomes. They are not the same as staff approval or automatic repair instructions.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Domain issue contract | [`domain/src/data_quality.rs`](../../domain/src/data_quality.rs) | issue kind, severity, resolution status, BI visibility, workflow-blocking flag |
| Hygiene workflow | [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs) | candidates, source freshness, sensitivity, allowed/blocked actions, draft validation, outcome record |
| Local smoke example | [`app/examples/data_quality_hygiene_local_smoke.rs`](../../app/examples/data_quality_hygiene_local_smoke.rs) | packet blocks customer/provider mutation and accepts/rejects drafts based on source refs and side effects |
| Storage outcome test | [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs) | source refs, resolution status, notes, action kind, persona, and minutes survive storage codecs |
| Labor-loop doc | [Data quality hygiene labor loop](data-quality-hygiene-labor-loop.md) | how cleanup labor is estimated and measured |
| Rustdoc/module paths | `domain::data_quality::Issue`; `app::data_quality_hygiene::Packet`; `storage::operations::DataQualityHygieneOutcomeRecord` | exact compiled contract once generated or published |

### Authoritative source system or human role

| Fact or decision | Source of record | Human role when incomplete or sensitive |
| --- | --- | --- |
| Issue kind/severity/source ref | `domain::data_quality::Issue` from source evidence | operations analyst or workflow owner verifies classification |
| Whether workflow should stop | `workflow_blocking` and app workflow validation | manager/front desk lead for operational gate |
| Cleanup priority/persona | hygiene `Action` owner persona and priority | general manager or regional operator |
| Sensitive evidence handling | `Sensitivity` plus document/quarantine status | compliance/security, manager, or trained staff |
| Resolution outcome | hygiene `OutcomeRecord` and storage projection | reviewer who performed/approved cleanup |

### Allowed actions

The workflow may summarize source evidence, rank hygiene actions, draft internal cleanup tasks, preserve ambiguity for review, estimate reconciliation minutes saved, validate that drafts carry source refs/issue refs/review gates, and record reviewed outcomes.

### Blocked actions and review gates

The hygiene workflow blocks customer messages, provider/PMS mutations, staff schedule changes, refund/discount/payment movement, hiding or auto-resolving source ambiguity, and exposing quarantined sensitive payloads. Draft validation must reject stale/unknown context packets, unsupported actions, missing source refs, missing issue refs, missing review gates, blocked side-effect requests, and attempted ambiguity resolution.

### Safe-use evidence and outcome fields

Safe use requires issue kind, severity, provenance, source ref, detected-at time, resolution status, BI visibility, workflow-blocking flag, candidate freshness/sensitivity, action issue refs/source refs, required review gates, actor persona, outcome code, actual minutes, and resolution status after review when stored.

### Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | Blocking `UnknownSourceStatus` issue with provenance and source ref. | It prevents a source status from becoming a silent workflow fact. |
| Example | Hygiene action to review stale vaccine source freshness owned by front desk lead, with source refs, issue refs, review gates, and labor estimate. | It is reviewable cleanup work, not automatic repair. |
| Non-example | “Auto-merge duplicate pet profiles because the names match.” | Duplicate/identity ambiguity requires staff review. |
| Non-example | “Hide the issue so the manager brief looks clean.” | Hiding source data-quality issues is explicitly blocked. |

## Entry: Document evidence and audit event

---
title: "Document evidence and audit event"
slug: "document-evidence-and-audit-event"
family: "source-provenance-data-quality"
status: "draft"
audience: ["front-desk", "general-manager", "compliance", "docs-writer", "it"]
plain_english_definition: "Document evidence is an uploaded or provider-supplied file with safety/review status; an audit event is the traceable id connecting source evidence, workflow decision, review gate, and resulting staff/customer action."
primary_labor_problem: "Routes vaccine proofs, waivers, medical records, photos, incident evidence, and provider/customer uploads through explicit review states instead of manual file inspection or untraceable decisions."
source_of_record: "domain::document values for classification/source/status/scanning/redaction/storage evidence; domain::audit::EventId and app audit drafts for traceability."
authoritative_human_role: "trained staff, manager, compliance/security reviewer, or approved sender depending on document class and action"
workflow_links: ["vaccine-document", "daily-updates", "booking-triage", "data-quality-hygiene", "incident-escalation"]
source_paths:
  - "domain/src/document.rs"
  - "domain/src/audit.rs"
  - "domain/src/entities.rs"
  - "app/src/data_quality_hygiene.rs"
  - "apps/api/src/http.rs"
rustdoc_contracts:
  - "domain::document::{Classification, Source, Status, VirusScanStatus, PiiRedactionStatus, OriginalFile, StorageRef}"
  - "domain::audit::EventId"
  - "domain::entities::audit::{Event, Subject, Action}"
glossary_links:
  - "../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence"
  - "../glossary-workflow-state-terms.md#review-gate"
allowed_action_summary: "classify, scan, redact, extract, store references, route for review, cite as source evidence, and record audit/outcome ids"
blocked_action_summary: "no live compliance clearance, medical acceptance, incident resolution, customer disclosure, provider write, retention/destruction decision, or sensitive-payload exposure without review"
outcome_fields: ["classification", "source route", "document status", "virus scan status", "PII redaction status", "storage ref", "sha256", "reviewer decision", "audit event id"]
---

### Plain-English pet-resort definition

Document evidence is a file or extracted artifact that may support a vaccine, waiver, medical, photo, incident, care, or audit decision. It carries classification, intake source, lifecycle status, virus-scan status, PII-redaction status, storage reference, hash, extraction/review state, and audit trail. An audit event id is the stable handle used to connect the source fact, workflow decision, review gate, and resulting staff/customer action.

### Purpose: labor-cost or safety problem

This entry helps staff avoid manually inspecting every upload before a workflow can use it, while preventing unscanned, unredacted, failed, unverified, superseded, rejected, or quarantined documents from becoming compliance or safety truth.

### Workflows where it appears

| Workflow | How document/audit evidence appears | Safe workflow result |
| --- | --- | --- |
| Vaccine Document | uploaded proof routes through scan/redaction/extraction/review | compliance draft or review packet, not automatic clearance |
| Booking Triage | vaccine/waiver/medical document state can block readiness | staff review gate |
| Daily Updates | photos/care notes and internal flags need approved inclusion | customer-safe draft after review |
| Incident Escalation | incident evidence needs severity/review/audit trail | manager escalation packet |
| Data Quality Hygiene | missing/stale/quarantined document evidence becomes a source-quality issue | cleanup/escalation candidate |

### Relationships and adjacency

```text
Customer/staff/provider document source
  -> Classification + Source route
  -> Virus scan + PII redaction + extraction status
  -> StorageRef + Sha256Digest + review status
  -> Source fact / DataQualityIssue / ReviewGate
  -> AuditEventId + OutcomeRecord
```

Documents are adjacent to source facts, provenance, vaccine/care/incident records, messages, review gates, approval records, storage refs, hashes, and audit events. They are not equivalent to verified compliance or approved customer disclosure until review gates clear.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Document domain contract | [`domain/src/document.rs`](../../domain/src/document.rs) | classifications, source routes, status states, virus scan, PII redaction, storage/hash values, blocked decisions |
| Audit id contract | [`domain/src/audit.rs`](../../domain/src/audit.rs) | event id purpose: correlate source fact, workflow decision, review gate, and resulting action |
| Domain entity audit records | [`domain/src/entities.rs`](../../domain/src/entities.rs) | document/audit entities and subject/action vocabulary |
| Data-quality workflow | [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs) | sensitive/quarantined payload handling and blocked actions |
| Safety citation map | [Source evidence map](../safety/source-evidence-map.md) | source evidence, review gates, and audit-friendly records |
| Rustdoc/module paths | `domain::document::*`; `domain::audit::EventId`; `domain::entities::audit::*` | exact compiled contract once generated or published |

### Authoritative source system or human role

| Fact or decision | Source of record | Human role when incomplete or sensitive |
| --- | --- | --- |
| File identity/integrity | immutable stored object, `StorageRef`, original metadata, content length, `Sha256Digest` | IT/compliance checks storage/hash anomalies |
| File safety for staff/agent use | `VirusScanStatus` and quarantine state | compliance/security or manager |
| PII-safe extracted text | `PiiRedactionStatus` and extraction evidence | compliance/security or trained reviewer |
| Vaccine/medical/incident acceptance | document status plus domain policy/review gate | trained staff or manager |
| Operational trace | `EventId` / audit record | reviewer or compliance can trace decision chain |

### Allowed actions

Workflows may classify, scan, redact, extract, store references, hash, route documents for review, attach document evidence to source facts or data-quality issues, cite audit ids, and record review/outcome evidence.

### Blocked actions and review gates

Documents must not trigger live compliance clearance, medical acceptance, incident resolution, customer disclosure, provider writes, retention/destruction decisions, or sensitive-payload exposure until the correct review gate clears. Failed virus scans, pending/failed PII redaction, rejected/quarantined/superseded statuses, and unknown source routes should block downstream safety or messaging use.

### Safe-use evidence and outcome fields

Safe use requires classification, source route, status, virus scan status, PII redaction status, storage ref, content hash, extraction/review evidence, reviewer role/decision, linked source ref/provenance where applicable, audit event id, and outcome disposition.

### Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | Vaccine proof document with customer upload source, passed virus scan, redacted PII where required, storage ref, hash, awaiting trained-staff review. | It can be routed for review while not yet being compliance truth. |
| Example | Audit event id attached to a data-quality cleanup outcome. | It lets reviewers trace the source fact, gate, and action. |
| Non-example | “Uploaded vaccine PDF means pet is cleared.” | Verification and policy review are separate. |
| Non-example | “Show quarantined sensitive payload to the agent so it can decide.” | Sensitive payload exposure is explicitly blocked. |

## Cross-entry writer checklist

Before describing an agent or workflow as safe with these entities, verify each claim against source:

1. Which source system or document route produced the fact?
2. Is there a source ref or provenance receipt with endpoint/import route, record id, batch, pull/import time, schema version, payload/document hash, and raw/storage ref where available?
3. Is the fact authoritative for the specific decision, or only provider/import evidence?
4. Are missing, unknown, stale, conflicting, duplicate, ambiguous, unmapped, unclosed, incomplete, or sensitive states represented as `DataQualityIssue`, source freshness, sensitivity, document status, or review gates?
5. Which human role can resolve the issue: front desk lead, general manager, operations analyst, regional operator, trained staff, compliance/security, or approved sender?
6. Which actions are explicitly blocked until quality or review improves?
7. What evidence proves safe use: source refs, provenance, document status/hash, review gates, audit event id, outcome record, actual minutes, notes, and resolution status?
8. If any answer is missing, write “review required” or “future source-contract gap” rather than implying live authority.

## Default blocked actions across this family

Unless a later linked contract explicitly authorizes the action through deterministic policy and human review, these entities never permit:

- autonomous customer/member sends;
- provider/PMS writes, record hiding, source deletion, or source ambiguity auto-resolution;
- booking, check-in, checkout, schedule, room, yard, service, or staff schedule changes;
- payment, refund, discount, deposit, package, subscription, or payroll movement;
- vaccine, medical, incident, temperament, safety, compliance, or policy approvals;
- exposure of quarantined sensitive payloads or unredacted PII;
- labor-savings claims without outcome records and source refs.

## Source-contract tests and evidence locations

| Evidence location | What it proves for atlas writers |
| --- | --- |
| [`app/examples/data_quality_hygiene_local_smoke.rs`](../../app/examples/data_quality_hygiene_local_smoke.rs) | Data-quality hygiene packets block provider/customer side effects, require source refs/issue refs, reject blocked side effects, and record outcomes. |
| [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs) | Data-quality hygiene outcomes preserve source refs, labor minutes, actor persona, action kind, resolution status, and review notes through storage codecs. |
| [`storage/tests/manager_daily_brief_outcome_storage.rs`](../../storage/tests/manager_daily_brief_outcome_storage.rs) | Manager daily brief outcomes require source refs and preserve labor/outcome evidence through storage codecs. |
| [`app/tests/manager_daily_brief_workflow_contracts.rs`](../../app/tests/manager_daily_brief_workflow_contracts.rs) | Manager brief source facts keep data-quality issues visible and block hiding source issues. |
| [`domain/src/source.rs`](../../domain/src/source.rs) | Source snapshots emit blocking issues for missing required ids, missing/unknown status, and ambiguous owner/pet relationships, plus warning assumptions for payload retention/refresh policy gaps. |
| [`domain/src/document.rs`](../../domain/src/document.rs) | Document evidence must pass intake, scan/redaction, review, storage/hash, and status gates before being used for compliance/safety decisions. |
