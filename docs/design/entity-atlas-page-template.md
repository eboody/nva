# Entity atlas page template

Purpose: give non-coder documentation writers a reusable page contract for entity-atlas entries. An entity page should help a resort operator, reviewer, or docs maintainer understand what a business concept means, where it appears in labor-saving workflows, which source or human role owns it, and what automation must not do with it.

Use this template after checking the [entity atlas inventory](entity-atlas-inventory.md), the [entity atlas evidence-link anchors](entity-atlas-evidence-link-anchors.md), the [documentation style guide](../quality/nva-documentation-style-guide.md), and the relevant source/Rustdoc/test contract. Markdown pages are orientation and review aids; they do not create new product behavior beyond the linked source, tests, and Rustdoc.

## Reusable frontmatter schema

Use compact frontmatter at the top of each entity page so later indexers can group pages consistently. Keep values short and human-readable.

```yaml
---
title: "Reservation"
slug: "reservation"
family: "reservations-boarding-daycare"
status: "draft|reviewed|published"
audience: ["front-desk", "general-manager", "regional-ops", "docs-writer"]
plain_english_definition: "The booking or stay record that ties a customer, pet, location, service, dates, status, add-ons, and policy stops together."
primary_labor_problem: "Reduces repeated booking/readiness reconciliation before staff confirm, check in, check out, or follow up."
source_of_record: "Gingr reservation evidence plus domain reservation contract; staff approval for policy exceptions."
authoritative_human_role: "front desk lead or general manager"
workflow_links: ["booking-triage", "checkout-completion", "daily-updates"]
source_paths:
  - "domain/src/entities.rs"
  - "domain/src/reservation/mod.rs"
  - "app/src/booking_triage.rs"
rustdoc_contracts:
  - "domain::entities::Reservation"
  - "domain::entities::reservation::{Id, Status, Source}"
  - "domain::reservation::{MinimumAgeWeeks, AddOnLabel, TransitionReason}"
glossary_links:
  - "../glossary-source-data-terms.md#source-of-record"
  - "../glossary-workflow-state-terms.md#review-gate"
allowed_action_summary: "draft, rank, validate, summarize, and record review evidence only where the app workflow allows it"
blocked_action_summary: "no autonomous customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, or safety/policy approvals"
outcome_fields: ["review disposition", "actual minutes saved", "source refs", "staff persona", "follow-up status"]
---
```

If a page needs to stay easier for non-coders to copy/paste, use this table instead of YAML:

| Field | Writer value |
| --- | --- |
| Title / slug |  |
| Entity family |  |
| Plain-English definition |  |
| Labor or safety problem reduced |  |
| Workflows where it appears |  |
| Related entities / adjacency |  |
| Source paths |  |
| Rustdoc/module/type contracts |  |
| Source of record |  |
| Authoritative human role |  |
| Allowed actions |  |
| Blocked actions / review gates |  |
| Safe-use evidence / outcome fields |  |
| Examples / non-examples |  |
| Glossary cross-links |  |

## Page body template

Copy the structure below for each entity page. Replace bracketed prompts with concrete pet-resort language before adding code paths.

### 1. Plain-English pet-resort definition

Open with one or two sentences that a non-coder at a resort can understand.

Good shape:

> A [Reservation] is the booking or stay record that ties a pet, customer, location, service, date window, status, add-ons, deposit, and policy stops together. Staff use it to decide whether a boarding stay can be confirmed, checked in, checked out, or followed up.

Avoid opening with:

> `domain::entities::Reservation` is a boundary aggregate.

### 2. Purpose: labor-cost or safety problem

Name whose time or risk this page helps reduce.

Include:

- Worker or reviewer: front desk, general manager, daycare lead, groomer, trainer, regional ops, docs writer, or engineer.
- Repeated work or safety risk: manual dashboard reconciliation, stale vaccine review, duplicate customer cleanup, checkout exception audit, unsafe customer-message send, policy exception, source-system mismatch.
- Safe outcome: ranked queue, reviewed draft, deterministic policy check, source-backed summary, or recorded outcome.

Template sentence:

> This page helps [role] avoid [manual work or safety risk] by explaining [entity] in plain English, showing which workflows use it, and naming what remains human-reviewed.

### 3. Workflows where it appears

List the workflows that read, produce, or review this entity. Do not imply a workflow can mutate a live system unless the app contract explicitly allows it.

| Workflow | How the entity appears | Safe workflow result |
| --- | --- | --- |
| [Manager Daily Brief](manager-daily-brief-measurable-labor-loop.md) | [source-backed action, risk, outcome, or reporting field] | [ranked action, brief item, outcome record] |
| [Data Quality Hygiene](data-quality-hygiene-labor-loop.md) | [missing/stale/duplicate/conflicting fact] | [review candidate, cleanup draft, disposition] |
| [Booking Triage](../workflows/booking-triage-agent.md) | [reservation/customer/pet/care facts] | [readiness packet, confirmation draft, review gate] |

### 4. Relationships and adjacency

Show what the entity touches and what it must not be confused with.

Include at least these relationship types when relevant:

- Parent/owner: customer owns pets; location owns operating policy; source system owns provider evidence.
- Subject: reservation, pet, customer, document, incident, message, or workflow event subject.
- Source evidence: provenance, source ref, provider record, raw payload reference, mapping candidate.
- Policy/safety: review gate, blocked action, approval record, care profile, vaccine rule, temperament/group-play eligibility.
- Outcome/reporting: labor minutes, outcome record, reporting group, staff persona, actual disposition.
- Runtime/storage: storage projection, API route, worker runtime, tool port, provider endpoint.

Suggested diagram style for non-coder pages:

```text
Gingr reservation evidence
  -> source ref / provenance
  -> domain reservation
  -> booking triage packet
  -> staff review gate + confirmation draft
  -> outcome / audit evidence
```

### 5. Contracts and source/Rustdoc links

Separate operator meaning from authority. Source and Rustdoc are the behavioral contract; this Markdown page is a guide.

Use a compact table:

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Domain source | [`domain/src/entities.rs`](../../domain/src/entities.rs) | business truth, ids, status, relationships, invariants |
| Domain module | [`domain/src/reservation/mod.rs`](../../domain/src/reservation/mod.rs) | reservation-specific policy, validation, status meaning |
| App workflow | [`app/src/booking_triage.rs`](../../app/src/booking_triage.rs) | packet fields, safe actions, blocked actions, review gates |
| Storage projection | [`storage/src/operations.rs`](../../storage/src/operations.rs) | durable record shape, source refs, outcome/reporting fields |
| Provider boundary | [`integrations/gingr/src/endpoint/reservations.rs`](../../integrations/gingr/src/endpoint/reservations.rs) | provider evidence and request shape, not domain truth |
| Rustdoc path | `domain::entities::Reservation` | exact compiled contract once generated or published |

If rendered Rustdoc is not present, write “Rustdoc/module path” and link the source path. Do not invent rendered Rustdoc URLs.

### 6. Authoritative source system or human role

Name the authority for each kind of fact. Ask “source of record for what?” instead of naming one blanket owner.

Examples:

| Fact or decision | Source of record | Human role when source is incomplete or sensitive |
| --- | --- | --- |
| Provider reservation status | Gingr/provider reservation evidence plus source ref | front desk lead verifies exceptions |
| Business policy exception | domain policy contract plus local manager decision | general manager |
| Vaccine/safety readiness | vaccine/care/temperament evidence plus policy gate | trained staff or manager |
| Customer outreach approval | app draft + approval record | front desk, manager, or approved sender |
| Outcome minutes saved | workflow outcome record | reviewer who performed the work |

### 7. Allowed actions

Say what automation or app workflows may safely do with this entity. Use verbs that match the app contracts.

Usually allowed:

- read source-backed evidence;
- map provider records into explicit candidates;
- validate required fields and policy gates;
- draft staff/customer-facing text for review;
- rank or summarize work queues;
- create internal task recommendations;
- record review dispositions, audit events, and outcome evidence.

Phrase carefully:

> The workflow may draft a confirmation message and explain missing facts; it may not send the message or write confirmation back to the provider unless a separate approved tool and human gate exist.

### 8. Blocked actions and review gates

Every entity page that touches customers, pets, safety, money, schedules, provider/PMS records, or policy exceptions must include a no-go boundary.

Default blocked actions unless a linked contract explicitly allows them:

- customer/member sends;
- PMS/provider writes, record hiding, or source-data deletion;
- booking, check-in, checkout, schedule, room, yard, or service mutations;
- payment, refund, discount, deposit, or package/session-balance movement;
- vaccine, medical, incident, temperament, safety, or policy approvals;
- personnel actions or staffing schedule changes;
- secret-dependent or live external side effects.

Link glossary entries for [review gate](../glossary-workflow-state-terms.md#review-gate), [blocked action](../glossary-workflow-state-terms.md#blocked-action), [draft](../glossary-workflow-state-terms.md#draft), and [outcome capture](../glossary-workflow-state-terms.md#outcome-capture) where these terms first appear.

### 9. Safe-use evidence and outcome fields

Explain what proof makes the entity safe to use in a recommendation and how outcomes are measured.

Evidence fields to look for:

- source refs and provenance;
- provider endpoint or raw payload reference;
- schema/payload hash or extraction batch when available;
- review gate name and reason;
- approval record or reviewer identity/role;
- audit event subject/action;
- outcome disposition;
- actual minutes saved or wasted;
- follow-up status, conversion, suppression, or correction reason;
- data-quality issue kind, severity, and resolution status.

Do not claim labor savings from intent alone. Name the outcome record or say the page describes a future/measurable loop.

### 10. Examples and non-examples

Include at least two examples and two non-examples so writers do not turn helper DTOs, provider ids, or storage codes into business entities accidentally.

Example format:

| Type | Item | Why |
| --- | --- | --- |
| Example | Reservation | Staff use it to reconcile booking readiness, checkout, daily updates, and retention. |
| Example | Provenance | Staff and reviewers need to know where a source-backed recommendation came from. |
| Non-example | `OwnerId` wrapper by itself | It is a provider identifier; mention it under the Gingr/provider boundary page unless it has independent operator meaning. |
| Non-example | raw payload hash by itself | It is audit support; document it inside Provenance unless a source-integrity page needs it. |

### 11. Glossary cross-links

Use glossary links near the first use of a term, not only at the end. Recommended starting links:

- [domain](../glossary-architecture-terms.md#domain), [app](../glossary-architecture-terms.md#app), [storage](../glossary-architecture-terms.md#storage), and [integrations/gingr](../glossary-architecture-terms.md#integration-integrationsgingr) for layer names;
- [DTO](../glossary-architecture-terms.md#dto), [adapter](../glossary-architecture-terms.md#adapter), and [tool port](../glossary-architecture-terms.md#tool-port-apptools) for technical boundary terms;
- [Gingr](../glossary-source-data-terms.md#domainsourcesystemgingr-gingr), [source-of-record](../glossary-source-data-terms.md#source-of-record), [provider record](../glossary-source-data-terms.md#provider-record), [provenance](../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence), and [data-quality issue](../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue) for evidence terms;
- [draft](../glossary-workflow-state-terms.md#draft), [review gate](../glossary-workflow-state-terms.md#review-gate), [blocked action](../glossary-workflow-state-terms.md#blocked-action), [workflow packet](../glossary-workflow-state-terms.md#workflow-packet), and [outcome capture](../glossary-workflow-state-terms.md#outcome-capture) for safety/workflow terms.

## When to merge small technical DTOs into a family page

Give a candidate its own page when a non-coder needs to understand it independently to make or review a resort decision.

Create a standalone page when at least one is true:

- Staff or reviewers must distinguish it from nearby concepts to avoid unsafe action. Example: `Review Gate`, `Blocked Action`, `Approval Record`.
- It crosses multiple workflows. Example: `Provenance`, `Reservation`, `Customer`, `Pet`, `Labor Minutes`.
- It carries source authority or source uncertainty. Example: `Source System`, `Provider Record`, `Data Quality Issue`.
- It controls customer, pet-safety, schedule, payment, provider-write, or policy behavior. Example: `Vaccine Record`, `Care Profile`, `Incident`.
- It is the main packet a workflow produces or reviews. Example: `Booking Triage Packet`, `Manager Daily Brief Packet`.
- It has outcome/labor measurement fields that prove value. Example: `Manager Daily Brief Outcome Record`, `Data Quality Hygiene Outcome Record`.

Merge into a family page or relationship section when most are true:

- The item is a scalar, id wrapper, enum code, DTO helper, request builder field, storage code, or adapter detail.
- The item has no independent operator workflow without its parent page.
- The safety story is identical to its parent entity.
- A writer would repeat the same source/Rustdoc links and examples as the parent page.
- The item exists mainly to support validation, serialization, or provider mapping.

Merge examples:

| Candidate | Recommended placement | Reason |
| --- | --- | --- |
| `gingr::endpoint::OwnerId` | Gingr Provider Boundary page | Provider id wrapper; not canonical customer truth. |
| `domain::source::PayloadHash` | Provenance page | Audit support for source chain-of-custody. |
| `storage::operations::ManagerDailyBriefReportingGroup` | Outcome/labor measurement family page | Reporting classifier; meaningful with outcome records. |
| `domain::daycare::assignment` subtypes | Daycare Contract relationship section | Operationally useful, but likely one daycare page until a yard-assignment workflow exists. |
| Provider grooming/training DTO gap markers | Gingr Provider Boundary or service-line page | Prevents invented source authority; not a standalone operator entity. |

Decision question for writers:

> If this item disappeared from the atlas index, would a non-coder reviewer lose an important source, safety, labor, or approval distinction? If yes, give it a page. If no, merge it into the parent family page and link the source contract there.

## Completed miniature example: Labor minutes

This example is intentionally low-risk: it explains measurement of time saved, not a live customer, pet-safety, schedule, provider-write, or payment action.

---
title: "Labor minutes"
slug: "labor-minutes"
family: "outcomes-and-labor-measurement"
status: "example"
audience: ["general-manager", "regional-ops", "docs-writer"]
plain_english_definition: "The minute-based estimate or recorded value used to say how much staff time a workflow might save or actually saved."
primary_labor_problem: "Keeps labor-saving claims tied to measured or reviewable outcomes instead of vague automation promises."
source_of_record: "workflow outcome records and reviewer-entered disposition/minute fields"
authoritative_human_role: "manager or staff reviewer who performed the work"
workflow_links: ["manager-daily-brief", "data-quality-hygiene", "regional-exceptions"]
source_paths:
  - "app/src/manager_daily_brief.rs"
  - "app/src/data_quality_hygiene.rs"
  - "storage/src/operations.rs"
rustdoc_contracts:
  - "app::manager_daily_brief::{LaborMinutes, AggregateLaborMinutes, LaborImpactEstimate}"
  - "app::data_quality_hygiene::{LaborMinutes, AggregateLaborMinutes}"
  - "storage::operations::{StoredManagerDailyBriefLaborMinutes, StoredDataQualityHygieneLaborMinutes}"
glossary_links:
  - "../glossary-workflow-state-terms.md#outcome-capture"
allowed_action_summary: "estimate, display, aggregate, and record reviewed minutes where the workflow contract provides fields"
blocked_action_summary: "do not claim proved savings without outcome evidence; do not use minutes to authorize staffing, payment, or customer actions by themselves"
outcome_fields: ["estimated minutes", "actual minutes", "review disposition", "action kind", "source refs", "reporting group"]
---

### Plain-English pet-resort definition

Labor minutes are the time units the NVA docs and app workflows use to say “this recommendation could save staff time” or “this reviewed action actually saved or cost staff time.” They turn manager-brief and data-cleanup claims into something a general manager or regional ops leader can audit.

### Purpose: labor-cost or safety problem

This page helps managers avoid vague “AI saved time” claims by requiring each labor-saving recommendation to name the manual task, source evidence, and outcome field that can later confirm or correct the estimate.

### Workflows where it appears

| Workflow | How labor minutes appear | Safe workflow result |
| --- | --- | --- |
| [Manager Daily Brief](manager-daily-brief-measurable-labor-loop.md) | brief actions can include estimated and recorded minutes for demand/staffing, checkout, follow-up, care-watch, or revenue tasks | ranked manager action plus outcome record |
| [Data Quality Hygiene](data-quality-hygiene-labor-loop.md) | cleanup candidates estimate rework avoided and record whether the cleanup saved time | reviewed cleanup disposition plus outcome record |
| Regional exceptions | aggregated outcomes can show where locations repeatedly lose or save labor | reporting signal, not an automatic staffing change |

### Relationships and adjacency

```text
source-backed workflow packet
  -> suggested action or cleanup candidate
  -> estimated labor minutes
  -> staff/manager review disposition
  -> actual labor minutes in outcome record
  -> storage/reporting projection
```

Labor minutes are adjacent to `OutcomeRecord`, `SourceFact`, `ActionKind`, reviewer persona, source refs, reporting group, and storage projections. They are not the same as payroll, staff scheduling authority, payment movement, or a proof of savings before review.

### Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Manager brief app source | [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs) | `LaborMinutes`, aggregate minutes, estimates, action kinds, and outcome record fields |
| Data hygiene app source | [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs) | cleanup candidate/action minute estimates and outcome records |
| Storage projection | [`storage/src/operations.rs`](../../storage/src/operations.rs) | stored minute values and reporting/outcome records |
| Design doc | [Manager daily brief measurable labor loop](manager-daily-brief-measurable-labor-loop.md) | how estimates become reviewed outcomes |
| Design doc | [Data quality hygiene labor loop](data-quality-hygiene-labor-loop.md) | how cleanup labor is estimated and recorded |
| Rustdoc/module paths | `app::manager_daily_brief::{LaborMinutes, AggregateLaborMinutes, LaborImpactEstimate}`; `app::data_quality_hygiene::{LaborMinutes, AggregateLaborMinutes}` | exact compiled contract once generated or published |

### Authoritative source system or human role

| Fact or decision | Source of record | Human role when source is incomplete or sensitive |
| --- | --- | --- |
| Estimated minutes | app workflow estimate tied to source facts and action kind | manager/reviewer checks whether estimate is plausible |
| Actual minutes saved/wasted | submitted outcome record | staff member or manager who performed/reviewed the work |
| Reporting rollup | storage projection from outcome records | regional ops reviews trend interpretation |

### Allowed actions

Workflows may calculate, show, aggregate, and store labor-minute estimates and reviewed actuals when the source/app/storage contracts provide those fields. Docs may say the minutes support measurement and reporting.

### Blocked actions and review gates

Labor minutes do not authorize automatic staffing changes, pay decisions, schedule changes, customer communication, provider writes, discounts, or policy exceptions. A page must not claim a workflow “saved X minutes” unless the linked outcome field records actual reviewed minutes or the text clearly says “estimated.”

### Safe-use evidence and outcome fields

Safe use requires the action kind, source refs, estimate, reviewer disposition, actual minutes when known, and reporting group or staff persona when the workflow records them. If any of those are absent, describe the value as a planning estimate, not proof.

### Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | “Estimated 12 minutes saved by ranking three checkout exceptions for manager review.” | It names the manual work and keeps the result review-based. |
| Example | “Actual 8 minutes saved recorded after a data-quality duplicate was resolved.” | It ties labor to a reviewed outcome. |
| Non-example | “AI saved the resort hours today.” | No source, reviewer, workflow, or outcome field. |
| Non-example | “Reduce next week’s staff schedule by 30 minutes.” | Labor minutes are measurement evidence, not schedule authority. |

### Glossary cross-links

Use [outcome capture](../glossary-workflow-state-terms.md#outcome-capture), [workflow packet](../glossary-workflow-state-terms.md#workflow-packet), [source ref/provenance](../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence), and [review gate](../glossary-workflow-state-terms.md#review-gate) when introducing labor-minute claims in public or operator docs.
