# Outcome/audit proof and reviewer-role crosswalk

This overlay helps front-desk leads, managers, medical/vaccine reviewers, behavior/daycare leads, customer-message reviewers, payment/accounting reviewers, IT/security, and product/ops owners reduce repeated “who signs off and what proves it?” handoffs by separating source evidence, draft/recommendation work, human approval, any live action elsewhere, and the durable outcome record that measures labor value.

Use this page as reusable language for sibling entity/action overlays. It is not a generic policy page and it does not approve new live automation. It tells writers how to route an entity/action to the right reviewer and how to prove safe use with outcome/audit fields.

## 1. Plain-English entity/action definition and labor-cost problem

Reviewer-role and outcome/audit proof is the cross-cutting safety layer that answers four non-coder questions for any pet-resort entity/action:

1. Which facts did automation read?
2. Which draft, recommendation, ranking, or internal task did it produce?
3. Which human role approved, rejected, deferred, suppressed, or escalated the sensitive step?
4. Which outcome/audit record proves what actually happened and how many minutes were saved or avoided?

The labor problem is repeated manager/front-desk reconciliation after an agent prepares work: staff have to re-check source records, decide whether a manager/medical/payment/message/security reviewer is required, and then later prove whether the recommendation helped. The safe outcome is a role-owned, source-backed workflow packet plus a durable outcome record with reviewer, disposition, correlation, source, blocked-action, and minute evidence.

## 2. Workflows/contracts featuring it and adjacent entities

| Workflow or contract | Entity/action role | Adjacent entities | Source path or doc evidence |
| --- | --- | --- | --- |
| Booking triage | Routes reservation readiness, missing info, vaccine/deposit/policy gates, and draft confirmations to staff, manager, medical, payment, or message review. | Customer, pet, reservation, policy snapshot, deposit/payment evidence, document/vaccine proof, confirmation draft, staff evaluation packet. | `../../../app/src/booking_triage.rs`; `../../../domain/src/policy.rs`; `../../../domain/src/workflow.rs`; `../review-boundaries-matrix.md`. |
| Manager Daily Brief | Ranks manager/front-desk work and records manager/staff feedback and actual minutes without changing schedules, PMS records, payments, or customer messages. | Operating day, source facts, checkout packets, retention packets, data-quality issues, brief action, outcome record. | `../../../app/src/manager_daily_brief.rs`; `../../../storage/src/operations.rs`; `../../design/manager-daily-brief-measurable-labor-loop.md`. |
| Data-quality hygiene | Turns duplicate/missing/stale/ambiguous source facts into reviewed cleanup tasks and outcome evidence without hiding ambiguity or repairing provider records autonomously. | Source refs, provenance, issue refs, source record, customer/pet/reservation/service fields, cleanup task, outcome record. | `../../../domain/src/source.rs`; `../../../domain/src/data_quality.rs`; `../../../storage/src/operations.rs`; `../source-evidence-map.md`. |
| Daily updates / customer messaging / retention | Prepares customer-visible drafts, suppression reasons, contact/channel checks, and follow-up packets for an approved sender. | Message, draft body ref, customer/pet/reservation/stay facts, contact permission, included/omitted facts, approval record. | `../../../app/src/agents.rs`; `../../../app/src/tools.rs`; `../../../domain/src/message.rs`; `../../design/labor-cost-reduction-crosswalk.md`. |
| Payment, deposit, refund, discount, and provider/tool-port actions | Keeps money movement and provider writes behind payment/accounting, manager, IT/security, and product/ops decisions while allowing review packets and failure/task drafts. | Payment reference, money/deposit status, provider receipt/ref, tool-port draft, external failure, approval/audit event. | `../../../app/src/tools.rs`; `../../../domain/src/payment/mod.rs`; `../../../domain/src/money/mod.rs`; `../review-boundaries-matrix.md`. |
| Agent specs and prompt packets | Defines what a helper may see, which draft output schema it may produce, and which forbidden actions/review gates stay outside model control. | Agent spec, prompt packet, workflow event, policy instruction, expected output schema, validation result. | `../../../app/src/agents.rs`; `../../../domain/src/agent.rs`; `../../../domain/src/workflow.rs`. |
| Storage outcome records | Persist reviewed outcome evidence and labor minute deltas; they do not authorize live provider, customer-message, payment, or schedule actions. | Action id, disposition, before/actual minutes, actor, persona, source refs, correlation id, reporting dimensions. | `../../../storage/src/operations.rs`; `../../../app/src/manager_daily_brief.rs`; `../source-evidence-map.md`. |

## 3. Who/what is authoritative

| Fact or decision | Authoritative source | What it can prove | What it does not prove |
| --- | --- | --- | --- |
| Source fact | `domain::source::{RecordRef, Provenance}`, provider/read-model source refs, source snapshots, issue refs. | Which source system, endpoint, record id, extraction batch, timestamp, schema, payload hash, raw payload ref, and request scope supported the recommendation. | Human approval, production permission, or that a live action was executed. |
| Business/review vocabulary | `domain::policy::{ReviewGate, automation::Level, denial::Reason}` and domain service-line policy modules. | Which named review gate or automation authority level applies. | Provider/PMS write-back permission or local operational approval outside the named gate. |
| Workflow packet and validation | `domain::workflow::{Event, PolicyContext, AllowedAction, Result, RecommendedAction}` and app workflow packets. | What may be read, drafted, ranked, validated, recommended, or routed for review. | That the recommendation was approved or executed. |
| Agent prompt/spec | `app::agents::{AgentSpec, WorkflowAgent, AgentPromptPacket}` and `domain::agent::Spec`. | Workflow identity, purpose, allowed tools, forbidden actions, default review gates, source event, input, policies, and output schema. | Live authority to book, charge, message, schedule, diagnose, or override policy. |
| Human approval | Approval record, reviewer lane, review gate, approval status, actor/reviewer, timestamp, decision reason. | The named reviewer accepted, rejected, deferred, suppressed, or escalated the target action. | Permission for unrelated actions, future similar actions, or broader automation authority. |
| Outcome/audit record | App/storage outcome record, workflow event/result, API validation result, tool-port draft/failure record. | What staff did with the recommendation, what remained blocked, whether a side-effect request was rejected, and what minutes were actually observed. | Guaranteed future ROI or proof that the agent itself performed a live action. |

## 4. Agent may read

When the workflow/app contract allows the read, an agent or app workflow may inspect only scoped, source-backed context needed for the entity/action:

- Source refs and provenance: `RecordRef`, `Provenance`, source system, endpoint, source record id, extraction batch, pulled timestamp, request scope, schema version, payload hash, and raw payload reference from `../../../domain/src/source.rs`.
- Workflow and policy context: workflow event id, event type, actor, location, subject, allowed actions, automation level, required reviews, risk flags, verification notes, and review reasons from `../../../domain/src/workflow.rs`.
- App-owned packets: booking triage `StaffEvaluationPacket`, manager daily brief `Packet` and `BriefAction`, checkout/retention/daily-update packet or draft shapes, and `AgentPromptPacket` inputs where the module exposes them.
- Reviewer-facing facts: customer/pet/reservation/document/vaccine/care/incident/payment/message/source-hygiene facts only when scoped to the packet, location, operating day, customer, pet, reservation, document, payment ref, or workflow run.
- Existing outcome evidence: prior dispositions, actual minutes, wrong-source findings, data-quality issue refs, and review statuses when used to prioritize or measure later work.

Do not describe this as broad database, provider, document, payment, or customer-message access. If the cited source only shows a narrow packet or tool port, the overlay should say the agent reads that packet/port result, not the whole system.

## 5. Agent may draft/recommend/rank/record

Allowed verbs must stay attached to reviewable artifacts:

| Allowed automation verb | Concrete artifact | Reviewer/value boundary |
| --- | --- | --- |
| Summarize source evidence | Evidence summary, staff evaluation packet, manager brief source facts, data-quality issue cluster. | Proves why the recommendation exists; reviewer still decides sensitive action. |
| Rank work | Manager daily brief action queue, front-desk checkout/retention/data-quality queue, regional exception queue when implemented. | Ranking saves scanning time; it does not change schedules, provider records, or priorities in source systems. |
| Draft internal task | Staff task draft, Hermes task/schedule draft, integration-failure ticket. | Task creation/draft is not approval for the underlying live change. |
| Draft customer-message text | Confirmation/missing-info/daily-update/retention/public-response draft and fact checklist. | Customer-message reviewer or approved sender must approve recipient, channel, timing, body, contact permission, and suppression. |
| Recommend review gate | `RequestHumanReview`, approval gate, review reason, blocked-action list. | Names the stop; it is not approval. |
| Validate and reject unsafe output | Validation result, blocked action reason, requested side-effect rejection, `outcome_persisted: false` where present. | Proves attempted unsafe side effect did not become persisted outcome. |
| Record reviewed outcome | Outcome record with disposition, actor/reviewer/persona, actual minutes, source refs, issue refs, feedback, correlation id. | Measures reviewed labor value; it still does not prove the agent performed the live action. |

Use `draft-only`, `internal-task-only`, `manager-approval-required`, and `never-automate` language when citing `domain::policy::automation::Level`. Do not upgrade a workflow from draft/review to live action in Markdown.

## 6. Agent must not do directly

| Blocked direct action | Why blocked | Correct safe path |
| --- | --- | --- |
| Send customer/member/public messages | Customer trust, consent, channel, timing, sensitive fact, and brand-voice risks. | Draft message plus `CustomerMessageApproval`; approved sender records status/disposition. |
| Confirm, reject, cancel, check in/out, release waitlist, allocate room/capacity, or complete care/task status | Provider/PMS and live operations remain source-of-record/human actions. | Staff/front-desk or manager review packet; provider action happens only through separately approved path. |
| Change staff schedules, labor assignments, payroll, or personnel decisions | Staffing and personnel decisions require manager/ops authority and source-system control. | Manager daily brief recommendation; manager/general manager records outcome and actual minutes. |
| Capture, retry, void, refund, waive, discount, forfeit, write off, or change rates/taxes/fees | Money movement requires payment/accounting controls and audit trails. | Payment/accounting review packet, manager approval when policy exception, payment outcome evidence. |
| Final-approve vaccine, medical, behavior, group-play, incident, legal, safety, or policy eligibility | Pet safety and compliance decisions need trained/authorized reviewers. | Medical/vaccine qualified staff or behavior/daycare lead review; approval record and source refs. |
| Mutate provider/PMS/source records, merge identities, delete data, hide source issues, or materially edit audit history | Source authority, data quality, and audit integrity risks. | Source-hygiene or provider-portal review task; IT/security and product/ops approve any write-back mode. |
| Expand tool-port scope, secrets, logging, retry behavior, or live external side effects | Security and production-change risk. | IT/security plus product/ops owner approval; runtime tests and audit evidence before live use. |

If any blocked action appears in an agent output, the safe behavior is to reject/fail closed, preserve the blocked action reason, route to the reviewer, and record the rejection or non-persistence evidence.

## 7. Required human reviewer roles and approval conditions

| Role | Usually approves | Does not approve | Entity/action examples and audit evidence |
| --- | --- | --- | --- |
| Staff/front-desk lead | Routine intake completeness, source-backed queue work, handoff quality, checkout/retention/internal task readiness when no escalated gate exists. | Manager exceptions, money movement, medical/vaccine/behavior approval, provider write-back permission, IT/security scope. | Booking triage missing-info packet; checkout exception task; retention queue review. Evidence: staff actor, packet/action id, source refs, disposition, actual minutes, draft/task id. |
| Manager/general manager | Capacity/staffing/policy exceptions, hard stops, incidents/complaints, suppression/escalation, source-quality visibility, customer-trust decisions, data-quality exception prioritization. | Medical/vaccine validity unless trained; payment processing unless assigned; integration/security authority; individual approved-sender duties unless local role allows. | Manager daily brief demand/staffing action, data-quality issue suppression/deferral, incident escalation. Evidence: `ManagerApproval`, action id, blocked schedule/provider/payment/message actions, manager persona, disposition, before/actual minutes. |
| Medical/vaccine qualified staff | Vaccine proof, medical/care-document ambiguity, medication/special-care readiness, licensed-vet proof checks, eligibility effects tied to medical evidence. | Payment/refund decisions, provider integration permissions, general marketing/customer-message approval outside medical wording, staff scheduling. | Booking triage vaccine-pending packet, document/OCR extraction candidate, medical care-note ambiguity. Evidence: document ref/OCR result/source ref, `MedicalDocumentReview`, reviewer status, decision reason, no auto-approval from OCR confidence. |
| Behavior/daycare lead | Temperament evidence, group-play eligibility, behavior flags, day play/day boarding safety, behavior-related incident implications. | Payment/provider write-back, broad policy changes, customer-message send authority unless also approved sender. | Daycare group-play candidate with bite/stress/intro-assessment evidence. Evidence: `BehaviorReview`, pet/temperament source facts, eligibility/ineligibility reason, reviewer/actor, disposition. |
| Customer-message reviewer / approved sender | Final recipient, channel, timing, body, contact preference/consent, included/omitted facts, suppression/escalation, send/no-send decision. | Provider/PMS mutation, payment movement, medical/behavior approval, schedule/capacity changes. | Daily update/Pawgress draft, retention/grooming rebooking outreach, booking missing-info message. Evidence: draft id/body ref, channel/recipient/consent, `CustomerMessageApproval`, reviewer, status, suppression/send disposition. |
| Payment/accounting | Deposits, refunds, waivers, discounts, balances, provider receipts, amount mismatch, duplicate/provider ambiguity, accounting task completion. | Medical/behavior/schedule decisions, broad customer-message approval except payment wording, provider integration scope. | Booking deposit exception, refund/discount implication from retention or complaint, checkout payment ambiguity. Evidence: `RefundOrDepositException`, payment ref/provider receipt, approval status, actor, disposition, no raw secret exposure. |
| IT/security | Integration scope, secrets, logging, rate limits, tool-port failure modes, provider write-back security, live external side-effect controls, audit/log retention. | Business policy approval, customer-message/payment/medical/behavior operational decisions. | Provider portal write-back proposal, payment gateway port behavior, Hermes automation hook, external failure retry policy. Evidence: tool-port scope, allowed/blocked side effects, security review, correlation id, error/failure record. |
| Product/ops owner | Whether a workflow or port may exist, which automation level is allowed, which reviewer lane owns a new action family, and what evidence/test coverage is required before rollout. | Individual live-action approval without the operational reviewer; secrets/security implementation approval without IT/security. | New regional exception loop, deterministic autosend proposal, provider write-back product decision. Evidence: approved policy/source/test change, automation level, owner decision, rollout constraints, gaps closed. |

## 8. Required source evidence before a recommendation

| Recommendation or draft | Required source evidence first | If missing/stale/ambiguous |
| --- | --- | --- |
| Booking readiness or missing-info draft | Intake/reservation/customer/pet/policy/deposit/vaccine source refs; deterministic rule evaluation; approval gates and blocked actions. | Route to staff/manager/medical/payment review, mark missing source, do not confirm/reject/cancel or send. |
| Manager daily brief action | Operating day/location/persona, service-demand facts, checkout/retention/source-quality facts, source refs, labor estimate, review gates. | Keep item visible as data-quality or manager review; do not hide source issue or change schedule/payment/provider/customer state. |
| Daily update / customer outreach draft | Staff note/care/reservation facts, contact/channel/consent status, included/omitted facts, sensitive-fact flags, source refs. | Suppress or route to message/medical/behavior/manager review; do not send. |
| Vaccine/medical/document review | Document metadata, OCR/extraction candidate, expected requirement/policy snapshot, pet/reservation context, prior verified records, source refs. | Route to qualified reviewer; OCR confidence alone is not approval. |
| Behavior/daycare eligibility | Pet temperament/group-play observations, behavior flags, incident/care context, service line, policy gate, source refs. | Route to behavior/daycare lead; do not approve group play or care eligibility. |
| Payment/deposit/refund/discount packet | Payment/deposit/balance status, provider receipt/ref, policy snapshot, amount/duplicate ambiguity flags, source refs. | Route to payment/accounting and manager if exception; do not move money or promise payment outcome. |
| Tool-port/provider/source-data task | Scoped port result, provider lookup evidence, external failure resource/id, source refs/provenance, requested side effects. | Route to staff/IT/security/product/ops; fail closed for unsupported or blocked side effects. |

## 9. Outcome/audit record proving safe use and value measurement

A safe overlay must separate the five proof layers below. Sibling pages can copy this table and replace examples with the entity/action family they cover.

| Proof layer | Example field or record | What it proves | What it does not prove |
| --- | --- | --- | --- |
| Source evidence collected | `RecordRef`, `Provenance`, source refs, issue refs, policy snapshot, document/OCR ref, payment ref. | The recommendation was grounded in traceable source facts. | Approval, completion, live side effect, or value realized. |
| Draft/recommendation produced | `AgentPromptPacket`, `StaffEvaluationPacket`, `BriefAction`, draft id/body ref, internal task id, safe action list, output schema, validation result. | Reviewable work product was prepared and constrained. | Customer/provider/payment/schedule action happened. |
| Human approval/decision | Review gate, approval status, reviewer/actor/persona, timestamp, decision reason, approval record target/status. | The named role reviewed the target and accepted/rejected/deferred/suppressed/escalated it. | Permission for unrelated future actions or broader automation authority. |
| Live action elsewhere, if any | Source-system action id, send stub/status, payment/provider receipt, staff-entered outcome, approved runtime log. | A separate approved path or human/system performed the downstream action. | That the agent had direct authority; this page does not create that authority. |
| Durable outcome/value record | Disposition, feedback, actual minutes, before/after minutes, minutes saved/avoided, actor, owner persona, source refs, data-quality findings, correlation id, reporting group. | What reviewed work happened and how labor value was measured. | Guaranteed ROI, revenue lift, or future savings without more records. |

Minimum outcome/audit evidence fields for this repo’s overlays:

- source refs/provenance and data-quality issue refs;
- context packet id or workflow event id, action id, draft id or task id, and correlation id;
- review gates / required reviews and blocked-action reasons;
- requested side effects and validation result;
- `live_side_effects_allowed: false`, `outcome_persisted: false`, or equivalent rejection proof where present;
- approval status, reviewer/actor/persona, timestamp, and decision reason;
- disposition, feedback, actual minutes, before/after minutes, minutes saved or avoided, wrong-source findings, and reporting dimensions.

For labor value, use measured phrasing: “this outcome record captured 12 actual minutes against 45 before minutes” or “the loop measures minutes saved after review.” Do not say “the agent saved ROI” or “automation reduced labor cost” unless outcome records across the relevant population support that claim. Estimates can prioritize queues; actual minutes and dispositions measure value.

## 10. Entity/action examples: evidence vs draft vs approval vs outcome

| Example | Source evidence | Draft/recommendation | Human approval | Outcome/audit proof |
| --- | --- | --- | --- | --- |
| Vaccine-pending booking triage | Reservation/pet/policy/vaccine evidence refs and deterministic failure code. | Staff evaluation packet and clearer-proof request draft. | Medical/vaccine qualified staff reviews proof; customer-message reviewer approves owner-facing wording. | Approval status, reviewer, document/source refs, blocked confirmation/send/provider/payment actions, staff disposition and minutes. |
| Demand-versus-staffing manager brief | Operating-day service-demand facts, labor/schedule source facts, source refs and data-quality flags. | Ranked manager action with labor estimate. | Manager/general manager reviews schedule/capacity/staffing implication. | `OutcomeRecord` / `ManagerDailyBriefOutcomeRecord`: action id, manager actor/persona, before/actual minutes, source refs, correlation id, disposition. |
| Retention or grooming follow-up draft | Completed stay/grooming history, contact permission, consent/channel, source refs, suppression reasons. | Customer-message draft and follow-up queue item. | Customer-message reviewer/approved sender; manager if discount/complaint/policy exception is involved. | Draft id/body ref, approval status, send/suppression disposition, no payment/discount/schedule/provider mutation, minutes saved in queue/draft work. |
| Source-data duplicate or stale vaccine issue | Source refs/provenance, duplicate candidates or stale evidence, data-quality issue severity/kind. | Internal cleanup task or data-quality hygiene draft. | Staff/front-desk lead for routine cleanup; manager for high-impact source ambiguity; medical reviewer for vaccine validity; IT/product for provider write-back mode. | Issue refs, blocked provider mutation/source hiding reasons, actor/reviewer, disposition, actual minutes, wrong-source or corrected-source finding. |
| Payment/deposit exception | Payment/deposit status, provider receipt/ref, policy snapshot, amount/duplicate ambiguity. | Accounting review packet and customer-message draft if needed. | Payment/accounting, manager for exception, customer-message reviewer for any owner text. | Payment ref, `RefundOrDepositException`, reviewer, blocked money-movement reason until approved payment path, disposition and minutes. |
| Tool-port external failure | Scoped port request/result, external failure resource, correlation id, source refs. | Integration-failure task or retry review packet. | IT/security for retry/scope/logging/secrets; product/ops for capability expansion. | Failure record/task id, blocked side effect, security/product decision, disposition, no silent unsafe retry. |

## 11. Source/Rustdoc/test evidence links

Use these local anchors when writing sibling overlays. Prefer source/Rustdoc/tests over copied prose when behavior matters.

- `../../../app/src/agents.rs` for `AgentSpec`, `WorkflowAgent`, `AgentPromptPacket`, baseline allowed tools, forbidden actions, and default review gates.
- `../../../domain/src/policy.rs` for `ReviewGate`, denial reasons, play-safety gates, and `domain::policy::automation::Level`.
- `../../../domain/src/workflow.rs` for `PolicyContext`, `AllowedAction`, `Result`, `Status`, `RecommendedAction`, workflow events, risk flags, and verification notes.
- `../../../domain/src/source.rs` for `RecordRef`, `Provenance`, source systems, reservation snapshots, assumptions, and data-quality issue promotion.
- `../../../app/src/booking_triage.rs` for deterministic booking evaluations, `StaffEvaluationPacket`, `ApprovalGate`, `SafeAgentAction`, and `BlockedAction`.
- `../../../app/src/manager_daily_brief.rs` for source-grounded manager actions, safe/blocked actions, labor estimates, and app `OutcomeRecord`.
- `../../../storage/src/operations.rs` for `ManagerDailyBriefOutcomeRecord`, `DataQualityHygieneOutcomeRecord`, before/actual minutes, actor/persona, source refs, correlation id, and saved-minute calculations.
- `../../design/labor-cost-reduction-crosswalk.md` and `../../design/manager-daily-brief-measurable-labor-loop.md` for labor-loop measurement wording and no-live-side-effect guardrails.
- `../source-evidence-map.md`, `../review-boundaries-matrix.md`, and `../../design/entity-atlas-review-safety-boundaries.md` for shared safety navigation and citation guidance.

## 12. Open gaps or owner decisions

| Gap or owner decision | Why it matters | Safest current behavior | Evidence needed to close |
| --- | --- | --- | --- |
| Production live customer-send, provider-write, payment-movement, schedule-change, and deterministic autosend paths are not proven by this overlay. | Sibling docs could accidentally turn draft/review contracts into execution claims. | Say draft/review/route/record only; keep live side effects blocked unless a later source/test/approval record proves the path. | App/source/runtime contract, tests, approval records, security/product/ops signoff, and audit/outcome fields. |
| `source-inventory.md` parent artifact is missing. | Overlay writers lack a single inventory page for all entity/action families. | Use `../source-evidence-map.md`, `../review-boundaries-matrix.md`, and current source paths; mark gaps rather than invent. | A source-inventory overlay with current Rustdoc/test/source anchors. |
| Several reviewer roles are operational lanes, not all are currently represented as first-class Rust enum variants. | Docs need non-coder routing without pretending every role has a compiled type. | Map role language to existing `ReviewGate`, `PolicyContext`, actor/persona/outcome fields, and workflow matrices; do not claim missing types exist. | Domain/app type or documented approval record if product wants first-class role modeling. |
| Labor value beyond manager daily brief/data-quality outcomes needs more records. | Product copy could overstate portfolio ROI from estimates. | Use measured-minutes language only when outcome records exist; otherwise say “intended metric” or “future source/read-model need.” | Outcome data by location/day/entity/action with source refs, before/actual minutes, reviewer/actor, disposition, and reporting group. |
| Product/ops and IT/security approval for wider tool-port capabilities remains an owner decision. | Tool ports describe boundaries but not production permission to write external systems. | Keep ports as read/check/draft/failure surfaces until explicit approval and tests exist. | Approved policy, security review, app/runtime implementation, integration tests, logs/audit records. |

## Final reviewer checklist

Before a sibling overlay reuses this page, verify:

- Does the entity/action name a concrete reviewer role, not just “human”?
- Does it say what that role approves and what the role does not approve?
- Are source evidence, draft/recommendation, human approval, live action elsewhere, and durable outcome record separate?
- Are blocked actions precise enough to prove what automation did not do?
- Does the outcome proof include source refs/provenance, review gates, blocked action reasons, draft/task/action ids, approval status, reviewer/actor, disposition, actual minutes, minutes saved/avoided, data-quality findings, and `live_side_effects_allowed` where present?
- Are labor-value claims phrased as measured outcome evidence, not unsupported ROI?
- Are all source links local/current and treated as evidence rather than broad implementation inventory?
