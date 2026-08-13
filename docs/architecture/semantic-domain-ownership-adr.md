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

`domain/src/strategic_ai_ops.rs` was useful to model missing strategic concepts quickly, but it now overlaps with existing owners. Keeping it as a permanent catch-all would create parallel concept ownership and make later refactors subjective.

## Decision

`strategic_ai_ops` is a bridge module only. New canonical owners must be service/domain/app/storage/provider modules whose names match the concept they own. Later implementation cards should move, bridge, or delete every strategic type according to the atlas.

Ownership defaults:

1. Stable business identity and aggregates belong in `domain::entities` or a concept-specific domain module.
2. Provider/source evidence and relationship confidence belong in `domain::source` and `domain::data_quality` until promoted.
3. Review authority belongs in `domain::policy` and `domain::entities::approval`.
4. Drafts, queue packets, ranked recommendations, and side-effect blocking belong in `app` workflows.
5. Storage codes/records are not domain authority; they convert explicitly to/from domain concepts.
6. Provider vocabulary is quarantined in `integrations/gingr` or another provider adapter.
7. Cross-workflow generic outcomes should not be generalized until at least two vertical slices prove a common invariant.

## Ownership decisions for strategic overlaps

| Strategic overlap | Decision | Canonical authority | Migration rule |
| --- | --- | --- | --- |
| `strategic_ai_ops::source::System` | Delete | `domain::source::System` | Add needed source variants to `domain::source::System` with source-code tests; replace strategic references; delete duplicate enum. |
| `strategic_ai_ops::time::Window` | Migrate | future `domain::time::Window` or owner-specific SLA/labor window | Introduce only with serde validation and end-after-start tests. Do not rely on `bon::Builder` for interval validity. |
| `strategic_ai_ops::access::ActorId` | Migrate | `domain::entities::ActorRef`, `StaffId`, `ManagerId`; possible `domain::access` for role scope | Convert actor identity through existing actor refs; keep role/title as access scope, not identity. |
| `strategic_ai_ops::access::{ActorRole, VisibilityScope, AllowedUse}` | Migrate | new `domain::access` or `domain::authorization` shared by CRM, assistant, and messages | Must be tied to consent/review gates and source evidence before marketing or assistant use. |
| `strategic_ai_ops::identity::Match` | Bridge then Migrate | `domain::source` promotion result or new `domain::identity` | Preserve candidate/ambiguous confidence; source snapshots remain authoritative for provider relationship proof. |
| `strategic_ai_ops::communication::Channel` | Delete | `domain::message::Channel` | Use one channel vocabulary at domain boundary; app/provider may adapt codes. |
| `strategic_ai_ops::communication::{Purpose, ConsentStatus, ConsentEvidence}` | Migrate | new `domain::consent` or `domain::message::consent` | Consent is per customer/channel/purpose with source refs and review state; contact preference is not consent. |
| `strategic_ai_ops::lead_response::*` | Migrate | new `domain::lead_response` or expanded `domain::lead` plus `domain::workflow`/`domain::entities::Message` | Event/SLA/attempt/conversion become canonical lead-response aggregate; attempts must link to message/draft approval before outbound use. |
| `strategic_ai_ops::crm::StructuredNote` | Migrate | new `domain::crm` or `domain::customer::intelligence` | Keep customer/pet note intelligence distinct from `entities::CareNote`; require source evidence, visibility, allowed use, review state, and consent for marketing. |
| `strategic_ai_ops::crm::SegmentMembership` | Migrate | same CRM/customer-intelligence owner | Segmentation must cite accepted notes/evidence and not infer marketing eligibility from operations-only facts. |
| `strategic_ai_ops::labor::{Minutes, SignedMinutes, PeopleCount, Role, ScheduledCoverage}` | Migrate | future `domain::operations::labor` plus `domain::staff` | Role/skill/shift/coverage facts must cite labor scheduling/timeclock source refs and must not mutate schedules. |
| `strategic_ai_ops::capacity::DemandUnit` | Bridge then Migrate | `domain::analytics::service_demand::Fact` for source demand; future capacity fact for bucketed units | Keep analytics demand canonical; add time-bucketed demand only if required by optimization slices. |
| `strategic_ai_ops::capacity::OptimizationRecommendation` | Bridge then Migrate | app manager brief for queue action now; future domain operations recommendation if reused | Recommendation consumes demand+coverage and carries manager gate; execution remains blocked. |
| `strategic_ai_ops::knowledge::{DocumentId, Document}` | Delete/Migrate | `domain::entities::DocumentId`, `domain::entities::Document`, `domain::document` | Do not create a second document identity. Knowledge metadata should reference core document IDs. |
| `strategic_ai_ops::knowledge::{Applicability, Citation}` | Migrate | future `domain::knowledge` or app assistant retrieval contract | Applicability scope must use location/service/role and approval/freshness; answer packets must cite approved documents or escalate. |
| `strategic_ai_ops::assistant::{ActorContext, AnswerPacket}` | Bridge | `app::agents` / future assistant app workflow; domain owns only policy/access/citation primitives | Assistant packets are runtime/use-case envelopes, not domain aggregates, unless they become durable audited decisions. |
| `strategic_ai_ops::financial::MoneyCents` | Delete | `domain::money::{Money, MinorUnits, Currency}` | Use checked money/currency values. No panicking arithmetic; no negative serde bypass. |
| `strategic_ai_ops::financial::{SitePeriod, RevenueFact, Insight}` | Migrate | future `domain::finance` or `domain::analytics::finance` | Facts need site/service/period/source refs and checked net arithmetic; insights are review-gated recommendations, not price/payment authority. |
| `strategic_ai_ops::outcome::Record` | Bridge then Migrate | app outcome records now; future `domain::outcome` after vertical slices | General outcome must cite reviewed action and source refs before supporting value claims. |

## Consequences

Positive:

- Later cards can mechanically reject duplicate owners.
- App/storage/provider boundaries stay narrow and reviewable.
- The strategic bridge remains useful as a test inventory while migration proceeds.
- Business-value claims stay tied to reviewed action and outcome evidence.

Costs:

- Several small canonical modules may be needed before feature work resumes.
- Existing strategic tests will need compatibility conversions or migration to new module paths.
- Some app-local outcome/consent concepts will be duplicated temporarily until canonical owners exist.

## Review checklist for later cards

- Does the change put the concept under the owner named in `semantic-domain-contract-atlas.md`?
- If a `strategic_ai_ops::*` type remains, is it explicitly a bridge with a removal target?
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
