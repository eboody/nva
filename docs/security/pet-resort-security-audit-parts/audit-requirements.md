# Pet resort audit requirements and event catalog

Purpose: define minimum audit requirements for security-relevant and business-critical state changes across booking, document/vaccine, incident, messaging, payment, AI, and human-approval workflows.

Status: draft security/audit input. This catalog is not a production retention schedule, role matrix, customer-visible history policy, or approval of AI/provider/customer-message automation. Open policy areas remain human approval gates.

## Source anchors

This artifact synthesizes:

- `docs/security/pet-resort-security-audit-parts/inputs.md`
- `docs/architecture/pet-resort-data-model.md`
- `docs/architecture/pet-resort-workflow-events.md`
- `docs/architecture/agent-permissions-by-workflow.md`
- `docs/architecture/agent-prompt-packet.md`
- `docs/architecture/pet-resort-ai-runtime-structured-output.md`
- `docs/workflows/payments-pricing.md`
- `docs/workflows/payments-pricing-parts/ai-boundaries.md`
- `docs/domain/petsuites/implementation-review.md`
- Current Rust anchors: `domain/src/entities.rs`, `domain/src/workflow.rs`, `domain/src/policy.rs`, `domain/src/agent.rs`, and `domain/src/tools.rs`.

## Audit invariants

1. Append-only by default: audit records are immutable after write. Corrections, voids, redactions, reversals, and supersessions are later events that reference the original event.
2. No hidden side effects: every state mutation, denied mutation, validation failure, policy denial, external command attempt, outbound message decision, and approval decision emits an audit event.
3. Source truth remains outside the audit log: raw documents, raw provider payloads, full message bodies, media, and prompt/tool traces live in governed evidence storage. Audit events store evidence IDs, hashes/tokens, redacted diffs, safe summaries, and access controls.
4. AI is never authority by itself: AI recommendations, classifications, drafts, and extraction candidates must link to AI run IDs and validation outcomes. Final business state changes require deterministic policy allowance or human approval.
5. Approval lineage is mandatory: any high-risk applied change references the source workflow event, AI suggestion/run when present, approval request, approval decision, approving actor, policy snapshot, before/after summary, and execution event.
6. Provider payloads are boundary evidence: provider events must be signature/source verified, deduped, and semantically mapped before domain state changes. Raw provider event names alone do not authorize mutations.
7. Redacted projections are first-class: ordinary staff and AI views receive safe audit summaries. Privileged compliance/security exports may reference governed evidence according to role, purpose, retention tier, and legal hold.
8. Retention is unresolved: all tiers below are classification labels for downstream policy. Actual durations, purge/legal-hold behavior, and customer-visible history require human approval.

## Minimum audit event fields

Every audit event should carry these fields unless explicitly marked optional.

| Field | Requirement |
| --- | --- |
| `audit_event_id` | Stable immutable audit event ID. |
| `event_name` | Catalog event name from this document or approved extension. |
| `event_family` | One of booking, document_vaccine, incident, messaging, payment, ai_runtime, approval, security_admin, external_integration, retention_export. |
| `occurred_at` | When the source action occurred, from source/provider clock when trustworthy. |
| `recorded_at` | When the app recorded the immutable audit event. |
| `actor` | Structured actor: type, actor ID, display-safe label, role/capability snapshot, location/tenant scope, impersonation/delegation chain when present. |
| `subject` | Primary subject: entity type and ID. Use one primary subject and place secondary IDs in `related_ids`. |
| `related_ids` | Customer, pet, reservation, task, document, vaccine, incident, message, payment, policy, provider, workflow, AI run, approval, evidence, and previous audit IDs as applicable. |
| `source_channel` | `customer_portal`, `staff_app`, `manager_app`, `provider_webhook`, `provider_poll`, `system_scheduler`, `policy_evaluator`, `workflow_worker`, `ai_runtime`, `manual_import`, `migration`, or approved extension. |
| `correlation_id` | Request/job/workflow correlation across events. Required for multi-step workflows. |
| `idempotency_key` | Required for derived events, external/provider commands, task creation, message send, payment operation, AI retry/replay, and imports. |
| `workflow_event_id` | Required when the audit event originates from or applies a workflow event. |
| `policy_snapshot_id` | Required for permission, eligibility, booking, payment, messaging, retention, redaction, and AI/tool decisions. |
| `permission_decision` | Allowed/denied/review-required/suppressed, with capability and denial/review reason. Required for mutating or high-risk actions. |
| `before` | Redacted before summary or field-level change descriptors. Omit only for pure create/import/intake with no prior state. |
| `after` | Redacted after summary or field-level change descriptors. Omit only for denied/suppressed/failed events that made no state change. |
| `change_summary` | Human-readable safe summary suitable for role-scoped audit projections. |
| `reason_codes` | Controlled reason vocabulary for change, denial, approval, rejection, suppression, retry, correction, or exception. |
| `risk_classification` | Sensitivity/risk tags such as customer_pii, pet_medical, behavior_safety, payment_legal, incident_liability, AI_tool_execution, security_admin, raw_provider_boundary. |
| `ai_run_id` | Required when an AI prompt/output/tool run contributed to the event. |
| `ai_output_id` | Required when applying, approving, rejecting, editing, or suppressing an AI draft/recommendation/classification/extraction. |
| `approval_id` | Required when a human approval was requested, decided, or applied; also required for high-risk final changes. |
| `external_ref` | Provider/system/account/object/event/command IDs when a boundary system is involved; avoid raw payloads. |
| `evidence_refs` | Governed evidence IDs, immutable storage refs, content hashes, source document refs, message refs, media refs, provider event refs, or tool trace refs. |
| `redaction_policy_id` | Redaction profile applied to before/after, evidence, prompt/output, message body, or provider payload references. |
| `retention_tier` | Classification tier from the retention table below. |
| `integrity` | Event hash, previous event hash or sequence pointer when implemented, writer service/version, schema version, and optional signature. |
| `visibility` | View/export classes permitted: operational_summary, manager_review, compliance_security, engineering_boundary, customer_visible_candidate. |
| `export_control` | Whether export is blocked, manager/compliance-approved, legal-hold-only, or ordinary role-scoped. |

### Actor requirements

Supported actor types:

- `customer`: authenticated or verified customer/owner/pet parent.
- `staff`: staff/front-desk/caregiver/groomer/trainer/lead/manager/admin human actor.
- `system`: deterministic application service, scheduler, policy evaluator, queue worker, validator, or migration.
- `ai_agent`: bounded Hermes/model workflow actor. Must include agent/workflow identity and tool-permission profile, not a human role.
- `external_integration`: provider, PMS, payment processor, email/SMS service, webhook source, or import source. This is distinct from `system`.
- `security_operator`: engineering/integration/compliance/privacy actor performing privileged configuration, export, redaction override, secret rotation, or incident response.

Actors must preserve `authenticated_actor_id`, `effective_role`, `capability_snapshot_id`, `location_id`, `tenant_id` if any, `session_or_job_id`, and `delegated_by_actor_id` when acting on behalf of another user. AI actors cannot appear as approvers for their own suggestions.

### Subject and linkage requirements

Each audit event must have one primary subject and zero or more related IDs. Downstream readers must be able to reconstruct this chain without joining through free text:

`source event -> workflow event -> AI run/output when any -> approval request/decision when any -> applied state change/external command/message send -> resulting final state`

Required linkage patterns:

- Booking events link `reservation_id`, `customer_id`, `pet_ids`, `service_id`, `location_id`, capacity/room/hold IDs, payment IDs, message IDs, task IDs, and provider reservation refs where present.
- Document/vaccine events link `document_id`, immutable file/evidence refs, OCR/extraction run IDs, `vaccine_record_id`, pet/customer/reservation IDs, source/provider refs, and reviewer/approval IDs.
- Incident events link `incident_id`, reservation/pet/customer IDs, staff reports, evidence/media refs, tasks, customer-message drafts/sends, approvals, and follow-up events.
- Messaging events link `message_id`, thread/conversation ID, recipient/contact alias, channel, template/draft IDs, approvals, provider send/delivery IDs, and source workflow/subject IDs.
- Payment events link `payment_id`, deposit/checkout/invoice refs, provider account/event/command IDs, reservation/customer IDs, policy snapshot, approval IDs, and reconciliation results.
- AI events link `ai_run_id`, prompt packet ID, model/provider/version, tool calls, structured output IDs, validation result IDs, policy snapshot, review packet/task IDs, and applied/rejected downstream event IDs.
- Approval events link `approval_id`, requested action, review gate, approver actor, policy snapshot, AI suggestion when any, source workflow event, and applied/rejected/cancelled state-change events.

## Retention and redaction tiers

These are classification tiers, not approved durations.

| Tier | Intended content | Default redaction | View/export posture |
| --- | --- | --- | --- |
| `operational_summary` | Safe state-change summaries, task status, non-sensitive workflow routing, message/send status without body. | Redact direct contact PII, medical details, raw payment/provider data, secrets, prompt/tool traces. | Staff/lead/manager role-scoped views. Export requires business purpose and role permission. |
| `sensitive_business` | Booking exceptions, capacity decisions, staff/manager approvals, customer-message draft metadata, payment semantic state, incident-safe summary. | Field-level summaries and hashes/tokens for sensitive values. | Manager/compliance/payment-operator views by purpose; ordinary staff see summaries only. |
| `regulated_sensitive` | Pet medical/vaccine/behavior/safety facts, incident details, liability/legal/payment exception records, customer-sensitive messages. | Strong redaction by default; raw evidence only by governed evidence link. | Manager/legal/compliance/privacy and explicitly authorized operational reviewers. Export requires approval and legal-hold awareness. |
| `ai_tool_trace` | Prompt packet IDs, model outputs, validation failures, tool call refs, AI risk flags, retry/dead-letter records. | Redact prompts/outputs to source refs and safe summaries unless review purpose requires full packet. Never include secrets/payment instruments. | Engineering/security/compliance plus workflow reviewers when needed. AI workers get minimal prior summaries only. |
| `provider_boundary` | Raw provider/webhook/import refs, signature verification result, provider event IDs, adapter errors, external command IDs. | Store raw payloads outside audit in boundary evidence; audit includes hashes, IDs, semantic mapping summary, and verification state. | Engineering/integration/security and domain owners by purpose. Export is restricted. |
| `security_privileged` | Role changes, permission grants, privileged audit views, exports/downloads, redaction overrides, secret rotation/configuration, retention/legal holds. | Do not expose secrets/tokens/signatures; preserve config item IDs, actor IDs, purpose, and before/after safe summary. | Security/admin/compliance only; exports logged and approval-gated. |
| `customer_visible_candidate` | Candidate portal history/status events or approved customer-facing communication history. | Customer-safe wording only; no internal notes, staff identifiers beyond approved label, raw audit metadata, AI traces, or other customers/pets. | Not customer-visible until product/privacy approval explicitly maps event classes. |

Redaction rules:

- Before/after values for sensitive fields should store `field_path`, `change_kind`, `old_hash_or_token`, `new_hash_or_token`, and `safe_summary`.
- Full customer message bodies, document images/OCR, incident evidence/media, raw provider payloads, prompt packets, tool traces, and staff/security notes should be referenced by evidence IDs, not copied into ordinary audit rows.
- Secrets, raw card/bank data, CVV, full payment instruments, auth tokens, webhook signatures, API keys, credential material, and unnecessary raw provider payloads must not be stored in audit fields or AI prompts.
- Redaction overrides are security-privileged audit events with purpose, approver, scope, expiration if any, and export/download IDs.

## Record mutability model

| Record type | Mutability | Notes |
| --- | --- | --- |
| `AuditEvent` | Immutable append-only. | Never update/delete ordinary rows. Corrections, redactions, reversals, supersessions, and late provider reconciliation emit new audit events referencing prior IDs. |
| `EvidenceRef` metadata | Append-only with governed metadata updates. | File/object storage refs, hashes, scan results, legal-hold status, and redaction artifacts may gain new records. Do not overwrite original provenance. |
| `WorkflowEvent` | Append-only input/result history; current workflow/task state may be mutable separately. | Replays/retries/dead-letter outcomes are new events with idempotency keys. |
| `ApprovalRecord` | Append-only decision history with mutable current projection. | Approval request/decision/application history is immutable; operational projection may show current status. |
| `Message`, `Payment`, `Reservation`, `Incident`, `Document`, `VaccineRecord` projections | Mutable current state plus immutable audit lineage. | Domain rows can update through validators, but every transition and high-risk denial/suppression emits audit. |
| Redacted projection | Rebuildable/read-model. | Can be regenerated from immutable events plus current redaction policy; exports of projections are themselves audited. |

## Visibility and export policy

Open gate: the final role matrix is not approved. The rules below define the minimum posture for downstream implementation.

| Viewer/exporter class | May view | Must not view by default | Export posture |
| --- | --- | --- | --- |
| Customer | Approved customer-visible status/history and messages for their own account only. | Internal notes, staff/security logs, AI/tool traces, raw audit metadata, other customer/pet data, privileged approvals. | Customer portal/export policy requires product/privacy approval before enabling. |
| Staff/front desk/caregiver/groomer/trainer | Operational summaries for assigned location/subject/task and customer-safe message/task status. | Raw payment instruments/provider payloads, raw documents unless assigned, incident/legal details outside need-to-know, security logs, broad AI traces. | Ordinary export disabled or manager-approved only. |
| Lead/manager/owner/admin | Manager review views, approvals, sensitive business summaries, assigned incident/payment/booking exceptions. | Secrets, full provider payloads, unrelated tenant/location data, raw AI/tool traces unless needed for review. | Export allowed only by role/purpose and logged. Sensitive exports may require compliance/legal approval. |
| Payment reconciliation/provider operator | Payment/provider semantic status, reconciliation events, provider refs, failed payment tasks. | Pet medical/behavior detail unless directly relevant; unrelated messages/incidents. | Payment exports are restricted, audited, and exclude raw instruments/secrets. |
| Legal/compliance/privacy/security | Privileged audit, retention/legal hold, exports/downloads, redaction overrides, sensitive investigations. | Secrets/raw credentials except through approved secret-management metadata; unrelated business data outside purpose. | Export/download/view is audited with reason, scope, and approval when required. |
| Engineering/integration owner | Provider boundary events, schema/adapter errors, idempotency failures, tool/runtime configuration, non-secret metadata. | Business approval authority, raw sensitive content unless necessary and approved, secrets/tokens. | Debug exports restricted and audited; prefer hashes/IDs/safe payload samples. |
| AI workflow worker | Minimal source/result/audit refs necessary for the active workflow. | Broad audit history, raw sensitive evidence, security logs, secrets, unrelated customer/location data. | No export authority. Outputs are audited and validated. |

## Event catalog by audited area

### Booking and reservation changes

Audited subjects: `Reservation`, customer, pet(s), service, capacity/room/resource holds, task, message, payment/deposit, provider reservation refs.

| Event name | When emitted | Required subject/link fields | Before/after requirements | Reason codes/examples | Retention tier | Redaction policy |
| --- | --- | --- | --- | --- | --- | --- |
| `booking.request_created` | Inquiry or booking request is created or mapped from portal/staff/provider. | `reservation_id` or external request ref, `customer_id`, `pet_ids`, `service_id`, `location_id`, source workflow/provider IDs. | `after` safe request summary; no raw free text unless evidence ref. | `customer_portal_request`, `staff_entered`, `provider_import`, `phone_email_intake`, `duplicate_possible`. | `sensitive_business` | Redact contact PII/free text; reference message/evidence IDs. |
| `booking.profile_or_requirement_gap_recorded` | Missing profile, vaccine, care, payment, or policy information blocks or routes triage. | `reservation_id`, gap task IDs, document/vaccine/payment refs, policy snapshot. | Gap summary and affected required fields. | `missing_vaccine`, `missing_care_instruction`, `payment_required`, `ambiguous_pet_identity`, `policy_conflict`. | `sensitive_business` or `regulated_sensitive` when medical/behavior. | Redact medical detail; log requirement label and evidence refs. |
| `booking.status_suggested` | AI/system/staff suggests transition such as MissingInfo, VaccinePending, SpecialReview, Waitlisted, Offered, Confirmed, CheckedIn, CheckedOut, Cancelled, Rejected, NoShow. | `reservation_id`, `workflow_event_id`, `ai_run_id` if any, policy snapshot, approval request if needed. | Current status in `before`; suggested status in `after`; mark as suggestion, not applied. | `triage_result`, `capacity_available`, `vaccine_pending`, `manager_review_required`, `payment_pending`. | `sensitive_business` | Safe status and rationale only; no raw prompt/free text. |
| `booking.status_changed` | Authorized reservation lifecycle transition is applied. | `reservation_id`, actor, approval ID when required, source workflow event, provider command/ref when any. | Required old/new status, state-machine guard summary, policy snapshot, customer-message state if relevant. | `approved_confirmation`, `staff_check_in`, `staff_check_out`, `customer_cancelled`, `manager_rejected`, `no_show_recorded`. | `sensitive_business`; `regulated_sensitive` if medical/behavior/safety reason. | Redact sensitive reason detail; store safe reason and evidence refs. |
| `booking.capacity_hold_created` | Room/suite/slot/resource hold is created or renewed. | `reservation_id`, `capacity_hold_id`, `room_or_resource_id`, `location_id`, policy snapshot, expiration. | New hold summary, dates/resources, expiration; no unrelated occupancy detail. | `availability_hold`, `manual_manager_hold`, `waitlist_offer_hold`, `provider_imported_hold`. | `sensitive_business` | Hide unrelated customer/reservation identities in shared views. |
| `booking.capacity_hold_released` | Hold is released, expires, is superseded, or converted. | `reservation_id`, `capacity_hold_id`, reason, source event, resulting availability event if any. | Hold state before/after and capacity return preconditions. | `expired`, `cancelled`, `converted_to_confirmed`, `room_cleaning_pending`, `manager_release`. | `sensitive_business` | Redact unrelated room/customer details. |
| `booking.capacity_or_policy_exception_requested` | Overbooking, waitlist release, holiday/minimum-stay, group-play, special-care, or service exception is requested. | `reservation_id`, policy snapshot, task/approval IDs, related pet/document/payment/incident refs. | Exception request summary and affected guard. | `overbooking`, `holiday_exception`, `waitlist_release`, `special_care`, `group_play_override`, `service_denial_exception`. | `regulated_sensitive` for medical/behavior/safety; otherwise `sensitive_business`. | Redact medical/behavior/payment detail; use review packet evidence refs. |
| `booking.capacity_or_policy_exception_decided` | Manager/authorized actor approves/rejects/returns an exception. | `approval_id`, `reservation_id`, approver actor, policy snapshot, source request event. | Requested vs decided scope, constraints, expiration, and applied/not-applied state. | `approved_limited_scope`, `rejected_hard_stop`, `returned_missing_evidence`, `expired_unapplied`. | `regulated_sensitive` or `sensitive_business` | Approver visible by role; redact sensitive evidence in broad views. |
| `booking.provider_write_requested` | App prepares a provider reservation mutation after approval/deterministic policy. | `reservation_id`, provider account/object refs, approval ID, idempotency key, command ID. | Intended command summary; required preconditions and rollback/reconciliation plan. | `approved_status_update`, `approved_check_in`, `approved_check_out`, `approved_cancel`, `copy_paste_assist`. | `provider_boundary` | No raw payload/secrets; log command shape and semantic fields. |
| `booking.provider_write_result_recorded` | Provider mutation succeeds, fails, is deduped, or requires reconciliation. | Provider command/event IDs, `reservation_id`, idempotency key, audit/event refs. | Command state before/after and semantic result. | `provider_success`, `provider_failure`, `deduped`, `conflict`, `reconciliation_required`. | `provider_boundary` | Raw response in evidence storage only; audit safe summary/hash. |

Booking approval requirements:

- Confirmation, rejection, cancellation with policy/payment consequence, waitlist release, overbooking, room/capacity exception, check-in/out with unresolved hard stops, no-show consequence, and provider mutation require a human approval or separately approved deterministic policy path.
- AI may create `booking.status_suggested`, draft messages, and review packets, but `booking.status_changed` must reference the approving actor or deterministic policy allowance.

### Document and vaccine verification

Audited subjects: `Document`, `VaccineRecord`, pet, customer, reservation, extraction/OCR run, review task, evidence storage.

| Event name | When emitted | Required subject/link fields | Before/after requirements | Reason codes/examples | Retention tier | Redaction policy |
| --- | --- | --- | --- | --- | --- | --- |
| `document.received` | Customer/staff/provider/import supplies a file or document reference. | `document_id`, `customer_id`, `pet_id` if mapped, uploader actor, immutable file/evidence ref, source channel. | New metadata: type hint, hash, size/class, source timestamp, scan status. | `customer_upload`, `staff_scan`, `provider_import`, `unknown_type`, `mapped_to_pet`, `unmapped_external`. | `regulated_sensitive` | Do not log file URL/image/OCR; store evidence ref/hash. |
| `document.quarantined_or_rejected` | Virus scan/type/source/privacy check blocks document. | `document_id`, scan/evidence refs, actor/system decision, policy snapshot. | Prior intake state to quarantined/rejected; safe reason. | `malware_scan_failed`, `unsupported_type`, `wrong_pet`, `privacy_risk`, `corrupt_file`. | `regulated_sensitive` or `security_privileged` if malware/security. | Avoid file details; security details privileged. |
| `document.extraction_started` | OCR/AI/extraction job starts. | `document_id`, `extraction_run_id`, tool/model/version if AI/OCR, prompt packet ID if any. | Extraction job state and scope. | `vaccine_policy_requires_extraction`, `manual_review_assist`, `retry`. | `ai_tool_trace` plus `regulated_sensitive` | Prompt excludes unnecessary raw content; audit references document and run IDs. |
| `document.extraction_completed` | OCR/AI returns candidate structured facts or fails safely. | `document_id`, `extraction_run_id`, `ai_run_id` if any, output/validation IDs. | Candidate facts as redacted field labels/spans/confidence; failure safe summary. | `completed`, `low_confidence`, `ambiguous_pet`, `unreadable`, `policy_rejected_output`, `tool_failed`. | `ai_tool_trace` and `regulated_sensitive` | Redact OCR text; store spans/crops as governed evidence refs. |
| `document.review_requested` | Staff/manager/medical-document review task is required. | `document_id`, task ID, review gate, reservation/pet IDs, policy snapshot. | Review reason and evidence refs. | `medical_document_review`, `conflicting_dates`, `missing_vet_source`, `expired`, `eligibility_impact`. | `regulated_sensitive` | Safe reason in staff queue; raw evidence restricted to reviewers. |
| `document.verified` | Authorized reviewer verifies document metadata/usefulness. | `document_id`, reviewer actor, approval/review record, evidence refs. | Prior state to verified; verified scope and limitations. | `valid_proof_received`, `source_trusted`, `staff_verified`, `manager_exception`. | `regulated_sensitive` | Reviewer/evidence visible by role; raw image restricted. |
| `document.rejected` | Authorized reviewer rejects document or extraction. | `document_id`, reviewer actor, review record, optional customer-message draft ID. | Prior state to rejected; safe rejection summary. | `wrong_pet`, `unaccepted_source`, `expired`, `unreadable`, `insufficient_proof`, `duplicate_superseded`. | `regulated_sensitive` | Customer-safe reason separate from internal review detail. |
| `document.superseded_or_archived` | New document replaces old; retention/archive state changes. | Original/new `document_id`, retention/legal-hold IDs, actor/system source. | Supersession/archive before/after and evidence refs. | `newer_record`, `duplicate`, `retention_archive`, `legal_hold`, `manual_correction`. | `regulated_sensitive` | Never delete audit lineage; raw evidence disposition by retention policy. |
| `vaccine_record.suggested` | Extraction/staff/import suggests vaccine fact. | `vaccine_record_id`, `document_id`, `pet_id`, extraction/AI output ID if any, policy snapshot. | Candidate vaccine name/date/source/confidence as redacted structured summary. | `ocr_candidate`, `staff_entered_unverified`, `provider_import_untrusted`. | `regulated_sensitive` and `ai_tool_trace` when AI. | Suggestion is not compliance truth; no broad raw values. |
| `vaccine_record.review_requested` | Suggested or existing vaccine requires review/exception decision. | `vaccine_record_id`, pet/reservation/document IDs, review gate, task/approval ID. | Review reason and eligibility impact summary. | `expired`, `missing`, `conflicting`, `low_confidence`, `source_unverified`, `exception_requested`. | `regulated_sensitive` | Role-scoped medical/vaccine evidence only. |
| `vaccine_record.verified_current` | Authorized review makes vaccine current/accepted for a scope. | `vaccine_record_id`, reviewer/approval ID, document/source evidence, policy snapshot. | Old/new verification state, effective date, expiry, service/location scope. | `reviewed_current`, `trusted_structured_source`, `manager_exception_limited_scope`. | `regulated_sensitive` | Broad views see current/expired flag; details restricted. |
| `vaccine_record.verified_expired_or_rejected` | Authorized review marks expired/rejected. | `vaccine_record_id`, reviewer/approval ID, evidence refs, policy snapshot. | Old/new status and safe reason. | `expired`, `not_accepted_vaccine`, `wrong_pet`, `source_untrusted`, `date_missing`. | `regulated_sensitive` | Customer wording requires message approval; no raw document in audit row. |
| `vaccine_record.exception_decided` | Manager/authorized role approves or rejects exception. | `vaccine_record_id`, `approval_id`, pet/reservation IDs, policy snapshot, expiration/scope. | Requested vs decided exception, constraints, affected booking/service. | `exception_approved_once`, `exception_rejected_hard_stop`, `returned_for_vet_proof`. | `regulated_sensitive` | Exception details restricted; broad views show hard-stop/review status. |

Document/vaccine approval requirements:

- OCR/AI extraction may produce suggestions only.
- Final vaccine compliance, medical-document acceptance, exception approval, eligibility effects, and customer-facing denial/explanation require authorized human review in v1 unless a future trusted structured integration policy explicitly approves otherwise.

### Incidents and incident follow-up

Audited subjects: `Incident`, pet, customer, reservation, staff task, evidence/media, customer messages, payment/waiver linkage when any.

| Event name | When emitted | Required subject/link fields | Before/after requirements | Reason codes/examples | Retention tier | Redaction policy |
| --- | --- | --- | --- | --- | --- | --- |
| `incident.reported` | Staff/provider/import creates an incident. | `incident_id`, pet/reservation/customer IDs, reporter actor, source channel, evidence/media refs. | New report safe summary, category, severity candidate, time/place summary. | `injury`, `altercation`, `behavior`, `medication`, `escape`, `property`, `customer_service`, `provider_import`. | `regulated_sensitive` | Redact medical/injury detail, staff names, media URLs in broad views. |
| `incident.severity_changed` | Authorized actor changes severity/risk classification. | `incident_id`, actor, approval/review ID if required, policy snapshot. | Old/new severity, rationale, impact on tasks/messages/eligibility. | `new_evidence`, `manager_review`, `downgraded_after_review`, `escalated_safety`. | `regulated_sensitive` | Safe risk category in staff views; detailed rationale restricted. |
| `incident.evidence_attached` | Photo/video/document/note/witness statement/provider evidence is attached. | `incident_id`, evidence ID/hash/type, attaching actor, source. | Evidence metadata only, not raw evidence. | `photo`, `video`, `staff_statement`, `customer_statement`, `vet_record`, `provider_note`. | `regulated_sensitive` | Evidence access governed; audit row stores ID/hash/type. |
| `incident.follow_up_task_created` | Manager, care, vet, customer-contact, legal, payment, or operational task is created. | `incident_id`, task ID, assigned role, due/escalation, source event. | New task safe summary and gate. | `manager_review`, `care_watch`, `vet_contact`, `customer_notification`, `legal_review`, `refund_review`. | `regulated_sensitive` | Task body may be redacted by role; full packet restricted. |
| `incident.customer_message_drafted` | Customer-facing incident/follow-up draft created. | `incident_id`, `message_id`, drafter/AI run if any, evidence refs, review gate. | Draft metadata, risk class, fact source refs, not full body unless governed evidence. | `owner_notification`, `apology_followup`, `care_update`, `policy_explanation`, `legal_sensitive`. | `regulated_sensitive` and `ai_tool_trace` when AI. | Customer message body in message/evidence store; audit has safe summary. |
| `incident.customer_message_decided` | Incident message approved/rejected/returned/suppressed. | `incident_id`, `message_id`, `approval_id`, reviewer actor, policy snapshot. | Decision, reason, approved scope, send/not-send state. | `approved_final_text`, `rejected_unsafe`, `returned_missing_fact`, `suppressed_legal_risk`. | `regulated_sensitive` | Approval detail restricted; broad views show message status only. |
| `incident.status_changed` | Incident moves through review/investigation/resolution/closure/reopen. | `incident_id`, actor, approval ID when required, related task/message/payment refs. | Old/new status, unresolved items, required follow-up, closure basis. | `investigation_started`, `resolved`, `closed`, `reopened`, `needs_manager_review`, `legal_hold`. | `regulated_sensitive` | Closure details restricted; customer-visible summary separately approved. |
| `incident.operational_restriction_applied` | Playgroup restriction, special handling, service restriction, care watch, or pet profile impact is applied. | `incident_id`, pet/reservation/profile IDs, approval/policy snapshot, effective scope. | Old/new restriction summary and affected services/dates. | `group_play_suspended`, `individual_care_required`, `medical_watch`, `service_restricted`, `manager_exception`. | `regulated_sensitive` | Broad views see restriction label; details/evidence restricted. |
| `incident.payment_or_comp_review_linked` | Refund/credit/waiver/discount/forfeit review is linked to incident. | `incident_id`, payment/approval/task IDs, reservation/customer IDs. | Link summary and current payment review status. | `refund_requested`, `comp_review`, `waiver_review`, `charge_dispute`, `do_not_commit_customer`. | `regulated_sensitive` plus payment. | Hide amounts except authorized payment/manager views; no raw payment data. |

Incident approval requirements:

- AI may summarize, flag risks, draft internal tasks, and prepare customer-message drafts.
- AI must not close/downgrade incidents, admit fault, assign blame, make medical/legal conclusions, reinstate group play, suppress escalation, issue refunds/credits/waivers, or send customer/public messages.
- Severity changes with operational consequence, customer notifications, legal/compliance escalation, pet restrictions, incident closure, and payment remedies require authorized human approval.

### Customer and staff messages

Audited subjects: `Message`, customer, reservation/pet/incident/task/payment subject when applicable, provider communication refs.

| Event name | When emitted | Required subject/link fields | Before/after requirements | Reason codes/examples | Retention tier | Redaction policy |
| --- | --- | --- | --- | --- | --- | --- |
| `message.inbound_received` | Customer/staff/provider inbound message is received or imported. | `message_id`, thread ID, sender/recipient aliases, channel, customer ID if mapped, evidence/provider refs. | New message metadata and safe topic/risk summary; raw body as governed evidence. | `email`, `sms`, `portal`, `phone_note`, `provider_import`, `unmapped_sender`. | `sensitive_business` or `regulated_sensitive` by topic. | Redact full body/contact PII in audit; store body in message/evidence store. |
| `message.draft_created` | Staff/system/AI creates customer/staff message draft. | `message_id`, subject IDs, drafter actor, `ai_run_id` if any, source refs, template ID if any. | Draft metadata, intended recipient/channel, source facts, risk class; body ref. | `routine_followup`, `missing_info`, `booking_clarification`, `daily_update`, `incident`, `payment`, `vaccine`. | Topic-dependent; `ai_tool_trace` when AI. | Body redacted; audit source facts and review gate. |
| `message.draft_edited` | Draft body, recipient, template, source facts, or risk class changes. | `message_id`, editor actor, prior draft version, new draft version/evidence refs. | Field-level redacted diff; body hash/token changes. | `staff_edit`, `manager_edit`, `fact_correction`, `tone_adjustment`, `template_applied`. | Topic-dependent. | Store body diffs in governed evidence only; audit safe diff summary. |
| `message.approval_requested` | Message is submitted for review. | `message_id`, `approval_id`, review gate, reviewer role/queue, source subject IDs. | Requested scope, risk class, send conditions, suppression checks. | `customer_message_approval`, `manager_review`, `payment_sensitive`, `incident_sensitive`, `medical_sensitive`. | Topic-dependent. | Review packet governs body visibility. |
| `message.approval_decided` | Reviewer approves, rejects, returns, or narrows message. | `message_id`, `approval_id`, reviewer actor, policy snapshot. | Decision, scope, final draft version, reason. | `approved_to_queue`, `rejected_unsafe`, `returned_missing_fact`, `approved_template_only`. | Topic-dependent. | Approver visible by role; body restricted. |
| `message.queued` | Approved/deterministic path queues message for send. | `message_id`, channel, recipient alias, provider/account ref, idempotency key, approval/template IDs. | Queue state, send conditions, suppression precheck. | `approved_queue`, `deterministic_template`, `manual_send`, `retry_queue`. | `sensitive_business` or higher. | Recipient details minimized; no full address/phone outside authorized views. |
| `message.send_attempted` | Provider send command attempted. | `message_id`, provider command/event ID, idempotency key, actor/system, approval ID. | Command status and provider semantic result. | `provider_attempt`, `retry`, `deduped`, `rate_limited`, `provider_error`. | `provider_boundary` plus topic tier. | Raw provider payload outside audit; redact recipient. |
| `message.delivery_status_recorded` | Delivered, failed, bounced, replied, unsubscribed, suppressed, or cancelled status recorded. | `message_id`, provider event ID, thread ID, status, source. | Old/new delivery state and safe provider summary. | `delivered`, `failed`, `bounced`, `reply_received`, `unsubscribed`, `suppressed`, `cancelled`. | `sensitive_business` or higher. | Raw inbound reply body stored separately and separately audited as inbound. |
| `message.suppressed` | Send or draft is suppressed for policy/facts/contact preference/risk. | `message_id`, subject IDs, suppression policy snapshot, actor/system, prior draft/queue ref. | Suppression reason and state before/after. | `do_not_contact`, `missing_fact`, `stale_source`, `sensitive_topic`, `legal_risk`, `outside_template`, `duplicate`. | Topic-dependent. | Safe suppression reason; sensitive detail restricted. |
| `message.thread_linked_or_merged` | Message thread linked to customer/subject or duplicate thread merged. | Thread IDs, customer/subject IDs, actor/system, identity-match evidence. | Old/new linkage summary. | `identity_match`, `manual_link`, `duplicate_merge`, `ambiguous_match_rejected`. | `sensitive_business` | Redact contact PII and unrelated thread contents. |

Messaging approval requirements:

- Default customer-message posture is draft/review.
- Sensitive medical, vaccine, medication, allergy, behavior, incident, safety, legal/liability, eligibility/refusal, payment/refund/waiver/discount/forfeit, cancellation/no-show, complaint, policy-exception, or ambiguous-fact messages require human approval.
- Deterministic auto-send, if later approved, must still emit draft/template/fact packet, suppression check, queued, send attempted, and delivery status events.

### Payment status changes

Audited subjects: `Payment/Deposit`, reservation, customer, checkout/invoice/deposit refs, provider refs, manager approvals.

| Event name | When emitted | Required subject/link fields | Before/after requirements | Reason codes/examples | Retention tier | Redaction policy |
| --- | --- | --- | --- | --- | --- | --- |
| `payment.policy_snapshot_evaluated` | Deposit/payment/refund/cancellation/no-show policy is evaluated for a reservation/action. | `payment_id` or reservation ID, policy snapshot, actor/system, source workflow. | Semantic requirement/status result; no raw payment instrument. | `deposit_required`, `not_required`, `balance_due`, `refund_policy`, `no_show_fee`, `requires_review`. | `sensitive_business` | Amounts visible only by payment/manager role; no raw provider data. |
| `payment.deposit_status_changed` | Deposit state changes: not required, required, pending, authorized, paid, failed, waived, cancelled. | `payment_id`, reservation/customer IDs, actor/source/provider refs, approval ID if waiver/exception. | Old/new semantic status, amount class, policy snapshot, provider event ref if any. | `provider_paid`, `manual_record`, `failed`, `waived_approved`, `cancelled_booking`, `reconciliation_update`. | `sensitive_business`; exceptions may be `regulated_sensitive`. | Redact instruments/provider payload; use semantic refs. |
| `payment.checkout_created` | Checkout/payment request is created. | `payment_id`, checkout ID, provider/account ref, reservation/customer IDs, idempotency key, approval/policy. | New checkout summary, amount/purpose class, expiration, source. | `deposit_checkout`, `balance_checkout`, `approved_payment_request`, `manual_checkout`. | `provider_boundary` and `sensitive_business` | No checkout URL/token in broad audit; store provider refs safely. |
| `payment.checkout_sent_or_suppressed` | Payment request message is sent/queued/suppressed. | Payment/message IDs, approval/template IDs, source event, suppression policy. | Send/suppression status and reason. | `approved_send`, `do_not_contact`, `missing_policy`, `payment_sensitive_review`, `duplicate_suppressed`. | `sensitive_business` | Redact recipient and payment details except authorized views. |
| `payment.provider_webhook_verified` | Payment provider webhook is received and signature/source validated/rejected/deduped. | Provider event ID/account, signature verification result, idempotency key, payment/reservation mapping if any. | Verification and mapping summary; raw body hash/ref. | `signature_verified`, `signature_rejected`, `deduped`, `unmapped`, `schema_mismatch`. | `provider_boundary` and `security_privileged` if rejected/security. | Raw body and signatures outside audit; never log secrets. |
| `payment.provider_status_recorded` | Provider says authorized/paid/failed/expired/disputed/refunded/partial/etc. | Provider event/ref, `payment_id`, reservation/customer IDs, reconciliation ID. | Old/new semantic provider status and trust state. | `authorized`, `succeeded`, `failed`, `expired`, `disputed`, `refunded`, `partial_refund`, `requires_reconciliation`. | `provider_boundary` and `sensitive_business`. | Provider refs safe; raw payload/instrument redacted. |
| `payment.reconciliation_result_recorded` | System/operator reconciles app state with provider/PMS/ledger. | `payment_id`, provider refs, reservation/customer IDs, reconciliation actor/job, evidence refs. | Reconciliation before/after, conflicts, resolved/unresolved status. | `matched`, `partial_match`, `duplicate`, `provider_conflict`, `manual_review_required`, `ledger_synced`. | `sensitive_business` | Payment details role-scoped; raw provider data excluded. |
| `payment.exception_requested` | Refund, waiver, discount, comp, credit, forfeiture, write-off, fee/deposit exception requested. | `payment_id`, reservation/customer IDs, approval/task ID, policy snapshot, incident ID if any. | Request summary, amount class, reason, customer-message impact. | `refund_request`, `deposit_waiver`, `discount`, `forfeit`, `write_off`, `incident_comp`, `cancellation_exception`. | `regulated_sensitive` | Hide amounts/reasons from unauthorized staff; no customer commitment. |
| `payment.exception_decided` | Authorized actor approves/rejects/returns payment exception. | `approval_id`, `payment_id`, approver actor, policy snapshot, source request. | Requested vs decided scope, amount class, execution permission. | `approved_refund`, `rejected_refund`, `approved_waiver`, `returned_missing_evidence`, `expired_unapplied`. | `regulated_sensitive` | Approver/payment detail restricted. |
| `payment.provider_command_requested` | Approved payment/refund/void/waiver/provider command is prepared. | `payment_id`, provider command ID, approval ID, idempotency key, actor/system. | Intended semantic command and approval scope. | `approved_refund_command`, `void`, `retry_payment`, `ledger_sync`, `manual_provider_action`. | `provider_boundary` and `regulated_sensitive`. | Raw command payload outside audit; no secrets/instruments. |
| `payment.provider_command_result_recorded` | Provider command succeeds/fails/dedupes/conflicts. | Command/provider event IDs, payment/reservation IDs, idempotency key. | Command result and reconciliation follow-up. | `success`, `failure`, `deduped`, `conflict`, `manual_reconciliation_required`. | `provider_boundary` and `regulated_sensitive`. | Raw response outside audit. |

Payment approval requirements:

- AI may summarize, flag risks, draft customer wording, and request internal review.
- AI may not approve or execute refunds, waivers, discounts, credits, forfeitures, write-offs, rate/tax/fee changes, payment retries, provider commands, or customer commitments.
- Provider `succeeded` events permit recording/reconciliation; they do not by themselves approve reservation confirmation, customer messaging, or ledger mutation unless deterministic policy and audit lineage allow it.

### AI recommendations, drafts, and classifications

Audited subjects: `WorkflowEvent`, `AiRun`, output/review packet/task/draft, and downstream domain subject.

| Event name | When emitted | Required subject/link fields | Before/after requirements | Reason codes/examples | Retention tier | Redaction policy |
| --- | --- | --- | --- | --- | --- | --- |
| `ai.workflow_enqueued` | App queues an AI/runtime job. | `workflow_event_id`, job ID, subject IDs, policy snapshot, idempotency key. | Queue/job metadata and allowed purpose. | `booking_triage`, `document_extraction`, `message_draft`, `incident_summary`, `payment_summary`, `daily_update`. | `ai_tool_trace` | No raw sensitive payload; reference source snapshots/evidence. |
| `ai.prompt_packet_built` | App builds least-privilege prompt packet. | Prompt packet ID, `ai_run_id` or job ID, subject IDs, policy/context refs, redaction policy. | Packet metadata: included source refs, excluded categories, tool permissions, schema ID. | `least_privilege_context`, `redacted_context`, `missing_required_input`, `policy_limited`. | `ai_tool_trace` | Store prompt body separately or redacted; audit has manifest/safe summary. |
| `ai.run_started` | Model/tool run starts. | `ai_run_id`, prompt packet ID, model/provider/version, agent/workflow identity, tool profile. | Run state start metadata. | `model_inference`, `tool_workflow`, `retry`, `replay`. | `ai_tool_trace` | Do not include secrets or raw prompt in ordinary audit. |
| `ai.tool_call_requested` | AI/runtime requests a tool or app-owned tool action. | `ai_run_id`, tool call ID, tool/capability name, subject IDs, allowed action check. | Requested tool/capability and policy decision. | `read_only_query`, `draft_task`, `forbidden_provider_mutation`, `forbidden_send`, `needs_approval`. | `ai_tool_trace` | Tool args redacted; raw sensitive query args in governed trace only. |
| `ai.tool_call_result_recorded` | Tool call completes/fails/denied. | Tool call ID, `ai_run_id`, result status, policy decision, evidence/output refs. | Result safe summary, no raw payload unless governed evidence. | `completed`, `failed`, `denied_by_policy`, `timeout`, `validation_failed`. | `ai_tool_trace` | Results redacted by data class. |
| `ai.output_generated` | AI returns structured output/draft/recommendation/classification. | `ai_run_id`, output ID, schema/version, subject IDs, source refs. | Output manifest: status, summary, recommended actions, risk flags, confidence/uncertainty. | `completed`, `needs_human_review`, `needs_more_information`, `failed_safely`, `rejected_by_policy`. | `ai_tool_trace` plus topic tier. | Full output may be governed evidence; audit safe manifest. |
| `ai.output_validated` | Application validates schema, policy, permission, source refs, and redaction. | Output ID, validation result ID, validator version, policy snapshot. | Validation before/after: accepted/rejected/converted-to-review. | `schema_valid`, `schema_invalid`, `policy_denied`, `missing_source_ref`, `unsafe_recommendation`, `converted_to_review`. | `ai_tool_trace` | Validation details safe; raw output only by evidence ref. |
| `ai.review_packet_created` | AI/system creates human review packet/task from output. | Output ID, review packet/task/approval ID, review gate, subject IDs. | Review packet safe summary, required decision, evidence refs. | `manager_review`, `medical_document_review`, `behavior_review`, `customer_message_approval`, `refund_or_deposit_exception`. | `ai_tool_trace` and topic tier. | Packet visibility by reviewer role and subject sensitivity. |
| `ai.output_applied` | Validated output is applied as draft/task/suggestion or through approved deterministic path. | Output ID, `ai_run_id`, applied audit IDs, approval/policy IDs. | What was applied and target state/draft/task IDs. | `draft_created`, `task_created`, `status_suggestion_recorded`, `deterministic_policy_applied`. | Topic tier plus `ai_tool_trace`. | Application event carries domain redaction; AI event links lineage. |
| `ai.output_rejected_or_suppressed` | Output is rejected, suppressed, dead-lettered, or needs retry. | Output ID, `ai_run_id`, validation/policy IDs, DLQ/retry IDs. | Rejection/suppression reason and no-state-change assertion. | `unsafe_content`, `unsupported_action`, `low_confidence`, `missing_input`, `stale_source`, `cross_customer_risk`, `dead_letter`. | `ai_tool_trace` | Redact unsafe/raw output except security review. |
| `ai.model_prompt_policy_changed` | Model, prompt, schema, tool permission, or policy version changes. | Config/policy IDs, actor, approval/security review IDs, rollout scope. | Old/new config safe summary and rollback plan ref. | `prompt_version`, `model_version`, `tool_permission_change`, `schema_change`, `redaction_policy_change`. | `security_privileged` and `ai_tool_trace` | Do not include secrets; config IDs and semantic diff only. |

AI approval requirements:

- All AI events include tool-permission profile and policy context.
- Disallowed or unsafe outputs become `ai.output_rejected_or_suppressed`, not hidden failures.
- Applying AI-authored customer messages, booking/payment/vaccine/incident decisions, provider mutations, or sensitive classifications to final state requires human approval or a separately approved deterministic path.

### Human approvals and overrides

Audited subjects: `ApprovalRecord`, target workflow/action/domain subject, approver actor, policy snapshot.

| Event name | When emitted | Required subject/link fields | Before/after requirements | Reason codes/examples | Retention tier | Redaction policy |
| --- | --- | --- | --- | --- | --- | --- |
| `approval.requested` | Workflow, staff, system, or AI asks for human approval/review. | `approval_id`, target action, subject IDs, requester actor, review gate, policy snapshot, source event/AI output IDs. | Request scope, proposed action, required authority, evidence refs. | `manager_approval`, `medical_document_review`, `behavior_review`, `customer_message_approval`, `refund_or_deposit_exception`. | Topic-dependent; often `regulated_sensitive`. | Review packet controls evidence visibility. |
| `approval.assigned_or_routed` | Approval routed to reviewer/queue/escalation. | `approval_id`, assigned role/actor/queue, due/escalation, location/scope. | Routing state before/after. | `manager_queue`, `lead_queue`, `payment_operator`, `legal_compliance`, `reassigned`, `escalated`. | Topic-dependent. | Avoid broad staff names if unnecessary; role labels safe. |
| `approval.decision_recorded` | Reviewer approves/rejects/returns/narrows/delegates. | `approval_id`, reviewer actor, role/capability snapshot, decision, policy snapshot. | Requested vs decision scope, constraints, expiration, reason. | `approved`, `rejected`, `returned_for_changes`, `approved_limited_scope`, `delegated`, `expired`. | Topic-dependent. | Rationale redacted by sensitivity; raw evidence by ref. |
| `approval.override_recorded` | Authorized actor overrides default policy/hard stop within allowed authority. | `approval_id`, override action, subject IDs, policy snapshot, reviewer actor. | Default decision vs override decision and conditions. | `manager_override`, `capacity_exception`, `medical_exception`, `payment_exception`, `customer_message_exception`. | `regulated_sensitive` or `security_privileged`. | Strong redaction; export restricted. |
| `approval.applied` | Approved action is actually executed/applied. | `approval_id`, applied audit event ID, actor/system, source workflow event, idempotency key. | Applied state change/command/message ref and resulting state. | `applied_to_reservation`, `provider_command_executed`, `message_queued`, `vaccine_verified`, `payment_refund_executed`. | Topic-dependent. | Domain event carries redaction; approval event links. |
| `approval.expired_or_cancelled` | Approval request/decision expires, is cancelled, superseded, or no longer applicable. | `approval_id`, actor/system, source/subject IDs, superseding approval if any. | Old/new approval status and reason. | `expired`, `superseded`, `cancelled_by_requester`, `subject_state_changed`, `policy_changed`. | Topic-dependent. | Safe reason; evidence restricted. |
| `approval.conflict_detected` | Same actor/self-approval, stale policy, missing authority, or conflict-of-interest detected. | `approval_id`, actor IDs, validation/policy IDs, target action. | Conflict summary and blocked state. | `ai_self_approval_blocked`, `insufficient_role`, `stale_policy`, `dual_control_required`, `delegation_invalid`. | `security_privileged` or topic tier. | Security detail restricted; no sensitive payloads. |

Approval requirements:

- Review gates are structured data, not prose: at minimum `ManagerApproval`, `MedicalDocumentReview`, `BehaviorReview`, `CustomerMessageApproval`, and `RefundOrDepositException`, with future extension for legal/compliance/privacy/security/export/retention gates.
- The approver must be a human or deterministic role/policy mechanism explicitly approved for the action. AI may request approval but never approve its own output.
- Applying an approval and approving a request are separate events. An approved action that is never applied must remain visible as such.

## Cross-area event families

These events are required when they affect any audited area above.

| Event name | When emitted | Retention tier | Notes |
| --- | --- | --- | --- |
| `workflow.event_recorded` | Normalized semantic workflow event accepted into app queue/history. | Topic-dependent | Links source/provider/import/customer/staff event to downstream audit lineage. |
| `workflow.event_deduped` | Event/job/task/action is deduped by idempotency/source key. | Topic-dependent | Prevents duplicate charges, messages, holds, tasks, approvals, and AI retries. |
| `workflow.validation_failed` | State-machine, schema, permission, source freshness, or invariant validation blocks an action. | Topic-dependent | Failure/denial is audit-worthy and must show no state change. |
| `workflow.dead_lettered` | Event/job/action cannot be processed safely and is routed to DLQ/manual review. | `ai_tool_trace` or `provider_boundary` plus topic tier | Preserve error class, source refs, retry/replay state, redaction. |
| `security.role_or_permission_changed` | Role, permission, capability, or reviewer authority is changed. | `security_privileged` | Requires actor, approval/change ticket if any, old/new safe summary, scope, rollback/ref. |
| `security.privileged_audit_viewed` | Privileged actor views governed raw evidence/audit detail. | `security_privileged` | Records purpose, scope, viewer, evidence/audit IDs, and result/export refs. |
| `security.export_or_download_created` | Audit/evidence/message/payment/document export or download is generated. | `security_privileged` | Records requester, approver if any, scope, filters, redaction profile, artifact ID/hash, retention/legal-hold state. |
| `security.redaction_override_used` | Actor bypasses default redacted projection for raw/sensitive evidence. | `security_privileged` | Requires purpose, approval if needed, scope, expiration, and viewed evidence refs. |
| `security.secret_or_integration_config_changed` | Credential, webhook, provider account, model/provider config, or tool permission is configured/rotated/disabled. | `security_privileged` | Never log secret values; log item/config IDs and semantic before/after. |
| `retention.legal_hold_changed` | Legal hold, retention class, purge eligibility, or archive state changes. | `security_privileged` | Requires actor, subject/evidence scope, policy snapshot, approval/legal basis. |
| `retention.purge_or_archive_executed` | Governed raw evidence/projection retention action runs. | `security_privileged` | Audit event itself remains append-only; purge target and result logged by ID/hash. |

## Event naming conventions

- Use lower-case dotted names: `<area>.<noun_or_state>_<verb_or_result>`.
- Prefer domain events over raw provider names. Example: map provider `animal_created` or PMS document webhooks into `document.received`, `pet_profile.created`, or a workflow event only after verification and semantic mapping.
- Add event names only when routing, permission, retention, or audit semantics differ. Otherwise use `reason_codes`, `risk_classification`, and `related_ids`.
- State suggestions and state applications are separate: `booking.status_suggested` vs `booking.status_changed`.
- Approval requests, decisions, and applied changes are separate: `approval.requested`, `approval.decision_recorded`, `approval.applied`.
- External commands have request and result pairs: `booking.provider_write_requested` / `booking.provider_write_result_recorded`, `payment.provider_command_requested` / `payment.provider_command_result_recorded`.

## Minimum reason-code vocabulary

Reason codes should be controlled, searchable, and safe to show in role-scoped projections. Initial cross-area codes:

- Source/reliability: `customer_reported`, `staff_entered`, `provider_import`, `provider_verified`, `provider_rejected`, `untrusted_source`, `stale_source`, `conflicting_sources`, `missing_source_ref`.
- Policy/permission: `policy_allowed`, `policy_denied`, `review_required`, `insufficient_role`, `stale_policy`, `outside_approved_template`, `forbidden_action`, `deterministic_path_allowed`.
- Data quality: `missing_required_field`, `ambiguous_identity`, `low_confidence`, `unreadable`, `schema_invalid`, `validation_failed`, `needs_more_information`.
- Booking/capacity: `capacity_available`, `capacity_unavailable`, `capacity_hold_expired`, `overbooking_exception`, `waitlist_release`, `holiday_exception`, `room_cleaning_pending`, `service_restricted`.
- Document/vaccine: `source_unverified`, `licensed_source_missing`, `expired`, `not_yet_effective`, `wrong_pet`, `duplicate_superseded`, `medical_document_review`, `exception_requested`.
- Incident/safety: `injury`, `altercation`, `behavior_risk`, `medication_issue`, `escape`, `property_damage`, `legal_risk`, `customer_notification_required`, `group_play_suspended`.
- Messaging: `approved_to_queue`, `sent`, `delivered`, `failed`, `bounced`, `unsubscribed`, `suppressed`, `do_not_contact`, `missing_fact`, `sensitive_topic`, `duplicate_message`.
- Payment: `deposit_required`, `paid`, `failed`, `expired`, `disputed`, `refund_requested`, `waiver_requested`, `approved_refund`, `reconciliation_required`, `provider_conflict`.
- AI/runtime: `prompt_built`, `model_completed`, `tool_denied`, `output_validated`, `output_rejected`, `converted_to_review`, `dead_letter`, `retry`, `replay`.
- Approval: `approved`, `rejected`, `returned_for_changes`, `approved_limited_scope`, `expired`, `cancelled`, `self_approval_blocked`, `conflict_detected`.
- Retention/security: `legal_hold`, `export_created`, `redaction_override`, `secret_rotated`, `permission_changed`, `privileged_view`, `purge_executed`.

## Implementation notes for downstream schemas

1. Model `AuditEvent` as an append-only event store plus rebuildable read projections. Domain aggregate rows can remain mutable current-state projections if every transition points back to immutable audit events.
2. Store `before`/`after` as structured redacted diffs, not arbitrary JSON blobs, wherever possible: `field_path`, `change_kind`, `old_token`, `new_token`, `safe_summary`, `evidence_ref`.
3. Add a first-class `external_integration` actor type. Current Rust anchors note that external provider actors should not be collapsed into system or staff actors.
4. Add first-class IDs for `ai_run_id`, `ai_output_id`, `approval_id`, `workflow_event_id`, `policy_snapshot_id`, `correlation_id`, `idempotency_key`, `evidence_ref`, and `redaction_policy_id` on events that cross boundaries.
5. Require per-subject sequence numbers or hash chains if downstream security review requires replay guarantees. This remains an open design question for v1 but the schema should not preclude it.
6. Treat audit read/export as audited actions. Viewing privileged raw evidence, exporting reports, or overriding redaction is itself security-relevant.
7. Keep customer-visible history as an explicit approved projection. Do not expose ordinary audit rows directly to customers.
8. Preserve open gates for role matrix, retention durations, AI tool permissions, deterministic customer-message send paths, and provider write mode.

## Open approval gates

1. Final role matrix: exact view, draft, approve, execute, export, configure, redaction-override, and security-admin authority by role/location/tenant.
2. Retention schedule: approved durations, purge/legal-hold behavior, archival handling, customer deletion constraints, and evidence-storage retention per tier.
3. Customer-visible history: which booking/message/payment/document status events can be shown to customers and in what wording.
4. AI governance: workflows that may be `SafeToAutomate`, if any; model/provider/prompt/tool trace retention; prompt/output inspection rights; task auto-creation limits.
5. Provider write mode: read-only import, copy/paste assist, approved write-back, or no integration for reservations, payments, messages, documents, and customer/pet records.
6. Payment policy: provider choice, exact refund/waiver/discount/forfeit authority, customer-facing copy, and reconciliation source of truth.
7. Incident/legal/compliance policy: severity levels, legal-hold triggers, customer-notification authority, and public/review-response handling.
8. Per-subject ordering/integrity: whether v1 needs sequence numbers, hash chains, signatures, or replay proofs beyond append-only timestamps/IDs.
