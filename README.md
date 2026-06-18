# NVA pet-resort agent platform wiki

This workspace is a Rust-first foundation for safe pet-resort workflow automation across a multi-location operator. The repository is intentionally documentation-heavy: the READMEs are the maintainer wiki, and the Rust module paths are the glossary. Start here, then follow the crate and module READMEs into the code.

The product goal is labor-cost reduction, not generic chat. The useful surfaces are the ones that reduce avoidable manager/front-desk handoffs: source-data normalization, deterministic policy checks, review-gated agent drafts, exception triage, outcome capture, and reusable operational workflows. [nva-pet-resorts-ai-context.md](nva-pet-resorts-ai-context.md) is the repository-level acceptance lens for that work; [docs/design/labor-cost-reduction-crosswalk.md](docs/design/labor-cost-reduction-crosswalk.md) maps the major labor-cost drivers to repo surfaces and next loops; [docs/architecture/agent-app-infrastructure.md](docs/architecture/agent-app-infrastructure.md) describes the agent/app contract; [docs/design/manager-daily-brief-measurable-labor-loop.md](docs/design/manager-daily-brief-measurable-labor-loop.md) is the first measurable labor loop design; [docs/ops/data-quality-hygiene-local-smoke.md](docs/ops/data-quality-hygiene-local-smoke.md) proves the second workflow's local fake-data context -> draft -> outcome capture path; and [docs/audits/2026-06-18-agent-app-infrastructure-readiness.md](docs/audits/2026-06-18-agent-app-infrastructure-readiness.md) captures the current go/no-go posture.

## Documentation contracts

READMEs in this repository are wiki and navigation surfaces. They should explain the labor-cost lens, crate/module ownership, source-of-truth boundaries, and where a maintainer or agent should go next. They should not accumulate duplicate Rust snippets that can drift away from the compiled API.

Executable API examples belong in Rustdoc on the source modules and crate roots, where `cargo test --doc` can compile-check them as contracts. When documenting behavior, prefer a README link to the relevant source/Rustdoc surface over copying code into Markdown. If a README must include a non-executable sketch, mark it as conceptual and keep it source-grounded.

The current plan for strengthening this split is [docs/plans/2026-06-18-labor-cost-docs-as-contracts-kanban.md](docs/plans/2026-06-18-labor-cost-docs-as-contracts-kanban.md). Its rule of thumb is canonical here: READMEs remain the wiki; Rustdoc examples become compile-checked API contracts; local-link checks protect wiki navigation.

## Workspace map

The workspace members are declared in [Cargo.toml](Cargo.toml):

- `domain` — semantic truths: pet-resort entities, service-line contracts, workflow events, source provenance, policies, review gates, evidence, and invariant-bearing values. Start with [domain/README.md](domain/README.md) and [domain/src/lib.rs](domain/src/lib.rs).
- `app` — use cases and agent-safe workflows that compose `domain` concepts into typed packets, deterministic evaluations, draft validation, audit drafts, and tool-port contracts. Start with [app/README.md](app/README.md) and [app/src/lib.rs](app/src/lib.rs).
- `storage` — storage-shaped records/projections, stable codes, codecs, and explicit promotion/demotion between persisted records and `domain` values. Start with [storage/README.md](storage/README.md) and [storage/src/lib.rs](storage/src/lib.rs).
- `integrations/gingr` — Gingr provider adapter boundary: DTOs, endpoints, transport, webhooks, responses, and mapping into source-agnostic/domain concepts. Start with [integrations/gingr/README.md](integrations/gingr/README.md) and [integrations/gingr/src/lib.rs](integrations/gingr/src/lib.rs).
- `apps/api` — HTTP runtime shell over the app/domain contracts. It owns route shape and runtime policy gates, not the domain model. See [apps/api/src/lib.rs](apps/api/src/lib.rs), [apps/api/src/http.rs](apps/api/src/http.rs), and [apps/api/src/main.rs](apps/api/src/main.rs).
- `apps/worker` — background-worker runtime shell for deterministic/stubbed local execution and future durable jobs. Start with [apps/worker/README.md](apps/worker/README.md) and [apps/worker/src/lib.rs](apps/worker/src/lib.rs).
- `apps/cli` — operator/developer CLI shell for local inspection of app-owned agent/tool surfaces. Start with [apps/cli/README.md](apps/cli/README.md) and [apps/cli/src/main.rs](apps/cli/src/main.rs).

The dependency direction is deliberate:

```text
domain                 semantic truths and invariants
  ↑
app                    use cases, ports, workflow packets, agent-safe drafts
  ↑
storage / integrations persistence records and provider adapters
  ↑
apps/*                 runtime shells: HTTP, worker, CLI
```

Keep provider vocabulary, storage codes, HTTP details, and runtime concerns from leaking upward. If a concept becomes important to business logic, promote it into a semantic `domain::*` type and adapt at the boundary.

## README index

Crate READMEs:

- `domain`: [domain/README.md](domain/README.md)
- `app`: [app/README.md](app/README.md)
- `storage`: [storage/README.md](storage/README.md)
- `integrations/gingr`: [integrations/gingr/README.md](integrations/gingr/README.md)
- `apps/api`: [apps/api/README.md](apps/api/README.md)
- `apps/worker`: [apps/worker/README.md](apps/worker/README.md)
- `apps/cli`: [apps/cli/README.md](apps/cli/README.md)

Domain module READMEs:

- Boarding: [domain/src/boarding/README.md](domain/src/boarding/README.md)
- Daycare: [domain/src/daycare/README.md](domain/src/daycare/README.md)
- Grooming: [domain/src/grooming/README.md](domain/src/grooming/README.md)
- Money: [domain/src/money/README.md](domain/src/money/README.md)
- Payment: [domain/src/payment/README.md](domain/src/payment/README.md)
- Reservation: [domain/src/reservation/README.md](domain/src/reservation/README.md)
- Retail: [domain/src/retail/README.md](domain/src/retail/README.md)
- Training: [domain/src/training/README.md](domain/src/training/README.md)

Storage module READMEs:

- Service-line records/codecs: [storage/src/service_line/README.md](storage/src/service_line/README.md)

Gingr module READMEs:

- Endpoint requests: [integrations/gingr/src/endpoint/README.md](integrations/gingr/src/endpoint/README.md)
- Provider DTOs: [integrations/gingr/src/dto/README.md](integrations/gingr/src/dto/README.md)
- Mapping/promotion: [integrations/gingr/src/mapping/README.md](integrations/gingr/src/mapping/README.md)
- Integration docs and fixtures: [docs/integrations/gingr/README.md](docs/integrations/gingr/README.md), [docs/integrations/gingr/fixtures/webhooks/README.md](docs/integrations/gingr/fixtures/webhooks/README.md)

## Type/module map

The primary flow is:

```text
Gingr/provider payloads
  -> integrations::gingr::dto::* / integrations::gingr::endpoint::*
  -> integrations::gingr::mapping::* candidates and source refs
  -> domain::* semantic truths and source/provenance values
  -> app::* workflow requests, deterministic evaluations, review packets, drafts, and tool ports
  -> storage::* records/projections/outcome rows
  -> apps/* runtime shells for HTTP, workers, or CLI inspection
```

Important public surfaces:

| Concept | Public path | Representative files |
| --- | --- | --- |
| Workspace membership and shared dependencies | [Cargo.toml](Cargo.toml) workspace members/dependencies | [Cargo.toml](Cargo.toml) |
| Domain crate root | `domain::*` modules | [domain/src/lib.rs](domain/src/lib.rs), [domain/README.md](domain/README.md) |
| Service-line truths | `domain::boarding`, `domain::daycare`, `domain::grooming`, `domain::retail`, `domain::training` | [domain/src/boarding/mod.rs](domain/src/boarding/mod.rs), [domain/src/daycare/mod.rs](domain/src/daycare/mod.rs), [domain/src/grooming/mod.rs](domain/src/grooming/mod.rs), [domain/src/retail/mod.rs](domain/src/retail/mod.rs), [domain/src/training/mod.rs](domain/src/training/mod.rs) |
| Reservation policy and transitions | `domain::reservation::{MinimumAgeWeeks, AgeThreshold, TransitionReason}` | [domain/src/reservation/mod.rs](domain/src/reservation/mod.rs), [domain/src/reservation/error.rs](domain/src/reservation/error.rs) |
| Money and payment safety | `domain::money::{Money, MinorUnits, Currency}`, `domain::payment::*` | [domain/src/money/mod.rs](domain/src/money/mod.rs), [domain/src/payment/mod.rs](domain/src/payment/mod.rs) |
| Workflow review boundary | `domain::workflow::{PolicyContext, AllowedAction, Result, Status, RecommendedAction}` | [domain/src/workflow.rs](domain/src/workflow.rs) |
| Source/provenance boundary | `domain::source::{RecordRef, Provenance}`, `domain::source::reservation::*`, `domain::source::gingr::*` | [domain/src/source.rs](domain/src/source.rs) |
| Data-quality exception triage | `domain::data_quality::{Issue, FieldPath, Kind, Severity, ResolutionStatus}` | [domain/src/data_quality.rs](domain/src/data_quality.rs) |
| Manager daily brief truths | `domain::daily_brief::{Section, Risk, Action, LaborSnapshot}` | [domain/src/daily_brief.rs](domain/src/daily_brief.rs) |
| Agent specs and prompt packets | `app::agents::{AgentSpec, WorkflowAgent, AgentPromptPacket}` | [app/src/agents.rs](app/src/agents.rs) |
| Booking triage use case | `app::booking_triage::{Request, DeterministicResult, StaffEvaluationPacket, Service}` | [app/src/booking_triage.rs](app/src/booking_triage.rs) |
| Daily update / checkout / CRM workflows | `app::daily_update`, `app::checkout_completion`, `app::crm_retention` | [app/src/daily_update.rs](app/src/daily_update.rs), [app/src/checkout_completion.rs](app/src/checkout_completion.rs), [app/src/crm_retention.rs](app/src/crm_retention.rs) |
| Manager daily brief workflow | `app::manager_daily_brief::*` | [app/src/manager_daily_brief.rs](app/src/manager_daily_brief.rs) |
| External tool ports | `app::tools::{CustomerStore, ReservationSystem, AgentRuntime, ExternalToolCandidate}` and nested `availability`, `draft_update`, `portal`, `payment`, `messaging`, `documents`, `media`, `hermes` modules | [app/src/tools.rs](app/src/tools.rs), [app/src/tools/error.rs](app/src/tools/error.rs) |
| Storage operation records | `storage::operations::{RecordKind, CodecError, ManagerDailyBriefOutcomeRecord, PetResortPortfolioRecord, ServiceOfferingRecord}` | [storage/src/operations.rs](storage/src/operations.rs) |
| Storage service-line codecs | `storage::service_line::{boarding, daycare, grooming, retail, training}` contract records and stable codes | [storage/src/service_line/mod.rs](storage/src/service_line/mod.rs), [storage/src/service_line/boarding.rs](storage/src/service_line/boarding.rs), [storage/src/service_line/daycare.rs](storage/src/service_line/daycare.rs), [storage/src/service_line/grooming.rs](storage/src/service_line/grooming.rs), [storage/src/service_line/retail.rs](storage/src/service_line/retail.rs), [storage/src/service_line/training.rs](storage/src/service_line/training.rs) |
| Gingr provider config/transport | `gingr::{Provider, BaseUrl, ApiKey, Subdomain}`, `gingr::transport::{Transport, Client, RequestParts}` | [integrations/gingr/src/config.rs](integrations/gingr/src/config.rs), [integrations/gingr/src/transport.rs](integrations/gingr/src/transport.rs) |
| Gingr endpoint vocabulary | `gingr::endpoint::{Request, Method, DateRange, Limit, Reservations}` plus endpoint modules | [integrations/gingr/src/endpoint/mod.rs](integrations/gingr/src/endpoint/mod.rs), [integrations/gingr/src/endpoint/reservations.rs](integrations/gingr/src/endpoint/reservations.rs), [integrations/gingr/src/endpoint/owners_animals.rs](integrations/gingr/src/endpoint/owners_animals.rs), [integrations/gingr/src/endpoint/labor_ops.rs](integrations/gingr/src/endpoint/labor_ops.rs), [integrations/gingr/src/endpoint/commerce_retail.rs](integrations/gingr/src/endpoint/commerce_retail.rs) |
| Gingr DTO boundary | `gingr::dto::{ProviderSurface, grooming, retail, training}` | [integrations/gingr/src/dto/mod.rs](integrations/gingr/src/dto/mod.rs), [integrations/gingr/src/dto/grooming.rs](integrations/gingr/src/dto/grooming.rs), [integrations/gingr/src/dto/retail.rs](integrations/gingr/src/dto/retail.rs), [integrations/gingr/src/dto/training.rs](integrations/gingr/src/dto/training.rs) |
| Gingr semantic promotion | `gingr::mapping::{ProviderField, Error}`, `gingr::mapping::customer::ContactCandidate`, `gingr::mapping::pet::NameCandidate`, `gingr::mapping::retail::ProductCandidate` | [integrations/gingr/src/mapping/mod.rs](integrations/gingr/src/mapping/mod.rs), [integrations/gingr/src/mapping/customer.rs](integrations/gingr/src/mapping/customer.rs), [integrations/gingr/src/mapping/pet.rs](integrations/gingr/src/mapping/pet.rs), [integrations/gingr/src/mapping/retail.rs](integrations/gingr/src/mapping/retail.rs) |
| Gingr webhook/response wrappers | `gingr::webhook::{Envelope, Verified, Payload, Ack}`, `gingr::response::{Raw, Envelope}` | [integrations/gingr/src/webhook.rs](integrations/gingr/src/webhook.rs), [integrations/gingr/src/response.rs](integrations/gingr/src/response.rs) |
| API runtime shell | `pet_resort_api::http`, `pet_resort_api::http::VaccineDocumentState` | [apps/api/src/lib.rs](apps/api/src/lib.rs), [apps/api/src/http.rs](apps/api/src/http.rs), [apps/api/src/main.rs](apps/api/src/main.rs) |
| Worker runtime shell | `pet_resort_worker::runtime::{Config, AgentRuntimeMode, SideEffectMode}` | [apps/worker/src/lib.rs](apps/worker/src/lib.rs), [apps/worker/src/runtime.rs](apps/worker/src/runtime.rs) |
| CLI runtime shell | `pet-resort agents`, `pet-resort tools` commands over app surfaces | [apps/cli/src/main.rs](apps/cli/src/main.rs) |

## Cross-crate relationships

- `domain` is the semantic center. Its README ([domain/README.md](domain/README.md)) links the service-line modules and explains where truths such as `domain::workflow::AllowedAction`, `domain::source::Provenance`, and `domain::daily_brief::Action` live.
- `app` depends on `domain` and owns use-case orchestration. Its README ([app/README.md](app/README.md)) is the entrypoint for `app::booking_triage`, `app::manager_daily_brief`, `app::agents`, and `app::tools`.
- `storage` depends on `domain` so persisted records can promote/demote into semantic values. It should satisfy app-needed capabilities rather than inventing workflow behavior. See [storage/README.md](storage/README.md) and [storage/src/service_line/README.md](storage/src/service_line/README.md).
- `integrations/gingr` translates external/provider facts into source-grounded app/domain inputs. Use [integrations/gingr/src/dto/README.md](integrations/gingr/src/dto/README.md) for raw provider payloads, [integrations/gingr/src/endpoint/README.md](integrations/gingr/src/endpoint/README.md) for request construction, and [integrations/gingr/src/mapping/README.md](integrations/gingr/src/mapping/README.md) for semantic promotion.
- `apps/api`, `apps/worker`, and `apps/cli` are runtime shells. They should wire app services, parse runtime input, expose local/manual surfaces, and keep business contracts in `domain`/`app`.

## Navigation by job

- To understand the domain language, read [domain/README.md](domain/README.md), then the relevant module README under `domain/src/*/README.md`, then the linked Rust module.
- To add or review an agent-safe workflow, start in [app/README.md](app/README.md), then inspect [app/src/agents.rs](app/src/agents.rs), [app/src/tools.rs](app/src/tools.rs), and the specific workflow module such as [app/src/booking_triage.rs](app/src/booking_triage.rs) or [app/src/manager_daily_brief.rs](app/src/manager_daily_brief.rs).
- To connect a provider field, start in [integrations/gingr/README.md](integrations/gingr/README.md), keep raw fields in `integrations/gingr/src/dto`, build requests in `integrations/gingr/src/endpoint`, and promote only named, reviewed facts through `integrations/gingr/src/mapping` into `domain::source` or another domain module.
- To persist a concept, start in [storage/README.md](storage/README.md); add storage-shaped records/codes in [storage/src/operations.rs](storage/src/operations.rs) or `storage/src/service_line/*`, and keep conversion to/from `domain` explicit.
- To wire a runtime, start in [apps/api/src/http.rs](apps/api/src/http.rs), [apps/worker/src/runtime.rs](apps/worker/src/runtime.rs), or [apps/cli/src/main.rs](apps/cli/src/main.rs); runtime crates should stay thin.

## Rust quality conventions

The repo biases toward making invalid business states unrepresentable:

- Use `nutype` for semantic scalar values that need trimming, non-empty checks, length limits, or future validation.
- Use `bon` builders when construction has multiple meaningful fields and named call sites reduce review mistakes.
- Use `statum` when a workflow phase should change what methods are legally callable.
- Preserve semantic module paths in prose and code: prefer `domain::boarding::capacity::Policy` style paths over flattened aliases when the path carries meaning.
- Keep boundary ugliness quarantined. DTOs, database rows, HTTP bodies, and external status codes should convert into semantic values before they drive business decisions.

The living acceptance lens is labor saved with safety preserved: fewer manual checks, source-grounded drafts, deterministic triage, cleaner handoffs, stronger review packets, and better exception queues.

## Verification

For code changes, run:

```sh
cargo fmt --all -- --check
cargo test --workspace --no-run
```

For executable docs and wiki/navigation checks, run:

```sh
./scripts/check_docs.sh
```

That docs gate runs Rust doctests for the contract crates (`domain`, `app`, `storage`, and `gingr`) and then checks local Markdown links plus required README coverage. The contract crates opt into `rustdoc::broken_intra_doc_links` at their crate roots so broken Rustdoc item links fail the doctest/doc build instead of silently rotting.

For the canonical local gate, run:

```sh
./scripts/test.sh
```

It includes formatting, clippy, workspace tests, doctests, bridge script tests, and the Markdown docs gate. The checked-in `rust-toolchain.toml` pins the repo to the stable toolchain with rustfmt and clippy so these commands are repeatable in shells without a global rustup default.

For docs-only README changes, the deterministic README/wiki contract gate is:

```sh
./scripts/check_markdown_links.py
```

That gate scans local Markdown links without network or secrets, excludes generated/vendor trees such as `.git`, `target`, and `node_modules`, and asserts the root README links every workspace-crate README plus the major domain, storage, and Gingr navigation READMEs.
