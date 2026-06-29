# NVA Owned Operations Platform Demo v2 Kanban Board

**Created:** 2026-06-29

**User intent:** Start over with the public demo. The new demo must show how the infrastructure already built in this repo can get used: source systems become read-only evidence, NVA owns workflow/domain/API/storage/review/audit/read-model authority, and multiple labor-cost tools use the same backend. Cards must be comprehensive enough that workers can execute high-quality work without repeated owner steering.

## North-star demo thesis

This is not “another dashboard.” It is a shareable executive/product demo for an owned operations platform:

```text
Gingr / PMS / exports / BI queries / schedules / docs
  -> source adapters + provenance snapshots
  -> NVA-owned operating facts and workflow packets
  -> versioned operations API + durable projections
  -> review gates + audit/outbox posture
  -> manager/staff/BI/AI tools
  -> measured labor-cost reduction
```

The CEO/CTO takeaway should be:

> Gingr and adjacent systems can remain evidence sources at first, but NVA should own its operating model. This platform lets NVA replace dependence piece by piece while immediately creating safer labor-cost tools.

## Non-negotiable presentation constraints

- No live NVA/Gingr data claims.
- No customer sends, PMS/provider writes, schedule changes, payment/refund/discount changes, medical/safety decisions, or staffing mandates.
- The no-access boundary must be visible as product state: `sample workspace`, `read-only source`, `write locked`, `manager review open`, `outbox candidate only`.
- Avoid giant disclaimer ribbons and meta-demo copy such as `DEMO MODE`, `presenter`, `talk track`, `proper demo page`, `Comprehensive Summary`, `Migration Strategy`.
- Lead with software-in-use and business value, not architecture jargon.
- Still make the infrastructure visible through use: every tool card should be traceable to source evidence, owned facts, review gates, and outcome/read-model proof.

## Desired first-screen story

Within 30 seconds, a CEO should understand:

1. Current operational risk/cost pressure across a sample resort/portfolio.
2. The platform has read-only source evidence from incumbent systems.
3. That evidence is transformed into NVA-owned workflow facts.
4. Several tools use the same backend: Manager Daily Brief, Data Quality Hygiene, Intake/Booking Triage, BI/Read Model.
5. Unsafe side effects are locked.
6. The narrow pilot ask is safe: provide read-only exports/field dictionaries/BI query inventory for one pilot slice.

## Board execution policy

- Shared checkout: `/home/eran/code/nva`.
- Serialize code/UI work unless explicit isolated worktrees are created.
- Use profiles that exist on this host: `pet-resort-docs`, `pet-resort-code`, `pet-resort-reviewer`.
- Use `npm` workspaces, not `pnpm`.
- Do not dirty/commit generated `.next` artifacts, `next-env.d.ts`, or `tsconfig.json` unless intentionally changed and justified.
- Run focused frontend checks after code changes:
  - `npm --workspace @pet-resort/staff-web run test`
  - `npm --workspace @pet-resort/staff-web run build`
- Before final handoff, also verify the public URL after deploy:
  - `curl -fsSL https://nva-demo.eman.network/`
  - browser visual QA
  - forbidden phrase checks
  - safety/no-live-side-effects posture visible

## Card graph

1. **Acceptance contract + product storyboard** (`pet-resort-docs`) — defines the new demo contract, visual hierarchy, copy budget, data/storyboard, and regression acceptance. Parent of all implementation.
2. **Demo data model + UI architecture scaffold** (`pet-resort-code`) — refactors staff-web demo data into explicit source/backend/tool/proof structures and adds regression tests before visual rebuild.
3. **Executive platform cockpit first screen** (`pet-resort-code`) — builds the above-the-fold owned-operations cockpit: risk/value metrics, source evidence lane, owned backend spine, tool portfolio lane.
4. **Interactive lineage + multi-tool workflows** (`pet-resort-code`) — makes each tool clickable and traces messy source evidence through owned facts/review gates/outcomes.
5. **Proof mode + real repo/backend evidence hooks** (`pet-resort-code`) — adds expandable proof using existing OpenAPI/local demo/read-model/storage/audit artifacts without overwhelming the CEO view.
6. **CEO pilot close + ROI scaler + safe next ask** (`pet-resort-docs` or `pet-resort-code`) — completes the executive close and pilot framing in UI/docs.
7. **Visual polish, accessibility, and regression hardening** (`pet-resort-code`) — density, responsiveness, visual hierarchy, copy/forbidden-phrase tests, screenshots.
8. **Final review, deploy, and public readiness verdict** (`pet-resort-reviewer`) — independent review, gates, deploy via Coolify if needed, public verification, final shareable URL and caveats.

## Final definition of done

- The deployed page at `https://nva-demo.eman.network/` clearly presents an owned operations platform demo, not a generic dashboard or architecture essay.
- The first screen shows source evidence -> owned backend -> multiple labor tools -> measured value -> locked side effects.
- Interactions let the viewer inspect lineage for at least four tool surfaces: Manager Daily Brief, Data Quality Hygiene, Intake/Booking Triage, BI/Read Model.
- The proof layer ties the UI to repo artifacts: source/provenance boundaries, owned API/OpenAPI, storage/projections, review gates, audit/outbox, outcome/labor metrics.
- Regression tests reject meta-demo labels, stale old-page copy, unsafe live-action claims, and excessive architecture-first copy in the primary surface.
- Local tests/build pass.
- Public deployment is verified externally and visually.
