# Grooming Rebooking / Retention

This page describes the current evidence-only grooming-retention packet. It preserves source-backed checkout, service, contact, consent, suppression, and opportunity claims for human inspection, but it cannot establish eligibility, rank or enqueue a candidate, create an internal task, or prepare a customer-follow-up draft.

Status: supported local retention-packet/outcome contract plus grooming-domain vocabulary. It is not evidence of autonomous grooming appointment creation, autonomous customer outreach, live discount/payment movement, or live provider-calendar mutation.

Navigation: start with the [operator workflow index](README.md). Entity-first backlinks: [revenue opportunity entities](../../design/entity-atlas-revenue-opportunity-entities.md), [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [workflow packets](../../design/entity-atlas-workflow-packets-agents.md), and [review gates and blocked actions](../../design/entity-atlas-review-safety-boundaries.md).

## 1. What problem does this solve?

Potential grooming-rebooking evidence is scattered across reported checkout/stay status, customer and pet identity, grooming cadence, service history, contact claims, preferred-channel claims, and suppression risks. The current workflow collects that evidence into a suppressed packet without deciding that anyone is a follow-up candidate.

Example: after a caller reports a completed grooming service, due cadence, and email permission, the workflow still returns `Ineligible`, keeps the draft suppressed, and emits neither queue nor internal-task authority. Those serialized claims remain evidence. A future non-serializable authenticated contact authority would be required to establish eligibility, and customer-message approval would still be required before drafting or sending.

## 2. Whose time does it save?

- Front-desk leads: one evidence packet to inspect instead of disconnected serialized claims.
- Grooming managers: source and service-history context for deciding what further authenticated evidence is required.
- Marketing or retention operators: nonclaimable reported dispositions without a current outreach queue or draft.
- General managers: visibility into why evidence remained suppressed or was reported wrong-source, without implying contact or conversion.

## 3. What source data does it need?

The retention packet needs source-backed facts, not model memory or raw provider names:

| Source fact or entity | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| Reported checkout or stay evidence | Current serialized evidence cannot admit a retention draft or queue item. | Provider/read-model evidence normalized into `checkout_completion::Packet` remains historical evidence; a future opaque accepted capability would be required for downstream authority. | `app/src/crm_retention.rs` `Request.checkout_packet`; `app/tests/crm_retention_workflow_contracts.rs` checkout packet fixture. |
| Customer and reservation ids | Correlates the suppressed evidence packet; it does not create a follow-up queue item. | `domain::entities` ids carried by the app packet. | `app/src/crm_retention.rs` `Request`, `Packet`, `StaffReviewPacket`, `OutcomeRecord`. |
| Pet and grooming service history | Cadence and service-history context explain why grooming follow-up is due, normal, risky, or not supported. | `domain::grooming::history`, `domain::grooming::rebooking`, and promoted source/provider records. | `domain/src/grooming/mod.rs`; `domain/src/grooming/README.md#grooming-workflow-surface`. |
| Grooming cadence and timing | Determines whether a completed service is due later, due now, overdue, or needs recommendation review. | `domain::grooming::rebooking::Policy` and `Cadence` after source facts are promoted into domain values. | `domain/src/grooming/mod.rs`; generated Rustdoc `target/doc/domain/grooming/index.html` after docs build. |
| Groomer/slot constraints and duration evidence | A rebook recommendation must not imply that a specific groomer or calendar slot is available. | `domain::grooming::calendar::Policy`, `DurationEstimate`, `ReviewRequirement`, and human/provider-calendar authority. | `domain/src/grooming/mod.rs`; `domain/src/grooming/README.md#operator-summary`. |
| Contact permission, consent, allowed channels, suppression flags | Preserves caller-reported contact evidence and suppression reasons. Current serialized values cannot establish eligibility or permit a draft. | `crm_retention::ContactPermission` and `crm_retention::SuppressionFlag` with `source::RecordRef`; future eligibility requires a separate authenticated authority. | `app/src/crm_retention.rs` `ContactPermission`, `SuppressionFlag`, `DraftFollowUp`, `FollowUpEligibility`; `app/tests/crm_retention_workflow_contracts.rs`. |
| Source-grounded opportunity reason and provenance | Explains why the opportunity exists and lets staff audit wrong-source findings. Grooming cadence uses `OpportunityReason::GroomingCadenceDue` with `domain::grooming::rebooking` status/rationale. | `crm_retention::RetentionOpportunity`, `OpportunityReason`, and `OpportunityEvidence` using `source::Provenance`; provider state remains evidence until promoted. | `app/src/crm_retention.rs` `RetentionOpportunity`, `OpportunityReason`, `OpportunityEvidence`, `SourceGroundedReasonCode`; `domain/src/source.rs`; `domain/src/grooming/mod.rs`. |

Grooming provider or calendar state remains source evidence unless it is promoted into domain/app packets with a [source ref](../../glossary-architecture-terms.md#source-ref-domainsourcerecordref) and [provenance](../../glossary-architecture-terms.md#provenance-domainsourceprovenance). Raw provider catalog names, calendar notes, or model summaries do not become booking authority by themselves.

## 4. Which entities are featured?

Featured entities:

- Reported retention opportunity: caller-serializable evidence represented by `crm_retention::RetentionOpportunity`; it is not an eligible candidate or queue item.
- Grooming cadence/rebooking status: the domain vocabulary for ordinary grooming intervals and due/overdue/recommendation-needed states.
- Contact permission: serialized consent/channel evidence that cannot authorize a customer draft.
- Staff review packet: a suppressed evidence packet showing universal current ineligibility, source evidence, and manager-review posture without creating work.
- Outcome record: a caller-reported disposition retained as nonclaimable evidence; it does not prove staff review, action, or completion.

Related entities that matter but should not become the page center:

- Checkout packet: reported evidence retained with a suppressed or manager-review packet; it cannot prepare retention review, establish eligibility, or unlock downstream work.
- Pet, customer, reservation, location, and staff ids: identity anchors that connect the retention packet to resort records.
- Grooming service history and style/care notes: context for cadence and risk, but sensitive handling/medical interpretation remains reviewed.
- Message channel/body state: vocabulary for drafts and sends; it does not authorize live outreach.
- Lead and reputation signals: optional supporting signals for follow-up/reputation context, not booking or discount authority.
- Provider calendar/service catalog records: source evidence only until validated into domain/app contracts.

For broader navigation, open the [workflow-to-entity navigation map](../../design/workflow-to-entity-navigation-map.md#workflow---entity-matrix), [Revenue opportunity entity families](../../design/entity-atlas-revenue-opportunity-entities.md), [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [Workflow packets](../../design/entity-atlas-workflow-packets-agents.md), and [Review gates and blocked actions](../../design/entity-atlas-review-safety-boundaries.md).

## 5. Which featured contracts are listed?

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | `crm_retention::Request`, `Packet`, `RetentionOpportunity`, `OpportunityKind`, `OpportunityReason`, `OpportunityEvidence`, `SourceGroundedReasonCode` | Building a suppressed source-grounded evidence packet and preserving reported opportunity labels. | Eligibility, queue/internal-task creation, a customer draft, live outreach, booking creation, provider/PMS mutation, payment/discount action. |
| `app` | `crm_retention::ContactPermission`, `SuppressionFlag`, `FollowUpEligibility`, `DraftFollowUp`, `StaffReviewPacket` | Keeping caller-serializable checkout/contact evidence suppressed or manager-reviewed; current evaluation cannot establish eligibility. | Treating any serialized status, consent, channel, or opportunity evidence as permission to contact. |
| `app` | `crm_retention::SafeAgentAction`, `BlockedAction`, `OutcomeRecord`, `FollowUpOutcome` | Summarizing evidence and recording staff-reported evidence; modeled draft/internal-task actions remain unavailable from serialized evidence. | Sending messages, unlocking queues/tasks/drafts, creating/changing bookings, changing records, moving money, or auto-applying discounts. |
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
- Classify serialized opportunities as suppressed or needing manager review; current packet fields cannot establish eligibility.
- Summarize staff review evidence without prioritizing or enqueuing downstream work.
- Treat `DraftCustomerFollowUpForReview` only as a compatibility label in the source model, never as a current agent capability. A future opaque authenticated contact authority would require a new executable contract.
- Record suppressed or ambiguous evidence without creating an internal task from caller-serializable claims.
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
- `crm_retention::BlockedAction::CreateOrChangeBooking`.
- Any live grooming appointment creation, calendar movement, groomer assignment, or reminder send.
- Any customer outreach where consent/contact permission lacks source evidence, consent is missing, the customer opted out, or the preferred channel is not allowed.

Human-reviewed by default:

- Current packets remain ineligible or suppressed and require `ManagerApproval`; they do not unlock follow-up drafts, queue work, or internal tasks.
- A future authenticated eligibility authority would still require `CustomerMessageApproval` before any customer-facing draft could proceed toward send review.
- Grooming estimates involving weak history, matted coat, special handling, care/medical references, repeat no-shows, or unusual calendar constraints.

## 10. What nonclaimable outcome evidence gets reported?

Staff disposition and [outcome capture](../../glossary-workflow-state-terms.md#outcome-capture) remain caller-reported evidence; they do not measure labor value or establish production revenue claims.

- Reported labor evidence: separately retain estimates and reported time spent without deriving minutes avoided or realized value.
- Current app outcome record: `crm_retention::OutcomeRecord` records reservation id, customer id, staff actor, timestamp, `FollowUpOutcome`, source provenance, and opportunity evidence.
- Supported dispositions/outcomes: booked next stay, interested/needs staff call, not interested, no response, suppressed by staff, converted, deferred, suppressed with reason, or wrong-source.
- Additional evidence summaries: deferred, suppressed, wrong-source, and other reported dispositions, explicitly nonclaimable and not completion, conversion, or revenue authority.
- Evidence gap: this page can cite the app outcome contract and tests. It should not claim production-verified NVA labor savings or a dedicated durable grooming-retention storage table until those exist.

## 11. What code/Rustdoc/test evidence backs this up?

Operator evidence and design:

- [CRM retention agent workflow](../crm-retention-agent.md)
- [Rebooking workflow](../crm-retention-parts/rebooking-workflow.md)
- [CRM retention inputs](../crm-retention-parts/inputs.md)
- [Workflow page source and Rustdoc backing map](../../design/workflow-page-source-rustdoc-map.md#grooming-rebooking-retention)
- [Entity-driven workflow page template and evidence matrix](../../design/entity-driven-workflow-page-template.md)

Source and test evidence:

- [app/src/crm_retention.rs](../../../app/src/crm_retention.rs) for `Request`, `Packet`, `RetentionOpportunity`, `OpportunityKind::GroomingRebook`, `OpportunityReason::GroomingCadenceDue`, `OpportunityEvidence`, `SourceGroundedReasonCode`, `ContactPermission`, `SuppressionFlag`, `DraftFollowUp`, `FollowUpEligibility`, `StaffReviewPacket`, `SafeAgentAction`, `BlockedAction`, `OutcomeRecord`, and `Workflow::evaluate`.
- [app/tests/crm_retention_workflow_contracts.rs](../../../app/tests/crm_retention_workflow_contracts.rs) for executable coverage that every current serialized-input packet remains ineligible, emits neither internal-task nor draft authority, keeps follow-up state suppressed, blocks send/provider/payment/booking mutations, and records converted/deferred/suppressed/wrong-source labels only as nonclaimable reported evidence.
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

- Existing evidence supports suppressed retention evidence packets, universal current ineligibility, blocked actions, nonclaimable reported outcome records, and grooming-domain vocabulary. It does not prove an eligible candidate, queue/internal task, customer draft, contact, conversion, completion, autonomous grooming appointment creation, provider calendar mutation, payment/discount action, or production labor/value measurement.

## Contract crosswalk links

Use the [workflow packet row](../../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map) for the bidirectional path from this workflow page back to the entities it consumes and produces. Use [surface inventory](../../entity-atlas/contract-crosswalk/surface-inventory.md) for source/Rustdoc/test proof, [source/provider flows](../../entity-atlas/contract-crosswalk/source-provider-flows.md) for source-entry and normalization evidence, [storage/persistence](../../entity-atlas/contract-crosswalk/storage-persistence.md) for `CRM retention storage gap`, and [runtime exposure](../../entity-atlas/contract-crosswalk/runtime-exposure.md) for API/worker/CLI/web/script exposure. Rustdoc/module path: `app::crm_retention::Packet`; operator-facing entity family: `CRM retention / grooming rebooking packet`.
