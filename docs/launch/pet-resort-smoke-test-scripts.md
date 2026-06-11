# Pet Resort launch smoke test scripts

Status: launch-readiness definition artifact. These scripts/checklists do not authorize production deployment, live customer messaging, payment movement, provider/PMS mutation, booking confirmation/rejection, vaccine acceptance, incident disposition, or live-customer pilot execution. They define what a reviewer should run after the MVP implementation board produces runnable local/demo software and explicit pilot approvals.

Source inputs consulted:

- `docs/launch/pet-resort-launch-readiness.md`
- `docs/roadmap/pet-resort-mvp-implementation.md`
- Current implementation-board status from `pet-resort-mvp-implementation`: `t_8986d4e5` has produced `docs/roadmap/pet-resort-mvp-stack.md` and is blocked `review-required` for human stack/MVP-cutline approval; project skeleton and all code implementation cards remain `todo`; final MVP smoke card remains `todo` and dependency-gated.
- `docs/security/pet-resort-security-audit.md`
- `docs/workflows/inquiry-intake-agent.md`
- `docs/workflows/booking-triage-agent.md`
- `docs/workflows/vaccine-document-agent.md`
- `docs/workflows/daily-care-update-agent.md`
- `docs/workflows/incident-escalation-agent.md`
- `docs/workflows/customer-messaging-agent.md`
- `docs/workflows/crm-retention-agent.md`
- `docs/workflows/staff-operations.md`
- `docs/workflows/payments-pricing.md`

## 1. Execution modes

### 1.1 Local/demo smoke execution

Use this mode by default until a human explicitly approves a pilot mode and stack/cutline gate.

Allowed inputs:

- Synthetic or de-identified customers, pets, reservations, vaccine documents, notes, incidents, and messages.
- Local/dev database, local object storage, local mail/SMS/portal stubs, and deterministic fixture accounts.
- Draft-only AI/provider adapters or mock adapters that cannot reach production providers.
- Staff/manager demo users with known roles and no real-customer scope.

Allowed effects:

- Create local records, tasks, drafts, review packets, audit events, and mock outbox rows.
- Exercise local UI paths, workflow events, validators, idempotency, suppression, and audit projections.
- Verify that no customer/provider/payment action occurs without an approved stubbed action.

Forbidden effects:

- No live customer sends.
- No production PMS/Gingr/provider writes.
- No live payment links, charges, refunds, discounts, credits, waivers, forfeits, or manual payment-status corrections.
- No accepted vaccine/medical/behavior/incident/eligibility state based only on AI/OCR/free text.
- No raw secrets, raw provider payloads, raw vaccine files, raw incident narratives, payment data, or hidden prompts in ordinary logs, client bundles, screenshots, or audit projections.

### 1.2 Limited live-pilot smoke execution

This mode is disabled unless the pilot-mode artifact and a human approval record explicitly name the location, dates, staff owners, consenting/test recipients, allowed workflows, data classes, and rollback owner.

Additional requirements before any limited live-pilot run:

- Stack/MVP cutline approved.
- Pilot launch approved by a human; live-customer use approved if any real customer is involved.
- Messaging policy approved for the exact recipient/channel/template/fact set, or messages remain draft-only and are sent manually outside automation after human review.
- Payment/provider mutation gates remain disabled unless separately approved.
- All live-pilot records are tagged with pilot run id, actor, location, approval id, and audit correlation id.
- Outbound kill switch and rollback/fallback path are verified before the run.

Live-pilot extra forbidden effects:

- No autonomous rejection, cancellation, booking confirmation, waitlist promotion, eligibility clearance, incident severity finalization, owner notice, review request, or payment action.
- No contact with non-consenting customers or unlisted recipients.
- No use of real medical/vaccine/incident/payment/raw-provider evidence in prompts/logs beyond approved, minimized, redacted prompt packets.

## 2. Global pre-flight checklist

Run before each script.

1. Confirm execution mode is `local_demo` unless a human approval id explicitly permits `limited_live_pilot`.
2. Confirm environment banner shows non-production mode, production provider writes disabled, live outbox disabled or approval-gated, and payment provider commands disabled.
3. Confirm the logged-in actor has the expected role: staff, lead staff, manager/admin, or system/AI worker.
4. Confirm seed data uses non-real names/contact destinations in local/demo mode.
5. Confirm all outbound destinations resolve to stub sinks or approved test recipients.
6. Confirm object-storage evidence refs are private and ordinary UI/logs show ids/hashes/safe summaries, not raw storage keys or raw sensitive content.
7. Confirm prompt packets contain least-privilege fields, sensitivity markers, allowed/forbidden actions, source refs, policy refs, and idempotency keys.
8. Confirm audit event capture is enabled and append-only for source ingest, AI run, validation, review packet, draft, approval request, suppression, and any attempted side effect.
9. Confirm rollback controls are visible: pause agents, hold outbound queue, disable provider commands, and continue manual staff workflow.

Global pass criteria:

- Pre-flight evidence is attached to the smoke run record.
- No production secrets or raw sensitive content appear in browser console, network logs, server logs, audit summaries, downloadable reports, or AI prompt safe logs.
- Any disabled or unapproved action fails closed with a visible review/suppression/blocked state and audit event.

Global fail criteria:

- Any live send/provider/payment effect occurs without explicit approval.
- Any secret, raw token, webhook signature, storage key, raw payment data, raw hidden prompt, or unrelated customer/pet data appears in ordinary logs/UI/client bundles.
- Any AI output is treated as final authority for booking, vaccine, care, behavior, incident, payment, or customer-message state.

## 3. Smoke script 1 — Happy boarding

Path: inquiry/request -> reservation review -> staff dashboard -> care note/update -> checkout-ready summary.

### Setup data

Local/demo fixture:

- Customer `Jordan Demo`, email `jordan.demo@example.test`, SMS `+15550101010`, consent state `demo_allowed`, no DNC.
- Pet `Milo`, dog, neutered, no behavior restrictions, routine feeding instructions, no medication.
- Location `demo-location-1`, timezone fixed, one available boarding accommodation, staff coverage present.
- Vaccine status already human-reviewed/verified in fixture, with proof refs and reviewer audit event.
- Service request: dog boarding for two future nights, standard accommodation, no special add-ons.
- Staff actors: front desk `fdemo`, care staff `cdemo`, manager `mdemo`.

Limited live-pilot variant:

- Use only a named consenting customer/test account and approved test pet/reservation, or staff-owned test household.
- Treat any provider reservation as read-only unless the live-provider mutation gate is separately approved.

### Steps

1. Submit a boarding inquiry/request through the local public/staff form or API.
2. Verify `inquiry.received` event and inquiry-intake result show extracted owner, pet, service, dates, missing-info state, and draft reply.
3. Open reservation/triage review. Run deterministic booking rules for date/service, availability, vaccine, staff coverage, behavior, special care, and payment/deposit readiness.
4. As staff, mark the request `ready_for_staff_approval` or equivalent review-ready state; do not auto-confirm.
5. Open staff dashboard today/reservation view. Confirm customer, pet, reservation, tasks, vaccine/document state, and audit history are visible through role-scoped safe projections.
6. Add routine care note: meal eaten, settled well, play/rest observed, no concerns, no photo required unless fixture includes approved media ref.
7. Generate daily update draft and preview it.
8. Move stay to checkout-prep/checkout-ready summary through local/demo staff action or mock provider state. Generate a checkout-ready summary and CRM/review candidate packet.

### Expected UI outcomes

- Inquiry appears in intake/lead queue with safe source summary and draft reply, not sent.
- Reservation review shows deterministic rule results separately from AI explanations.
- Staff dashboard shows arrival/active/departure views, task queue, pet profile, reservation view, daily care evidence, and audit-visible staff actions.
- Daily update preview is warm, factual, source-backed, and marked `requires_approval=true` unless exact deterministic send policy is approved.
- Checkout-ready summary separates customer-safe summary from internal notes and unresolved payment/incident/document flags.

### Expected data and audit outcomes

- Events: `inquiry.received`, booking triage result, staff review action, staff note, daily update draft, checkout-prep/summary, CRM/review candidate.
- Audit captures actor, subject refs, policy refs, source evidence refs, AI run refs, validation result, approval requirements, before/after safe summaries, and idempotency keys.
- Draft reply, daily update, and review request candidate have draft ids, categories, recipient refs, suppression/consent checks, and no outbox send unless approved.

### Forbidden side effects

- No automatic booking confirmation, provider write, room assignment, capacity hold/release, payment request, or live customer send.
- No claim that the stay is confirmed unless source-backed by staff/provider approval evidence.
- No review request send; only a candidate or draft.

### Pass criteria

- Full local path can be completed with all customer-facing text draft/review-gated.
- Audit trail links every state change and draft to source evidence and actor.
- Re-running the same inquiry fixture dedupes or creates a safe replay result without duplicate sends/tasks beyond idempotency policy.

### Fail criteria

- AI output alone confirms/reserves/charges/sends.
- Staff dashboard exposes raw internal notes or unrelated subject data.
- Checkout/review packet ignores suppression, unresolved incidents, payment exceptions, or missing approval gates.

## 4. Smoke script 2 — Missing vaccine

Path: upload/request state -> extraction/review queue -> staff/customer-safe draft -> eligibility remains gated.

### Setup data

Local/demo fixture:

- Customer `Riley Demo`, pet `Biscuit`, dog boarding request in `vaccine_pending` or `booking_request` state.
- Vaccine document fixture: synthetic PDF/image missing Bordetella expiration or containing OCR ambiguity.
- Vaccine policy snapshot for dogs requiring Rabies, DHPP, Bordetella, and CIV for boarding; policy status marked draft/approved-for-demo only.
- Document reviewer role and manager role available.

Limited live-pilot variant:

- Use a test/consenting account and a non-sensitive de-identified sample when possible. If real vaccine evidence is used, verify the pilot approval permits this exact evidence handling and raw evidence is private.

### Steps

1. Upload or attach the vaccine document through the document intake path.
2. Verify private storage metadata: document id, evidence id, hash, source, uploader actor, retention/access policy refs, and no raw storage key in ordinary UI/logs.
3. Trigger `vaccine_document.uploaded` extraction.
4. Open document review queue and inspect extraction candidates, policy comparison, uncertainty, missing inputs, and review packet.
5. Generate staff/customer-safe missing-vaccine or reupload draft through messaging workflow.
6. Attempt to advance booking eligibility without reviewer approval.
7. As document reviewer, optionally record `needs_more_info` or `rejected/pending` in local/demo mode; do not accept unless fixture has complete source-backed proof and manual approval.

### Expected UI outcomes

- Document queue shows extraction candidates as unverified suggestions.
- Review packet highlights missing/ambiguous vaccine facts and exact review gate `MedicalDocumentReview`.
- Customer draft asks for missing/updated proof in safe language and is approval-gated.
- Reservation remains `vaccine_pending` or blocked from confirmation/check-in/group-play clearance until approved review evidence exists.

### Expected data and audit outcomes

- Audit events for upload/import, quarantine/scan, extraction run, schema validation, review requested, draft created, failed/blocked eligibility attempt, and any reviewer decision.
- Vaccine facts are stored as candidates with source refs and confidence/uncertainty, not verified compliance records.
- Eligibility recompute only occurs after approved reviewer decision and remains auditable.

### Forbidden side effects

- No OCR/AI auto-acceptance.
- No raw vaccine document, OCR text, storage key, contact PII, or prompt text in ordinary logs/client bundles.
- No customer send unless exact approval path is completed.

### Pass criteria

- Missing/ambiguous proof routes to review and blocks eligibility-affecting states.
- Reviewer decision and blocked action are auditable.
- Customer-safe draft excludes medical overreach and internal uncertainty details.

### Fail criteria

- Upload success or high confidence marks vaccines verified.
- Booking can be confirmed/checked in solely from extraction output.
- Raw medical document content leaks into general staff dashboard, logs, or customer draft.

## 5. Smoke script 3 — Full dates / no capacity

Path: capacity/date conflict -> waitlist/rejection draft -> no autonomous rejection unless approved.

### Setup data

Local/demo fixture:

- Customer `Sam Demo`, pet `Luna`, dog boarding request for a known full/blackout date range.
- Capacity snapshot showing zero compatible accommodations or staff coverage for at least one requested night, with freshness timestamp and policy refs.
- Waitlist policy configured as either `allowed_for_demo` or `unknown` to test both branches.
- No manager approval initially.

Limited live-pilot variant:

- Use only pilot-approved request data. Do not mutate live capacity, waitlist, reservation, or provider status.

### Steps

1. Submit request for full dates.
2. Run booking triage deterministic rules for date/service support, accommodation availability, staff coverage, holiday/blackout, and waitlist policy.
3. Inspect AI explanation and draft customer-safe waitlist/alternative-date or rejection language.
4. Attempt to auto-reject, auto-waitlist, or send denial language without approval.
5. As manager in local/demo mode, approve a waitlist route or return for alternate dates; verify the resulting draft remains approval-gated for send.

### Expected UI outcomes

- Triage result shows `waitlisted`, `special_review`, `missing_policy`, or `hard_block` as a recommendation/review state, not an executed rejection.
- Staff sees capacity evidence, stale/unknown flags, waitlist/rejection gates, and manager-review task.
- Customer draft avoids promises, priority claims, definitive denial, or availability commitments unless source-backed and approved.

### Expected data and audit outcomes

- Audit events for capacity snapshot used, deterministic rule outputs, manager review task, draft creation, blocked attempted side effect, and optional manager approval.
- Idempotency prevents duplicate manager tasks/outbound drafts on replay.

### Forbidden side effects

- No autonomous rejection, waitlist promotion, capacity hold/release, room assignment, provider mutation, or customer send.
- No invented local capacity/holiday/waitlist policy.

### Pass criteria

- Full/no-capacity request routes to human review or safe waitlist candidate.
- Any hard-denial or waitlist state transition requires approval evidence.
- Draft uses safe wording and records required approval gates.

### Fail criteria

- System rejects or waitlists customer autonomously without approved path.
- AI invents availability, room counts, waitlist priority, or alternate-date guarantees.

## 6. Smoke script 4 — Special-care pet

Path: medical/feeding/medication/behavior ambiguity -> care/manager review queue.

### Setup data

Local/demo fixture:

- Customer `Taylor Demo`, pet `Cooper`, dog boarding or daycare request.
- Care notes: medication name/dose ambiguity, allergy note conflict, special feeding instruction, anxiety/handling note, or prior behavior flag.
- No current manager/care-team approval.
- Staff dashboard with care review and manager review queues enabled.

Limited live-pilot variant:

- Use only approved test/consenting account. Do not expose raw medical/behavior details in prompt/logs beyond minimized approved packet.

### Steps

1. Submit or import request with special-care ambiguity.
2. Run booking triage and staff dashboard readiness evaluation.
3. Open pet profile/care review queue and inspect flagged missing/stale/conflicting facts.
4. Generate internal care/manager packet and optional customer-safe clarification draft.
5. Attempt to mark pet ready for group play/boarding/special-care acceptance without approval.
6. As manager/care reviewer in local/demo mode, request clarification or approve a limited handling plan; verify approval scope and expiration are recorded.

### Expected UI outcomes

- Reservation/pet appears as `Needs care review`, `Needs manager review`, or `SpecialReview`, not ready.
- Staff dashboard separates internal-only notes, customer-safe copy, manager-only rationale, and source refs.
- Customer-safe draft asks for clarification without diagnosis, blame, unsupported reassurance, or acceptance promises.

### Expected data and audit outcomes

- Review packet lists `CareTeamApproval`, `ManagerApproval`, `BehaviorReview`, or `MedicalDocumentReview` as applicable.
- Blocked attempted readiness/eligibility change emits audit event.
- Any approved handling plan records actor, role, source refs, scope, expiration/review date, and before/after safe summary.

### Forbidden side effects

- No final medical/care truth from free text or AI.
- No group-play/behavior exception, special-care acceptance, medication instruction change, or eligibility update without authorized approval.
- No sensitive medical/behavior wording in customer draft without review.

### Pass criteria

- Ambiguity routes to care/manager review and blocks readiness.
- AI output is explanatory/draft-only and cannot clear the gate.
- Audit preserves source facts and human decision boundaries.

### Fail criteria

- Staff or AI can bypass review with a generic ready flag.
- Customer-facing copy states diagnosis, eligibility, acceptance, or safety guarantee from ambiguous evidence.

## 7. Smoke script 5 — Daily update

Path: staff notes -> AI draft -> preview/edit/approval/audit; no live send by default.

### Setup data

Local/demo fixture:

- Active stay/reservation for pet `Milo` or equivalent.
- Staff notes for meal, play/rest, mood, bathroom, and optional approved media ref.
- One note marked internal-only or restricted-sensitive to verify suppression.
- Messaging destination uses stub sink or approved test recipient only.

Limited live-pilot variant:

- Only run for named consenting/test recipient and with manager/staff approval. Live send remains disabled unless exact template/category/channel/facts approval exists.

### Steps

1. Add staff care notes with classification, visibility state, source provenance, and review state.
2. Trigger `DailyNoteCreated` or `DailyUpdateNeeded` workflow.
3. Inspect AI-generated daily update draft, included/omitted facts, safety checks, media refs, risk flags, and required approvals.
4. Preview draft as staff, edit text, and request approval.
5. Attempt direct send without approval.
6. Approve in local/demo mode only, then verify outbox behavior remains stubbed or approval-bound.

### Expected UI outcomes

- Draft is concise, warm, factual, and source-backed.
- Internal-only/restricted notes are not copied into customer text.
- Preview/edit/approval state and audit history are visible.
- Send action is disabled, stubbed, or approval-gated by default.

### Expected data and audit outcomes

- Staff note events include author, observed/created timestamps, source version/hash, visibility, review state, and evidence refs.
- Daily update output includes customer_message draft, source refs, omitted/suppressed facts, risk flags, validation result, approval requirements, and safe log summary.
- Edit and approval actions append audit events; content edits require new approval/version.

### Forbidden side effects

- No live send by default.
- No invented meals, play, bathroom, photos, reassurance, diagnosis, or care completion.
- No raw staff notes, internal debate, other pet/customer data, payment/provider data, or hidden prompts in customer copy.

### Pass criteria

- Routine notes produce a safe draft and approval packet.
- Sensitive/missing/conflicting notes trigger suppression or review.
- Direct send fails closed without approval and records audit.

### Fail criteria

- Daily update sends automatically in local/demo default.
- Draft includes unsupported claims, internal-only details, or photo claims without approved media ref.

## 8. Smoke script 6 — Incident draft

Path: incident capture -> severity suggestion -> manager review -> owner-message draft; no autonomous severity finalization/send.

### Setup data

Local/demo fixture:

- Active stay/daycare pet `Nala`, customer `Morgan Demo`.
- Incident fixture: minor scrape, bite/aggression candidate, medication issue, or escape near-miss; include one missing or ambiguous required field.
- Staff reporter, lead staff, manager actors available.
- Messaging stub and incident manager queue enabled.

Limited live-pilot variant:

- Do not run with real emergencies. If a real incident exists, the smoke script is not a substitute for operational response; only verify read-only/draft surfaces with explicit manager approval.

### Steps

1. Staff reporter opens incident form and captures observed facts, immediate action already taken, unknowns, evidence refs, and no-diagnosis attestation.
2. Trigger incident/escalation agent.
3. Inspect proposed incident type(s), severity candidate, unknowns, required gates, manager packet, temporary flag candidates, follow-up tasks, and owner-message draft.
4. Verify manager review queue item appears with due/urgency and closure blockers.
5. Attempt to finalize medium/high/emergency severity, clear/apply eligibility flag, close incident, or send owner message without manager approval.
6. In local/demo mode, manager edits/approves draft or returns for more info; verify audit chain.

### Expected UI outcomes

- Incident packet is factual, chronological, and uncertainty-preserving.
- Severity is clearly labeled candidate/suggestion until manager finalizes.
- Owner-message draft requires `CustomerMessageApproval` plus manager/legal/privacy/payment gates when applicable.
- Serious incidents remain open until authorized closure evidence exists.

### Expected data and audit outcomes

- Audit events for report creation, evidence attach, AI run, validation, manager review request, draft creation, blocked attempted side effects, manager decision, and any task/flag candidate.
- Temporary profile/behavior flags remain candidates unless manager-approved; eligibility-affecting state is not mutated by AI.

### Forbidden side effects

- No autonomous severity finalization, downgrade, closure, owner send, provider write, behavior/eligibility flag apply/clear, diagnosis, blame/liability statement, refund/credit promise, or task completion.

### Pass criteria

- Incident routes to manager review with source-backed severity candidate and owner-message draft.
- Blocked side effects fail closed and are audited.
- Customer copy excludes internal-only, legal/liability, diagnosis, unsupported reassurance, and blame language.

### Fail criteria

- AI can close, downgrade, or suppress a serious incident.
- Owner message can send before approval.
- Eligibility flag changes without manager approval.

## 9. Smoke script 7 — Cancellation

Path: customer/staff cancellation path -> task/audit/refund/payment-sensitive manual gate.

### Setup data

Local/demo fixture:

- Customer `Alex Demo`, reservation in offered/confirmed/demo state with deposit/payment status represented by semantic test records only.
- Cancellation request source: customer message or staff action.
- Cancellation/deposit policy snapshot is either approved-for-demo or intentionally missing to verify manager review.
- Payment provider commands disabled; payment records are stubs.

Limited live-pilot variant:

- Use only named test/consenting account and do not execute provider/PMS cancellation or payment/refund commands unless explicitly approved. Manual human process remains source of truth.

### Steps

1. Submit cancellation request through customer message/staff UI.
2. Normalize source event and route to booking/payment review.
3. Evaluate cancellation state, policy snapshot, timing, deposit/payment/refund implications, and customer-message needs.
4. Verify staff task/manager/payment review task is created for refund/waiver/forfeit/payment-sensitive decision.
5. Generate acknowledgement or cancellation draft.
6. Attempt to cancel provider reservation, waive fee, forfeit deposit, refund, or send payment-sensitive message without approval.
7. In local/demo mode, manager/payment reviewer records decision or returns for policy clarification; verify audit.

### Expected UI outcomes

- Cancellation request appears with source evidence, policy status, payment/reconciliation status, and manual gate.
- Draft acknowledgement avoids legal/policy/refund commitments unless approved and source-backed.
- Payment-sensitive controls are manager/payment-gated and visibly disabled or review-only.

### Expected data and audit outcomes

- Audit records cancellation request, policy evaluation, payment status read, review task, draft, blocked attempted payment/provider effects, approval/return decision.
- Payment data is semantic/redacted; no raw card/bank/token/provider secrets appear.
- Reservation lifecycle change is not executed without approved path/source-of-truth evidence.

### Forbidden side effects

- No autonomous provider cancellation, refund, waiver, discount, deposit forfeit, balance adjustment, manual payment status correction, or payment-sensitive customer send.
- No invented cancellation/refund policy.

### Pass criteria

- Cancellation creates review/audit trail and blocks payment-sensitive outcomes.
- Draft is safe and approval-gated.
- Missing policy routes to review instead of guessed refund terms.

### Fail criteria

- System moves money or mutates provider status without approval.
- AI decides refundability, fee waiver, forfeiture, or payment truth.

## 10. Smoke script 8 — Review request

Path: completed safe stay -> CRM/review draft -> suppression/preferences/review gate.

### Setup data

Local/demo fixture:

- Customer `Casey Demo`, completed safe stay with no unresolved incidents, complaints, payment disputes, DNC/opt-out, or unresolved care/document issues.
- Second negative fixture: completed stay with unresolved incident, complaint, DNC/opt-out, failed delivery, or payment dispute to verify suppression.
- Review link/template policy is absent or draft-only unless explicitly approved-for-demo.
- Messaging channels use stub sink.

Limited live-pilot variant:

- Use only approved consenting/test recipients. Live review-request send requires exact approved category/template/channel/recipient/fact policy; otherwise draft-only.

### Steps

1. Mark local/demo stay completed with checkout summary and no unresolved blockers.
2. Trigger CRM/retention evaluation for review request candidate.
3. Verify lifecycle stage, suppression flags, consent/opt-out/quiet-hours/over-contact state, complaint/incident/payment checks, and review-request policy refs.
4. Generate review request draft or suppression/no-action result.
5. Repeat with negative/suppression fixture.
6. Attempt to send review request without approved auto-send category/template/fact/channel policy.

### Expected UI outcomes

- Safe completion fixture produces a review request candidate/draft requiring approval unless exact deterministic policy is approved.
- Negative/suppression fixture produces `suppressed` or `no_send` with reason and next owner, not a draft/send.
- Staff can see source evidence and suppression reasons without raw sensitive incident/payment/internal notes.

### Expected data and audit outcomes

- CRM output records lifecycle classification, suppression flags, evidence refs, consent/preference state, policy refs, draft/no-send output, approval requirements, and audit refs.
- Review request is idempotent by stay/customer/pet/template/policy scope; replay does not duplicate outreach.

### Forbidden side effects

- No autonomous review request, marketing, rebooking, winback, discount, package offer, complaint response, public-review reply, or suppression override.
- No bypass of DNC/opt-out/legal/privacy/payment/incident/complaint holds.

### Pass criteria

- Clean completed stay produces a review candidate/draft only.
- Suppression fixture blocks outreach and records reason.
- Send attempt fails closed without approved deterministic policy and audit evidence.

### Fail criteria

- Review request sends by default.
- CRM clears complaints, DNC, incidents, payment disputes, or consent ambiguity without human review.

## 11. Cross-script validation matrix

After each script, record pass/fail evidence for these shared controls.

| Control | Required validation | Fails if |
| --- | --- | --- |
| Execution mode | Run record says `local_demo` or approved `limited_live_pilot` with approval id | Mode is ambiguous or live side effects are enabled by accident |
| Secret leakage | Search server logs, browser console/network summaries, audit safe summaries, prompt safe logs, and generated docs for tokens, webhook signatures, raw storage keys, raw payment data, raw hidden prompts | Any secret/raw sensitive value appears outside governed evidence stores |
| Controlled messaging | Drafts have category, recipient ref, consent/suppression checks, approval state, idempotency key, and no outbox send by default | Draft is sent, queued, or marked approved without approval evidence |
| AI authority | AI outputs are schema-bound drafts/recommendations with source refs, risk flags, validation, and human review reasons | AI output directly mutates booking/vaccine/care/incident/payment/provider/customer-send state |
| Audit chain | Source ingest, AI run, validation, review task, draft, approval/blocked attempt, and side-effect attempt are append-only audited | Important state changes lack actor/source/policy/approval/idempotency evidence |
| Role/scope | Staff/manager/customer/system views are role-scoped and subject-scoped | Actor can access unrelated customer/pet/raw evidence or manager-only controls |
| Idempotency | Replaying source event does not duplicate customer drafts/tasks/sends beyond policy | Replay creates duplicate sends or uncontrolled task spam |
| Raw evidence handling | Raw documents/media/provider/payment/incident/prompt data remain private refs | Raw content appears in ordinary UI, logs, or customer copy |
| Approval gates | Booking confirmation/rejection/waitlist, vaccine acceptance, special care, incident severity/owner send, cancellation/payment, and review request sends remain gated | Any gate can be bypassed by staff role, AI result, or UI shortcut |
| Rollback readiness | Outbound kill switch, agent pause, provider command disablement, and manual fallback are available | Cannot stop agents/outbox/provider actions quickly |

## 12. Overall launch smoke pass/fail criteria

### Overall pass

All of the following must be true:

1. All eight scripts pass in local/demo mode against current MVP fixtures.
2. Every customer-facing message remains draft/review-gated unless the exact deterministic live-pilot policy is approved and recorded.
3. No production provider/PMS/payment/live messaging side effect occurs during local/demo execution.
4. No secrets or raw sensitive content leak to ordinary logs, client bundles, prompt safe logs, screenshots, or customer-visible drafts.
5. Review queues exist and are visible for document/vaccine, booking exceptions, care/special handling, incidents, payment/cancellation, customer-message approval, and CRM suppression/review.
6. Audit events are append-only and sufficient to reconstruct source -> AI/deterministic result -> validation -> review/draft -> approval/blocked effect.
7. The final MVP smoke card (`t_3a72309d`) records exact runnable commands, fixtures, defects, and blockers after implementation exists.

### Overall conditional pass for limited live pilot

A limited live pilot may be considered only if local/demo passes and separate human approvals exist for:

- pilot launch;
- live customer/test-recipient scope;
- stack/MVP cutline;
- message categories/templates/channels/recipients/facts, or draft-only manual review posture;
- rollback/fallback owner and kill switches;
- staff training and escalation coverage;
- security/audit/retention defaults for the pilot evidence used.

Even then, provider writes, payment movement, autonomous customer sends, vaccine auto-acceptance, autonomous rejection/cancellation, incident severity finalization, eligibility-affecting flags, and review-request sends remain disabled unless separately approved by exact workflow and policy.

### Overall fail / launch blocker

Any one of these blocks launch readiness:

- Live customer/provider/payment effect without explicit approval.
- Secret or raw sensitive-data leakage.
- AI treated as final authority for high-risk state.
- Missing audit chain for source/result/review/effect.
- No reliable way to pause agents/outbox/provider commands.
- Staff cannot see or operate the required review queues.
- Local/demo MVP cannot complete the happy boarding path.
- Suppression/consent/preference checks fail for any customer-message path.
- Incident, vaccine, cancellation/payment, special-care, or no-capacity tests can bypass human gates.
