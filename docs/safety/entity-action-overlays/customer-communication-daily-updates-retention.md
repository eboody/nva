# Customer communication, daily updates, retention, and manager brief safety overlay

This overlay helps care staff, front-desk leads, customer-message reviewers, grooming/retention operators, and general managers reduce blank-screen drafting, repeated queue scanning, morning reconciliation, and wrong-source rework for customer communication actions by showing what source facts automation may read, what it may draft, rank, recommend, or record, what a human must approve, and which outcome or audit record proves safe use.

Safe outcome in one sentence: automation may prepare source-backed message drafts, daily-update previews, retention review packets, internal task drafts, manager brief rankings, review requests, and outcome records, but customer sends, provider/PMS writes, schedule/capacity changes, money movement, medical/vaccine/behavior decisions, destructive cleanup, and policy changes stay human-reviewed or blocked.

This page summarizes source/Rustdoc/test evidence for non-coders. If this page disagrees with source, Rustdoc, or tests, source/Rustdoc/tests win.

## 1. Plain-English entity/action definition and labor-cost problem

Entity/action family:

- Message drafts and customer-message approval packets: draft email/SMS/portal/body artifacts and approval records for later human review.
- Daily Updates / Pawgress drafts: customer-safe stay/daycare update previews built from reviewed care notes.
- Retention and grooming rebooking follow-up: source-grounded follow-up candidates and draft-only outreach after completed stays, checkout, or grooming cadence evidence.
- Manager Daily Brief recommendations: ranked internal actions for a location and operating day.
- Internal tasks and review notes: internal handoff/task drafts that route evidence gaps or blocked actions to the right reviewer.

Manual work or error cost reduced:

- Care staff and front desk avoid rewriting routine pet-parent updates from scratch.
- Front-desk leads and grooming/retention operators avoid manually scanning completed stays, contact permission, service history, and follow-up candidates.
- Managers avoid morning dashboard reconciliation across demand, checkout exceptions, retention opportunities, and source data-quality warnings.
- Reviewers avoid re-reading every source record because packets list required evidence, review gates, and blocked actions.
- Operators reduce wrong-pet, wrong-reservation, unsupported-contact, hidden-source-issue, and premature-send errors.

Safe outcome:

- A reviewed draft, ranked queue, source-backed recommendation, internal review task, approval request, blocked-action reason, or outcome record. Not a live send or live operational mutation.

## 2. Workflows/contracts featuring it and adjacent entities

| Workflow or contract | Entity/action role | Adjacent entities | Source path or doc evidence |
| --- | --- | --- | --- |
| Daily Updates / Pawgress | Builds a local preview packet, customer-message draft, included/omitted fact list, internal flags, approval request, blocked send stub, and audit log from reviewed care notes. | Care note, pet, owner/customer, reservation/update subject, workflow event, message draft, approval record, audit event. | `../../../app/src/daily_update.rs`; `../../../app/tests/daily_care_update_mvp.rs`; `../../workflows/operator/daily-updates-pawgress-drafts.md`. |
| CRM retention and grooming rebooking | Builds a staff review packet for eligible or suppressed follow-up opportunities; can include grooming rebook opportunities when source evidence and contact permission support a draft. | Checkout packet, customer, reservation, grooming/service-history evidence, contact permission, source refs, review gates, outcome record. | `../../../app/src/crm_retention.rs`; `../../../app/tests/crm_retention_workflow_contracts.rs`; `../../workflows/operator/grooming-rebooking-retention.md`. |
| Checkout completion feeding retention | Supplies staff-verified or unresolved checkout context before retention follow-up can be reviewed. | Reservation, source checkout/PMS status, staff handoff, belongings, care summary, departure notes, audit-event drafts. | `../../../app/src/checkout_completion.rs`; `../../../app/tests/checkout_completion_workflow_contracts.rs`; `../../workflows/operator/checkout-completion.md`. |
| Manager Daily Brief | Ranks operating-day manager/front-desk actions and records labor feedback without changing schedules, provider records, customer messages, money, or source issues. | Location, operating day, service-demand facts, checkout packets, retention packets, source facts, labor estimate, outcome record. | `../../../app/src/manager_daily_brief.rs`; `../../../app/tests/manager_daily_brief_workflow_contracts.rs`; `../../workflows/operator/manager-daily-brief.md`. |
| Agent specs and prompt packets | Defines workflow-specific allowed read/draft tools, forbidden actions, review gates, policy instructions, and source event/input context. | Agent spec, workflow event, policy instructions, output schema, typed app input. | `../../../app/src/agents.rs` (`AgentSpec`, `WorkflowAgent`, `AgentPromptPacket`, `baseline_agent_specs`). |
| Message and draft tool ports | Exposes draft-message and draft-reservation-update ports as reviewable artifacts rather than send/write authority. | Message channel, recipient, body, review policy, draft id, draft status, reservation update draft. | `../../../app/src/tools.rs` (`messaging`, `draft_update`); `../../../domain/src/message.rs`. |
| Shared policy and workflow vocabulary | Names review gates, allowed actions, workflow events, prompt/evidence context, source refs, and provenance. | Policy snapshot, review gate, allowed action, workflow event, source ref, provenance. | `../../../domain/src/policy.rs`; `../../../domain/src/workflow.rs`; `../../../domain/src/source.rs`. |
| Outcome/labor evidence | Stores or shapes reviewed outcome evidence where supported. Manager brief has durable storage projection; retention has app outcome record; Daily Updates and checkout currently identify outcome gaps. | Actor/reviewer, disposition, before/after or actual minutes, source refs, reporting group, blocked action reasons. | `../../../storage/src/operations.rs`; `../../../app/src/manager_daily_brief.rs`; `../../../app/src/crm_retention.rs`. |

## 3. Who/what is authoritative

| Fact or decision | Authoritative source | What it can prove | What it does not prove |
| --- | --- | --- | --- |
| Care-note and daily-update source facts | Staff/source-system note evidence normalized as `domain::entities::CareNote`, plus `app::daily_update::MvpPreviewRequest.notes`. | What care facts may be summarized, included, omitted, or flagged. | Customer send approval, media publication, medical/behavior decision, or care-task completion. |
| Message lifecycle vocabulary | `domain/src/message.rs` distinguishes inbound, outbound draft, queued, sent, approval, failed, suppressed, and channel values. | The state vocabulary for draft/approval/delivery evidence. | Permission to send a message or bypass consent/review. |
| Prompt packet and agent spec | `app/src/agents.rs` packages workflow name, goal, event, typed input, policy instructions, output schema, allowed tools, forbidden actions, and review gates. | What evidence and policy instructions the agent may use for one workflow run. | Live booking, charge, customer-message, schedule, provider/PMS, or policy authority. |
| Daily update preview | `app/src/daily_update.rs` owns `build_mvp_preview`, `CustomerMessageDraft`, `ReviewDisposition`, `InternalFlag`, `SendStub`, approval record, and audit log. | A customer-safe draft preview was prepared and blocked behind approval. | Production Pawgress delivery, live messaging integration, media publication, or verified labor savings. |
| Retention/grooming follow-up packet | `app/src/crm_retention.rs` owns eligibility, contact permission, safe actions, blocked actions, review gates, source refs, and app outcome shape. | A follow-up candidate is source-grounded, eligible/suppressed, and routed to customer-message or manager review. | Booking creation, autonomous outreach, discount/payment action, calendar mutation, or dedicated durable grooming-retention storage. |
| Checkout completion packet | `app/src/checkout_completion.rs` owns source checkout observation, staff handoff evidence, checkout classification, safe actions, blocked actions, and audit-event drafts. | Whether checkout context is safe enough to feed a retention draft review or needs handoff/source reconciliation. | Final checkout execution, capacity release, billing closeout, provider/PMS write, or customer send. |
| Manager brief packet and outcome | `app/src/manager_daily_brief.rs` and `storage/src/operations.rs` own ranked actions, safe/blocked actions, before/after labor estimates, and durable manager brief outcome/labor projection. | Reviewed manager/front-desk work, actual minutes, source refs, and reporting dimensions where recorded. | Production ROI, staffing/schedule changes, source cleanup, customer sends, or money movement. |
| Human approval | Customer-message reviewer/approved sender, front-desk lead, manager/GM, medical/vaccine qualified staff, behavior/daycare lead, payment/accounting, IT/security, and product/ops owner by condition. | Permission for the named reviewed downstream step. | Authority for unrelated live actions or broad automation expansion. |

## 4. Agent may read

The agent/app workflow may read only scoped, source-backed facts carried by a workflow packet, source ref, provenance record, policy snapshot, or approved read/draft port. It may not browse broad live systems just because a related entity appears in prose.

Allowed read/inspect inputs:

- Daily update context: triggering `domain::workflow::Event`, pet name, owner display name, policy snapshot id, and reviewed `CareNote` list in `MvpPreviewRequest` (`../../../app/src/daily_update.rs`).
- Daily update draft state: `CustomerMessageDraft`, included facts, omitted facts, internal flags, `ReviewDisposition`, approval record, `SendStub`, and audit log, only as review artifacts (`../../../app/src/daily_update.rs`).
- Message vocabulary: `domain::message::{Direction, Channel, Status, BodyRef}` and app tool `messaging::draft::Request`/`Result`; this is draft/approval state, not send permission (`../../../domain/src/message.rs`; `../../../app/src/tools.rs`).
- Retention context: reservation id, customer id, checkout completion packet, contact permission, allowed/preferred channels, marketing/transactional consent status, opportunities, source refs, and opportunity provenance (`../../../app/src/crm_retention.rs`).
- Grooming rebooking context: grooming cadence/history/service evidence only after it is promoted into source-grounded retention/grooming domain values; do not use model memory or raw calendar/provider names as authority (`../../workflows/operator/grooming-rebooking-retention.md`; `../../../domain/src/grooming/mod.rs`).
- Checkout prerequisite context: source reservation status, `source::Provenance`, staff handoff actor/timestamp, belongings status, care summary, and departure-note review (`../../../app/src/checkout_completion.rs`).
- Manager brief context: location, operating day, prepared-for persona, demand threshold, service-demand facts, checkout packets, retention packets, source facts, and prior outcome/source refs (`../../../app/src/manager_daily_brief.rs`).
- Agent prompt context: workflow name, source event, typed input, policy instructions, and expected output schema in `AgentPromptPacket` (`../../../app/src/agents.rs`).
- Shared evidence context: `domain::source::{RecordRef, Provenance}` and `domain::policy::ReviewGate` (`../../../domain/src/source.rs`; `../../../domain/src/policy.rs`).

Scope limits:

- Scope daily updates to the triggering pet/customer/reservation/update event and reviewed care notes.
- Scope retention to a reservation/customer and its checkout/contact/opportunity evidence.
- Scope manager brief to the requested location and operating day; the app filters scoped checkout and retention packets before ranking actions.
- Scope message drafting to the named recipient, channel, body, and review policy in the draft request.

## 5. Agent may draft/recommend/rank/record

| Allowed automation verb | Concrete artifact | Conditions and evidence |
| --- | --- | --- |
| Summarize care/source facts | Daily update preview or manager brief source-fact summary. | Facts must come from reviewed notes/source refs; omitted/sensitive facts stay visible to reviewers. |
| Draft a customer-message body for approval | `CustomerMessageDraft` or `tools::messaging::draft::Request`/`Result`. | Draft-only or manager/customer-message-review required; no send. Daily update `ReviewDisposition::allows_live_send()` is false and `SendStub` is `ApprovalRequiredStub`. |
| List included and omitted facts | `IncludedFact`, `OmittedFact`, `InternalFlag`. | Customer-visible claims must trace to care-note ids; internal/sensitive/medical/behavior/policy gaps are flagged or omitted for review. |
| Recommend/rank a retention or grooming follow-up review item | `crm_retention::StaffReviewPacket` and `Packet`. | Requires staff-verified checkout, source-grounded opportunity, contact-permission source refs, allowed preferred channel, and review gates. |
| Draft retention follow-up for review | `crm_retention::SafeAgentAction::DraftCustomerFollowUpForReview`. | Only when `FollowUpEligibility::Eligible`; still requires `CustomerMessageApproval`. |
| Create internal staff review task | Retention/checkout/manager brief safe actions or `tools::draft_update`/task-draft equivalents. | Internal-task-only; routes missing evidence, suppressed outreach, checkout handoff issues, source ambiguity, or data-quality issues to reviewers. |
| Rank manager/front-desk queue | `manager_daily_brief::BriefAction` and packet `actions`. | Actions must be source-grounded; manager brief ranks demand-staffing review, checkout exception, retention follow-up, and data-quality investigation without live side effects. |
| Recommend review gate or blocked-action route | `policy::ReviewGate`, `required_review_gates`, blocked actions, and side-effect rejection reason. | Recommendation is a route to staff/manager/customer-message/payment/medical/IT/product review, not approval. |
| Record reviewed outcome/value | `crm_retention::OutcomeRecord`, `manager_daily_brief::OutcomeRecord`, and storage manager daily brief outcome records. | Human disposition, actor/reviewer, source refs/provenance, before/actual minutes, actual minutes saved, or feedback must exist. |
| Validate output or reject unsafe side effect | `WorkflowAgent::validate_output`, `requested_side_effect_rejection_reason`, packet blocked actions. | Unsafe output is rejected or routed to review; it is not converted into live action. |

## 6. Agent must not do directly

| Blocked direct action | Why blocked | Correct safe path |
| --- | --- | --- |
| Send customer messages, Pawgress updates, retention outreach, reminders, public replies, or health/incident concern copy. | Customer trust, consent, channel preference, privacy/media, complaint/incident, and review gates. | Draft message only; customer-message reviewer or approved sender reviews recipient, body, channel, timing, consent, and suppression state. |
| Mutate provider/PMS/Gingr/source records, create/delete/merge records, hide source issues, or edit audit-material source facts. | Source systems are evidence and systems of record; destructive cleanup can erase audit truth. | Create internal task or source-data-quality review item; manager, ops owner, IT/security, or approved system handles write path. |
| Confirm, cancel, check in/out, release capacity, assign rooms/slots/groomers, change calendar or staff schedule, or promise availability. | Operational capacity, staffing, animal safety, and provider authority require human/system-of-record execution. | Rank/recommend the review item; front-desk lead, manager/GM, or approved system reviews and executes outside the agent. |
| Move money: capture/retry/void/refund, discount, waive/forfeit deposit, apply offers/packages/credits, change rate/tax/fee, or collect payment. | Accounting, customer money, and policy exception risk. | Route to payment/accounting reviewer and record blocked/action-needed evidence. |
| Approve vaccine, medical, behavior, temperament, group-play, incident, legal, safety, or local-SOP decisions. | Pet safety and regulated/local policy require qualified reviewers. | Flag and route to medical/vaccine qualified staff, behavior/daycare lead, manager, or legal/safety reviewer. |
| Publish or attach media/photo/video, claim a photo exists, or reveal internal-only notes/staff debate/provider ids. | Privacy, wrong-pet/wrong-media, sensitive internal context, and customer trust. | Require approved media refs and privacy/customer-message review; otherwise omit/flag. |
| Change automation policy, expand tools, use secrets, or perform live external side effects. | Product/security boundary and credential risk. | Product/ops owner and IT/security review integration scope, secrets, logs, and effect ledgers before any implementation. |
| Treat estimated minutes or draft creation as realized labor savings. | Value must be measured after reviewed disposition. | Record outcome/feedback/actual minutes or mark the value gap. |

## 7. Required human reviewer role(s) and approval condition

| Role | Usually approves for this family | Does not approve |
| --- | --- | --- |
| Care staff or front-desk agent | Routine care-note factuality, whether a routine draft reflects the observed note, and whether an internal task is complete enough to route. | Sensitive customer sends, medical/behavior interpretation, provider writes, money movement, policy exceptions. |
| Front-desk lead | Checkout handoff quality, unresolved handoff queue routing, retention queue triage, and routine source-backed internal review tasks. | Final customer-send authority unless assigned, payments/refunds, medical/behavior approval, broad source cleanup, automation policy. |
| Customer-message reviewer or approved sender | Final recipient, channel, timing, body, tone, consent/contact preference, suppression, and approved send path for daily updates or retention drafts. Maps to `CustomerMessageApproval`. | Provider/PMS writes, payment movement, medical/behavior/vaccine approval, staffing or capacity changes. |
| Manager/general manager | Manager Daily Brief actions, source/data-quality issue visibility, suppression/escalation, checkout/retention exceptions, staffing/capacity decisions, and policy/customer-trust exceptions. Maps to `ManagerApproval` where present. | Medical/vaccine validity unless qualified, payment processing unless assigned, IT/security integration scope, customer-message body unless also approved sender. |
| Grooming manager or retention operator | Grooming cadence/rebooking context, service-history interpretation, follow-up queue prioritization, and whether a grooming rebook opportunity is operationally appropriate. | Live booking/calendar mutation, discounts/payment, final customer sends, DNC/consent override. |
| Medical/vaccine qualified staff | Medical, medication, vaccine, health-document, or care-readiness ambiguity referenced in an update or follow-up. Maps to medical review gates where applicable. | General pricing, provider integration, schedule/capacity, or non-medical customer-message approval. |
| Behavior/daycare lead | Behavior, group-play, temperament, incident-care implications, and whether sensitive behavior language must be suppressed or escalated. | Payment, provider/PMS write-back, broad policy changes, unrelated customer sends. |
| Payment/accounting reviewer | Refund/discount/deposit/balance/payment language and any money movement raised by a message or retention offer. | Medical/behavior/schedule decisions and general message tone outside payment wording. |
| IT/security | Tool-port scope, secrets, logging, rate limits, external provider write-back controls, effect ledgers, and failure modes. | Business policy and individual customer/payment/message decisions. |
| Product/ops owner | Whether a workflow, draft port, outcome record, or future auto-send/write capability is allowed to exist and under what policy. | Individual live-action approval without the proper operational reviewer. |

## 8. Required source evidence before a recommendation

| Recommendation or draft | Required source evidence first | If missing/stale/ambiguous |
| --- | --- | --- |
| Daily update / Pawgress draft | Triggering `DailyNoteCreated` or `DailyUpdateNeeded` event; pet/owner identity; policy snapshot id; at least one reviewed care note; allowed summarize/draft action; care-note ids for included/omitted facts. | Stop or route to staff/manager review; do not invent routine care facts, health reassurance, meals, bathroom events, medications, photos, or staff actions. |
| Customer-message draft | Named recipient, channel, body, review policy, source facts, consent/contact evidence where relevant, and approval gate. | Draft only or suppress; do not send or queue. Route to customer-message reviewer/approved sender. |
| Retention/grooming follow-up draft | Staff-verified checkout packet; source-grounded retention/grooming opportunity; opportunity provenance; customer/reservation ids; contact permission with source refs; granted consent; preferred channel allowed; no opt-out. | Ineligible/suppressed packet with `ManagerApproval`; do not draft outreach if consent/source/channel evidence is absent or opted out. |
| Checkout-completion-derived retention recommendation | Source reservation status provenance and staff handoff evidence showing belongings/care summary/departure notes status. | Route to staff handoff review or source-status reconciliation; do not suggest checked-out status when source/staff evidence is incomplete. |
| Manager Daily Brief ranked action | Location/operating day scope; service-demand facts or scoped checkout/retention packets; source refs on every source fact; demand threshold; review gates for data-quality or exceptions. | Hide nothing; include data-quality issue or route to manager review; do not rank model-invented actions. |
| Internal task draft | The blocked action, missing evidence, source refs/provenance, reviewer role, and desired review outcome. | Create no live side effect; fail closed and route to the reviewer lane. |
| Outcome/value record | Human actor/reviewer, disposition, source refs/provenance, outcome code, timestamp, before/after or actual minutes where supported. | Do not claim realized savings; mark as planned/future or evidence gap. |

## 9. Outcome/audit record proving safe use and value measurement

Source evidence, draft creation, human approval, downstream live action, and durable outcome records are different proof layers. Drafts and packets prove work was prepared; they do not prove a message was sent, a booking changed, money moved, or value was realized.

| Proof needed | Example field or record | What it proves | What it does not prove |
| --- | --- | --- | --- |
| Source evidence | `domain::source::RecordRef`, `domain::source::Provenance`, care-note ids, policy snapshot id, checkout source status, contact permission source refs. | The draft/recommendation was grounded in traceable facts. | Approval, completion, send, provider write, or realized value. |
| Draft/recommendation | `CustomerMessageDraft`, `tools::messaging::draft::Result.draft_id`, `crm_retention::StaffReviewPacket`, `manager_daily_brief::BriefAction`, `checkout_completion::AuditEventDraft`. | A reviewable work product was prepared. | That customer/provider/payment/schedule action happened. |
| Review gate / approval | `domain::policy::ReviewGate`, `entities::approval::Record`, `required_review_gates`, customer-message or manager approval lifecycle. | A sensitive step is waiting for or received named review. | Authority for unrelated steps or automatic execution. |
| Blocked action proof | `SendStub::is_blocked_until_human_approval`, `BlockedAction` lists, `requested_side_effect_rejection_reason`, audit action such as `message.send.blocked_stub`. | The workflow refused or blocked live side effects. | That the blocked downstream action was safely completed elsewhere. |
| Retention outcome/value | `crm_retention::OutcomeRecord` fields: reservation id, customer id, recorded_by, recorded_at, `FollowUpOutcome`, source provenance, evidence; `records_staff_evidence_only()`. | Staff recorded a follow-up disposition such as booked next stay, interested, not interested, no response, or suppressed by staff. | Dedicated durable production persistence or autonomous outreach. |
| Manager brief outcome/value | `manager_daily_brief::OutcomeRecord` and `storage::operations::ManagerDailyBriefOutcomeRecord`: action id, actor/persona, outcome, before/actual minutes, actual minutes saved, source refs, reporting group. | Reviewed manager/front-desk work and measured labor value after disposition. | Production ROI, staffing changes, customer sends, source cleanup, provider writes, or future savings. |
| Daily update MVP audit proof | `MvpPreview.approval`, `SendStub`, `audit_log`, included/omitted facts, internal flags. | Draft was review-required, facts were traceable, and send was blocked in the MVP. | Durable Daily Updates labor outcome storage or production Pawgress delivery. |
| Checkout proof | `checkout_completion::Packet` completion status, review gates, blocked actions, and audit-event drafts. | Checkout evidence was classified and routed safely. | Dedicated checkout outcome persistence or live PMS checkout execution. |

Value-measurement rules for this family:

- Minutes avoided: use manager brief before/after/actual minutes where supported; for Daily Updates and checkout, treat draft-writing/checking minutes as estimated/planned until an outcome record lands.
- Rework reduced: count omitted facts, internal flags, source-data-quality issues, wrong-source findings, suppressed outreach, and manager/staff dispositions.
- Handle time reduced: compare reviewed action packet time with prior manual queue scanning only where outcome records capture actual minutes.
- Customer-trust value: record approval/suppression/disposition and blocked send evidence; do not turn it into ROI without measured outcomes.
- Wrong-source prevention: record source refs, provenance, issue refs, and disposition such as source fact was wrong or suppressed by staff/manager.

## 10. Source/Rustdoc/test evidence links

Shared evidence:

- Agent specs and prompt packets: `../../../app/src/agents.rs` (`AgentSpec`, `WorkflowAgent`, `AgentPromptPacket`, `baseline_agent_specs`).
- Policy/review vocabulary: `../../../domain/src/policy.rs` (`ReviewGate`, automation authority, denial/review vocabulary).
- Workflow event and allowed action vocabulary: `../../../domain/src/workflow.rs`.
- Source evidence vocabulary: `../../../domain/src/source.rs` (`RecordRef`, `Provenance`, source systems, source snapshots).
- Message lifecycle vocabulary: `../../../domain/src/message.rs`.
- Durable outcome/labor projection evidence: `../../../storage/src/operations.rs`.
- Safety navigation: `../source-evidence-map.md`, `../review-boundaries-matrix.md`, and `../../design/entity-atlas-review-safety-boundaries.md`.

Specialized app/source evidence:

- Daily update MVP: `../../../app/src/daily_update.rs`; tests `../../../app/tests/daily_care_update_mvp.rs`; operator page `../../workflows/operator/daily-updates-pawgress-drafts.md`; workflow spec `../../workflows/daily-care-update-agent.md`.
- CRM retention and grooming rebooking: `../../../app/src/crm_retention.rs`; tests `../../../app/tests/crm_retention_workflow_contracts.rs`; operator page `../../workflows/operator/grooming-rebooking-retention.md`; workflow spec `../../workflows/crm-retention-agent.md`.
- Checkout completion prerequisite: `../../../app/src/checkout_completion.rs`; tests `../../../app/tests/checkout_completion_workflow_contracts.rs`; operator page `../../workflows/operator/checkout-completion.md`.
- Manager Daily Brief: `../../../app/src/manager_daily_brief.rs`; tests `../../../app/tests/manager_daily_brief_workflow_contracts.rs`; domain vocabulary `../../../domain/src/daily_brief.rs`; operator page `../../workflows/operator/manager-daily-brief.md`.
- Message/draft tool ports: `../../../app/src/tools.rs` (`messaging`, `draft_update`, read-only/draft-only tool boundary).
- App and domain maps: `../../../app/README.md` and `../../../domain/README.md`.

Evidence status to preserve:

- Daily Updates evidence supports local MVP preview/draft behavior, not production Pawgress delivery, media publication, live messaging integration, care-task completion, provider/PMS mutation, or measured production labor savings.
- Retention/grooming evidence supports local retention packet, eligibility/suppression, blocked actions, and app outcome record, not autonomous appointment creation, autonomous outreach, provider-calendar mutation, discount/payment action, or dedicated durable grooming-retention persistence.
- Checkout evidence supports local packet classification and blocked actions, not durable checkout-specific persistence, production labor measurement, live billing, provider writes, or customer sends.
- Manager brief evidence supports local ranked actions, source-grounding, blocked side-effect rejection, outcome records, and durable manager brief labor projection, not production ROI or live staffing/customer/provider/payment/source-cleanup side effects.

## 11. Open gaps or owner decisions

| Gap or owner decision | Why it matters | Safest current behavior | Evidence needed to close |
| --- | --- | --- | --- |
| Daily Updates durable labor/outcome storage is not identified. | Draft-writing minutes and review savings cannot be claimed as realized production value. | Treat value as draft/review time avoided only when supported by local packet/audit evidence; do not claim ROI. | Storage record or equivalent outcome projection with disposition, reviewer, actual minutes, approved payload, suppression/correction lineage, and source refs. |
| Production Pawgress delivery/media publication is outside current MVP evidence. | Draft copy and media use can expose wrong-pet, privacy, or sensitive care facts. | Keep all sends and media publication behind customer-message/media approval. | Approved outbox/effect ledger, channel/consent policy, media approval refs, audit trail, and tests proving blocked unsafe sends. |
| Retention/grooming has app outcome shape but no cited dedicated durable grooming-retention projection. | Staff disposition can be modeled locally, but durable reporting needs persistence evidence. | Record app-level outcome evidence; avoid claims of durable production retention ROI. | Storage/API projection and tests for follow-up outcome, suppression, wrong-source, source refs, actor, timestamp, and conversion metrics. |
| Checkout completion lacks dedicated durable checkout outcome persistence. | Checkout labor value and exception resolution cannot be proven durably by this overlay alone. | Use checkout packet/audit drafts as review evidence; mark durable outcome as planned/future. | Checkout outcome record or storage projection with exception disposition, reviewer, minutes, source refs, and no-live-side-effect proof. |
| Future auto-send or provider-write capability would require product/ops/security approval. | It would change the safety boundary from draft/review to live execution. | Assume no autonomous live execution; fail closed and route to reviewers. | Product/ops owner decision, IT/security controls, effect ledger, consent/channel policy, reviewer gates, and source/test evidence. |
| Medical/vaccine/behavior-sensitive wording in customer drafts. | Pet safety and customer trust require qualified review. | Flag/omit sensitive facts and route to medical/vaccine qualified staff, behavior/daycare lead, manager, and customer-message reviewer as appropriate. | Workflow tests and policy records proving role-specific review gates and approved wording paths. |

## Final reviewer checklist

- Does the first section name concrete labor/error costs for care staff, front desk, retention/grooming operators, and managers? Yes.
- Can a non-coder route each listed entity/action to the right reviewer? Yes: reviewer roles and approval conditions are named in section 7.
- Are “agent may read,” “agent may draft/recommend/rank/record,” and “agent must not do directly” separate? Yes: sections 4, 5, and 6.
- Are source evidence, human approval, draft creation, and outcome proof clearly different? Yes: sections 3, 8, and 9 separate them.
- Are blocked actions precise and entity-specific? Yes: sends, provider/PMS writes, schedules/capacity, money movement, medical/vaccine/behavior approvals, destructive cleanup, policy/tool changes, and labor-value overclaims are named.
- Does every labor-value claim require outcome/audit evidence rather than intent? Yes: section 9 separates estimated/planned value from outcome records.
- Are source/Rustdoc/test links current and local? Yes: source, tests, operator docs, and safety docs are local relative links.
- Are gaps marked as gaps instead of being filled with assumptions? Yes: section 11 lists current gaps and safest behavior.
