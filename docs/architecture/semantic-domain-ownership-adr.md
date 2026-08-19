# ADR: Semantic domain ownership for strategic AI operations

Status: accepted for the semantic-domain-hardening board.
Date: 2026-08-12.

## Context

The repo already has strong semantic owners for core pet-resort operations:

- `domain::entities` owns stable location, customer, pet, reservation, document, vaccine, care-note, incident, message, approval, audit, and actor aggregates.
- `domain::source` owns provenance, source record refs, provider/source IDs, source reservation snapshots, relationship evidence, and source promotion blockers.
- `domain::policy` owns review gates and automation authority levels.
- `domain::workflow` owns workflow events, allowed actions, recommended actions, and reviewable results.
- Service modules (`domain::boarding`, `domain::daycare`, `domain::grooming`, `domain::training`, `domain::retail`) own service-line rules.
- `domain::analytics` owns normalized source-backed projections such as service demand facts.
- `domain::money`/`domain::payment` own money/deposit/payment scalars and lifecycle values.
- `app` owns workflow packets, drafts, orchestration services, safe/blocked action lists, and per-workflow outcome feedback.
- `storage` owns persisted records/codes/codecs/projections and explicit conversion with domain types.
- `integrations/gingr` owns provider DTOs/endpoints/mapping/transport/webhook quirks.

An earlier catch-all modeled several concepts before their owners existed. The current tree removed that parallel public surface after moving each retained definition to its semantic owner.

## Decision

Canonical owners are service/domain/app/storage/provider modules whose names match the concept they own. No compatibility facade or bridge module remains authorized in the clean-slate public surface.

The current layer, conversion, repository, compatibility-codec, and authority-issuer assignments are maintained in the [bounded-context ownership map](bounded-context-map.md). That map is the navigation index; this ADR remains the decision record for why semantic ownership cannot collapse into a strategic catch-all.

Ownership defaults:

1. Stable business identity and aggregates belong in `domain::entities` or a concept-specific domain module.
2. Provider/source evidence and relationship confidence belong in `domain::source` and `domain::data_quality` until promoted.
3. Review authority belongs in `domain::policy` and `domain::entities::approval`.
4. Drafts, queue packets, ranked recommendations, and side-effect blocking belong in `app` workflows.
5. Storage codes/records are not domain authority; they convert explicitly to/from domain concepts.
6. Provider vocabulary is quarantined in `integrations/gingr` or another provider adapter.
7. Cross-workflow generic outcomes should not be generalized until at least two vertical slices prove a common invariant.

## Resolved ownership decisions

| Concept family | Resolution | Canonical authority | Current rule |
| --- | --- | --- | --- |
| Source and identity evidence | Canonicalized | `domain::source`, `domain::identity` | Provider evidence is promoted fallibly; ambiguity remains observed/candidate evidence. |
| Access and consent | Canonicalized | `domain::access`, `domain::consent`, `domain::policy` | Serializable roles and consent labels do not authenticate identity or issue permission. |
| Lead response and customer intelligence | Canonicalized | `domain::lead::response`, `domain::customer::intelligence` | Queue/contact and note-acceptance authority remain opaque and narrowly issued. |
| Labor and capacity | Canonicalized | `domain::operations::{labor,capacity,time_bucket}`, `domain::analytics::service_demand` | Recommendations require source relationships and manager gates; no schedule mutation authority exists. |
| Knowledge and assistant packets | Canonicalized | `domain::agent::{knowledge,assistant}`, core document owners | Missing approval, applicability, or citations force escalation. |
| Finance, money, and outcomes | Canonicalized | `domain::money`, `domain::payment`, `domain::analytics::{finance,outcome}` plus workflow-specific app outcomes | Caller-created outcome labels do not prove completion, measurement, savings, payment, or value. |

## Consequences

Positive:

- Later cards can mechanically reject duplicate owners.
- App/storage/provider boundaries stay narrow and reviewable.
- The public surface has one current owner per retained concept.
- Business-value claims stay tied to reviewed action and outcome evidence.

Costs:

- Several small canonical modules may be needed before feature work resumes.
- Tests with historical filenames remain executable contract coverage; their names do not authorize a parallel API.
- Workflow-specific outcome evidence remains app-owned where its invariants are not genuinely shared.

## Review checklist for later cards

- Does the change put the concept under the owner named in `semantic-domain-contract-atlas.md`?
- Does the change avoid reintroducing a parallel facade or historical type path?
- Does every source/provider/storage promotion use `TryFrom` or a named fallible constructor when validation/trust/authorization changes?
- Does serde validate the same invariants as constructors?
- Does every recommendation/action carry review gates and blocked side effects?
- Does every claimed outcome cite reviewed action proof and source evidence?
- Does the implementation avoid live customer, provider/PMS/CRM, staffing/schedule, money/payment, production, or unsupported value-claim side effects?

## Unresolved owner decisions

No blocking product-owner decision remains. The following are implementation choices for later cards:

1. Whether to name the consent module `domain::consent` or nest it under `domain::message`.
2. Whether lead response becomes `domain::lead_response` or an expanded `domain::lead` module.
3. Whether generalized outcome becomes `domain::outcome` after the next vertical slice or remains app-local longer.
4. Whether financial facts live under `domain::finance` or `domain::analytics::finance`.

These can safely default to the most semantically narrow module when each implementation card starts.
