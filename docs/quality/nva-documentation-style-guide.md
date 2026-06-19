# NVA documentation style guide

Purpose: make future README, Rustdoc, and operator-page edits reduce pet-resort labor cost without inventing authority. This is a practical checklist for repo editors and Kanban workers, not a general writing essay.

Use this guide before editing any NVA documentation surface. If a page cannot answer who saves time, what source backs the claim, what remains review-gated, and where the compiled contract lives, it is not ready.

## Quoteable rule: lead with labor saved

Every new page or major section must start by naming the staff/operator time or error cost it helps reduce.

Checklist:

- Name the worker: front desk, general manager, regional ops leader, groomer, trainer, kennel/daycare lead, or engineering maintainer.
- Name the costly work: dashboard reconciliation, phone/inbox repeat questions, checkout exception audits, grooming rebooking search, duplicate-record cleanup, stale vaccine evidence review, handoff note rewriting, or policy lookup.
- Name the safe outcome: ranked queue, reviewed draft, source-backed summary, deterministic policy check, outcome record, or navigation to the source contract.
- Do not open with crate inventory, generic AI platform language, or abstract module/API phrasing.

Preferred opening shape:

> This page helps a general manager avoid morning demand-versus-staffing spreadsheet reconciliation by showing which source-backed actions the Manager Daily Brief ranks, what the app can draft, what a human must approve, and where outcomes record actual minutes saved.

Avoid:

> This module exposes operational signals carried by the contract boundary.

## Quoteable rule: operator English before implementation detail

Put a non-coder explanation before Rust paths, DTO names, builders, endpoints, or module ownership maps.

Order for README/operator Markdown:

1. Plain-English resort workflow: what staff are trying to do.
2. Labor/error cost: what repeated work or mistake the doc helps prevent.
3. Source facts required: which provider/read-model/policy facts are needed.
4. Agent/app boundary: what can be drafted, ranked, validated, or recorded.
5. Human approval boundary: what cannot happen automatically.
6. Code/Rustdoc links: where the compiled contracts and examples live.

Order for Rustdoc:

1. Concrete invariant or workflow decision the item protects.
2. Construction/usage contract, preferably with a compile-checked doctest when executable.
3. Source/review boundary if the type influences agent packets, provider mapping, or live actions.

If the first paragraph would not help a non-coding resort operator understand the reason the surface exists, rewrite it before adding implementation detail.

## Quoteable rule: use pet-resort examples before generic API phrasing

Every explanation should include at least one concrete resort workflow before generic module language.

Good example anchors:

- Boarding: checkout exception, open stay, suite/capacity mismatch, holiday minimum stay.
- Daycare: eligibility, temperament, staff-to-pet ratio, incident/escalation flag.
- Grooming: no-show, rebooking cadence, duration estimate, add-on or exit bath opportunity.
- Training: progress package, session count, trainer handoff, graduation/follow-up task.
- Retail/payment: inventory recommendation, payment/refund/discount boundary.
- Source hygiene: duplicate customer/pet candidate, stale vaccine evidence, missing profile field, ambiguous service-line name.
- Manager/regional ops: demand-versus-staffing risk, portfolio exception, outcome minutes.

Anti-pattern:

> Source-derived no shows carried by this grooming contract.

Better pattern:

> Grooming no-show counts help staff decide whether a rebooking candidate needs manager review before outreach; the source count may explain risk, but it does not authorize an automatic customer send or schedule change.

Anti-pattern:

> Promotes boundary input into a validated domain value.

Better pattern:

> A positive grooming-duration estimate prevents a zero-minute service from corrupting schedule, labor, and revenue math before the value reaches app workflows or storage records.

## Source authority and safety boundaries

Documentation must say what is authoritative, what is illustrative, and what remains a question.

Authoritative/source-of-truth surfaces:

- `domain` source and Rustdoc: semantic business truth, invariants, policy/review/source contracts.
- `app` source and Rustdoc: use-case workflow packets, draft/review boundaries, tool-port contracts, outcome capture.
- `integrations/gingr` source and Rustdoc: Gingr/provider request, response, DTO, endpoint, and mapping facts. These are provider evidence, not domain truth.
- `storage` source and Rustdoc: durable projection records, stable codes, codecs, and explicit conversion to/from domain values.
- `nva-pet-resorts-ai-context.md`: repo-level acceptance lens and original NVA operating context, including public/inferred context boundaries.
- `docs/design/labor-cost-reduction-crosswalk.md`: product/architecture crosswalk and guardrails for labor loops.

Derivative/navigation surfaces:

- Root and crate READMEs: maintainer wiki, navigation, ownership map, labor-cost framing, and links to authoritative source/Rustdoc.
- Operator-facing Markdown/workflow pages: plain-English explanations and source-backed summaries, not new behavioral authority.
- Public landing/glossary pages: audience translation, not replacements for source/Rustdoc contracts.
- Kanban comments/review cards: active acceptance context for current work, not permanent product authority unless copied into a reviewed repo artifact.

Required safety language when relevant:

- Agent loops may draft, summarize, rank, classify, validate, or record outcomes only when the app contract allows it.
- No live customer send, PMS/provider write, schedule mutation, payment/refund/discount movement, medical/vaccine/safety decision, personnel action, or secret-dependent action is authorized by prose alone.
- If source packets/fixtures do not prove a fact, write it as a discovery question or future source/read-model need.
- Keep provider facts distinct from domain truths. “Gingr says X” is not the same as “the business policy is X.”

## When to link Rustdoc vs duplicate explanation in Markdown

Markdown is for orientation. Rustdoc is for compiled API contracts.

Link Rustdoc/source from Markdown when:

- The reader needs constructor names, field names, enum variants, module ownership, or exact API behavior.
- A code example should be compile-checked with `cargo test --doc`.
- The explanation could drift if copied into multiple README/operator pages.
- The page is a wiki/navigation surface whose job is to send maintainers to the authoritative contract.

Duplicate explanation in Markdown only when:

- The audience needs a plain-English operator summary before code detail.
- The text explains labor cost, safety boundary, source authority, or human workflow rather than API mechanics.
- A small conceptual sketch helps non-coders, and it is explicitly marked conceptual/non-executable.
- The duplicate is short, source-grounded, and links back to the authoritative source/Rustdoc.

Do not paste executable Rust snippets into README/operator pages unless there is a specific reason they cannot live in Rustdoc. If a Markdown sketch is not compile-checked, label it “conceptual sketch” and avoid exact API claims that can rot.

## README and operator-page checklist

Before submitting a README or operator Markdown edit, verify:

- Labor saved: the opening names whose time/error cost is reduced.
- Plain English first: a non-coder can understand the workflow before module paths appear.
- Pet-resort example: at least one concrete boarding/daycare/grooming/training/retail/source-hygiene/manager example appears before generic API wording.
- Source facts: the page says which system, read model, fixture, Rust type, or policy source backs the claim.
- Human approval: live/member-facing or business-sensitive actions are explicitly blocked or review-gated.
- Outcome measurement: labor-saving claims mention outcome capture or distinguish measurable minutes from product intent.
- Authority: Markdown summarizes and links; it does not create new behavior that source/Rustdoc lacks.
- Link hygiene: local links point to existing files; broad navigation changes are minimal and intentional.

## Rustdoc checklist

Before submitting a Rustdoc edit, verify:

- The doc explains the resort decision or invariant, not just that a type is a boundary/contract/signal.
- The doc names the concrete bad state prevented: zero duration, missing source evidence, ambiguous service-line name, unsafe customer send, provider write, payment movement, stale vaccine evidence, etc.
- Repeated field docs are specialized enough that they would not make equal sense on any other field.
- Source-derived fields say what they can explain and what they cannot authorize.
- Agent-facing structs name the review packet, draft, or outcome they support.
- Provider DTO/endpoint docs stay provider-specific and do not promote provider vocabulary into domain policy.
- Executable examples live in Rustdoc/doctests when useful; Markdown links to them.

### Filler Rustdoc replacement contract

Use this acceptance rubric before replacing any vague Rustdoc. A replacement is acceptable only when it uses entity-specific operational English and passes these checks:

- Entity named: the first sentence names the concrete entity, value, packet, DTO, endpoint, workflow, or field being documented, not a reusable phrase like “operational signal,” “boundary input,” “validated scalar,” or “contract type.”
- At least one operational role: the text covers at least one entity-specific reason the item exists: purpose, relationship to adjacent entities, authority/source of truth, workflow or contract participation, automation recommendation boundary, human-review/safety boundary, labor-cost value, or source/Rustdoc/test evidence.
- Source and authority clear: source-derived values say what source evidence can explain, which domain/app/storage/integration surface is authoritative, and whether unanswered facts remain discovery questions.
- Automation boundary clear: if the item influences agent packets, rankings, drafts, live customer/provider/payment/schedule/safety work, or secret-dependent actions, the doc names what can be drafted/recommended/recorded and what stays blocked or human-reviewed.
- Evidence grounded: the wording is supported by nearby source, existing Rustdoc, fixtures, tests, README/operator docs, or explicit source gaps; it does not invent product authority to make prose sound complete.
- Not portable boilerplate: if the sentence could be pasted unchanged onto an unrelated module, field, or crate, it is still filler.

Unacceptable generic replacements:

> Represents a source-derived operational signal for downstream consumers.

> Validated boundary input used by the application contract.

> Stores metadata for automation workflows.

Acceptable entity-specific replacements:

> A grooming rebooking candidate keeps no-show history, last service date, and source evidence together so staff can decide whether outreach is worth manager review. It may explain why a pet appears in the queue, but it does not authorize an automatic customer message or calendar change.

> A positive grooming duration estimate prevents a zero-minute bath, trim, or exit service from corrupting staffing, room capacity, and revenue math before the value reaches scheduling or outcome records.

> A Gingr retail endpoint response is provider evidence for inventory and checkout review; mapping code may normalize it into storage projections, but domain policy decides whether a recommendation is safe to show and humans still approve refunds, discounts, and provider writes.

Compact review quote:

> Before replacing filler Rustdoc, require entity-specific operational English: name the entity, one concrete purpose/relationship/authority/workflow/automation/human-review/labor/evidence role, source authority, blocked live actions when relevant, and nearby code/Rustdoc/test evidence. Reject text that could fit ten unrelated items.

## Anti-slop rejection checks

Reject or rewrite documentation that fails any of these tests:

- Could this sentence apply unchanged to ten unrelated modules? If yes, it is boilerplate.
- Does it say “operational signal,” “source-derived,” “contract,” “boundary input,” or “validated scalar” without naming a resort decision? If yes, make it concrete.
- Does it claim labor savings without a measured outcome, queue, or avoided manual task? If yes, qualify it.
- Does it imply the agent can act live when the app only drafts or ranks? If yes, add the blocked action and human approval boundary.
- Does it treat a README/operator page as authoritative for behavior not present in source/Rustdoc? If yes, move authority to source or mark the Markdown as illustrative.
- Does it turn unanswered source-of-record questions into facts? If yes, restore question language.
- Does it lead with implementation names before saying why a resort operator cares? If yes, reorder.

## Kanban docs-card prompt block

Use this block when creating or assigning future README, Rustdoc, operator-page, glossary, or public-doc rewrite cards. It is intentionally a prompt/checklist, not a mandatory repo-wide gate for unrelated engineering work.

Before editing, future docs workers should naturally read:

1. The assignment card, including any source paths or acceptance notes from parent cards.
2. The [root README documentation contracts](../../README.md#documentation-contracts) for the repo-wide README/Rustdoc split.
3. This guide's checklist and examples, especially this prompt block.
4. The closest local README, Rustdoc/source module, operator page, or fixture that is authoritative for the page being changed.

Copy this self-contained block into future documentation cards:

> Follow `docs/quality/nva-documentation-style-guide.md` for this docs edit. Lead with the labor or error cost reduced for a specific pet-resort role; use operator English before module/API detail; include a concrete boarding/daycare/grooming/training/retail/source-hygiene/manager example; name the source fact or Rustdoc/code evidence behind each behavioral claim; state the human approval boundary for customer/provider/payment/schedule/safety actions; keep Markdown as orientation while executable API details live in Rustdoc or source; and avoid turning examples, provider payloads, or open discovery questions into business authority. In the handoff, report the changed files and the docs check you ran, usually `./scripts/check_markdown_links.py` for Markdown-only link changes or `./scripts/check_docs.sh` when Rustdoc/source links or executable docs changed.

Compact review quote:

> For every changed paragraph, check: labor saved, operator English, pet-resort example, source fact, human approval boundary, outcome measured, Rustdoc/code evidence.

## Reusable before/after examples

Keep examples short enough to paste into Kanban prompts. Replace the workflow name, source path, and blocked action, but preserve the pattern: labor cost first, source authority clear, and live actions review-gated.

Copyable review prompt:

> For each changed paragraph, provide one before/after rewrite if it starts with generic implementation language, duplicates Rust internals in Markdown, or claims authority without saying whether the value is authoritative, derived, illustrative, or still a discovery question.

### Templated Rustdoc rewrite: generic API wording to pet-resort framing

Before:

> Represents a source-derived boundary input for the grooming workflow. Consumers should use this type when promoting provider data into the application contract.

After:

> A grooming rebooking candidate keeps no-show history, last service date, and source evidence together so staff can decide whether outreach is worth manager review. It may explain why a pet appears in the queue, but it does not authorize an automatic customer message or calendar change.

Before:

> Validated duration scalar for service scheduling.

After:

> A positive grooming duration estimate prevents a zero-minute bath, trim, or exit service from corrupting staffing, room capacity, and revenue math before the value reaches scheduling or outcome records.

Prompt shape:

> Rewrite this Rustdoc so the first sentence names the resort decision or bad state prevented. Avoid generic “boundary input/validated scalar/source-derived signal” wording unless it is tied to grooming, boarding, daycare, training, retail, or source-hygiene work.

### README/operator prose: labor cost before technical mechanism

Before:

> The Manager Daily Brief composes read-model packets from storage projections and app tools, then emits ranked operational signals for downstream consumers.

After:

> This page helps a general manager avoid morning demand-versus-staffing spreadsheet reconciliation. The Manager Daily Brief ranks boarding capacity, daycare eligibility, grooming follow-up, and checkout exceptions from source-backed read models; it can draft the review queue and record outcomes, but a human still approves schedule, staffing, customer, and payment actions.

Before:

> The source-hygiene workflow normalizes profile fields and exposes duplicate candidates through the app layer.

After:

> This page helps front desk staff avoid retyping the same customer/pet cleanup notes across systems. The source-hygiene workflow can show duplicate candidates, stale vaccine evidence, and missing profile fields from provider evidence; it cannot merge records, decide vaccine compliance, or overwrite provider data without the approved human workflow.

Prompt shape:

> Start the README/operator section with whose time is saved and which repeated manual task is reduced. Explain the Rust/app/storage mechanism only after the operator workflow and review boundary are clear.

### Source-authority and safety-boundary language

Use “authoritative” when the repo artifact is the source of behavior:

> Authoritative: `domain/src/grooming` defines the grooming scheduling and follow-up invariants. README text may summarize them, but behavior changes belong in source/Rustdoc and tests.

Use “derived” when a value comes from source evidence or a read model but is not itself policy:

> Derived: a Gingr no-show count can help explain why a grooming rebooking candidate was ranked. It is evidence for review, not permission to send outreach or change an appointment.

Use “illustrative” when numbers or scenarios teach the workflow but are not current production facts:

> Illustrative: “three checkout exceptions every Monday” is an example of the labor loop this page protects. Replace it with measured outcome data before claiming current resort savings.

Use “question/future source need” when the repo does not yet prove a fact:

> Discovery question: this page needs a provider-backed source for holiday minimum-stay policy before it can say whether a boarding exception is policy-backed or only operator-reported.

Safety boundary snippet:

> The agent may draft, rank, summarize, validate, or record outcomes only where the app contract allows it. Prose does not authorize live customer sends, PMS/provider writes, schedule mutations, payment/refund/discount movement, vaccine/safety decisions, personnel actions, or secret-dependent actions.

### Link Rustdoc instead of duplicating internals in Markdown

Before:

> `GroomingRebookingCandidate::new` accepts a pet id, customer id, last visit date, no-show count, and source evidence, then returns an error when required evidence is missing. The enum variants are `NeedsManagerReview`, `ReadyForDraft`, and `Blocked`.

After:

> Grooming rebooking queues help staff find missed follow-up revenue without sending unreviewed customer messages. Keep constructor details, variants, and compile-checked examples in [`domain/src/grooming`](../../domain/src/grooming/README.md) and Rustdoc; this README should explain the labor loop and safety boundary.

Use this when the audience is maintainers and the details can drift. Markdown should link to the compiled contract instead of copying constructor names, enum variants, DTO fields, or doctest examples.

### Duplicate a short operational explanation instead of forcing non-coders into Rustdoc

Before:

> See `app::booking_triage` Rustdoc for how booking triage packets are ranked.

After:

> Booking triage reduces phone and inbox churn by putting source-backed boarding capacity, daycare eligibility, and policy exceptions into one review queue. The app can rank and explain the queue; staff still approve customer communication, schedule changes, and provider writes. Maintainers can use `app::booking_triage` Rustdoc for packet fields and testable contracts.

Use this when the reader is an operator, reviewer, or public-docs editor who needs the labor loop and approval boundary before any code path is useful.

## Page template

Use this shape for new or heavily revised Markdown pages:

```markdown
# <workflow or surface name>

Purpose: help <worker/role> reduce <manual work/error cost> by <safe app/agent/doc outcome>.

## Operator workflow

Plain-English description of the resort workflow and concrete example.

## Source facts and authority

Which source systems, fixtures, read models, domain/app/storage/integration contracts, or policy docs back the page. Mark future/discovery questions explicitly.

## Safe automation boundary

What the app/agent can draft, rank, validate, summarize, or record. What remains human-approved or blocked.

## Where the compiled contract lives

Links to crate/module Rustdoc/source and any doctests. Keep executable API details there.

## Editor checklist

Labor saved; operator English; concrete example; source fact; human approval; outcome measurement; Rustdoc/code evidence.
```

## Minimal acceptance standard for future Kanban docs cards

A docs card is ready for review when it can point to:

- changed files limited to the requested artifact/scope;
- the labor-cost opening added or preserved;
- at least one concrete pet-resort example before implementation detail;
- explicit authoritative-vs-illustrative source language;
- explicit blocked live actions where the page touches customer/provider/payment/schedule/safety surfaces;
- a Rustdoc-vs-Markdown decision for any code/API explanation;
- local link check or a clear reason it was not run.
