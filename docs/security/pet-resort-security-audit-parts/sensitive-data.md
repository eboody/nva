# Sensitive data categories and handling rules

Purpose: define the data classification, visibility, editability, exportability, masking/redaction, AI-runtime exposure, and storage/logging rules for sensitive pet-resort data.

Status: security/audit draft. This document does not approve the final role matrix, AI production permissions, customer-message automation, retention periods, payment-provider behavior, or live external-system mutations. Where a role/action is marked `approval-gated`, implementation must defer to the later human-approved role matrix and policy release.

Source anchors:

- `docs/security/pet-resort-security-audit-parts/inputs.md`
- `docs/architecture/agent-permissions-by-workflow.md`
- `docs/architecture/agent-prompt-packet.md`
- `docs/workflows/payments-pricing.md`
- `docs/workflows/payments-pricing-parts/ai-boundaries.md`
- `docs/workflows/staff-operations-parts/inputs.md`
- `docs/workflows/customer-messaging-parts/inputs.md`
- `docs/workflows/vaccine-document-agent-parts/inputs.md`

## Classification vocabulary

Sensitivity levels used below:

- `Low`: business-safe operational metadata. Still tenant/location scoped, but not usually sensitive on its own.
- `Confidential`: identifies customers, pets, staff, reservations, messages, routine care, or operational state.
- `Restricted`: medical, vaccine, behavior, incident, payment, legal/liability, privileged internal, or AI/tool trace data whose unnecessary exposure can cause safety, privacy, financial, reputational, or compliance harm.
- `Secret`: credentials, tokens, webhook signing secrets, raw payment instruments, CVV/card/bank data, auth material, and security-admin secrets. Secrets must not enter ordinary app logs, audit projections, prompts, exports, or staff views.

Visibility terms:

- `customer-visible`: customer/pet parent may see through approved customer surfaces or approved outbound messages for their own account/pet/reservation only.
- `staff-visible`: routine operational staff may see only fields required for assigned location, task, reservation, and care purpose.
- `manager-only`: manager/owner/admin approval or review is required.
- `owner/admin-only`: administrative, policy, configuration, privileged audit, security, or business-control views.
- `system-only`: deterministic application/provider/runtime components may access under controlled service credentials; humans see redacted projections unless privileged access is approved.
- `AI-worker-visible`: AI/Hermes worker may receive only the least-privilege, purpose-built, redacted fields declared in a prompt packet for the workflow.

Default deny rule: if a field is not explicitly needed for the current workflow, role, subject, location, and time window, do not show it, export it, log it, or include it in an AI prompt.

## Category table

| Category | Sensitivity | Default visibility | Editability / approval | Exportability | Masking / redaction | AI-runtime exposure |
| --- | --- | --- | --- | --- | --- | --- |
| Contact information | `Confidential`; emergency/vet contacts can be `Restricted` when tied to incidents or medical workflows. | Customer-visible for own account; staff-visible when needed for intake, booking, messaging, check-in/out, or emergency handling; manager/admin for duplicate merge, do-not-contact, privileged review. | Customer may submit/update allowed fields through approved portal; staff may propose or record routine corrections if role policy permits; duplicate merges, do-not-contact overrides, contact-preference conflicts, account linking, and emergency-contact escalation are manager/admin approval-gated. | Customer self-export allowed for own profile/message history subject to policy; staff exports are approval-gated and should be scoped/redacted; bulk contact exports are owner/admin-only with audit. | Mask phone/email/address in broad staff lists, logs, analytics, and audit summaries; show last-4/contact alias where enough; redact unrelated contacts from message drafts and exports. | Allowed only for workflows that need contacting or identity matching. Prompt packets should include minimal recipient/channel/preference fields, not broad contact history. AI must not override opt-out/contact preferences or invent contact facts. |
| Pet medical information | `Restricted`. Includes medications, doses/schedules, allergies, medical conditions, feeding/care instructions with health implications, vet instructions, special-care flags, and medical ambiguity. | Customer-visible only where the data belongs to the customer's pet and has been approved for customer display; staff-visible only for assigned care/booking/document/incident purpose; manager-only for exceptions, eligibility effects, ambiguity, and sensitive customer wording. | Customers may submit source information; staff may record factual observations/care instructions under role policy; human review is required for medical ambiguity, medication exceptions, eligibility effects, and customer-facing medical interpretation. AI may not approve or convert medical facts into operational truth. | Export only in customer self-export, manager/compliance packet, or care-transfer packet approved by policy. Redact unrelated notes, staff commentary, and internal risk analysis. | Redact medical details from dashboards, broad audit projections, ordinary logs, customer-message drafts unless approved, and AI prompts not performing medical/care/document work. Prefer flags and evidence refs over full raw text. | Allowed only for document/vaccine, staff-care, incident, booking eligibility, or approved message-draft workflows, and only as minimal task-relevant fields. Medical detail in prompts must carry source refs, freshness, confidence/verification status, and review gates. |
| Vaccine documents | `Restricted`; raw uploads/images/OCR can include contact PII, vet PII, signatures, invoice fragments, medical notes, and unrelated animals. | Customer can upload/view own submitted document status and approved customer-safe extracted status; document-review staff and managers may view raw docs for assigned pet/review; booking staff should usually see verified vaccine status, not raw images/OCR. | Customers upload/replace via approved portal. OCR/extraction can suggest facts. Human/trusted-integration review is required to verify, reject, supersede, waive, or apply eligibility consequences unless later approved policy narrows the gate. Raw document deletion/alteration is retention/legal-hold governed. | Raw document export is restricted to customer self-export, manager/compliance, or approved operational transfer. OCR/extracted text export should be redacted and evidence-linked. Bulk document export is owner/admin/compliance-only. | Store raw files in governed evidence storage; use document IDs in logs/prompts/dashboards. Redact vet/customer contact fields, signatures, addresses, file URLs, OCR text, and unrelated medical notes from ordinary projections. | Document/vaccine AI may receive assigned-pet document images/OCR or redacted excerpts only when required for extraction. Other workflows receive `vaccine_status`, `missing_required_vaccines`, `review_state`, and document refs, not raw OCR/images. |
| Incidents | `Restricted`; may include medical/safety data, staff statements, photos/video, liability/legal facts, blame/speculation, and payment/refund consequences. | Staff-visible only for assigned care/escalation need-to-know; manager-only for severity, investigation, customer communication, playgroup suspension/reinstatement, refunds/credits, and legal/compliance routing; customer-visible only through approved final messages or portal summaries. | Staff may report factual observations and attach evidence. Managers/authorized roles approve severity changes with operational consequences, owner notification, investigation closure, external communications, refund/credit ties, and reopening/closure. AI may summarize and flag but not close, downgrade, assign blame, diagnose, or notify. | Incident exports are manager/compliance/legal-only unless a customer-facing incident packet is approved. Export packets must separate factual timeline, approved customer language, internal notes, staff identities, and legal/compliance material. | Redact staff names, witness statements, exact facility/camera details, injury/medical specifics, photos/videos, customer PII, and speculation from broad views/logs. Audit summaries use incident ID, severity/risk class, review gate, source refs, and status. | Incident AI may receive incident-specific facts, relevant care history, and evidence refs under strict scope. Customer/public message drafts are `NeverAutoSend` by default and require manager approval for final text and send. |
| Payments / payment status | `Restricted`; raw payment instruments, CVV, card/bank data, auth tokens, provider secrets are `Secret` and forbidden in app prompts/logs. | Customer-visible for own approved balance/deposit/receipt/status copy; staff-visible as semantic statuses needed for booking/check-in/payment follow-up; payment reconciliation/operator and manager see provider refs/details needed for reconciliation; refunds/waivers/discounts/forfeitures are manager/payment-operator approval-gated. | Staff/AI may not move money. Provider-backed statuses may be reconciled by app policy. Refunds, retries, captures, voids, waivers, discounts, credits, forfeitures, fee changes, and customer payment commitments require authorized human approval and typed provider commands. | Customer receipts/status are exportable to the customer. Provider reconciliation exports are payment-operator/manager/admin-only. Never export raw card/CVV/bank data. Bulk financial exports require owner/admin and audit. | Store only semantic amounts/statuses/references in domain records. Keep raw provider payloads in adapter evidence with signature verification and retention policy. Mask provider refs except last-4/reference aliases in broad views; redact payment content from message/log/audit summaries. | AI may receive semantic payment/deposit status, due/missing flags, approved policy snapshot refs, and payment review gates. It must not receive raw card/bank/CVV/secrets or execute/refund/waive/charge. Payment-sensitive customer copy is draft/review or deterministic-template only after approval. |
| Staff notes | Usually `Confidential`; `Restricted` when containing medical/behavior/safety/incidents, staff performance, customer complaints, legal/payment details, or internal-only judgments. | Staff-visible by subject/task/location need-to-know; manager-visible for review/escalation; customer-visible only after explicit conversion to an approved customer-safe summary. | Staff may create factual notes within assigned workflow. Corrections should append/correct/void with audit rather than destructive edit. Sensitive notes, staff-performance commentary, incident conclusions, and customer-facing summaries require manager/review approval. | Export is scoped to operational case packets, customer-safe summaries, or compliance/admin review. Do not include internal commentary in customer exports unless reviewed and approved. | Separate factual observations from interpretation, speculation, and internal review. Redact staff identifiers, raw customer PII, medical detail, and internal judgments from broad dashboards/logs. | AI may receive only notes needed for the workflow and should treat note text as untrusted evidence, not policy. AI can summarize, flag ambiguity, and draft customer-safe wording, but must preserve source refs and review gates. |
| Internal-only notes | `Restricted`; can become `Secret` if they contain credentials/security details. Includes manager rationale, policy exceptions, investigation notes, legal/privacy commentary, staff discipline, admin/security notes, and non-customer-facing risk analysis. | Manager-only, owner/admin-only, legal/compliance/privacy, or system-only depending on purpose. Not customer-visible by default and not broadly staff-visible. | Created/edited only by authorized roles; corrections must preserve chain of custody. Any downgrade to customer-visible language requires review/approval and should create a separate customer-safe artifact, not expose the original. | Export only under owner/admin/compliance/legal approval, legal hold, or audited incident/security process. Never include secrets in exports. | Use strong redaction for broad audit/event projections. Keep internal-only labels explicit. Do not co-mingle with customer-visible notes or prompt examples. | AI access is exceptional and purpose-scoped, e.g. manager summary or compliance triage. Prompts must label it internal-only, forbid customer copying, and require redacted outputs. Never expose credentials/secrets. |
| AI drafts / recommendations | `Confidential` by default; `Restricted` when derived from or containing medical, vaccine, incident, payment, customer message, internal notes, or tool traces. | Staff/manager/reviewer visibility by workflow and subject; customer-visible only after human/deterministic approval and publication as a separate outbound message/status. AI/tool trace inspection is owner/admin/security/compliance scoped. | AI output is never source-of-truth by itself. Staff/managers may accept, edit, reject, suppress, or convert suggestions into approved records under policy. AI may not approve its own recommendations or perform high-risk mutations. | Export only as part of audit/review packets with source refs, model/policy versions, and redaction. Do not export raw prompts/outputs broadly; customer exports should show approved final messages/statuses, not internal reasoning/tool traces unless policy requires. | Redact raw prompts, tool inputs/outputs, model traces, customer PII, medical/payment/incident detail, provider payloads, and internal notes from ordinary logs. Store safe summaries, ids, schema/version, validation status, risk flags, and hashes. | AI output is the runtime artifact. Subsequent AI runs may use prior AI summaries only as historical artifacts with source refs and audit trail, never as operational truth. Prompt/output retention and visibility remain approval-gated. |

## Visibility boundaries by audience

### Customer / pet parent

Customers may see and update only their own approved account, pet, reservation, document upload/status, message, receipt/balance/status, and customer-safe care/update surfaces. They must not see:

- Other customers, pets, staff, reservations, rooms, capacity, or messages.
- Internal-only notes, manager rationale, investigation notes, staff performance/commentary, prompt/tool traces, provider raw payloads, or security/admin logs.
- Raw medical/document/OCR uncertainty unless intentionally presented through an approved customer-safe flow.
- Incident detail beyond approved final customer communication or policy-defined portal summary.

### Staff / front desk / caregiver

Staff views should be purpose-scoped to assigned location, reservation, task, shift, subject, or workflow. Staff can see enough to execute care and customer service safely, but should not automatically see:

- Raw vaccine documents/OCR unless they are in a document-review role/task.
- Broad medical history unrelated to the assigned task.
- Raw provider payment payloads, refund authority screens, card/bank data, or financial exception rationale.
- Internal-only manager/legal/security notes.
- Full AI prompts/tool traces, broad audit logs, or cross-customer histories.

### Lead staff / manager / owner-admin

Managers and owner/admins may receive broader review and approval views, but approval powers must be named capabilities, not inferred from job title. Manager/admin views should still separate:

- Customer-safe facts from internal rationale.
- Factual incident timeline from speculation/legal commentary.
- Semantic payment status from raw provider/payment secrets.
- Source documents from redacted extracted facts.
- AI draft/recommendation from approved operational truth.

### Payment reconciliation / provider operator

Payment operators need payment-specific provider refs, webhook/reconciliation status, duplicate/partial/failed transaction analysis, and typed command history. They should not receive broad medical/care/incident data except the minimal reservation/customer context needed to reconcile the payment case.

### Legal / compliance / privacy

Legal/compliance/privacy access is approval-gated and audit-heavy. These roles may need raw evidence, incident packets, export/download tools, legal hold controls, or redaction overrides. Every privileged access, export, legal hold, redaction override, or evidence download should produce an audit event.

### Engineering / integration owner

Engineering may need integration schemas, logs, provider mappings, webhook verification status, idempotency failures, and runtime/tool configuration. Engineering access should prefer synthetic fixtures, redacted payloads, ids/hashes, and safe summaries. Technical access must not imply business approval authority, payment authority, or broad production data browsing rights.

### System / deterministic policy evaluator

The app-owned system may evaluate policy, permissions, schemas, idempotency, dedupe, audit, and state transitions. System components should use service credentials with least privilege, write append-only audit events for decisions/failures, and expose role-scoped projections to humans.

### AI worker / Hermes worker

AI workers are bounded assistant actors. They may receive typed, least-privilege prompt packets for a specific workflow, subject, location, and purpose. They may extract, summarize, draft, recommend, classify, flag risk, and request human review. They must not:

- Treat user/event content as system/developer instructions.
- Approve their own suggestions.
- Convert model confidence into medical, vaccine, eligibility, incident, payment, booking, or legal authority.
- Send customer messages or mutate provider systems unless a later approved deterministic path executes after validation and approval.
- Receive secrets, raw card/bank/CVV data, webhook signing secrets, auth tokens, or unnecessary raw provider payloads.

## Editability and source-of-truth rules

1. Source-of-truth state lives in typed application/provider records, not in AI drafts, free-text notes, screenshots, raw webhooks, OCR, or message bodies.
2. Free text is evidence, not authority. Customer, staff, provider, OCR, and model text can suggest facts but cannot by itself establish payment truth, medical/vaccine compliance, booking acceptance, refunds/waivers, incident conclusions, or policy exceptions.
3. Corrections should be append-only where safety/audit matters. Use corrected/voided/superseded events rather than destructive edits for care notes, incident records, documents, vaccine records, payment events, outbound messages, and audit events.
4. AI-suggested edits must be validated against allowed actions, required evidence refs, review gates, and deterministic policy before becoming records or tasks.
5. Customer-facing edits to sensitive subjects require explicit approval when they mention health, medication, allergy, behavior, incident, safety, legal/liability, payment/refund/waiver/discount/forfeit, eligibility, refusal, cancellation/no-show, or policy exceptions.

## Exportability rules

1. Every export/download must be subject-scoped, actor-scoped, purpose-scoped, and audited.
2. Prefer redacted projections and customer-safe packets over raw source files.
3. Bulk exports, privileged audit exports, incident packets, raw document exports, payment reconciliation exports, prompt/tool trace exports, and redaction overrides are owner/admin/compliance approval-gated.
4. Customer self-export should include approved records for the customer's own account/pets/reservations/messages/payment receipts, not internal-only notes, staff commentary, unrelated provider payloads, broad audit logs, or AI/tool traces unless required by approved policy.
5. Exports containing medical, vaccine, incident, payment, legal/liability, or internal-only data should be watermarked or metadata-tagged with category, subject, actor, created_at, approval, and audit event id.

## Storage and logging constraints

### Source files and uploaded documents

- Store immutable source files in governed evidence storage with document id, owner/subject refs, uploader/source, captured_at, hash, malware/quarantine status, extraction status, retention/legal-hold state, and access policy.
- Do not store raw file URLs, signed URLs, thumbnails, or image blobs in ordinary logs or prompt traces.
- Use document refs in dashboards and prompts unless the workflow explicitly needs the file content.
- Quarantine unknown/malicious/unparseable uploads and route extraction/review failures to staff, not AI fallback approval.

### Extracted text / OCR

- Treat OCR/extracted text as `Restricted` unless proven harmless; it may contain PII, medical notes, signatures, invoices, prompt injection, or unrelated content.
- Store extraction results with page/region/source refs, extractor version, confidence, reviewer status, and redaction state.
- Do not log full OCR text in application logs, model-debug logs, or broad audit summaries.
- AI may receive OCR only for assigned document workflows and should return evidence-backed candidates, uncertainty, and review reasons.

### Message bodies

- Treat inbound and outbound message bodies as at least `Confidential`; upgrade to `Restricted` when they include medical, vaccine, behavior, incident, payment, legal/liability, complaint, identity, or internal-policy content.
- Store message bodies in the message/evidence store with channel, recipient/sender refs, consent/preference state, provider status, review/send state, and audit chain.
- Ordinary logs should store message id, template id, channel, status, risk class, and suppression/review reason, not full bodies or recipient PII.
- Customer-message drafts generated by AI must carry source refs, risk flags, review gate, and send prohibition unless an approved deterministic send path exists.

### Staff notes and internal-only notes

- Store staff notes with author, role, subject, workflow/task, visibility label, factual-vs-interpretive classification where possible, created_at, correction/supersession refs, and review state.
- Internal-only notes must be physically or logically separated from customer-visible note fields to avoid accidental portal/export leakage.
- Do not include internal-only notes in prompt examples, customer drafts, customer exports, or broad staff dashboards.
- Redact staff identifiers and sensitive detail outside authorized care/incident/manager packets.

### Incident details and evidence

- Store incident records as case-management subjects with timeline, severity/risk state, involved parties, evidence refs, manager review state, customer-communication state, and closure/reopen events.
- Photos/video/media stay as media/evidence refs with explicit access controls; logs and AI prompts should use media ids/labels unless authorized review requires actual content.
- Legal/compliance/privacy escalation material should be separate from routine care notes and staff task summaries.

### Payment references and provider payloads

- Domain records should store semantic statuses, amounts, due dates, deposit requirements, provider references, provider event ids, reconciliation status, and approval refs.
- Raw provider payloads should be stored only in adapter/evidence storage after signature verification, with retention and redaction policy. They are untrusted until verified, mapped, deduped, and reconciled.
- Never store CVV, raw card/bank credentials, auth tokens, webhook signing secrets, or payment-provider secrets in app DB records, prompts, audit summaries, or logs.
- Logs should use provider event id, provider account id, dedupe key, semantic status, and error class. Mask customer/payment identifiers.

### AI prompts, outputs, and tool traces

- Prompt packets must be canonicalized, classified, minimized, and linked to runtime_call_id, workflow_version, policy refs, source snapshot ids, model_config_ref, idempotency key, and audit refs.
- Prompt packets should include redacted excerpts and evidence refs, not raw provider payloads, raw documents, full message histories, broad customer histories, or unnecessary medical/payment detail.
- Store safe summaries, schema names, validation outcomes, risk flags, action recommendations, review gates, and hashes in ordinary audit projections.
- Store raw prompts/outputs/tool traces only in governed runtime evidence storage if the retention/visibility policy approves that class. Access should be owner/admin/security/compliance scoped.
- Never include secrets, tokens, raw payment instruments, webhook signatures, or credential material in prompts or tool traces.

## Masking and redaction rules

Use these defaults unless a later approved policy is stricter:

- Names: show only when subject identity is needed; otherwise use customer/pet/staff IDs or initials.
- Email/phone/address: show only to roles/workflows that need contact; otherwise mask (`a***@example.com`, last-4 phone, city/state only when enough).
- Emergency/vet contacts: mask outside emergency, medical, document, or incident workflows.
- Medical/medication/allergy/feeding: show exact values only to assigned care/document/incident/manager workflows; otherwise use flags such as `special_care_required` and review refs.
- Vaccine docs/OCR: show verified status and due/missing fields outside document review; mask raw text/images/file URLs.
- Incidents: show case id, status, risk class, required gate, and safe summary in broad views; restrict detailed evidence/timeline to authorized reviewers.
- Payment: show semantic status and amount due only where needed; mask provider refs; never show card/CVV/bank data.
- Staff/internal notes: keep internal-only label; redact from customer-visible views; separate customer-safe summaries.
- AI traces: redact prompt content, tool payloads, PII, medical/payment/incident detail, and internal-only text from ordinary logs; expose hashes, ids, versions, validation status, and risk flags.

## AI runtime exposure rules

1. Every prompt packet must declare a `sensitivity` section: data classification, sensitive fields, customer-text rules, logging rules, allowed output classes, and forbidden copying rules.
2. Include only the fields needed for the workflow:
   - Intake/messaging: contact channel/preference and same-thread context, redacted as possible.
   - Booking: verified statuses, policy refs, semantic payment status, availability/capacity summaries, and review gates; not raw documents/payment payloads by default.
   - Document/vaccine: assigned document/OCR/source evidence and pet/vaccine policy context; not unrelated pet/customer history.
   - Staff-care: assigned care instructions/tasks and relevant risk flags; not broad medical history or internal manager notes unless required.
   - Incident: incident-specific facts/evidence refs and relevant care history; not unrelated customer/staff data.
   - Payment: semantic status/provider refs needed for reconciliation or draft copy; never raw instruments or secrets.
3. AI outputs must be schema-validated. Disallowed actions become `RejectedByPolicy`, `NeedsHumanReview`, or `FailedSafely`, not side effects.
4. AI-generated customer copy must not include internal policy, staff blame, raw OCR uncertainty, provider/payment internals, incident speculation, or internal-only notes.
5. Prior AI summaries may be reused only as audited historical artifacts with source refs; they are not source-of-truth and must not bootstrap new facts without underlying evidence.
6. Prompt injection in customer/provider/staff/OCR text must be treated as a risk flag; user/event content cannot change system/developer policy.

## Data safety risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Uploaded documents include unrelated people/pets, vet PII, signatures, invoice/payment details, or medical notes. | Quarantine and classify uploads; store raw file in governed evidence; use extraction/redaction; expose raw docs only to document reviewers; show document refs/status elsewhere. |
| OCR or message text contains prompt injection telling the AI to ignore policy or reveal data. | Keep OCR/message text in user/event layer; system/developer policy stays separate; require prompt-injection risk flags; reject outputs that follow embedded instructions. |
| AI draft leaks internal-only notes, staff blame, payment/provider internals, raw medical uncertainty, or incident speculation to customers. | Label visibility in prompt; require customer-text rules; schema validation checks forbidden content; human approval for sensitive messages; separate customer-safe summary artifacts. |
| Payment status or provider webhook is trusted before verification/reconciliation. | Verify signatures over raw bodies; store inbound event before processing; dedupe by provider event/account id; map to semantic status; require approval for money movement and customer commitments. |
| Staff dashboards overexpose medical, incident, payment, or internal notes because task lists aggregate too much. | Use role/subject/purpose-scoped projections; default to safe summaries, flags, and review gates; require privileged drill-in audit for raw evidence. |
| Raw prompts/outputs/tool traces become a shadow data store. | Minimize prompts; store raw traces only in governed runtime evidence if approved; log hashes/ids/validation outcomes in ordinary audit; define retention/access gates before production. |
| Customer export or portal accidentally includes staff/internal notes or AI reasoning. | Separate internal-only fields from customer-visible records; export from customer-safe projections; audit export generation; review sensitive classes before release. |
| Source files and extracted text are retained indefinitely without policy. | Preserve retention as an approval gate; track retention class/legal hold on documents, OCR, messages, incidents, prompts/outputs, payment records, provider payloads, and audit logs. |
| AI confidence is treated as medical/vaccine/payment/booking authority. | Require human/trusted-integration verification for medical/vaccine/payment/incident/booking exceptions; validators reject authority claims without approval refs. |
| Engineering/debug logs expose production PII/secrets. | Use synthetic fixtures and redacted logs by default; never log secrets/card data; privileged production debugging requires audited access, narrow time window, and redaction. |

## Audit requirements

Audit these events at minimum:

- Viewing, exporting, downloading, redacting, overriding redaction, or privileged-accessing `Restricted`/`Secret` data.
- Uploading, quarantining, extracting, reviewing, rejecting, verifying, superseding, archiving, deleting, or exporting documents/OCR.
- Creating, editing, correcting, voiding, approving, publishing, or suppressing staff notes, internal-only notes, care notes, incident updates, and customer-safe summaries.
- Prompt packet creation, AI invocation, model/tool result, validation result, rejected/disallowed output, risk flag, review request, and accepted/rejected AI recommendation.
- Provider webhook verification/rejection/dedupe, payment status reconciliation, checkout/request creation, failed payment handling, refund/waiver/discount/forfeit approval/execution.
- Customer-message draft, edit, approval request, approval/rejection, queue, send attempt, delivery failure, suppression, unsubscribe/opt-out, and reply.
- Role/permission changes, policy changes, secret configuration/rotation, retention/legal-hold changes, and production tool-permission changes.

Audit events should store safe summaries and redacted before/after diffs by default. Raw values belong only in governed evidence storage when justified and approved for that category.

## Open approval gates

The following remain unresolved and must be approved before production enforcement:

1. Final role matrix: exact view/create/edit/approve/export/delete/tool permissions for staff, lead staff, manager, owner/admin, payment operator, legal/compliance/privacy, engineering/integration owner, system, and AI workers.
2. Retention and deletion/legal-hold periods for raw documents, extracted text/OCR, message bodies, staff notes, incidents, payment records, raw provider payloads, prompts, outputs, tool traces, media, DLQ records, and audit logs.
3. Which raw values may be stored in governed evidence storage, who may inspect them, and which redacted projections are available to staff, customers, managers, compliance, and AI.
4. Which customer-message paths, if any, may be deterministic auto-send, and which always remain draft/review or never-auto-send.
5. Which AI workflows, if any, may create internal tasks automatically, use read-only tools, or request provider/customer-send actions under deterministic approval.
6. Which provider/PMS/payment integrations are approved source-of-truth systems and which live mutations are allowed in v1.
7. Security incident response process for accidental sensitive-data exposure, prompt leakage, webhook-secret compromise, unauthorized export, or role misconfiguration.

## Conservative implementation rule

When source facts, role authority, retention policy, AI governance, visibility, or redaction state are missing, stale, contradictory, unverified, or outside the declared purpose, the system must choose deny, redact, suppress, route to review, or request more information. It must not silently broaden access, expose raw sensitive data, invent policy, treat AI output as authority, send a customer message, or execute a provider/system mutation.
