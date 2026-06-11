# Customer history summary agent

Status: staff-facing workflow design handoff. This document defines a conservative customer/pet history summary used to prepare front-desk, lodging, daycare, grooming, training, care, and manager staff. It does not authorize customer-facing automation, customer profile edits, incident closure, medical/vaccine approval, eligibility decisions, discounts/refunds, provider writes, or outbound messages.

Source basis:

- `docs/workflows/crm-retention-parts/inputs.md` is the canonical CRM/retention input packet and suppression/review posture for this workflow.
- `docs/architecture/ai-runtime-memory-context-policy.md` defines minimized prompt packets, role/purpose scoped context, audit manifests, redaction, and the rule that AI output is not source truth.
- `docs/security/pet-resort-security-audit-parts/sensitive-data.md` defines default masking and AI runtime exposure rules for names, contact data, vaccine documents, medical/care details, incidents, payments, staff-only notes, and AI traces.
- `docs/workflows/incident-escalation-agent-parts/inputs.md` defines incident/behavior/medical/legal caution language and human review gates that history summaries must preserve.
- Staff operations, booking triage, customer messaging, payments/pricing, daycare, grooming, boarding, and training domain notes provide service-history and operational-context anchors.

## Purpose and non-goals

The agent answers: “What should authorized staff know before serving this customer and pet today?” The output is an internal preparation artifact with prominent risk/unresolved flags and source citations.

Allowed outcomes:

- Produce a staff-facing customer/pet history summary with cited source refs, freshness, missing/conflicting data markers, and review gates.
- Highlight unresolved concerns, safety/care/behavior/payment/document flags, suppression reasons, and manager/staff follow-up tasks.
- Suggest internal opportunities such as rebooking reminders, VIP/loyalty handling, package/membership review, grooming cadence reminder, daycare recurrence review, or retention follow-up candidates, only as staff tasks/review prompts.
- Route sensitive, stale, conflicting, or unresolved facts to human review.

Not allowed:

- Send or expose the summary to customers, owners, reviewers, public channels, or customer portals.
- Diagnose medical issues, infer treatment, approve vaccine/document status, clear behavior restrictions, close incidents, resolve complaints, or declare eligibility.
- Promise availability, booking acceptance, discounts, refunds, credits, waivers, service outcomes, or policy exceptions.
- Treat prior AI summaries, raw provider payloads, raw OCR, raw email/SMS threads, or staff free text as authoritative facts without underlying source evidence.
- Use staff-only notes as customer-safe language or leak internal policy, blame, investigation details, payment/provider internals, or unrelated customer history.

## Invocation scope

Each invocation must declare a narrow purpose and subject:

```text
customer_history_summary_request:
  scope:
    location_id
    customer_id
    pet_ids[]                 # explicit pet set; no household-wide expansion unless requested and authorized
    service_context           # front_desk | boarding | daycare | grooming | training | manager | checkout | retention_review
    service_date_or_window
    requester_role
    requester_actor_id
    timezone
    policy_refs[]
  output_visibility: staff_internal_only
  review_profile: routine | sensitive | manager_review_required
```

Default context window:

- Current and upcoming reservations/services first.
- Recent completed service history relevant to the requested service line.
- Longer-term patterns only when they drive risk, preference, cadence, VIP, package/membership, complaint, incident, or rebooking context.
- Cross-pet or household context only when operationally relevant and permitted by role/purpose.

## Input packet recommendation

```text
customer_history_summary_input:
  scope:
    location_id
    customer:
      customer_id
      display_name
      identity_confidence
      preferred_contact_channel_ref
      communication_preferences_refs[]
      consent_dnc_suppression_summary
    pets:
      - pet_id
        display_name
        species_breed_age_refs
        profile_status
        service_eligibility_summary_refs[]
        vaccine_document_status_refs[]
        care_flags_summary_refs[]
        behavior_flags_summary_refs[]
        feeding_medication_summary_refs[]
    requested_context:
      service_context
      service_date_or_window
      requester_role
      policy_refs[]
      redaction_profile
  history:
    reservations:
      upcoming[]
      active_or_in_care[]
      recent_completed[]
      cancellations_no_shows_waitlists[]
    services:
      boarding_stays[]
      daycare_attendance_pattern
      grooming_history
      training_history
      day_spa_or_addons[]
      packages_memberships[]
    customer_interactions:
      relevant_message_thread_refs[]
      customer_sentiment_refs[]
      complaint_refs[]
      review_refs[]
      last_contact_by_purpose_channel[]
    operational_context:
      staff_notes_refs[]
      manager_notes_refs[]
      open_tasks[]
      audit_or_approval_refs[]
      suppression_flags[]
      unresolved_concerns[]
  opportunities:
    rebooking_candidates[]
    service_reminder_candidates[]
    vip_or_loyalty_candidate
    retention_or_recovery_candidate
    package_membership_candidate[]
  source_manifest:
    source_refs[]
    source_trust_markers[]          # trusted | untrusted | imported | OCR_only | staff_note | provider_verified | customer_provided
    freshness_markers[]             # current | stale | missing | conflicting | superseded | source_pending
    redactions_applied[]
    denied_or_unavailable_sources[]
```

The context builder should provide normalized semantic records and refs, not raw provider payloads, raw documents, unbounded inbox/message history, raw payment instruments, or unscoped staff notes.

## Output schema

```text
customer_history_summary:
  schema_version
  summary_id
  generated_at
  generated_for:
    location_id
    customer_id
    pet_ids[]
    service_context
    service_date_or_window
    requester_role
  status: ready | needs_staff_review | needs_manager_review | failed_safely
  sensitivity:
    visibility: staff_internal_only
    redaction_profile
    contains_sensitive_flags[]       # incident | medical | medication | vaccine | behavior | payment | complaint | staff_internal | legal
    customer_facing_use: prohibited
  headline:
    one_line_context
    immediate_attention_flags[]      # unresolved and risk flags appear here first
  identities:
    customer:
      display_name
      customer_id
      identity_confidence
      preferred_contact_summary
      suppression_or_contact_caution
    pets:
      - display_name
        pet_id
        species_breed_age_summary
        current_status_or_visit_context
  urgent_flags:
    unresolved_concerns[]
    incident_or_safety_flags[]
    medical_medication_feeding_flags[]
    vaccine_document_flags[]
    behavior_or_eligibility_flags[]
    payment_deposit_billing_flags[]
    complaint_recovery_flags[]
    manager_review_required[]
  service_history:
    boarding_summary
    daycare_summary
    grooming_summary
    training_summary
    other_services_summary
    cancellations_no_shows_waitlists_summary
  preferences_and_handling:
    customer_preferences[]
    pet_preferences[]
    handling_notes[]
    feeding_medication_care_note_refs[]
    document_or_vaccine_status_refs[]
  issues_and_notes:
    open_issues[]
    resolved_but_relevant_issues[]
    manager_notes[]
    staff_notes[]
    operational_flags[]
  opportunities:
    rebooking_or_cadence_candidates[]
    service_reminders[]
    vip_or_loyalty_handling[]
    retention_or_recovery_follow_up[]
    package_membership_or_add_on_candidates[]
  recommended_staff_actions:
    - action_type
      owner_role
      reason
      priority
      source_refs[]
      review_gate
  unknowns_and_conflicts:
    missing_facts[]
    stale_facts[]
    conflicting_facts[]
    denied_sources[]
  source_citations:
    - claim_id
      source_ref
      source_type
      source_timestamp
      fetched_at
      trust_marker
      freshness
  audit:
    invocation_id
    prompt_packet_manifest_ref
    output_validation_result
    policy_refs[]
    model_runtime_ref
```

## Required sections and content rules

### 1. Risk-forward headline

The first screen must make unresolved concerns and risk flags impossible to miss. If any unresolved complaint, open incident, active behavior restriction, medication ambiguity, vaccine/document blocker, payment dispute, legal/privacy hold, DNC/suppression flag, or manager-review task exists, the headline must start with that before ordinary loyalty or service history.

Examples of safe internal language:

- “Manager review open: complaint about last boarding checkout; suppress review/rebooking outreach until resolved.”
- “Medication instructions have conflicting sources; use current care-task ref only after staff review.”
- “Grooming cadence suggests follow-up window, but active payment dispute suppresses outreach.”

### 2. Customer and pet identities

Include only identity fields needed for staff preparation: customer display name, customer ID, pet names/IDs, species/breed/age summary if relevant, identity confidence, and service context. Contact details should be summarized or masked unless the staff role needs them for a task or reviewed send workflow.

Identity conflicts, duplicate profiles, imported-only records, or low confidence must route to staff review. Do not merge identities or choose between conflicting owners/pets.

### 3. Pets, stays, daycare, grooming, and service history

Summarize relevant history by service line:

- Boarding: recent/upcoming stays, holiday/peak patterns, accommodation/add-on patterns, checkout/follow-up refs, care/watchlist refs.
- Daycare: attendance recurrence, missed visits/no-shows, package/membership state, group-play/individual-play posture, playgroup restrictions by ref.
- Grooming/DaySpa: last completed grooming anchor, ordinary cadence if supported by trusted history, groomer/handling preferences, coat/service notes by ref, no-show/deposit caveats.
- Training: program/package status, trainer progress refs, parent follow-up/homework/re-enrollment candidates if source-backed.
- Other add-ons/packages: recurring service/add-on use, package balance refs, expiration/renewal candidates where policy exists.

Do not infer future demand, value, or eligibility from sparse or stale history. Mark the history as incomplete/source-pending when provider mapping is uncertain.

### 4. Preferences, handling, feeding, medication, vaccines, and documents

Separate operational preferences from sensitive care instructions:

- Preferences: customer communication preferences, pet comfort/handling preferences, favorite services/add-ons, arrival/checkout preferences, and staff-approved VIP touches.
- Care notes: allergies, feeding, medication, special care, anxiety, mobility, isolation, or medical condition references should be exact only for authorized care/manager contexts. Otherwise summarize as flags with care-task/document refs.
- Vaccines/documents: show verified status, missing/expired/pending-review refs, and due dates where trusted. Do not include raw OCR, document images, or unverified medical conclusions.

If feeding/medication/care instructions are vague, conflicting, or stale, the summary must say “review required” and point to the authoritative care-task/document refs; it must not convert ambiguity into executable instructions.

### 5. Issues, incidents, complaints, manager notes, and unresolved concerns

This section is mandatory even when empty. It must distinguish:

- Open/unresolved issues requiring action.
- Resolved but operationally relevant history.
- Staff-only or manager-only notes with role visibility.
- Customer sentiment/review/complaint refs and follow-up commitments.
- Suppression states that block marketing, review requests, rebooking prompts, or customer-facing drafts.

Rules:

- Use incident/complaint case IDs, status, risk class, required gate, and safe summary by default.
- Restrict detailed evidence/timelines to authorized manager/incident reviewers.
- Never blame staff/customer/pet, admit liability, diagnose, speculate, downgrade severity, hide uncertainty, or mark issues resolved without source approval refs.

### 6. Staff notes and operational flags

Staff notes may be summarized only when they are relevant to the stated purpose and visible to the requester role. Preserve labels such as `internal_only`, `manager_only`, `care_team_only`, or `customer_safe_summary_available`.

Operational flags can include:

- Arrival/check-in preparation.
- Required document review.
- Medication/feeding/care task confirmation.
- Behavior/group-play review.
- Payment/deposit/reconciliation review.
- Manager follow-up due.
- DNC/contact suppression.
- Known preference/VIP hospitality note.

Do not expose HR/payroll/disciplinary, privileged/legal, unrelated staff commentary, internal investigation details, or raw free-text that is not needed for the purpose.

### 7. Opportunities for rebooking, VIP, retention, and reminders

Opportunities are internal candidates, not actions. Each candidate must include source refs, policy/freshness status, suppression checks, and required review gate.

Allowed examples:

- “Grooming follow-up candidate: last completed groom 7 weeks ago, ordinary cadence 6-8 weeks in history; requires consent/DNC/over-contact checks and approved template before outreach.”
- “VIP hospitality candidate: frequent holiday boarding and manager tag; staff may prepare preferred suite note if capacity/provider record confirms.”
- “Daycare package review: recurring attendance with package balance ref nearing renewal; staff review needed before discussing package.”

Suppress or route to review when there is an unresolved complaint, incident, negative sentiment, DNC/opt-out, payment/refund dispute, stale/missing consent, over-contact risk, or unclear policy.

## Freshness and source citations

Every non-trivial claim must be traceable to source refs. A claim without a source ref must be marked as missing, uncertain, or omitted.

Citation requirements:

- Include source record ID/ref, source type, source timestamp/effective date, fetched-at timestamp, trust marker, and freshness state.
- Mark facts as `current`, `stale`, `missing`, `conflicting`, `untrusted`, `imported_only`, `customer_provided`, `staff_note`, `provider_verified`, `OCR_only`, or `superseded` where applicable.
- Prior AI summaries may be cited only as historical artifacts and must not bootstrap new facts without underlying records.
- For care/medical/vaccine/payment/incident/complaint/staff-only claims, prefer refs and safe summaries over raw text.

Freshness defaults until policy is finalized:

- Current active/upcoming reservation and open task data should be fetched at invocation time.
- Vaccine/document, medication, feeding, incident, complaint, behavior eligibility, and payment flags should be considered stale if the authoritative source cannot be checked during invocation.
- Service history cadence should include the anchor date and source; if the anchor is imported-only or conflicting, do not produce a cadence candidate.

## Redaction and sensitivity boundaries

The output is `staff_internal_only`, but internal does not mean unrestricted. Redact by requester role, purpose, and service context.

Default redaction:

- Names: show customer/pet names when needed for service prep; otherwise use IDs or initials in logs/audit.
- Email/phone/address: mask unless needed for a specific staff contact task or approved send workflow.
- Emergency/vet contacts: omit/mask except for emergency, medical, document, or incident workflows.
- Medical/medication/allergy/feeding: exact details only for authorized care/document/incident/manager contexts; otherwise show flags and refs.
- Vaccine documents/OCR: status and refs only; no raw images/OCR/file URLs outside document review.
- Incidents/complaints: case/status/risk/safe summary by default; detailed timeline/evidence only for authorized reviewers.
- Payment: semantic status and amount due only where needed; no card, CVV, bank, raw payment provider payloads, or secrets.
- Staff notes: preserve internal-only labels and omit unrelated or privileged content.
- AI/runtime logs: store manifests, schema, validation, risk flags, source hashes/refs, and review gates; avoid raw prompt/output retention by default.

## Safe language constraints

Use factual, compact, source-grounded language. The summary may say what records show, what is unresolved, and what review gate is needed. It must not diagnose, make unsupported claims, soften risk, or imply authority.

Required wording posture:

- “Source shows/reports/records...” rather than “is definitely...” when trust/freshness is limited.
- “Requires staff/manager review” when a fact is sensitive, unresolved, conflicting, stale, or action-affecting.
- “Candidate” for rebooking, VIP, package, retention, or reminder opportunities.
- “Suppressed/blocked pending review” for customer-facing outreach when risk flags exist.

Forbidden wording:

- Medical diagnosis, treatment advice, or conversion of vague notes into medication/care instructions.
- Blame, legal conclusions, liability admissions, certainty about fault, or promises of refund/credit/exception.
- Eligibility/behavior clearance, group-play reinstatement, or incident closure without approval refs.
- Customer-facing marketing language, apology drafts, review requests, or rebooking prompts presented as ready to send.
- Unsupported sentiment labels such as “angry,” “difficult,” “negligent,” or “aggressive” unless they are quoted/cited from approved source fields and operationally necessary; prefer behavior/issue facts and refs.

## Human review rules

The output status must be `needs_manager_review` or `needs_staff_review` when any of the following are present:

- Unresolved complaint, negative sentiment, public review response need, recovery commitment, or customer follow-up ambiguity.
- Open incident, active restriction, behavior/eligibility flag, bite/aggression/safety concern, or group-play suspension/reinstatement question.
- Medical, medication, feeding, allergy, vaccine/document, or special-care ambiguity.
- Payment/deposit/refund/waiver/discount/forfeiture dispute or customer-facing payment-sensitive wording.
- DNC/opt-out/legal/privacy suppression, consent conflict, over-contact risk, or uncertain channel permission.
- Duplicate/low-confidence identity, conflicting owner/pet/service records, imported-only provider mapping, or stale source evidence.
- Manager-only note, legal/compliance-sensitive note, privileged investigation, or staff-only content that could affect customer interaction.
- Any proposed discount, credit, refund, VIP exception, package adjustment, policy exception, provider mutation, or customer-facing message.

Human reviewers must receive the source refs, unknowns/conflicts, suggested next steps, and the exact reason the summary is not routine. The agent must not downgrade a review requirement because the customer is VIP/high-value or because an opportunity exists.

## Validation and failure behavior

Before persistence/display, validators should check:

- `visibility` is `staff_internal_only` and customer-facing use is prohibited.
- Required sections exist, including urgent flags and issues/notes even when empty.
- Every claim with operational impact has a source citation and freshness marker.
- Unresolved concerns and risk flags are promoted to `headline` and `urgent_flags`.
- Sensitive sections have appropriate review gates and redaction markers.
- Opportunities include suppression checks and are labeled as candidates.
- No forbidden actions, unsupported claims, raw secrets/payment data, raw documents/OCR, internal-only leakage to customer-facing fields, or uncited sensitive assertions appear.

If validation fails, output `failed_safely` with a minimal error/review packet: scope, missing/invalid categories, denied source categories, and recommended human review owner. Do not display a partial optimistic summary that omits known risk flags.

## Audit requirements

Audit each invocation with:

- Invocation ID, actor/requester role, tenant/location, customer/pet IDs, service context, timestamp, and agent/schema version.
- Prompt packet manifest: field categories, source refs, redaction profile, denied sources, source freshness, and token/count estimates where available.
- Output manifest: status, urgent flags, review gates, recommended staff actions, opportunities, validation result, and citations.
- Runtime/model/provider/tool permission set and policy versions.
- Human review or suppression handoff records.

Ordinary audit logs should store safe summaries, manifests, hashes/refs, validation status, and review gates. Raw prompts, completions, staff notes, customer messages, document/OCR text, payment payloads, and incident details belong only in governed evidence storage if explicitly approved.

## Conservative downstream rule

When customer/pet history is missing, stale, contradictory, sensitive, broad, source-pending, role-denied, or unresolved, the customer history summary agent must make the uncertainty and review gate visible. It may create a staff-facing preparation summary, internal task recommendation, suppression reason, or review packet; it must not invent facts, hide risk, clear issues, send customer-facing copy, or execute provider/customer/account actions.
