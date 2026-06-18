# `apps/cli`

`apps/cli` is the local operator/developer command shell for the pet-resort workspace. It is intentionally thin: the executable in [`src/main.rs`](./src/main.rs) parses a `pet-resort` command with [`clap`](https://docs.rs/clap), calls existing `app` crate functions, and prints JSON to stdout for manual inspection or scripted smoke checks.

The CLI does not own business policy, storage records, provider DTOs, HTTP routes, worker scheduling, or live side effects. Its current job is to expose a small manual surface over app/domain contracts so maintainers can inspect baseline agent specs and external tool candidates without starting the API or worker processes.

## Entry point and command surface

Start at [`src/main.rs`](./src/main.rs). The file contains the complete current command surface:

- `pet-resort agents` maps to the private `Command::Agents` branch in [`src/main.rs`](./src/main.rs). It calls [`app::agents::baseline_agent_specs`](../../app/src/agents.rs) and serializes the resulting `Vec<app::agents::AgentSpec>` as pretty JSON. Those specs are `domain::agent::Spec` values built with `domain::agent::Name`, `domain::agent::Purpose`, `domain::agent::ToolName`, `domain::agent::ForbiddenAction`, and `domain::policy::ReviewGate` in [`domain/src/agent.rs`](../../domain/src/agent.rs) and [`domain/src/policy.rs`](../../domain/src/policy.rs).
- `pet-resort tools` maps to the private `Command::Tools` branch in [`src/main.rs`](./src/main.rs). It prints the current hard-coded inventory of [`app::tools::ExternalToolCandidate`](../../app/src/tools.rs) variants: `GingrPortal`, `PaymentProvider`, `SmsProvider`, `EmailProvider`, `FileStorage`, `OcrOrDocumentAi`, `CameraOrWebcamProvider`, `HermesKanban`, `HermesCronOrWebhook`, and `Postgres`.

There are no nested subcommands, configuration files, runtime adapters, network clients, database connections, or live provider mutations in this crate today. If a future CLI command needs to execute a workflow, the command should stay a shell over `app` services and adapter implementations rather than moving domain logic into `apps/cli`.

## What this shell exercises

The current commands are read-only JSON inspection tools:

1. `agents` exercises the app/domain agent-spec boundary. [`app::agents::baseline_agent_specs`](../../app/src/agents.rs) assembles source-controlled baseline specs for inquiry intake, booking triage, vaccine document review, daily updates, incident escalation, manager daily briefs, lead conversion, grooming rebooking, reputation triage, and SOP/policy assistance. Each spec carries allowed tool names, forbidden actions, and default review gates from `domain::policy::ReviewGate`.
2. `tools` exercises the app external-tool inventory. [`app::tools::ExternalToolCandidate`](../../app/src/tools.rs) names candidate systems that app workflows may eventually use through ports: Gingr/PMS portal lookup, payment, SMS/email messaging, file storage, OCR/document AI, camera/webcam capture, Hermes Kanban, Hermes cron/webhook, and Postgres.
3. Both commands exercise serialization boundaries. The CLI uses [`serde_json::to_string_pretty`](./src/main.rs) so maintainers can diff or inspect the app/domain vocabulary without adding a runtime API request.

Because this shell is thin, richer workflow behavior lives elsewhere. For executable local workflow fixtures, see [`app::local_smoke::run_fixture`](../../app/src/local_smoke.rs). For HTTP-facing demos and staff/API workflows, see [`apps/api/src/http.rs`](../api/src/http.rs). For future scheduled/background execution, see [`apps/worker`](../worker/README.md).

## Type/module map

| Concept | Public type/module path | Defined in | Current role |
| --- | --- | --- | --- |
| CLI crate | `cli` package / `pet-resort` binary | [`Cargo.toml`](./Cargo.toml), [`src/main.rs`](./src/main.rs) | Declares the operator/developer shell and its dependencies on `app`, `clap`, `serde_json`, and `anyhow`. |
| Parser root | private `Cli` | [`src/main.rs`](./src/main.rs) | `clap::Parser` entry point for the `pet-resort` binary. Private because this crate has no library API. |
| Subcommand enum | private `Command` | [`src/main.rs`](./src/main.rs) | `clap::Subcommand` enum with the current `Agents` and `Tools` commands. |
| Agents command | private `Command::Agents` | [`src/main.rs`](./src/main.rs) | Prints [`app::agents::baseline_agent_specs`](../../app/src/agents.rs) as pretty JSON. |
| Tools command | private `Command::Tools` | [`src/main.rs`](./src/main.rs) | Prints selected [`app::tools::ExternalToolCandidate`](../../app/src/tools.rs) variants as pretty JSON. |
| Baseline agent specs | `app::agents::baseline_agent_specs`, `app::agents::AgentSpec` | [`../../app/src/agents.rs`](../../app/src/agents.rs) | App-owned baseline agent inventory consumed by the CLI's `agents` command. |
| Agent spec domain values | `domain::agent::{Spec, Name, Purpose, ToolName, ForbiddenAction, OutputSchemaName, PolicyInstruction}` | [`../../domain/src/agent.rs`](../../domain/src/agent.rs) | Semantic agent vocabulary that backs `app::agents::AgentSpec` and prompt packets. |
| Review gates | `domain::policy::ReviewGate` | [`../../domain/src/policy.rs`](../../domain/src/policy.rs) | Safety/review requirements included in baseline agent specs. |
| External tool candidates | `app::tools::ExternalToolCandidate` | [`../../app/src/tools.rs`](../../app/src/tools.rs) | Candidate integration/tool inventory printed by the CLI's `tools` command. |
| Tool ports | `app::tools::{CustomerStore, ReservationSystem, AgentRuntime}`, `app::tools::{portal, payment, messaging, documents, media, hermes}` | [`../../app/src/tools.rs`](../../app/src/tools.rs) | App-owned port contracts that future CLI commands may exercise through concrete adapters. |

## Cross-crate relationships

- [`app`](../../app/README.md) owns the use-case contracts the CLI displays. The `agents` command reaches into [`app/src/agents.rs`](../../app/src/agents.rs); the `tools` command reaches into [`app/src/tools.rs`](../../app/src/tools.rs). Keep new command behavior pointed at app services, workflow packets, and tool ports.
- [`domain`](../../domain/README.md) owns semantic pet-resort truth. The CLI should preserve paths such as `domain::agent::Spec`, `domain::policy::ReviewGate`, and `domain::workflow::Event` instead of flattening those concepts into CLI-local names.
- [`storage`](../../storage/README.md) owns persistence records/codecs and promotion/demotion with `domain` values. This CLI currently does not open storage, but a future import/export or smoke command should use storage adapters as implementations of app ports.
- [`integrations/gingr`](../../integrations/gingr/README.md) owns Gingr provider DTOs, endpoints, transport, response parsing, webhook verification, and mapping. The CLI currently only prints `app::tools::ExternalToolCandidate::GingrPortal`; it does not call Gingr or parse provider payloads.
- [`apps/api`](../api/src/lib.rs) is the HTTP runtime shell for staff/API workflows; [`apps/api/src/http.rs`](../api/src/http.rs) contains the current route implementation. Use that shell for HTTP contract demos rather than adding route behavior to the CLI.
- [`apps/worker`](../worker/README.md) is the background runtime shell. Long-running queue consumers, schedules, and agent-runtime wiring belong there once implemented; the CLI remains the local/manual command surface.

## Labor-cost-reduction contribution

`apps/cli` contributes to the labor-cost-reduction goal by making the typed automation inventory inspectable without live infrastructure. A maintainer can run `pet-resort agents` to review which operational loops are being modeled as agent specs, what each agent is forbidden to do, and which `domain::policy::ReviewGate` values are required. They can run `pet-resort tools` to review the external systems the app layer expects to integrate before building concrete adapters.

That inspection path reduces coordination cost: operator/developer review starts from source-controlled JSON generated from `app` and `domain` types, not from stale spreadsheets or free-form agent prompts. The CLI is not itself the labor-saving automation; it is the thin manual shell that helps keep automation candidates, review gates, and integration boundaries visible while the API and worker runtimes mature.

## Maintainer notes

- Keep `apps/cli` thin. New commands should parse operator input, call `app` services or adapter-backed ports, and print reviewable output; business rules belong in `domain` and workflow decisions belong in `app`.
- Do not add live customer messaging, provider/PMS writes, payment actions, or schedule mutations as direct CLI shortcuts. Route those through app-owned review gates, tool ports, audit evidence, idempotency, and concrete adapters.
- Preserve semantic paths in prose and code: prefer `app::agents::baseline_agent_specs`, `app::tools::ExternalToolCandidate`, `domain::agent::Spec`, and `domain::policy::ReviewGate` when those boundaries matter.
- When the CLI surface grows, update this README from [`src/main.rs`](./src/main.rs) first. The command list above should match the `Command` enum exactly.
