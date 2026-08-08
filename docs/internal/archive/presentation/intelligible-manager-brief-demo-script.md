# Intelligible Manager Brief Demo Script

Audience: job contact, resort operator, front-desk lead, or product/BI stakeholder.
Length: 3-5 minutes.
Goal: make the workflow obvious before discussing architecture.

## 15-second setup: no-access honesty

"I want to start with the boundary: I did not have live NVA or Gingr access, so this does not claim production data, live customer sends, provider/PMS writes, payment actions, schedule changes, or medical/safety decisioning.

What I built is the safe thing first: a synthetic, source-backed Manager Daily Brief that shows how messy pet-resort morning signals can become reviewed manager actions, with risky side effects blocked."

## Minute 1: the pain

"Start at the top of the screen. This is the morning chaos strip: 7:20am lobby rush, 12 arrivals before 10, rabies proof unclear, coverage two short, and a quiet-room request buried in a note.

That is the work problem. A manager or front-desk lead is not short on data; they are short on trusted context. The same morning can be spread across reservation records, staff notes, document attachments, capacity sheets, and labor plans.

Without a workflow layer, someone has to gather context manually, decide which source to trust, notice what is stale or unclear, and still avoid doing anything unsafe. That is where time disappears: not in one big task, but in repeated triage, re-checking, and report prep."

## Minute 2: the transformation

"Now follow the four steps.

First: messy morning. The demo starts with raw operational signals, not a clean dashboard.

Second: facts tracked. Each fact keeps source evidence attached: where it came from, what field or note supports it, how fresh it is, what caveat exists, and whether a manager or document review gate is required.

Third: manager brief. The system turns those tracked facts into a ranked action plan. It is not saying, 'trust the AI.' It is saying, 'here are the next things a manager can review, why they matter now, who owns them, and how much time this should save compared with manual context gathering.'

Fourth: review recorded. The proof is not just that a card changed color. The workflow records what was reviewed, what stayed blocked, and the estimated-versus-reviewed labor minutes. That is the beginning of outcome proof instead of another disconnected dashboard."

## Minute 3: safe action plan

"Look at the action statuses. This is where the demo is intentionally conservative.

Some actions are review ready. For example, clearing the rabies document is ready for front-desk or document review because the attachment exists but the expiration still needs a human check. The quiet-room plan is also review ready because it is based on a visible stay note.

Other actions remain blocked. Boarding versus labor is a capacity and coverage risk, but this local proof is not allowed to mutate schedules, capacity, PMS records, payments, or customer messages.

That is the safety rule I would want in a real NVA environment: source facts can become workflow packets; managers can review and record outcomes; unsafe side effects stay locked until an approved adapter, policy, and audit path exist."

## Minute 4: proof drawer and next ask

"Now open or point to the proof drawer. This is the technical credibility layer, but I would keep it secondary until the manager workflow makes sense.

The proof drawer shows this is synthetic/local only, that source refs and caveats are attached, that staff-web smoke tests cover the source-to-brief anchors, and that the local operations API proof keeps side effects disabled. It also points to the local demo script for the owned operations API.

The next ask is intentionally narrow: read-only sample exports or source snapshots, field dictionaries, and the current BI query inventory for one workflow. No live writes. No customer sends. No payment, schedule, capacity, or medical/safety actions.

With that read-only access, the next proof is not 'can we touch production?' It is: can this workflow map real source fields, preserve provenance, compare against current BI queries, and show a manager where labor and review burden are actually reduced?"

## Direct answer: "Is this real?"

"It is real as a local, synthetic product proof and contract shape. It is not a live NVA or Gingr integration, and I would not represent it that way.

The real parts are the workflow design, source/provenance model, review gates, side-effect-disabled API proof, smoke-covered UI anchors, and outcome/labor proof concept. The part that requires NVA access is validation against real exports, field dictionaries, and BI queries. That is why the next ask is read-only validation, not production credentials."

## Direct answer: "Why would NVA care?"

"NVA should care because a 170-location operation does not just need prettier dashboards. It needs repeatable workflow authority above provider-shaped data.

If messy source signals can become reviewable manager packets with provenance, caveats, safe gates, and labor-outcome records, then operations and BI get cleaner upstream answers. Managers spend less time gathering context and preparing manual reports. BI spends less time reverse-engineering business meaning from provider exports. Product gets a safer migration path: start read-only, prove one workflow, then expand only where review gates and audit controls are working.

That is the value of this demo: it shows judgment under access constraints and a practical path from source evidence to safer operations workflow."

## One-sentence close

"I built this to show the first useful seam: messy resort signals in, source-backed and review-gated Manager Daily Brief out, with proof of labor saved and risky live actions still blocked."
