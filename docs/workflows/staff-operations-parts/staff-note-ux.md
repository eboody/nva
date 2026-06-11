# Staff note UX

Purpose: define a fast, safe staff note capture experience for pet-resort operations. This is a workflow/UX contract for downstream product and domain cards; it does not authorize live care decisions, customer messages, reservation mutations, payment actions, or provider write-backs.

Status: draft UX definition using `docs/workflows/staff-operations-parts/inputs.md` as the canonical input packet. Any role permission, review SLA, or location policy detail not already approved remains provisional and reviewable.

## Source anchors and constraints

Use this UX with the constraints from `inputs.md`:

- Staff notes support MVP emphasis on intake/missing-info handling, staff tasking, care/incident notes, capacity/availability snapshots, audit events, and draft messages/summaries.
- AI/workflows may read, summarize, detect gaps, draft internal tasks/messages, and recommend review gates; they must not be autonomous medical, safety, payment, eligibility, capacity-exception, or customer-message authorities.
- Free-text and media evidence are allowed for speed, but safety-sensitive meaning must be promoted into typed review states, staff tasks, draft messages, and audit records before it drives policy behavior.
- Missing, stale, contradictory, or sensitive evidence creates a review state or draft internal task rather than a silent ready state.
- Customer-facing sends require staff/manager approval, especially for medical, medication, allergy, behavior, incident, safety, payment, eligibility, refusal, or policy-exception facts.

## UX goals

1. Let staff capture useful operational evidence in under 10 seconds during busy care, front-desk, or handoff moments.
2. Bias the UI toward structured chips for common events while preserving free text for context that does not fit a chip.
3. Make the internal-only vs customer-safe boundary visible at capture time, not only during message generation.
4. Route risky notes into the right staff/lead/manager review path without relying on AI confidence as authority.
5. Preserve auditability: who captured the note, when, where, on which subject, from which device/source, and what later review/approval changed.
6. Keep future voice-to-text and AI drafting bounded as capture aids, not as automatic care completion or customer communication.

## Primary capture surfaces

### 1. Pet/stay timeline quick capture

Use when staff is looking at a pet or active stay.

Required controls:

- note type selector or inferred type from the current task/stay context;
- quick buttons / structured note chips;
- free-text field;
- optional photo/attachment picker;
- internal-only/customer-safe toggle with conservative default;
- flag/escalate action;
- attach-to selector when the current context is ambiguous.

Default attachment: active stay/reservation plus pet. If no active stay exists, attach to pet profile and mark as profile-level evidence pending review before it can affect a future stay.

### 2. Task completion evidence panel

Use when staff completes or blocks a task such as feeding, medication, cleaning, playgroup assessment, daily update draft review, document review, or incident follow-up.

Required controls:

- task-specific chip set;
- required evidence prompts for safety-sensitive tasks;
- completion/block/escalate decision;
- required review gate display when the task kind cannot be closed by the current role;
- photo/attachment slot when the task type allows media evidence.

Free-text/media-only evidence may support a staff decision, but AI must not mark care complete from it. The task completion actor remains a staff/manager actor with audit identity.

### 3. Shift handoff capture

Use when outgoing staff records continuing obligations for incoming staff.

Required controls:

- next owner selector: role, named staff when available, shift lead, manager, pending external evidence, or cross-service owner;
- severity selector: info, incoming staff, shift lead, manager, safety critical;
- due timing;
- acceptance criteria;
- source task/stay/pet link;
- flag/escalate action.

Every unresolved handoff item must have a next owner or review gate. Vague notes without subject and owner are allowed only as drafts until a shift lead resolves them.

### 4. Customer-update draft evidence drawer

Use when staff is collecting notes that may feed a Pawgress/daily update, checkout summary, or customer follow-up draft.

Required controls:

- customer-safe candidate marker;
- approved-to-mention category chips;
- suppress-from-customer reason;
- sensitive content marker;
- manager-review-required marker;
- link to source care/task evidence.

No note goes directly to a customer. Customer update draft generation may read approved or reviewable note evidence and produce draft text with citations to source notes/tasks; staff/manager approval is required before send.

## Note types

| Note type | Typical chips | Default attachment | Default audience | Review trigger |
| --- | --- | --- | --- | --- |
| Feeding observation | ate all, ate some, refused, vomited, water refreshed, instruction unclear | pet + stay + feeding task when present | internal-only; customer-safe only after review if routine | refusal, vomiting, conflicting instruction, unclear amount, medical concern |
| Medication observation | verified med, administered dose, skipped dose, customer clarification needed, vet clarification needed | pet + stay + medication task | internal-only | any dose/schedule/source ambiguity, skipped dose, side effect, storage issue |
| Potty / elimination | normal potty, accident, diarrhea, blood observed, cleanup needed | pet + stay; cleaning task if created | internal-only; routine summary candidate after review | diarrhea/blood/repeated accidents/medical concern |
| Behavior / temperament | playful, calm, anxious, reactive, resource guarding, bite attempt, escape risk, needs individual care | pet + stay; playgroup assessment/incident task when relevant | internal-only by default | aggression, injury, incident, group-play restriction/reinstatement, sensitive customer wording |
| Playgroup / enrichment | joined group, moved groups, individual play, rest break, overstimulated, staff review needed | pet + reservation/stay + playgroup assignment evidence | internal-only; customer-safe only after staff confirmation | unknown/stale temperament, incident, capacity/ratio exception, reassignment due to safety |
| Health / medical observation | limping, coughing, lethargic, wound noticed, eating concern, vet contact needed | pet + stay + incident/manager task when needed | internal-only | any health signal, medical ambiguity, customer/vet follow-up |
| Incident / safety | injury, altercation, bite/scratch, escape attempt, facility hazard, manager notified | pet/reservation/location + incident record/task | internal-only | always manager/lead review; blocks autonomous customer send |
| Grooming / bath / DaySpa | service started, service complete, matting noticed, add-on recommended, pet stressed | reservation + service add-on/task | internal-only; customer-safe draft after review | injury/stress/sensitive recommendation/charge implication |
| Training | session complete, skill practiced, follow-up recommended, behavior concern | reservation + service add-on/task | internal-only; customer-safe draft after review | sensitive behavior claim or package/charge implication |
| Belongings / front desk | belongings received, label missing, item returned, owner question, signature needed | reservation/customer + check-in/out task | internal-only or customer-safe script if routine | lost/damaged item, dispute, missing required item |
| Document / vaccine | document received, unreadable, expired, mismatch, manager review | pet + document review task | internal-only | vaccine approval/eligibility remains staff/manager-gated |
| Payment / policy note | payment question, deposit issue, refund request, waiver/discount question | customer + reservation + payment follow-up task | internal-only | always routed to approved payment/policy gate; no AI decision |
| Customer message draft note | owner asked X, draft update, do not mention yet, approved routine highlight | customer + reservation + message draft | draft-only | sensitive topic, complaint, policy exception, incident, medical/behavior detail |
| General operational note | room needs cleaning, supply low, maintenance, schedule issue | location/task; pet/stay if specific | internal-only | safety hazard, capacity/room hold, staffing exception |

## Structured quick chips

Chips should be semantic capture aids, not final policy decisions. They should store typed values plus optional staff note text.

Recommended chip groups:

- Routine care: ate all, ate some, refused, water refreshed, potty normal, cleaned room, walk complete, enrichment complete.
- Exceptions: refused food, skipped medication, vomiting, diarrhea, injury observed, behavior concern, missing belongings, document unreadable.
- Review gates: needs staff review, needs shift lead, needs manager, needs vet/customer clarification, do not use for customer update yet.
- Communication safety: customer-safe routine highlight, internal-only, manager wording review, suppress from update, draft follow-up.
- Play/daycare: eligible evidence observed, needs temperament review, individual care today, moved playgroup, rest break, incident restriction.
- Handoff: carry to next shift, next owner required, due before pickup, safety critical, evidence incomplete.

Chip behavior:

- Every chip has a stable semantic code, display label, allowed roles, default audience, severity, and review-gate mapping.
- Staff can add free text after selecting chips, but free text cannot weaken the chip's review gate. Example: selecting `skipped medication` always creates/escalates medication review even if text says "probably fine".
- Chip sets are context-filtered by note attachment and task kind. Medication chips appear on medication tasks; payment/policy chips do not appear on kennel care surfaces except as escalation shortcuts.
- Locations may configure labels and allowed chips, but the underlying semantic codes remain portable for audit and automation.

## Free-text notes

Free text is required because real operations contain messy context, provider imports, customer statements, and staff observations that do not fit predefined chips.

UX rules:

- Default to internal-only unless the user explicitly marks a routine, customer-safe candidate or the note is entered in a customer-update evidence drawer.
- Encourage short factual observations: what was seen/done, when, by whom, and what follow-up is needed.
- Warn when free text appears to contain sensitive categories: health, medication, allergy, injury, aggression, payment/refund, legal/policy exception, staff/customer conflict, personal data not needed for care.
- Offer structured follow-up chips after text entry, but never silently reinterpret free text into executable medication, medical, eligibility, payment, or customer-message decisions.
- Preserve original text as source evidence with redaction controls; downstream summaries should use redacted semantic values where possible.

## Photos and attachments

Photos/attachments are evidence references, not automatic truth.

Allowed MVP attachment purposes:

- pet routine photo for customer-update draft consideration;
- meal/medication package label evidence for staff/manager review;
- document/vaccine image for document review queue;
- belongings photo at check-in/out;
- room/cleaning/maintenance evidence;
- incident/injury evidence with restricted visibility.

Photo review rules:

1. Staff must choose or confirm an attachment purpose.
2. The UI must separate `routine customer-update candidate` from `restricted incident/medical/document evidence`.
3. Incident, injury, medical, medication label, vaccine/document, payment, or identifying third-party images are restricted by default and cannot be used in customer updates without authorized review.
4. Photos with other customers/staff, other pets not part of the reservation, facility security details, documents, payment data, or visible private contact info require redaction or exclusion before customer use.
5. AI/image analysis may suggest tags or redaction needs only as draft assistance. A staff/manager actor must approve use in customer-facing drafts.
6. Deleted/rejected customer-use photos should remain auditable as restricted evidence if needed for incidents, documents, or task completion, subject to retention policy.

## Voice-to-text later

Voice capture is a future/later capability, not an MVP assumption. Define requirements now so the note model does not block it later.

Future requirements:

- Voice-to-text creates a draft free-text note with transcript source, actor, timestamp, subject, confidence/quality metadata, and original audio retention setting if policy permits.
- The transcribed note must be reviewed/accepted by the speaking staff member before it can complete tasks, enter handoff, or feed customer-message drafts.
- The UI should prompt for a note type and attachment before or immediately after recording.
- Sensitive-keyword detection should conservatively mark transcripts internal-only and review-required until staff confirms.
- Voice transcripts must support correction history; the audit trail should distinguish machine transcript, staff correction, and manager review.
- Voice should not be used to capture executable medication instructions without explicit structured verification of medication name, dose, route, schedule, source, and authorized reviewer.

## Internal-only vs customer-safe boundary

Every note has an audience state:

- `InternalOnly`: visible to staff/manager workflows; not eligible for customer drafts except as a source requiring review.
- `CustomerSafeCandidate`: staff believes the note may be used in a customer update, but it still needs draft review before send.
- `CustomerApprovedSource`: authorized actor approved the note or selected portions as source evidence for a customer-facing draft.
- `RestrictedSensitive`: incident, health, medication, allergy, aggression, policy, payment, legal, personnel, or privacy-sensitive evidence with restricted visibility and manager/lead review.
- `SuppressedFromCustomer`: intentionally excluded from customer updates with a reason and reviewer.

Boundary rules:

- Default for care, behavior, medication, incident, document, payment, and policy notes is `InternalOnly` or `RestrictedSensitive`.
- Routine positive updates can be `CustomerSafeCandidate` when authored by an authorized staff role and not contradicted by open incidents/review gates.
- Customer-safe does not mean sendable. It means eligible as input to a draft that still needs approval.
- Customer message drafts must cite source notes/tasks and show any suppressed or conflicting evidence to the reviewer.
- AI may propose safer wording, but it cannot downgrade a restricted note or approve send.

## Role permissions

Role permissions should use the provisional roles from `inputs.md` until a pilot role taxonomy is approved.

| Role | Can create notes | Can mark customer-safe candidate | Can attach restricted media | Can close review | Can approve customer-message source |
| --- | --- | --- | --- | --- | --- |
| Front desk / host | pet/reservation/customer/front-desk notes, belongings, document intake, customer questions | routine front-desk facts and owner preferences | document/belongings/customer-provided evidence | only ordinary collection tasks approved for front desk | routine scripts/drafts only if policy permits; sensitive messages require manager |
| Kennel technician / care staff | feeding, medication evidence, potty, cleaning, behavior observations, handoff notes | routine positive care observations | care, medication label, room/cleaning evidence; incident evidence if involved | care task completion when instructions are reviewed and role-authorized | routine care highlights as candidates, not final approval for sensitive facts |
| Playgroup / daycare staff | playgroup/enrichment observations, reassignment signals, behavior/incident notes | routine play/enrichment highlights | playgroup/incident evidence | staff-confirmed play observations where policy permits; no automation override | routine highlights as candidates; incidents/restrictions need manager |
| Groomer / bath / DaySpa staff | service evidence, grooming notes, condition concerns, add-on follow-up | routine service completion/highlight | service photos and condition evidence | grooming/bath task evidence when authorized | routine service notes as candidates; injury/stress/charge implications need review |
| Trainer | session notes, skill practiced, follow-up recommendation, behavior concern | routine training progress | training evidence | training task evidence when authorized | routine progress as candidate; sensitive behavior/package claims need review |
| Lead staff / shift lead | all ordinary operational notes, handoff acceptance/amendments, escalations | routine operational/customer-safe candidates | restricted operational evidence | ordinary blocked-task and handoff review within policy | may approve non-sensitive drafts if location policy permits |
| Manager / admin | all note types | yes | yes | manager-only gates, overrides, incidents, sensitive wording, policy exceptions | final approval for sensitive customer-facing sources/drafts |
| AI workflow worker / system | cannot author live staff observations; may draft extracted/summarized notes clearly marked `AiExtractedDraft` | no | no | no | no |

Permission invariants:

- Only human staff/manager actors attest that an event happened or a care task was completed.
- Manager/admin is required for payment/refund/waiver/discount/credit/cancellation exceptions, overbooking/waitlist exceptions, group-play reinstatement after incident, sensitive incident/medical/behavior messaging, and policy exceptions.
- AI-extracted draft notes must be reviewed before becoming operational evidence.

## Attachment targets and subject rules

A note may attach to multiple subjects, but one primary subject is required.

| Target | Use when | Required context | Important boundary |
| --- | --- | --- | --- |
| Pet | profile-level observation, long-lived care/behavior/document evidence | pet id, source, actor, review state | future stays may use it only after appropriate review/freshness checks |
| Reservation | check-in/out, payment, belongings, owner request, service plan | reservation id and customer/pet linkage | no reservation status mutation from note alone |
| Stay | active boarding/daycare execution, care notes, daily update evidence | stay/reservation id, date/time, pet id for pet-specific notes | open care notes may affect handoff/review, not automatic provider changes |
| Staff task | completion evidence, blocked reason, escalation, follow-up | task id, actor, outcome, evidence | AI cannot complete tasks; free-text/media-only evidence may require review |
| Incident | injury, aggression, safety, lost/escaped pet, hazard, complaint | incident id or incident draft, severity, manager/lead owner | restricted by default; customer sends suppressed until reviewed |
| Message draft | customer-update source, checkout summary source, follow-up draft | draft id or reservation/customer id, audience state, reviewer | source note remains distinct from approved outbound text |
| Location / operating day | supply, facility, staffing, capacity, maintenance, handoff context | location/date, role/owner, severity | pet/customer details should not be copied unless needed and authorized |

Subject rules:

- A note attached to a task should inherit task context but still store its own note type, audience state, actor, and source evidence.
- A note attached to a message draft is not itself customer-facing; the approved outbound message is a separate artifact.
- Incident notes may reference pets/reservations/customers but should be managed through incident review surfaces to avoid leaking restricted evidence into routine timelines.
- Payment/policy notes attach to customer/reservation follow-up tasks and must not be mixed into routine care summaries.

## Flags and escalations

The capture UI must make escalation one tap away. It should also auto-suggest escalation when selected chips or text/media category requires it.

Escalation levels:

- `NeedsStaffReview`: ordinary care/document/assignment uncertainty.
- `NeedsShiftLead`: handoff completeness, blocked task, ordinary cross-role coordination.
- `NeedsManagerReview`: incident, policy exception, payment/deposit/refund issue, sensitive customer language, capacity/ratio exception, overbooking/waitlist, group-play reinstatement/suspension, complaint.
- `SafetyCritical`: injury, bite/aggression, medical distress, escaped/lost pet, facility hazard, severe medication issue. Requires immediate lead/manager routing according to local policy.
- `NeedsExternalClarification`: customer, veterinarian, provider, or document source clarification is required.

Escalation behavior:

- Flagged notes create or update a review item/task draft with source link, reason, severity, owner, and due timing.
- Rejected notes that attempted customer-safe use move to manager/lead review when the rejection reason indicates sensitivity, contradiction, unsafe wording, bad photo, or insufficient source evidence.
- Escalations should appear in daily brief/handoff packets when unresolved.
- Safety-critical flags cannot be dismissed by AI and should not wait for routine shift handoff.

## Audit history

Every note and note-derived action must retain an audit trail.

Minimum audit fields:

- note id;
- actor (`Staff`, `Manager`, `System`, `Agent`) and actor id where applicable;
- created/updated/reviewed timestamps with location timezone context;
- location id;
- primary and secondary subjects;
- note type and semantic chips;
- source channel: staff typed, future voice transcript, provider import, customer supplied, AI extracted draft, task evidence, attachment upload;
- original text/media reference and redacted display value;
- audience state transitions and reviewer;
- review gate, escalation level, owner, and status;
- customer-message draft references and approval/suppression outcomes;
- task/incident/handoff records created or amended from the note.

Audit rules:

- Edits do not overwrite history. They create revisions with actor, reason when required, and previous value reference.
- Redaction must preserve enough source reference for authorized review while avoiding broad debug/log exposure.
- AI summaries should cite note ids and should not become the only audit evidence for staff actions.
- Rejected customer-use decisions should keep rejection reason: unsafe wording, sensitive content, unapproved photo, contradicted evidence, stale source, wrong subject, insufficient reviewer authority, or duplicate/irrelevant.

## Customer-update draft generation inputs

Customer-update draft generation may use note evidence only through a bounded input packet.

Input packet fields:

- reservation/stay/customer/pet ids;
- service context: boarding, daycare/day play, day boarding, grooming/DaySpa, training;
- desired update type: daily/Pawgress, checkout summary, customer follow-up, incident follow-up draft;
- approved or candidate source notes with note id, type, timestamp, author role, audience state, and redacted content;
- related completed task evidence and unresolved task/review gates;
- approved photos or photo candidates with review status;
- suppressed/restricted note summary visible to reviewer but not used in draft text unless approved;
- location/customer communication preferences if approved;
- policy gates for sensitive content.

Drafting rules:

- The draft should prefer routine positive, factual, staff-approved observations.
- The draft must not hide open safety, medical, incident, behavior, payment, or policy-exception review gates from the reviewer.
- If only internal/restricted notes exist, produce `no safe draft available` plus review reasons rather than inventing cheerful copy.
- Medical, medication, allergy, injury, aggression, incident, refusal, eligibility, payment, refund, cancellation, or policy-exception language requires manager or approved role review before send.
- Drafts should include citations to source note/task/photo ids for reviewer traceability.

## Rejected and flagged note review flow

### Rejected customer-safe candidate

A note/photo/draft source can be rejected from customer use without deleting the operational note.

Flow:

1. Reviewer rejects with reason.
2. Note audience state becomes `SuppressedFromCustomer` or remains `RestrictedSensitive`.
3. The source remains available internally for care, incident, document, or task evidence if permitted.
4. If rejection reason implies operational risk, create/attach manager or lead review task.
5. Customer-update draft generator excludes the note unless a later authorized review changes the state.

Common rejection reasons:

- sensitive medical/medication/behavior/incident detail;
- unapproved or unsafe wording;
- photo contains third party/private info/document/payment detail;
- source evidence is stale, contradicted, or not staff-approved;
- note belongs to wrong pet/reservation/customer;
- customer already received a conflicting update;
- reviewer lacks authority and escalates.

### Flagged operational note

Flow:

1. Staff selects escalation chip or system suggests one from chip/text/media category.
2. Staff confirms severity and owner when possible.
3. The system creates/updates a review item, staff task, incident draft, or handoff item.
4. The note appears in the appropriate queue: staff review, shift lead review, manager review, safety critical, external clarification.
5. Resolution records reviewer, decision, task/incident outcome, customer-message boundary, and audit event.

## MVP vs later

MVP should include:

- typed note types and structured chips;
- free-text note capture;
- attachment/photo references with purpose and review status;
- internal-only/customer-safe/restricted/suppressed audience states;
- role-aware note creation and review gates;
- escalation routing into task/handoff/incident/message-draft review surfaces;
- audit history and source links;
- customer-update draft input packet definition.

Later should include:

- voice-to-text capture and correction history;
- image-based redaction suggestions;
- configurable per-location chip catalogs and role permissions;
- richer incident case management;
- staff mobile offline capture/sync conflict handling;
- AI-assisted duplicate detection and structured extraction from provider imports;
- analytics on note volume, review latency, recurring care risks, and customer-update source quality.

## Open questions

1. Which exact pilot roles may approve routine customer-safe notes without manager review?
2. Which note types require photo evidence, prohibit photo evidence, or require dual review by policy?
3. What retention and deletion policy applies to restricted incident/medical/document media?
4. Should routine customer-update candidates be available to all care staff or only selected roles after training?
5. What is the first integration mode for provider/Gingr notes: read-only import, copy/paste, approved write-back, or no integration?
6. What SLAs and notification channels should apply to safety-critical, manager-review, and external-clarification queues?
7. How should multi-pet reservations handle shared notes versus pet-specific notes in customer drafts?

## Conservative rule

When the note source, subject, meaning, review authority, or customer-safety status is unclear, keep the note internal, attach it to the narrowest truthful subject, preserve source evidence, and route it to review. Fast capture is successful only when later staff can trust what happened, who said it, what it affects, and what must not be sent to a customer without approval.
