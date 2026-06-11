# Staff note capture contract for daily care updates

Purpose: define the structured staff-note capture contract that feeds the daily care update agent. This contract turns fast operational observations into auditable, reviewable evidence for a draft Pawgress/daily update. It does not authorize live customer sends, provider write-backs, care completion, medical conclusions, or automatic media publication.

Status: draft contract based on `docs/workflows/daily-care-update-agent-parts/inputs.md`. Photo, privacy/consent, tone, cadence, staff-initial display, and low-risk auto-send policies are still missing; any behavior depending on those policies must remain reviewable and conservative.

## Contract principles

1. Staff notes are evidence, not final customer copy.
2. Raw free text, imported provider notes, photos, and AI-extracted drafts are not operational truth until verified or accepted by an authorized staff/manager actor.
3. Customer-facing wording must be generated from approved or review-eligible structured evidence, not from unsupported cheerful filler.
4. Missing, stale, contradictory, uncertain, or sensitive facts create `NeedsMoreInformation`, `NeedsStaffReview`, `NeedsManagerReview`, or `SuppressedFromCustomer` states.
5. Medication, medical, incident, behavior/safety, allergy, policy/payment, owner complaint, and privacy-sensitive facts are never downgraded by AI.
6. Each captured item preserves source attribution: who entered it, when it was observed/entered, what system/task/source produced it, and which note version was used.

## Capture envelope

Every staff-note record used by the daily care update agent should include this envelope.

| Field | Required? | Notes |
| --- | --- | --- |
| `note_id` | Required | Stable care-note/provider-note/task-evidence id. Required for citations, correction handling, and replay/idempotency. |
| `location_id` | Required | Needed for policy lookup, media permissions, and operating-day context. |
| `reservation_id` | Required for active stay/daycare updates | Use stay-scoped reservation id when available. If absent, mark the note `ProfileLevelOnly` until staff attaches it to a stay/window. |
| `pet_id` | Required | Subject pet. Multi-pet reservations should capture one subject per observation unless the note explicitly applies to all listed pets. |
| `customer_id` | Optional | Include only when needed for draft linkage/recipient selection; avoid unnecessary customer PII in prompt packets. |
| `service_line` | Required | Boarding, daycare, grooming, training, add-on, or unknown. Drives allowed chips and review gates. |
| `update_window` | Required when known | Reservation/day/window this note can support. Unknown windows do not block capture, but they require staff review before draft inclusion. |
| `source_kind` | Required | `StaffEntered`, `TaskEvidence`, `ProviderImported`, `CustomerProvided`, `IncidentFollowUp`, `ProfileSnapshot`, or `AiExtractedDraft`. |
| `source_ref` | Required when available | Staff task id, provider note id/version, incident id, media ref, audit event id, or profile snapshot ref. |
| `author_actor` | Required when known | Prefer `ActorRef::Staff`/staff id plus role. For provider/system imports, preserve source system identity. |
| `staff_initials_display` | Optional/reviewable | Display initials are not first-class domain truth today. Store only as a display candidate mapped to staff id; do not expose in customer copy unless a messaging policy approves it. |
| `observed_at` | Required when known | When the care event happened. If unknown, set `observed_at_status: Unknown` and use `created_at` only for audit. |
| `created_at` | Required | When the note/evidence was entered/imported. |
| `updated_at` | Optional | Required for corrected versions. Corrections create new versions/audit events rather than silently changing prior customer evidence. |
| `source_version_hash` | Required for imports/replays | Used to collapse duplicate `DailyNoteCreated` events and avoid duplicate draft tasks. |
| `classification` | Required | One or more of `Meal`, `PlayActivity`, `MoodComfort`, `Bathroom`, `Medication`, `PhotoMedia`, `Concern`, `Mixed`, or `Unknown`. |
| `visibility_state` | Required | `InternalOnly`, `CustomerSafeCandidate`, `CustomerApprovedSource`, `RestrictedSensitive`, or `SuppressedFromCustomer`. Default conservative by category. |
| `review_state` | Required | `RecordedInternal`, `NeedsMoreInformation`, `NeedsStaffReview`, `NeedsManagerReview`, `ApprovedForCustomerSummary`, `SentPublished`, or `CorrectedVoided`. |
| `raw_note_text` | Optional | Preserve original staff/provider wording as source evidence with redaction controls. Raw internal text is not customer copy. |
| `customer_safe_summary` | Optional/reviewable | Staff-approved or draft wording candidate. Must cite supporting structured fields. |
| `structured_observations` | Required when present | Typed category payloads defined below. Empty only for free-text captures awaiting review/classification. |
| `media_refs` | Optional | References only; do not embed raw images in prompt packets unless a separate approved image workflow needs pixels. |
| `concern_flags` | Required | Empty array allowed. Include uncertainty, symptoms, behavior/safety, owner complaint, photo issue, missing data, contradiction, or manager-review reason. |
| `follow_up_request` | Optional | Staff task recommendation, replacement-photo request, owner/vet clarification, manager review, or suppression reason. |
| `audit_refs` | Optional | Audit event ids, policy snapshot ids, approval ids, or task ids. |

## Category payloads

### Meals

Required when a meal/feeding observation is captured:

| Field | Required? | Allowed examples / notes |
| --- | --- | --- |
| `meal_slot` | Required when known | `Breakfast`, `Lunch`, `Dinner`, `Snack`, `Treat`, `Other`, `Unknown`. |
| `offered_amount` | Required when known | Use typed amount when available: portion, unit, percentage, or free-text label from feeding instruction. Unknown is acceptable but must be explicit. |
| `ate_amount` | Required | `All`, `Most`, `Some`, `Little`, `None`, `Refused`, `Unknown`, or a measured amount if staff recorded it. |
| `appetite` | Required | `Normal`, `Strong`, `Reduced`, `Picky`, `Refused`, `Unknown`. Do not infer appetite from a vague note without staff confirmation. |
| `treats_given` | Optional | Treat type/amount when relevant. If treats are restricted by care profile or allergy, route to review. |
| `special_diet_notes` | Optional | Instruction source, substitutions, owner food, allergy/diet concern, or ambiguity. Sensitive/exception cases are internal/review-gated. |
| `feeding_exception` | Required | Boolean plus reason when ate amount is `None`, `Refused`, contradictory, vomited, wrong food, unclear instruction, or staff uncertainty. |

Validation notes:

- Refusal, vomiting, wrong food, allergy/diet ambiguity, or inconsistent offered/ate values require staff or manager review before customer wording.
- The agent may say a pet ate a meal only when `ate_amount` is known and source-backed.
- Special diet and allergy details should be minimized in customer copy and routed through review.

### Play and activity

Required when play/activity evidence is captured:

| Field | Required? | Allowed examples / notes |
| --- | --- | --- |
| `activity_type` | Required | `GroupPlay`, `IndividualPlay`, `Walk`, `Enrichment`, `RestBreak`, `Training`, `GroomingActivity`, `Other`, `Unknown`. |
| `session_status` | Required | `Completed`, `Partial`, `Skipped`, `NotScheduled`, `Unavailable`, `Unknown`. |
| `engagement` | Required when observed | `Playful`, `Curious`, `Calm`, `Shy`, `Overstimulated`, `PreferredRest`, `Unknown`. |
| `duration_or_period` | Optional | Use only if policy/source supports it. Avoid invented exact times. |
| `rest_observed` | Optional | Rest/nap/quiet time signal; useful for balanced updates. |
| `enrichment_detail` | Optional | Toy, puzzle, cuddle time, walk route, or enrichment type if source-backed and customer-safe. |
| `social_compatibility_signal` | Required when relevant | `ComfortableGroup`, `NeedsSmallGroup`, `IndividualCareToday`, `SeparatedForRest`, `Reactive`, `Unknown`, with review state. |
| `play_restriction_or_review` | Required | Boolean plus reason for incident, temperament uncertainty, group reassignment, safety concern, or conflicting observations. |

Validation notes:

- Do not mention other pets by name or imply group compatibility beyond reviewed evidence.
- Behavior restrictions, reactivity, altercations, injuries, or safety signals suppress routine upbeat copy until manager/lead review.
- If the source only says "played" with no service/activity context, mark the activity as `Unknown` and request staff review before detailed wording.

### Mood and comfort

Required when demeanor/comfort evidence is captured:

| Field | Required? | Allowed examples / notes |
| --- | --- | --- |
| `demeanor` | Required when observed | `Happy`, `Calm`, `Relaxed`, `Playful`, `Affectionate`, `Shy`, `Anxious`, `Stressed`, `Tired`, `Unknown`. |
| `settling_status` | Optional | `SettledWell`, `StillSettling`, `NeededExtraSupport`, `NotObserved`, `Unknown`. |
| `affection_or_staff_interaction` | Optional | Cuddles, pets, tail wags, enjoyed attention, preferred space. Keep factual. |
| `energy_level` | Optional | `High`, `Moderate`, `Low`, `Resting`, `Unknown`. Low energy plus symptoms is review-gated. |
| `comfort_support` | Optional | Blanket, quiet room, decompression, one-on-one attention, rest break, owner item; avoid implying treatment. |
| `mood_concern` | Required | Boolean plus reason for stress, lethargy, unusual behavior, aggression, injury, or staff uncertainty. |

Validation notes:

- Mood words must be observations, not diagnoses. Prefer "seemed calm" or "staff observed relaxed body language" over clinical or causal claims.
- Anxiety/stress can be routine settling context internally, but customer-facing wording needs staff/manager review when persistent, severe, or connected to behavior/medical concerns.

### Bathroom

Required when bathroom/elimination evidence is captured:

| Field | Required? | Allowed examples / notes |
| --- | --- | --- |
| `urination_observed` | Required when bathroom note is present | `Yes`, `No`, `NotObserved`, `Unknown`. |
| `bm_observed` | Required when bathroom note is present | `Yes`, `No`, `NotObserved`, `Unknown`. |
| `stool_quality` | Optional/reviewable | `Normal`, `Soft`, `Loose`, `Diarrhea`, `Hard`, `BloodObserved`, `MucusObserved`, `Unknown`. Capture only where appropriate and policy allows. |
| `accident_observed` | Optional | Boolean plus location/context and cleanup task ref when relevant. |
| `walk_or_potty_break` | Optional | Potty walk/break completed, skipped, unavailable, or unknown. |
| `bathroom_concern` | Required | Boolean plus reason for diarrhea, blood, repeated accidents, straining, no elimination when expected, or uncertainty. |

Validation notes:

- Bathroom data is not a current first-class Rust domain value; treat it as care note/task evidence until a typed elimination value exists.
- Stool quality, blood, diarrhea, repeated accidents, or abnormal elimination are sensitive and should be internal/review-gated before customer copy.
- Do not diagnose or reassure medically; route concerns to staff/manager/vet-contact workflows as policy requires.

### Medications

Required when medication evidence is captured:

| Field | Required? | Allowed examples / notes |
| --- | --- | --- |
| `medication_ref` | Required when known | Reference reviewed medication instruction id/name. Avoid placing full med detail in prompt packets unless needed and approved. |
| `schedule_ref` | Required when known | Reviewed schedule/source. Unknown schedule means `NeedsMoreInformation`. |
| `administration_status` | Required | `Scheduled`, `Administered`, `Skipped`, `Refused`, `HeldByInstruction`, `Unavailable`, `Uncertain`, `NotDue`. |
| `administered_at` | Required for administered claims when known | If unknown, do not claim timing in customer copy. |
| `authorized_staff_actor` | Required for administered/skipped/refused claims | Staff id/role who recorded the medication evidence. |
| `refusal_or_skip_reason` | Required when status is skipped/refused/held/unavailable/uncertain | Use factual reason or `Unknown`. |
| `medication_exception` | Required | Boolean plus reason for any ambiguity, missed/refused dose, wrong med/dose concern, side-effect-like observation, storage issue, or source mismatch. |

Validation notes:

- Do not infer medication administration from vague notes like "meds ok" unless structured authorized evidence exists.
- The agent may summarize medication evidence internally but must not overclaim medical effects, side effects, efficacy, or clinical interpretation.
- Skipped, refused, uncertain, conflicting, or medically sensitive medication facts require manager/staff review and often suppress routine customer copy.

### Photos and media

Photo/media capture is optional for routine updates unless a service package, paid add-on, customer preference, or location policy makes it required.

| Field | Required? | Allowed examples / notes |
| --- | --- | --- |
| `photo_requirement` | Required | `RequiredByPolicy`, `RequiredByAddOn`, `Optional`, `NotAllowed`, `Unknown`. Unknown behaves like optional plus review when customer update depends on it. |
| `media_ref` | Required when available | Stable `MediaRef`/document ref only. No raw image blob in routine prompt packet. |
| `capture_purpose` | Required when media exists | `PetStatusCheck`, `RoutineUpdateCandidate`, `FacilitySafetyCheck`, `IncidentReview`, `DocumentEvidence`, `Other`. |
| `caption_candidate` | Optional | Staff caption or draft caption. Must be source-backed and reviewable. |
| `photo_quality_state` | Required when media exists | `UsableCandidate`, `Blurry`, `WrongPet`, `UnsafeContext`, `ContainsThirdParty`, `SensitiveContent`, `PermissionDenied`, `CameraOffline`, `RetentionExpired`, `Unavailable`, `Unknown`. |
| `selection_rank` | Optional | Candidate ordering for staff review, not automatic send. |
| `unavailable_reason` | Required when required photo is absent | Poor quality, no consent, camera offline, permission denied, retention expired, staff unavailable, wrong pet, or unknown. |
| `media_review_state` | Required | `NeedsStaffReview`, `NeedsManagerReview`, `ApprovedForCustomerSummary`, `RestrictedInternal`, `RejectedForCustomerUse`. |

Selection policy:

1. Prefer current-day, correct-pet, routine, flattering, non-sensitive photos tied to the reservation/update window.
2. Exclude or route to review any photo with other customers, staff faces/name tags, other pets not part of the reservation, documents, payment data, medical/incident/sanitation context, unsafe/unflattering content, or policy/consent uncertainty.
3. If multiple usable candidates exist, present ranked options with captions and source refs to staff; do not auto-publish.
4. If a photo is required but unavailable or rejected, produce an internal reason and recommend either a replacement-photo task or text-only staff-review draft. Never imply a photo exists.

### Concerns and manager-review triggers

Every note must include an explicit concerns array. Empty is valid only when the staff actor selected no uncertainty/review flags and the category validations pass.

Concern fields:

| Field | Required? | Notes |
| --- | --- | --- |
| `concern_type` | Required | `StaffUncertainty`, `Symptom`, `MedicationException`, `FeedingException`, `BehaviorConcern`, `BathroomConcern`, `IncidentSafety`, `PhotoPrivacyQuality`, `OwnerComplaint`, `ContradictoryEvidence`, `MissingRequiredEvidence`, `PolicyPayment`, `Other`. |
| `severity` | Required | `Info`, `NeedsStaffReview`, `NeedsManagerReview`, `SafetyCritical`. |
| `description_internal` | Required | Factual internal explanation. May be redacted from prompt/customer copy. |
| `customer_copy_allowed` | Required | `No`, `DraftOnly`, `AfterStaffReview`, `AfterManagerReview`, `Approved`. |
| `recommended_action` | Required | Review draft, create task, suppress update, request more info, replace photo, contact owner/vet per policy, or no action. |
| `owner_complaint_ref` | Optional | Owner complaint/customer concern id if this note relates to a complaint. |

Manager review is required for incidents/safety, injury/illness symptoms, medication exception, allergy/diet safety, aggression or group-play restriction, owner complaint, privacy/photo uncertainty, legal/liability wording, payment/policy issue, conflicting evidence that changes customer copy, or any staff uncertainty that would otherwise be hidden from the update.

## Required vs optional summary

Minimum required for a note to be eligible as daily-update evidence:

- capture envelope: `note_id`, `location_id`, `pet_id`, active `reservation_id` or explicit profile-level status, `service_line`, `source_kind`, `author_actor` when known, timestamps, classification, visibility state, review state, and source/version refs;
- at least one category payload or an explicit `Unknown/Mixed` free-text payload needing classification;
- staff initials/source attribution if available, mapped to audit actor rather than treated as proof by itself;
- concerns array, even when empty;
- missing/uncertain representation for every required category field.

Optional but useful:

- customer-safe summary candidate;
- photo/media refs and caption candidates;
- duration/period fields for play/activity;
- detailed treat/enrichment/comfort-support notes;
- follow-up task recommendation;
- policy snapshot/audit refs.

## Missing and uncertain data representation

Use explicit unknown states instead of blanks when a field is relevant but not known.

| Case | Representation | Draft behavior |
| --- | --- | --- |
| Staff did not observe a category | `NotObserved` or absent category payload with no claim | Do not mention it. |
| Staff observed but did not know details | `Unknown` plus `NeedsMoreInformation` or `NeedsStaffReview` | Ask for review/details; avoid customer claim. |
| Source is contradictory | `concern_type: ContradictoryEvidence` and `NeedsStaffReview`/`NeedsManagerReview` | Suppress affected claim until resolved. |
| Sensitive fact exists | `RestrictedSensitive` visibility and concern flag | Keep internal or route manager-approved wording. |
| Required photo unavailable | `photo_requirement` plus `unavailable_reason` | Recommend replacement task or text-only review draft; do not fake photo. |
| Medication status uncertain | `administration_status: Uncertain` with exception reason | Needs medication review; no customer claim of administration. |
| Bathroom/meal/play/mood missing from routine update | Omit unsupported sentence | Do not use generic filler like "had a great day" unless evidence supports it. |
| Staff initials missing | `author_actor` unknown/provider/system, initials absent | Preserve source attribution; require review if customer signature/initials are required. |

## Validation rules for the daily care update agent

Before a staff note can be used in a draft update packet:

1. Subject linkage must be unambiguous: pet, reservation/stay or profile-level status, service line, and update window when relevant.
2. Source provenance must be present: source kind/ref, actor/source, created/observed timestamps, and version/hash for imports or replayable events.
3. Each customer-facing sentence must cite one or more approved or review-eligible source notes/tasks/media refs.
4. Raw internal notes, sensitive facts, provider memos, AI drafts, and imported free text cannot be copied directly into customer text.
5. Required category fields must be either populated or explicitly `Unknown`/`NotObserved` with review behavior.
6. Medication claims require reviewed instructions plus authorized staff evidence.
7. Photo claims require an available, reviewed candidate media ref; unavailable or poor photos produce internal reasons/tasks.
8. Incidents, symptoms, safety/behavior restrictions, owner complaints, medication exceptions, feeding exceptions, and privacy concerns must be surfaced to the reviewer and may suppress routine copy.
9. Corrections/voids must invalidate or re-review drafts that used the older note version.
10. Duplicate or replayed `DailyNoteCreated` events with the same source/version converge on the same evidence set and must not create duplicate customer-update tasks.

## Examples of terse staff entries

These examples show compact staff input and the structured interpretation the agent should expect. They are not automatic customer copy.

### Routine meal

Staff entry: `BF offered 1 cup kibble, ate all. - JM 8:15a`

Structured interpretation:

- classification: `Meal`
- meal slot: `Breakfast`
- offered amount: `1 cup kibble`
- ate amount: `All`
- appetite: `Normal` or `Unknown` unless staff selected appetite chip
- concerns: none
- visibility: `CustomerSafeCandidate` only if staff role/policy allows; otherwise `InternalOnly`

### Meal uncertainty

Staff entry: `Dinner bowl mostly gone but not sure if Buddy or roommate ate it. - LK`

Structured interpretation:

- classification: `Meal`
- ate amount: `Unknown` or `Some/Most` only if staff confirms subject
- concern: `StaffUncertainty`, `ContradictoryEvidence`/subject ambiguity
- review: `NeedsStaffReview`
- customer behavior: do not claim Buddy ate dinner until resolved

### Play/activity highlight

Staff entry: `Group play AM: chased balls, took a rest break, did well w/small group. - AR`

Structured interpretation:

- classification: `PlayActivity`
- activity type: `GroupPlay`
- engagement: `Playful`
- rest observed: true
- social compatibility signal: `NeedsSmallGroup` or `ComfortableGroup` depending selected chip
- concerns: none unless group restriction/review chip selected
- customer behavior: eligible as a reviewable routine highlight, without naming other pets

### Mood/settling

Staff entry: `A little nervous at check-in, settled after blanket + one-on-one pets. - NP 11a`

Structured interpretation:

- classification: `MoodComfort`
- demeanor: `Anxious` then `Calm/SettledWell` if staff selected settling chip
- comfort support: blanket and one-on-one attention
- concern: staff review if anxiety is persistent/severe; otherwise routine settling context
- customer behavior: warm wording allowed only after review and without diagnosis

### Bathroom

Staff entry: `Potty walk complete. Pee yes, BM normal. - TS`

Structured interpretation:

- classification: `Bathroom`
- urination observed: `Yes`
- BM observed: `Yes`
- stool quality: `Normal`
- concerns: none
- customer behavior: eligible as a factual routine note if bathroom details are approved for the update template

### Bathroom concern

Staff entry: `Accident in run; loose stool. Cleaned. - MR`

Structured interpretation:

- classification: `Bathroom`
- accident observed: true
- stool quality: `Loose`
- concern: `BathroomConcern`
- review: `NeedsStaffReview` or `NeedsManagerReview` per policy/severity
- customer behavior: do not bury in a cheerful update; reviewer decides if/how to mention

### Medication administered

Staff entry: `7pm med given per card. - SR`

Structured interpretation:

- classification: `Medication`
- administration status: `Administered` only if linked medication task/reviewed instruction and authorized staff actor exist
- administered at: `7pm` if source time is valid
- concern: none only when task evidence verifies med/schedule/source
- customer behavior: if mentioned at all, avoid medical-effect claims; may require staff/manager review

### Medication uncertain/refused

Staff entry: `Wouldn't take evening pill; hid in food and still refused. - SR`

Structured interpretation:

- classification: `Medication`
- administration status: `Refused`
- refusal reason: refused in food
- concern: `MedicationException`
- review: `NeedsManagerReview` or medication review path
- customer behavior: no claim that medication was administered; routine update may be suppressed

### Photo candidate

Staff entry: `Photo 4281 cute nap pic, okay for update. - JM`

Structured interpretation:

- classification: `PhotoMedia`
- media ref: `4281`
- capture purpose: `RoutineUpdateCandidate`
- caption candidate: optional staff text
- photo quality state: `UsableCandidate` only after staff confirms correct pet/no privacy issue
- media review state: `NeedsStaffReview` or `ApprovedForCustomerSummary` depending role/policy

### Unavailable required photo

Staff entry: `Pawgress photo required but kennel cam offline; need floor pic later. - LK`

Structured interpretation:

- classification: `PhotoMedia`
- photo requirement: `RequiredByPolicy` or `RequiredByAddOn`
- unavailable reason: `CameraOffline`
- concern: `PhotoPrivacyQuality` / `MissingRequiredEvidence`
- recommended action: create replacement-photo task or text-only staff-review draft
- customer behavior: do not imply a photo is attached

### Owner complaint / manager review

Staff entry: `Owner called upset about yesterday's update being too vague; wants manager call. - FD`

Structured interpretation:

- classification: `Concern`
- concern type: `OwnerComplaint`
- severity: `NeedsManagerReview`
- recommended action: manager/customer follow-up task
- customer behavior: suppress routine automated/draft send until manager review decides next message

## Prompt-packet projection

When constructing a prompt packet for the daily care update agent, include only minimized structured evidence:

- note ids/source refs, not raw provider payloads;
- pet/reservation/service context needed for wording;
- category payload values and review states;
- customer-safe summaries only when approved or explicitly marked as draft candidates;
- media refs and quality/review state, not image pixels by default;
- concern flags, suppression reasons, and required human-review reasons;
- policy snapshot/version when available.

Exclude unnecessary customer PII, payment details, staff HR details, raw internal notes that are not needed for drafting, other pet/customer identities, and unapproved medical/incident detail.

## Conservative default

If the capture contract cannot prove a routine, source-backed, customer-safe fact, the daily care update agent should omit the claim, request staff clarification, recommend a task, or mark the draft for human review. It must not invent meals, play, mood, bathroom events, medications, photos, staff initials, or reassurance to make an update sound complete.
