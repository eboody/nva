# Information lifespan demo presenter walkthrough

Status: 3-5 minute presenter run sheet for the local Docker Compose information-lifespan demo. Use it for an NVA/job-contact/CEO/CTO conversation when the goal is to show the machine working without burying the viewer in implementation detail.

Safety framing to say out loud: this is a local synthetic demo. The Gingr/source evidence is mocked read-only fixture input. The stack does not require or use live NVA/Gingr credentials, does not send customer messages, does not write to a PMS/provider, does not move money, does not change schedules/capacity, and does not make medical or safety decisions. Unsafe actions stay locked or review-required.

## One-command run path

From the repo root:

```sh
./scripts/demo_information_lifespan.sh
```

Primary references:

- Board artifact: [Information Lifespan + Hermes Processor Demo Board](../kanban/2026-06-29-information-lifespan-hermes-demo-board.md).
- Operator runbook: [Local Docker Compose demo runbook](../ops/local-demo-compose.md).
- UI entrypoint: `staff-web` at `http://127.0.0.1:${PET_RESORT_STAFF_WEB_HOST_PORT:-3000}` after the script starts Compose.
- API proof endpoints: `POST /v0/demo/information-lifespan/run` and `GET /v0/demo/information-lifespan/{correlation_id}/report`.

The script writes proof artifacts under `.artifacts/information-lifespan/` and `.var/information-lifespan/`. Treat those files as local demo evidence, not production logs.

## 30-second setup before the call

1. Run the one-command script or keep its successful transcript open.
2. Open the browser to the staff-web page.
3. Keep a terminal tab at the repo root with these local paths ready:
   - `.artifacts/information-lifespan/demo-information-lifespan-transcript.log`
   - `.artifacts/information-lifespan/api-information-lifespan-run.json`
   - `.artifacts/information-lifespan/api-information-lifespan-report.json`
   - `.var/information-lifespan/processor-output.json`
   - `.var/information-lifespan/processor-log.jsonl`
4. If default ports are occupied, use the documented override path:

```sh
PET_RESORT_API_HOST_PORT=33101 \
PET_RESORT_STAFF_WEB_HOST_PORT=33100 \
PET_RESORT_POSTGRES_HOST_PORT=35532 \
PET_RESORT_MINIO_HOST_PORT=39020 \
PET_RESORT_MINIO_CONSOLE_HOST_PORT=39021 \
  ./scripts/demo_information_lifespan.sh
```

## 3-5 minute talk track

### 0:00-0:30 — frame the thesis

Suggested line:

> The point of this demo is the lifespan of one piece of resort information. It begins as mocked, read-only Gingr/source evidence. Then it becomes NVA-owned typed facts, database/projection proof, a Dockerized Hermes processing step, browser-visible API calls, and finally a Manager Daily Report with lineage, calculations, review locks, and explicitly nonclaimable reported labor evidence.

Point at the page header and the interactive `Run information lifespan` stage machine. Do not start with an architecture diagram. Start with the product question: can a manager trust where the report came from?

### 0:30-1:10 — source starts here

Click or point to the first source cards:

- `Mock Gingr event received`
- `Provider DTO / source model shown`
- source badges such as `gingr::response::ReservationRecord`, `app::information_lifespan::SourcePayload`, and `domain::*` target paths

Suggested line:

> This is the provider boundary. Gingr is not being treated as automatic product truth here. The page labels it as mocked read-only source evidence, keeps source refs visible, and shows the provider/source model before any NVA-owned workflow meaning is attached.

Useful inspect paths if asked:

- `fixtures/information-lifespan/hermes-processor-input.json`
- `integrations/gingr/src/dto/README.md`
- `integrations/gingr/src/response.rs`
- `integrations/gingr/src/mapping/`

Do not say: connected to live Gingr, production data, live customer records, or a write-capable integration.

### 1:10-1:55 — transformations happen here

Point to the middle stage cards and proof drawers:

- `Source snapshot + provenance stored`
- `NVA-owned models normalized`
- `Database rows/projections written/read`

Suggested line:

> The important turn is that provider evidence is promoted into NVA-owned contracts only after preserving provenance. The database proof is not a screenshot-only claim: local migrations and seeds create source import runs, workflow events, source-quality issues, review packets, audit events, and a proof view keyed by the same correlation id.

Useful inspect paths if asked:

- `app/src/information_lifespan.rs`
- `app/src/manager_daily_brief.rs`
- `migrations/0001_mvp_foundation.sql`
- `migrations/0002_data_quality_read_models.sql`
- `fixtures/seed/local-demo.sql`
- `fixtures/seed/local-demo-data-quality.sql`

Presenter label to use: actual local Postgres/projection proof where the stack is running; deterministic trace proof where the page is showing compact redacted row previews.

### 1:55-2:35 — Hermes processes here

Point to the Hermes processor panel and, if the stack is running, click `Run information lifespan`.

Suggested line:

> This is the Dockerized Hermes bridge. It consumes the deterministic synthetic trace, enriches the manager report artifact, writes structured JSON and JSONL logs, and labels fallback honestly if the processor artifact is unavailable. The demo is allowed to process local fixture evidence; it is not allowed to touch customers, providers, schedules, money, or medical decisions.

Success anchors from the script:

```text
processor_contract_ok correlation_id=info-lifespan-demo-2026-06-29 ... reported_estimated_labor_minutes_difference=42
api_run_ok correlation_id=info-lifespan-demo-2026-06-29
api_report_replay_ok final_report=artifact://manager-daily-report/synthetic-2026-06-29
```

Useful inspect paths:

- `apps/hermes-processor/processor.py`
- `.var/information-lifespan/processor-output.json`
- `.var/information-lifespan/processor-log.jsonl`

### 2:35-3:20 — DB/log/network proof appears here

Point to the network-visible run card:

- browser route: `/api/local-demo/v0/demo/information-lifespan/run`
- upstream route: `/v0/demo/information-lifespan/run`
- replay route: `/v0/demo/information-lifespan/{correlation_id}/report`
- response preview fields: `correlation_id`, `artifact_ref`, `stage_count`, `reported_estimated_labor_minutes_difference`, `review_gate_count`, `live_side_effects_allowed=false`

Suggested line:

> The browser is not just rendering static copy. A local browser route proxies to the local API, the API returns the trace/report payload, and the report can be replayed by correlation id. The proof drawer keeps model paths, DB rows, logs, and network responses tied to the same run.

If asked for files:

- `apps/api/src/http.rs`
- `apps/staff-web/app/api/local-demo/[...path]/route.ts`
- `.artifacts/information-lifespan/api-information-lifespan-run.json`
- `.artifacts/information-lifespan/api-information-lifespan-report.json`

### 3:20-4:10 — final Manager Daily Report appears here

Point to the final report card:

- `Manager Daily Report — synthetic 2026-06-29`
- `artifact://manager-daily-report/synthetic-2026-06-29`
- value proof: `3 sources`, `3 facts`, `6 DB refs`, `5 locks`, `42 reported estimated minute difference`
- ranked actions, source lineage, calculations, review requirements, and locked side effects

Suggested line:

> The output is not 'AI says something.' It is a manager packet with visible source lineage, calculations, review gates, and reported labor evidence that does not prove realized value. The agent/Hermes step can rank and summarize; it cannot send, write back, change schedules, move money, or decide medical/safety status.

Close with the safe next ask:

> The production next step is not live writes. It is read-only validation: approved docs, sample exports, BI query definitions, and one or two pilot workflows so these source/ref/model/read-model contracts can be compared against real field shape.

## What to click or show in order

1. Page header: establish read-only source evidence, owned backend, write locks.
2. `Run information lifespan`: show stage cards lighting up.
3. First source event card: show mocked Gingr source labels and payload preview.
4. DB/projection panel: show row timeline and compact redacted row JSON.
5. Hermes processor panel: show running/done/unavailable states and structured log output.
6. Network run card: show POST/GET response previews and correlation id.
7. Manager Daily Report: show value proof, ranked actions, calculations, review requirements, locked side effects.
8. Executive close: show safe pilot ask and what is explicitly not being requested.

## Screenshot and static fallback packet

If the live local stack fails during a conversation, do not imply a stale artifact is fresh live proof. Say exactly what failed and switch to this fallback path:

1. Show the board artifact and say the target machine is the same sequence: source -> model -> DB/projection -> Hermes processor -> API/network -> report.
2. Show the last successful transcript if available:
   - `.artifacts/information-lifespan/demo-information-lifespan-transcript.log`
3. Show the saved API/report artifacts if available:
   - `.artifacts/information-lifespan/api-information-lifespan-run.json`
   - `.artifacts/information-lifespan/api-information-lifespan-report.json`
4. Show the saved processor output/logs if available:
   - `.var/information-lifespan/processor-output.json`
   - `.var/information-lifespan/processor-log.jsonl`
5. If browser screenshots were captured by a prior QA run, use them only as labeled screenshots. Recommended local screenshot path is outside git:
   - `/tmp/nva-information-lifespan-demo.png`
6. If no screenshot exists, use the browser page itself as a static visual and say: `This view is static/fallback until I rerun the local Compose stack; it still names the code, DB, API, and artifact paths that prove the concept.`

Optional screenshot command once the stack is up and a browser-capable environment is available:

```sh
# Example only; use any local browser/screenshot tool available on the presenter machine.
# Save outside git so screenshots do not become stale repo evidence.
# /tmp/nva-information-lifespan-demo.png
```

Fallback wording:

> The live stack is local-only and safe. If Docker or a port is slow, I switch to captured local artifacts rather than pretending the system is live. The concept proof is still visible: source evidence, typed transformations, DB/projection proof, Dockerized Hermes output, API/network payloads, final Manager Daily Report, and locked side effects.

## Claims to avoid

Do not claim:

- live NVA/Gingr credentials or production data;
- live customer sends;
- provider/PMS writes;
- payment, refund, discount, or schedule/capacity changes;
- automated medical, vaccine, incident, behavior, or safety decisions;
- production deployment readiness or SSO/location authorization;
- that the fallback screenshot is a fresh live run.

Safe claims:

- synthetic/mock Gingr source evidence enters a local trace;
- NVA-owned Rust/app contracts and local DB projections preserve lineage and review posture;
- a Dockerized local Hermes processor produces deterministic report proof or labels fallback;
- browser-visible local API calls return/replay the report by correlation id;
- side-effect locks and review requirements remain visible;
- the demo is ready for read-only validation discussion, not live operations.

## Rehearsal checklist

Run before presenting:

```sh
./scripts/demo_information_lifespan.sh
./scripts/check_docs.sh
python scripts/check_markdown_links.py --repo-root .
```

During rehearsal, verify you can point to:

- the source/model proof path;
- at least one DB row/projection proof card;
- the Hermes processor output/log path;
- the POST and GET network proof card;
- the Manager Daily Report value proof;
- the locked side-effect/review-gate language;
- the fallback artifact list.

## Production work intentionally deferred

This walkthrough deliberately does not claim production status. Deferred work includes live NVA/Gingr read access, production auth/SSO and location scoping, production deployment, monitoring/retention/redaction policies, durable worker leasing/dead-letter views, approved BI/KPI definitions, security review, and explicit owner approval for any live send/write/payment/schedule/medical/safety side effect.
