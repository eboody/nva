# Inquiry intake input constraints

Purpose: canonical input handoff for downstream cards that define the `inquiry-intake` agent. This is a source-collection artifact, not live operating policy and not approval for customer-facing sends, booking promises, provider writes, payment actions, medical/vaccine decisions, or task auto-generation in production.

Status: draft input collection for inquiry-intake definitions. Facts below are sourced from current repo artifacts where available; gaps are called out as missing assumptions rather than invented product/runtime behavior.

## Source artifacts checked

- `README.md` — product framing: Rust-first spike for pet-resort workflow agents, typed packets instead of free-form strings, open product questions for inquiry intake and channels.
- `docs/workflows/staff-operations-parts/inputs.md` — current substitute for the missing product-map artifact; includes product shape, MVP emphasis, service scope, actors, approval posture, data-model anchors, and explicit caveats.
- `docs/architecture/pet-resort-workflow-events.md` — canonical event catalog, especially `inquiry.received`, event envelope fields, approval posture, and event-name mapping to Rust enums.
- `docs/workflows/workflow-event-idempotency-replay.md` — idempotency/replay recommendations for `InquiryReceived`, customer-message drafts, outbound sends, internal tasks, and provider writes.
- `docs/architecture/ai-runtime-memory-context-policy.md` — AI/Hermes context packet, minimal fetch policy, inquiry-intake allowed fetch shape, customer-message privacy boundaries, audit/redaction requirements.
- `docs/architecture/workflow-result-envelope.md` — `WorkflowResult`-style result envelope examples for draft customer messages, tasks, verification, unchecked sources, redactions, and human review reasons.
- `domain/src/entities.rs` — implemented data anchors: customer/pet/reservation/location types, `ContactChannel`, `ReservationSource`, service/status enums, hard stops, actors, audit subjects/actions.
- `domain/src/workflow.rs` — implemented workflow event/result anchors: `WorkflowEvent`, `WorkflowEventType::InquiryReceived`, `WorkflowSubject`, `PolicyContext`, `AllowedAction`, `WorkflowResult`, `WorkflowStatus`, and `RecommendedAction`.
- `domain/src/agents.rs` — baseline `inquiry-intake` and `lead-conversion` agent specs, allowed tools, forbidden actions, and default review gates.
- `domain/src/tools.rs` — app-owned repository/runtime/tool traits, `CustomerStore`, `AgentRuntime`, `MessageDrafting`, `DeliveryChannel`, and draft-message review policy.
- `docs/integrations/gingr/sdk-readiness-review.md`, `docs/integrations/gingr/sdk-webhooks.md`, and `integrations/gingr/src/webhook.rs` — Gingr/provider webhooks are verified boundary inputs; `lead_created` can map to inquiry only after signature verification/normalization.

## Product/source-of-truth constraints

- The requested product-map path `docs/product/pet-resort-product-map.md` is not present. `docs/workflows/staff-operations-parts/inputs.md` records the same gap and currently serves as the best repo-local product-scope source.
- Current product shape is an internal staff/manager workflow and operations tool for a single pet resort or small resort group, with later vertical-SaaS path after proof.
- Core actors for intake: customer/owner, pet as deterministic record, front-line staff/caregiver, manager/admin, vet/emergency contact, system, and bounded AI workflow worker.
- Services to capture during inquiry intake: dog boarding, cat boarding, dog daycare/day play, day boarding/individual play, grooming/bathing/DaySpa, training where offered, loyalty/memberships when relevant, and optional webcam/customer-update experience.
- MVP emphasis includes intake/missing-info handling, pet/customer/reservation records, vaccine/document review queues, staff tasking, capacity/availability snapshots, audit events, and draft messages/summaries.
- V1 posture: final vaccine/eligibility/group-play decisions, booking acceptance/exceptions, deposit/payment/refund/loyalty handling, high-risk customer-facing sends, and live Gingr/provider mutations are manual/review-gated unless later approved by explicit policy.

## Canonical intake event

Primary semantic event:

- Documentation name: `inquiry.received`.
- Rust enum name: `WorkflowEventType::InquiryReceived`.
- Source triggers: lead form, customer portal inquiry, phone/email/SMS intake entered by staff, or verified provider lead webhook/poll.
- Actor: customer/owner when self-submitted; staff when entered manually; system for provider import.
- Subject: `Customer` when mapped, otherwise `External` lead/provider record.
- Required envelope anchors: `event_id`, `event_type`, `occurred_at`, `actor`, `location_id`, `subject`, `policy_context`, and source/audit references. The architecture doc also requires source, related ids, typed payload, and result in the broader event envelope.
- Related IDs to preserve when present: `location_id`, optional `customer_id`, optional `pet_ids`, optional `lead_id`/provider id, optional `source_message_id`.
- Expected payload shape: contact channel, customer name/contact refs, requested service/date range, pet summary if supplied, free-text needs reference, source/evidence refs, and consent/contact preferences if known.
- Policy context: location intake policy, contact-permission policy, service availability snapshot if already evaluated, and review gates for ambiguous/sensitive text.
- Allowed actions: `ReadEntities`, `CreateInternalTask`, optional `DraftCustomerMessage` for acknowledgement/follow-up draft, and `FlagRisk` for urgent/sensitive content.
- Expected result: inquiry-intake summary, optional customer-follow-up task draft, optional message draft, and status `NeedsMoreInformation` or `Completed` as a safe recommendation/draft result.

## Current data-model anchors

Existing implemented fields/types useful for inquiry intake:

- Identity: `LocationId`, `CustomerId`, `PetId`, `ReservationId`, `StaffId`, `ManagerId`.
- Actor: `ActorRef::{Customer, Staff, Manager, System, Agent}`.
- Customer: `Customer { id, full_name, email, mobile_phone, preferred_contact, portal_account }`.
- Contact preference: `ContactChannel::{Email, Sms, Phone, Portal}`.
- Portal: `PortalAccountRef { provider, external_customer_id }`, `PortalProvider::{Gingr, Other}`.
- Pet: `Pet { id, customer_id, name, species, birth_date, sex, spay_neuter_status, temperament, care_profile }`; `Species::{Dog, Cat, Other}`.
- Reservation/request: `Reservation { location_id, customer_id, pet_ids, service, status, starts_at, ends_at, deposit, source, requested_add_ons, hard_stops }`.
- Services: `ServiceKind::{Boarding, DayPlay, DayBoarding, Grooming, Training, DaySpa}`.
- Statuses relevant to intake: `ReservationStatus::{Inquiry, Requested, MissingInfo, VaccinePending, SpecialReview, Waitlisted, Offered, Confirmed, ...}`. Intake should not autonomously progress to confirmed/provider-mutating states.
- Request sources currently modeled: `ReservationSource::{Portal(PortalProvider), WebsiteForm, PhoneTranscript, Sms, Email, StaffCreated}`.
- Hard stops to route, not decide: `MissingRequiredVaccine`, `IneligibleForGroupPlay`, `InHeat`, `AgeBelowMinimumWeeks`, `MedicalOrMedicationReviewRequired`, `BehaviorReviewRequired`, `DepositRequired`.
- Workflow result: `WorkflowResult<T>` with `WorkflowStatus::{Completed, NeedsHumanReview, RejectedByPolicy, NeedsMoreInformation, FailedSafely}`, `recommended_actions`, `risk_flags`, `verification`, and optional `human_review_reason`.

## Inquiry source channels

Canonical inbound channel mapping for downstream design:

| Required source channel | Current repo anchor | Handling boundary |
| --- | --- | --- |
| Website form | `ReservationSource::WebsiteForm`; `inquiry.received` source trigger `Lead form` | Treat submitted fields as inquiry evidence. Create/match customer/pet/request only through deterministic repositories/mapping. Do not promise booking/availability. |
| SMS | `ReservationSource::Sms`; `ContactChannel::Sms`; `DeliveryChannel::Sms` for drafts/sends | Use minimal message excerpt and source message id. Customer follow-up can be drafted but outbound send is separately approval-gated. |
| Email | `ReservationSource::Email`; `ContactChannel::Email`; `DeliveryChannel::Email`; Gingr `email_sent` exists as provider observation | Use minimal relevant excerpts, not whole inboxes/threads. Emails/screenshots are not payment or policy truth before reconciliation. |
| Phone transcript | `ReservationSource::PhoneTranscript`; `ContactChannel::Phone` | Treat transcript as staff-entered/customer-message evidence requiring provenance, timestamp, and uncertainty notes. Sensitive/ambiguous content routes to review. |
| Chat widget | No current `ReservationSource`, `ContactChannel`, or `DeliveryChannel` variant found for chat widget | Model as missing source-channel vocabulary. Downstream card should decide whether chat maps to `WebsiteForm`, `Portal`, or a new `ChatWidget`/`WebChat` source; do not silently collapse if channel-specific consent, transcript, or send behavior matters. |
| Customer portal / provider lead | `ReservationSource::Portal(PortalProvider)`; `ContactChannel::Portal`; `PortalProvider::Gingr`; provider `lead_created` can map to `inquiry.received` only after verification | Raw provider events are boundary inputs; signature verification, source normalization, and identity reconciliation must occur before semantic workflow routing. |
| Staff-created/manual entry | `ReservationSource::StaffCreated`; `ActorRef::Staff` | Preserve staff actor/source refs and original channel if known. Manual entry should not erase source-message evidence or customer consent uncertainty. |

## Minimal prompt/runtime packet for the inquiry-intake agent

Based on `docs/architecture/ai-runtime-memory-context-policy.md`, the direct Hermes/AI packet should contain only:

- `workflow_name` / agent spec name, version, goal, and output schema name.
- `WorkflowEvent` routing fields: `event_id`, `event_type = InquiryReceived`, `occurred_at`, minimized actor role/id, `location_id`, `subject`, `policy_context`, allowed actions, review gates, and forbidden actions.
- Source record IDs and time windows, not broad database rows or whole message inboxes.
- Minimal, redacted source excerpts only when the worker cannot fetch by ID and the workflow allows excerpts.
- For inquiry intake / lead conversion, typical fetch-by-ID allowance is: customer/lead record, pet names/species, requested service/date, prior approved contact preference, and approved policy snippets.
- Minimized payload shape should be: missing-field checklist, service/date preferences, safe reply draft facts, source citations, and uncertainty/review gates; avoid full history unless directly relevant.
- Output must be a `WorkflowResult`-style structured packet with citations/source refs, redactions, unchecked sources, risk flags, review reason, optional internal-task draft, and optional customer-message draft.

## Agent contract anchors

Current baseline specs in `domain/src/agents.rs`:

- `inquiry-intake`: purpose is to extract new customer/pet/service/date details, identify missing info, and draft safe follow-up replies. Allowed tools: `portal-read`, `crm-read`, `task-create`. Forbidden actions: `confirm booking`, `send sensitive message without approval`. Default gate: `CustomerMessageApproval`.
- `lead-conversion`: purpose is to classify inquiry intent, identify missing intake requirements, and draft next-best follow-up for boarding, daycare, grooming, or training leads. Allowed tools: `lead-read`, `customer-read`, `portal-read`, `draft-message`. Forbidden actions: `book reservation`, `promise availability`, `override requirements`. Default gate: `CustomerMessageApproval`.

Downstream cards should decide whether `lead-conversion` remains a separate follow-up workflow or becomes a stage/result subtype of inquiry intake. Do not merge them by assumption.

## Customer-message boundaries

- Inquiry acknowledgement/follow-up messages are drafts requiring human approval unless the product owner later approves a deterministic receipt-only acknowledgement path.
- No booking promise, availability promise, waitlist/denial, confirmation, policy exception, payment/deposit instruction, refund/waiver/discount claim, medical/vaccine decision, group-play eligibility decision, or sensitive incident/safety language may be sent by the inquiry-intake agent.
- Draft creation and outbound send are separate side effects. Drafts are keyed by draft intent/evidence/template/policy; sends require approved draft/version, recipient, channel, intent, and approval id.
- Include a customer first name/preferred channel or one specific email/phone only when required for a draft/review packet or approved send workflow.
- Include only minimal, relevant message excerpts for intake/missing-info handling. Do not send whole inboxes, historical threads, unrelated profile fields, broad contact exports, raw payment payloads, unredacted documents/OCR, or provider webhook bodies to the AI runtime.
- Customer claims, free-text notes, screenshots, emails, attachments, chat messages, and unverified webhooks are evidence, not authoritative payment/policy/medical truth before reconciliation.
- Runtime/audit logs should store manifests, refs, hashes, field/category lists, redaction profile, citations, and review gates rather than raw sensitive prompt/completion text by default.

## Idempotency and replay constraints for inquiry intake

Use `docs/workflows/workflow-event-idempotency-replay.md` as the current design recommendation:

- `source_event_key = v1:{location_id}:{event_type}:{primary_subject}:{source_kind}:{source_fingerprint}`.
- For `InquiryReceived`, primary subject is `Customer` when known, otherwise `External`.
- Key shape: `location + InquiryReceived + customer/external lead id + source_kind + lead/provider event id or normalized contact+submitted_at hash`.
- Duplicate inquiry evidence should merge into one lead/review packet when contact + source + submitted timestamp match.
- Conflicting contact/profile fields should create review rather than overwrite.
- Replays may rebuild lead summary and missing-info recommendation.
- Backfills must not send conversion messages unless campaign/send approval exists.
- Internal lead/follow-up task key: `internal_task:customer/external:inquiry_follow_up:{policy_version}`.
- Draft message key includes template/copy version; outbound customer send has a separate approved-send key so duplicate lead webhooks cannot double-message a customer.

## Missing assumptions / gaps for downstream cards

1. `docs/product/pet-resort-product-map.md` is absent; cite `docs/workflows/staff-operations-parts/inputs.md` and this file until the product map is restored or synthesized.
2. Chat widget is a required channel in this card but is not represented in current `ReservationSource`, `ContactChannel`, or `DeliveryChannel`. A downstream data-model card should add or explicitly map chat before implementation.
3. No canonical inquiry-intake input/output Rust struct exists yet. Current anchors are generic `WorkflowEvent`, `AgentPromptPacket<T>`, and `WorkflowResult<T>`.
4. No explicit lead/inquiry entity type was found beyond `WorkflowSubject::External`, reservation `Inquiry` status, provider lead references, and baseline agent specs. Downstream design must decide whether an inquiry becomes a reservation request immediately or has a separate lead/inquiry aggregate.
5. Contact consent/preference policy is named in the workflow architecture but not yet represented as a concrete policy type in the current domain model.
6. Website form field schema, SMS/email/phone/chat transcript metadata schemas, and source-message storage/evidence IDs are not currently defined in repo artifacts.
7. Production LLM provider/model, retention settings, exact field allowlist, and tool permissions remain approval-gated by the AI runtime policy.
8. Provider-specific Gingr `lead_created` mapping is available only as a boundary trigger after signature verification/normalization; it is not semantic authority by itself.
9. Availability/capacity, vaccine/medical, payment/deposit, group-play/behavior, policy-exception, and customer-message approvals are outside inquiry-intake authority and must route to later workflow/review gates.

## Concise downstream handoff

Define the inquiry-intake agent around `WorkflowEventType::InquiryReceived` / `inquiry.received`. Its canonical input should be a minimized event/prompt packet containing source channel, source/evidence refs, location, actor, mapped or external subject, optional customer/pet IDs, requested service/date range, pet summary if supplied, contact preference/consent if known, redacted message excerpt, and policy/review context. It may read approved lead/customer/pet/request/policy snippets, detect missing fields, draft internal follow-up tasks, draft customer follow-up copy, and flag risks. It must not confirm bookings, promise availability, make medical/vaccine/payment/eligibility/policy-exception decisions, mutate providers, or send customer messages without a separate approved send path.
