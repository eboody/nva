# Pet Resort MVP final smoke test and review

Status: refreshed after semantic-code beautification closeout.
Run timestamp: 2026-06-14.
Workspace: `/home/eran/code/nva`.
Execution mode: `local_demo` only.

This review did not perform production deployment, production/provider/PMS mutation, payment movement, live customer messaging, or any live-customer operation. Production deployment and live customer messaging remain explicit human approval gates.

## Current gate result

Decision: **LOCAL_DEMO_TECHNICAL_GATE_PASS / LIVE_SCOPE_NO_GO_WITH_BLOCKERS**.

The previous final-smoke blockers around the default Rust test gate and domain doctests have been cleared in the current tree. The canonical local gate now passes:

```bash
./scripts/test.sh
```

Observed current result:

- Rust fmt/test/doctest gate: pass.
- API health and vaccine workflow contracts: pass.
- Storage contracts: pass.
- Frontend staff-web typecheck: pass.
- Frontend staff-web lint: pass.
- Frontend staff-web source smoke tests: pass.
- Modum semantic lint report: 33 diagnostics remain under `report.diagnostics`; these are tracked as semantic-ownership follow-up, not local/demo smoke gate failures.

## Evidence preserved from the MVP smoke review

The MVP has executable local/demo evidence for individual slices:

- Inquiry intake API creates review-gated records and draft-only replies.
- Vaccine document upload/review API preserves medical-document review gate before eligibility update.
- Booking triage domain contracts keep confirmation, rejection, vaccine, behavior, and special-care gates human-reviewed.
- Daily update domain contracts keep owner messages draft-only and suppress sensitive/internal notes.
- Staff dashboard source smoke verifies required operational surfaces and visible approval boundaries.
- Incident UI source smoke verifies serious incident classification/owner-message/follow-up/eligibility-impacting flags stay behind human gates.
- Frontend typecheck/lint/smoke tests pass.

## Resolved defects

### RESOLVED: Default test gate nondeterminism

Previous evidence: `./scripts/test.sh` failed in `apps/api/tests/health_contract.rs` because API tests shared process-global inquiry state.

Current result: `./scripts/test.sh` passes with the health contract tests included. The default gate is no longer blocked by the previous state-sharing failure.

### RESOLVED: Workspace doctest failure

Previous evidence: serialized `cargo test --workspace -- --test-threads=1` reached `domain` doctests and failed with missing crates such as `uuid`, `chrono`, and `statum`.

Current result: workspace doctests pass through the canonical `./scripts/test.sh` gate.

## Remaining launch-readiness blockers

These are now contract/product-readiness gaps, not basic build-gate failures.

### BLOCKER-1: Final happy path lacks one executable E2E harness

There is still no single executable command that drives the full local/demo journey:

```text
inquiry -> profile -> vaccine docs -> booking triage -> confirmation draft -> check-in/today view -> staff note/daily update draft -> checkout/completion state -> follow-up/retention
```

Needed contract: a local smoke runner or API/browser integration test with seeded fixture IDs and stubbed outbound/provider/payment adapters. It should assert every required audit event, review gate, draft-only message boundary, idempotency/replay behavior, and absence of live side effects.

### BLOCKER-2: Checkout/completion and CRM/retention are not executable enough

Current coverage for checkout/retention is still weaker than the rest of the MVP: dashboard text plus service-contract/docs evidence, but not a full executable chain.

Needed contracts:

- checkout/completion state transition;
- checkout summary draft;
- CRM suppression and review-request candidate;
- retention/follow-up task creation;
- persistence/API/UI assertions tying those together from an actual completed local stay.

### BLOCKER-3: Idempotency/replay is not proven across the full chain

Slice tests prove useful local truths, but the full scenario must show replaying inquiry/doc/checkout/follow-up events does not duplicate tasks, drafts, audit events, provider commands, or customer sends.

### BLOCKER-4: Full-chain redaction and evidence hygiene need executable coverage

The full-chain smoke should assert no secrets, raw evidence blobs, hidden prompt content, raw payment/provider payloads, or unsafe internal staff notes appear in ordinary logs, client bundles, customer drafts, or audit summaries.

### BLOCKER-5: Human approval gates remain intentionally unresolved for live scope

Live-customer/demo-recipient/production scope still requires a human launch owner to record approval with:

- exact mode and scope;
- approved recipients/channels/templates/fact set;
- provider/PMS mutation posture;
- payment action posture;
- security/audit/retention defaults;
- rollback owner and stop-line contacts.

Until then, live/customer/provider/payment behavior remains disabled or draft-only.

## Contract-first implementation roadmap

The next implementation pass should focus on executable contracts rather than more naming/lint cleanup:

1. **Checkout/completion domain contract** — model the completion state, approved checkout summary draft, required review gates, and audit lineage.
2. **CRM/retention domain contract** — model review-request candidacy, suppression reasons, follow-up task creation, consent/incident/payment holds, and draft-only customer copy.
3. **API/local persistence contract** — expose local/demo checkout completion and retention candidate read paths without live side effects.
4. **Staff UI/source smoke contract** — make checkout and retention readiness visible in the local staff dashboard.
5. **Full-chain local E2E smoke** — seed one inquiry and walk it through vaccine review, triage, draft confirmation, check-in/today, daily update, checkout, and retention candidate creation.
6. **Replay/redaction/rollback contracts** — prove idempotency, evidence hygiene, outbound holds, provider/payment disablement, and manual fallback boundaries.

## Current review decision

- **CI/PR technical posture:** green for the current local/demo test gate, but semantic-beautification commit remains under review until the remaining Modum diagnostics and doc contradictions are resolved.
- **Local/demo MVP posture:** promising, with core slice contracts passing.
- **Live/pilot/production posture:** still **NO-GO_WITH_BLOCKERS** until the contract-first roadmap above is implemented and human approval gates are recorded.
