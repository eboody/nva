# Operator Workflow Demo Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Replace the current sales-pitch-heavy presentation page with a concrete, interactive pet-resort operator workflow that shows messy work becoming a manager action plan, with safety gates and live API proof as supporting evidence.

**Architecture:** Keep the existing Next.js staff demo and Rust API, but change the presentation hierarchy: the first screen becomes an operator scenario, the middle is a working before/after workflow, and architecture/proof becomes secondary/expandable evidence. Add a small typed scenario model in `apps/staff-web/app/page.tsx` first; only add/adjust API endpoints if the UI cannot honestly show a live manager brief from existing `/v0/agent/context/manager-daily-brief` and `/v0/read-models/source-quality-backlog` data.

**Tech Stack:** Next.js/React in `apps/staff-web`, existing local-demo proxy at `apps/staff-web/app/api/local-demo/[...path]/route.ts`, Rust API endpoints under `apps/api`, existing demo script `./scripts/demo_owned_operations_api.sh`, deployed Coolify app at `https://nva-demo.eman.network`.

---

## Non-negotiable demo bar

The demo must make sense to someone who does **not** care about “owned backend,” “read models,” or “Gingr migration” yet.

Primary audience reaction should be:

> “I see the operational mess. I see the software turn it into a useful manager action plan. I see where it refuses to do unsafe things. I see why read-only access would make this real.”

If the first minute still feels like architecture or sales copy, the plan failed.

## Current problem

The live site currently has real technical proof, but the visible experience leads with explanation:

- owned backend map;
- labor tools portfolio;
- pilot close;
- architecture labels;
- proof counters.

That is useful backup, but it is not a compelling demo. The next pass should **demote pitch material** and **promote a concrete operator workflow**.

## Target experience

### First screen: “Morning manager brief”

Show one realistic synthetic shift:

- 12 boarding/daycare arrivals;
- 3 source-data problems;
- 2 customer replies needing staff review;
- 1 capacity/staffing risk;
- 1 unsafe action blocked;
- estimated 37 minutes of manager/front-desk work avoided.

Primary button: **Run morning brief**.

### Main demo interaction

The viewer sees three columns:

1. **Messy source feed**
   - customer request snippets;
   - vaccine/document mismatch;
   - duplicate or stale source field;
   - coverage/capacity cue;
   - “from Gingr/source export” labels as evidence, not authority.

2. **Owned workflow processing**
   - normalized work packets;
   - missing facts;
   - risk labels;
   - draft reply created;
   - action blocked when unsafe.

3. **Manager action plan**
   - prioritized actions;
   - who owns each action;
   - what can be approved now;
   - what remains blocked;
   - minutes saved and proof links.

### Supporting evidence

Below the workflow, keep a compact “technical proof” drawer:

- live `/v0/readyz` status;
- live `/v0/ops/metrics/summary` counters;
- live `/v0/read-models/source-quality-backlog` rows;
- live `/v0/agent/context/manager-daily-brief?...` response;
- raw JSON excerpt.

Architecture/migration language should move under a “Why this scales” section after the workflow proves value.

---

## Task 1: Freeze the current state and define the demo acceptance tests

**Objective:** Create a small written acceptance contract so workers do not keep adding copy instead of demo value.

**Files:**
- Create: `docs/presentation/operator-workflow-demo-acceptance.md`
- Modify: none

**Step 1: Write the acceptance contract**

Create `docs/presentation/operator-workflow-demo-acceptance.md` with these sections:

```markdown
# Operator Workflow Demo Acceptance Contract

## Goal

The demo must show a pet-resort manager/front-desk workflow becoming easier, safer, and measurable in under two minutes.

## Must be visible above the fold

- A concrete synthetic shift/scenario, not an architecture thesis.
- A button or obvious interaction labeled "Run morning brief" or equivalent.
- A before/after operational transformation.
- A safety boundary: no live sends / no PMS writes.
- A measurable result: minutes saved, actions prioritized, risks blocked.

## Must be visible in the main workflow

- Messy source facts.
- Normalized work packets.
- Prioritized manager actions.
- At least one unsafe action blocked.
- At least one staff-approvable draft/action.
- Evidence that some data comes from live API proof, not only static page copy.

## Allowed supporting sections

- Technical proof drawer.
- Architecture explanation after the workflow.
- Pilot/read-only-access ask after the workflow.

## Failure conditions

- The first minute is mostly sales/architecture copy.
- The user has to understand "owned backend" before understanding the demo.
- The demo claims live NVA/Gingr access.
- The demo claims production savings.
- The demo enables or implies live customer sends, PMS writes, payment/refund actions, schedule/capacity mutations, or medical/safety decisions.
```

**Step 2: Verify it reads as a product bar**

Run:

```bash
python scripts/check_markdown_links.py --repo-root .
```

Expected: passes.

**Step 3: Commit**

```bash
git add docs/presentation/operator-workflow-demo-acceptance.md
git commit -m "docs: define operator workflow demo acceptance bar"
```

---

## Task 2: Refactor staff-web data into a scenario model

**Objective:** Stop scattering pitch copy through JSX and introduce a typed scenario that can drive a concrete workflow UI.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`

**Step 1: Add types near the existing type definitions**

Add:

```ts
type SourceFeedItem = {
  id: string;
  source: string;
  title: string;
  detail: string;
  risk: "low" | "medium" | "high";
  evidence: string;
};

type ManagerAction = {
  id: string;
  priority: "now" | "today" | "watch";
  owner: "front desk" | "manager" | "system";
  title: string;
  reason: string;
  status: "ready for staff approval" | "blocked" | "informational";
  minutesSaved: number;
};

type DemoScenario = {
  name: string;
  daySummary: string;
  shiftFacts: string[];
  sourceFeed: SourceFeedItem[];
  managerActions: ManagerAction[];
  blockedAction: {
    title: string;
    reason: string;
  };
};
```

**Step 2: Add one concrete scenario constant**

Add near `backendPieces` / `laborTools`:

```ts
const morningBriefScenario: DemoScenario = {
  name: "Morning manager brief: boarding + daycare exceptions",
  daySummary: "Synthetic 7:20am shift: arrivals are starting, source records disagree, and staff need a clean action plan before the lobby rush.",
  shiftFacts: [
    "12 arrivals before 10am",
    "3 source-data issues",
    "2 reply drafts need review",
    "1 unsafe confirmation blocked",
    "37 estimated minutes avoided"
  ],
  sourceFeed: [
    {
      id: "src-vax-miso",
      source: "Gingr/source export",
      title: "Miso boarding request has uncertain vaccine proof",
      detail: "Customer says rabies record may be attached; source field is stale and cannot support automatic confirmation.",
      risk: "high",
      evidence: "source_quality_backlog row + manager brief action"
    },
    {
      id: "src-noise-miso",
      source: "Customer request text",
      title: "Noise sensitivity buried in intake message",
      detail: "Important handling note is present in prose, not in a clean operational field.",
      risk: "medium",
      evidence: "normalized into review packet"
    },
    {
      id: "src-capacity-yard",
      source: "Synthetic schedule/read model",
      title: "Late afternoon coverage looks tight",
      detail: "Manager should review coverage before promising additional daycare capacity.",
      risk: "medium",
      evidence: "manager daily brief action"
    }
  ],
  managerActions: [
    {
      id: "act-vax-review",
      priority: "now",
      owner: "front desk",
      title: "Request current vaccine document before confirming Miso",
      reason: "Prevents a risky booking confirmation from stale source evidence.",
      status: "ready for staff approval",
      minutesSaved: 12
    },
    {
      id: "act-manager-capacity",
      priority: "today",
      owner: "manager",
      title: "Review afternoon coverage before accepting extra daycare",
      reason: "Turns a hidden capacity concern into a manager-visible decision.",
      status: "blocked",
      minutesSaved: 15
    },
    {
      id: "act-cleanup-queue",
      priority: "watch",
      owner: "system",
      title: "Keep source-quality cleanup in a read-only queue",
      reason: "Creates BI/operator evidence without mutating Gingr or provider data.",
      status: "informational",
      minutesSaved: 10
    }
  ],
  blockedAction: {
    title: "Automatic booking confirmation",
    reason: "Blocked because vaccine evidence is stale and availability/capacity have not been reviewed by staff."
  }
};
```

**Step 3: Run type/lint checks**

Run:

```bash
npm --prefix apps/staff-web run typecheck
npm --prefix apps/staff-web run lint
```

Expected: both pass.

**Step 4: Commit**

```bash
git add apps/staff-web/app/page.tsx
git commit -m "refactor: model concrete operator demo scenario"
```

---

## Task 3: Replace the hero with the operator scenario

**Objective:** Make the first screen feel like a working demo, not a sales deck.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css`

**Step 1: Change hero copy**

Replace the current hero headline/subtitle with:

```tsx
<h1>Run the morning brief before the lobby rush.</h1>
<p className="hero-subtitle">
  Synthetic pet-resort shift: source records disagree, staff need safe replies, and the manager needs a prioritized action plan — with no live sends or PMS writes.
</p>
```

Change the primary button to:

```tsx
<button onClick={() => setActiveStep("intake")}>Run morning brief</button>
```

**Step 2: Replace the hero impact card with scenario facts**

Render `morningBriefScenario.shiftFacts` in the impact card instead of only a generic mini bar.

Example JSX:

```tsx
<div className="impact-card" aria-label="Morning shift summary">
  <span>{morningBriefScenario.name}</span>
  <strong>37 min</strong>
  <p>{morningBriefScenario.daySummary}</p>
  <div className="shift-fact-list">
    {morningBriefScenario.shiftFacts.map((fact) => <small key={fact}>{fact}</small>)}
  </div>
</div>
```

**Step 3: Add CSS for `.shift-fact-list`**

In `apps/staff-web/app/globals.css`, add styling that makes the facts look like operational chips, not prose.

**Step 4: Verify visually and with checks**

Run:

```bash
npm --prefix apps/staff-web run lint
npm --prefix apps/staff-web run typecheck
npm --prefix apps/staff-web run build
```

Expected: pass.

**Step 5: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css
git commit -m "feat: lead demo with morning manager scenario"
```

---

## Task 4: Move architecture/pitch sections below the workflow

**Objective:** Put the concrete workflow before backend map, labor tools, and pilot close.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css` if spacing/order styles need adjustment

**Step 1: Reorder JSX sections**

Current order is roughly:

1. hero;
2. KPI strip;
3. system map;
4. labor tools;
5. pilot close;
6. walkthrough;
7. technical proof;
8. proof grid.

Change to:

1. hero;
2. KPI strip;
3. **workflow cockpit** / walkthrough;
4. technical proof drawer;
5. system map / why this scales;
6. labor tools portfolio;
7. pilot close;
8. proof grid if still needed.

**Step 2: Rename architecture heading**

Change “Owned backend migration map” area to a lower-priority heading like:

```tsx
<p className="eyebrow">Why this scales after the demo works</p>
<h2>The backend reason this can become more than a one-off tool.</h2>
```

Do not remove architecture entirely; just stop making it the first thing.

**Step 3: Build**

Run:

```bash
npm --prefix apps/staff-web run build
```

Expected: pass.

**Step 4: Browser check**

Start local app if needed and verify the first viewport shows the scenario/workflow, not the architecture map.

**Step 5: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css
git commit -m "feat: prioritize operator workflow over architecture pitch"
```

---

## Task 5: Build the three-column workflow cockpit

**Objective:** Make the main demo visibly transform messy source work into manager actions.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css`

**Step 1: Replace or augment the existing `walkthrough-card`**

Keep the step rail if useful, but add a three-column cockpit inside the walkthrough area:

```tsx
<section className="workflow-cockpit" aria-label="Morning manager workflow cockpit">
  <div className="cockpit-column source-feed">
    <p className="eyebrow">1. Messy source feed</p>
    <h2>What staff would otherwise chase manually</h2>
    {morningBriefScenario.sourceFeed.map((item) => (
      <article className={`source-feed-item risk-${item.risk}`} key={item.id}>
        <span>{item.source}</span>
        <h3>{item.title}</h3>
        <p>{item.detail}</p>
        <small>{item.evidence}</small>
      </article>
    ))}
  </div>

  <div className="cockpit-column processing-feed">
    <p className="eyebrow">2. Owned workflow processing</p>
    <h2>Normalize, draft, gate</h2>
    <ol>
      <li>Turn source facts into review packets.</li>
      <li>Create safe staff draft without promising availability.</li>
      <li>Block confirmation until vaccine/capacity review.</li>
      <li>Record labor and audit evidence.</li>
    </ol>
    <div className="blocked-action-card">
      <span>Unsafe action blocked</span>
      <h3>{morningBriefScenario.blockedAction.title}</h3>
      <p>{morningBriefScenario.blockedAction.reason}</p>
    </div>
  </div>

  <div className="cockpit-column action-plan">
    <p className="eyebrow">3. Manager action plan</p>
    <h2>What to do next</h2>
    {morningBriefScenario.managerActions.map((action) => (
      <article className={`manager-action ${action.priority}`} key={action.id}>
        <span>{action.priority} · {action.owner}</span>
        <h3>{action.title}</h3>
        <p>{action.reason}</p>
        <footer>
          <b>{action.minutesSaved} min</b>
          <small>{action.status}</small>
        </footer>
      </article>
    ))}
  </div>
</section>
```

**Step 2: Add cockpit CSS**

Add responsive styles:

- desktop: 3 columns;
- tablet/mobile: stacked cards;
- high-risk source items and blocked action must be visually obvious;
- manager action statuses must be scannable.

**Step 3: Add an approval simulation tied to actions**

Keep `approved` state, but make it apply to the first ready action:

- button label: `Approve vaccine-record request draft`;
- after click: show `Recorded locally; live send still disabled`;
- blocked action remains blocked.

**Step 4: Verify**

Run:

```bash
npm --prefix apps/staff-web run lint
npm --prefix apps/staff-web run typecheck
npm --prefix apps/staff-web run build
```

Expected: pass.

**Step 5: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css
git commit -m "feat: add interactive manager workflow cockpit"
```

---

## Task 6: Make live API proof support the workflow instead of the other way around

**Objective:** Make technical proof reinforce the scenario with obvious evidence, not overwhelm the demo.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css`
- Modify only if necessary: `apps/api/src/http.rs`, `apps/api/openapi/owned-operations-v0.openapi.json`, tests under `apps/api/tests/`

**Step 1: Rename technical proof section**

Change heading from “Show the API and DB doing work” to:

```tsx
<h2>Proof behind the manager brief</h2>
<p>
  These calls show the scenario is backed by the deployed proxy, Rust API, seeded Postgres read model, and manager-brief context endpoint — not only page copy.
</p>
```

**Step 2: Add proof-to-workflow labels**

For each call row, add a short purpose label:

- `/v0/readyz`: system alive;
- `/v0/ops/metrics/summary`: counters/audit evidence;
- `/v0/read-models/source-quality-backlog`: messy source facts;
- `/v0/agent/context/manager-daily-brief?...`: manager actions.

**Step 3: If manager brief JSON has useful actions, display them**

If the existing manager brief response includes actions/minutes, parse and show a small “API returned manager actions” panel. If it does not, do **not** fake a live claim; leave the static scenario as synthetic and show the endpoint status/JSON excerpt.

**Step 4: Only add API shape if genuinely necessary**

If the API response cannot support a credible manager-action panel, add a small read-only field to the existing manager brief endpoint. Do not add a new endpoint unless needed.

If API changes are needed, use TDD:

- update/add contract test in `apps/api/tests/manager_daily_brief_agent_context_contract.rs`;
- update OpenAPI contract test in `apps/api/tests/owned_api_openapi_contract.rs` if schema/path changes;
- implement in `apps/api/src/http.rs` or the app layer as appropriate;
- regenerate/update `apps/api/openapi/owned-operations-v0.openapi.json` if required.

**Step 5: Verify**

Run:

```bash
npm --prefix apps/staff-web run lint
npm --prefix apps/staff-web run typecheck
npm --prefix apps/staff-web run build
./scripts/demo_owned_operations_api.sh
```

Expected: pass.

**Step 6: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css apps/api/src/http.rs apps/api/openapi/owned-operations-v0.openapi.json apps/api/tests scripts/demo_owned_operations_api.sh
git commit -m "feat: tie live proof to manager workflow demo"
```

Only include API files if changed.

---

## Task 7: Cut pitch copy by at least 40 percent

**Objective:** Remove or collapse copy that makes the demo feel like a brochure.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css`

**Step 1: Collapse system map**

Convert the full `backendPieces` grid into a compact expandable/secondary section, or reduce from 8 cards to 4 bullets:

- source evidence;
- owned workflow authority;
- read-model proof;
- approval-gated side effects.

**Step 2: Collapse labor tools portfolio**

Move labor tools below the workflow and phrase them as “what else this backend can support” instead of “look at all these claims.”

**Step 3: Shorten pilot close**

Keep only:

- read-only ask;
- 2–3 week validation frame;
- no live writes/customers/payments/schedule/safety decisions.

**Step 4: Verify the page still has safety boundaries**

Search source:

```bash
python - <<'PY'
from pathlib import Path
s=Path('apps/staff-web/app/page.tsx').read_text().lower()
for phrase in ['no live', 'no pms', 'read-only', 'blocked']:
    assert phrase in s, phrase
print('safety copy present')
PY
```

**Step 5: Build and commit**

```bash
npm --prefix apps/staff-web run build
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css
git commit -m "chore: reduce pitch copy in staff demo"
```

---

## Task 8: Add a presenter run sheet for the actually-good demo

**Objective:** Give the user a short script that matches the new demo experience.

**Files:**
- Create: `docs/presentation/operator-workflow-demo-run-sheet.md`
- Modify: `README.md` presentation path links if needed

**Step 1: Write the run sheet**

Include:

```markdown
# Operator Workflow Demo Run Sheet

## 20-second setup

"I did not have NVA or Gingr access, so I did not pretend this is production. I built the safe seam first: a synthetic pet-resort workflow that turns messy source evidence into a manager action plan while blocking live side effects."

## 90-second live path

1. Open https://nva-demo.eman.network.
2. Click "Run morning brief".
3. Point to messy source feed.
4. Point to normalized workflow processing.
5. Point to manager action plan.
6. Click the staff approval simulation.
7. Point out the unsafe action remains blocked.
8. Open proof drawer and rerun live proof.

## What to say when asked "why does this matter?"

"Because this is the first safe step before integration. With read-only exports/sample rows, this becomes a validation project: compare owned read models to current operator/BI questions, measure real minutes, and only then consider controlled writeback."

## Claims to avoid

- Do not claim live NVA/Gingr data.
- Do not claim production deployment.
- Do not claim real savings.
- Do not imply autonomous customer sends or PMS writes.
```

**Step 2: Link it from README presentation path**

Add the run sheet near the top of the presentation path list.

**Step 3: Verify**

```bash
python scripts/check_markdown_links.py --repo-root .
./scripts/check_docs.sh
```

Expected: pass.

**Step 4: Commit**

```bash
git add docs/presentation/operator-workflow-demo-run-sheet.md README.md
git commit -m "docs: add operator workflow demo run sheet"
```

---

## Task 9: Browser QA the new demo locally

**Objective:** Verify the demo feels like software, not a pitch, before deploying.

**Files:**
- No source changes unless QA finds issues

**Step 1: Build**

```bash
npm --prefix apps/staff-web run build
```

Expected: pass.

**Step 2: Run local app**

Use the existing local demo stack/runbook. If the deployed proxy/API is easier and safe, use that; otherwise run the app locally.

**Step 3: Browser checklist**

Verify:

- first viewport shows the morning brief scenario;
- “Run morning brief” is obvious;
- messy source feed is visible without scrolling too far;
- manager actions are visible and understandable;
- approval simulation works;
- blocked unsafe action remains blocked;
- technical proof can be run;
- no raw secret/token/customer-real data appears;
- mobile/narrow viewport remains usable.

**Step 4: If issues are found**

Patch only presentation/UX bugs required for the checklist. Do not add new architecture copy.

**Step 5: Commit QA fixes**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css
git commit -m "fix: polish operator workflow demo QA"
```

Skip commit if no changes.

---

## Task 10: Final verification, push, deploy, and live smoke

**Objective:** Ship only after the demo is visibly better.

**Files:**
- No source changes unless final smoke finds issues

**Step 1: Full local verification**

Run:

```bash
git diff --check
npm --prefix apps/staff-web run lint
npm --prefix apps/staff-web run typecheck
npm --prefix apps/staff-web run build
./scripts/demo_owned_operations_api.sh
python scripts/check_markdown_links.py --repo-root .
./scripts/check_docs.sh
```

Expected: pass, or documented pre-existing unrelated failures only.

**Step 2: Push**

```bash
git status -sb
git push origin main
```

Expected: branch no longer ahead.

**Step 3: Deploy via existing Coolify path**

Use the already-established deployment process for `https://nva-demo.eman.network`. Do not change live secrets or service topology unless needed.

**Step 4: Live smoke**

Open `https://nva-demo.eman.network` and verify:

- first viewport is the operator scenario;
- workflow cockpit appears before architecture map;
- technical proof returns HTTP 200 rows/counters/actions;
- manager brief proof works;
- blocked action wording is visible;
- no live-access overclaims.

**Step 5: Final report**

Report:

- commit SHA;
- deployed URL;
- verification commands;
- what changed in the first minute of the demo;
- remaining caveats.

---

## Kanban execution shape

If implemented through Kanban, create a new board rather than reusing the completed `nva-owned-backend-migration-demo` board.

Suggested board: `nva-operator-workflow-demo`

Serialized shared-checkout chain:

1. `docs: operator workflow acceptance contract`
2. `refactor: scenario model`
3. `ui: morning brief hero`
4. `ui: reorder workflow before architecture`
5. `ui: three-column workflow cockpit`
6. `proof: tie API evidence to manager workflow`
7. `copy: cut pitch copy`
8. `docs: presenter run sheet`
9. `qa: browser review and polish`
10. `release: verify, push, deploy, live smoke`

Assignees:

- docs cards: `pet-resort-docs`
- UI/API cards: `pet-resort-code`
- QA/release: `pet-resort-reviewer`

Dispatch policy:

- Shared checkout: one runnable card at a time unless explicit isolated worktrees are created.
- Routine `review-required` gates may be cleared by owner review with focused checks.
- Stop for credentials, destructive actions, real customer/member-facing actions, production data, live PMS/provider writes, payment/refund actions, schedule/capacity changes, or medical/safety decisions.

## Definition of done

The final deployed demo is good only if a viewer can understand the value in this order:

1. “Here is the messy pet-resort shift.”
2. “Here is what the software did with it.”
3. “Here is what the manager should do next.”
4. “Here is the unsafe thing it refused to do.”
5. “Here is the live API/DB proof.”
6. “Here is why read-only access is the right next step.”

If the viewer has to start with “owned backend migration map,” it is not done.
