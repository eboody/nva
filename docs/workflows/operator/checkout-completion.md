# Checkout Completion

Checkout completion helps front-desk agents and leads review open stays, handoff notes, checkout status, and follow-up blockers without treating the packet as authority that a guest has completed checkout. It prepares source-backed manager-review or source-reconciliation evidence and draft audit notes while billing, provider/PMS, customer-message, retention, and guest-impacting actions remain unavailable from serialized packet state.

Status: supported local app workflow and tests. Dedicated durable checkout outcome persistence is still planned because no separate checkout outcome projection is identified in `storage/src/operations.rs`.

Navigation: start with the [operator workflow index](README.md). Entity-first backlinks: [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [workflow packets](../../design/entity-atlas-workflow-packets-agents.md), [review gates and blocked actions](../../design/entity-atlas-review-safety-boundaries.md), and [revenue opportunity entities](../../design/entity-atlas-revenue-opportunity-entities.md).

## Problem solved and reported labor evidence

- Problem solved: open stays and incomplete handoffs require staff to rediscover whether source checkout status, belongings, care summary, departure notes, and payment/care/source exceptions agree before manager review; the packet never closes checkout or starts retention follow-up.
- First role associated with caller-reported handling-time evidence: front-desk agents doing checkout audits and internal handoff cleanup. The report does not establish measured labor.
- Secondary reviewers/operators: front-desk leads and managers who need a short, source-backed account of why evidence is routed to manager review or source reconciliation; the packet never reports checkout readiness.
- Pet-resort example: after a boarding stay, the source reservation reports checked out, the care summary says medication was given, belongings are returned, and departure notes are staff-reviewed. The workflow still routes the packet to manager review without suggesting checkout completion, customer messaging, or retention work. If the medication bag needs a second staff check, that unresolved evidence remains visible in the same fail-closed route.

## Source data and featured entities

The workflow needs a provider [record](../../glossary-source-data-terms.md#provider-record) or read-model fact for the reservation, traceable [source refs](../../glossary-architecture-terms.md#source-ref-domainsourcerecordref) and [provenance](../../glossary-architecture-terms.md#provenance-domainsourceprovenance), plus staff-submitted handoff evidence. Source checkout/PMS status is evidence from the system of record; an agent summary never overwrites it.

| Entity or source fact | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| Reservation id | Ties the packet, staff handoff, source status, and audit draft to one stay/reservation. | `app::checkout_completion::Request` carries `domain::entities::reservation::Id`; provider/source evidence identifies the record. | Source `app/src/checkout_completion.rs` (`Request::reservation_id`, `Packet::reservation_id`); tests `app/tests/checkout_completion_workflow_contracts.rs`. |
| Source checkout/PMS status | Distinguishes manager-review evidence from source-status reconciliation; it never permits a checkout-completion suggestion. | Provider/PMS evidence through `domain::source::reservation::Status`; app workflow only observes it. | Source `app/src/checkout_completion.rs` (`observed_source_status`, `completion_status_for`); source vocabulary `domain/src/source.rs`; Gingr endpoint `integrations/gingr/src/endpoint/reservations.rs`. |
| Staff handoff | Shows who completed the handoff, when, whether belongings were returned, the care summary, and whether departure notes were reviewed. | Staff-submitted handoff evidence in the app packet; managers/front-desk leads own unresolved handoff decisions. | Source `app/src/checkout_completion.rs` (`StaffHandoff`, `BelongingsStatus`, `CareSummary`, `DepartureNotesReview`); domain `domain/src/boarding/handoff.rs` (`DepartureTaskDraft`); tests `resolved_staff_handoff` and `open_staff_handoff` fixtures. |
| Unresolved checkout exceptions | Names belongings, care/departure-note, payment, and provider/source conflicts so the handoff becomes a short queue instead of another open-stay audit. | Domain/app exception vocabulary only; staff, billing/PMS, or manager review owns resolution. | Source `app/src/checkout_completion.rs` (`UnresolvedException`, `PaymentException`, `SourceException`, `staff_task_drafts_for`); domain `domain/src/payment/mod.rs`, `domain/src/reservation/mod.rs`, `domain/src/care.rs`, `domain/src/boarding/handoff.rs`; focused checkout exception test. |
| Completion status | Preserves reported staff-checkout, needs-handoff-review, or source-not-checked-out evidence. A reported staff label is never completion authority. | Deterministic review classification from source status plus serialized handoff evidence. | Source `app/src/checkout_completion.rs` (`CompletionStatus`, `Workflow::evaluate`, `completion_status_for`). |
| Reported disposition and minute evidence | Routes every serialized checkout packet through manager review and retains reported work-duration evidence. | Reviewable app packet evidence only; it cannot prove checkout completion or realized savings. | Source `app/src/checkout_completion.rs` (`ReportedDisposition`, `LaborImpact`, `LaborMinutes`); domain `domain/src/reservation/mod.rs` (`CheckoutCompletionDisposition`, `OutcomeRecord`, `OutcomeState`). |
| Review gates | Requires manager review for all serialized checkout evidence before any guest-impacting action. | `domain::policy::ReviewGate` and future authenticated authority. | Source `app/src/checkout_completion.rs` (`required_review_gates_for`); domain `domain/src/policy.rs`. |
| Audit-event drafts | Records what the app may draft for review without treating the draft as checkout completion or a live provider/customer action. | App workflow draft artifact; provider/PMS/customer systems remain separate authorities. | Source `app/src/checkout_completion.rs` (`AuditEventDraft`, `audit_event_drafts_for`); tests assert source evidence and handoff reporting while checkout suggestion remains blocked and manager review remains required. |

Related entities to mention without making them the page center:

- Stay/reservation: checkout completion is about the departure/closeout state of a reservation, not a new booking or capacity mutation.
- Charges, invoices, payments, refunds, discounts, and waivers: payment exceptions can block checkout confidence, but money movement stays outside agent authority.
- Incidents, medications, services, and care notes: care-summary facts can explain why a handoff needs review; medical/incident/service decisions remain human/provider-controlled.
- Owner communication and retention follow-up: serialized checkout evidence cannot prepare or unlock a draft; a future opaque authenticated contact authority would still require customer-message approval.
- Data-quality/source exceptions: source-not-checked-out or conflicting evidence should be routed to review rather than hidden or overwritten.

## Featured contracts

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | `app::checkout_completion::{Request, Packet, CompletionStatus, StaffHandoff, UnresolvedException, StaffTaskDraft, ReviewedDisposition, LaborImpact, SafeAgentAction, BlockedAction, AuditEventDraft, Workflow}` | Build a source-grounded [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet), classify manager-review or reconciliation evidence, list unresolved exception work, draft internal handoff tasks, retain reported labor evidence, and prepare reviewable audit drafts. | Checkout readiness/completion, retention eligibility/drafts, live PMS/provider mutation, payment/refund/discount movement, customer sends, or final status execution. |
| `domain` | `domain::source::{Provenance, reservation::Status}`, `domain::entities::reservation::Status`, `domain::policy::ReviewGate`, `domain::workflow`, `domain::boarding::handoff::DepartureTaskDraft`, `domain::payment::CheckoutException`, `domain::reservation::{CheckoutSourceException, CheckoutCompletionDisposition}`, `domain::care::CheckoutException` | Vocabulary for observed source evidence, checkout handoff/payment/care/source exceptions, reported disposition, review gates, workflow state, and blocked-action semantics. | A normalized checkout-status suggestion, provider-specific payload authority, money movement, source mutation, or staff approval by itself. |
| `storage` | No dedicated checkout outcome projection identified yet; `storage/src/operations.rs` currently includes manager daily brief/data-quality outcome records and action kinds such as checkout exceptions. | Future durable checkout outcome/labor evidence should live here or in an equivalent storage projection once implemented. | Current docs must not claim durable checkout-specific persistence or verified production labor savings. |
| `integrations/gingr` | `integrations/gingr/src/endpoint/reservations.rs` plus reservation mapping/read-model evidence | Provider reservation/checkout source evidence and mapping boundary. | Domain truth, approved side effects, billing action, or checkout completion authority. |

## Authority and source of truth

- Provider/PMS reservation state is authoritative for observed checkout status and must carry source/provenance evidence.
- Staff handoff evidence is authoritative for what staff recorded about belongings, care summary, and departure-note review.
- The app workflow preserves `ReportedStaffCheckout`, `NeedsStaffHandoffReview`, or `SourceNotCheckedOut` as review evidence. None is checkout-completion authority, and every serialized packet stays under manager review.
- `domain::policy::ReviewGate` is authoritative vocabulary for human approval gates; the assigned human role still owns the approval.
- Payment, refund, discount, invoice, and waiver facts remain in approved payment/ledger/provider records, not in the checkout-completion agent.
- Serialized checkout evidence cannot create customer communication or retention drafts; any future separately authorized draft would remain review-only until an approved sender acts.

## Agent work, approvals, and blocked actions

- Agent may: summarize checkout evidence, create an internal handoff task, name unresolved belongings/care/payment/source exceptions, report estimated packet-review effort, and prepare [draft](../../glossary-workflow-state-terms.md#draft) audit-event artifacts.
- Agent may rank/recommend only manager-review or source-reconciliation work. Serialized checkout evidence cannot enable a retention draft or suppress an exception queue item.
- Human must approve: unresolved handoffs, departure-note concerns, manager-review routes, customer-message drafts, provider/PMS changes, final checkout-status execution, and all payment/refund/discount/waiver or billing decisions.
- Blocked by default: suggest checked-out status, send customer messages, mutate provider/PMS records, move refunds/discounts/payments, hide source disagreement, release capacity, or treat a draft audit event as a completed external action.

## Nonclaimable outcome and labor evidence

- Reported labor evidence: retain estimates about packet review and reported time spent without deriving minutes avoided or realized value.
- Reported outcome candidates: caller-reported checkout review effort, count of exceptions routed for review, count of wrong-source findings, count of incomplete handoffs routed, and count of payment/care/source exception tasks drafted. These remain evidence and do not prove completion, realized savings, or retention authority.
- Current evidence status: supported local app workflow/test contract for review packets, unresolved exception task drafts, caller-reported disposition/minute estimate, and blocked actions.
- Gap/future source need: durable checkout-specific [outcome capture](../../glossary-workflow-state-terms.md#outcome-capture) is planned/future until a storage projection or equivalent outcome record is added. The page should not claim production NVA savings, live billing completion, provider checkout writes, or customer-message sends.

## Contract crosswalk links

Use the [workflow packet row](../../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map) for the bidirectional path from this workflow page back to the entities it consumes and produces. Use [surface inventory](../../entity-atlas/contract-crosswalk/surface-inventory.md) for source/Rustdoc/test proof, [source/provider flows](../../entity-atlas/contract-crosswalk/source-provider-flows.md) for source-entry and normalization evidence, [storage/persistence](../../entity-atlas/contract-crosswalk/storage-persistence.md) for `dedicated checkout outcome storage gap`, and [runtime exposure](../../entity-atlas/contract-crosswalk/runtime-exposure.md) for API/worker/CLI/web/script exposure. Rustdoc/module path: `app::checkout_completion::Packet`; operator-facing entity family: `Checkout completion packet`.

## Evidence citations

- Source: `app/src/checkout_completion.rs` (`app::checkout_completion::{Request, Packet, CompletionStatus, StaffHandoff, BelongingsStatus, DepartureNotesReview, CareSummary, UnresolvedException, StaffTaskDraft, ReviewedDisposition, LaborImpact, SafeAgentAction, BlockedAction, AuditEventDraft, Workflow}`) supports packet inputs, deterministic checkout classification, exception/task draft assembly, caller-reported disposition/minute estimate, safe agent actions, blocked actions, review gates, and audit-event drafts.
- Source: `domain/src/boarding/handoff.rs`, `domain/src/care.rs`, `domain/src/payment/mod.rs`, and `domain/src/reservation/mod.rs` provide the handoff task, care/payment/source exception, and checkout disposition vocabulary reused by the app packet.
- Source: `domain/src/source.rs` (`domain::source::{Provenance, reservation::Status}`), `domain/src/policy.rs` (`domain::policy::ReviewGate`), and `domain/src/workflow.rs` support source evidence, provenance, review-gate, and workflow-state vocabulary.
- Source: `integrations/gingr/src/endpoint/reservations.rs` is the provider reservation endpoint surface to cite for checkout/PMS source evidence and provider-boundary language.
- Storage caveat: `storage/src/operations.rs` has no dedicated checkout completion outcome projection identified; durable checkout-specific outcome persistence remains planned/future.
- Tests: `app/tests/checkout_completion_workflow_contracts.rs` verifies every serialized packet remains review-required and proves no manager review, `suggested_reservation_status` remains absent, `SuggestCheckedOutStatus` stays blocked, retention drafting stays unavailable, source disagreement remains visible, and checkout exceptions assemble reviewable belongings/care/payment/source work while payment/provider mutation remains blocked.
- Supporting docs: `docs/design/entity-driven-workflow-page-template.md`, `docs/design/operator-workflow-page-inventory.md`, `docs/design/workflow-page-source-rustdoc-map.md#checkout-completion`, and `docs/design/labor-cost-reduction-crosswalk.md` checkout bottleneck row.
- Rustdoc: `target/doc/app/checkout_completion/index.html`, `target/doc/domain/source/index.html`, `target/doc/domain/policy/index.html`, `target/doc/domain/workflow/index.html`, and provider/storage module docs after running `cargo doc --no-deps --workspace`.
- Evidence status: supported local app workflow/tests; durable checkout outcome persistence, production labor measurement, live PMS/provider writes, live billing actions, and customer-message sends are not claimed.
