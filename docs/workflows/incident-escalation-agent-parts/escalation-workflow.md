# Incident escalation workflow

Purpose: define the end-to-end workflow for incident escalation from staff intake through manager review, owner communication, follow-up, post-incident review, and closeout. This is a workflow design artifact, not live operating policy. It does not authorize autonomous incident closure, customer messaging, provider writes, care/medical decisions, group-play reinstatement, eligibility changes, refunds, or legal/compliance decisions.

Source basis:

- `docs/workflows/incident-escalation-agent-parts/inputs.md` is the canonical input packet for this workflow.
- `docs/domain/petsuites/daycare/implications/04-incident-tracking.md` defines the incident aggregate, severity/disposition vocabulary, review gates, restrictions, and no-autonomous-closure invariants.
- `docs/domain/petsuites/daycare/implications/05-pet-health-behavior-notes.md` defines health/behavior note review states and eligibility-impacting boundaries.
- `docs/workflows/staff-operations-parts/manager-review-queue.md` defines shared manager-review queue semantics.
- `docs/architecture/pet-resort-workflow-events.md` defines `incident.created` and approval posture.
- `docs/architecture/workflow-result-envelope.md` defines result/draft/task/review semantics.

## Core rule

The Incident/Escalation Agent is a triage, drafting, and routing workflow. It may summarize source-backed facts, identify missing information, draft manager-review packets, draft owner-notification copy for review, recommend temporary profile flags/restrictions, and recommend follow-up tasks.

It must not autonomously resolve serious incidents. For this workflow, serious incidents include every medium, high, emergency, owner-notice-required, manager-review-required, medical/care ambiguous, behavior/eligibility-impacting, legal/compliance-sensitive, provider-write, or customer-message incident.

Every serious incident remains open until an authorized human records the required decision, approval, execution evidence, and closure rationale. AI-generated summaries, classifications, draft messages, or confidence scores are never closure evidence.

## Actors and authority

| Actor | May do | May not do |
| --- | --- | --- |
| Staff reporter | Create or update the incident report, record observed facts, record immediate care actions, add evidence/media references, and complete assigned info-gathering tasks. | Finalize medium/high/emergency severity, approve owner-facing copy, clear eligibility-impacting restrictions, or close serious incidents unless local policy explicitly grants that role and audit records the authority. |
| Lead staff / supervisor | Validate immediate operational response, request missing information, delegate staff follow-up, preserve temporary restrictions, and prepare manager handoff. | Send sensitive owner messages, clear group-play restrictions after serious incidents, or close serious incidents without required manager gates. |
| Manager/admin | Approve final severity/disposition, owner-facing messages, temporary profile flags that affect eligibility, restriction clearance, serious-incident closure, service-recovery/payment escalation, and post-incident review disposition. | Skip audit evidence, approve customer copy without source facts/unknowns, or clear restrictions while required follow-up remains open. |
| Vet/emergency/legal/compliance/payment specialist | Receives escalations when policy requires external/specialist handling. | Is not represented by AI; the workflow only records referrals, tasks, and evidence. |
| Incident/Escalation Agent | Read approved context, summarize facts, classify candidate severity, draft packets/messages/tasks, flag risk, and recommend review gates. | Diagnose, assign blame/liability, hide material facts, contact owners, contact vets/emergency services, mutate provider records, mark tasks complete, clear eligibility flags, or close serious incidents. |

## Incident state model

Recommended incident states for workflow routing:

1. `SignalReceived`: staff, provider import, customer-after-pickup report, daily note, or workflow detection indicates a possible incident.
2. `ReportDraft`: a report exists but is not complete enough for policy routing.
3. `NeedsStaffCompletion`: required intake fields or evidence are missing; staff/lead collection tasks are open.
4. `SubmittedForTriage`: minimum facts exist; the workflow can summarize evidence and propose severity/disposition.
5. `NeedsLeadReview`: immediate care mode, same-day staff action, or missing-field validation needs lead/supervisor attention.
6. `NeedsManagerReview`: medium/high/emergency, owner-notice, medical/care, behavior/eligibility, legal/compliance, payment/recovery, or unresolved-policy concern requires manager decision.
7. `OwnerNoticeDrafted`: owner-facing copy exists as a draft only and is waiting for approval, rejection, revision, or suppression.
8. `TemporaryFlagsActive`: temporary care-mode, group-play, eligibility, handling, checkout, or profile flags/restrictions are active pending review.
9. `FollowUpOpen`: staff/customer/care/behavior/provider/payment/post-incident tasks are linked and not all resolved.
10. `ResolvedPendingClosure`: manager-approved disposition exists and required sends/tasks/escalations are completed or explicitly suppressed, but closeout audit has not been finalized.
11. `Closed`: authorized human closure with evidence; unavailable while unresolved review gates, active restrictions, required owner notice, or open follow-up tasks remain.
12. `VoidedWithAudit`: duplicate/mistaken incident with manager/staff reason and pointer to replacement record if applicable.

State invariants:

- `Closed` is impossible while any serious-incident review gate, owner notification decision, temporary eligibility flag, restriction, unresolved missing info, or required follow-up task remains open.
- `Emergency` cannot be downgraded by the agent. A manager/safety actor must record the revised disposition and evidence.
- `TemporaryFlagsActive` cannot be silently removed by reruns, stale imports, newer owner messages, or positive daily notes.
- Owner-message draft creation never implies approval or delivery.

## Event flow

### 1. Staff intake and signal creation

Triggers:

- Staff observes injury, illness, bite/aggression, stress, escape/near-miss, facility/supervision issue, medication/feeding/care deviation, customer-service concern, property issue, or customer-reported-after-pickup concern.
- Provider import emits an incident signal after boundary verification and identity mapping.
- A daily note, manager brief, care watchlist, or staff task detects unresolved incident-like language.

Required intake fields:

- Location, operating day, source actor/system, reporter role or identity when known.
- Pet, customer, reservation/stay/attendance references when available.
- Observed-at/discovered-at time, place/context, category, candidate severity, immediate care action taken, current care mode, and affected future care mode.
- Observed facts, raw/internal narrative, customer-safe summary status, evidence/media refs, witnesses/staff refs, owner-notification state, and unknowns.

Workflow behavior:

- If required fields are missing, create or recommend `IncidentFollowUp` / staff completion tasks and keep state `NeedsStaffCompletion`.
- The agent may produce a missing-field checklist and internal summary, but must not fabricate facts or treat missing fields as clear.
- If active safety/medical/escape/bite/aggression/emergency signals appear, route immediately to lead/manager/emergency path while staff completion continues in parallel.

Audit events:

- `incident.signal_received`
- `incident.report_drafted`
- `incident.source_mapped` or `incident.source_unmapped`
- `incident.missing_info_recorded`
- `staff_task.drafted_or_created` for completion work

### 2. Triage and candidate classification

The workflow reads the report, current pet/reservation/care/temperament context, existing restrictions, staff tasks, and policy snapshot. It returns an incident triage packet with:

- source-backed timeline;
- candidate severity (`Low`, `Medium`, `High`, `Emergency`);
- candidate disposition;
- missing/conflicting facts;
- proposed review gates;
- proposed temporary flags/restrictions;
- owner-notification posture;
- follow-up task recommendations;
- verification evidence and unchecked sources.

Classification authority:

- AI may propose classifications only.
- Final `Medium`, `High`, and `Emergency` classifications require human approval.
- Any downgrade from `High`/`Emergency`, or from a suspending/restrictive classification to a less restrictive one, requires manager approval and audit evidence.

Audit events:

- `incident.triage_packet_created`
- `incident.candidate_severity_proposed`
- `incident.review_gates_proposed`
- `incident.risk_flags_recorded`

### 3. Manager task creation and review queue routing

A manager-review task must be drafted or created for every incident that is medium, high, emergency, owner-notice-required, behavior/eligibility-impacting, medical/care ambiguous, legal/compliance/privacy-sensitive, payment/service-recovery-sensitive, provider-write-related, or unresolved after staff completion.

Manager task contract:

| Field | Requirement |
| --- | --- |
| Kind | Current nearest kind is `IncidentFollowUp`; future refinement should carry `incident_id` and review sub-intent. |
| Status | `NeedsManagerReview` or equivalent queue state; `Blocked` / `NeedsInfo` when required fields are missing. |
| Subject | Incident, pet, reservation/stay, customer, location, operating day, related tasks/messages/restrictions. |
| Priority | Critical for emergency/active safety; High for owner-notice, restriction, checkout, medical/care ambiguity, or same-day eligibility; Normal/Low only for retrospective non-sensitive review. |
| Due basis | Immediate for emergency/active safety; before checkout/release/customer contact; before future group-play/booking eligibility; or policy-defined review window. |
| Evidence | Source refs, staff report, timeline, media refs, unknowns, policy snapshot, proposed draft messages, existing restrictions, prior incidents/notes if in scope. |
| Decision requested | Approve/reject severity, owner notice, temporary flags, follow-up tasks, external escalation, suppression, closure readiness, or post-incident review. |
| Forbidden actions | No customer send, provider mutation, refund/credit, eligibility clearance, or incident closure from task creation alone. |

Manager task outcomes:

- `Approved`: manager approves a bounded disposition, draft, flag, or next action.
- `Rejected`: proposed classification/message/flag/task is rejected with reason.
- `NeedsInfo`: staff/customer/provider/vet facts are missing.
- `Escalated`: emergency/vet/legal/compliance/payment/engineering escalation is required.
- `Suppressed`: manager records why no owner-visible action or specific follow-up is appropriate.
- `Closed`: only after all incident closure invariants pass.

Audit events:

- `manager_review.requested`
- `manager_review.assigned`
- `manager_review.decision_recorded`
- `manager_review.escalated`
- `manager_review.needs_info`

### 4. Temporary profile flags and restrictions

Temporary flags are provisional profile/care/eligibility restrictions linked to the incident. They are not permanent profile truth unless a human-approved profile update later promotes them.

Recommended flag types:

- `GroupPlaySuspendedPendingReview`
- `IndividualCareOnlyForDay`
- `ManagerApprovalRequiredBeforeCheckIn`
- `CareReviewRequiredBeforeAttendance`
- `MedicalDocumentReviewRequired`
- `TemperamentReassessmentRequired`
- `OwnerCommunicationHold`
- `CheckoutReleaseReviewRequired`
- `ReviewRequestSuppressedDueToIncident`

Creation rules:

- The workflow may recommend flags when source facts show a potential safety, medical/care, behavior, eligibility, communication, or checkout risk.
- Production auto-creation is allowed only if a later deterministic policy explicitly authorizes the exact flag type, source facts, duration, and assignee. Until then, flags are draft/review-required recommendations except where staff/manager has explicitly applied them.
- Emergency/high/suspending incidents should preserve the most conservative active restriction until reviewed.

Approval and clearance authority:

- Eligibility-impacting behavior flags require manager approval and/or `BehaviorReview` before creation becomes authoritative and before clearance.
- Medical/care flags require manager/authorized care reviewer and/or `MedicalDocumentReview` before clearance.
- Owner-communication holds and review-request suppression may be recommended by the workflow but final suppression/send decisions require manager approval.
- AI cannot clear group-play restrictions, mark a pet eligible after a serious incident, or remove behavior/care flags.

Audit events:

- `profile_flag.recommended`
- `profile_flag.applied`
- `profile_flag.extended`
- `profile_flag.clearance_requested`
- `profile_flag.cleared_by_manager`
- `profile_flag.clearance_denied`

### 5. Owner-notification draft review

Owner-facing incident communication is always a draft until an authorized human approves it. This includes incident notices, daily updates containing incident facts, checkout handoffs, apologies, follow-up requests, service-recovery language, restriction explanations, review-request suppression/replacement text, and any medical/behavior/safety/legal/payment-sensitive copy.

Draft packet requirements:

- Audience, proposed channel, subject, body, tone goal, and customer-visible intent.
- Source refs for every factual claim.
- Explicit unknowns and unresolved follow-up where relevant.
- Redaction note separating raw/internal facts from customer-safe wording.
- Review gates: at minimum `CustomerMessageApproval`; also `ManagerApproval` for medium/high/emergency, medical/care, behavior/eligibility, legal/compliance, payment/recovery, or sensitive copy.
- Send policy: `DraftOnly` or `RequiresApproval`; never `send` from the agent result.

Who can approve owner-facing messages:

- Low/internal-only incidents: no owner-facing message is sent unless staff/manager approves a draft under local policy.
- Medium: manager or explicitly authorized lead/manager-review role must approve owner-facing copy before send.
- High: manager/admin approval is required; legal/compliance/payment/care review may also be required by content.
- Emergency: manager/safety handling is required; owner communication follows emergency procedure and cannot be agent-approved.

Owner draft outcomes:

- `ApprovedForSend`: approved human actor, channel, exact copy/version, source refs, and send constraints recorded; actual sending remains a separate outbox/tool execution event.
- `RejectedForRevision`: reviewer reason recorded; regenerated copy returns to draft review.
- `SuppressedWithReason`: manager records why no owner-visible message is appropriate.
- `NeedsInfo`: owner message waits for staff/customer/vet/provider facts.

Audit events:

- `owner_notice.draft_created`
- `owner_notice.review_requested`
- `owner_notice.approved`
- `owner_notice.rejected`
- `owner_notice.suppressed`
- `owner_notice.sent` or `owner_notice.delivery_failed` from the execution layer only

### 6. Follow-up task creation

Follow-up tasks turn incident decisions into accountable work. The workflow may draft tasks; deterministic policy or human approval decides which become live tasks.

Common follow-up tasks:

| Task | Owner | Trigger | Completion evidence |
| --- | --- | --- | --- |
| Staff completion | Reporter/lead | Missing who/what/when/where/source, witnesses, immediate action, media refs, owner-contact state. | Completed intake fields, actor, timestamp, evidence refs. |
| Care monitoring | Kennel/play staff/lead | Injury/illness/stress/feeding/medication/care deviation or emergency aftercare. | Observation timestamps, care actions, escalation notes, handoff status. |
| Playgroup/temperament reassessment | Lead/manager/qualified staff | Bite/aggression, stress, incompatible play, suspension, future group-play eligibility impact. | Assessment result, manager/behavior review, active flag disposition. |
| Document/medical review | Authorized care reviewer/manager | Medical/care ambiguity, vet note, medication/allergy/feeding uncertainty. | Reviewed source, safe instructions, unresolved caveats, effective date. |
| Owner follow-up | Manager/front desk under approval | Approved owner notice, owner question, pickup conversation, after-pickup report. | Approved copy/call note, sent/contact attempt proof, owner response. |
| Provider/system reconciliation | Manager/ops/integration | Provider incident import mismatch or approved provider write needed. | Mapping/write audit, retry result, unresolved adapter issue. |
| Cleaning/facility/hazard remediation | Lead/facilities/staff | Facility hazard, sanitation issue, escape/near-miss, damaged equipment. | Hazard cleared or escalated, before/after evidence, reopen criteria. |
| Staff debrief/training | Manager/lead | Process failure, serious incident, repeated pattern, ratio/supervision concern. | Debrief record, action items, policy/training follow-up. |
| Payment/service recovery | Manager/payment role | Refund/credit/waiver/request, customer complaint, service failure. | Manager/payment decision, approved customer copy, separate payment audit. |
| Post-incident review | Manager/lead | High/emergency, repeated pattern, unresolved root cause, policy gap. | Review notes, root cause/unknowns, follow-up tasks, final disposition. |

Task creation rules:

- Task drafts must carry incident id/source refs, due basis, assignment role, priority rationale, duplicate key, review gate, and completion evidence.
- Critical/high safety tasks may be immediately surfaced to staff/manager, but completion remains human/tool-evidence-based.
- Follow-up task completion cannot by itself close a serious incident unless all closure invariants and manager approval are present.

Audit events:

- `incident_followup.task_drafted`
- `incident_followup.task_created`
- `incident_followup.task_completed`
- `incident_followup.task_blocked`
- `incident_followup.task_cancelled_or_superseded`

### 7. Missing-information and unresolved loops

Missing or conflicting information keeps the incident in a review state. The workflow must loop through tasking and review rather than silently proceeding.

Missing-info loop:

1. Workflow detects missing required intake fields, evidence, policy, owner-contact state, or source mapping.
2. Incident state becomes `NeedsStaffCompletion`, `NeedsInfo`, or `Blocked` with exact missing fields.
3. Staff/lead/manager task is drafted or created with due basis and evidence requested.
4. Staff/manager updates source facts or records that facts are unavailable.
5. Workflow reruns triage using the updated evidence and preserves previous attempts and unknowns.
6. If missing facts are material to owner message, eligibility, safety, closure, or legal/compliance posture, the review gate remains open.

Conflict loop:

1. Workflow detects conflicting staff/customer/provider/media/task facts.
2. It preserves all conflicting refs and marks the conflict type.
3. Manager review decides whether to request more information, prefer a source, suppress a draft, amend a report, or escalate externally.
4. Audit records the decision and why the less restrictive interpretation was or was not accepted.

Unresolved policy loop:

1. Workflow detects missing approved severity mapping, owner-notice SLA/channel, emergency procedure, flag duration, or approver role.
2. It sets `human_review_reason = unresolved_policy` and routes manager/policy review.
3. It does not invent timing, approvers, emergency contact behavior, legal language, or eligibility rules.

### 8. Post-incident review and closeout

Post-incident review is required for high and emergency incidents, repeated medium incidents, incidents with active restrictions, incidents with customer complaint/service-recovery risk, and incidents exposing policy/source-system/process gaps.

Post-incident review packet:

- Final source-backed timeline and evidence list.
- Final severity/disposition and approver.
- Immediate actions taken and unresolved facts.
- Owner communication decision and delivery/suppression evidence.
- Temporary flags/restrictions created, current status, and clearance decision.
- Follow-up tasks created/completed/cancelled/escalated.
- Any provider/system reconciliation, payment/service recovery, legal/compliance, training, or policy follow-up.
- Root-cause notes and prevention tasks where appropriate.
- Closure checklist and remaining risks.

Closure checklist:

- Required intake fields are complete or unavailable-with-reason.
- Final severity/disposition is human-approved for every serious incident.
- Owner-notification decision is approved, sent, suppressed, or not required with reason.
- Emergency/vet/legal/compliance/payment escalations are completed or have active owner tasks outside incident closure scope.
- Temporary profile flags/restrictions are either cleared by authorized review or intentionally remain active with follow-up owner.
- Eligibility-impacting behavior/health flags have required `BehaviorReview`, `MedicalDocumentReview`, and/or `ManagerApproval` decisions.
- Required follow-up tasks are completed, cancelled with reason, or moved to a long-running external follow-up with manager approval.
- Audit trail includes creation, triage, manager review, owner-message review, flag changes, follow-up tasks, external execution, and closure actor.

Only a manager/admin or other explicitly authorized human role may close serious incidents. The agent may recommend `ResolvedPendingClosure` when the checklist appears satisfied, but cannot set `Closed`.

Audit events:

- `post_incident_review.requested`
- `post_incident_review.completed`
- `incident.closure_recommended`
- `incident.closed_by_manager`
- `incident.closure_denied`

## Severity-specific escalation paths

### Low path: note-only / internal follow-up candidate

Use for minor, source-backed incidents with no owner-notice requirement, no medical/care ambiguity, no behavior/eligibility impact, no active safety risk, no provider/payment/legal impact, and no unresolved required facts.

Flow:

1. Staff intake creates signal/report.
2. Workflow summarizes facts and checks for hard stops.
3. If no hard stops are present, it may draft an internal note/follow-up recommendation.
4. Human/staff policy determines whether no owner message is needed.
5. Closure may be allowed only when no serious gates, restrictions, owner notice, or follow-up tasks remain.

Escalate out of low if any missing or conflicting fact could affect safety, owner notice, eligibility, medical/care, legal/compliance, provider writes, or checkout.

### Medium path: manager/owner-notice likely

Use for incidents with owner-notice likely, customer-impacting facts, same-day care changes, moderate behavior/health concerns, unresolved staff details, or review-sensitive wording.

Flow:

1. Staff intake creates report and immediate action record.
2. Workflow proposes `Medium`, creates manager/lead review packet, and drafts owner notice if appropriate.
3. Temporary flags may be recommended for care mode, review-request suppression, owner communication hold, or checkout review.
4. Manager/authorized lead reviews severity, missing facts, owner draft, and follow-up tasks.
5. Approved owner communication is sent only through the approved outbox/tool path.
6. Follow-up tasks and post-message state are resolved before closeout.

Required approvals:

- Final medium classification: human approval.
- Owner-facing copy: `CustomerMessageApproval`; manager/lead authority as approved by local policy.
- Eligibility-impacting flags: `BehaviorReview` and/or `ManagerApproval`.
- Closure: human approval when owner notice, temporary flags, or follow-up tasks were involved.

### High path: safety, suspension, medical/care, legal/compliance, or eligibility impact

Use for injury/health concern, bite/aggression, group-play suspension, medication/care deviation with possible harm, escape/near-miss, sensitive complaint, checkout/release risk, provider/payment/legal/compliance implication, or any incident requiring manager-owned disposition.

Flow:

1. Staff/lead preserves safety first and records immediate actions.
2. Workflow routes to critical/high manager queue, proposes high severity, and recommends conservative flags/restrictions.
3. Manager reviews source evidence, immediate safety state, owner notice, external escalation, and eligibility impact.
4. Owner-facing copy is drafted only after enough facts exist and is manager-approved before send.
5. Follow-up tasks are created for care monitoring, behavior reassessment, owner follow-up, provider/payment/legal/compliance, staff debrief, and post-incident review as needed.
6. Closure remains blocked until manager-approved disposition, communication decision, flags/restrictions, and follow-up tasks are resolved.

Required approvals:

- Final high classification and any downgrade: manager/admin.
- Owner-facing copy: manager/admin plus specialist review when content is medical/legal/payment-sensitive.
- Eligibility-impacting flags and clearance: manager/admin plus `BehaviorReview` and/or `MedicalDocumentReview` as relevant.
- Closure: manager/admin with post-incident review when required.

### Emergency path: active safety/vet/emergency escalation

Use for active or suspected emergency/vet escalation, severe injury/medical distress, escape/lost pet, severe aggression/bite, medication error with possible harm, suspected neglect/abuse, or active facility/safety hazard.

Flow:

1. Staff/lead/manager follows local emergency procedure outside the AI workflow.
2. Workflow records/updates the incident packet, flags emergency candidate, and surfaces immediate manager/safety task.
3. Owner/emergency/vet contact is handled by authorized humans or a later explicitly approved emergency tool path; the agent does not contact anyone autonomously.
4. Temporary restrictions/holds remain active until manager/safety review records disposition.
5. Owner communication, legal/compliance, provider writes, payment/service recovery, and post-incident review remain manager/specialist-gated.
6. Closure is impossible until emergency disposition, owner-contact decision/evidence, active restrictions, required follow-ups, and post-incident review are complete.

Required approvals:

- Final emergency classification, downgrade, and closure: manager/admin/safety authority.
- Owner contact/copy: manager/safety authority under local emergency procedure.
- Any eligibility/care/medical restriction clearance: manager/admin plus relevant specialist review.

## Workflow result shape

A typical serious-incident workflow result should have:

```json
{
  "status": "needs_human_review",
  "summary": "Source-backed incident triage summary with unresolved facts and required review gates.",
  "structured_output": {
    "incident_state": "NeedsManagerReview",
    "candidate_severity": "High",
    "candidate_disposition": "SuspendGroupPlayPendingManagerReview",
    "temporary_flags_recommended": ["GroupPlaySuspendedPendingReview"],
    "owner_notification_state": "DraftRequiresApproval"
  },
  "recommended_actions": [
    "Request manager review",
    "Preserve active restriction pending review"
  ],
  "draft_messages": [
    {
      "audience": "Owner",
      "send_policy": "RequiresApproval(CustomerMessageApproval)",
      "review_gate": "ManagerApproval"
    }
  ],
  "tasks_to_create": [
    {
      "kind": "IncidentFollowUp",
      "creation_policy": "RequiresReview(ManagerApproval)"
    }
  ],
  "risk_flags": [
    "customer_message_requires_review",
    "behavior_or_eligibility_impact",
    "incident_closure_requires_manager"
  ],
  "verification": {
    "evidence": [],
    "unchecked_sources": [],
    "redactions": [],
    "confidence": "source_backed"
  },
  "human_review_reason": "High incident with owner-facing and eligibility-impacting decisions cannot be resolved by AI."
}
```

Allowed `success` results should be rare and limited to internal, low-risk, non-customer-facing, non-eligibility, non-medical, non-provider-mutating summaries/tasks where policy explicitly allows the route and no serious-incident gates are present.

## Audit event catalog

The implementation should preserve distinct audit events for:

- Intake/source: `incident.signal_received`, `incident.report_drafted`, `incident.report_submitted`, `incident.source_mapped`, `incident.source_unmapped`.
- Missing/conflict: `incident.missing_info_recorded`, `incident.conflict_detected`, `incident.needs_info`, `incident.info_supplied`.
- Triage: `incident.triage_packet_created`, `incident.candidate_severity_proposed`, `incident.review_gates_proposed`, `incident.risk_flags_recorded`.
- Review: `manager_review.requested`, `manager_review.assigned`, `manager_review.decision_recorded`, `manager_review.escalated`, `manager_review.rejected`.
- Owner communication: `owner_notice.draft_created`, `owner_notice.review_requested`, `owner_notice.approved`, `owner_notice.rejected`, `owner_notice.suppressed`, `owner_notice.sent`, `owner_notice.delivery_failed`.
- Flags/restrictions: `profile_flag.recommended`, `profile_flag.applied`, `profile_flag.extended`, `profile_flag.clearance_requested`, `profile_flag.cleared_by_manager`, `profile_flag.clearance_denied`.
- Tasks: `incident_followup.task_drafted`, `incident_followup.task_created`, `incident_followup.task_completed`, `incident_followup.task_blocked`, `incident_followup.task_cancelled_or_superseded`.
- External/specialist: `external_escalation.requested`, `external_escalation.completed`, `provider_write.approved`, `provider_write.executed`, `payment_review.requested`, `legal_compliance_review.requested`.
- Closeout: `post_incident_review.requested`, `post_incident_review.completed`, `incident.closure_recommended`, `incident.closed_by_manager`, `incident.closure_denied`, `incident.voided_with_audit`.

Every audit event should carry actor, role, timestamp, subject ids, source refs, policy snapshot, before/after state where applicable, decision reason, and execution/proof refs for side effects.

## Implementation guardrails

- Raw provider payloads, raw photos/media, raw OCR, and unscoped staff-only notes remain boundary evidence refs; workflow payloads expose typed, policy-ready facts only.
- Use duplicate keys for manager tasks, owner drafts, temporary flags, and follow-up tasks so reruns reconcile instead of creating parallel work.
- A newer positive note or owner response does not clear a prior serious incident restriction unless the required clearance gate is recorded.
- Review-request/reputation workflows should be suppressed or routed to manager while incident owner notice, complaint, payment dispute, or follow-up remains unresolved.
- Checkout/release packets must surface active incident gates, owner-notice state, temporary flags, and required manager decisions.
- Provider writes and message sends require approved action/outbox records separate from workflow recommendations.
- If exact location severity mapping, owner-notice SLA/channel, emergency procedure, approver roster, photo-retention rule, or legal language is missing, the workflow must record unresolved policy and route manager review rather than inventing it.
