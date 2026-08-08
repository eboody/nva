# Information-lifespan Hermes processor

This directory contains the Piece 3 local demo processor for the Information Lifespan board. It is a Dockerized, deterministic bridge that accepts the synthetic mock Gingr trace input and emits a Manager Daily Report-ready enriched trace.

Safety posture:

- Input is `fixtures/information-lifespan/hermes-processor-input.json` and is synthetic/local only.
- Output is written to `.var/information-lifespan/processor-output.json` with companion structured logs in `.var/information-lifespan/processor-log.jsonl`.
- No real Hermes credentials, NVA/Gingr access, customer sends, provider/PMS writes, payments/refunds/discounts, schedule changes, or medical/safety decisions are required or attempted.
- If a future environment needs to prove that a real `hermes` CLI is installed in the image, set `HERMES_PROCESSOR_REQUIRE_RUNTIME=true`; the current local bridge exits with an explicit `hermes_runtime_unavailable` failure when the CLI is absent instead of pretending an LLM ran.

Run from the repository root:

```sh
docker compose build hermes-processor
docker compose run --rm hermes-processor
./scripts/demo_information_lifespan.sh
```

The schema contract lives at `schemas/information-lifespan-hermes-processor-output.schema.json`.
