# Inquiry intake triage categories and routing

Purpose: define source-backed triage categories and routing logic for the `inquiry-intake` / lead intake workflow. This document is a workflow-design artifact, not approval for autonomous booking, provider writes, medical/vaccine/behavior decisions, payment actions, policy exceptions, or customer-facing sends.

Status: draft routing definition. It builds on `docs/workflows/inquiry-intake-inputs.md`, `docs/architecture/pet-resort-workflow-events.md`, `docs/architecture/workflow-result-envelope.md`, `docs/workflows/customer-messaging-parts/inputs.md`, `docs/workflows/booking-triage-parts/inputs.md`, and the current Rust anchors in `domain/src/entities.rs`, `domain/src/workflow.rs`, `domain/src/agents.rs`, and `domain/src/policy.rs`.

## Non-negotiable routing boundaries

- Primary inbound event: `inquiry.received` / `WorkflowEventType::InquiryReceived`.
- Provider/webhook inputs such as Gingr `lead_created` are boundary evidence until signature verification, source normalization, identity reconciliation, and redaction occur.
- Raw free text, transcripts, screenshots, attachments, provider payloads, and customer claims are evidence, not authority. Preserve source/evidence refs and uncertainty.
- The inquiry-intake workflow may classify, summarize, identify missing facts, flag risk, draft internal tasks, and create customer-message drafts.
- It must not confirm bookings, promise availability, reject service, approve vaccines, approve medical/care/behavior eligibility, override group-play restrictions, collect/waive/alter payment/deposit policy, mutate providers, or send customer-facing messages without a separate approved execution path.
- Customer replies are draft-only by default. A narrow deterministic receipt-only auto-send may be added later only if product policy approves exact template, recipient/channel/consent facts, triggering condition, idempotency key, audit event, and suppression rules.
- Aggression, bite, fight, attack, threat, severe anxiety, escape risk, human/dog selectivity, guarding, unresolved incident, suspension, or similar behavior/safety flags always route to behavior/manager review before any customer-facing eligibility language or group-play/service acceptance.

## Classification precedence

Apply categories in this order. A higher-precedence category can still emit secondary flags/tasks for lower-precedence issues, but the primary route should be the first matching category.

1. `spam`
2. `unsupported`
3. `behavior_review`
4. `special_care_review`
5. `vaccine_docs_needed`
6. `missing_information`
7. `ready_for_booking_review`

Rationale: obvious non-leads should be suppressed before operational work; unsupported requests should not create booking-review pressure; safety/behavior outranks routine care, vaccine, and missing-info follow-up; medical/special-care outranks routine document collection; and `ready_for_booking_review` is only safe when no unresolved information/review route remains.

## Lead status vocabulary

Use these lead statuses as internal recommendations until a data-model card introduces a dedicated lead/inquiry aggregate. If inquiry intake is implemented directly on `ReservationStatus`, map cautiously and preserve `WorkflowResult` evidence/review gates.

| Inquiry triage category | Generated lead status | Reservation-status suggestion, if a reservation/request already exists |
| --- | --- | --- |
| `ready_for_booking_review` | `ReadyForBookingReview` | `Requested` or keep current status with booking-triage task; never `Offered`/`Confirmed` |
| `missing_information` | `NeedsMoreInformation` | `MissingInfo` |
| `vaccine_docs_needed` | `NeedsVaccineDocuments` | `VaccinePending` |
| `special_care_review` | `NeedsSpecialCareReview` | `SpecialReview` |
| `behavior_review` | `NeedsBehaviorReview` | `SpecialReview` with behavior hard stop/review reason |
| `unsupported` | `UnsupportedInquiry` | no status mutation by default; `Rejected` only after approved policy/human denial path |
| `spam` | `SpamSuppressed` | no reservation/status mutation |

## Category routing matrix

### 1. Ready for booking review

| Field | Definition |
| --- | --- |
| Trigger conditions | A real pet-service inquiry with enough normalized facts to evaluate booking next: customer/contact is identifiable or matchable; location is known or inferable from the intake source; service line is supported; requested date/date range or timeframe is present; pet species/count/name or enough pet summary exists; no unresolved vaccine-document request is apparent from the inquiry itself; no medical/special-care/behavior/aggression/safety flag is present; no obvious spam or unsupported service signal exists. |
| Priority / urgency | Normal by default. Raise to high when the requested start date is within the location's short-notice window, holiday/peak wording is present, customer says today/tomorrow/urgent, or the source is staff-entered while the customer is waiting. |
| Generated lead status | `ReadyForBookingReview`. If bridged to reservation status, suggest `Requested` and create booking-triage work rather than offering/confirming. |
| Downstream outputs | `booking.triage_needed` event draft or booking-triage task keyed by inquiry/source evidence; structured intake summary; source/evidence refs; customer/pet/request match candidates; missing assumptions list if any non-blocking uncertainties remain; optional customer-message draft acknowledging receipt and saying staff will review. |
| Escalation path | Front desk / intake staff for ordinary review; booking triage workflow for capacity, eligibility, deposit, and availability; manager only if booking triage later finds capacity/holiday/payment/policy exceptions. |
| Human approval gates | Staff approval is required before promising availability, offering/waitlisting/denying, confirming, collecting deposit/payment, or mutating a provider reservation. Customer replies are drafts requiring `CustomerMessageApproval` unless a future deterministic receipt-only acknowledgement path is explicitly approved. |

### 2. Missing information

| Field | Definition |
| --- | --- |
| Trigger conditions | The inquiry appears supportable, but required intake facts are absent, stale, contradictory, or not confidently mapped: customer name/contact missing; pet name/species/count missing; requested service missing/ambiguous; date/time/range missing; location missing; source channel/contact consent unknown; pet/customer/provider identity conflict; free text says "call me" or contains incomplete details; attachments/source refs are present but not yet parsed. Do not use this category when the missing fact is specifically vaccine proof or when behavior/special-care risks are present; those categories outrank routine missing info. |
| Priority / urgency | Normal for ordinary missing fields. High when the requested start date is soon, staff/customer is waiting on live channel, repeated follow-ups failed, or conflicting identity could create a privacy/safety issue. |
| Generated lead status | `NeedsMoreInformation`; reservation-status suggestion `MissingInfo` when a request exists. |
| Downstream outputs | Internal follow-up task for front desk/intake staff with a checklist of missing fields; optional customer-message draft asking only for the missing safe facts; source/evidence refs and redaction profile; no provider mutation. Dedupe key should include source inquiry id/contact/timeframe/policy version so repeat messages do not create duplicate follow-up work. |
| Escalation path | Front desk / intake staff owns routine follow-up. Escalate to manager/privacy review if identity conflicts, consent uncertainty, cross-customer data collision, or sensitive free text makes a customer reply risky. |
| Human approval gates | Customer follow-up is a draft requiring `CustomerMessageApproval` by default. Auto-send is not allowed unless a later approved deterministic template covers exactly the missing-safe-fact request, consent/quiet-hours are known, no sensitive flags exist, and the send path records approval/policy. Staff must approve any status mutation or provider/customer-record merge. |

### 3. Vaccine docs needed

| Field | Definition |
| --- | --- |
| Trigger conditions | Inquiry requests a service likely requiring vaccine proof, but vaccine documents/proof are missing, expired/stale per policy snapshot, not mapped to the pet, ambiguous, unsupported, or only mentioned in free text without accepted document evidence. Also route here when a customer attaches/includes something that appears to be vaccine proof and needs the vaccine-document workflow. If medical/care ambiguity or behavior/aggression is present, emit vaccine task as secondary but primary route to `special_care_review` or `behavior_review`. |
| Priority / urgency | Normal when dates are not imminent. High when requested arrival/daycare/grooming date is soon, customer is already in a booking flow, or document review backlog blocks staff response. |
| Generated lead status | `NeedsVaccineDocuments`; reservation-status suggestion `VaccinePending` when a request exists. |
| Downstream outputs | `document.uploaded` / `vaccine.extraction_needed` event draft when an attachment/evidence id exists; vaccine-document review task with pet/customer/request refs; customer-message draft for missing/upload instructions only if policy allows; risk flags such as `missing_required_vaccine_proof`, `unverified_veterinary_source`, `ambiguous_pet_identity`, or `expired_or_stale_vaccine`. |
| Escalation path | Front desk may collect documents and map obvious records. Medical document reviewer / trained staff verifies vaccine facts and source. Manager/admin handles policy exceptions, disputes, rejection implications, or customer-visible sensitive language. |
| Human approval gates | Vaccine acceptance, expiry/freshness decisions, licensed-vet source acceptance, rejected/expired vaccine customer copy, and reservation status transitions are human/policy approved. AI/OCR confidence is never approval. Customer replies are drafts by default; no auto-send unless a future deterministic document-receipt or document-request template is approved and does not include eligibility/medical judgment. |

### 4. Special-care review

| Field | Definition |
| --- | --- |
| Trigger conditions | Inquiry includes care, medical, medication, allergy, feeding, mobility, isolation, senior/special-needs, post-surgery, seizure, contagious illness, heat/intact/minimum-age concern, emergency/vet-contact complexity, or staff-capacity/care-lane ambiguity that could affect safe service fit. Also use for vague statements like "needs meds," "has anxiety," "special diet," "medical condition," or "cannot be around other dogs" when not primarily aggression/behavior. Behavior-specific safety/aggression flags route to `behavior_review` first. |
| Priority / urgency | High for near-term stays, medication/medical claims, illness/contagion, post-surgery, safety-sensitive handling, or staff-entered urgent inquiries. Normal for future routine care notes that simply need clarification. |
| Generated lead status | `NeedsSpecialCareReview`; reservation-status suggestion `SpecialReview`. |
| Downstream outputs | Care/medical review task packet with structured care questions, evidence refs, requested service/date, pet/customer identifiers, and source uncertainty; optional manager/special-care task; optional customer-message draft requesting clarification without medical advice or acceptance promise; risk flags such as `medical_or_medication_review_required`, `special_care_capacity_unknown`, `care_instructions_ambiguous`, or `veterinary_clarification_needed`. |
| Escalation path | Front desk collects missing care details only. Care lead / trained staff reviews routine care feasibility. Manager/admin approves special-care acceptance, exception pricing, policy exceptions, or service denial. Vet/emergency contact is only contacted through approved staff policy, not by the AI workflow. |
| Human approval gates | Human approval is required before accepting or rejecting special-care fit, creating executable medication/care tasks from vague notes, promising accommodations, quoting special-care fees, giving medical advice, or sending customer-visible medical/care language. Customer messages are draft-only by default and must not imply eligibility until approved. |

### 5. Behavior review

| Field | Definition |
| --- | --- |
| Trigger conditions | Inquiry mentions or source records indicate aggression, biting, bite history, attack/fight, growling/snapping/lunging, dog/human selectivity, resource/food guarding, escape risk, severe anxiety/stress, suspension, incident history, muzzle/restraint needs, group-play uncertainty, "not good with other dogs/people/kids," or staff/manager behavior-review flags. Use this category even if enough booking info exists. It outranks `special_care_review`, `vaccine_docs_needed`, `missing_information`, and `ready_for_booking_review`. |
| Priority / urgency | High by default because it may affect pet/staff/customer safety and service fit. Critical/urgent if the message implies immediate danger, recent bite/injury, legal threat, emergency, or same-day arrival. |
| Generated lead status | `NeedsBehaviorReview`; reservation-status suggestion `SpecialReview` plus behavior review reason/hard stop such as `BehaviorReviewRequired` or `IneligibleForGroupPlay` if already present in trusted domain state. |
| Downstream outputs | Behavior review task for trained staff/manager with minimal source excerpt, evidence ids, pet/request context, and requested service/group-play implications; risk flags such as `aggression_or_bite_history`, `group_play_eligibility_unknown`, `escape_or_safety_risk`, `unresolved_incident`, or `behavior_source_unverified`; optional customer-message draft that only acknowledges review/requests safe clarification and avoids acceptance/rejection/eligibility language. |
| Escalation path | Trained behavior reviewer / lead staff for routine temperament evaluation; manager/admin for bite/aggression, incident, service restriction, group-play exception, denial/reinstatement, customer dispute, or sensitive wording. Emergency/safety policy path if immediate danger is indicated. |
| Human approval gates | Behavior/group-play eligibility, service acceptance/denial, restrictions, reinstatement/suspension, incident-related language, and any customer-facing aggression/safety wording require explicit human approval. AI must not soften, hide, reinterpret, or auto-clear aggression/behavior flags. Customer replies are never auto-sent when behavior/aggression flags are present unless a future policy explicitly approves a very narrow internal-only/staff-mediated path; default is draft + manager/staff approval. |

### 6. Unsupported

| Field | Definition |
| --- | --- |
| Trigger conditions | The inquiry is real but outside current product/service/location scope: non-pet-resort service, species/service not supported by the location/policy, location not served, dates far outside allowed booking window if policy says unsupported, request for veterinary diagnosis/treatment, emergency animal care, legal/insurance/payment dispute outside intake, vendor/sales/job inquiry, franchise/corporate contact, or channel/action not supported by current workflow. Use `spam` instead when the message is clearly junk, malicious, or not a real business inquiry. |
| Priority / urgency | Low for ordinary out-of-scope requests. High if the unsupported request includes animal/human safety emergency, legal threat, abuse/neglect allegation, payment dispute, or complaint requiring manager/compliance review. |
| Generated lead status | `UnsupportedInquiry`. Do not create or mutate reservation status by default. If a human later denies/rejects a request, that is a separate approved policy path. |
| Downstream outputs | Suppression or unsupported-route packet; optional staff/manager review task when sensitive; optional customer-message draft with neutral redirect/limitations only if approved copy exists; source/evidence refs. Do not create booking-triage tasks unless a supported service/request can be split out. |
| Escalation path | Front desk can close obvious unsupported but non-sensitive requests under approved policy. Manager/admin reviews service-denial wording, complaints, legal/payment/emergency issues, and any uncertain unsupported classification. |
| Human approval gates | Customer-facing denial/unsupported replies require approval unless there is a later approved deterministic neutral redirect template for that exact unsupported class. AI must not diagnose, provide emergency advice beyond approved safe redirect language, reject a customer reservation in provider state, or promise exceptions. |

### 7. Spam

| Field | Definition |
| --- | --- |
| Trigger conditions | Message is clearly junk or malicious: marketing spam, SEO/vendor pitch, phishing, credential/payment-token request, malware/link farm, bot gibberish, duplicate flood with no plausible pet-service intent, abusive content with no service request, or source fails authenticity/verification checks. If there is any plausible customer/pet-service intent plus concerning content, prefer the relevant supported/unsupported/behavior route with risk flags rather than spam. |
| Priority / urgency | Low for ordinary spam suppression. High/security if phishing, credential/payment attempt, malware, impersonation, harassment/threat, or flood/abuse pattern is present. |
| Generated lead status | `SpamSuppressed`; no reservation status. |
| Downstream outputs | Spam/suppression audit event or review packet with source id/hash, reason code, and minimal redacted excerpt; optional security/admin task for phishing, credential exposure, repeated abuse, or provider verification failure; no customer-message draft by default. |
| Escalation path | Suppress obvious spam by deterministic policy when approved. Route suspicious/security-sensitive cases to admin/security or manager. Route abusive-but-real customer inquiries to manager/behavior/customer-message review rather than silently suppressing. |
| Human approval gates | No auto-reply by default. Human/admin approval is required before blocking a known customer/contact, reporting, preserving legal/security evidence beyond normal retention, or sending any response. AI must not expose suspicious links/content to staff beyond safe redacted excerpts. |

## Multi-label secondary flags

The primary category should remain one of the seven required categories, but outputs may include secondary flags to preserve all concerns:

- `missing_contact`, `missing_pet_identity`, `missing_service`, `missing_date_range`, `identity_conflict`, `contact_consent_unknown`.
- `vaccine_document_missing`, `vaccine_document_unverified`, `document_uploaded_needs_extraction`.
- `medical_or_medication_review_required`, `allergy_or_feeding_review_required`, `special_handling_or_isolation`, `vet_clarification_needed`.
- `aggression_or_bite_history`, `group_play_review_required`, `escape_risk`, `unresolved_incident`, `behavior_source_unverified`.
- `unsupported_service_or_species`, `unsupported_location`, `emergency_or_veterinary_redirect_needed`, `vendor_or_non_customer`.
- `spam_or_phishing`, `credential_or_payment_secret_risk`, `abusive_or_threatening_content`.

Secondary flags should drive task/checklist content and risk summaries; they should not weaken the human gates associated with the primary or highest-risk secondary concern.

## Routing output contract

A triage result should be a `WorkflowResult`-style packet with at least:

- `category`: one of `ready_for_booking_review`, `missing_information`, `vaccine_docs_needed`, `special_care_review`, `behavior_review`, `unsupported`, `spam`.
- `lead_status`: generated internal lead status from the vocabulary above.
- `priority`: `low`, `normal`, `high`, or `urgent`, with evidence-backed reason.
- `summary`: short internal staff summary with no unnecessary raw PII.
- `structured_output`: normalized requested service/date/location/customer/pet/source fields when available, missing-field checklist, secondary flags, identity confidence, and source/evidence refs.
- `recommended_actions`: internal task drafts, derived event drafts, message drafts, risk flags, and review requests. Drafts must be distinct from execution/sends.
- `human_review_reason`: required for `vaccine_docs_needed`, `special_care_review`, `behavior_review`, sensitive `unsupported`, suspicious/security `spam`, identity conflicts, and any customer-message draft containing sensitive or policy-dependent language.
- `verification`: source ids, channel, provider verification/mapping state, redactions, unchecked sources, policy snapshot ids, and idempotency keys.

Suggested idempotency keys:

- Triage result: `inquiry_triage:v1:{location_id}:{source_kind}:{source_event_or_message_id_or_hash}:{policy_version}`.
- Internal follow-up task: `internal_task:v1:{lead_or_external_subject}:inquiry_follow_up:{category}:{policy_version}`.
- Message draft: `message_draft:v1:{lead_or_external_subject}:{draft_intent}:{template_or_copy_version}:{source_evidence_hash}:{policy_version}`.
- Customer send, if ever approved later: separate `approved_send:v1:{approved_draft_id}:{recipient_ref}:{channel}:{approval_id}`.

## Customer-reply automation policy by category

| Category | Can the inquiry workflow auto-send a customer reply? | Default safe draft posture |
| --- | --- | --- |
| `ready_for_booking_review` | No by default. Future deterministic receipt-only acknowledgement may be approved separately. | Draft acknowledgement only; no booking/availability promise. |
| `missing_information` | No by default. Future deterministic missing-safe-field template may be approved only when no sensitive flags/consent gaps exist. | Draft checklist asking for missing safe facts. |
| `vaccine_docs_needed` | No by default. Future deterministic document-receipt/request template may be approved only if it avoids eligibility judgment. | Draft upload/request copy; no vaccine acceptance/rejection language. |
| `special_care_review` | No. | Draft staff-reviewed clarification only; no medical advice or acceptance promise. |
| `behavior_review` | No. | Draft only after behavior/manager review; avoid eligibility/restriction conclusions until approved. |
| `unsupported` | No by default. Future deterministic neutral redirect template may be approved for narrow non-sensitive classes. | Draft neutral redirect/limitation if appropriate; manager review for denial/sensitive cases. |
| `spam` | No. | No reply; security/admin review for risky cases. |

## Aggression and behavior flag handling

- Preserve the exact source signal and citation/evidence ref. Do not paraphrase away severity; do not amplify beyond the evidence.
- Use `behavior_review` as primary category for any aggression/bite/fight/attack/guarding/escape/suspension/unresolved-incident signal even if the inquiry otherwise looks ready.
- Suppress autonomous customer replies. Any wording that mentions aggression, bites, incidents, restrictions, group-play eligibility, safety, service denial, or acceptance requires `BehaviorReview` plus `CustomerMessageApproval`; manager approval is required for severe/sensitive cases.
- Never mark group play, daycare, boarding, or training eligibility as approved from customer free text or AI inference. At most suggest review tasks and preserve source uncertainty.
- If behavior flags coexist with vaccine or missing-info gaps, create secondary tasks/checklist items but do not downgrade the primary route away from behavior review.
- If the message includes immediate danger, injury, threat, abuse/neglect allegation, or legal language, raise priority to urgent and route to manager/safety/compliance according to local policy.

## Conservative implementation rule

When evidence is missing, stale, conflicting, unverified, unsupported by the current data model, or sensitive, route to an explicit review category/task rather than treating the inquiry as ready. A `Completed` workflow result means the intake workflow produced a safe classification, summary, task draft, or message draft; it does not mean the booking, vaccine, medical/care, behavior, unsupported/denial, provider, or customer-message action is approved.
