# NVA Semantic Domain Hardening Implementation Plan

> **For Hermes:** Execute this plan through the `nva-semantic-domain-hardening` Kanban board using strict RED-GREEN-REFACTOR, one mutating worker at a time in `/home/eran/code/nva`, with independent review gates between phases.

**Goal:** Make NVA genuinely data-modeling-first by consolidating concept ownership, preventing invalid aggregate and serde states, encoding evidence and authorization relationships, and proving strategic models through real application/storage/API/provider vertical slices.

**Architecture:** Preserve the existing evidence-first proof chain and service-owned domain modules. Replace the permanent `strategic_ai_ops` catch-all with canonical owners and explicit bridges. External/provider/storage representations remain untrusted until fallibly promoted into validated domain facts; reviewed evidence and authorization produce typed capabilities for legal actions.

**Tech stack:** Rust 2024 workspace, `bon`, `nutype`, Serde, `statum` where phase-specific legal operations justify typestate, existing NVA domain/app/storage/API/integration crates, Cargo tests/Clippy/rustdoc, Modum as non-authoritative discovery.

---

## Binding proof chain

```text
raw or provider representation
→ source provenance and data-quality classification
→ validated semantic evidence
→ relationship-checked aggregate
→ policy/review decision
→ typed capability or proof
→ legal application action
→ observed outcome and attribution evidence
```

## Non-negotiable boundaries

- No customer-visible sends, provider/PMS/CRM writes, schedule mutations, pricing/discount/refund/payment actions, or production release.
- No measured-value claim without source and attribution evidence.
- No raw secrets or customer/pet PII in logs, fixtures, `Debug`, card comments, or commits.
- Preserve NVA’s deliberate latest-tracking policy for `statum`; do not “fix” it into a pin.
- Keep provider vocabulary quarantined from owned domain authority.
- Builders must not be treated as validation merely because they require fields.
- Serde must not bypass constructor, interval, lifecycle, currency, consent, or relationship invariants.
- Use `From` only for total, lossless, authority-neutral conversions; use `TryFrom` or named promotion for validation/trust changes.

## Maturity definition

- **L1:** concept named.
- **L2:** concept structurally represented.
- **L3:** invariants and relationships enforced by types.
- **L4:** adopted across domain, application, storage, API, and provider boundaries.

The board is complete only when priority strategic paths reach L3 and at least the lead-response vertical slice reaches demonstrated L4 without live side effects.

## Execution phases

### Phase 0: baseline and immediate correctness

1. Establish the canonical concept/contract atlas and ownership ADR.
2. Close scalar and interval Serde bypasses with failing regression tests first.
3. Replace panicking/duplicate financial arithmetic with canonical currency-aware checked money.
4. Audit and harden lifecycle rehydration, beginning with payment/message/reservation state combinations.
5. Independent P0 review and full workspace gate.

### Phase 1: canonical vocabulary and boundaries

6. Consolidate source/provenance, communication channel/consent, staff role/actor, location, document, and outcome ownership.
7. Consolidate money, quantities, percentages, units, and time-window semantics.
8. Split/rehome `strategic_ai_ops` into canonical semantic modules with explicit compatibility conversions and no permanent duplicate authority.
9. Define explicit persistence/API/agent serialization contracts and stable enum codes.
10. Independent ownership/conversion/serde review.

### Phase 2: invariant-bearing aggregates

11. Lead response packet and legal lifecycle.
12. CRM accepted notes, evidence sets, segment membership, and marketing-use proof.
13. Capacity/labor optimization input set, feasibility, recommendation, and manager review proof.
14. Knowledge document versions, applicability, authorized evidence, claim citations, and cited/escalated answer states.
15. Financial facts/insights and generalized outcome measurement/attribution evidence.
16. Independent aggregate-invariant and impossible-state review.

### Phase 3: vertical integration

17. Lead response source → domain → app → storage/API/provider dry-run slice.
18. Extend the existing retention workflow with canonical CRM evidence and outcomes.
19. Connect service-owned capacity/labor facts to Manager Daily Brief recommendations.
20. Connect permissioned knowledge retrieval to the assistant packet without unsafe tools.
21. Connect source-backed finance facts to reviewed recommendations and measured outcomes.
22. Generalize durable approval/outbox/repository/read-model infrastructure only after the slices prove common abstractions.

### Phase 4: proof and closeout

23. Property/boundary tests, compile-fail/typestate tests where useful, schema snapshots, rustdoc examples, and Modum triage.
24. Independent final architecture, security, semantic, test, and cross-layer adoption review.
25. Resolve findings, run full workspace gates, update the contract atlas and roadmap, commit intentional changes, and push only through the approved authenticated path. Closeout note: the final blocking owner-module finding was resolved by moving strategic AI-ops definitions into canonical domain owner modules and shrinking `strategic_ai_ops` to compatibility re-exports.

## Required verification

Every code card must capture RED and GREEN commands in its handoff. Phase and final gates include:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
modum check --format json
```

Focused tests must be run before full gates. Changed tracked content must be scanned for secret-shaped literals and sensitive `Debug` leaks. Passing builds prove implementation health, not business correctness; each card must cite the invariant or relationship its tests prove.

## Safety stop conditions

Stop and request user input only for credentials without an approved path, destructive/irreversible cleanup, live customer/provider/system-of-record actions, production publish/release/merge decisions, or genuine product-owner policy choices. Routine implementation and review gates are autonomous operator work.
