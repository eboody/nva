<!-- archived-historical-record -->
# Archived semantic domain Phase 2 aggregate review

This review records a superseded tree and is not current architecture authority.

Scope: independent review gate for Phase 2 invariant-bearing aggregates in `domain/src/strategic_ai_ops_bridge.rs` and the canonical owner facades exposed through `domain::{lead, customer, operations, agent, analytics}`.

Safety boundary: this review only hardens local domain contracts and tests. It does not send customer messages, write provider/PMS/CRM records, mutate schedules/capacity/staffing, move money, deploy, or claim live NVA value.

## Binding proof chain checked

| Chain step | Review result |
| --- | --- |
| Source evidence | Aggregates require provenance or source system evidence where source grounding is part of the authority surface. |
| Provenance/data quality | Capacity recommendations reject empty source evidence; knowledge evidence is authorized only after document freshness, applicability, section, and purpose checks. |
| Validated fact | Scalar and interval constructors/serde reject zero, reversed, expired, or unit-mismatched facts. |
| Relationship-checked aggregate | Lead packets, CRM memberships, capacity recommendations, cited answers, financial insights, and outcome records now reject mismatched or empty high-risk relationships. |
| Review/authorization proof | Customer contact, marketing use, capacity/labor recommendations, financial insights, and cited answers require the expected consent/review/authorization path. |
| Legal application action | Domain surfaces expose queue/review-only proof tokens and explicit blockers; live send, schedule mutation, and financial mutation remain unavailable. |
| Observed outcome/attribution | Outcome records require measured changes and reviewed-action attribution before value claims. |

## Aggregate matrix

| Aggregate | Status | Approved proof | Rejected/impossible states covered | Residual risk / route |
| --- | --- | --- | --- | --- |
| Lead response packet | Approved after hardening | `ResponsePacket::try_new`, builder, and serde all route through event/SLA/consent/attempt/attribution checks. | SLA receipt mismatch, due-time mismatch, attempts before receipt, attempts out of order, channel/purpose mismatch, opted-out consent, empty attempts, bad serde rehydration, mismatched review/message approval, converted attribution without reservation evidence, idempotency debug leakage. | Still domain-only; storage/API/provider vertical integration must preserve the same packet and message-ref relationships before any queue/outbox slice. |
| CRM/customer intelligence segment membership | Approved | Accepted-note promotion requires accepted review state, reviewer, current interval, allowed use, and visibility; membership requires nonempty same-customer evidence and basis; marketing permission requires exact marketing consent. | Empty evidence/basis, wrong-customer evidence, rejected/superseded/expired notes, operations-only evidence used for marketing, missing/wrong-purpose marketing consent, note body debug leakage. | Consent subject/recorded-time is still coarse in this bridge; downstream canonical consent owner should add subject/version when integrating. |
| Capacity/labor optimization recommendation | Approved | Recommendation constructor derives labor delta from demand x labor standard versus scheduled coverage and requires same location/window, source evidence, feasible solver status, role compatibility, and manager review. | Wrong location/window, missing source evidence, infeasible solver status, wrong role, non-manager review gate, action minutes disagreeing with derived labor gap, overflow. | Source freshness/data-quality severity remains a downstream integration concern; storage/API projections must not accept hand-supplied delta as authority. |
| Knowledge document / authorized evidence | Approved | Retrieval promotion checks approved/current document, declared section, role/location/service applicability, and actor allowed-use for purpose. | Draft/stale/review-expired documents, missing section, wrong location/service/role, unauthorized purpose, source conflict. | Applicability is exact-list based; future enterprise policy inheritance should be explicit rather than wildcarded silently. |
| Assistant answer packet | Approved after hardening | `AnswerPacket::try_cited` and custom serde now require nonempty claims/citations/authorized evidence and claim-to-evidence support for `Cited`; escalated state requires a reason and does not retain fake citations. | Builder-created arbitrary citations do not satisfy `is_cited`; serde cannot rehydrate `Cited` without authorized evidence; uncited claims and conflicting evidence fail; answer text debug is redacted. | Builder still permits draft envelopes with citation-shaped fields, but `is_cited()` remains fail-closed and custom serde normalizes draft/escalated states. |
| Financial site period/fact/insight | Approved after hardening | Site period validates forward ranges at builder and serde boundaries; financial insight builder/serde now requires manager review before recommendation use; net revenue uses checked currency-aware money. | Equal/reversed period, serde period bypass, deductions exceeding gross without panic, negative/overflow money, currency-mismatched arithmetic, non-manager financial recommendation gate. | Revenue fact itself is source-system backed but not full provenance-backed; finance vertical should persist source refs and site/period quality before reporting. |
| Outcome/attribution record | Approved | Outcome record derives metric from `MeasuredChange`, requires unit-compatible before/after values, and `can_support_value_claim()` requires reviewed-action attribution plus relationship consistency. | Metric/value unit mismatch, weak correlated attribution used as value claim, wrong-source evidence, revenue currency mismatch through measured-change relationship check. | Site/period/source/projection-version compatibility across upstream recommendation and outcome is deferred to finance/storage/API vertical integration. |

## RED/GREEN evidence captured

RED failures observed before implementation:

- `cargo test -p domain --test strategic_ai_ops_model_contracts assistant_answer_deserialize_rejects_cited_state_without_authorized_evidence -- --nocapture` failed because derived serde accepted `AnswerState::Cited` with no authorized evidence.
- `cargo test -p domain --test strategic_ai_ops_model_contracts lead_response_packet_rejects_empty_attempts_at_builder_and_serde_boundaries -- --nocapture` failed to compile because `MissingContactAttempt` did not exist and empty attempt vectors were accepted.
- `cargo test -p domain --test strategic_ai_ops_model_contracts financial_insight_requires_manager_review_gate_before_recommendation_use -- --nocapture` failed to compile because `Insight::try_new` and `ManagerReviewRequired` did not exist and the bon builder did not validate review gate semantics.

GREEN focused checks after implementation:

- `cargo test -p domain --test strategic_ai_ops_model_contracts lead_response_packet_rejects_empty_attempts_at_builder_and_serde_boundaries -- --nocapture` passed 1/1.
- `cargo test -p domain --test strategic_ai_ops_model_contracts financial_insight_requires_manager_review_gate_before_recommendation_use -- --nocapture` passed 1/1.
- `cargo test -p domain --test strategic_ai_ops_model_contracts assistant_answer_deserialize_rejects_cited_state_without_authorized_evidence -- --nocapture` passed 1/1.

## Review verdict

Approved for Phase 2 closeout with residual integration assumptions: downstream storage/API/provider/app slices must preserve these promotion points rather than deserialize or project raw shapes as authority, and value/finance claims must remain source/ref/review/outcome-backed before any external NVA-facing statement.
