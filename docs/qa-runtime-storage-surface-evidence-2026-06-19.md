# QA: runtime and storage surface evidence — 2026-06-19

Task: `t_bc8fe612`

Scope: docs-evidence audit for representative runtime and storage surfaces. The goal was not to redesign the docs, but to verify whether a non-coder can understand what each surface stores/runs, why it matters operationally, and where the authoritative source/Rustdoc/tests live.

## Verification performed

- Ran the full docs gate:

```sh
./scripts/check_docs.sh
```

Result: passed. The gate ran crate doctests for `domain`, `app`, `storage`, and `gingr`; strict `RUSTDOCFLAGS='-D missing_docs' cargo doc --workspace --no-deps`; Rustdoc HTML smoke checks; Markdown link checks; and public landing source checks.

- Confirmed generated Rustdoc pages exist for representative audited runtime/storage entities:

```text
FOUND target/doc/storage/operations/struct.ManagerDailyBriefOutcomeRecord.html
FOUND target/doc/storage/operations/struct.DataQualityHygieneOutcomeRecord.html
FOUND target/doc/storage/operations/struct.StoredSourceRecordRef.html
FOUND target/doc/pet_resort_api/http/struct.VaccineDocumentState.html
FOUND target/doc/pet_resort_worker/runtime/struct.Config.html
FOUND target/doc/pet_resort_worker/runtime/enum.AgentRuntimeMode.html
FOUND target/doc/pet_resort_worker/runtime/enum.SideEffectMode.html
FOUND target/doc/cli/index.html
FOUND target/doc/app/tools/enum.ExternalToolCandidate.html
```

## Representative surface matrix

| Surface | What it stores/runs in plain language | Docs orientation | Authoritative source/Rustdoc | Executable tests/evidence | QA read |
| --- | --- | --- | --- | --- | --- |
| `storage::operations::ManagerDailyBriefOutcomeRecord` | Durable labor-outcome evidence for a manager brief action: action id, outcome, before/actual minutes, actor, source refs, reporting group, and savings evidence. | `storage/README.md` lines 49–51, 84, and 100 explain that it is a storage projection, not a domain policy object. `docs/glossary-workflow-state-terms.md` explains outcome capture for non-coders. | `storage/src/operations.rs` defines the record and JSON codecs; Rustdoc generated at `target/doc/storage/operations/struct.ManagerDailyBriefOutcomeRecord.html`. | `storage/tests/manager_daily_brief_outcome_storage.rs`; `apps/api/tests/manager_daily_brief_outcome_capture_contract.rs`; docs gate passes. | Strong. Non-coder meaning is clear: measured feedback/evidence after review, not an autonomous decision. |
| `storage::operations::DataQualityHygieneOutcomeRecord` | Durable evidence for source-data cleanup labor: action, issue refs, before/actual minutes, outcome, owner persona, source refs, and resolution status after review. | Root/public docs and API tests discuss the data-quality hygiene loop; storage README currently explains the generic record/code/codec families and mentions data-quality hygiene in Rustdoc, but its type/module map does not yet list this newer record alongside manager daily brief. | `storage/src/operations.rs`; Rustdoc generated at `target/doc/storage/operations/struct.DataQualityHygieneOutcomeRecord.html`. | `storage/tests/data_quality_hygiene_outcome_storage.rs`; `apps/api/tests/data_quality_hygiene_agent_contract.rs`; docs gate passes. | Mostly good, but storage README should add `DataQualityHygieneOutcomeRecord` to the type/module map and labor-cost bullets so the storage surface does not look manager-brief-only. |
| `storage::operations::StoredSourceRecordRef` | Persisted pointer back to a provider/source record so stored evidence can be audited back to Gingr, a warehouse export, or another upstream source. | `storage/README.md` line 48, glossary source/provenance entries, and public landing glossary links explain source refs/provenance. | `storage/src/operations.rs`; Rustdoc generated at `target/doc/storage/operations/struct.StoredSourceRecordRef.html`. | Exercised through storage outcome tests and app/API source-evidence tests; docs gate passes. | Strong. The docs make clear that a source pointer is evidence, not approval or source correctness. |
| `storage::service_line::{boarding, daycare, grooming, retail, training}` contract records/codes | Service-line persistence wrappers and stable codes that let stored offerings/contracts rehydrate into `domain::*` service-line contracts. | `storage/README.md` lines 19–24 and 60–65; `storage/src/service_line/README.md`; root README service-line codec row. | `storage/src/service_line/*.rs`; generated Rustdoc under `target/doc/storage/service_line/*`. | `storage/tests/core_service_contract_storage.rs`, `storage/tests/core_service_contract_storage.rs`, `storage/tests/operations_storage_contracts.rs`, and storage doctest in `storage/src/service_line/mod.rs`. | Strong. Non-coder can tell these are normalized persistence wrappers, not provider DTOs or business policy. |
| `pet_resort_api::http::VaccineDocumentState` and private `VaccineDocumentStore` | API-local in-memory state for deterministic local/test workflows: documents, extractions, review packets, approvals, eligibility, manager/data-quality outcomes, inquiry records, and audit events. | `apps/api/README.md` lines 11–22, 23–32, 34–40, 51–67, and 85–91 are clear that this is a local/test runtime shell and not durable storage. Root README maps API runtime shell to `apps/api/src/http.rs`. | `apps/api/src/http.rs`; Rustdoc for the public wrapper at `target/doc/pet_resort_api/http/struct.VaccineDocumentState.html`; private store is source-only. | `apps/api/tests/health_contract.rs`, `vaccine_document_workflow_contract.rs`, manager brief API tests, and data-quality hygiene API tests. | Strong for the public wrapper and route behavior. Private records are intentionally local scaffolding; docs correctly warn they must be promoted to domain/app/storage by future cards if they become durable product concepts. |
| `pet_resort_worker::runtime::{Config, AgentRuntimeMode, SideEffectMode}` | Background worker runtime configuration: fake deterministic or disabled agent execution, with side effects always stubbed today. | `apps/worker/README.md` lines 3–16 and 18–27 explain what the worker runs and what it explicitly does not do yet. Root README maps worker runtime shell. | `apps/worker/src/runtime.rs`; generated Rustdoc pages for `Config`, `AgentRuntimeMode`, and `SideEffectMode`. | `apps/worker/tests/runtime_mode_contract.rs`; docs gate passes. | Mostly strong. One crate-root Rustdoc sentence is stale/over-specific: `apps/worker/src/lib.rs` calls this a “durable Postgres-backed” worker shell even though the README says durable leasing/queue/Postgres wiring is not implemented yet. |
| `pet-resort` CLI runtime shell | Read-only command shell that prints source-controlled JSON for baseline agent specs and external tool candidates. | `apps/cli/README.md` lines 3–24 and 50–60 explain that it is a local/manual shell, not live infrastructure. Root README maps CLI runtime shell. | `apps/cli/src/main.rs`; generated Rustdoc at `target/doc/cli/index.html`; app tool inventory Rustdoc at `target/doc/app/tools/enum.ExternalToolCandidate.html`. | CLI command behavior is simple; app tool contracts are covered by `app/tests/application_quality_patterns.rs`. | Good. Non-coder can understand it as inspection-only, not automation execution. |
| `app::tools::ExternalToolCandidate` and tool ports | App-owned inventory and typed capability ports for future external systems: Gingr, payment, messaging, file storage, OCR, cameras, Hermes, Postgres. | `app/README.md` lines 90–94, `app/src/tools/README.md`, `apps/cli/README.md`, and public glossary/tool-port entry explain the operational meaning. | `app/src/tools.rs`; Rustdoc generated at `target/doc/app/tools/enum.ExternalToolCandidate.html`. | `app/tests/application_quality_patterns.rs`; CLI `pet-resort tools` source surface. | Strong. Docs repeatedly say candidate/port does not imply live implementation. |

## Findings

### Passes

1. The docs now generally lead with operational meaning before implementation details. `storage/README.md`, `apps/api/README.md`, `apps/worker/README.md`, `apps/cli/README.md`, and the glossary all explain what a surface stores/runs and what not to infer from it.
2. The authoritative path pattern is consistent: README/wiki pages point to source files; source files have Rustdoc; tests exercise representative contracts; `./scripts/check_docs.sh` verifies doctests, missing Rustdoc, generated docs, Markdown links, and public landing requirements.
3. Runtime side-effect boundaries are visible to non-coders. API readiness/health docs, worker mode docs, and CLI docs repeatedly distinguish local/test/stubbed shells from live customer/provider/payment side effects.
4. Storage boundary semantics are mostly strong. Records/codes/codecs/scalars/source refs are explained as persistence/projection artifacts that promote/demote through `domain::*`, not as alternate policy or provider truth.

### Issues to fix or track

1. Stale/over-claimed Rustdoc in `apps/worker/src/lib.rs`:
   - Current wording: “Worker shell for durable Postgres-backed pet-resort workflows.”
   - Why it matters: `apps/worker/README.md` says durable leasing, queue consumers, schedulers, concrete app services, and provider writes are not implemented yet. A non-coder could infer a deployed/durable Postgres worker exists.
   - Suggested fix: change the crate root to “Worker shell for future durable pet-resort workflows” or “Worker shell for safe local/background pet-resort workflow execution,” and mention Postgres only as a future adapter/storage implementation.

2. Stale copied Rustdoc sentence in `storage/src/operations.rs`:
   - `StoredManagerDailyBriefLaborMinutes::try_new` says “Validates and wraps a non-empty brand name before persistence.”
   - Why it matters: the type validates non-zero labor minutes, not a brand name. The generated Rustdoc has a misleading implementation-detail fragment.
   - Suggested fix: “Validates and wraps a non-zero labor-minute quantity before persistence.”

3. Storage README type map is behind the current storage surface for data-quality hygiene:
   - `DataQualityHygieneOutcomeRecord` has source, Rustdoc, and tests, but `storage/README.md` line 18 and type/module map lines 49–51 foreground manager daily brief and omit the data-quality hygiene outcome projection.
   - Why it matters: non-coders following the storage README may miss that data-quality hygiene outcome capture is a durable storage surface too.
   - Suggested fix: add a table row for `DataQualityHygieneOutcomeRecord` plus codes/scalars, and add it to the “What this crate owns”/labor-cost examples alongside `ManagerDailyBriefOutcomeRecord`.

4. Public landing/runtime docs rely on source/test paths rather than generated Rustdoc links for some runtime shell details:
   - This is acceptable for source-controlled Markdown, but the public landing currently links primarily to workflow Rustdocs and crate roots. Runtime shell details are more discoverable from README/source paths than from direct Rustdoc item links.
   - Suggested fix if public docs need stronger runtime evidence: add a small “Runtime shells” evidence section linking `/pet_resort_api/`, `/pet_resort_worker/`, `/cli/`, and naming API/worker/CLI shell responsibilities in plain language.

## Missing Rustdoc/tests assessment

- Missing Rustdoc: none found for the representative public runtime/storage entities above. The strict missing-docs gate passed for the workspace.
- Missing tests: no blocker found for representative public runtime/storage contracts. Manager/data-quality storage outcomes, API outcome/draft/context behavior, worker runtime modes, and app tool-port semantics all have tests. CLI has no dedicated integration test for command output; current behavior is small/read-only and indirectly backed by `app` tests, but a future richer CLI should add command-level snapshot/smoke tests.

## Recommended remediation order

1. Fix the two stale Rustdoc comments (`apps/worker/src/lib.rs`, `storage/src/operations.rs`).
2. Update `storage/README.md` to include `DataQualityHygieneOutcomeRecord` in the type/module and labor-cost maps.
3. Optionally add a public landing “Runtime shells” evidence block if runtime shell discoverability becomes a public-docs requirement.

## Bottom line

The runtime/storage docs evidence surface is largely coherent and source-grounded. Non-coders can understand the key distinction between durable evidence, local runtime shells, app workflow packets, and live operational authority. The remaining issues are localized stale/omitted documentation, not missing core Rustdoc or broken evidence paths.
