# Daily Care Update Agent

Purpose: define the canonical integration artifact for the Daily Care Update Agent. This document synthesizes the part files under `docs/workflows/daily-care-update-agent-parts/` into one implementation-facing workflow contract.

Status: conservative design artifact. It does not authorize autonomous customer sends, provider/PMS writes, care-task completion, medication verification, incident disposition, medical advice, media publication, or policy exceptions. Until an approved location/template/channel/media policy exists, customer-facing daily updates are drafts or review packets only.

Source detail is preserved in:

- `docs/workflows/daily-care-update-agent-parts/inputs.md`
- `docs/workflows/daily-care-update-agent-parts/staff-note-capture.md`
- `docs/workflows/daily-care-update-agent-parts/tone-guide.md`
- `docs/workflows/daily-care-update-agent-parts/output-schema.md`
- `docs/workflows/daily-care-update-agent-parts/safety-rules.md`
- `docs/workflows/daily-care-update-agent-parts/example-transformations.md`

## 1. Purpose and operating boundary

The Daily Care Update Agent turns source-backed staff care notes, task evidence, and approved media references into a structured daily update packet for a pet/reservation/day/window. It supports Pawgress-style daily updates for active boarding/daycare care execution while preserving source evidence, review state, and auditability.

The agent may:

- summarize routine care evidence into concise customer-safe draft text;
- identify which source facts were included, omitted, or suppressed;
- recommend staff, manager, media, medication, medical/care, behavior, privacy, or integration review;
- recommend internal tasks such as collecting missing evidence, replacing/reviewing a photo, or resolving contradictory notes;
- return a validated `WorkflowResult<DailyCareUpdateOutput>`-style payload for persistence by the application.

The agent must not:

- send or publish a customer update by itself;
- attach or publish photos/media by itself;
- mark feeding, medication, play, bathroom, cleaning, incident, handoff, or photo tasks complete;
- diagnose illness, interpret symptoms, give veterinary/medical advice, or make medication decisions;
- close incidents, resolve complaints, approve refunds/credits, or make policy/legal/liability statements;
- copy raw internal staff notes, provider payloads, staff debate, or unreviewed sensitive facts into customer copy;
- invent cheerful filler, meals, play, bathroom events, medication status, photos, staff initials, or reassurance to make a message feel complete.

Workflow events that may invoke this agent are `DailyNoteCreated` and `DailyUpdateNeeded`. Duplicate/replay events must converge on the same location + reservation + pet + service date + update-window draft/review packet unless the evidence version changed.

## 2. Required inputs

The runtime should construct a least-privilege prompt packet from typed, minimized inputs. Raw provider payloads, raw photos, full customer PII, payment data, and unrelated staff/labor detail should not be included unless an approved policy explicitly requires them.

### Staff note format

Staff notes are operational evidence, not customer copy. Each note used for daily-update drafting should carry:

- identity: `note_id`, location, pet, reservation/stay when applicable, customer ref only when needed for draft linkage;
- provenance: source kind/ref, author actor or source system, observed/created/updated timestamps, source version/hash;
- classification: meal/feeding, play/activity, mood/comfort, bathroom, medication, photo/media, concern, mixed, or unknown;
- visibility state: internal-only, customer-safe candidate, customer-approved source, restricted-sensitive, or suppressed from customer;
- review state: recorded/internal, needs more information, needs staff review, needs manager review, approved for customer summary, sent/published, or corrected/voided;
- structured observations by category when present;
- raw note text only as redacted internal evidence, never as final customer copy;
- customer-safe summary candidate only when staff-approved or explicitly marked as a draft candidate;
- evidence refs and audit refs sufficient for replay, correction, and validation.

### Photo policy

Photos and videos are optional unless a paid add-on, service package, customer preference, or approved location policy requires them. Media is sensitive by default until consent, suitability, retention, and customer-use policy are approved.

Prompt packets should include media refs and minimal metadata, not raw pixels, unless a separate approved image-analysis workflow needs pixels. Photo/media facts should include capture purpose, media ref, review state, suitability/quality state, consent/policy state when known, and unavailable/rejection reason when relevant.

The agent must never imply that a photo exists or is attached unless an approved media ref is present. Wrong-pet, blurry, stale, privacy-risky, sensitive, camera-offline, permission-denied, retention-expired, or text-mismatched media requires review or replacement rather than auto-send.

### Tone/brand guide

Daily updates should sound like a calm, observant care team member: warm, honest, concise, transparent, source-backed, and not overly cutesy. Drafts should use observed behaviors and approved care evidence rather than inferred feelings or causes.

Allowed routine topics, when source-backed and policy-safe, include meals, play/enrichment, rest, mood/demeanor, bathroom/elimination, care-plan adherence, and approved photo/media mention. The copy should usually be one to three short sentences for portal/SMS-style updates, with source refs and review details kept out of customer text.

Avoid diagnosis, medical advice, guarantees, unsupported reassurance, blame/fault, legal/liability language, payment/refund/booking promises, staff-defensive language, other pet/customer identities, unapproved staff identifiers, and claims about photos or care completion that are not supported.

### Customer messaging policy

Customer-facing daily updates are approval-gated unless and until a deterministic approved template/send policy exists for the exact location, service line, channel, category, template version, consent/opt-out state, media policy, and update window.

High-risk messages involving health, injury, medication, behavior/aggression, incidents, safety, complaints, payment, policy exceptions, legal/liability topics, privacy/media concerns, or unresolved evidence require staff/manager approval before customer wording or delivery. The AI output may recommend `should_send=true` only for a future policy-approved auto-send path; it is never itself a send command.

### Care note model

No first-class `CareNote` aggregate currently exists in `domain/src`; implementation should preserve the parent handoff semantics until the domain model is added:

- CareNote is pet/reservation-scoped observation history that may produce customer-visible message drafts.
- Lifecycle states are `Draft`, `Recorded/Internal`, `NeedsReview`, `ApprovedForCustomerSummary`, `Sent/Published`, and `Corrected/Voided`.
- Pet owns longitudinal care notes; Reservation ties stay-scoped notes, tasks, messages, incidents, documents, and audit events together.
- AI may suggest, draft, and summarize; verified human-approved state is required for customer summaries, sent/published state, corrections, sensitive facts, and care completion.

Current implementation anchors include `Pet`, `CareProfile`, `MedicationInstruction`, `TemperamentProfile`, `Reservation`, `StaffTaskKind::DailyUpdateDraft`, `PetCareWatchReason`, `WorkflowEventType::DailyNoteCreated`, `WorkflowEventType::DailyUpdateNeeded`, `AllowedAction::SummarizeCareNotes`, `DraftMessageRequest`, and `MediaSnapshotRequest`.

## 3. Staff note capture fields and validation

Every note eligible for daily-update evidence should include the capture envelope below. Unknown values should be represented explicitly rather than left blank when the field is relevant.

| Field group | Required content | Validation behavior |
| --- | --- | --- |
| Subject linkage | `note_id`, `location_id`, `pet_id`, active `reservation_id` or explicit profile-level status, `service_line`, update window when known | Ambiguous pet/reservation/window requires staff review before draft inclusion. |
| Source provenance | `source_kind`, `source_ref`, author/source actor when known, observed/created timestamps, source version/hash for imports/replays | Missing provenance blocks auto-send and may block customer copy. |
| Classification | Meal, play/activity, mood/comfort, bathroom, medication, photo/media, concern, mixed, or unknown | Unknown/mixed notes need classification or staff review. |
| Visibility and review state | Internal-only, customer-safe candidate, customer-approved source, restricted-sensitive, suppressed; recorded/internal, needs review, approved, sent, corrected/voided | Internal-only or restricted-sensitive content cannot be copied to customers. |
| Structured observations | Category payloads for meals, activity, mood, bathroom, medication, media, and concerns | Required category fields must be populated or marked unknown/not observed. |
| Concern flags | Empty array allowed only when staff selected no concerns and validations pass | Any concern flag routes to review/suppression according to severity. |
| Evidence/audit refs | Task ids, care-note versions, media refs, incident ids, policy snapshots, audit events | Customer-visible claims must cite stable refs. |

Category validations:

- Meals: require meal slot when known, offered amount when known, ate amount, appetite, and feeding exception state. Refusal, vomiting, wrong food, allergy/diet ambiguity, missed feeding, or contradictory meal evidence requires review.
- Play/activity: require activity type, session status, engagement when observed, and restriction/review state. Aggression, group-play removal, reactivity, incidents, safety concerns, or conflicting behavior notes require behavior/manager review.
- Mood/comfort: require observed demeanor when present and an explicit mood concern flag. Anxiety/stress labels, unusual lethargy, injury-adjacent observations, or staff uncertainty require review.
- Bathroom: require urination and BM observed states when bathroom evidence is present. Diarrhea, blood/mucus, repeated loose stool, vomiting, repeated accidents, straining, or abnormal elimination requires review.
- Medication: require reviewed medication/schedule refs when known, administration status, authorized staff actor for administered/skipped/refused claims, and exception state. Skipped/refused/uncertain/wrong-dose/side-effect-like facts require medication/manager review.
- Photos/media: require photo requirement state, media ref when available, capture purpose, quality/suitability state, unavailable reason when required media is absent, and media review state. Media is not auto-published by the agent.
- Concerns: represent concern type, severity, internal description, customer-copy allowance, recommended action, and complaint ref when applicable.

Corrections and voids must invalidate or re-review drafts that used the older source version. AI summaries and free-text evidence may support drafting but cannot prove care completion.

## 4. Daily update tone guide

Customer copy should be warm, factual, concise, and grounded:

- Say what staff observed: “Milo ate most of breakfast,” not “Milo loved breakfast.”
- Use modest, specific positives: “Bella enjoyed her scheduled playtime,” not “best day ever.”
- Make neutral details calm and proportionate: “Cooper took a little extra time to settle, then rested quietly.”
- Preserve transparency in review metadata when sensitive or missing facts are omitted from customer copy.
- Keep customer copy free of source ids, raw internal notes, reviewer debate, or policy jargon.

Preferred patterns:

- “Hi Jordan — Milo had a good day with us. He joined his scheduled playtime, rested quietly afterward, and ate most of dinner.”
- “Luna took a little extra time to settle this morning and rested quietly later in the day.”
- “Today’s update is text-only because no approved customer-use photo is available yet.”
- Internal only: “Needs staff review: today’s meal evidence is incomplete and should not be used in customer copy yet.”

Avoid or reject:

- “best day ever,” “perfect all day,” “nothing to worry about,” “completely fine,” “guaranteed,” “we promise,” “definitely,” “cured,” “diagnosed”;
- unsupported labels such as “sick,” “anxious,” “depressed,” “aggressive,” or “failed group play”;
- blame or defensive wording such as “your dog caused it,” “our staff forgot,” “staff mistake,” or liability language;
- “photo attached” without an approved media ref;
- “normal” for health, behavior, meals, or bathroom unless `normal` is an approved source value.

If a warm, honest, concise, source-backed update cannot be produced without diagnosis, guarantees, blame, unsupported reassurance, privacy risk, or hidden sensitive facts, the agent should produce a review reason, suppression reason, or internal task recommendation instead of customer-ready copy.

## 5. Output schema

The agent output should be embedded as the `structured_output` payload of a `WorkflowResult<DailyCareUpdateOutput>`-style result. The application owns parsing, deterministic validation, persistence, review routing, idempotency, audit logging, and any later side effect.

Required top-level `DailyCareUpdateOutput` fields:

```json
{
  "customer_message": {
    "body": "Milo had a relaxed afternoon and enjoyed his individual play session. He ate dinner as noted by the care team.",
    "channel_hint": "portal",
    "template_id": null,
    "language": "en-US",
    "tone": "warm_concise_factual",
    "media_refs": [],
    "audience": "customer",
    "redaction_profile": "customer_safe_daily_update_v1"
  },
  "internal_flags": [],
  "should_send": false,
  "requires_review": true,
  "review_reason": "customer_message_approval_not_configured",
  "included_facts": [],
  "omitted_facts": []
}
```

### `customer_message`

Object containing the proposed customer-facing daily update, or an explicit null-body message object for no-send/suppression cases.

Required subfields: `body`, `channel_hint`, `template_id`, `language`, `tone`, `media_refs`, `audience`, and `redaction_profile`.

Validation:

- `body` may be `null` only when `should_send=false` and review/no-send/suppression applies.
- Non-null `body` must be concise, customer-safe, and source-grounded.
- Every factual sentence or media mention must map to `included_facts`.
- No raw internal notes, provider payloads, medical advice, legal/payment content, other pet/customer identity, unapproved staff identifiers, unsupported claims, or unapproved photo claims may appear.
- `media_refs` may include only approved media/document refs, never raw blobs or direct camera snapshots.

### `internal_flags`

Array of machine-readable reason objects for review, suppression, audit, and downstream task creation.

Recommended shape:

```ts
type InternalFlag = {
  code: string;
  severity: "info" | "needs_staff_review" | "needs_manager_review" | "do_not_send" | "runtime_error";
  message: string;
  source_refs: string[];
  recommended_action: "none" | "staff_review" | "manager_review" | "collect_more_info" | "replace_or_review_photo" | "create_internal_task" | "suppress_update" | "dead_letter";
};
```

Recommended codes include `customer_message_approval_not_configured`, `missing_required_update_evidence`, `missing_or_unapproved_photo`, `poor_or_sensitive_photo`, `incident_or_safety_signal`, `medical_or_medication_review_required`, `feeding_exception_review_required`, `behavior_review_required`, `raw_internal_note_not_customer_safe`, `unverified_provider_or_customer_claim`, `conflicting_staff_notes`, `care_task_completion_not_verified`, `policy_gap_requires_review`, `opt_out_or_suppression`, `duplicate_or_replay_no_new_send`, and `validation_failed_safe`.

### `should_send`

Boolean recommendation that the draft is eligible for delivery after deterministic validators and approval gates have passed. It is not a send command.

Default v1 value is `false`. `true` is valid only when `requires_review=false`, `review_reason=null`, customer copy is non-null, every fact is source-backed and policy-approved, no non-info internal flags exist, recipient/channel/consent/media/template policy is approved, and deterministic runtime validation recorded an approval or safe-send policy ref.

### `requires_review`

Boolean that routes the output to staff/manager review before customer delivery. It must be `true` for review-gated evidence, missing policy, sensitive facts, missing/stale/conflicting input, unapproved media, incident/safety signals, medication/health/behavior concerns, customer-message approval gaps, validation warnings, or do-not-send explanations.

### `review_reason`

Concise operator-facing string or `null`. Required and non-empty when `requires_review=true`; must be `null` when `requires_review=false`.

Recommended values include `customer_message_approval_not_configured`, `missing_required_update_evidence`, `missing_or_unapproved_photo`, `photo_quality_or_privacy_review_required`, `incident_or_safety_review_required`, `medical_or_medication_review_required`, `feeding_exception_review_required`, `behavior_review_required`, `raw_note_requires_customer_safe_projection`, `conflicting_or_stale_evidence`, `policy_gap_requires_review`, `consent_or_opt_out_blocks_send`, `duplicate_or_replay_no_new_send`, `validation_failed_safe`, and `not_applicable_no_update_due`.

### `included_facts`

Array of source-backed facts used in `customer_message.body` or `customer_message.media_refs`.

Each included fact should record `fact_id`, `fact_kind`, `customer_text_span`, `normalized_value`, stable source refs with source kind/id/version/observed time/actor/visibility, customer visibility, review state, freshness, and confidence. Empty is valid only when `body=null` or when a deterministic template placeholder carries no model-authored factual claim.

### `omitted_facts`

Array of source-backed facts available to the agent but excluded from customer copy.

Each omitted fact should record `fact_id`, `fact_kind`, omission reason, stable source refs, review state, and an operator-safe staff note. Omitted incident, medical/health, medication exception, feeding exception, behavior concern, unsafe photo, missing-photo, policy/consent, internal-only, conflicting, stale, or unverified facts must be represented here or in `internal_flags` when they affect review, suppression, or customer interpretation.

Cross-field invariants:

1. All seven top-level fields are required.
2. `requires_review=true` implies non-empty `review_reason` and normally `should_send=false`.
3. `requires_review=false` implies `review_reason=null`.
4. `customer_message.body=null` implies `should_send=false`.
5. Any non-info `internal_flags` item implies `should_send=false` and usually `requires_review=true`.
6. Every customer-visible factual claim must map to `included_facts` with stable source refs.
7. Unsafe, sensitive, internal, unverified, stale, or conflicting facts must not silently disappear; represent them in `omitted_facts` or `internal_flags`.
8. AI output cannot complete care tasks, publish media, send messages, or create provider side effects.

## 6. Safety and review rules

The hard rule: the Daily Care Update Agent must require human review for illness, injury, aggression, medication issue, concerning stool/vomiting, missed feeding, staff uncertainty, and complaint. These signals are never routine auto-send content.

Set `requires_review=true` and `should_send=false` whenever any of the following is present, suspected, ambiguous, stale, or contradicted:

| Trigger | Examples | Minimum gate |
| --- | --- | --- |
| Illness or health concern | lethargy, coughing, sneezing, limping, eye/ear issue, skin irritation, “possible sick,” unusual low energy | Medical/care review plus manager approval for customer wording when needed |
| Injury or incident-adjacent concern | cut, scrape, swelling, bite mark, fall, escape/near miss, facility hazard, limping after play | Incident/safety review and usually manager approval |
| Aggression or behavior concern | bite, growl/snap, fight, rough play concern, group-play removal, fear/stress escalation, temperament restriction | Behavior review and manager approval |
| Medication issue | missed/late/wrong/unclear medication, refusal, spit-out dose, unverified instruction, possible side effect | Medication review and manager approval for owner-facing wording |
| Concerning stool/vomiting | diarrhea, blood/mucus, repeated loose stool, vomiting/retching, abnormal elimination, repeated accidents | Medical/care review; manager review if owner-facing or incident-linked |
| Missed feeding or feeding exception | refused meal, no/partial intake of concern, wrong meal, unavailable food, missed task, feeding instruction conflict | Care staff review; manager approval when repeated, sensitive, or customer-facing |
| Staff uncertainty | “not sure,” “maybe,” subject ambiguity, conflicting notes, unverified import, possible wrong-pet media | Needs more information / staff review |
| Complaint or negative sentiment | owner complaint, update-quality concern, bad review, service recovery, refund/credit request, staff conduct concern | Manager approval, with payment/legal/privacy gates as relevant |
| Privacy/media risk | wrong pet, other people/pets/customers, name tags, unsafe context, unapproved camera source, photo contradicts text | Privacy/media review and photo replacement if needed |
| Missing/conflicting required facts | missing pet/reservation/window, stale note, duplicate conflict, unreviewed raw note, required media absent | Needs more information or integration-owner review |

A positive routine note later in the day does not clear an earlier review trigger. The concern remains gated until a human records a disposition, suppression, or exact approved customer wording.

Disallowed customer language includes diagnosis, veterinary advice, medically definitive claims, minimization, unsupported reassurance, blame/fault, legal/liability statements, policy/payment promises, behavior eligibility conclusions, and unsupported group-play safety claims.

## 7. Example transformations

These examples show the intended transformation pattern. Use the part file for the full set of source examples and JSON packet variations.

### Routine notes to draft packet

Input notes:

- `AM breakfast: ate all.`
- `Group play: chased ball, took rest break.`
- `Potty walk complete: pee and normal stool.`
- approved current-day media ref `media:milo-yard-001`.

Safe output behavior:

- draft concise customer copy using only those facts;
- include the media ref only because it is approved;
- keep `should_send=false` and `requires_review=true` unless a deterministic auto-send policy exists;
- cite every sentence in `included_facts`.

Example customer draft:

> Milo had a good day with us. He enjoyed group play, chased the ball, and took a rest break. He ate all of his breakfast and had a normal potty walk. We also have a photo from his play time ready for review.

### Shy or settling-in pet

Input notes:

- `Luna shy/hiding early, accepted slow approach, came out for treats in PM.`
- internal-only handling note: `Do not rush; may swat if cornered.`

Safe output behavior:

- be honest about settling without diagnosis or alarm;
- omit the internal handling note from customer copy and record it in `omitted_facts`;
- require review if behavior language could affect safety or eligibility.

Example customer draft:

> Luna is still settling in, and our team is giving her the quiet, patient approach she prefers. She was shy earlier today, then came out for treats this afternoon.

### Limited evidence

Input note:

- `Bella played fetch and relaxed in shade.`
- meal and bathroom status missing.

Safe output behavior:

- draft only the play fact if the service/location policy permits limited updates;
- omit meals/bathroom rather than inventing them;
- flag missing required evidence for staff review.

Example customer draft:

> Bella enjoyed time in the daycare yard today. She played fetch and relaxed in the shade.

### Vague note that must be held

Input note:

- `Rocky okay.`
- required photo missing.

Safe output behavior:

- no customer copy;
- record the vague note and missing photo in `omitted_facts`;
- request staff clarification/replacement media.

### Medication administered normally

Input notes:

- reviewed medication instruction exists;
- authorized staff note says medication was given per instructions;
- routine play/mood note exists.

Safe output behavior:

- medication copy remains review-gated even when normal;
- minimize medication detail in customer copy unless policy requires it;
- do not claim dose effects or clinical outcomes.

### Medication issue, concerning stool/vomiting, aggression, missed feeding, uncertainty, or complaint

Input signals such as “refused/spit out med,” “loose stool x2/vomited,” “snapped toward leash/removed from group,” “breakfast missed/refused,” “not sure if this pet ate,” “photo might be wrong pet,” or an owner complaint must suppress routine upbeat customer copy. The output should set `customer_message.body=null` unless manager/staff-approved wording is already supplied, record all routine and sensitive facts in `omitted_facts`, add the appropriate `internal_flags`, and recommend review or internal tasks.

## 8. Human approval gates

Human approval gates are part of the workflow contract, not optional UI polish.

### Auto-send daily updates

Current posture: daily updates are draft/review only. No approved auto-send policy, deterministic template catalog, media/consent policy, cadence policy, or channel send adapter is present in the repo sources.

A future daily update may become an auto-send candidate only when all of the following are true and recorded in policy/audit context:

1. Location, service line, update category, channel, template id/version, and update window are explicitly approved for auto-send.
2. Copy is deterministic template-bound or a human-approved template variant; freeform AI text is not auto-sendable.
3. Pet, reservation, customer, location, service day/window, recipient, consent/opt-out, quiet hours, and delivery suppression state are verified.
4. Required care evidence is current, non-conflicting, source-backed, and approved for customer summary.
5. Required media, if any, has approved consent/suitability state and matches both pet and text.
6. No unresolved internal flags exist for illness, injury, aggression, medication issue, concerning stool/vomiting, missed feeding, staff uncertainty, complaint, incident, behavior restriction, privacy risk, payment/policy exception, wrong recipient, wrong pet, or source conflict.
7. Deterministic validators confirm every customer-visible sentence maps to allowed evidence or template variables.
8. Idempotency and audit identify the exact approved payload and prevent duplicate sends.

Even when `should_send=true` is valid in that future state, delivery remains a separate audited outbox/provider action. The agent does not send.

### Health/behavior concern language

Health, injury, medication, bathroom concern, aggression/behavior, incident, safety, and complaint language requires human-approved wording before customer delivery. The agent should use internal review language such as:

- “Suppressed from customer copy: health-related observation requires manager/medical review.”
- “Behavior review required before customer-facing wording about today’s playgroup note.”
- “Do not draft routine meal copy until the feeding exception is reviewed.”
- “Customer copy withheld pending approved incident disposition.”

Customer-facing wording, when approved by the proper reviewer, should be observational and non-diagnostic. It must not minimize, reassure without evidence, assign blame, or promise outcomes.

Manager/admin approval is required before customer-facing language or send decisions involving illness, injury, incident, safety, aggression, bite, escape/near miss, medication issue, medical/care ambiguity, concerning stool/vomiting, repeated missed feeding, complaints, negative sentiment, staff conduct concerns, service recovery, privacy/legal/liability language, policy exceptions, already-sent incorrect content, or sensitive media.

## 9. Integration notes

### AI runtime

The application owns the durable event inbox/queue, typed prompt-packet construction, deterministic policy checks, validation, persistence, idempotency, audit, retries, dead-letter handling, review task creation, and side effects. Hermes/LLM workers are bounded assistants that consume minimal typed packets and return validated structured outputs.

Prompt packet should include only:

- workflow identity: event type, event id, source version/hash, idempotency key, location, reservation/pet/day/window;
- subject snapshots: pet/reservation/service context and approved care profile/temperament facts needed for wording;
- evidence set: care notes, staff task evidence, media refs, incident refs, visibility/review state, freshness, and provenance;
- policy context: allowed actions, automation level, review gates, message/photo/consent/tone/template/cadence refs when available;
- output contract and validator instructions;
- redaction instructions excluding raw internal notes, diagnoses, unapproved incidents, payment data, other pets/customers, and provider internals from customer copy.

### Audit logging

Persist refs/hashes and redacted summaries by default. Do not store raw prompts, raw staff notes, raw provider payloads, raw photos, or full model traces unless an approved retention/privacy policy requires and permits them.

Audit fields should cover:

- workflow event id/type/version/hash and idempotency key;
- location, pet, reservation, service line, day/window, and recipient ref when needed;
- policy snapshot, template version, automation level, validator version, agent/schema/model/runtime version;
- source evidence manifest and which facts were included or omitted;
- `requires_review`, `should_send`, `review_reason`, `internal_flags`, suppression reason, review gates, and recommended actions;
- reviewer actor/role, edits, approval/rejection/suppression decision, and rationale;
- media refs, consent/suitability state, rejection/replacement reason, and privacy review refs;
- approved payload, outbox ref, channel/provider response, retry/dead-letter state, and sent/published timestamp for any later send;
- correction/void links to prior drafts/sent updates when source evidence changes.

### Manager review queue

Review queue items should be actionable and prioritized. Each item should include queue type, subject refs, source refs, reason codes, proposed customer copy or `not_generated`, omitted/suppressed facts, required decision, due basis, assignee role, priority, and idempotency key.

Queue types may include `care_staff_review`, `manager_review`, `medical_care_review`, `medication_review`, `behavior_review`, `privacy_media_review`, `integration_review`, and `dead_letter`.

Priority guidance:

- Critical/immediate: active safety/emergency, escape/lost pet, severe injury/illness, active medication error, urgent owner-contact hold.
- High: owner notice likely, injury/illness/medication/aggression/concerning stool-vomiting, missed feeding, privacy breach risk, checkout soon, complaint.
- Normal: routine draft review, missing non-sensitive evidence, photo replacement for expected routine update.
- Low: retrospective correction, duplicate cleanup, policy/template metadata issue with no same-day customer impact.

### Future MVP implementation

MVP should start with read-only/import or staff-entered evidence, draft-only customer updates, staff/manager review queues, and audit-first persistence. It should not write back to provider/PMS systems, publish media, or send customer messages until explicit adapters, policies, effect ledgers, and approval workflows exist.

Recommended MVP domain gaps to close:

- first-class `CareNote` aggregate with lifecycle states and stable source refs;
- typed meal amount/status, bathroom/elimination, photo requirement/review/consent/quality, daily update approval state, and suppression reason types;
- richer message approval policy than only `DraftOnly` and `ManagerApprovalRequired`, including staff-approved routine drafts and manager-approved sensitive drafts;
- daily-update key: location + reservation + pet + service date + update window + policy scope;
- deterministic validators for source-to-span citation, banned phrases, review triggers, media suitability, consent/opt-out/quiet-hours, idempotency, and correction invalidation;
- review UI that shows customer draft, included facts, omitted facts, flags, source refs, media candidates, replacement-photo tasks, and exact approval/suppression actions;
- outbox/effect ledger that sends only exact approved payloads and handles retries without regenerating copy.

Conservative fallback: when facts, review state, media consent, care evidence, tone policy, source freshness, or provider state are missing, stale, contradictory, sensitive, or unverified, produce `requires_review=true` and `should_send=false`. Do not invent upbeat filler, imply unavailable photos, diagnose, hide incidents, minimize concerns, mark care complete, or send customer updates autonomously.
