# Entity-relevant architecture and Rust term inventory

Purpose: decide which architecture, Rust, and source-data terms should be translated for non-coder NVA docs because they clarify the Entity Atlas. This is not a second glossary and not a Rust style guide. Keep only terms that help a resort operator, reviewer, or docs writer understand entities, relationships, contracts, authority, evidence, safety/human-review boundaries, or labor-cost measurement.

Source basis checked:

- Glossary artifacts: [`../glossary.md`](../glossary.md), [`../glossary-architecture-terms.md`](../glossary-architecture-terms.md), [`../glossary-source-data-terms.md`](../glossary-source-data-terms.md), [`../glossary-workflow-state-terms.md`](../glossary-workflow-state-terms.md), [`../glossary-translation-layer-inventory.md`](../glossary-translation-layer-inventory.md), and [`glossary-translation-layer.md`](glossary-translation-layer.md).
- Entity Atlas artifacts: [`entity-atlas-inventory.md`](entity-atlas-inventory.md), [`entity-atlas-page-template.md`](entity-atlas-page-template.md), [`entity-atlas-relationships.md`](entity-atlas-relationships.md), plus drafted family pages under `docs/design/entity-atlas-*.md`.
- Repo/entity surfaces: [`../../README.md`](../../README.md), [`../../domain/README.md`](../../domain/README.md), [`../../app/README.md`](../../app/README.md), [`../../storage/README.md`](../../storage/README.md), [`../../integrations/gingr/README.md`](../../integrations/gingr/README.md), service-line READMEs under `domain/src/*/README.md`, and source modules under `domain/`, `app/`, `storage/`, and `integrations/gingr/`.
- Old-board handoffs found via session history for `nva-glossary-translation-layer`: glossary shape, source-map inventory, architecture/source/workflow glossary entries, and reading-guide examples. Those handoffs created the glossary artifacts named above; no separate repo-local old-board file was found.

## Selection rule

Translate a term only when it answers one of the Entity Atlas questions:

1. What entity or relationship does this term protect?
2. Who or what is authoritative for the fact or action?
3. What evidence must travel with the entity?
4. What can automation draft, rank, summarize, or record?
5. What must stay human-reviewed or blocked?
6. How is labor cost or rework measured?

If the term only helps a Rust maintainer read implementation mechanics, defer or drop it from non-coder docs.

## Keep decisions

These terms belong in the public glossary or nearby Entity Atlas pages because misunderstanding them creates real operator risk.

| Term | Related entity/entities | Concept family | Decision and primary placement | Source evidence | Reader risk if misunderstood |
| --- | --- | --- | --- | --- | --- |
| `domain` | All domain entities: Location, Customer, Pet, Reservation, Care, Vaccine, Incident, Message, Approval, Source, Workflow, Service-line contracts | Architecture / authority | Keep in glossary. Link from entity pages when explaining the source-of-truth business vocabulary. | `README.md`; `domain/README.md`; `domain/src/lib.rs`; `domain/src/entities.rs`; service modules | Reader may think raw provider rows, storage records, or app packets are canonical business truth. |
| `app` | Workflow packets, drafts, agent prompt packets, tool ports, outcome records | Architecture / workflow | Keep in glossary. Link from workflow pages and packet entity pages. | `README.md`; `app/README.md`; `app/src/{booking_triage,checkout_completion,crm_retention,daily_update,manager_daily_brief,agents,tools}.rs` | Reader may think the app layer is a live bot/front-end that can mutate Gingr, send messages, or move money. |
| `storage` | Storage projections, outcome records, source refs, service offering records, service-line contract records | Architecture / evidence persistence | Keep in glossary. Also mention on outcome/storage entity pages. | `README.md`; `storage/README.md`; `storage/src/operations.rs`; `storage/src/service_line/*` | Reader may treat database/projection shape as policy or source-of-record truth. |
| `integrations/gingr` / adapter | Gingr provider records, endpoint requests, DTOs, mappings, webhooks, source evidence | Source boundary / provider evidence | Keep in glossary and source/provenance Entity Atlas pages. | `README.md`; `integrations/gingr/README.md`; `integrations/gingr/src/{endpoint,response,dto,mapping,transport,webhook}.rs` | Reader may assume all Gingr facts are supported, normalized, live, or authoritative. |
| DTO / provider payload shape | Provider record, retail item, owner/animal/reservation records, provider-surface gaps | Source boundary / data normalization | Keep in glossary; repeat in Gingr provider-boundary pages. | `integrations/gingr/src/dto/README.md`; `integrations/gingr/src/dto/*.rs`; `integrations/gingr/src/response.rs` | High risk: DTO fields may be mistaken for safe customer/pet/reservation truth. |
| Provider record | Customer, Pet, Reservation, Retail product candidate, Reference records | Source boundary / evidence | Keep in glossary and source/provider pages. | `integrations/gingr/README.md`; `integrations/gingr/src/response.rs`; `docs/integrations/gingr/bi-read-model-contract.md` | “Record” sounds authoritative; readers may skip mapping, provenance, or data-quality checks. |
| Source-of-record | Reservation status, customer contact, payment/deposit, schedule, care/safety review, outcome evidence | Authority / review boundary | Keep in glossary; every entity page should ask “source of record for what?” | `docs/integrations/gingr/source-inventory.md`; `docs/integrations/gingr/bi-read-model-contract.md`; `README.md`; `storage/README.md` | Reader may believe a read model, draft, or manager brief can overwrite provider/PMS or human decisions. |
| `RecordRef` / source ref | Source system, Provenance, Data-quality issue, Workflow packet, Outcome record | Evidence / traceability | Keep in glossary and Source/Provenance atlas page. | `domain/src/source.rs`; `domain/README.md`; app workflow modules; `storage/src/operations.rs` | Reader may treat a pointer as proof, full payload, or write permission. |
| `Provenance` | Source system, provider record, data-quality issue, read model, workflow packet | Evidence / chain of custody | Keep in glossary and Source/Provenance atlas page. | `domain/src/source.rs`; `app/src/{checkout_completion,crm_retention,manager_daily_brief}.rs`; `storage/src/operations.rs` | Reader may infer source data is clean/current/approved because lineage exists. |
| Data-quality issue | Field path, source/ref/provenance, hygiene candidate, manager brief source fact | Source quality / review queue | Keep in glossary and Data Quality entity/workflow pages. | `domain/src/data_quality.rs`; `app/src/data_quality_hygiene.rs`; `app/src/manager_daily_brief.rs`; `storage/src/operations.rs` | Reader may treat uncertainty as an engineering bug or let automation hide/fix it without review. |
| Draft | Confirmation draft, customer message draft, send stub, audit draft, tool-port draft update | Workflow state / safe preparation | Keep in workflow glossary and workflow pages. | `app/README.md`; `app/src/{booking_triage,daily_update,checkout_completion,crm_retention,manager_daily_brief,tools}.rs` | High risk: “draft confirmation” may be mistaken for a sent/applied/approved action. |
| Review gate | Policy context, approval record, vaccine/medical/behavior/customer-message/refund/deposit review | Safety / authority | Keep in workflow glossary and every safety-sensitive entity page. | `domain/src/policy.rs`; `domain/src/workflow.rs`; `app/src/agents.rs`; workflow modules | Reader may treat the gate as a mere UI warning or self-check instead of required human/system approval. |
| Blocked action | Provider/PMS write, customer send, schedule change, payment/refund/discount movement, source hiding | Safety / boundary | Keep in workflow glossary and workflow/entity safety sections. | `app/src/{booking_triage,checkout_completion,crm_retention,manager_daily_brief,data_quality_hygiene}.rs`; `app/README.md` | Reader may think blocked means impossible, or worse, that a good-looking draft can bypass the boundary. |
| Workflow packet / review bundle | Booking triage, checkout completion, CRM retention, daily update, manager daily brief, data-quality hygiene packets | Workflow / relationship | Keep in workflow glossary; primary home is workflow/entity packet pages. | `app/README.md`; workflow source modules; app tests | Reader may think a packet is an executed job, message, provider mutation, or BI truth. |
| Agent spec / prompt packet | Agent helper, allowed tools, forbidden actions, review gates, workflow input/output | Agent contract / safety | Keep in workflow glossary and workflow packet/agent family page. | `domain/src/agent.rs`; `app/src/agents.rs`; `docs/architecture/agent-app-infrastructure.md` | Reader may assume a deployed autonomous bot or broad model authority. |
| Tool port | CustomerStore, ReservationSystem, AgentRuntime, availability, draft update, payment, messaging, documents, media, Hermes | Capability / runtime boundary | Keep in architecture glossary; mention on workflow pages only where capabilities matter. | `app/src/tools.rs`; `app/src/tools/README.md`; `app/README.md` | Reader may assume a live implementation exists or that a port can skip policy review. |
| Read model / projection | Analytics facts, service demand, stay facts, manager brief evidence, storage records | Reporting / evidence | Keep in architecture/source glossary and source/outcome pages. | `docs/integrations/gingr/bi-read-model-contract.md`; `storage/README.md`; `storage/src/operations.rs`; `domain/src/analytics.rs` | Reader may treat BI/read projections as operational source-of-record or write authority. |
| Outcome capture / outcome record | ManagerDailyBriefOutcomeRecord, DataQualityHygieneOutcomeRecord, CRM OutcomeRecord, labor minutes | Labor measurement / audit | Keep in workflow glossary and outcome/labor entity pages. | `app/src/manager_daily_brief.rs`; `app/src/data_quality_hygiene.rs`; `app/src/crm_retention.rs`; `storage/src/operations.rs`; API/storage tests | Reader may mistake estimated value for observed labor savings or think outcome capture performs the underlying action. |
| Labor minutes / labor impact estimate | Outcome records, manager actions, data-quality hygiene actions, reporting groups | Labor measurement | Keep as Entity Atlas/outcome-page term, not necessarily global glossary unless it recurs in public docs. | `app/src/manager_daily_brief.rs`; `app/src/data_quality_hygiene.rs`; `storage/src/operations.rs`; labor-loop docs | Reader may overclaim ROI without reviewed outcome evidence. |
| Approval record | Review gate, customer message, vaccine/document approval, audit subject | Safety / durable human decision | Keep as Entity Atlas page or review-safety page; link from glossary review-gate entry. | `domain/src/entities.rs`; `app/src/daily_update.rs`; API tests; workflow approval docs | Reader may confuse “requires approval” with “approval was recorded.” |
| Audit event / audit draft | Workflow event, approval record, source evidence, outcome | Evidence / accountability | Keep as Entity Atlas relationship term; link from workflow packet/outcome pages. | `domain/src/audit.rs`; `domain/src/entities.rs`; `domain/src/workflow.rs`; app audit draft types | Reader may lose the trail from source fact to human decision to measured outcome. |
| Mapping / promotion candidate | Customer contact candidate, pet name candidate, retail product candidate, mapping error | Source normalization / domain promotion | Keep on provider/source entity pages; likely not a global glossary term unless public docs use it often. | `integrations/gingr/src/mapping/*.rs`; `integrations/gingr/src/mapping/README.md`; `domain/src/customer.rs`; `domain/src/pet.rs`; `domain/src/retail/*` | Reader may think provider fields become domain facts by default or that missing fields can be guessed. |
| Provider-surface gap | Grooming/training service DTO gaps, catalog gaps, unsupported provider surface | Source boundary / evidence gap | Keep as source/provider page note; not a standalone glossary entry unless exposed publicly. | `integrations/gingr/src/dto/mod.rs`; `integrations/gingr/src/dto/{grooming,training}.rs`; `integrations/gingr/src/endpoint/catalog.rs` | Reader may assume a provider DTO or integration exists when the code intentionally records a gap. |
| Transport seam / mock transport / live HTTP not implemented | Gingr provider client, runtime shells, tests | Runtime safety / capability boundary | Keep as Gingr/runtime page note; not a main public glossary term. | `integrations/gingr/src/transport.rs`; `integrations/gingr/README.md`; integration tests | Reader may overclaim live Gingr read/write capability. |
| Storage `*Record`, `*Code`, `Stored*`, codec, promotion/demotion | Storage operations, service-line contract records, outcome records | Persistence / conversion boundary | Keep as storage/runtime entity-page relationship terms; avoid broad public glossary unless used in public docs. | `storage/README.md`; `storage/src/operations.rs`; `storage/src/service_line/*` | Reader may think storage codes are alternate domain policy or provider data. |

## Defer decisions

These are real repo concepts, but they should be documented inside the closest Entity Atlas page or workflow page instead of promoted into the top-level glossary now.

| Term | Related entity/entities | Concept family | Decision and placement | Source evidence | Reader risk if misunderstood |
| --- | --- | --- | --- | --- | --- |
| Typestate request / `Request<S>` / builder stages | Booking triage request, policy decision staging | Rust/API shape | Defer to Rustdoc and Booking Triage packet page. Mention only as “required evidence is collected before policy decision.” | `app/src/booking_triage.rs`; `app/README.md` | Non-coders do not need generated type-state names; the entity page should explain readiness stages. |
| `nutype` validated scalar | Names, IDs, dates, money, labor minutes, storage scalars | Rust invariant implementation | Defer to maintainer docs/Rust conventions. Entity pages should say “validated value” only when it affects operator trust. | `README.md` Rust quality conventions; many domain/storage files | If overexposed, readers may focus on Rust mechanics instead of why invalid values are rejected. |
| `bon` builders | Multi-field constructors, workflow packets, provenance, records | Rust construction ergonomics | Defer to Rustdoc/maintainer docs. | `README.md` Rust quality conventions; source call sites | Not needed for non-coder entity understanding. |
| `statum` generated transition/builder names | Booking request state transitions | Rust generated implementation | Drop from public Entity Atlas; defer to Rustdoc only. | `README.md`; `app/src/booking_triage.rs`; QA audit notes | Generated names can obscure the operating meaning: collect evidence before policy decision. |
| `Result<T>`, `Error`, `CodecError`, local error enums | Workflow/storage/provider failure reporting | Error contract | Defer to Rustdoc unless an error becomes an operator-facing review reason. | `domain/src/workflow.rs`; `storage/src/operations.rs`; `integrations/gingr/src/mapping/mod.rs`; `app/src/tools/error.rs` | Public pages should explain the business failure/review reason, not generic Rust error plumbing. |
| Endpoint primitives (`Method`, `Path`, `Limit`, `DateRange`) | Gingr endpoint requests | Provider request implementation | Defer to Gingr endpoint/source page. | `integrations/gingr/src/endpoint/mod.rs`; endpoint README | Important for maintainers, but not entity-first except as evidence that provider access is typed. |
| Provider id wrappers (`OwnerId`, `AnimalId`, etc.) | Provider record, mapping, source ref | Provider boundary | Defer to Gingr provider-boundary page; mention that provider IDs are not domain IDs. | `integrations/gingr/src/endpoint/mod.rs`; `integrations/gingr/README.md` | Needs boundary warning but not standalone glossary treatment. |
| Raw payload ref, payload hash, schema version, extraction batch | Provenance | Evidence detail | Defer under Provenance page. | `domain/src/source.rs`; source glossary | If promoted alone, readers may treat audit metadata as business truth. |
| Field path / field segment | Data-quality issue | Data-quality evidence detail | Defer under Data Quality Issue/Hygiene page. | `domain/src/data_quality.rs`; `app/src/data_quality_hygiene.rs` | Useful for cleanup precision, but too granular for global glossary. |
| Runtime shell / API router / worker runtime / CLI shell | Apps/API, worker, CLI | Runtime exposure | Defer to runtime/storage/API Entity Atlas page. | `apps/api/README.md`; `apps/worker/README.md`; `apps/cli/README.md`; source files | Reader may think shells own domain models unless page says they expose app/domain contracts. |
| Webhook envelope / verified event | Gingr source events | Provider event evidence | Defer to Gingr provider-boundary page and future event-driven workflow pages. | `integrations/gingr/src/webhook.rs`; webhook tests/docs | Important safety term, but only relevant where event-driven provider facts appear. |

## Drop decisions for non-coder Entity Atlas docs

These terms should not be translated for public/non-coder docs unless a specific source page needs them for a reviewer.

| Term | Why drop from Entity Atlas glossary | Safer replacement when needed |
| --- | --- | --- |
| Crate, module, prelude, trait, impl, enum, struct, newtype | General Rust vocabulary; useful to maintainers but not entity-first. | Link source path and use “source file,” “type,” or “contract” sparingly. |
| Generic serde/JSON codec mechanics | Implementation detail unless a storage/projection failure is being explained. | “Stored record can be encoded/decoded and validated before use.” |
| Generated builder/type-state internals such as `__StatumRequest...MissingSlot...` | Actively distracts non-coders from the workflow readiness concept. | “The request must collect pet profile and policy evidence before policy decision.” |
| Raw HTTP status/body details | Provider adapter internals except on Gingr runtime/source pages. | “Provider response evidence” or “transport result.” |
| Generic `String`, `Option`, `Vec`, `HashMap`, flattened unknown-field mechanics | Not entity-relevant by itself. | “Optional provider field,” “unknown provider field retained as drift evidence.” |
| Test fixture names, local smoke helper wrappers, internal fake-data constructors | Evidence/support, not entities. | Link tests or smoke docs as verification evidence. |
| Package-manager/workspace mechanics | Not part of business entity or authority model. | Keep in contributor docs if needed. |

## Dependencies on Entity Atlas naming and template

Future glossary/entity work should follow these dependencies:

1. Prefer Entity Atlas page names for business concepts and glossary names for cross-cutting translation terms.
   - Entity page: “Reservation,” “Data Quality Issue,” “Manager Daily Brief Outcome Record,” “Gingr Provider Boundary.”
   - Glossary term: `domain`, `app`, `storage`, DTO, provenance, review gate, blocked action.
2. Every kept architecture/Rust term should appear in an Entity Atlas page only in one of the template slots:
   - “Source paths” / “Rustdoc/module/type contracts” for code authority.
   - “Source of record” for authority boundaries.
   - “Allowed actions” and “Blocked actions / review gates” for automation limits.
   - “Safe-use evidence / outcome fields” for provenance, source refs, data-quality status, and labor minutes.
   - “Glossary cross-links” for reusable explanations.
3. Avoid making architecture terms the page title when an entity title is clearer. Use “Workflow packets, agents, drafts, and review queues” rather than a page named only “app.” Use “Source, provenance, and data quality” rather than a page named only “domain::source.”
4. Preserve source paths when they carry authority. A public page can say “source evidence (provenance)” in the body, but its evidence table should still name `domain/src/source.rs` and `domain::source::Provenance`.
5. Do not promote implementation helpers into standalone pages unless they answer a non-coder Entity Atlas question. For example, `FieldPath` belongs under Data Quality Issue; `PayloadHash` belongs under Provenance; `ProviderField` belongs under Mapping Candidate/provider boundary.

## Recommended next edits

1. Keep the current top-level glossary focused on the already-kept terms: architecture layers, source evidence/data quality, workflow/review/operator state.
2. Add Entity Atlas glossary cross-links from family pages to these entries where first used: source-of-record, provenance/source ref, data-quality issue, draft, review gate, blocked action, workflow packet, outcome capture.
3. When drafting final entity pages, use this inventory as a filter: if a source/Rust term is in “defer,” explain the operating meaning in the page body and leave the exact Rust machinery to source/Rustdoc.
4. Review future public docs for dropped terms before publication; if generated Rust or provider plumbing appears in operator-facing prose, replace it with the relevant entity relationship or evidence boundary.
