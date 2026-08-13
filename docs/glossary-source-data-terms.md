# Glossary: source-system and data-quality terms

Purpose: public/non-coder glossary entries for source-system and data-quality vocabulary used in the NVA pet-resort Rust workspace. These entries translate repo terms into operator-facing meaning while preserving the code-derived contract: provider/source facts are evidence until explicitly mapped, promoted, reviewed, or projected by the owning layer.

Source basis for these entries:

- Entry shape and anti-overclaim rules: `docs/design/glossary-translation-layer.md`.
- Term inventory/source map: `docs/glossary-translation-layer-inventory.md`.
- Source/provenance contracts: `domain/src/source.rs`, `domain/README.md`.
- Data-quality contracts: `domain/src/data_quality.rs`, `docs/integrations/gingr/bi-read-model-contract.md`.
- Gingr provider boundary: `integrations/gingr/README.md`, `integrations/gingr/src/response.rs`, `integrations/gingr/src/dto/mod.rs`, `integrations/gingr/src/mapping/mod.rs`.
- Source-of-record/read-model posture: `docs/integrations/gingr/source-inventory.md`, `docs/integrations/gingr/bi-read-model-contract.md`.
- App evidence hooks: `app/src/manager_daily_brief.rs`, especially `SourceFact` and `SourceFactKind::SourceDataQualityIssue`.

These glossary entries are not new product promises. They should stay reviewed against Rustdoc/source before public wording is widened or reused elsewhere.

## `domain::source::System::Gingr` / Gingr

Term:
  `domain::source::System::Gingr`; Gingr as represented by `integrations/gingr`.

Plain-language label:
  Gingr operating-system evidence source.

Audience:
  Resort leaders, operators, product stakeholders, maintainers, and docs reviewers.

Where it appears:
  `domain/src/source.rs` defines `System::Gingr` as the Gingr reservation and pet-care operating system. `integrations/gingr/README.md` describes `integrations/gingr` as the provider adapter boundary for Gingr configuration, endpoint requests, transport seams, response envelopes, webhook verification, provider DTOs, and first mapping into semantic `domain::*` candidates. `README.md` maps the Gingr adapter, endpoint vocabulary, DTO boundary, semantic promotion, and source/provenance boundary.

Code-derived contract:
  The repo treats Gingr as an upstream source system and provider boundary. Gingr-specific ids, status strings, endpoint shapes, undocumented fields, webhooks, and API quirks can be captured by `integrations/gingr`; source lineage can name Gingr through `domain::source::System::Gingr`; usable provider fields can be explicitly mapped into domain candidates. The current adapter contract includes typed request/response shapes, DTOs, webhook verification, mapping errors, mock-safe transport seams, and explicit provider-surface gaps.

Pet-resort operational meaning:
  Gingr is the outside system where reservation, pet-care, owner/animal, catalog, webhook, timeclock/labor, and related operational evidence may originate. For a manager daily brief, checkout exception, data-hygiene queue, or labor/read-model projection, “Gingr” means “the provider evidence being cited or normalized,” not “the final NVA business truth.”

Why an operator should care:
  Naming Gingr as a source system makes review work traceable. A resort leader can ask which provider record, endpoint/export, webhook, or batch supported a recommendation instead of treating an automation draft as an unexplained assertion. It also keeps provider quirks from silently becoming local policy.

What not to infer:
  Do not infer that Gingr is NVA’s canonical domain model, that all Gingr surfaces are supported, that provider data is clean or complete, or that this repo currently performs live Gingr mutations. `integrations/gingr/README.md` states that live HTTP transport is not implemented in this slice, and the source-inventory docs say the reservation/stay slice is fixture/test oriented rather than a live Gingr or BI database integration.

Boundary and authority:
  `integrations/gingr` owns provider-facing request/response, DTO, webhook, transport, and mapping vocabulary. `domain::source::System::Gingr` is the domain source-system label used in provenance and record references. `domain` owns normalized business concepts; `storage` owns durable projections; `app` owns workflow packets and reviewable draft surfaces.

Evidence and review hooks:
  `domain/src/source.rs` provides the `System::Gingr` enum variant and provenance example. `integrations/gingr/README.md` describes provider facts vs domain truth and warns not to claim live HTTP support until transport implements it. `docs/integrations/gingr/source-inventory.md` records that no live credentials or data were used for its source inspection.

Suggested public wording:
  Gingr (`domain::source::System::Gingr`) is the provider operating-system source named in NVA source evidence. The repo can capture and map Gingr-shaped records for reviewable workflows, but Gingr facts are provider evidence until validated, promoted, and reviewed; they are not automatic permission to change reservations, schedules, payments, or customer messages.

Related terms:
  `integrations/gingr`, `gingr::response::*Record`, `gingr::dto::*`, `gingr::mapping::*`, `domain::source::Provenance`, `domain::source::RecordRef`, source-of-record, provider record.

## Provider record

Term:
  Provider record; examples include `gingr::response::{OwnerRecord, AnimalRecord, ReservationRecord, ReferenceRecord}` and provider-native ids such as `domain::source::gingr::ProviderRecordId`.

Plain-language label:
  Provider-native evidence record.

Audience:
  Operators, product stakeholders, maintainers, reviewers, and agent-doc writers.

Where it appears:
  `integrations/gingr/README.md` lists provider records in `gingr::response::{OwnerRecord, AnimalRecord, ReservationRecord, ReferenceRecord}` and warns that provider ids are not canonical domain ids. `docs/integrations/gingr/bi-read-model-contract.md` describes raw Gingr DTOs/records as what Gingr returned, including partial schemas and messy provider naming. `docs/integrations/gingr/source-inventory.md` names `source::gingr::ProviderRecordId` and `source::gingr::reservation::Snapshot` in the Gingr adapter path.

Code-derived contract:
  A provider record is a provider-shaped record retained at the adapter/source boundary before it becomes domain truth. It may preserve provider ids, raw statuses, unknown fields, endpoint-specific vocabulary, source payload identity, and related provider ids. It can be cited through provenance or mapped into domain candidates, but the mapping step must be explicit and can produce typed errors, assumptions, or data-quality issues.

Pet-resort operational meaning:
  A provider record is the “what Gingr said” row or payload behind a workflow claim: an owner record, animal record, reservation record, retail item, reference record, webhook payload, or similar provider fact. It helps staff trace a recommendation back to a concrete provider artifact while keeping provider vocabulary quarantined from NVA’s normalized operating language.

Why an operator should care:
  Staff and managers need to know whether a draft recommendation is based on a real upstream artifact and whether that artifact has been normalized. Provider records support traceability and repair: if an owner/pet relationship is ambiguous, a reservation status is unknown, or a payment state conflicts, the review queue can point back to the affected provider evidence.

What not to infer:
  Do not infer that a provider record is the canonical NVA customer, pet, reservation, invoice, payment, or service record. Do not infer that an agent may mutate it. Do not treat provider ids, raw statuses, or unknown fields as safe workflow facts unless a mapping/promotion function and associated tests or data-quality records say so.

Boundary and authority:
  `integrations/gingr` owns provider-shaped records and DTOs. `domain::source` owns source record identity and provenance. `domain` owns normalized business meaning after promotion. `storage` may later persist projections or snapshots, but persistence does not make a provider record the source-of-record for every operational decision.

Evidence and review hooks:
  `integrations/gingr/README.md` states that provider request/response shapes belong under endpoint/response/dto/webhook modules and that missing provider fields should become mapping errors, assumptions, data-quality issues, or documented provider-surface gaps. `docs/integrations/gingr/bi-read-model-contract.md` says raw DTOs must preserve provider ids/raw statuses/unknown fields and must not be serialized as the BI model or imported directly by dashboard/reporting code.

Suggested public wording:
  A provider record is the provider-native evidence the system received before NVA normalizes it. It can support a manager review or data-quality queue, but it is not the final NVA customer, pet, reservation, or payment truth until the relevant source and domain contracts explicitly promote it.

Related terms:
  Gingr, DTO, `domain::source::RecordRef`, `domain::source::Provenance`, source-of-record, data-quality issue, semantic promotion.

## Source-of-record

Term:
  Source-of-record.

Plain-language label:
  Authorized place or process for a specific fact or action.

Audience:
  Resort leaders, operators, product stakeholders, and docs reviewers.

Where it appears:
  `docs/integrations/gingr/source-inventory.md` is titled as a Gingr source inventory for source-of-record and BI read models and says Gingr should be treated as a provider/source-data boundary, not as a clean domain model. `docs/integrations/gingr/bi-read-model-contract.md` states that Gingr is an upstream operational source, not NVA’s domain model and not a clean BI warehouse. `README.md` and `storage/README.md` distinguish source/provider facts, domain truth, app workflows, and storage projections.

Code-derived contract:
  In this repo, source-of-record is not a single universal system label. It is a fact/action-specific authority boundary: provider/PMS facts may originate in Gingr; semantic business truth is modeled in `domain`; durable projections live in `storage`; workflow packets and draft/review boundaries live in `app`; final provider, payment, customer-message, medical, or manager decisions require the appropriate system-of-record or human review authority. BI/read models and draft packets are downstream evidence surfaces, not automatic write authority.

Pet-resort operational meaning:
  For each resort operation, the reader should ask “source-of-record for what?” Gingr may be the operational provider source for a reservation status, while a manager or staff review may be the authority for an exception, a local policy call, a refund/deposit decision, or a customer-facing message approval. A read model can help a manager see the situation, but it does not become the place where the reservation is changed.

Why an operator should care:
  Source-of-record clarity prevents unsafe automation and bad reporting. It keeps labor-saving drafts, BI rows, and manager briefs from being mistaken for authorized edits to the PMS, payments, medical/care records, or customer communications.

What not to infer:
  Do not infer that every clean-looking workflow output becomes source-of-record. Do not infer that BI/read models, analytics facts, source snapshots, manager daily briefs, or agent drafts can overwrite Gingr or bypass staff/manager/system approval. Do not infer that Gingr is the authoritative source for every NVA business concept just because some raw facts originate there.

Boundary and authority:
  Authority depends on the layer and decision: `integrations/gingr` captures provider evidence; `domain` defines semantic truth and invariants; `storage` stores durable projections; `app` creates reviewable workflow bundles and draft artifacts; runtime shells and humans/systems-of-record must enforce side-effect boundaries.

Evidence and review hooks:
  `docs/integrations/gingr/bi-read-model-contract.md` documents the allowed trust-boundary chain from Gingr API/report/webhook facts through raw DTOs, versioned snapshots, source-agnostic snapshots, analytics/read models, and future workflow validator inputs. It also states that analytics/read models are not the source of operational truth. `domain/src/data_quality.rs` and `app/src/manager_daily_brief.rs` carry blocking/review evidence without performing live action.

Suggested public wording:
  “Source-of-record” means the authorized owner for a specific fact or action. Gingr may provide reservation evidence, a read model may summarize it, and an app workflow may draft a review packet, but only the correct provider/system or human authority can make the live operational change.

Related terms:
  Gingr, provider record, read model, `domain`, `storage`, `app`, review gate, blocked action, outcome capture.

## `domain::data_quality::Issue` / data-quality issue

Term:
  `domain::data_quality::Issue`; data-quality issue.

Plain-language label:
  Source data exception that must remain visible.

Audience:
  Operators, resort leaders, product stakeholders, reviewers, and maintainers.

Where it appears:
  `domain/src/data_quality.rs` defines `FieldPath`, `Kind`, `Severity`, `ResolutionStatus`, and `Issue`. `domain/README.md` says data-quality types name missing or inconsistent data without burying problems in strings. `docs/integrations/gingr/bi-read-model-contract.md` says data-quality issues explain missing, ambiguous, contradictory, unmapped, stale, or unsafe source facts. `app/src/manager_daily_brief.rs` includes `SourceFactKind::SourceDataQualityIssue` and source refs on manager brief facts.

Code-derived contract:
  A data-quality issue is an evidence-backed domain value attached to source records and derived facts. It carries kind, severity, provenance, source record ref, detection time, resolution status, BI visibility, and workflow-blocking status. Issue kinds include missing required fields, assumptions in force, unknown source statuses, conflicting timestamps, duplicate source records, ambiguous owner/pet relationships, unmapped service types, location scope ambiguity, payment state conflicts, missing checkout evidence, unclosed reservations, incomplete pet profiles, missing vaccination records, and quarantined sensitive payloads.

Pet-resort operational meaning:
  A data-quality issue is a visible exception in the source-data chain: for example, a reservation cannot be safely projected because a pet record id is missing, a provider status is unknown, two timestamps conflict, a payment state does not reconcile, or vaccination evidence is missing. It should drive cleanup, manager review, BI caveats, or workflow blocking rather than disappear into logs or model text.

Why an operator should care:
  Data-quality issues prevent bad source facts from becoming unsafe automation, misleading labor reports, or customer-facing mistakes. They let managers distinguish “actual demand/staffing problem” from “source data needs repair” and reserve human review for records where the evidence is incomplete or contradictory.

What not to infer:
  Do not infer that a data-quality issue is just an engineering bug, a staff mistake, or something an AI can silently fix. Do not infer that a draft can mark an issue repaired, ignored, hidden, or superseded without reviewed outcome authority. A warning issue may allow reporting with caveats; a blocking issue must stop or route workflow according to the contract.

Boundary and authority:
  `domain::data_quality` owns the issue vocabulary and invariant-bearing value. Source/provenance values tie the issue to upstream evidence. App workflows can route, display, or include the issue in review packets. BI/read models may expose issue status. Actual resolution decisions require the appropriate review/outcome process and must not be hidden by provider mappings or agent drafts.

Evidence and review hooks:
  `domain/src/data_quality.rs` provides getters for severity, provenance, source record ref, detected-at timestamp, resolution status, BI visibility, and `workflow_blocking()`. `docs/integrations/gingr/bi-read-model-contract.md` says issues should be queryable by BI/reporting, audit/replay jobs, manager review queues, and workflow validators. `app/src/manager_daily_brief.rs` carries source record refs so daily-brief facts can cite source evidence without changing provider, customer, payment, or schedule state.

Suggested public wording:
  A data-quality issue (`domain::data_quality::Issue`) is a tracked source-data exception, such as missing checkout evidence, conflicting payment state, or an unknown provider status. It keeps the problem visible for BI, manager review, or workflow blocking instead of letting automation silently treat uncertain evidence as clean truth.

Related terms:
  `domain::source::Provenance`, `domain::source::RecordRef`, provider record, source-of-record, manager daily brief, BI/read model, review gate.

## `domain::source::Provenance` and `domain::source::RecordRef` as data evidence

Term:
  `domain::source::Provenance`; `domain::source::RecordRef`; provenance/source ref as data evidence.

Plain-language label:
  Evidence trail and source pointer for an operational fact.

Audience:
  Operators, resort leaders, product stakeholders, maintainers, reviewers, and agent-doc writers.

Where it appears:
  `domain/src/source.rs` defines `RecordRef` as a stable pointer to an upstream record and its owning system, and defines `Provenance` as lineage metadata tying normalized data back to its provider record. The Rustdoc example builds provenance with system, endpoint, record id, extraction batch, pulled-at time, request scope, schema version, payload hash, and raw payload ref, then derives a `RecordRef`. `README.md` maps `domain::source::{RecordRef, Provenance}` as the source/provenance boundary. `app/src/manager_daily_brief.rs` carries `source_record_refs` on `SourceFact`.

Code-derived contract:
  `Provenance` carries the chain of custody for a fact: source system, endpoint/import route, record id, related record ids, extraction batch, pull time, request scope, schema version, payload hash, and raw payload reference. `RecordRef` is the smaller stable pointer derived from provenance: source system plus record id. Both are evidence values; they identify where a fact came from and what raw/source context supports review.

Pet-resort operational meaning:
  When a manager brief, exception queue, labor/read-model projection, or agent draft says “this reservation looks checked out” or “this pet profile is incomplete,” provenance and source refs are the “show your work” trail. They let staff trace the statement back to the provider/import record and extraction context instead of accepting generated text or dashboard rows without evidence.

Why an operator should care:
  Evidence trails make automation auditable. They help staff verify sensitive recommendations, investigate data-quality issues, replay snapshots, and decide whether a problem is a real operational exception or a source-data cleanup item.

What not to infer:
  Do not infer that provenance proves the source data is correct, current, complete, or approved. Do not infer that a `RecordRef` contains the raw payload or authorizes mutation of the upstream record. Provenance can support review and mapping, but it does not approve customer sends, schedule changes, payment/refund actions, medical decisions, or policy overrides.

Boundary and authority:
  `domain::source` owns the lineage and source pointer contracts. Provider payloads remain at adapter/storage boundaries until promoted into semantic source/domain values. App workflows may cite provenance/source refs in packets and drafts, but runtime shells and humans/systems-of-record remain responsible for live side effects.

Evidence and review hooks:
  `domain/src/source.rs` has a compile-checkable Rustdoc example for building `Provenance` and deriving `RecordRef`. `domain/src/data_quality.rs` stores both provenance and source record ref on `Issue`. `app/src/manager_daily_brief.rs` exposes `SourceFact::has_source_evidence()` by checking whether source record refs are present.

Suggested public wording:
  `domain::source::Provenance` is the evidence trail for a normalized fact, and `domain::source::RecordRef` is the stable pointer to the upstream record. Together they let reviewers see which Gingr/import record and extraction context supported a draft or issue, without treating that evidence as automatic approval or live write permission.

Related terms:
  Gingr, provider record, source-of-record, data-quality issue, raw payload ref, payload hash, schema version, manager daily brief.

## Reviewer checklist for these entries

- Does each entry keep the source/Rust term visible rather than replacing it with a business-only synonym?
- Does each entry name the owning layer: `integrations/gingr`, `domain::source`, `domain::data_quality`, `app`, or `storage`?
- Does the entry separate provider evidence from normalized domain truth?
- Does the entry avoid implying live Gingr integration, customer sends, payment/refund action, medical approval, or provider mutation?
- Does “source-of-record” state the specific fact or action whose authority is being discussed?
- Does the data-quality entry preserve workflow blocking, BI visibility, and resolution status instead of treating issues as logs?
- Do provenance/source-ref claims say “evidence/traceability” rather than “proof/approval”?
