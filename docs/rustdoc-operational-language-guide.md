# Rustdoc operational-language rewrite guide

Glossary bridge: when Rustdoc text introduces repo terms for non-coders, prefer operational links to [`domain`](glossary-architecture-terms.md#domain), [`app`](glossary-architecture-terms.md#app), [`storage`](glossary-architecture-terms.md#storage), [DTO](glossary-architecture-terms.md#dto), [workflow packet](glossary-workflow-state-terms.md#workflow-packet), [review gate](glossary-workflow-state-terms.md#review-gate), [blocked action](glossary-workflow-state-terms.md#blocked-action), [outcome capture](glossary-workflow-state-terms.md#outcome-capture), and the draft [Rustdoc/source link](design/core-entity-linked-glossary-draft.md#rustdoc-source-link) glossary entry. Do not invent rendered Rustdoc URLs; link source/module paths when generated docs are unavailable.

Purpose: help Rustdoc cleanup workers replace generated/template-like comments with English that a resort operator can use: what decision the value supports, what source fact it records, and what action remains review-gated.

Primary cleanup targets from the scan are `domain/src/training/mod.rs`, `domain/src/retail/*`, `domain/src/grooming/mod.rs`, boarding/daycare domain docs, and Gingr source-mapping adapters.

Use this guide for the rustdoc filler cleanup pass only. Do not broaden the rewrite into behavior changes: comments should clarify existing code, not create new product authority.

## Rewrite rules

1. Name the resort workflow before the software shape. Say "grooming rebooking", "trainer progress report", "daycare group-play eligibility", or "Gingr request parameter" before saying contract, boundary, value, or signal.
2. Replace "contract" with the business promise it protects: review packet, scheduling estimate, deposit decision, source mapping, checkout attachment, package balance, or safe draft.
3. Replace "boundary" with the specific crossing: provider input, staff-entered value, app workflow packet, storage projection, or live-action gate.
4. Replace "validated scalar" with the bad state prevented: zero-minute appointment, zero-session package, empty note, impossible count, invalid date, unsafe limit.
5. Delete "operational signal" unless the sentence names who acts on it and how. Most enum variants should describe the queue, decision, or staff-facing label directly.
6. For source-derived fields, say what the source fact explains and what it does not authorize. Example: a no-show count can explain rebooking risk; it cannot send a customer message, charge a deposit, or move an appointment by itself.
7. For getters, describe why a caller needs the value, not that it "returns evidence recorded on this contract." If the getter is purely mechanical and the type already explains the invariant, a short plain comment may be enough.
8. Provider/Gingr docs stay provider-specific. Do not turn Gingr vocabulary into domain policy; say when a value is only a request parameter, response field, DTO mapping, or source evidence.
9. Keep repeated macro-generated comments concrete enough for every expansion. If a macro cannot generate a useful sentence for all callers, use the narrowest true invariant.
10. Acceptance standard: a non-coding manager, front desk worker, trainer, groomer, or kennel lead should understand what the comment helps protect without reading the Rust implementation.

## Bad → better examples from the current repository

### 1. Grooming no-show history

Current pattern in `domain/src/grooming/mod.rs`:

> Source-derived no shows carried by this grooming contract.

Better:

> Grooming no-show count used to decide whether a rebooking candidate needs a deposit or manager review; the count explains risk but does not authorize an automatic charge, customer send, or appointment change.

Why: names the rebooking/deposit decision, source authority, and live-action limits.

### 2. Grooming late-cancel history

Current pattern in `domain/src/grooming/mod.rs`:

> Source-derived late cancels carried by this grooming contract.

Better:

> Late-cancel count considered with no-shows when staff review repeat grooming rebooking risk before offering another slot.

Why: avoids generic "source-derived"/"contract" language and explains how the field participates in the same operational decision.

### 3. Grooming no-show decision variants

Current pattern in `domain/src/grooming/mod.rs`:

> Require deposit for rebooking grooming operational signal for schedule, estimate, history, or review handling.

Better:

> Staff may prepare a rebooking path only after the required grooming deposit is reviewed or collected according to local policy.

Why: explains what the state means for front desk/grooming staff instead of labeling it an "operational signal."

### 4. Grooming duration getter

Current pattern in `domain/src/grooming/mod.rs`:

> Returns the minutes evidence recorded on this grooming contract.

Better:

> Minutes reserved for groomer-calendar planning and checkout labor estimates.

Why: says why callers need the number and removes "evidence recorded on this contract" boilerplate.

### 5. Training milestone status

Current pattern in `domain/src/training/mod.rs`:

> Not started training operational signal for enrollment, curriculum, progress, package, or follow-up handling.

Better:

> Parent-facing progress reports can say this milestone has not begun yet; trainer notes still decide what work should start next.

Why: names the parent report and trainer decision instead of repeating a generic service-line phrase.

### 6. Training package/session count

Current pattern in `domain/src/training/mod.rs`:

> Exposes the validated scalar for serialization and adapter boundaries.

Better:

> Session count used in package balances, trainer scheduling, and customer summaries after zero-session packages have been rejected.

Why: explains the invariant and why a nonzero count matters to training operations.

### 7. Training readiness gates

Current pattern in `domain/src/training/mod.rs`:

> Source-derived gate carried by this training contract.

Better:

> Trainer review must clear before assignment when enrollment facts are present but program fit or trainer capacity still needs a human decision.

Use variants to specialize the sentence:

- `BehaviorOrCareReviewRequired`: behavior/care review must clear before a training assignment or parent claim is drafted.
- `PackageOrPaymentReviewRequired`: payment/package review must clear before the app treats sessions as usable.

Why: each gate should tell staff what is blocked and who must review it.

### 8. Retail POS sale source

Current pattern in `domain/src/retail/pos.rs`:

> Integrated with reservation checkout retail operational signal for inventory, POS, reorder, recommendation, or review handling.

Better:

> Sale originated during reservation checkout, so inventory and payment review should stay tied to the stay being closed.

Why: identifies the source of the sale and why operations care.

### 9. Retail recommendation care sensitivity

Current pattern in `domain/src/retail/recommendation.rs`:

> Supplement or diet review required retail operational signal for inventory, POS, reorder, recommendation, or review handling.

Better:

> Care review is required before a supplement or diet recommendation can appear in customer copy.

Why: tells retail/front-desk staff exactly what action is blocked.

### 10. Boarding/daycare positive quantities

Current pattern in `domain/src/boarding/mod.rs` and `domain/src/daycare/mod.rs`:

> Promotes boundary input into a validated boarding/daycare domain value.

Better for boarding:

> Rejects impossible boarding quantities before they affect capacity, minimum-stay, deposit, or checkout calculations.

Better for daycare:

> Rejects impossible daycare counts before they affect group-play capacity, ratio checks, or eligibility queues.

Why: names the bad state prevented and the service-line calculations protected.

### 11. Daycare playgroup ID

Current pattern in `domain/src/daycare/assignment.rs`:

> Playgroup identifier boundary for daycare assignment contracts.

Better:

> Identifier for the playgroup a daycare lead is reviewing or assigning; it should come from the scheduling/source system, not a generated guess.

Why: speaks to assignment work and source trust rather than generic boundaries/contracts.

### 12. Gingr request/response mapping

Current pattern in `integrations/gingr/src/endpoint/mod.rs`:

> Describes the provider wire contract for this Gingr request.

Better:

> Returns the Gingr HTTP method/path/parameters this adapter will send, keeping provider request details explicit before they become source evidence for app workflows.

Current pattern in `integrations/gingr/src/endpoint/mod.rs`:

> Original provider/caller value rejected before it could become a typed boundary value.

Better:

> Original Gingr/caller value rejected before the adapter can use it in a provider request.

Why: Gingr docs should explain provider request mechanics and source-evidence limits, not imply domain authority.

## Service-line rewrite anchors

Grooming:
- Use words like duration estimate, groomer calendar, style notes/photos, no-show history, deposit review, rebooking prompt, exit bath, add-on, and customer-message draft.
- Always preserve gates around booking changes, customer sends, deposits/waivers, medical/handling interpretation, and provider-calendar assignment.

Training:
- Use enrollment readiness, trainer capacity, curriculum unit, milestone status, package/session balance, parent-facing progress report, graduation/follow-up, and trainer handoff.
- Never imply automation can make outcome claims or trainer assignments without the relevant trainer/manager/payment/customer-message gate.

Boarding/daycare:
- Boarding anchors: room/suite capacity, holiday minimum stay, deposit state, open stay, checkout exception, care/feeding/medication note, and handoff queue.
- Daycare anchors: group-play eligibility, temperament/behavior evidence, ratio/capacity, playgroup assignment, incident/escalation flag, and check-in readiness.
- Preserve review gates for safety, medical/vaccine decisions, incident interpretation, and any live schedule/PMS change.

Retail/payment:
- Use inventory availability, SKU/product, recommendation rationale, prior purchase, checkout attachment, discount/comp/refund reason, manager approval, reorder threshold, and vendor notice.
- Block customer copy, payment movement, refunds/discounts, and care-sensitive product claims until the right approval is present.

Gingr/source mapping:
- Use provider request, response envelope, DTO field, raw provider value, mapping rule, source evidence, and read-model projection.
- State when a value remains a Gingr/provider fact and what must promote it before domain/app workflows treat it as business policy.

## Acceptance checks for rewrite workers

Before completing a Rustdoc batch, check every changed comment against these questions:

- Does the first sentence name a concrete resort decision, queue, invariant, or source-mapping concern?
- Would the sentence still make sense if pasted into ten unrelated modules? If yes, rewrite it.
- Did you remove or specialize generic terms: contract, boundary, validated scalar, operational signal, source-derived?
- For source fields, did you say what the source fact can explain and what it cannot authorize?
- For enum variants, did you explain the staff-facing state or blocked action instead of restating the variant name?
- For constructors and newtypes, did you name the impossible/unsafe value being rejected?
- For getters/accessors, did you name the caller workflow that needs the value?
- For Gingr/integration docs, did you keep provider mechanics separate from NVA domain policy?
- Did you avoid changing behavior, identifiers, tests, or broad prose outside the assigned Rustdoc scope?
- Can a manager/front desk worker/trainer/groomer/kennel lead understand why the documented item exists without knowing Rust?
