# Incident/Escalation Agent inputs

Purpose: collect canonical source inputs for the Incident/Escalation Agent workflow before definition work starts. This is an input packet, not live operating policy. Anything labeled provisional is a conservative modeling assumption that must remain reviewable until a location policy, manager decision, or implementation artifact approves it.

Status: draft input collection. This document does not authorize live incident closure, medical/safety decisions, behavior or eligibility decisions, provider writes, staff-task completion, or owner-facing messages.

## Source index

Primary sources checked:

- `docs/domain/petsuites/daycare/implications/04-incident-tracking.md` — current incident-tracking model, actors, inputs, decisions, outputs, invariants, workflow events, staff tasks, and agent boundary.
- `docs/domain/petsuites/daycare/implications/05-pet-health-behavior-notes.md` — typed health/behavior note model, review states, note-to-eligibility impacts, customer-safe summary boundary, and no-diagnosis rules.
- `docs/workflows/staff-operations-parts/inputs.md` — staff roles, operating-day workflow, staff-task model, approval gates, customer-message posture, and downstream conservative rule.
- `docs/architecture/pet-resort-workflow-events.md` — MVP workflow event catalog and `incident.created` approval posture.
- `docs/architecture/workflow-result-envelope.md` — workflow result contract for drafts, recommendations, tasks, risk flags, verification, and human review reasons.
- `docs/architecture/ai-runtime-memory-context-policy.md` — runtime context, privacy, audit, incident/behavior, medical, staff-note, and customer-message boundaries.
- `docs/data-model/workflow-queue-retry-dead-letter.md` — durable queue/outbox model, waiting-for-human conditions, approved-action side-effect model, and redaction expectations.
- `docs/integrations/gingr/sdk-webhooks.md` — Gingr `incident_created` / `incident_edited` webhook boundary and signature-verification requirements.
- `domain/src/entities.rs` — current canonical identities, pet/customer/reservation/care/temperament/audit/actor surfaces.
- `domain/src/operations.rs` — current staff task, daily brief, pet-care watchlist, operations risk/action, daycare contract, incident policy, and daycare readiness surfaces.
- `domain/src/policy.rs` — current `ReviewGate`, `AutomationLevel`, conservative play eligibility policy, and behavior-flag review routing.
- `domain/src/workflow.rs` — current `WorkflowEvent`, `WorkflowEventType::IncidentCreated`, `PolicyContext`, allowed actions, result status, and recommended-action surface.

Missing or caveated sources:

- No final location-approved incident SOP, emergency/vet escalation tree, legal/compliance language, notification routing matrix, or owner-message template library was found in this repo.
- No final incident aggregate/table implementation exists yet. The strongest current source is the daycare incident modeling artifact plus existing Rust/domain contracts.
- No canonical provider payload shape for Gingr incident `entity_data` was found. Current Gingr docs establish event names and verification only; incident payload fields must stay quarantined until fixtures or integration tests prove the shape.
- Exact pilot/local policy for severity thresholds, who may approve each class, required owner-notice timing, emergency contact procedure, photo/media retention, and legal review is unknown.

## Canonical workflow posture

The Incident/Escalation Agent is a read/draft/recommend workflow. It may:

- Read typed incident, pet, reservation, care, temperament, task, audit, media-reference, and policy context that the event packet or approved tools authorize.
- Summarize a source-grounded incident timeline and missing-field checklist.
- Propose a conservative severity/disposition classification for human review.
- Draft manager/lead review packets.
- Draft owner-facing copy as a review-gated artifact only.
- Recommend or draft internal tasks such as incident follow-up, care/document review, playgroup reassessment, customer follow-up, and daily-update review.
- Flag risks for manager daily briefs and pet-care watchlists.

It must not:

- Diagnose, infer treatment, or tell an owner what medical action to take.
- Minimize, hide, soften away, or omit material incident, health, behavior, safety, or uncertainty facts to make a message sound smoother.
- Blame parties, make liability/legal claims, or publish public/legal responses.
- Close an incident autonomously.
- Downgrade medium/high/emergency severity or clear restrictions autonomously.
- Reinstate group play or remove behavior/care restrictions autonomously.
- Mark staff care/medication/incident tasks complete without authorized staff/manager evidence.
- Send owner-facing messages without explicit human approval.
- Write provider records, mutate reservation/status/eligibility, or execute outbox sends without an approved action record.

## Incident policy and response boundaries

Canonical incident scope from current sources:

- An incident can include staff-observed safety, behavior, health, escape, injury, near-miss, medication/care-instruction, facility, owner-notice, or customer-reported-after-pickup events that matter to operations.
- Current daycare triggers include bites/attempted bites, repeated mounting, guarding, escalating chase, barrier reactivity, human selectivity, stress/inability to settle, injury, lameness, vomiting/diarrhea, allergic reaction, heat stress, missed medication, food exposure, care-plan deviation, escape attempt, ratio/capacity breach, incompatible playgroup mix, cleaning/sanitation hazard, near-miss, and unresolved incident text detected by another workflow.
- The current coarse daycare incident policy enum is `StaffNoteOnly`, `ManagerReviewAndCustomerNotice`, and `SuspendGroupPlayPendingReview`. The incident-tracking artifact recommends refining this into explicit severity, disposition, eligibility, and communication policies before implementation.
- Current recommended severity vocabulary: `NoteOnly`, `OwnerNoticeRequired`, `ManagerReviewRequired`, `SuspendGroupPlayPendingReview`, and `EmergencyOrVetEscalation`.
- For this workflow, preserve the requested owner classifications as explicit review categories:
  - `Low` / note-only or internal follow-up: may produce internal review/task output, but owner-facing copy remains approval-gated if drafted.
  - `Medium` / owner notice or manager review likely: manager/lead review required before final classification, owner copy, or closure.
  - `High` / suspending, injury/health, behavior, safety, legal, or policy-sensitive incident: manager approval required; restrictions remain active until reviewed.
  - `Emergency` / vet/safety escalation: manager/safety handling required; cannot be downgraded or customer-closed by an agent.
- Provisional mapping: the existing detailed severity enum can map to the requested `Low`/`Medium`/`High`/`Emergency` labels for UI/triage, but this mapping is not final policy until approved.

Response boundary rules:

- Unknown or incomplete incident facts create review work and conservative restrictions, not clearance.
- `IncidentStatus::Closed` should be impossible while active restrictions, unresolved required review gates, or open incident follow-up tasks remain.
- `SuspendGroupPlayPendingReview` always creates or preserves a group-play suspension/restriction until manager-approved disposition clears it.
- `DocumentOnly` / note-only outcomes are not allowed for bite/aggression hard stops, medical/care ambiguity, escape/near-miss, owner-notice events, or emergency/vet escalation.
- Customer-message drafts and send proofs are separate values; draft creation is not approval or delivery.

## Staff reporting process inputs

Canonical staff reporting process:

1. Staff, lead, manager, provider import, or customer-after-pickup report creates or updates an incident signal.
2. The report must capture enough typed fields to explain who/what/when/where/source, involved pet/reservation/location, observed facts, immediate action taken, and unknowns.
3. Missing required fields create `NeedsStaffCompletion` / follow-up work; the workflow must not fabricate missing details.
4. Lead/manager validates immediate operational response: continue care, move to individual care/rest lane, isolate safely pending pickup, request manager review, or emergency/vet escalation.
5. The workflow may draft an internal review packet and recommended tasks, including `IncidentFollowUp`, `DocumentReview`, `PlaygroupAssessment`, `DailyUpdateDraft`, and `CustomerFollowUp`.
6. Manager/staff approval is required before customer notice, final medium/high/emergency classification, closure, or restriction clearance.
7. Shift-handoff and daily-brief outputs should surface unresolved incidents, active restrictions, missed follow-ups, and safety/care risks.

Minimum incident capture inputs:

- Location, operating day, source system/source actor, reporter/staff identity when available.
- Pet, customer, reservation/attendance/stay references.
- Observed-at timestamp and time/place/context.
- Category and severity candidate.
- Typed observed facts: behavior, injury/health, care-instruction deviation, facility/supervision, escape/near-miss, customer-reported-after-pickup, other reviewed.
- Narrative or fact summary, with raw text separated from customer-safe wording.
- Immediate action taken and current care mode / affected care mode.
- Current review gates, restrictions, staff tasks, media/attachment references, audit/source refs, and owner-notification status.

## Existing and target data-model inputs

Existing implemented anchors:

- Identities and actors: `LocationId`, `CustomerId`, `PetId`, `ReservationId`, `StaffId`, `ManagerId`, `ActorRef::{Customer, Staff, Manager, System, Agent}`.
- Customer and pet: `Customer`, `PortalAccountRef`, `ContactChannel`, `Pet`, `Species`, `Sex`, `SpayNeuterStatus`, `CareProfile`, `MedicationInstruction`, `TemperamentProfile`.
- Care profile: feeding instructions, medications, allergies, medical conditions, emergency contact, veterinarian contact, and medication review requirement.
- Temperament profile: group-play observation, people orientation, rating, behavior observations, and staff notes.
- Reservation: `Reservation`, `ServiceKind::{Boarding, DayPlay, DayBoarding, Grooming, Training, DaySpa}`, `ReservationStatus`, `ReservationSource`, add-ons, hard stops.
- Hard stops relevant to incidents/eligibility: `IneligibleForGroupPlay`, `MedicalOrMedicationReviewRequired`, `BehaviorReviewRequired`, and vaccine/deposit/age gates.
- Staff tasks: `StaffTask`, `StaffTaskKind`, `StaffTaskStatus`, `StaffTaskPriority`, `StaffTaskAssignment`, `StaffRole`, `StaffTaskSource`.
- Incident-adjacent task kinds: `IncidentFollowUp`, `PlaygroupAssessment`, `DocumentReview`, `MedicationAdministration`, `DailyUpdateDraft`, `CustomerFollowUp`.
- Operations projections: `PetCareWatch`, `PetCareWatchReason::{MedicationDue, FeedingException, AnxietyOrStressFlag, BehaviorReview, IncidentFollowUp}`, `OperationsRisk::PetSafetyOrCareRisk`, `DailyBriefSection::PetCareWatchlist`, and `OperationsAction::{CreateInternalTask, DraftCustomerMessage, EscalateToManager}`.
- Policy: `ReviewGate::{ManagerApproval, MedicalDocumentReview, BehaviorReview, CustomerMessageApproval, RefundOrDepositException}`, `AutomationLevel`, conservative play eligibility policy, and behavior-flag review routing.
- Workflow: `WorkflowEvent`, `WorkflowEventType::IncidentCreated`, `PolicyContext`, `AllowedAction`, `WorkflowResult`, `WorkflowStatus`, `RecommendedAction`.
- Audit: `AuditEvent`, `AuditSubject::{Customer, Pet, Reservation, Location, WorkflowEvent, External}`, `AuditAction`, typed audit metadata, and actor attribution.

Target incident model from current artifacts:

- `Incident` / `IncidentId` aggregate with report facts, severity, status, review gates, audit events, restrictions, dispositions, and disposition history.
- `IncidentReport` with builder-enforced required pet/location/observed-at/category/source fields.
- `IncidentCategory`, `IncidentSeverity`, `IncidentStatus`, `IncidentDisposition`, `IncidentRestriction`, `IncidentRestrictionStatus`, and review requirement values.
- `IncidentNarrative` and `IncidentFactSummary` as separate, length-limited, redacted-debug values; customer-safe wording should be separate from raw/internal text.
- `IncidentMediaRef` / `IncidentDocumentRef` / photo or attachment refs stored as references to approved storage, not raw blobs in the aggregate.
- `CustomerNoticeDraftId` as a reference to a draft, not proof of approval or send.
- Repositories: incident, incident-task, eligibility, roster/context, note/audit, and boundary storage adapters.
- Workflow events: incident reported/created, severity classified, manager review requested, restriction applied/cleared, customer notice drafted/approved, closed, voided with audit.

Provider/boundary inputs:

- Gingr exposes `incident_created` and `incident_edited` webhook names, but the raw webhook must be parsed as untrusted, signature-verified, and semantically mapped before a domain workflow sees it.
- Raw provider payloads, raw OCR, raw photos/media, and untyped notes stay in boundary storage and are referenced by evidence IDs. Workflow payloads should expose only typed, policy-ready facts.
- Unknown provider incident shapes must produce blocked/review states or mapping tasks, not policy decisions.

## Notification policy inputs

Known notification posture:

- Owner/customer-facing incident messages are always drafts until human approved. This includes pickup conversation notes, daily/Pawgress updates containing incident facts, incident summaries, apologies, follow-up instructions, review requests after incidents, and any legal/safety/medical/behavior/payment-sensitive message.
- Manager attention is required for blocked/manager-review staff tasks, high/critical priority tasks, incident follow-up tasks, medication tasks, document review tasks, pet-safety/care risks, customer-experience risks, capacity/ratio exceptions, suspensions, high/emergency incidents, customer-message approval, and policy exceptions.
- Staff/lead notifications may be internal task drafts or review packets when source-backed; production auto-creation and assignment rules are not approved by these inputs.
- Outbound side effects should use an approved-action/outbox model. A queued workflow result may draft messages/tasks, but sends/provider writes require approval records and idempotent outbox execution.

Unknown notification policy:

- Exact channels for manager/owner/staff alerts (SMS, email, portal, Slack, app push, phone call) are not canonical in this repo.
- Exact SLA/timing for owner notice, manager escalation, emergency call, and staff follow-up is unknown.
- Exact escalation hierarchy and after-hours routing are unknown.
- Exact criteria for when staff vs manager vs owner receives a notification are not final beyond the review gates above.

Provisional notification assumptions:

- Internal manager/lead review packets can be drafted for all medium/high/emergency candidates and all incomplete incident reports.
- Emergency candidates should produce immediate internal escalation recommendations, but the agent still does not contact owner/vet/emergency services autonomously unless a later approved emergency procedure explicitly grants a narrow tool path.
- Owner-facing drafts should default to `RequiresApproval { gate: CustomerMessageApproval }` and include source refs, unknowns, and redaction notes.

## Legal and medical caution language

Canonical cautions to preserve in workflow definitions:

- No diagnosis: the agent may describe observed facts and staff/source reports, but may not diagnose, infer treatment, or tell the owner what medical action to take.
- No minimization: the agent may not suppress concerning facts, uncertainty, active restrictions, or required follow-up to make the incident sound less serious.
- No autonomous serious-incident closure: medium, high, emergency, owner-notice-required, suspending, medical/care ambiguous, behavior/eligibility affecting, legal/compliance-sensitive, or manager-review incidents require human review before closure.
- No unreviewed owner-facing messages: all owner-facing incident messages require explicit human approval. Drafting is allowed only as a draft/review artifact.
- No liability/legal claims: avoid blame assignment, promises, admissions, legal conclusions, or policy interpretations unless approved legal/manager language exists.
- Customer-safe language is a separate approved projection, not reused raw staff notes or internal incident narratives.
- Staff-only notes and internal investigation details must not appear in owner-facing drafts unless transformed into approved customer-safe wording.
- Graphic/unnecessary details should be omitted from customer drafts and minimized in manager briefs unless operationally necessary for the authorized recipient.

## Approval gates to preserve

These gates must survive downstream decomposition:

1. Owner-facing incident messages.
   - Every owner-facing incident message, daily update with incident facts, follow-up request, apology, review request suppression/replacement, or legal/safety/health/behavior copy requires explicit human approval.
   - Use `CustomerMessageApproval` at minimum; manager approval is also required for sensitive/severe incidents unless a later policy says otherwise.

2. Medium/high/emergency classifications.
   - Medium, high, emergency, owner-notice-required, manager-review-required, group-play-suspending, medical/care ambiguous, legal/compliance-sensitive, or emergency/vet escalation classifications require human approval.
   - The agent may propose a classification with evidence and unknowns; it may not finalize, downgrade, or close it.

3. Behavior flags affecting eligibility.
   - Bite history, explicit manager-review behavior flags, stressed group setting, intro-assessment-needed, review-required temperament rating, active incident restrictions, unknown/stale/conflicting behavior facts, and spay/neuter review conditions must preserve `BehaviorReview` and/or `ManagerApproval` gates.
   - The agent must not clear group-play restrictions or mark a pet eligible for group play after an incident.

4. Medical/care ambiguity.
   - Injury/health, medication, allergy, feeding, care-instruction deviation, emergency/vet-contact, and medical-document ambiguity require `MedicalDocumentReview`, manager/staff care review, or emergency workflow gates as appropriate.
   - The agent must not convert vague notes into executable care instructions.

5. Incident closure and restriction clearance.
   - Active restrictions, unresolved review gates, open incident follow-up tasks, unresolved owner notice, and incomplete reports block closure.
   - Restriction clearance, group-play reinstatement, and final closure require manager/staff approval and audit evidence.

6. Provider/system mutations.
   - Provider incident writes, reservation status changes, eligibility changes, message sends, task completion, and outbox execution require approved actions and audit records.

7. Photos/attachments/media.
   - Photo/media use in incident review is by reference only and must respect approved storage, retention, privacy, and review policy.
   - No raw media should be sent to a runtime or customer draft unless the workflow and data-sent-to-runtime gate explicitly approves it.

## Known constraints

- The product posture is conservative: AI/workflows read, summarize, draft, recommend, and flag risk; they do not independently decide medical, safety, payment, eligibility, capacity-exception, customer-message, or provider-write actions.
- `WorkflowResult` statuses and recommendations are not execution receipts. Proposed actions remain proposals unless a tool/outbox/audit record proves execution.
- Required source evidence, policy snapshot IDs, review gates, risk flags, and verification notes must be preserved in each output.
- Raw provider payloads, raw OCR, unredacted documents, broad histories, full customer profiles, payment secrets, signed webhook material, and unscoped staff-only notes are not default runtime context.
- Audit must distinguish source records, deterministic policy decisions, human approvals, model drafts, external tool outcomes, and final state changes.
- Current Rust types do not yet contain a full incident aggregate/table, incident-specific task payloads, message approval records, media/attachment aggregate, or temporary profile flags. Downstream definitions must either propose these explicitly or map to current extension/audit/task surfaces as provisional.
- Current `ReviewGate` lacks incident/legal/emergency-specific variants beyond manager, medical document, behavior, customer-message, and refund/deposit gates. Use existing gates conservatively and label any richer gate vocabulary as proposed.

## Unknowns for downstream definition work

1. What is the approved local incident severity taxonomy and exact mapping to low/medium/high/emergency?
2. Which incident categories require owner notice, manager approval, legal review, vet/emergency escalation, or external reporting?
3. Who may approve medium, high, emergency, owner-message, restriction-clearance, and closure gates by role/location/time of day?
4. What owner-notice timing/SLA applies by severity, and which channels/templates are approved?
5. What emergency procedure exists for vet contact, owner phone call, transport, after-hours escalation, and audit evidence?
6. What exact photo/attachment storage, retention, access, and owner-sharing policy applies?
7. What source-of-truth table owns incidents, restrictions, temporary profile flags, and closure records?
8. Should temporary profile flags live on pet profile, daycare eligibility snapshots, incident restrictions, staff tasks, or a dedicated profile-flag aggregate?
9. Which internal staff-task drafts may be auto-created in production, and which remain review-only?
10. What Gingr/provider incident fields are available in real payloads, and which writes, if any, are in MVP scope?
11. What customer-safe legal/medical caution templates are approved for owner drafts?
12. How should incidents suppress review-request/reputation workflows and marketing/customer follow-ups?

## Provisional assumptions

These are safe modeling defaults, not policy:

- Treat every incident workflow result as `needs_human_review` unless it only creates a low-risk internal summary/task draft and no customer-facing, eligibility, medical, closure, or provider-write effect.
- Treat unknown/conflicting source facts as review blockers, not as clearance.
- Treat any behavior or health note connected to an incident as at least staff review, and as manager review when it affects group play, owner notice, safety, emergency, or legal/compliance posture.
- Treat all owner-facing incident copy as draft-only with `CustomerMessageApproval` until a narrow deterministic send policy is approved.
- Treat current `IncidentFollowUp { pet_id }` tasks as provisional; later implementation should carry `incident_id` once the incident aggregate exists.
- Treat temporary profile flags as proposed typed restrictions/notes linked to the incident and pet until a profile-flag aggregate is designed.
- Treat notification routing as internal manager/lead/staff review packet generation only; exact channel delivery remains unresolved.

## Conservative downstream rule

When source facts or policy are missing, stale, contradictory, sensitive, provider-unverified, legally risky, medically ambiguous, behavior/eligibility affecting, or owner-facing, the Incident/Escalation Agent workflow should produce a sourced review packet, explicit unknowns, draft-only messages, and internal task recommendations. It must not invent policy, silently mark work ready, close the incident, clear restrictions, or send owner-facing messages.