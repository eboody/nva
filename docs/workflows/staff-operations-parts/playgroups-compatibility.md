# Playgroups and compatibility

Purpose: define staff-operations support for playgroup and compatibility decisions where automation collects evidence and suggests safe groupings, but staff or managers confirm every grouping before it becomes operational. This is a workflow definition, not live resort policy and not approval for autonomous playgroup assignment.

Status: draft staff-operations artifact. AI may prepare evidence, draft candidates, and explain review gates. It must not confirm group play, override exclusions, reinstate a pet after an incident, send customer-facing behavior language, or mutate provider/reservation state without the required human approval.

## Source inputs

Canonical upstream input packet: `docs/workflows/staff-operations-parts/inputs.md`.

Primary source anchors used here:

- `docs/domain/petsuites/daycare/implications/02-group-assignment.md` for assignment triggers, eligibility, roster/care-lane outputs, staff confirmation, review gates, and customer-message boundaries.
- `docs/domain/petsuites/daycare/implications/03-staff-to-pet-ratios.md` for staff coverage, lane-specific capacity, ratio review, and manager override boundaries.
- `docs/domain/petsuites/boarding/implications/06-medication-feeding-behavior-notes.md` for behavior-note review, special handling, sensitive report language, and incident/customer-message escalation.
- Current domain anchors from `inputs.md`: `entities::Pet`, `CareProfile`, `TemperamentProfile`, `Reservation`, `StaffTaskKind::PlaygroupAssessment`, `StaffTaskKind::IncidentFollowUp`, `StaffRole`, `ActorRef`, `AuditEvent`, `workflow::WorkflowEvent`, `policy::ReviewGate`, and `operations::daycare` assignment/coverage concepts proposed by daycare implication docs.

Open source caveats inherited from `inputs.md`:

- Product-map and final data-model artifacts are missing from the repo; this document uses current repo docs/code and kanban handoff metadata as canonical until those artifacts are restored.
- Exact location policies for playgroup labels, staff-to-pet ratio, compatibility thresholds, manager authority, and customer phrasing are unresolved.
- Missing, stale, conflicting, or provider-unverified facts route to review. They do not default to group play.

## Approval posture

Human approval gate: playgroup suggestion automation.

Until explicitly approved:

- AI output is suggestion-only.
- Staff must confirm all groupings and same-day reassignments.
- Manager approval is required for overrides, capacity/ratio exceptions, incident restriction changes, reinstatement after suspension, or sensitive customer-facing language.
- Provider mutations, reservation status changes, staff schedule changes, and customer sends remain outside this workflow unless a separately approved bounded tool/action performs them.

Suggested internal state labels:

| State | Meaning | Allowed next human action |
| --- | --- | --- |
| `EvidenceIncomplete` | Required compatibility data is missing, stale, or contradictory. | Staff collects or reviews source facts. |
| `NeedsStaffReview` | A candidate might be possible but behavior/group-fit requires human review. | Playgroup/daycare staff accepts, changes, or rejects. |
| `AutomationProposed` | AI has prepared one or more candidate groupings with rationale and gates. | Staff confirms one candidate or rejects with reason. |
| `StaffConfirmed` | Authorized staff confirmed the grouping for the scoped date/window. | Execute roster; continue observation. |
| `RejectedByStaff` | Staff rejected the candidate. | Choose alternative lane/group, individual care, or manager review. |
| `ManagerReviewRequired` | Override, incident, ratio/capacity, or sensitive language gate exists. | Manager approves, denies, suspends, or requests more evidence. |
| `TemporarilySuspended` | Incident/behavior/safety restriction blocks group play. | Manager/lead review before reinstatement. |
| `IndividualCareRecommended` | Group play is not currently safe/available but individual care may be. | Staff confirms individual lane if capacity/care gates pass. |
| `WaitlistOrCapacityHold` | Capacity/staffing facts do not support placement. | Manager/front desk resolves roster, staffing, or customer follow-up. |

## Required compatibility inputs

The suggestion engine and staff confirmation UX should show source/provenance for every input. Unknown values should be visible as review gaps rather than hidden behind a score.

### Pet and profile facts

Required:

- Pet identity, species, age/life stage, sex, spay/neuter status where policy makes it relevant.
- Size or weight band, with source and last-reviewed timestamp.
- Temperament profile: group-play observation, temperament rating, people/dog orientation when available, intro-assessment status, and staff-entered notes.
- Care profile facts that alter group safety: medications that affect timing/handling, allergies, medical conditions, feeding/food-guarding concerns, handling instructions, escape risk, or quiet-room needs.
- Behavior flags and incident history: bite history, dog selectivity, human selectivity, anxiety, food guarding, escape risk, requires-manager-review, unresolved incident restrictions, suspension/reinstatement status, and incident follow-up tasks.
- Manager-only notes that may constrain compatibility without exposing raw details to all staff or customers.

### Relationship facts

Required where available:

- Exclusions / do-not-pair list: pet ids or provider identifiers, reason category, source, scope, expiry/review date, and approving actor.
- Known friends / preferred companions: pet ids or provider identifiers, observation source, confidence, date/window scope, and whether staff has confirmed the relationship recently.
- Prior group history: previously confirmed playgroups, rejected candidates, same-day reassignments, rest/split periods, and incident-linked invalidations.

### Reservation and stay facts

Required:

- Reservation id, service kind/variant, requested date/window, check-in or active-stay state, and whether dog group play, hybrid room/play, individual dog care, or cat enrichment applies.
- Vaccine/document eligibility state and hard stops that prevent attendance or group play.
- Add-ons or care obligations that conflict with play windows, such as medication timing, meals, grooming/training handoff, or rest periods.
- Current room/care-lane assignment if hybrid or individual care is involved.

### Occupancy and staffing facts

Required if available; otherwise review-gated:

- Current/planned roster snapshot for the date/window.
- Existing playgroups/care lanes, lane capacity, current assignments, and any locked staff decisions.
- Scheduled staff, qualification/role coverage, break/absence flags, and staff-to-pet ratio decision for each candidate lane.
- Capacity/coverage risk: sufficient, insufficient, unknown, or manager-override-required.
- Location/day policy snapshot or explicit caveat that policy is missing.

## Suggestion workflow

### 1. Build the evidence packet

Trigger when one of these occurs:

- Daycare/day-play reservation is requested, offered, confirmed, checked in, or active.
- Staff edits temperament, size, behavior flags, manager notes, incident state, do-not-pair list, preferred companions, vaccine status, care notes, or spay/neuter status.
- Operating-day prep builds the next roster.
- Same-day staffing/capacity changes alter safe grouping.
- Incident or behavior observation invalidates a current assignment.

Evidence packet contents:

- Source refs for pet, reservation, profile, care, temperament, incident, manager-note, roster, staff schedule, policy, and prior assignment records.
- Freshness markers for temperament/group-play observations, known friends, exclusions, roster snapshot, and staff coverage snapshot.
- Review gates raised by missing/stale/conflicting facts.
- Redacted rationale fields for staff-facing display; raw sensitive notes stay behind permissioned detail views.

### 2. Decide care mode before matching companions

The workflow must first decide whether the service path is eligible for dog group play at all:

- Dog group-play variants may receive playgroup candidates only if eligibility, care notes, and staff coverage are resolved enough for staff review.
- Dog individual day boarding can be recommended when group play is ineligible but individual care is safe and staffed.
- Hybrid play-and-room requires both play/enrichment capacity and room/rest capacity.
- Cat care never enters dog playgroup matching; it uses individual enrichment/room handling.
- Missing or contradictory evidence returns `NeedsStaffReview`, `ManagerReviewRequired`, `IndividualCareRecommended`, or `WaitlistOrCapacityHold`, not an automatic playgroup.

### 3. Generate candidate groupings

For each pet/window, AI may suggest candidates that include:

- Candidate playgroup/care lane and date/window.
- Compatibility rationale based on size band, life stage, energy/temperament, staff-observed group comfort, known friends, do-not-pair exclusions, behavior flags, incident status, and care constraints.
- Explicit blockers and warnings, including unknown size/temperament, stale assessment, incident pending review, capacity/staffing unknown, or manager-only note present.
- Staff coverage decision and roster/capacity effect.
- Confidence as a review aid only; never as a final authority.
- Alternative path: different playgroup, split/rest period, individual day boarding, assessment task, waitlist, or manager review.

Candidate generation must apply hard exclusions before positive affinity:

1. Species/service-mode mismatch.
2. Vaccine/document/hard-stop or attendance ineligibility.
3. Unresolved incident suspension or manager-only restriction.
4. Do-not-pair exclusions.
5. Care/medical/handling notes that block or alter group play.
6. Staff coverage/capacity insufficiency.
7. Stale/unknown temperament, size, or group observation.
8. Preferred companions / known friends as positive evidence only after safety gates pass.

### 4. Staff confirmation

Staff-facing UX should make the human decision primary:

- Roster board grouped by operating day/time window and care lane.
- Candidate cards with pet name/id, size/life stage band, temperament band, known friends, exclusions, behavior/care warnings, staffing/capacity status, and source/freshness chips.
- Side-by-side “why suggested” and “why review needed” sections.
- One-click actions: `Confirm grouping`, `Move to another group`, `Assign individual care`, `Needs assessment`, `Reject suggestion`, `Escalate to manager`, `Mark unavailable/waitlist`.
- Required reason capture for every non-confirm action and every override.
- Role-aware detail views: manager-only notes visible only to manager/authorized lead roles; staff sees a safe constraint label such as “manager restriction present” when they lack detail permission.
- No bulk auto-confirm while the approval gate is active. Bulk actions may only mark selected suggestions for staff review or apply staff-confirmed choices with explicit actor identity.

Confirmation requirements:

- Staff actor id, role, timestamp, operating-day/window, assignment id, roster snapshot id, and displayed evidence version.
- Staff must acknowledge any warnings that remain open.
- Manager approval token/event is required before confirming an override candidate.
- Staff rejection must preserve the AI candidate and rationale in audit rather than deleting it.

## Override and rejection reasons

Use typed reason categories; allow a short staff note behind a redacted/sensitive boundary.

Staff rejection reasons:

- `SizeMismatch`
- `EnergyMismatch`
- `TemperamentMismatch`
- `KnownExclusion`
- `KnownFriendNotEnoughEvidence`
- `BehaviorFlagRequiresAssessment`
- `IncidentPendingReview`
- `CareOrMedicalConstraint`
- `MedicationOrFeedingTimingConflict`
- `StaffCoverageConcern`
- `CapacityConcern`
- `NeedsIntroAssessment`
- `StaleOrMissingEvidence`
- `ManagerRestrictionPresent`
- `CustomerOrReservationStateBlocks`
- `OtherStaffJudgment`

Manager override reasons:

- `ApprovedRatioOrCapacityException`
- `ApprovedIncidentReinstatement`
- `ApprovedPolicyException`
- `ApprovedCareAccommodation`
- `ApprovedDoNotPairModification`
- `ApprovedSensitiveCommunicationLanguage`
- `ApprovedWaitlistRelease`

Override constraints:

- Overrides must be scoped to a pet, reservation/date/window, playgroup/care lane, and policy snapshot.
- Overrides expire or require re-review according to local policy; if no policy exists, treat as same-day only.
- Overrides preserve the original deterministic denial/review reason and do not rewrite it as “eligible”.

## Audit trail

Every evidence packet, suggestion, confirmation, rejection, reassignment, invalidation, incident feedback update, and manager override should record:

- Assignment/suggestion id, reservation id, pet id, customer id if available, location id, operating day/window.
- Service kind/variant, requested care mode, candidate playgroup/care lane, and final confirmed lane if different.
- Required inputs used: size, temperament, exclusions, known friends, behavior flags, incident history, manager notes presence, reservation/stay state, roster, capacity, staff coverage, and policy refs.
- Source refs and freshness for each input.
- AI/tool actor for suggestions; staff/manager actor for human decisions.
- Review gates required, satisfied, rejected, or still open.
- Typed rationale, rejection reason, or override reason.
- Customer-visible language state: none, draft, approved, sent by external system, or suppressed.
- Invalidation links when a later event makes the assignment stale.

Audit redaction rules:

- Staff notes, manager-only notes, raw incident narratives, and sensitive behavior labels must not appear in customer-safe logs or broad staff summaries.
- Debug/log values should follow the existing `temperament::StaffNote` pattern: semantic labels and redacted details.
- Manager-only rationale should be accessible in a permissioned view while the general roster shows only the operational constraint.

## Safety escalation

Escalate to manager or lead review when any of these appear:

- Bite history, aggression signal, serious dog/human selectivity, escape risk, food/resource guarding, injury, or safety incident.
- Incident pending follow-up, temporary suspension, or requested reinstatement.
- Do-not-pair conflict, manager-only restriction, or staff disagreement about fit.
- Medical, medication, allergy, feeding, or handling issue that affects group safety.
- Unknown/stale temperament for a pet requested for group play.
- Staff coverage insufficient/unknown, capacity limit exceeded/unknown, or ratio override requested.
- Customer-facing language would mention behavior, incident, injury, medical, eligibility refusal, or safety.

Escalation outputs:

- `PlaygroupAssessment` task for temperament/group-fit review.
- `IncidentFollowUp` task for unresolved behavior/safety incidents.
- `CheckInPrep` or `CustomerFollowUp` task when front desk needs missing info or owner follow-up.
- Manager daily-brief risk row for capacity/staffing/incident-heavy playgroup risk.
- Customer-message draft only when communication is needed and review-gated.

## Incident feedback loop

Incident and observation events must update future suggestions without allowing AI to make final safety decisions.

Loop:

1. Staff records incident/behavior observation with pet ids, involved group/lane, time, severity, source evidence, and immediate handling action.
2. Current assignment is invalidated or marked `TemporarilySuspended` when the incident affects group safety.
3. Follow-up task is created for lead/manager review.
4. Compatibility packet gains new evidence: incident status, do-not-pair changes, behavior flags, special handling, reassessment due date, or reinstatement conditions.
5. Future suggestions show the incident-derived restriction and route to manager review until cleared.
6. Manager/staff disposition records whether the pet remains suspended, moves to individual care, requires intro-only assessment, may rejoin with restrictions, or has an approved reinstatement.
7. Customer-facing incident language stays draft/suppressed until manager-approved.

Incident feedback must preserve history; clearing a restriction does not delete the incident, the staff rejection, or the original candidate.

## Customer-visible vs internal-safe language

Internal-safe language can be operationally direct, but still redacted by role:

- “Needs intro assessment before group play.”
- “Do-not-pair restriction with another checked-in pet.”
- “Manager restriction present; see permissioned note.”
- “Incident follow-up pending; group play suspended until manager review.”
- “Coverage unknown for large-dog high-energy lane; roster review needed.”

Customer-visible draft language should be practical, non-diagnostic, and approved before sending:

- “Our team will complete a playgroup assessment before confirming group play.”
- “We need updated records before confirming today’s play plan.”
- “We recommend an individual enrichment option today so our team can provide the safest care.”
- “A manager will follow up with the best care plan for today.”

Do not expose to customers without explicit manager-approved wording:

- Raw behavior labels such as “aggressive”, “bite risk”, “dog selective”, or “food guarding”.
- Internal staff counts, exact ratio thresholds, or employee names.
- Unreviewed incident conclusions, blame, medical speculation, or safety guarantees.
- Manager-only notes or do-not-pair counterpart details.

## Suggested downstream data/model needs

This workflow can initially be represented with existing task/audit primitives and the daycare implication contracts, but later implementation likely needs first-class types for:

- `CompatibilityEvidencePacket`
- `CompatibilityInputFreshness`
- `DoNotPairRestriction`
- `KnownCompanionPreference`
- `ManagerOnlyCompatibilityConstraint`
- `PlaygroupSuggestion`
- `PlaygroupCandidateRationale`
- `StaffConfirmationOutcome`
- `SuggestionRejectionReason`
- `CompatibilityOverrideReason`
- `AssignmentInvalidationReason`
- `CustomerLanguageSensitivity`

Semantic invariants for later implementation:

- A playgroup suggestion is not an assignment until staff confirmation exists.
- A manager override is not eligibility; it is an exception with scope, reason, actor, and expiry/review policy.
- Known friends can improve candidate ranking only after exclusions, incident restrictions, care gates, and staff coverage pass.
- Do-not-pair restrictions and incident suspensions are hard blockers unless manager policy explicitly permits an override.
- Missing/stale compatibility inputs produce review tasks, not inferred compatibility.
- Customer-safe text is generated from reviewed outcomes, not raw staff notes or AI rationale.

## Verification checklist for this workflow

A future implementation or UX spec should be considered aligned only if it demonstrates:

1. AI suggestions remain visibly pending staff confirmation.
2. Every candidate shows size, temperament, exclusions, known friends, behavior/incident state, manager-note presence, reservation/stay state, and staffing/capacity status or an explicit missing-data reason.
3. Staff can confirm, move, reject, assign individual care, or escalate with typed reasons.
4. Manager-only notes and incident details are permissioned and redacted from customer-safe surfaces.
5. Do-not-pair and incident restrictions beat positive affinity/friend matches.
6. Staff coverage/capacity uncertainty blocks confirmation and routes to review.
7. Audit records preserve AI suggestion, human action, sources, review gates, and override/rejection reasons.
8. Customer-facing language is draft/review-gated and avoids raw internal behavior labels.
9. Incident feedback invalidates stale assignments and affects future suggestions until reviewed.
10. The playgroup suggestion automation approval gate remains explicit in product/release readiness decisions.

Doc-only status: this artifact defines the workflow contract. It does not change code, schemas, live resort operations, provider data, customer messaging, staff schedules, or playgroup policy.