# Money, payment, provider/source-data, and tool-port action safety overlay

## 1. Plain-English entity/action definition and labor-cost problem

This overlay helps front desk leads, general managers, payment/accounting reviewers, integration owners, IT/security, and docs writers reduce repeated payment exception review, provider/source reconciliation, tool-port triage, and unsafe automation wording for money, payment, deposit/refund/rate/discount, Gingr/provider lookup, source-data mapping, and app/Hermes tool-port actions.

The safe outcome is a reviewed draft, source-backed recommendation, ranked exception queue, internal task draft, deterministic policy check, or outcome record. It is not an autonomous customer send, Gingr/PMS write, schedule/capacity mutation, payment/refund/discount movement, policy change, source cleanup, or secret-dependent live side effect.

Use this page when the operator asks:

- “Can automation inspect a deposit, balance, payment reference, refund window, rate, discount, or invoice-like fact?”
- “Can it prepare payment/accounting review or draft a payment-related customer message?”
- “Can it read Gingr/provider data, map a provider record into an NVA candidate, or repair source data?”
- “Can it call an app tool-port, create a Hermes task/schedule draft, or report an external failure?”

## 2. Workflows/contracts featuring it and adjacent entities

| Workflow or contract | Entity/action role | Adjacent entities | Source path or doc evidence |
| --- | --- | --- | --- |
| Payment/deposit/refund review | Represents money amount, deposit status, external payment reference, authorization/refund result, duplicate/amount/provider ambiguity, and deposit record draft. | Customer, reservation, payment reference, money amount, manager/payment approval, outcome record. | [`../../../domain/src/money/mod.rs`](../../../domain/src/money/mod.rs), [`../../../domain/src/payment/mod.rs`](../../../domain/src/payment/mod.rs), [`../../../app/src/tools.rs`](../../../app/src/tools.rs) payment module, [`../review-boundaries-matrix.md`](../review-boundaries-matrix.md). |
| Booking triage and checkout completion | Reads deposit/payment evidence and source-backed reservation state to route missing payment, failed deposit, checkout, refund, or balance exceptions. | Customer, pet, reservation, provider/source record, policy denial, front desk/manager/payment reviewer. | [`../../../app/src/booking_triage.rs`](../../../app/src/booking_triage.rs), [`../../../app/src/checkout_completion.rs`](../../../app/src/checkout_completion.rs), [`../../../domain/src/workflow.rs`](../../../domain/src/workflow.rs), [`../../../domain/src/policy.rs`](../../../domain/src/policy.rs). |
| Gingr/provider source-data boundary | Reads provider endpoint/response/webhook facts, preserves unknown provider fields, maps only supported provider fields into reviewable domain candidates, and records provider gaps. | Gingr owner, animal, reservation, retail item, webhook, endpoint, raw payload ref, mapping candidate, data-quality issue. | [`../../../integrations/gingr/README.md`](../../../integrations/gingr/README.md), [`../../integrations/gingr/provider-boundary-atlas.md`](../../integrations/gingr/provider-boundary-atlas.md), [`../../integrations/gingr/adapter-boundary-and-labor-source-expansion.md`](../../integrations/gingr/adapter-boundary-and-labor-source-expansion.md). |
| Source/data-quality hygiene | Turns missing, ambiguous, conflicting, sensitive, or unsafe source facts into reviewable issues instead of silently normalizing them. | Source provenance, record ref, reservation/stay/source field path, issue severity, repair disposition, BI visibility. | [`../../../domain/src/source.rs`](../../../domain/src/source.rs), [`../../../domain/src/data_quality.rs`](../../../domain/src/data_quality.rs), [`../../../storage/src/operations.rs`](../../../storage/src/operations.rs). |
| App tool-port actions | Defines read-only stores, availability checks, draft reservation updates, portal lookup, payment gateway requests, message drafts, document/OCR, media snapshot, Hermes task/schedule drafts, and typed errors. | Customer, pet, reservation, capacity snapshot, portal account, payment provider, message draft, document, media ref, Hermes task/schedule. | [`../../../app/src/tools.rs`](../../../app/src/tools.rs), [`../../../app/src/tools/README.md`](../../../app/src/tools/README.md), [`../../../app/src/tools/error.rs`](../../../app/src/tools/error.rs). |
| Agent/runtime validation | Runs structured agent calls only after deterministic app context is built; successful output remains draft/evidence until policy gates run. | Workflow event, policy context, allowed actions, recommended actions, risk flags, verification notes, human review reason. | [`../../../app/src/agents.rs`](../../../app/src/agents.rs), [`../../../domain/src/workflow.rs`](../../../domain/src/workflow.rs), [`../../design/entity-atlas-review-safety-boundaries.md`](../../design/entity-atlas-review-safety-boundaries.md). |
| Durable outcome/value evidence | Stores source refs, workflow/action ids, reviewer/actor/persona, dispositions, before/after/actual minutes, data-quality issue refs, wrong-source findings, and correlation ids. | Stored source record ref, manager brief outcome, data-quality hygiene outcome, labor-minute fields, reporting group. | [`../../../storage/src/operations.rs`](../../../storage/src/operations.rs), [`../source-evidence-map.md`](../source-evidence-map.md). |

Gap: there is no evidence in the inspected contracts that agents may autonomously execute live provider writes, capture/void/refund payments, change rates/discounts, mutate source records, or send customer messages. Treat those as blocked unless a later source/Rustdoc/test contract explicitly adds a reviewed path.

## 3. Who/what is authoritative

| Fact or decision | Authoritative source | What it can prove | What it does not prove |
| --- | --- | --- | --- |
| Money amount and currency | `domain::money::{Money, MinorUnits, Currency}` in [`../../../domain/src/money/mod.rs`](../../../domain/src/money/mod.rs) | A non-zero minor-unit USD amount was represented as a typed domain value. | That a price is correct, approved, charged, discounted, refunded, or posted to a ledger. |
| Deposit state and payment reference | `domain::payment::{Deposit, DepositStatus, Reference}` in [`../../../domain/src/payment/mod.rs`](../../../domain/src/payment/mod.rs) | A reservation deposit is required, paid, refunded, failed, not required, or waived by manager, with a reviewed external reference when present. | Permission to collect, retry, refund, waive, forfeit, discount, or send a customer payment message. |
| Payment authorization/refund/deposit tool request | `app::tools::payment` in [`../../../app/src/tools.rs`](../../../app/src/tools.rs) | The app can ask a gateway for policy-authorized evidence, provider result, or deposit record artifact using typed requests. | That money moved safely, that provider output is business approval, or that an agent may call a live gateway directly. |
| Provider/source fact | `integrations::gingr` endpoint/response/webhook/DTO/mapping modules plus `domain::source` provenance. | Gingr/provider evidence was read, redacted, verified, preserved, or mapped into a candidate with source lineage. | NVA policy approval, source truth beyond the observed provider fact, or provider write-back permission. |
| Data-quality issue | `domain::data_quality::Issue` in [`../../../domain/src/data_quality.rs`](../../../domain/src/data_quality.rs) | A missing, unknown, conflicting, duplicate, ambiguous, payment-conflicting, checkout-missing, vaccine-missing, or sensitive source issue was detected and can block workflow projection. | That the source was repaired, that a business decision was approved, or that destructive cleanup is safe. |
| Tool-port failure | `app::tools::Error` and `ExternalFailure` in [`../../../app/src/tools/error.rs`](../../../app/src/tools/error.rs) | A missing resource, policy denial, or external unavailability was reported in a typed way. | Permission to retry indefinitely, bypass review, expose secrets, or perform live provider work. |
| Human approval | Payment/accounting, manager/general manager, front desk lead, IT/security, customer-message reviewer, or product/ops owner. | Permission for the named reviewed downstream step. | Permission for unrelated customer sends, source mutation, payment movement, policy change, or schedule/capacity action. |
| Outcome/value proof | `storage::operations` records in [`../../../storage/src/operations.rs`](../../../storage/src/operations.rs) and workflow result fields in [`../../../domain/src/workflow.rs`](../../../domain/src/workflow.rs). | What source refs, issues, review gate, actor/reviewer, disposition, minutes, feedback, or correlation id were recorded. | That the agent performed the live action or that future ROI is guaranteed. |

## 4. Agent may read

Agents and agent-facing workflows may read only scoped, source-backed facts that a cited app/domain/storage/integration contract exposes:

- Money and deposit facts: typed amount/currency, deposit status, refundable-until deadline, and reviewed payment reference from [`../../../domain/src/money/mod.rs`](../../../domain/src/money/mod.rs) and [`../../../domain/src/payment/mod.rs`](../../../domain/src/payment/mod.rs).
- Reservation/customer/pet source context through `app::tools::CustomerStore`, which is explicitly read-only and must not create, edit, merge, contact, confirm, cancel, check in/out, or otherwise mutate records; see [`../../../app/src/tools.rs`](../../../app/src/tools.rs).
- Availability evidence through `app::tools::ReservationSystem::check_availability`, including available/unavailable/review-required reasons and a capacity snapshot id, without confirming a booking; see [`../../../app/src/tools.rs`](../../../app/src/tools.rs).
- Portal/provider lookup evidence through `app::tools::portal::Lookup`, including customer, pet, reservation, not-found, or ambiguous matches, without editing provider records or contacting customers; see [`../../../app/src/tools.rs`](../../../app/src/tools.rs).
- Payment provider result evidence returned to the app boundary: authorized/declined/requires-human-review, refund accepted/rejected, deposit record result, amount mismatch, duplicate risk, or provider ambiguity; see `app::tools::payment` in [`../../../app/src/tools.rs`](../../../app/src/tools.rs).
- Document/OCR and media evidence: document classification, extracted text or human-review reason, camera snapshot media ref or unavailable reason. These are evidence packets, not medical/vaccine/safety approvals; see [`../../../app/src/tools.rs`](../../../app/src/tools.rs).
- Gingr source facts: endpoint request shape, response envelope, owner/animal/reservation/reference records, retail DTOs, mapping candidates, source inventory, and webhook verified envelope; see [`../../../integrations/gingr/README.md`](../../../integrations/gingr/README.md), endpoint/DTO/mapping modules, [`../../../integrations/gingr/src/response.rs`](../../../integrations/gingr/src/response.rs), and [`../../../integrations/gingr/src/webhook.rs`](../../../integrations/gingr/src/webhook.rs).
- Source provenance and record refs: source system, endpoint, record id, related record roles, extraction batch, timestamps, request scope, schema version, payload hash, raw payload ref, Gingr provider IDs/statuses, and source reservation snapshots; see [`../../../domain/src/source.rs`](../../../domain/src/source.rs).
- Data-quality issue metadata: kind, severity, provenance, detected_at, resolution status, BI visibility, and workflow_blocking flag; see [`../../../domain/src/data_quality.rs`](../../../domain/src/data_quality.rs).
- Prior reviewed outcome/value records when available, including source refs, issue refs, actor/reviewer/persona, disposition, actual minutes, before/after minutes, wrong-source findings, and correlation ids; see [`../../../storage/src/operations.rs`](../../../storage/src/operations.rs).

Scope limits:

- Read access is tied to a location, operating day, customer, pet, reservation, document, payment reference, source snapshot, workflow event, or typed request packet.
- Provider credentials, raw sensitive lookup values, phone/email lookups, high-PII documents, payment details, and webhook secrets are not documentation artifacts and must not be pasted into docs, logs, comments, or prompts.
- Unknown provider fields and raw payload refs may be preserved as audit evidence, but they are not policy or workflow facts until a mapping/source contract promotes them.

## 5. Agent may draft/recommend/rank/record

Allowed actions are review-safe and artifact-specific:

| Entity/action | Agent may draft/recommend/rank/record | Artifact and authority |
| --- | --- | --- |
| Deposit required/failed/paid/refunded/waived | Recommend payment/accounting review, flag duplicate/amount/provider ambiguity, summarize payment evidence, and record reviewed deposit status or exception disposition. | `domain::payment::DepositStatus`, `app::tools::payment::{Subject, ReviewReason}`, `domain::policy::ReviewGate::RefundOrDepositException`, storage outcome fields. |
| Authorization/refund request | Draft a payment/accounting review packet from typed amount, subject, idempotency key, payment reference, provider result, and review reason. | `app::tools::payment::authorization::Request`, `refund::Request`, provider `Result`, and typed review reasons in [`../../../app/src/tools.rs`](../../../app/src/tools.rs). |
| Rate/discount/fee/tax/write-off question | Flag as a gap or payment/accounting/product-owner decision unless a source contract supplies reviewed rate/discount authority; summarize evidence only. | Current inspected contracts model money/payment/deposit/refund but do not authorize autonomous rate/discount changes. |
| Provider lookup or mapping candidate | Recommend which source candidate needs staff, manager, or integration-owner review; record not-found/ambiguous/provider-gap outcome. | `app::tools::portal::lookup::Match`, `gingr::mapping::{customer, pet, retail}`, `domain::source::RecordRef`, `domain::data_quality::Issue`. |
| Source-data quality issue | Rank repair queues, flag blocking/critical issues, recommend “do not rely on this source fact,” and record issue disposition/wrong-source findings after review. | `domain::data_quality::{Issue, Severity, ResolutionStatus}`, storage data-quality hygiene outcome records. |
| Message or customer-facing explanation | Draft a message body only, with DraftOnly or ManagerApprovalRequired review policy. | `app::tools::messaging::draft::Request`, `ReviewPolicy`, `draft::Status`, and customer-message reviewer approval. |
| Document/OCR/media evidence | Extract/summarize evidence, flag low-confidence/ambiguous/unavailable reasons, and route to the correct reviewer. | `app::tools::documents::*`, `app::tools::media::*`; medical/vaccine/behavior/incident approvals remain separate reviewer gates. |
| Hermes task/schedule action | Draft an internal task or schedule-change proposal for staff/manager review; record returned draft id/status. | `app::tools::hermes::AutomationHooks`, `task::DraftRequest`, `schedule::DraftRequest`, `DraftStatus`. |
| External failures | Record or route not-found, policy-denied, portal/payment/message/storage unavailable, and other typed failures. | `app::tools::Error`, `ExternalFailure`, workflow verification/human review fields. |

Use `read`, `summarize`, `compare`, `validate`, `draft`, `recommend`, `rank`, `flag`, `route`, and `record` language. Do not use live execution verbs such as “charge,” “refund,” “discount,” “send,” “write back,” “check in,” “check out,” “delete,” “repair source,” or “change schedule” unless the sentence explicitly says the agent must not do that directly.

## 6. Agent must not do directly

| Blocked direct action | Why blocked | Correct safe path |
| --- | --- | --- |
| Capture, retry, void, refund, waive, forfeit, discount, write off, change tax/fee/rate, or move money. | Payment/accounting trust, customer financial harm, provider ambiguity, duplicate risk, and lack of live payment authority in inspected contracts. | Draft payment/accounting review packet with source refs, amount, payment reference, idempotency key, provider result, review reason, and outcome field. |
| Send a customer/member/parent payment, refund, collections, daily update, marketing, or portal message. | Customer trust, contact preference/consent, payment wording, medical/vaccine/behavior ambiguity, and suppression rules. | Draft message only; approved sender/customer-message reviewer approves final recipient, channel, timing, body, and suppression. |
| Confirm, cancel, reject, check in/out, release waitlist, allocate room/capacity, mutate schedule, or complete care/staff tasks. | Booking/care/labor actions can affect pet safety, capacity, customer promises, and staff work. | Draft reservation update, internal task, or manager review packet; front desk lead or manager executes outside the agent after review. |
| Write to Gingr/PMS/provider records, create/delete/merge/hide source records, or edit audit material. | Provider facts are evidence, not NVA policy; destructive cleanup can erase audit trails. | Preserve source refs/raw payload refs, create data-quality issue or source repair candidate, route to integration owner/IT/security/staff reviewer. |
| Approve vaccine, medical, medication, behavior, temperament, group-play, incident, legal, safety, or local-SOP decisions. | These require qualified operational reviewers and often depend on documents, local policy, and direct observation. | Extract/summarize evidence and route to medical/vaccine qualified staff, behavior/daycare lead, manager, or safety reviewer. |
| Treat unknown provider fields, provider IDs, provider statuses, webhook events, or DTO gaps as domain truth. | Provider wire data can be incomplete, ambiguous, stale, undocumented, or unmapped. | Keep provider facts quarantined, map only supported fields, preserve unknowns for audit, mark mapping/data-quality gaps. |
| Use credentials, API keys, webhook secrets, passwords, raw PII, payment-sensitive payloads, phone/email lookups, or high-PII documents in docs/prompts/logs. | Security and privacy boundary; the Gingr config/transport/webhook contracts redact or verify secrets. | Use redacted request capture, sanitized fixture/reference, source ref, or raw payload ref without copying sensitive content. |
| Change automation policy, add live tool authority, enable secret-dependent external side effects, or broaden a tool-port beyond its typed contract. | Product/ops/IT/security authority and source/Rustdoc/tests must define capabilities before use. | Mark owner decision needed; keep draft-only/read-only behavior until source and review gates exist. |

## 7. Required human reviewer role(s) and approval condition

| Role | Usually approves | Does not approve |
| --- | --- | --- |
| Payment/accounting reviewer | Deposit collection/waiver/refund/balance exception, duplicate risk, amount mismatch, provider ambiguity, payment wording in a draft. | Medical/behavior/schedule decisions, provider write-back security, broad customer-message approval outside payment wording, policy changes. |
| Front desk lead | Routine source-backed intake completeness, booking/payment handoff quality, internal task routing, customer/reservation fact correction candidates. | Manager exceptions, money movement, medical/behavior approval, provider write-back permission, IT integration scope. |
| Manager/general manager | Capacity/staffing/customer-trust exceptions, deposit waiver policy where locally authorized, incident/escalation disposition, schedule/capacity review. | Payment processing authority unless assigned, integration secrets, medical/vaccine validity unless qualified, source schema policy. |
| Customer-message reviewer/approved sender | Final recipient, channel, timing, body, contact preference/consent, suppression, customer-facing payment wording. | Provider/PMS mutation, payment movement, medical/behavior approval, policy/tool authority. |
| Medical/vaccine qualified staff | Vaccine/medical/document ambiguity, medication/care readiness, health evidence review. | Payment/refund/discount decisions, provider write-back, broad customer communication outside medical wording. |
| Behavior/daycare lead | Temperament, group-play, behavior safety, incident-care implications. | Payment, provider write-back, broad policy/tool changes. |
| IT/security/integration owner | Gingr/API/webhook integration scope, credential handling, redaction, provider endpoint allowlist, retry/failure handling, logging boundaries. | Business approval for payment/customer/care decisions, individual live actions. |
| Product/ops owner | Whether a workflow/tool-port is allowed to exist, whether a new live authority may be designed, and what approval policy it needs. | Individual payment/customer/provider/care action approval without the appropriate operational reviewer. |

`domain::policy::ReviewGate::RefundOrDepositException` is the named gate for refund/deposit exceptions. `ManagerApproval`, `CustomerMessageApproval`, `MedicalDocumentReview`, and `BehaviorReview` apply when payment/source/tool-port facts are adjacent to capacity, messages, medical/vaccine, or behavior safety.

## 8. Required source evidence before a recommendation

| Recommendation or draft | Required source evidence first | If missing/stale/ambiguous |
| --- | --- | --- |
| Deposit/payment exception review | Reservation id, customer/pet context if relevant, typed amount/currency, deposit status, payment reference if present, refundable-until if present, provider result or failure, source record refs, policy/review gate. | Route to payment/accounting; do not recommend collection/refund/waiver/discount/money movement. |
| Refund request packet | Reviewed payment reference, amount, refund reason, idempotency key, reservation/customer context, refund window/source evidence, manager/payment approval condition. | Fail closed; record missing evidence or provider ambiguity; do not call refund live. |
| Discount/rate/fee/tax/write-off decision | Current gap: inspected contracts do not define a live rate/discount authority. Need explicit source, policy, approval, and outcome record. | Mark owner decision needed; draft evidence summary only; do not change charges or promises. |
| Provider lookup/mapping candidate | Provider system/account, endpoint/event, criteria, include flags, provider record id/entity id, raw payload ref, payload hash, mapper name, mapping error or candidate type. | Preserve as provider evidence; create data-quality issue or integration-owner task; do not treat as NVA truth. |
| Webhook-driven workflow cue | Verified webhook envelope, event/entity type/id, signature verification result, payload ref, mapping contract, workflow event/correlation id. | Reject or quarantine unverified/malformed payload; do not trigger customer/payment/provider actions. |
| Source-data repair or hygiene queue | `domain::source::Provenance`, `RecordRef`, field path, issue kind/severity, detected_at, workflow_blocking flag, raw payload ref, current resolution status. | Keep issue open/acknowledged; do not delete/hide/edit source data; route to source owner. |
| Tool-port draft or failure report | Typed request packet, allowed action/policy context, tool result or `app::tools::Error`, external failure classifier, draft id/status if produced. | Record failure and route to IT/security or workflow owner; do not bypass gates or retry live side effects from the agent. |
| Message draft based on payment/source data | Source refs, reviewer-facing facts, suppression/contact preference evidence when available, body, recipient, channel, review policy. | Draft only or stop; customer-message reviewer/approved sender must approve before send. |

## 9. Outcome/audit record proving safe use and value measurement

Safe use is proven by the chain of source evidence, draft/recommendation, human review, and durable outcome/value record. Draft creation alone is not completion; source evidence alone is not approval.

| Proof needed | Example field or record | What it proves | What it does not prove |
| --- | --- | --- | --- |
| Source evidence | `domain::source::RecordRef`, `Provenance`, provider endpoint/event, payload hash, raw payload ref, source refs, issue refs. | The recommendation was grounded in traceable evidence. | Approval, completion, or live action. |
| Draft/recommendation | Workflow event id, context packet id, draft id, safe action list, recommended action, risk flag, verification note, tool-port request/result. | Work product was prepared and validated for review. | Customer send, provider write, payment movement, schedule/capacity change, or source repair. |
| Human approval | Review gate, reviewer role, actor/persona, approval status, timestamp, decision reason, correction/suppression reason. | The named sensitive step was reviewed by the right role. | Authority for unrelated steps or future cases. |
| Payment/source disposition | Deposit status, payment provider result, refund result, not-found/ambiguous mapping result, data-quality resolution status, blocked-action reason. | What staff/accounting/integration decided or rejected. | That the agent executed money movement or source mutation. |
| Labor/value evidence | Before minutes, actual minutes, minutes saved/avoided, wrong-source findings, rework reduced, handle time reduced, feedback, reporting group, correlation id. | Reviewed work was measured after the fact. | Guaranteed ROI or safe future automation without review. |
| Blocked-action proof | `live_side_effects_allowed: false`, outcome persisted false, blocked action reason, policy denial, external failure, workflow_blocking issue. | Automation stopped rather than taking unsafe action. | That the downstream operational issue is resolved. |

Value-measurement rows to capture for this family:

| Entity/action family | Labor-value signal | Reviewer disposition to record |
| --- | --- | --- |
| Deposit/payment/refund exceptions | Minutes avoided by pre-assembling amount/reference/provider evidence; duplicate or amount mismatch caught before staff/provider work; handle time reduced for accounting review. | Approved, corrected, rejected, routed to manager, routed to customer-message reviewer, or blocked due to missing evidence. |
| Provider/source mapping | Wrong-source findings, ambiguous candidate count, mapping error frequency, source facts promoted without manual screen comparison. | Accepted candidate, corrected candidate, source gap, provider gap, data-quality issue opened/repaired/acknowledged. |
| Tool-port/Hermes drafts | Internal task/schedule draft created without live mutation; external failure classified; review queue narrowed. | Drafted, drafted requires review, failed closed, policy denied, external unavailable, owner decision needed. |
| Documents/OCR/media used as supporting evidence | Rework reduced by routing low-confidence, ambiguous, or unavailable evidence to the right reviewer. | Needs human review, unreadable/unavailable, accepted as evidence only, routed to medical/behavior/manager. |

## 10. Source/Rustdoc/test evidence links

Shared safety anchors:

- [`../../../app/src/agents.rs`](../../../app/src/agents.rs) for `AgentSpec`, `WorkflowAgent`, `AgentPromptPacket`, forbidden actions, prompt packets, and review gates.
- [`../../../domain/src/policy.rs`](../../../domain/src/policy.rs) for `domain::policy::automation::Level`, denial reasons, and `ReviewGate::{ManagerApproval, MedicalDocumentReview, BehaviorReview, CustomerMessageApproval, RefundOrDepositException}`.
- [`../../../domain/src/workflow.rs`](../../../domain/src/workflow.rs) for workflow events, allowed actions, recommendations, results, risk flags, verification notes, and human review reasons.
- [`../../../domain/src/source.rs`](../../../domain/src/source.rs) for `RecordRef`, `Provenance`, source systems, Gingr provider provenance, source reservation snapshots, source assumptions, and promotion boundaries.
- [`../../../domain/src/data_quality.rs`](../../../domain/src/data_quality.rs) for source/data-quality issue kinds including payment state conflict, checkout evidence missing, duplicate records, ambiguous owner-pet relationships, sensitive payload quarantine, and workflow-blocking status.
- [`../../../storage/src/operations.rs`](../../../storage/src/operations.rs) for stored source refs, manager daily brief/data-quality hygiene outcome records, labor-minute fields, actor/reviewer/persona, feedback, issue refs, and correlation ids.
- [`../source-evidence-map.md`](../source-evidence-map.md), [`../review-boundaries-matrix.md`](../review-boundaries-matrix.md), and [`../../design/entity-atlas-review-safety-boundaries.md`](../../design/entity-atlas-review-safety-boundaries.md) for shared routing and safety-navigation language.

Money/payment/tool-port anchors:

- [`../../../domain/src/money/mod.rs`](../../../domain/src/money/mod.rs) for `Money`, `MinorUnits`, and `Currency` validation.
- [`../../../domain/src/payment/mod.rs`](../../../domain/src/payment/mod.rs) for `DepositStatus`, `Deposit`, `Reference`, and review-safe money movement state.
- [`../../../app/src/tools.rs`](../../../app/src/tools.rs) for `CustomerStore`, `ReservationSystem`, `AgentRuntime`, `availability`, `draft_update`, `portal`, `payment`, `messaging`, `documents`, `media`, `hermes`, and `ExternalToolCandidate`.
- [`../../../app/src/tools/README.md`](../../../app/src/tools/README.md) for the tool-port module map and safe context/draft/outcome posture.
- [`../../../app/src/tools/error.rs`](../../../app/src/tools/error.rs) for `Error::{NotFound, PolicyDenied, External}` and `ExternalFailure` classifiers.

Gingr/provider anchors:

- [`../../../integrations/gingr/README.md`](../../../integrations/gingr/README.md) for the provider boundary, module map, labor-cost role, and explicit split between provider facts, domain truth, and persisted projections.
- [`../../../integrations/gingr/src/endpoint/mod.rs`](../../../integrations/gingr/src/endpoint/mod.rs), [`../../../integrations/gingr/src/endpoint/reservations.rs`](../../../integrations/gingr/src/endpoint/reservations.rs), [`../../../integrations/gingr/src/endpoint/owners_animals.rs`](../../../integrations/gingr/src/endpoint/owners_animals.rs), [`../../../integrations/gingr/src/endpoint/commerce_retail.rs`](../../../integrations/gingr/src/endpoint/commerce_retail.rs), [`../../../integrations/gingr/src/endpoint/reference_data.rs`](../../../integrations/gingr/src/endpoint/reference_data.rs), [`../../../integrations/gingr/src/endpoint/report_cards_files.rs`](../../../integrations/gingr/src/endpoint/report_cards_files.rs), [`../../../integrations/gingr/src/endpoint/labor_ops.rs`](../../../integrations/gingr/src/endpoint/labor_ops.rs), and [`../../../integrations/gingr/src/endpoint/catalog.rs`](../../../integrations/gingr/src/endpoint/catalog.rs) for provider read request shapes and documented gaps.
- [`../../../integrations/gingr/src/response.rs`](../../../integrations/gingr/src/response.rs) for raw response/envelope/provider-record quarantine and unknown-field preservation.
- [`../../../integrations/gingr/src/dto/mod.rs`](../../../integrations/gingr/src/dto/mod.rs), [`../../../integrations/gingr/src/dto/retail.rs`](../../../integrations/gingr/src/dto/retail.rs), [`../../../integrations/gingr/src/dto/grooming.rs`](../../../integrations/gingr/src/dto/grooming.rs), and [`../../../integrations/gingr/src/dto/training.rs`](../../../integrations/gingr/src/dto/training.rs) for modeled DTOs and provider-surface gap markers.
- [`../../../integrations/gingr/src/mapping/mod.rs`](../../../integrations/gingr/src/mapping/mod.rs), [`../../../integrations/gingr/src/mapping/customer.rs`](../../../integrations/gingr/src/mapping/customer.rs), [`../../../integrations/gingr/src/mapping/pet.rs`](../../../integrations/gingr/src/mapping/pet.rs), and [`../../../integrations/gingr/src/mapping/retail.rs`](../../../integrations/gingr/src/mapping/retail.rs) for promoted customer contact, pet name, and retail product candidates plus mapping errors.
- [`../../../integrations/gingr/src/webhook.rs`](../../../integrations/gingr/src/webhook.rs) for signature verification, event/entity normalization, unknown value preservation, acknowledgement, and verification errors.
- [`../../integrations/gingr/provider-boundary-atlas.md`](../../integrations/gingr/provider-boundary-atlas.md) and [`../../integrations/gingr/adapter-boundary-and-labor-source-expansion.md`](../../integrations/gingr/adapter-boundary-and-labor-source-expansion.md) for non-coder provider/source boundary wording.

## 11. Open gaps or owner decisions

| Gap or owner decision | Why it matters | Safest current behavior | Evidence needed to close |
| --- | --- | --- | --- |
| Live payment movement authority is not established for agents. | Payment capture/refund/void/retry/waiver/discount can create financial and customer-trust harm. | Draft payment/accounting review packets only; record provider evidence and review disposition. | Source/Rustdoc/tests for a reviewed payment execution workflow, payment/accounting approval record, idempotency/audit design, and blocked-action tests. |
| Rate/discount/tax/fee/write-off entity model is not clearly authorized in inspected source. | Operators may ask about discounts or rate changes, but current evidence supports money/payment/deposit/refund language more clearly than live rate edits. | Mark as owner decision; summarize evidence only; do not promise or mutate charges. | Domain/app/storage contracts for rate/discount policy, reviewer role, outcome record, and tests. |
| Provider/PMS write-back is not authorized by the Gingr boundary. | Read/mapping contracts exist; live HTTP is not implemented in this slice and provider writes are broadly blocked. | Keep Gingr/provider work read-only, redacted, mapped, or draft-only. | Explicit provider write contract, integration-owner approval, IT/security review, audit/redaction tests, rollback/idempotency plan. |
| Source inventory artifact for entity-action overlays is missing. | This page cites current source/docs directly, but the overlay directory still lacks a parent inventory artifact. | Use `../source-evidence-map.md` and cited source files; mark unsupported claims as gaps. | Completed `docs/safety/entity-action-overlays/source-inventory.md` or equivalent accepted source inventory. |
| Data-quality repair outcomes for payment/provider/source issues may need more specialized storage fields. | Existing storage outcome fields support source refs, issue refs, labor minutes, wrong-source findings, feedback, and correlation ids, but payment-specific disposition fields may be added later. | Record existing source refs/issues/disposition/minutes where available; do not invent storage fields. | Storage/app contract for payment/provider hygiene outcomes and tests proving persistence. |
| Customer-message suppression/consent evidence is not fully enumerated in this overlay. | Payment/source facts often feed customer-facing messages, but message approval requires contact preference and suppression context. | Draft only; require approved sender/customer-message reviewer. | Message workflow contract and tests naming suppression/contact preference evidence. |
| Medical/vaccine/behavior/incident approvals adjacent to documents/media/source facts are outside this family. | Tool ports can read documents/OCR/media, but approval authority belongs to specialized reviewers. | Route to the pet-health overlay and qualified reviewer; do not approve. | Specialized pet-health overlay and linked source/test evidence. |

## Final reviewer checklist

- The opening names concrete labor/error costs for front desk, manager, accounting, integration owner, IT/security, and docs-writer readers.
- A non-coder can route deposits/refunds/discount gaps, provider/source mapping, data-quality issues, message drafts, documents/media evidence, and Hermes/tool-port drafts to the right reviewer.
- “Agent may read,” “agent may draft/recommend/rank/record,” and “agent must not do directly” are separate.
- Source evidence, human approval, draft creation, and outcome proof are clearly different.
- Blocked actions are precise: no customer sends, provider/PMS writes, schedule/capacity changes, payment/refund/discount/rate movement, medical/vaccine/behavior approvals, destructive cleanup, policy changes, or secret-dependent live side effects.
- Labor-value claims require outcome/audit fields rather than intent.
- Source/Rustdoc/test links are current and local.
- Gaps are marked as gaps instead of being filled with assumptions.
