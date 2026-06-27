# Coolify shareable NVA demo deployment

This is the deployment runbook for the shareable NVA owned-operations API demo.

## Coolify project

- Project: `NVA`
- Environment: `production`
- Intended public entrypoint: `staff-web`
- Recommended hostname: `https://nva-demo.eman.network` or another demo-specific hostname

## Compose file

Use:

```text
docker-compose.coolify.yml
```

Do not use the local `docker-compose.yml` directly for Coolify. The local file binds host ports to `127.0.0.1` for developer safety. The Coolify file keeps Postgres, MinIO, API, and worker private by default and exposes only the staff web service through Coolify routing.

## Required environment variables

Set these in Coolify as service/application environment variables:

```env
POSTGRES_PASSWORD=<generated-alphanumeric-demo-password>
MINIO_ROOT_USER=<generated-demo-minio-user>
MINIO_ROOT_PASSWORD=<generated-alphanumeric-demo-secret>
```

Keep generated values out of git and chat logs.

The deployed app intentionally sets:

```env
PET_RESORT_AGENT_RUNTIME_MODE=fake
PET_RESORT_SIDE_EFFECT_MODE=stubbed
LIVE_CUSTOMER_MESSAGING=disabled
LIVE_PROVIDER_WRITES=disabled
LIVE_PAYMENT_ACTIONS=disabled
LIVE_SCHEDULE_CHANGES=disabled
DEMO_DATA_LABEL=synthetic-fixture-only
```

## Public/private boundary

Public by default:

- `staff-web` only.

Private/internal by default:

- `pet-resort-api`
- `pet-resort-worker`
- `postgres`
- `minio`

Only expose the API with a second hostname if there is a deliberate demo need. Do not expose Postgres or MinIO publicly.

## Safety claim

This deployment is a synthetic-data demo. It does not use live NVA/Gingr access, production data, provider/PMS writes, customer/member sends, payments, schedule changes, capacity decisions, or medical/safety decisions.

Suggested footer/banner language:

> Synthetic demo data only. Fake agent runtime. Live customer/provider/payment/schedule side effects are disabled.

## Verification after deployment

From the host or any machine with network access:

```sh
curl -fsS https://nva-demo.eman.network/
```

From the Coolify host/container context, verify internal services:

```sh
docker ps --format 'table {{.Names}}\t{{.Status}}' | grep -i 'nva\|pet-resort'
```

Expected app checks:

- staff-web loads successfully;
- staff-web can fetch through `/api/local-demo`;
- API `/healthz` and `/readyz` pass internally;
- demo labels say fake/stubbed/synthetic;
- no live side-effect flags are enabled.
