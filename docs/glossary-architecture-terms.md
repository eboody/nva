# Architecture glossary for non-coder NVA docs

Purpose: translate recurring repo/Rust architecture terms into pet-resort operational meaning without replacing the source contract. These are public/non-coder glossary entries. Source and Rustdoc remain authoritative; each entry names where to verify the contract and what a reader should not infer.

Source basis for these entries:

- Root workspace map and type/module map in `README.md`.
- Layer READMEs: `domain/README.md`, `app/README.md`, `storage/README.md`, `integrations/gingr/README.md`.
- DTO guide: `integrations/gingr/src/dto/README.md`.
- Tool-port guide: `app/src/tools/README.md`.
- Read-model plan: `docs/integrations/gingr/bi-read-model-contract.md`.
- Source files and Rustdoc surfaces named below.

## `domain`

Plain-language label: source-of-truth business vocabulary.

Where it appears: `README.md`; `domain/README.md`; `domain/src/lib.rs`; domain modules such as `domain/src/entities.rs`, `domain/src/source.rs`, `domain/src/workflow.rs`, `domain/src/policy.rs`, and service-line modules under `domain/src/{boarding,daycare,grooming,retail,training}`.

Code-derived contract: `domain` owns semantic business concepts, invariant-bearing values, policy/review vocabulary, source provenance, workflow decision outputs, service-line contracts, and core customer/pet/reservation/care/message/approval records. It explicitly does not own provider clients, database schemas, HTTP APIs, or UI shells.

Pet-resort operational meaning: this is the repo's precise language for resort facts and safety rails: what a reservation status means, which vaccine or behavior review is required, what a payment/deposit state is called, what evidence a source fact carries, and which service-line policy a workflow may rely on.

Why an operator should care: when automation produces a draft, review packet, or manager brief, `domain` terms are the guardrails that keep it aligned with operational policy instead of raw vendor strings or model guesses.

What not to infer: do not infer that `domain` reads Gingr, stores rows, sends messages, exposes screens, mutates reservations, or proves a workflow has been deployed. It names and validates business meaning; other layers wire persistence, integrations, and runtime behavior.

Boundary and authority: semantic truth layer. Provider facts, storage records, app workflow packets, and runtime shells must promote into or compose these terms rather than redefining them.

Evidence and review hooks: the root type/module map points to representative files; `domain/README.md` lists public types and module ownership; Rustdoc/source files such as `domain/src/source.rs`, `domain/src/workflow.rs`, `domain/src/policy.rs`, and service-line modules contain the executable contracts.

Safe example: a boarding-capacity rule or daycare group-play eligibility decision should be expressed as a `domain::*` service-line policy value before an app workflow uses it to route a staff review packet.

Suggested public wording: `domain` is the repo's source-of-truth business vocabulary. It names pet-resort facts, policies, review gates, source evidence, and service-line rules so automation can draft and triage work without treating raw provider data or AI text as authority.

Related terms: `app`, `storage`, `integration`, `source ref`, `provenance`, `read model`.

## `app`

Plain-language label: workflow and review-bundle layer.

Where it appears: `README.md`; `app/README.md`; `app/src/lib.rs`; workflow modules such as `app/src/booking_triage.rs`, `app/src/checkout_completion.rs`, `app/src/crm_retention.rs`, `app/src/daily_update.rs`, `app/src/manager_daily_brief.rs`; `app/src/agents.rs`; `app/src/tools.rs`.

Code-derived contract: `app` composes normalized `domain` concepts into use-case request packets, deterministic evaluations, draft-only artifacts, audit drafts, workflow packets, agent prompt packets, and tool-port contracts. It may define local workflow classifications, but those are not alternate domain models.

Pet-resort operational meaning: this is where resort work becomes a reviewable operating flow: booking triage, checkout completion, CRM follow-up, daily updates, manager daily briefs, local smoke fixtures, and agent-safe context packets.

Why an operator should care: `app` is the labor-saving layer that turns already-grounded facts into narrow work queues, drafts, and manager summaries, while keeping customer sends, provider/PMS mutations, payments, and sensitive approvals behind explicit review or tool boundaries.

What not to infer: do not infer that `app` is a mobile/web front-end, a deployed bot, a live Gingr client, a payment processor, or a customer-message sender. Current app workflows can draft, evaluate, rank, packetize, and validate; live side effects belong behind ports, adapters, and shells with review gates.

Boundary and authority: application/use-case composition layer. It depends on `domain` for truth, calls `storage` or `integrations/*` through contracts, and is exposed by `apps/*` runtime shells.

Evidence and review hooks: `app/README.md` maps request/evaluation/draft/packet/action families; `app/src/agents.rs` defines agent prompt/spec packets; workflow source modules and tests under `app/tests` show source-grounded packets and blocked actions.

Safe example: a manager daily brief packet can rank service-demand facts and checkout exceptions, estimate labor minutes, and propose draft actions for review, but it does not itself change a schedule or message a customer.

Suggested public wording: `app` is the workflow layer. It packages trusted `domain` facts into reviewable packets, drafts, and deterministic evaluations so staff and managers can handle exceptions faster without giving automation live operational authority by default.

Related terms: `domain`, `tool port`, `read model`, `provenance`, `storage`.

## `storage`

Plain-language label: persisted projection and conversion boundary.

Where it appears: `README.md`; `storage/README.md`; `storage/src/lib.rs`; `storage/src/operations.rs`; service-line records under `storage/src/service_line/*`.

Code-derived contract: `storage` owns storage-shaped records, stable persisted code values, JSON codecs, storage-side validated scalars, errors, and explicit promotion/demotion paths between persisted shapes and semantic `domain::*` values.

Pet-resort operational meaning: it is the durable/auditable shape for facts the system needs to save or report: service-offering records, service-line contract records, technology ecosystem records, manager daily brief outcome records, source refs, and labor-savings evidence.

Why an operator should care: storage projections make review and reporting repeatable without asking staff to reinterpret raw provider payloads or mixed optional fields every time a workflow needs evidence.

What not to infer: do not infer that the database shape is the business truth, that storage mirrors every Gingr payload, or that storage decides customer-facing outcomes. Storage records must convert to/from domain values and should not own policy, provider transport, or live actions.

Boundary and authority: persistence/projection boundary. It demotes domain values into durable records and promotes stored records back into domain values through explicit conversion code.

Evidence and review hooks: `storage/README.md` explains record/code/codec/error families; `storage/src/operations.rs` and `storage/src/service_line/*` hold the concrete records and conversions; `storage/tests` exercises storage contracts.

Safe example: `storage::operations::ManagerDailyBriefOutcomeRecord` can store labor minutes, outcome, actor, source refs, reporting group, and savings evidence after review, but it is not the manager's policy decision itself.

Suggested public wording: `storage` is the repo's persisted projection layer. It saves normalized records and stable codes with explicit conversion back to `domain` truth, so reporting and workflow evidence stay durable without turning database rows into policy.

Related terms: `domain`, `read model`, `source ref`, `DTO`, `adapter`.

## `integration` / `integrations/gingr`

Plain-language label: provider boundary for Gingr evidence.

Where it appears: `README.md`; `integrations/gingr/README.md`; `integrations/gingr/src/lib.rs`; `integrations/gingr/src/{config,endpoint,transport,response,webhook,dto,mapping}.rs`; module READMEs under `integrations/gingr/src/*/README.md`.

Code-derived contract: `integrations/gingr` owns Gingr-specific configuration, endpoint request builders, transport seams, response envelopes, webhook verification, provider DTOs, and the first mapping step from provider records into semantic `domain::*` candidates. It treats provider ids, statuses, endpoints, undocumented fields, and API quirks as provider facts.

Pet-resort operational meaning: this is where Gingr data enters the system in a controlled way. It captures what Gingr returned, redacts/quarantines sensitive boundary details, preserves unknown fields or documented gaps, and maps only supported facts forward for workflow use.

Why an operator should care: the adapter reduces raw-JSON/provider-screen spelunking. Staff and maintainers can see which facts came from Gingr, which were mapped cleanly, and which must remain review gaps before automation depends on them.

What not to infer: do not infer that Gingr facts are NVA canonical truth, that all Gingr surfaces are supported, or that live HTTP is implemented. `integrations/gingr/README.md` states the default `HttpTransport` currently returns `TransportError::HttpNotImplemented`.

Boundary and authority: external/provider adapter boundary. Gingr vocabulary stays here until mapping, provenance, domain promotion, storage projection, or app workflow contracts explicitly accept it.

Evidence and review hooks: endpoint, response, webhook, DTO, and mapping modules preserve typed request/response shapes, unknown fields, verification status, provider-surface gaps, and mapping errors; provider docs and fixtures under `docs/integrations/gingr` explain source assumptions.

Safe example: an owner or animal provider record may be mapped into a customer/contact or pet-name candidate, but a provider id remains Gingr-scoped evidence unless a domain/source relationship explicitly records it.

Suggested public wording: `integrations/gingr` is the controlled Gingr boundary. It captures provider requests, responses, webhooks, DTOs, and mappings as evidence, then promotes only validated facts into `domain`, `storage`, or `app` contracts.

Related terms: `adapter`, `DTO`, `source ref`, `provenance`, `read model`.


## contract

Plain-language label: source-backed rule or code promise.

Where it appears: `README.md`; `domain/README.md`; `app/README.md`; `storage/README.md`; service-line READMEs such as `domain/src/boarding/README.md`, `domain/src/daycare/README.md`, `domain/src/grooming/README.md`, `domain/src/training/README.md`, and `domain/src/retail/README.md`; app workflow specs and contract-crosswalk pages under `docs/entity-atlas/contract-crosswalk`.

Code-derived contract: in this repo, a contract is a source-backed behavioral promise: a domain service-line rule, an app workflow packet shape, a storage conversion guarantee, an integration boundary rule, a test/Rustdoc-backed proof path, or a documentation shape that must stay aligned with source. It is not a legal, customer, employee, vendor, or pricing contract.

Pet-resort operational meaning: a contract says “this is the rule the system must preserve before staff can trust the workflow.” Examples include boarding deposit policy, daycare group-play eligibility, a booking-triage review packet, a blocked-action list, a storage conversion, or a Gingr mapping boundary.

Why an operator should care: contract language marks authority and safety boundaries. If a page says a workflow follows a contract, the reader should expect source/Rustdoc/test evidence and should not treat a draft, dashboard, or provider payload as permission to act outside that rule.

What not to infer: do not infer a legal/customer agreement, vendor obligation, pricing promise, production deployment, or live side-effect authority. A repo contract can prove how code should behave locally while still requiring human or system-of-record approval before an operational action.

Boundary and authority: contracts belong to the layer that owns the rule: `domain` for business vocabulary and policy, `app` for workflow packets and review boundaries, `storage` for persisted projections/conversions, `integrations/gingr` for provider evidence boundaries, and humans/systems of record for live decisions.

Evidence and review hooks: cite the source module, generated Rustdoc, tests, and contract-crosswalk row beside the claim. Use the root README's entity reading contract for public docs shape, and `docs/entity-atlas/contract-crosswalk/*` when proof matters more than narrative.

Safe example: `domain::boarding::Contract` can state the boarding policy bundle that workflows read, but it does not book a room, collect a deposit, or create a legal boarding agreement with a customer.

Suggested public wording: a contract in this repo is a source-backed rule or code promise, not a legal/customer contract. It tells readers which layer owns the rule and where source/Rustdoc/test evidence proves it.

Related terms: `domain`, `app`, `storage`, `source-of-record`, `review gate`, `blocked action`.

## semantic

Plain-language label: business meaning or source-of-truth vocabulary.

Where it appears: `README.md`; `domain/README.md`; service-line READMEs; `integrations/gingr/README.md`; mapping and DTO docs; storage conversion docs.

Code-derived contract: semantic values are validated business concepts that source/provider/storage/app surfaces must promote into before they can be treated as NVA pet-resort meaning. They include domain entities, service-line policies, workflow statuses, source/provenance values, and review vocabulary. The word is often paired with `domain` because `domain` owns the repo's canonical business language.

Pet-resort operational meaning: semantic means “what this fact means for resort work after validation.” A raw provider status, spreadsheet column, DTO field, or AI phrase is not semantic truth until the owning source/domain contract says which customer, pet, reservation, service, policy, review gate, or outcome it represents.

Why an operator should care: semantic boundaries keep raw provider data and model text from becoming unsafe business decisions. They let a manager ask, “Has this been translated into the business meaning we actually trust?”

What not to infer: do not infer that “semantic” means subjective wording, an LLM interpretation, SEO keywords, or a hidden ontology. It is source-backed business vocabulary with validation and authority boundaries.

Boundary and authority: `domain` owns canonical semantic truth. `integrations/gingr`, `storage`, `app`, and runtime shells may carry candidates, projections, or packets, but they should not redefine business meaning independently.

Evidence and review hooks: look for source constructors, enums, conversion functions, mapping errors, tests, and Rustdoc examples that prove how a raw fact becomes a domain/app/storage value.

Safe example: a Gingr owner name becomes semantic customer evidence only after mapping validates it into `domain::customer::Name`; until then it is provider evidence.

Suggested public wording: semantic means business meaning after validation — the source-of-truth vocabulary staff and workflows use — not raw provider text or AI interpretation.

Related terms: `domain`, `contract`, `promotion / demotion`, `DTO`, `provider record`.

## projection

Plain-language label: database/reporting-friendly view of facts.

Where it appears: `README.md`; `storage/README.md`; `docs/design/entity-atlas-runtime-storage-api-surfaces.md`; storage/read-model docs; analytics and outcome-record pages.

Code-derived contract: a projection is a storage, analytics, or read-model shape built from source-grounded/domain values so reporting, review, replay, or local API/worker surfaces can read evidence efficiently. It can persist selected fields, stable codes, source refs, outcome records, or derived metrics, but it must not invent source truth or own live decision authority.

Pet-resort operational meaning: a projection is the filing-cabinet or dashboard-friendly copy of facts staff may review later: outcome rows, labor-minute evidence, source refs, service-line records, read models, or local smoke/API records.

Why an operator should care: projections make review and reporting repeatable without re-reading raw provider screens every time, but staff still need to know which source/human/system owns the real-world decision.

What not to infer: do not infer that a projected database row can change Gingr, approve a booking, send a customer message, override policy, or become the universal source of record. Projection is evidence/review shape, not live authority.

Boundary and authority: `storage` and analytics/read-model docs own most projections. `domain` owns business meaning; provider systems and humans/systems of record own live facts/actions; `app` owns workflow packets that may read or write reviewed outcome projections.

Evidence and review hooks: cite storage records/conversion tests, analytics/read-model contracts, source refs/provenance, and outcome-record tests. Check that the projection names its source and review status.

Safe example: `storage::operations::ManagerDailyBriefOutcomeRecord` is a projection that records reviewed labor evidence; it is not the manager's staffing decision or payroll truth.

Suggested public wording: a projection is a durable reporting/review view of source-grounded facts. It helps readers audit and measure work, but it does not replace the source of record or authorize live actions.

Related terms: `storage`, `read model`, `source ref`, `provenance`, `outcome capture`.

## promotion / demotion

Plain-language label: validated conversion between boundary shapes and business meaning.

Where it appears: `README.md`; `domain/README.md`; `storage/README.md`; `integrations/gingr/README.md`; `integrations/gingr/src/dto/README.md`; `integrations/gingr/src/mapping/README.md`; storage service-line docs.

Code-derived contract: promotion converts a raw/provider/storage shape into a validated `domain::*`, app, or storage-normalized value with explicit errors when required evidence is missing or unsafe. Demotion converts a validated business value back into a storage/provider-shaped record or code when a persisted or boundary form is needed. Both directions must be explicit; they are not string copying or silent defaulting.

Pet-resort operational meaning: promotion answers “is this provider/database thing safe to use as business meaning?” Demotion answers “how do we save or send the already-validated meaning without losing its boundary?”

Why an operator should care: these conversions stop raw Gingr fields, storage codes, BI rows, or generated text from silently becoming customer, pet, reservation, payment, or policy truth. Failures become review/data-quality work instead of hidden guesses.

What not to infer: do not infer staff promotion/demotion, job status changes, sales funnel movement, or automatic live provider updates. This term is about data conversion and validation only.

Boundary and authority: `integrations/gingr::mapping` promotes provider records into domain candidates; `storage` promotes/demotes persisted records and codes; app workflows consume promoted values and should surface failures through review gates or data-quality issues.

Evidence and review hooks: look for `TryFrom`/`From` implementations, mapping functions, typed errors, data-quality issue creation, tests, and Rustdoc examples. If a conversion can fail, public docs should say what review lane receives the failure.

Safe example: a Gingr retail DTO can be promoted into a `domain::retail` product candidate only after required name/SKU/status fields validate; a validated domain service-line contract can later be demoted into a storage record for persistence.

Suggested public wording: promotion/demotion means explicitly translating and validating between raw provider/storage shapes and trusted business meaning. It is data conversion, not employee status or customer-facing action.

Related terms: `semantic`, `DTO`, `provider record`, `storage`, `adapter`, `data-quality issue`.

## `adapter`

Plain-language label: quarantined translator for an outside system or boundary shape.

Where it appears: `integrations/gingr/README.md`; `integrations/gingr/src/{endpoint,response,dto,mapping,transport,webhook}.rs`; `storage/README.md` when distinguishing provider adapters from persisted projections; `app/src/tools/README.md` when concrete capabilities satisfy app ports.

Code-derived contract: an adapter keeps boundary-specific details local, translates typed requests/responses, preserves provider or storage evidence, and promotes only validated facts into domain/app/storage contracts. In this repo, the clearest adapter is `integrations/gingr`, while future concrete implementations can satisfy `app::tools` ports.

Pet-resort operational meaning: an adapter is the staff-safe translation desk between an external system and NVA workflows. It prevents raw provider status strings, ids, unknown fields, or transport errors from becoming business decisions without reviewable mapping.

Why an operator should care: adapters make integration behavior inspectable. If a recommendation depends on Gingr or another provider, reviewers can ask whether the adapter captured, redacted, verified, mapped, or rejected the source fact before it reached a workflow.

What not to infer: do not infer that an adapter owns business policy, performs live sync, bypasses review gates, or guarantees provider data is correct. It may only expose typed evidence or stubs until a concrete live implementation exists.

Boundary and authority: boundary translation layer. It sits outside `domain` truth and should report provider/storage/runtime facts through explicit mappings, source refs, or app tool outcomes.

Evidence and review hooks: adapter modules should have typed request/response shapes, mapping errors, redacted logs, unknown-field retention, provider-surface gap markers, tests, or docs pointing to fixtures/source inventory.

Safe example: a Gingr retail DTO can pass through an adapter mapping into a `domain::retail` product candidate only after required provider fields such as name/SKU validate; unsupported grooming/training service DTOs remain provider-surface gaps.

Suggested public wording: an adapter is a controlled translator at the edge of the system. It lets NVA workflows use provider evidence without letting provider-specific fields or errors become business truth by accident.

Related terms: `integration`, `DTO`, `tool port`, `storage`, `provenance`.

## DTO

Plain-language label: provider payload shape.

Where it appears: `README.md`; `integrations/gingr/README.md`; `integrations/gingr/src/dto/README.md`; `integrations/gingr/src/dto/{mod,retail,grooming,training}.rs`; related provider records in `integrations/gingr/src/response.rs`.

Code-derived contract: a DTO is a provider-shaped record that is documented or fixture-backed enough to deserialize at the adapter boundary. Current `gingr::dto` examples preserve provider ids, optional fields, unknown fields, and explicit provider-surface gap markers; they are not domain models.

Pet-resort operational meaning: a DTO says, “this is what the provider appears to have said,” not “this is now a safe operational fact.” It lets maintainers review catalog, owner, animal, reservation, webhook, or other provider evidence before mapping.

Why an operator should care: DTOs keep messy provider data from silently entering staff workflows. Optional fields, unknown fields, and unsupported surfaces can become review items instead of being guessed into customer/pet/reservation truth.

What not to infer: do not infer that DTO fields are safe for automation, reporting, customer action, or canonical identity without mapping and validation. Provider ids are scoped to the provider; unknown fields are drift evidence, not hidden business semantics.

Boundary and authority: provider payload boundary inside an adapter. Semantic promotion belongs in `gingr::mapping` and then `domain`, storage projections, or app packets as appropriate.

Evidence and review hooks: `integrations/gingr/src/dto/README.md` documents current DTOs and gaps; `gingr::dto::retail::Item` retains unknown fields; `ProviderSurface::NoDocumentedServiceDto` records grooming/training service DTO gaps.

Safe example: a retail item DTO may contain a Gingr item id, optional name, optional SKU, optional category, active flag, quantity, and unknown fields. Only after mapping validates required fields should it become a retail product candidate.

Suggested public wording: a DTO is a provider payload shape. It captures source evidence from Gingr or another provider, but the repo does not treat it as business truth until mapping and domain validation say which facts are safe to use.

Related terms: `integration`, `adapter`, `source ref`, `provenance`, `read model`.

## source ref / `domain::source::RecordRef`

Plain-language label: stable pointer to source evidence.

Where it appears: `README.md`; `domain/README.md`; `domain/src/source.rs`; workflow/app modules that carry source evidence such as `app/src/checkout_completion.rs`, `app/src/crm_retention.rs`, and `app/src/manager_daily_brief.rs`; storage source-reference projections in `storage/src/operations.rs`.

Code-derived contract: `RecordRef` is the stable pointer to an upstream record and the system that owns it. It is derived from source/provenance context and lets records, packets, and projections cite the source behind a claim.

Pet-resort operational meaning: this is the “which record are we talking about?” handle. A manager brief or staff packet can point back to the Gingr reservation, BI extract, import row, or other upstream record that supported a recommendation.

Why an operator should care: source refs make review targeted. Staff can investigate the record behind a draft or exception instead of trusting a summary with no traceable evidence.

What not to infer: do not infer that a source ref contains the full payload, proves the source is correct/current, resolves duplicates, or authorizes mutation of the upstream record. It is a pointer/evidence handle, not approval.

Boundary and authority: domain source-lineage contract that can be carried by app packets and storage projections. It preserves evidence across layer boundaries without importing raw provider payloads into every layer.

Evidence and review hooks: `domain/src/source.rs` defines `RecordRef` and `Provenance`; README type/module maps list the source/provenance boundary; storage/app records that include source refs can be checked against the source system and extraction context.

Safe example: a checkout-completion packet may cite a Gingr reservation source ref so staff can verify the observed checkout status before accepting any audit draft.

Suggested public wording: a source ref (`domain::source::RecordRef`) is a trace pointer to the upstream record behind a workflow fact. It helps reviewers find the evidence, but it does not prove the data is correct or permit an automated change.

Related terms: `provenance`, `DTO`, `integration`, `read model`, `storage`.

## provenance / `domain::source::Provenance`

Plain-language label: chain of custody for a source-backed fact.

Where it appears: `README.md`; `domain/README.md`; `domain/src/source.rs`; app workflow modules such as `app/src/checkout_completion.rs`, `app/src/crm_retention.rs`, and `app/src/manager_daily_brief.rs`; storage projections that preserve source references.

Code-derived contract: `Provenance` carries lineage metadata tying normalized data back to a provider/source record: system, endpoint/import route, record id, related record ids, extraction batch, pulled-at timestamp, request scope, schema version, payload hash, and raw payload reference. A `RecordRef` can be derived for joins and citations.

Pet-resort operational meaning: provenance is the evidence trail for a fact in a draft, manager brief, exception queue, projection, or data-quality issue. It says where the fact came from and under what extraction context.

Why an operator should care: provenance makes automation auditable. If a recommendation looks wrong, reviewers can trace it to the source pull, payload, and schema context instead of arguing with ungrounded text.

What not to infer: do not infer that provenance means the fact is clean, current, complete, approved, or safe to act on. It documents lineage; review gates, mapping validation, and data-quality checks still decide whether the fact can drive a workflow.

Boundary and authority: domain source-lineage contract consumed by app, storage, analytics, and integration mapping surfaces.

Evidence and review hooks: `domain/src/source.rs` Rustdoc/source examples show building provenance and deriving record refs; workflow/storage modules that carry provenance or source refs expose reviewable citations.

Safe example: a CRM retention opportunity can carry provenance showing which checkout/completion evidence and contact-permission source supported the draft follow-up, without sending a customer message.

Suggested public wording: provenance (`domain::source::Provenance`) is the evidence trail attached to a normalized fact. It lets NVA trace which provider/import record and extraction context support a draft or recommendation, but it is not approval or proof of correctness.

Related terms: `source ref`, `DTO`, `read model`, `data-quality issue`, `storage`.

## tool port / `app::tools`

Plain-language label: approved capability interface.

Where it appears: `README.md`; `app/README.md`; `app/src/tools.rs`; `app/src/tools/README.md`; architecture guide `docs/architecture/agent-app-infrastructure.md`; runtime shells under `apps/*` when implementing or exposing tool capabilities.

Code-derived contract: `app::tools` defines traits and typed request/outcome/error packets for capabilities that workflows need: customer/reservation stores, availability checks, draft reservation updates, portal lookup, payment gateway actions, message drafting, document/OCR intake, media capture, agent runtime execution, and Hermes task/schedule drafts. It names the contract without choosing one provider or runtime.

Pet-resort operational meaning: a tool port is the safe doorway a workflow uses when it needs an external capability. It lets the app request “draft a message,” “check availability,” “look up a portal account,” or “record a draft task” through typed packets rather than raw API calls.

Why an operator should care: tool ports keep side effects visible and reviewable. They can return draft ids, review-required statuses, unavailable reasons, policy denials, or typed provider results instead of hiding ambiguity in free-text logs.

What not to infer: do not infer that a port implementation exists, is live, sends a message, changes Gingr, captures payment, or overrides policy. A port may be a trait, stub, mock, draft-only interface, or future adapter contract until a shell safely implements it.

Boundary and authority: application capability contract. Deterministic app workflows own context and policy; concrete adapters/shells satisfy ports; `domain` still owns business truth and review gates.

Evidence and review hooks: `app/src/tools/README.md` lists port families and safe statuses such as draft-only messaging, human-review payment/OCR outcomes, unavailable media, and Hermes draft statuses; `app/src/tools.rs` and `app/src/tools/error.rs` define the typed contracts.

Safe example: a messaging port can create a customer-message draft with `DraftedRequiresReview`, but that is not the same as sending the message to a pet parent.

Suggested public wording: a tool port (`app::tools`) is an approved capability interface. Workflows use it to request typed reads, checks, or drafts while keeping live external actions behind policy review and concrete adapter implementations.

Related terms: `app`, `adapter`, `integration`, `draft`, `review gate`.

## read model

Plain-language label: reporting or review projection optimized for reading.

Where it appears: `docs/integrations/gingr/bi-read-model-contract.md`; `README.md`; `storage/README.md`; `domain/src/analytics.rs`; `storage/src/operations.rs`; source/read-model plans under `docs/plans/*gingr*read-model*`.

Code-derived contract: a read model is a denormalized or projection shape for reading, BI, analytics, dashboards, local lab queries, or workflow evidence after source/provider facts have crossed explicit trust boundaries. The Gingr BI read-model contract says raw DTOs and snapshots must promote through provenance, source-agnostic snapshots, assumptions/issues, and analytics facts before workflow validators consume them.

Pet-resort operational meaning: this is a manager/reporting-friendly view of normalized evidence: stay facts, service-demand facts, labor snapshots, outcome records, and review queues that can be read quickly without treating BI rows as the system of record.

Why an operator should care: read models reduce dashboard reconciliation and manual investigation, but only if they preserve source/provenance and data-quality status so staff know when a number is clean enough for review.

What not to infer: do not infer that a read model is the source of record, can update Gingr, contains every operational field, or proves a business decision. It is optimized for reading and review, not live mutation or final authority.

Boundary and authority: analytics/storage projection boundary. It consumes normalized/source-grounded facts and exposes readable projections while leaving source-of-record and policy decisions with provider/domain/review owners.

Evidence and review hooks: `docs/integrations/gingr/bi-read-model-contract.md` documents the DTO -> snapshot -> source-agnostic snapshot -> analytics fact -> workflow-validator flow; `domain/src/analytics.rs` owns analytics facts; storage operation records carry persisted projections and source refs.

Safe example: a service-demand read model can show boarding/daycare demand trends with source refs and data-quality flags for a manager brief, but it cannot by itself change capacity, staff schedules, or reservation state.

Suggested public wording: a read model is a source-grounded view built for reporting or review. It helps managers read normalized evidence quickly, but it does not replace the source of record or authorize operational changes.

Related terms: `storage`, `provenance`, `source ref`, `DTO`, `domain`.
