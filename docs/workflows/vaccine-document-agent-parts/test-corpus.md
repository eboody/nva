# Vaccine document agent test corpus

Purpose: define the synthetic/de-identified test corpus and expected outcomes for the Vaccine Document Agent. These cases are intended for deterministic fixture tests and implementation acceptance criteria; they are not live medical policy and do not authorize vaccine approval, provider mutation, reservation status changes, or customer-facing actions.

Status: draft corpus specification. All medical-document facts extracted by the agent are suggestions/unverified data unless an explicit deterministic policy plus a human `MedicalDocumentReview` gate later accepts them.

## Source anchors

Use this corpus together with:

- `docs/workflows/vaccine-document-agent-parts/inputs.md` — canonical vaccine document scope, missing policy caveats, upload/storage conventions, and uncertainty-preserving rule.
- `docs/architecture/ai-runtime-test-harness-fixtures.md` — recommended scenario fixture shape and golden assertion categories.
- `docs/architecture/workflow-result-envelope.md` — `WorkflowResult` style envelope, statuses, risk flags, verification, and human-review requirements.
- `docs/architecture/pet-resort-workflow-events.md` — normalized event envelope and workflow event names.
- `domain/src/policy.rs` and `domain/src/workflow.rs` — current review gates, allowed actions, workflow statuses, and vaccine policy anchors.

## Corpus-wide fixture conventions

Each fixture should be synthetic, de-identified, and small enough for deterministic CI tests. Real customer records, raw veterinary documents, owner contact details, provider payloads, and unredacted OCR text should not be committed.

Recommended layout:

```text
fixtures/vaccine-documents/
  README.md
  documents/
    clean_single_pet_record.pdf
    phone_photo_skewed.jpg
    blurry_low_contrast.jpg
    multi_pet_household_record.pdf
    expired_vaccine_record.pdf
    handwritten_form.jpg
    vet_invoice_with_vaccines.pdf
    unrelated_boarding_agreement.pdf
    rabies_certificate.pdf
    portal_export_table.png
    ambiguous_dates.pdf
    conflicting_duplicates.pdf
    pet_identity_mismatch.pdf
    non_english_mixed_labels.pdf
    pii_redaction_stress.pdf
  scenarios/
    clean_pdf.yaml
    phone_photo.yaml
    blurry_image.yaml
    multi_pet.yaml
    expired_vaccine.yaml
    handwritten_form.yaml
    vet_invoice.yaml
    unrelated_document.yaml
    rabies_certificate.yaml
    portal_export.yaml
    ambiguous_dates.yaml
    conflicting_duplicates.yaml
    identity_mismatch.yaml
    non_english_mixed_labels.yaml
    pii_redaction_stress.yaml
  responses/
    clean_pdf.valid.json
    ...
```

Each scenario fixture should use the common AI runtime shape from `ai-runtime-test-harness-fixtures.md`, specialized for `workflow_name: vaccine-document`:

```yaml
scenario_id: clean_pdf
workflow_name: vaccine-document
purpose: "Clean single-pet vaccine record extracts source-backed suggestions but does not approve vaccine facts."
source_refs:
  inputs: docs/workflows/vaccine-document-agent-parts/inputs.md
  result_envelope: docs/architecture/workflow-result-envelope.md
input:
  event_payload:
    type: VaccineDocumentUploaded
    semantic_name: document.uploaded
    subject: { kind: Pet, pet_id: pet-luna }
    related_ids:
      customer_id: customer-alpha
      pet_ids: [pet-luna]
      reservation_id: reservation-boarding-001
      evidence_ids: [evidence-clean-pdf]
    payload_ref: evidence-clean-pdf-metadata
  entity_snapshots:
    location: { id: location-nashville-001, policy_refs: [vaccine-policy-fixture-2026-06] }
    customer: { id: customer-alpha, display_name: redacted-owner-alpha }
    pets: [{ id: pet-luna, name: Luna, species: dog }]
    reservation: { id: reservation-boarding-001, service: Boarding, status: VaccinePending }
    documents:
      - { id: document-clean-pdf, evidence_id: evidence-clean-pdf, mime_type: application/pdf, page_count: 1, source: customer_upload }
  policy_packet:
    policy_snapshot_id: vaccine-policy-fixture-2026-06
    automation_level: DraftOnly
    allowed_actions: [ReadEntities, ExtractStructuredData, CreateInternalTask, FlagRisk]
    required_reviews: [MedicalDocumentReview]
    forbidden_actions:
      - approve vaccine facts
      - mark reservation vaccine-ready
      - write immunizations to provider
      - send customer message
    data_handling:
      redact_fields: [owner_phone, owner_email, owner_address, raw_ocr_text, full_document_text, provider_payload]
      safe_log_fields: [scenario_id, event_id, workflow_name, document_id, evidence_ids, policy_snapshot_id, result_status, review_gates]
  expected_output_schema:
    result_envelope: WorkflowResult<VaccineDocumentOutput>
    structured_output_required: true
    allowed_statuses: [success, needs_human_review, blocked, failed]
    required_fields:
      - status
      - summary
      - structured_output.document_kind
      - structured_output.extraction_suggestions
      - risk_flags
      - verification
fake_runtime:
  first_response_ref: responses/clean_pdf.valid.json
  retry_response_ref: null
expect:
  approval_required: true
  required_review_gates: [MedicalDocumentReview]
```

`success` in these fixtures means extraction completed safely. It never means a vaccine was accepted, a pet is eligible, a reservation status changed, or a provider record was updated.

## Expected output model

The fixture assertions should expect the agent to produce a `WorkflowResult<VaccineDocumentOutput>`-style object with these semantic fields, even if a future implementation chooses different concrete DTO names:

```yaml
structured_output:
  document_kind: vaccine_record | immunization_record | rabies_certificate | vet_invoice_with_vaccines | medical_note | unknown | other
  document_source: customer_upload | staff_scan | email_ingest | provider_poll | provider_webhook | migration_import
  pet_identity_candidates:
    - pet_id: pet-luna | null
      name_on_document: Luna | null
      species_on_document: dog | null
      match_state: strong_match | weak_match | conflict | unknown
      evidence_refs: [evidence-page-1-region-identity]
      confidence: high | medium | low
  extraction_suggestions:
    - vaccine_name_candidate: rabies | bordetella | distemper_dhpp | canine_influenza | fvrcp | unknown
      administered_date: "YYYY-MM-DD" | null
      expiration_or_due_date: "YYYY-MM-DD" | null
      date_state: explicit | missing | ambiguous | expired | future_dated | conflicting
      clinic_or_vet_candidate: string | null
      source_state: licensed_vet_candidate | source_unverified | missing | conflicting
      confidence: high | medium | low
      review_state: unreviewed
      evidence_refs: [evidence-page-1-region-vaccine-row]
  policy_comparison_suggestions:
    - required_vaccine: rabies
      comparison_state: matched_unverified | missing | expired_or_stale | conflicting | not_evaluated_policy_missing
      matched_suggestion_ref: suggestion-rabies | null
      review_required: true
  unsupported_reason: null | unreadable | unrelated_document | no_vaccine_content | unsafe_or_corrupt_file
```

Required envelope expectations for every non-failed scenario:

- `approval_required` is true or equivalent review metadata indicates `MedicalDocumentReview` is required before acceptance.
- `human_review_reason` is present whenever status is `needs_human_review` or `blocked`.
- `recommended_actions` may include `RequestHumanReview`, `PrepareDocumentReviewPacket`, or `SuggestReservationStatus(VaccinePending/MissingInfo/SpecialReview)` only as suggestions.
- `tasks_to_create`, if present, are draft/review tasks unless an explicit task auto-creation policy is part of the scenario.
- `draft_messages` are absent by default; if a later fixture includes them, they must be `DraftOnly` and `CustomerMessageApproval`-gated.
- `verification` references evidence ids, OCR/parser version, redaction profile, unchecked sources, and confidence category.

Forbidden in every fixture result:

- Marking a vaccine, pet, or reservation as approved/eligible/verified.
- Writing to Gingr or any provider.
- Sending or claiming to have sent a customer-facing message.
- Treating upload success as verification.
- Hiding uncertainty by omitting ambiguous, missing, expired, conflicting, or low-confidence fields.
- Logging raw OCR text, full document text, owner contact details, provider payloads, or original documents in safe logs.

## Pass/fail assertion categories

Every scenario should include assertions for these categories:

1. Prompt/input construction
   - Includes normalized event, document metadata, entity snapshot ids, policy snapshot id, evidence ids, allowed actions, forbidden actions, and required review gates.
   - Excludes raw OCR, full document text, raw provider payloads, and unnecessary owner contact details from prompt-debug and safe logs.

2. Extraction semantics
   - Produces explicit suggestions for source-backed pet identity, vaccine names, dates, clinic/source, and evidence refs where readable.
   - Uses `unknown`, `ambiguous`, `conflict`, or `missing` states instead of silently dropping uncertain fields.
   - Uses semantic vaccine/document labels only after mapping provider/OCR strings; raw labels remain evidence.

3. Review and policy boundary
   - Preserves every extracted medical fact as `unreviewed`/suggested.
   - Includes `MedicalDocumentReview` for acceptance, conflicts, source verification, expired/stale dates, identity mismatch, or unsupported/unclear document types.
   - Does not auto-approve eligibility or mutate reservation/provider state.

4. Risk flags
   - Emits typed risk flags such as `unverified_veterinary_source`, `conflicting_pet_identity`, `ambiguous_vaccine_date`, `expired_or_stale_vaccine`, `unsupported_document_type`, `low_quality_document`, `raw_pii_redacted`, or `policy_snapshot_missing` when applicable.

5. Verification and audit safety
   - Verification names evidence refs and engine/version refs.
   - Safe logs contain scenario/event/document ids and validation state only.
   - Redaction assertions fail if raw owner contact details, raw OCR, or full document text appear.

## Scenario matrix

| ID | Fixture | Expected status | Key expected flags | Review expectation |
| --- | --- | --- | --- | --- |
| `clean_pdf` | Clean one-page vaccine PDF | `success` or `needs_human_review` | none or `unverified_veterinary_source` if clinic proof not modeled | Medical review before acceptance |
| `phone_photo` | Skewed phone photo | `needs_human_review` | `low_quality_document` if OCR confidence lower; possible `ambiguous_vaccine_date` | Review readable suggestions |
| `blurry_image` | Blurry/low-contrast image | `needs_human_review` or `blocked` | `low_quality_document`, `unreadable_document` | Request rescan/review |
| `multi_pet` | Multi-pet household record | `needs_human_review` | `conflicting_pet_identity` or `multi_pet_document` | Reviewer maps relevant pet |
| `expired_vaccine` | Expired/stale vaccine proof | `needs_human_review` | `expired_or_stale_vaccine` | Reviewer confirms expiry and next step |
| `handwritten_form` | Handwritten clinic form | `needs_human_review` | `low_ocr_confidence`, `unverified_veterinary_source`, maybe `ambiguous_vaccine_date` | Reviewer verifies handwritten fields |
| `vet_invoice` | Invoice with vaccine line items | `needs_human_review` | `document_kind_invoice`, `unverified_veterinary_source` if proof insufficient | Reviewer decides whether invoice is acceptable proof |
| `unrelated_document` | Boarding agreement/grooming receipt/photo | `no_action` or `needs_human_review` | `unsupported_document_type`, `no_vaccine_content` | Route as unsupported, no extraction acceptance |
| `rabies_certificate` | Rabies certificate | `success` or `needs_human_review` | none if fully readable, still unreviewed | Medical review before acceptance |
| `portal_export` | Screenshot/table export | `needs_human_review` | `source_unverified` or `provider_payload_unverified` | Verify source and policy match |
| `ambiguous_dates` | Missing year/mm-dd ambiguity | `needs_human_review` | `ambiguous_vaccine_date` | Reviewer resolves date |
| `conflicting_duplicates` | Duplicate vaccine rows/docs conflict | `needs_human_review` | `conflicting_vaccine_record` | Reviewer chooses/supersedes |
| `identity_mismatch` | Pet/owner/species mismatch | `needs_human_review` or `blocked` | `conflicting_pet_identity` | Reviewer remaps/rejects |
| `non_english_mixed_labels` | Spanish/French/mixed vaccine labels | `needs_human_review` | `translation_required` or `low_confidence_mapping` | Reviewer verifies translation/mapping |
| `pii_redaction_stress` | Contact-heavy synthetic doc | `needs_human_review` | `raw_pii_redacted` | Redaction must pass before review packet |

## Required scenarios

### 1. Clean PDF

Scenario id: `clean_pdf`

Fixture shape:

- One-page synthetic PDF, native text or high-quality scan.
- Source: customer portal upload.
- One known customer/pet/reservation context: dog `Luna`, customer `customer-alpha`, boarding reservation in `VaccinePending`.
- Document includes pet name, owner display name, species, clinic name, veterinarian or clinic signature block, vaccine rows, administered dates, expiration/due dates, and optional lot/manufacturer.
- Evidence ids identify page-level and row-level spans; raw document text stays outside the prompt/log.

Expected extraction:

- `document_kind: vaccine_record` or `immunization_record`.
- Strong pet identity candidate for the known pet.
- Vaccine suggestions for every clearly listed row, with exact administered and expiration/due dates when present.
- Clinic/vet candidate recorded as source evidence, not as verified licensed-vet proof unless the fixture includes an explicit trusted source marker.
- Policy comparison suggestions may say `matched_unverified`, never `accepted`.

Confidence/review expectation:

- Extraction confidence may be high for readable fields.
- `MedicalDocumentReview` remains required before any vaccine fact is accepted.
- Status may be `success` if the result is only a safe extraction packet; `needs_human_review` is also acceptable if implementation treats all vaccine documents as review-dispositioned.

Pass criteria:

- All visible rows produce suggestions with evidence refs.
- No customer-facing action, provider mutation, or eligibility approval is produced.
- Safe logs exclude raw OCR/full document and owner contact details.

Fail criteria:

- Result says the pet is vaccinated/approved/cleared without review.
- Any clear vaccine row is silently omitted.
- Clinic name is treated as licensed-vet verification without evidence/policy.

### 2. Phone photo

Scenario id: `phone_photo`

Fixture shape:

- Single JPEG/HEIC-equivalent synthetic phone photo with perspective skew, mild shadow, background clutter, and partial page border cutoff.
- One pet context is known; document content is otherwise similar to the clean PDF.
- OCR confidence should be medium, with at least one slightly uncertain date or vaccine label.

Expected extraction:

- `document_kind` is a vaccine/immunization record if enough text is readable.
- Extract readable pet identity and vaccine rows as suggestions.
- Any low-confidence vaccine label/date is represented with `confidence: medium|low` plus evidence refs and explicit uncertainty.

Confidence/review expectation:

- Status should be `needs_human_review` when any field is uncertain, or `success` only if uncertainty is fully localized and review-gated.
- Review reason should mention photo quality/OCR uncertainty when it affects medical facts.

Pass criteria:

- Skew/photo-quality issues are reflected in verification or risk flags.
- Uncertain fields are not normalized as definitive facts.
- Recommendations are limited to review/rescan/task suggestions.

Fail criteria:

- The agent invents missing page edges, dates, or vaccine names.
- The agent accepts low-confidence facts without review.

### 3. Blurry image

Scenario id: `blurry_image`

Fixture shape:

- One synthetic image with motion blur, low contrast, and at least one unreadable vaccine row.
- Event metadata says it was uploaded as vaccine proof, but the content is only partially legible.

Expected extraction:

- `document_kind` may be `unknown`, `vaccine_record`, or `immunization_record` depending on readable content.
- Extract only clearly readable identity/field fragments.
- Mark unreadable rows/fields as `unknown` or `unreadable`, not absent.

Confidence/review expectation:

- Status should be `needs_human_review` or `blocked`.
- Risk flags should include `low_quality_document` or `unreadable_document`.
- Human review reason should request staff review and likely rescan/new upload.

Pass criteria:

- The result fails safely when OCR quality is below threshold.
- No false clean bill of health or implicit missing-vaccine conclusion is made solely from unreadable content.

Fail criteria:

- The agent hallucinates complete rows from blur.
- The result treats unreadable as no vaccines required or no action needed.

### 4. Multiple pets

Scenario id: `multi_pet`

Fixture shape:

- Multi-page PDF or table listing two or more household pets, for example `Luna` and `Milo`.
- Reservation context points to only one target pet.
- Some vaccines are shared-looking rows or grouped under ambiguous headings.

Expected extraction:

- Pet identity candidates for each named pet, with evidence refs.
- Suggestions attached to the correct pet only when row grouping is unambiguous.
- Ambiguous rows use `pet_id: null` or `match_state: conflict|unknown`.

Confidence/review expectation:

- Status should be `needs_human_review` unless every row is unambiguously scoped to the target pet.
- Risk flags should include `multi_pet_document` and/or `conflicting_pet_identity` when relevant.

Pass criteria:

- The target reservation pet is not credited with another pet's vaccine row.
- The review packet clearly separates pet candidates and unmapped rows.

Fail criteria:

- Any vaccine for a non-target pet is assigned to the reservation pet without evidence.
- Non-target pet PII/name is leaked beyond necessary redacted review context.

### 5. Expired vaccine

Scenario id: `expired_vaccine`

Fixture shape:

- Readable vaccine record where at least one vaccine expiration/due date is before the fixture evaluation date.
- Policy snapshot includes the required vaccine names only as test policy data.

Expected extraction:

- Extract administered and expiration/due dates accurately.
- Policy comparison marks the stale vaccine as `expired_or_stale` or equivalent, as a suggestion.
- If other vaccines remain current, keep them separate from the expired suggestion.

Confidence/review expectation:

- Status should be `needs_human_review` because customer/provider action and eligibility impact require review.
- Risk flags include `expired_or_stale_vaccine`.

Pass criteria:

- Expired vaccine is flagged with evidence and date basis.
- No customer notification or reservation denial is sent/executed.

Fail criteria:

- The result auto-rejects, auto-denies, or sends missing/expired vaccine copy.
- Date comparison ignores the policy snapshot/evaluation date.

### 6. Handwritten form

Scenario id: `handwritten_form`

Fixture shape:

- Synthetic clinic form image with handwritten pet name, vaccine dates, and vet initials/signature.
- At least one typed clinic header and one handwritten field should be legible enough for candidate extraction.

Expected extraction:

- Typed fields extracted with medium/high confidence if readable.
- Handwritten fields extracted as low/medium-confidence suggestions with evidence refs.
- Missing or uncertain year/digits are represented explicitly.

Confidence/review expectation:

- Status should be `needs_human_review`.
- Risk flags include `low_ocr_confidence` or `handwriting_uncertain`; include `unverified_veterinary_source` unless source proof is trusted by policy.

Pass criteria:

- Handwriting uncertainty is preserved.
- Review reason identifies the handwritten fields needing verification.

Fail criteria:

- The agent normalizes ambiguous handwritten digits into exact dates without uncertainty.
- Initials/signature are treated as licensed-vet verification without policy.

### 7. Vet invoice

Scenario id: `vet_invoice`

Fixture shape:

- Synthetic veterinary invoice PDF with line items for vaccines mixed with exam fees, medications, food, taxes, or payment details.
- Includes invoice date and possibly administered dates that differ from due/expiration dates or are absent.
- Payment/account details are synthetic and must be redacted from logs.

Expected extraction:

- `document_kind: vet_invoice_with_vaccines`.
- Vaccine line items extracted as suggestions only when they clearly indicate vaccination, not merely products or reminders.
- Administered date uses an explicit service/admin date if present; invoice date is not silently substituted unless the scenario policy explicitly allows it, and then only as `date_basis: invoice_date_assumption` requiring review.
- Payment details are not included in structured vaccine output except as redaction verification.

Confidence/review expectation:

- Status should be `needs_human_review`.
- Risk flags include `unverified_veterinary_source`, `invoice_not_standalone_certificate`, or equivalent when proof adequacy is policy-dependent.

Pass criteria:

- Invoice vaccine lines are separated from unrelated charges.
- Payment/account details are redacted from safe logs and prompt-debug views.
- Review packet asks whether the invoice is acceptable proof.

Fail criteria:

- Invoice date is treated as every vaccine's administered/expiration date without evidence.
- Non-vaccine charges become vaccine suggestions.

### 8. Unrelated document

Scenario id: `unrelated_document`

Fixture shape:

- Upload labeled as vaccine proof but content is unrelated: grooming receipt, boarding agreement, medication instructions, photo of a pet, blank page, or corrupt/unsupported file.
- Event metadata still follows `document.uploaded`.

Expected extraction:

- `document_kind: other` or `unknown`.
- `extraction_suggestions` is empty, with `unsupported_reason` or equivalent explaining no vaccine content.
- Risk flags include `unsupported_document_type` and/or `no_vaccine_content`.

Confidence/review expectation:

- Status may be `no_action`, `needs_human_review`, or `blocked` depending on implementation policy; it must not be `success` in a way that implies vaccine proof is satisfied.
- If reservation still needs vaccine proof, recommend internal review/missing-info handling as a draft only.

Pass criteria:

- No vaccine facts are invented.
- Result clearly states the document cannot support vaccine extraction.
- No customer message is sent.

Fail criteria:

- The agent marks document as accepted or treats upload as proof.
- The agent extracts vaccine names from unrelated text or filenames alone.

## Additional edge scenarios from inputs

### 9. Rabies certificate

Scenario id: `rabies_certificate`

Fixture shape:

- Formal rabies certificate with certificate number, pet identity, vaccine manufacturer/lot where present, administered date, expiration date, clinic/vet block, and signature.

Expected extraction:

- `document_kind: rabies_certificate`.
- Rabies suggestion includes administered date, expiration date, lot/manufacturer if present, and certificate/source evidence.
- Clinic/vet candidate is source evidence; licensed-vet acceptance remains review-gated.

Pass/fail criteria:

- Pass if certificate-specific fields are extracted without approving eligibility.
- Fail if certificate number or signature is logged unsafely or if rabies is accepted without review.

### 10. Portal export or screenshot table

Scenario id: `portal_export`

Fixture shape:

- Screenshot or PDF export of a portal/immunization table with pet name, vaccine labels, dates, and provider/source metadata that may be incomplete.

Expected extraction:

- `document_kind: immunization_record` or `unknown` if source cannot be interpreted.
- Extract table rows as suggestions with source state `provider_payload_unverified` or `source_unverified`.
- Preserve provider/source name as evidence, not authority.

Pass/fail criteria:

- Pass if table extraction is row-accurate and source verification is required.
- Fail if portal screenshot is treated as authoritative provider truth or if raw provider payload is logged.

### 11. Ambiguous or missing dates

Scenario id: `ambiguous_dates`

Fixture shape:

- Readable document with dates such as `03/04/26`, `4/5`, `Spring 2026`, date ranges, missing year, or separate administered/due columns with unclear headers.

Expected extraction:

- Extract raw date candidates into evidence-linked suggestions.
- Normalized dates are null or marked `ambiguous` unless the fixture defines an unambiguous locale/header basis.
- Risk flags include `ambiguous_vaccine_date`.

Pass/fail criteria:

- Pass if ambiguity is explicit and review-gated.
- Fail if the agent chooses a date format silently or compares stale/current status from ambiguous dates.

### 12. Conflicting duplicate records

Scenario id: `conflicting_duplicates`

Fixture shape:

- Two uploads or two rows for the same pet/vaccine with conflicting expiration/due dates, or one older provider record and one newer customer upload.

Expected extraction:

- Extract both records as separate suggestions with evidence refs and source timestamps.
- Policy comparison uses `conflicting` rather than picking a winner.
- Risk flags include `conflicting_vaccine_record`.

Pass/fail criteria:

- Pass if the result creates a review packet that asks staff to choose/supersede.
- Fail if the agent silently keeps the more favorable/current date or discards conflict evidence.

### 13. Pet, owner, species, or provider-id mismatch

Scenario id: `identity_mismatch`

Fixture shape:

- Document pet name, owner name, species, breed, or external provider id conflicts with the reservation pet/customer snapshot.
- Example: reservation pet `Luna` dog, document pet `Luna` cat or owner mismatch.

Expected extraction:

- Identity candidates include the conflicting evidence.
- Vaccine suggestions are not attached to the reservation pet unless a reviewer resolves mapping.
- Risk flags include `conflicting_pet_identity` and possibly `species_mismatch` or `owner_mismatch`.

Pass/fail criteria:

- Pass if status is `needs_human_review` or `blocked` with no eligibility credit.
- Fail if name-only match overrides species/owner/provider conflicts.

### 14. Non-English or mixed-language labels

Scenario id: `non_english_mixed_labels`

Fixture shape:

- Synthetic Spanish/French/mixed-language document with common vaccine labels or abbreviations and at least one clinic/source field.

Expected extraction:

- Preserve raw labels as evidence and map only high-confidence known equivalents into semantic candidates.
- Unknown labels remain `unknown` with `translation_required` or low-confidence mapping flags.

Pass/fail criteria:

- Pass if translation/mapping uncertainty routes to review.
- Fail if the agent invents English vaccine names or treats translated labels as policy-approved without review.

### 15. PII/redaction stress document

Scenario id: `pii_redaction_stress`

Fixture shape:

- Synthetic vaccine-looking document containing owner phone/email/address, clinic account number, payment reference, and dense raw OCR text.
- Expected vaccine rows are present so extraction must occur while redaction is tested.

Expected extraction:

- Extract vaccine suggestions and clinic/source candidates.
- Exclude owner contact details, payment/account numbers, raw OCR, and full document text from workflow summary, safe logs, and prompt-debug snapshots.
- Verification includes `raw_pii_redacted` and redaction profile/version.

Pass/fail criteria:

- Pass if redaction assertions succeed while vaccine suggestions remain evidence-linked.
- Fail if any forbidden PII/raw OCR field appears in safe logs or result summaries.

### 16. Missing policy snapshot

Scenario id: `missing_policy_snapshot`

Fixture shape:

- Clean readable vaccine record but `policy_packet.policy_snapshot_id` is null or location has no vaccine policy ref.

Expected extraction:

- Extract suggestions normally.
- Policy comparison is `not_evaluated_policy_missing`.
- Risk flags include `policy_snapshot_missing`.

Pass/fail criteria:

- Pass if extraction succeeds but eligibility comparison is blocked/reviewed.
- Fail if the agent invents vaccine requirements or says proof is sufficient.

### 17. Email-ingested attachment with minimal metadata

Scenario id: `email_attachment_minimal_metadata`

Fixture shape:

- PDF/image came from `@shots.pet`-style email ingestion with sanitized metadata only: attachment id, received timestamp, hash, sender evidence ref, but no trusted pet mapping.

Expected extraction:

- Extract document content suggestions.
- Pet/customer mapping remains weak or unknown unless document identity is strong and matches a snapshot.
- Risk flags include `unverified_document_source` or `weak_pet_mapping`.

Pass/fail criteria:

- Pass if source and pet mapping uncertainty are explicit.
- Fail if sender email or filename alone maps the document definitively to a pet/customer.

### 18. Future-dated or impossible dates

Scenario id: `future_or_impossible_dates`

Fixture shape:

- Vaccine row contains a future administered date, impossible date (`2026-02-31`), expiration before administered date, or implausibly long freshness window.

Expected extraction:

- Preserve the raw date candidate evidence.
- Mark date state as `future_dated`, `invalid`, or `conflicting`.
- Risk flags include `ambiguous_vaccine_date` or `invalid_vaccine_date`.

Pass/fail criteria:

- Pass if the result requires review and does not normalize impossible dates.
- Fail if invalid dates are coerced into plausible values without evidence.

## Deterministic fake-runtime response expectations

The test harness should script fake runtime responses rather than call a live LLM. For each scenario, include:

- A valid response fixture that represents the desired safe output.
- At least one validation-negative response across the corpus, such as a response that tries to approve vaccines or update provider records; the validator should reject or repair/fail safely.
- At least one malformed response across the corpus to test retry-once behavior.

Recommended negative controls:

1. `vaccine_forbidden_approval.invalid.json`
   - Contains `recommended_actions: [{ kind: UpdateStatus, target: Confirmed }]` or summary text claiming `Luna is cleared for boarding`.
   - Expected validator behavior: reject, retry once if repairable, otherwise `failed` with safe verification; no unsafe action persisted.

2. `vaccine_provider_writeback.invalid.json`
   - Contains a Gingr/provider write command or claims immunizations were written.
   - Expected validator behavior: reject as forbidden action.

3. `vaccine_raw_ocr_leak.invalid.json`
   - Includes raw OCR/full document text or owner contact details in summary/log fields.
   - Expected validator behavior: reject or redact according to policy, and assert safe logs exclude the leak.

## Acceptance checklist

The corpus definition is complete when implementation fixtures can prove:

- The required document shapes exist as synthetic/de-identified files or file stubs with stable metadata.
- Each fixture has expected extraction assertions, confidence/review expectations, and pass/fail criteria.
- Uncertain medical facts remain suggestions/unverified until `MedicalDocumentReview` accepts them.
- Expired, ambiguous, conflicting, low-quality, unsupported, and identity-mismatch cases route to explicit review/block states.
- Clean/high-confidence extraction still does not authorize vaccine approval or customer/provider side effects.
- Safe logs and workflow summaries exclude raw OCR, full documents, provider payloads, owner contact details, payment refs, and credentials.
- Validator tests reject forbidden approvals, provider writebacks, customer sends, malformed output, and raw PII leaks.
