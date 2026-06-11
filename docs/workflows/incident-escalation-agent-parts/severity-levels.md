# Incident severity levels

Purpose: define proposed incident severity levels and the action, notification, review, and escalation expectations for the Incident/Escalation Agent. This is a workflow design artifact, not approved live operating policy. It does not authorize autonomous incident closure, owner-facing messages, medical/safety decisions, behavior eligibility changes, provider writes, or final medium/high/emergency classification.

Source basis:

- `docs/workflows/incident-escalation-agent-parts/inputs.md` is the canonical input packet for this file.
- `docs/workflows/staff-operations-parts/manager-review-queue.md` defines shared manager-review queue expectations for incidents.
- Current review gates include `ManagerApproval`, `MedicalDocumentReview`, `BehaviorReview`, and `CustomerMessageApproval`; richer incident-specific gates below are proposed labels until implemented.
- Exact local SOPs for owner-notice timing, vet/emergency routing, after-hours escalation, and legal/compliance review are not final in this repo. Timing values below are due semantics and conservative draft defaults, not approved clock-time SLAs.

## Core rules

1. Severity is a triage and review-routing signal, not proof that the incident is complete.
2. The agent may propose a severity with cited source facts, unknowns, and risk rationale. It may not finalize, downgrade, close, or suppress medium, high, or emergency incidents.
3. Medium, high, and emergency classifications require human approval before they become final business state.
4. Owner-facing incident messages are always draft-only until explicitly approved through `CustomerMessageApproval`, with manager approval for sensitive or severe cases.
5. Any classification that affects group-play eligibility, restrictions, checkout/release, medical/care handling, legal/compliance posture, or provider/customer-visible records must preserve the relevant human review gate.
6. Missing, stale, contradictory, or provider-unverified facts raise review need. They do not justify downgrading severity.
7. Emergency behavior is conservative: immediate manager/operator escalation and appropriate real-world care process. The AI is limited to structuring facts, surfacing unknowns, drafting internal packets/messages for review, and recommending approved human-run escalation steps.

## Shared incident severity packet

Every severity proposal should carry:

| Field | Requirement |
| --- | --- |
| Proposed severity | One of `info`, `low`, `medium`, `high`, `emergency`. Mark `medium`, `high`, and `emergency` as proposed until approved. |
| Confidence posture | Evidence quality such as source-backed, incomplete, conflicting, provider-unverified, or policy-dependent. Do not use model confidence as authority. |
| Source facts | Staff note IDs, provider refs, workflow event IDs, pet/reservation/customer IDs, timestamps, observed facts, immediate action taken, media refs, and policy refs. |
| Unknowns | Missing required fields, unresolved risk, who still needs to be contacted, and what must be verified before disposition. |
| Current safety state | Whether active harm, medical distress, escape risk, behavior risk, facility hazard, care deviation, or release/check-out blocker remains. |
| Required review gates | Manager, behavior, medical/care, customer-message, legal/compliance/privacy, provider mutation, or proposed incident-specific gate. |
| Staff actions | Immediate care/safety actions, evidence collection, task creation, monitoring, handoff, or manager escalation expected. |
| Notification posture | Internal staff/lead/manager routing and whether owner-facing content may be drafted, approved, sent, or suppressed. |
| Escalation timing | Due basis tied to safety state, operating milestone, checkout, medication/care window, owner communication, or approved location policy. |
| Closure blocker | What evidence or approval must exist before the incident or related review item can close. |

## Severity matrix

| Level | Typical meaning | Final approval posture | Escalation timing |
| --- | --- | --- | --- |
| `info` | Non-incident operational note or retrospective context with no safety, care, customer, eligibility, or policy effect. | May be recorded as internal context if source-backed and no owner/provider/business state is affected. | Normal backlog or next relevant shift handoff. |
| `low` | Minor incident or staff follow-up with no live safety risk, no owner notice requirement, and no eligibility/care/legal sensitivity. | May remain internal, but owner drafts and any closure/restriction/provider effects still require approval. | Same shift or before the next relevant handoff. |
| `medium` | Owner notice, manager review, care/behavior ambiguity, incomplete incident report, or customer-impacting follow-up likely. | Human approval required before final classification, owner notice, suppression, closure, or business-state mutation. | Prompt lead/manager review; before checkout/release, daily update send, next care/play decision, or other affected milestone. |
| `high` | Significant safety/behavior/health/care/customer/legal risk, active restriction, group-play suspension, injury, bite/aggression, escape/near-miss, medication/care deviation, or sensitive complaint. | Manager approval required; preserve behavior/medical/customer-message/legal gates as applicable. AI cannot downgrade or clear restrictions. | Immediate or same-operating-period manager escalation; before checkout/release and before any future eligibility decision. |
| `emergency` | Active or potentially severe animal/person safety issue, medical distress, lost/escape event, serious injury, severe bite/aggression, toxin/allergy/heat distress concern, or situation requiring real-world emergency/vet/operator process. | Emergency/operator/manager handling required. AI cannot finalize, downgrade, close, contact owner/vet autonomously, or substitute for care judgment. | Immediate manager/operator escalation and real-world care process now. |

## Level: info

### Use when

Use `info` only when the event is operationally useful context but not an incident requiring follow-up, customer notice, manager review, behavior/care review, or business-state change.

Examples:

- Staff records that a pet preferred a quieter rest area, with no distress, safety concern, owner-notice need, or future eligibility effect.
- A duplicate provider webhook arrives for an already-recorded incident with no new facts; the workflow links it as source evidence.
- A retrospective internal quality note says a daily update could have included a clearer play description, with no incident facts.
- A staff note records normal cleanup after routine play, with no injury, illness, hazard, or customer impact.

Do not use `info` for bites, attempted bites, injury, lameness, vomiting/diarrhea, medication/care deviations, escape/near-miss, customer complaints, owner-notice events, active restrictions, or unclear facts that could hide risk.

### Staff actions

- Attach the source note/event to the relevant pet, reservation, workflow event, or operating-day context.
- Deduplicate against existing incident/review items.
- Add to shift handoff only if useful for care continuity.
- Reclassify to `low` or higher if later facts show customer, safety, care, behavior, eligibility, or policy impact.

### Notification expectations

- No owner notification by default.
- No manager alert by default unless the note is part of an audit/retraining/rejected-output review.
- Internal visibility may be staff-only or shift-handoff context.
- Any owner-facing reuse of the information, including daily update copy, becomes a draft requiring `CustomerMessageApproval` if it mentions incident-like facts or sensitive observations.

### Review requirements

- Human review is not required merely to store source-backed internal context.
- Human review is required if the note is ambiguous, customer-facing, affects eligibility/care/business state, resolves a duplicate incident, or suppresses an incident-like signal.
- The agent must not use `info` to bypass incident review for unclear or sensitive source text.

### Escalation timing

- Normal backlog or next shift-handoff cadence.
- Escalate before the next operational milestone if facts change or if a staff member flags concern.

### Closure evidence

- Source reference and duplicate/suppression rationale if applicable.
- No open safety/care/customer/eligibility questions.
- No owner-facing draft pending approval.

## Level: low

### Use when

Use `low` for minor, source-backed incidents that may need internal follow-up but do not currently require owner notice, manager disposition, behavior/medical review, restrictions, provider writes, or customer-visible business changes.

Examples:

- Minor play scuffle with no injury, no escalation, no repeated behavior pattern, and staff separated/redirected successfully.
- Pet briefly resisted entering a play area but settled after staff redirection, with no stress/inability-to-settle pattern.
- Small facility/cleanup issue was resolved immediately with no pet/customer impact and no hazard remaining.
- Minor care handoff note needs staff completion, but no medication, allergy, injury, feeding, or special-care ambiguity exists.

Promote to `medium` or higher when owner notice may be appropriate, the report is incomplete, the event affects future group play/care, or the event is part of a repeated pattern.

### Staff actions

- Complete required incident fields: who/what/when/where/source, involved pet/reservation, observed facts, immediate action, and unknowns.
- Create or update an internal `IncidentFollowUp` or shift-handoff task if staff work remains.
- Monitor for recurrence during the current stay/daycare day.
- Preserve source evidence and staff attribution.
- Re-route to manager review if missing facts, repeated pattern, customer impact, or safety/care ambiguity emerges.

### Notification expectations

- Staff/lead may receive internal task or handoff routing.
- Owner notification is not automatic. If staff or policy wants an owner mention, the message is a draft and requires `CustomerMessageApproval`.
- Manager notification is optional unless local policy requires manager visibility for all incidents or the low incident remains unresolved by handoff.

### Review requirements

- Staff/lead review is expected to confirm required fields and follow-up completion.
- Manager review is required before low incidents are used to justify customer-visible decisions, eligibility changes, restriction clearance, provider writes, or suppression of an owner-notice question.
- Closure requires evidence that internal follow-up is done and no higher-severity triggers remain.

### Escalation timing

- Same shift or before the next relevant handoff.
- Before checkout/release if the note could become part of a customer conversation.
- Before the next play/care block if recurrence would affect grouping or handling.

### Closure evidence

- Complete source-backed report or explicit missing-info task.
- Staff follow-up/monitoring complete or assigned.
- No open owner-message draft, restriction, manager-review gate, behavior gate, medical/care ambiguity, or unresolved recurrence signal.

## Level: medium

### Use when

Use `medium` when the incident likely requires manager/lead review, owner notice, sensitive customer-safe wording, care/behavior review, missing-info resolution, or operational follow-up before final disposition.

Examples:

- Minor injury, lameness, vomiting/diarrhea, appetite change, or health observation that is not an emergency but needs owner-safe communication or care review.
- Behavior event such as guarding, repeated mounting, escalating chase, barrier reactivity, human selectivity, or stress/inability to settle that may affect grouping or handling.
- Incomplete staff incident report where owner notice, care implication, or restriction need cannot be ruled out.
- Customer reports after pickup that something happened during the stay/daycare day and staff need to investigate.
- Missed non-emergency care instruction or feeding deviation with no active distress but requiring review and truthful follow-up.
- Facility or supervision near-miss with no injury but a required remediation/handoff.

### Staff actions

- Preserve immediate safety/care state and collect missing facts from staff, lead, provider, media refs, or customer report.
- Create a manager/lead review item with source facts, unknowns, severity rationale, and recommended next actions.
- Create follow-up tasks such as `IncidentFollowUp`, `DocumentReview`, `PlaygroupAssessment`, `DailyUpdateDraft`, or `CustomerFollowUp` as drafts/recommendations unless auto-creation is approved by policy.
- Draft owner-facing language only as a review artifact, clearly separating observed facts from unknowns and avoiding diagnosis, blame, minimization, or promises.
- Preserve restrictions or caution flags until the correct human reviewer clears them.

### Notification expectations

- Notify or route to lead/manager review promptly through the approved internal queue/channel.
- Owner-facing notice is likely or possible, but it remains draft-only until `CustomerMessageApproval` and any required manager approval are complete.
- Staff handoff should surface current care mode, unresolved facts, active restrictions/cautions, and customer-contact state.
- If the incident blocks checkout/release, daily update, or future service decision, route it as a manager-review priority for that milestone.

### Review requirements

- Human approval is required before `medium` becomes final business state.
- Manager or lead approval is required for final severity/disposition, owner notice or suppression, closure, and any care/behavior/eligibility effect.
- `MedicalDocumentReview` or staff/manager care review is required for injury, illness, medication, allergy, feeding, or care-instruction ambiguity.
- `BehaviorReview` and/or `ManagerApproval` is required for behavior events affecting group play or future eligibility.
- `CustomerMessageApproval` is required for every owner-facing draft.

### Escalation timing

- Prompt review during the same operating period.
- Before checkout/release if the incident should be communicated or affects release conditions.
- Before the next daily update or owner-contact window if customer copy may mention the incident.
- Before the next playgroup/care decision if behavior or care handling may change.
- If facts suggest active harm, escalating health/behavior risk, escape, bite, or serious care deviation, promote to `high` or `emergency` immediately.

### Closure evidence

- Approved final classification/disposition.
- Completed or delegated missing-info and follow-up tasks.
- Approved and sent owner message, or manager-approved suppression reason.
- Review evidence for care/medical/behavior gates where applicable.
- Audit trail for any restrictions, provider writes, or business-state changes.

## Level: high

### Use when

Use `high` for significant incidents that affect safety, medical/care posture, behavior restrictions, customer trust, legal/compliance/privacy risk, or operational readiness, but where the current facts do not require immediate emergency/vet/operator process.

Examples:

- Bite, attempted bite, significant aggression, repeated escalation, or group-play suspension pending review.
- Injury, visible wound, lameness, illness, allergic concern, heat-stress concern, medication error, missed medication, food exposure, or care-plan deviation with possible harm but no confirmed emergency presentation.
- Escape attempt, lost-pet near-miss, door/gate failure, ratio/capacity breach, sanitation hazard, or unsafe facility condition.
- Customer complaint alleging harm, neglect, misinformation, or unsafe handling.
- Incident requiring restriction, special handling, checkout discussion, refund/waiver/service recovery review, legal/compliance/privacy caution, or provider-status mutation.
- Any medium incident with contradictory facts, repeated pattern, or unresolved owner-facing sensitivity that cannot safely wait for routine review.

### Staff actions

- Take or preserve immediate real-world safety steps appropriate to staff role and location SOP: separate pets, move to safe/individual care, secure area, remove from group play, prevent release until manager review when required, and collect witness/source facts.
- Escalate to manager/operator immediately through approved internal channel.
- Create a high-priority manager-review packet with timeline, source facts, unknowns, active restrictions, photos/media refs if allowed, staff actions already taken, and customer-contact state.
- Preserve or recommend restrictions such as `SuspendGroupPlayPendingReview`; do not clear them without manager approval.
- Draft owner/checkout/follow-up copy only for review. Include source-backed facts, unknowns, next-step commitment, and prohibited-language warnings.
- Create or recommend follow-up tasks for care monitoring, staff debrief, cleaning/hazard remediation, behavior review, document/care review, customer follow-up, and provider record update.

### Notification expectations

- Immediate internal manager/operator notification or queue escalation.
- Lead/staff handoff must show active restrictions, current care mode, safety state, and owner-contact status.
- Owner-facing notification is usually expected unless a manager records an approved suppression reason, but AI must not send it.
- Legal/compliance/privacy or payment/recovery routing may be needed when customer complaint, fault allegation, refund/waiver, media/privacy, or public/reputation risk exists.
- Provider writes or external notifications require approved action records and audit proof.

### Review requirements

- Manager approval is required before high classification becomes final business state.
- The agent cannot downgrade high to medium/low/info, close it, clear restrictions, reinstate group play, or suppress owner notice.
- `BehaviorReview` is required for aggression, group-play suspension, temperament/eligibility impact, or repeated pattern.
- `MedicalDocumentReview` or approved care/medical review is required for injury, illness, medication, allergy, feeding, or care-instruction ambiguity.
- `CustomerMessageApproval` plus manager approval is required for owner-facing copy.
- Legal/compliance/privacy review is required when the incident involves liability-sensitive claims, privacy/media exposure, suspected abuse/neglect, public complaint, or external reporting uncertainty.

### Escalation timing

- Immediate or same-operating-period manager/operator review.
- Before checkout/release when the incident affects owner communication or release conditions.
- Before any future group-play/service eligibility decision.
- Before any provider write, customer message, refund/waiver discussion, or public/legal response.
- Promote to `emergency` immediately if active distress, serious injury, escape/lost pet, severe aggression, toxin/allergy/heat concern, or other real-world emergency criterion appears.

### Closure evidence

- Manager-approved final severity/disposition and restriction state.
- Completed immediate safety actions or documented transfer to emergency/vet/operator process.
- Approved owner-contact outcome: sent message/call note or suppression reason.
- Completed/delegated follow-up tasks and staff handoff.
- Audit evidence for behavior, medical/care, legal/compliance, provider, payment/recovery, and customer-message decisions.

## Level: emergency

### Use when

Use `emergency` whenever source facts indicate active or potentially severe animal/person safety risk or a real-world emergency/vet/operator process may be required. When uncertain between high and emergency, route as emergency until a qualified human downgrades it.

Examples:

- Severe injury, uncontrolled bleeding, collapse, seizure-like event, breathing difficulty, heat distress, suspected toxin exposure, severe allergic reaction, or acute medical distress.
- Lost pet, active escape, unsecured animal, break-in/security threat, fire, flood, dangerous facility failure, or severe environmental hazard.
- Severe bite/aggression involving animal or human injury, ongoing fight, or immediate staff/customer safety risk.
- Medication error, food/allergen exposure, or care deviation with signs of distress or unknown serious risk.
- Customer/staff emergency report requiring immediate operator/manager intervention.
- Any incident where staff are considering vet contact, emergency transport, emergency services, or urgent owner phone call under local SOP.

### Staff actions

- Follow location emergency SOP and real-world care process immediately. This may include separating animals safely, securing the scene, contacting the manager/operator, contacting owner/vet/emergency services through approved human process, arranging transport, documenting staff actions, and preserving evidence.
- The AI may structure an emergency fact packet: timeline, involved parties, observed facts, current safety state, actions already taken, unknowns, contacts needed, and draft internal/owner scripts for human review.
- The AI may recommend that staff use approved emergency contacts and escalation tree, but it must not decide care, diagnose, contact owner/vet/emergency services autonomously, or represent that a real-world action occurred.
- Create or recommend critical follow-up tasks for manager/operator handling, care monitoring, evidence preservation, customer contact, staff debrief, and incident audit.
- Preserve all restrictions and blockers until manager/operator disposition and appropriate care review are recorded.

### Notification expectations

- Immediate manager/operator escalation is required.
- Lead/staff must be alerted through the approved urgent internal channel so real-world response is not delayed by the workflow queue.
- Owner, vet, emergency-services, legal/compliance/privacy, or provider notifications must follow approved human-run SOP. AI-generated copy is draft support only.
- The workflow should mark owner-facing drafts as blocked on `CustomerMessageApproval` and manager/operator approval unless an approved emergency procedure later grants a narrow human-confirmed send path.
- Daily briefs, shift handoffs, and review queues must surface the emergency status until an authorized human disposition closes or reclassifies it.

### Review requirements

- Emergency classification cannot become final, be downgraded, be closed, or be converted to an owner-visible disposition without manager/operator approval and audit evidence.
- Care/medical/vet review is required for injury, illness, medication, allergy, feeding, or other health/care emergency facts.
- Behavior/eligibility review is required for bite, aggression, escape, or group-play safety events.
- Customer-message approval and manager/operator approval are required for owner-facing content.
- Legal/compliance/privacy review is required when external reporting, suspected abuse/neglect, human injury, public incident, media/privacy, or liability-sensitive facts are involved.

### Escalation timing

- Now. Use immediate real-world escalation, not ordinary queue cadence.
- If the approved channel is unavailable, follow location fallback/escalation tree rather than waiting for AI.
- Human responders should update the incident packet as facts arrive, but documentation must not delay care or safety response.

### Closure evidence

- Manager/operator-approved disposition and final severity state.
- Evidence of real-world response actions taken by humans: safety secured, owner/vet/emergency contact outcome if applicable, care/transport disposition, and staff debrief.
- Approved customer-contact record or explicit pending/suppressed reason from manager/operator.
- Completed follow-up tasks, preserved audit/media/source refs, and active restrictions or care instructions carried into future operations.
- No unresolved emergency, medical/care, behavior, legal/compliance, owner-message, or provider/action gate.

## Escalation and downgrade rules

- Escalate immediately when new facts show active safety risk, medical distress, escape/lost-pet risk, bite/aggression, medication/allergen/care deviation with possible harm, customer/legal sensitivity, or unresolved facts that could materially change care or customer communication.
- Do not automatically downgrade because an incident appears resolved. Downgrade requires human approval when the current or prior proposed level is medium, high, or emergency.
- A low/info item can be promoted by deterministic rules or staff/manager signal; it cannot be used to suppress a medium/high/emergency signal.
- Repeated low events may become medium or high when they show behavior, care, quality, staffing, facility, or customer-experience pattern risk.
- Emergency remains emergency until a manager/operator with appropriate evidence records a disposition or approved reclassification.

## Notification and approval summary

| Action | Info | Low | Medium | High | Emergency |
| --- | --- | --- | --- | --- | --- |
| Internal staff note/handoff | Allowed if source-backed | Expected if follow-up remains | Required when operationally relevant | Required | Required immediately |
| Lead/manager routing | Optional unless ambiguous | Optional/required by policy or unresolved | Required before final disposition | Immediate | Immediate manager/operator |
| Owner-facing draft | Only if useful and approval-gated | Approval-gated | Approval-gated and likely | Approval-gated and manager-reviewed | Draft support only; human emergency SOP controls |
| Owner-facing send | Human-approved only | Human-approved only | Human-approved only | Human-approved only | Human-approved emergency/operator process only |
| Final classification | Internal context may be recorded | Staff/lead may close only if no sensitive effects and policy allows | Human approval required | Manager approval required | Manager/operator approval required |
| Closure | No open gates | Staff/lead evidence and no higher triggers | Manager/lead disposition and gates resolved | Manager disposition and gates resolved | Manager/operator disposition and emergency follow-up complete |
| Restriction/eligibility change | Not applicable | Human-approved if any effect | Human-approved | Manager/behavior review | Manager/operator/behavior review |
| Provider write/outbox send | Approved action required | Approved action required | Approved action required | Approved action required | Approved action required after human response |

## Implementation notes

- Store agent severity output as a proposal with evidence and review state, not as final incident state, whenever the proposal is `medium`, `high`, or `emergency`.
- Preserve both `proposed_severity` and `approved_severity` or equivalent history so human reclassification is auditable.
- Keep raw/internal narrative, customer-safe wording, and owner-send proof as separate values.
- Use task/review states such as `NeedsInfo`, `NeedsManagerReview`, `Escalated`, and `Closed` to avoid treating a draft summary as completion.
- Use due basis fields such as `immediate_safety`, `before_checkout`, `before_next_playgroup`, `before_customer_send`, `before_care_window`, or `policy_defined_sla` rather than invented times.
- When local policy is missing, record `due_basis = unresolved_policy` and route to manager instead of inventing an SLA.
- Record all approvals, suppressions, downgrades, restriction changes, customer-contact decisions, and provider writes as separate audited actions.
