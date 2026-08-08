# Intelligible Manager Brief Demo Acceptance Contract

## Goal

The demo must let a job contact understand the pet-resort workflow, no-live-access boundary, manager review gate, labor value, and narrow next ask in 3–5 minutes.

It must read as a product workflow first: messy morning signals become tracked facts, tracked facts become a review-gated Manager Daily Brief, and reviewed actions become outcome/labor proof. Architecture, owned-backend proof, and Gingr-migration strategy are secondary and must not be required to understand the value.

## Above-the-fold requirements

The first screen must make the work problem and safe boundary obvious without presenter rescue. It must include:

- a concrete synthetic morning scenario;
- messy operational signals before the clean brief;
- a visible manager action plan;
- a visible safety boundary;
- a visible labor/value metric;
- a clear synthetic/no-live-access label.

A viewer should be able to say, within the first minute: this is a synthetic resort morning, the system gathered messy source signals, and the manager is getting a safer action plan rather than a live automation tool.

## Interaction requirements

The demo must have a visible start, middle, and finish:

1. **Messy morning signals** — the viewer sees the before-state: arrivals, documents, staffing, capacity, and buried notes.
2. **Facts tracked with source refs/caveats** — the viewer sees that the system preserves provenance, freshness, field paths, quality flags, and review status instead of blindly trusting source data.
3. **Ranked manager brief** — the viewer sees what the manager should review or do next, why now, who owns it, the expected labor impact, and whether it is safe to approve now or blocked.
4. **Review/outcome proof** — the viewer sees that approved work records evidence and labor minutes while unsafe side effects stay locked.

Every interaction should answer one of these questions:

- What was painful before?
- What did the software organize or clarify?
- What can the manager safely act on?
- What remains blocked because there is no live access or approval?
- What evidence exists that the workflow is more than a static mockup?

## Safety requirements

The demo must explicitly label itself as synthetic/local proof only. It must not imply live NVA access, live Gingr access, production data access, production ROI proof, writeback readiness, or customer-facing automation.

The demo must explicitly block or disclaim all forbidden side effects:

- live customer sends;
- PMS/provider writes;
- payment/refund actions;
- schedule/capacity mutation;
- medical/safety decisions;
- claims of production NVA/Gingr access.

Blocked states are part of the product promise. They should be visible, plain-English, and framed as safety/review controls rather than missing functionality.

## Proof requirements

The page or adjacent presentation docs must point to concrete local proof, including:

- staff-web smoke tests covering the workflow and blocked side effects;
- a local API demo script proving side effects are disabled;
- source refs/caveats attached to facts;
- outcome proof that records estimated versus reviewed labor minutes;
- the synthetic-data-only/no-live-access boundary.

Proof must support the manager workflow. It should not make the first impression about architecture, DTOs, owned-backend replacement, or Gingr migration.

## Failure conditions

The demo fails this contract if any of the following are true:

- A non-technical job contact cannot understand the scenario, safety boundary, and next ask in 3–5 minutes.
- The first minute sounds like architecture or migration strategy instead of a resort workflow.
- The first screen feels like labeled cards rather than a concrete morning scenario.
- The viewer cannot explain what the manager does next.
- The viewer cannot explain what is review-ready versus blocked.
- The viewer cannot explain what real access would unlock next.
- The demo hides or softens the synthetic/no-live-access label.
- The demo implies live NVA/Gingr data, live customer sends, PMS/provider writes, schedule/capacity mutation, payment/refund action, or medical/safety decisioning.
- The demo lacks a clear start, middle, finish, or narrow read-only next ask.
