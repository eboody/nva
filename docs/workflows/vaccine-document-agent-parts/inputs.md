# Vaccine document agent inputs

Purpose: collect the canonical inputs that downstream cards should use to synthesize `docs/workflows/vaccine-document-agent.md`. This is an input packet, not live medical, customer-service, or provider-integration policy. Any field extracted from a medical document must remain a suggestion or unverified fact until an approved human/policy gate accepts it.

Status: draft input collection for vaccine-document-agent workflow design. Do not use this document to approve vaccines, determine medical eligibility, mutate provider records, send customer messages, or perform customer-facing/live actions in production.

## Source index

Primary sources checked:

- `docs/architecture/pet-resort-workflow-events.md` — canonical event envelope and event catalog, including `document.uploaded`, `vaccine.extraction_needed`, `booking.triage_needed`, and the rule that raw provider payloads/OCR remain boundary evidence referenced by ids.
- `docs/architecture/workflow-result-envelope.md` — proposed result envelope with structured output, recommended actions, draft messages, task drafts, risk flags, verification records, and human-review reasons.
- `docs/workflows/staff-operations-parts/inputs.md` — product/MVP scope, staff roles, task model, approval gates, and conservative downstream rule for missing/uncertain source facts.
- `domain/src/entities.rs` — current domain anchors for `LocationPolicyRefs.vaccine_policy_id`, `Pet`, `CareProfile`, `ReservationStatus::VaccinePending`, `HardStop::MissingRequiredVaccine`, `HardStop::MedicalOrMedicationReviewRequired`, `AuditEvent`, `AuditSubject`, `AuditAction`, and `ActorRef`.
- `domain/src/policy.rs` — current policy anchors for `VaccineRequirement`, `VaccineName`, `ReviewGate::MedicalDocumentReview`, `PolicyDenialReason::MedicalDocumentReviewRequired`, and automation levels.
- `domain/src/workflow.rs` — current Rust workflow names: `WorkflowEventType::VaccineDocumentUploaded`, `AllowedAction::{ReadEntities, ExtractStructuredData, CreateInternalTask, FlagRisk}`, `WorkflowResult`, `WorkflowStatus`, and `RecommendedAction::RequestHumanReview`.
- `domain/src/agent.rs` and `domain/src/tools.rs` — AI runtime/tool boundaries: agent prompt packets carry default review gates and forbidden actions; tools expose structured workflow execution and draft/provider-facing capabilities that must stay approval-gated.
- `integrations/gingr/src/endpoint/reference_data.rs` and `docs/integrations/gingr/sdk-endpoint-catalog.md` — read-only Gingr endpoints for `get_immunization_types` and `get_animal_immunizations`; raw provider responses are incomplete-schema and must be quarantined before mapping into domain concepts.
- `docs/integrations/gingr/articles/26757720199309-upload-vaccination-records-how-to.md` — customer portal upload flow: scanned PDF/image drag-drop, manual file selection, or designated `@shots.pet` email; upload success does not equal vaccine verification.
- `docs/integrations/gingr/sdk-readiness-review.md` and `docs/integrations/gingr/sdk-architecture.md` — read-only provider boundary, API key redaction, no live Gingr calls, and no side-effect endpoints in SDK v0.

Missing or caveated sources:

- `docs/product/pet-resort-product-map.md` is not present in the repo. Use `docs/workflows/staff-operations-parts/inputs.md` and the data-model/domain artifacts as current source of product scope.
- No final, location-approved vaccine policy artifact was found. The domain has `VaccineRequirement` and `LocationPolicyRefs.vaccine_policy_id`, but no concrete vaccine names, duration/freshness windows, grace periods, or service-specific requirements.
- No document/blob storage implementation was found beyond generic storage codec infrastructure. Treat upload/storage conventions below as safe defaults until an implementation card approves concrete storage.
- No sample vaccine-document corpus was found in the repo. Test-corpus needs below are proposed, synthetic/de-identified defaults.
- No OCR/parser/LLM extraction implementation was found. Downstream cards should design an extraction contract rather than assuming a specific OCR vendor/model.

## Product and workflow scope inputs

Use this workflow as an internal intake/review assistant for pet vaccine or immunization documents. In MVP scope it may:

- receive or reference uploaded vaccine/immunization documents from customer portal, staff scans, provider imports/polls, or designated email ingestion;
- identify whether a document likely contains vaccine proof and whether extraction is needed;
- extract structured vaccine suggestions with evidence spans/pages/images when possible;
- compare extracted suggestions against a location-scoped policy snapshot when one exists;
- produce internal review packets, risk flags, and staff task drafts;
- suggest reservation/pet-profile readiness statuses such as `VaccinePending`, `MissingInfo`, or `SpecialReview`;
- draft customer follow-up copy only as review-gated draft text.

Out of scope for this document and for MVP automation unless later explicitly approved:

- approving vaccines or medical eligibility without human review;
- asserting licensed-veterinarian source verification when the source is ambiguous;
- writing back immunization records to Gingr or any provider;
- changing reservation status in a live provider system;
- sending customer-facing messages;
- making medical advice, diagnosis, or care decisions;
- handling real customer uploads outside controlled/test workflows.

## Data-model inputs to preserve

Existing anchors:

- Identity: `LocationId`, `CustomerId`, `PetId`, `ReservationId`, and provider external ids.
- Policy: `LocationPolicyRefs.vaccine_policy_id`, `policy::VaccineRequirement { species, service, vaccines, source_must_be_licensed_vet }`, `policy::VaccineName`, `policy::ReviewGate::MedicalDocumentReview`, `policy::AutomationLevel`.
- Pet/reservation: `Pet { species, care_profile, temperament }`, `Reservation { service, status, hard_stops }`, `ReservationStatus::VaccinePending`, `ReservationStatus::MissingInfo`, `ReservationStatus::SpecialReview`, and `HardStop::MissingRequiredVaccine(policy::VaccineName)`.
- Workflow: `WorkflowEventType::VaccineDocumentUploaded`, architecture event names `document.uploaded` and `vaccine.extraction_needed`, `AllowedAction::ExtractStructuredData`, `AllowedAction::CreateInternalTask`, `AllowedAction::FlagRisk`.
- Review/tasking: `StaffTaskKind::DocumentReview { pet_id }`, `StaffTaskStatus::{Open, Blocked, NeedsManagerReview}`, `StaffTaskPriority`, and `ReviewGate::MedicalDocumentReview`.
- Audit: `AuditEvent`, `AuditSubject::{Pet, Reservation, WorkflowEvent, External}`, `AuditAction::{PetProfileUpdated, ReservationStatusSuggested, PolicyDecisionRecorded, WorkflowEventRecorded, Extension}`.

Proposed vaccine-document concepts downstream cards may need to add or model explicitly:

- `DocumentId` / `EvidenceId` for raw upload/OCR/provider artifacts.
- `DocumentKind`: vaccine_record, immunization_record, rabies_certificate, vet_invoice_with_vaccines, medical_note, unknown, other.
- `DocumentSource`: customer_upload, staff_scan, email_ingest, provider_poll, provider_webhook, migration_import.
- `VaccineExtractionSuggestion`: pet identity candidates, vaccine name, administered date, expiration/due date, veterinarian/clinic/source candidate, lot/manufacturer if present, confidence, evidence refs, parser/OCR version.
- `VaccineReviewState`: unreviewed, needs_more_information, needs_medical_document_review, accepted_by_staff, rejected_or_not_relevant, superseded.
- `PolicyComparison`: required vaccine, matched suggestion, missing, expired/stale, future-dated, conflicting, source_unverified, not_required_for_service.
- `DocumentRetentionPolicyRef` or storage-class reference for privacy/retention controls.

Keep provider-specific names and payload shapes quarantined until mapped into semantic domain values. Do not branch product behavior on raw provider strings, OCR text, filenames, or LLM confidence alone.

## Required vaccine policy assumptions

Because no approved policy artifact is present, use the following as assumptions only:

1. Location-scoped policy is required.
   - Assumption: every vaccine decision references a `vaccine_policy_id` and policy snapshot/version.
   - Safe default if missing: `blocked` or `needs_human_review`, not approved.

2. Species and service matter.
   - Assumption: dog boarding/day play/day boarding and cat boarding may have different required vaccine sets; grooming/training/DaySpa requirements may vary by location and service package.
   - Safe default if unknown: extract document data but do not evaluate eligibility.

3. Licensed veterinarian/source proof may be required.
   - Current model supports `source_must_be_licensed_vet`.
   - Safe default if source is unclear: flag `unverified_veterinary_source` and route to `MedicalDocumentReview`.

4. Common vaccine names are not canonical policy.
   - Examples likely relevant in pet-resort contexts include rabies, bordetella, distemper/DHPP/DA2PP, canine influenza, FVRCP, and feline rabies, but these are not approved requirements here.
   - Safe default: downstream docs may mention them as example corpus labels only, never as live policy.

5. Dates are high-risk.
   - The agent may suggest administered, expiration, due, or next-due dates only when backed by evidence.
   - Missing year, ambiguous date format, conflicting dates, stale docs, future-dated entries, or OCR-low-confidence dates require review.

6. Pet identity matching is required.
   - The document must be mapped to the correct customer/pet before it can support a reservation/pet profile.
   - Name-only matches, multi-pet documents, renamed pets, mismatched species/breed/age, or external provider id conflicts require review.

7. Upload success is not verification.
   - Gingr customer portal upload supports scanned PDFs/images and `@shots.pet` email ingestion, but source docs only show successful submission, not medical verification.
   - Safe default: `document.uploaded` may create extraction/review work, never acceptance.

## Event and result inputs

Canonical inbound event:

- Architecture name: `document.uploaded`.
- Current Rust enum: `WorkflowEventType::VaccineDocumentUploaded` for vaccine-specific uploads.
- Subject: pet when mapped; external document when not mapped.
- Related ids: document/evidence id, customer id, pet id, reservation id if relevant, provider document id, storage key, OCR job id, policy snapshot id.
- Payload should include only typed metadata: document kind hint, uploader/source, file metadata, expected content hint, OCR/extraction status ref, source timestamp. Raw provider JSON, raw OCR, and full document text stay in boundary/evidence storage.

Canonical derived event:

- Architecture name: `vaccine.extraction_needed`.
- Trigger: document uploaded or provider/policy state indicates vaccine proof needs extraction or structured review.
- Allowed actions: `ReadEntities`, `ExtractStructuredData`, `CreateInternalTask`, `FlagRisk`; customer follow-up only as a separate review-gated draft if downstream policy includes it.
- Expected result: extraction request/summary, document-review task draft, structured vaccine suggestions, and status `Completed`, `NeedsMoreInformation`, or `NeedsHumanReview` as recommendation only.

Result envelope requirements:

- `structured_output` should carry extraction suggestions and policy-comparison suggestions, not approvals.
- `risk_flags` should include typed flags such as `missing_required_vaccine_proof`, `unverified_veterinary_source`, `conflicting_pet_identity`, `ambiguous_vaccine_date`, `expired_or_stale_vaccine`, `unsupported_document_type`, `raw_pii_redacted`, or `provider_payload_unverified`.
- `verification` must list evidence ids, OCR/parser versions when known, unchecked sources, redactions, and confidence category.
- `human_review_reason` is required whenever status is `needs_human_review` or `blocked`.
- A `Completed`/`success` result means the workflow produced safe extraction/review output, not that a vaccine is accepted.

## Upload and storage conventions

Safe defaults until a storage implementation is approved:

- Store raw uploads in private, access-controlled object/blob storage; never inline raw PDF/image bytes or raw OCR text into workflow events.
- Use immutable evidence ids and content hashes for dedupe; keep original filename as optional metadata only after sanitization.
- Maintain a separate normalized metadata record: document id, source, uploader actor, received timestamp, MIME type, size, page/image count, hash, storage key, pet/customer/reservation mapping state, retention class, and redaction status.
- Store OCR/extraction outputs as derived artifacts linked to the raw evidence id and extraction engine/version.
- Redact or quarantine full documents, raw OCR, provider payloads, API keys, owner contact details, and payment/credential-like fields from logs and workflow summaries.
- Reference provider data via external provider/id refs and ingestion evidence ids; do not persist raw Gingr API responses in domain events.
- Treat `report_card_files` and immunization endpoints as read-only provider lookups. They may inform review packets but should not be considered authoritative without source verification and human review.

## Document examples and test-corpus needs

No real corpus is present. Downstream implementation/testing should create a synthetic/de-identified corpus that covers:

- Clean single-pet vaccine record PDF with explicit pet name, owner name, clinic, vaccine names, administered dates, and expiration/due dates.
- Scanned image/photo with skew, blur, low contrast, or handwriting-like fields.
- Multi-pet household record where only one pet is relevant to the reservation.
- Rabies certificate with clinic/vet identity and expiration date.
- Vet invoice that mentions vaccines among unrelated charges.
- Screenshot/portal export with vaccine table.
- Email-ingested attachment with minimal metadata.
- Document with missing year, ambiguous date format, or date ranges.
- Expired/stale vaccine proof.
- Conflicting duplicate records for the same vaccine.
- Pet-name mismatch, owner-name mismatch, species mismatch, and provider-id mismatch cases.
- Unsupported document: grooming receipt, boarding agreement, medication instructions, photo of pet, or unreadable file.
- Non-English or mixed-language labels if the product expects them.
- Synthetic PII/redaction fixture to verify logs/results do not leak raw owner contact details.

Testing should assert that uncertain cases produce explicit unknown/conflict/review states rather than silently omitting fields or marking readiness.

## Human review and approval constraints

Preserve these gates in downstream design:

- Medical/vaccine document review is required for accepting vaccine facts, resolving conflicts, or treating a source as licensed-vet proof.
- Customer messages about missing/expired/rejected vaccines are drafts requiring approval unless a later policy approves a narrow deterministic template path.
- Reservation status changes such as moving into/out of `VaccinePending`, `MissingInfo`, `SpecialReview`, `Confirmed`, or `Rejected` are suggestions until approved and executed by a bounded adapter/tool.
- Provider write-back to Gingr or any PMS is out of scope for MVP unless a separate reviewed side-effect design exists.
- Automation may draft internal tasks, but task auto-creation policy must define trigger, priority, assignee/role, source evidence, dedupe, and review requirements before production use.
- AI confidence is not approval. Low, medium, or high confidence extraction still requires policy/human acceptance before medical eligibility changes.
- Ambiguous, stale, contradictory, or unmapped source facts should route to `NeedsMoreInformation`, `NeedsHumanReview`, or an internal document-review task.

Suggested review roles:

- Front desk / intake staff: collect missing documents, map obvious customer/pet/reservation context, prepare review packet.
- Medical document reviewer / trained staff: verify vaccine names, dates, pet identity, and veterinary source according to approved policy.
- Manager/admin: resolve policy exceptions, customer disputes, rejected/expired vaccine implications, and any provider/customer-facing action.
- AI workflow worker: extract, summarize, flag gaps/risks, draft internal tasks, and preserve evidence; no approval authority.

## Open questions for downstream cards

1. Where should the approved location vaccine policy live, and what exact vaccine names/freshness windows apply by species/service/location?
2. What counts as licensed-veterinarian proof, and how should clinic/vet identity be verified?
3. What document storage backend, retention period, encryption/access policy, and evidence id scheme should be used?
4. Which OCR/parser/LLM stack will produce extraction suggestions, and how are versions recorded for audit/reproducibility?
5. What review UI/state machine accepts, rejects, supersedes, or requests more information for each vaccine suggestion?
6. Which staff roles may approve vaccine facts, request customer follow-up, or execute provider write-back?
7. What provider integration mode is MVP for vaccine data: no integration, read-only reference, copy/paste by staff, or approved write-back?
8. What synthetic/de-identified fixtures should be committed, and where should large/private test docs live?
9. How should multi-pet, multi-location, stale provider records, and duplicate document submissions dedupe or merge?
10. What minimal audit trail is required for extraction, review, acceptance/rejection, customer follow-up draft, and any future provider mutation?

## Conservative downstream rule

When a vaccine document, source, date, pet identity, policy snapshot, provider payload, OCR output, or extracted field is missing, stale, ambiguous, conflicting, or unverified, the vaccine-document agent should preserve the uncertainty as an explicit suggestion/review state. It may create review packets and internal task drafts; it must not auto-accept medical facts, approve eligibility, change live provider state, or send customer-facing messages without an approved policy and human/tool execution record.
