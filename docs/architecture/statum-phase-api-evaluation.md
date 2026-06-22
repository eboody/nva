# Statum phase API evaluation

**Date:** 2026-06-19

**Scope:** evaluate whether the current pet-resort workflow surfaces need additional `statum` phase APIs beyond the existing booking-triage request typestate. This pass looked for places where legal methods should differ by phase, not just places where a runtime status enum exists.

## Recommendation

Do **not** add any new `statum` use yet.

The existing `app::booking_triage` typestate remains justified because the legal method surface changes as source evidence is attached: an intake request may attach pet-profile evidence, a pet-profile request may attach policy evidence, and only a policy-attached request may be marked ready for deterministic policy decision. That sequence directly reduces front-desk rework by making incomplete booking-review packets harder to assemble before an agent or staff packet consumes them.

Other audited workflow surfaces currently use runtime decision packets because their next step is a reviewed recommendation, not a compile-time-owned transition. Adding `statum` there would make the code look safer without creating a new legal API boundary.

## Current justified use

| Surface | Decision | Why |
| --- | --- | --- |
| `app::booking_triage::Request<Intake -> PetProfileAttached -> PolicyAttached -> ReadyForPolicyDecision>` | Keep `statum` in `app`. | Legal methods differ by phase: evidence can only be attached in order before readiness is declared. The API encodes the source-evidence order needed before deterministic review and customer-message draft gates. |
| `domain/Cargo.toml` | Keep `statum` absent. | Domain core has semantic enums and value objects, but no current domain-level phase API whose legal methods differ by state. Keeping the dependency out prevents future workers from assuming every status enum wants typestate. |

The workspace intentionally keeps `statum = "*"` as the latest-tracking policy. This evaluation does not change that policy.

## Audited opportunities not adopted

| Candidate | Current code shape | Statum decision |
| --- | --- | --- |
| Checkout completion release (`app::checkout_completion`) | `Workflow::evaluate(Request) -> Packet` derives `CompletionStatus`, required review gates, safe agent actions, blocked actions, and audit drafts from source status plus staff handoff evidence. | Not justified yet. The app does not expose a separate method that can only be called after verified checkout; the packet remains a review recommendation. A typestate wrapper would duplicate `CompletionStatus` without preventing a live checkout/provider write because those side effects are intentionally absent. |
| Outbound message release / customer drafts (`app::daily_update`, `app::crm_retention`, `app::tools::messaging`) | Draft/readiness enums and review gates describe customer-message approval posture. Tool ports create drafts or review packets, not sends. | Not justified yet. The current boundary is “draft only until human/customer-message approval,” and there is no app-owned approved-send API to narrow. Add `statum` only if a future deterministic workflow introduces separate `Draft -> Approved -> ReleasedByHuman/SystemOfRecord` methods. |
| Payment/provider-write release (`app::tools::payment`, reservation draft-update ports) | Tool traits return semantic authorization/refund/deposit/draft-update results and errors, while review-boundary docs explicitly block live money/provider writes. | Not justified yet. There is no live execution surface in this repo to phase-protect. Typestate would be misleading unless an approved payment/provider-write contract exists and exposes different methods after review. |
| Reviewed data-quality remediation (`app::data_quality_hygiene`, `storage::operations::DataQualityHygieneOutcomeRecord`) | The workflow records reviewed outcomes and resolution status codes, with source ambiguity kept visible. | Not justified yet. Resolution remains a reviewed outcome/disposition, not a multi-step API where earlier phases must lack later mutation methods. Keep runtime enums plus outcome validation until remediation gains an app-owned reviewed-write contract. |
| Source fact promotion (`domain::source`, Gingr mapping candidates) | Source/provenance types and mapper errors preserve provider facts, source refs, and promotion failures. | Not justified yet. Promotion is currently explicit conversion plus typed error context, not a state-machine API. Consider `statum` only if future mapper code has a repeated `RawProviderEvidence -> Candidate -> ReviewedDomainFact -> PersistedProjection` protocol with legal methods that differ at each phase. |
| Grooming/rebooking/customer opportunity states (`domain::grooming`, `app::crm_retention`) | Domain enums name estimation/readiness/send boundaries and app packets keep retention follow-up review-gated. | Not justified yet. The state vocabulary is valuable, but methods do not currently differ enough by phase to offset macro/generated API and Rustdoc complexity. |

## Adoption trigger for future passes

Add `statum` only when all of these are true:

1. The workflow owns at least three meaningful phases where callers should not even see later methods before earlier evidence/review exists.
2. The phases are pet-resort workflow truth, not a wrapper over an external provider status string.
3. The typestate API prevents a labor-cost or safety failure that a runtime enum packet currently leaves easy to make, such as assembling an agent packet without source evidence or releasing an approved-send/provider-write path before review.
4. Rustdoc and tests name the operational phases and blocked live side effects, so generated macro surfaces do not become the reader’s only explanation.
5. The dependency remains scoped to the crate that owns the phase API, while preserving the repo’s intentional latest-tracking `statum = "*"` workspace policy.

## Verification checklist for a future `statum` adoption

- Add a positive test showing the legal phase progression and the resulting staff/review packet.
- Add Rustdoc or compile-fail evidence that earlier phases do not expose later methods when that is stable with the macro output.
- Keep blocked actions explicit: no customer sends, provider/PMS writes, payment movement, capacity/schedule changes, source hiding, or medical/safety approvals are implied by phase progression.
- Run `cargo fmt --all -- --check`, focused crate tests, `cargo clippy` for changed crates, and the docs check.
