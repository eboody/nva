# Daily care update agent inputs

Purpose: collect the canonical inputs needed before defining the daily-care update agent. This is an input packet for downstream design cards, not live operating policy and not approval to send customer messages, capture photos, infer care/medical facts, or mutate a provider/PMS.

Status: draft input collection. Anything marked provisional is a safe modeling assumption derived from current repo docs/code and board handoffs; it must remain reviewable until a location policy, privacy/consent policy, or implementation artifact approves it.

## Source index

Primary repo sources checked:

- `docs/workflows/staff-operations-parts/inputs.md` — canonical staff-operations input packet, including roles, operating-day/stay states, active care obligations, daily update/Pawgress assumptions, task-model gaps, and approval gates.
- `docs/domain/petsuites/boarding/service-domain-map.md` — Boarding service vocabulary for Pawgress Report, photo/video update add-ons, care/report evidence, approval state, and Boarding-owned care/report surfaces.
- `docs/domain/petsuites/boarding/implications/06-medication-feeding-behavior-notes.md` — medication/feeding/behavior care-review triggers, staff note and handoff assumptions, Pawgress/customer-message review rules, and safety exceptions.
- `docs/domain/petsuites/daycare/implications/05-pet-health-behavior-notes.md` — daycare health/behavior note source, author/time/review-state requirements, customer-safe projection, and daily-care-update agent participation.
- `docs/domain/petsuites/daycare/implications/04-incident-tracking.md` — incident/media/customer-notice boundaries, incident follow-up tasks, and customer-facing incident-message review gates.
- `docs/workflows/workflow-event-idempotency-replay.md` — idempotency/replay rules for `DailyNoteCreated` and `DailyUpdateNeeded` events.
- `docs/workflows/payments-pricing.md` and `docs/workflows/payments-pricing-parts/ai-boundaries.md` — customer-facing/payment-sensitive message boundaries that update drafts must not cross.
- `docs/quality/semantic-code-doctrine-inventory.md` — semantic baseline, redaction expectations, and current typed agent/tool/workflow surfaces.
- `domain/src/entities.rs` — current `Pet`, `CareProfile`, `MedicationInstruction`, `TemperamentProfile`, `Reservation`, `ActorRef`, and audit identity surfaces.
- `domain/src/care.rs` — redacted semantic care values for feeding, medication, allergies, medical conditions, contacts, and review reasons.
- `domain/src/operations.rs` — current daily brief, care watchlist, staff task, `DailyUpdateDraft`, role, assignment, source, status, priority, and completion-evidence surfaces.
- `domain/src/workflow.rs` — current `DailyNoteCreated`, `DailyUpdateNeeded`, `AllowedAction::SummarizeCareNotes`, `WorkflowResult`, review, risk, verification, and recommended-action envelopes.
- `domain/src/tools.rs` — current typed message drafting and media snapshot tool boundaries.
- Kanban handoff `t_cf266086` — canonical AI/Hermes runtime constraints for typed prompt packets, validated `WorkflowResult` outputs, queue/idempotency/audit/retry/dead-letter handling, prompt minimization, and approval gates.
- Kanban handoff `t_d13548b9` — data-model entity catalog handoff; the generated artifact path was in a scratch workspace that is no longer available, so use the handoff metadata as the source for CareNote/Message lifecycle states and core relationship summary.

Missing or caveated sources:

- `docs/product/pet-resort-product-map.md` is still absent in this repo; staff-operations and runtime input packets used board handoff metadata instead.
- `docs/data-model/core-entities.md` is referenced by handoff metadata but is not present in this repo; the scratch workspace artifact is also unavailable. Current canonical input for CareNote/Message lifecycle is the `t_d13548b9` handoff metadata.
- `docs/architecture/pet-resort-ai-runtime.md` is not present in this repo yet; current canonical input is the `t_cf266086` handoff metadata plus current domain code.
- No approved standalone photo/privacy/consent policy, tone/brand guide, customer-message template set, daily-update cadence policy, or per-location Pawgress/update requirement policy was found.

## Product and workflow scope

Current product posture from staff-operations/runtime inputs:

- First product shape is an internal staff/manager workflow layer for a pet resort or resort group; customer-facing communication remains draft/review or deterministic preapproved template only.
- Daily updates are part of the active-stay/day-care execution surface, alongside feeding, medication, play/enrichment, cleaning/housekeeping, incident/behavior/safety signals, and shift handoff.
- Source workflows/events are `workflow::WorkflowEventType::DailyNoteCreated` and `DailyUpdateNeeded`.
- Allowed agent actions already include `ReadEntities`, `ExtractStructuredData`, `DraftCustomerMessage`, `CreateInternalTask`, `SummarizeCareNotes`, and `FlagRisk`, but the existing policy context/review-gate system decides which are permitted for a given event.
- The agent should produce a draft/update packet or `WorkflowResult` recommendation, not send or publish the update directly.

Provisional update trigger model:

- `DailyNoteCreated`: a staff note/care note is created or updated for a reservation or pet. Rebuild the evidence set or update an existing draft/review packet; do not create duplicate report tasks for exact duplicate note versions.
- `DailyUpdateNeeded`: a reservation/day/window requires a daily update. Converge multiple triggers for the same reservation/day/window on one daily-update draft/review task.
- Incident, medication exception, feeding exception, behavior flag, or care-watchlist signals may trigger a draft suppression/review path instead of an upbeat routine update.

## Staff note format and existing UX assumptions

Current canonical note assumptions:

- Staff notes are operational evidence, not casual free text. Daycare note docs require source metadata, timestamps, author/staff role, customer-visible intent when known, classification, visibility, affected care modes, review state, and audit metadata.
- Current `entities::TemperamentProfile` includes `staff_notes: Vec<temperament::StaffNote>` and behavior observations; `CareProfile` holds longitudinal care facts. Boarding/daycare service docs say service-line workflows should consume snapshots and create stay-scoped obligations, not duplicate master profile truth.
- Boarding care docs explicitly allow free-text notes at intake/provider boundaries but require immediate promotion into redacted semantic values, typed review outcomes, staff tasks, handoff items, audit evidence, and customer-safe projections before policy behavior depends on them.
- Current generic staff task evidence is `operations::TaskCompletionEvidence`, attached to `StaffTask` completion. The docs warn that free-text/media-only evidence may be summarized by AI but must not let AI mark care complete.
- Staff task sources are typed as `Reservation`, `Pet`, `Customer`, `DailyBrief`, `WorkflowEvent`, or `StaffCreated`; a daily-care update draft should preserve the source event/task/note references used to generate it.

Provisional staff-note input shape for the daily-care update agent:

- identity: note/care-note id or provider note id, location id, reservation id where stay-scoped, pet id, customer id if needed only for recipient/draft linkage;
- source: staff-entered, customer-provided, provider-imported, task evidence, incident follow-up, profile snapshot, or AI-extracted draft;
- author/actor: staff id/role when known, customer when supplied, provider/system when imported, agent spec only for drafts;
- timestamp/version: created/observed/updated timestamp and source version/hash for idempotency;
- classification: routine care, meal/feeding, medication, play/enrichment, bathroom/elimination, mood/behavior, grooming/bath/training add-on, photo/media, incident/safety, customer preference, mixed/unknown;
- visibility: internal-only, customer-provided, customer-safe after review, customer-facing approved;
- review state: draft/recorded/internal, needs staff review, needs manager review, approved for customer summary, sent/published, corrected/voided;
- note text or summary: redacted internal text plus separate customer-safe wording when approved;
- evidence refs: staff task ids, care profile snapshot, media refs, incident ids, source documents, audit event ids;
- staff initials: not modeled as a first-class field today. Provisional handling: map displayed initials to `StaffId`/`ActorRef::Staff` for audit and only render initials in customer copy if an approved messaging policy allows it.

UX assumptions to preserve:

- Staff can enter short notes/evidence quickly during care execution, but downstream policy cannot branch on raw text alone.
- Staff and managers need to see why an update is draft, approved, suppressed, or escalated.
- Sensitive/internal note text must remain separate from customer-safe summaries.
- Corrections should create a new version or corrective audit event rather than silently overwriting what a prior customer update used.

## Photo/media policy inputs

Current explicit source surface:

- Boarding service map includes optional `PhotoVideoUpdate` as an add-on/update-related service concept and `PawgressReport` as a customer-facing report built from staff observations/media.
- `domain/src/tools.rs` exposes a `media::MediaCapture` trait with `MediaSnapshotRequest { location_id, camera_id, purpose }`; purposes are `PetStatusCheck(PetId)`, `FacilitySafetyCheck`, and `IncidentReview(ReservationId)`. Results are `Captured { media_ref }` or `Unavailable { CameraOffline | PermissionDenied | RetentionExpired }`.
- Daycare incident docs allow optional media/document references through approved storage/tool boundaries, not raw blobs inside the incident aggregate.
- Runtime handoff treats media/camera snapshots as sensitive data and requires minimization/redaction in prompts/logs, plus an approved retention/privacy policy before production.

Provisional photo/update assumptions, pending approval:

- Photos are optional for routine daily care updates unless a paid add-on, service package, customer preference, or location policy makes a photo/video update required.
- If a photo is required but unavailable, poor quality, camera-offline, permission-denied, or retention-expired, the agent should produce a transparent internal reason and either draft a text-only update for staff review or create a staff task to collect/replace media; it must not fabricate or imply a photo exists.
- Poor photos should not be sent automatically. Route to staff review or replacement if the image is blurry, wrong pet, unsafe context, contains another customer/person/pet without approved consent, shows injury/incident/sanitation concern, or conflicts with the draft text.
- Photo/media references should be passed as `MediaRef`/document refs plus minimal metadata, not raw images, unless a downstream approved image-analysis workflow explicitly needs the pixels and privacy policy allows it.
- Incident/safety/media evidence remains internal unless manager/safety review approves customer-facing use.

Unknown photo/privacy/consent policy requiring synthesis/review:

- Whether routine updates require a current-day pet photo, how many photos per update, and which service lines/add-ons require them.
- Whether owner consent is captured globally, per stay, per service, or per photo/video update add-on.
- Rules for photos containing staff, other pets, other customers, facility areas, name tags, medical/incident context, kennels/rooms, or unsafe/unflattering content.
- Retention period for media refs, thumbnails, prompts/responses that mention media, and rejected/unavailable media events.
- Whether webcam/camera provider snapshots are permitted for customer updates or only for internal status/safety review.

## Tone, brand, and customer messaging policy

Current sources do not define a formal tone/brand guide or customer-message template set. Existing repo docs only establish conservative communication boundaries:

- Customer-facing messages are drafts/approval-gated unless a deterministic preapproved template/send path is later approved.
- High-risk customer-facing sends remain manual for v1 unless later approved.
- Sensitive customer messages involving health, safety, legal, payment, incident, eligibility, refusal, or policy exceptions require staff/manager approval.
- Incident docs require approved customer notices to preserve important facts without diagnosis, blame-shifting, or unreviewed promises.
- Daycare note docs require customer-facing summaries to be separate reviewed values, not raw internal text.
- Payment docs prohibit inventing amounts, quoting stale/location-unknown rates, or promising availability.

Provisional tone/brand assumptions for drafting only:

- Warm, concise, pet-parent friendly, and factual.
- Positive routine updates may mention meals, play/enrichment, mood, bathroom, rest, and approved photos when source-backed.
- Avoid diagnosis, medical advice, legal/liability language, blame, guarantees/promises, unapproved policy explanations, invented details, or staff speculation.
- Use neutral language for concerns: draft an internal review note or staff follow-up rather than customer copy when the source fact is sensitive or unresolved.
- Do not mention other pets/customers/staff beyond approved initials/signature policy.
- If required evidence is missing, draft a safe internal explanation and request staff review rather than filling gaps with generic cheerful copy.

Unknown messaging inputs requiring synthesis/review:

- Approved daily update template(s), brand voice, emoji policy, sign-off/signature/initials policy, and channel-specific length/timing rules.
- Which low-risk updates, if any, can be sent without per-message staff approval.
- Customer preference/opt-out rules by channel and service line.
- Whether update copy should use pet name only, owner name, staff initials, location name, or brand name in the sign-off.

## Care note / data model fields relevant to daily updates

Existing implemented anchors:

- `entities::Pet`: `id`, `customer_id`, `name`, `species`, optional `birth_date`/`sex`, `spay_neuter_status`, `temperament`, and `care_profile`.
- `entities::CareProfile`: optional `feeding_instructions`, `medications`, `allergies`, `medical_conditions`, optional emergency and veterinarian contacts.
- `entities::MedicationInstruction`: `name`, `dose`, `schedule`, and `care::MedicationReviewRequirement`.
- `entities::TemperamentProfile`: group-play observation, people orientation, rating, behavior observations, and staff notes.
- `entities::Reservation`: location/customer/pet ids, service kind, status, start/end times, deposit, source, requested add-ons, and hard stops.
- `entities::AddOn`: currently includes `GroupPlay`, `IndividualPlay`, `WebcamSuite`, `ExitBath`, and `PawgressReport`.
- `operations::StaffTaskKind`: includes `Feeding`, `MedicationAdministration`, `PlaygroupAssessment`, `DailyUpdateDraft`, `IncidentFollowUp`, `CheckInPrep`, `CheckOutPrep`, `CleaningTurnover`, `DocumentReview`, and `CustomerFollowUp`.
- `operations::PetCareWatchReason`: `MedicationDue`, `FeedingException`, `AnxietyOrStressFlag`, `BehaviorReview`, `IncidentFollowUp`.
- `tools::messaging::DraftMessageRequest`: channel, recipient, body, and review policy (`DraftOnly` or `ManagerApprovalRequired`).
- `tools::media::MediaSnapshotRequest`: location, camera, and capture purpose; result returns a media ref or unavailable reason.

CareNote canonical handoff fields from `t_d13548b9` metadata:

- Purpose/ownership: CareNote is pet/reservation-scoped observation history that may produce customer-visible message drafts.
- Lifecycle states: `Draft`, `Recorded/Internal`, `NeedsReview`, `ApprovedForCustomerSummary`, `Sent/Published`, `Corrected/Voided`.
- Relationships: Pet owns longitudinal care notes; Reservation ties stay-scoped notes, tasks, messages, incidents, documents, and audit events together.
- AI-write policy: AI may suggest/draft/summarize; verified/human-approved state is required for customer summary, sent/published, corrections, sensitive facts, and care completion.

Provisional daily-care update evidence fields to require downstream:

- meal/feeding: offered/eaten/refused amount or qualitative status, feeding instruction source/review state, exceptions/refusals, staff task evidence;
- play/enrichment: service variant/care mode, group or individual play, duration/period where policy allows, observed engagement, playgroup assessment/restriction state;
- mood/behavior: routine demeanor and customer-safe behavior observations; separate internal concerns/restrictions from approved customer-safe wording;
- bathroom/elimination: potty walk or elimination status and exceptions. No current first-class Rust field exists; model as care note/task evidence until a typed bathroom/elimination value is added;
- meds: medication due/administered/skipped/exception, but only from reviewed medication instructions and authorized staff evidence; do not infer from vague notes;
- photos/media: `MediaRef`/document refs, capture purpose, consent/review state, unavailable/poor-photo reason;
- concerns/incidents: incident ids, care-watchlist reasons, review gates, suppression reason, staff/manager action required;
- staff initials/actor: `ActorRef::Staff`/`StaffId`, role, optional approved initials display;
- review/audit state: draft/internal, needs staff review, needs manager review, approved for customer summary, sent/published, corrected/voided; source event id, idempotency key, policy snapshot/version, audit event refs.

Known data-model gaps:

- No first-class `CareNote` struct exists in current `domain/src`; use board handoff lifecycle and current task/workflow/message surfaces until implemented.
- No typed meal amount/status, bathroom/elimination, photo consent/review, poor-photo reason, staff initials display policy, or daily-update approval state exists as a dedicated domain type today.
- `StaffTaskKind::DailyUpdateDraft` is reservation-scoped but does not capture service day/update window, evidence set, draft id, approval state, suppression reason, or sent status.
- Message review policy currently only distinguishes `DraftOnly` and `ManagerApprovalRequired`; staff-approved/non-sensitive vs manager-approved/sensitive may need richer states.

## AI/Hermes runtime constraints for this agent

Canonical runtime posture from `t_cf266086` and current domain code:

- The app owns durable inbox/queue, source verification, typed prompt packet construction, deterministic policy checks, output validation, audit, task/draft persistence, idempotency, retries, and side effects.
- Hermes/LLM workers are bounded assistants. They consume minimal typed prompt packets and return validated `WorkflowResult<T>`-style outputs containing summary, structured output, recommended actions, risk flags, verification notes, and optional human review reason.
- Production automation/permissions/security policy remain approval-gated. Customer-message automation and final send permissions are not approved.
- Prompt packets must be least-privilege and should include only required entity snapshots/references. Redact/minimize customer PII, pet medical/care details, payment data, raw provider payloads, webhook secrets, staff/labor details, and media unless required and approved.
- Raw provider JSON, customer claims, staff free text, screenshots/media, and previous AI summaries are not truth until verified/reconciled or reviewed.
- AI outputs default to draft/suggestion/internal task. Deterministic policy and/or human approval decides whether any side effect occurs.

Daily-care update prompt packet should include only:

- workflow/event identity: `DailyNoteCreated` or `DailyUpdateNeeded`, idempotency/replay key, location, reservation/day/window, source event version/hash;
- subject snapshots: pet/reservation/service line and approved care profile/temperament fields needed for wording, excluding unnecessary PII/payment data;
- approved evidence set: care notes/task evidence/media refs with source provenance, visibility, review state, and freshness;
- policy context: allowed actions, automation level, required review gates, message/photo/consent/tone policy refs when available;
- output contract: structured draft sections, source citations/evidence refs, risk/suppression flags, requested staff/manager review, and no-send guarantee;
- redaction instructions: do not expose raw internal notes, diagnoses, unapproved incidents, payment details, other pets/customers, or provider internals in customer copy.

Output validation/escalation should require:

- every customer-facing sentence traces to approved or review-eligible evidence;
- no unsupported claims about meals, play, mood, bathroom, meds, photos, incident status, staff action, payment, policy, availability, or health;
- sensitive facts either excluded from customer copy or marked `NeedsManagerReview`/suppressed;
- missing required evidence produces `NeedsMoreInformation`, `NeedsHumanReview`, or an internal task recommendation, not invented content;
- unavailable/poor photo is surfaced as a reason/task, not hidden or replaced with fabricated media;
- duplicate/replay events converge on existing draft/review task keys and do not send or create duplicate tasks;
- any validation failure, unsafe output, missing required input, or policy denial results in human review/dead-letter style handling rather than side effects.

## Human approval gates discovered

These gates must survive downstream synthesis:

1. Customer-message send approval.
   - Daily care updates are customer-facing messages. Conservative default: draft/review only.
   - Low-risk auto-send has no approval artifact yet and must remain unapproved.

2. Sensitive content approval.
   - Medical, medication, allergy, feeding exception, behavior, incident, safety, legal/liability, eligibility, payment, policy exception, or customer complaint content requires staff/manager review before customer copy.

3. Care completion and operational truth.
   - AI may summarize evidence but cannot mark feeding, medication, play, bathroom, cleaning, incident, or care tasks complete.
   - Medication administration requires reviewed medication name/dose/schedule/source and authorized staff evidence.

4. Photo/media consent and suitability.
   - No approved policy currently defines when photos are required, what consent covers, or what content is sendable. Treat media as review-gated and sensitive until policy exists.

5. Staff note/customer-safe projection.
   - Raw internal notes, ambiguous health/behavior text, imported provider memos, and AI-extracted drafts cannot be used directly in customer-facing text without staff/manager-safe projection.

6. Incident/safety gate.
   - Incident/health/safety signals suppress routine upbeat updates until manager/safety review decides whether and how to communicate.

7. Provider/runtime side effects.
   - Sending messages, writing back to Gingr/PMS/provider systems, changing reservation/status records, or attaching/publishing media requires approved adapters, idempotency/effect ledger, audit, and authorized approval.

8. Privacy/retention/AI governance.
   - Final policy is missing for prompts/responses, care notes, audit logs, documents/OCR, media snapshots, rejected media, provider payloads, and model/tool traces.

## Open questions for downstream synthesis/review

1. Are daily updates/Pawgress reports required for all stays, only boarding, daycare, specific add-ons, customer preference, or per-location policy?
2. What is the daily update cadence/window, and should there be one draft per reservation/day, per pet/day, or per service window?
3. What exact staff note fields are available in the source UX/provider today? Is there a structured note form, checklist, tags, photos, initials, and approval state, or only free text?
4. Which meal, bathroom, mood, play, and medication status values should become first-class semantic enums for customer updates?
5. What photo/video consent policy applies, and which photo failures should create replacement tasks vs text-only drafts?
6. What tone/brand templates are approved by channel and service line?
7. Which staff role may approve a routine update? Which cases require manager/admin/safety review?
8. Should customer copy include staff initials/signature, and how are initials mapped to auditable `StaffId`?
9. How should multi-pet reservations produce updates: one combined customer update, one per pet, or both?
10. How should corrected/voided care notes affect already drafted or sent updates?
11. What source/provider integration mode is MVP: read-only import, staff copy/paste, approved write-back, or no integration?
12. What retention/redaction rules apply to prompt packets and LLM outputs containing care notes/media metadata?

## Conservative downstream rule

When source facts, note visibility, photo consent, care task evidence, policy, tone guidance, or provider state are missing, stale, contradictory, sensitive, or unverified, the daily-care update agent should produce a review state, suppression reason, or internal task recommendation. It should not invent cheerful filler, imply unavailable photos, diagnose, hide incidents, mark care complete, or send customer updates autonomously.
