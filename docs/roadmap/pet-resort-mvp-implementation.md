# Pet Resort MVP Implementation Roadmap

Board: `pet-resort-mvp-implementation`
Default workspace: `/home/eran/code/pet-resort-agent-foundation`
Meta source task: `t_9a262a99`

This roadmap converts the pet-resort design artifacts into executable engineering phases. It intentionally separates stack/cutline approval, foundation work, feature slices, and final smoke testing so the MVP can advance without losing human approval gates for stack choice, production deployment, and live customer messaging.

## Required inputs

Workers should consult the canonical artifacts when they exist and block/comment clearly if a required artifact is missing or still being synthesized:

- Product map and MVP cutline.
- Data model / ERD.
- Workflow event contracts and queue/retry behavior.
- AI runtime / agent invocation contract.
- Core workflow specs:
  - inquiry intake
  - booking triage
  - vaccine document handling
  - staff operations
  - daily care updates
  - incident escalation
  - customer messaging
  - payments/pricing where relevant
- Security/audit spec: role matrix, sensitive-data handling, audit-event catalog, retention, AI governance.

## Human approval gates preserved

These are not implied approvals. Cards must block or produce draft-only behavior where applicable:

1. Stack choice and MVP cutline.
2. Production deployment.
3. Live customer messaging / auto-send behavior.
4. Confirmed booking automation, rejection, special-care acceptance, and behavior exceptions.
5. Vaccine auto-accept thresholds and medical-document uncertainty policy.
6. Incident owner-facing messages, medium/high/emergency classification, and eligibility-affecting behavior flags.
7. Payment collection/refunds/discounts/fee waivers when touched by downstream implementation.

## Execution graph

```text
Meta roadmap task t_9a262a99
  -> t_8986d4e5 Define MVP technical stack and integration architecture
      -> t_ffcc45ad Create MVP project skeleton, local dev, tests, and CI
          -> t_71c866a7 Implement core MVP data model and migrations
              -> t_70c9f6b6 Implement staff dashboard MVP surfaces
                  -> t_b0d73d7e Implement inquiry intake MVP slice
                  -> t_776c8848 Implement booking triage MVP slice
                  -> t_8d95b551 Implement vaccine document MVP slice
                  -> t_921dba60 Implement daily care update MVP slice
                  -> t_b6372868 Implement incident escalation MVP slice
                      -> t_3a72309d Run final MVP end-to-end smoke test and review
```

The five feature slices fan out after the staff dashboard foundation. They all share one repo checkout and the `pet-resort-code` profile, so the dispatcher will naturally serialize most implementation work unless isolated worktrees are added later. The graph still records the conceptual fan-out and makes the final smoke test depend on all slices.

## Board cards

### 1. `t_8986d4e5` — Define MVP technical stack and integration architecture

Assignee: `pet-resort-docs`
Parents: `t_9a262a99`
Output: `docs/roadmap/pet-resort-mvp-stack.md`

Scope:
- Frontend choice and UI structure.
- Backend/API runtime.
- Database and migration strategy.
- Queue/event runtime.
- File storage for documents/vaccines.
- Auth/session model.
- Hosting/deployment target.
- Observability/logging.
- AI runtime integration contract.

Acceptance:
- Explicit recommendation with tradeoffs.
- Stack choice and MVP cutline approval gate preserved.
- No implementation code in this card.

### 2. `t_ffcc45ad` — Create MVP project skeleton, local dev, tests, and CI

Assignee: `pet-resort-code`
Parents: `t_8986d4e5`
Output: code/config plus `docs/roadmap/pet-resort-project-skeleton.md`

Scope:
- Repo/app layout.
- Frontend shell.
- Backend API shell.
- DB migration framework.
- Test harness.
- Local dev scripts.
- CI workflow.
- Lint/typecheck gates.
- Seed fixtures and README/runbook.

Acceptance:
- Local dev starts from a clean checkout.
- Tests/lint/typecheck have concrete commands.
- No production deployment.

### 3. `t_71c866a7` — Implement core MVP data model and migrations

Assignee: `pet-resort-code`
Parents: `t_ffcc45ad`

Scope:
- Customers.
- Pets.
- Reservations.
- Documents.
- Vaccines.
- Tasks.
- Notes.
- Incidents.
- Messages.
- Audit events.
- Lifecycle/status enums, relationships, constraints, and local tests.

Acceptance:
- Migrations compile/apply locally.
- Model-level tests cover required entities and invariants.
- Audit event write paths are represented.
- Domain-specific enums/newtypes/builders are preferred over raw primitive/stringly state.

### 4. `t_70c9f6b6` — Implement staff dashboard MVP surfaces

Assignee: `pet-resort-code`
Parents: `t_71c866a7`

Scope:
- Auth/session guard.
- Today view.
- Pet profile.
- Reservation view.
- Task queue.
- Document review queue.
- Staff notes.
- Incident entry/listing.
- Audit-visible staff actions.

Acceptance:
- Staff can navigate happy-path operational surfaces against local/dev data.
- Document/incident/message actions are draft/review-oriented.
- No live customer sends.

### 5. `t_b0d73d7e` — Implement inquiry intake MVP slice

Assignee: `pet-resort-code`
Parents: `t_70c9f6b6`

Scope:
- Public/staff form or API endpoint.
- Normalized `inquiry.received` event.
- Agent invocation boundary.
- Parsed lead persistence.
- Draft reply generation.
- Missing-info/internal task creation.
- Audit events.

Acceptance:
- Local smoke can submit an inquiry and see structured lead + draft reply + task in staff UI.
- Live customer replies remain draft/approval-only unless separately approved.

### 6. `t_776c8848` — Implement booking triage MVP slice

Assignee: `pet-resort-code`
Parents: `t_70c9f6b6`

Scope:
- Availability/capacity primitives.
- Reservation lifecycle states.
- Deterministic eligibility gates.
- Triage-agent recommendation boundary.
- Staff confirm/decline UI.
- Confirmation draft generation.
- Audit events.

Acceptance:
- Staff can evaluate a request, see hard-rule results + AI recommendation, and produce a draft confirmation.
- Confirmed booking automation, rejection, special-care acceptance, and behavior exceptions remain human approval gates.

### 7. `t_8d95b551` — Implement vaccine document MVP slice

Assignee: `pet-resort-code`
Parents: `t_70c9f6b6`

Scope:
- Upload/storage path.
- Document record creation.
- OCR/extraction invocation boundary.
- Vaccine extraction schema persistence.
- Review UI.
- Approval/rejection workflow.
- Eligibility update.
- Audit events.

Acceptance:
- Local smoke can upload a sample document, persist extraction output, require staff review on uncertainty, update pet eligibility after approval, and preserve document/audit records.
- Medical-document uncertainty policy and auto-accept threshold remain approval-gated.

### 8. `t_921dba60` — Implement daily care update MVP slice

Assignee: `pet-resort-code`
Parents: `t_70c9f6b6`

Scope:
- Staff note input.
- Event creation.
- Daily-update agent invocation boundary.
- Owner-message draft generation.
- Preview UI.
- Staff approval/send stub.
- Audit log.

Acceptance:
- Local smoke can transform staff notes into a previewed owner update draft and record approval/audit state.
- Live customer sending and health/behavior concern language remain human approval gates.

### 9. `t_b6372868` — Implement incident escalation MVP slice

Assignee: `pet-resort-code`
Parents: `t_70c9f6b6`

Scope:
- Incident form.
- Incident type/severity classification draft.
- Manager review queue.
- Owner message draft.
- Follow-up task creation.
- Behavior/eligibility flag boundary.
- Audit events.

Acceptance:
- Staff can record an incident, see classification/escalation draft, route manager review, generate owner-message draft, and create follow-up tasks.
- Owner-facing messages, medium/high/emergency classifications, and eligibility-affecting flags remain human approval gates.

### 10. `t_3a72309d` — Run final MVP end-to-end smoke test and review

Assignee: `pet-resort-reviewer`
Parents:
- `t_b0d73d7e`
- `t_776c8848`
- `t_8d95b551`
- `t_921dba60`
- `t_b6372868`

Output: `docs/roadmap/pet-resort-mvp-smoke-test.md`

Smoke path:
1. Inquiry submitted.
2. Customer/pet profile exists.
3. Vaccine document uploaded/reviewed.
4. Booking triaged.
5. Confirmation draft generated.
6. Check-in/today view reflects reservation.
7. Staff note produces daily update draft.
8. Checkout/completion state is captured.
9. Follow-up/retention task exists.

Acceptance:
- Exact commands, fixtures, pass/fail results, defects, and launch-readiness blockers are documented.
- No production deployment.
- No live customer messaging.

## MVP cutline

Included in MVP:
- One local/dev staff dashboard.
- Manual-review-first agent workflows.
- Draft customer messages, not live auto-send.
- Local/dev document upload and extraction pathway.
- Local/dev booking triage and reservation lifecycle.
- Audit-event coverage for sensitive staff/agent actions.
- End-to-end smoke path with fixtures.

Deferred unless explicitly approved:
- Production deployment.
- Live customer messaging.
- Payment collection automation.
- Auto-accepting vaccine documents without staff review.
- Confirmed booking automation without staff confirmation.
- Eligibility-affecting incident automation without manager review.
- Multi-location or complex capacity optimization.

## Verification expectations for implementation workers

Each implementation card should leave a structured handoff containing:
- Changed files.
- Commands run.
- Tests/lint/typecheck status.
- Any missing upstream artifact or assumption.
- Approval gates preserved.
- Screenshots or local URLs only if useful and non-secret.

Because most implementation cards change code, workers should usually comment their handoff and block with `review-required:` rather than self-completing, unless the change is purely documentation or the board supervisor/reviewer explicitly approves completion.
