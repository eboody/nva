# Incident/Escalation Agent incident type taxonomy

Purpose: define the draft incident type taxonomy used by the Incident/Escalation Agent to classify staff/customer/provider incident signals into review packets, missing-field checks, internal task recommendations, temporary restriction proposals, and owner-message drafts.

Status: draft workflow taxonomy. This document does not authorize final incident disposition, owner notification, provider writes, pet/profile mutations, eligibility changes, restriction clearance, staff-task completion, or incident closure.

## Global classification rule

AI classification is a draft signal for manager review, not a final incident disposition. The agent may propose one or more incident types with evidence, uncertainty, and recommended review gates. A human reviewer must approve any final incident disposition, medium/high/emergency classification, owner-facing message, eligibility-impacting flag, provider mutation, restriction clearance, or closure.

Use source-backed classification only:

- Preserve raw staff/customer/provider text as evidence references; do not rewrite it into final customer language.
- Separate internal facts, customer-safe drafts, manager-review packets, and approved customer communications.
- If multiple types apply, keep all material types rather than forcing a single label. Example: a dog bite with puncture wound is both `bite/aggression` and `injury`; a missed medication that causes symptoms is both `medication issue` and `illness`.
- Missing, stale, contradictory, provider-unverified, legally sensitive, medically ambiguous, owner-facing, or eligibility-impacting facts create review work, not clearance.
- A low/note-only draft classification never overrides explicit policy hard stops such as bite/aggression, escape/near-miss, medical/care ambiguity, emergency/vet escalation, customer complaint with disputed facts, or staff safety risk.

## Shared fields for every incident type

Every incident record or review packet should capture these common fields when available:

- Incident id/source reference, source system, source actor, reporter, and reporter role.
- Location, operating day, observed-at timestamp, report-created timestamp, and time zone.
- Pet id, customer id, reservation/attendance/stay id, current service/care mode, room/yard/playgroup/suite, and staff on duty.
- Narrative/raw report reference, structured fact summary, attachments/media references, and redaction notes.
- Immediate staff action already taken, current care mode/restriction, current owner-notification state, open staff tasks, and unresolved unknowns.
- Draft severity candidate, review gates, likely owner-notice posture, and whether manager review is required before any eligibility-impacting flag.

## Temporary flags and manager-review policy

Temporary pet/profile flags are proposed workflow outputs, not autonomous writes. Until a dedicated profile-flag aggregate exists, model them as proposed incident-linked restrictions/notes with audit evidence and explicit review state.

Temporary flags may be proposed for these reasons:

- Safety or eligibility: `GroupPlaySuspendedPendingReview`, `IndividualCareOnlyPendingReview`, `TemperamentReassessmentRequired`, `ManagerApprovalRequiredBeforeGroupPlay`, `ManagerApprovalRequiredBeforeCheckIn`.
- Health or care: `CareReviewRequiredBeforeAttendance`, `MedicalDocumentReviewRequired`, `MedicationPlanReviewRequired`, `FeedingPlanReviewRequired`, `AllergyExposureReviewRequired`.
- Handling or operations: `SpecialHandlingRequiredPendingReview`, `EscapeRiskReviewRequired`, `StaffSafetyHandlingReviewRequired`, `CustomerFollowUpRequired`, `IncidentFollowUpRequired`.

Manager review is required before any eligibility-impacting flag is applied, cleared, downgraded, or used to deny/reinstate group play. For medical/care ambiguity, preserve `MedicalDocumentReview` or equivalent staff/manager care review. For behavior or safety ambiguity, preserve `BehaviorReview` and/or `ManagerApproval`. AI may draft the proposed flag and rationale; it must not finalize the flag as an eligibility-impacting disposition.

## Incident types

### 1. injury

Scope:

An observed or reported physical injury, wound, lameness, soreness, bleeding, swelling, impact, fall, cut, scrape, puncture, bite wound, nail injury, paw/limb issue, heat-related physical concern, or other trauma/physical condition connected to care, play, facility conditions, grooming/training/boarding/daycare activity, or customer report after pickup. Injury classification describes observed facts only; it is not a diagnosis.

Examples:

- Staff observes a limp after group play or after jumping from a cot/platform.
- Small cut, scrape, torn nail, bleeding paw pad, swelling, or abrasion noticed during care.
- Bite wound or puncture discovered during or after a bite/aggression event.
- Customer reports soreness, bruising, or injury after pickup.
- Pet is overheated, collapses, or has acute distress after activity.

Data fields that matter:

- Body area affected, observed signs, approximate size/severity, bleeding/swelling/lameness/distress, and whether signs are worsening.
- When and where noticed, last-known-normal time, likely activity/context, and witnesses.
- Pet handling status, current care mode, whether separated/rested, whether owner/vet/emergency escalation was initiated by staff/manager.
- Related behavior/incident type, involved pets/staff, photos/media refs, staff notes, and customer-after-pickup report details.
- Known medical conditions, allergies, medication/care profile, veterinarian/emergency contact refs, and review-state of any care documents.

Likely immediate staff actions:

- Separate or rest the pet safely as needed; stop group play if injury happened in group play or the cause is unknown.
- Notify lead/manager for severity routing and emergency/vet escalation if acute, worsening, severe, or ambiguous.
- Document observed facts, time, location, witnesses, and immediate care steps; attach approved photo/media references if policy allows.
- Draft owner-notice packet only for review; do not diagnose or recommend treatment.

Common follow-up tasks:

- `IncidentFollowUp` for complete report and status check.
- `DocumentReview` or care/medical review when injury intersects known medical conditions or unclear care instructions.
- `CustomerFollowUp` / approved owner notice draft.
- `DailyUpdateDraft` only as review-gated customer-safe wording.
- `PlaygroupAssessment` if injury happened in group play or was caused by another pet/handling context.

Temporary flags and review:

- May propose temporary flags: `IndividualCareOnlyPendingReview`, `CareReviewRequiredBeforeAttendance`, `MedicalDocumentReviewRequired`, `IncidentFollowUpRequired`, and, if group play context is implicated, `GroupPlaySuspendedPendingReview`.
- Manager review is required before any eligibility-impacting restriction is applied or cleared. Medical/care ambiguity also requires care/document review. Emergency or severe injury cannot be downgraded or closed by the agent.

### 2. illness

Scope:

Observed or reported acute illness, abnormal health signs, contagious-disease concern, vomiting, diarrhea, coughing, sneezing, lethargy, collapse, seizure-like observation, allergic reaction, heat stress, appetite/energy abnormality with health concern, or customer report after pickup that suggests illness during or adjacent to care. Illness classification records observations; it must not diagnose.

Examples:

- Vomiting, diarrhea, repeated coughing, nasal discharge, unusual lethargy, or refusal to rise.
- Suspected allergic reaction, hives/swelling, heat stress, or breathing difficulty.
- Customer reports illness symptoms after pickup and asks whether anything happened during the stay.
- Staff notices symptoms that may affect other pets, sanitation, isolation, or pickup timing.

Data fields that matter:

- Observed symptoms, frequency/count, severity, duration, last-known-normal, and whether symptoms are worsening.
- Location/room/yard, contact with other pets, sanitation/cleanup actions, isolation/separation status.
- Known medical conditions, allergies, medications, vaccines/document status when relevant, food/treat exposure, and care-plan deviations.
- Owner/vet/emergency contact state, staff/manager notified, and whether pickup/vet escalation was initiated by a human.
- Attachment/media refs and raw notes separated from customer-safe summary.

Likely immediate staff actions:

- Move pet to an appropriate safe/rest/isolation care mode per staff policy; preserve comfort and supervision.
- Notify lead/manager for illness severity, contagious-risk, owner-contact, and vet/emergency routing.
- Document facts and timing; do not diagnose or create treatment instructions.
- Route ambiguous care/medical facts to review rather than changing the care plan autonomously.

Common follow-up tasks:

- `IncidentFollowUp` for symptom timeline and status.
- `DocumentReview` / care review for medical records, vaccine/health documents, medications, allergies, or care instructions.
- `CustomerFollowUp` / owner notice draft requiring approval.
- `DailyUpdateDraft` only when reviewed and customer-safe.
- Cleaning/sanitation or room/yard follow-up task if policy supports it.

Temporary flags and review:

- May propose temporary flags: `CareReviewRequiredBeforeAttendance`, `MedicalDocumentReviewRequired`, `IndividualCareOnlyPendingReview`, `IncidentFollowUpRequired`, and possibly `ManagerApprovalRequiredBeforeCheckIn` for unresolved contagious, severe, or unclear health concerns.
- Eligibility-impacting or attendance-impacting flags require manager/staff care review before application or clearance. AI must not diagnose, determine contagiousness, approve return-to-care, or close emergency/medical incidents.

### 3. bite/aggression

Scope:

Bites, attempted bites, snapping, lunging, escalating chase, repeated mounting with escalation, guarding, sustained intimidating behavior, barrier/kennel reactivity, human-directed aggression, dog-directed aggression, incompatible playgroup behavior, or any behavior incident that may affect group-play safety, staff handling, customer notice, or eligibility.

Examples:

- Bite or attempted bite toward another pet, staff member, or customer.
- Guarding toys/space/food, escalating chase, pinning, repeated mounting, or failure to disengage.
- Growling/snapping when handled, leashed, kenneled, moved, fed, medicated, or approached.
- Barrier reactivity or human selectivity that changes handling requirements.
- Customer reports a bite mark after pickup possibly connected to daycare.

Data fields that matter:

- Trigger/context, target, involved pets/people, room/playgroup, staff present, roster/ratio snapshot, and playgroup assignment.
- Behavior observed before/during/after, escalation level, body language notes, whether contact occurred, whether injury occurred, and whether the pet could be redirected.
- Prior temperament profile, behavior observations, group-play eligibility evidence, previous incidents/restrictions, and current care mode.
- Immediate separation/rest/individual-care action, owner-notice status, manager/lead notified, and media/evidence refs.

Likely immediate staff actions:

- Stop the interaction and separate safely; move pet(s) to individual care/rest lane when warranted.
- Notify lead/manager and preserve facts for behavior review.
- Document all involved pets/people, injuries if any, and whether owner notice is likely required.
- Do not clear group play, downgrade severity, or apply final blame/disposition autonomously.

Common follow-up tasks:

- `PlaygroupAssessment` or temperament reassessment.
- `IncidentFollowUp` for full behavior timeline and involved-party details.
- `CustomerFollowUp` / owner notice draft requiring approval.
- Manager review packet for disposition, restrictions, and future handling.
- Daily brief/watchlist entry for active restrictions or safety risk.

Temporary flags and review:

- May propose temporary flags: `GroupPlaySuspendedPendingReview`, `TemperamentReassessmentRequired`, `ManagerApprovalRequiredBeforeGroupPlay`, `IndividualCareOnlyPendingReview`, `SpecialHandlingRequiredPendingReview`, and `IncidentFollowUpRequired`.
- Requires manager review before any eligibility-impacting flag is applied, cleared, or used to reinstate group play. Bite/aggression hard stops cannot be `DocumentOnly` and cannot be closed or downgraded by AI.

### 4. escape attempt

Scope:

Escape, attempted escape, near-miss elopement, gate/door breach, yard/room/suite/kennel breach, leash slip, fence/gate manipulation, pet found outside expected area, or supervision/facility condition that creates escape risk. Include actual escapes and near misses.

Examples:

- Dog rushes a gate/door and crosses a threshold or nearly exits.
- Pet slips leash during transfer, check-in, checkout, playgroup movement, or potty break.
- Gate/door left unsecured, latch failure, fence gap, or room/suite breach discovered.
- Pet repeatedly attempts to climb/jump/dig/push out of an enclosure.

Data fields that matter:

- Exact location/path, containment point, actual vs attempted escape, duration, whether pet left secure area, and recovery status.
- Staff present, handoff/transfer context, ratio/supervision snapshot, door/gate/facility condition, weather/time, and witnesses.
- Pet behavior/temperament, prior escape risk, leash/handling requirements, injuries or staff safety issues, and other pets affected.
- Immediate containment actions, facility fix/escalation, manager notified, owner-notice posture, and audit/media refs.

Likely immediate staff actions:

- Secure pet and area immediately; move pet to safer handling/care mode as needed.
- Notify lead/manager; escalate emergency/safety protocol if pet is loose, missing, injured, or outside secure area.
- Document containment failure/near-miss facts and preserve facility evidence.
- Create follow-up for facility/supervision review where applicable.

Common follow-up tasks:

- `IncidentFollowUp` for timeline and containment facts.
- Facility/property maintenance or safety review task.
- `PlaygroupAssessment` or handling review if behavior contributed.
- `CustomerFollowUp` / owner notice draft requiring approval.
- Daily brief/watchlist entry for active escape-risk handling.

Temporary flags and review:

- May propose temporary flags: `EscapeRiskReviewRequired`, `SpecialHandlingRequiredPendingReview`, `IndividualCareOnlyPendingReview`, `ManagerApprovalRequiredBeforeCheckIn`, and possibly `GroupPlaySuspendedPendingReview` if group play/yard context is implicated.
- Manager review is required before any attendance, handling, or group-play eligibility flag is applied or cleared. Actual escape, missing-pet, injury, or severe near-miss should be treated as high/emergency candidate until reviewed.

### 5. medication issue

Scope:

Missed, late, partial, extra, wrong, refused, vomited, spilled, undocumented, ambiguous, expired, unavailable, or conflicting medication administration; medication storage/labeling concern; medication instruction mismatch; or medication-related customer/staff report. Includes medication-adjacent allergy or health escalation when the core issue is medication handling.

Examples:

- Scheduled medication missed, given late, given at uncertain time, or not documented.
- Pet refuses medication, vomits after administration, or staff cannot confirm dose.
- Medication label/instructions conflict with profile or owner message.
- Medication unavailable, expired, mislabeled, or assigned to wrong pet risk.

Data fields that matter:

- Medication name as source text/reference, dose/instructions as source text/reference, scheduled time, actual/attempted time, staff actor, and administration evidence.
- Variance type: missed/late/refused/wrong/extra/unclear/storage/label mismatch.
- Pet health signs, allergies/contraindications, known medical conditions, owner/vet contact refs, and current care profile review state.
- Immediate staff action, manager/lead notified, owner-contact posture, and whether emergency/vet escalation was initiated by a human.

Likely immediate staff actions:

- Pause assumptions and escalate to lead/manager/care reviewer when medication facts are unclear or concerning.
- Record exact facts and source instructions; do not infer a new dose, timing, or treatment plan.
- Separate owner-facing explanation from raw medication notes; all owner messages require approval.
- Create review task before any care-profile change or task completion if evidence is incomplete.

Common follow-up tasks:

- `DocumentReview` / medication-plan review.
- `IncidentFollowUp` for medication variance timeline.
- `CustomerFollowUp` / owner notice draft requiring approval.
- `MedicationAdministration` corrective/verification task only if assigned and approved by staff workflow.
- Manager review for severe, ambiguous, wrong-medication, or health-impacting events.

Temporary flags and review:

- May propose temporary flags: `MedicationPlanReviewRequired`, `CareReviewRequiredBeforeAttendance`, `MedicalDocumentReviewRequired`, `IncidentFollowUpRequired`, and possibly `ManagerApprovalRequiredBeforeCheckIn` for unresolved serious ambiguity.
- Medication/care flags affecting attendance/readiness require staff/manager care review before application or clearance. AI must not change medication instructions, decide treatment, mark medication tasks complete, or close medication incidents autonomously.

### 6. feeding issue

Scope:

Missed, late, partial, extra, wrong, refused, vomited, contaminated, unavailable, ambiguous, conflicting, or allergy-sensitive feeding/treat event; feeding instruction mismatch; food exposure; resource guarding around food when the primary issue is feeding/care; or customer/staff report about feeding concerns.

Examples:

- Meal missed, served late, wrong food/treat given, or feeding amount uncertain.
- Pet refuses food, vomits after eating, eats another pet's food, or receives unapproved treats.
- Allergy or restricted-diet exposure concern.
- Feeding instructions conflict between profile, reservation notes, and owner message.

Data fields that matter:

- Feeding instruction source, scheduled feeding time, actual/attempted time, food/treat type/source, amount, staff actor, and documentation evidence.
- Variance type: missed/late/wrong/extra/refused/vomited/allergy exposure/conflicting instructions/unavailable food.
- Known allergies, medical conditions, medication timing dependencies, appetite history, and owner-provided instructions.
- Immediate action, health signs, owner/manager notification posture, and related illness/medication issue links.

Likely immediate staff actions:

- Record exact feeding facts and variance; do not invent revised feeding instructions.
- Escalate allergy exposure, vomiting, repeated refusal, or medically sensitive feeding issues to lead/manager/care review.
- Preserve evidence and create staff follow-up for missing/conflicting instructions.
- Draft customer-safe explanation only for approval.

Common follow-up tasks:

- `DocumentReview` / care-profile feeding review.
- `IncidentFollowUp` for timeline and variance.
- `CustomerFollowUp` / owner notice draft requiring approval.
- `DailyUpdateDraft` for reviewed customer-safe status when appropriate.
- Staff task to clarify feeding instructions before next attendance/stay.

Temporary flags and review:

- May propose temporary flags: `FeedingPlanReviewRequired`, `CareReviewRequiredBeforeAttendance`, `AllergyExposureReviewRequired`, `IncidentFollowUpRequired`, and sometimes `IndividualCareOnlyPendingReview` for feeding safety/monitoring needs.
- Eligibility/readiness-impacting flags require staff/manager care review before application or clearance. AI must not change feeding instructions or determine medical significance.

### 7. bathroom concern

Scope:

Bathroom, elimination, continence, stool/urine, accident, frequency, blood/mucus observation, straining, diarrhea, constipation concern, urination concern, marking issue, sanitation issue tied to a pet, or customer report after pickup that may affect care, health review, cleaning, owner notice, or profile notes.

Examples:

- Diarrhea, repeated accidents, blood in stool/urine, straining, or no bathroom output when expected.
- House-soiling/marking in a room or play area that creates sanitation or handling needs.
- Customer reports abnormal stool/urination after pickup and asks for staff observations.
- Bathroom pattern conflicts with care instructions or medication/feeding context.

Data fields that matter:

- Observed output type, frequency/count, appearance using non-diagnostic descriptors, time/location, cleanup/sanitation actions, and whether signs are repeated/worsening.
- Pet's feeding/medication/health profile, known conditions, recent illness/feeding issues, and owner/customer report details.
- Staff witnesses, photos/media refs only if policy allows, affected area, other pets exposed, and owner-notice posture.
- Whether issue is health-like, sanitation-only, behavior/marking, or mixed.

Likely immediate staff actions:

- Document objective observations, clean/sanitize according to policy, and monitor/separate if health or sanitation risk warrants.
- Notify lead/manager/care reviewer for blood, repeated diarrhea/vomiting, straining, severe symptoms, contagious concern, or conflicting facts.
- Avoid diagnosis and avoid telling owners what medical action to take.

Common follow-up tasks:

- `IncidentFollowUp` or care-note review.
- `DocumentReview` / care review if health/medical context matters.
- Cleaning/sanitation follow-up task if supported.
- `CustomerFollowUp` / owner notice draft requiring approval for significant or repeated concerns.
- `DailyUpdateDraft` with reviewed customer-safe phrasing where appropriate.

Temporary flags and review:

- May propose temporary flags: `CareReviewRequiredBeforeAttendance`, `IndividualCareOnlyPendingReview`, `IncidentFollowUpRequired`, and possibly `MedicalDocumentReviewRequired` for unresolved health concerns.
- Attendance/readiness-impacting or medical flags require staff/manager care review before application or clearance. Minor sanitation-only incidents may be internal-note-only, but repeated/severe/ambiguous health facts require review.

### 8. customer complaint

Scope:

A customer-reported concern, dissatisfaction, dispute, allegation, request for explanation, refund/credit-adjacent incident, after-pickup report, service-quality complaint, communication complaint, staff-conduct concern, care concern, safety concern, or conflicting owner/staff account connected to an incident or possible incident.

Examples:

- Customer reports an injury, illness, missing medication, feeding concern, or behavior issue after pickup.
- Customer disputes staff explanation, requests manager callback, refund, credit, or formal response.
- Customer complains about staff behavior, communication, facility conditions, wait time, lost belongings, or pet care quality.
- Customer says prior instructions were ignored or that they were not notified about an event.

Data fields that matter:

- Customer identity/contact, pet/reservation, complaint channel, received-at time, staff recipient, exact customer statement/reference, requested resolution, and urgency.
- Related incident(s), staff notes, care records, media/docs, message history, refund/payment context, and disputed/unknown facts.
- Sentiment/severity indicators, legal/liability/medical/behavior/payment sensitivity, and whether owner notice or manager callback is requested.
- Current communication state: draft, approved, sent, waiting for manager, or blocked by missing facts.

Likely immediate staff actions:

- Acknowledge receipt only through approved staff process; route sensitive/disputed complaints to manager.
- Preserve the customer's statement and do not overwrite staff facts or choose one account without review.
- Gather source records and create manager review packet.
- Do not draft promises, admissions, legal conclusions, refunds/credits, or medical advice except as approval-gated draft language.

Common follow-up tasks:

- `CustomerFollowUp` / manager callback task.
- `IncidentFollowUp` to connect complaint to operational facts.
- `DocumentReview`, care review, or `PlaygroupAssessment` if complaint references medical/behavior/care issues.
- Refund/payment exception review task if compensation is requested.
- Manager daily brief/customer-experience risk entry.

Temporary flags and review:

- May propose temporary flags only when complaint includes source-backed care/safety/behavior facts: `IncidentFollowUpRequired`, `CustomerFollowUpRequired`, `ManagerApprovalRequiredBeforeCheckIn`, `CareReviewRequiredBeforeAttendance`, or `GroupPlaySuspendedPendingReview` depending on underlying facts.
- Manager review is required before applying any eligibility-impacting flag from a complaint, especially when facts are disputed or customer-reported after pickup. Complaint classification itself should not become a behavior/medical flag without reviewed evidence.

### 9. staff safety

Scope:

Any incident or near miss involving risk of injury, threat, unsafe handling, staff-directed aggression, unsafe facility condition affecting staff, unsafe customer interaction, hazardous material/cleaning exposure, staffing/ratio condition that creates safety risk, or staff injury/near miss during pet care. Staff safety may overlap with bite/aggression, escape attempt, property damage, injury, or customer complaint.

Examples:

- Pet bites, snaps at, lunges at, knocks down, corners, or otherwise endangers staff.
- Staff is injured during handling, leash transfer, medication, feeding, grooming, boarding, daycare, cleaning, or containment.
- Facility hazard creates risk: broken gate, slippery floor, broken kennel, exposed sharp edge, chemical/cleaning hazard.
- Customer conduct creates staff safety concern or requires manager/owner/admin escalation.

Data fields that matter:

- Staff actor(s), injury/near-miss details, pet/customer/facility involved, location, task being performed, witnesses, and immediate safety response.
- Behavior/handling context, prior restrictions, staffing/ratio, equipment/facility state, and whether work stopped or area was secured.
- Internal-only HR/safety/legal sensitivity, manager/owner/admin escalation state, and separation from ordinary customer-safe narrative.
- Related incident types and active restrictions/tasks.

Likely immediate staff actions:

- Secure the scene and pet/customer interaction safely; stop unsafe activity.
- Notify lead/manager and route staff injury, severe threat, or customer-conduct issue through internal safety procedures.
- Document objective facts and preserve privileged/internal details separately from customer-facing drafts.
- Do not assign blame, legal liability, or final disciplinary/staff conclusions.

Common follow-up tasks:

- `IncidentFollowUp` with safety packet.
- Manager/owner/admin safety review task.
- `PlaygroupAssessment` or handling review for pet-related staff risk.
- Facility/property maintenance task if hazard-related.
- Customer communication draft only if manager-approved and customer-safe.

Temporary flags and review:

- May propose temporary flags: `StaffSafetyHandlingReviewRequired`, `SpecialHandlingRequiredPendingReview`, `IndividualCareOnlyPendingReview`, `ManagerApprovalRequiredBeforeCheckIn`, `GroupPlaySuspendedPendingReview`, and `TemperamentReassessmentRequired` when pet behavior is involved.
- Manager review is required before applying or clearing eligibility/handling flags. Staff HR/safety/legal facts may require owner/admin/compliance review and should not be exposed to ordinary prompts or customer drafts unless explicitly approved.

### 10. property damage

Scope:

Damage to resort property, customer property, staff property, pet belongings, rooms/suites/yards/doors/gates/fences/equipment, cleaning/sanitation assets, or third-party property during care. Include damage caused by pets, facility failure, staff handling, weather, or unknown cause when operational follow-up is needed.

Examples:

- Pet damages kennel, suite, door, gate, fence, bed, toy, leash, bowl, grooming/training equipment, or another customer's belongings.
- Broken latch, fence gap, damaged flooring, sharp edge, or equipment failure discovered during operations.
- Customer reports lost/damaged belongings after pickup.
- Property damage creates escape risk, injury risk, sanitation issue, or service disruption.

Data fields that matter:

- Property item/area, owner of property, damage description, estimated severity, location, time noticed, suspected/known cause, pet/customer/staff involved, and witnesses.
- Safety/escape/injury implications, whether area/equipment is still in use, photos/media refs, maintenance status, and related incident types.
- Customer communication/payment/refund/charge sensitivity, manager approval state, and audit evidence.

Likely immediate staff actions:

- Secure unsafe area/item and stop using damaged equipment if needed.
- Notify lead/manager for safety, customer-property, payment/refund, or facility implications.
- Document objective facts and create maintenance/follow-up task.
- Do not promise reimbursement, charge a customer, admit liability, or send owner-facing explanation without approval.

Common follow-up tasks:

- Facility/property maintenance or repair task.
- `IncidentFollowUp` linking damage to pet/care/facility facts.
- `CustomerFollowUp` / owner notice draft requiring approval for customer property or pet-caused damage.
- Payment/refund/charge exception review task if money is involved.
- Safety review task if damage creates injury/escape/staff-safety risk.

Temporary flags and review:

- May propose temporary flags: `IncidentFollowUpRequired`, `SpecialHandlingRequiredPendingReview`, `EscapeRiskReviewRequired`, `StaffSafetyHandlingReviewRequired`, or `ManagerApprovalRequiredBeforeCheckIn` if damage indicates handling, safety, or facility risk. Pet behavior-related damage may also propose `TemperamentReassessmentRequired` or `GroupPlaySuspendedPendingReview`.
- Manager review is required before any eligibility-impacting flag, customer charge/refund, liability-sensitive communication, or restriction clearance. Pure facility damage may create maintenance tasks without pet/profile flags unless linked to pet behavior, escape risk, staff safety, or customer/property dispute.

## Cross-type review matrix

| Incident type | Temporary pet/profile flags may be proposed? | Manager review before eligibility-impacting flag? | Typical review gates |
| --- | --- | --- | --- |
| injury | Yes, for care review, individual care, medical document review, incident follow-up, or group-play suspension when relevant. | Yes. | ManagerApproval, MedicalDocumentReview, CustomerMessageApproval; BehaviorReview if behavior/group play involved. |
| illness | Yes, for care review, medical document review, individual care, attendance/check-in review, or incident follow-up. | Yes. | MedicalDocumentReview, ManagerApproval, CustomerMessageApproval. |
| bite/aggression | Yes, commonly group-play suspension, temperament reassessment, individual care, special handling, manager approval before group play. | Yes; never AI-final. | BehaviorReview, ManagerApproval, CustomerMessageApproval. |
| escape attempt | Yes, escape-risk review, special handling, individual care, manager approval before check-in/group play. | Yes. | ManagerApproval, CustomerMessageApproval; BehaviorReview if pet behavior contributed. |
| medication issue | Yes, medication/care review, medical document review, check-in review, incident follow-up. | Yes when readiness/attendance/eligibility is affected. | MedicalDocumentReview, ManagerApproval, CustomerMessageApproval. |
| feeding issue | Yes, feeding/care review, allergy exposure review, individual monitoring/care, incident follow-up. | Yes when readiness/attendance/eligibility is affected. | MedicalDocumentReview or staff care review, ManagerApproval, CustomerMessageApproval. |
| bathroom concern | Sometimes, for care review, individual care/monitoring, medical document review, or incident follow-up. | Yes when readiness/attendance/eligibility is affected. | MedicalDocumentReview or staff care review, ManagerApproval for severe/ambiguous cases, CustomerMessageApproval. |
| customer complaint | Sometimes, only when source-backed care/safety/behavior facts warrant it. | Yes; complaint alone is not final evidence for eligibility impact. | ManagerApproval, CustomerMessageApproval, RefundOrDepositException if money is involved, MedicalDocumentReview/BehaviorReview as applicable. |
| staff safety | Yes, for special handling, staff-safety review, individual care, group-play suspension, temperament reassessment. | Yes. | ManagerApproval, BehaviorReview when pet-related, CustomerMessageApproval for customer copy; owner/admin/compliance for privileged safety/legal facts. |
| property damage | Sometimes, if damage implies safety, escape, handling, behavior, or check-in risk. | Yes when the flag affects pet eligibility/handling. | ManagerApproval, CustomerMessageApproval for customer/property communication, RefundOrDepositException if money is involved, BehaviorReview if pet behavior is implicated. |

## Common follow-up task vocabulary

Use existing task kinds where possible and label richer task names as proposed until implemented:

- `IncidentFollowUp`: complete report, gather missing facts, confirm immediate response, track unresolved incident state.
- `PlaygroupAssessment`: behavior/group-play reassessment, temperament review, compatibility review, handling plan review.
- `DocumentReview`: medical/care/medication/feeding/allergy/vaccine or customer-provided document review.
- `MedicationAdministration`: only for authorized staff workflow; never auto-completed by incident classification.
- `DailyUpdateDraft`: customer-safe daily update draft requiring approval when it contains incident facts.
- `CustomerFollowUp`: owner notice, manager callback, complaint response, or follow-up request draft requiring approval.
- Proposed facility/safety/maintenance tasks: use only as draft/internal task recommendations until a canonical task kind exists.
- Proposed refund/payment exception tasks: preserve `RefundOrDepositException` or manager approval gates when customer complaint/property damage involves money.

## Final boundaries

The taxonomy is intentionally conservative. It helps the Incident/Escalation Agent produce source-grounded draft classification, missing-field checks, review packets, and task recommendations. It must not be used as a final policy engine for severity, liability, diagnosis, customer communications, provider writes, pet eligibility, profile flags, restriction clearance, or incident closure.
