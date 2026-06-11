# Pet resort data retention and deletion policy

Status: draft policy candidate for security/audit synthesis. This document is not legal advice and does not approve a production retention schedule. The final retention periods, archival locations, legal-hold rules, customer-deletion behavior, and AI-runtime logging settings require human/legal/compliance approval before production use.

Human approval gate: final retention policy.

Sources used:

- `docs/security/pet-resort-security-audit-parts/inputs.md`
- `docs/architecture/pet-resort-data-model.md`
- `docs/architecture/ai-runtime-memory-context-policy.md`
- `docs/architecture/agent-prompt-packet.md`
- `docs/workflows/vaccine-document-agent-parts/inputs.md`
- `docs/workflows/customer-messaging-parts/inputs.md`
- `docs/workflows/payments-pricing.md`

## Policy goals

Retention policy should let the resort prove operational, medical/vaccine, safety, payment, message, and AI-assisted decisions without keeping unnecessary sensitive data forever.

Default goals:

1. Keep source-of-truth operational records long enough to support customer service, disputes, incident follow-up, payment reconciliation, tax/accounting obligations, and insurance/legal defense.
2. Keep audit events immutable and append-only, with redacted projections for ordinary staff and governed evidence references for privileged review.
3. Minimize raw sensitive data in logs, prompts, provider payloads, messages, OCR, and AI runtime artifacts.
4. Prefer deletion or anonymization of live customer/pet PII when it is no longer needed, while preserving non-identifying tombstones, legal-hold evidence, accounting references, and audit integrity.
5. Treat retention durations as location/jurisdiction/provider policy, not model inference. Unknown policy must produce a review/gate, not silent retention forever.

## Retention tiers

These tiers are proposed defaults for product design. They need legal/compliance approval and may vary by jurisdiction, insurance carrier, payment provider, PMS, tax obligations, and resort policy.

| Tier | Proposed default | Intended use | Deletion/archival posture |
| --- | --- | --- | --- |
| Ephemeral runtime | 0-30 days | Debug traces, transient prompt text, temporary validation errors, queue working payloads, non-authoritative drafts | Delete or compact quickly; keep only manifests, hashes, source refs, and safe summaries when possible |
| Short operational | 90 days after resolution | Routine workflow attempts, transient provider errors, dead-letter diagnostics after remediation, duplicate-detection scratch state | Delete or redact raw payloads after issue is resolved; preserve audit event refs |
| Active relationship | While customer/pet/account is active plus approved grace period | Current profiles, pet care facts, current vaccine facts, active reservations, unresolved messages/tasks/payments | Keep in live stores while operationally needed; archive superseded evidence when no longer active |
| Business records | 3-7 years after transaction/stay/resolution, subject to approval | Payments, invoices/receipts, approved messages, completed reservations, material policy decisions, non-incident service records | Archive or keep in governed business-record store; minimize raw payloads and contact detail |
| Safety/incident/liability | 7 years or longer if approved | Incidents, safety/behavior/medical follow-up, legal threats, injuries, bites, insurance claims, high-risk customer communications | Governed evidence store with legal-hold support; do not hard-delete while hold/dispute/claim window remains open |
| Immutable audit | Minimum 7 years; possibly longer/indefinite with redacted payloads | Audit events for state transitions, approvals, security/admin actions, AI suggestions, messages, payments, provider actions, retention/legal-hold changes | Append-only. Correct by superseding event. Redact projections; preserve integrity refs and hashes |
| Legal hold / dispute hold | Until released by authorized human/legal/compliance role | Chargebacks, payment disputes, complaints, litigation threats, suspected fraud, safety incidents, regulatory inquiries | Suspend deletion/anonymization for scoped subjects and evidence. Release requires audited approval |

## Retention matrix

### Required records

| Record type | Proposed retention tier / period | Legal/business rationale | Deletion/anonymization behavior | Customer deletion impact | Backup/archive handling | Exceptions / holds |
| --- | --- | --- | --- | --- | --- | --- |
| Vaccine documents: raw uploads, images, PDFs, scans, email attachments, provider evidence refs | Active relationship while vaccine proof is current or needed for a reservation; archive superseded documents for Business or Safety tier if used for decisions. Proposed default: keep current proof while pet is active; keep superseded/rejected raw documents 1-3 years unless tied to incident/dispute; longer if legal/insurance policy requires. | Needed to verify vaccine compliance, prove source evidence, resolve customer/provider disputes, and support service eligibility decisions. Medical/vaccine data is sensitive and should not be retained longer than needed. | Store raw files in private governed evidence storage with immutable hash, source, scan status, retention class, and legal-hold flag. Delete raw superseded/rejected files when the approved retention window expires unless referenced by an incident/dispute/legal hold. Keep redacted metadata, hash, document kind, dates, reviewer decision, and audit refs. | If customer requests deletion and no active stay, unpaid balance, dispute, incident, or legal hold exists, remove or anonymize raw document files and unlink owner PII. Keep minimal tombstone/evidence hashes and approved vaccine fact history only if required for audit/business defense. | Backups may contain raw files until normal backup expiry. Restores must reapply deletion/legal-hold tombstones before data returns to live service. Archives must be encrypted, access-controlled, and searchable by retention class. | Do not delete if tied to incident, injury, bite/safety review, service-denial dispute, legal threat, chargeback, insurance inquiry, or active reservation eligibility review. |
| Extracted vaccine facts and policy comparisons | Active relationship for current facts; Business tier for facts used to approve/deny service. Proposed default: keep verified current/expired/superseded facts for active pet plus 3-7 years after last related stay or account closure. | Needed to explain eligibility, verify review decisions, avoid repeated document requests, and prove policy basis for booking/check-in decisions. | Verified facts are domain records with source refs and reviewer refs; do not silently hard-delete if they supported a decision. Supersede or mark expired/rejected. Anonymize owner identifiers when retention permits. | Customer/pet deletion should mark future use disabled and anonymize direct PII, but preserve decision tombstones and source refs needed for historical reservations/audit. | Archived facts keep semantic values, source evidence ids/hashes, policy snapshot refs, reviewer/action audit refs, not raw OCR unless retained under document policy. | Preserve under legal hold, incident/safety, vaccine dispute, service denial/acceptance dispute, or reservation/payment dispute. |
| Incidents and incident follow-up: reports, severity, investigation notes, evidence refs, customer notice drafts/approvals, reopen/closure state | Safety/incident/liability tier. Proposed default: 7 years after closure, or longer if minors, serious injury, insurance, jurisdictional, or legal policy requires. | Incidents can involve animal/human injury, behavior restrictions, medical/safety facts, liability, insurance, customer complaints, and service eligibility. | Never hard-delete during open investigation or legal hold. Close by status transition; correct by superseding note/event. After retention expires, anonymize customer/pet identifiers where allowed and keep aggregate safety pattern data only if de-identified. | Customer deletion cannot remove incident evidence while open, disputed, legally held, insurance-relevant, or within approved liability retention. Future marketing/contact should stop; operational account may be archived. | Store raw evidence/media/docs in governed evidence storage; archive closed incidents with restricted access. Backups must honor legal holds and deletion tombstones on restore. | Legal threats, insurance claims, chargebacks connected to incident, staff injury, bite/aggression events, suspected neglect/abuse, medication/medical errors, or unresolved customer complaint suspend deletion. |
| Incident follow-up tasks and internal review packets | Same as linked incident; otherwise Business tier for ordinary task completion evidence. Proposed default: 7 years for incident-linked tasks, 1-3 years for routine non-incident tasks. | Needed to prove follow-up, manager review, customer-contact decisions, and closure basis. | Delete routine task working notes after retention; preserve completion status, actor, timestamp, evidence refs, and audit event. Incident-linked follow-up inherits incident retention. | Customer deletion does not remove incident-linked follow-up until hold/retention expires. Routine tasks may be anonymized with the customer account. | Archive as part of incident or task/audit store; raw notes redacted in broad staff views. | Same as incident; also staff/admin/legal hold. |
| Messages: inbound customer messages, outbound drafts, approved sends, delivery status, replies, suppression reasons | Business tier for approved/sent business communications; Short/ephemeral for unapproved drafts. Proposed default: sent/received material customer communications 3-7 years; routine unsent drafts 30-90 days after superseded unless needed for audit; incident/payment/legal messages inherit longer linked tier. | Needed for customer-service continuity, proof of notice, consent/suppression, payment reminders, incident notices, and chain of custody for AI/staff drafts. | Approved sends are immutable message records; edits create new drafts/approvals. Delete or redact unapproved drafts quickly. Keep provider message ids/statuses and safe summaries when raw body retention expires. | Customer deletion should stop future contact, remove marketing/nonessential message bodies when allowed, and anonymize contact details. Preserve transactional/incident/payment/legal messages required for business/audit. | Backups retain messages until backup expiry; archives should separate body content from metadata so bodies can be redacted while preserving send proof. | Preserve if linked to incident, dispute, payment/chargeback, cancellation/no-show/refund, consent/opt-out dispute, legal complaint, or safety issue. |
| Payment/payment status references: payment requests, checkout ids, provider ids, semantic statuses, receipts/invoices, refund/waiver/discount/forfeit approvals, disputes | Business records tier. Proposed default: 7 years after transaction/reservation/accounting period, or per tax/accounting/payment-provider rules. Disputes/chargebacks held until final plus approved period. | Needed for accounting, tax, reconciliation, customer support, disputes, chargebacks, refund/waiver evidence, and fraud controls. App must not store raw card/bank/CVV/payment tokens. | Keep semantic amounts/statuses/currency/provider refs/approval refs. Delete raw provider payloads after mapping unless required for dispute/audit and approved. Never store raw card data. Redact customer contact where possible after account deletion. | Customer deletion cannot remove accounting records, payment refs, receipts, disputes, or approval audit while required by law/business policy. Anonymize nonessential PII and stop account use. | Payment archives encrypted and access-controlled. Provider raw payload archives, if any, must be short-lived and separately approved. Restore process must not resurrect deleted customer PII outside required accounting refs. | Chargebacks, payment disputes, suspected fraud, refund/waiver/discount dispute, tax/accounting audit, legal hold, provider reconciliation conflict. |
| Audit logs: entity transitions, approvals, before/after summaries, AI suggestions, validation, provider actions, messages, payment events, retention/legal-hold changes, security/admin events | Immutable audit tier. Proposed default: minimum 7 years; security/admin, legal-hold, and incident audit may be longer/indefinite with redacted payloads if approved. | Required to prove who/what/when/why, review gates, source evidence, policy versions, and chain of custody. Supports security investigations, compliance, dispute resolution, and model/runtime accountability. | Append-only. Do not hard-delete ordinary audit rows. Corrections are later events. Store redacted before/after summaries, hashes, evidence refs, and field paths rather than raw sensitive values. If law requires erasure, pseudonymize subject identifiers while preserving event integrity where permitted. | Customer deletion should create an audit event and anonymize/pseudonymize customer/pet identifiers in general projections when allowed. Immutable core audit may keep tombstone subject ids and hashes needed for integrity and legal/business defense. | Audit archives should be WORM/append-only or tamper-evident where feasible. Backups must preserve audit integrity and deletion tombstones. | Audit records under legal hold, security incident, admin misuse investigation, payment dispute, incident/safety review, or provider reconciliation cannot be removed until release. |
| Deleted customers and pets: tombstones, deletion requests, anonymized identifiers, merge/supersession refs, subject ids used in audit/history | Tombstone/immutable audit tier. Proposed default: tombstone indefinitely or for audit retention period; no raw profile PII beyond approved business/legal need. | Needed to prevent orphan records, duplicate recreation mistakes, audit breaks, re-linking payments/incidents/messages, and accidental contact after deletion/do-not-contact. | Soft-delete/archive active profile first. After retention review, hard-delete or anonymize direct PII from live profile fields while preserving tombstone id, deletion timestamp, reason, actor, legal-hold state, merge/supersession refs, and hashes/refs. | Customer deletion is implemented as archived/deleted state plus PII minimization, not immediate cascade hard-delete of all dependent records. Pet deletion similarly disables future service use while preserving historical stay, incident, vaccine, and audit references as policy requires. | Backups must not reanimate deleted profiles as active. Restore/import jobs must check tombstones and reapply deletion/anonymization. | Preserve raw or identifiable fields only under legal hold, payment/accounting, incident/safety, active reservation, unresolved message/payment dispute, or required audit period. |

### AI-runtime artifacts

| Artifact | Proposed retention tier / period | Rationale | Deletion/anonymization behavior | Customer deletion impact | Backup/archive handling | Exceptions / holds |
| --- | --- | --- | --- | --- | --- | --- |
| Prompt packets / runtime context | Ephemeral by default. Proposed default: do not retain raw prompt text for production sensitive workflows; retain prompt manifest/hash/field categories/source refs for audit 7 years with audit event. If raw prompt retention is approved for debugging, keep 7-30 days max by default. | Need enough to audit model decisions without creating a second sensitive data lake. Prompts may contain customer PII, pet medical/vaccine/incident/payment facts, message excerpts, and staff notes. | Store manifest, canonical hash, workflow/version, subject refs, policy refs, redaction profile, data categories, and model config ref. Delete raw text quickly or never store it unless approved. | Raw prompt text containing customer/pet data should be deleted/anonymized with customer deletion unless under legal/business hold. Manifests/audit refs may remain pseudonymized. | Backups with raw prompts must expire quickly; raw prompt archives require explicit approval and access controls. | Preserve scoped prompts only for legal hold, security incident, model incident investigation, dispute over AI-assisted decision, or regulator/legal request. |
| Structured AI outputs / extracted suggestions / recommended actions / risk flags | Business/audit tier when persisted as workflow results; ephemeral/short for rejected malformed outputs. Proposed default: valid workflow result retained with linked subject record/audit period; invalid raw output 30-90 days as redacted error unless needed. | Outputs explain AI contribution, review gates, source refs, and why an action became a draft/review packet. AI output is not source truth but may be evidence of process. | Persist structured result envelope, status, citations, risk flags, validation outcome, human review reason, and safe summary. Avoid raw completion text when structured output exists. Supersede/correct rather than mutate. | Delete/anonymize outputs that are not tied to retained business/audit decisions. Retained outputs should pseudonymize subject identifiers where allowed after deletion. | Archive with workflow/audit records; keep raw completions separate and short-lived if retained. | Preserve if tied to incident, payment, vaccine/eligibility dispute, customer-message dispute, safety/legal issue, or AI incident. |
| Validation errors, policy denials, malformed outputs, dead-letter diagnostics | Short operational tier. Proposed default: 90 days after remediation; keep safe aggregate counters longer. | Needed for debugging, safety, and proving unsafe outputs did not drive side effects. | Store safe error class, schema/version, validator, workflow ids, redacted reason, and retry/dead-letter state. Delete raw payloads, prompt fragments, and sensitive body text quickly. | Customer deletion should remove raw diagnostic content and anonymize subject refs unless hold applies. | Backups expire normally; restored diagnostics must respect retention expiry. | Preserve for security incident, model/provider incident, legal dispute over failed/suppressed action, or regulator/legal request. |
| Run metadata: model/provider/runtime, agent spec, tool permissions, timestamps, costs/token counts, idempotency keys, source refs | Audit/business tier. Proposed default: 7 years for workflow decisions; 1-3 years for purely operational non-customer runs. | Needed to prove runtime configuration, policy version, allowed tool set, and reproducibility/accountability. Usually less sensitive if it excludes raw data. | Keep metadata and hashes/refs. Do not include secrets, prompt text, raw provider payloads, or raw payment/card data. | Pseudonymize subject ids after deletion where allowed; preserve operational/accounting/security audit refs. | Archive with audit store; restore keeps tombstones. | Legal/security/AI incident holds. |
| Drafts: message drafts, task drafts, reservation/payment/vaccine suggestions not approved | Ephemeral/short unless approved or acted on. Proposed default: delete unapproved superseded drafts after 30-90 days; approved/applied drafts become business/audit records under linked type. | Drafts often contain sensitive language, mistakes, or unapproved claims. Need temporary review but not indefinite retention. | Mark superseded/rejected; delete body after window; keep safe summary, author/AI refs, review outcome, and audit event. Approved drafts are immutable approved payloads. | Delete/anonymize unapproved drafts on customer deletion unless hold applies. Approved transactional/incident/payment drafts may remain under linked retention. | Backups expire; archives should separate body text from metadata. | Preserve if disputed, linked to incident/payment/legal complaint, or needed to explain rejected/approved message. |
| Embeddings / vector indexes / memory | Disabled for customer/pet/reservation/medical/vaccine/incident/payment/message/staff data by default. If approved later, retain only tenant/location-scoped policy/SOP or de-identified product-learning memory with explicit review date. | Prevent open-ended LLM memory from becoming an uncontrolled source of truth or privacy leak. | Do not embed raw customer/pet/message/payment/medical records by default. If embeddings are used for approved documents, store source refs, retention class, deletion path, and reindex/delete support. | Customer deletion must delete any embeddings derived from that customer/pet or prove none exist. If deletion cannot be guaranteed, do not use embeddings for personal data. | Vector backups/index snapshots must support deletion propagation and retention expiry. | Legal hold may preserve source records but should not require retaining broad embeddings unless approved and scoped. |
| Tool-call requests/results and provider raw payloads used by AI/runtime | Ephemeral/short for raw payloads; audit/business for semantic mapped results. Proposed default: raw tool I/O 0-30 days unless approved; semantic result/audit refs follow linked record retention. | Tool traces can expose secrets, raw provider data, payment refs, message bodies, and medical/vaccine data. | Store allowed action, tool name/version, subject refs, status, redacted result summary, policy decision, and idempotency key. Do not store secrets, tokens, raw card/bank data, webhook signatures, or unnecessary provider JSON. | Delete raw traces and anonymize subject refs on deletion unless legal/business hold applies. | Backups containing raw traces expire quickly; archives keep semantic refs only. | Security incident, provider reconciliation dispute, payment/incident/legal hold, AI/tool incident. |

## Soft delete, hard delete, and tombstones

### Customers

Customer deletion should be a staged workflow, not an immediate cascade.

1. Request/approval record
   - Create an immutable audit event for deletion request, requester, approving actor, scope, reason, legal-hold check, and policy version.
   - Check active reservations, unpaid balances, payment disputes, open incidents, unresolved messages, legal holds, and retention exceptions.

2. Soft-delete / archive
   - Set `Customer.profile_state` to `Archived`, `DeletedRequested`, `Deleted`, `DoNotContact`, or equivalent approved enum.
   - Disable portal/session access and future marketing/contact.
   - Suppress outbound automation and provider writes unless required by legal/business workflow.
   - Preserve links to pets, reservations, messages, payments, incidents, documents, and audit through subject refs.

3. PII minimization / anonymization
   - Remove or pseudonymize name, email, phone, address, emergency contacts, vet contacts, portal refs, preferences, and free-text profile fields when no longer required.
   - Keep minimal tombstone: customer id, deletion timestamp, actor, reason category, legal-hold status, merge/supersession refs, hash of former identity if approved for duplicate prevention, and audit refs.
   - Maintain do-not-contact/suppression tombstone if needed to avoid accidental re-contact.

4. Hard-delete only when safe
   - Hard-delete raw profile PII and nonessential drafts/logs once retention/legal checks pass.
   - Do not hard-delete historical payment records, incident evidence, approved messages, audit events, or reservation records still under required retention. Instead anonymize direct PII and preserve tombstone links.

### Pets

Pet deletion/archive follows similar rules but must preserve historical service, vaccine, care, and safety context.

1. Set pet state to `ArchivedInactive`, `Deleted`, `MergedSuperseded`, or approved equivalent.
2. Disable future booking/readiness use and group-play/care automation.
3. Remove or pseudonymize pet name, photo/media, breed/age/sex/weight, care profile, medication/allergy/medical/free-text details where retention permits.
4. Preserve historical reservation, incident, vaccine decision, payment, message, and audit references as required.
5. Keep a tombstone with `PetId`, owner/customer tombstone ref, deletion/archive timestamp, legal-hold status, merge/supersession refs, and audit refs.

### Dependent records

- Reservations: retain as business records; anonymize customer/pet profile fields when possible while preserving service dates, statuses, policy refs, payment refs, and audit refs.
- Documents: delete raw files according to document retention; preserve hashes/metadata/review refs if needed for audit.
- Vaccine records: supersede/expire/anonymize rather than erase facts that supported a retained decision.
- Messages: delete nonessential draft bodies; retain approved transactional/safety/payment/legal messages under linked retention.
- Payments: retain accounting/provider refs and semantic statuses; remove nonessential PII; never store raw card/bank data.
- Incidents: retain under safety/legal tier; anonymize only after hold/retention expires and approval permits.
- Audit events: append-only; delete only if legally required and then via explicit redaction/pseudonymization event, not silent row removal.
- AI artifacts: delete raw prompts, raw completions, drafts, raw tool traces, embeddings, and validation payloads unless approved retention/hold applies; retain manifests/hashes/audit refs.

## Backup, archive, and restore rules

1. Backups are not a loophole for retention.
   - Backup retention must have its own approved window.
   - Restores must reapply deletion requests, tombstones, legal holds, and anonymization before restored data is exposed to users or workers.

2. Archives must be purpose-scoped.
   - Business archives: reservations, payments, approved messages, policy snapshots, and semantic audit refs.
   - Evidence archives: vaccine documents, incident evidence, raw message bodies, raw provider payloads only when approved.
   - Security/audit archives: append-only audit events, admin/security events, access/export events, retention/legal-hold events.

3. Raw sensitive archives require higher controls.
   - Encryption at rest, strict role-based access, tenant/location scoping, access audit, export logging, retention class, deletion/hold metadata, and periodic review.

4. Deletion propagation must be testable.
   - Every raw object, OCR artifact, embedding, prompt log, draft body, message body, provider payload, and evidence blob needs a retention class and deletion path.
   - Retention jobs should produce audit events for deletion, anonymization, skipped-due-to-hold, and restore/replay actions.

## Audit-log immutability and redaction

Audit logs are append-only by default.

Rules:

- Record the event, actor, subject, source, policy/version, before/after safe summary or redacted diff, approval refs, evidence refs, AI suggestion refs, provider refs, and idempotency key.
- Do not store secrets, raw payment instruments, webhook signatures, full raw documents, raw OCR, or unrestricted message bodies in ordinary audit rows.
- Use governed evidence refs for raw materials and redact broad staff projections.
- Corrections, reversals, and erasure/anonymization actions are new audit events that reference prior events.
- Customer deletion creates an audit event and may pseudonymize subject identifiers in role-scoped projections, but it should not silently rewrite the event chronology.
- Privileged audit access, export/download, redaction override, legal-hold set/release, and retention-policy changes are themselves audit events.

## Legal hold and exception handling

Deletion, anonymization, and archival jobs must check for scoped exceptions before acting.

Holds/exceptions include:

- Open or recently closed incidents, injury, bite/aggression, medical/safety concern, medication/care error, insurance claim, or legal threat.
- Payment dispute, chargeback, refund/waiver/discount/forfeiture disagreement, suspected fraud, provider reconciliation conflict, tax/accounting inquiry.
- Customer complaint, consent/opt-out dispute, contested message/customer notice, service acceptance/denial dispute, vaccine/eligibility dispute.
- Security/admin incident, unauthorized access, data exposure, role/permission misuse, credential/webhook compromise, model/tool incident.
- Active reservation, unresolved task, unread/unsent approved customer communication, pending review, active provider sync issue.

Required behavior:

1. Place a scoped legal/dispute/safety hold with subject ids, evidence refs, actor, reason, approving role, review date, and policy version.
2. Suspend hard-delete/anonymization only for records in scope; do not turn a narrow hold into indefinite retention of unrelated data.
3. Allow deletion of unrelated ephemeral/runtime data when it is not required for the hold.
4. Release holds only through an audited approval event.
5. After release, resume normal retention evaluation rather than deleting blindly.

## Operational retention controls to implement

Minimum fields each retained record should carry where applicable:

- `retention_class`
- `retention_policy_ref` and version
- `retain_until` or review date
- `legal_hold_status` and scoped hold ids
- `deletion_state`: active, archived, deletion_requested, anonymized, deleted, skipped_due_to_hold
- `subject_refs`: customer, pet, reservation, incident, payment, message, document, workflow, audit ids
- `source_refs` and evidence hashes
- `created_at`, `updated_at`, `superseded_at`, `deleted_at/anonymized_at`
- `deleted_by/anonymized_by` actor or system job id
- `redaction_profile`
- `backup_restore_tombstone_version` where applicable

Retention jobs should:

1. Select candidates by retention class and `retain_until`.
2. Check legal holds, active workflows, active reservations, payment/incident/message exceptions, and provider sync state.
3. Delete raw objects first only when metadata/audit can prove safe deletion.
4. Anonymize PII in live projections and secondary indexes.
5. Delete embeddings/vector rows derived from deleted subjects.
6. Compact or delete raw AI/runtime traces.
7. Emit audit events for actions and skipped records.
8. Produce review reports for failures, orphaned evidence, unclassified records, and raw payloads without retention metadata.

## Open legal/compliance questions

1. What jurisdictions apply to the pilot and production resorts, and do state privacy, veterinary/animal-care, consumer-protection, biometric/media, or employment laws impose specific retention/deletion periods?
2. What insurance carrier or franchise/operator policy governs incident, injury, bite/aggression, medication/care error, and liability records?
3. What is the legally approved retention period for vaccine/medical documents and extracted vaccine facts after a pet becomes inactive or a customer requests deletion?
4. Which vaccine records count as medical/veterinary records versus operational eligibility evidence, and who may approve destruction?
5. What accounting/tax retention period applies to invoices, deposits, refunds, waivers, discounts, forfeitures, payment-provider refs, and receipts?
6. What payment provider/PMS will be used, and what are its required webhook, dispute, chargeback, receipt, and raw-payload retention rules?
7. May audit logs be retained indefinitely if they contain only pseudonymized subject refs, hashes, and safe summaries, or must they have a fixed destruction date?
8. What customer deletion rights will the product offer contractually, and what records are exempt for accounting, safety, legal, insurance, fraud, and audit reasons?
9. What exact behavior is required for deletion from backups and object-storage version history? Is delayed deletion through backup expiry acceptable?
10. What legal-hold roles are approved, and who may set/release holds for incidents, disputes, chargebacks, security events, or regulator/legal requests?
11. Are raw AI prompts, raw completions, tool traces, and provider payloads allowed to be retained for debugging at all in production? If so, for how long and who can inspect them?
12. Are embeddings/vector indexes allowed for any customer/pet/reservation/message/vaccine/incident/payment content, or must vector memory be limited to approved SOP/policy knowledge?
13. What customer-visible retention/deletion disclosures, privacy policy text, and staff SOPs are required before production launch?
14. What access/export audit must be shown to customers, managers, compliance, or regulators, if any?
15. What incident-response retention override applies after suspected data exposure, prompt leakage, webhook secret compromise, unauthorized role change, or model/tool misbehavior?

## Approval checklist for final policy

Before production, approve and record:

1. Retention periods for each matrix row and jurisdiction/location.
2. Backup and archive retention windows.
3. Legal-hold roles, reasons, workflow, and release process.
4. Customer/pet deletion workflow, allowed exemptions, and tombstone contents.
5. Raw document/OCR/provider/message/prompt/tool-trace retention limits.
6. Whether raw AI prompt/completion logging is disabled or allowed in narrow workflows.
7. Whether embeddings/memory are disabled or allowed for specific de-identified/approved categories.
8. Audit-log immutability, redaction, pseudonymization, and retention period.
9. Staff/admin/compliance access matrix for raw evidence, redacted projections, exports, and deletion jobs.
10. Test plan proving deletion/anonymization/hold/restore behavior across live stores, object storage, queues, search indexes, vector stores, logs, backups, and archives.
