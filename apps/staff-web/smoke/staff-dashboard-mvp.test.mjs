import { readFileSync } from "node:fs";
import { test } from "node:test";
import assert from "node:assert/strict";

const page = readFileSync(new URL("../app/page.tsx", import.meta.url), "utf8");

const requiredOperationalSurfaces = [
  "Session guard",
  "Today operations",
  "Pet profile",
  "Reservation view",
  "Inquiry intake queue",
  "Parsed lead",
  "Inquiry draft reply",
  "Missing-info task",
  "Booking triage",
  "Hard-rule results",
  "AI recommendation",
  "Staff confirmation controls",
  "Confirmation draft",
  "Task queue",
  "Document review queue",
  "Staff notes",
  "Incident entry",
  "Incident list",
  "Manager Daily Brief",
  "Daily brief action review",
  "Review gates",
  "Blocked action boundaries",
  "Outcome capture",
  "Labor savings evidence",
  "Audit-visible staff actions"
];

test("staff dashboard exposes all MVP operational surfaces", () => {
  for (const surface of requiredOperationalSurfaces) {
    assert.match(page, new RegExp(surface, "i"), `${surface} surface is missing`);
  }
});

test("sensitive actions are visibly draft or review oriented", () => {
  assert.match(page, /no live customer sends/i);
  assert.match(page, /draft/i);
  assert.match(page, /manager review/i);
  assert.match(page, /append-only audit/i);
});

test("booking triage makes confirmation rejection and exception gates explicit", () => {
  assert.match(page, /confirmed booking automation requires staff approval/i);
  assert.match(page, /reject\/decline remains human approval gated/i);
  assert.match(page, /special-care acceptance requires care-team approval/i);
  assert.match(page, /behavior exceptions require behavior review/i);
  assert.match(page, /produce draft confirmation/i);
});

test("incident escalation MVP keeps serious decisions behind human gates", () => {
  for (const expected of [
    "Record incident",
    "classification draft",
    "Generate owner-message draft",
    "Create follow-up task",
    "CustomerMessageApproval",
    "BehaviorReview",
    "ManagerApproval",
    "final classification requires manager approval",
    "Eligibility-impacting flag recommendation only",
    "manager-review queue"
  ]) {
    assert.match(page, new RegExp(expected, "i"), `missing incident MVP evidence: ${expected}`);
  }
});

test("vaccine document review UI shows upload extraction approval eligibility and audit boundaries", () => {
  for (const expected of [
    "Vaccine document MVP",
    "Upload sample vaccine document",
    "vaccine_extraction.v1",
    "medical document uncertainty policy",
    "Approve vaccine record",
    "Reject vaccine record",
    "pet eligibility updates only after approval",
    "document.received",
    "vaccine_record.review_requested",
    "approval.decision.recorded"
  ]) {
    assert.match(page, new RegExp(expected, "i"), `missing vaccine document MVP evidence: ${expected}`);
  }
});

test("manager daily brief UI exposes review outcomes and labor-savings loop", () => {
  for (const expected of [
    "Manager Daily Brief",
    "source evidence summary",
    "Daily brief action review",
    "Approve action",
    "Defer action",
    "Suppress action",
    "Source fact wrong",
    "actual minutes spent",
    "estimated vs actual labor minutes saved",
    "change_staff_schedule",
    "mutate_provider_or_pms_record",
    "send_customer_message",
    "move_refund_discount_or_payment",
    "hide_source_data_quality_issue"
  ]) {
    assert.match(page, new RegExp(expected, "i"), `missing manager daily brief UI evidence: ${expected}`);
  }
});

test("staff dashboard exposes api readiness metrics and repository contract posture", () => {
  for (const expected of [
    "API readiness and observability contract",
    "/readyz",
    "/ops/metrics/summary",
    "runtime_readiness",
    "ops_metrics_summary",
    "api_runtime_dto",
    "active adapter: in_memory",
    "planned adapter: postgres same-contract",
    "live_side_effects: disabled",
    "audit_event_count",
    "review_packet_count",
    "outcome_count",
    "inquiry_count",
    "Prometheus/OpenTelemetry plan"
  ]) {
    assert.match(page, new RegExp(expected, "i"), `missing API readiness/metrics story evidence: ${expected}`);
  }
});
