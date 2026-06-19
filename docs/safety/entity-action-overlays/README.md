# Entity/action safety overlays

Purpose: help a non-coding resort operator, reviewer, or docs writer choose an entity/action first, then see what automation may safely read, draft, rank, recommend, or record; what must stay human-reviewed; which evidence is required; and which outcome or audit record proves the work stayed safe while reducing manual labor.

These overlays are not generic AI policy pages. Each page must name the pet-resort entity or action being handled, the expensive staff work it reduces, the source authority behind a recommendation, the human role that approves sensitive work, and the record that proves what actually happened.

## How this overlay set fits with the existing safety docs

Use these pages alongside, not instead of, the current safety map:

- `../review-boundaries-matrix.md` is the workflow-lane matrix. It answers, by workflow or capability, which reviewer lane owns booking triage, manager daily brief, daily care updates, checkout/retention, tool ports, provider portal, payment, document/vaccine/medical evidence, and Hermes automation hooks.
- `../../design/entity-atlas-review-safety-boundaries.md` is the atlas family page for review gates, blocked actions, allowed actions, drafts/messages, approval records, agent specs, prompt packets, and outcome evidence.
- `../source-evidence-map.md` is the citation map for safety claims. It tells writers which source files, tests, and docs can back a claim.
- This directory is the entity/action overlay layer. It starts from the thing a non-coder is trying to route: a reservation action, a pet-health document, a grooming rebooking draft, a payment exception, a source-data repair, or an outcome record.

If those pages disagree, source/Rustdoc/tests win over this navigation page. Markdown here summarizes and routes; it does not create new automation authority.

## Reader path: choose the entity/action first

1. Pick the entity/action family in the table below.
2. Open the overlay page when its status is `ready`.
3. For unfinished rows, use the listed planned path and the source anchors in `../source-evidence-map.md`, `../review-boundaries-matrix.md`, and `../../design/entity-atlas-review-safety-boundaries.md` until the child overlay is written.
4. In every overlay entry, look for the same safety questions:
   - What may an agent read?
   - What may it draft, recommend, rank, or record?
   - What must it not do directly?
   - Who approves, and under what condition?
   - Which source evidence is required before recommendation?
   - Which outcome/audit record proves safe use and value?

## Overlay pages and status

| Entity/action family | Use when the reader is asking... | Planned page | Status |
| --- | --- | --- | --- |
| Source inventory and evidence backbone | Which entity/action families need overlays, and which source/Rustdoc/test surfaces back them? | `source-inventory.md` | `missing parent artifact`: the prior card was completed as a parallelization gate and did not create this file. Use `../source-evidence-map.md` until this exists. |
| Customer, pet, reservation, booking, checkout | Can automation inspect a customer/pet/reservation, triage a booking, suggest waitlist/capacity/confirmation/cancellation/check-in/out/checkout work, or prepare retention follow-up? | [`customer-pet-reservation-booking-checkout.md`](customer-pet-reservation-booking-checkout.md) | `ready`: source-backed overlay for customer/pet/reservation booking, checkout, waitlist/capacity, cancellation, and retention review boundaries. |
| Pet health, documents, vaccines, temperament, incidents | Can automation extract document/vaccine/care/temperament/incident facts, flag risk, or suggest group-play/medical/behavior review? | [`pet-health-documents-vaccines-incidents.md`](pet-health-documents-vaccines-incidents.md) | `ready`: created from the shared overlay template. |
| Service-line operations, capacity, assignments, packages | Can automation summarize boarding/daycare/grooming/training/retail service-line facts, capacity, packages, assignments, cancellation, or minimum-stay questions? | [`service-line-operations-capacity-assignments.md`](service-line-operations-capacity-assignments.md) | `ready`: source-backed overlay for service-line draft/review-only decisions. |
| Customer communication, daily updates, retention, manager brief actions | Can automation draft daily updates/Pawgress notes, retention or grooming rebooking outreach, internal tasks, manager daily brief recommendations, or customer-message approval packets? | [`customer-communication-daily-updates-retention.md`](customer-communication-daily-updates-retention.md) | `ready`: source-backed overlay for draft/review-only communication and manager-brief actions. |
| Money, payment, provider/source-data, tool-port actions | Can automation read payment/deposit/refund/discount/rate facts, prepare accounting review, inspect Gingr/source data, draft provider/tool-port work, or report external failures? | [`money-payment-provider-tools-source-data.md`](money-payment-provider-tools-source-data.md) | `ready` child overlay. |
| Reviewer roles, outcome/audit proof, labor-value measurement | Which human role approves a sensitive action, and what evidence proves reviewed safe use rather than just source evidence or draft creation? | [`outcome-audit-review-roles.md`](outcome-audit-review-roles.md) | `ready` child crosswalk. |
| Final reviewer usability check | Can a non-coder, operator, IT/security reviewer, compliance reviewer, or product owner use the full overlay set without inventing authority? | [`reviewer-usability-check.md`](reviewer-usability-check.md) | `ready` final QA artifact. |

Statuses deliberately distinguish missing/planned pages from ready pages so a non-coder does not mistake navigation for implemented evidence.

## Required entry shape

Every child overlay page must use `template.md` and keep the same section order. Do not replace the template with generic policy prose. A useful overlay entry should read like:

- “Reservation confirmation” rather than “workflow status update.”
- “Agent may read source-backed intake, pet, reservation, deposit, vaccine, and policy evidence” rather than “agent has access to context.”
- “Agent may draft a staff evaluation packet and customer-message draft for approval” rather than “agent can handle booking.”
- “Agent must not confirm, cancel, check in/out, allocate capacity, waive deposits, move money, mutate provider records, or send customer messages” rather than “human review required.”
- “Front desk lead, manager, medical/vaccine qualified staff, customer-message reviewer, or payment/accounting reviewer approves under named conditions” rather than “a user approves.”
- “Outcome record contains source refs, review gate, blocked action reasons, actor/reviewer, disposition, actual minutes, correlation id, and `live_side_effects_allowed` where present” rather than “logged.”

## Cross-cutting blocked actions

Unless a linked source/Rustdoc/test contract proves a narrower approved path, overlay pages must preserve these blocked actions:

- no autonomous customer/member/public sends;
- no Gingr/PMS/provider writes, record creation/deletion/merge, source hiding, or audit-material edits;
- no booking confirmation/rejection/cancellation, check-in/out, waitlist release, room/capacity/schedule changes, care-task completion, or staff schedule/labor assignment;
- no payment capture/retry/void/refund, discount, deposit waiver/forfeit, credit, write-off, rate/tax/fee change;
- no vaccine, medical, behavior, group-play, incident, legal, safety, policy, or local-SOP approval;
- no destructive source-data cleanup or broad PII/document exposure;
- no secret-dependent or live external side effects.

Writers may say an agent can prepare, draft, rank, validate, flag, route, summarize, or record reviewed outcomes only where the cited app/domain/storage contract allows it.

## Evidence and value rules for all overlays

- Source evidence is not approval. A `domain::source::RecordRef`, `Provenance`, provider fixture, OCR result, or read-model fact explains where a recommendation came from; it does not authorize the live resort action.
- Draft creation is not completion. A message draft, internal task draft, booking triage packet, manager brief action, or tool-port draft must still pass the named review gate before any sensitive downstream action.
- Value is measured after review. Use outcome fields such as disposition, actual minutes, before/after minutes, minutes saved or avoided, wrong-source findings, data-quality issue refs, reviewer/actor, and correlation id. Do not claim realized ROI from intent or estimates alone.
- Markdown is orientation. Behavioral authority stays in source/Rustdoc/tests such as `../../../app/src/agents.rs`, `../../../domain/src/policy.rs`, `../../../domain/src/workflow.rs`, `../../../domain/src/source.rs`, and `../../../storage/src/operations.rs`.

## Source anchors most overlays should cite

Use these anchors before adding more specialized source links:

- Agent specs and prompt packets: `../../../app/src/agents.rs`.
- Workflow events, allowed actions, review recommendations, results, risk flags, and verification notes: `../../../domain/src/workflow.rs`.
- Review gates, automation authority levels, and denial reasons: `../../../domain/src/policy.rs`.
- Source provenance and record references: `../../../domain/src/source.rs`.
- Durable labor/outcome evidence records: `../../../storage/src/operations.rs`.
- App workflow wiki and module map: `../../../app/README.md`.
- Domain entity and policy map: `../../../domain/README.md`.
- Safety citation map: `../source-evidence-map.md`.
- Workflow review matrix: `../review-boundaries-matrix.md`.
- Review-boundary atlas family page: `../../design/entity-atlas-review-safety-boundaries.md`.

## Editor checklist

Before marking an overlay page ready, verify:

- The opening names a specific staff/operator labor or error cost reduced.
- A non-coder can route the entity/action to the right reviewer without reading Rust.
- Every allowed automation verb is a draft/read/rank/recommend/record verb, not a live execution verb.
- Every sensitive action names the blocked action and approval role.
- Every recommendation requirement names the source evidence required before recommendation.
- Every labor-saving claim points to an outcome/audit field and avoids unsupported ROI.
- Every source path is current, local, and used as evidence rather than copied as generic implementation inventory.
