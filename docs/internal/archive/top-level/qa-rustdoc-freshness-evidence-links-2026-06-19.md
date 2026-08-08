# QA: Rustdoc freshness and entity evidence links — 2026-06-19

Task: verify Rustdoc/doc-generation freshness and sample source/Rustdoc/test evidence links for entity-centered docs. This is an evidence report only; no broad docs rewrites were performed.

## Commands run

1. `bash scripts/check_docs.sh`
   - Result: failed during `app` doctests with `error[E0460]: found possibly newer version of crate zmij which serde_json depends on` from `app/src/local_smoke.rs`.
   - Assessment: build-cache/toolchain artifact drift, not a documentation contract failure. The command also reported `Blocking waiting for file lock on build directory`, and the failure listed two compiled `zmij` artifacts under `target/debug/deps`.

2. `cargo clean && bash scripts/check_docs.sh`
   - Result: passed.
   - Doctests:
     - `domain`: 13 passed.
     - `app`: 4 passed.
     - `storage`: 2 passed.
     - `gingr`: 3 passed.
   - Rustdoc freshness/completeness:
     - strict gate `RUSTDOCFLAGS='-D missing_docs' cargo doc --workspace --no-deps`: passed.
     - rendered Rustdoc smoke checks in `scripts/check_rustdoc_completeness.py`: passed.
   - Markdown/docs checks:
     - `scripts/check_markdown_links.py`: passed, scanning 304 markdown files and 21 required README entries.
     - `scripts/check_public_docs_landing.py`: passed.

3. `cargo doc --workspace --no-deps`
   - Result: passed.
   - Confirmed generated crate roots under `target/doc`: `app`, `cli`, `domain`, `gingr`, `pet_resort_api`, `pet_resort_worker`, and `storage`.

## Generated-doc freshness findings

- `target/doc` is fresh after the clean rebuild and explicit workspace doc render.
- The strict missing-docs gate passed without using the narrow statum-generated exception whitelist.
- The rendered Rustdoc smoke expectations exist and include the required fragments for:
  - `app/agents/struct.AgentPromptPacket.html`
  - `app/agents/struct.AgentPromptPacketBuilder.html`
  - `app/booking_triage/struct.Request.html`
- The public docs landing page source is validated by `scripts/check_public_docs_landing.py`; Rustdoc pages are validated separately by the Rustdoc smoke checks and the link sample below.

## Public landing Rustdoc link sample

I parsed `docs/public/index.html` and verified all 58 non-external, non-anchor Rustdoc hrefs against the generated `target/doc` tree. Result: 58/58 exist.

Representative verified links:

| Landing link | Generated target checked | Result |
| --- | --- | --- |
| `app/manager_daily_brief/` | `target/doc/app/manager_daily_brief/index.html` | exists |
| `app/data_quality_hygiene/` | `target/doc/app/data_quality_hygiene/index.html` | exists |
| `app/crm_retention/` | `target/doc/app/crm_retention/index.html` | exists |
| `app/daily_update/` | `target/doc/app/daily_update/index.html` | exists |
| `domain/message/` | `target/doc/domain/message/index.html` | exists |
| `gingr/endpoint/` | `target/doc/gingr/endpoint/index.html` | exists |
| `gingr/mapping/` | `target/doc/gingr/mapping/index.html` | exists |
| `domain/boarding/`, `domain/daycare/`, `domain/grooming/`, `domain/training/`, `domain/retail/` | generated service-line module docs | exist |
| `domain/policy/`, `domain/workflow/`, `domain/source/`, `domain/data_quality/` | generated domain module docs | exist |
| `storage/operations/` | `target/doc/storage/operations/index.html` | exists |

## Entity-centered evidence samples

Sampled `docs/design/entity-atlas-inventory.md` and `docs/design/entity-index.md` claims against source files, test files, and generated Rustdoc pages. All sampled paths existed and supported the entity claims checked.

| Entity/doc claim sampled | Source/test evidence checked | Rustdoc evidence checked | Result |
| --- | --- | --- | --- |
| Booking triage packet is a staff-review packet with deterministic results, safe/blocked actions, and review gates. | `app/src/booking_triage.rs`; `app/tests/booking_triage_mvp.rs` | `target/doc/app/booking_triage/index.html`; `struct.StaffEvaluationPacket.html`; `struct.DeterministicResult.html` | supported |
| Checkout completion packet keeps checkout completion review-gated and names blocked actions. | `app/src/checkout_completion.rs`; `app/tests/checkout_completion_workflow_contracts.rs` | `target/doc/app/checkout_completion/index.html`; `struct.Packet.html`; `enum.BlockedAction.html` | supported |
| CRM retention packet is source-grounded and consent/review-gated, with follow-up outcomes and blocked actions. | `app/src/crm_retention.rs`; `app/tests/crm_retention_workflow_contracts.rs` | `target/doc/app/crm_retention/index.html`; `struct.StaffReviewPacket.html`; `struct.OutcomeRecord.html` | supported |
| Manager Daily Brief packet carries source facts, labor-minute estimates, blocked actions, and outcome feedback. | `app/src/manager_daily_brief.rs`; `domain/src/daily_brief.rs`; `app/tests/manager_daily_brief_workflow_contracts.rs` | `target/doc/app/manager_daily_brief/index.html`; `struct.Packet.html`; `target/doc/domain/daily_brief/index.html` | supported |
| Data-quality hygiene turns ambiguous source facts into internal review work and outcome evidence. | `app/src/data_quality_hygiene.rs`; `domain/src/data_quality.rs` | `target/doc/app/data_quality_hygiene/index.html`; `struct.Packet.html`; `target/doc/domain/data_quality/index.html` | supported |
| Reservation is the booking/stay aggregate tying customer, pet, service, status, deposit/add-ons, and safety stops together. | `domain/src/entities.rs`; `domain/src/reservation/mod.rs` | `target/doc/domain/entities/struct.Reservation.html`; `target/doc/domain/reservation/index.html` | supported |
| Gingr endpoint/mapping/webhook surfaces are provider-boundary evidence, not domain truth. | `integrations/gingr/src/endpoint/mod.rs`; `integrations/gingr/src/mapping/mod.rs`; `integrations/gingr/src/webhook.rs`; `integrations/gingr/tests/webhook_contracts.rs` | `target/doc/gingr/endpoint/index.html`; `target/doc/gingr/mapping/index.html`; `target/doc/gingr/webhook/index.html` | supported |
| Storage operations/outcome records are projection/proof surfaces, not live-action authority. | `storage/src/operations.rs` | `target/doc/storage/operations/index.html`; `struct.ManagerDailyBriefOutcomeRecord.html`; `struct.DataQualityHygieneOutcomeRecord.html` | supported |

Additional source spot checks:

- `domain/src/source.rs` defines `RecordRef` and `Provenance` as source lineage/evidence types.
- `domain/src/policy.rs` defines `ReviewGate` and `policy::automation` levels.
- `domain/src/data_quality.rs` defines `Issue` as an evidence-backed data-quality issue.
- `domain/src/entities.rs` defines `Reservation` with customer, pet, service, status, deposit, add-ons, and hard stops.
- `storage/src/operations.rs` explicitly says storage records never authorize live provider writes or customer messaging, and defines `StoredSourceRecordRef`, `ManagerDailyBriefOutcomeRecord`, and `DataQualityHygieneOutcomeRecord` with source/labor evidence fields.

## Stale paths, missing generated docs, or unsupported evidence links

No stale paths or missing generated Rustdoc pages were found in the sampled entity-centered docs or public landing Rustdoc links after the clean rebuild.

Operational caveat: the first `scripts/check_docs.sh` run failed because of stale `target` artifacts (`E0460` crate-version mismatch). A clean rebuild fixed it and all doc gates passed. If this recurs in CI or repeated local runs, treat it as build-cache hygiene rather than a docs content failure.

## Verdict

Pass after clean rebuild. The entity-centered docs sampled here point to existing source/test/Rustdoc evidence, and the generated Rustdocs are fresh according to the repo's documented checks.
