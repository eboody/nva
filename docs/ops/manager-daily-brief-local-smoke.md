# Manager Daily Brief local smoke

This smoke proves two separate properties without conflating evidence with authority:

1. The production API fails closed because no authenticated trusted-actor issuer is currently wired into the local production router.
2. The deterministic app-owned Manager Daily Brief workflow contracts still pass independently of production authority issuance.

Serializable actor labels, review history, outcome rows, bearer-shaped strings, and local fixture metadata are not accepted as authority.

## Command

From the repository root:

```bash
./scripts/smoke_manager_daily_brief_local_loop.sh
```

The script starts local PostgreSQL, MinIO, API, and worker containers. It waits for `/healthz`, then calls the protected Manager Daily Brief context bridge without a token or trusted actor issuer. Success requires a secret-safe HTTP 401 response and no context payload. It then runs:

```bash
cargo test -p app --test manager_daily_brief_workflow_contracts --locked --quiet
```

This separation is deliberate. Application policy can be exercised with fixture evidence, while the production route remains unusable until an authenticated root of trust can issue actor authority.

## Expected proof

A passing run includes:

```text
authority_boundary_ok http_401=true trusted_actor_issuer_available=false live_side_effects_allowed=false
production API stayed fail closed; deterministic workflow contracts passed without live side effects
```

The script must fail if:

- the protected route returns Manager Daily Brief context without authenticated authority;
- the unauthorized request returns a payload;
- the error is not a secret-safe HTTP 401;
- deterministic workflow contracts fail; or
- any local service needed for the smoke cannot start.

## Reported labor evidence

App/API outcome contracts retain `reported_labor_evidence.reported_estimated_minutes_difference` and `reported_labor_evidence.reported_actual_minutes_spent` as nonclaimable evidence. Serializable outcomes cannot establish realized savings, workflow completion, or execution authority. No production value-claim issuer exists.

## Side-effect posture

The local stack keeps customer messaging, provider writes, payments, refunds, discounts, and schedule changes disabled. Passing this smoke does not establish pilot or production readiness. Production usability requires a separately reviewed authenticated actor issuer bound to exact subject, action, scope, gate, and provenance.

## Custom ports

If local ports are occupied, set the compose variables before running:

```bash
export PET_RESORT_POSTGRES_HOST_PORT=55441
export PET_RESORT_MINIO_HOST_PORT=19000
export PET_RESORT_MINIO_CONSOLE_HOST_PORT=19001
export PET_RESORT_API_HOST_PORT=13001
./scripts/smoke_manager_daily_brief_local_loop.sh
```

The script does not stop containers automatically. Operators running a disposable smoke can clean up with the same compose project and environment using `docker compose down -v`.
