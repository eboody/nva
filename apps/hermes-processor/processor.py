#!/usr/bin/env python3
"""Deterministic local Hermes processor bridge for the information-lifespan demo.

The service is intentionally safe-by-default: it consumes only synthetic fixture
input, refuses live side-effect posture, writes schema-shaped JSON artifacts, and
logs an explicit Hermes runtime posture. If HERMES_PROCESSOR_REQUIRE_RUNTIME=true
is set, missing `hermes` CLI becomes a hard, visible failure instead of silently
pretending an LLM ran.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import sys
from pathlib import Path
from typing import Any

DETERMINISTIC_TIMESTAMP = "2026-06-29T15:00:00Z"
FORBIDDEN_PATTERNS = [
    re.compile(r"sk-[A-Za-z0-9_-]{12,}"),
    re.compile(r"(?i)(api[_-]?key|secret|token|password)\s*[:=]\s*[^\s,}]+"),
    re.compile(r"(?i)live_side_effects_allowed\s*[:=]\s*true"),
]
LOCKED_SIDE_EFFECTS = [
    "provider/PMS writes",
    "customer sends",
    "schedule changes",
    "payments/refunds/discounts",
    "medical or vaccine acceptance decisions",
]


class ProcessorError(RuntimeError):
    """Raised when the deterministic processor contract is violated."""


def _json_dump(value: Any) -> str:
    return json.dumps(value, indent=2, sort_keys=True) + "\n"


def _read_json(path: Path) -> dict[str, Any]:
    try:
        with path.open("r", encoding="utf-8") as handle:
            data = json.load(handle)
    except FileNotFoundError as exc:
        raise ProcessorError(f"input_not_found:{path}") from exc
    except json.JSONDecodeError as exc:
        raise ProcessorError(f"input_invalid_json:{exc}") from exc
    if not isinstance(data, dict):
        raise ProcessorError("input_root_must_be_object")
    return data


def _require_bool(data: dict[str, Any], key: str, expected: bool) -> None:
    if data.get(key) is not expected:
        raise ProcessorError(f"{key}_must_be_{str(expected).lower()}")


def _validate_input(data: dict[str, Any]) -> None:
    required = [
        "contract_version",
        "correlation_id",
        "source_system",
        "synthetic_data_only",
        "provider_payloads_are_source_evidence_only",
        "live_side_effects_allowed",
        "source_payloads",
        "db_projection_proof",
        "calculation_inputs",
        "requested_output",
        "safety_gates",
    ]
    missing = [key for key in required if key not in data]
    if missing:
        raise ProcessorError(f"input_missing_required:{','.join(missing)}")
    if data["contract_version"] != "information_lifespan_hermes_processor_input.v1":
        raise ProcessorError("unsupported_contract_version")
    if data["source_system"] != "mock_gingr_readonly_fixture":
        raise ProcessorError("source_system_must_be_mock_gingr_readonly_fixture")
    _require_bool(data, "synthetic_data_only", True)
    _require_bool(data, "provider_payloads_are_source_evidence_only", True)
    _require_bool(data, "live_side_effects_allowed", False)
    payloads = data.get("source_payloads")
    if not isinstance(payloads, list) or len(payloads) != 3:
        raise ProcessorError("source_payloads_must_contain_three_synthetic_records")
    for index, payload in enumerate(payloads):
        if not isinstance(payload, dict):
            raise ProcessorError(f"source_payload_{index}_must_be_object")
        if payload.get("authority") != "provider_evidence_only":
            raise ProcessorError(f"source_payload_{index}_authority_must_be_provider_evidence_only")
    gates = data.get("safety_gates")
    if not isinstance(gates, list) or len(gates) < 5:
        raise ProcessorError("safety_gates_must_lock_all_demo_side_effects")


def _scan_forbidden(value: Any, context: str) -> None:
    text = _json_dump(value) if not isinstance(value, str) else value
    for pattern in FORBIDDEN_PATTERNS:
        if pattern.search(text):
            raise ProcessorError(f"forbidden_secret_or_live_claim_in_{context}")


def _build_output(data: dict[str, Any], hermes_cli_path: str | None, mode: str) -> dict[str, Any]:
    inputs = data["calculation_inputs"]
    before = int(inputs["manual_morning_scan_minutes"])
    after = int(inputs["reviewed_packet_minutes"])
    reported_estimated_minutes_difference = before - after
    if reported_estimated_minutes_difference <= 0:
        raise ProcessorError("reported_estimated_labor_minutes_difference_must_be_positive")

    runtime_status = "hermes_cli_available" if hermes_cli_path else "hermes_cli_unavailable_explicit_fallback"
    simulation_label = None if hermes_cli_path else (
        "No real Hermes CLI/LLM credentials are required for this local demo; "
        "the container runs a deterministic processor bridge and exposes a hard "
        "failure mode via HERMES_PROCESSOR_REQUIRE_RUNTIME=true."
    )

    output = {
        "contract_version": "information_lifespan_hermes_processor_output.v1",
        "correlation_id": data["correlation_id"],
        "synthetic_data_only": True,
        "provider_payloads_are_source_evidence_only": True,
        "live_side_effects_allowed": False,
        "processor": {
            "service": "hermes-processor",
            "mode": mode,
            "hermes_cli_available": bool(hermes_cli_path),
            "hermes_cli_path": hermes_cli_path,
            "runtime_status": runtime_status,
            "simulation_label": simulation_label,
            "processed_at": DETERMINISTIC_TIMESTAMP,
        },
        "lineage": {
            "source_system": data["source_system"],
            "source_payload_refs": [payload["raw_payload_ref"] for payload in data["source_payloads"]],
            "db_projection": data["db_projection_proof"],
            "artifact_ref": data["requested_output"]["artifact_ref"],
        },
        "calculations": {
            "source_snapshots": inputs["source_snapshot_count"],
            "normalized_facts": inputs["normalized_fact_count"],
            "workflow_packets": inputs["workflow_packet_count"],
            "review_gates": inputs["review_gate_count"],
            "reported_estimated_labor_minutes_difference": reported_estimated_minutes_difference,
            "expression": f"{before} minute caller-reported manual baseline - {after} minute caller-reported workflow estimate",
        },
        "review_gates": [
            {"gate": gate, "locked": True, "reason": "demo boundary keeps unsafe side effects review-required"}
            for gate in data["safety_gates"]
        ],
        "final_report": {
            "artifact_kind": data["requested_output"]["artifact_kind"],
            "artifact_ref": data["requested_output"]["artifact_ref"],
            "title": data["requested_output"]["title"],
            "summary": (
                f"{inputs['source_snapshot_count']} source snapshots, "
                f"{inputs['normalized_fact_count']} normalized facts, "
                f"{inputs['workflow_packet_count']} workflow packet, "
                f"{inputs['review_gate_count']} review locks, "
                f"{reported_estimated_minutes_difference} reported estimated labor minute difference"
            ),
            "manager_actions": [
                {
                    "priority": "high",
                    "action": "Review near-expiry rabies vaccine evidence for animal 8101",
                    "source_refs": ["fixture://mock-gingr/vaccines/8101-rabies.json"],
                    "blocked_side_effects": LOCKED_SIDE_EFFECTS,
                },
                {
                    "priority": "medium",
                    "action": "Check dinner appetite after internal feeding note before any customer-facing update",
                    "source_refs": ["fixture://mock-gingr/care-notes/9001001-feeding.json"],
                    "blocked_side_effects": LOCKED_SIDE_EFFECTS,
                },
            ],
        },
        "logs_ref": "processor-log.jsonl",
        "schema_ref": "schemas/information-lifespan-hermes-processor-output.schema.json",
    }
    _scan_forbidden(output, "processor_output")
    return output


def _write_logs(path: Path, data: dict[str, Any], output: dict[str, Any]) -> None:
    events = [
        {
            "timestamp": DETERMINISTIC_TIMESTAMP,
            "level": "INFO",
            "target": "hermes_processor.information_lifespan",
            "correlation_id": data["correlation_id"],
            "event": "accepted_synthetic_trace",
            "synthetic_data_only": True,
        },
        {
            "timestamp": DETERMINISTIC_TIMESTAMP,
            "level": "INFO",
            "target": "hermes_processor.runtime",
            "correlation_id": data["correlation_id"],
            "event": "runtime_posture_checked",
            "runtime_status": output["processor"]["runtime_status"],
            "mode": output["processor"]["mode"],
        },
        {
            "timestamp": DETERMINISTIC_TIMESTAMP,
            "level": "WARN",
            "target": "hermes_processor.safety",
            "correlation_id": data["correlation_id"],
            "event": "unsafe_side_effects_locked",
            "locked_side_effects": LOCKED_SIDE_EFFECTS,
        },
        {
            "timestamp": DETERMINISTIC_TIMESTAMP,
            "level": "INFO",
            "target": "hermes_processor.report",
            "correlation_id": data["correlation_id"],
            "event": "manager_daily_report_enriched",
            "artifact_ref": output["final_report"]["artifact_ref"],
            "reported_estimated_labor_minutes_difference": output["calculations"]["reported_estimated_labor_minutes_difference"],
        },
    ]
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for event in events:
            _scan_forbidden(event, "processor_log")
            handle.write(json.dumps(event, sort_keys=True) + "\n")


def main() -> int:
    input_path = Path(os.environ.get("HERMES_PROCESSOR_INPUT", "/workspace/input/hermes-processor-input.json"))
    output_dir = Path(os.environ.get("HERMES_PROCESSOR_OUTPUT_DIR", "/workspace/output"))
    mode = os.environ.get("HERMES_PROCESSOR_MODE", "deterministic_local_bridge")
    require_runtime = os.environ.get("HERMES_PROCESSOR_REQUIRE_RUNTIME", "false").lower() in {"1", "true", "yes"}
    hermes_cli_path = shutil.which("hermes")

    try:
        if require_runtime and not hermes_cli_path:
            raise ProcessorError("hermes_runtime_unavailable:set_HERMES_PROCESSOR_REQUIRE_RUNTIME_false_for_deterministic_local_bridge")
        data = _read_json(input_path)
        _scan_forbidden(data, "processor_input")
        _validate_input(data)
        output = _build_output(data, hermes_cli_path, mode)
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / "processor-output.json"
        log_path = output_dir / "processor-log.jsonl"
        output_path.write_text(_json_dump(output), encoding="utf-8")
        _write_logs(log_path, data, output)
        print(_json_dump({
            "status": "ok",
            "correlation_id": data["correlation_id"],
            "output": str(output_path),
            "logs": str(log_path),
            "runtime_status": output["processor"]["runtime_status"],
            "reported_estimated_labor_minutes_difference": output["calculations"]["reported_estimated_labor_minutes_difference"],
        }), end="")
        return 0
    except ProcessorError as exc:
        print(_json_dump({"status": "error", "error": str(exc), "service": "hermes-processor"}), file=sys.stderr, end="")
        return 78


if __name__ == "__main__":
    raise SystemExit(main())
