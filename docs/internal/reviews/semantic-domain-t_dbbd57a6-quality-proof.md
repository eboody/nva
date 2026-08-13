# Semantic domain hardening quality proof — t_dbbd57a6

Date: 2026-08-13

## What changed in this pass

This pass added executable proof around three domain-hardening seams:

1. Booking triage evidence order is now protected by trybuild compile-fail fixtures. Callers cannot attach policy evidence before pet-profile evidence, and cannot mark a request policy-ready before policy evidence exists.
2. Inquiry intake idempotency now rejects same-key payload drift with a conflict response instead of treating a changed payload as an idempotent replay. The rejection payload remains an owned API DTO and keeps live send/provider write disabled.
3. Lead-response Rustdoc now shows the exact safe promotion chain from source event and SLA evidence through consent, reviewed attempt, approval, queueable action, and still no live send method.

A small rustdoc completeness fix in `app/src/site_finance.rs` was also needed after strict docs verification surfaced pre-existing missing public docs in that module.

## Focused proof commands

All focused gates passed:

```sh
cargo test -p pet-resort-api --test api_dto_contracts inquiry_intake_idempotency_replay_reuses_exact_source_event_and_rejects_payload_drift -- --nocapture
cargo test -p app --test booking_triage_typestate_compile_fail -- --nocapture
cargo test -p domain --doc lead::response -- --nocapture
cargo test -p domain --test source_code_contracts -- --nocapture
```

Observed results:

- API idempotency drift proof: 1 passed.
- Booking-triage trybuild compile-fail proof: 1 passed, with 2 accepted UI fixtures.
- Lead-response doctest proof: 1 passed.
- Source code table contracts: 2 passed.

## Workspace gates

Passed:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
python scripts/check_rustdoc_completeness.py
git diff --check
```

`python scripts/check_rustdoc_completeness.py` ran the strict rustdoc gate with `RUSTDOCFLAGS='-D missing_docs' cargo doc --workspace --no-deps` and passed after the `site_finance` doc fix.

Caveat:

```sh
./scripts/check_docs.sh
```

still fails on broad documentation/navigation link debt that is outside the narrow code hardening in this pass. The failure includes missing README navigation links and many archived/internal markdown targets. This should be handled as a separate docs-navigation cleanup lane rather than hidden inside semantic domain hardening.

## Modum classification snapshot

Command:

```sh
modum check --format json > /tmp/nva-t_dbbd57a6-modum.json
```

The installed Modum exits nonzero in deny mode and reported 209 diagnostics under `report.diagnostics`. Top categories:

- `api_raw_id_surface`: 65
- `api_builder_candidate`: 19
- `api_redundant_leaf_context`: 19
- `namespace_qualified_child_facet_follow_through`: 17
- `api_candidate_semantic_module`: 14
- `api_string_error_surface`: 14
- `api_candidate_semantic_module_unsupported_construct`: 12
- `api_semantic_string_scalar`: 8
- `namespace_flat_pub_use`: 7
- `namespace_flat_pub_use_redundant_leaf_context`: 7

Classification: these are repo-wide semantic-design prompts, not regressions from this pass. The new trybuild and idempotency proof changes intentionally avoid alias/re-export appeasement and preserve the existing semantic surfaces. Any Modum remediation should be a dedicated follow-up program with classification before edits.

## Safety boundary check

The added API behavior does not enable customer sends, provider/PMS writes, booking confirmations, payment movement, or live external side effects. Drifted idempotency keys now fail closed with explicit `accepted=false`, `live_send_allowed=false`, and `provider_write_allowed=false`.
