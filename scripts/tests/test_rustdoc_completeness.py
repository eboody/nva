from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "check_rustdoc_completeness.py"


def load_rustdoc_module():
    spec = importlib.util.spec_from_file_location("check_rustdoc_completeness", SCRIPT)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_booking_triage_smoke_follows_the_current_staff_packet_owner() -> None:
    rustdoc = load_rustdoc_module()
    booking_expectations = [
        expectation
        for expectation in rustdoc.RENDERED_DOC_EXPECTATIONS
        if expectation.relative_path.startswith("app/booking_triage/")
    ]

    assert [expectation.relative_path for expectation in booking_expectations] == [
        "app/booking_triage/struct.StaffEvaluationPacket.html"
    ]
    fragments = booking_expectations[0].required_fragments
    assert "Staff evaluation packet used by the booking-readiness workflow" in fragments
    assert "method.deterministic_result" in fragments
    assert "method.audit_event_drafts" in fragments
