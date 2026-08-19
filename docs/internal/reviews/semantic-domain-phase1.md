<!-- archived-historical-record -->
# Archived semantic domain Phase 1 review

This review records a superseded tree and is not current architecture authority.

Status: rejected with precise blocker and RED regression test.
Date: 2026-08-13.
Task: `t_fb8049bb` review gate for canonical ownership conversions and wire stability.

## Verdict

Phase 1 does improve several runtime invariants: source-system codes are explicit and tested, money arithmetic is checked and currency-aware, strategic positive scalars validate at constructor and serde boundaries, time windows reject non-forward ranges through serde, customer intelligence redacts sensitive note bodies, and storage/API/agent wire schema versions are explicitly tested.

However, I cannot approve the ownership conversion as complete. The current rehome still hides most canonical owners behind direct `strategic_ai_ops_bridge` re-exports. That leaves the old catch-all implementation as the real source of definitions while making call sites look canonical. This is exactly the failure mode the review gate was intended to catch: duplication is reduced cosmetically, but conversion authority and module ownership are not explicit enough.

## Blocking finding

### B1: canonical owner modules are bridge aliases, not owned definitions

Evidence:

- `domain/src/access.rs` is only `pub use crate::strategic_ai_ops_bridge::access::*;`.
- `domain/src/consent.rs` is only `pub use crate::strategic_ai_ops_bridge::communication::*;`.
- `domain/src/identity.rs` is only `pub use crate::strategic_ai_ops_bridge::identity::*;`.
- `domain/src/lead.rs` exposes `lead::response` by re-exporting `strategic_ai_ops_bridge::lead_response::*`.
- `domain/src/customer.rs` exposes `customer::intelligence` by re-exporting `strategic_ai_ops_bridge::crm::*`.
- `domain/src/agent.rs` exposes `agent::{assistant, knowledge}` by re-exporting bridge modules.
- `domain/src/analytics.rs` exposes `analytics::{finance, outcome}` by re-exporting bridge modules.
- `domain/src/operations.rs` exposes `operations::{labor, capacity, time_bucket}` by re-exporting bridge modules.
- `domain/src/lib.rs` keeps `strategic_ai_ops_bridge.rs` as both the private implementation module and the deprecated public facade target.

Why this blocks approval:

- The module path says `domain::access`, `domain::lead::response`, and `domain::analytics::finance`, but the definitions still live in `strategic_ai_ops_bridge.rs`.
- A future change can add or mutate bridge definitions without touching the canonical owner files, so ownership is not mechanically reviewable from the atlas.
- Rustdoc and search will still center the bridge file for real contracts, which weakens the entity/type proof spine.
- It preserves a parallel `strategic_ai_ops` implementation shape instead of making the compatibility facade depend on canonical owner modules.

Required fix:

Move the definitions into their canonical owner modules, then make `domain::strategic_ai_ops` a deprecated compatibility facade that re-exports from those owner modules. The bridge file should disappear or shrink to facade-only glue with no primary definitions. Keep compatibility only where it is explicitly marked temporary and mechanically points back to canonical owners.

## RED evidence added by this review

I added a focused review test to prevent this alias-hidden ownership pattern from being approved again:

- Changed file: `domain/tests/semantic_rehome_contracts.rs`
- Test: `canonical_semantic_owner_modules_do_not_hide_direct_bridge_reexports`
- Command:
  `cargo test -p domain --test semantic_rehome_contracts canonical_semantic_owner_modules_do_not_hide_direct_bridge_reexports -- --nocapture`
- Expected/current result:
  failed with `domain/src/access.rs is still a direct bridge re-export instead of owning canonical definitions`.

This is an intentional RED test. It should turn GREEN only after canonical modules own the definitions directly.

## Review evidence gathered

Focused commands run:

- `cargo test -p domain --test semantic_rehome_contracts -- --nocapture`
  - Before adding the blocker test: passed, 1/1.
- `cargo test -p domain --test strategic_ai_ops_model_contracts -- --nocapture`
  - Passed, 20/20.
- `cargo test -p domain --test canonical_units_time_contracts -- --nocapture`
  - Passed, 6/6.
- `cargo test -p domain --test source_code_contracts -- --nocapture`
  - Passed, 2/2.
- `python scripts/tests/test_semantic_domain_contract_atlas.py`
  - Passed, 1/1.
- `modum check --format json`
  - Ran and parsed from `report.diagnostics`; 198 diagnostics remain. These are not all Phase 1 blockers, but they confirm ongoing semantic lint pressure. Relevant examples include builder candidates in new/touched app/domain/storage surfaces and existing boolean protocol decision pressure.

Repository searches:

- `strategic_ai_ops_bridge` appears in canonical owner modules listed above, proving direct bridge re-export aliasing remains.
- `impl From<...>` in touched source promotion code is mostly localized to validated Gingr/provider wrappers into normalized source primitives. I did not find a clear currency/consent/review/source-status drop via `From` in the inspected paths, but the bridge ownership blocker prevents approval.
- Source system stable code coverage includes the new source variants and `strum::VariantArray` order.

## Nonblocking observations

- The conversion from Gingr wrapper types to normalized source wrappers uses `From` after wrapper validation, with `expect("already validated")`. That appears authority-neutral and lossless for string wrapper promotion, but it should stay narrowly local to provider→source evidence and not expand to consent, money, review, or outcome claims.
- `operations::capacity::RecommendedAction` now uses directional variants instead of forcing callers to interpret signed minutes. That is a real semantic improvement.
- `analytics::outcome::Record::try_new` enforces metric/unit agreement, but the builder remains available and can construct mismatched records. If `Record` becomes durable or external before the next phase, this should be closed with serde/build validation, not just named constructor tests.
- `analytics::finance::SitePeriod` uses a manual builder because interval validation is semantic. That is consistent with the builder policy.

## Residual risks for the next card

- Moving definitions out of `strategic_ai_ops_bridge.rs` may be mechanically large. Keep it as a pure move/re-export cleanup and avoid changing behavior while turning the RED test green.
- After the move, rerun rustdoc and duplicate-ownership searches because Rustdoc paths are part of the proof chain.
- Do not treat this review as a rejection of all Phase 1 invariant work. The hardening looks directionally good; the blocker is that ownership is still implemented through bridge aliases.

## Assumptions for successor work

- No live provider/PMS/customer/payment/schedule side effects are authorized.
- `strategic_ai_ops` may remain as deprecated compatibility facade only after canonical modules own the definitions.
- Keep `From` only for total, lossless, authority-neutral conversions from already-validated wrapper types; use `TryFrom` or named promotion for trust/validation/authorization changes.
- Full gates should be run only after the RED ownership test is made green.
