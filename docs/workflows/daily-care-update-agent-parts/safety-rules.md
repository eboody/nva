# Daily care update safety and review rules

Purpose: define safety, escalation, human review, and no-send rules for the Daily Care Update Agent. This is a workflow design artifact, not live operating policy. It does not authorize autonomous customer sends, provider writes, care completion, medical decisions, incident closure, media publishing, or policy exceptions.

Source basis:

- `docs/workflows/daily-care-update-agent-parts/inputs.md` is the canonical input packet for this workflow part.
- `docs/workflows/customer-messaging-parts/send-draft-approval-policy.md` defines customer-message draft, auto-send, suppression, and review posture.
- `docs/workflows/customer-messaging-parts/tone-and-compliance-rules.md` defines customer-safe tone, grounding, medical/legal/privacy boundaries, and review checklists.
- `docs/workflows/incident-escalation-agent-parts/escalation-workflow.md` defines incident and owner-notification review gates.
- `docs/domain/petsuites/boarding/implications/06-medication-feeding-behavior-notes.md` and `docs/domain/petsuites/daycare/implications/05-pet-health-behavior-notes.md` define care-note, medication, feeding, behavior, and customer-safe projection boundaries.

Status: conservative draft rules. Anything that would send a customer-facing daily update, publish a photo, or communicate a concern to an owner remains approval-gated until a location policy, consent/media policy, template catalog, and send adapter are explicitly approved.

## Core safety stance

The Daily Care Update Agent is draft-first and review-first. It may summarize source-backed care evidence, draft routine update copy, identify missing information, recommend internal tasks, and route review packets. It must not autonomously send updates, publish media, mark care tasks complete, close incidents, diagnose health conditions, soften concerning facts into cheerful copy, or use raw internal notes as customer-facing truth.

Default behavior:

1. If all required evidence is routine, verified, current, customer-safe, non-conflicting, and covered by an approved send/template policy, the result may be an `auto_send_candidate`.
2. If facts are routine but the approved auto-send policy is missing or the copy is AI-authored, the result is `requires_review=true` and `send_mode=draft_only`.
3. If facts include illness, injury, aggression, medication issue, concerning stool/vomiting, missed feeding, staff uncertainty, complaint, incident, sensitive photo/media, privacy risk, missing evidence, stale evidence, or conflicting evidence, the result is `requires_review=true` and `should_send=false` until the appropriate human approves a final disposition.
4. If customer-visible copy cannot be made safe without hiding or minimizing a concern, the agent should suppress customer copy and create a review packet or internal task instead.

Hard review rule: the agent must require human review for illness, injury, aggression, medication issue, concerning stool/vomiting, missed feeding, staff uncertainty, or complaint. These signals are never routine auto-send content.

## Output routing fields

Every daily update result should expose explicit routing fields rather than relying on prose:

- `requires_review`: whether a staff, lead, manager, medical/document reviewer, behavior reviewer, privacy reviewer, or integration owner must act before customer delivery.
- `should_send`: whether the current result is safe to send now. For v1, this is normally `false` unless a separate deterministic send policy and approved template allow the exact case.
- `send_mode`: `auto_send_candidate`, `draft_only`, `staff_review_required`, `manager_approval_required`, `suppressed`, or `dead_letter`.
- `review_gates`: structured gates such as `CustomerMessageApproval`, `CareStaffReview`, `ManagerApproval`, `MedicalCareReview`, `MedicationReview`, `BehaviorReview`, `IncidentReview`, `PrivacyMediaReview`, `PhotoReplacementNeeded`, `NeedsMoreInformation`, or `IntegrationOwnerReview`.
- `internal_flags`: machine-readable risk/suppression flags.
- `review_reason`: concise reason codes suitable for review queues and audit search.
- `suppression_reason`: why customer delivery or customer copy is withheld when `should_send=false`.
- `evidence_refs`: care note ids, task evidence ids, media refs, incident ids, policy/template refs, reviewer refs, event ids, and idempotency keys.
- `customer_copy_status`: `not_generated`, `drafted_for_review`, `approved_for_send`, `suppressed`, `sent`, `corrected`, or `voided`.

`requires_review=false` must never be inferred from model confidence. It is allowed only when deterministic validation proves the message category, evidence, media, recipient/channel, template, policy version, and suppression checks all pass.

## Trigger taxonomy

### A. Routine draft triggers

These triggers may produce a routine draft or review packet when source evidence is clean:

| Trigger | Review posture | Send posture | Notes |
| --- | --- | --- | --- |
| `DailyUpdateNeeded` for an active reservation/day/window with routine care evidence | Staff review by default | `should_send=false` unless approved deterministic auto-send policy exists | One draft per reservation/day/window or policy-approved scope. |
| Routine meal evidence with no exception | Staff review by default | Auto-send candidate only under approved template/policy | Use only recorded facts; do not infer appetite from missing data. |
| Routine play/enrichment evidence | Staff review by default | Auto-send candidate only under approved template/policy | Avoid group-play promises or behavioral conclusions. |
| Routine bathroom/elimination note with normal status | Staff review by default | Auto-send candidate only under approved template/policy | Must be sourced; no health interpretation. |
| Approved routine photo/media ref | Staff/media review by default until media policy exists | Auto-send candidate only under approved consent/suitability policy | Photos remain sensitive until policy proves otherwise. |

Routine does not mean sendable. It only means the trigger did not itself reveal a sensitive or unresolved concern.

### B. `requires_review=true` triggers

Set `requires_review=true` whenever any of the following is present, suspected, ambiguous, stale, or contradicted by source evidence:

| Trigger class | Examples | Minimum review gate |
| --- | --- | --- |
| Illness or health concern | lethargy, coughing, sneezing, limping, eye/ear issue, skin irritation, not acting normal, staff note says `possible sick` | `MedicalCareReview` plus `ManagerApproval` for customer wording when needed |
| Injury or incident-adjacent concern | cut, scrape, swelling, limping after play, fall, bite mark, escape/near miss, facility hazard | `IncidentReview` and usually `ManagerApproval` |
| Aggression or behavior concern | bite, growl/snap, fight, rough play concern, group-play removal, fear/stress escalation, temperament restriction | `BehaviorReview` and `ManagerApproval` |
| Medication issue | missed dose, late dose, wrong/unclear medication, refusal, spit-out dose, unverified instruction, medication side effect concern | `MedicationReview` and `ManagerApproval` for owner-facing wording |
| Concerning stool/vomiting | diarrhea, blood/mucus, repeated loose stool, vomiting, retching, abnormal elimination, multiple accidents with concern | `MedicalCareReview`; manager review if owner-facing or incident-linked |
| Missed feeding or feeding exception | refused meal, partial/no intake, wrong meal, unavailable food, staff missed task, feeding instruction conflict | `CareStaffReview`; `ManagerApproval` if customer-facing, repeated, or service-recovery-sensitive |
| Staff uncertainty | `not sure`, `maybe`, `appears`, conflicting staff notes, unknown completion, unverified imported note, low-confidence media match | `NeedsMoreInformation` / `CareStaffReview` |
| Complaint or negative sentiment | owner complaint, staff conduct concern, already-sent incorrect update, bad review, service recovery, refund/credit request | `ManagerApproval`; add payment/legal/privacy gates when relevant |
| Privacy/media risk | wrong pet, other customer/pet/person visible, name tags, unsafe context, unapproved camera source, photo contradicts copy | `PrivacyMediaReview` / `PhotoReplacementNeeded`; manager for sensitive content |
| Customer-message sensitivity | medical, behavior, incident, payment, policy exception, eligibility/restriction, legal/liability, complaint language | `ManagerApproval` and category-specific gate |
| Missing or conflicting required facts | missing pet/reservation, stale care note, duplicate event conflict, unreviewed raw note, missing required media, provider mismatch | `NeedsMoreInformation`; `IntegrationOwnerReview` if system/provider issue |

A positive routine note later in the day does not clear an earlier review trigger. The concern remains gated until a human records a disposition, suppression, or approved customer-safe wording.

### C. `should_send=false` triggers

Set `should_send=false` when the current output must not be delivered to the customer as-is. This includes all `requires_review=true` cases above plus the following send-specific blockers:

| Blocker | Required result |
| --- | --- |
| No approved auto-send category/template/policy for daily updates | `send_mode=draft_only`; staff review queue |
| AI-authored freeform customer copy without human approval | `send_mode=draft_only`; preserve draft and source refs |
| Recipient/channel consent, opt-out, quiet-hours, or delivery suppression unknown/failed | `send_mode=suppressed` or staff contact review |
| Required update evidence missing, stale, duplicate-conflicted, or source-unverified | `send_mode=suppressed`; `NeedsMoreInformation` |
| Required photo unavailable, poor quality, wrong pet, privacy-risky, or mismatched with copy | `send_mode=suppressed`; `PhotoReplacementNeeded` or media review |
| Care note visibility is internal-only or raw/unreviewed | `send_mode=suppressed`; request customer-safe projection |
| Incident, health, behavior, medication, feeding exception, complaint, or policy exception is unresolved | `send_mode=suppressed`; manager/staff review |
| Draft contains diagnosis, medical advice, definitive safety claim, minimization, blame/fault, legal/liability language, payment/refund promise, booking/service guarantee, or unsupported fact | `send_mode=dead_letter` or revision review |
| Idempotency/replay cannot prove this is the intended update window or exact approved payload | `send_mode=suppressed`; integration/audit review |

`should_send=true` means the exact payload is safe to enter the approved send path now. It does not mean the agent may bypass outbox, audit, channel consent, provider response handling, or retry controls.

## Health and behavior language boundaries

Customer-facing daily updates must be warm and factual, but never medically definitive, diagnostic, minimizing, or falsely reassuring. The rule is no diagnosis, no medically definitive claims, and no minimization.

### Allowed patterns

Use source-backed observational language only after the relevant review gate allows customer-facing wording:

- `Our team noticed that Max did not finish breakfast this morning, so we are keeping an eye on him and will update you with approved next steps if needed.`
- `Bella enjoyed individual playtime today and rested afterward.`
- `Luna had a quieter day than usual, and our care team is reviewing her notes before we send today's update.`
- `A manager is reviewing a care note from today before we share the final update.`

### Disallowed patterns

Do not use:

- Diagnosis: `Max has an upset stomach`, `Bella has kennel cough`, `Luna is anxious`, `Charlie is aggressive` unless an authorized human-approved wording policy explicitly permits that exact statement.
- Veterinary advice or treatment instruction: `You should give half the dose tonight`, `No need to call a vet`, `This should clear up soon`.
- Medically definitive claims: `He is healthy`, `She is fine`, `It was not serious`, `There is no injury`, `The vomiting was just from excitement` without authorized review and evidence.
- Minimization or smoothing: `No worries`, `just a little limp`, `probably nothing`, `still had a great day` when a concern exists.
- Blame or fault: `Our staff forgot`, `another dog caused it`, `your dog started it` unless manager/legal-approved incident wording exists.
- Unsupported behavior conclusions: `failed temperament`, `unsafe for group play`, `will be fine in daycare`, `not aggressive anymore` without behavior/manager approval.

### Required separation

Daily update packets must separate:

- raw/internal note text;
- approved evidence summary;
- customer-safe draft wording;
- sensitive language needing manager approval;
- omitted/suppressed facts and why;
- review questions for staff or manager.

The customer copy must not expose raw internal notes, staff debate, provider payloads, model uncertainty, source ids, other pets/customers, or unresolved concerns.

## Auto-send eligibility

Daily care updates are draft-only by default. A future daily update may become an `auto_send_candidate` only when every condition below is true and recorded in the policy/audit packet:

1. The location has approved daily update auto-send for the exact service line, update category, channel, template id/version, and update window.
2. The copy is deterministic template-bound or a human-approved template variant; freeform AI copy is not auto-sendable.
3. The pet, reservation, customer, location, service day/window, channel, recipient, consent/opt-out, quiet-hours, and delivery-suppression state are verified and current.
4. Required care evidence is present, source-backed, current, non-conflicting, and approved for customer summary.
5. Required media, if any, has approved consent/suitability state and matches the pet and text.
6. There are no unresolved internal flags for illness, injury, aggression, medication issue, concerning stool/vomiting, missed feeding, staff uncertainty, complaint, incident, behavior restriction, privacy risk, payment/policy exception, wrong recipient, wrong pet, or source conflict.
7. The output validator confirms every customer-visible sentence maps to allowed evidence or template variables.
8. Idempotency identifies the exact reservation/day/window and prevents duplicate sends or regenerated retries.
9. Audit can preserve policy version, template version, source refs, validation result, approved payload, outbox ref, delivery result, and retry/dead-letter state.

If any condition is not provably true, route to draft/review/suppression. Do not treat lack of a risk flag as proof of auto-send eligibility when the source packet is incomplete.

## Human approval gates

### Staff / care-team review

Use staff or lead/care review for routine care evidence validation and low-sensitivity daily update drafts when no manager-gated signal is present. Reviewers should confirm:

- the evidence belongs to the right pet/reservation/day/window;
- meal/play/bathroom/rest/photo facts are accurate and customer-safe;
- raw internal note text has been converted into approved customer-safe wording;
- omitted facts are truly unnecessary rather than hidden concerns;
- any missing evidence has a task or suppression reason;
- the final draft does not imply care completion that was not recorded by an authorized staff action.

### Manager approval

Manager/admin approval is required before customer-facing language or send decisions involving:

- illness, injury, incident, safety, aggression, bite, escape/near miss, facility hazard, or behavior/eligibility impact;
- medication issue, medical/care ambiguity, concerning stool/vomiting, repeated missed feeding or care deviation;
- complaints, negative sentiment, staff conduct concern, service recovery, refund/credit/waiver/discount/payment dispute;
- privacy/legal/liability language, already-sent incorrect content, policy exceptions, restrictions, declines, or sensitive media;
- any decision to suppress an otherwise expected update because of a concern.

Manager approval must record the exact approved customer wording or suppression disposition. Approval of a draft does not itself send the message; send execution remains a separate audited outbox/provider action.

### Specialist / operational gates

Use additional gates when applicable:

- `MedicationReview`: medication name/dose/schedule/source uncertainty, missed/late/refused dose, possible medication error, or medication-related owner copy.
- `MedicalCareReview`: health concern, concerning stool/vomiting, injury, illness-like symptom, allergy/medical-condition reference, or care-instruction uncertainty.
- `BehaviorReview`: aggression, fear/stress escalation, group-play removal/reinstatement, temperament restriction, or eligibility-impacting behavior language.
- `PrivacyMediaReview`: media consent, wrong pet, other people/pets/customers, private facility details, unsafe/unflattering content, or raw camera-source concern.
- `IntegrationOwnerReview`: provider/source mismatch, duplicate/replay conflict, missing policy/template, failed validator, unavailable media adapter, or outbox/send inconsistency.

## Review queue expectations

Daily update review queues should be actionable, auditable, and prioritized. Each queued item should include:

- queue type: `care_staff_review`, `manager_review`, `medical_care_review`, `medication_review`, `behavior_review`, `privacy_media_review`, `integration_review`, or `dead_letter`;
- subject refs: location, pet, reservation, customer when needed for delivery, service line, operating day, update window;
- source refs: workflow event id/version, care note ids, task evidence ids, media refs, incident ids, policy/template refs, prior draft ids;
- reason codes: `review_reason` and `internal_flags`;
- proposed customer copy, if safe to draft, or `not_generated` with suppression reason;
- omitted/suppressed facts with explanation;
- required decision: approve final copy, revise, suppress, request more information, replace media, escalate, or dead-letter;
- due basis: before normal daily update window, before checkout, immediately for active safety/medical concerns, or policy-defined timing;
- assignee role and priority;
- idempotency key to converge duplicate notes/events into one open draft/review task.

Priority guidance:

- Critical/immediate: active safety, emergency, escape/lost pet, severe injury/illness, active medication error, urgent owner contact hold.
- High: owner-notice likely, injury/illness/medication/aggression/concerning stool-vomiting, missed feeding, privacy breach risk, checkout soon, complaint.
- Normal: routine draft review, missing non-sensitive evidence, photo replacement for expected routine update.
- Low: retrospective correction, duplicate cleanup, policy/template metadata issue with no same-day customer impact.

## Audit evidence

Every draft, review, suppression, approval, send, correction, and void should produce durable audit evidence. Minimum audit fields:

- `daily_update_id` or draft id, `workflow_event_id`, source event version/hash, idempotency key, and replay status;
- location, pet, reservation, service line, operating day/update window, and customer/recipient ref when needed;
- policy snapshot/version, template id/version, send-mode decision, validator version, and automation level;
- source/evidence refs for each customer-visible claim;
- `requires_review`, `should_send`, `review_gates`, `internal_flags`, `review_reason`, `suppression_reason`;
- reviewer actor/role, review timestamp, edits made, approval/rejection/suppression decision, and rationale;
- media refs, consent/suitability state, rejection/replacement reason, and privacy review refs where applicable;
- exact approved payload for any send, outbox ref, channel/destination ref, provider response, delivery failure/retry/dead-letter state;
- correction/void reason and link to prior draft/sent update when evidence changes after approval.

Retries may resend only the exact approved payload through the approved outbox path. They must not regenerate copy, loosen review gates, switch channels, choose replacement photos, or mark a suppressed item sendable.

## Photo/media handling

Photos and video are optional unless a paid add-on, service package, customer preference, or approved location policy requires them. Media is sensitive by default until consent, suitability, and retention policy are approved.

### Media allowed for review

A media ref may be included in an internal review packet when:

- it is represented by a `MediaRef`/document ref and minimal metadata, not raw pixels, unless an approved image-analysis workflow requires pixels;
- capture purpose, source, timestamp, and retention state are known;
- the media plausibly belongs to the right pet/reservation/day/window;
- consent/suitability state is available or explicitly marked unknown for review.

### Media blocks auto-send

Set `requires_review=true` and `should_send=false` when a photo/media item:

- shows the wrong pet or the pet cannot be confidently identified;
- includes another customer, person, staff member, other pet, name tag, private paperwork, screen, payment data, room/camera details, or other privacy risk without approved consent;
- shows injury, illness, incident context, aggression, stress, unsafe handling, sanitation/facility concern, medication/care concern, or anything a customer could reasonably read as concerning;
- is blurry, dark, cropped poorly, stale, duplicate, unavailable, retention-expired, permission-denied, or camera-offline;
- mismatches the text, such as cheerful play copy with a visibly distressed/resting/injured pet, or meal copy with no relevant meal evidence;
- conflicts with a staff note, incident, or care-watchlist signal;
- is required by policy/add-on but missing or unsuitable.

### Media outcomes

The agent should route media cases to one of these outcomes:

- `media_approved_for_customer_review`: media is suitable for human review as part of a draft packet, not necessarily auto-send.
- `photo_replacement_needed`: create/recommend a staff task to collect a new photo or approve text-only handling.
- `text_only_review_required`: media unavailable or unsuitable; staff/manager decides whether a text-only update is acceptable.
- `privacy_media_review_required`: privacy/consent risk must be reviewed before any customer use.
- `incident_or_safety_review_required`: media shows or may show a concern; route through incident/safety review.
- `media_suppressed`: media is withheld from customer use with an audit reason.

Do not describe a missing/unusable photo as if it exists. Do not substitute a prior-day photo unless policy explicitly allows it and the copy makes no misleading current-day claim.

## Internal flags and review reason examples

Example `internal_flags` values:

- `health_concern_observed`
- `injury_or_incident_signal`
- `aggression_or_bite_signal`
- `behavior_review_required`
- `medication_exception`
- `medication_instruction_unverified`
- `feeding_refusal_or_missed_feeding`
- `concerning_stool_or_vomiting`
- `staff_uncertainty_present`
- `customer_complaint_or_negative_sentiment`
- `raw_internal_note_not_customer_safe`
- `care_evidence_missing`
- `care_evidence_conflicting`
- `source_stale_or_unverified`
- `photo_required_but_missing`
- `photo_wrong_pet`
- `photo_privacy_risk`
- `photo_concern_visible`
- `photo_text_mismatch`
- `customer_message_sensitive_language`
- `auto_send_policy_missing`
- `template_policy_missing`
- `channel_consent_unknown`
- `idempotency_conflict`
- `provider_mapping_mismatch`
- `already_sent_update_needs_correction`

Example `review_reason` values:

- `illness_or_health_concern_requires_review`
- `injury_or_incident_requires_manager_review`
- `aggression_or_behavior_concern_requires_review`
- `medication_issue_requires_review`
- `concerning_stool_or_vomiting_requires_review`
- `missed_feeding_requires_care_review`
- `staff_uncertainty_requires_more_information`
- `customer_complaint_requires_manager_review`
- `routine_update_draft_requires_staff_approval`
- `daily_update_auto_send_policy_not_approved`
- `customer_copy_contains_sensitive_language`
- `raw_note_not_approved_for_customer_summary`
- `care_evidence_missing_or_stale`
- `conflicting_care_evidence_requires_review`
- `photo_required_but_unavailable`
- `photo_wrong_pet_or_identity_unclear`
- `photo_privacy_or_consent_risk`
- `photo_shows_possible_concern`
- `photo_does_not_match_update_text`
- `recipient_channel_or_consent_not_verified`
- `duplicate_or_replay_conflict_requires_audit_review`
- `provider_source_mapping_mismatch`
- `already_sent_update_requires_correction_review`

Example suppressed/no-send result:

```json
{
  "requires_review": true,
  "should_send": false,
  "send_mode": "suppressed",
  "review_gates": ["MedicalCareReview", "ManagerApproval", "CustomerMessageApproval"],
  "internal_flags": ["concerning_stool_or_vomiting", "customer_message_sensitive_language"],
  "review_reason": "concerning_stool_or_vomiting_requires_review",
  "suppression_reason": "Daily update contains a health-adjacent elimination concern and no manager-approved customer wording exists.",
  "customer_copy_status": "not_generated"
}
```

Example routine draft result:

```json
{
  "requires_review": true,
  "should_send": false,
  "send_mode": "draft_only",
  "review_gates": ["CareStaffReview", "CustomerMessageApproval"],
  "internal_flags": ["auto_send_policy_missing"],
  "review_reason": "routine_update_draft_requires_staff_approval",
  "suppression_reason": "Daily update auto-send is not approved for this location/template/channel.",
  "customer_copy_status": "drafted_for_review"
}
```

Example future auto-send candidate result:

```json
{
  "requires_review": false,
  "should_send": true,
  "send_mode": "auto_send_candidate",
  "review_gates": [],
  "internal_flags": [],
  "review_reason": null,
  "suppression_reason": null,
  "customer_copy_status": "approved_for_send"
}
```

This future state is valid only after deterministic policy confirms the approved category/template/media/channel/evidence conditions. It is not available for illness, injury, aggression, medication issues, concerning stool/vomiting, missed feeding, staff uncertainty, complaints, incidents, sensitive media, or any other unresolved concern.

## Validation checklist

Before a daily update can be considered customer-ready, deterministic validation and/or human review must answer yes to every applicable question:

1. Is the subject exactly the intended pet/reservation/day/window?
2. Does every customer-visible sentence trace to approved evidence or an approved template variable?
3. Are raw internal notes, unverified imports, and AI summaries excluded from customer copy unless human-approved as customer-safe?
4. Are illness, injury, aggression, medication issue, concerning stool/vomiting, missed feeding, staff uncertainty, complaints, incidents, behavior restrictions, and privacy/media concerns routed to review with `should_send=false`?
5. Does the copy avoid diagnosis, medical advice, medically definitive claims, minimization, blame, legal/liability language, payment/refund promises, service guarantees, and unsupported behavior conclusions?
6. If photos/media are included or required, are identity, consent, suitability, privacy, freshness, and text match verified?
7. Are recipient/channel consent, opt-out, quiet-hours, duplicate-send, and delivery-suppression checks satisfied?
8. Is the final send path covered by an approved policy/template/channel adapter, or is the result draft/review/suppression only?
9. Are reviewer decisions, source refs, policy versions, payload versions, and idempotency keys auditable?
10. If any answer is no or unknown, is the result routed to review, missing information, suppression, or dead-letter rather than send?

Conservative fallback: when facts, review state, media consent, care evidence, tone policy, source freshness, or provider state are missing, stale, contradictory, sensitive, or unverified, produce `requires_review=true` and `should_send=false`. Do not invent upbeat filler, imply unavailable photos, diagnose, hide incidents, minimize concerns, mark care complete, or send customer updates autonomously.
