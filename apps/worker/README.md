# `apps/worker`

`apps/worker` is the background-agent runtime shell for the pet-resort workspace. It is intentionally thin: it boots tracing, reads runtime mode from environment, and holds the current worker-side safety switch for agent/runtime side effects. Durable leasing, queue consumers, schedulers, concrete app services, and provider writes are explicitly not implemented here yet; [`src/main.rs`](./src/main.rs) logs that those pieces are reserved for downstream workflow/data-model cards.

Start with the crate root in [`src/lib.rs`](./src/lib.rs), which exposes [`worker::runtime`](./src/runtime.rs). The executable entry point is [`src/main.rs`](./src/main.rs). Runtime configuration and side-effect-mode types live in [`src/runtime.rs`](./src/runtime.rs), with their current behavior covered by [`tests/runtime_mode_contract.rs`](./tests/runtime_mode_contract.rs).

## What this shell owns

`apps/worker` owns process/runtime concerns for future background work:

1. Runtime-mode configuration for whether the local worker uses deterministic fake agent output or disables the agent path entirely. See [`runtime::Config::from_env_defaults`](./src/runtime.rs), [`runtime::AgentRuntimeMode`](./src/runtime.rs), and the `PET_RESORT_AGENT_RUNTIME_MODE` branch in [`src/runtime.rs`](./src/runtime.rs).
2. A fail-safe side-effect posture. [`runtime::SideEffectMode::Stubbed`](./src/runtime.rs) is the only implemented side-effect mode, and [`runtime::Config::from_env_defaults`](./src/runtime.rs) always returns it.
3. Worker process boot and shutdown plumbing. [`src/main.rs`](./src/main.rs) initializes JSON tracing, logs the selected `agent_runtime_mode` and `side_effect_mode`, and waits for `tokio::signal::ctrl_c()`.
4. The boundary where future queue/schedule/agent-runtime wiring can be attached to `app` workflow ports without moving business policy into the shell.

`apps/worker` does not own business/domain contracts, app workflow packets, storage records, Gingr DTOs, HTTP routes, live customer messaging, PMS/provider mutation, payment moves, or OpenViking operational truth. Those belong to the related crates and docs linked below.

## Runtime/config/side-effect map

The worker currently has three public runtime concepts:

- [`runtime::Config`](./src/runtime.rs) stores the process-local runtime switches. It is built by [`runtime::Config::from_env_defaults`](./src/runtime.rs) and read through [`runtime::Config::agent_runtime_mode`](./src/runtime.rs) and [`runtime::Config::side_effect_mode`](./src/runtime.rs).
- [`runtime::AgentRuntimeMode`](./src/runtime.rs) names the agent-execution posture. `PET_RESORT_AGENT_RUNTIME_MODE=disabled` selects `AgentRuntimeMode::Disabled`; every other value, including the unset default, selects `AgentRuntimeMode::FakeDeterministic`.
- [`runtime::SideEffectMode`](./src/runtime.rs) names the side-effect posture. Its only current variant is `SideEffectMode::Stubbed`, so local development and CI cannot accidentally send customer messages or write to provider systems through this worker shell.

That map matters because the rest of the repository already distinguishes deterministic application truth from agent assistance. The [`app::tools::AgentRuntime`](../../app/src/tools.rs) port describes structured agent execution for app workflows; [`app::tools::hermes::AutomationHooks`](../../app/src/tools.rs) describes draft-only Hermes task/schedule hooks; and [`app::tools::ExternalToolCandidate`](../../app/src/tools.rs) lists candidate external systems such as Hermes Kanban, Hermes cron/webhook, Gingr, messaging, file storage, OCR/document AI, cameras, payment, and Postgres. `apps/worker` is the natural runtime place to wire concrete implementations of those ports later, but the contracts remain in [`app`](../../app/README.md).

## Workflow role

Future background workflows should use this shell to run app-owned work asynchronously while preserving the workspace architecture from the root [`README`](../../README.md): `domain` defines what is true, `app` defines what the system needs to do, adapters satisfy those contracts, and runtime crates boot/wire the process.

Examples of work that belongs behind this shell once implemented:

- scheduled manager daily brief generation using typed `app::manager_daily_brief` packets and source-grounded facts;
- draft-only Hermes task or schedule creation through [`app::tools::hermes::AutomationHooks`](../../app/src/tools.rs);
- deterministic polling/queue consumers that load storage-normalized facts, build app context packets, and submit reviewable drafts;
- local fake-agent smoke workflows that exercise app validation without live provider, payment, schedule, or customer-message side effects.

Examples of work that should not be hidden in this crate:

- pet-resort business invariants, which belong in [`domain`](../../domain/README.md);
- workflow request/evaluation/draft/audit types, which belong in [`app`](../../app/README.md);
- storage records and promotion/demotion codecs, which belong in [`storage`](../../storage/README.md);
- provider DTOs, endpoints, transport, response parsing, and mapping, which belong in [`integrations/gingr`](../../integrations/gingr/README.md);
- HTTP route contracts, which belong in [`apps/api/src/http.rs`](../api/src/http.rs) and are exposed by [`apps/api/src/lib.rs`](../api/src/lib.rs).

## Hermes tools, OpenViking context, and safe side-effect boundaries

The canonical agent/app boundary is documented in [`docs/architecture/agent-app-infrastructure.md`](../../docs/architecture/agent-app-infrastructure.md). The worker shell should follow that pattern:

- Deterministic Rust app code owns source facts, provenance, policy, workflow state, storage, audit logs, review gates, and external writes.
- Hermes/agent runtimes consume typed context and propose, draft, summarize, triage, rank, or route through constrained tools.
- Agent output re-enters the app as a draft/recommendation packet and is validated before review or execution.
- Customer sends, live provider/PMS mutation, schedule changes, payments, refunds, discounts, and safety-sensitive decisions stay blocked unless app policy and human review explicitly allow them.

OpenViking is optional agent memory/context infrastructure, not app-owned operational truth. [`docs/ops/openviking-local-memory.md`](../../docs/ops/openviking-local-memory.md) and the memory section of [`docs/architecture/agent-app-infrastructure.md`](../../docs/architecture/agent-app-infrastructure.md) state the boundary: OpenViking may help Hermes remember SOP context, implementation lessons, glossary terms, indexed docs, or reasoning patterns, but it cannot replace app persistence, current source refs, audit/replay records, review decisions, or side-effect authorization.

Because [`runtime::SideEffectMode`](./src/runtime.rs) only has `Stubbed` today, this crate is currently safe for docs, local development, and CI smoke runs. A future live side-effect mode should be added only with explicit app validation, review-gate evidence, idempotency/replay controls, redaction/evidence hygiene, and tests that prove blocked actions fail closed.

## Type/module map

| Concept | Public type/module path | Defined in | Current role |
| --- | --- | --- | --- |
| Crate root | `pet_resort_worker` | [`src/lib.rs`](./src/lib.rs) | Exposes the worker shell module surface. |
| Runtime module | `pet_resort_worker::runtime` | [`src/runtime.rs`](./src/runtime.rs) | Holds process-local runtime configuration and safety modes. |
| Runtime configuration | `pet_resort_worker::runtime::Config` | [`src/runtime.rs`](./src/runtime.rs) | Stores `agent_runtime_mode` and `side_effect_mode`; built from environment defaults. |
| Agent runtime mode | `pet_resort_worker::runtime::AgentRuntimeMode` | [`src/runtime.rs`](./src/runtime.rs) | Selects fake deterministic agent behavior or disabled agent behavior for the shell. |
| Side-effect mode | `pet_resort_worker::runtime::SideEffectMode` | [`src/runtime.rs`](./src/runtime.rs) | Keeps side effects stubbed; no live mode exists yet. |
| Runtime config constructor | `pet_resort_worker::runtime::Config::from_env_defaults` | [`src/runtime.rs`](./src/runtime.rs) | Reads `PET_RESORT_AGENT_RUNTIME_MODE` and defaults to `FakeDeterministic` plus `Stubbed`. |
| Runtime accessors | `Config::agent_runtime_mode`, `Config::side_effect_mode` | [`src/runtime.rs`](./src/runtime.rs) | Expose the selected modes to the process entry point or future wiring. |
| Executable entry point | `pet_resort_worker` binary `main` | [`src/main.rs`](./src/main.rs) | Initializes tracing, logs runtime/safety modes, and waits for shutdown. |
| Runtime-mode contract test | `default_worker_runtime_is_fake_and_side_effect_safe` | [`tests/runtime_mode_contract.rs`](./tests/runtime_mode_contract.rs) | Verifies the default mode remains deterministic and side-effect safe. |
| App agent-runtime port | `app::tools::AgentRuntime` | [`../../app/src/tools.rs`](../../app/src/tools.rs) | App-facing contract for structured agent execution that a future worker adapter may implement. |
| Hermes automation port | `app::tools::hermes::AutomationHooks` | [`../../app/src/tools.rs`](../../app/src/tools.rs) | Draft-only task/schedule hook contract for Hermes Kanban/cron-style automation. |

## Cross-crate relationships

- [`domain`](../../domain/README.md) owns semantic pet-resort truth: entities, workflows, policies, source refs, audit values, review gates, and invariant-bearing types. The worker should consume those truths through `app` contracts, not redefine them.
- [`app`](../../app/README.md) owns use-case workflows and ports. It defines workflow packets such as booking triage, checkout completion, CRM retention, daily update, manager daily brief, local smoke, and tool contracts such as [`app::tools::AgentRuntime`](../../app/src/tools.rs) and [`app::tools::hermes`](../../app/src/tools.rs).
- [`storage`](../../storage/README.md) owns persistence-shaped records and promotion/demotion between storage and semantic domain values. Future worker jobs should use storage adapters as implementations of app-needed capabilities, not as a source of new workflow policy.
- [`integrations/gingr`](../../integrations/gingr/README.md) owns Gingr provider DTOs, endpoints, transport, response parsing, webhook verification, and mapping into source-agnostic/domain/app contracts. A worker may schedule or run Gingr sync work later, but provider vocabulary should stay quarantined there.
- [`apps/api`](../api/src/lib.rs) is the HTTP runtime shell. Its route implementation in [`apps/api/src/http.rs`](../api/src/http.rs) exposes staff/API workflows and keeps live-side-effect readiness explicit in health/readiness payloads.
- [`apps/cli`](../cli/src/main.rs) is the local/manual operator shell. It is useful for smoke workflows and scripted operator actions; long-running or scheduled execution belongs in `apps/worker` once implemented.
- [`docs/architecture/agent-app-infrastructure.md`](../../docs/architecture/agent-app-infrastructure.md) is the canonical agent/app boundary. [`docs/ops/openviking-local-memory.md`](../../docs/ops/openviking-local-memory.md) documents the optional OpenViking memory/context setup and its non-authoritative role.

## Labor-cost-reduction contribution

The worker shell contributes to the repo's labor-cost-reduction goal by giving background agents a safe process boundary before they become live operational automation. The current fake/stubbed defaults let maintainers develop manager briefs, draft messages, review packets, task drafts, and context-packet workflows without accidentally moving money, mutating Gingr/PMS records, sending customer messages, or creating unreviewed schedules.

That safety posture is part of the labor story: background workers should reduce manager/admin time by preparing source-grounded briefs, exception queues, draft tasks, and review packets, while deterministic app contracts and human review keep the costly failures — wrong source facts, unsafe customer communication, unauthorized provider writes, or untraceable decisions — out of the live path.

## Maintainer notes

- Keep this crate thin. If a new type represents business meaning, place it in `domain`; if it represents use-case workflow shape or a port, place it in `app`; if it represents persisted records, place it in `storage`; if it represents provider payloads, place it in `integrations/gingr`.
- Preserve semantic paths in docs and code: prefer `pet_resort_worker::runtime::SideEffectMode`, `app::tools::AgentRuntime`, `app::tools::hermes::AutomationHooks`, and `domain::policy::ReviewGate` over flattened names when the boundary matters.
- Do not add a live side-effect mode as a convenience flag. Add it only with app-owned validation, review gates, audit/replay, idempotency, and failing tests for unknown or blocked side effects.
- Treat OpenViking and Hermes memory as agent context, not as source truth. Accepted operational drafts still need current app-owned source refs and policy validation.
