# Vaccine document upload workflow

Purpose: define the internal upload -> private storage -> `document.uploaded` -> extraction -> unverified result -> review/auto-accept workflow for vaccine and immunization documents. This is a workflow-design artifact for `docs/workflows/vaccine-document-agent.md`; it does not authorize live customer messages, provider writes, reservation mutations, or medical/vaccine approval by AI.

Status: draft workflow part. Preserve all medical-document uncertainty as suggestions or unverified data unless an explicit deterministic policy and audit record allow a narrow auto-accept path.

Source basis:

- `docs/workflows/vaccine-document-agent-parts/inputs.md` is the canonical input packet for this workflow.
- `docs/architecture/pet-resort-workflow-events.md` defines the global workflow envelope, `document.uploaded`, `vaccine.extraction_needed`, and the rule that raw provider payloads/OCR stay in evidence storage.
- `docs/architecture/workflow-result-envelope.md` defines result statuses, structured output, recommended actions, draft/task semantics, risk flags, verification, and human-review reasons.
- `domain/src/workflow.rs` currently names `WorkflowEventType::VaccineDocumentUploaded`, `AllowedAction::{ReadEntities, ExtractStructuredData, CreateInternalTask, FlagRisk}`, `WorkflowStatus`, and `RecommendedAction::RequestHumanReview`.
- `domain/src/policy.rs` anchors `VaccineRequirement`, `VaccineName`, `ReviewGate::MedicalDocumentReview`, `PolicyDenialReason::MedicalDocumentReviewRequired`, and automation levels.
- `domain/src/entities.rs` anchors customer, pet, reservation, policy refs, `ReservationStatus::VaccinePending`, `HardStop::MissingRequiredVaccine`, and audit concepts.

## Core rule

Upload success is document intake only. It may create a private evidence record, emit `document.uploaded`, queue extraction, produce unverified vaccine suggestions, draft internal review tasks, and recommend reservation/profile readiness changes. It must not by itself accept vaccine facts, mark a pet compliant, update a live provider, send customer-facing communication, or change reservation status.

Every accepted vaccine fact must trace to:

1. private raw evidence or provider evidence reference;
2. extraction/parser version or reviewer-entered source;
3. location-scoped vaccine policy snapshot;
4. staff/manager approval or an explicit auto-accept policy decision;
5. audit record naming actor, timestamp, before/after state, reason, and evidence.

If any of those are missing, stale, ambiguous, conflicting, or unverified, the output remains `unverified` or `needs_review`.

## End-to-end lifecycle

1. Receive upload or document reference.
   - Sources: customer portal, staff scan, staff upload, designated email ingestion, read-only provider poll/import, or migration import.
   - Actor: customer/owner, staff, or system import. A pet is a subject, not an actor.
   - Validate file envelope before storage: size, MIME allowlist, extension sanity, malware scan status if available, parseability hint, and source authentication.
   - Do not inspect or log raw file text in the request path beyond minimal file-envelope validation.

2. Store raw object privately.
   - Write original bytes to private object/blob storage under an immutable storage key.
   - Compute content hash and create an immutable `document_id`/`evidence_id` pair before any downstream event is emitted.
   - Store original filename only after sanitization. Never let filename, OCR text, or provider strings drive policy behavior directly.
   - Store raw bytes, raw OCR text, and raw provider payloads as evidence/boundary records, not workflow event payloads.

3. Create normalized document metadata.
   - Record upload/source facts, storage refs, mapping confidence, privacy/retention class, and processing statuses in a normalized document metadata record.
   - Keep pet/customer/reservation mappings explicit: `mapped`, `candidate`, `conflicting`, or `unmapped`.
   - If pet identity is not safely mapped, emit the event with an external-document subject and route to review before any pet-level vaccine comparison.

4. Emit `document.uploaded`.
   - Emit only after private storage and metadata creation succeed.
   - Use event idempotency based on source evidence id and content hash so retries cannot create duplicate review packets.
   - The event payload includes typed metadata and refs only. It excludes raw document bytes, raw OCR text, full extracted text, unredacted owner contact details, and raw provider JSON.

5. Decide whether extraction is needed.
   - If document kind hint, upload context, reservation/profile gaps, or policy says vaccine proof may be present, derive `vaccine.extraction_needed`.
   - If the document is clearly unsupported or irrelevant, route to `no_action` or document review with `unsupported_document_type`; do not silently delete or accept.
   - If policy snapshot is missing, extraction may still run to prepare review suggestions, but eligibility comparison is blocked.

6. Run OCR/parser/extraction.
   - Extraction reads raw evidence through a privacy-scoped tool and writes derived artifacts linked to evidence id.
   - Outputs are `VaccineExtractionSuggestion` records: pet identity candidates, vaccine-name candidates, dates, source/clinic/vet candidates, confidence, evidence spans/pages/images, parser/OCR version, and explicit unknown/conflict markers.
   - AI confidence can prioritize review; it cannot approve medical facts.

7. Produce unverified workflow result.
   - Result `structured_output` contains suggestions and policy-comparison suggestions, not accepted vaccine facts.
   - Result status is usually `needs_human_review` when medical/vaccine facts could affect eligibility, `blocked` when policy/source/pet mapping is missing, `failed` for safe processing failure, `no_action` for valid irrelevant documents, or `success` only for safe internal artifact creation with no implied medical acceptance.
   - Include `verification` with evidence ids, extraction version, unchecked sources, redactions, and confidence category.

8. Route review or approved auto-accept.
   - Default: create or draft a `DocumentReview`/medical document review item for staff.
   - Reviewer can accept, reject, supersede, split multi-pet records, request more information, or escalate to manager/admin.
   - Auto-accept is allowed only for a future narrow deterministic policy that names allowed document types, source trust, policy snapshot, required fields, confidence/evidence thresholds, reviewer bypass rationale, and audit fields. In the current MVP posture, vaccine facts remain review-gated.

9. Record final disposition and downstream suggestions.
   - Accepted/rejected facts update internal vaccine-document state only after approval or approved auto-accept.
   - Reservation/profile status changes remain recommendations until executed through an approved adapter/tool path.
   - Customer follow-up remains draft-only unless a later policy explicitly permits deterministic receipt or missing-info messaging.

## Storage object and metadata contract

Raw object storage must be private, least-privilege, encrypted at rest where supported, and excluded from analytics/log sinks. Use separate access controls for raw files, derived OCR artifacts, normalized metadata, and review/audit records.

Recommended raw object metadata:

| Field | Requirement |
| --- | --- |
| `document_id` | Stable semantic document id used by workflow events and review UI. |
| `evidence_id` | Immutable evidence id for the exact raw object version. |
| `storage_key` | Private object key; do not expose directly to customers or logs. |
| `content_hash` | SHA-256 or equivalent content hash for dedupe and integrity. |
| `source` | `customer_upload`, `staff_scan`, `staff_upload`, `email_ingest`, `provider_poll`, `provider_webhook`, `migration_import`. |
| `uploader_actor` | Customer/staff/system actor ref; avoid embedding contact details. |
| `received_at` | Server receipt timestamp; preserve source timestamp separately if present. |
| `source_timestamp` | Timestamp from portal/email/provider when trustworthy enough to display as source context. |
| `original_filename_sanitized` | Optional display filename after sanitization; not a policy input. |
| `mime_type` / `extension` | Detected and supplied values; mismatches become review/security flags. |
| `size_bytes` | Stored for validation, review, and troubleshooting. |
| `page_or_image_count` | If known without expensive processing; otherwise set `unknown`. |
| `document_kind_hint` | Customer/staff/provider hint such as `vaccine_record`; not authoritative. |
| `expected_content_hint` | Context like vaccine proof, medication instructions, or boarding agreement. |
| `mapping_state` | `mapped`, `candidate`, `conflicting`, or `unmapped` for customer/pet/reservation. |
| `customer_id` / `pet_id` / `reservation_id` | Optional typed refs; absent or candidate refs are not approval. |
| `provider_ref` | Provider name and external id when source is provider-backed; raw provider payload stays quarantined. |
| `privacy_class` | Medical/customer document class, retention policy ref, and access policy ref. |
| `redaction_status` | `not_applicable`, `pending`, `redacted_copy_available`, or `failed`. |
| `scan_status` | Malware/security scan status if available; unsafe/unknown blocks extraction. |
| `ocr_status` / `extraction_status` | `not_started`, `queued`, `running`, `completed`, `needs_review`, `failed`, or `superseded`. |
| `supersedes_document_id` | Optional prior document superseded by a clearer or newer upload. |

Derived artifact metadata should additionally record engine/vendor/model, parser version, prompt/template version when applicable, run id, created-at timestamp, evidence span/page refs, redaction notes, and failure category.

## Privacy boundaries

Do not put these in workflow events, ordinary logs, queue titles, customer-visible summaries, or model prompts unless a privacy-scoped extraction/review tool explicitly requires them:

- raw PDF/image bytes;
- raw OCR text or full document transcription;
- raw provider JSON or webhook bodies;
- API keys, signed URLs, credentials, or provider auth headers;
- full owner contact details beyond typed customer refs;
- payment-like strings, IDs, or credentials accidentally present in uploads;
- unrelated medical/medication/incident content not needed for vaccine review.

Allowed in workflow events and result summaries:

- typed ids and evidence refs;
- sanitized file envelope facts;
- document kind hints and processing statuses;
- semantic risk flags;
- extraction confidence category and parser version;
- limited operator-safe summaries that do not reveal raw medical details beyond the internal review audience.

When an extraction prompt or OCR job needs raw content, the job should run behind a privacy-scoped boundary and return structured suggestions with evidence refs. Prompt/result retention must follow the document retention policy.

## Event names and payloads

### `document.uploaded`

Emit after raw storage and normalized metadata creation. This is the canonical intake event for all document kinds, including vaccine documents.

Example payload shape:

```json
{
  "event_id": "workflow-event-id",
  "type": "document.uploaded",
  "occurred_at": "2026-06-11T00:00:00Z",
  "source": "customer_portal",
  "actor": { "type": "customer", "customer_id": "customer-id" },
  "location_id": "location-id",
  "subject": { "type": "pet", "pet_id": "pet-id" },
  "related_ids": {
    "document_id": "document-id",
    "evidence_id": "evidence-id",
    "customer_id": "customer-id",
    "pet_id": "pet-id",
    "reservation_id": "reservation-id",
    "policy_snapshot_id": "vaccine-policy-snapshot-id"
  },
  "payload": {
    "document_kind_hint": "vaccine_record",
    "expected_content_hint": "vaccine_proof",
    "source": "customer_upload",
    "file_metadata": {
      "mime_type": "application/pdf",
      "size_bytes": 123456,
      "content_hash": "sha256:...",
      "page_or_image_count": 2,
      "original_filename_sanitized": "vaccine-record.pdf"
    },
    "storage_ref": {
      "storage_key_ref": "evidence-id",
      "access_class": "private_medical_document"
    },
    "mapping_state": "mapped",
    "ocr_status": "not_started",
    "extraction_status": "not_started",
    "source_timestamp": "2026-06-11T00:00:00Z"
  },
  "policy_context": {
    "allowed_actions": ["ReadEntities", "ExtractStructuredData", "CreateInternalTask", "FlagRisk"],
    "automation_level": "InternalTaskOnly",
    "required_reviews": ["MedicalDocumentReview"],
    "policy_snapshot_ids": ["vaccine-policy-snapshot-id", "document-retention-policy-id"]
  }
}
```

If pet mapping is unresolved, use `subject = external document` and put candidate customer/pet/reservation ids in metadata with `mapping_state = candidate` or `conflicting`.

### `vaccine.extraction_needed`

Derive this when a stored document may contain vaccine proof or when a reservation/profile gap requires vaccine document extraction.

Example payload shape:

```json
{
  "event_id": "workflow-event-id",
  "type": "vaccine.extraction_needed",
  "occurred_at": "2026-06-11T00:01:00Z",
  "source": "policy_evaluator",
  "actor": { "type": "system" },
  "location_id": "location-id",
  "subject": { "type": "pet", "pet_id": "pet-id" },
  "related_ids": {
    "document_id": "document-id",
    "evidence_id": "evidence-id",
    "customer_id": "customer-id",
    "pet_id": "pet-id",
    "reservation_id": "reservation-id",
    "source_event_id": "document-uploaded-event-id",
    "policy_snapshot_id": "vaccine-policy-snapshot-id"
  },
  "payload": {
    "extraction_reason": "vaccine_proof_uploaded_for_reservation",
    "document_kind_hint": "vaccine_record",
    "required_vaccine_policy_ref": "vaccine-policy-snapshot-id",
    "required_vaccine_names": ["policy-defined-name-or-empty-if-policy-missing"],
    "current_known_vaccine_record_refs": [],
    "mapping_state": "mapped",
    "source_status": "uploaded_unverified",
    "freshness_requirement": "policy_defined_or_unknown"
  },
  "policy_context": {
    "allowed_actions": ["ReadEntities", "ExtractStructuredData", "CreateInternalTask", "FlagRisk"],
    "automation_level": "InternalTaskOnly",
    "required_reviews": ["MedicalDocumentReview"],
    "policy_snapshot_ids": ["vaccine-policy-snapshot-id"]
  }
}
```

Do not include raw OCR, full extracted text, or raw provider payloads in either event.

## Idempotency, retries, and dedupe

Use deterministic keys so retries are safe:

| Step | Idempotency key | Retry behavior |
| --- | --- | --- |
| Raw object create | `source + source_document_ref + content_hash` | If same content/source already exists, return existing `document_id` and append audit observation if needed. |
| Metadata create | `document_id` | Upsert processing status only when transition is valid. |
| `document.uploaded` emit | `document_id + evidence_id + source_event_ref` | Do not emit duplicate intake events for the same immutable evidence. |
| `vaccine.extraction_needed` derive | `document_id + pet_id/candidate_subject + policy_snapshot_id + extraction_reason` | Re-derive when policy snapshot changes or pet mapping changes; otherwise reuse existing event/job. |
| OCR/extraction job | `evidence_id + extraction_engine_version + parser_version + redaction_profile` | Safe to retry failed/transient jobs; new engine/version creates new derived artifact. |
| Review task draft/create | `document_id + pet_id + review_gate + active_policy_snapshot_id` | Avoid duplicate open review tasks; update the existing review packet with new evidence. |
| Suggestion record | `evidence_id + normalized_vaccine_candidate + evidence_span + parser_version` | Preserve conflicts; do not collapse contradictory suggestions into silent replacement. |

Retry categories:

- Transient storage or queue failure: retry with same idempotency key.
- OCR/parser timeout: retry job; if still failing, mark extraction `failed` and route `needs_human_review`/`blocked` with `extraction_failed`.
- Unsafe file or malware scan failure: block extraction and route to privacy/security review.
- Policy snapshot missing: keep document stored and extraction suggestions allowed, but policy comparison/status recommendation is `blocked`.
- Pet/customer mapping conflict: pause pet-level extraction comparison; route mapping review.
- Duplicate upload: link duplicate evidence to existing active review or accepted record; do not auto-reapprove.

Supersession must be explicit. A newer document can supersede older unverified suggestions, but previously accepted vaccine facts require a review/audit decision to replace or invalidate.

## Status transitions

### Document processing state

Recommended document-level states:

| State | Meaning | Allowed next states |
| --- | --- | --- |
| `received` | Upload request accepted but raw storage not finalized. | `stored`, `rejected_unsafe`, `failed` |
| `stored` | Raw object and metadata exist. | `event_emitted`, `duplicate`, `failed` |
| `event_emitted` | `document.uploaded` emitted. | `extraction_queued`, `review_needed`, `no_action`, `failed` |
| `extraction_queued` | Extraction job requested. | `extracting`, `extraction_failed`, `superseded` |
| `extracting` | OCR/parser running. | `extracted_unverified`, `extraction_failed`, `superseded` |
| `extracted_unverified` | Suggestions produced; no medical facts accepted. | `review_needed`, `auto_accept_candidate`, `superseded` |
| `review_needed` | Staff/medical document review required. | `accepted_by_staff`, `rejected_or_not_relevant`, `needs_more_information`, `escalated`, `superseded` |
| `auto_accept_candidate` | Meets a future deterministic policy candidate; still awaiting validator/audit. | `accepted_by_policy`, `review_needed`, `rejected_or_not_relevant` |
| `accepted_by_staff` | Human accepted one or more structured facts. | `superseded`, `closed` |
| `accepted_by_policy` | Future policy auto-accepted facts with audit record. | `superseded`, `closed` |
| `rejected_or_not_relevant` | Reviewer/policy found the document unsupported, irrelevant, wrong pet, or unusable. | `closed`, `superseded` |
| `needs_more_information` | Staff/customer/provider evidence needed. | `review_needed`, `superseded`, `closed` |
| `superseded` | Replaced by newer/better evidence or corrected mapping. | `closed` |
| `failed` | Safe processing failure after retries. | `review_needed`, `closed` |

### Extraction suggestion review state

Each extracted vaccine candidate has its own review state:

- `suggested_unverified`: parser/OCR/AI found a candidate field with evidence refs.
- `needs_identity_review`: pet/customer/reservation mapping is uncertain.
- `needs_policy_review`: vaccine name, service, species, source, or freshness depends on policy.
- `needs_date_review`: date is ambiguous, missing, future-dated, stale, conflicting, or low-confidence.
- `needs_source_review`: clinic/vet/source proof is missing or unverified.
- `accepted_by_staff`: authorized reviewer accepted the fact.
- `rejected`: reviewer rejected it with reason.
- `superseded`: better evidence or policy changed the active candidate.

Do not infer an accepted vaccine record from document state alone. Acceptance is per fact, per pet, per policy snapshot.

### Reservation/profile readiness suggestions

The upload workflow may recommend, but not execute, readiness changes:

- `VaccinePending`: document is present but proof is unverified, missing, expired/stale, or review is pending.
- `MissingInfo`: no usable document, wrong pet, unreadable upload, or needed source/date/vaccine field is missing.
- `SpecialReview`: medical/source/policy conflict, manager exception, customer dispute, or privacy/security issue.
- `ready_for_staff_approval`: only after policy-backed facts appear satisfied, and still not a provider/customer-visible mutation.

No transition to confirmed/accepted provider state occurs from this workflow result alone.

## Review and auto-accept policy

Default MVP posture: medical document review is required. The agent can produce review packets and unverified suggestions only.

A future auto-accept path must be opt-in and deterministic. It must define all of the following before any extracted fact bypasses staff review:

- allowed source(s), such as trusted provider import or staff-entered reviewed scan, not generic customer upload;
- allowed document kind(s) and file/security scan requirements;
- required pet identity match criteria;
- required vaccine policy snapshot and exact vaccine-name mapping;
- required date fields and acceptable freshness windows;
- required licensed-vet/source evidence if policy requires it;
- minimum OCR/parser evidence quality and parser version allowlist;
- conflict checks against existing records and duplicates;
- audit fields and reversal/supersession behavior;
- explicit exclusion of customer-facing sends and provider writes unless separately approved.

Even when a future `accepted_by_policy` path exists, low-confidence fields, conflicts, missing policy, missing source proof, multi-pet ambiguity, unsupported document type, customer disputes, and stale provider records must route to `MedicalDocumentReview`.

## Staff UI expectations

Staff-facing UI should show a review packet, not a hidden automation result.

Minimum review packet:

- customer, pet, reservation, service/date context, and location;
- document source, received time, sanitized filename, MIME/size/page count, and scan/extraction status;
- safe preview link gated by document permissions;
- extracted suggestions grouped by pet and vaccine candidate;
- evidence snippets/spans/page thumbnails where privacy policy allows;
- policy snapshot used for comparison and any missing policy warning;
- current known vaccine records and conflicts/duplicates;
- risk flags and human-review reason;
- recommended action(s): accept fact, reject fact, mark not relevant, request more info, split multi-pet document, escalate, or supersede;
- audit history for upload, extraction, review, and downstream execution.

Reviewer actions must require a reason when rejecting, superseding, overriding conflicts, accepting ambiguous fields, or marking source proof verified. Bulk accept should be disabled unless every selected candidate has policy-backed evidence and no conflict/review flag.

## Customer UI expectations

Customer-facing UI is out of scope for live actions, but the workflow should support safe future experiences:

- Upload receipt may say the file was received, not that vaccines are approved.
- Status language should distinguish `Uploaded`, `Under review`, `Need more information`, `Accepted by staff`, and `Rejected/not usable`.
- Customer should not see raw AI confidence or internal risk flags.
- Customer-visible rejection/missing-info explanations are draft/customer-message artifacts requiring approval unless a later deterministic template policy exists.
- A customer should be able to upload a replacement document without deleting the audit trail for the earlier upload.
- Multi-pet or wrong-pet ambiguity should ask for clarification rather than exposing another pet/customer's details.

## Result envelope expectations

Extraction workflow result shape:

```json
{
  "status": "needs_human_review",
  "summary": "Vaccine document extracted into unverified suggestions; staff medical document review required before acceptance.",
  "structured_output": {
    "document_id": "document-id",
    "evidence_id": "evidence-id",
    "document_review_state": "review_needed",
    "suggestions": [
      {
        "suggestion_id": "suggestion-id",
        "pet_identity": { "pet_id": "pet-id", "confidence": "candidate", "review_state": "needs_identity_review" },
        "vaccine_name_candidate": "rabies-or-policy-mapped-name",
        "administered_date_candidate": "2026-01-01",
        "expiration_or_due_date_candidate": "2027-01-01",
        "source_candidate": "clinic/vet text if allowed or evidence ref",
        "evidence_refs": ["evidence-page-span-id"],
        "confidence": "medium",
        "review_state": "suggested_unverified"
      }
    ],
    "policy_comparison": [
      {
        "required_vaccine_ref": "policy-vaccine-ref",
        "matched_suggestion_id": "suggestion-id",
        "state": "source_unverified",
        "review_required": true
      }
    ]
  },
  "recommended_actions": [
    { "type": "RequestHumanReview", "gate": "MedicalDocumentReview" }
  ],
  "draft_messages": [],
  "tasks_to_create": [
    {
      "kind": "DocumentReview",
      "title": "Review uploaded vaccine document",
      "creation_policy": "RequiresReview"
    }
  ],
  "risk_flags": ["unverified_veterinary_source", "ambiguous_vaccine_date"],
  "verification": {
    "evidence": ["document-id", "evidence-id", "ocr-artifact-id"],
    "unchecked_sources": ["licensed veterinarian source not verified"],
    "redactions": ["raw OCR text excluded from event/result summary"],
    "confidence": "extracted_unverified"
  },
  "human_review_reason": "Vaccine facts affect medical/eligibility state and require MedicalDocumentReview before acceptance."
}
```

`status = success` is permitted only when the workflow successfully created safe internal artifacts and no sensitive decision is being claimed. It never means vaccine approval. If any consumer might interpret success as compliance, prefer `needs_human_review` with an explicit reason.

## Audit trail

Record append-only audit events for each meaningful step:

| Step | Audit content |
| --- | --- |
| Upload received | Actor, source, location, request id, sanitized file envelope, no raw contents. |
| Raw object stored | Document id, evidence id, content hash, storage ref, privacy class, retention ref. |
| Metadata mapped | Customer/pet/reservation mapping state, candidate ids, confidence/source, conflicts. |
| Event emitted | `document.uploaded` event id, idempotency key, related ids, policy context. |
| Extraction queued/started/completed | Job id, engine/parser version, evidence id, status, failure category if any. |
| Suggestions created | Suggestion ids, evidence refs, confidence category, unknown/conflict markers. |
| Policy compared | Policy snapshot id, comparison state, missing policy/freshness/source warnings. |
| Review task drafted/created | Task id or draft id, dedupe key, assignee role, priority/due basis, source evidence. |
| Staff review decision | Reviewer actor/role, accepted/rejected/superseded fact ids, reason, policy snapshot, before/after state. |
| Auto-accept decision | Policy id/version, validator result, exact criteria satisfied, actor/system id, reversal path. |
| Downstream execution | Separate audited command/result for any profile/reservation/provider/customer-message change. |

Audit records must be durable, queryable by document, pet, reservation, workflow event, reviewer, and policy snapshot. They must avoid secrets and raw document text while preserving enough evidence refs for an authorized reviewer to reconstruct the decision.

## Risk flags and review reasons

Use typed risk flags so routing and UI do not depend on prose:

- `missing_required_vaccine_proof`
- `unverified_veterinary_source`
- `conflicting_pet_identity`
- `ambiguous_vaccine_date`
- `expired_or_stale_vaccine`
- `future_dated_vaccine`
- `unsupported_document_type`
- `unreadable_or_low_quality_document`
- `multi_pet_document_requires_split`
- `policy_snapshot_missing`
- `provider_payload_unverified`
- `raw_pii_redacted`
- `security_scan_failed_or_missing`
- `duplicate_document_detected`
- `extraction_failed`

Every `needs_human_review` or `blocked` result should name a concise human-review reason, such as: `MedicalDocumentReview required because extracted vaccine date is ambiguous and source proof is unverified`.

## Implementation guardrails

- Do not let upload endpoint code call provider mutation APIs.
- Do not let OCR/parser output write accepted vaccine records directly.
- Do not use filename, customer note text, provider raw strings, or model confidence as policy authority.
- Do not emit raw document text, raw OCR, or raw provider JSON in workflow events.
- Do not auto-create unlimited review tasks during retry storms; dedupe by document/pet/policy/review gate.
- Do not overwrite accepted facts on duplicate upload without review/supersession audit.
- Do not delete raw evidence solely because a review rejected a suggestion; apply retention policy.
- Do not expose another customer/pet's information when resolving multi-pet or wrong-pet documents.

## Open decisions for implementation cards

1. Which storage backend, bucket/container names, encryption mode, retention classes, and signed-preview rules will be used?
2. What exact `DocumentId`, `EvidenceId`, extraction job id, and audit id types should be added to the Rust domain model?
3. Which OCR/parser/LLM stack is allowed, and what prompt/result retention policy applies?
4. What location-approved vaccine policy artifact defines vaccine names, freshness windows, source requirements, species/service scope, and auto-accept eligibility if any?
5. Which staff roles can accept vaccine facts, verify source proof, reject documents, or supersede accepted records?
6. What deterministic receipt-only customer message, if any, is safe without review?
7. What synthetic/de-identified fixtures cover duplicate, multi-pet, unreadable, ambiguous-date, wrong-pet, expired, source-unverified, and unsupported-document cases?
