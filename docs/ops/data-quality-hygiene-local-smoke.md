# Data-Quality Hygiene local smoke

This runbook proves the second labor-cost workflow on local fake data only. It exercises the deterministic app-owned contract for:

1. building a source-grounded data-quality hygiene context packet;
2. validating a Hermes-style internal cleanup draft;
3. rejecting an unsafe side-effect request; and
4. recording reviewed outcome evidence with estimated and actual labor reported time difference.

The acceptance lens is NVA Pet Resorts labor-cost reduction across a 170-location portfolio: reduce repeated manager/front-desk source reconciliation without turning the agent into a generic chatbot or a source-system writer.

## Safety boundary

This smoke is fixture-only and has no live/customer/provider side effects.

It does not:

- send customer email, SMS, portal messages, or review requests;
- mutate Gingr, PMS/provider records, customer/pet profiles, reservations, schedules, packages, invoices, payments, refunds, or discounts;
- read production/customer/provider credentials;
- hide, merge, or auto-resolve source ambiguity; or
- authorize eligibility, vaccine, incident, payment, refund, discount, or schedule decisions.

The app-owned workflow allows the agent to summarize source evidence, rank internal hygiene actions, draft an internal cleanup task, preserve ambiguity for review, and estimate reconciliation reported time difference. Human review and deterministic app policy remain the contract boundary.

## Command

From the repository root:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
```

The script runs `cargo run -p app --example data_quality_hygiene_local_smoke --quiet` and then checks the emitted smoke markers. It requires only local Rust tooling plus Python for the marker assertions. It does not start Docker or call network services.

## Expected output

A passing run prints markers like:

```text
context_ok workflow=data-quality-hygiene actions=1 reported_estimated_minutes_difference=15 claimable=false live_side_effects_allowed=false
draft_validation_ok accepted_actions=1 requested_side_effects=0
blocked_draft_validation_ok blocked_side_effect=send_customer_message
outcome_ok reported_estimated_minutes_difference=15 reported_actual_minutes_spent=8 claimable=false live_side_effects_allowed=false
smoke_assertions_ok reported_estimated_minutes_difference=15 reported_actual_minutes_spent=8 claimable=false
```

The exact action ids and local temporary directory can vary. The important proof points are:

- `context_ok` confirms the deterministic app built a source-grounded context/action packet from fake data-quality evidence.
- `draft_validation_ok` confirms a draft that cites the context source refs and requests no side effects is accepted.
- `blocked_draft_validation_ok` confirms app validation rejects a customer-send side effect before any live action exists.
- `outcome_ok` confirms outcome capture retains caller-reported estimate and time-spent evidence while explicitly denying claim authority and live side effects.
- `smoke_assertions_ok` confirms the script parsed positive reported evidence and the required `claimable=false` marker.

## What the fake fixture represents

The example uses a fake stale/missing vaccination source-evidence issue for a single local operating day. That shape is intentionally representative of data-quality hygiene work that saves repeated staff reconciliation time:

- source evidence and provenance stay attached;
- source ambiguity remains visible to a manager/front-desk review path;
- the recommended work is an internal cleanup/review task, not a provider mutation;
- the labor metric compares the pre-agent manual reconciliation estimate to the reviewed actual minutes spent.

## Troubleshooting

- If Rust compilation fails, first run the focused contract test:

  ```sh
  cargo test -p app --test data_quality_hygiene_workflow_contracts -- --nocapture
  ```

- If the script prints `missing required command`, install or activate the missing local tool. No secrets are required.
- If marker assertions fail, inspect the full output file under the temporary directory printed by the script. A failure usually means the example stopped printing a contract marker, stopped reporting the evidence values, or omitted the nonclaimability marker.

## Promotion boundary

Passing this local smoke proves only local/demo readiness for the data-quality hygiene loop. Pilot/live readiness still requires separate evidence for production data access, retention, monitoring, rollback, authorization, durable audit storage, and explicit policy changes for any side effect. Until then, customer sends, provider/PMS writes, schedule changes, payment/refund/discount movement, and hidden ambiguity resolution remain blocked.
