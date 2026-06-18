# Hermes/OpenViking agent memory runbook

Purpose: configure and smoke-test OpenViking as optional Hermes agent memory/context infrastructure for NVA pet-resort agent-app work while preserving the deterministic app as the operational source of truth.

This is a labor-cost-reduction support service, not a workflow authority. OpenViking may help Hermes remember implementation lessons, SOP/project context, glossary terms, reasoning patterns, and indexed documents. It must not replace app-owned source refs, policy, review gates, persistence, audit/replay records, or side-effect controls.

## Boundary model

Keep three stores conceptually separate:

1. Compact Hermes memory
   - Built-in `MEMORY.md` and `USER.md` are always active in a Hermes profile.
   - They are small, curated, and injected at session start.
   - Use them for durable user/project/environment facts that must fit in the system prompt.

2. OpenViking provider/context
   - `memory.provider=openviking` enables an external Hermes memory provider in addition to compact memory.
   - OpenViking is for larger searchable knowledge, documents, project context, and provider tools such as `viking_browse`, `viking_search`, `viking_read`, `viking_remember`, and `viking_add_resource`.
   - OpenViking context can enrich reasoning, but it cannot authorize operational facts or side effects.

3. NVA deterministic app source-of-truth
   - The Rust app and its database own source facts, source refs, workflow state, policy validation, review decisions, persistence, audit, replay, and external writes.
   - Hermes/agents consume typed context packets and submit draft/recommendation packets through app-owned validation.
   - Accepted operational drafts still need current app-owned source refs. No source refs, no accepted operational draft.

## Local dev vs shared home-server OpenViking

Use the local Docker service when a future worker needs reproducible repo/dev setup without depending on the user's private infrastructure:

- endpoint: `http://127.0.0.1:1933`
- storage: Docker volume `pet_resort_openviking`
- compose profile: `agent-infra`
- scope: local agent memory/context only

Use a shared home-server or tailnet OpenViking endpoint when the operator intentionally wants multiple Hermes profiles/workers to share the same indexed knowledge base. In that case, the endpoint/account/user/agent values come from the operator's secret/config channel, not from this repo. Do not hard-code private hostnames, tailnet IPs, or API keys into committed files.

The local container exists for repo reproducibility. It does not imply production should run OpenViking beside the app, and it does not make OpenViking a dependency of deterministic app tests.

## Non-secret config shape

Hermes profile config should select the provider in `config.yaml`:

```yaml
memory:
  memory_enabled: true
  user_profile_enabled: true
  provider: openviking
```

Equivalent safe commands for the active Hermes profile:

```bash
hermes config set memory.memory_enabled true
hermes config set memory.user_profile_enabled true
hermes config set memory.provider openviking
```

Add provider connection values to that Hermes profile's `.env`, 1Password-backed environment, or deployment secret store. The variable names are part of the contract; real values are not committed:

```dotenv
OPENVIKING_ENDPOINT=http://127.0.0.1:1933
OPENVIKING_API_KEY=***
OPENVIKING_ACCOUNT=nva-local
OPENVIKING_USER=<developer-or-agent-user>
OPENVIKING_AGENT=hermes-nva-local
```

For a shared OpenViking service, keep the same variable names and replace the values through the operator's secret channel:

```dotenv
OPENVIKING_ENDPOINT=<shared-home-server-or-tailnet-url>
OPENVIKING_API_KEY=<secret-if-required>
OPENVIKING_ACCOUNT=<shared-account>
OPENVIKING_USER=<human-or-profile-user>
OPENVIKING_AGENT=<profile-or-agent-name>
```

`OPENVIKING_API_KEY` may be optional for a local unauthenticated service. If a local provider treats any non-empty value as a real key, remove the placeholder from the local `.env` or set it blank instead of using `***`. If a key is required, store it only in `.env`, 1Password, deployment secrets, or another local secret source. Never put a real key in `.env.example`, docs, git history, logs, kanban comments, or screenshots.

Restart Hermes after changing `memory.provider` or these environment variables so the profile reloads config and provider tools.

## Local container setup

`docker-compose.yml` defines an optional `openviking` service under the `agent-infra` profile:

```bash
docker compose --profile agent-infra up -d openviking
```

The service is intentionally loopback-bound:

```text
127.0.0.1:1933 -> openviking:1933
```

Persistent OpenViking state is stored in the named Docker volume `pet_resort_openviking` mounted at `/app/.openviking`, matching the upstream container layout for `ov.conf`, `ovcli.conf`, and workspace data.

Copy the example environment if this checkout does not already have local env configuration:

```bash
cp .env.example .env
```

The example file contains only local placeholders:

```dotenv
OPENVIKING_ENDPOINT=http://127.0.0.1:1933
OPENVIKING_API_KEY=***
OPENVIKING_ACCOUNT=nva-local
OPENVIKING_USER=local-developer
OPENVIKING_AGENT=hermes-nva-local
OPENVIKING_WITH_BOT=0
OPENVIKING_PUBLIC_BASE_URL=
OPENVIKING_CONFIG_FILE=/app/.openviking/ov.conf
# OPENVIKING_CONF_CONTENT=  # optional full ov.conf JSON from local secret channel only
```

## Initialize or preflight OpenViking server config

The upstream OpenViking container expects `/app/.openviking/ov.conf` before the full server starts. If the config is absent, the container may expose a pending health response with setup instructions.

This repo provides a deterministic preflight for the optional local agent-infra service:

```bash
scripts/preflight_openviking_agent_infra.sh
```

The preflight validates `docker compose --profile agent-infra config`, starts the `openviking` service, checks that the mounted config file exists, then waits for `http://127.0.0.1:1933/health`. On a fresh uninitialized volume it exits non-zero and prints the exact remediation instead of silently claiming turnkey local agent-infra.

Interactive local initialization:

```bash
docker compose --profile agent-infra up -d openviking
docker compose exec openviking openviking-server init
docker compose restart openviking
scripts/preflight_openviking_agent_infra.sh
```

If `openviking-server init` asks for model/provider/API-key details, use local or operator-approved values and keep secrets outside git. This repo deliberately does not commit provider credentials, `root_api_key`, or a production model policy.

Non-interactive local initialization is available when an operator supplies the full `ov.conf` JSON through a local secret channel. Compose passes this value through to the upstream entrypoint; do not write it into committed files or logs:

```bash
export OPENVIKING_CONF_CONTENT='<full ov.conf JSON from .env, 1Password, or another local secret source>'
docker compose --profile agent-infra up -d openviking
scripts/preflight_openviking_agent_infra.sh
```

## Smoke checks

Run these checks from the repo root unless noted otherwise.

### 1. Compose config is valid

```bash
docker compose --profile agent-infra config >/tmp/nva-compose-agent-infra.yml
```

Expected: exit code 0.

### 2. Container is running and healthy

```bash
docker compose --profile agent-infra up -d openviking
docker compose ps openviking
curl -fsS "$OPENVIKING_ENDPOINT/health"
```

For local dev, `OPENVIKING_ENDPOINT` should normally be `http://127.0.0.1:1933`. Expected: HTTP 2xx health response after initialization. If the health response says setup is pending, run the initialization step above and restart the service.

### 3. Hermes sees OpenViking as the active memory provider

```bash
hermes memory status
```

Expected shape:

```text
Memory status
  Built-in:  always active
  Provider:  openviking
  Plugin:    installed ✓
  Status:    available ✓
```

If the provider is not active, set `memory.provider openviking`, confirm the profile `.env` contains the `OPENVIKING_*` variables, then restart Hermes.

### 4. Provider tool smoke from a Hermes session

Inside a Hermes session with OpenViking tools available, browse the root and run a narrow search:

```text
Use viking_browse with action=list and path=viking://
Use viking_search for "NVA Pet Resorts deterministic app source of truth OpenViking" with mode=fast and limit=5
```

Expected: `viking_browse` returns top-level `viking://` entries such as `user`, `agent`, `resources`, or `session`; `viking_search` returns either relevant memory/resource URIs or an empty result without tool/provider errors.

Equivalent CLI one-shot for a configured profile:

```bash
hermes chat -q 'Use viking_browse action=list path=viking://, then viking_search query="NVA Pet Resorts deterministic app source of truth OpenViking" mode=fast limit=5. Report only whether the provider tools worked.'
```

## Operational guardrails for NVA workflows

Hermes may use OpenViking to retrieve background knowledge before drafting recommendations, but every accepted NVA workflow action still requires app-owned evidence and policy validation:

- current source refs in typed context packets;
- app-owned policy and review-gate validation;
- deterministic draft rejection for missing evidence, unsupported actions, unsafe claims, or wrong gates;
- persisted audit/replay records in the app, not only agent memory;
- explicit human review before customer sends, PMS/provider writes, schedule changes, refunds, discounts, payments, or safety-sensitive decisions.

OpenViking can remember that a workflow pattern was useful. It cannot prove that today's reservation, vaccine, incident, staffing, payment, or customer-message fact is true. Current operational truth must come from deterministic app contracts and source refs.

## Verification performed for this runbook

This runbook was checked against the repo compose shape and Hermes memory-provider docs. The safe local gate for future changes remains:

```bash
./scripts/test.sh
```

For this specific runbook, the relevant targeted checks are:

```bash
docker compose --profile agent-infra config >/tmp/nva-compose-agent-infra.yml
hermes memory status
```

A local `curl http://127.0.0.1:1933/health` is expected to fail until the optional local OpenViking container is started and initialized. That is not an app test failure; it only means local agent-infra is not running.
