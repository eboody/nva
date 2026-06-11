# Auto-accept vs human review rules for vaccine documents

Purpose: define the narrow conditions under which extracted vaccine document records could be accepted without staff review, and the broader conditions that must route to human `MedicalDocumentReview`. This is a workflow-design artifact for `docs/workflows/vaccine-document-agent.md`; it does not approve live vaccine policy, customer-facing communication, provider write-back, reservation mutation, or medical eligibility decisions.

Status: draft rule proposal. The current MVP posture remains review-first: extracted medical-document facts are suggestions/unverified data unless an approved deterministic auto-accept policy and audit path explicitly allow a narrower path.

Human approval gates:

1. Auto-accept threshold and bypass scope.
   - A manager/admin or approved policy owner must approve the exact confidence threshold, eligible document/source types, required fields, audit requirements, and reviewer-bypass rationale before any production auto-accept path is enabled.
   - Until approved, every vaccine fact remains `needs_medical_document_review` even when extraction confidence is high.

2. Medical-document uncertainty policy.
   - A policy owner must approve how the system handles uncertain medical-document facts: ambiguous dates, unverified veterinary source, identity mismatch, aliases, conflicting documents, stale provider data, fallback validity windows, grace periods, and exceptions.
   - Until approved, uncertainty is preserved as suggestions, risk flags, `NeedsHumanReview`, `VaccinePending`, or `SpecialReview`; it must not be silently normalized into acceptance.

## Core principle

Auto-accept is an exception, not the default. A record may be auto-accepted only when every required signal is complete, source-backed, policy-covered, non-contradictory, and auditable. Any missing, stale, ambiguous, conflicting, low-confidence, handwritten/low-quality, unrelated, or policy-unknown condition routes to human review.

`WorkflowStatus::Completed`, `status: "success"`, or a high AI/OCR confidence score means the workflow produced a safe internal artifact. It never means vaccine proof is accepted unless the `acceptance_decision` explicitly records an approved deterministic auto-accept rule or human reviewer action.

## Required acceptance context

Before considering any extracted vaccine suggestion for auto-accept, the workflow must have all of the following context:

| Context | Required for auto-accept | Review if missing/uncertain |
| --- | --- | --- |
| Raw evidence | Immutable private `document_id` and `evidence_id`, content hash, received timestamp, source, and derived OCR/parser artifact refs | Missing raw evidence, missing hash, unsafe storage state, unknown scan status, or raw-only data in event payload |
| Extraction version | OCR/parser/model/prompt version and run id recorded for reproducibility | Unknown extractor version or non-reproducible manual/imported mapping |
| Policy snapshot | Approved location-scoped `vaccine_policy_id`, version/snapshot, species/service row, required vaccines, alias map, source rule, validity rule, and service window | Missing, stale, draft, unknown service, unknown species, or policy row requiring manager approval |
| Subject mapping | One selected customer, one selected pet, and reservation/service context if eligibility is being compared | Unmapped document, candidate-only match, multiple possible pets, selected pet absent, reservation/service unknown |
| Audit path | Deterministic rule id, acceptance reason, actor/system ref, timestamp, evidence refs, before/after state, and rollback/supersede path | No typed audit sink or no way to show why the fact bypassed human review |

If any required context is unavailable, extraction may still run and produce a review packet, but the result must not accept vaccine facts.

## Proposed auto-accept eligibility checklist

All checklist items must pass for every vaccine fact that would satisfy a requirement. Failure of any item routes the affected record, document, pet, or reservation to human review according to the routing table below.

1. Document type and source are eligible.
   - Eligible only if the approved auto-accept policy explicitly names the `document.kind` and `document.source` combination.
   - Suggested initial eligible set for later approval: clean printed `vaccine_record`, `immunization_record`, or `rabies_certificate` from a trusted customer upload/staff scan/provider reference where the clinic/veterinary source is evident and extraction quality is high.
   - Not eligible by default: vet invoices with mixed charges, screenshots, portal exports without provenance, email bodies without attachment evidence, medication instructions, boarding/grooming agreements, pet photos, unreadable files, unsupported document types, and documents dominated by handwriting.

2. Field completeness is sufficient.
   - Required document fields: `document_id`, source, kind, received timestamp, raw attachment/evidence refs, page/image count when known, OCR/extraction artifact refs, and redaction/privacy state.
   - Required identity fields: accepted customer id, accepted pet id, species, owner-name or provider-id corroboration when policy requires it, and no blocking identity warnings.
   - Required vaccine fields for each requirement: canonical vaccine name, original label/alias, administered date if policy needs it, explicit expiration/due/next-due date, date role, precision, clinic/vet/source candidate, source refs for name and dates, and confidence values.
   - Required policy fields: policy snapshot id/version, species/service requirement, accepted aliases, source verification rule, service date or service end timestamp, validity mode, grace/fallback settings if any, and review/automation level.

3. Expiration or due date is future-valid for the relevant service.
   - The accepted expiration/due/next-due date must be explicit, unambiguous, day-precision or policy-approved precision, and on or after the relevant service date/window.
   - Boarding requires current-through checkout/end date in the location timezone.
   - Day play, day boarding, grooming, training, and DaySpa require current on service date unless the approved policy defines a wider window.
   - No one-year/three-year fallback, grace period, puppy/kitten schedule, titer, exemption, or partial vaccine-series acceptance is allowed unless encoded in the approved policy snapshot.

4. Pet and owner identity match is deterministic enough.
   - Auto-accept requires exactly one selected pet and one selected customer, with no species mismatch, provider-id conflict, owner mismatch, or unresolved name ambiguity.
   - Preferred match basis: verified provider external id or existing portal upload bound to the customer/pet plus corroborating pet name/species.
   - Name-only matches, renamed pets, missing owner context, household-level uploads, and documents listing multiple pets require review unless staff has already split and mapped the document into pet-specific evidence records.

5. Extraction confidence meets the approved threshold.
   - Proposed starting threshold for approval discussion: overall record confidence >= 0.95 and every acceptance-critical field >= 0.90, with `confidence.band: "high"` and source refs present.
   - Acceptance-critical fields are pet identity, species, canonical vaccine name/alias mapping, administered date when used, expiration/due date, date role, clinic/veterinary source, policy row, and service window.
   - Confidence is necessary but never sufficient. High confidence cannot override missing policy, unverified source, contradictions, ambiguous dates, multi-pet ambiguity, or unsupported document types.
   - Any confidence score below threshold, missing score, `none`, `low`, `medium`, model-disagreement flag, or parser limitation note routes to review.

6. Policy coverage is exact.
   - The vaccine name or alias must map to an approved canonical requirement for the pet's species, service, and location.
   - The policy snapshot must say whether source must be licensed-veterinarian proof, whether fallback validity is allowed, whether CIV/DHPP/DHLPP/DA2PP/DAPP aliases count, and how partial series are handled.
   - Unknown species/service rows, optional/location-specific vaccines, alias-equivalence uncertainty, missing policy snapshot, stale policy version, or draft policy status block auto-accept.

7. Source proof satisfies policy.
   - If policy requires a licensed veterinarian or veterinary professional source, the result must have accepted source proof under the approved source rule.
   - Extracted clinic/vet text alone is a suggestion unless the auto-accept policy defines an allowed deterministic source-trust mechanism.
   - Customer upload, staff scan, email ingestion, provider read-only lookup, and portal screenshot are evidence carriers; they are not proof of licensed-vet source by themselves.

8. There are no contradictions or blocking warnings.
   - Auto-accept requires empty `contradiction_flags`, no `blocking_review` warnings, no unresolved `ambiguous_fields`, and no mismatch warnings affecting identity, vaccine name, source, dates, policy, or service mapping.
   - Duplicate records are acceptable only if an approved deterministic supersede/dedupe rule selects one non-conflicting current record and audits why older records were ignored.

9. The document quality is adequate.
   - Auto-accept requires printed/typed or otherwise reliably parsed content with source refs to each critical field.
   - Handwritten forms, low-resolution photos, blur/skew/glare, cropped pages, low contrast, partial pages, mixed-language labels not covered by policy, or OCR output with missing region/page refs route to review.

10. The result stays internal and auditable.
   - Auto-accept, if later enabled, may update internal vaccine-document review state only through an approved adapter/tool path.
   - It must not send customer messages, write provider/PMS records, confirm/reject reservations, or change live customer-facing booking state.
   - Any downstream reservation/profile readiness change remains a recommendation unless a separate side-effect policy and reviewed tool execution exists.

## Human review routing rules

Route to `MedicalDocumentReview` or manager/admin review when any of these conditions occur.

| Condition | Required routing | Risk flags / review reasons |
| --- | --- | --- |
| Field completeness gap | `NeedsHumanReview` or `blocked` if the missing field prevents extraction | `missing_inputs`, `missing_required_vaccine_proof`, `source_refs_missing` |
| Missing explicit future-valid expiration/due date | `MedicalDocumentReview`; usually suggest `VaccinePending` for relevant booking | `ambiguous_vaccine_date`, `expired_or_stale_vaccine`, `unknown_validity` |
| Expired before service end/date | `MedicalDocumentReview` or staff missing-proof workflow | `expired_or_stale_vaccine`, `missing_required_vaccine_proof` |
| Ambiguous date format, missing year, date range, future administered date, impossible order | `MedicalDocumentReview` / `SpecialReview` when material | `ambiguous_vaccine_date`, `conflicting_vaccine_fact` |
| Pet/customer unmapped, name-only match, multiple pet candidates, species mismatch | `MedicalDocumentReview`; split document by pet if possible | `pet_identity_unmapped`, `conflicting_pet_identity`, `multi_pet_ambiguity` |
| Multi-pet household document | Review and, if accepted, create pet-specific reviewed facts | `multi_pet_ambiguity`, `needs_document_split` |
| Policy snapshot missing/stale/draft or service row unknown | `SpecialReview` or policy-owner review | `missing_vaccine_policy`, `service_policy_unknown`, `policy_snapshot_missing` |
| Alias or series rule not covered by policy | `MedicalDocumentReview` or manager/admin review | `policy_coverage_unknown`, `canonical_vaccine_mapping_uncertain` |
| Clinic/vet source missing or unverified | `MedicalDocumentReview` | `unverified_veterinary_source` |
| Contradictory documents/rows/provider data | `MedicalDocumentReview` / manager review if eligibility impact is disputed | `contradictory_vaccine_evidence`, `provider_payload_unverified` |
| Handwritten, blurred, cropped, low-quality, unreadable, or OCR-low-confidence scan | Staff review, reupload request draft, or failed/blocked extraction | `unreadable_document`, `low_quality_scan`, `ocr_confidence_low` |
| Unrelated or unsupported document | `no_action` with audit if clearly irrelevant; otherwise document review | `unsupported_document_type`, `not_vaccine_document` |
| Raw PII/provider payload required for decision but not safely available | Privacy-scoped review path, not auto-accept | `raw_pii_redacted`, `provider_payload_unverified` |
| Any customer-facing or live provider action requested | Block until separate approval path | `side_effect_requires_approval` |

## Suggested decision states

Use explicit decision states separate from extraction status.

```text
acceptance_decision.state:
  unreviewed_suggestion
  auto_accept_eligible_pending_policy_approval
  auto_accepted_by_policy
  needs_medical_document_review
  needs_more_information
  rejected_or_not_relevant
  superseded_by_newer_evidence
  blocked_policy_missing
```

Rules:

- `auto_accept_eligible_pending_policy_approval` is the safest current state for clean records that appear to satisfy all proposed rules but the human approval gates are not yet complete.
- `auto_accepted_by_policy` may appear only after the auto-accept threshold and uncertainty policy are approved and encoded as deterministic configuration.
- `accepted_by_staff` from the extraction schema remains a human review outcome, not an AI extraction outcome.
- `rejected_or_not_relevant` should preserve the reason and evidence refs so staff can audit why a document did not affect vaccine readiness.

## Proposed policy configuration fields

A future implementation should avoid hard-coding these rules in prompts. Store them as versioned policy/configuration values that can be audited and replayed.

```text
auto_accept_policy_id
policy_version
location_id
enabled: false by default
eligible_document_kinds[]
eligible_sources[]
required_identity_match_basis[]
required_field_paths[]
minimum_overall_confidence
minimum_critical_field_confidence
accepted_confidence_band
source_trust_rule
allowed_policy_snapshot_ids_or_versions
validity_rule_ref
alias_rule_ref
series_rule_ref
quality_thresholds
blocking_risk_flags[]
blocking_warning_severities[]
review_gate_when_blocked: MedicalDocumentReview | ManagerApproval
audit_event_template
side_effect_scope: internal_review_state_only
```

Default values should keep `enabled: false`, `side_effect_scope: internal_review_state_only`, and `review_gate_when_blocked: MedicalDocumentReview` until approved.

## Minimum output requirements

Every vaccine-document result should make the decision boundary clear:

- include `acceptance_decision.state` for each vaccine suggestion or policy comparison;
- include the evaluated policy snapshot id/version or a `missing_inputs` entry blocking acceptance;
- include confidence score and band for every acceptance-critical field;
- include source refs for identity, vaccine label/name, administered date when used, expiration/due date, and clinic/source;
- include `human_review_reason` whenever any review route applies;
- include risk flags for every blocking condition rather than hiding uncertainty in prose;
- include audit-ready reason text for `no_action`, `rejected_or_not_relevant`, `superseded`, and any future auto-accept decision;
- never inline raw OCR, raw document bytes, raw provider JSON, full owner contact details, or unrelated medical information.

## Example outcomes

### Clean printed single-pet record

If the document is a printed veterinary record for one mapped pet, includes explicit current expiration dates, has high-confidence source refs for every critical field, maps to an approved policy snapshot, and has no contradictions:

- Current MVP output: `auto_accept_eligible_pending_policy_approval` plus `MedicalDocumentReview` reason, because the auto-accept threshold and uncertainty policy are not approved yet.
- Future approved output: `auto_accepted_by_policy` only if the policy configuration explicitly allows this document/source type and all audit fields are recorded.

### Multi-pet document

If a household record lists multiple pets or vaccine rows not clearly tied to one pet:

- Output all pet candidates and row-level evidence refs.
- Set `acceptance_decision.state: needs_medical_document_review`.
- Add `multi_pet_ambiguity` and any identity mismatch flags.
- Do not attach vaccine facts to a final pet id until staff splits or maps the evidence.

### Expired or ambiguous date

If a vaccine row has an expiration date before the service end date, a missing year, an ambiguous locale format, or a next-due date that might not be an expiration date:

- Preserve the observed value and candidate interpretations.
- Set policy comparison to `expired_or_stale`, `conflicting`, or `not_evaluated` as appropriate.
- Route to review or missing-proof workflow; do not infer a validity period.

### Unrelated document

If the upload is clearly a grooming receipt, boarding agreement, medication instruction sheet, pet photo, or unrelated medical note with no vaccine proof:

- Set `document.kind: unsupported` or `other` and `acceptance_decision.state: rejected_or_not_relevant` or `needs_medical_document_review` if uncertain.
- Use `no_action` only for clearly irrelevant documents after recording evidence refs and a safe audit reason.
- Do not delete silently; do not send a customer message except through a separately approved draft/review path.

## Approval checklist before enabling auto-accept

Before production auto-accept is enabled, humans must approve:

1. The final location vaccine policy by species/service, including vaccine names, aliases, validity windows, grace periods, and series rules.
2. The source-verification policy, including what counts as licensed-veterinarian proof.
3. The auto-accept threshold: minimum overall confidence, per-field confidence, required evidence refs, and quality thresholds.
4. The uncertainty policy for ambiguous dates, multi-pet records, contradictions, handwritten/low-quality scans, provider payloads, screenshots, and fallback validity.
5. The side-effect boundary: whether auto-accept only changes internal review state or can later trigger any reservation/profile/provider action.
6. The staff override and audit workflow for rejects, supersedes, exceptions, and disputes.
7. The monitoring plan: periodic sampling of auto-accepted records, false-positive reporting, policy rollback, and model/parser version drift alerts.

Until all seven are approved and encoded, the safe workflow behavior is extraction plus review packet generation, not auto-acceptance.
