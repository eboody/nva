# Core entity-linked glossary draft

Purpose: draft the architecture/Rust glossary entries that most affect non-coder understanding of NVA entities, workflow contracts, source authority, review safety, and labor evidence. This file applies the [entity-linked glossary entry template](entity-linked-glossary-entry-template.md) to the highest-risk terms from the [entity-relevant architecture and Rust term inventory](entity-relevant-architecture-rust-term-inventory.md).

Status: draft for review. These entries are written as a bridge between the public glossary files and the Entity Atlas. They should be folded into `docs/glossary-architecture-terms.md`, `docs/glossary-source-data-terms.md`, `docs/glossary-workflow-state-terms.md`, or the nearest Entity Atlas family page after review.

Review rule: every entry below either links to an entity family or explicitly says it is a cross-cutting support term. Source files, module/Rustdoc paths, and tests remain authoritative.


Link-map note: the downstream-consumable cross-link map for this pass lives at [Glossary/entity-atlas cross-link map](glossary-entity-atlas-link-map.md). It records which glossary targets should be linked from entity atlas, workflow, safety, README/public, and Rustdoc/operator surfaces, plus the stub convention for missing future entity pages.

## Entity / domain truth

Term in code/docs:
  `domain` entities and domain modules, especially `domain::entities::*`, `domain::reservation`, `domain::boarding`, `domain::daycare`, `domain::grooming`, `domain::training`, `domain::retail`, `domain::policy`, and `domain::workflow`.

Plain-English operational translation:
  A domain entity is the repo's normalized pet-resort meaning for a business thing: a location, customer, pet, reservation, care profile, service-line contract, review gate, workflow event, or outcome-related value. It is the language a workflow should use after provider evidence has been mapped, validated, or marked as uncertain.

Why this matters to operators / product / IT:
  Domain truth prevents raw Gingr rows, storage codes, BI projections, or agent text from becoming business authority by accident. It lets readers ask whether a workflow is using the NVA concept of a reservation, pet, vaccine, policy gate, or labor outcome rather than a provider-specific field.

Linked entity/entities:
  Core entity families in [PetSuites core entity atlas](entity-atlas-petsuites-core-entities.md), [Revenue opportunity entity families](entity-atlas-revenue-opportunity-entities.md), [Review gates, blocked actions, and human approval boundaries](entity-atlas-review-safety-boundaries.md), and [Outcome, labor, operations, analytics, money, and safety evidence atlas](entity-atlas-outcomes-operations-money.md).

Linked workflows / contracts:
  [Workflow-to-entity navigation map](workflow-to-entity-navigation-map.md), [entity relationship map](entity-atlas-relationships.md), [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), and workflow modules such as `app::booking_triage`, `app::checkout_completion`, `app::crm_retention`, `app::daily_update`, `app::manager_daily_brief`, and `app::data_quality_hygiene`.

Authoritative source / Rustdoc / test evidence:
  [`domain/README.md`](../../domain/README.md), [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/policy.rs`](../../domain/src/policy.rs), [`domain/src/workflow.rs`](../../domain/src/workflow.rs), service-line modules under `domain/src/{boarding,daycare,grooming,retail,training}`, and the source/Rustdoc paths named in [entity atlas inventory](entity-atlas-inventory.md#family-reservations-boarding-daycare-and-core-pet-resort-entities).

Automation boundary:
  - May draft/recommend/validate/record: compose domain values into review packets, apply deterministic policy checks, validate readiness, cite required review gates, and record reviewed outcomes where an app/storage contract exists.
  - Blocked / human-reviewed: treating a domain entity mention as a live provider/PMS write, customer send, payment movement, schedule/capacity change, medical/safety approval, or proof that a workflow was executed.

Common confusion / what not to infer:
  Do not collapse domain truth into generic “data.” A provider record can support a domain entity, a storage record can project it, and a workflow packet can carry it, but those are different authority layers.

Link targets:
  [Glossary: `domain`](../glossary-architecture-terms.md#domain), [entity relationship map](entity-atlas-relationships.md), [workflow-to-entity navigation map](workflow-to-entity-navigation-map.md), [review boundaries matrix](../safety/review-boundaries-matrix.md), and [source evidence map](../safety/source-evidence-map.md).

Suggested public wording:
  A domain entity is NVA's normalized pet-resort meaning for a business concept such as a pet, reservation, vaccine record, review gate, or outcome. It is not a raw provider row, database shape, or agent draft; it is the semantic vocabulary workflows use when evidence has been mapped and safety boundaries are visible.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Source/provider fact

Term in code/docs:
  Source fact, provider fact, `domain::source::{System, Provenance, RecordRef}`, `domain::source::gingr::*`, and Gingr provider records/DTOs.

Plain-English operational translation:
  A source/provider fact is what an upstream system, import, webhook, staff note, or fixture says before NVA treats it as normalized business truth. It is evidence to inspect, map, and cite.

Why this matters to operators / product / IT:
  Operators need to know when a workflow claim is backed by a provider record and when it has been promoted into a safe domain input. That distinction prevents unsafe reservation, payment, customer-message, or safety decisions from being made from raw provider evidence alone.

Linked entity/entities:
  [Source, provenance, and data-quality atlas](source-provenance-data-quality-atlas.md), [Gingr provider boundary atlas](../integrations/gingr/provider-boundary-atlas.md), and source/provider rows in [entity atlas inventory](entity-atlas-inventory.md#family-source-provenance-and-data-quality).

Linked workflows / contracts:
  [Gingr/source-provider normalization crosswalk](../entity-atlas/contract-crosswalk/source-provider-flows.md), [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), Booking Triage, Checkout Completion, CRM Retention, Manager Daily Brief, and Data Quality Hygiene.

Authoritative source / Rustdoc / test evidence:
  [`domain/src/source.rs`](../../domain/src/source.rs), [`domain/src/data_quality.rs`](../../domain/src/data_quality.rs), [`integrations/gingr/src/response.rs`](../../integrations/gingr/src/response.rs), [`integrations/gingr/src/dto/mod.rs`](../../integrations/gingr/src/dto/mod.rs), [`integrations/gingr/src/mapping/mod.rs`](../../integrations/gingr/src/mapping/mod.rs), [`domain/tests/reservation_source_contracts.rs`](../../domain/tests/reservation_source_contracts.rs), and [`integrations/gingr/tests/expanded_endpoint_contracts.rs`](../../integrations/gingr/tests/expanded_endpoint_contracts.rs).

Automation boundary:
  - May draft/recommend/validate/record: cite source refs, preserve provenance, map provider candidates through named mappers, flag data-quality issues, and summarize evidence for review.
  - Blocked / human-reviewed: provider/PMS writes, source record hiding/deletion, unsupported provider-surface assumptions, or treating raw provider values as customer/pet/reservation/payment truth without mapping and review.

Common confusion / what not to infer:
  Provider evidence is not domain truth. Provenance and source refs help reviewers trace evidence; they do not prove the provider is correct or authorize live action.

Link targets:
  [Glossary: Gingr/source](../glossary-source-data-terms.md#domainsourcesystemgingr-gingr), [provider record](../glossary-source-data-terms.md#provider-record), [source-of-record](../glossary-source-data-terms.md#source-of-record), [source evidence map](../safety/source-evidence-map.md), and [source/provider flows](../entity-atlas/contract-crosswalk/source-provider-flows.md).

Suggested public wording:
  A source/provider fact is evidence from Gingr, an import, a webhook, a staff note, or another upstream source. It can support a workflow only after the relevant source, mapping, provenance, data-quality, and review contracts say how it may be used.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## DTO / provider payload shape

Term in code/docs:
  DTO, provider payload shape, `gingr::dto::*`, `gingr::response::*Record`, and provider-specific endpoint/response/webhook envelopes.

Plain-English operational translation:
  A DTO is the provider-shaped package the integration can read or deserialize. It says “this is what Gingr or another provider returned,” not “this is now safe NVA business truth.”

Why this matters to operators / product / IT:
  DTOs keep messy provider data visible at the edge. Optional fields, unknown fields, unsupported surfaces, and provider ids can become review or mapping evidence instead of silently entering customer, pet, reservation, retail, or reporting workflows.

Linked entity/entities:
  [Gingr provider boundary atlas](../integrations/gingr/provider-boundary-atlas.md), provider-boundary rows in [entity atlas inventory](entity-atlas-inventory.md#family-gingr-dto-mapping-endpoint-and-provider-boundary), and [Source, provenance, and data-quality atlas](source-provenance-data-quality-atlas.md).

Linked workflows / contracts:
  [Source/provider flows](../entity-atlas/contract-crosswalk/source-provider-flows.md), Data Quality Hygiene, Retail source normalization, Booking Triage, Checkout Completion, and Manager Daily Brief where source refs/provenance enter app packets.

Authoritative source / Rustdoc / test evidence:
  [`integrations/gingr/src/dto/README.md`](../../integrations/gingr/src/dto/README.md), [`integrations/gingr/src/dto/retail.rs`](../../integrations/gingr/src/dto/retail.rs), [`integrations/gingr/src/dto/grooming.rs`](../../integrations/gingr/src/dto/grooming.rs), [`integrations/gingr/src/dto/training.rs`](../../integrations/gingr/src/dto/training.rs), [`integrations/gingr/src/response.rs`](../../integrations/gingr/src/response.rs), [`integrations/gingr/src/webhook.rs`](../../integrations/gingr/src/webhook.rs), and Gingr DTO/webhook tests under `integrations/gingr/tests`.

Automation boundary:
  - May draft/recommend/validate/record: deserialize provider evidence, preserve unknown fields, mark provider-surface gaps, pass supported fields into explicit mapping candidates, and cite DTO evidence in review packets.
  - Blocked / human-reviewed: assuming DTO field names are NVA policy, guessing missing fields, using unsupported provider surfaces, or treating provider ids/raw status strings as domain ids/statuses.

Common confusion / what not to infer:
  A DTO is not an entity just because it has fields. It is boundary evidence until mapper/source contracts promote the supported facts or record a gap.

Link targets:
  [Glossary: DTO](../glossary-architecture-terms.md#dto), [provider record](../glossary-source-data-terms.md#provider-record), [Gingr provider boundary atlas](../integrations/gingr/provider-boundary-atlas.md), and [source/provider flows](../entity-atlas/contract-crosswalk/source-provider-flows.md).

Suggested public wording:
  A DTO is a provider payload shape. It captures what the provider returned at the boundary, but NVA does not treat it as a customer, pet, reservation, retail, or payment truth until mapping and review contracts say which facts are safe.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Workflow packet / review bundle

Term in code/docs:
  Workflow packet, `app::*::Packet`, `app::booking_triage::StaffEvaluationPacket`, `app::agents::AgentPromptPacket<T>`, and workflow-local staff/manager review packets.

Plain-English operational translation:
  A workflow packet is the review bundle for one job: it gathers source-backed facts, domain entities, policy context, suggested/draft actions, review gates, blocked actions, and outcome hooks so a person or shell can review work without reinterpreting raw source data.

Why this matters to operators / product / IT:
  Packets are the labor-saving middle layer. They reduce dashboard hunting and repetitive draft writing, but they also show where automation stops before live customer/provider/schedule/payment/safety action.

Linked entity/entities:
  [Workflow packets, agents, drafts, and review queues](entity-atlas-workflow-packets-agents.md), [Review gates, blocked actions, and human approval boundaries](entity-atlas-review-safety-boundaries.md), and the workflow rows in [workflow-to-entity navigation map](workflow-to-entity-navigation-map.md#workflow---entity-matrix).

Linked workflows / contracts:
  [Workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), Booking Triage, Data Quality Hygiene, Checkout Completion, Grooming Rebooking / Retention, Daily Updates / Pawgress, Manager Daily Brief, and planned Regional Labor Exceptions.

Authoritative source / Rustdoc / test evidence:
  [`app/README.md`](../../app/README.md), [`app/src/agents.rs`](../../app/src/agents.rs), workflow modules under `app/src`, workflow tests under `app/tests`, and API contract tests under `apps/api/tests` named in [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map).

Automation boundary:
  - May draft/recommend/validate/record: rank work, summarize facts, build drafts, validate requested side effects, request human review, and capture outcomes according to the packet contract.
  - Blocked / human-reviewed: treating the packet as an executed transaction, customer send, provider/PMS write, booking/check-in/out mutation, BI truth, final approval, or payment/schedule/safety authority.

Common confusion / what not to infer:
  A packet is not a queued job or completed workflow. It is a typed review shape; separate app, shell, adapter, storage, and human authority decide what happens next.

Link targets:
  [Glossary: workflow packet](../glossary-workflow-state-terms.md#workflow-packet), [entity relationship map](entity-atlas-relationships.md), [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), and [review boundaries matrix](../safety/review-boundaries-matrix.md).

Suggested public wording:
  A workflow packet is a source-backed review bundle for a specific resort workflow. It can organize evidence and prepare drafts or recommendations, but it is not permission to act live without the required review and source-of-record authority.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Review gate / approval boundary

Term in code/docs:
  Review gate, `domain::policy::ReviewGate`, `domain::workflow::PolicyContext`, workflow-local `required_review_gates()`, approval record.

Plain-English operational translation:
  A review gate is the named stop that says a sensitive step must be approved by the right person, role, or system-of-record process before it can happen.

Why this matters to operators / product / IT:
  It keeps pet safety, medical/vaccine review, behavior review, customer-message approval, refunds/deposits, source cleanup, schedule/capacity, and local policy exceptions visible instead of buried as implementation warnings.

Linked entity/entities:
  [Review gates, blocked actions, and human approval boundaries](entity-atlas-review-safety-boundaries.md), Approval Record in [entity atlas inventory](entity-atlas-inventory.md#family-review-gates-and-blocked-actions), and the safety sections of workflow/entity pages.

Linked workflows / contracts:
  Every app workflow packet in [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), [review boundaries matrix](../safety/review-boundaries-matrix.md), and [evidence policy / blocked actions / outcomes](../safety/evidence-policy-blocked-actions-outcomes.md).

Authoritative source / Rustdoc / test evidence:
  [`domain/src/policy.rs`](../../domain/src/policy.rs), [`domain/src/workflow.rs`](../../domain/src/workflow.rs), [`domain/src/entities.rs`](../../domain/src/entities.rs) approval/audit entities, [`app/src/agents.rs`](../../app/src/agents.rs), workflow modules under `app/src`, and workflow/API tests linked from [surface inventory](../entity-atlas/contract-crosswalk/surface-inventory.md).

Automation boundary:
  - May draft/recommend/validate/record: list required reviews, route a packet, explain the needed approval, prepare evidence, and record a reviewed disposition where implemented.
  - Blocked / human-reviewed: assuming that naming a gate is the same as approval, letting an agent proceed because the gate appears in text, or skipping local policy/source-system permission.

Common confusion / what not to infer:
  A review gate is not the approval record. It is the requirement for review; approval/rejection/disposition needs a separate human/system outcome.

Link targets:
  [Glossary: review gate](../glossary-workflow-state-terms.md#review-gate), [review/safety atlas](entity-atlas-review-safety-boundaries.md), [review boundaries matrix](../safety/review-boundaries-matrix.md), and [blocked action](../glossary-workflow-state-terms.md#blocked-action).

Suggested public wording:
  A review gate (`domain::policy::ReviewGate`) is a named human-approval stop. It lets workflows prepare evidence and drafts while keeping manager approval, medical/behavior review, customer-message approval, and refund/deposit exceptions outside automatic execution.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Blocked action

Term in code/docs:
  Blocked action, workflow-local `BlockedAction` enums, forbidden actions in `domain::agent::Spec`, and app-side requested-side-effect validation.

Plain-English operational translation:
  A blocked action is a live move the agent/app workflow must not perform directly, even if it can prepare evidence or a draft that helps a person do the work through the right channel.

Why this matters to operators / product / IT:
  It makes no-go boundaries explicit: no autonomous customer sends, provider/PMS writes, schedule/capacity changes, payment/refund/discount movement, safety/medical approvals, source-data hiding, or policy exceptions.

Linked entity/entities:
  [Review gates, blocked actions, and human approval boundaries](entity-atlas-review-safety-boundaries.md), workflow packet entities in [Workflow packets atlas](entity-atlas-workflow-packets-agents.md), and human-role/evidence fields in [entity relationship map](entity-atlas-relationships.md#blocked-actions---human-roles---evidence-fields).

Linked workflows / contracts:
  Booking Triage, Checkout Completion, CRM Retention, Daily Updates, Manager Daily Brief, Data Quality Hygiene, and planned Regional Labor Exceptions via [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md).

Authoritative source / Rustdoc / test evidence:
  [`app/src/booking_triage.rs`](../../app/src/booking_triage.rs), [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs), [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs), [`app/src/daily_update.rs`](../../app/src/daily_update.rs), [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs), [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs), [`app/src/agents.rs`](../../app/src/agents.rs), and workflow/API tests that assert draft-only or blocked-side-effect behavior.

Automation boundary:
  - May draft/recommend/validate/record: name the blocked action, explain why it is blocked, prepare an internal task/draft/review packet, and reject requested side effects in validation.
  - Blocked / human-reviewed: the action named by the term itself unless an approved human/source-system workflow performs it outside the agent/app boundary.

Common confusion / what not to infer:
  “Blocked” does not mean the business action is impossible or the product is broken. It means the automated workflow is not the authority to do it directly.

Link targets:
  [Glossary: blocked action](../glossary-workflow-state-terms.md#blocked-action), [review/safety atlas](entity-atlas-review-safety-boundaries.md), [evidence policy / blocked actions / outcomes](../safety/evidence-policy-blocked-actions-outcomes.md), and [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md).

Suggested public wording:
  A blocked action is a side effect the app/agent cannot take directly. The workflow may prepare or route the work, but customer sends, provider writes, schedule changes, money movement, safety approvals, and source cleanup require the proper human or system-of-record authority.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Storage record / projection

Term in code/docs:
  Storage record, projection, `storage::operations::*Record`, `Stored*`, storage codes, codecs, and promotion/demotion between storage and domain values.

Plain-English operational translation:
  A storage record is the durable shape the system saves for reporting, replay, or audit. It preserves proof and projection fields, but it is not the policy decision or provider source itself.

Why this matters to operators / product / IT:
  Storage records make outcomes, source refs, service-line contracts, operations context, and labor evidence repeatable without letting a database row become business authority. They are how review evidence survives after a packet is handled.

Linked entity/entities:
  [Runtime, storage, API, worker, CLI, and contract-test surfaces](entity-atlas-runtime-storage-api-surfaces.md), [Outcome, labor, operations, analytics, money, and safety evidence atlas](entity-atlas-outcomes-operations-money.md), and storage rows in [entity atlas inventory](entity-atlas-inventory.md#family-storage-api-and-runtime-shells).

Linked workflows / contracts:
  [Storage persistence crosswalk](../entity-atlas/contract-crosswalk/storage-persistence.md), Manager Daily Brief outcome capture, Data Quality Hygiene outcome capture, service-line contract records, source refs in stored outcome/projection records, and API/runtime proof surfaces.

Authoritative source / Rustdoc / test evidence:
  [`storage/README.md`](../../storage/README.md), [`storage/src/operations.rs`](../../storage/src/operations.rs), service-line storage modules under [`storage/src/service_line`](../../storage/src/service_line), [`storage/tests/operations_storage_contracts.rs`](../../storage/tests/operations_storage_contracts.rs), [`storage/tests/manager_daily_brief_outcome_storage.rs`](../../storage/tests/manager_daily_brief_outcome_storage.rs), and [`storage/tests/data_quality_hygiene_outcome_storage.rs`](../../storage/tests/data_quality_hygiene_outcome_storage.rs).

Automation boundary:
  - May draft/recommend/validate/record: persist reviewed outcomes/projections, encode/decode storage-shaped records, keep source refs with stored proof, and expose read-only/reporting evidence.
  - Blocked / human-reviewed: treating storage as source-of-record policy, provider truth, approval, payment authority, schedule authority, or permission for future live action.

Common confusion / what not to infer:
  A read model or storage projection is not domain truth and not provider authority. It is a durable, conversion-backed view of evidence and outcomes.

Link targets:
  [Glossary: storage](../glossary-architecture-terms.md#storage), [read model](../glossary-architecture-terms.md#read-model), [storage persistence crosswalk](../entity-atlas/contract-crosswalk/storage-persistence.md), and [runtime/storage/API atlas](entity-atlas-runtime-storage-api-surfaces.md).

Suggested public wording:
  A storage record is durable proof or projection. It can save source refs, reviewed outcomes, labor minutes, or service-line records for reporting and audit, but it does not replace domain policy, provider truth, or human approval.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Outcome / metric / labor minutes

Term in code/docs:
  Outcome capture, outcome record, labor impact estimate, labor minutes, `app::*::OutcomeRecord`, `storage::operations::*OutcomeRecord`, and reporting groups.

Plain-English operational translation:
  An outcome records what staff or a reviewer did after a recommendation and what labor result was observed or estimated. Labor minutes are the unit used to keep value claims tied to reviewed evidence.

Why this matters to operators / product / IT:
  It separates “this could save time” from “this reviewed action saved or cost this many minutes.” That distinction protects ROI claims, audit trails, and regional reporting.

Linked entity/entities:
  [Outcome, labor, operations, analytics, money, and safety evidence atlas](entity-atlas-outcomes-operations-money.md), outcome rows in [entity atlas inventory](entity-atlas-inventory.md#family-outcomes-and-labor-measurement), and outcome/storage rows in [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md).

Linked workflows / contracts:
  Manager Daily Brief, Data Quality Hygiene, CRM Retention follow-up outcomes, Checkout Completion evidence, Daily Updates approval/send-stub outcomes, and planned Regional Labor Exceptions.

Authoritative source / Rustdoc / test evidence:
  [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs), [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs), [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs), [`storage/src/operations.rs`](../../storage/src/operations.rs), [`docs/design/manager-daily-brief-measurable-labor-loop.md`](manager-daily-brief-measurable-labor-loop.md), [`docs/design/data-quality-hygiene-labor-loop.md`](data-quality-hygiene-labor-loop.md), and storage/API tests cited in [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map).

Automation boundary:
  - May draft/recommend/validate/record: estimate labor impact, record reviewed disposition, store before/actual minutes where contracts exist, and report with source refs and reviewer context.
  - Blocked / human-reviewed: claiming guaranteed ROI, retroactively approving the underlying action, or hiding missing outcome/storage evidence behind an estimate.

Common confusion / what not to infer:
  Outcome capture measures or records reviewed work; it does not perform the underlying action. A labor estimate is not the same as observed labor evidence.

Link targets:
  [Glossary: outcome capture](../glossary-workflow-state-terms.md#outcome-capture), [outcome/labor atlas](entity-atlas-outcomes-operations-money.md), [storage persistence crosswalk](../entity-atlas/contract-crosswalk/storage-persistence.md), and [source evidence map](../safety/source-evidence-map.md).

Suggested public wording:
  Outcome capture records what happened after a reviewed recommendation, including disposition, actor, source refs, and labor minutes where implemented. It turns labor-saving claims into auditable evidence instead of assuming every draft saved time.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Adapter / integration

Term in code/docs:
  Adapter, integration, `integrations/gingr`, mapper, transport seam, webhook verification, endpoint request builder, and concrete implementations of app tool ports.

Plain-English operational translation:
  An adapter is the controlled translation boundary between NVA workflows and an outside system or runtime capability. In this repo, Gingr is the clearest provider adapter; future live tool implementations would also be adapters around app ports.

Why this matters to operators / product / IT:
  Adapters show what evidence entered from a provider, what was redacted or quarantined, what mapping succeeded or failed, and what remains a gap. They prevent provider quirks or runtime mechanics from becoming hidden business rules.

Linked entity/entities:
  [Gingr provider boundary atlas](../integrations/gingr/provider-boundary-atlas.md), [Source, provenance, and data-quality atlas](source-provenance-data-quality-atlas.md), [Runtime, storage, API, worker, CLI, and contract-test surfaces](entity-atlas-runtime-storage-api-surfaces.md), and provider-boundary rows in [entity atlas inventory](entity-atlas-inventory.md#family-gingr-dto-mapping-endpoint-and-provider-boundary).

Linked workflows / contracts:
  [Source/provider flows](../entity-atlas/contract-crosswalk/source-provider-flows.md), tool-port contracts in [`app/src/tools.rs`](../../app/src/tools.rs), Runtime/API Operations, Gingr Source Normalization, Data Quality Hygiene, and workflow packets that consume source refs.

Authoritative source / Rustdoc / test evidence:
  [`integrations/gingr/README.md`](../../integrations/gingr/README.md), [`integrations/gingr/src/endpoint/mod.rs`](../../integrations/gingr/src/endpoint/mod.rs), [`integrations/gingr/src/transport.rs`](../../integrations/gingr/src/transport.rs), [`integrations/gingr/src/response.rs`](../../integrations/gingr/src/response.rs), [`integrations/gingr/src/webhook.rs`](../../integrations/gingr/src/webhook.rs), [`integrations/gingr/src/mapping/mod.rs`](../../integrations/gingr/src/mapping/mod.rs), [`app/src/tools.rs`](../../app/src/tools.rs), and adapter/mapping/webhook tests under `integrations/gingr/tests`.

Automation boundary:
  - May draft/recommend/validate/record: translate typed requests/responses, preserve evidence, map supported fields, verify webhooks, return draft/review outcomes, and expose unavailable/mock/not-implemented status honestly.
  - Blocked / human-reviewed: assuming a provider adapter owns business policy, guarantees source correctness, has live HTTP/write capability, or can skip review gates because it can parse a provider payload.

Common confusion / what not to infer:
  An adapter is not the domain owner. It can bring evidence in or satisfy a capability contract, but domain/app/review contracts decide what that evidence means and what action is allowed.

Link targets:
  [Glossary: adapter](../glossary-architecture-terms.md#adapter), [Glossary: integrations/gingr](../glossary-architecture-terms.md#integration-integrationsgingr), [Gingr provider boundary atlas](../integrations/gingr/provider-boundary-atlas.md), and [source/provider flows](../entity-atlas/contract-crosswalk/source-provider-flows.md).

Suggested public wording:
  An adapter is a controlled translator at the system edge. It lets NVA use provider or runtime evidence without letting provider-specific fields, API quirks, or mock/live implementation details become business truth by accident.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Runtime shell / API, worker, CLI exposure

Term in code/docs:
  Runtime shell, API router, worker runtime, CLI shell, `apps/api`, `apps/worker`, `apps/cli`, local smoke, and runtime exposure.

Plain-English operational translation:
  A runtime shell is where app/domain/storage contracts are exposed for local demos, HTTP routes, workers, CLI inspection, scripts, or future operations. It is the delivery surface, not a separate business model.

Why this matters to operators / product / IT:
  Runtime shells show whether a contract can be inspected, tested, or executed in a controlled environment. They also show side-effect modes and disabled/stubbed behavior so docs do not overclaim production automation.

Linked entity/entities:
  [Runtime, storage, API, worker, CLI, and contract-test surfaces](entity-atlas-runtime-storage-api-surfaces.md), runtime rows in [entity atlas inventory](entity-atlas-inventory.md#family-storage-api-and-runtime-shells), and runtime exposure in [contract crosswalk](../entity-atlas/contract-crosswalk/runtime-exposure.md).

Linked workflows / contracts:
  Manager Daily Brief API/draft/outcome routes, Data Quality Hygiene API/draft/outcome routes, vaccine/document contract routes, local smoke chains, worker runtime agent/side-effect modes, and CLI inspection commands.

Authoritative source / Rustdoc / test evidence:
  [`apps/api/src/http.rs`](../../apps/api/src/http.rs), [`apps/worker/src/runtime.rs`](../../apps/worker/src/runtime.rs), [`apps/cli/src/main.rs`](../../apps/cli/src/main.rs), [`app/src/local_smoke.rs`](../../app/src/local_smoke.rs), runtime READMEs under `apps/*/README.md`, API/worker tests under `apps/api/tests` and `apps/worker/tests`, and [runtime exposure crosswalk](../entity-atlas/contract-crosswalk/runtime-exposure.md).

Automation boundary:
  - May draft/recommend/validate/record: expose deterministic routes, run disabled/draft-only workers, inspect local proof, persist/retrieve contract evidence where implemented, and make side-effect mode visible.
  - Blocked / human-reviewed: treating an API route, worker config, CLI command, or local smoke fixture as production authority for customer sends, provider writes, schedule changes, payments, or safety approvals.

Common confusion / what not to infer:
  Runtime exposure is not business ownership. A shell can run or show a contract, but it must not redefine entity truth or widen app/domain safety boundaries.

Link targets:
  [Runtime/storage/API atlas](entity-atlas-runtime-storage-api-surfaces.md), [runtime exposure crosswalk](../entity-atlas/contract-crosswalk/runtime-exposure.md), [Glossary: app](../glossary-architecture-terms.md#app), [Glossary: tool port](../glossary-architecture-terms.md#tool-port-apptools), and [review boundaries matrix](../safety/review-boundaries-matrix.md).

Suggested public wording:
  A runtime shell exposes app/domain/storage contracts through an API, worker, CLI, local smoke, or script. It can make a workflow inspectable or runnable in a safe mode, but it does not own business truth or grant live side-effect authority by itself.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Idempotency / replay key

Term in code/docs:
  Idempotency, replay, workflow event id, correlation id, source refs on outcome records, and contract tests that repeat a request without duplicating a live effect.

Plain-English operational translation:
  Idempotency means the system can recognize the same workflow fact/action context again instead of accidentally double-counting, double-sending, or double-recording work. For non-coders, the important idea is repeat-safe evidence, not the implementation mechanism.

Why this matters to operators / product / IT:
  Repeat-safe workflow evidence protects customer trust and reporting. It prevents duplicate customer outreach, duplicate cleanup tasks, duplicate labor credit, duplicate provider/provider-like changes, or confusing replay during local/API demos.

Linked entity/entities:
  Cross-cutting support term for workflow packets, audit events, source refs, outcome records, approval records, storage projections, and runtime shells. It supports [Workflow packets atlas](entity-atlas-workflow-packets-agents.md), [Outcome/labor atlas](entity-atlas-outcomes-operations-money.md), and [Runtime/storage/API atlas](entity-atlas-runtime-storage-api-surfaces.md).

Linked workflows / contracts:
  [Workflow event idempotency/replay docs](../workflows/workflow-event-idempotency-replay.md), workflow packets crosswalk, API/runtime contract routes, Manager Daily Brief outcome capture, Data Quality Hygiene outcome capture, and storage projection tests.

Authoritative source / Rustdoc / test evidence:
  [`domain/src/workflow.rs`](../../domain/src/workflow.rs), [`app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs), [`app/src/data_quality_hygiene.rs`](../../app/src/data_quality_hygiene.rs), [`storage/src/operations.rs`](../../storage/src/operations.rs), [`apps/api/src/http.rs`](../../apps/api/src/http.rs), tests named in [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), and [`docs/workflows/workflow-event-idempotency-replay.md`](../workflows/workflow-event-idempotency-replay.md).

Automation boundary:
  - May draft/recommend/validate/record: carry event/action/correlation/source identifiers, reject duplicate draft/outcome submissions where implemented, and make replay proof visible.
  - Blocked / human-reviewed: using idempotency wording to imply that a live side effect is safe, reversible, or approved; duplicate prevention is not permission to act.

Common confusion / what not to infer:
  Idempotency is not a business approval and not a rollback guarantee. It is a repeat-safety/evidence discipline around packets, outcomes, and runtime requests.

Link targets:
  [Workflow event idempotency/replay](../workflows/workflow-event-idempotency-replay.md), [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), [storage persistence crosswalk](../entity-atlas/contract-crosswalk/storage-persistence.md), and [runtime exposure crosswalk](../entity-atlas/contract-crosswalk/runtime-exposure.md).

Suggested public wording:
  Idempotency means a workflow can recognize the same source-backed request or outcome context again, so review packets, drafts, and outcome records do not accidentally duplicate labor evidence or side effects. It supports auditability; it does not authorize live action.

Review status:
  draft — docs worker pass, source-linked but not product/engineering reviewed.

## Validated value / newtype

Term in code/docs:
  Validated value, newtype, `nutype`-style scalar, semantic IDs/names/dates/money/labor minutes, and domain/storage scalar wrappers.

Plain-English operational translation:
  A validated value is a small field with a business rule attached: an email must look like an email, a name cannot be blank, a labor-minute value must be sane, a provider id is scoped to a provider, and money/status/codes should not be swappable by accident.

Why this matters to operators / product / IT:
  Non-coders do not need the Rust mechanics. They do need to know that the repo rejects or quarantines invalid values before they become customer contact details, pet names, labor metrics, payment evidence, or workflow decisions.

Linked entity/entities:
  Cross-cutting support term for Customer, Pet, Reservation, Source/Provenance, Data Quality Issue, Outcome/Labor, Money/Payment, Service-line Contracts, Storage Records, and Provider Boundary entities. See [entity atlas inventory](entity-atlas-inventory.md) rows for Customer, Pet, Reservation, Data Quality Issue, Labor Minutes, and Storage Operations Boundary.

Linked workflows / contracts:
  Provider mapping candidates, Data Quality Hygiene, Booking Triage, Manager Daily Brief, storage projection codecs, and source/provider normalization flows.

Authoritative source / Rustdoc / test evidence:
  Domain modules under `domain/src` such as [`customer.rs`](../../domain/src/customer.rs), [`pet.rs`](../../domain/src/pet.rs), [`source.rs`](../../domain/src/source.rs), [`data_quality.rs`](../../domain/src/data_quality.rs), [`money/mod.rs`](../../domain/src/money/mod.rs), service-line modules, storage scalars in [`storage/src/operations.rs`](../../storage/src/operations.rs), and mapping/storage tests that validate provider-to-domain promotion.

Automation boundary:
  - May draft/recommend/validate/record: reject blank/invalid/scoped values, convert provider fields through named constructors/mappers, and raise data-quality issues instead of guessing.
  - Blocked / human-reviewed: treating a validation pass as correctness, approval, or source authority; validation only says the value satisfies a local shape/invariant.

Common confusion / what not to infer:
  Do not explain Rust newtypes for their own sake in public docs. Say “validated value” only when it protects an operator-facing entity, source fact, labor metric, or safety boundary.

Link targets:
  [Entity-relevant term inventory: defer decisions](entity-relevant-architecture-rust-term-inventory.md#defer-decisions), [source/provider flows](../entity-atlas/contract-crosswalk/source-provider-flows.md), [Data-quality issue glossary](../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue), and [semantic mapping rows](../entity-atlas/contract-crosswalk/source-provider-flows.md#crosswalk-where-provider-entities-enter-and-normalize).

Suggested public wording:
  A validated value is a field the repo checks before it can be used as business evidence, such as a customer email, pet name, provider id, status code, labor-minute value, or storage code. It protects workflows from blank, invalid, swapped, or provider-scoped values without asking non-coders to learn Rust internals.

Review status:
  draft — cross-cutting support term; keep out of public glossary unless tied to a specific entity page.

## Typestate / readiness state

Term in code/docs:
  Typestate request, staged request builder, readiness state, `app::booking_triage::Request<S>`, and workflow readiness/completeness markers.

Plain-English operational translation:
  A readiness state means a workflow has or has not collected the evidence needed for the next safe step. The public concept is “required evidence is collected before the policy decision,” not the generated Rust type names.

Why this matters to operators / product / IT:
  Readiness staging explains why a booking, data-hygiene item, checkout packet, or manager action may stop for missing pet profile, vaccine, payment, source, care, or review evidence instead of producing a confident recommendation.

Linked entity/entities:
  Cross-cutting support term for Booking Triage Packet, Reservation, Pet, Care Profile, Vaccine Record, Review Gate, Data Quality Issue, Workflow Packet, and Agent Prompt Packet. Primary home: [Workflow packets atlas](entity-atlas-workflow-packets-agents.md) and [Review gates atlas](entity-atlas-review-safety-boundaries.md).

Linked workflows / contracts:
  Booking Triage, Data Quality Hygiene, Checkout Completion, Daily Updates, and any workflow page that describes readiness, missing evidence, deterministic validation, or staff evaluation packets.

Authoritative source / Rustdoc / test evidence:
  [`app/src/booking_triage.rs`](../../app/src/booking_triage.rs), [`app/tests/booking_triage_mvp.rs`](../../app/tests/booking_triage_mvp.rs), [`domain/src/workflow.rs`](../../domain/src/workflow.rs), [`domain/src/policy.rs`](../../domain/src/policy.rs), and the Booking Triage row in [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map).

Automation boundary:
  - May draft/recommend/validate/record: enforce missing-evidence stops, build staff evaluation packets, explain readiness buckets, and route incomplete items to review or data-quality cleanup.
  - Blocked / human-reviewed: treating a complete request as permission for live confirmation, provider write, customer send, payment action, or safety approval.

Common confusion / what not to infer:
  Do not surface generated typestate or builder internals in operator docs. Translate them into evidence readiness and safe workflow staging.

Link targets:
  [Entity-relevant term inventory: defer decisions](entity-relevant-architecture-rust-term-inventory.md#defer-decisions), [Booking Triage operator page](../workflows/operator/booking-triage.md), [Workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), and [review boundaries matrix](../safety/review-boundaries-matrix.md).

Suggested public wording:
  A readiness state means the workflow has checked whether the required source, pet, reservation, care, payment, and review evidence is present before it recommends the next step. It protects staff from acting on incomplete packets; it is not approval to perform the action.

Review status:
  draft — cross-cutting support term; keep implementation names in Rustdoc/source unless a workflow page needs the operator-facing readiness explanation.

## Rustdoc / source link

Term in code/docs:
  Rustdoc/module path, source path, test evidence, rendered Rustdoc, and source/Rustdoc evidence link.

Plain-English operational translation:
  A Rustdoc/source link is the proof trail back to the code contract behind a glossary or Entity Atlas claim. It lets a docs reviewer verify whether a workflow, entity, DTO, storage record, or review gate is actually implemented, tested, future/planned, or docs-only.

Why this matters to operators / product / IT:
  It prevents public prose from promising live capabilities that source does not prove. It also lets non-coders ask an engineering reviewer for the exact contract rather than debating vague architecture language.

Linked entity/entities:
  Cross-cutting support term for every Entity Atlas page, especially source/provider, workflow packet, review gate, outcome/storage, and runtime shell entries.

Linked workflows / contracts:
  [Entity atlas evidence-link anchors](entity-atlas-evidence-link-anchors.md), [surface inventory](../entity-atlas/contract-crosswalk/surface-inventory.md), [workflow source/Rustdoc backing map](workflow-source-rustdoc-backing-map.md), [Rustdoc operational language guide](../rustdoc-operational-language-guide.md), and all workflow/entity pages that cite source contracts.

Authoritative source / Rustdoc / test evidence:
  The linked code/test paths are the evidence. No rendered `target/doc` tree is assumed; when rendered Rustdoc is unavailable, cite source/module/type paths as specified by [entity-linked glossary entry template](entity-linked-glossary-entry-template.md#required-entry-shape).

Automation boundary:
  - May draft/recommend/validate/record: cite source/module/test paths, mark rendered Rustdoc unavailable, distinguish source-backed claims from docs-only TODOs, and fail link checks when evidence paths break.
  - Blocked / human-reviewed: inventing rendered Rustdoc URLs, treating docs-only concept edges as shipped contracts, or widening product claims beyond tests/source.

Common confusion / what not to infer:
  A source link proves where the contract can be checked; it does not by itself prove production deployment, live credentials, or customer/provider side effects.

Link targets:
  [Entity atlas evidence-link anchors](entity-atlas-evidence-link-anchors.md), [surface inventory](../entity-atlas/contract-crosswalk/surface-inventory.md), [Rustdoc operational language guide](../rustdoc-operational-language-guide.md), and [QA Rustdoc freshness/evidence links](../qa-rustdoc-freshness-evidence-links-2026-06-19.md).

Suggested public wording:
  A Rustdoc/source link is the evidence trail behind a glossary or Entity Atlas statement. If the docs say a workflow can draft, block, record, or expose something, the link should point to the source, module path, test, or explicit TODO that proves the claim’s current status.

Review status:
  draft — cross-cutting support term; should remain a reviewer convention and glossary aid, not a public Rust lesson.

## Non-coder example translations tied to entities and workflows

Use these examples when a glossary term appears in code, Rustdoc, workflow prose, or a contract crosswalk and a non-coder needs the pet-resort meaning before the implementation detail. Each example intentionally starts from a term-in-context, then points back to the entity/workflow authority instead of teaching generic Rust.

| Example | Code/docs term in context | Plain-English translation | Clarifies these entity/workflow pages | Authority / safety boundary |
| --- | --- | --- | --- | --- |
| Pet, customer, and source reservation | `domain::reservation` uses source refs/provenance from Gingr provider records while Booking Triage builds a `StaffEvaluationPacket`. | “This booking claim is about a specific pet, pet parent, stay, location, and source receipt. Gingr can provide evidence, but the workflow must map that evidence into NVA reservation meaning before staff use it.” | [PetSuites core entity atlas](entity-atlas-petsuites-core-entities.md#reservation), [Source/provenance atlas](source-provenance-data-quality-atlas.md), [Booking Triage](../workflows/operator/booking-triage.md), and [workflow-to-entity map](workflow-to-entity-navigation-map.md#workflow---entity-matrix). | Automation may summarize required facts, source refs, missing evidence, and readiness. It may not confirm/reject the reservation, write Gingr/PMS, change capacity, take payment, or send the customer message without the front-desk/manager/source-of-record process. |
| Boarding stay readiness | `domain::boarding::*`, `domain::care::*`, `domain::vaccine::*`, and booking readiness states feed Booking Triage or Checkout Completion. | “A boarding stay is not just dates on a reservation. It carries room/capacity, care, vaccine, deposit/payment, handoff, and local-policy readiness.” | [PetSuites core entity atlas](entity-atlas-petsuites-core-entities.md#boarding-contract), [Booking Triage](../workflows/operator/booking-triage.md), [Checkout Completion](../workflows/operator/checkout-completion.md), and [Review gates atlas](entity-atlas-review-safety-boundaries.md). | Automation may flag incomplete readiness and route review packets. It may not admit, check in/out, assign rooms, waive policies, approve care/vaccine exceptions, or mutate the source schedule. |
| Daycare attendance and package opportunity | `domain::daycare::{attendance, eligibility, package_opportunity}` and package opportunity rows in revenue/service-line docs. | “A daycare visit/package signal is a reviewable service-line opportunity, not an automatic sale or group-play clearance.” | [Core entities: daycare contract](entity-atlas-petsuites-core-entities.md#daycare-contract), [Revenue opportunity entity families](entity-atlas-revenue-opportunity-entities.md), [Manager Daily Brief](../workflows/operator/manager-daily-brief.md), and [workflow packets atlas](entity-atlas-workflow-packets-agents.md). | Automation may identify attendance/package evidence, draft an internal opportunity, and estimate labor or revenue follow-up. It may not sell a package, adjust session balances, clear group play, change attendance, discount, or contact the customer without approval. |
| Grooming appointment and rebooking | `app::crm_retention::{RetentionOpportunity, ContactPermission, StaffReviewPacket}` consumes grooming cadence and completed-service evidence. | “A grooming rebooking candidate is a staff-reviewed follow-up opportunity for a customer/pet, not an appointment booking or marketing send.” | [Revenue opportunity entity families](entity-atlas-revenue-opportunity-entities.md), [Grooming Rebooking / Retention](../workflows/operator/grooming-rebooking-retention.md), [Workflow packets atlas](entity-atlas-workflow-packets-agents.md), and [Review gates atlas](entity-atlas-review-safety-boundaries.md). | Automation may find source-backed rebooking candidates and draft safe copy. It may not send outreach, book an appointment, promise availability, alter consent/DNC, apply discounts, or write to the provider calendar. |
| Checkout and payment evidence | `app::checkout_completion::Packet`, `domain::money::*`, and payment/deposit facts appear in checkout packets and money evidence rows. | “Checkout/payment evidence tells staff what may need review at departure; it is not the system moving money.” | [Checkout Completion](../workflows/operator/checkout-completion.md), [Outcome/money atlas](entity-atlas-outcomes-operations-money.md), [Core entity atlas: reservation/stay](entity-atlas-petsuites-core-entities.md#reservation), and [storage persistence crosswalk](../entity-atlas/contract-crosswalk/storage-persistence.md). | Automation may summarize open handoffs, payment/deposit status, and reconciliation questions. It may not check out a pet, charge/refund/discount/waive fees, promise a payment resolution, or mutate provider/POS state. |
| Incident record and escalation | `domain::incident::*`, incident/care facts, omitted sensitive facts, and review gates in Daily Updates or incident escalation docs. | “An incident is safety evidence that may shape internal review and customer wording; it is not an automatic blame, medical decision, or customer-facing disclosure.” | [PetSuites core entity atlas: incident](entity-atlas-petsuites-core-entities.md#incident), [Incident escalation workflow](../workflows/incident-escalation-agent-parts/escalation-workflow.md), [Daily Updates / Pawgress Drafts](../workflows/operator/daily-updates-pawgress-drafts.md), and [review boundaries matrix](../safety/review-boundaries-matrix.md). | Automation may summarize incident evidence, suppress sensitive details from drafts, and route manager/care review. It may not approve safety/medical/behavior decisions, remove restrictions, assign fault, send incident wording, or edit provider incident records. |
| Vaccine/document evidence | `domain::vaccine::*`, `domain::document::*`, source refs, and upload/review workflow terms. | “A vaccine document is proof to review against policy; the presence of a file or provider field is not the same as vaccine approval.” | [PetSuites core entity atlas: vaccine record](entity-atlas-petsuites-core-entities.md#vaccine-record-and-vaccine-policy), [Vaccine document upload workflow](../workflows/vaccine-document-agent-parts/upload-workflow.md), [Source/provenance atlas](source-provenance-data-quality-atlas.md), and [Review gates atlas](entity-atlas-review-safety-boundaries.md). | Automation may extract/cite document evidence, flag missing/stale/conflicting records, and prepare review tasks. It may not approve vaccine status, override medical policy, expose sensitive documents, or write provider records without the authorized review path. |
| Daily update / Pawgress draft | `app::daily_update::{CustomerMessageDraft, IncludedFact, OmittedFact, ReviewDisposition, SendStub}` and `app::agents::AgentPromptPacket`. | “A Pawgress update is a customer-message draft built from reviewed source facts, with included/omitted facts visible; it is not a sent message.” | [Daily Updates / Pawgress Drafts](../workflows/operator/daily-updates-pawgress-drafts.md), [Workflow packets atlas](entity-atlas-workflow-packets-agents.md), [Core entity atlas: message state](entity-atlas-petsuites-core-entities.md#message-and-message-state), and [Review gates atlas](entity-atlas-review-safety-boundaries.md). | Automation may draft copy, cite facts, omit sensitive items, and create a send stub. Approved staff/manager/care/privacy reviewers control sends, media use, incident/medical wording, and provider/customer side effects. |
| Review gate and blocked action | `domain::policy::ReviewGate`, workflow-local `BlockedAction` enums, and required review gates in app packets. | “The docs are naming the stop sign and who must review it. Naming the gate is not the approval.” | [Review gates, blocked actions, and human approval boundaries](entity-atlas-review-safety-boundaries.md), [review boundaries matrix](../safety/review-boundaries-matrix.md), [workflow packets crosswalk](../entity-atlas/contract-crosswalk/workflow-packets.md), and [glossary workflow terms](../glossary-workflow-state-terms.md#review-gate). | Automation may identify the gate, explain why work is blocked, and route the packet. It may not treat the gate text, agent output, or packet completion as permission for customer sends, provider writes, schedule/capacity changes, money movement, safety approvals, or policy exceptions. |
| Outcome and labor metric | `app::*::OutcomeRecord`, `storage::operations::*OutcomeRecord`, `LaborImpactEstimate`, `actual_minutes`, and reporting-group fields. | “The metric is the receipt after reviewed work: what happened, who reviewed it, which source facts support it, and how much time was actually saved or spent.” | [Outcome/labor atlas](entity-atlas-outcomes-operations-money.md), [Manager Daily Brief](../workflows/operator/manager-daily-brief.md), [Data Quality Hygiene](../workflows/operator/data-quality-hygiene.md), and [storage persistence crosswalk](../entity-atlas/contract-crosswalk/storage-persistence.md). | Automation may estimate and record reviewed dispositions where a storage/app contract exists. It may not claim guaranteed ROI, count unreviewed drafts as savings, trigger staffing/payroll/payment actions, or hide missing source/outcome proof. |

## Review checklist for folding these entries into glossary files

- Does the entry tie the term to an entity family, relationship/crosswalk, workflow/contract, and safety/evidence link?
- Does the first paragraph translate the term into pet-resort operating meaning before implementation detail?
- Does the entry preserve the difference between domain truth, source/provider fact, app draft/packet, storage projection, and reviewed outcome?
- Does the entry cite source/module/test evidence without inventing rendered Rustdoc URLs?
- Does the automation boundary say what may be drafted/recommended/validated/recorded and what remains human-reviewed or blocked?
- Does the public wording avoid generic “data,” “record,” “model,” or “AI can” claims that blur authority?
- Are cross-cutting Rust terms (`validated value`, readiness/typestate, Rustdoc/source link) kept only where they clarify entity/workflow safety rather than explaining Rust for its own sake?
