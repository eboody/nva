---
title: "PetSuites core entity atlas: reservations, service lines, pets, customers, and care facts"
slug: "petsuites-core-entity-atlas"
family: "reservations-boarding-daycare-pets-care"
status: "draft"
audience: ["front-desk", "daycare-lead", "boarding-lead", "general-manager", "regional-ops", "docs-writer"]
plain_english_definition: "A family atlas for the booking, stay, pet, customer, safety, and staff facts that front-desk and care teams use before confirming, checking in, updating, checking out, or escalating pet-resort work."
primary_labor_problem: "Reduces repeated dashboard reconciliation and unsafe handoff rewriting by showing which facts drive capacity, eligibility, care, incident, customer-message, and outcome review queues."
source_of_record: "Domain source and promoted provider/source evidence; staff/manager approval for policy, safety, customer-message, payment, and provider-write decisions."
authoritative_human_role: "front desk lead, daycare/boarding lead, trained care staff, or general manager depending on the fact"
workflow_links: ["booking-triage", "daily-care-update", "checkout-completion", "incident-escalation", "manager-daily-brief", "data-quality-hygiene"]
source_paths:
  - "domain/src/entities.rs"
  - "domain/src/reservation/mod.rs"
  - "domain/src/boarding/mod.rs"
  - "domain/src/daycare/mod.rs"
  - "domain/src/{pet,customer,vaccine,temperament,incident,care,location,staff}.rs"
  - "domain/src/message.rs"
  - "app/src/daily_update.rs"
  - "app/src/crm_retention.rs"
  - "app/src/tools.rs"
  - "storage/src/service_line/{boarding,daycare}.rs"
rustdoc_contracts:
  - "domain::entities::{Reservation, Pet, Customer, Message, VaccineRecord, Incident, CareProfile, TemperamentProfile, Location, StaffId, ManagerId}"
  - "domain::message::{Direction, Channel, Status, BodyRef}"
  - "app::tools::messaging::*"
  - "domain::reservation::*"
  - "domain::boarding::*"
  - "domain::daycare::*"
  - "storage::service_line::{boarding,daycare}::*"
glossary_links:
  - "../glossary-source-data-terms.md#source-of-record"
  - "../glossary-workflow-state-terms.md#review-gate"
  - "../glossary-workflow-state-terms.md#blocked-action"
allowed_action_summary: "read source-backed evidence; validate deterministic policy/readiness gates; draft packets, staff tasks, and customer copy for review; rank work queues; record dispositions and outcome evidence"
blocked_action_summary: "no autonomous customer sends, provider/PMS writes, booking/check-in/checkout/room/group/schedule changes, payment movement, vaccine/medical/behavior/incident approvals, staffing changes, or policy exceptions"
outcome_fields: ["review disposition", "source refs", "review gate", "staff persona", "actual minutes saved", "follow-up status", "incident or care outcome", "data-quality resolution"]
---

# PetSuites core entity atlas

This page helps front-desk, boarding, daycare, and manager teams avoid rechecking the same pet-resort dashboards and free-text notes before a booking, check-in, daily update, checkout, or incident review. It explains the core reservation, pet, customer, service-line, and care/safety entities in operator English, names the source or human authority for each fact, and keeps every customer-, provider-, payment-, schedule-, and pet-safety action behind its required [review gate](../glossary-workflow-state-terms.md#review-gate).

This is a family atlas page rather than one Markdown file per entity. The entities below are first-class enough for non-coder writers and reviewers to find, but they share the same source-authority and safety boundary: source/Rustdoc contracts are authoritative; this Markdown is navigation and review aid.

## 1. Plain-English pet-resort definition

The PetSuites core entity family is the set of facts staff use to answer everyday operating questions:

- Can this reservation move toward staff approval, waitlist, check-in, checkout, or follow-up?
- Is the requested boarding stay within capacity, deposit, cancellation, care, handoff, and housekeeping rules?
- Is the daycare pet eligible for group play, assigned safely, covered by staff ratio, and routed to the correct front-desk lane?
- Which customer, pet, vaccine, care, temperament, incident, location, and staff facts prove the recommendation or block it?
- Which outcome fields prove labor saved or route unresolved work to a human reviewer?

The family should be read together with the [entity atlas inventory](entity-atlas-inventory.md), [entity atlas page template](entity-atlas-page-template.md), [NVA documentation style guide](../quality/nva-documentation-style-guide.md), and the linked source paths.

## 2. Purpose: labor-cost and safety problem

This page helps the front desk avoid repeated booking-readiness reconciliation, helps care teams avoid unsafe note handoffs, and helps managers see why capacity, eligibility, incident, care, and customer-message decisions remain human-reviewed. The safe outcome is a source-backed packet, review lane, draft, task, or outcome record; it is never an automatic confirmation, customer send, provider write, payment movement, room/group assignment, vaccine clearance, incident closure, or staffing change.

## 3. Workflows where this family appears

| Workflow | How the family appears | Safe workflow result |
| --- | --- | --- |
| [Booking Triage](../workflows/booking-triage-agent.md) | Reservation, customer, pet, service, capacity, vaccine, care, behavior, payment, staff coverage, and source evidence drive readiness buckets. | Staff review packet, missing-info draft, manager/care/payment/behavior review gate, or failed-safe data-quality task. |
| Checkout Completion | Reservation/stay status, care summary, checkout handoff, incident/payment exceptions, and customer follow-up readiness are compared before a departure status is suggested. | Checkout packet and audit draft; no autonomous checkout/provider mutation. |
| [Daily Care Update](../workflows/daily-care-update-agent.md) | Pet, reservation, care profile, care notes, temperament, incident, message, media/document refs, and staff review state become customer-safe draft evidence. | Draft daily update, omitted-fact record, internal flag, or staff/manager review reason. |
| [Incident Escalation](../workflows/incident-escalation-agent.md) | Incident, pet, customer, reservation, care, temperament, staff, media/document, and audit refs build a manager/lead packet. | Provisional summary, severity candidate, follow-up tasks, review gates, and owner-message draft for approval. |
| Manager Daily Brief | Location, reservation/stay, capacity, daycare coverage, care-watch, checkout, incident, and outcome facts become ranked manager actions. | Ranked action and outcome record with actual minutes/disposition when reviewed. |
| Data Quality Hygiene | Missing/stale/duplicate/conflicting customer, pet, vaccine, reservation, source, or care facts become cleanup candidates. | Review candidate, cleanup draft, resolution status, and labor outcome record. |

## 4. Relationships and adjacency

```text
Gingr/provider/source evidence + staff-entered evidence
  -> source refs / provenance / freshness
  -> location policy + service-line contract
  -> customer + pet + care/safety facts
  -> reservation or attendance/stay context
  -> booking/daycare/boarding/daily-update/incident packet
  -> review gate + allowed draft/rank/validate action
  -> staff disposition + audit/outcome evidence
```

Do not collapse these concepts:

- A provider id is not the customer, pet, reservation, or location itself.
- A storage code is not the domain policy; it is a durable projection of a source/domain value.
- A recommended room, suite, yard, playgroup, or lane is not an assignment.
- A vaccine/document status is not medical approval until the appropriate reviewer records it.
- A temperament or incident summary is not group-play clearance or restriction removal.
- A customer message draft is not a send, and a staff task draft is not task completion.

## 5. Entity family entries

### Reservation

| Field | Writer value |
| --- | --- |
| Plain-English definition | The booking or stay record that ties a customer, pet, location, service kind, date window, status, add-ons, deposit/payment state, care/safety context, and source evidence together. |
| Labor or safety problem reduced | Prevents front-desk staff from rechecking multiple dashboards before deciding whether a request is missing info, vaccine pending, special review, waitlisted, offered, confirmed by source, checked in, active, checked out, cancelled, or rejected by an approved path. |
| Workflows where it appears | Booking Triage, Checkout Completion, Daily Care Update, CRM Retention, Manager Daily Brief, Data Quality Hygiene. |
| Related entities / adjacency | Customer owns the booking; pets are subjects; location policy and service-line contracts define the operating context; care, vaccine, temperament, incident, payment, source, staff, message, audit, and outcome facts explain the next safe action. |
| Source paths | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/reservation/mod.rs`](../../domain/src/reservation/mod.rs), [`domain/src/reservation/README.md`](../../domain/src/reservation/README.md), [`app/src/booking_triage.rs`](../../app/src/booking_triage.rs), [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs). |
| Rustdoc/module/type contracts | `domain::entities::Reservation`, `domain::entities::reservation::{Id, Status, Source}`, `domain::reservation::{MinimumAgeWeeks, AgeThreshold, AddOnLabel, TransitionReason}`. |
| Source of record | Provider/source reservation evidence plus promoted domain reservation contract; staff approval for lifecycle execution or exception states. |
| Authoritative human role | Front desk lead for routine readiness, manager for policy/capacity/payment exceptions, care/behavior reviewer for safety gates. |
| Allowed actions | Validate required facts, summarize source status, draft missing-info or manager packets, rank reservation queues, record review/audit/outcome evidence. |
| Blocked actions / review gates | No autonomous confirmation, rejection, check-in, checkout, cancellation, waitlist promotion, capacity hold/release, room/run/group assignment, provider/PMS write, payment movement, customer send, or policy/safety approval. |
| Safe-use evidence / outcome fields | Reservation id, location id, customer/pet ids, service kind, source refs/provenance, policy refs, review gates, freshness, staff disposition, audit event, actual minutes saved/wasted. |
| Examples / non-examples | Example: a booking triage packet with vaccine_pending and source refs. Example: a checkout handoff comparing provider checkout status with staff handoff. Non-example: a raw Gingr reservation id by itself. Non-example: generated “confirmed” wording without provider or approval evidence. |

### Boarding contract

| Field | Writer value |
| --- | --- |
| Plain-English definition | The overnight-stay rule set for accommodation, room/suite capacity, deposits, cancellation, care readiness, housekeeping, handoff, minimum stays, and boarding-specific upsell opportunities. |
| Labor or safety problem reduced | Replaces manual policy lookup for “can we promise this boarding stay?” with named capacity, care, payment, and handoff decisions that staff can review. |
| Workflows where it appears | Booking Triage, Checkout Completion, Daily Care Update, Manager Daily Brief, Data Quality Hygiene. |
| Related entities / adjacency | Reservation supplies the stay; pet and care profile supply safety requirements; location owns local policy; source/provider snapshots supply inventory/status; storage projections preserve service catalog and core service contracts. |
| Source paths | [`domain/src/boarding/mod.rs`](../../domain/src/boarding/mod.rs), [`domain/src/boarding/README.md`](../../domain/src/boarding/README.md), [`domain/src/boarding/capacity.rs`](../../domain/src/boarding/capacity.rs), [`domain/src/boarding/care.rs`](../../domain/src/boarding/care.rs), [`domain/src/boarding/deposit.rs`](../../domain/src/boarding/deposit.rs), [`domain/src/boarding/handoff.rs`](../../domain/src/boarding/handoff.rs), [`storage/src/service_line/boarding.rs`](../../storage/src/service_line/boarding.rs). |
| Rustdoc/module/type contracts | `domain::boarding::Contract`; `capacity::{Snapshot, Request, Policy, Decision}`; `care::{Plan, Readiness, ReviewGate}`; `deposit::{Policy, ConfirmationReadiness, Blocker}`; `handoff::Requirement`; `storage::service_line::boarding::{ContractRecord, AccommodationCode, CareFeatureCode, AddOnCode}`. |
| Source of record | `domain::boarding::Contract` and policy modules for semantic meaning; provider/read-model evidence for inventory/stay facts; storage for durable projections; manager/front desk for exceptions. |
| Authoritative human role | Boarding/front-desk lead for readiness and handoff; general manager for capacity, payment, cancellation, or policy exceptions; trained care staff for feeding/medication ambiguity. |
| Allowed actions | Evaluate availability/readiness from source snapshots, surface waitlist/denial/review reasons, draft internal tasks or customer-safe scripts for review, record outcome evidence. |
| Blocked actions / review gates | No autonomous booking, cancellation, overbooking, room assignment, capacity hold/release, deposit/refund/fee decision, medical/care acceptance, or customer upsell send. |
| Safe-use evidence / outcome fields | Nightly capacity snapshot refs, accommodation preference, species compatibility, care-plan readiness, medication review gate, deposit status, cancellation notice, handoff requirement, review disposition, minutes saved. |
| Examples / non-examples | Example: capacity policy returns waitlist with a manager-review reason. Example: boarding care plan blocks check-in when feeding instructions are missing. Non-example: storage `AccommodationCode` treated as full boarding policy. Non-example: exit-bath upsell sent to a customer without approval. |

### Daycare contract

| Field | Writer value |
| --- | --- |
| Plain-English definition | The daytime-care rule set for service variant, attendance, package policy, staff-to-pet ratio, group-play eligibility, playgroup assignment, incident restrictions, front-desk lanes, and package-opportunity review. |
| Labor or safety problem reduced | Keeps busy check-in and playgroup decisions from becoming ad hoc manual judgment by naming fast lane, collection, care-team review, manager review, policy block, waitlist, or package-review outcomes. |
| Workflows where it appears | Booking Triage, Daily Care Update, Incident Escalation, Manager Daily Brief, Data Quality Hygiene, daycare staff operations. |
| Related entities / adjacency | Pet supplies species/spay-neuter/temperament/care facts; vaccine and incident facts gate eligibility; reservation/attendance gives the day context; staff coverage sets ratio; customer message policy gates package/update outreach. |
| Source paths | [`domain/src/daycare/mod.rs`](../../domain/src/daycare/mod.rs), [`domain/src/daycare/README.md`](../../domain/src/daycare/README.md), [`domain/src/daycare/attendance.rs`](../../domain/src/daycare/attendance.rs), [`domain/src/daycare/coverage.rs`](../../domain/src/daycare/coverage.rs), [`domain/src/daycare/eligibility.rs`](../../domain/src/daycare/eligibility.rs), [`domain/src/daycare/assignment.rs`](../../domain/src/daycare/assignment.rs), [`domain/src/daycare/front_desk.rs`](../../domain/src/daycare/front_desk.rs), [`domain/src/daycare/incident.rs`](../../domain/src/daycare/incident.rs), [`storage/src/service_line/daycare.rs`](../../storage/src/service_line/daycare.rs). |
| Rustdoc/module/type contracts | `domain::daycare::{Contract, ServiceVariant, CareMode, AttendancePolicy, PackagePolicy, StaffPetRatio, GroupAssignmentRule, EligibilityRequirement}`; `eligibility::{Evidence, GroupPlayPolicy, GroupPlayDecision}`; `assignment::{Request, Service, Decision}`; `front_desk::{ThroughputPolicy, QueueTicket, QueueLane}`; `storage::service_line::daycare::{ContractRecord, FormatCode, EligibilityRuleCode}`. |
| Source of record | `domain::daycare::Contract` and child policy modules for semantics; provider/reservation/attendance/source evidence for the day; staff roster/read model for coverage; manager/care reviewers for exceptions. |
| Authoritative human role | Daycare lead or care-team reviewer for group play and care lanes; manager for ratio/incident/eligibility exceptions; front desk for routine check-in queue. |
| Allowed actions | Materialize attendance candidates, evaluate ratio/eligibility/readiness, produce queue tickets, draft package/review tasks, record review and outcome evidence. |
| Blocked actions / review gates | No autonomous admission, playgroup placement, group-play override, incident restriction clearance, package sale/enrollment, billing action, provider write, customer update/send, or manager override. |
| Safe-use evidence / outcome fields | Attendance recurrence/date, coverage snapshot, group-play evidence, vaccine readiness, temperament freshness, incident restriction, queue lane, review gate, staff disposition, minutes saved. |
| Examples / non-examples | Example: front-desk throughput policy sends a pet to care-team review because medication/care readiness is unresolved. Example: group-play policy returns temporarily suspended after a daycare incident restriction. Non-example: `FormatCode::all_day_play` alone as eligibility proof. Non-example: a recommended playgroup candidate treated as an assignment. |

### Pet

| Field | Writer value |
| --- | --- |
| Plain-English definition | The animal profile staff use to connect species, sex, spay/neuter status, temperament, care plan, vaccine proof, incidents, reservations, and customer ownership. |
| Labor or safety problem reduced | Prevents unsafe booking/daycare/daily-update decisions by keeping the actual pet subject visible instead of relying on customer free text or provider ids. |
| Workflows where it appears | Booking Triage, Daily Care Update, Incident Escalation, Data Quality Hygiene, Manager Daily Brief. |
| Related entities / adjacency | Customer owns the pet; reservations/stays/attendance provide time-bounded context; vaccine, care, temperament, incident, document, message, and audit records attach to the pet. |
| Source paths | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/pet.rs`](../../domain/src/pet.rs), [`domain/src/care.rs`](../../domain/src/care.rs), [`domain/src/temperament.rs`](../../domain/src/temperament.rs), [`domain/src/vaccine.rs`](../../domain/src/vaccine.rs), [`domain/src/incident.rs`](../../domain/src/incident.rs). |
| Rustdoc/module/type contracts | `domain::entities::{PetId, Pet, Species, Sex, SpayNeuterStatus, TemperamentProfile, CareProfile}`; `domain::pet::Name`. |
| Source of record | Promoted domain pet record with provider/source provenance; staff/care review for sensitive behavior/medical/care corrections. |
| Authoritative human role | Front desk for identity cleanup, care team for care/behavior facts, manager for eligibility-impacting exceptions. |
| Allowed actions | Summarize source-backed pet facts, flag missing/stale/conflicting profile data, draft internal cleanup/care tasks, link facts to reservation packets. |
| Blocked actions / review gates | No autonomous pet-profile overwrites, group-play clearance, medical/care acceptance, incident flag changes, or customer-sensitive behavior/health messaging. |
| Safe-use evidence / outcome fields | Pet id, source refs, owner/customer link, current profile version, care/vaccine/temperament/incident refs, review status, correction reason, outcome disposition. |
| Examples / non-examples | Example: “Maple needs behavior review because temperament evidence is stale.” Example: “Rex has current source-backed feeding instructions for check-in.” Non-example: an `AnimalId` provider wrapper alone. Non-example: a generated breed/size assumption from notes. |

### Customer

| Field | Writer value |
| --- | --- |
| Plain-English definition | The pet parent/customer profile that owns pets, contact channels, portal references, booking requests, message permissions, and follow-up context. |
| Labor or safety problem reduced | Reduces repeated customer lookup and duplicate cleanup while preventing unapproved customer sends or contact-channel mistakes. |
| Workflows where it appears | Booking Triage, Daily Care Update, CRM Retention, Customer Messaging, Data Quality Hygiene, Incident Escalation. |
| Related entities / adjacency | Customer owns pets and reservations; contact channel and portal refs feed message drafts; payment/package/retention facts may appear in app workflows but do not authorize outreach by themselves. |
| Source paths | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/customer.rs`](../../domain/src/customer.rs), [`domain/src/portal.rs`](../../domain/src/portal.rs), [`domain/src/message.rs`](../../domain/src/message.rs), [`integrations/gingr/src/mapping/customer.rs`](../../integrations/gingr/src/mapping/customer.rs). |
| Rustdoc/module/type contracts | `domain::entities::{CustomerId, Customer, PortalAccountRef, ContactChannel}`; `domain::customer::{Name, Email, Phone}`; `domain::portal::CustomerId`. |
| Source of record | Promoted domain customer/contact record plus provider/portal evidence; customer-message policy and human approval for sends. |
| Authoritative human role | Front desk for routine contact/profile verification; manager or approved sender for sensitive messages or customer-impacting exceptions. |
| Allowed actions | Validate and summarize contact facts, draft missing-info or customer-message copy for review, create duplicate/source-hygiene candidates, record contact approval/disposition. |
| Blocked actions / review gates | No autonomous sends, opt-in/consent changes, duplicate merges, provider writes, billing/payment actions, policy promises, or sensitive health/behavior/payment/incident messages. |
| Safe-use evidence / outcome fields | Customer id, contact channel/source ref, portal ref, consent/message policy, approval record, duplicate candidate evidence, follow-up outcome, correction reason. |
| Examples / non-examples | Example: a missing rabies-document request draft that requires approval. Example: duplicate customer cleanup candidate with evidence refs. Non-example: phone/email string used as consent. Non-example: provider owner id treated as canonical identity. |

### Message and message state

| Field | Writer value |
| --- | --- |
| Plain-English definition | A message is a customer, staff, or internal communication record that tracks subject, direction, channel, lifecycle status, body reference, approval gate, and audit refs. It can be a draft, approval request, queued item, sent/delivered item, failed send, or suppressed/cancelled record; it is not arbitrary chatbot text or a promise that a message already went out. |
| Labor or safety problem reduced | Prevents unsafe customer sends, repeated handoff rewriting, and hidden sensitive-fact disclosure by preserving the exact message state and the approval evidence needed before staff use the draft. |
| Workflows where it appears | Daily Updates/Pawgress, Retention/Grooming Rebooking, Booking Triage, Incident Escalation, Data Quality Hygiene, Customer Messaging, and manager/front-desk review queues. |
| Related entities / adjacency | Customer is usually the recipient/owner; pet, reservation, care note, incident, approval record, and workflow event explain the subject; review gate and approval record decide send authority; draft/send stub, audit event, outcome record, provider/source refs, channel permission, and included/omitted facts prove what was drafted, sent, blocked, or suppressed. |
| Source paths | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/message.rs`](../../domain/src/message.rs), [`app/src/daily_update.rs`](../../app/src/daily_update.rs), [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs), [`app/src/tools.rs`](../../app/src/tools.rs). |
| Rustdoc/module/type contracts | `domain::entities::Message`; `domain::entities::{MessageId, MessageSubject, approval::Record, audit::Event}`; `domain::message::{Direction, Channel, Status, BodyRef}`; `app::tools::messaging::*`; daily-update `CustomerMessageDraft`, `IncludedFact`, `OmittedFact`, and `SendStub`; CRM retention follow-up/contact-permission packet types. |
| Source of record | Domain message state plus source/provider/contact evidence for recipient/channel facts; app workflow packets for draft, omitted/included facts, send stubs, and blocked reasons; approval/audit records for reviewed outcomes. |
| Authoritative human role | Approved sender, front desk lead, or manager owns the final customer-facing send and sensitive wording; care/incident reviewers own sensitive care or safety facts before they appear in a message. |
| Allowed actions | Read source-backed message/contact evidence, draft customer-safe or staff-internal text, summarize included/omitted facts, validate recipient/channel/status/body refs, request `CustomerMessageApproval`, prepare a blocked send stub, and record audit/outcome evidence after review. |
| Blocked actions / review gates | No autonomous customer/member sends, message queueing, suppression, consent/contact-policy changes, sensitive health/behavior/payment/incident disclosure, provider/PMS write, or alteration of message history without an approved send path and approval evidence. See the safety-family [draft/message boundary](entity-atlas-review-safety-boundaries.md#draft-message). |
| Safe-use evidence / outcome fields | Approval record, reviewer role/id, channel permission/contact policy, included facts, omitted facts and omission reason, draft body ref, message status, send stub or blocked reason, audit event, source/provider refs, follow-up/review disposition, suppression/correction reason, and actual minutes or outcome record when measured. |
| Examples / non-examples | Example: daily update creates a `CustomerMessageDraft` with included/omitted care facts and a blocked `SendStub` until approval. Example: CRM retention drafts grooming rebooking outreach only when contact permission and suppression checks are present. Non-example: generated chatbot prose pasted into a body without a `BodyRef`, channel, status, approval gate, or audit trail. Non-example: a queued/sent status inferred because text looks ready. |

### Vaccine record and vaccine policy

| Field | Writer value |
| --- | --- |
| Plain-English definition | The proof and policy state that says whether required vaccines are suggested, pending review, current, expired, rejected, exception-requested, exception-approved, or superseded before boarding/daycare/use in customer messaging. |
| Labor or safety problem reduced | Prevents staff from rereading documents and portal notes for every booking by routing missing, expired, OCR-only, unverified, or conflicting proof to document/medical review. |
| Workflows where it appears | Booking Triage, Vaccine Document, Daily Care Update, Data Quality Hygiene, Daycare Eligibility. |
| Related entities / adjacency | Pet is the subject; document/uploaded evidence and source refs prove the record; location/service policy sets requirements; reservation/daycare eligibility consumes status. |
| Source paths | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/vaccine.rs`](../../domain/src/vaccine.rs), [`domain/src/policy.rs`](../../domain/src/policy.rs), [`docs/workflows/vaccine-document-agent.md`](../workflows/vaccine-document-agent.md). |
| Rustdoc/module/type contracts | `domain::entities::VaccineRecord`; `domain::vaccine::Status`; `domain::policy::VaccineRequirement`. |
| Source of record | Reviewed vaccine document/source evidence plus policy requirement; trained reviewer/medical document role for acceptance or exceptions. |
| Authoritative human role | Medical/document reviewer, trained staff, or manager depending on local policy and exception type. |
| Allowed actions | Detect missing/expired/pending/unverified proof, draft document requests, summarize status for staff, route to review, record review outcome. |
| Blocked actions / review gates | No autonomous vaccine clearance, medical approval, exception approval, group-play clearance, booking confirmation, provider write, or customer-facing medical conclusion. |
| Safe-use evidence / outcome fields | Vaccine type, expiration date, document/source ref, review status, reviewer role, policy version, required service/date, exception reason, outcome disposition. |
| Examples / non-examples | Example: “vaccine_pending because rabies proof is OCR-only and needs review.” Example: “current after reviewer-approved document evidence.” Non-example: customer text claim treated as proof. Non-example: expired proof silently accepted because the stay is short. |

### Temperament and group-play eligibility

| Field | Writer value |
| --- | --- |
| Plain-English definition | The behavior and group-play evidence used to decide whether a pet can join daycare play groups, needs staff review, should stay in individual care, or is blocked by safety restrictions. |
| Labor or safety problem reduced | Reduces repeated behavior-note interpretation while keeping group-play, incident, and sensitive owner-language decisions review-gated. |
| Workflows where it appears | Daycare Eligibility, Booking Triage, Daily Care Update, Incident Escalation, Data Quality Hygiene. |
| Related entities / adjacency | Pet profile owns temperament; daycare eligibility reads freshness and observations; incident restrictions can suspend or block group play; customer messages about behavior require review. |
| Source paths | [`domain/src/temperament.rs`](../../domain/src/temperament.rs), [`domain/src/daycare/eligibility.rs`](../../domain/src/daycare/eligibility.rs), [`domain/src/daycare/assignment.rs`](../../domain/src/daycare/assignment.rs), [`domain/src/policy.rs`](../../domain/src/policy.rs), [`docs/workflows/staff-operations-parts/playgroups-compatibility.md`](../workflows/staff-operations-parts/playgroups-compatibility.md). |
| Rustdoc/module/type contracts | `domain::temperament::{StaffNote, BehaviorObservationLabel, GroupPlayObservation, PeopleOrientation, Rating, BehaviorObservation}`; `domain::daycare::eligibility::{Evidence, GroupPlayPolicy, GroupPlayDecision, ReviewReason, DenialReason}`. |
| Source of record | Staff/provider behavior observations with provenance and freshness; daycare/behavior reviewer for final eligibility or exceptions. |
| Authoritative human role | Daycare lead, behavior reviewer, or manager. |
| Allowed actions | Summarize current evidence, flag stale/missing behavior facts, recommend review lane, draft internal task or manager packet, cite incident restrictions. |
| Blocked actions / review gates | No autonomous group-play approval, override, reinstatement after incident, restriction clearance, behavior diagnosis, customer-sensitive behavior message, or playgroup assignment. |
| Safe-use evidence / outcome fields | Observation source refs, assessment freshness, vaccine readiness, spay/neuter status when policy uses it, incident restriction, group-play decision, review gate, reviewer disposition. |
| Examples / non-examples | Example: “staff_review because temperament assessment is stale.” Example: “temporarily_suspended because active daycare incident restriction exists.” Non-example: a cheerful daily note treated as eligibility proof. Non-example: “friendly” free text used to override bite history. |

### Incident

| Field | Writer value |
| --- | --- |
| Plain-English definition | A safety, care, behavior, facility, customer-service, or staff-safety event that needs classification, severity, evidence, follow-up, possible owner-message review, and closure/restriction boundaries. |
| Labor or safety problem reduced | Turns scattered staff reports into a manager/lead review packet without letting AI minimize, close, downgrade, or hide safety/owner-notice issues. |
| Workflows where it appears | [Incident Escalation](../workflows/incident-escalation-agent.md), Daily Care Update, Daycare Eligibility, Manager Daily Brief, Data Quality Hygiene, Checkout Completion. |
| Related entities / adjacency | Pet/customer/reservation/location/staff are subjects and actors; care, temperament, vaccine, media/document, audit, message, and task records provide evidence and follow-up. |
| Source paths | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/incident.rs`](../../domain/src/incident.rs), [`domain/src/daycare/incident.rs`](../../domain/src/daycare/incident.rs), [`docs/workflows/incident-escalation-agent.md`](../workflows/incident-escalation-agent.md). |
| Rustdoc/module/type contracts | `domain::entities::Incident`; `domain::incident::{Category, Severity, Status, Summary}`; `domain::daycare::incident::{Policy, Restriction, Disposition, Classifier}`. |
| Source of record | Staff/provider incident evidence, media/document refs, audit trail, and manager/lead decision records. |
| Authoritative human role | Lead staff/supervisor for intake completeness; manager/admin for serious classification, owner notice, restriction clearance, and closure. |
| Allowed actions | Summarize source-backed facts, build timeline, identify missing fields, propose provisional severity/type, draft manager/owner-message packets for approval, recommend follow-up tasks. |
| Blocked actions / review gates | No autonomous incident closure, serious severity finalization/downgrade, owner send, provider write, restriction clearance, group-play reinstatement, medical/legal/payment decision, blame/liability statement, or task completion. |
| Safe-use evidence / outcome fields | Incident id, observed/submitted timestamps, source refs, subject refs, severity candidate, manager review gate, owner-notice state, active restrictions, follow-up tasks, closure blockers, outcome disposition. |
| Examples / non-examples | Example: daycare incident classifier creates a suspension pending manager review. Example: owner-message draft is stored as review-gated. Non-example: “resolved” generated from a positive later note. Non-example: hiding an injury-adjacent fact from the manager packet. |

### Care profile, care notes, and medication review

| Field | Writer value |
| --- | --- |
| Plain-English definition | Feeding, allergy, medical-condition, medication, emergency/vet contact, and care-note facts that staff use for safe check-in, daily updates, checkout handoff, and incident/care review. |
| Labor or safety problem reduced | Prevents staff from rewriting or interpreting free-text care notes for every stay by naming missing/ambiguous medication, feeding, allergy, or medical review gates. |
| Workflows where it appears | Booking Triage, Boarding Care Readiness, Daily Care Update, Checkout Completion, Incident Escalation, Manager Daily Brief. |
| Related entities / adjacency | Pet owns longitudinal care profile; reservation/stay scopes day-specific notes and tasks; boarding/daycare policies consume care readiness; customer messages only use reviewed customer-safe facts. |
| Source paths | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/care.rs`](../../domain/src/care.rs), [`domain/src/boarding/care.rs`](../../domain/src/boarding/care.rs), [`app/src/daily_update.rs`](../../app/src/daily_update.rs), [`app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs). |
| Rustdoc/module/type contracts | `domain::entities::{CareProfile, MedicationInstruction, CareNote}`; `domain::care::{FeedingInstruction, AllergyName, MedicalConditionName, MedicationReviewRequirement}`; `domain::boarding::care::{Policy, Plan, Readiness, ReviewGate, GateReason}`. |
| Source of record | Reviewed care profile and staff/source notes with provenance; trained care staff or manager for medical/medication ambiguity and special-care acceptance. |
| Authoritative human role | Care team/trained staff for routine instructions; manager or medical/document reviewer for exceptions, ambiguity, and customer-facing sensitive wording. |
| Allowed actions | Summarize routine source-backed care facts, flag missing/ambiguous instructions, route medication review, draft customer-safe daily-update copy for approval, record omitted/suppressed facts. |
| Blocked actions / review gates | No autonomous medical judgment, medication verification, care-task completion, special-care acceptance, customer health claims, provider write, or incident/feeding/medication closure. |
| Safe-use evidence / outcome fields | Care profile version, feeding instruction, medication name/dose/schedule/source, allergy/medical-condition refs, review reason, omitted facts, included facts, reviewer disposition, task/audit refs. |
| Examples / non-examples | Example: boarding care plan blocks check-in when feeding instruction is missing. Example: daily update omits internal-only medication uncertainty and creates review flag. Non-example: AI says a pet “is fine” after vomiting. Non-example: free-text “give meds” treated as verified schedule. |

### Location and service-policy context

| Field | Writer value |
| --- | --- |
| Plain-English definition | The resort/site boundary that scopes time zone, service offerings, policy versions, capacity, staff coverage, and manager reporting. |
| Labor or safety problem reduced | Keeps multi-site recommendations from mixing policies, time zones, staffing, capacity, or service availability. |
| Workflows where it appears | All workflows, especially Booking Triage, Manager Daily Brief, Regional Exceptions, Data Quality Hygiene. |
| Related entities / adjacency | Location owns policy refs and service contracts; reservations, staff, source systems, outcome records, and storage projections are scoped by location. |
| Source paths | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/location.rs`](../../domain/src/location.rs), [`domain/src/operations.rs`](../../domain/src/operations.rs), [`storage/src/operations.rs`](../../storage/src/operations.rs). |
| Rustdoc/module/type contracts | `domain::entities::{LocationId, Location, Brand, LocationPolicyRefs}`; `domain::location::{Name, Timezone}`; `domain::operations::{ServiceOffering, service_core::ServiceContracts}`. |
| Source of record | Domain location/policy/service-contract records and source-backed operating context; manager/regional ops for policy interpretation and exceptions. |
| Authoritative human role | General manager for site policy; regional ops for multi-site reporting interpretation. |
| Allowed actions | Scope packets by location, validate policy/source freshness, route service-line contracts, aggregate reviewed outcomes. |
| Blocked actions / review gates | No autonomous local policy creation, hours/capacity/staffing changes, exception approvals, or cross-site policy assumptions. |
| Safe-use evidence / outcome fields | Location id, timezone, policy refs/version, service offering, core service contracts, source system refs, reporting group, outcome minutes. |
| Examples / non-examples | Example: booking triage returns unknown when location policy version is missing. Example: manager brief groups actions by location and reporting group. Non-example: assuming one site’s ratio/deposit policy applies to another. |

### Staff task and actor refs

| Field | Writer value |
| --- | --- |
| Plain-English definition | The staff identity, manager identity, role, task, assignment, priority, status, and completion evidence that prove who reviews or performs work. |
| Labor or safety problem reduced | Converts recommendations into assignable, reviewable work without treating AI output as staff completion or approval. |
| Workflows where it appears | All review queues: Booking Triage, Daily Care Update, Incident Escalation, Manager Daily Brief, Data Quality Hygiene, staff operations. |
| Related entities / adjacency | Staff tasks are linked to reservation, pet, customer, location, incident, care, document, message, workflow event, and outcome records. |
| Source paths | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/staff.rs`](../../domain/src/staff.rs), [`domain/src/workflow.rs`](../../domain/src/workflow.rs), [`docs/workflows/staff-operations.md`](../workflows/staff-operations.md). |
| Rustdoc/module/type contracts | `domain::entities::{StaffId, ManagerId, ActorRef}`; `domain::staff::{Task, Role, task::{Kind, Status, Priority}, completion_evidence::Evidence}`; `domain::workflow::{Event, Subject, AllowedAction}`. |
| Source of record | Staff/task system or workflow/audit evidence; human reviewer for approval/completion. |
| Authoritative human role | Assigned staff, lead, manager, or approved reviewer named by the workflow. |
| Allowed actions | Draft internal tasks, route review packets, record reviewer role/disposition, preserve completion evidence when supplied. |
| Blocked actions / review gates | No autonomous personnel decisions, schedule changes, task completion, manager approval, incident closure, or staff-performance conclusions. |
| Safe-use evidence / outcome fields | Task id/kind/status/priority, assignee/role, due time, source workflow event, completion evidence, reviewer disposition, actual minutes. |
| Examples / non-examples | Example: create_internal_task recommendation for missing vaccine proof. Example: manager approval record before owner incident message. Non-example: AI draft marked completed. Non-example: labor minutes used to alter staff schedule. |

## 6. Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Core entities source | [`domain/src/entities.rs`](../../domain/src/entities.rs) | ids, relationships, care/vaccine/incident/message/audit/approval record shapes. |
| Reservation source | [`domain/src/reservation/mod.rs`](../../domain/src/reservation/mod.rs) | reservation support facts, age/add-on/transition reason semantics. |
| Boarding source and README | [`domain/src/boarding/mod.rs`](../../domain/src/boarding/mod.rs), [`domain/src/boarding/README.md`](../../domain/src/boarding/README.md) | capacity, deposit, care, cancellation, handoff, housekeeping, minimum-stay, and upsell boundaries. |
| Daycare source and README | [`domain/src/daycare/mod.rs`](../../domain/src/daycare/mod.rs), [`domain/src/daycare/README.md`](../../domain/src/daycare/README.md) | attendance, coverage, group-play eligibility, assignment, incident, front-desk, and package-opportunity boundaries. |
| Pet/customer/care/safety source | [`domain/src/pet.rs`](../../domain/src/pet.rs), [`domain/src/customer.rs`](../../domain/src/customer.rs), [`domain/src/care.rs`](../../domain/src/care.rs), [`domain/src/vaccine.rs`](../../domain/src/vaccine.rs), [`domain/src/temperament.rs`](../../domain/src/temperament.rs), [`domain/src/incident.rs`](../../domain/src/incident.rs) | identity and sensitive safety/care facts before app workflows use them. |
| Message source and app messaging workflows | [`domain/src/entities.rs`](../../domain/src/entities.rs), [`domain/src/message.rs`](../../domain/src/message.rs), [`app/src/daily_update.rs`](../../app/src/daily_update.rs), [`app/src/crm_retention.rs`](../../app/src/crm_retention.rs), [`app/src/tools.rs`](../../app/src/tools.rs) | customer/staff/internal message state, channel/status/body refs, draft/send-stub evidence, approval gates, included/omitted facts, and app messaging tool boundaries. |
| Location/staff source | [`domain/src/location.rs`](../../domain/src/location.rs), [`domain/src/staff.rs`](../../domain/src/staff.rs) | policy/location scoping and human-review task ownership. |
| Storage service-line projections | [`storage/src/service_line/boarding.rs`](../../storage/src/service_line/boarding.rs), [`storage/src/service_line/daycare.rs`](../../storage/src/service_line/daycare.rs), [`storage/src/operations.rs`](../../storage/src/operations.rs) | storage codes, contract records, shape checks, and outcome/reporting projections. |
| Workflow docs | [Booking Triage](../workflows/booking-triage-agent.md), [Daily Care Update](../workflows/daily-care-update-agent.md), [Incident Escalation](../workflows/incident-escalation-agent.md) | app-side allowed actions, blocked actions, review gates, safe evidence, and outcome fields. |
| Rustdoc/module paths | `domain::entities::*`; `domain::entities::Message`; `domain::message::{Direction, Channel, Status, BodyRef}`; `app::tools::messaging::*`; `domain::reservation::*`; `domain::boarding::*`; `domain::daycare::*`; `domain::{pet,customer,vaccine,temperament,incident,care,location,staff}`; `storage::service_line::{boarding,daycare}` | exact compiled contract once Rustdoc is generated or published; do not invent rendered URLs. |

## 7. Authoritative source system or human role

| Fact or decision | Source of record | Human role when incomplete, sensitive, or exception-based |
| --- | --- | --- |
| Provider reservation/status/source facts | Promoted provider evidence with source refs/provenance | Front desk lead verifies conflicts or execution state. |
| Location policy and service contract | Domain policy/contract record plus approved local policy source | General manager or regional ops. |
| Boarding room/capacity readiness | Capacity snapshot/read model plus `domain::boarding::capacity` policy | Front desk/boarding lead; manager for over-capacity or waitlist exception. |
| Daycare group-play eligibility | Pet/care/vaccine/temperament/incident/staffing evidence plus `domain::daycare::eligibility` policy | Daycare lead, behavior reviewer, or manager. |
| Care/medication/feeding facts | Care profile, source notes, documents, and review status | Trained care staff, medical/document reviewer, or manager. |
| Vaccine readiness | Vaccine document/source evidence and policy requirement | Medical/document reviewer or trained staff. |
| Incident severity/closure/restriction | Incident source evidence, audit trail, manager decision records | Lead/manager/admin; AI can only propose and route. |
| Customer-message approval | Message draft, policy, consent/contact evidence, approval record | Front desk, manager, or approved sender. |
| Staff task completion | Staff/task system or audit/completion evidence | Assigned staff/lead/manager. |
| Outcome/labor savings | Submitted outcome record with reviewer disposition and actual minutes | Reviewer who performed or approved the work. |

## 8. Allowed actions

Workflows and documentation may safely say these entities support:

- reading source-backed reservation, customer, pet, care, vaccine, temperament, incident, location, staff, and service-line evidence;
- mapping provider/source records into explicit candidates with provenance and freshness markers;
- validating deterministic capacity, eligibility, care, vaccine, incident, and readiness gates;
- drafting internal tasks, manager packets, missing-info requests, and customer-safe text for review;
- ranking booking, daycare, boarding, incident, care-watch, source-hygiene, and manager action queues;
- recording review dispositions, audit events, outcome records, actual minutes, suppression/correction reasons, and follow-up status.

## 9. Blocked actions and review gates

Unless a linked app/source contract and human approval record explicitly allow an execution path, this family blocks:

- autonomous customer/member sends, owner incident notices, daily updates, retention or package outreach;
- PMS/provider writes, source-data deletion/hiding, profile overwrites, booking lifecycle mutations;
- booking confirmation/rejection/cancellation, check-in/checkout, waitlist promotion, room/run/group/playgroup assignment, capacity hold/release;
- payment, refund, discount, fee, deposit, package/session-balance, or billing movement;
- vaccine/medical/document acceptance, medication/care judgment, special-care acceptance;
- behavior exceptions, group-play override/reinstatement, incident restriction clearance, serious incident final classification or closure;
- staff/personnel/schedule changes or task completion;
- local policy invention, cross-location policy assumptions, or secret-dependent/live external side effects.

## 10. Safe-use evidence and outcome fields

Use a recommendation only when the packet names the evidence that made it safe to show and the outcome fields that will prove or correct the labor claim:

| Evidence or outcome field | Why it matters |
| --- | --- |
| Source refs and provenance | Shows which provider/staff/document/source record caused the recommendation. |
| Freshness / observed-at / policy version | Prevents stale capacity, vaccine, staff ratio, or behavior facts from becoming promises. |
| Subject refs | Links the fact to the correct pet, customer, reservation, location, staff task, incident, or document. |
| Review gate and reason | Explains exactly why the workflow stops before a sensitive action. |
| Approval record / reviewer role | Proves who cleared customer, provider, payment, policy, safety, or care-sensitive steps. |
| Audit event | Keeps source fact, workflow decision, staff action, and outcome replayable. |
| Outcome disposition | Records approved, rejected, suppressed, corrected, escalated, no-op, or failed-safe results. |
| Actual minutes saved/wasted | Turns “could save time” into a measurable labor loop. |
| Follow-up status / correction reason | Prevents unresolved care, source, incident, or customer tasks from disappearing. |

## 11. Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | Reservation triage says `vaccine_pending` with policy ref and document source ref. | Staff see the next safe queue without AI approving the vaccine or confirming the booking. |
| Example | Boarding capacity policy returns waitlist with manager-review reason. | It reduces manual room-count reconciliation while keeping over-capacity decisions human-reviewed. |
| Example | Daycare front-desk ticket routes to care-team review because medication readiness is unresolved. | It saves check-in time and avoids unsafe admission. |
| Example | Incident escalation packet proposes severity candidate and owner-message draft. | It prepares manager review without finalizing or sending anything. |
| Non-example | Raw provider owner/animal/reservation id wrappers treated as customer/pet/reservation truth. | Provider ids are evidence keys, not canonical business facts by themselves. |
| Non-example | Storage `FormatCode` or `AccommodationCode` used as policy authority. | Storage codes preserve projections; domain contracts own semantics. |
| Non-example | “AI confirmed the room/playgroup.” | A recommendation candidate is not a booking, room, or group assignment. |
| Non-example | Labor minutes used to alter staff schedules. | Outcome minutes support measurement/reporting; they do not authorize personnel actions. |

## 12. Glossary cross-links

Use these glossary entries near the first mention of each term in downstream pages: [domain](../glossary-architecture-terms.md#domain), [app](../glossary-architecture-terms.md#app), [storage](../glossary-architecture-terms.md#storage), [Gingr](../glossary-source-data-terms.md#domainsourcesystemgingr-gingr), [source-of-record](../glossary-source-data-terms.md#source-of-record), [provider record](../glossary-source-data-terms.md#provider-record), [provenance](../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence), [draft](../glossary-workflow-state-terms.md#draft), [review gate](../glossary-workflow-state-terms.md#review-gate), [blocked action](../glossary-workflow-state-terms.md#blocked-action), [workflow packet](../glossary-workflow-state-terms.md#workflow-packet), and [outcome capture](../glossary-workflow-state-terms.md#outcome-capture).

## 13. Open source questions for later pages

These questions should stay explicit rather than being turned into asserted policy:

- Which provider/read-model source will supply current room/suite/condo inventory, daycare headcount, playgroup roster, and staff coverage freshness by location?
- Which local policy source owns holiday minimum stays, deposit/cancellation overlays, vaccine-pending allowances, group-play exception authority, and care-load thresholds?
- Which workflow/output records will capture actual minutes for booking triage, daycare check-in, boarding readiness, care handoff, incident escalation, and source-hygiene cleanup?
- Which approved customer-message/send policy, if any, permits a deterministic no-review path for routine updates or missing-info requests?
- Which storage/reporting projections will distinguish recommendations from executed provider/customer/payment/safety actions?
