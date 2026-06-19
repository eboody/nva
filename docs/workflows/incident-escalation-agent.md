# Incident/Escalation Agent

> Successor route: this is a detailed specification/supporting-proof artifact, not the current reader spine. Start with the [docs successor and archive map](../design/successor-archive-map.md#older-workflow-and-specification-docs), [workflow-to-entity map](../design/workflow-to-entity-navigation-map.md), and [operator workflow index](operator/README.md) before using this page for current claims.

Purpose: define the integrated Incident/Escalation Agent workflow from intake through classification, AI summaries, owner-message drafts, manager escalation, temporary profile flags, follow-up tasks, post-incident review, and closeout.

Status: draft integration artifact. This document is not live operating policy and does not authorize autonomous incident closure, owner-facing communications, medical/safety decisions, behavior eligibility changes, provider writes, task completion, refunds, legal/compliance decisions, or final medium/high/emergency classification.

## Source parts

This artifact synthesizes:

- `docs/workflows/incident-escalation-agent-parts/inputs.md`
- `docs/workflows/incident-escalation-agent-parts/incident-types.md`
- `docs/workflows/incident-escalation-agent-parts/severity-levels.md`
- `docs/workflows/incident-escalation-agent-parts/input-form.md`
- `docs/workflows/incident-escalation-agent-parts/ai-summary-behavior.md`
- `docs/workflows/incident-escalation-agent-parts/escalation-workflow.md`

Related architecture/domain anchors include the workflow result envelope, workflow event catalog, runtime memory/context policy, queue/outbox model, Gingr incident webhooks, daycare incident tracking, pet health/behavior notes, staff operations, and current Rust domain/policy/workflow surfaces.

## Core operating posture

The Incident/Escalation Agent is a triage, drafting, and routing assistant. It may:

- Read approved typed incident, pet, customer, reservation, care, temperament, task, attachment-reference, audit, and policy context.
- Summarize source-backed incident facts and build a chronology.
- Identify missing, conflicting, stale, or provider-unverified fields.
- Propose incident types, severity candidates, review gates, temporary flags, follow-up tasks, and escalation routes.
- Draft internal manager/lead review packets.
- Draft owner-facing incident messages only as review-gated artifacts.
- Surface unresolved incidents in manager queues, shift handoffs, daily briefs, pet-care watchlists, and post-incident review packets.

The agent must not:

- Autonomously resolve serious incidents.
- Diagnose medical conditions, infer treatment, determine whether veterinary care is needed, or convert vague medical/care notes into executable care instructions.
- Minimize risk, hide material facts, omit uncertainty, or soften away active restrictions or required follow-up.
- Make final medium, high, or emergency classifications, or downgrade serious candidates.
- Close incidents requiring manager review, clear restrictions, reinstate group play, or mark staff tasks complete without authorized human/tool evidence.
- Apply, clear, or finalize behavior/profile flags that affect eligibility.
- Send owner-facing communications, queue them for outbox execution, or treat drafts as approved copy.
- Mutate provider records, reservation status, eligibility state, payment/refund state, or permanent audit-affecting business state without approved action records.
- Assign blame, make liability/legal claims, publish public/legal responses, or promise refunds/credits unless approved language and authority exist.

Serious incidents include every medium, high, emergency, owner-notice-required, manager-review-required, medical/care-ambiguous, behavior/eligibility-impacting, legal/compliance-sensitive, provider-write, or owner-message incident. Serious incidents remain open until an authorized human records the required decision, approval, execution evidence, and closure rationale.

## Actors and authority

| Actor | May do | May not do |
| --- | --- | --- |
| Staff reporter | Record observed or reported facts, create/update intake, record immediate actions already taken, attach evidence refs, answer missing-field prompts, complete assigned information-gathering tasks. | Diagnose, assign blame/liability, finalize medium/high/emergency severity, approve owner-facing copy, clear eligibility-impacting restrictions, or close serious incidents unless local policy explicitly grants that authority and audit records it. |
| Lead staff / supervisor | Validate immediate operational response, request missing information, delegate staff follow-up, preserve temporary restrictions, and prepare manager handoff. | Send sensitive owner messages, clear group-play restrictions after serious incidents, or close serious incidents without required manager gates. |
| Manager/admin | Approve final severity/disposition, owner-facing messages, temporary profile flags affecting eligibility, restriction clearance, serious-incident closure, service-recovery/payment escalation, and post-incident review disposition. | Skip audit evidence, approve copy without source facts/unknowns, or clear restrictions while required follow-up remains open. |
| Specialist role | Vet/emergency/legal/compliance/payment or similar role receives escalation where policy requires. | The AI does not stand in for specialist judgment; the workflow records referrals, tasks, and evidence only. |
| Incident/Escalation Agent | Read approved context, summarize facts, classify candidates, draft packets/messages/tasks, flag risk, and recommend review gates. | Diagnose, minimize, contact owners/vets/emergency services, mutate provider records, mark tasks complete, clear eligibility flags, finalize serious classifications, or close serious incidents. |

## Intake triggers

The workflow starts when any approved source indicates a possible incident:

- Staff observes or reports injury, illness, bite/aggression, stress, escape/near-miss, facility/supervision issue, medication/feeding/care deviation, customer-service concern, property damage, staff-safety issue, or owner/customer-reported-after-pickup concern.
- A lead/manager creates or updates an incident signal.
- A provider import, such as Gingr `incident_created` or `incident_edited`, emits an incident signal after boundary verification and identity mapping.
- A daily note, manager brief, care watchlist, staff task, customer message, or workflow detects unresolved incident-like language.

Raw provider payloads, raw photos/media, raw OCR, unscoped staff-only notes, and unverified customer reports remain boundary evidence. Workflow payloads should expose typed, policy-ready facts and evidence references, not raw sensitive material by default.

## Staff intake form

The incident intake form captures source facts, evidence references, and unresolved questions. It does not send messages, approve drafts, close incidents, diagnose, clear restrictions, change eligibility, write provider records, or complete staff tasks.

### Intake principles

1. Capture observations, not diagnosis.
2. Preserve raw/internal facts separately from customer-safe wording.
3. Mark unknowns instead of guessing.
4. Store media and documents by reference with sensitivity, redaction, retention, and owner-sharing review metadata.
5. Keep an audit trail for every submission, edit, attachment, source import, AI prompt/output, human approval, notification decision, task, and closure action.

### Required intake fields

Every report should capture, when available:

- Incident/report id, source system, source actor, reporter identity/role, source record refs, and location.
- Operating day, shift/service window, observed-at timestamp and precision, submitted-at timestamp, time zone, and where the incident happened.
- Pet, customer, reservation/attendance/stay refs, current service/care mode, room/yard/playgroup/suite, and staff on duty.
- Multi-select incident types and candidate severity.
- Source-grounded factual summary, timeline entries, raw/internal narrative refs, customer-safe summary status, and explicit unknowns.
- Involved pets with role, uncertainty, current care mode, and active restriction candidates.
- Staff and witnesses with roles, statements, actions, and follow-up needs.
- Immediate action taken, action actor/time, current status after action, current safety/care concern, and urgent human attention flag.
- Observed behavior, observed health/injury signs, body location, duration/frequency, objective measurements if actually taken, and no-diagnosis acknowledgement.
- Attachments/media/document refs, sensitivity, redaction status, and owner-share review status.
- Owner notification status, owner-message draft request, owner-message constraints, owner contact refs, and required `CustomerMessageApproval` gate.
- Manager notification status, manager/lead ref, review reason, and approval/decision refs when available.
- Follow-up needs, recommended task kinds, blocking unknowns, active review gates, and closure blockers.
- Staff attestations that facts are source-grounded, no diagnosis/liability statement was entered, and urgent path was used outside the AI form when needed.

### Requiredness by severity

- Low: core report fields are required; missing immediate action or follow-up may be accepted only if explicitly marked and routed as needed. Owner-facing drafts remain approval-gated.
- Medium: pet/reservation/staff refs, immediate action, current care state, observed signs/behavior where relevant, owner notification status, manager notification status, source refs, and follow-up needs are required for review.
- High: all medium fields plus stronger detail on injury/body location, care-plan/medication/allergy, facility/equipment, active restrictions, urgent human attention, attachments or explanation if absent, and manager routing are required.
- Emergency: same as high, with immediate human escalation confirmation; documentation must not delay real-world safety/care response.

Missing required fields create `NeedsStaffCompletion`, `NeedsInfo`, `Blocked`, or follow-up tasks. Missing data must not be fabricated or used as clearance.

## Incident type taxonomy

Incident types are multi-select. A single incident may span multiple categories; for example, a bite with puncture wound is both `bite/aggression` and `injury`, while a missed medication followed by symptoms is both `medication issue` and `illness`.

AI type classification is a draft signal for review, not a final disposition. The agent should preserve all material applicable types with evidence, uncertainty, and review gates.

| Type | Scope | Common follow-up / review |
| --- | --- | --- |
| `injury` | Physical injury, wound, lameness, soreness, bleeding, swelling, fall, cut, scrape, bite wound, nail/paw issue, heat-related physical concern, or customer-reported injury. | Incident follow-up, care/document review, owner notice draft, playgroup assessment if group context, possible individual care or group-play hold. No diagnosis. |
| `illness` | Vomiting, diarrhea, coughing, lethargy, allergic-reaction concern, heat stress, abnormal breathing, collapse, contagious concern, or owner-reported illness. | Care/medical review, isolation/rest handling, owner notice draft, sanitation task, attendance/check-in review. No diagnosis or contagiousness determination by AI. |
| `bite/aggression` | Bite, attempted bite, snapping, lunging, escalating chase, guarding, repeated mounting with escalation, barrier reactivity, human/dog-directed aggression, or behavior affecting group-play safety. | Playgroup/temperament reassessment, behavior review, manager review, owner notice draft, active group-play/handling restrictions. Never note-only as an AI hard stop when material. |
| `escape attempt` | Escape, attempted escape, near-miss elopement, door/gate/kennel breach, leash slip, pet outside expected area, or containment/supervision/facility condition creating escape risk. | Manager escalation, facility/safety task, escape-risk handling flag, owner notice draft, emergency route if actual escape/missing pet/injury. |
| `medication issue` | Missed, late, partial, extra, wrong, refused, vomited, spilled, undocumented, ambiguous, expired, unavailable, conflicting, storage/labeling, or instruction mismatch. | Medication/care document review, incident follow-up, owner notice draft, authorized medication task only through staff workflow. AI cannot change dose/timing or mark meds complete. |
| `feeding issue` | Missed, late, partial, wrong, extra, refused, vomited, allergy/food exposure, feeding instruction mismatch, unavailable food, or feeding-related customer/staff report. | Feeding/care review, allergy exposure review, incident follow-up, owner notice draft, daily update draft. AI cannot change feeding instructions or determine medical significance. |
| `bathroom concern` | Bathroom/elimination/stool/urine concern, accident, blood/mucus observation, straining, diarrhea/constipation concern, marking, sanitation issue, or related customer report. | Incident or care-note follow-up, care/document review for health context, sanitation task, owner notice draft for significant/repeated concern. |
| `customer complaint` | Customer-reported concern, dissatisfaction, dispute, allegation, refund/credit-adjacent issue, after-pickup report, communication complaint, staff-conduct concern, or conflicting owner/staff account. | Manager callback/review, incident follow-up, document/care/playgroup review as applicable, refund/payment exception review, customer-experience risk. Complaint alone is not final evidence for eligibility impact. |
| `staff safety` | Staff injury/near-miss, staff-directed aggression, unsafe handling, hazardous facility condition, unsafe customer interaction, ratio/supervision safety risk, or staff safety exposure. | Manager/admin safety review, handling/playgroup review, facility task, internal safety packet, customer communication only if manager-approved and customer-safe. |
| `property damage` | Damage to resort/customer/staff/pet/third-party property, rooms, suites, yards, doors, gates, fences, equipment, belongings, or facility assets. | Maintenance/facility task, incident follow-up, owner notice draft for customer property or pet-caused damage, payment/refund review if money is involved, safety review if hazard. |

Temporary pet/profile flags may be proposed for safety, handling, health/care, behavior, escape, staff safety, customer follow-up, and incident follow-up reasons. Manager review is required before any eligibility-impacting flag is applied, cleared, downgraded, or used to deny/reinstate group play.

## Severity levels

Severity is a triage and review-routing signal, not proof that the incident is complete.

The agent may propose a severity with source facts, uncertainty, and rationale. It may not finalize, downgrade, close, or suppress medium/high/emergency incidents. Missing, stale, contradictory, or provider-unverified facts raise review need rather than justifying downgrade.

| Level | Typical meaning | Approval posture | Escalation timing |
| --- | --- | --- | --- |
| `info` | Non-incident operational context with no safety, care, customer, eligibility, or policy effect. | May be recorded as internal context if source-backed and no owner/provider/business state is affected. | Normal backlog or next shift handoff. |
| `low` | Minor incident or staff follow-up with no live safety risk, no owner notice requirement, and no eligibility/care/legal sensitivity. | May remain internal, but owner drafts, closure/restriction/provider effects, and customer-visible decisions still require approval. | Same shift or before next relevant handoff/milestone. |
| `medium` | Owner notice, manager/lead review, care/behavior ambiguity, incomplete report, or customer-impacting follow-up likely. | Human approval required before final classification, owner notice/suppression, closure, or business-state mutation. | Prompt lead/manager review; before checkout, daily update, next care/play decision, or affected milestone. |
| `high` | Significant safety/behavior/health/care/customer/legal risk, active restriction, group-play suspension, injury, bite/aggression, escape/near-miss, medication/care deviation, or sensitive complaint. | Manager approval required; behavior/medical/customer/legal gates preserved. AI cannot downgrade or clear restrictions. | Immediate or same-operating-period manager escalation; before checkout and future eligibility decisions. |
| `emergency` | Active or potentially severe animal/person safety issue, medical distress, lost/escape event, severe bite/aggression, toxin/allergy/heat concern, or real-world emergency/vet/operator process. | Manager/operator/safety handling required. AI cannot finalize, downgrade, close, contact owner/vet, or substitute for care judgment. | Now; immediate real-world escalation under approved procedure. |

Every severity proposal should carry proposed severity, confidence/evidence quality, source facts, unknowns, current safety state, required review gates, staff actions, notification posture, escalation due basis, and closure blockers.

## Approval gates

The following gates are mandatory and must be explicit in every relevant workflow result, task, draft, and audit record.

### 1. Owner-facing incident messages

All owner-facing incident messages require explicit human approval before delivery. This includes:

- Incident notices.
- Daily/Pawgress updates containing incident facts.
- Checkout handoffs or pickup conversation notes.
- Apologies, follow-up requests, review-request suppression/replacement text, and customer complaint replies.
- Medical, behavior, safety, legal, payment, restriction, or service-recovery language.

Minimum gate: `CustomerMessageApproval`.

Additional manager/admin approval is required for medium/high/emergency, medical/care ambiguous, behavior/eligibility affecting, legal/compliance/privacy, payment/recovery, complaint, policy-sensitive, or otherwise sensitive incidents.

Draft creation is never approval or delivery. Actual sending requires a separate approved action/outbox execution record and delivery audit.

### 2. Medium, high, and emergency classification

Medium, high, and emergency classifications require human approval before becoming final business state.

The AI may propose a candidate classification with evidence and unknowns. It must not:

- Present medium/high/emergency as final.
- Downgrade a serious candidate.
- Suppress owner notice because a lower label is easier.
- Convert an emergency/vet/safety signal into a non-urgent state without manager/operator evidence.

### 3. Behavior flags affecting eligibility

Behavior and handling signals that affect group play, eligibility, temperament, restrictions, checkout, or future attendance require `BehaviorReview`, `ManagerApproval`, or both. Examples include bite history, attempted bite, group-play suspension, stressed group setting, intro-assessment-needed, review-required temperament rating, active incident restrictions, stale/conflicting behavior facts, and serious handling concerns.

The AI must not apply, clear, downgrade, or finalize these flags; mark a pet eligible after an incident; reinstate group play; or remove behavior hard stops.

### 4. Medical/care ambiguity

Injury, illness, medication, allergy, feeding, care-instruction deviation, emergency/vet-contact, and medical-document ambiguity require `MedicalDocumentReview`, manager/staff care review, or emergency workflow gates as appropriate.

The AI must not diagnose, determine treatment, decide contagiousness, approve return-to-care, convert vague notes into care instructions, or tell an owner whether veterinary care is required.

### 5. Incident closure and restriction clearance

Closure is blocked while any of the following remain unresolved:

- Active restriction or temporary profile flag.
- Unresolved owner notification decision.
- Open incident follow-up task.
- Missing required field or unresolved conflict.
- Medical/care, behavior/eligibility, legal/compliance, payment/recovery, provider-write, or customer-message gate.
- Manager review requirement.

Restriction clearance, group-play reinstatement, and final closure require manager/staff approval and audit evidence. The AI may recommend `ResolvedPendingClosure` when the checklist appears satisfied; it cannot set `Closed` for serious incidents.

### 6. Provider/system mutations

Provider incident writes, reservation status changes, eligibility changes, task completion, message sends, payment/refund actions, permanent profile changes, and outbox execution require approved actions and audit records. Workflow recommendations are not execution receipts.

### 7. Photos, attachments, and media

Incident media is stored and used by reference. Owner sharing requires separate approval. Runtime prompts should receive only scoped metadata or approved derived observations unless a future data-sent-to-runtime gate authorizes raw media.

## AI summary behavior

The AI summary is a source-grounded review artifact, not the authority of record.

### Allowed behavior

- Summarize staff/source facts with evidence refs.
- Build a timeline from source-backed timestamps and observations.
- Separate direct observations, secondhand reports, model interpretation, and unknowns.
- Identify missing required fields and create staff prompts.
- Propose provisional severity/type/disposition candidates.
- Draft manager/lead review packets.
- Draft owner-facing messages only as `DraftOnly` or `RequiresApproval(CustomerMessageApproval)`.
- Recommend review gates, risk flags, temporary flags, and follow-up tasks.
- Record confidence as evidence quality, not permission.

### Required wording posture

Use phrases such as:

- `staff reported`
- `the incident report says`
- `the record indicates`
- `source not yet verified`
- `unknown`
- `not documented`
- `needs staff confirmation`
- `provisional severity candidate`
- `final classification requires manager approval`

Avoid unsupported finality, reassurance, diagnosis, blame, legal conclusions, or delivery claims.

### Forbidden behavior

The AI must refuse or route to review rather than:

- Diagnose, recommend treatment, or advise whether vet care is or is not needed.
- Omit material facts, uncertainty, active restrictions, injuries, behavior concerns, or required follow-up.
- Mark medium/high/emergency, owner-notice-required, medical/care-ambiguous, behavior/eligibility-impacting, restriction-bearing, legal/compliance-sensitive, or incomplete incidents closed.
- Send, queue, approve, or imply delivery of owner-facing messages.
- Finalize medium/high/emergency severity or downgrade serious candidates.
- Apply, clear, or finalize eligibility-impacting behavior flags.

### Structured output contract

Incident summaries should be returned inside the workflow result envelope. A serious incident result should usually use `status = needs_human_review` and carry structured fields like:

```json
{
  "status": "needs_human_review",
  "summary": "Source-backed incident triage summary with unresolved facts and required review gates.",
  "structured_output": {
    "incident_state": "NeedsManagerReview",
    "incident_id": "inc_...",
    "source_event_id": "evt_...",
    "subject_refs": {
      "location_id": "loc_...",
      "pet_ids": ["pet_..."],
      "customer_ids": ["cust_..."],
      "reservation_ids": ["res_..."]
    },
    "facts_summary": {
      "timeline": [
        {
          "time": "2026-01-01T12:00:00Z",
          "fact": "Staff reported ...",
          "source_refs": ["evidence:incident_report:line_12"],
          "confidence": "source_backed"
        }
      ],
      "immediate_actions_reported": ["separated from group"],
      "current_care_or_restriction_state": "pending_manager_review",
      "owner_notification_state": "not_approved_or_unknown"
    },
    "missing_fields": [
      {
        "field": "observed_at",
        "why_it_matters": "Required for timeline and owner notice review.",
        "blocking": true,
        "recommended_follow_up": "Ask reporting staff to confirm the observed time."
      }
    ],
    "provisional_classification": {
      "incident_types": ["bite/aggression", "injury"],
      "severity_candidate": "High",
      "classification_status": "draft_provisional",
      "rationale": ["behavior safety signal", "possible injury", "owner notice likely"],
      "alternative_candidates": ["Emergency"],
      "requires_final_approval_gate": "ManagerApproval"
    },
    "review_gates": [
      "ManagerApproval",
      "CustomerMessageApproval",
      "BehaviorReview"
    ],
    "recommended_actions": [
      {
        "kind": "RequestHumanReview",
        "gate": "ManagerApproval",
        "reason": "Final classification and disposition are not AI-authorized."
      }
    ],
    "temporary_flags_recommended": ["GroupPlaySuspendedPendingReview"],
    "draft_messages": [
      {
        "audience": "manager",
        "send_policy": "DraftOnly",
        "body_ref": "draft:manager_packet"
      },
      {
        "audience": "owner",
        "send_policy": "RequiresApproval(CustomerMessageApproval)",
        "review_gate": "ManagerApproval",
        "body_ref": "draft:owner_notice"
      }
    ],
    "audit_logging": {
      "model_output_id": "ai_output_...",
      "prompt_packet_ref": "prompt_packet_...",
      "policy_snapshot_refs": ["policy:incident:v1"],
      "evidence_refs": ["evidence:incident_report:line_12"],
      "redactions": ["raw staff-only note omitted from owner draft"],
      "not_authorized_for": [
        "final_classification",
        "message_send",
        "incident_closure",
        "restriction_clearance"
      ]
    }
  },
  "human_review_reason": "High incident with owner-facing and eligibility-impacting decisions cannot be resolved by AI."
}
```

`success` should be rare and limited to internal, low-risk, non-customer-facing, non-eligibility, non-medical, non-provider-mutating analysis where policy explicitly allows the route and no serious-incident gates are present.

## Workflow state model

Recommended incident states:

1. `SignalReceived`: possible incident signal from staff, manager, provider import, customer report, daily note, task, or workflow.
2. `ReportDraft`: intake exists but is incomplete.
3. `NeedsStaffCompletion`: required intake fields/evidence are missing.
4. `SubmittedForTriage`: minimum facts exist for summarization and candidate classification.
5. `NeedsLeadReview`: immediate care mode, staff action, or same-day validation needs lead/supervisor attention.
6. `NeedsManagerReview`: medium/high/emergency, owner-notice, medical/care, behavior/eligibility, legal/compliance, payment/recovery, provider-write, or unresolved-policy concern requires manager decision.
7. `OwnerNoticeDrafted`: owner-facing copy exists as a draft awaiting approval/revision/rejection/suppression.
8. `TemporaryFlagsActive`: care, group-play, eligibility, handling, checkout, communication, or profile flags/restrictions are active pending review.
9. `FollowUpOpen`: staff/customer/care/behavior/provider/payment/post-incident tasks are linked and unresolved.
10. `ResolvedPendingClosure`: manager-approved disposition exists and required sends/tasks/escalations are complete or explicitly suppressed, but closeout audit is not final.
11. `Closed`: authorized human closure with evidence; unavailable while unresolved gates, restrictions, owner notice, missing info, or required follow-up remain.
12. `VoidedWithAudit`: duplicate/mistaken incident with reason and replacement pointer if applicable.

State invariants:

- `Closed` is impossible while serious review gates, owner notification decisions, temporary eligibility flags, restrictions, unresolved missing info, or required follow-up tasks remain open.
- `Emergency` cannot be downgraded by the agent.
- Temporary flags cannot be silently removed by reruns, stale imports, newer owner messages, or positive daily notes.
- Owner-message draft creation never implies approval or delivery.

## End-to-end event flow

### 1. Staff intake and signal creation

The workflow receives an incident signal and maps it to typed intake fields. If required fields are missing, it creates or recommends staff completion work and keeps the incident in `NeedsStaffCompletion`.

Outputs may include:

- Missing-field checklist.
- Internal fact summary.
- Staff completion task draft.
- Immediate lead/manager/emergency route if active safety, medical, escape, bite/aggression, or emergency signals appear.

Audit events:

- `incident.signal_received`
- `incident.report_drafted`
- `incident.report_submitted`
- `incident.source_mapped` / `incident.source_unmapped`
- `incident.missing_info_recorded`
- `staff_task.drafted_or_created`

### 2. Triage and candidate classification

The workflow reads the report, relevant pet/reservation/care/temperament context, existing restrictions, staff tasks, evidence refs, and policy snapshot. It produces a triage packet with source-backed timeline, candidate type/severity/disposition, missing/conflicting facts, proposed gates, temporary flags, owner-notification posture, follow-up tasks, verification evidence, and unchecked sources.

AI may propose classifications only. Final medium/high/emergency classification, serious downgrades, and less-restrictive disposition changes require human approval and audit evidence.

Audit events:

- `incident.triage_packet_created`
- `incident.candidate_severity_proposed`
- `incident.review_gates_proposed`
- `incident.risk_flags_recorded`

### 3. Manager review queue routing

A manager-review task must be drafted or created for every incident that is medium, high, emergency, owner-notice-required, behavior/eligibility-impacting, medical/care ambiguous, legal/compliance/privacy-sensitive, payment/service-recovery-sensitive, provider-write-related, unresolved after staff completion, or blocked by missing policy.

Manager task packet should include:

- Incident, pet, reservation/stay, customer, location, operating day, related tasks/messages/restrictions.
- Priority rationale: critical for emergency/active safety; high for owner notice, restriction, checkout, medical/care ambiguity, or same-day eligibility.
- Due basis: immediate safety, before checkout/release/customer contact, before next play/care decision, before care window, or policy-defined review window.
- Evidence refs, timeline, media refs, unknowns, policy snapshot, draft messages, restrictions, and in-scope prior incidents/notes.
- Decision requested: approve/reject severity, owner notice, temporary flag, follow-up task, external escalation, suppression, closure readiness, or post-incident review.
- Forbidden actions: no customer send, provider mutation, refund/credit, eligibility clearance, or closure from task creation alone.

Manager outcomes include `Approved`, `Rejected`, `NeedsInfo`, `Escalated`, `Suppressed`, and `Closed` only after closure invariants pass.

Audit events:

- `manager_review.requested`
- `manager_review.assigned`
- `manager_review.decision_recorded`
- `manager_review.escalated`
- `manager_review.needs_info`
- `manager_review.rejected`

### 4. Temporary profile flags and restrictions

Temporary flags are provisional profile/care/eligibility restrictions linked to the incident. They are not permanent profile truth unless a human-approved profile update later promotes them.

Recommended flag types:

- `GroupPlaySuspendedPendingReview`
- `IndividualCareOnlyForDay`
- `IndividualCareOnlyPendingReview`
- `ManagerApprovalRequiredBeforeGroupPlay`
- `ManagerApprovalRequiredBeforeCheckIn`
- `CareReviewRequiredBeforeAttendance`
- `MedicalDocumentReviewRequired`
- `MedicationPlanReviewRequired`
- `FeedingPlanReviewRequired`
- `AllergyExposureReviewRequired`
- `TemperamentReassessmentRequired`
- `SpecialHandlingRequiredPendingReview`
- `EscapeRiskReviewRequired`
- `StaffSafetyHandlingReviewRequired`
- `OwnerCommunicationHold`
- `CheckoutReleaseReviewRequired`
- `ReviewRequestSuppressedDueToIncident`
- `CustomerFollowUpRequired`
- `IncidentFollowUpRequired`

Creation and clearance rules:

- The workflow may recommend flags when source facts show potential safety, medical/care, behavior, eligibility, communication, checkout, or follow-up risk.
- Production auto-creation is allowed only if later deterministic policy authorizes the exact flag type, source facts, duration, and assignee.
- Emergency/high/suspending incidents preserve the most conservative active restriction until reviewed.
- Eligibility-impacting behavior flags require manager approval and/or `BehaviorReview` before authoritative creation and clearance.
- Medical/care flags require manager/authorized care reviewer and/or `MedicalDocumentReview` before clearance.
- Owner-communication holds and review-request suppression require manager approval before final suppression/send decisions.
- AI cannot clear group-play restrictions, mark a pet eligible after a serious incident, or remove behavior/care flags.

Audit events:

- `profile_flag.recommended`
- `profile_flag.applied`
- `profile_flag.extended`
- `profile_flag.clearance_requested`
- `profile_flag.cleared_by_manager`
- `profile_flag.clearance_denied`

### 5. Owner-message draft review

Owner-facing incident communication is always draft-only until approved. The draft packet must include audience, proposed channel, subject, body, tone goal, source refs for factual claims, explicit unknowns, redaction notes, required gates, and send policy.

Approver posture:

- Low/internal-only: no owner message is sent unless staff/manager approves a draft under local policy.
- Medium: manager or explicitly authorized lead/manager-review role must approve owner-facing copy before send.
- High: manager/admin approval is required; legal/compliance/payment/care review may also be required by content.
- Emergency: manager/safety handling and local emergency procedure control owner communication; AI cannot approve or send.

Draft outcomes:

- `ApprovedForSend`: approved human actor, channel, exact copy/version, source refs, and send constraints recorded; actual sending remains separate execution.
- `RejectedForRevision`: reviewer reason recorded; revised copy returns to draft review.
- `SuppressedWithReason`: manager records why no owner-visible message is appropriate.
- `NeedsInfo`: owner message waits for staff/customer/vet/provider facts.

Audit events:

- `owner_notice.draft_created`
- `owner_notice.review_requested`
- `owner_notice.approved`
- `owner_notice.rejected`
- `owner_notice.suppressed`
- `owner_notice.sent` / `owner_notice.delivery_failed` from execution layer only

### 6. Follow-up task creation

Follow-up tasks turn incident decisions into accountable work. The workflow may draft tasks; deterministic policy or human approval decides which become live tasks.

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
| Payment/service recovery | Manager/payment role | Refund/credit/waiver request, customer complaint, service failure. | Manager/payment decision, approved customer copy, separate payment audit. |
| Post-incident review | Manager/lead | High/emergency, repeated pattern, unresolved root cause, policy gap. | Review notes, root cause/unknowns, follow-up tasks, final disposition. |

Task drafts must carry incident id/source refs, due basis, assignment role, priority rationale, duplicate key, review gate, and completion evidence. Follow-up task completion cannot by itself close a serious incident unless all closure invariants and manager approval are present.

Audit events:

- `incident_followup.task_drafted`
- `incident_followup.task_created`
- `incident_followup.task_completed`
- `incident_followup.task_blocked`
- `incident_followup.task_cancelled_or_superseded`

### 7. Missing-information and unresolved loops

Missing, conflicting, or policy-dependent information keeps the incident in review state.

Missing-info loop:

1. Workflow detects missing intake fields, evidence, policy, owner-contact state, or source mapping.
2. Incident state becomes `NeedsStaffCompletion`, `NeedsInfo`, or `Blocked` with exact missing fields.
3. Staff/lead/manager task is drafted or created with due basis and requested evidence.
4. Staff/manager updates source facts or records that facts are unavailable.
5. Workflow reruns triage using updated evidence and preserves prior unknowns.
6. If missing facts affect owner message, eligibility, safety, closure, or legal/compliance posture, the review gate remains open.

Conflict loop:

1. Workflow detects conflicting staff/customer/provider/media/task facts.
2. It preserves all conflicting refs and marks conflict type.
3. Manager review decides whether to request more information, prefer a source, suppress a draft, amend a report, or escalate externally.
4. Audit records the decision and why the less restrictive interpretation was or was not accepted.

Unresolved-policy loop:

1. Workflow detects missing severity mapping, owner-notice SLA/channel, emergency procedure, flag duration, approver role, media-retention rule, or legal language.
2. It sets `human_review_reason = unresolved_policy` and routes manager/policy review.
3. It does not invent timing, approvers, emergency contact behavior, legal language, or eligibility rules.

### 8. Post-incident review and closeout

Post-incident review is required for high and emergency incidents, repeated medium incidents, active restrictions, customer complaint/service-recovery risk, and incidents exposing policy/source-system/process gaps.

Post-incident review packet:

- Final source-backed timeline and evidence list.
- Final severity/disposition and approver.
- Immediate actions taken and unresolved facts.
- Owner communication decision and delivery/suppression evidence.
- Temporary flags/restrictions created, current status, and clearance decision.
- Follow-up tasks created/completed/cancelled/escalated.
- Provider/system reconciliation, payment/service recovery, legal/compliance, training, or policy follow-up.
- Root-cause notes and prevention tasks where appropriate.
- Closure checklist and remaining risks.

Closure checklist:

- Required intake fields are complete or unavailable-with-reason.
- Final severity/disposition is human-approved for every serious incident.
- Owner-notification decision is approved, sent, suppressed, or not required with reason.
- Emergency/vet/legal/compliance/payment escalations are completed or have active owner tasks outside incident closure scope.
- Temporary profile flags/restrictions are cleared by authorized review or intentionally remain active with follow-up owner.
- Eligibility-impacting behavior/health flags have required `BehaviorReview`, `MedicalDocumentReview`, and/or `ManagerApproval` decisions.
- Required follow-up tasks are completed, cancelled with reason, or moved to long-running external follow-up with manager approval.
- Audit trail includes creation, triage, manager review, owner-message review, flag changes, follow-up tasks, external execution, and closure actor.

Only a manager/admin or explicitly authorized human role may close serious incidents.

Audit events:

- `post_incident_review.requested`
- `post_incident_review.completed`
- `incident.closure_recommended`
- `incident.closed_by_manager`
- `incident.closure_denied`

## Severity-specific paths

### Low path: note-only / internal follow-up candidate

Use for minor, source-backed incidents with no owner-notice requirement, medical/care ambiguity, behavior/eligibility impact, active safety risk, provider/payment/legal impact, or unresolved required facts.

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
3. Temporary flags may be recommended for care mode, owner communication hold, review-request suppression, or checkout review.
4. Manager/authorized lead reviews severity, missing facts, owner draft, and follow-up tasks.
5. Approved owner communication is sent only through approved outbox/tool path.
6. Follow-up tasks and post-message state resolve before closeout.

Required approvals:

- Final medium classification: human approval.
- Owner-facing copy: `CustomerMessageApproval`; manager/lead authority as approved by local policy.
- Eligibility-impacting flags: `BehaviorReview` and/or `ManagerApproval`.
- Closure: human approval when owner notice, temporary flags, or follow-up tasks were involved.

### High path: safety, suspension, medical/care, legal/compliance, or eligibility impact

Use for injury/health concern, bite/aggression, group-play suspension, medication/care deviation with possible harm, escape/near-miss, sensitive complaint, checkout/release risk, provider/payment/legal/compliance implication, or manager-owned disposition.

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
2. Workflow records/updates incident packet, flags emergency candidate, and surfaces immediate manager/safety task.
3. Owner/emergency/vet contact is handled by authorized humans or a later explicitly approved emergency tool path; the agent does not contact anyone autonomously.
4. Temporary restrictions/holds remain active until manager/safety review records disposition.
5. Owner communication, legal/compliance, provider writes, payment/service recovery, and post-incident review remain manager/specialist-gated.
6. Closure is impossible until emergency disposition, owner-contact decision/evidence, active restrictions, required follow-ups, and post-incident review are complete.

Required approvals:

- Final emergency classification, downgrade, and closure: manager/admin/safety authority.
- Owner contact/copy: manager/safety authority under local emergency procedure.
- Eligibility/care/medical restriction clearance: manager/admin plus relevant specialist review.

## Owner-message draft requirements

Owner-message drafts should be calm, factual, and review-gated. They should preserve material facts without diagnosis, blame, unsupported reassurance, minimization, or legal conclusions.

Every owner-facing draft must include metadata:

- `audience = Owner` or equivalent.
- `send_policy = DraftOnly` or `RequiresApproval(CustomerMessageApproval)`.
- Source refs for every factual claim.
- Unknowns and review notes where material.
- Required manager/specialist gates when severity or content requires them.
- Redaction notes separating raw/internal facts from customer-safe wording.
- Explicit statement that no send occurred from the AI output.

Safe draft patterns:

- Low/note-only candidate:
  - `Hi {{owner_name}}, we wanted to let you know that during {{pet_name}}'s visit today, our team noted {{brief_source_backed_observation}}. Staff documented the observation and {{immediate_action_taken_if_any}}. This message is a draft for review before sending.`
- Medium owner-notice candidate:
  - `Hi {{owner_name}}, our team documented an incident involving {{pet_name}} today at approximately {{time_if_known}}. The report notes {{brief_customer_safe_fact_summary}}. Staff took the following immediate steps: {{immediate_actions}}. A manager is reviewing the report and any next steps. We will share confirmed follow-up information after review.`
- Injury/health ambiguity:
  - `Hi {{owner_name}}, our team observed {{customer_safe_observed_sign_or_event}} involving {{pet_name}} today and documented it for review. Staff took {{immediate_action_taken}}. We are not making a medical determination in this message; a manager will review the report and follow the approved care/escalation procedure before any owner-facing update is sent.`
- Behavior/group-play review:
  - `Hi {{owner_name}}, our team documented a playgroup incident involving {{pet_name}} today: {{brief_customer_safe_behavior_fact}}. Staff responded by {{immediate_action_taken}}. A manager will review the report before any final playgroup or care-plan decision is made. This draft requires review before sending.`
- Unknowns:
  - `Hi {{owner_name}}, we have an incident report involving {{pet_name}} from today, but some details are still being confirmed, including {{missing_detail_plain_language}}. Staff have documented the report and it is being routed for manager review. We will only send a final update after review.`

Prohibited patterns include:

- Diagnosis/treatment: `probably has a sprain`, `looks like kennel cough`, `no vet visit is needed`, medication changes.
- Unsupported reassurance/minimization: `nothing happened`, `just a small scuffle` when material facts or missing fields exist, `no need to worry`, `everything is resolved` without closure proof.
- Finality: `this incident is closed`, `final severity`, `group play restriction cleared`, `approved owner message`, `message delivered` without execution proof.
- Legal/liability/blame: `we are liable/not liable`, `another dog caused the injury`, unapproved staff-fault statements, legal conclusions.

## Audit and verification

Audit must separate source facts, AI outputs, human approvals, and side effects.

Minimum audit fields:

- Workflow event id, incident id, location/tenant, subject ids.
- Actor, role, timestamp, source refs, and policy snapshot.
- Agent/spec version and model/runtime metadata when production policy permits.
- Prompt packet id/hash rather than raw prompt text by default.
- Approved data categories accessed and repository/tool refs.
- Evidence refs used in summary/classification/drafts.
- Structured output id and draft message ids.
- Confidence/uncertainty status.
- Redaction/minimization notes.
- Forbidden side effects explicitly not performed: no send, no closure, no final classification, no restriction clearance, no eligibility mutation.
- Human reviewer, approval timestamp, approved action/outbox refs, execution proof, and delivery proof when later actions occur.

Audit event catalog:

- Intake/source: `incident.signal_received`, `incident.report_drafted`, `incident.report_submitted`, `incident.source_mapped`, `incident.source_unmapped`.
- Missing/conflict: `incident.missing_info_recorded`, `incident.conflict_detected`, `incident.needs_info`, `incident.info_supplied`.
- Triage: `incident.triage_packet_created`, `incident.candidate_severity_proposed`, `incident.review_gates_proposed`, `incident.risk_flags_recorded`.
- Review: `manager_review.requested`, `manager_review.assigned`, `manager_review.decision_recorded`, `manager_review.escalated`, `manager_review.rejected`.
- Owner communication: `owner_notice.draft_created`, `owner_notice.review_requested`, `owner_notice.approved`, `owner_notice.rejected`, `owner_notice.suppressed`, `owner_notice.sent`, `owner_notice.delivery_failed`.
- Flags/restrictions: `profile_flag.recommended`, `profile_flag.applied`, `profile_flag.extended`, `profile_flag.clearance_requested`, `profile_flag.cleared_by_manager`, `profile_flag.clearance_denied`.
- Tasks: `incident_followup.task_drafted`, `incident_followup.task_created`, `incident_followup.task_completed`, `incident_followup.task_blocked`, `incident_followup.task_cancelled_or_superseded`.
- External/specialist: `external_escalation.requested`, `external_escalation.completed`, `provider_write.approved`, `provider_write.executed`, `payment_review.requested`, `legal_compliance_review.requested`.
- Closeout: `post_incident_review.requested`, `post_incident_review.completed`, `incident.closure_recommended`, `incident.closed_by_manager`, `incident.closure_denied`, `incident.voided_with_audit`.

## Implementation guardrails

- Use duplicate keys for manager tasks, owner drafts, temporary flags, and follow-up tasks so reruns reconcile instead of creating parallel work.
- Preserve both proposed and approved severity/disposition values, plus disposition history.
- Keep raw/internal narrative, customer-safe wording, and owner-send proof as separate values.
- Use review states such as `NeedsInfo`, `NeedsManagerReview`, `Escalated`, and `ResolvedPendingClosure` to avoid treating a draft summary as completion.
- Use due-basis fields such as `immediate_safety`, `before_checkout`, `before_next_playgroup`, `before_customer_send`, `before_care_window`, or `policy_defined_sla` rather than invented times.
- When local policy is missing, record `due_basis = unresolved_policy` and route manager review instead of inventing an SLA.
- A newer positive note, customer response, or stale import does not clear a prior serious incident restriction without the required clearance gate.
- Review-request/reputation workflows should be suppressed or routed to manager while incident owner notice, complaint, payment dispute, or follow-up remains unresolved.
- Checkout/release packets must surface active incident gates, owner-notice state, temporary flags, and required manager decisions.
- Provider writes and message sends require approved action/outbox records separate from workflow recommendations.

## Open questions and assumptions requiring human confirmation

These items require policy, legal, medical/care, operations, or implementation confirmation before production use.

### Policy and operations

1. What is the approved local incident severity taxonomy and exact mapping to low/medium/high/emergency?
2. Which incident categories require owner notice, manager approval, legal review, vet/emergency escalation, external reporting, or payment/service-recovery review?
3. Who may approve medium, high, emergency, owner-message, restriction-clearance, provider-write, payment/recovery, and closure gates by role, location, and time of day?
4. What owner-notice timing/SLA applies by severity, and which channels/templates are approved?
5. What emergency procedure exists for owner phone call, vet contact, emergency services, transport, after-hours escalation, and audit evidence?
6. Which internal staff-task drafts may be auto-created in production, and which remain review-only?
7. How should incidents affect checkout/release packets and same-day staff handoff requirements?

### Legal/compliance/privacy

1. What customer-safe legal/medical caution language is approved for owner drafts?
2. Which legal/compliance/privacy triggers require specialist review before owner communication or closure?
3. What photo/attachment storage, retention, access, redaction, and owner-sharing policy applies?
4. What public review/reputation, refund/waiver, service-recovery, or liability-sensitive language is approved?
5. Are any incident categories subject to external reporting obligations, and who owns that process?

### Medical/care cautions

1. What role may approve medical/care review, medication-plan review, feeding/allergy review, return-to-care, and emergency/vet escalation records?
2. What exact wording may staff use when describing observed signs without diagnosis?
3. Which health/illness observations require isolation, owner pickup, vet contact, sanitation, or attendance restriction under local SOP?
4. What evidence is required to mark medical/care ambiguity resolved?

### Behavior and eligibility

1. Should temporary profile flags live on pet profile, daycare eligibility snapshots, incident restrictions, staff tasks, or a dedicated profile-flag aggregate?
2. What durations and clearance criteria apply to group-play suspension, individual-care holds, escape-risk holds, temperament reassessment, and manager-approval-before-check-in flags?
3. What evidence and roles are required to reinstate group play or clear behavior restrictions after a serious incident?
4. How should repeated low/medium behavior incidents aggregate into future eligibility decisions?

### Integration and data model

1. What source-of-truth table owns incidents, restrictions, temporary profile flags, closure records, owner-message approvals, and post-incident review records?
2. What Gingr/provider incident fields are available in real payloads, and which writes, if any, are in MVP scope?
3. How should provider payload mapping failures be quarantined, retried, and surfaced to managers?
4. Which workflow event names and audit metadata are canonical for incident lifecycle transitions?
5. How should incidents suppress review-request/reputation workflows, marketing/customer follow-ups, and routine customer messaging?

### Safe provisional assumptions

Until those questions are answered:

- Treat every incident workflow result as `needs_human_review` unless it is strictly internal, source-backed, low-risk, non-customer-facing, non-eligibility, non-medical, and non-provider-mutating.
- Treat unknown/conflicting source facts as review blockers, not clearance.
- Treat all owner-facing incident copy as draft-only with `CustomerMessageApproval`.
- Treat medium/high/emergency, behavior/eligibility, medical/care, legal/compliance, closure, restriction-clearance, and provider-write effects as human-review-gated.
- Treat temporary profile flags as proposed typed restrictions/notes linked to the incident and pet until a profile-flag aggregate is designed.
- Treat notification routing as internal manager/lead/staff review packet generation unless exact channel delivery is approved.
