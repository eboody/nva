# Owned Operations Platform Demo v2 Acceptance Contract

## Purpose

This document is the product spec and regression target for rebuilding the public demo at `https://nva-demo.eman.network/`. Later workers should treat it as canonical over the existing single-dashboard page and over earlier text-heavy architecture/presentation framing.

## One-sentence thesis

Incumbent systems are source evidence; NVA owns operating workflow authority and can replace dependence piece by piece.

## Audience hierarchy

1. CEO first: understand risk, labor value, safety posture, and the pilot ask within 30 seconds.
2. Operator second: see reviewable work, source lineage, and what a manager can safely do today.
3. CTO / technical reviewer third: inspect proof mode for API contracts, provenance, storage/read-model posture, audit/outbox posture, and testable safety boundaries.

## Explicit rejection of the old framing

The new demo must not be a single Daily Manager Report dashboard, a generic executive dashboard, a presenter page, or an architecture essay. The first screen must show an owned operations platform in use: source evidence flows into NVA-owned facts, multiple labor-cost tools reuse the same backend spine, unsafe side effects remain locked, and the close asks for a narrow read-only pilot.

The previous page can be mined for useful concepts such as sample workspace, evidence chains, manager review, and locked actions. It should not be preserved as the user experience. The old text-heavy migration/architecture story belongs behind proof mode, not in the primary surface.

## First-screen storyboard

The first screen should read as one product cockpit, not as separate documentation panels. It needs these visible lanes in this order or in a visual arrangement that preserves this narrative:

### 1. Portfolio risk/value strip

Purpose: CEO-level reason to care before any architecture appears.

Required visible labels / copy candidates:

- `Sample portfolio operating risk`
- `$25.1k modeled monthly labor + rework exposure`
- `48 manager minutes shifted from source chasing to review`
- `4 reusable tools on one owned backend`
- `0 live side effects: sends, PMS writes, schedule changes, payments, medical decisions`

Rules:

- Use business language first: labor, rework, review, risk, portfolio, pilot.
- Metrics may be sample/modelled, but must be labeled as sample/modelled.
- Do not imply measured production NVA performance.

### 2. Read-only source evidence lane

Purpose: make incumbent systems visible as evidence without granting them workflow authority.

Required visible labels / copy candidates:

- `Read-only source evidence`
- `PMS reservation feed sample`
- `Labor schedule / timeclock export sample`
- `Uploaded document sample`
- `Room inventory projection sample`
- `BI query inventory`
- `source refs preserved`
- `freshness + caveats visible`

Required behavior:

- Source cards expose raw signal, source name, observed timestamp/freshness, caveat, and source reference/provider ID where useful.
- Every source item is explicitly read-only.
- The page must not say or imply connected live Gingr/NVA credentials.

### 3. Owned backend spine lane

Purpose: show the product-owned seam between incumbent evidence and NVA workflow tools.

Required visible labels / copy candidates:

- `NVA-owned operating facts`
- `Workflow packets`
- `Review gates`
- `Audit + outcome events`
- `Read models for BI`
- `source evidence -> owned facts -> reviewable work -> outcomes`

Business-first visible copy:

- `NVA keeps the work rules, review decisions, labor outcomes, and reporting meaning in its own operating layer.`

Proof-mode labels may include:

- `Operations API contract`
- `provenance_snapshot_id`
- `workflow_packet_id`
- `review_gate_id`
- `outbox_candidate_id`
- `read_model_projection`
- `audit_event_id`

Rules:

- Architecture terms are allowed as concise labels or in proof mode; they must not dominate the CEO surface.
- This lane must make clear that owned authority does not mean autonomous side effects.

### 4. Tool portfolio lane

Purpose: prove this is a platform that supports multiple labor-saving tools, not one dashboard.

Required visible labels / copy candidates:

- `Tool portfolio on the same backend`
- `Manager Daily Brief`
- `Data Quality Hygiene`
- `Intake / Booking Triage`
- `BI / Read Model Reporting`

Each tool card must include:

- source signals;
- normalized NVA facts;
- review gates;
- output/action;
- outcome/labor metric;
- proof artifact.

Each tool must be selectable or inspectable. Selection should update lineage/proof details without navigating away from the first-screen story.

### 5. Locked side effects / review gates lane

Purpose: make safety a product behavior, not a disclaimer.

Required visible labels / copy candidates:

- `Write locked`
- `Manager review open`
- `Outbox candidate only`
- `Customer send locked`
- `PMS/provider write locked`
- `Schedule change locked`
- `Payment/refund/discount locked`
- `Medical/safety decision locked`
- `Staffing mandate locked`

Required behavior:

- Unsafe actions are visually disabled/locked.
- Review is open for internal manager decisions and evidence validation.
- If an outbox concept is shown, it is an outbox candidate only, not a live send/write.

### 6. Pilot ask close

Purpose: end with a safe next step rather than a production/replacement claim.

Required visible labels / copy candidates:

- `Safe pilot ask`
- `One pilot slice`
- `Read-only exports`
- `Field dictionaries`
- `BI query inventory`
- `Source snapshots or sample rows`
- `Dual-run against current workflow before any write path`

Close copy candidate:

`Pilot ask: approve read-only exports, field dictionaries, and BI query inventory for one resort/workflow slice so NVA can validate source mappings, compare read models, and keep all sends/writes locked until an owner-approved gate exists.`

## Tool portfolio definitions

### Manager Daily Brief

- Source signals: reservation arrivals/departures, lodging/service add-ons, labor schedule/timeclock coverage, room/capacity projection, open document/care-note flags.
- Normalized NVA facts: arrival density, coverage gap, role pressure, revenue service concentration, care constraint, document blocker, capacity caveat, labor/rework exposure.
- Review gates: manager chooses staffing response; no autonomous schedule change; care or room trade-offs require manager approval; customer communication remains locked.
- Output/action: ranked morning brief with top labor/rework risks, suggested internal prep actions, evidence links, and review owners.
- Outcome/labor metric: manager prep minutes shifted from source chasing to review; modeled avoidable labor/rework dollars; reviewed disposition captured after shift.
- Proof artifact: workflow packet with source refs, ranked recommendations, review decisions, outcome event, and read-model row for labor/rework reporting.

Visible card copy candidate:

`Manager Daily Brief: turns messy morning source evidence into a reviewed action list before labor waste starts.`

Proof-mode copy candidate:

`workflow_packet: manager_daily_brief; inputs: reservation_snapshot, labor_schedule_snapshot, capacity_projection; blocked_actions: customer_send, schedule_write, PMS_write.`

### Data Quality Hygiene

- Source signals: unreadable vaccine dates, duplicate/missing pet or owner fields, unsupported source values, stale profile fields, source/export mismatches, BI cleanup exceptions.
- Normalized NVA facts: source quality issue, affected workflow, blocker reason, confidence/caveat, reviewed cleanup disposition, source-field mapping gap.
- Review gates: human validates ambiguous documents/fields; no provider repair/write; no customer message; no destructive merge/delete; owner-approved mapping policy required before automation.
- Output/action: internal cleanup queue with reason, affected reservation/profile, source refs, suggested next review step, and blocked-action explanation.
- Outcome/labor metric: cleanup minutes saved, repeated front-desk rework avoided, source-quality backlog aging, reviewed-resolution rate.
- Proof artifact: local Data-Quality Hygiene workflow proof, draft validation, blocked draft validation, disabled worker/outbox proof, reviewed outcome event, BI/read-model projection.

Visible card copy candidate:

`Data Quality Hygiene: turns source ambiguity into reviewable cleanup work instead of front-desk surprises.`

Proof-mode copy candidate:

`draft_validation_ok; blocked_draft_validation_ok; live_side_effects_allowed=false; outcome estimated_minutes_saved and actual_minutes_saved.`

### Intake / Booking Triage

- Source signals: inbound booking request sample, pet profile notes, vaccination/document status, room/capacity projection, enrichment/service availability, care constraints, historical stay caveats.
- Normalized NVA facts: intake readiness, missing requirement, capacity fit, care constraint, revenue opportunity, review reason, triage priority.
- Review gates: manager/front-desk review before any customer response; no autonomous booking confirmation; no PMS/provider write; no payment/discount action; medical/safety concerns escalated to human policy.
- Output/action: triage queue item with recommended internal next step, missing-info checklist, evidence links, and candidate customer response kept locked.
- Outcome/labor metric: minutes saved per intake review, avoidable back-and-forth reduced, conversion/risk opportunity tracked as modelled/sample until validated.
- Proof artifact: triage workflow packet, source lineage, locked outbox candidate, review disposition, intake read-model row.

Visible card copy candidate:

`Intake / Booking Triage: prepares the front desk with evidence and missing-info checks; it does not book or message for them.`

Proof-mode copy candidate:

`outbox_candidate_only=true; customer_send_locked=true; provider_booking_write_locked=true; review_gate=front_desk_or_manager.`

### BI / Read Model Reporting

- Source signals: current BI query inventory, recurring spreadsheet/report questions, workflow outcome events, review dispositions, source-quality backlog, labor/rework metrics, audit/outbox posture.
- Normalized NVA facts: reviewed business meaning, labor minutes saved, source-quality issue class, workflow aging, disposition, caveat, projection freshness, KPI definition owner.
- Review gates: KPI definitions require owner approval; no raw provider-table meaning is silently reinterpreted; caveats remain visible; production reporting claims wait for read-only validation.
- Output/action: portfolio reporting/read-model views over reviewed operating meaning: labor value, cleanup backlog, review throughput, source caveats, outbox posture, pilot comparison.
- Outcome/labor metric: analyst/reporting cleanup time reduced, clearer KPI definitions, fewer ad hoc spreadsheet reconciliations, pilot read-model comparison against current BI questions.
- Proof artifact: read-model projection, API/OpenAPI proof, audit/outcome events, BI query mapping table, freshness/caveat metadata.

Visible card copy candidate:

`BI / Read Model Reporting: gives portfolio reporting NVA-owned meaning instead of reverse-engineering provider tables.`

Proof-mode copy candidate:

`read_model_projection includes workflow_outcome, source_quality_backlog, labor_minutes_saved, review_queue_aging, outbox_posture, caveats.`

## Copy budget and language rules

Primary surface:

- Lead with business-visible text: labor leakage, manager time, review, source quality, pilot, portfolio risk, safety locks.
- Keep paragraphs short; prefer compact cards, labels, and one-line explanations.
- Architecture is visible through lineage and proof, not through long explanations.
- The CEO should be able to ignore proof mode and still understand value and safety.

Proof mode:

- May expose concise technical terms: OpenAPI, operations API, source adapter, provenance, workflow packet, review gate, audit event, outbox candidate, read model, projection, fixture/local proof.
- Must tie every technical term to a business purpose or artifact.
- Must remain honest that current proof is sample/local/read-only unless later workers verify otherwise.

Forbidden on visible primary surface:

- Architecture-first headings that make the page feel like a slide deck rather than software.
- Giant disclaimers or apology banners.
- Presenter/talk-track/meta-demo copy.
- Claims that NVA/Gingr live systems are connected.

## No-access honesty rules

The page should treat lack of live access as a deliberate safe operating posture:

- Say `sample workspace`, not `fake data`.
- Say `read-only source evidence`, not `we could not access the real system`.
- Say `write locked`, `manager review open`, and `outbox candidate only` as state labels on the product surface.
- Say `read-only pilot ask` as the next step.
- Keep caveats close to the data they qualify: source freshness, sample/modelled metrics, projection caveats, locked side effects.
- Never hide that the demo is sample/local/read-only.
- Never make no-access look apologetic; present it as the correct first safety boundary before validation.

Approved no-access copy candidates:

- `Sample workspace: read-only sources, owned workflow facts, writes locked.`
- `This pilot starts by validating source shape and reporting meaning before any live action path exists.`
- `Source systems remain evidence while NVA owns review, outcomes, and reporting meaning.`

## Safety boundaries

The demo must not enable, imply, or visually suggest any of these live side effects:

- customer/member sends;
- PMS/provider writes;
- schedule/capacity changes;
- payment/refund/discount changes;
- medical/safety decisions;
- staffing mandates;
- destructive merge/delete behavior;
- production deployment or production data claims;
- full Gingr replacement claims before workflow-by-workflow validation.

Safe actions the UI may show:

- read source evidence;
- preserve provenance/source refs;
- normalize sample evidence into NVA-owned facts;
- create reviewable internal workflow packets;
- show manager review as open;
- create outbox candidates that remain locked;
- record reviewed outcomes in sample/local proof;
- project read models for BI proof;
- ask for read-only exports/field dictionaries/BI query inventory.

## Proof mode requirements

Proof mode is for the CTO/technical reviewer and skeptical operator. It should be expandable, selectable, or otherwise secondary to the executive surface.

Required proof artifacts / labels:

- source evidence item with source name, raw signal, source ref, timestamp/freshness, and caveat;
- owned fact with normalized field names and source lineage;
- workflow packet ID or equivalent stable reference;
- review gate and blocked-action policy;
- outbox candidate only, when candidate action is shown;
- audit/outcome event;
- read-model projection;
- local/API proof references such as checked OpenAPI artifact or local demo wrapper, if surfaced;
- `live_side_effects_allowed=false` or equivalent concise safety proof.

Proof mode should not become the default view. It should answer: `Why should a technical reviewer believe this is more than a mockup?`

## Regression contract

Later code workers must translate this section directly into smoke tests, UI assertions, and data-structure checks.

### Required phrases or equivalent visible labels

The rebuilt public page must include these exact phrases somewhere visible or in accessible text unless a later reviewer approves an equivalent:

- `sample workspace`
- `read-only source evidence`
- `NVA-owned operating facts`
- `workflow packets`
- `review gates`
- `Tool portfolio on the same backend`
- `Manager Daily Brief`
- `Data Quality Hygiene`
- `Intake / Booking Triage`
- `BI / Read Model Reporting`
- `write locked`
- `manager review open`
- `outbox candidate only`
- `customer send locked`
- `PMS/provider write locked`
- `schedule change locked`
- `payment/refund/discount locked`
- `medical/safety decision locked`
- `Safe pilot ask`
- `read-only exports`
- `field dictionaries`
- `BI query inventory`

### Forbidden phrases / stale framing

The rebuilt public page must reject these visible phrases:

- `DEMO MODE`
- `presenter`
- `talk track`
- `proper demo page`
- `Comprehensive Summary`
- `Migration Strategy`
- `Daily Manager Report` as the page title or dominant product frame
- `CEO cost reduction daily report demo` as the main aria/page frame
- `What makes it real` as the closing strip title
- claims of live NVA access, live Gingr access, production credentials, production data, enabled customer sends, enabled PMS/provider writes, enabled schedule changes, enabled payments/refunds/discounts, autonomous medical/safety decisions, or staffing mandates

The phrase `Daily Manager Brief` or `Manager Daily Brief` is allowed as one tool in the portfolio. It must not define the entire demo.

### Required interaction / data structures

The frontend data model must include explicit structures for at least:

- source evidence items;
- owned facts / normalized NVA facts;
- tool portfolio cards;
- review gates / blocked actions;
- output/action candidates;
- outcome/labor metrics;
- proof artifacts;
- pilot ask items.

Each tool card must map to:

- `sourceSignals`;
- `normalizedFacts`;
- `reviewGates`;
- `outputAction`;
- `outcomeMetric`;
- `proofArtifact`.

Interaction requirements:

- Selecting or inspecting each of the four tool cards must reveal its lineage from source signals through owned facts, review gates, output/action, outcome/labor metric, and proof artifact.
- Locked side effects must remain visible regardless of selected tool.
- Proof mode must be available without replacing the CEO-first view.
- The pilot ask must remain visible or reachable from the first screen.

### Safety lock assertions

Tests should assert that the page shows locks for:

- customer sends;
- PMS/provider writes;
- schedule changes;
- payments/refunds/discounts;
- medical/safety decisions;
- staffing mandates.

Tests should also assert that any outbox wording includes `candidate` or equivalent non-live wording.

### Proof-mode assertions

Tests should assert that proof mode includes:

- source lineage / provenance;
- review gates;
- blocked-action policy;
- audit/outcome events;
- read-model/projection proof;
- `live_side_effects_allowed=false` or equivalent.

### Copy-density assertions

Tests or review should reject a primary surface that is dominated by architecture prose. The above-the-fold experience should contain compact product labels, metrics, cards, and lineage, with detailed API/architecture explanations behind proof mode.

## Definition of done for the whole board

The board is done only when all of the following are true:

1. The public page at `https://nva-demo.eman.network/` presents an owned operations platform, not a single dashboard or architecture essay.
2. First screen includes the six lanes: portfolio risk/value strip; read-only source evidence lane; owned backend spine lane; tool portfolio lane; locked side effects / review gates lane; pilot ask close.
3. At least four tools are present and inspectable: Manager Daily Brief, Data Quality Hygiene, Intake / Booking Triage, BI / Read Model Reporting.
4. Each tool exposes source signals, normalized NVA facts, review gates, output/action, outcome/labor metric, and proof artifact.
5. Safety locks are visible as product state: sample workspace, read-only source, write locked, manager review open, outbox candidate only.
6. The page avoids visible meta-demo/presenter copy and old text-heavy presentation labels.
7. Regression tests enforce required phrases, forbidden phrases, safety boundaries, interaction/data structures, and proof-mode requirements.
8. Local checks pass with npm workspaces, not pnpm:
   - `npm --workspace @pet-resort/staff-web run test`
   - `npm --workspace @pet-resort/staff-web run build`
9. Repository docs checks pass:
   - `python scripts/check_markdown_links.py --repo-root .`
   - `./scripts/check_docs.sh` when reasonably fast and available.
10. A visual screenshot is captured/reviewed for first-screen hierarchy, safety locks, copy density, and responsive layout.
11. Public URL verification succeeds after deploy:
   - `curl -fsSL https://nva-demo.eman.network/`
   - browser visual QA of the deployed page;
   - forbidden phrase checks against deployed HTML/text;
   - visible safety/no-live-side-effects posture.
12. Final reviewer records deploy state: commit/branch or artifact reviewed, local tests/build status, public URL status, screenshot path or visual QA notes, and any caveats.

## Later-worker handoff summary

Build a first-screen owned operations cockpit with six lanes: portfolio risk/value; read-only source evidence; owned backend spine; four-tool portfolio; locked side effects/review gates; and safe read-only pilot ask. The required regression terms are the product boundary: sample workspace, read-only source evidence, NVA-owned operating facts, workflow packets, review gates, write locked, manager review open, outbox candidate only, and explicit locks for customer sends, provider writes, schedules, payments/discounts, medical/safety decisions, and staffing mandates.
