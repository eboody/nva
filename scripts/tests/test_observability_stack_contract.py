import json
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
OBS = REPO_ROOT / "ops" / "observability"


def test_observability_stack_has_durable_trace_metrics_dashboard_and_alert_owners():
    compose = yaml.safe_load((OBS / "compose.yml").read_text())
    services = compose["services"]
    assert {"otel-collector", "tempo", "prometheus", "grafana"} <= set(services)
    assert "tempo-data" in compose["volumes"]
    assert "prometheus-data" in compose["volumes"]
    assert "grafana-data" in compose["volumes"]
    assert compose["secrets"]["grafana_admin_password"]["file"].startswith(
        "${GRAFANA_ADMIN_PASSWORD_FILE:?"
    )
    grafana = services["grafana"]
    assert grafana["environment"]["GF_AUTH_ANONYMOUS_ENABLED"] == "false"
    assert grafana["environment"]["GF_SECURITY_ADMIN_PASSWORD__FILE"].startswith(
        "/run/secrets/"
    )
    assert "grafana_admin_password" in grafana["secrets"]

    collector = yaml.safe_load((OBS / "otel-collector.yml").read_text())
    traces = collector["service"]["pipelines"]["traces"]
    assert "otlp" in traces["receivers"]
    assert "batch" in traces["processors"]
    assert "otlp/tempo" in traces["exporters"]

    prometheus = yaml.safe_load((OBS / "prometheus.yml").read_text())
    assert "alerts.yml" in prometheus["rule_files"]
    assert any(job["job_name"] == "pet-resort-api" for job in prometheus["scrape_configs"])

    alerts = yaml.safe_load((OBS / "alerts.yml").read_text())
    alert_names = {
        rule["alert"]
        for group in alerts["groups"]
        for rule in group["rules"]
    }
    assert {"PetResortApiUnavailable", "PetResortApiElevatedServerErrors"} <= alert_names

    runbook = (REPO_ROOT / "docs" / "runbooks" / "observability.md").read_text()
    assert "## API unavailable" in runbook
    assert "## Elevated server errors" in runbook
    assert "live side effects remain disabled" in runbook

    dashboard = json.loads((OBS / "grafana" / "dashboards" / "pet-resort-api.json").read_text())
    assert dashboard["uid"] == "pet-resort-api"
    assert {panel["title"] for panel in dashboard["panels"]} >= {
        "Request rate",
        "Server error rate",
        "Request latency",
    }


def test_observability_stack_keeps_operator_ports_loopback_only():
    compose = yaml.safe_load((OBS / "compose.yml").read_text())
    for service in compose["services"].values():
        for port in service.get("ports", []):
            assert str(port).startswith("127.0.0.1:"), port
