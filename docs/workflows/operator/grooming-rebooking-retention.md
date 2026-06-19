# Grooming Rebooking / Retention

This page helps front-desk leads, grooming managers, and retention operators avoid manual candidate hunting and repeated follow-up drafting by using a source-backed retention packet to classify grooming rebook opportunities, prepare staff-review drafts, and record outcomes while humans keep approval over customer messages, offers, booking changes, and calendar/provider writes.

Status: supported local retention-packet/outcome contract plus grooming-domain vocabulary. It is not evidence of autonomous grooming appointment creation, autonomous customer outreach, live discount/payment movement, or live provider-calendar mutation.

Navigation: start with the [operator workflow index](README.md). Entity-first backlinks: [revenue opportunity entities](../../design/entity-atlas-revenue-opportunity-entities.md), [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [workflow packets](../../design/entity-atlas-workflow-packets-agents.md), and [review gates and blocked actions](../../design/entity-atlas-review-safety-boundaries.md).

## 1. What problem does this solve?

Grooming rebooking opportunities are easy to miss because the evidence is scattered across completed checkout/stay status, customer and pet identity, grooming cadence, service history, contact permission, preferred channel, and suppression risks. Staff then spend additional time writing similar follow-up copy and deciding whether a consent, complaint, no-show, care, or source-evidence issue should block outreach.

Example: after a completed grooming service, the workflow can flag that the pet is due or overdue for the next grooming cadence, that the customer has source-grounded email permission, and that a friendly rebook draft can be prepared for staff review. If consent is missing, a channel is not allowed, checkout was not staff-verified, or the source evidence is weak, the same workflow suppresses the draft and routes the item to manager review instead.

## 2. Whose time does it save?

- Front-desk leads: less time scanning completed stays, grooming history, contact preferences, and notes to find safe follow-up candidates.
- Grooming managers: less time rebuilding service-history context before deciding whether a rebooking prompt is appropriate.
- Marketing or retention operators: less time preparing approved outreach queues and recording dispositions.
- General managers: less time auditing why a candidate was contacted, deferred, suppressed, or marked wrong-source.

## 3. What source data does it need?

The retention packet needs source-backed facts, not model memory or raw provider names:

| Source fact or entity | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| Completed checkout or stay evidence | A retention draft is only safe after staff-verified completion evidence exists. | Provider/read-model evidence normalized into `checkout_completion::Packet`; staff checkout completion remains authoritative for checkout state. | `app/src/crm_retention.rs` `Request.checkout_packet`; `app/tests/crm_retention_workflow_contracts.rs` checkout packet fixture. |
| Customer and reservation ids | Staff need to know which customer/reservation the follow-up queue item belongs to. | `domain::entities` ids carried by the app packet. | `app/src/crm_retention.rs` `Request`, `Packet`, `StaffReviewPacket`, `OutcomeRecord`. |
| Pet and grooming service history | Cadence and service-history context explain why grooming follow-up is due, normal, risky, or not supported. | `domain::grooming::history`, `domain::grooming::rebooking`, and promoted source/provider records. | `domain/src/grooming/mod.rs`; `domain/src/grooming/README.md#grooming-workflow-surface`. |
| Grooming cadence and timing | Determines whether a completed service is due later, due now, overdue, or needs recommendation review. | `domain::grooming::rebooking::Policy` and `Cadence` after source facts are promoted into domain values. | `domain/src/grooming/mod.rs`; generated Rustdoc `target/doc/domain/grooming/index.html` after docs build. |
| Groomer/slot constraints and duration evidence | A rebook recommendation must not imply that a specific groomer or calendar slot is available. | `domain::grooming::calendar::Policy`, `DurationEstimate`, `ReviewRequirement`, and human/provider-calendar authority. | `domain/src/grooming/mod.rs`; `domain/src/grooming/README.md#operator-summary`. |
| Contact permission, consent, allowed channels, suppression flags | Decides whether a customer-message draft may exist or whether the item is suppressed/manager-reviewed. | `crm_retention::ContactPermission` with `source::RecordRef`; DNC/consent decisions remain human/source-system governed. | `app/src/crm_retention.rs` `ContactPermission`, `FollowUpEligibility`; `app/tests/crm_retention_workflow_contracts.rs`. |
| Source-grounded reason codes and provenance | Explains why the opportunity exists and lets staff audit wrong-source findings. | `crm_retention::OpportunityEvidence` using `source::Provenance`; provider state remains evidence until promoted. | `app/src/crm_retention.rs` `OpportunityEvidence`, `SourceGroundedReasonCode`; `domain/src/source.rs`. |

Grooming provider or calendar state remains source evidence unless it is promoted into domain/app packets with a [source ref](../../glossary-architecture-terms.md#source-ref-domainsourcerecordref) and [provenance](../../glossary-architecture-terms.md#provenance-domainsourceprovenance). Raw provider catalog names, calendar notes, or model summaries do not become booking authority by themselves.

## 4. Which entities are featured?

Featured entities:

- Retention opportunity: the central opportunity item, represented by `crm_retention::RetentionOpportunity` with `OpportunityKind::GroomingRebook` when the reason is grooming-specific.
- Grooming cadence/rebooking status: the domain vocabulary for ordinary grooming intervals and due/overdue/recommendation-needed states.
- Contact permission: the consent/channel evidence that decides whether any customer draft is allowed.
- Staff review packet: the reviewable packet that shows eligibility, draft channel, source evidence, and review gates.
- Outcome record: the staff-recorded disposition proving what happened after review.

Related entities that matter but should not become the page center:

- Checkout packet: prerequisite completion evidence for retention work; checkout authority belongs to the checkout workflow.
- Pet, customer, reservation, location, and staff ids: identity anchors that connect the retention packet to resort records.
- Grooming service history and style/care notes: context for cadence and risk, but sensitive handling/medical interpretation remains reviewed.
- Message channel/body state: vocabulary for drafts and sends; it does not authorize live outreach.
- Lead and reputation signals: optional supporting signals for follow-up/reputation context, not booking or discount authority.
- Provider calendar/service catalog records: source evidence only until validated into domain/app contracts.

For broader navigation, open the [workflow-to-entity navigation map](../../design/workflow-to-entity-navigation-map.md#workflow---entity-matrix), [Revenue opportunity entity families](../../design/entity-atlas-revenue-opportunity-entities.md), [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [Workflow packets](../../design/entity-atlas-workflow-packets-agents.md), and [Review gates and blocked actions](../../design/entity-atlas-review-safety-boundaries.md).

## 5. Which featured contracts are listed?

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | `crm_retention::Request`, `Packet`, `RetentionOpportunity`, `OpportunityKind`, `OpportunityEvidence`, `SourceGroundedReasonCode` | Building a source-grounded retention review packet and classifying opportunities. | Live customer outreach, booking creation, provider/PMS mutation, payment/discount action. |
| `app` | `crm_retention::ContactPermission`, `FollowUpEligibility`, `StaffReviewPacket` | Deciding whether a draft can be prepared for human review or must be suppressed/manager-reviewed. | Treating missing/opted-out/unavailable-channel evidence as permission to contact. |
| `app` | `crm_retention::SafeAgentAction`, `BlockedAction`, `OutcomeRecord` | Summarizing evidence, creating internal review tasks, drafting follow-up for review when eligible, recording staff evidence/outcomes. | Sending messages, changing records, moving money, auto-applying discounts. |
| `domain` | `grooming::Contract`, `DurationEstimate`, `ReviewRequirement`, `calendar`, `history`, `rebooking`, `reminder` | Grooming cadence, service-history, duration, calendar-review, and reminder-boundary vocabulary. | Assigning a live groomer/slot, sending reminders, charging/waiving deposits, or overriding care/medical review. |
| `domain` | `message`, `source`, `policy`, `lead`, `reputation` | Message-state vocabulary, source refs/provenance, review gates, and optional signal context. | Replacing source-system consent, booking, payment, or complaint authority. |
| `storage` | Current cited page has no dedicated grooming-retention storage projection. | App `OutcomeRecord` provides local outcome-capture vocabulary for staff disposition. | Durable production persistence claims unless a storage record/operation is added and cited. |
| `integrations/gingr` | Provider catalog/source surfaces and documented grooming DTO gap | Provider evidence and adapter-boundary context. | Domain truth, approved booking side effects, or automatic grooming DTO mapping authority. |

## 6. Who or what is authoritative?

- Source systems/provider records are authoritative for raw reservation, checkout, contact, service-history, and calendar facts until normalized into app/domain packets.
- `domain::grooming` is authoritative for grooming vocabulary such as service type, duration evidence, service-history requirements, cadence, reminder send boundaries, and review requirements.
- `app::crm_retention` is authoritative for the local workflow packet: eligibility, safe agent actions, blocked actions, staff review packet, source refs, and outcome record shape.
- Human staff/managers are authoritative for customer-message approval, DNC/consent resolution, complaint/incident-sensitive outreach, offers, discounts, refunds, payment movement, and booking/provider/calendar changes.
- Storage is evidence of durable outcomes only where a concrete storage record exists. For this page, the evidence is app-level `OutcomeRecord`; do not claim a dedicated durable grooming-retention projection unless one is added later.

## 7. What does the agent draft, rank, recommend, or record?

The agent may work only inside a [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet) boundary:

- Summarize source-grounded retention and grooming cadence evidence for staff.
- Classify opportunities as eligible, suppressed, wrong-source, or needing review based on packet fields.
- Prioritize staff review packets when source evidence and contact permission are present.
- Prepare a customer follow-up [draft](../../glossary-workflow-state-terms.md#draft) only when `FollowUpEligibility::Eligible` and the safe action `DraftCustomerFollowUpForReview` are present.
- Create internal staff-review tasks for suppressed or ambiguous cases.
- Record staff disposition/outcome evidence in the app outcome record shape.

The agent must not decide that a groomer/slot is available, change a booking, send a message, apply an offer, move money, or reinterpret DNC/consent/complaint evidence.

## 8. What must a human approve?

A human or approved system of record must approve:

- Customer sends and any final customer-facing copy.
- Offers, discounts, refunds, deposits, payment movement, or package/membership changes.
- Booking creation, rebooking, cancellation, provider/PMS mutation, or provider-calendar slot/groomer assignment.
- DNC, consent, allowed-channel, or suppression-list handling.
- Complaint, incident, care, medical, handling, or reputation-sensitive outreach.
- Any source ambiguity, wrong-source candidate, or unsupported cadence/service-history inference.

These are [review gates](../../glossary-workflow-state-terms.md#review-gate), not implementation details.

## 9. What actions are blocked or human-reviewed by default?

Blocked by default:

- `crm_retention::BlockedAction::SendCustomerMessage`.
- `crm_retention::BlockedAction::MutateProviderOrPmsRecord`.
- `crm_retention::BlockedAction::MoveRefundDiscountOrPayment`.
- `crm_retention::BlockedAction::AutoApplyDiscount`.
- Any live grooming appointment creation, calendar movement, groomer assignment, or reminder send.
- Any customer outreach where consent/contact permission lacks source evidence, consent is missing, the customer opted out, or the preferred channel is not allowed.

Human-reviewed by default:

- Eligible follow-up drafts: `Workflow::evaluate` attaches `CustomerMessageApproval` before any send.
- Ineligible or suppressed items: the packet routes them through `ManagerApproval` so staff can decide whether to defer, suppress, correct the source, or review consent.
- Grooming estimates involving weak history, matted coat, special handling, care/medical references, repeat no-shows, or unusual calendar constraints.

## 10. What outcome or labor value gets measured?

Measured labor value should be tied to staff disposition and [outcome capture](../../glossary-workflow-state-terms.md#outcome-capture), not to unverified production revenue claims.

- Estimated labor value: minutes avoided finding candidates, checking service/cadence/contact evidence, and writing first-pass follow-up copy.
- Current app outcome record: `crm_retention::OutcomeRecord` records reservation id, customer id, staff actor, timestamp, `FollowUpOutcome`, source provenance, and opportunity evidence.
- Supported dispositions/outcomes: booked next stay, interested/needs staff call, not interested, no response, or suppressed by staff.
- Additional operator metrics: sent-by-human count, deferred count, suppressed count, wrong-source count, converted count, and optional revenue/fill-rate tracked separately.
- Evidence gap: this page can cite the app outcome contract and tests. It should not claim production-verified NVA labor savings or a dedicated durable grooming-retention storage table until those exist.

## 11. What code/Rustdoc/test evidence backs this up?

Operator evidence and design:

- [CRM retention agent workflow](../crm-retention-agent.md)
- [Rebooking workflow](../crm-retention-parts/rebooking-workflow.md)
- [CRM retention inputs](../crm-retention-parts/inputs.md)
- [Workflow page source and Rustdoc backing map](../../design/workflow-page-source-rustdoc-map.md#grooming-rebooking-retention)
- [Entity-driven workflow page template and evidence matrix](../../design/entity-driven-workflow-page-template.md)

Source and test evidence:

- [app/src/crm_retention.rs](../../../app/src/crm_retention.rs) for `Request`, `Packet`, `RetentionOpportunity`, `OpportunityKind::GroomingRebook`, `OpportunityEvidence`, `SourceGroundedReasonCode`, `ContactPermission`, `FollowUpEligibility`, `StaffReviewPacket`, `SafeAgentAction`, `BlockedAction`, `OutcomeRecord`, and `Workflow::evaluate`.
- [app/tests/crm_retention_workflow_contracts.rs](../../../app/tests/crm_retention_workflow_contracts.rs) for executable coverage that eligible packets still require customer-message approval, missing/opted-out/unavailable contact evidence suppresses drafts, blocked actions include send/provider/payment mutations, and outcome capture records staff evidence only.
- [domain/src/grooming/mod.rs](../../../domain/src/grooming/mod.rs) and [domain/src/grooming/README.md](../../../domain/src/grooming/README.md) for grooming services, cadence/rebooking, duration estimates, calendar policy, no-show/deposit review, reminder send boundaries, service history, and review requirements.
- [domain/src/message.rs](../../../domain/src/message.rs) for message channel/state vocabulary.
- [domain/src/source.rs](../../../domain/src/source.rs) for `RecordRef` and `Provenance` source evidence.
- [domain/src/policy.rs](../../../domain/src/policy.rs) for shared `ReviewGate` vocabulary.
- [domain/src/lead.rs](../../../domain/src/lead.rs) and [domain/src/reputation.rs](../../../domain/src/reputation.rs) for optional supporting signals.

Generated Rustdoc backing exists after running `cargo doc --no-deps --workspace` under:

- `target/doc/app/crm_retention/index.html`
- `target/doc/domain/grooming/index.html`
- `target/doc/domain/message/index.html`
- `target/doc/domain/source/index.html`
- `target/doc/domain/policy/index.html`

Caveats:

- Generated Rustdoc has pre-existing broken intra-doc link warnings in `domain/src/grooming/mod.rs`; this page cites concrete files and generated module locations rather than relying on broken bare grooming links.
- Existing evidence supports retention packets, eligibility/suppression rules, blocked actions, outcome records, and grooming-domain vocabulary. It does not prove autonomous grooming appointment creation, autonomous customer outreach, provider calendar mutation, payment/discount action, or production labor-savings measurement.

## Contract crosswalk links

Use the [workflow packet row](../../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map) for the bidirectional path from this workflow page back to the entities it consumes and produces. Use [surface inventory](../../entity-atlas/contract-crosswalk/surface-inventory.md) for source/Rustdoc/test proof, [source/provider flows](../../entity-atlas/contract-crosswalk/source-provider-flows.md) for source-entry and normalization evidence, [storage/persistence](../../entity-atlas/contract-crosswalk/storage-persistence.md) for `CRM retention storage gap`, and [runtime exposure](../../entity-atlas/contract-crosswalk/runtime-exposure.md) for API/worker/CLI/web/script exposure. Rustdoc/module path: `app::crm_retention::Packet`; operator-facing entity family: `CRM retention / grooming rebooking packet`.
