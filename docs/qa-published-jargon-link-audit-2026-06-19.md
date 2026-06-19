# Published docs QA: jargon density and link-quality audit

Task: `t_2bb4fb3f`
Published docs root: https://nva.eman.network/
Audit date: 2026-06-19

## Scope and method

Focused crawl of the published Rustdoc site, prioritizing the root entry page, crate entry pages, module entry pages, and representative API pages rather than exhaustive item coverage.

Automated evidence:
- Crawled 80 published pages from `https://nva.eman.network/`.
- Checked 1,424 unique/observed HTTP(S) link destinations from those pages with GET requests and redirects enabled.
- Searched href/src destinations for accidental local-only targets: `localhost`, `127.0.0.1`, `0.0.0.0`, `file://`, `/home/...`, Windows drive paths, repo-relative `.md`, and root-relative `.md` paths.
- Scored visible page text for jargon terms that are likely difficult for an ops leader/non-coder reader without a glossary or plain-English lead-in.

Caveat: this is a representative published-site audit, not a full exhaustive crawl of every generated Rustdoc item and source-code page.

## Broken/local-link findings

No broken or accidental local-only link findings were found in the representative crawl.

Evidence:
- Pages crawled: 80
- Link checks: 1,424
- Broken public HTTP(S) links: 0 returned HTTP 4xx/5xx or request failure
- Local-only href/src destinations: 0 matched the local-only patterns above

Recommended follow-up:
- Keep a CI/publish check for local-only links because the source repository contains local dev URLs in ops docs as examples; those are acceptable in local setup docs but should not become entry-page navigation links.
- Future exhaustive pass should include fragment/anchor validation if published docs start using hand-authored HTML anchors heavily.

## Jargon-density findings

### J1. Root page uses technical labels before explaining the site for ops leaders

Severity: High
Source page: https://nva.eman.network/
Evidence: 45 visible words, 7 jargon hits, ~15.56 jargon hits per 100 words.
Observed text/examples:
- `domain — semantic pet-resort service and operations contracts`
- `app — review-gated agent workflows and tool ports`
- `storage — persistence boundary contracts`
- `gingr — source adapter/request/webhook boundaries`
Why it matters: this is the first page an ops leader or non-coder sees. It is technically accurate, but it immediately asks the reader to understand `semantic`, `contracts`, `review-gated`, `tool ports`, `persistence boundary`, `adapter`, and `webhook` before explaining what business question each section answers.
Suggested fix direction: add a plain-English lead and audience paths above the Rustdoc crate list, for example “Start here if you want to know what labor-saving workflows exist,” “Start here if you want to understand safety/review gates,” and “Developer API reference.” Keep the crate links, but pair each with an operational translation.

### J2. `domain` crate page is thorough but contract-heavy for non-coders

Severity: Medium
Source page: https://nva.eman.network/domain/
Evidence: 687 visible words, 59 jargon hits, ~8.59 jargon hits per 100 words; `contracts` appears 43 times.
Observed text/examples:
- `Typed domain contracts for NVA Pet Resorts labor-cost automation.`
- `source-system provenance`
- `review-gated`
- repeated module summaries framed as `contracts`
Why it matters: this page is likely the main “what does the system know about the business?” page, but the repeated `contracts` phrasing makes it feel like an API catalog rather than an operator-readable map.
Suggested fix direction: keep the exact Rust concepts, but add one opening paragraph that translates “domain contracts” into “the shared business vocabulary and safety rules the software uses.” Consider replacing repeated module summaries like “X contracts for…” with “X covers…” on pages meant to be read by non-coders.

### J3. `app` crate page compresses too many architecture terms into the summary

Severity: Medium
Source page: https://nva.eman.network/app/
Evidence: 160 visible words, 19 jargon hits, ~11.88 jargon hits per 100 words.
Observed text/examples:
- `Shared application/workflow orchestration`
- `composes semantic domain contracts`
- `executable workflow packets, agent prompts, tool-port contracts, and MVP orchestration previews`
- module names/descriptions: `deterministic review`, `source-grounded`, `provider mutation`
Why it matters: this page is probably where a reader goes to understand workflows. The current summary is useful to engineers but does not first say “this is where booking triage, daily updates, checkout completion, retention, and manager briefs are assembled into reviewable staff packets.”
Suggested fix direction: add a workflow-first sentence before the architecture sentence. Use business nouns first, then technical terms in parentheses or glossary links.

### J4. `gingr` crate page assumes the reader already knows integration-layer vocabulary

Severity: Medium
Source page: https://nva.eman.network/gingr/
Representative dense subpage: https://nva.eman.network/gingr/dto/index.html
Evidence:
- `gingr/`: 128 visible words, 12 jargon hits, ~9.38 per 100 words.
- `gingr/dto/index.html`: 91 visible words, 26 jargon hits, ~28.57 per 100 words; repeated `DTO`, `promotion`, `semantic`, `provider DTO`.
Observed text/examples:
- `Gingr integration contracts, DTO mappings, endpoints, transport, and webhook verification.`
- `Raw Gingr DTO surfaces that are intentionally quarantined before NVA domain promotion.`
- `Promotion helpers from quarantined Gingr records into source-agnostic candidates.`
Why it matters: this is one of the most likely areas where an ops leader might ask “what does this connect to?” but the page answers with adapter vocabulary. `DTO` and `promotion` need translation.
Suggested fix direction: add a plain-language integration summary: “Gingr is the source system. These pages show how we safely read Gingr data, keep raw provider fields separate, and turn reviewed facts into NVA workflows.” Add glossary links for `DTO`, `endpoint`, `mapping`, `source-agnostic`, and `promotion`.

### J5. `storage` crate page is accurate but exposes persistence language too early

Severity: Medium
Source page: https://nva.eman.network/storage/
Evidence: 161 visible words, 14 jargon hits, ~8.70 per 100 words.
Observed text/examples:
- `Storage adapter contracts`
- `persistence projection`
- `stable storage codes, JSON codecs, flattened records`
- `explicit promotion/demotion between storage records and core domain types`
Why it matters: the page explains an important safety boundary, but non-coders may not understand why storage is separate from the business model.
Suggested fix direction: add a lead sentence such as “Storage is the filing cabinet, not the decision-maker.” Then explain that records are kept in database-friendly shapes and must be translated back into business concepts before staff/customer actions.

### J6. Representative API item pages expose generated Rust/typestate details without a reader escape hatch

Severity: Medium
Source page: https://nva.eman.network/app/booking_triage/struct.Request.html
Evidence from extracted page text:
- `Request<RequestState: RequestStateTrait = UninitializedRequestState>`
- `type parameter encodes the current request state at compile time`
- `DeclaredTransitionMapEdge`, `CanTransitionMap`, generated builder names like `__StatumRequest...MissingSlot...`
Why it matters: item pages are allowed to be technical, but this representative page is linked from workflow pages. If an ops leader lands here, the important story is “a booking must collect pet profile and policy evidence before policy decision,” not the generated builder/type-state machinery.
Suggested fix direction: add a short human-readable “Operational meaning” paragraph to representative workflow structs/enums before generated details. Keep generated signatures for engineers.

## High-severity follow-up need

The only high-severity issue is J1: the published root entry page is still too jargon-first for the target audience. This should be handled as a docs fix card because it is a content/navigation change rather than a QA-only task.

## Audit artifacts

Raw audit JSON for this run was written to `/tmp/nva-docs-qa-audit.json` during execution. The temporary crawl script was removed after the report was generated; the reproducible method and patterns are summarized above.
