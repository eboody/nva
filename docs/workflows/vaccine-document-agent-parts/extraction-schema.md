# Vaccine document extraction schema

Purpose: define the workflow-specific `structured_output` contract for the vaccine-document agent. The schema preserves document facts as extraction suggestions and review evidence; it does not approve vaccines, determine medical eligibility, update a provider/PMS, change reservation status, or send customer-facing messages.

Status: draft schema for `docs/workflows/vaccine-document-agent.md` synthesis. Use this shape under the shared workflow result envelope described in `docs/architecture/workflow-result-envelope.md` and `docs/architecture/pet-resort-ai-runtime-structured-output.md`.

## Safety posture

Non-negotiable rules for this output:

- Every medical-document fact is `suggested`, `unknown`, `ambiguous`, `conflicting`, or `unverified` until a deterministic rule and required human review gate accept it.
- `status: "success"` or `WorkflowStatus::Completed` means the extraction packet is valid and safe to persist as review support. It never means vaccine proof is accepted.
- Confidence is evidence quality, not approval authority. High-confidence fields can still require `MedicalDocumentReview`.
- Raw uploads, OCR text, email bodies, provider payloads, and full medical documents stay in evidence/blob storage. The result references them by ids, page/region/span refs, hashes, or redacted excerpt ids.
- Customer-facing/live actions are out of scope. Drafts and task suggestions, if emitted by the shared envelope, remain review-gated.

## Envelope placement

The vaccine-specific payload belongs in `structured_output`:

```json
{
  "schema_name": "VaccineDocumentExtractionResult",
  "schema_version": "2026-06-11",
  "workflow_name": "vaccine-document",
  "event_id": "evt_document_uploaded_...",
  "subject": { "type": "pet", "id": "pet_..." },
  "status": "needs_human_review",
  "summary": "Vaccine document parsed; rabies date found, Bordetella date ambiguous, clinic source unverified.",
  "structured_output": { "...": "see schema below" },
  "risk_flags": ["ambiguous_vaccine_date", "unverified_veterinary_source"],
  "verification": {
    "evidence": ["evidence_raw_upload_...", "evidence_ocr_..."],
    "unchecked_sources": ["location vaccine policy not final"],
    "redactions": ["raw owner contact details omitted from result"],
    "confidence": "partial_source_backed"
  },
  "human_review_reason": "MedicalDocumentReview required before accepting vaccine facts."
}
```

## JSON-like structured output shape

```json
{
  "document": {
    "document_id": "doc_123",
    "source": "customer_upload",
    "kind": "vaccine_record",
    "received_at": "2026-06-11T14:30:00Z",
    "document_date": {
      "value": "2026-06-01",
      "state": "suggested",
      "confidence": { "score": 0.86, "band": "high", "reason": "printed document date" },
      "source_refs": [{ "evidence_id": "ocr_123", "page": 1, "span_id": "span_doc_date" }]
    },
    "language_hints": ["en"],
    "page_count": 2,
    "raw_attachments": [
      {
        "evidence_id": "raw_123",
        "kind": "raw_upload",
        "storage_ref": "private://vaccines/doc_123/original.pdf",
        "content_hash": "sha256:...",
        "mime_type": "application/pdf",
        "redaction_state": "raw_private_not_inlined"
      },
      {
        "evidence_id": "ocr_123",
        "kind": "ocr_output",
        "storage_ref": "private://vaccines/doc_123/ocr.json",
        "engine": "ocr-vendor-or-model",
        "engine_version": "version-or-build-id",
        "redaction_state": "raw_private_not_inlined"
      }
    ]
  },
  "identity": {
    "pet_candidates": [
      {
        "candidate_pet_id": "pet_123",
        "extracted_name": { "value": "Riley", "state": "suggested", "confidence": { "score": 0.92, "band": "high" }, "source_refs": [] },
        "species": { "value": "dog", "state": "suggested", "confidence": { "score": 0.7, "band": "medium" }, "source_refs": [] },
        "breed_or_description": { "value": "Lab mix", "state": "suggested", "confidence": { "score": 0.55, "band": "medium" }, "source_refs": [] },
        "provider_external_refs": [{ "system": "gingr", "external_id": "animal_456", "state": "unverified" }],
        "match_basis": ["pet_name", "owner_name", "provider_external_id"],
        "match_state": "candidate_match",
        "mismatch_warnings": []
      }
    ],
    "owner_candidates": [
      {
        "candidate_customer_id": "cust_123",
        "extracted_name": { "value": "Alex Example", "state": "suggested", "confidence": { "score": 0.8, "band": "medium" }, "source_refs": [] },
        "contact_redacted": true,
        "match_basis": ["owner_name"],
        "match_state": "candidate_match",
        "mismatch_warnings": []
      }
    ],
    "identity_state": "needs_review",
    "identity_warnings": ["name_only_match_requires_review"]
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
      "aliases": ["rabies", "rabies 3 year", "rabies 3yr"],
      "review_state": "needs_medical_document_review",
      "administered_date": {
        "value": "2025-05-10",
        "state": "suggested",
        "precision": "day",
        "confidence": { "score": 0.94, "band": "high", "reason": "date printed in vaccine row" },
        "source_refs": [{ "evidence_id": "ocr_123", "page": 1, "span_id": "span_rabies_admin" }]
      },
      "expiration_or_due_date": {
        "value": "2028-05-10",
        "date_role": "expiration",
        "state": "suggested",
        "precision": "day",
        "confidence": { "score": 0.93, "band": "high", "reason": "date printed under expires column" },
        "source_refs": [{ "evidence_id": "ocr_123", "page": 1, "span_id": "span_rabies_exp" }]
      },
      "lot_number": { "value": "LOT123", "state": "suggested", "confidence": { "score": 0.6, "band": "medium" }, "source_refs": [] },
      "manufacturer": { "value": null, "state": "unknown", "confidence": { "score": 0.0, "band": "none" }, "source_refs": [] },
      "clinic_or_vet_ref": "source_clinic",
      "source_refs": [{ "evidence_id": "ocr_123", "page": 1, "span_id": "row_rabies" }],
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
      "source_refs": [{ "evidence_id": "ocr_123", "page": 1, "span_id": "row_rabies" }]
    }
  ],
  "ambiguous_fields": [
    {
      "field_path": "vaccine_suggestions[1].expiration_or_due_date",
      "state": "ambiguous",
      "observed_values": ["03/04/26", "04/03/26"],
      "reason": "Date format could be MM/DD/YY or DD/MM/YY.",
      "required_resolution": "MedicalDocumentReview",
      "source_refs": []
    }
  ],
  "mismatch_warnings": [
    {
      "kind": "pet_identity_mismatch",
      "severity": "blocking_review",
      "message": "Document pet name does not exactly match selected pet profile.",
      "related_fields": ["identity.pet_candidates[0].extracted_name"],
      "source_refs": []
    }
  ],
  "contradiction_flags": [
    {
      "kind": "conflicting_vaccine_dates",
      "severity": "blocking_review",
      "message": "Two rabies rows have different expiration dates.",
      "related_suggestion_ids": ["vx_sugg_001", "vx_sugg_002"],
      "source_refs": []
    }
  ],
  "missing_inputs": [
    {
      "field": "policy_snapshot_id",
      "required_source": "approved location vaccine policy",
      "blocks": ["eligibility_decision", "auto_acceptance"]
    }
  ],
  "review_packet": {
    "recommended_review_gate": "MedicalDocumentReview",
    "review_reason_codes": ["medical_document_fact_unaccepted", "source_unverified"],
    "suggested_operator_next_steps": ["verify pet identity", "verify clinic/source", "confirm vaccine date interpretation"],
    "safe_summary": "Extraction suggestions are available for staff review; no vaccine fact is accepted."
  }
}
```

## Reusable field contracts

### Field suggestion

Use this wrapper for extracted scalar fields such as pet name, owner name, clinic name, vaccine label, administered date, expiration date, and document date.

```json
{
  "value": "string, date, enum, number, or null",
  "state": "suggested | unknown | ambiguous | conflicting | unverified | not_applicable",
  "confidence": {
    "score": 0.0,
    "band": "none | low | medium | high",
    "reason": "short evidence-quality explanation"
  },
  "source_refs": []
}
```

Validation rules:

- `unknown`, `not_applicable`, and unresolved `ambiguous` fields may use `value: null`, but must explain why in `ambiguous_fields`, `missing_inputs`, or field-level warnings when the field matters.
- `conflicting` fields must list the competing values in `ambiguous_fields` or `contradiction_flags`.
- `source_refs` are required for any extracted value that affects vaccine review, policy comparison, identity matching, or task/message drafts.

### Date suggestion

Dates are high risk and require explicit precision and role.

```json
{
  "value": "YYYY-MM-DD or null",
  "state": "suggested | unknown | ambiguous | conflicting | unverified",
  "date_role": "administered | expiration | due | document_date | received_at",
  "precision": "day | month | year | range | unknown",
  "raw_observed_text_ref": "redacted_excerpt_or_span_id",
  "confidence": { "score": 0.0, "band": "none | low | medium | high" },
  "source_refs": []
}
```

Date validation rules:

- Never silently normalize ambiguous dates. If the source says `03/04/26` and locale/order is unclear, set `state: "ambiguous"` and list candidate interpretations.
- Future administered dates, expired/stale dates, missing years, date ranges, and multiple conflicting dates require review flags.
- `expiration_or_due_date.date_role` must distinguish a true expiration date from a next-due/reminder date when possible; otherwise use `state: "ambiguous"`.

### Source reference

```json
{
  "evidence_id": "ocr_123",
  "document_id": "doc_123",
  "page": 1,
  "region": { "x": 0.12, "y": 0.31, "width": 0.44, "height": 0.05, "unit": "normalized" },
  "span_id": "span_abc",
  "redacted_excerpt_ref": "excerpt_abc_redacted"
}
```

Rules:

- Prefer immutable evidence ids over filenames or raw URLs.
- Page/region/span refs are optional only when the parser cannot provide them. If unavailable, include an evidence-level ref and note the limitation in `verification`.
- Redacted excerpts may support review UI display; raw OCR text must not be inlined into workflow events/results.

## Enumerations

Recommended enums for validation:

- `document.source`: `customer_upload`, `staff_scan`, `email_ingest`, `provider_poll`, `provider_webhook`, `migration_import`, `unknown`.
- `document.kind`: `vaccine_record`, `immunization_record`, `rabies_certificate`, `vet_invoice_with_vaccines`, `medical_note`, `unknown`, `other`, `unsupported`.
- field `state`: `suggested`, `unknown`, `ambiguous`, `conflicting`, `unverified`, `not_applicable`.
- confidence `band`: `none`, `low`, `medium`, `high`.
- `identity.match_state`: `matched_by_verified_id`, `candidate_match`, `multiple_candidates`, `mismatch`, `unmapped`, `unknown`.
- `source_trust_state`: `licensed_vet_verified`, `clinic_candidate`, `unverified_veterinary_source`, `customer_supplied_only`, `provider_payload_unverified`, `unknown`.
- `vaccine_suggestions.review_state`: `unreviewed`, `needs_more_information`, `needs_medical_document_review`, `accepted_by_staff`, `rejected_or_not_relevant`, `superseded`.
- `policy_comparison_suggestions.comparison_state`: `matched_unverified`, `missing`, `expired_or_stale`, `future_dated`, `conflicting`, `source_unverified`, `not_required_for_service`, `policy_missing`, `not_evaluated`.
- warning/flag severity: `info`, `review`, `blocking_review`, `engineering_review`.

## Risk flag mapping

The workflow envelope `risk_flags` should be populated from structured schema conditions. Suggested mappings:

| Condition | Risk flag |
| --- | --- |
| No confident pet/customer match | `conflicting_pet_identity` or `pet_identity_unmapped` |
| Clinic/vet source missing or unverified | `unverified_veterinary_source` |
| Required policy input missing | `missing_vaccine_policy` |
| Required vaccine not found in suggestions | `missing_required_vaccine_proof` |
| Administered/expiration/due date unclear | `ambiguous_vaccine_date` |
| Suggested expiration is before review/reservation date | `expired_or_stale_vaccine` |
| Multiple rows/documents disagree | `contradictory_vaccine_evidence` |
| Unsupported or unreadable file | `unsupported_document_type` or `unreadable_document` |
| Raw OCR/provider/PII was redacted or quarantined | `raw_pii_redacted` / `provider_payload_unverified` |

## Minimum valid outputs by scenario

### Clean extraction, still review-gated

- `status`: `needs_human_review` unless a future approved policy allows narrower deterministic handling.
- `vaccine_suggestions[*].review_state`: `needs_medical_document_review`.
- Include source refs for pet identity, clinic/source, vaccine name, administered date, and expiration/due date.
- Include `human_review_reason: MedicalDocumentReview required before accepting vaccine facts`.

### Unreadable or unsupported document

- `status`: `blocked` or `failed` depending on whether retry/reupload is appropriate.
- `document.kind`: `unsupported` or `unknown`.
- `vaccine_suggestions`: empty.
- `missing_inputs` names a readable vaccine document or OCR output.
- `risk_flags`: `unsupported_document_type` or `unreadable_document`.

### Identity mismatch or multi-pet document

- Preserve all plausible `pet_candidates` and relevant source refs.
- Set `identity_state: "needs_review"`.
- Add `mismatch_warnings` or `identity_warnings` with `blocking_review` severity.
- Do not attach any vaccine suggestion to a final `pet_id` unless a verified mapping already exists.

### Conflicting vaccine dates

- Keep each source-backed vaccine suggestion or row separate.
- Mark the affected date field `conflicting` or `ambiguous`.
- Add a `contradiction_flags` item referencing the related suggestion ids.
- Policy comparison must be `conflicting` or `not_evaluated`, never accepted.

## Validation invariants

A runtime validator should reject or escalate output when any of these invariants fail:

1. `schema_name`, `schema_version`, `workflow_name`, `event_id`, and subject do not match the prompt packet.
2. `structured_output.document.document_id` or at least one raw attachment/evidence id is missing.
3. Extracted vaccine names/dates that affect review lack `source_refs` without a verification note explaining the parser limitation.
4. A field is ambiguous/conflicting but there is no corresponding `ambiguous_fields`, `mismatch_warnings`, or `contradiction_flags` entry.
5. A policy comparison claims `matched`/`satisfied` semantics without a policy snapshot id and review state.
6. The output implies vaccine acceptance, reservation readiness, provider write-back, or customer send.
7. Raw OCR, raw email text, raw provider JSON, full contact details, or unnecessary PII appears inline in the result.
8. `needs_human_review` or medical-document uncertainty is present without `human_review_reason` / review gate metadata.

## Implementation notes for future Rust types

Suggested module ownership when this moves from docs to code:

- `domain::document` or `domain::vaccine_document`: `DocumentId`, `EvidenceId`, `DocumentKind`, `DocumentSource`, `EvidenceRef`, raw attachment metadata.
- `domain::vaccine`: canonical vaccine names, aliases, extraction suggestions, review state, policy comparison state.
- `domain::identity`: pet/owner candidate match results and mismatch warnings.
- `domain::workflow`: shared result envelope, risk flags, verification records, review reasons, missing/uncertainty records.

Provider-specific labels, OCR vendor payloads, and Gingr immunization response shapes should stay in integration/boundary modules until mapped into these semantic types.
