# Pet Resort Agent Foundation

A Rust-first spike for a 170-location pet resort workflow/agent platform.

The first step is to make the business boundary explicit before writing agents:

1. Model the core entities as Rust types.
2. Model workflow events and structured results.
3. Model automation/review policy as traits and decisions.
4. Model external systems as tool traits.
5. Let Hermes agents consume/produce typed packets rather than free-form strings.

## Architecture

This workspace follows a ports-and-adapters shape with one important rule:

> `domain` defines what is true. `app` defines what the system needs to do. Adapters satisfy those app-defined contracts. Runtime crates only wire and boot the process.

```text
domain
  ↑
app
  ↑
storage / integrations/*
  ↑
apps/api / apps/worker / apps/cli
```

### Crate responsibilities

- `domain` — pure pet-resort language: entities, service-line contracts, workflow events, policies, review gates, semantic IDs/statuses, and invariant-bearing values. This crate should not know about databases, HTTP servers, queues, or provider APIs.
- `app` — application use-cases and ports. Workflows such as booking triage or daily updates compose domain contracts and declare the repository/service/tool traits they need, e.g. customer lookup, reservation availability, payment gateway, portal lookup, messaging draft, document intake, media capture, and agent runtime.
- `storage` — persistence adapter boundary. It owns storage-shaped records, stable storage codes, codecs, and promotion/demotion between storage records and `domain` values. As concrete persistence is built out, storage implementations should satisfy traits defined by `app`, not define application behavior themselves.
- `integrations/gingr` — Gingr provider adapter boundary. It owns provider DTOs, endpoint request/response contracts, transport, webhook verification, and mappings into domain/app contracts while quarantining provider vocabulary inside the integration crate.
- `apps/api` — thin HTTP runtime shell. It should parse/serialize HTTP, load config, wire concrete adapters into app services, and expose routes.
- `apps/worker` — thin background-worker runtime shell. It should wire queues/schedules to app services without owning business contracts.
- `apps/cli` — thin operator CLI shell for local/manual workflows.

### Port ownership

Repository and service traits belong in `app` when they describe capabilities the application needs:

```rust
app::customer::Repository
app::reservation::Repository
app::payment::Gateway
app::portal::Lookup
app::agent::Runtime
```

Concrete adapters implement those ports:

```rust
storage::customer::postgres::Repository     -> app::customer::Repository
storage::reservation::memory::Repository    -> app::reservation::Repository
integrations::gingr::Portal                 -> app::portal::Lookup
```

The runtime crates should compose these pieces rather than define the contracts. `apps/api` and `apps/worker` may define runtime-local glue such as shutdown signals or job runners, but business/application traits should stay in `app` so every runtime shares the same contract surface.


## Rust quality conventions

This repo should bias toward making invalid business states unrepresentable early:

- Use `nutype` for semantic scalar values that need trimming, non-empty checks, length limits, or future validation.
- Use `bon` for ordinary domain builders so required fields are compile-checked and call sites stay named. Entity aggregates with many meaningful fields (`Pet`, `Reservation`, `Customer`) should expose builders with defaults for optional/collection fields instead of forcing raw struct literals everywhere.
- Use `statum` when a workflow phase should change what methods are legally callable; e.g. booking triage must attach pet profile and policy context before it can become ready for deterministic policy decisions.
- Keep semantic names module-qualified at call sites (`agent::Spec`, `pet::Name`, `booking_triage::Request<Intake>`) rather than flattening everything into globally verbose type names. For ergonomic consumers, `domain::prelude` re-exports common boundary contracts with disambiguated aliases such as `AgentName` and `AgentSpec`.

The `domain_quality_patterns` integration test is the living example for these conventions. See `docs/architecture/domain-quality-gate.md` for the rule that implementation work should pause for prerequisite domain refactors when a tweak exposes weak abstractions, and `docs/quality/semantic-code-doctrine-inventory.md` for the current completed/debt inventory.

## Call prep questions for Tyler

1. Is the 170-location operator actually NVA/PetSuites, a franchise/network, or a similar pet-resort group?
2. What is the incumbent source of truth: Gingr, another kennel/PMS system, custom software, spreadsheets, or mixed by location?
3. Are they asking for replacement software, an AI workflow layer on top of the incumbent system, or internal automation prototypes?
4. Which first workflow matters most: inquiry intake, booking triage, vaccine/document review, daily updates, incident escalation, or staff operations?
5. Do agents need live write access initially, or should v1 be draft/recommendation-only?
6. What data access can they provide: sandbox portal, API docs, exports, database snapshots, sample PDFs/photos/notes, or screen recordings?
7. What are the hard compliance/safety lines for medical/vaccine decisions, group-play eligibility, incidents, refunds/deposits, and customer messages?
8. What channels matter first: email, SMS, portal messages, phones/transcripts, webcams, internal task queues?
9. What does success look like in 30 days: architecture, prototype, one workflow in pilot, or cross-location platform plan?

## Suggested first engineering slice

Build a deterministic `booking_triage` pipeline around typed inputs/outputs:

- Input: reservation request + pet profile + vaccine state + location policy + capacity snapshot.
- Deterministic stage: hard stops and eligibility rules.
- Agent stage: draft explanation, missing-info questions, and internal tasks.
- Output: `WorkflowResult<BookingTriageOutput>` with review gates.

This demonstrates the right architecture: normal Rust application logic owns policy, state, and writes; Hermes/LLM agents are bounded assistants that receive typed context and return validated structured output.
