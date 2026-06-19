# Reviewer usability check for entity/action safety overlays

Purpose: this is the final QA evidence artifact for the entity/action safety overlay set. It checks whether a non-coder can start from an entity/action family, see what automation may read/draft/rank/record, identify blocked live actions, route the work to the right reviewer, and find source or test evidence without treating Markdown as new authority.

Scope reviewed:

- `README.md`
- `customer-pet-reservation-booking-checkout.md`
- `pet-health-documents-vaccines-incidents.md`
- `service-line-operations-capacity-assignments.md`
- `customer-communication-daily-updates-retention.md`
- `money-payment-provider-tools-source-data.md`
- `outcome-audit-review-roles.md`
- `template.md`

Authority reminder: this check is an audit/navigation artifact. Behavioral authority remains in source/Rustdoc/tests and the shared safety docs: [`../source-evidence-map.md`](../source-evidence-map.md), [`../review-boundaries-matrix.md`](../review-boundaries-matrix.md), [`../../design/entity-atlas-review-safety-boundaries.md`](../../design/entity-atlas-review-safety-boundaries.md), [`../../../app/src/agents.rs`](../../../app/src/agents.rs), [`../../../domain/src/policy.rs`](../../../domain/src/policy.rs), [`../../../domain/src/workflow.rs`](../../../domain/src/workflow.rs), [`../../../domain/src/source.rs`](../../../domain/src/source.rs), and [`../../../storage/src/operations.rs`](../../../storage/src/operations.rs).

## 1. Non-coder entry test

A reviewer should be able to use this directory in this order:

1. Open [`README.md`](README.md) and pick the entity/action family from the status table.
2. Open the family overlay and read sections 1 through 3 for plain-English meaning, labor/error cost, related workflows, and source authority.
3. Use sections 4 through 6 to separate read access, draft/recommend/rank/record permissions, and blocked direct actions.
4. Use sections 7 through 9 to identify the required human reviewer, source evidence, outcome/audit proof, and labor-value measurement.
5. Use the source/Rustdoc/test evidence section and open-gaps table before making product, ops, compliance, or security claims.

Result: pass, with one caveat. The table still marks `source-inventory.md` as a missing parent artifact. That is safe because the README explicitly points readers back to the source evidence map until the inventory exists.

## 2. Overlay-by-overlay QA

| Overlay | Entity/action organization | Source grounding | Safety boundary | Labor/value proof | Usability result |
| --- | --- | --- | --- | --- | --- |
| [`customer-pet-reservation-booking-checkout.md`](customer-pet-reservation-booking-checkout.md) | Starts from customer, pet, reservation, booking, checkout, cancellation, waitlist/capacity, and retention actions. | Cites app booking/checkout/CRM/manager-brief modules, domain policy/workflow/source, storage outcome records, API/app tests, and gap rows. | Blocks customer sends, provider/PMS writes, capacity/waitlist/room actions, payment/refund/discount movement, and medical/behavior/care approval. | Uses packet/outcome rows, before/actual minutes, wrong-source findings, handle time, and checkout audit fields instead of unsupported ROI. | Pass for front desk, manager, medical/behavior, payment, IT/security, product, and non-coder readers. |
| [`pet-health-documents-vaccines-incidents.md`](pet-health-documents-vaccines-incidents.md) | Starts from document, vaccine, care, medication, temperament, behavior, group-play, incident, and owner-message actions. | Cites document/vaccine/care/temperament/incident/domain source, app booking/daily-update/tool contracts, API tests, and open evidence gaps. | Blocks vaccine validity/waiver, medical/care/behavior/group-play/incident approval, provider writes, customer sends, schedule/capacity changes, payments, destructive cleanup, and policy changes. | Requires review gate, reviewer role, decision reason, audit event, source refs, blocked-action proof, and rework/handle-time fields. | Pass for qualified medical/vaccine reviewers, behavior/daycare leads, managers, customer-message reviewers, payment reviewers, and non-coders. |
| [`service-line-operations-capacity-assignments.md`](service-line-operations-capacity-assignments.md) | Starts from boarding, daycare, grooming, training, retail, capacity, assignment, package, reminder, and reorder actions. | Cites domain service-line modules/READMEs, app manager brief/agents, policy/workflow/source, Gingr retail/provider evidence, and storage outcome records. | Blocks live booking/check-in/out, room/capacity/schedule changes, package/session consumption, POS/provider writes, payment movement, customer sends, medical/behavior approval, and local policy changes. | Uses service-line action queues, manager-brief outcome records, data-quality findings, before/actual minutes, and disposition fields. | Pass for operations, service-line specialists, manager, payment, IT/security, product, compliance, and non-coder readers. |
| [`customer-communication-daily-updates-retention.md`](customer-communication-daily-updates-retention.md) | Starts from message draft, daily update/Pawgress, retention/grooming rebooking, checkout handoff, manager brief, and public/customer-message actions. | Cites app daily update/CRM/checkout/manager-brief/tool messaging, domain message/policy/workflow/source, storage outcomes, workflow docs, and tests. | Blocks live sends, queue/suppression without approval, provider writes, money movement, medical/behavior approval, schedule/capacity changes, destructive cleanup, and policy/tool changes. | Requires source evidence, draft id/body ref, approval/disposition, blocked-send proof, send stub/audit proof, and manager-brief or app outcome records. | Pass for customer-message reviewers, front desk, managers, grooming/retention operators, care/behavior reviewers, payment reviewers, IT/security, product, and non-coders. |
| [`money-payment-provider-tools-source-data.md`](money-payment-provider-tools-source-data.md) | Starts from money, payment, deposit/refund/rate/discount, provider/source mapping, data-quality, tool-port, and Hermes draft actions. | Cites money/payment domain modules, app tools/errors, Gingr endpoint/DTO/mapping/webhook docs, source/data-quality, policy/workflow, storage outcomes, and evidence gaps. | Blocks payment movement, customer sends, provider/PMS/source mutation, schedule/capacity actions, medical/behavior approval, secret-dependent effects, policy/tool authority expansion, and destructive cleanup. | Requires source refs, provider result or ambiguity, review reason, payment/source disposition, wrong-source findings, labor minutes, and blocked-action proof. | Pass after QA fixed stale `MedicalReview` references to the current `MedicalDocumentReview` gate. |
| [`outcome-audit-review-roles.md`](outcome-audit-review-roles.md) | Crosswalk starts from reviewer roles, outcome/audit entities, proof states, and sensitive action families. | Cites policy/workflow/source/storage/app tests and cross-links each role/action to evidence and outcome requirements. | Separates source evidence, draft creation, human approval, outcome proof, and live action elsewhere; blocks live customer/provider/payment/schedule/safety actions without proof. | Gives reviewer/action/outcome examples and required fields for labor-value measurement. | Pass as the reusable role/proof crosswalk for sibling overlays and downstream reviewers. |
| [`template.md`](template.md) | Enforces the entity/action-first section order. | Requires each claim to cite source/Rustdoc/tests or be listed as a gap. | Provides the default blocked-action and reviewer-role scaffold. | Requires outcome/audit fields and labor-value proof. | Pass as future overlay authoring template. |

## 3. Unsupported claims fixed or caveated during final QA

- Fixed stale gate spelling in [`money-payment-provider-tools-source-data.md`](money-payment-provider-tools-source-data.md): `ReviewGate::MedicalReview` was not source-backed by [`../../../domain/src/policy.rs`](../../../domain/src/policy.rs), whose current variant is `ReviewGate::MedicalDocumentReview`.
- Confirmed that the overlay README already marks `source-inventory.md` as a missing parent artifact and routes readers to [`../source-evidence-map.md`](../source-evidence-map.md) instead of inventing a source inventory.
- Kept every live customer send, provider/PMS write, schedule/capacity mutation, payment/refund/discount movement, medical/vaccine/behavior approval, destructive cleanup, policy change, and secret-dependent side effect blocked unless future source/Rustdoc/tests add reviewed authority.

## 4. Remaining evidence gaps / owner decisions

| Gap or decision | Current safest behavior | Owner/evidence needed |
| --- | --- | --- |
| `source-inventory.md` does not exist in this overlay directory. | Use the source evidence map, review-boundary matrix, and cited source files; do not claim missing inventory coverage. | Docs integration/source-inventory owner creates or accepts an equivalent artifact. |
| Production live customer send/provider-write/payment movement/schedule-change authority is not proven by these overlays. | Treat all such actions as draft/review-only or blocked. | Product/ops, IT/security, payment/accounting, and compliance approval plus source/Rustdoc/tests for any deterministic live path. |
| Some entity families have local app outcomes but not durable specialized storage projections. | State labor value as measurable intent or app-local outcome evidence; use storage records only where present. | Storage/app contract and tests for specialized checkout, daily update, retention, payment/source, or pet-health outcome projections. |
| Actual ROI is not established by documentation alone. | Use before/actual minutes, wrong-source findings, rework reduced, handle-time reduced, reviewer disposition, and correlation id fields when outcomes exist. | Real reviewed outcome data, not prose estimates. |

## 5. Stakeholder usability conclusion

- Operations: usable. Entity/action pages route front desk, care team, manager, service-line specialist, and approved sender decisions without requiring Rust knowledge.
- IT/security: usable. Provider/source/tool-port and automation boundaries identify read/draft surfaces, secrets/PII exclusions, write-back gaps, logging/audit expectations, and owner-decision points.
- Compliance: usable. Pages separate source evidence, draft creation, human approval, blocked actions, outcome proof, and live-action gaps.
- Product: usable. Labor-cost claims are tied to measurable outcome fields and open gaps rather than unsupported ROI.
- Non-coder stakeholders: usable. The overlay README table lets a reader choose an entity/action first, then answer what/why/where/relations/authority/automation/blocked/value/proof using consistent sections.
