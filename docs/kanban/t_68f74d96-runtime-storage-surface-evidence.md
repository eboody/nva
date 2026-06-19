# QA: runtime and storage surface evidence

Task: `t_68f74d96`
Date: 2026-06-19
Scope: docs-evidence audit for representative runtime and storage surfaces. This is an audit artifact, not a remediation patch.

## Verification performed

- Ran the full docs gate from the repo root:

```sh
./scripts/check_docs.sh
```

Result: passed. The gate ran doctests for `domain`, `app`, `storage`, and `gingr`; strict `RUSTDOCFLAGS='-D missing_docs' cargo doc --workspace --no-deps`; rendered Rustdoc HTML smoke checks; Markdown README/wiki link checks; and public landing source checks.

- Confirmed generated Rustdoc pages exist for representative audited entities:

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
| `storage::operations::ManagerDailyBriefOutcomeRecord` | Durable labor-outcome evidence for a manager brief action: action id, outcome, before/actual minutes, actor, source refs, reporting group, and savings evidence. | `storage/README.md` explains that storage owns projection records/codecs, not business policy; `docs/design/entity-atlas-runtime-storage-api-surfaces.md` ties it to outcome evidence and review gates. | `storage/src/operations.rs`; rendered Rustdoc at `target/doc/storage/operations/struct.ManagerDailyBriefOutcomeRecord.html`. | `storage/tests/manager_daily_brief_outcome_storage.rs`; `apps/api/tests/manager_daily_brief_outcome_capture_contract.rs`; docs gate passed. | Strong. A non-coder can tell this is measured feedback/evidence after review, not an autonomous decision engine. |
| `storage::operations::DataQualityHygieneOutcomeRecord` | Durable evidence for data-quality cleanup labor: issue refs, outcome, before/actual minutes, actor/persona, source refs, resolution status, and reporting group. | The runtime/storage atlas page names this clearly. `storage/README.md` describes the storage crate role, but its navigation/type map still omits this newer record while foregrounding manager daily brief. | `storage/src/operations.rs`; rendered Rustdoc at `target/doc/storage/operations/struct.DataQualityHygieneOutcomeRecord.html`. | `storage/tests/data_quality_hygiene_outcome_storage.rs`; `apps/api/tests/data_quality_hygiene_agent_contract.rs`; docs gate passed. | Mostly good, but storage README should add this record/codes/scalar so data-quality outcome storage is as discoverable as manager brief outcome storage. |
| `storage::operations::StoredSourceRecordRef` | Persisted pointer back to a provider/source record so stored evidence can be audited back to Gingr, a warehouse export, or another source. | `storage/README.md`, glossary/source-provenance docs, and the runtime/storage atlas page explain source refs/provenance as evidence, not approval. | `storage/src/operations.rs`; rendered Rustdoc at `target/doc/storage/operations/struct.StoredSourceRecordRef.html`. | Exercised through storage outcome tests and API/app source-evidence tests; docs gate passed. | Strong. The docs make clear that a source pointer supports traceability but does not certify source correctness or authorize action. |
| `storage::service_line::{boarding, daycare, grooming, retail, training}` records/codes | Service-line persistence wrappers and stable codes that rehydrate into semantic `domain::*` contracts. | `storage/README.md` and `storage/src/service_line/README.md` explain normalized persistence wrappers versus domain policy. | `storage/src/service_line/*.rs`; rendered Rustdoc under `target/doc/storage/service_line/*`. | `storage/tests/core_service_contract_storage.rs`, `storage/tests/operations_storage_contracts.rs`, and storage doctests. | Strong. Non-coders can tell these are storage wrappers/code tables, not provider DTOs or replacement business rules. |
| `pet_resort_api::http::VaccineDocumentState` and private `VaccineDocumentStore` | API-local in-memory state for deterministic local/test workflows: inquiries, vaccine documents, review packets, approvals/rejections, eligibility, outcomes, and audit events. | `apps/api/README.md` and `docs/design/entity-atlas-runtime-storage-api-surfaces.md` describe this as a local HTTP runtime shell, not durable storage or live side-effect authority. | `apps/api/src/http.rs`; rendered Rustdoc for the public state wrapper at `target/doc/pet_resort_api/http/struct.VaccineDocumentState.html`; private store is source-only. | `apps/api/tests/health_contract.rs`, `vaccine_document_workflow_contract.rs`, manager brief API tests, and data-quality hygiene API tests. | Strong for the public wrapper and route behavior. Private records are intentionally local scaffolding and docs warn they need explicit domain/app/storage promotion if they become durable concepts. |
| `pet_resort_worker::runtime::{Config, AgentRuntimeMode, SideEffectMode}` | Background worker runtime configuration: fake deterministic or disabled agent execution, with side effects stubbed today. | `apps/worker/README.md` and runtime/storage atlas page explain the shell role and the missing durable queue/scheduler/provider-write pieces. | `apps/worker/src/runtime.rs`; rendered Rustdoc for `Config`, `AgentRuntimeMode`, and `SideEffectMode`. | `apps/worker/tests/runtime_mode_contract.rs`; docs gate passed. | Mostly strong. One crate-root Rustdoc sentence in `apps/worker/src/lib.rs` still overclaims “durable Postgres-backed” even though README says durable leasing/queue/Postgres wiring is not implemented. |
| `pet-resort` CLI runtime shell | Read-only command shell that prints source-controlled JSON for baseline agent specs and external tool candidates. | `apps/cli/README.md` explains the local/manual shell and that it does not call live infrastructure. | `apps/cli/src/main.rs`; rendered Rustdoc at `target/doc/cli/index.html`; app tool inventory Rustdoc at `target/doc/app/tools/enum.ExternalToolCandidate.html`. | App tool/agent contracts are covered by `app` tests; current CLI behavior is thin JSON printing. | Good. Non-coders can understand it as inspection-only, not workflow execution. A future richer CLI should add command-level smoke/snapshot tests. |
| `app::tools::ExternalToolCandidate` and tool ports | App-owned inventory and typed capability ports for future external systems: Gingr, payment, messaging, file storage, OCR, cameras, Hermes, and Postgres. | `app/README.md`, `app/src/tools/README.md`, `apps/cli/README.md`, and atlas/glossary docs explain that candidate/port does not imply live implementation. | `app/src/tools.rs`; rendered Rustdoc at `target/doc/app/tools/enum.ExternalToolCandidate.html`. | `app/tests/application_quality_patterns.rs`; CLI `pet-resort tools` exposes the source-controlled inventory. | Strong. Docs repeatedly say candidate/tool-port is a contract boundary, not granted side-effect authority. |

## Findings

### Passes

1. The runtime/storage docs generally lead with operational meaning before implementation detail. `storage/README.md`, `apps/api/README.md`, `apps/worker/README.md`, `apps/cli/README.md`, and `docs/design/entity-atlas-runtime-storage-api-surfaces.md` explain what each surface stores or runs and what not to infer from it.
2. Authoritative paths are clear: Markdown pages point to source files; representative public items have Rustdoc; tests exercise storage codecs, API draft/outcome behavior, worker runtime modes, and app tool boundaries.
3. Runtime side-effect boundaries are understandable to non-coders. API health/readiness docs, worker mode docs, CLI docs, and tool-port docs distinguish local/test/stubbed shells from live customer/provider/payment/schedule authority.
4. Storage boundary semantics are mostly strong. Records/codes/codecs/scalars/source refs are described as projection and promotion/demotion artifacts that preserve evidence; they do not replace `domain::*`, `app::*`, or provider-source truth.

### Issues to fix or track

1. Stale/over-claimed worker crate-root Rustdoc in `apps/worker/src/lib.rs`:
   - Current wording: “Worker shell for durable Postgres-backed pet-resort workflows.”
   - Why it matters: `apps/worker/README.md` says durable leasing, queue consumers, schedulers, concrete app services, and provider writes are not implemented yet. A non-coder could infer a deployed durable Postgres worker exists.
   - Suggested fix: change the crate root to “Worker shell for safe local/background pet-resort workflow execution” or mention Postgres only as a future adapter/storage implementation.

2. Stale copied Rustdoc sentence in `storage/src/operations.rs`:
   - `StoredManagerDailyBriefLaborMinutes::try_new` says “Validates and wraps a non-empty brand name before persistence.”
   - Why it matters: the type validates non-zero labor minutes, not a brand name. The generated Rustdoc exposes a misleading implementation-detail fragment.
   - Suggested fix: “Validates and wraps a non-zero labor-minute quantity before persistence.”

3. `storage/README.md` navigation lags the current data-quality storage surface:
   - `DataQualityHygieneOutcomeRecord`, `DataQualityHygieneOutcomeCode`, and `StoredDataQualityHygieneLaborMinutes` have source, Rustdoc, and tests, but the README type/module map and labor-cost examples still foreground only manager daily brief outcome storage.
   - Why it matters: non-coders following the storage README may miss that data-quality hygiene outcome capture is now a durable storage projection too.
   - Suggested fix: add rows/bullets for data-quality hygiene outcome record, codes, and labor scalar alongside the manager daily brief entries.

4. Runtime shell public evidence is present but indirect in some places:
   - The public landing links crate roots for `pet_resort_api/`, `pet_resort_worker/`, and `cli/`, while the most useful plain-language details live in READMEs and source/test paths.
   - This is acceptable for repo-local docs; if public-doc discoverability becomes a requirement, add a small “Runtime shells” evidence section naming API/worker/CLI responsibilities and linking the rendered crate roots.

## Missing Rustdoc/tests assessment

- Missing Rustdoc: no blocker found for the representative public runtime/storage entities above. The strict missing-docs gate passed for the workspace.
- Missing tests: no blocker found for representative public runtime/storage contracts. Manager/data-quality storage outcomes, API outcome/draft/context behavior, worker runtime modes, and app tool-port semantics all have tests. CLI lacks a dedicated command-level integration/snapshot test, but the current command behavior is small/read-only and indirectly backed by `app` tests.

## Recommended remediation order

1. Fix the two stale Rustdoc comments: `apps/worker/src/lib.rs` and `storage/src/operations.rs`.
2. Update `storage/README.md` to include `DataQualityHygieneOutcomeRecord`, its outcome codes, and `StoredDataQualityHygieneLaborMinutes` in the type/module and labor-cost maps.
3. Optionally add public runtime-shell evidence links if the published landing page needs more direct non-coder runtime discoverability.

## Bottom line

The runtime/storage docs evidence surface is coherent and source-grounded. A non-coder can understand the key distinction between durable evidence, local runtime shells, app workflow packets, source refs, and live operational authority. Remaining issues are localized stale/omitted documentation, not missing core Rustdoc or missing contract tests.
