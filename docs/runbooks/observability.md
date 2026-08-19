# Pet Resort API observability runbook

The production telemetry path is API spans over OTLP to the collector, a persistent collector queue, Tempo trace blocks, Prometheus scrapes, and the provisioned Grafana dashboard. The stack is operator-only and binds host ports to loopback. During observability incidents, live side effects remain disabled: no customer messages, provider writes, payments, or equivalent authority is enabled.

`/v1/readyz` reports whether production telemetry has been configured. It does not
probe Collector, Tempo, Prometheus, or Grafana and says so explicitly; the operator
probes below are the operational evidence. A successful API readiness response must
not be interpreted as external observability health.

## Start and verify

```bash
install -m 600 /dev/null /tmp/pet-resort-grafana-admin-password
openssl rand -base64 32 > /tmp/pet-resort-grafana-admin-password
export GRAFANA_ADMIN_USER=pet-resort-observability-admin
export GRAFANA_ADMIN_PASSWORD_FILE=/tmp/pet-resort-grafana-admin-password
docker compose -f ops/observability/compose.yml up -d
curl --fail http://127.0.0.1:3001/metrics
curl --fail http://127.0.0.1:9090/-/ready
curl --fail http://127.0.0.1:3200/ready
```

Keep the password file outside the repository, distribute it through the deployment secret manager in hosted environments, and remove the local temporary file when the stack is destroyed. The Compose contract has no default Grafana administrator credential.

Start the API with production telemetry configured:

```bash
PET_RESORT_TELEMETRY_MODE=production \
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317 \
PET_RESORT_OBSERVABILITY_DASHBOARD_URL=http://127.0.0.1:3302/d/pet-resort-api \
PET_RESORT_OBSERVABILITY_ALERT_POLICY=ops/observability/alerts.yml \
PET_RESORT_API_ADDR=0.0.0.0:3001 \
cargo run -p pet-resort-api
```

The all-interface bind lets the containerized Prometheus scraper reach the host API. Use it only on a host whose firewall/private network policy restricts port 3001; otherwise point Prometheus at a private interface and bind the API there.

Verify `/readyz` reports the configured telemetry components truthfully and send a safe read-only request. Confirm request counters in Prometheus and a correlated trace in Tempo before treating telemetry as operational.

## API unavailable

1. Check `/healthz`, then `/readyz`, then `/metrics`; distinguish process failure from a required dependency failure.
2. Inspect API structured logs by request ID and correlation ID. Do not enable payload logging.
3. Check `docker compose -f ops/observability/compose.yml ps` and collector export failures.
4. Restore the API or its required dependency. Do not bypass authentication, review gates, or side-effect locks.
5. Confirm `up{job="pet-resort-api"} == 1` and close only after two alert evaluation windows.

## Elevated server errors

1. Use the Grafana server-error and latency panels to identify onset and scope.
2. Group safe spans by bounded route and status class. Never add actor IDs, tenant IDs, source payloads, or idempotency keys as metric labels.
3. Correlate representative request IDs with redacted logs and traces.
4. Roll back or repair the failing read/model path. Keep live side effects disabled.
5. Verify the five-minute server-error ratio remains below five percent for two evaluation windows.

## Telemetry backlog or outage

The collector queue at `/var/lib/otelcol` survives collector restarts, and Tempo/Prometheus use persistent volumes. If the trace backend is unavailable, preserve the queue volume and restore Tempo before restarting the collector repeatedly. Readiness must not claim durable tracing is healthy while export is unavailable.

## Data safety

Metrics use bounded method, route, and status-class labels. Spans may carry request ID and correlation ID but not credentials, raw provider payloads, free-form feedback, medical details, actor identifiers, or raw idempotency keys. Grafana is not anonymously accessible.
