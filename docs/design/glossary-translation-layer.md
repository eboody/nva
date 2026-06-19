# Glossary translation layer for non-coder NVA docs

Purpose: give public, non-coder readers a repeatable way to translate repo/Rust architecture vocabulary into NVA pet-resort operational meaning without flattening the code-derived contract. This is a design for the glossary shape, not the full glossary inventory.

The glossary should sit between two truths:

- Source/Rust truth: module paths, types, Rustdoc, READMEs, and tests describe what the system actually promises.
- Operator truth: resort leaders care about labor, safety, handoffs, evidence, review boundaries, customer trust, and measurable outcomes.

A glossary entry must connect those truths. It may explain what a term means in operating language, but it must not turn a code contract into a broader product claim.

## Audience model

Primary readers:

1. Resort leaders and operators who need to know what an automation surface can safely do.
2. Product/program stakeholders who need to map engineering terms to labor-cost, safety, and workflow outcomes.
3. Documentation readers who are not Rust developers but need enough precision to avoid overreading agent/app capabilities.

Secondary readers:

1. Maintainers writing public README/Rustdoc prose.
2. Reviewers checking whether docs preserve source evidence and authority boundaries.
3. Agents generating public documentation from repo terms.

Assumed knowledge:

- Readers understand boarding, daycare, grooming, training, retail, reservations, staff review, customer messaging, and manager approval.
- Readers may not understand Rust crates, modules, enums, typestate, DTOs, provenance, storage projections, or adapter boundaries.
- Readers should not need to read Rust syntax first, but each entry should point them back to the source surface that carries the actual contract.

## Entry template

Each glossary entry should use the same fields so reviewers can compare entries without rediscovering the rules.

```text
Term:
  The exact repo/Rust term, preserving module path when the path carries meaning.

Plain-language label:
  A short operator-facing phrase. This is a translation label, not a replacement for the term.

Audience:
  Who most needs this translation: operator, resort leader, product stakeholder, maintainer, reviewer, or agent-doc writer.

Where it appears:
  Source-backed links to crate/module READMEs, Rust source, Rustdoc item paths, tests, or design docs. Prefer specific source files over vague package names.

Code-derived contract:
  What the term actually promises in the repository. Use precise verbs: carries, validates, routes, records, gates, drafts, blocks, promotes, demotes, converts, evaluates.

Pet-resort operational meaning:
  How the contract maps to resort work: staff handoffs, source evidence, manager briefings, policy checks, customer-message drafts, pet safety review, payment/deposit exceptions, capacity/labor review, or outcome capture.

Why an operator should care:
  The labor/safety/trust reason the term matters. This should answer, “What could this prevent, speed up, or make auditable?”

What not to infer:
  Explicit anti-claims. Name the product behavior the code does not promise: no live mutation, no final medical approval, no invented availability, no local policy override, no customer send, no payment/refund action, no complete BI truth, etc.

Boundary and authority:
  Which layer owns the truth: `domain`, `app`, `storage`, `integrations/gingr`, or `apps/*`. Explain whether the term is domain truth, app workflow composition, storage projection, provider evidence, or runtime shell behavior.

Evidence and review hooks:
  Source refs, provenance, tests, Rustdoc examples, review gates, outcome records, or audit surfaces that let a reader verify the claim.

Suggested public wording:
  1-3 sentences suitable for a non-coder doc, with the Rust term retained in parentheses or path form.

Related terms:
  Links or names of adjacent terms that should be read together.
```

Optional fields for complex entries:

```text
Lifecycle position:
  Where the term sits in the flow from provider payload -> mapping -> domain truth -> app packet -> storage/outcome -> runtime shell.

Safe example:
  A source-grounded, non-executable sketch. Mark it conceptual unless it is copied from compile-checked Rustdoc.

Reviewer checklist:
  Entry-specific questions that catch overclaims.
```

## Tone rules

1. Lead with operations, then anchor to code. Start with what a resort operator can understand, but preserve the Rust term and source path before making claims.
2. Use “means in this repo” language. Avoid universal definitions of architecture terms; the glossary explains this repository’s contract.
3. Prefer boundary verbs over hype verbs. Use “drafts,” “routes,” “flags,” “records,” “evaluates,” “blocks,” “cites,” and “promotes” instead of “automates everything,” “decides,” or “optimizes” unless the source proves that exact behavior.
4. Keep the non-coder translation concrete. Tie terms to a manager looking at a brief, staff reviewing a behavior issue, a customer message draft waiting for approval, or a source record being repaired.
5. Treat uncertainty as part of the contract. If the code preserves evidence or requires review, say that; do not turn review gates into implied approvals.
6. Keep source-system names honest. Gingr/provider facts are evidence or boundary payloads until mapped/promoted; they are not automatically domain truth.
7. Do not hide semantic module paths when they carry meaning. A path such as `domain::source::Provenance` teaches ownership and authority better than a generic label like “data origin.”
8. Avoid implementation trivia unless it protects meaning. Mention Rust, enums, typestate, builders, DTOs, or storage projections only when those shapes explain a safety/authority boundary.

## Contract-preservation rules

Every entry must satisfy these rules:

1. Preserve the owning layer.
   - `domain` owns semantic truths and invariant-bearing values.
   - `app` owns use-case/workflow composition, packets, drafts, deterministic evaluations, and tool-port contracts.
   - `storage` owns persisted records/projections and explicit promotion/demotion with domain values.
   - `integrations/gingr` owns provider DTOs, endpoint vocabulary, transport seams, and source mapping evidence.
   - `apps/*` own runtime shells and wiring, not domain truth.

2. Preserve source-vs-derived boundaries.
   - Provider payloads, BI rows, and imported fields are evidence.
   - Domain values are normalized business concepts.
   - App packets are workflow artifacts built from source-grounded context.
   - Storage records are durable projections or outcome rows.

3. Preserve draft/review/live-action boundaries.
   - Agent or app workflows may draft, rank, summarize, route, and recommend when source supports it.
   - Live provider/PMS changes, customer sends, payment/refund actions, medical approvals, capacity overrides, and local-policy exceptions require explicit source-backed policy authority and review gates.

4. Preserve negative space.
   - Each entry must include “what not to infer.” This is not defensive boilerplate; it prevents docs from overstating product readiness.

5. Preserve evidence.
   - Cite source files, Rustdoc surfaces, README sections, tests, or design docs. Do not rely on prose-only memory.

6. Preserve path meaning.
   - If the module path distinguishes domain truth from storage/provider/runtime shape, keep the path in the entry and in suggested public wording.

7. Preserve testability.
   - If an entry describes an executable API example, link to Rustdoc/source where `cargo test --doc` can check it. If the example is conceptual, label it conceptual.

8. Do not fill product gaps with glossary prose.
   - If code only models a draft or review gate, the glossary must not imply a shipped UI, live integration, automated send, or deployed policy engine.

## How entries connect repo terms to operations

Use this translation path when writing an entry:

```text
Repo/Rust term
  -> owning layer and source file
  -> exact code-derived promise
  -> resort operation affected
  -> labor/safety/trust reason it matters
  -> authority boundary and what it does not do
  -> evidence hook for verification
```

For architecture terms, emphasize ownership and authority. Examples: `domain::*`, `app::*`, `storage::*`, `integrations::gingr::*`, DTO boundary, semantic promotion, source provenance, runtime shell.

For workflow terms, emphasize action legality and review. Examples: `policy::ReviewGate`, `workflow::RecommendedAction`, `manager_daily_brief::Workflow`, `SafeAgentAction`, `BlockedAction`, outcome record.

For source/data terms, emphasize evidence, lineage, and repair. Examples: `domain::source::Provenance`, `RecordRef`, `data_quality::Issue`, mapping candidates, provider fields.

For service-line terms, emphasize operational surfaces without inventing location policy. Examples: boarding capacity, daycare group-play eligibility, grooming rebooking, training progress, retail reorder signals.

## Reading guide: translating Rustdoc phrases into operating meaning

Use these worked examples when a non-coder sees a Rustdoc phrase or module path and needs to translate it without widening the code promise. Each example is conceptual unless a linked Rustdoc/source example is named; it shows how to read the phrase, why a resort operator should care, and what not to infer.

### Worked example: architecture path

Rustdoc phrase or path:
  `domain` values flow into `app` workflow packets and may later be projected by `storage`.

Read it as:
  Business meaning starts in `domain`, where source-backed pet-resort concepts, policies, provenance, and review vocabulary are named. `app` composes those meanings into a workflow/review bundle. `storage` can persist a projection or outcome record after explicit conversion, but it does not become the business truth.

Operational translation:
  A manager daily brief may use domain facts such as service demand, review gates, source refs, or labor-evidence values; the app layer packages those facts into a reviewable packet; storage can later keep outcome/labor evidence for reporting. This is a traceable handoff path, not a single magic automation layer.

Why it matters:
  Resort leaders can ask which layer owns each claim: `domain` for business vocabulary, `app` for reviewable workflow composition, `storage` for durable evidence/projections. That keeps reports, drafts, and database rows from being mistaken for live operational authority.

Do not infer:
  Do not infer that `domain` reads Gingr or stores rows, that `app` sends customer messages or changes schedules, or that `storage` decides policy. Live provider/PMS changes, customer sends, payment/refund movement, and local policy exceptions still require the appropriate system, shell, adapter, or human review authority.

Evidence hooks:
  See `domain/README.md`, `app/README.md`, `storage/README.md`, `app/src/manager_daily_brief.rs`, `storage/src/operations.rs`, and the draft entries in `docs/glossary-architecture-terms.md`.

### Worked example: workflow packet plus review gate

Rustdoc phrase or path:
  `app::manager_daily_brief::Packet` carries `domain::policy::ReviewGate` values and blocked-action metadata.

Read it as:
  The packet is a review bundle for one workflow. A review gate is a named human-approval stop inside that bundle. Blocked actions name side effects the workflow must not perform directly.

Operational translation:
  The system can gather evidence for a manager brief, rank work, draft a task or summary, and show that a behavior review, customer-message approval, manager approval, or refund/deposit exception is required. The packet helps staff review the work faster; the gate says which sensitive step still needs authorized review.

Why it matters:
  Operators get labor-saving prep without losing accountability for pet safety, customer trust, payments, schedules, or source-of-record changes. The packet makes the handoff auditable: what evidence was used, what action was suggested, which review was required, and which actions remained forbidden.

Do not infer:
  Do not infer that the packet is a queued job, final approval, sent message, provider write, schedule change, payment/refund action, or medical/behavior decision. A gate being present does not mean approval has happened; it means approval is still required before the sensitive step.

Evidence hooks:
  See `app/README.md`, `app/src/manager_daily_brief.rs`, `app/src/agents.rs`, `domain/src/policy.rs`, `domain/src/workflow.rs`, `docs/safety/source-evidence-map.md`, and the draft entries in `docs/glossary-workflow-state-terms.md`.

### Worked example: Gingr, source-of-record, and data-quality path

Rustdoc phrase or path:
  Gingr provider records are captured with `domain::source::Provenance` / `RecordRef`; mapping can surface `domain::data_quality::Issue`; read models are not the source of record.

Read it as:
  Gingr data is provider evidence first. Provenance and record refs show which provider/import record supports a fact. Mapping and data-quality contracts decide whether that evidence is clean enough for a domain fact, a review packet, BI visibility, or workflow blocking. A read model can summarize normalized evidence for review, but it is not the authority for live changes.

Operational translation:
  If a checkout exception says a reservation status, payment state, owner/pet link, or vaccination fact is missing or conflicting, the system should keep the Gingr/source pointer and show the data-quality issue instead of guessing. Staff can repair or review the source-of-record path; the manager brief can cite the issue and avoid hiding it.

Why it matters:
  This prevents dirty source data from turning into customer-facing mistakes, misleading labor reporting, or unsafe automation. It lets a leader distinguish “real operational exception” from “provider/source data needs cleanup.”

Do not infer:
  Do not infer that Gingr is the canonical NVA domain model, that a provider record is safe to mutate, that provenance proves correctness, that a data-quality issue is just an engineering bug, or that a read model can overwrite the provider/PMS. Source-of-record must always be named for the specific fact or action.

Evidence hooks:
  See `integrations/gingr/README.md`, `domain/src/source.rs`, `domain/src/data_quality.rs`, `docs/integrations/gingr/source-inventory.md`, `docs/integrations/gingr/bi-read-model-contract.md`, `app/src/manager_daily_brief.rs`, and the draft entries in `docs/glossary-source-data-terms.md`.

## Before/after example: architecture term

Term: `domain::source::Provenance`

Before: flattened/non-coder wording that loses the contract

> Provenance means we know where data came from.

Why this is insufficient:

- It hides the owning layer (`domain::source`).
- It does not say what the code carries: source system, endpoint, record id, extraction batch, pulled-at time, request scope, schema version, payload hash, and raw payload reference.
- It may imply source data is automatically trusted, when the repo treats provider/import data as evidence that must be mapped or reviewed before it drives automation.

After: contract-preserving glossary entry

```text
Term:
  `domain::source::Provenance`

Plain-language label:
  Source evidence tag for an operational fact.

Where it appears:
  `domain/src/source.rs`; repository type/module map in `README.md`.

Code-derived contract:
  Carries source-system lineage for a fact, including system, endpoint/import route, source record id, extraction batch, pulled-at timestamp, request scope, schema version, payload hash, and raw payload reference. A `RecordRef` can be derived from it for joins and citations.

Pet-resort operational meaning:
  When a manager brief, exception queue, or agent draft mentions an operational fact, provenance is the “show your work” tag that says which Gingr/BI/import record the fact came from.

Why an operator should care:
  It makes automation reviewable: staff can trace a draft recommendation back to the source evidence instead of treating model text as authority.

What not to infer:
  Provenance does not prove the source data is correct, does not normalize the provider payload, does not approve an action, and does not grant permission to mutate a reservation, schedule, payment, or customer message.

Boundary and authority:
  Domain source-lineage contract. Provider payloads still live at integration/storage boundaries until promoted into semantic values.

Evidence and review hooks:
  Rustdoc example in `domain/src/source.rs` builds `Provenance` and derives `RecordRef`; README type/module map lists `domain::source::{RecordRef, Provenance}` as the source/provenance boundary.

Suggested public wording:
  `domain::source::Provenance` is the source-evidence tag attached to operational facts. It tells reviewers which provider/import record and extraction context support a draft or recommendation, but it does not make that fact automatically correct or authorize live changes.

Related terms:
  `domain::source::RecordRef`, `integrations::gingr::mapping`, `domain::data_quality::Issue`, `app::manager_daily_brief`.
```

## Before/after example: workflow term

Term: `policy::ReviewGate`

Before: flattened/non-coder wording that overclaims

> A review gate is where the AI asks for approval before doing something.

Why this is insufficient:

- It centers “AI” rather than the policy contract.
- It suggests the system is otherwise free to act live.
- It does not name which sensitive decisions are currently represented.
- It loses the distinction between drafting/routing and live provider/customer/payment action.

After: contract-preserving glossary entry

```text
Term:
  `policy::ReviewGate`

Plain-language label:
  Named human-approval stop for sensitive resort work.

Where it appears:
  `domain/src/policy.rs`; `domain/src/workflow.rs`; `domain/src/agent.rs`; app workflow Rustdocs that carry required reviews or blocked actions.

Code-derived contract:
  Enumerates human review categories required before automation may proceed with sensitive work: manager approval, medical document review, behavior review, customer-message approval, and refund/deposit exception. Workflow contexts and recommended actions can carry these gates as explicit values.

Pet-resort operational meaning:
  A review gate is the repo’s way of saying, “this can be prepared or routed, but a human with the right authority must review before the sensitive step happens.” Examples include behavior concerns before group play, medical document uncertainty, customer-facing messages, and refund/deposit exceptions.

Why an operator should care:
  It protects pet safety, customer trust, payment controls, and local manager authority while still letting automation reduce triage and draft work.

What not to infer:
  A review gate is not the approval itself, not a UI workflow guarantee, not permission to send a message or change a provider record, and not a substitute for local resort policy.

Boundary and authority:
  Domain policy vocabulary consumed by workflow/app contracts. It records required human authority; runtime shells and tools must still respect it before side effects.

Evidence and review hooks:
  `domain/src/policy.rs` defines the variants; policy tests route behavior/group-play concerns to `ReviewGate::BehaviorReview`; `domain/src/workflow.rs` stores required reviews and `RecommendedAction::RequestHumanReview(policy::ReviewGate)`.

Suggested public wording:
  `policy::ReviewGate` is a named human-approval stop. It lets the system draft or route work while keeping manager approval, medical review, behavior review, customer-message approval, and refund/deposit exceptions outside automatic execution.

Related terms:
  `policy::automation::Level`, `domain::workflow::PolicyContext`, `domain::workflow::RecommendedAction`, `app::manager_daily_brief::BlockedAction`.
```

## Reviewer checklist for future glossary entries

- Does the entry keep the exact term/path visible?
- Does it name the owning layer and source file?
- Does “code-derived contract” say only what the source proves?
- Does the operational meaning help a resort leader understand labor, safety, trust, or handoff impact?
- Does “what not to infer” block the most likely overclaim?
- Does it distinguish provider evidence, domain truth, app workflow packet, storage record, and runtime shell?
- Does it preserve draft/review/live-action boundaries?
- Are examples either linked to compile-checked Rustdoc or labeled conceptual?
- Would a non-coder understand why the term matters without believing the product does more than the repo currently promises?

## Placement and maintenance recommendation

Keep glossary entries in public docs near the README/Rustdoc navigation layer, but treat source/Rustdoc as the authority. A future implementation card can choose one of these shapes:

1. A single `docs/glossary.md` index for public readers, with each entry linking back to source/Rustdoc.
2. A `docs/glossary/` directory split by layer (`domain`, `app`, `storage`, `integrations`, `runtime`) if entries become numerous.
3. Short glossary callouts in existing READMEs that link to canonical entries, avoiding duplicate definitions.

Whichever shape is chosen, do not copy executable Rust examples into the glossary unless they come from compile-checked Rustdoc. The glossary translates contracts; it does not become the contract authority.
