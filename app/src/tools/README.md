# `app::tools`

This page helps engineering maintainers cut front-desk and manager handoff time by showing which external-tool capabilities the app can ask for safely. A booking example: the app may request an availability check or draft a reservation-status update from source-backed customer, pet, reservation, and capacity facts, but a human-approved workflow still owns live PMS/provider writes, customer messages, payments, and schedule changes.

In plain English, `app::tools` is the promise between deterministic pet-resort workflows and concrete systems. It names the store, reservation-system, agent-runtime, portal, payment, messaging, document/OCR, media, and Hermes automation capabilities workflows may use without choosing Gingr, a database adapter, a worker runtime, or another provider implementation.

Read this module with [`../tools.rs`](../tools.rs) open. The Rust module is implemented as `app/src/tools.rs`, while the shared error submodule lives at [`error.rs`](./error.rs) and is loaded as `app::tools::error` by the `pub mod error;` declaration in [`../tools.rs`](../tools.rs). The crate root exposes this surface through [`../lib.rs`](../lib.rs), and [`../../README.md`](../../README.md) explains how `app` composes `domain` truth instead of owning it.

This surface is deliberately a contract boundary, not an adapter implementation. It defines traits and typed request/outcome packets so provider clients, storage adapters, worker scheduling, HTTP routing, live payment capture, and customer-message delivery satisfy app contracts instead of moving business policy into a runtime.

## Module navigation

- [`../tools.rs`](../tools.rs) is the public `app::tools` module. It re-exports [`app::tools::error::{Error, ExternalFailure, Resource, ResourceId, Result}`](./error.rs), defines the top-level ports `CustomerStore`, `ReservationSystem`, and `AgentRuntime`, then groups narrower port vocabularies by tool family.
- [`error.rs`](./error.rs) is `app::tools::error`. It owns shared tool-port failures: missing typed resources, policy denials from `domain::policy::denial::Reason`, and external-system failures such as unavailable portal, payment, messaging, storage, or other provider systems.
- [`../lib.rs`](../lib.rs) publishes `pub mod tools;` and keeps `app::prelude` intentionally small by re-exporting only `tools::{availability, draft_update}` alongside common agent helpers.

## Boundary role

`app::tools` sits between deterministic application workflows and concrete capabilities:

1. Workflows in [`../booking_triage.rs`](../booking_triage.rs), [`../checkout_completion.rs`](../checkout_completion.rs), [`../crm_retention.rs`](../crm_retention.rs), [`../daily_update.rs`](../daily_update.rs), [`../manager_daily_brief.rs`](../manager_daily_brief.rs), and [`../local_smoke.rs`](../local_smoke.rs) should depend on typed app/domain packets, not raw external APIs.
2. Concrete adapters should implement the traits here or translate into these request/outcome types from `storage`, `integrations/gingr`, `apps/api`, `apps/worker`, or `apps/cli`.
3. Tool outcomes must return typed results, draft ids, review statuses, unavailable reasons, provider results, or `app::tools::Error` values. Do not smuggle policy decisions, live side effects, or provider ambiguity through unstructured strings.
4. Agent-facing tools should expose typed context packets and accept typed drafts/recommendations. The canonical architecture guide is [`../../../docs/architecture/agent-app-infrastructure.md`](../../../docs/architecture/agent-app-infrastructure.md): deterministic app code owns source facts, policy, workflow state, storage, review gates, audit/replay, and external writes; Hermes/agents draft, summarize, rank, route, or propose through constrained contracts.

## Type/module map

| Concept | Public type/module path | Defined in | Role |
| --- | --- | --- | --- |
| Tool module surface | `app::tools` | [`../tools.rs`](../tools.rs) | App-facing port and request/outcome namespace. |
| Shared tool result alias | `app::tools::Result<T>` / `app::tools::error::Result<T>` | [`error.rs`](./error.rs) | Alias for `std::result::Result<T, app::tools::Error>`. |
| Shared tool error | `app::tools::Error` / `app::tools::error::Error` | [`error.rs`](./error.rs) | Typed not-found, policy-denial, and external-failure surface for ports. |
| Error resource classifier | `app::tools::error::Resource` | [`error.rs`](./error.rs) | Names the missing resource family: customer, pet, reservation, availability snapshot, or draft reservation update. |
| Error resource identity | `app::tools::error::ResourceId` | [`error.rs`](./error.rs) | Carries typed ids such as `domain::entities::CustomerId`, `domain::entities::PetId`, `domain::entities::reservation::Id`, `app::tools::availability::CapacitySnapshotId`, or `app::tools::draft_update::draft::Id`. |
| External failure classifier | `app::tools::error::ExternalFailure` | [`error.rs`](./error.rs) | Names unavailable external systems without exposing provider-specific internals. |
| Domain aggregate store port | `app::tools::CustomerStore` | [`../tools.rs`](../tools.rs) | Async read port for `domain::entities::Customer`, `Pet`, and `Reservation`. |
| Reservation-system port | `app::tools::ReservationSystem` | [`../tools.rs`](../tools.rs) | Async port for availability checks and draft reservation updates. |
| Agent runtime port | `app::tools::AgentRuntime` | [`../tools.rs`](../tools.rs) | Structured agent execution over `domain::workflow::Event`, serializable input, and `domain::workflow::Result<TOut>`. |
| Availability request/outcome | `app::tools::availability::{Request, Outcome, Decision}` | [`../tools.rs`](../tools.rs) | Typed capacity/availability check surface with explicit available/unavailable decision reasons. |
| Availability scalars | `app::tools::availability::{ServiceNotes, CapacitySnapshotId}` | [`../tools.rs`](../tools.rs) | Validated `nutype` strings for request notes and capacity snapshot identity. |
| Draft reservation update | `app::tools::draft_update::{Request, Rationale, draft::Id}` | [`../tools.rs`](../tools.rs) | Typed draft-only reservation-status update request and returned draft id. |
| Portal lookup port | `app::tools::portal::Lookup` | [`../tools.rs`](../tools.rs) | Async lookup port for provider/account criteria. |
| Portal lookup packets | `app::tools::portal::lookup::{Request, Outcome, Match, Criteria}` | [`../tools.rs`](../tools.rs) | Typed provider lookup request/outcome vocabulary for customer, pet, reservation, not-found, and ambiguous matches. |
| Portal supporting values | `app::tools::portal::{Provider, Include, AccountId, ExternalRecordId}` | [`../tools.rs`](../tools.rs) | Provider label, include flags, account id, and external record id wrappers. |
| Payment gateway port | `app::tools::payment::Gateway` | [`../tools.rs`](../tools.rs) | Async port for payment authorization, refund, and deposit-recording requests. |
| Payment shared values | `app::tools::payment::{Subject, CapturePolicy, ReviewReason, IdempotencyKey}` | [`../tools.rs`](../tools.rs) | Semantic payment subject, capture posture, provider-review reasons, and idempotency. |
| Authorization packets | `app::tools::payment::authorization::{Request, provider::Result, provider::DeclineReason, provider::AuthorizationId}` | [`../tools.rs`](../tools.rs) | Payment authorization request and provider outcome vocabulary. |
| Refund packets | `app::tools::payment::refund::{Request, Reason, provider::Result, provider::RejectionReason, provider::RefundId}` | [`../tools.rs`](../tools.rs) | Refund request and provider outcome vocabulary. |
| Deposit recording packets | `app::tools::payment::deposit::{RecordRequest, RecordResult}` | [`../tools.rs`](../tools.rs) | Records a reservation payment reference and resulting `domain::payment::DepositStatus`. |
| Message drafting port | `app::tools::messaging::Drafting` | [`../tools.rs`](../tools.rs) | Async port for drafting customer/staff/manager messages without sending them. |
| Message drafting packets | `app::tools::messaging::draft::{Request, Result, Status}` | [`../tools.rs`](../tools.rs) | Draft request, draft id, and review status. |
| Message supporting values | `app::tools::messaging::{DeliveryChannel, Recipient, ReviewPolicy, message_body::Body}` | [`../tools.rs`](../tools.rs) | Channel, recipient, review posture, and validated message body. |
| Document intake/OCR port | `app::tools::documents::document::Intake` | [`../tools.rs`](../tools.rs) | Async port for document intake and OCR extraction. |
| Document intake packets | `app::tools::documents::document::{IntakeRequest, IntakeResult, Source, ExpectedContent, Classification, reference::Ref}` | [`../tools.rs`](../tools.rs) | Document reference, source, expected content, and classification. |
| OCR packets | `app::tools::documents::ocr::{Request, Result, ReviewReason, extracted_text::Text}` | [`../tools.rs`](../tools.rs) | OCR request and either extracted text or human-review reason. |
| Media capture port | `app::tools::media::Capture` | [`../tools.rs`](../tools.rs) | Async port for requesting camera/media snapshots. |
| Media capture packets | `app::tools::media::{SnapshotRequest, SnapshotResult, CapturePurpose, UnavailableReason, CameraId, Ref}` | [`../tools.rs`](../tools.rs) | Typed camera request, purpose, media reference, and unavailable reason. |
| Hermes automation port | `app::tools::hermes::AutomationHooks` | [`../tools.rs`](../tools.rs) | Draft-only port for Hermes task and schedule drafts. |
| Hermes task draft packets | `app::tools::hermes::task::{DraftRequest, Trigger, kanban::DraftResult, kanban::TaskId}` | [`../tools.rs`](../tools.rs) | Drafts a queue task from workflow task title/body and trigger. |
| Hermes schedule draft packets | `app::tools::hermes::schedule::{DraftRequest, DraftResult, Cadence, Name, Id}` | [`../tools.rs`](../tools.rs) | Drafts a schedule with cadence and queue. |
| Hermes shared values | `app::tools::hermes::{QueueName, DraftStatus}` | [`../tools.rs`](../tools.rs) | Queue identity and draft/review status shared by task and schedule hooks. |
| Tool inventory | `app::tools::ExternalToolCandidate` | [`../tools.rs`](../tools.rs) | Enumerates candidate external systems displayed by the CLI and useful for planning adapters. |

## Error surfaces and validation boundaries

`app::tools::error` is the shared failure vocabulary for these ports, not a catch-all logging channel:

- `app::tools::Error::NotFound` combines a [`Resource`](./error.rs) and [`ResourceId`](./error.rs) so callers can distinguish missing customers, pets, reservations, capacity snapshots, and draft updates without parsing text.
- `app::tools::Error::PolicyDenied` carries `domain::policy::denial::Reason` from [`../../../domain/src/policy.rs`](../../../domain/src/policy.rs). Use it when deterministic policy rejects an action before a provider call should happen.
- `app::tools::Error::External` carries [`ExternalFailure`](./error.rs), which classifies external-system unavailability at the app boundary. Provider-specific HTTP envelopes, retry headers, DTO parse failures, and transport internals belong in adapter crates such as [`../../../integrations/gingr`](../../../integrations/gingr/README.md) until translated into this app-facing result.

Validation boundaries are expressed with typed requests and `nutype` scalars in [`../tools.rs`](../tools.rs): service notes, snapshot ids, draft ids, portal account/external ids, idempotency keys, authorization/refund ids, message bodies, document references, OCR text, camera ids, media refs, Hermes queue names, schedule names, and schedule ids all trim input and reject empty/overlong strings at construction. That keeps app workflows from accepting vague provider strings as operational truth.

## Safe context, draft validation, and outcome capture

Tools that are exposed to Hermes or another agent runtime should follow the same pattern as [`../../../docs/architecture/agent-app-infrastructure.md`](../../../docs/architecture/agent-app-infrastructure.md):

1. The deterministic app produces typed context packets from `domain`/`storage`/provider-normalized evidence. The agent does not directly own source reads, raw database access, or provider credentials.
2. The agent calls a constrained tool with app-owned types such as `app::tools::availability::Request`, `app::tools::messaging::draft::Request`, or `app::tools::hermes::task::DraftRequest`.
3. The tool returns a typed outcome, draft id, review status, unavailable reason, or `app::tools::Error`. It should not imply that a customer was messaged, a provider/PMS record was mutated, a payment moved, or a schedule changed unless an app policy/review path explicitly executes that side effect.
4. Drafts and recommendations re-enter deterministic app validation before staff review or execution. Accepted outcomes should be captured in workflow/storage records with correlation/source refs; rejected drafts should become reviewable validation failures, not silent tool errors.

Several current types encode this safe posture directly: `messaging::ReviewPolicy::DraftOnly`, `messaging::draft::Status::DraftedRequiresReview`, `payment::authorization::provider::Result::RequiresHumanReview`, `documents::ocr::Result::NeedsHumanReview`, `media::SnapshotResult::Unavailable`, and `hermes::DraftStatus::DraftedRequiresReview` all keep exception triage visible instead of pretending the tool completed live work.

## Cross-crate relationships

- [`domain`](../../../domain/README.md) owns semantic truth used by tool contracts: aggregate ids and records from [`../../../domain/src/entities.rs`](../../../domain/src/entities.rs), money/payment values from [`../../../domain/src/money/mod.rs`](../../../domain/src/money/mod.rs) and [`../../../domain/src/payment/mod.rs`](../../../domain/src/payment/mod.rs), policy denials/review gates from [`../../../domain/src/policy.rs`](../../../domain/src/policy.rs), and workflow events/results from [`../../../domain/src/workflow.rs`](../../../domain/src/workflow.rs).
- [`app`](../../README.md) owns workflows and ports. The umbrella app README documents `app::tools` in its module map, and workflow modules such as [`../booking_triage.rs`](../booking_triage.rs), [`../daily_update.rs`](../daily_update.rs), [`../checkout_completion.rs`](../checkout_completion.rs), [`../crm_retention.rs`](../crm_retention.rs), and [`../manager_daily_brief.rs`](../manager_daily_brief.rs) should consume typed tool results rather than raw adapter data.
- [`storage`](../../../storage/README.md) owns persistence-shaped records and normalized projections. Future storage adapters can implement app store/tool ports, but storage should not redefine workflow behavior or policy.
- [`integrations/gingr`](../../../integrations/gingr/README.md) owns provider DTOs, endpoint builders, transport, webhook verification, and mapping. Gingr ids/statuses become app inputs only after provider-boundary validation and domain/source normalization.
- [`apps/api`](../../../apps/api/src/lib.rs) is the HTTP runtime shell. It should expose app workflows or context/draft endpoints while keeping HTTP parsing/serialization outside `app::tools`.
- [`apps/worker`](../../../apps/worker/README.md) is the background runtime shell that can later wire concrete implementations of `app::tools::AgentRuntime` and `app::tools::hermes::AutomationHooks` while preserving stubbed/fake defaults until safe.
- [`apps/cli`](../../../apps/cli/README.md) currently prints `app::tools::ExternalToolCandidate` through its `pet-resort tools` command, making the candidate integration inventory inspectable without live infrastructure.
- [`../../../docs/architecture/agent-app-infrastructure.md`](../../../docs/architecture/agent-app-infrastructure.md) is the canonical agent/app contract for context packets, draft submission validation, human review gates, audit/replay, and memory boundaries.

## Labor-cost-reduction contribution

`app::tools` contributes to the labor-cost-reduction goal by converting integration work into typed, reviewable contracts:

1. Store and portal ports let workflow code ask for customer, pet, reservation, and lookup facts through typed ids and criteria instead of sending staff to reconcile screens manually.
2. Availability and draft-update packets let reservation workflows produce capacity decisions and reviewable update drafts while keeping live PMS/provider mutation behind policy and adapter boundaries.
3. Payment, messaging, document/OCR, and media ports make exception paths explicit: declines, provider ambiguity, low-confidence OCR, unavailable cameras, and manager-review-required drafts can be triaged instead of disappearing into ad hoc tool logs.
4. `AgentRuntime` and `hermes::AutomationHooks` provide a controlled way for agents to run over typed workflow events and draft tasks/schedules, reducing handoffs without bypassing deterministic app validation.
5. The shared error surface and `ExternalToolCandidate` inventory help maintainers see which integration gaps remain before relying on automation in a manager/front-desk workflow.

The labor saving is not that a tool can do everything. It is that each tool capability has a typed request, typed outcome, and failure/review boundary, so staff and managers spend less time interpreting raw provider/source state and more time reviewing narrow exceptions.

## Maintainer notes

- Add a trait to `app::tools` only when the application layer needs a capability independent of one adapter. If the shape is provider-specific, keep it in `integrations/*` until it can be promoted into an app contract.
- Preserve semantic module paths in docs and code. Prefer `app::tools::payment::authorization::provider::Result`, `app::tools::documents::ocr::ReviewReason`, and `domain::workflow::Result<T>` over flattened names when the boundary matters.
- Keep live side effects explicit. Message drafts, Hermes task/schedule drafts, payment provider results, OCR review outcomes, and media snapshots should expose draft/review/unavailable statuses rather than implying execution.
- When adding an external implementation, translate adapter errors into `app::tools::Error` only after preserving enough source/provider detail for logs, audit, or review records in the adapter layer.
- Update this README when [`../tools.rs`](../tools.rs) adds a new public port, request/outcome family, `ExternalToolCandidate`, or shared error variant.
