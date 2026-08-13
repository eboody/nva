# Agent-App Infrastructure Guide

> **Purpose:** Guide the NVA repo toward applications that work safely with AI agents while preserving deterministic app ownership of facts, policy, workflow state, storage, and side effects.

## Core architecture principle

Deterministic apps own the operational truth. Agents assist.

```text
Deterministic apps own:
- source facts and provenance
- domain rules and policy
- workflow state
- storage and audit logs
- review gates
- external writes and side effects

Hermes / agents consume typed context and propose, draft, reason, triage, summarize, and route through constrained tools.
```

The goal is not to let an agent freely inspect databases or mutate app state. The goal is to build a safe agent sidecar/control-plane pattern where every agent action passes through typed app-owned contracts.

## Trust boundary and threat model

Treat the agent boundary as an untrusted integration boundary, even when Hermes runs in a first-party container. Agents can summarize, rank, draft, and recommend; they must not become the path that owns operational truth.

### Source-of-truth boundary

The deterministic app owns:

- source-system reads, normalized source refs, and provenance;
- business policy, review-gate selection, and blocked-action decisions;
- persistence, replay state, audit logs, and correlation IDs;
- all writes to provider/PMS systems, customer channels, schedules, payments, refunds, discounts, and staff task systems.

Hermes/agents own only transient reasoning over typed context. Raw database access, raw object-store access, ad hoc SQL, and direct provider API credentials are not the primary integration pattern. If an agent needs a fact, the app exposes it through a typed context packet. If an agent proposes an action, the app validates it as a draft before any human sees or executes it.

### Context packet requirements

Every context packet is an app-produced contract, not a prompt dump. It should include:

- workflow name and schema version;
- stable `context_packet_id` plus request/replay `correlation_id`;
- location, operating day, persona, and intended review surface;
- typed facts grouped by source-system concept, not by agent prompt section;
- source refs for each operational claim, including system name, source record id/key, observed timestamp, adapter/version, and any transformation step;
- data-quality issues and ambiguity flags preserved as first-class facts;
- allowed agent actions, blocked actions, required review gates, and redaction/sensitivity metadata.

A packet with missing provenance can still be useful for a human-facing warning, but it must not produce an accepted operational recommendation. No source refs, no accepted draft.

### Draft submission validation

Agent output re-enters the app as a draft/recommendation packet. The app validates:

- the referenced `context_packet_id` and `correlation_id` exist and are replayable;
- action kinds are in the allowed set for that workflow;
- every claim/action cites source refs from the packet;
- required review gates match the workflow policy;
- blocked side effects are absent;
- customer-facing language, policy claims, and labor-savings claims satisfy deterministic checks.

Rejected drafts are audit events, not silent failures. Store the reason so future evaluations can distinguish agent hallucination, stale context, missing evidence, and app-policy rejection.

### Human review gates

Human review is a contract boundary, not a UI afterthought. Staff can approve, defer, suppress, correct, or mark source facts wrong. Approval records should name the actor/persona, decision, timestamp, source refs considered, and resulting side-effect eligibility. Sensitive areas — incidents, pet safety, medical/vaccine ambiguity, refunds, discounts, payments, schedule/PMS mutation, and customer sends — remain review-gated until the app has explicit smoke/evaluation evidence and a deliberate policy change.

### Audit, replay, and correlation

Every context packet, draft submission, validation result, review decision, side-effect attempt, and outcome record should share correlation IDs. The app should be able to replay a workflow from persisted source refs and packet versions to answer:

- what facts did the agent see?
- which policy version allowed or blocked the recommendation?
- who reviewed it?
- what changed externally, if anything?
- how many minutes were estimated and actually saved?

Replayability is what makes labor-cost reduction measurable instead of anecdotal.

### Memory and knowledge boundary

OpenViking or similar memory/knowledge infrastructure can help agents remember implementation lessons, SOP context, glossary terms, and historical reasoning patterns. It must not replace app persistence, source refs, audit logs, review decisions, or operational source-of-truth records. Agent memory may enrich reasoning; it cannot authorize a fact, prove provenance, or justify a live side effect. If remembered knowledge influences a recommendation, the accepted draft still needs current app-owned source refs and policy validation.

## Recommended high-level shape

```text
             ┌────────────────────────────┐
             │        Staff Web UI         │
             │  review, approve, feedback  │
             └─────────────┬──────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────┐
│                 Deterministic App API              │
│ Rust app owns: domain rules, policy, review gates, │
│ persistence, source refs, audit logs, writes       │
└─────────────┬───────────────────────┬──────────────┘
              │                       │
              ▼                       ▼
   ┌──────────────────┐      ┌──────────────────┐
   │ Agent Context API │      │ Agent Command API │
   │ read-only packets │      │ draft/review only │
   └─────────┬────────┘      └─────────┬────────┘
             │                         │
             ▼                         ▼
      ┌────────────────────────────────────┐
      │ Hermes Agent Runtime / Sidecar      │
      │ - reads typed context               │
      │ - calls safe app tools              │
      │ - drafts briefs/messages/tasks      │
      │ - never bypasses app policy         │
      └────────────────────────────────────┘
```

## Container topology

The current repo already has local Docker Compose services for Postgres and MinIO, plus an optional `agent-infra` profile for local OpenViking agent memory/context infrastructure. OpenViking stays on the agent side of the boundary: it can enrich Hermes context, but it cannot replace app-owned facts, source refs, persistence, audit, review decisions, or side-effect controls. Over time, evolve the compose stack toward explicit app and agent services:

```yaml
services:
  postgres:
    image: postgres:17-alpine
    # existing local database service

  minio:
    image: minio/minio
    # existing local object-storage service

  pet-resort-api:
    build:
      context: .
      dockerfile: apps/api/Dockerfile
    environment:
      DATABASE_URL: postgres://pet_resort:pet_resort@postgres:5432/pet_resort
    depends_on:
      - postgres

  pet-resort-worker:
    build:
      context: .
      dockerfile: apps/worker/Dockerfile
    depends_on:
      - postgres
      - minio

  staff-web:
    build:
      context: staff-web
    depends_on:
      - pet-resort-api

  agent-runtime:
    image: hermes-agent:local
    environment:
      PET_RESORT_API_URL: http://pet-resort-api:3001
      # model/provider secrets should come from env, 1Password, or deployment secrets
    depends_on:
      - pet-resort-api
```

The important boundary is not Docker itself. The important boundary is that the agent runtime talks to the deterministic app through typed APIs/tools rather than raw Postgres, raw object storage, or provider-specific records.

The first minimal Manager Daily Brief bridge is intentionally scripts-based: `scripts/hermes-tools/get_manager_daily_brief_context`, `scripts/hermes-tools/submit_manager_daily_brief_draft`, and `scripts/hermes-tools/record_manager_daily_brief_outcome`. See `docs/ops/hermes-manager-daily-brief-bridge.md` for worker-profile usage and the later MCP migration note.

## Agent/app interface pattern

Each agent workflow should expose three app-owned surfaces.

### 1. Read-only context packets

Example:

```http
GET /agent/context/manager-daily-brief?location_id=...&operating_day=...
```

The app returns a typed context packet:

```json
{
  "location_id": "...",
  "operating_day": "2026-06-17",
  "service_demand_facts": [],
  "checkout_exceptions": [],
  "retention_opportunities": [],
  "data_quality_issues": [],
  "allowed_agent_actions": [
    "summarize_source_evidence",
    "rank_manager_actions",
    "draft_internal_tasks",
    "estimate_labor_minutes_saved"
  ],
  "blocked_actions": [
    "change_staff_schedule",
    "mutate_pms_record",
    "send_customer_message",
    "move_money"
  ]
}
```

This gives Hermes enough context to reason without letting it invent operational facts.

Context packets should include:

- workflow name and version;
- source refs and provenance;
- data-quality issues;
- policy/review gates;
- allowed agent actions;
- blocked actions;
- correlation IDs for audit and replay;
- redaction/sensitivity metadata where needed.

### 2. Draft/recommendation command endpoints

Example:

```http
POST /agent/drafts/manager-daily-brief
```

Hermes submits a draft/recommendation packet:

```json
{
  "context_packet_id": "...",
  "actions": [
    {
      "kind": "review_demand_against_staffing_plan",
      "rationale": "...",
      "source_refs": [],
      "estimated_minutes_saved": 30,
      "requires_review_gate": "manager_approval"
    }
  ]
}
```

The app validates the submitted packet. If Hermes includes unsupported actions, missing source refs, unsafe claims, wrong review gates, or forbidden side effects, the deterministic app rejects it.

Draft endpoints should be strict:

- no source refs, no accepted draft;
- no policy-compatible review gate, no accepted draft;
- no unsupported action kinds;
- no live external writes;
- no customer-facing send without explicit approval flow;
- no payment/refund/discount movement;
- no schedule or provider/PMS mutation from an agent draft.

### 3. Human-review and outcome capture

Example:

```http
POST /manager-daily-brief/actions/{id}/outcome
```

Captures staff or manager feedback:

```json
{
  "outcome": "completed",
  "actual_minutes": 12,
  "actor": "front_desk_lead",
  "feedback": "Useful, but checkout exception was already resolved"
}
```

Outcome capture turns agent output into labor-cost evidence. It should record:

- completed/deferred/suppressed/wrong-source outcome;
- actual minutes spent;
- actor/persona;
- feedback;
- source refs when outcome depends on source facts;
- timestamp and audit correlation ID.

## Hermes integration options

The migration path is **HTTP tools first, MCP later**. Start with narrow HTTP tool scripts against app-owned endpoints because they are easy to test, log, and evolve while the workflow contract settles. Promote stable endpoints into MCP only after the context packet, draft validation, review gate, and audit semantics are proven by smoke tests.

### Option A: Hermes sidecar with HTTP tools

This is the fastest and best first step.

- Run Hermes in its own container or profile.
- Give it a small allowlisted set of tools that call the app API:
  - `get_manager_daily_brief_context`
  - `submit_manager_daily_brief_draft`
  - `record_manager_daily_brief_outcome`
  - later, `search_sop` or `get_policy_context`
- Keep app validation deterministic.
- Keep all live external writes behind app-owned policy and human review.

This is the recommended first implementation path for this repo.

### Option B: App exposes an MCP server

This is a cleaner longer-term contract.

- The deterministic app exposes MCP tools.
- Hermes connects via a configured MCP server.
- Tool schemas become first-class and discoverable.
- Each app can declare exactly which agent-safe capabilities it exposes.

Example eventual command shape:

```bash
hermes mcp add nva-app --url http://pet-resort-api:3001/mcp
hermes mcp test nva-app
hermes mcp configure nva-app
```

The repo can start with HTTP tools and later wrap/promote them into MCP.

### Option C: Hermes as an enterprise agent control plane

Longer term, the agent platform can coordinate tools across multiple deterministic services:

```text
Hermes / Agent Platform
  ├── Pet Resort API tools
  ├── Gingr/source adapter tools
  ├── SOP/RAG tools
  ├── BI/read-model tools
  ├── review/reputation tools
  ├── scheduling/labor tools
  └── audit/evaluation/reporting tools
```

This is the scalable NVA AI-program shape, but it should be earned through one workflow first.

## Recommended first repo slice

Start with **Option A**, designed so it can become MCP later.

First slice:

> Containerized app + agent sidecar for Manager Daily Brief.

Concrete work:

1. Add Dockerfiles for:
   - `apps/api`
   - `apps/worker`
   - optionally `staff-web`
2. Extend `docker-compose.yml` with:
   - `pet-resort-api`
   - `pet-resort-worker`
   - `agent-runtime`
3. Add app API endpoints:
   - `GET /agent/context/manager-daily-brief`
   - `POST /agent/drafts/manager-daily-brief`
   - `POST /manager-daily-brief/actions/:id/outcome`
4. Add a tiny Hermes tool bridge:
   - local Python/HTTP tool scripts first;
   - later convert to MCP.
5. Add staff UI surface:
   - view daily brief;
   - approve/defer/suppress actions;
   - record actual minutes.
6. Add smoke test:
   - app creates context packet;
   - Hermes/tool bridge reads it;
   - draft is submitted;
   - app validates review gates/source refs;
   - outcome is captured;
   - report shows estimated vs actual minutes saved.

## Safety and ownership rules

Hermes should never be the source of truth.

Hermes can say:

> Based on source facts A/B/C, I recommend these three manager actions.

The deterministic app decides:

- is this action allowed?
- are source refs present?
- is the review gate correct?
- is this draft safe?
- can this be shown to staff?
- can this ever become a live action?

The app should reject any agent output that attempts to bypass policy or invent facts.

## How this maps to the NVA context pack

The NVA Pet Resorts context pack frames the highest-value work as:

> turn enterprise Claude into safe, measurable operational agents.

This infrastructure is exactly that. It gives the repo reusable rails for:

- Manager Daily Brief;
- SOP assistant;
- lead conversion;
- grooming rebooking;
- customer response drafts;
- regional exception reporting;
- data-quality hygiene.

The first workflow should be the one with the strongest current repo support: **Manager Daily Brief + labor minutes saved**. The broader product crosswalk lives in [../design/labor-cost-reduction-crosswalk.md](../design/labor-cost-reduction-crosswalk.md): use it to decide which future workflows deserve source/read-model work, deterministic app contracts, and review-gated agent loops next.

## Acceptance criteria for the infrastructure

The infrastructure is doing its job when:

- agents read typed context packets instead of raw databases;
- every agent recommendation cites source evidence;
- deterministic app code validates every submitted draft;
- unsafe actions are blocked even if the agent suggests them;
- staff can review, approve, defer, suppress, or correct outputs;
- outcome capture records actual labor impact;
- smoke tests prove the full loop without live customer/PMS/payment side effects;
- the pattern can be reused for the second and third agents without rebuilding the rails.

## Anti-goals

Do not build:

- a generic chatbot that is disconnected from operational workflows;
- direct agent access to production databases as the primary integration pattern;
- agent-owned business rules or policy decisions;
- unreviewed live customer sends;
- unreviewed provider/PMS writes;
- refund, discount, payment, or schedule mutation by agent output;
- labor-savings claims without outcome capture.

## Future implementation plan sketch

A full implementation plan should break this into small TDD tasks:

1. Document the app/agent contract and threat model.
2. Add `pet-resort-api` Dockerfile and compose service.
3. Add `pet-resort-worker` Dockerfile and compose service.
4. Add Manager Daily Brief context DTO and API contract test.
5. Add draft submission DTO and rejection tests for missing source refs/review gates.
6. Add outcome capture persistence and contract tests.
7. Add staff-web daily brief surface.
8. Add local Hermes HTTP tool bridge.
9. Add end-to-end smoke test for context → draft → review/outcome → labor report.
10. Promote stable HTTP tools into MCP if/when the contract settles.
