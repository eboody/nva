---
title: "Revenue opportunity entity families"
slug: "revenue-opportunity-entities"
family: "grooming-training-retail-retention-commerce"
status: "draft"
audience: ["front-desk", "grooming-manager", "trainer", "general-manager", "regional-ops", "docs-writer"]
plain_english_definition: "The grooming, training, retail, retention, commerce, portal, reputation, and lead concepts that turn source-backed resort history into reviewable revenue and labor-saving opportunities."
primary_labor_problem: "Reduces manual rebooking search, package/session reconciliation, retail upsell triage, stock/reorder scanning, lead follow-up triage, and review-response routing."
source_of_record: "Domain service-line contracts, app retention packets, storage service-line projections, and Gingr provider evidence after explicit mapping."
authoritative_human_role: "front desk lead, grooming manager, trainer, general manager, or regional operations reviewer depending on the action"
workflow_links: ["grooming-rebooking-retention", "manager-daily-brief", "data-quality-hygiene", "booking-triage"]
source_paths:
  - "domain/src/grooming/mod.rs"
  - "domain/src/training/mod.rs"
  - "domain/src/retail/mod.rs"
  - "domain/src/portal.rs"
  - "domain/src/reputation.rs"
  - "domain/src/lead.rs"
  - "app/src/crm_retention.rs"
  - "storage/src/service_line/grooming.rs"
  - "storage/src/service_line/training.rs"
  - "storage/src/service_line/retail.rs"
  - "integrations/gingr/src/endpoint/commerce_retail.rs"
rustdoc_contracts:
  - "domain::grooming::{Contract, Service, DurationEstimate, rebooking, reminder, history}"
  - "domain::training::{Contract, Program, ProgressEvidence, SessionBalance, availability, package, follow_up}"
  - "domain::retail::{Contract, Product, inventory, pos, recommendation, reorder, vendor}"
  - "app::crm_retention::{Packet, RetentionOpportunity, OpportunityEvidence, ContactPermission, FollowUpEligibility, OutcomeRecord}"
  - "gingr::endpoint::commerce_retail::{get, list, Transaction, ResponseSensitivity}"
allowed_action_summary: "read source-backed evidence; validate eligibility, cadence, inventory, package, and consent gates; draft staff/customer copy for review; rank internal opportunities; record review dispositions and outcome evidence"
blocked_action_summary: "no autonomous customer sends, provider/PMS writes, appointment moves, POS/payment/refund/discount/package balance movement, vendor purchase orders, review responses, consent changes, or safety/legal approvals"
outcome_fields: ["source refs", "review gate", "draft status", "staff disposition", "follow-up outcome", "actual minutes saved", "conversion or suppression reason", "inventory/reorder decision"]
---

# Revenue opportunity entity families

This page helps front-desk leads, grooming managers, trainers, general managers, and regional operators avoid repeated manual opportunity hunting: grooming rebooking searches, training package/session reconciliation, retail upsell checks, low-stock scans, lead follow-up triage, and review-response routing. It is an atlas-family draft for non-coders; the linked Rust source, app workflow packets, storage projections, and Gingr adapter contracts remain the behavior authority.

Use this page with the [entity atlas template](entity-atlas-page-template.md), [entity atlas inventory](entity-atlas-inventory.md), and [documentation style guide](../quality/nva-documentation-style-guide.md). It covers family entries rather than one page per scalar, because the requested concepts are tightly connected: a recommendation only makes sense when its source evidence, review gate, draft state, and outcome record travel together.

## 1. Plain-English pet-resort definitions

| Entity or family entry | Plain-English definition | Labor or revenue opportunity |
| --- | --- | --- |
| Grooming contract | A location's grooming rules for services, duration estimates, no-show handling, rebooking cadence, reminders, and service-history review. | Reduces groomer/front-desk time spent checking prior notes, estimating appointment length, and finding overdue rebooks. |
| Grooming rebooking cadence | The repeat-service timing that turns completed groom history into due-later, due-now, overdue, or needs-review follow-up. | Finds safe rebooking candidates without staff manually scanning each pet's past grooms. |
| Training contract | A location's training program, enrollment, trainer capacity, curriculum/progress, outcome, package/session, and follow-up rules. | Reduces manual trainer-capacity, package-balance, progress-evidence, and graduation/re-enrollment checks. |
| Training package/session opportunity | A reviewable chance to reserve, consume, reconcile, or follow up on reusable training sessions. | Prevents lost training revenue and avoids promising sessions when balance, evidence, or trainer availability is unclear. |
| Retail contract | A location's retail product, inventory, POS, recommendation, reorder, and vendor policy bundle. | Helps staff see safe add-on and reorder work without hand-checking catalog, stock, preference, and care-sensitivity facts. |
| Product | A sellable retail item with SKU/name/category/usage and per-location offering status. | Keeps product upsell candidates grounded in saleable inventory rather than raw provider text. |
| Inventory / stock position | On-hand, reserved, available, threshold, and availability state for a product at a location. | Prevents staff from recommending or selling unavailable products and creates threshold-backed reorder work. |
| Vendor / catalog relationship | The partner or catalog relationship tied to a product. | Routes stock exceptions to staff task, manager review, or vendor-managed notice instead of a silent purchase order. |
| Reorder decision | A threshold-backed decision that says no action, staff task, manager review, or vendor notice. | Replaces manual inventory report scanning with reviewable reorder tasks. |
| Recommendation | A staff-facing candidate for a product, grooming, training, boarding, or daycare follow-up. | Turns source-backed service history and product fit into drafts or tasks; it does not authorize live outreach. |
| Package / opportunity | A source-grounded revenue or retention candidate such as grooming rebook, training consult, next boarding stay, recurring daycare, or retail upsell. | Gives staff a ranked, evidence-backed reason to review an offer rather than inventing one from model memory. |
| POS / commerce | Checkout, retail transaction, package/subscription, invoice, and payment-sensitive provider evidence. | Supports safe reference and reconciliation while keeping money movement and POS mutations human/system-of-record controlled. |
| Portal account/id | External customer-portal identity used for joins and source lineage. | Helps connect customer records without treating a portal id as internal domain truth. |
| Reputation signal | A source-derived review with platform, sentiment, themes, and escalation state. | Helps managers triage reviews while safety/legal/public responses stay gated. |
| Lead triage | Cross-service sales/intake state with source, intent, conversion stage, requested service, and next action. | Reduces stale lead work and routes booking/revenue opportunities without overpromising availability or policy exceptions. |

## 2. Purpose: labor-cost and safety problems

These entities exist to make revenue opportunities reviewable instead of magical. A grooming rebook, training consult, retail add-on, reorder task, review response, or lead follow-up should show:

- which source fact created the opportunity;
- which staff role should review it;
- which action is only a [draft](../glossary-workflow-state-terms.md#draft) or internal task;
- which [review gate](../glossary-workflow-state-terms.md#review-gate) blocks live work;
- which outcome proves minutes saved or revenue impact later.

The safe outcome is a ranked queue, draft, task, source-backed summary, or outcome record. The unsafe outcome would be an agent that books a groom, changes a package balance, sends an upsell, places a vendor order, writes to Gingr, or posts a public review response without explicit human/system approval.

## 3. Workflows where these entities appear

| Workflow | How the entities appear | Safe workflow result |
| --- | --- | --- |
| [Grooming Rebooking / Retention](../workflows/operator/grooming-rebooking-retention.md) | Completed checkout, grooming history/cadence, source-grounded opportunity, consent/contact permission, draft channel, suppression reason. | Staff review packet, follow-up draft, suppression/defer reason, outcome capture. |
| [CRM retention agent](../workflows/crm-retention-agent.md) | `RetentionOpportunity`, `OpportunityEvidence`, `ContactPermission`, `FollowUpEligibility`, safe/blocked actions, follow-up outcome. | Evidence summary, internal staff review task, customer follow-up draft for review, outcome evidence. |
| [Manager Daily Brief](../workflows/operator/manager-daily-brief.md) | Revenue opportunities, retail/reorder work, lead/reputation exceptions, labor-minute estimate, source facts. | Prioritized manager action with source refs and actual outcome feedback. |
| [Data Quality Hygiene](../workflows/operator/data-quality-hygiene.md) | Unknown provider product fields, ambiguous service catalog names, stale portal/contact facts, conflicting inventory/package evidence. | Cleanup candidate or draft submission, not hidden source-data changes. |
| [Booking Triage](../workflows/operator/booking-triage.md) | Training consult readiness, service-line add-ons, care/safety review gates, missing customer/pet facts. | Readiness packet and review gate before confirmation or customer promise. |
| Gingr source normalization | Commerce/retail endpoints, service catalog gaps, provider ids, raw payload refs, mapping candidates. | Provider evidence and mapping candidates, not domain policy or live provider mutation. |

## 4. Relationships and adjacency

```text
Gingr/provider evidence or internal source record
  -> source ref / provenance / raw payload reference
  -> mapping candidate or data-quality issue when fields are missing
  -> domain service-line contract
  -> app packet or manager/review queue
  -> draft, internal task, or blocked action
  -> staff disposition and outcome evidence
```

Important adjacency rules:

- Grooming history and cadence are service-history authority for rebooking timing, but they do not authorize appointment creation, groomer assignment, deposits, or customer sends.
- Training package/session balances are reconciliation evidence, but they do not authorize payment movement, package adjustment, trainer assignment, or graduation/outcome claims.
- Retail products and inventory support recommendation and reorder decisions, but a raw provider item, SKU, or stock count is not enough to send an upsell or place an order.
- Portal ids help join records; customer/contact truth still needs validated customer and permission evidence.
- Reputation and lead signals are cross-service triage facts; they should route drafts/tasks/escalations, not bypass safety, legal, capacity, or booking review.

## 5. Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Grooming domain source | [`domain/src/grooming/mod.rs`](../../domain/src/grooming/mod.rs) and [`domain/src/grooming/README.md`](../../domain/src/grooming/README.md) | Services, history, duration estimates, no-show rules, rebooking cadence, reminders, and customer-message gates. |
| Training domain source | [`domain/src/training/mod.rs`](../../domain/src/training/mod.rs) and [`domain/src/training/README.md`](../../domain/src/training/README.md) | Programs, enrollment readiness, trainer availability, progress/outcome evidence, package ledger, follow-up states. |
| Retail domain source | [`domain/src/retail/mod.rs`](../../domain/src/retail/mod.rs), [`product.rs`](../../domain/src/retail/product.rs), [`inventory.rs`](../../domain/src/retail/inventory.rs), [`pos.rs`](../../domain/src/retail/pos.rs), [`recommendation.rs`](../../domain/src/retail/recommendation.rs), [`reorder.rs`](../../domain/src/retail/reorder.rs), [`vendor.rs`](../../domain/src/retail/vendor.rs) | Product identity, inventory math, POS decisions, recommendation gates, reorder thresholds, vendor relationships. |
| CRM retention app surface | [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs) | Opportunity evidence, contact permission, eligibility, draft channel, safe agent actions, blocked actions, outcome record. |
| Portal domain source | [`domain/src/portal.rs`](../../domain/src/portal.rs) | External portal ids as source facts for joins, not internal authority. |
| Reputation domain source | [`domain/src/reputation.rs`](../../domain/src/reputation.rs) | Review signal sentiment/theme/escalation state and response gates. |
| Lead domain source | [`domain/src/lead.rs`](../../domain/src/lead.rs) | Sales lead source, intent, conversion stage, requested service, and next safe action. |
| Storage service-line projections | [`storage/src/service_line/grooming.rs`](../../storage/src/service_line/grooming.rs), [`training.rs`](../../storage/src/service_line/training.rs), [`retail.rs`](../../storage/src/service_line/retail.rs) | Stable codes and domain/storage conversions for cadence, training program duration, retail partner/category. |
| Storage operations projection | [`storage/src/operations.rs`](../../storage/src/operations.rs) | Service offering records, core service contract records, source refs, and shape checks across service lines. |
| Gingr commerce/reference source | [`integrations/gingr/src/endpoint/commerce_retail.rs`](../../integrations/gingr/src/endpoint/commerce_retail.rs), [`dto/retail.rs`](../../integrations/gingr/src/dto/retail.rs), [`mapping/retail.rs`](../../integrations/gingr/src/mapping/retail.rs), [`endpoint/reference_data.rs`](../../integrations/gingr/src/endpoint/reference_data.rs) | Provider request shapes, retail item DTOs, product candidates, reference data, and sensitivity boundaries. |
| Gingr DTO gap markers | [`integrations/gingr/src/dto/grooming.rs`](../../integrations/gingr/src/dto/grooming.rs), [`dto/training.rs`](../../integrations/gingr/src/dto/training.rs), [`endpoint/catalog.rs`](../../integrations/gingr/src/endpoint/catalog.rs) | Grooming/training service DTOs are not documented enough here; do not invent provider service DTO truth. |

Rustdoc/module paths to use when rendered Rustdoc exists: `domain::grooming`, `domain::training`, `domain::retail`, `domain::portal::CustomerId`, `domain::reputation::Signal`, `domain::lead::Triage`, `app::crm_retention::Packet`, `storage::service_line::{grooming, training, retail}`, and `gingr::endpoint::commerce_retail`.

## 6. Authoritative source system or human role

| Fact or decision | Source of record | Human role when source is incomplete or sensitive |
| --- | --- | --- |
| Grooming service history, duration evidence, cadence | Grooming domain contract plus approved service history and source refs. | Groomer or grooming manager. |
| Training program, trainer availability, progress evidence, package ledger | Training domain contract plus package/payment/source evidence. | Trainer, front desk lead, or general manager for exceptions. |
| Retail product, saleability, inventory, POS/reorder/recommendation policy | Retail domain contract and storage/provider evidence after mapping. | Front desk lead or general manager; vendor contact for vendor-managed notices. |
| Commerce/transaction/package provider evidence | Gingr commerce endpoint evidence and raw/source refs. | General manager or approved system-of-record operator for money/package changes. |
| Portal customer identity | Validated portal id joined to domain customer/pet records. | Front desk/customer account owner when duplicates or stale links appear. |
| Reputation signal and public response readiness | Reputation source signal and escalation state. | General manager; safety/legal reviewer for injury/legal-sensitive themes. |
| Lead conversion next action | Lead triage contract plus availability/policy evidence. | Sales/front desk lead or manager before capacity or policy promise. |
| Follow-up/send approval | App packet, contact permission, consent source refs, and approval record. | Approved sender, front desk lead, or manager. |
| Labor/revenue impact | Outcome record with staff disposition, conversion/suppression, and actual minutes saved. | Reviewer who performed or approved the work. |

## 7. Allowed actions

Automation or app workflows may safely:

- read provider/domain/storage evidence and source refs;
- map provider retail items into explicit product candidates;
- validate grooming cadence, training package/session balance, retail inventory, POS, recommendation, reorder, consent, and eligibility gates;
- draft customer follow-up, review response, product recommendation, training progress/outcome, or lead reply text for review;
- create internal staff review tasks, manager review tasks, or vendor-managed notices when the source contract allows that draft/task state;
- rank opportunities for a manager daily brief or staff queue;
- record follow-up outcomes, suppression reasons, conversion, actual minutes saved, and reviewer disposition.

Keep verbs review-safe: draft, rank, summarize, validate, recommend internally, route, and record. Do not say these entities send, book, charge, refund, discount, order, modify, approve, or publish unless a linked system-of-record/human approval contract explicitly allows it.

## 8. Blocked actions and review gates

Default blocked actions for this family:

- customer/member sends for grooming, training, retail, review, or lead follow-up;
- Gingr/PMS/provider writes, provider record hiding, package/subscription mutation, POS/transaction mutation, or source-data deletion;
- appointment, groomer, trainer, room, schedule, capacity, or waitlist changes;
- payment, refund, discount, deposit, credit, package balance, or session-balance movement;
- vendor purchase orders or irreversible reorder placement;
- medical, diet, supplement, injury, temperament, legal, review-response, or policy-exception approval;
- consent/DNC changes or sensitive-data release;
- public reputation response publishing.

Required review gates should be named close to the entity: `CustomerMessageApproval` for sends, manager/groomer/trainer review for service exceptions, payment/package reconciliation for money/session mismatches, safety/legal review for reputation themes, and data-quality review for source gaps or unknown provider fields.

## 9. Safe-use evidence and outcome fields

A recommendation is safe to place in a staff queue only when evidence travels with it:

| Evidence field | Why it matters |
| --- | --- |
| Source refs / provenance / raw payload ref | Lets staff trace the provider/source record behind the opportunity. |
| Entity kind and source-grounded reason | Explains why this is a grooming rebook, training consult, retail upsell, reorder, lead, or reputation task. |
| Review gate and blocked action | Proves the suggestion stayed draft/internal until a human or approved system acted. |
| Draft channel / customer-copy status | Separates staff-visible draft text from a live customer send. |
| Inventory or package/session position | Prevents recommending unavailable products or overpromising training capacity/balance. |
| Contact permission / consent evidence | Prevents retention/lead outreach when the channel or consent is missing/opted out. |
| Staff disposition | Records approved, rejected, deferred, suppressed, wrong source, or needs correction. |
| Outcome | Captures booked/rebooked, converted, no response, not interested, vendor notice, reorder task, or suppressed-by-staff result. |
| Actual minutes saved or wasted | Separates measurable labor savings from product intent. |

Do not claim revenue or labor savings from a model suggestion alone. Claim only a reviewable opportunity, or point to a durable outcome record after staff disposition exists.

## 10. Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | Grooming rebooking cadence | Staff use completed service history and cadence to decide whether a follow-up draft is due; cadence also carries the review boundary. |
| Example | Training package/session opportunity | Trainer availability, package balance, and progress evidence together explain whether staff can review a re-enrollment or consult draft. |
| Example | Retail reorder decision | Low stock plus threshold policy can produce an internal task or vendor notice while still blocking live purchase orders. |
| Example | Lead triage | Source, intent, stage, requested service, and next action reduce stale-lead work without promising availability. |
| Non-example | Raw Gingr `OwnerId`, `ItemId`, `PackageId`, or `SubscriptionId` by itself | Provider ids are source references; they need mapping/provenance and domain context before becoming business evidence. |
| Non-example | A retail DTO `quantity_on_hand` alone | It is provider evidence, not a sellability or reorder decision until domain inventory policy validates it. |
| Non-example | Customer portal id alone | It helps joins and lineage, but it is not customer-contact permission or source-of-record approval. |
| Non-example | Model-generated upsell copy | Copy is only a draft until source facts, review gates, consent, and staff approval are attached. |

## 11. Entity family entry checklist for future split pages

When splitting this family page into individual atlas pages, keep these groupings:

1. Grooming Contract + Grooming Rebooking Cadence + Grooming Reminder/History.
2. Training Contract + Training Package/Session Opportunity + Progress/Outcome/Follow-up.
3. Retail Contract + Product + Inventory + POS + Recommendation + Reorder + Vendor.
4. CRM Retention Opportunity + Contact Permission + Follow-up Outcome.
5. Gingr Commerce/Reference Provider Boundary + Retail DTO/Mapping Candidate + Provider Surface Gap Marker.
6. Portal Account, Reputation Signal, and Lead Triage as lighter cross-service relationship pages unless a workflow makes them first-class.

Each split page should preserve the same boundary: suggestions stay drafts or reviewable actions until a human/system-of-record approval exists.
