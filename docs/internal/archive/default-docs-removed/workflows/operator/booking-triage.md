# Booking Triage

Booking triage saves front-desk teams from turning every new inquiry or reservation request into a scavenger hunt. It assembles a source-backed review packet that shows whether the request is ready for staff approval, missing information, vaccine pending, blocked by capacity or policy, or needs manager/care/behavior/payment review before anyone changes a booking or contacts the guest.

Status: supported local/MVP app contract. The evidence supports source-grounded triage packets, deterministic readiness buckets, staff review drafts, and tests; it does not prove autonomous live booking confirmation, provider/PMS mutation, waitlist movement, vaccine approval, payment capture, or customer-message sending.

Navigation: start with the [operator workflow index](README.md). Entity-first backlinks: [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [source/provenance/data quality](../../design/source-provenance-data-quality-atlas.md), [review gates and blocked actions](../../design/entity-atlas-review-safety-boundaries.md), and [workflow packets](../../design/entity-atlas-workflow-packets-agents.md).

## Problem solved and time saved

- Problem solved: availability, vaccine/document, deposit/payment, behavior, special-care, holiday, staffing, and provider-state facts are often missing, stale, conflicting, or scattered before staff can safely answer a booking request.
- First role whose time is saved: front-desk agents reviewing inquiries, booking requests, and reservation changes.
- Secondary reviewers/operators: front-desk leads, managers, medical/document reviewers, behavior leads, care-team reviewers, and payment/manager approvers.
- Pet-resort example: a holiday boarding inquiry can be ranked as `vaccine_pending` plus `deposit_or_payment_review`, with source refs for missing dates/pet profile/vaccine/payment/policy blockers, a draft missing-info message, and a manager packet ready for staff approval instead of an unsafe confirmation.

## Source data and featured entities

Booking triage should be read as a reviewable [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet). The packet may contain a [draft](../../glossary-workflow-state-terms.md#draft), but source facts stay tied to a [source-of-record](../../glossary-source-data-terms.md#source-of-record) and [source refs](../../glossary-architecture-terms.md#source-ref-domainsourcerecordref) rather than model memory or raw provider text.

| Entity or source fact | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| Inquiry or typed booking request | Establishes customer intent, service, location, requested dates/window, pets, and missing fields before deterministic review. | Customer/staff/provider request source plus app `Request` typestate; missing facts stay `missing_info`. | `docs/workflows/booking-triage-agent.md`; `app/src/booking_triage.rs` `Request`, `RequestState`, `PolicyAttachedData`; `domain/src/entities.rs` `reservation::Source`. |
| Reservation/provider state | Shows whether a record is inquiry/requested/offered/confirmed/waitlisted/cancelled/etc. without letting the agent execute that state. | Provider/PMS state or approved execution evidence, normalized into domain/app facts. | `domain/src/entities.rs` `reservation::Status`; `domain/src/reservation/mod.rs`; `integrations/gingr/src/endpoint/reservations.rs`; `app/src/booking_triage.rs` `StaffEvaluationPacket::suggested_status`. |
| Customer and pet profile | Connects contact preferences, pet identity, species, temperament, care profile, and multi-pet constraints to the request. | Customer/pet source records and normalized domain entities. | `domain/src/entities.rs` `Customer`, `Pet`, `PetProfile`; `app/src/booking_triage.rs` `PetProfileCompleteness`. |
| Service, dates, availability, and capacity | Determines whether the request is service/date supported, whether accommodation or group/play/care-lane capacity exists, and whether waitlist review is needed. | Fresh provider/read-model snapshots plus location policy; staff/manager authority for offers, holds, assignments, overbooking, or waitlist movement. | `docs/workflows/booking-triage-agent.md#deterministic-rules`; `app/src/booking_triage.rs` `rule::Id::{DateRangeAndServiceSupported,AccommodationAvailability,SizeCapacityRoomOrGroupFit,ServiceCapacityAndAddons}`; `integrations/gingr/src/endpoint/reservations.rs` reservation type/search surfaces. |
| Vaccine/document evidence | Prevents confirmation, check-in, or group-play clearance when proof is missing, expired, unverified, OCR-only, unmapped, or conflicting. | Vaccine/document source plus medical/document reviewer approval; agent cannot clear proof. | `docs/workflows/booking-triage-agent.md#required-approval-gates`; `app/src/booking_triage.rs` `FailureCode::MissingOrUnverifiedVaccine`, `ApprovalGate::MedicalDocumentReview`; `app/tests/booking_triage_mvp.rs`. |
| Payment/deposit state | Routes unpaid, disputed, waived, refunded, or policy-unclear deposits to payment/manager review before offer/confirmation. | Verified payment/deposit records and local policy; manager/payment approval for exceptions. | `app/src/booking_triage.rs` `DepositAndPricingRequirements`, `MovePayment`; `app/tests/workflow_service_composition_contracts.rs`; `domain/src/policy.rs` `ReviewGate::RefundOrDepositException`. |
| Behavior, care, medical, and incident notes | Keeps group-play, anxiety/aggression, medication, allergy, mobility, and special-care exceptions under named human review. | Source notes/documents plus behavior/care/manager approvals; sensitive owner-facing copy needs customer-message approval. | `app/src/booking_triage.rs` `BehaviorRestrictions`, `MedicationSpecialCareLimits`, `ApprovalGate::{BehaviorReview,CareTeamApproval}`; `domain/src/entities.rs` pet care/temperament and hard-stop vocabulary. |
| Policy/staffing/holiday snapshots | Keeps local rules, staffing ratios, holiday/minimum-stay/blackout rules, and freshness constraints explicit. | Location policy, staff/manager approvals, and fresh source snapshots. | `docs/workflows/booking-triage-agent.md#source-gaps-and-assumptions`; `app/src/booking_triage.rs` `FailureCode::{StaleSnapshot,MissingPolicy,ConflictingSource}`; `domain/src/policy.rs`. |

Featured entities: booking request/inquiry, reservation, customer, pet, service, deterministic result, staff evaluation packet, confirmation draft, audit event draft, review gate, blocked action.

Related entities to mention without making them the page center:

- Vaccine/document record: evidence and medical/document review input, not autonomous clearance.
- Deposit/payment record: readiness evidence and exception trigger, not a charge/refund/waiver authority.
- Availability/capacity snapshot: source evidence for staff review, not a capacity hold or room assignment.
- Behavior/care/incident notes: sensitive review evidence, not agent-owned safety decisions.
- Data-quality issue: stale, conflicting, unmapped, or missing source facts should route to review or cleanup instead of becoming inferred truth.
- Checkout/retention workflows: downstream consumers after safe booking/stay state exists; booking triage does not perform checkout or retention outreach.

## Featured contracts

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | `app::booking_triage::{Request, Service, DeterministicResult, StaffEvaluationPacket, ConfirmationDraft, MissingInfoDraft, AuditEventDraft, SafeAgentAction, BlockedAction, ApprovalGate, MissingInfoReason, BlockerKind, BlockerEvidence}` | Source-grounded request progression, deterministic readiness ranking, missing-info reason capture, care/vaccine/payment/policy blocker evidence, review packet assembly, draft confirmation/missing-info language for approval, and audit-event draft markers. | Live booking confirmation/rejection, provider/PMS mutation, room/group assignment, waitlist promotion, customer sends, vaccine/care/behavior approvals, or payment movement. |
| `domain` | `domain::entities::{Reservation, Customer, Pet, reservation::Status}`, `domain::policy::ReviewGate`, `domain::workflow::{PolicyContext, RecommendedAction}` | Business vocabulary for reservation state, customer/pet facts, policy review gates, workflow recommendations, and source-aware decisions. | Provider payload truth by itself, approved side effects, or local policy invention. |
| `integrations/gingr` | `integrations/gingr::endpoint::reservations::{reservation::Types, reservation::SearchFilters, Reservations, BackOfHouse, GetServicesByType}` and mapping docs | Provider/read-model evidence for reservation types, dates, animal ids, statuses, services, and back-of-house context. | Domain authority, customer-safe messaging, or direct booking mutation from docs prose. |
| `storage` | No dedicated booking-triage outcome projection is identified in the current evidence map. | Future durable labor/outcome capture could store handle-time, disposition, source-wrong, or reviewed outcome evidence. | Current page must not claim a shipped booking-triage storage outcome record unless code adds one. |

## Authority and source of truth

| Fact or decision | Authority |
| --- | --- |
| Executed provider lifecycle state such as confirmed, checked in/out, cancelled, or rejected | Provider/PMS record or approved execution evidence, normalized into domain reservation status. |
| Request readiness bucket such as `missing_info`, `vaccine_pending`, `special_review`, `waitlisted`, or `failed_safely` | Deterministic app rules in `app::booking_triage::DeterministicResult`; AI explanations cannot upgrade or override it. |
| Availability, room/run/group, capacity hold/release, and waitlist position | Fresh provider/read-model/capacity evidence plus staff/manager approval; agent can only summarize or propose review. |
| Vaccine/document clearance | Medical/document reviewer and policy source; AI may request documents or explain pending review only. |
| Behavior, incident, group-play, anxiety/aggression, or special-care acceptance | Behavior lead, care team, manager, and local policy review gates. |
| Deposit, fee, refund, waiver, discount, or payment reconciliation | Verified payment source and payment/manager approval. |
| Customer-facing copy | Staff/customer-message approval, especially for availability, confirmation, rejection, vaccine, behavior, incident, payment, or policy language. |
| Source ambiguity | Source refs/provenance and data-quality review; ambiguity must remain visible, not hidden by generated prose. |

## Agent work, approvals, and blocked actions

Agent may:

- Explain deterministic rule results and readiness buckets.
- Rank requests by urgency/readiness such as vaccine pending, special review, waitlisted, failed safely, or ready for staff approval.
- Summarize source evidence and missing/stale/conflicting facts.
- Draft internal tasks for front desk, manager, medical document review, behavior review, care-team review, capacity recheck, staffing review, payment review, or provider-data cleanup.
- Draft missing-info requests, customer-safe scripts, manager packets, or confirmation drafts when deterministic gates allow a [draft](../../glossary-workflow-state-terms.md#draft) for review.
- Recommend the next staff-owned action, such as request missing info, prepare manager packet, request human review, or prepare a provider update for approval.

Human must approve:

- Offers, confirmations, denials/rejections, waitlist movement, capacity holds/releases, room/run/group assignments, and provider/PMS status changes.
- Behavior, care, vaccine/document, medical, medication, special-handling, holiday, staffing, capacity, and payment/deposit exceptions through the named [review gate](../../glossary-workflow-state-terms.md#review-gate).
- Customer-facing messages, especially when they mention availability, confirmation, rejection, medical/vaccine judgment, incidents, behavior, safety, payment, or policy exceptions.

Blocked by default:

- Confirm booking, reject request, mutate provider/PMS records, promise availability, assign room/run/group, hold/release capacity, promote from waitlist, check in/out, cancel, or alter lifecycle state.
- Approve vaccines/documents, clear group play, accept special care, override behavior policy, close incident/safety concerns, or decide medical/care exceptions.
- Charge, refund, waive, discount, forfeit, reconcile, or move payment/deposit money.
- Send customer messages or sensitive copy without approval.
- Hide source conflicts, infer missing source facts, expose raw provider payloads or unredacted internal notes, or turn unknown/stale/conflicting evidence into a pass.

## Outcome and labor value

- Estimated labor value: fewer manual front-desk minutes per request spent checking availability, vaccine proof, deposits, behavior/care notes, and manager-review reasons across systems.
- Measured outcome to capture: triage handle time, readiness bucket, rule reason codes, missing/stale/conflicting source fact counts, draft/review disposition, staff-approved vs deferred/suppressed/wrong-source result, and fewer unsafe confirmations.
- Current evidence status: supported local/MVP app contract and tests for deterministic readiness, missing-info reasons, blocker evidence, blocked actions, confirmation/missing-info draft gating, audit-event drafts, and payment/deposit composition.
- Gap/future source need: a dedicated durable booking-triage outcome projection was not identified in the current evidence map, so persistent labor-value reporting should be described as planned/future until storage/API code exists.

## Contract crosswalk links

Use the [workflow packet row](../../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map) for the bidirectional path from this workflow page back to the entities it consumes and produces. Use [surface inventory](../../entity-atlas/contract-crosswalk/surface-inventory.md) for source/Rustdoc/test proof, [source/provider flows](../../entity-atlas/contract-crosswalk/source-provider-flows.md) for source-entry and normalization evidence, [storage/persistence](../../entity-atlas/contract-crosswalk/storage-persistence.md) for `future booking outcome storage gap`, and [runtime exposure](../../entity-atlas/contract-crosswalk/runtime-exposure.md) for API/worker/CLI/web/script exposure. Rustdoc/module path: `app::booking_triage::StaffEvaluationPacket`; operator-facing entity family: `Booking triage packet`.

## Evidence citations

- Source: `app/src/booking_triage.rs` (`app::booking_triage::{Request, Service, DeterministicResult, StaffEvaluationPacket, ConfirmationDraft, MissingInfoDraft, AuditEventDraft, SafeAgentAction, BlockedAction, ApprovalGate, ReadinessBucket, FailureCode, MissingInfoReason, BlockerKind, BlockerEvidence}`) backs deterministic readiness, staff packet assembly, allowed draft work, missing-info/blocker evidence, and blocked actions.
- Source: `domain/src/entities.rs` (`domain::entities::{Reservation, Customer, Pet, reservation::Status, reservation::Source}`) backs normalized reservation/customer/pet/source entities.
- Source: `domain/src/reservation/mod.rs` backs reservation policy vocabulary such as capacity-unavailable, policy-hard-stop, missing-required-information, and staff override transition reasons.
- Source: `domain/src/policy.rs` (`domain::policy::ReviewGate`) backs shared human-review gates for manager, medical document, behavior, customer-message, and refund/deposit exception decisions.
- Source: `domain/src/workflow.rs` (`domain::workflow::{PolicyContext, RecommendedAction}`) backs workflow recommendation and policy-context vocabulary.
- Source: `integrations/gingr/src/endpoint/reservations.rs` backs Gingr reservation type/search/provider-read surfaces; provider values are evidence to normalize, not domain truth by themselves.
- Rustdoc: `target/doc/app/booking_triage/index.html`, `target/doc/domain/entities/index.html`, `target/doc/domain/reservation/index.html`, `target/doc/domain/policy/index.html`, `target/doc/domain/workflow/index.html`, and `target/doc/gingr/endpoint/reservations/index.html` after `cargo doc --no-deps --workspace`.
- Tests: `app/tests/booking_triage_mvp.rs` covers vaccine-review blocking, missing-info reasons, blocker evidence families, staff-bounded confirmation/missing-info drafts, audit events, behavior special review, hard rejection dominance, and deterministic gate rejection of premature confirmation drafts.
- Tests: `app/tests/workflow_service_composition_contracts.rs` covers composition with boarding deposit policy and ensures booking triage blocks payment movement when deposit readiness is unresolved.
- Supporting docs: `docs/workflows/booking-triage-agent.md`; `docs/design/entity-driven-workflow-page-template.md`; `docs/design/operator-workflow-page-inventory.md`; `docs/design/workflow-page-source-rustdoc-map.md`; `docs/safety/review-boundaries-matrix.md`; `docs/safety/evidence-policy-blocked-actions-outcomes.md`.
- Evidence status: supported local/MVP contract; no autonomous booking/provider/payment/vaccine/waitlist/customer-send authority; durable labor outcome storage is planned/future unless a later storage record is added.
