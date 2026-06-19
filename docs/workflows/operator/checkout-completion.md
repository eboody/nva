# Checkout Completion

Checkout completion saves front-desk agents and leads from manually auditing every open stay, handoff note, checkout status, and follow-up blocker before they can trust that a guest is ready to leave the workflow. It prepares a source-backed review packet that can suggest staff verification readiness, draft internal handoff/audit notes, and keep billing, provider/PMS, customer-message, and guest-impacting actions under human approval.

Status: supported local app workflow and tests. Dedicated durable checkout outcome persistence is still planned because no separate checkout outcome projection is identified in `storage/src/operations.rs`.

Navigation: start with the [operator workflow index](README.md). Entity-first backlinks: [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [workflow packets](../../design/entity-atlas-workflow-packets-agents.md), [review gates and blocked actions](../../design/entity-atlas-review-safety-boundaries.md), and [revenue opportunity entities](../../design/entity-atlas-revenue-opportunity-entities.md).

## Problem solved and time saved

- Problem solved: open stays and incomplete handoffs require staff to rediscover whether source checkout status, belongings, care summary, departure notes, and payment/care/source exceptions agree before the resort can close the loop or start retention follow-up.
- First role whose time is saved: front-desk agents doing checkout audits and internal handoff cleanup.
- Secondary reviewers/operators: front-desk leads and managers who need a short, source-backed reason why checkout is ready, blocked, or routed to review.
- Pet-resort example: after a boarding stay, the source reservation is checked out, the care summary says medication was given, belongings are returned, and departure notes are staff-reviewed. The workflow can prepare a checkout-completion suggestion and a customer-message approval request. If the medication bag still needs a second staff check, it routes the packet to manager/staff handoff review instead of implying checkout is complete.

## Source data and featured entities

The workflow needs a provider [record](../../glossary-source-data-terms.md#provider-record) or read-model fact for the reservation, traceable [source refs](../../glossary-architecture-terms.md#source-ref-domainsourcerecordref) and [provenance](../../glossary-architecture-terms.md#provenance-domainsourceprovenance), plus staff-submitted handoff evidence. Source checkout/PMS status is evidence from the system of record; an agent summary never overwrites it.

| Entity or source fact | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| Reservation id | Ties the packet, staff handoff, source status, and audit draft to one stay/reservation. | `app::checkout_completion::Request` carries `domain::entities::reservation::Id`; provider/source evidence identifies the record. | Source `app/src/checkout_completion.rs` (`Request::reservation_id`, `Packet::reservation_id`); tests `app/tests/checkout_completion_workflow_contracts.rs`. |
| Source checkout/PMS status | Decides whether the packet can even suggest checkout completion or must route to source-status reconciliation. | Provider/PMS evidence through `domain::source::reservation::Status`; app workflow only observes it. | Source `app/src/checkout_completion.rs` (`observed_source_status`, `completion_status_for`); source vocabulary `domain/src/source.rs`; Gingr endpoint `integrations/gingr/src/endpoint/reservations.rs`. |
| Staff handoff | Shows who completed the handoff, when, whether belongings were returned, the care summary, and whether departure notes were reviewed. | Staff-submitted handoff evidence in the app packet; managers/front-desk leads own unresolved handoff decisions. | Source `app/src/checkout_completion.rs` (`StaffHandoff`, `BelongingsStatus`, `CareSummary`, `DepartureNotesReview`); tests `resolved_staff_handoff` and `open_staff_handoff` fixtures. |
| Completion status | Classifies the packet as staff-verified checkout, needs staff handoff review, or source not checked out. | Deterministic app workflow classification from source status plus staff handoff evidence. | Source `app/src/checkout_completion.rs` (`CompletionStatus`, `Workflow::evaluate`, `completion_status_for`); tests cover all three routes. |
| Review gates | Keeps customer-message approval and manager approval explicit before guest-impacting or unresolved actions. | `domain::policy::ReviewGate` and human role approval. | Source `app/src/checkout_completion.rs` (`required_review_gates_for`); domain `domain/src/policy.rs`; tests assert `CustomerMessageApproval` and `ManagerApproval`. |
| Audit-event drafts | Records what the app may draft for review without treating the draft as a live provider or customer action. | App workflow draft artifact; provider/PMS/customer systems remain separate authorities. | Source `app/src/checkout_completion.rs` (`AuditEventDraft`, `audit_event_drafts_for`); tests assert source checkout observed, handoff recorded, checkout completion suggested, and review requested behavior. |

Related entities to mention without making them the page center:

- Stay/reservation: checkout completion is about the departure/closeout state of a reservation, not a new booking or capacity mutation.
- Charges, invoices, payments, refunds, discounts, and waivers: payment exceptions can block checkout confidence, but money movement stays outside agent authority.
- Incidents, medications, services, and care notes: care-summary facts can explain why a handoff needs review; medical/incident/service decisions remain human/provider-controlled.
- Owner communication and retention follow-up: a draft may be prepared only after safe checkout evidence exists, and customer-message approval remains required.
- Data-quality/source exceptions: source-not-checked-out or conflicting evidence should be routed to review rather than hidden or overwritten.

## Featured contracts

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | `app::checkout_completion::{Request, Packet, CompletionStatus, StaffHandoff, SafeAgentAction, BlockedAction, AuditEventDraft, Workflow}` | Build a source-grounded [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet), classify checkout readiness, list safe agent actions, list blocked actions, and prepare reviewable audit drafts. | Live PMS/provider checkout mutation, payment/refund/discount movement, customer sends, or final status execution. |
| `domain` | `domain::source::{Provenance, reservation::Status}`, `domain::entities::reservation::Status`, `domain::policy::ReviewGate`, `domain::workflow` | Vocabulary for source evidence, normalized reservation status suggestion, review gates, workflow state, and blocked-action semantics. | Provider-specific payload authority or staff approval by itself. |
| `storage` | No dedicated checkout outcome projection identified yet; `storage/src/operations.rs` currently includes manager daily brief/data-quality outcome records and action kinds such as checkout exceptions. | Future durable checkout outcome/labor evidence should live here or in an equivalent storage projection once implemented. | Current docs must not claim durable checkout-specific persistence or verified production labor savings. |
| `integrations/gingr` | `integrations/gingr/src/endpoint/reservations.rs` plus reservation mapping/read-model evidence | Provider reservation/checkout source evidence and mapping boundary. | Domain truth, approved side effects, billing action, or checkout completion authority. |

## Authority and source of truth

- Provider/PMS reservation state is authoritative for observed checkout status and must carry source/provenance evidence.
- Staff handoff evidence is authoritative for what staff recorded about belongings, care summary, and departure-note review.
- The app workflow is authoritative for deterministic classification into `StaffVerifiedCheckout`, `NeedsStaffHandoffReview`, or `SourceNotCheckedOut` based on the packet inputs.
- `domain::policy::ReviewGate` is authoritative vocabulary for human approval gates; the assigned human role still owns the approval.
- Payment, refund, discount, invoice, and waiver facts remain in approved payment/ledger/provider records, not in the checkout-completion agent.
- Customer communication remains draft-only until an approved staff/customer-message path sends it.

## Agent work, approvals, and blocked actions

- Agent may: summarize checkout evidence, create an internal handoff task, draft retention follow-up for review when checkout evidence is safe, and prepare [draft](../../glossary-workflow-state-terms.md#draft) audit-event artifacts.
- Agent may rank/recommend: whether the packet appears ready for staff verification, needs staff handoff review, or should be treated as source not checked out.
- Human must approve: unresolved handoffs, departure-note concerns, manager-review routes, customer-message drafts, provider/PMS changes, final checkout-status execution, and all payment/refund/discount/waiver or billing decisions.
- Blocked by default: suggest checked-out status when source/staff evidence is incomplete, send customer messages, mutate provider/PMS records, move refunds/discounts/payments, hide source disagreement, release capacity, or treat a draft audit event as a completed external action.

## Outcome and labor value

- Estimated labor value: fewer minutes spent rediscovering checkout state across PMS status, care notes, belongings/handoff records, payment exceptions, and manager review threads.
- Measured outcome candidates: checkout audit minutes avoided, count of exceptions resolved or reviewed, count of wrong-source findings, count of incomplete handoffs routed, and count of safe retention follow-up drafts prepared for review.
- Current evidence status: supported local app workflow/test contract for review packets and blocked actions.
- Gap/future source need: durable checkout-specific [outcome capture](../../glossary-workflow-state-terms.md#outcome-capture) is planned/future until a storage projection or equivalent outcome record is added. The page should not claim production NVA savings, live billing completion, provider checkout writes, or customer-message sends.

## Contract crosswalk links

Use the [workflow packet row](../../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map) for the bidirectional path from this workflow page back to the entities it consumes and produces. Use [surface inventory](../../entity-atlas/contract-crosswalk/surface-inventory.md) for source/Rustdoc/test proof, [source/provider flows](../../entity-atlas/contract-crosswalk/source-provider-flows.md) for source-entry and normalization evidence, [storage/persistence](../../entity-atlas/contract-crosswalk/storage-persistence.md) for `dedicated checkout outcome storage gap`, and [runtime exposure](../../entity-atlas/contract-crosswalk/runtime-exposure.md) for API/worker/CLI/web/script exposure. Rustdoc/module path: `app::checkout_completion::Packet`; operator-facing entity family: `Checkout completion packet`.

## Evidence citations

- Source: `app/src/checkout_completion.rs` (`app::checkout_completion::{Request, Packet, CompletionStatus, StaffHandoff, BelongingsStatus, DepartureNotesReview, CareSummary, SafeAgentAction, BlockedAction, AuditEventDraft, Workflow}`) supports packet inputs, deterministic checkout classification, safe agent actions, blocked actions, review gates, and audit-event drafts.
- Source: `domain/src/source.rs` (`domain::source::{Provenance, reservation::Status}`), `domain/src/policy.rs` (`domain::policy::ReviewGate`), and `domain/src/workflow.rs` support source evidence, provenance, review-gate, and workflow-state vocabulary.
- Source: `integrations/gingr/src/endpoint/reservations.rs` is the provider reservation endpoint surface to cite for checkout/PMS source evidence and provider-boundary language.
- Storage caveat: `storage/src/operations.rs` has no dedicated checkout completion outcome projection identified; durable checkout-specific outcome persistence remains planned/future.
- Tests: `app/tests/checkout_completion_workflow_contracts.rs` verifies resolved handoff plus source checkout can suggest `CheckedOut` with customer-message approval; open handoff routes to manager/staff review without status suggestion; source-not-checked-out suppresses false checkout-observed and checkout-suggested audit drafts.
- Supporting docs: `docs/design/entity-driven-workflow-page-template.md`, `docs/design/operator-workflow-page-inventory.md`, `docs/design/workflow-page-source-rustdoc-map.md#checkout-completion`, and `docs/design/labor-cost-reduction-crosswalk.md` checkout bottleneck row.
- Rustdoc: `target/doc/app/checkout_completion/index.html`, `target/doc/domain/source/index.html`, `target/doc/domain/policy/index.html`, `target/doc/domain/workflow/index.html`, and provider/storage module docs after running `cargo doc --no-deps --workspace`.
- Evidence status: supported local app workflow/tests; durable checkout outcome persistence, production labor measurement, live PMS/provider writes, live billing actions, and customer-message sends are not claimed.
