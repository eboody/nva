# Intelligible Manager Brief Demo Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Turn the current polished Manager Daily Brief screen into an actually intelligible, job-contact-ready demo that a non-NVA, non-technical viewer can understand in 3–5 minutes without live NVA/Gingr access.

**Architecture:** Keep the current `apps/staff-web` visual-first Next.js demo, but add a stronger narrative layer, explicit before/after progression, live/local proof affordances, and a rehearsable presentation package. The primary screen must remain product-in-use; API, owned-backend, Gingr replacement, docs, and architecture proof stay secondary and are only surfaced after the viewer understands the manager workflow.

**Tech Stack:** Next.js/React in `apps/staff-web`, static/synthetic scenario data in `apps/staff-web/app/page.tsx`, CSS in `apps/staff-web/app/globals.css`, smoke assertions in `apps/staff-web/smoke/staff-dashboard-mvp.test.mjs`, existing local API proof from `./scripts/demo_owned_operations_api.sh`, presentation docs under `docs/presentation/`.

---

## Current state

The current demo is significantly better than the previous sales-pitch screen. It now shows:

- `Manager Daily Brief` as the main product artifact.
- Five synthetic source facts: reservation, pet profile, rabies proof, capacity, labor plan.
- A tracking layer: source ref, field path, freshness, quality flag, review gate, labor estimate.
- Safety boundaries: customer send locked, PMS write locked, manager review ready.
- Ranked manager actions and outcome proof.
- A record-review interaction.

The remaining problem is **intelligibility under presentation pressure**. A job contact should not have to infer the story from cards. The demo needs to clearly answer:

1. What was painful before?
2. What did the software do?
3. What can the manager safely act on?
4. What remains blocked because there is no live access / approval?
5. What proof exists that this is more than a mockup?
6. What is the narrow next ask?

## Non-negotiable demo bar

The finished demo should produce this viewer reaction:

> “I understand the work problem. I can see the system taking messy resort signals and turning them into a manager action plan. I can see that unsafe actions are blocked. I understand why this is useful even without live access. I know what access would unlock next.”

Failure conditions:

- The first screen feels like cards with labels rather than a scenario.
- The viewer has to understand “owned backend,” “Gingr migration,” “DTOs,” or “read models” before understanding value.
- The demo hides the no-access boundary.
- The demo implies live NVA/Gingr data, live customer sends, PMS writes, schedule/capacity mutation, payment/refund action, or medical/safety decisioning.
- The demo has no obvious start, middle, finish, or next ask.

## Target demo structure

### 0. Opening line, outside the app

Use this when presenting:

> “Because I did not have NVA or Gingr access, I built the safe thing first: a synthetic but source-backed manager brief that shows how messy resort signals become reviewed actions, with live side effects blocked.”

### 1. First screen: the pain

Before the current source cards, add a compact “morning chaos” strip:

- “7:20am lobby rush”
- “12 arrivals before 10”
- “rabies proof unclear”
- “coverage 2 short”
- “quiet-room request buried in note”

This makes the source cards feel like a real morning, not abstract data architecture.

### 2. Main interaction: four visible beats

Keep the four existing steps but rename them into human phrases:

1. **Messy morning** — the raw signals.
2. **Facts tracked** — what the system records and refuses to trust blindly.
3. **Manager brief** — the prioritized action plan.
4. **Review recorded** — the safe outcome/audit proof.

Each step should visibly change emphasis and include one plain-English sentence explaining what happened.

### 3. Manager action plan: make each action answer “why now?”

Each ranked action should show:

- action;
- owner;
- reason;
- required review gate;
- before/after time;
- whether it is safe to approve now or blocked.

At least one action must be ready-for-review, and at least one must be blocked.

### 4. Safety boundary: make blocked actions unmistakable

The safety card should say, in very few words:

- “No live customer send”
- “No PMS write”
- “No schedule/payment/medical decision”
- “Manager review only”

The no-access honesty is a strength. Do not bury it.

### 5. Proof drawer: show that this is backed by repo/runtime work

Below the visual workflow, add a collapsible or compact “Proof behind the scene” section:

- `synthetic scenario only`
- `local API smoke passes`
- `source refs attached`
- `side effects disabled`
- `outcome recorded`
- link/label for `./scripts/demo_owned_operations_api.sh`

This should be visible enough for technical credibility but not part of the first impression.

### 6. Close: narrow next ask

Add a final footer/card:

> “Next useful access: read-only sample exports, field dictionary, and BI query inventory for one workflow. No writes or customer sends.”

This turns the demo into a credible conversation rather than just a portfolio artifact.

---

## Implementation tasks

### Task 1: Add a durable acceptance contract for intelligibility

**Objective:** Prevent future polish from drifting back into sales copy or architecture-first presentation.

**Files:**
- Create: `docs/presentation/intelligible-manager-brief-demo-acceptance.md`

**Step 1: Create the acceptance contract**

Write this file:

```markdown
# Intelligible Manager Brief Demo Acceptance Contract

## Goal

The demo must let a job contact understand the pet-resort workflow, no-access boundary, and next ask in 3–5 minutes.

## Above-the-fold requirements

- A concrete synthetic morning scenario.
- Messy operational signals before the clean brief.
- A visible manager action plan.
- A visible safety boundary.
- A visible labor/value metric.
- A clear synthetic/no-live-access label.

## Interaction requirements

The demo must have a start, middle, and finish:

1. messy morning signals;
2. facts tracked with source refs/caveats;
3. ranked manager brief;
4. review/outcome proof.

## Safety requirements

The demo must explicitly block or disclaim:

- live customer sends;
- PMS/provider writes;
- payment/refund actions;
- schedule/capacity mutation;
- medical/safety decisions;
- claims of production NVA/Gingr access.

## Proof requirements

The page or adjacent presentation docs must point to:

- staff-web smoke tests;
- local API demo script;
- side-effect-disabled proof;
- synthetic-data-only boundary.

## Failure conditions

- The first minute sounds like architecture.
- The viewer cannot explain what the manager does next.
- The viewer cannot explain what is blocked.
- The viewer cannot explain what real access would unlock.
```

**Step 2: Verify markdown links**

Run:

```bash
python scripts/check_markdown_links.py --repo-root .
```

Expected: pass.

**Step 3: Commit**

```bash
git add docs/presentation/intelligible-manager-brief-demo-acceptance.md
git commit -m "docs: define intelligible manager brief demo bar"
```

---

### Task 2: Rename the four demo steps into human workflow beats

**Objective:** Make the demo self-guiding for a non-technical viewer.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/smoke/staff-dashboard-mvp.test.mjs`

**Step 1: Update the `steps` labels**

Replace the current labels:

```ts
const steps: Array<{ id: StepId; number: string; label: string }> = [
  { id: "collect", number: "01", label: "collect" },
  { id: "track", number: "02", label: "track" },
  { id: "brief", number: "03", label: "brief" },
  { id: "outcome", number: "04", label: "prove" }
];
```

with:

```ts
const steps: Array<{ id: StepId; number: string; label: string; explainer: string }> = [
  { id: "collect", number: "01", label: "messy morning", explainer: "Five disconnected signals become one reviewable operating picture." },
  { id: "track", number: "02", label: "facts tracked", explainer: "Every fact keeps source, freshness, caveat, and review status attached." },
  { id: "brief", number: "03", label: "manager brief", explainer: "The manager sees the next safest actions, ranked by urgency and labor impact." },
  { id: "outcome", number: "04", label: "review recorded", explainer: "Approved work records labor proof; unsafe side effects stay locked." }
];
```

**Step 2: Render the active explainer near the step dock**

Add:

```ts
const activeStepDetail = steps[activeIndex] ?? steps[0];
```

Then render near the step dock:

```tsx
<p className="step-explainer">{activeStepDetail.explainer}</p>
```

**Step 3: Update smoke assertions**

Add expected strings:

```js
"messy morning",
"facts tracked",
"manager brief",
"review recorded"
```

**Step 4: Run focused test**

```bash
npm --workspace @pet-resort/staff-web run test
```

Expected: pass.

**Step 5: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/smoke/staff-dashboard-mvp.test.mjs
git commit -m "feat: make demo steps explain the manager workflow"
```

---

### Task 3: Add a “morning chaos” strip above the source facts

**Objective:** Show the before-state so the brief feels useful instead of decorative.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css`
- Modify: `apps/staff-web/smoke/staff-dashboard-mvp.test.mjs`

**Step 1: Add scenario pain data**

Near `sourceFacts`, add:

```ts
const morningChaos = [
  "7:20am lobby rush",
  "12 arrivals before 10",
  "rabies proof unclear",
  "coverage 2 short",
  "quiet-room request buried"
];
```

**Step 2: Render the strip under the hero row**

Add after `<header className="hero-row">...</header>`:

```tsx
<section className="chaos-strip" aria-label="Synthetic morning scenario">
  <b>Before the brief:</b>
  {morningChaos.map((item) => <span key={item}>{item}</span>)}
</section>
```

**Step 3: Style the strip**

Add to `globals.css`:

```css
.chaos-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  padding: 10px 12px;
  border-radius: 24px;
  background: rgba(251, 191, 36, .10);
  border: 1px solid rgba(251, 191, 36, .22);
}
.chaos-strip b {
  color: #fde68a;
  font-weight: 950;
}
.chaos-strip span {
  border-radius: 999px;
  padding: 7px 10px;
  background: rgba(255,255,255,.08);
  color: #fff7ed;
  font-weight: 850;
  font-size: .86rem;
}
```

Adjust `.demo-frame` grid rows if needed so the strip has its own row.

**Step 4: Update smoke assertions**

Assert the key pain strings exist:

```js
"Before the brief",
"7:20am lobby rush",
"12 arrivals before 10",
"rabies proof unclear",
"coverage 2 short"
```

**Step 5: Run focused test and build**

```bash
npm --workspace @pet-resort/staff-web run test
npm --workspace @pet-resort/staff-web run build
```

Expected: pass.

**Step 6: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css apps/staff-web/smoke/staff-dashboard-mvp.test.mjs
git commit -m "feat: show the messy morning before the manager brief"
```

---

### Task 4: Make action status visually explicit

**Objective:** Make it obvious which actions are review-ready and which are blocked.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css`
- Modify: `apps/staff-web/smoke/staff-dashboard-mvp.test.mjs`

**Step 1: Extend `briefActions` with status**

Change the action type to include:

```ts
status: "review-ready" | "blocked" | "watch";
```

Set statuses:

- Review boarding vs labor: `blocked`
- Clear rabies document: `review-ready`
- Quiet-room plan for Miso: `review-ready`

**Step 2: Render a visible badge**

Inside each `.brief-action`, add:

```tsx
<span className={`status-badge ${action.status}`}>
  {action.status === "review-ready" ? "review ready" : action.status}
</span>
```

**Step 3: Style badges**

Add:

```css
.status-badge {
  justify-self: end;
  align-self: start;
  border-radius: 999px;
  padding: 6px 9px;
  font-size: .68rem;
  font-weight: 950;
  text-transform: uppercase;
  letter-spacing: .08em;
}
.status-badge.review-ready { color: #052e16; background: #bbf7d0; }
.status-badge.blocked { color: #450a0a; background: #fecaca; }
.status-badge.watch { color: #1e1b4b; background: #c7d2fe; }
```

Adjust `.brief-action` grid if necessary.

**Step 4: Update assertions**

Assert:

```js
"review ready",
"blocked"
```

**Step 5: Verify**

```bash
npm --workspace @pet-resort/staff-web run test
npm --workspace @pet-resort/staff-web run build
```

Expected: pass.

**Step 6: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css apps/staff-web/smoke/staff-dashboard-mvp.test.mjs
git commit -m "feat: mark manager brief actions as ready or blocked"
```

---

### Task 5: Add a compact no-access honesty card

**Objective:** Turn the lack of production access into a trust signal instead of an awkward caveat.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css`
- Modify: `apps/staff-web/smoke/staff-dashboard-mvp.test.mjs`

**Step 1: Add `blockedBoundaries` data**

```ts
const blockedBoundaries = [
  "no live NVA/Gingr data",
  "no customer sends",
  "no PMS writes",
  "no payment/schedule/medical decisions"
];
```

**Step 2: Render below or inside the gate card**

```tsx
<div className="honesty-card" aria-label="No access safety boundary">
  <b>Built without live access</b>
  <p>Synthetic proof only. The useful next step is read-only validation, not live writes.</p>
  <div>
    {blockedBoundaries.map((boundary) => <span key={boundary}>{boundary}</span>)}
  </div>
</div>
```

**Step 3: Style it as strong but not dominant**

```css
.honesty-card {
  border-radius: 22px;
  padding: 12px;
  background: rgba(15,23,42,.72);
  border: 1px solid rgba(248,113,113,.28);
}
.honesty-card b { color: #fecaca; font-weight: 950; }
.honesty-card p {
  margin: 7px 0 10px;
  color: #cbd5e1;
  font-size: .82rem;
  line-height: 1.35;
}
.honesty-card div { display: flex; flex-wrap: wrap; gap: 7px; }
.honesty-card span {
  border-radius: 999px;
  padding: 5px 8px;
  background: rgba(248,113,113,.12);
  color: #fecaca;
  font-size: .68rem;
  font-weight: 900;
  text-transform: uppercase;
}
```

**Step 4: Update assertions**

Assert:

```js
"Built without live access",
"no live NVA/Gingr data",
"no customer sends",
"no PMS writes",
"read-only validation"
```

**Step 5: Verify**

```bash
npm --workspace @pet-resort/staff-web run test
npm --workspace @pet-resort/staff-web run build
```

Expected: pass.

**Step 6: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css apps/staff-web/smoke/staff-dashboard-mvp.test.mjs
git commit -m "feat: make demo no-access boundary explicit"
```

---

### Task 6: Add a proof drawer below the visual demo

**Objective:** Give technical/job contacts a credible evidence trail without polluting the first impression.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css`
- Modify: `apps/staff-web/smoke/staff-dashboard-mvp.test.mjs`

**Step 1: Add proof bullets**

```ts
const proofBullets = [
  "staff-web smoke tests assert the workflow and blocked side effects",
  "local API demo script proves side effects disabled",
  "source refs and caveats stay attached to facts",
  "outcome proof records estimated vs reviewed minutes",
  "synthetic fixture only until read-only access is approved"
];
```

**Step 2: Render a `<details>` section after the main lab**

```tsx
<details className="proof-drawer">
  <summary>Proof behind the scene</summary>
  <ul>
    {proofBullets.map((bullet) => <li key={bullet}>{bullet}</li>)}
  </ul>
  <code>./scripts/demo_owned_operations_api.sh</code>
</details>
```

**Step 3: Style the drawer**

```css
.proof-drawer {
  border-radius: 24px;
  padding: 14px 16px;
  background: rgba(3,7,18,.52);
  border: 1px solid rgba(255,255,255,.12);
}
.proof-drawer summary {
  cursor: pointer;
  font-weight: 950;
  color: #bfdbfe;
}
.proof-drawer ul {
  margin: 12px 0;
  display: grid;
  gap: 7px;
  color: #cbd5e1;
}
.proof-drawer code {
  display: inline-block;
  border-radius: 12px;
  padding: 8px 10px;
  background: rgba(15,23,42,.9);
  color: #a7f3d0;
}
```

**Step 4: Update assertions**

Assert:

```js
"Proof behind the scene",
"./scripts/demo_owned_operations_api.sh",
"side effects disabled",
"synthetic fixture only"
```

**Step 5: Verify**

```bash
npm --workspace @pet-resort/staff-web run test
npm --workspace @pet-resort/staff-web run build
```

Expected: pass.

**Step 6: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css apps/staff-web/smoke/staff-dashboard-mvp.test.mjs
git commit -m "feat: add proof drawer to manager brief demo"
```

---

### Task 7: Add a presentation close / next ask section

**Objective:** Make the demo lead naturally into a job-contact conversation.

**Files:**
- Modify: `apps/staff-web/app/page.tsx`
- Modify: `apps/staff-web/app/globals.css`
- Modify: `apps/staff-web/smoke/staff-dashboard-mvp.test.mjs`

**Step 1: Render the close after the proof drawer**

```tsx
<section className="next-ask" aria-label="Safe next step">
  <h2>What real access would unlock</h2>
  <p>Start with read-only sample exports, field dictionaries, and BI query inventory for one workflow. Keep writes, sends, payments, schedules, and medical/safety decisions out of scope.</p>
</section>
```

**Step 2: Style it compactly**

```css
.next-ask {
  border-radius: 24px;
  padding: 16px 18px;
  background: linear-gradient(135deg, rgba(34,211,238,.14), rgba(99,102,241,.12));
  border: 1px solid rgba(125,211,252,.24);
}
.next-ask h2 {
  margin: 0 0 7px;
  letter-spacing: -.04em;
}
.next-ask p {
  margin: 0;
  color: #dbeafe;
  font-weight: 750;
  line-height: 1.4;
}
```

**Step 3: Update assertions**

Assert:

```js
"What real access would unlock",
"read-only sample exports",
"field dictionaries",
"BI query inventory"
```

**Step 4: Verify**

```bash
npm --workspace @pet-resort/staff-web run test
npm --workspace @pet-resort/staff-web run build
```

Expected: pass.

**Step 5: Commit**

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css apps/staff-web/smoke/staff-dashboard-mvp.test.mjs
git commit -m "feat: add safe next ask to demo"
```

---

### Task 8: Create a 3–5 minute demo script

**Objective:** Give the user a ready-to-say track for the job contact.

**Files:**
- Create: `docs/presentation/intelligible-manager-brief-demo-script.md`

**Step 1: Write the script**

Use this structure:

```markdown
# Intelligible Manager Brief Demo Script

## 15-second setup

“Because I did not have live NVA or Gingr access, I did not fake production integration. I built the safe seam first: a synthetic manager brief that turns messy resort signals into reviewed actions while keeping live side effects blocked.”

## Minute 1: The pain

Point to the morning chaos strip. Explain that pet-resort managers lose time gathering context across reservation data, staff notes, documents, capacity, and labor plans.

## Minute 2: The transformation

Click through messy morning → facts tracked → manager brief. Explain source refs, caveats, review gates, and why the system does not blindly trust source fields.

## Minute 3: The safe action plan

Point to the ranked actions. Call out what is review-ready and what is blocked. Emphasize that blocked is a feature, not a weakness.

## Minute 4: Proof and next ask

Open the proof drawer. Mention staff-web smoke tests and the local API side-effect-disabled proof. Close with the next ask: read-only sample exports, field dictionary, and BI query inventory for one workflow.

## If they ask “is this real?”

“It is real as a local/synthetic proof of the product boundary and workflow contract. It is not a live NVA/Gingr integration. That is intentional: the next safe step is read-only validation before any writeback or customer-facing action.”

## If they ask “why would NVA care?”

“Because this moves BI and operations away from raw provider-shaped cleanup and toward reviewed, source-backed workflow outcomes: what staff should do, what is blocked, what saved time, and what evidence supports it.”
```

**Step 2: Link it from the executive brief or checklist**

Modify `docs/presentation/nva-demo-executive-brief.md` quick links to include:

```markdown
- [Intelligible manager brief demo script](intelligible-manager-brief-demo-script.md)
```

**Step 3: Verify links**

```bash
python scripts/check_markdown_links.py --repo-root .
```

Expected: pass.

**Step 4: Commit**

```bash
git add docs/presentation/intelligible-manager-brief-demo-script.md docs/presentation/nva-demo-executive-brief.md
git commit -m "docs: add manager brief demo talk track"
```

---

### Task 9: Add visual QA checks for “intelligible, not pitchy”

**Objective:** Encode the product-story bar in tests so future edits do not regress.

**Files:**
- Modify: `apps/staff-web/smoke/staff-dashboard-mvp.test.mjs`

**Step 1: Add a regression test for forbidden first-impression copy**

Add:

```js
test("demo stays product-first instead of architecture-first", () => {
  for (const forbidden of [
    "owned backend migration map",
    "architecture story",
    "DTO",
    "read model replacement strategy",
    "technical proof first"
  ]) {
    assert.doesNotMatch(page, new RegExp(forbidden.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }
});
```

**Step 2: Add a regression test for explanatory anchors**

Add:

```js
test("demo explains the before after safety and next ask", () => {
  for (const expected of [
    "Before the brief",
    "messy morning",
    "manager brief",
    "review recorded",
    "Built without live access",
    "What real access would unlock"
  ]) {
    assert.match(page, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i"));
  }
});
```

**Step 3: Verify**

```bash
npm --workspace @pet-resort/staff-web run test
```

Expected: pass.

**Step 4: Commit**

```bash
git add apps/staff-web/smoke/staff-dashboard-mvp.test.mjs
git commit -m "test: guard manager brief demo intelligibility"
```

---

### Task 10: Run full local verification and capture fresh screenshot

**Objective:** Prove the demo is actually usable before telling the user it is ready.

**Files:**
- No source changes expected unless QA finds issues.
- Screenshot artifact path can remain outside git, e.g. `/tmp/nva-intelligible-manager-brief-demo.png`.

**Step 1: Run gates**

```bash
npm --workspace @pet-resort/staff-web run test
npm --workspace @pet-resort/staff-web run build
./scripts/demo_owned_operations_api.sh
python scripts/check_markdown_links.py --repo-root .
```

Expected:

- staff-web test passes;
- Next build passes;
- local API demo reports side effects disabled / fixture-only success;
- markdown links pass.

**Step 2: Start local web server**

```bash
npm --workspace @pet-resort/staff-web run dev
```

Use a tracked background process if running via Hermes.

**Step 3: Browser QA**

Open the page and verify:

- the first screen fits on a normal laptop viewport;
- the morning chaos strip is visible;
- step buttons visibly change emphasis;
- action badges are readable;
- no-access boundary is visible;
- proof drawer does not dominate first impression;
- next ask is visible after scrolling;
- no overflow or unreadably small text.

**Step 4: Capture screenshot**

Save a screenshot at:

```text
/tmp/nva-intelligible-manager-brief-demo.png
```

**Step 5: Commit any fixes**

If browser QA requires layout fixes:

```bash
git add apps/staff-web/app/page.tsx apps/staff-web/app/globals.css apps/staff-web/smoke/staff-dashboard-mvp.test.mjs

git commit -m "fix: polish manager brief demo readability"
```

---

## Final readiness checklist

Before calling this “ready to present,” verify:

- [ ] A non-technical viewer can explain the scenario in one sentence.
- [ ] The first 30 seconds show a workflow, not architecture.
- [ ] The demo has a clear start/middle/end.
- [ ] At least one action is review-ready.
- [ ] At least one action is blocked.
- [ ] No live-access claims are made.
- [ ] No unsafe side effects are implied.
- [ ] The proof drawer points to local/runtime evidence.
- [ ] The next ask is read-only and narrow.
- [ ] `npm --workspace @pet-resort/staff-web run test` passes.
- [ ] `npm --workspace @pet-resort/staff-web run build` passes.
- [ ] `./scripts/demo_owned_operations_api.sh` passes.
- [ ] `python scripts/check_markdown_links.py --repo-root .` passes.
- [ ] Fresh screenshot captured.

## Suggested execution order

This can be implemented as one focused pass, but keep commits separate:

1. acceptance contract;
2. human step labels;
3. morning chaos strip;
4. action status badges;
5. no-access boundary;
6. proof drawer;
7. next ask;
8. talk track;
9. regression tests;
10. full QA and screenshot.

## Presentation verdict after this plan is complete

If all tasks pass, the repo should be safe to present as:

> “A job-contact-ready, access-constrained product demo showing how NVA could turn messy pet-resort source signals into a review-gated manager brief, with synthetic data, blocked live side effects, local proof, and a clear read-only validation ask.”

Do **not** present it as:

- a live NVA integration;
- a live Gingr integration;
- production ROI proof;
- a writeback-ready product;
- a medical/safety/payment/scheduling automation system.
