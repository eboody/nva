# Vaccine document agent workflow

Status: integrated draft. This document is not approval for production LLM use, customer-facing sends, provider writes, booking/profile mutations, or autonomous medical-document acceptance. It defines the conservative upload, extraction, review, policy-comparison, audit, and fixture contracts that implementation cards should encode.

## Source artifacts synthesized

- `docs/workflows/vaccine-document-agent-parts/inputs.md` — workflow boundaries, source anchors, event/result inputs, open questions, and conservative downstream rule.
- `docs/workflows/vaccine-document-agent-parts/vaccine-policy.md` — draft policy shape, proposed species/service requirements, validity handling, proof/source states, eligibility mapping, and policy approval questions.
- `docs/workflows/vaccine-document-agent-parts/upload-workflow.md` — private storage metadata, event payloads, idempotency, status transitions, UI expectations, result envelope expectations, risk flags, and audit trail.
- `docs/workflows/vaccine-document-agent-parts/extraction-schema.md` — structured output schema for document, identity, source clinic, vaccine suggestions, policy comparison suggestions, ambiguity, contradictions, missing inputs, and review packets.
- `docs/workflows/vaccine-document-agent-parts/auto-accept-review-rules.md` — review-first posture, proposed future auto-accept checklist, decision states, human-review routing rules, and approval checklist.
- `docs/workflows/vaccine-document-agent-parts/test-corpus.md` — synthetic/de-identified fixture matrix and pass/fail assertions for clean, low-quality, multi-pet, expired, handwritten, invoice, unrelated, and edge documents.

## 1. Scope and operating posture

### In scope

The vaccine document agent may:

1. Accept an uploaded/document-referenced evidence object through an approved intake path.
2. Store raw evidence privately and emit normalized workflow events that reference evidence by id.
3. Run OCR/parser/LLM extraction to produce structured vaccine-document suggestions.
4. Compare suggestions against an approved location-scoped vaccine policy snapshot when one exists.
5. Produce internal review packets, risk flags, recommended staff actions, and draft-only follow-up needs.
6. Preserve audit evidence for intake, extraction, comparison, review, and any later approved downstream execution.
7. Support synthetic/de-identified test fixtures that exercise safe extraction and review gating.

### Out of scope / forbidden without a separate approved execution path

The agent must not:

- approve vaccine proof, medical eligibility, check-in readiness, or group-play eligibility by AI/OCR alone;
- confirm, reject, cancel, modify, or otherwise mutate reservations;
- write accepted vaccine records back to Gingr/provider systems or any other live provider;
- send customer-facing messages, reupload requests, approval/rejection notices, or policy explanations;
- expose raw uploads, raw OCR, raw email, raw provider payloads, storage keys, contact details, or unnecessary PII in events, logs, prompt packets, or result envelopes;
- treat upload success, workflow success, high confidence, provider payload presence, or a clean-looking PDF as vaccine acceptance.

### Core rule

All extracted medical-document facts are suggestions/unverified data unless an approved `MedicalDocumentReview` decision or an explicitly approved deterministic auto-accept rule accepts them with a complete audit record.

`WorkflowStatus::Completed`, `status: "success"`, or high model/OCR confidence means the workflow produced a valid internal artifact. It never means the vaccine proof is accepted.

## 2. Domain anchors and actors

Current repository anchors include:

- `LocationPolicyRefs.vaccine_policy_id` for location-scoped vaccine policy references.
- `Pet { species, care_profile, temperament }` and `Species::{Dog, Cat, Other}`.
- `Reservation { service, status, hard_stops }`, with services `Boarding`, `DayPlay`, `DayBoarding`, `Grooming`, `Training`, and `DaySpa`.
- `ReservationStatus::{VaccinePending, MissingInfo, SpecialReview}` as possible suggestions, never autonomous mutations.
- `HardStop::MissingRequiredVaccine(policy::VaccineName)` and `HardStop::MedicalOrMedicationReviewRequired`.
- `StaffTaskKind::DocumentReview { pet_id }`, staff task states, and `ReviewGate::MedicalDocumentReview`.
- `WorkflowEventType::VaccineDocumentUploaded`, `AllowedAction::{ReadEntities, ExtractStructuredData, CreateInternalTask, FlagRisk}`, `WorkflowStatus`, and `RecommendedAction::RequestHumanReview`.
- `VaccineRequirement`, `VaccineName`, `PolicyDenialReason::MedicalDocumentReviewRequired`, and policy automation levels.
- `AuditEvent`, `AuditSubject`, `AuditAction`, and `ActorRef`.

Human authority remains with approved staff/manager/admin roles. Exact role names and permissions are an open implementation decision.

## 3. Vaccine policy

### Policy snapshot requirement

Every policy comparison must use a location-scoped policy snapshot with:

- `vaccine_policy_id`;
- `policy_version` or immutable snapshot timestamp;
- `location_id`;
- species and service row;
- canonical required vaccine names plus accepted aliases;
- source rule, including whether proof must come from a licensed veterinarian/veterinary professional;
- validity rule, expiration/freshness window, fallback validity rules if any, grace periods if any, and series rules if any;
- eligibility effect when proof is missing, expired, unverified, or unknown;
- review gate and automation level.

If any policy field needed for the comparison is absent, stale, draft-only, or unknown, the workflow may extract and summarize facts but must not evaluate eligibility. Route to `MedicalDocumentReview` or `SpecialReview`.

### Canonical vaccine names and alias handling

Use canonical names for policy comparison and keep aliases as extraction/mapping hints:

| Canonical name | Common aliases / labels | Species | Draft policy status |
| --- | --- | --- | --- |
| Rabies | Rabies certificate, rabies 1-year, rabies 3-year | Dog, Cat | Proposed baseline required |
| DHPP | DHLPP, DA2PP, DAPP, distemper/parvo combo, distemper/hepatitis/parainfluenza/parvovirus; leptospirosis may appear as `L` | Dog | Proposed baseline required; equivalence and lepto handling require approval |
| Bordetella | Kennel cough, bordetella bronchiseptica | Dog | Proposed baseline required |
| Canine Influenza | CIV, canine flu, H3N2, H3N8, bivalent influenza | Dog | Public PetSuites examples include it; local/series handling requires approval |
| FVRCP | Feline distemper combo, feline viral rhinotracheitis/calicivirus/panleukopenia | Cat | Proposed baseline required where cat services exist |
| FeLV | Feline leukemia | Cat | Unknown/location-specific |
| Other vaccine | Any unmapped label | Any | Unknown; route to review if relevant |

Alias equivalence is a policy decision, not an OCR decision. The extractor may suggest mappings with evidence spans, but only reviewed or policy-approved mappings can satisfy requirements.

### Proposed baseline by species and service

This baseline is a proposal to approve or edit, not a live policy.

Dogs:

| Service | Proposed required vaccines | Approval-sensitive points | Suggested eligibility effect if missing/expired/unverified |
| --- | --- | --- | --- |
| Boarding | Rabies, DHPP/DHLPP family, Bordetella, Canine Influenza | CIV two-dose/first-dose handling, lepto requirements, puppy exceptions | Suggest `VaccinePending` with `MissingRequiredVaccine` hard stops until reviewed |
| DayPlay/daycare | Rabies, DHPP/DHLPP family, Bordetella, Canine Influenza | Same as boarding plus same-day freshness rules | Suggest `VaccinePending`; block group-play/daycare readiness until reviewed |
| DayBoarding/individual play | Same proposed dog baseline unless local policy narrows it | Whether individual play can use a narrower set | Suggest `VaccinePending` or `SpecialReview` depending on policy |
| Grooming | Same dog baseline only if location adopts an all-services rule | May be narrower, e.g. rabies-only or no resort-core bundle | If policy requires vaccines, suggest `VaccinePending`; if unknown, `SpecialReview` |
| Training | Same dog baseline only if location adopts an all-services rule | Class/private-training variants may differ | If policy requires vaccines, suggest `VaccinePending`; if unknown, `SpecialReview` |
| DaySpa/bathing | Same dog baseline only if location adopts an all-services rule | Bath-only/add-on contexts may differ | If policy requires vaccines, suggest `VaccinePending`; if unknown, `SpecialReview` |

Cats:

| Service | Proposed required vaccines | Approval-sensitive points | Suggested eligibility effect if missing/expired/unverified |
| --- | --- | --- | --- |
| Boarding | Rabies, FVRCP | FeLV if local policy requires; kitten exceptions; non-vaccine neuter policies are separate | Suggest `VaccinePending` with missing-vaccine hard stops until reviewed |
| DayPlay/daycare | Unknown unless offered; start from Rabies and FVRCP if offered | Whether cat daycare exists and follows boarding requirements | `SpecialReview` until policy exists |
| DayBoarding/individual enrichment | Rabies and FVRCP if offered | Exact local cat lane policy | `VaccinePending` if policy exists; otherwise `SpecialReview` |
| Grooming | Rabies and FVRCP if all-services rule applies | FeLV or rabies-only exceptions | If policy requires vaccines, `VaccinePending`; if unknown, `SpecialReview` |
| Training | Unknown unless offered | Whether cat training exists | `SpecialReview` until policy exists |
| DaySpa/bathing | Rabies and FVRCP if offered and all-services rule applies | FeLV or rabies-only exceptions | If policy requires vaccines, `VaccinePending`; if unknown, `SpecialReview` |

`Species::Other` has no baseline. Route to `SpecialReview`; do not auto-evaluate vaccine eligibility.

### Validity and expiration handling

- Prefer explicit expiration, due, or next-due dates from the document.
- A vaccine is current only if the reviewed expiration/due date is on or after the relevant service end/date in the location timezone.
- Boarding requires current-through checkout/end date, not merely current on upload or check-in.
- Missing expiration/due dates default to `unknown_validity`, not accepted.
- Do not infer one-year or three-year validity from administered date unless a policy-approved fallback rule exists.
- Expired proof suggests `VaccinePending` and `expired_or_stale_vaccine`.
- Future-dated administered dates, impossible date order, missing years, locale ambiguity, conflicting duplicate dates, or unclear date roles require review.
- Grace periods, puppy/kitten schedules, medical exemptions, titers, jurisdiction rules, and CIV series exceptions are not assumed.

### Source and proof handling

A policy may require proof from a licensed veterinarian/veterinary professional. Evidence carriers include customer uploads, staff scans/uploads, email ingests, portal exports, provider polls/webhooks, and migration imports. Upload success and provider payload presence are evidence only, not verification.

Source states:

| Source state | Meaning | Booking/readiness effect |
| --- | --- | --- |
| `verified_veterinary_source` | Human reviewer accepted clinic/vet source evidence under policy | May satisfy source component if dates, identity, vaccine name, and policy also pass |
| `source_suggested_unverified` | Extractor found a clinic/vet candidate but no human acceptance exists | `VaccinePending` / `MedicalDocumentReview` |
| `source_missing_or_ambiguous` | No clear source or conflicting source evidence | `VaccinePending` with `unverified_veterinary_source` |
| `provider_record_unverified` | Provider record exists but raw provenance is not accepted | Review packet input only |

## 4. Upload and storage workflow

### Intake sources

Supported source classes:

- `customer_upload` from a portal upload, scanned PDF/image, drag-and-drop, manual file selection, or designated vaccine-record email path;
- `staff_scan` or `staff_upload`;
- `email_ingest`;
- `provider_poll` or `provider_webhook` for read-only provider evidence;
- `migration_import`.

Provider and OCR payloads stay quarantined as private evidence and are referenced by ids.

### Storage metadata contract

Every raw object should have a normalized metadata record:

| Field | Requirement |
| --- | --- |
| `document_id` | Stable semantic document id for workflow events and review UI |
| `evidence_id` | Immutable id for the exact raw object version |
| `storage_key` | Private object key; never expose directly to customers or logs |
| `content_hash` | SHA-256 or equivalent hash for dedupe/integrity |
| `source` | `customer_upload`, `staff_scan`, `staff_upload`, `email_ingest`, `provider_poll`, `provider_webhook`, or `migration_import` |
| `uploader_actor` | Customer/staff/system actor ref without embedding contact details |
| `received_at` / `source_timestamp` | Server receipt timestamp plus optional trustworthy source timestamp |
| `original_filename_sanitized` | Optional display name after sanitization; not a policy input |
| `mime_type`, `extension`, `size_bytes`, `page_or_image_count` | Validation and review metadata; mismatches become risk flags |
| `document_kind_hint` / `expected_content_hint` | Intake hints, never authoritative classifications |
| `mapping_state` | `mapped`, `candidate`, `conflicting`, or `unmapped` |
| `customer_id`, `pet_id`, `reservation_id` | Optional typed refs; candidate refs are not approval |
| `provider_ref` | Provider name/external id without raw provider payload inline |
| `privacy_class`, `retention_policy_ref`, `access_policy_ref` | Medical/customer document privacy controls |
| `redaction_status`, `scan_status`, `ocr_status`, `extraction_status` | Processing/safety state |
| `supersedes_document_id` | Optional replaced prior document |

Unsafe, unscanned, or scan-failed objects block extraction and route to review/security handling.

### End-to-end lifecycle

1. Receive upload or document reference.
2. Store raw object privately, compute hash, scan, classify privacy, and record immutable evidence metadata.
3. Create normalized document metadata and candidate subject mapping.
4. Emit `document.uploaded` after storage and metadata succeed.
5. Derive `vaccine.extraction_needed` only when the document is vaccine-relevant or expected to be vaccine proof and safe for extraction.
6. Run OCR/parser/LLM extraction against private evidence refs and approved prompt packets.
7. Persist structured extraction suggestions and policy-comparison suggestions.
8. Route to review, no-action, or future auto-accept candidate state according to deterministic rules.
9. Draft or create internal review tasks according to task-creation policy and dedupe keys.
10. Record staff review decisions or future policy validator decisions with audit links.
11. Any downstream reservation/profile/provider/customer-message change is a separate approved/audited execution path.

### Status transitions

Document processing states:

- `received` -> `stored`, `rejected_unsafe`, or `failed`.
- `stored` -> `event_emitted`, `duplicate`, or `failed`.
- `event_emitted` -> `extraction_queued`, `review_needed`, `no_action`, or `failed`.
- `extraction_queued` -> `extracting`, `extraction_failed`, or `superseded`.
- `extracting` -> `extracted_unverified`, `extraction_failed`, or `superseded`.
- `extracted_unverified` -> `review_needed`, `auto_accept_candidate`, or `superseded`.
- `review_needed` -> `accepted_by_staff`, `rejected_or_not_relevant`, `needs_more_information`, `escalated`, or `superseded`.
- `auto_accept_candidate` -> `accepted_by_policy`, `review_needed`, or `rejected_or_not_relevant`.
- `accepted_by_staff` / `accepted_by_policy` -> `superseded` or `closed`.
- `rejected_or_not_relevant`, `needs_more_information`, `superseded`, and `failed` close or return to review only through explicit transitions.

Extraction suggestion review states:

- `suggested_unverified`;
- `needs_identity_review`;
- `needs_policy_review`;
- `needs_date_review`;
- `needs_source_review`;
- `accepted_by_staff`;
- `rejected`;
- `superseded`.

Reservation/profile readiness outputs are suggestions only:

- `VaccinePending` for present but unverified, missing, expired/stale, or review-pending proof;
- `MissingInfo` for no usable document, wrong pet, unreadable upload, or missing required field;
- `SpecialReview` for medical/source/policy conflict, manager exception, customer dispute, or privacy/security issue;
- `ready_for_staff_approval` only after policy-backed facts appear satisfied, still without a provider/customer-visible mutation.

## 5. Event flow

All events use the shared workflow envelope. Payloads must include typed ids and safe metadata only; raw document content, raw OCR, raw provider JSON, emails, contact details, and unnecessary PII remain in private evidence storage.

### `document.uploaded`

Purpose: record safe document intake after raw storage and metadata normalization.

Minimum payload:

```json
{
  "event_type": "document.uploaded",
  "event_id": "evt_...",
  "occurred_at": "2026-06-11T14:30:00Z",
  "actor": { "type": "customer|staff|system", "actor_id": "..." },
  "location_id": "loc_...",
  "document": {
    "document_id": "doc_...",
    "evidence_id": "ev_...",
    "source": "customer_upload",
    "content_hash": "sha256:...",
    "privacy_class": "medical_customer_document",
    "document_kind_hint": "vaccine_record",
    "expected_content_hint": "vaccine_proof",
    "scan_status": "passed",
    "redaction_status": "pending",
    "storage_ref": "private-evidence-ref-only"
  },
  "subject_mapping": {
    "mapping_state": "candidate",
    "customer_id": "cust_...",
    "pet_id": "pet_...",
    "reservation_id": "res_...",
    "candidate_subjects": []
  },
  "policy_context": {
    "vaccine_policy_id": "policy_...",
    "policy_snapshot_id": "policy_snapshot_...",
    "policy_status": "approved|draft|missing"
  },
  "idempotency_key": "source+source_document_ref+content_hash"
}
```

Routing:

- If scan/storage is unsafe or unknown, block extraction and route to review/security handling.
- If the document is clearly unrelated and safely classifiable, record no-action with audit.
- If vaccine-relevant or expected to be vaccine proof, derive `vaccine.extraction_needed`.

### `vaccine.extraction_needed`

Purpose: queue OCR/parser/LLM extraction and policy comparison support.

Minimum payload:

```json
{
  "event_type": "vaccine.extraction_needed",
  "event_id": "evt_...",
  "derived_from_event_id": "evt_document_uploaded",
  "document_id": "doc_...",
  "evidence_id": "ev_...",
  "location_id": "loc_...",
  "candidate_subject": {
    "customer_id": "cust_...",
    "pet_id": "pet_...",
    "reservation_id": "res_...",
    "species": "dog",
    "service": "boarding"
  },
  "policy_snapshot_id": "policy_snapshot_...",
  "extraction_reason": "expected_vaccine_proof",
  "allowed_actions": ["ReadEntities", "ExtractStructuredData", "CreateInternalTask", "FlagRisk"],
  "forbidden_actions": ["SendCustomerMessage", "WriteProviderRecord", "MutateReservation", "AcceptMedicalDocument"],
  "idempotency_key": "document_id+pet_id+policy_snapshot_id+extraction_reason"
}
```

Routing:

- Missing policy, mapping ambiguity, unsafe scan state, or unsupported document type may still produce a review packet, but cannot produce acceptance.
- Re-derive when policy snapshot, subject mapping, or evidence version changes.

### Idempotency, retries, and dedupe

| Step | Idempotency key | Retry behavior |
| --- | --- | --- |
| Raw object create | `source + source_document_ref + content_hash` | Return existing `document_id` for same content/source and append audit observation if needed |
| Metadata create | `document_id` | Upsert only valid processing-state transitions |
| `document.uploaded` emit | `document_id + evidence_id + source_event_ref` | Do not duplicate intake for same immutable evidence |
| `vaccine.extraction_needed` derive | `document_id + pet_id/candidate_subject + policy_snapshot_id + extraction_reason` | Re-derive only when policy/mapping/reason changes |
| OCR/extraction job | `evidence_id + extraction_engine_version + parser_version + redaction_profile` | Retry transient failures; new engine/version creates new derived artifact |
| Review task draft/create | `document_id + pet_id + review_gate + active_policy_snapshot_id` | Avoid duplicate open review tasks; update existing packet with new evidence |
| Suggestion record | `evidence_id + normalized_vaccine_candidate + evidence_span + parser_version` | Preserve conflicts; do not silently collapse contradictory suggestions |

## 6. Extraction schema

The workflow result uses the shared `WorkflowResult` envelope and places the document-specific packet under `structured_output`.

Recommended schema identity:

- `schema_name`: `VaccineDocumentExtractionResult`.
- `schema_version`: `2026-06-11` until versioned by implementation.
- `workflow_name`: `vaccine_document_agent`.
- `status`: `success`, `needs_human_review`, `blocked`, or `failed`; status never means acceptance.

Core structured output shape:

```json
{
  "document": {
    "document_id": "doc_123",
    "source": "customer_upload",
    "kind": "vaccine_record",
    "received_at": "2026-06-11T14:30:00Z",
    "document_date": { "value": "2026-06-01", "state": "suggested", "confidence": { "score": 0.86, "band": "high" }, "source_refs": [] },
    "language_hints": ["en"],
    "page_count": 2,
    "raw_attachments": [
      { "evidence_id": "raw_123", "kind": "raw_upload", "storage_ref": "private://...", "content_hash": "sha256:...", "redaction_state": "raw_private_not_inlined" },
      { "evidence_id": "ocr_123", "kind": "ocr_output", "storage_ref": "private://...", "engine": "...", "engine_version": "...", "redaction_state": "raw_private_not_inlined" }
    ]
  },
  "identity": {
    "pet_candidates": [],
    "owner_candidates": [],
    "identity_state": "needs_review",
    "identity_warnings": []
  },
  "source_clinic": {
    "clinic_name": { "value": "Example Veterinary Clinic", "state": "suggested", "confidence": { "score": 0.88, "band": "high" }, "source_refs": [] },
    "veterinarian_name": { "value": null, "state": "unknown", "confidence": { "score": 0.0, "band": "none" }, "source_refs": [] },
    "license_or_accreditation": { "value": null, "state": "unknown", "confidence": { "score": 0.0, "band": "none" }, "source_refs": [] },
    "source_trust_state": "unverified_veterinary_source",
    "source_warnings": ["licensed_vet_source_not_verified"]
  },
  "vaccine_suggestions": [
    {
      "suggestion_id": "vx_sugg_001",
      "canonical_vaccine_name": "rabies",
      "extracted_label": "Rabies 3yr",
      "aliases": ["rabies", "rabies 3 year"],
      "review_state": "needs_medical_document_review",
      "administered_date": { "value": "2025-05-10", "state": "suggested", "precision": "day", "confidence": { "score": 0.94, "band": "high" }, "source_refs": [] },
      "expiration_or_due_date": { "value": "2028-05-10", "date_role": "expiration", "state": "suggested", "precision": "day", "confidence": { "score": 0.93, "band": "high" }, "source_refs": [] },
      "lot_number": { "value": "LOT123", "state": "suggested", "confidence": { "score": 0.6, "band": "medium" }, "source_refs": [] },
      "manufacturer": { "value": null, "state": "unknown", "confidence": { "score": 0.0, "band": "none" }, "source_refs": [] },
      "clinic_or_vet_ref": "source_clinic",
      "source_refs": [],
      "field_warnings": [],
      "contradiction_flags": []
    }
  ],
  "policy_comparison_suggestions": [
    {
      "policy_snapshot_id": "policy_snapshot_123",
      "requirement_ref": "vaccine_policy/rabies/dog/boarding",
      "required_vaccine_name": "rabies",
      "matched_suggestion_ids": ["vx_sugg_001"],
      "comparison_state": "matched_unverified",
      "reason": "Extracted rabies row has dates but requires medical document review before acceptance.",
      "source_refs": []
    }
  ],
  "ambiguous_fields": [],
  "mismatch_warnings": [],
  "contradiction_flags": [],
  "missing_inputs": [],
  "review_packet": {
    "recommended_review_gate": "MedicalDocumentReview",
    "review_reason_codes": ["medical_document_fact_unaccepted"],
    "suggested_operator_next_steps": ["verify pet identity", "verify clinic/source", "confirm vaccine date interpretation"],
    "safe_summary": "Extraction suggestions are available for staff review; no vaccine fact is accepted."
  }
}
```

Field contract:

- Every acceptance-relevant extracted value needs `state`, `confidence`, and `source_refs` where available.
- `unknown`, `not_applicable`, and unresolved `ambiguous` values may use `value: null` but must explain why when the field matters.
- Conflicts must appear in `ambiguous_fields`, `mismatch_warnings`, or `contradiction_flags`.
- Source refs are required for values affecting review, policy comparison, identity matching, or task/message drafts.
- Raw OCR/provider/email data and unnecessary PII must never be inlined.

Enumerations:

- `document.source`: `customer_upload`, `staff_scan`, `email_ingest`, `provider_poll`, `provider_webhook`, `migration_import`, `unknown`.
- `document.kind`: `vaccine_record`, `immunization_record`, `rabies_certificate`, `vet_invoice_with_vaccines`, `medical_note`, `unknown`, `other`, `unsupported`.
- `identity.match_state`: `matched_by_verified_id`, `candidate_match`, `multiple_candidates`, `mismatch`, `unmapped`, `unknown`.
- `source_trust_state`: `licensed_vet_verified`, `clinic_candidate`, `unverified_veterinary_source`, `customer_supplied_only`, `provider_payload_unverified`, `unknown`.
- `vaccine_suggestions.review_state`: `unreviewed`, `needs_more_information`, `needs_medical_document_review`, `accepted_by_staff`, `rejected_or_not_relevant`, `superseded`.
- `policy_comparison_suggestions.comparison_state`: `matched_unverified`, `missing`, `expired_or_stale`, `future_dated`, `conflicting`, `source_unverified`, `not_required_for_service`, `policy_missing`, `not_evaluated`.

Validation must reject or route to review if the result implies acceptance, readiness, provider write-back, customer send, missing evidence ids, missing source refs for critical facts, missing human-review reasons, or raw/private data inline.

## 7. Auto-accept vs human review

### MVP posture

The MVP is review-first. Extracted facts remain suggestions and route through `MedicalDocumentReview` unless and until the policy owner approves deterministic auto-accept configuration and monitoring.

Clean records may be marked `auto_accept_eligible_pending_policy_approval` for analysis, but not `auto_accepted_by_policy`.

### Required context for any future auto-accept

Auto-accept is impossible unless all of the following exist and are approved:

- immutable private `document_id`/`evidence_id`, content hash, timestamps, source, storage safety, and OCR/parser refs;
- reproducible OCR/parser/model/prompt version and run id;
- approved location-scoped policy snapshot with species/service row, vaccines, aliases, source rule, validity rule, and service window;
- exactly one selected customer, one selected pet, and relevant service/reservation context if eligibility is compared;
- deterministic rule id, acceptance reason, system/actor ref, timestamp, evidence refs, before/after state, audit sink, rollback/supersede path, and monitoring plan.

### Future auto-accept eligibility checklist

A future deterministic policy may accept only if all configured checks pass:

1. Document type/source is eligible for the narrow path.
2. Required fields are complete: pet identity, species, source/clinic state, canonical vaccine name, administered date if required, expiration/due date if required, evidence refs, and policy snapshot.
3. Expiration/due date is valid for the relevant service window.
4. Identity is unambiguous and mapped to exactly one pet/customer.
5. Source proof satisfies policy.
6. Vaccine alias/series mapping is explicitly covered by policy.
7. No contradictions, missing required policy fields, low-quality scan blockers, unsupported document types, or privacy/security blockers are present.
8. Confidence and evidence-ref thresholds meet approved minimums. Proposed discussion threshold: overall record confidence at least 0.95 and acceptance-critical fields at least 0.90, but confidence is never sufficient by itself.
9. Auto-accept only changes internal review state unless a separate side-effect policy is approved.

### Human-review routing rules

Route to `MedicalDocumentReview`, `SpecialReview`, `NeedsHumanReview`, or `blocked` when any of these apply:

| Condition | Risk flags / review reasons |
| --- | --- |
| Missing or incomplete required fields | `missing_inputs`, `missing_required_vaccine_proof`, `source_refs_missing` |
| Missing explicit future-valid expiration/due date | `ambiguous_vaccine_date`, `expired_or_stale_vaccine`, `unknown_validity` |
| Expired before service end/date | `expired_or_stale_vaccine`, `missing_required_vaccine_proof` |
| Ambiguous date format, missing year, future administered date, impossible date order | `ambiguous_vaccine_date`, `conflicting_vaccine_fact` |
| Pet/customer unmapped, multiple candidates, species mismatch, selected pet absent | `pet_identity_unmapped`, `conflicting_pet_identity`, `multi_pet_ambiguity` |
| Multi-pet document | `multi_pet_ambiguity`, `needs_document_split` |
| Policy snapshot missing/stale/draft or service row unknown | `missing_vaccine_policy`, `service_policy_unknown`, `policy_snapshot_missing` |
| Alias or series rule not covered | `policy_coverage_unknown`, `canonical_vaccine_mapping_uncertain` |
| Clinic/vet source missing or unverified | `unverified_veterinary_source` |
| Contradictory documents, rows, or provider data | `contradictory_vaccine_evidence`, `provider_payload_unverified` |
| Handwritten, blurred, cropped, low-quality, unreadable, or low OCR confidence | `unreadable_document`, `low_quality_scan`, `ocr_confidence_low` |
| Unrelated or unsupported document | `unsupported_document_type`, `not_vaccine_document` |
| Raw PII/provider payload required but not safely available | `raw_pii_redacted`, `provider_payload_unverified` |
| Any customer-facing or live provider action requested | `side_effect_requires_approval` |

### Decision states

Use these states for review/automation tracking:

- `suggested_unverified` — extraction suggestion only.
- `needs_medical_document_review` — staff must review before acceptance.
- `needs_more_information` — readable proof, source, date, or identity information is missing.
- `auto_accept_eligible_pending_policy_approval` — appears clean but gates are not approved.
- `accepted_by_staff` — authorized reviewer accepted facts.
- `auto_accepted_by_policy` — future deterministic policy accepted facts with audit.
- `rejected_or_not_relevant` — unsupported, irrelevant, wrong pet, wrong source, unusable, or rejected.
- `superseded` — newer/better evidence or corrected policy/mapping replaced it.

## 8. Staff, customer, provider, and UI expectations

### Staff UI

Staff review should show:

- safe document preview or redacted preview according to privacy/access policy;
- document source, upload timestamp, hash/dedupe context, scan/redaction/OCR status;
- selected/candidate customer, pet, reservation, service, and location;
- extraction suggestions with source refs, confidence bands, ambiguity, contradictions, and missing fields;
- policy snapshot id/version and per-requirement comparison suggestions;
- explicit review actions: accept fact, reject fact, request more information, split multi-pet document, supersede with another document, escalate to manager/admin, or mark not relevant;
- audit-visible reviewer identity, reason, and before/after state.

### Customer UI

Customer-facing behavior is limited to safe intake confirmation unless separately approved. The UI may say that a document was received/uploaded. It must not say vaccines are verified, accepted, rejected, expired, or missing unless a reviewed/approved customer-message path exists. Any missing-proof or reupload text from the agent is draft-only and approval-gated.

### Provider integrations

Provider data is read-only evidence in the MVP. Raw provider payloads are quarantined, schema-incomplete, and referenced by ids. No provider write-back, mutation, immunization update, or customer-facing provider action is in scope.

## 9. Result envelope expectations

The workflow result should include:

- `status`: `success`, `needs_human_review`, `blocked`, or `failed`.
- `structured_output`: `VaccineDocumentExtractionResult` suggestions.
- `recommended_actions`: internal suggestions such as `RequestHumanReview`, `PrepareDocumentReviewPacket`, or `SuggestReservationStatus(VaccinePending/MissingInfo/SpecialReview)`.
- `tasks_to_create`: review-task drafts or policy-approved internal tasks only.
- `draft_messages`: absent by default; if present in a future path, always `DraftOnly` and customer-message approval-gated.
- `risk_flags`: typed review/security/privacy/policy flags.
- `verification`: evidence ids, OCR/parser/model version, redaction profile, unchecked sources, confidence category, and policy snapshot id.
- `human_review_reason`: required whenever status is `needs_human_review` or `blocked`.

Recommended actions are suggestions. They do not execute mutations.

## 10. Risk flags

Canonical risk flags include:

- `missing_required_vaccine_proof`
- `unverified_veterinary_source`
- `conflicting_pet_identity`
- `pet_identity_unmapped`
- `ambiguous_vaccine_date`
- `expired_or_stale_vaccine`
- `future_dated_vaccine`
- `contradictory_vaccine_evidence`
- `unsupported_document_type`
- `not_vaccine_document`
- `unreadable_or_low_quality_document`
- `low_quality_scan`
- `ocr_confidence_low`
- `multi_pet_document_requires_split`
- `policy_snapshot_missing`
- `missing_vaccine_policy`
- `service_policy_unknown`
- `policy_coverage_unknown`
- `canonical_vaccine_mapping_uncertain`
- `provider_payload_unverified`
- `raw_pii_redacted`
- `security_scan_failed_or_missing`
- `duplicate_document_detected`
- `extraction_failed`
- `side_effect_requires_approval`

## 11. Audit trail

Audit every major step without raw/private payloads:

| Step | Audit content |
| --- | --- |
| Upload received | Actor, source, location, request id, sanitized file envelope, no raw contents |
| Raw object stored | Document id, evidence id, content hash, storage ref, privacy class, retention ref |
| Metadata mapped | Customer/pet/reservation mapping state, candidate ids, source, conflicts |
| Event emitted | `document.uploaded` event id, idempotency key, related ids, policy context |
| Extraction queued/started/completed | Job id, engine/parser/model version, evidence id, status, failure category |
| Suggestions created | Suggestion ids, evidence refs, confidence category, unknown/conflict markers |
| Policy compared | Policy snapshot id, comparison state, missing policy/freshness/source warnings |
| Review task drafted/created | Task/draft id, dedupe key, assignee role, priority/due basis, source evidence |
| Staff review decision | Reviewer actor/role, accepted/rejected/superseded facts, reason, policy snapshot, before/after state |
| Auto-accept decision | Future only: policy id/version, deterministic criteria, validator result, actor/system id, reversal path |
| Downstream execution | Separate audited command/result for any profile/reservation/provider/customer-message change |

Reviewed vaccine facts should record pet/customer/reservation/location ids, policy id/version, canonical vaccine name, accepted alias/source label, accepted dates, evidence ids/page/span refs, source verification state, reviewer/actor, accepted/rejected/superseded state, comparison result, audit event id, and workflow event id.

## 12. Test corpus summary

The fixture corpus should be synthetic or de-identified. It must prove that extraction can help staff while preserving uncertainty and approval gates.

Corpus-wide requirements:

- no raw real customer PII unless separately approved and isolated;
- deterministic fixture ids, evidence ids, policy snapshot ids, and expected outputs;
- raw document/OCR/provider payloads kept in fixture evidence storage or safe synthetic text, not in logs;
- every scenario asserts that medical-document facts remain suggestions until review/policy acceptance;
- safe-log assertions reject raw OCR/full documents/PII/provider payload leaks;
- customer-facing/live actions and provider mutations are forbidden in fixtures.

Required scenario matrix:

| ID | Fixture | Expected status | Key expected flags | Review expectation |
| --- | --- | --- | --- | --- |
| `clean_pdf` | Clean one-page vaccine PDF | `success` or `needs_human_review` | none or `unverified_veterinary_source` if source proof not modeled | Medical review before acceptance |
| `phone_photo` | Skewed phone photo | `needs_human_review` | `low_quality_document` if OCR confidence lower; possible `ambiguous_vaccine_date` | Review readable suggestions |
| `blurry_image` | Blurry/low-contrast image | `needs_human_review` or `blocked` | `low_quality_document`, `unreadable_document` | Request rescan/review |
| `multi_pet` | Multi-pet household record | `needs_human_review` | `conflicting_pet_identity` or `multi_pet_document` | Reviewer maps/splits relevant pet |
| `expired_vaccine` | Expired/stale vaccine proof | `needs_human_review` | `expired_or_stale_vaccine` | Reviewer confirms expiry and next step |
| `handwritten_form` | Handwritten clinic form | `needs_human_review` | `low_ocr_confidence`, `unverified_veterinary_source`, maybe `ambiguous_vaccine_date` | Reviewer verifies handwritten fields |
| `vet_invoice` | Invoice with vaccine line items | `needs_human_review` | `document_kind_invoice`, `unverified_veterinary_source` if proof insufficient | Reviewer decides whether invoice counts as proof |
| `unrelated_document` | Boarding agreement/grooming receipt/photo | `no_action` or `needs_human_review` | `unsupported_document_type`, `no_vaccine_content` | No vaccine acceptance |
| `rabies_certificate` | Rabies certificate | `success` or `needs_human_review` | none if fully readable, still unreviewed | Medical review before acceptance |
| `portal_export` | Screenshot/table export | `needs_human_review` | `source_unverified` or `provider_payload_unverified` | Verify source and policy match |
| `ambiguous_dates` | Missing year or MM/DD vs DD/MM ambiguity | `needs_human_review` | `ambiguous_vaccine_date` | Reviewer resolves date |
| `conflicting_duplicates` | Duplicate rows/docs conflict | `needs_human_review` | `conflicting_vaccine_record` | Reviewer chooses/supersedes |
| `identity_mismatch` | Pet/owner/species mismatch | `needs_human_review` or `blocked` | `conflicting_pet_identity` | Reviewer remaps/rejects |
| `non_english_mixed_labels` | Spanish/French/mixed vaccine labels | `needs_human_review` | `translation_required` or `low_confidence_mapping` | Reviewer verifies translation/mapping |
| `pii_redaction_stress` | Contact-heavy synthetic doc | `needs_human_review` | `raw_pii_redacted` | Redaction must pass before review packet |
| `missing_policy_snapshot` | Document available but no approved policy snapshot | `needs_human_review` or `blocked` | `policy_snapshot_missing`, `missing_vaccine_policy` | Policy owner/staff review required |
| `email_attachment_minimal_metadata` | Email attachment with limited source metadata | `needs_human_review` | `unverified_veterinary_source`, maybe `source_timestamp_unknown` | Verify provenance/source |
| `future_or_impossible_dates` | Future administered date or expiration before administered date | `needs_human_review` | `ambiguous_vaccine_date`, `conflicting_vaccine_fact` | Reviewer resolves/rejects |

Pass/fail categories:

1. Prompt/input construction includes safe ids, policy snapshot context, allowed/forbidden actions, and default review gates.
2. Extraction semantics preserve canonical vaccine names, aliases, dates, identity candidates, source refs, confidence, ambiguity, and contradictions.
3. Review/policy boundary asserts no acceptance, no booking/provider/customer mutation, and required human-review reasons.
4. Privacy/security assertions prevent raw/private payload leakage.
5. Idempotency/dedupe assertions prevent duplicate review tasks while preserving contradictory evidence.

## 13. Implementation guardrails

- Treat the workflow as document intake plus review support, not eligibility automation.
- Require policy snapshot ids for any policy comparison.
- Keep source facts, dates, identity, and vaccine aliases independently reviewable.
- Do not collapse contradictions into a single best guess.
- Do not create customer-visible status text from unreviewed suggestions.
- Do not let auto-accept exist as a model confidence shortcut; it must be deterministic, policy-approved, auditable, reversible, and monitored.
- Store raw evidence privately; result payloads carry ids and safe summaries only.
- Keep future provider write-back and customer messaging as separate approval-gated workflows.

## 14. Open assumptions and approval gates

These must be explicitly approved before production behavior relies on them:

1. Final location vaccine policy by species/service/location, including required vaccines, optional/location-specific vaccines, disabled services, and `Species::Other` handling.
2. Exact vaccine names, aliases, equivalences, CIV strain/series rules, DHPP/DHLPP/lepto handling, FeLV handling, puppy/kitten rules, medical exemptions, titers, legal exceptions, and grace periods.
3. Freshness/validity windows by vaccine/species/service, including whether missing expiration dates can use fallback windows.
4. Licensed-veterinarian proof definition and source-verification method.
5. Document storage backend, bucket/container names, encryption mode, retention classes, redaction/signed-preview rules, evidence id scheme, and access policy.
6. OCR/parser/LLM stack, prompt/result retention policy, versioning, reproducibility, and drift monitoring.
7. Review UI/state machine, staff roles, manager/admin gates, override/exception paths, rejection/supersede workflow, and dispute handling.
8. Whether internal review tasks are automatically created or drafted for staff approval.
9. Whether any deterministic receipt-only customer message is safe without review.
10. Whether future auto-accept is allowed at all; if yes, its threshold, deterministic criteria, scope, audit, sampling, rollback, and monitoring plan.
11. Whether any downstream reservation/profile/provider/customer-message action can ever be triggered from accepted vaccine facts, and what separate approval path controls it.
12. Where committed synthetic fixtures live and where large/private/de-identified document assets are stored.

Until these gates are approved, the canonical behavior is: receive and store evidence privately, extract suggestions, compare only against approved policy snapshots when available, route uncertainty to human review, and never execute customer-facing or live provider/reservation actions.
