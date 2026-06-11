# Booking triage canonical inputs

Status: source-backed input packet for downstream booking triage definition cards. This document is not a final workflow artifact and does not authorize live booking, rejection, special-care acceptance, behavior exceptions, payment actions, customer messages, or provider mutations.

## Source anchors

Repository/code sources:

- `README.md` lines 43-47: initial booking-triage shape: input is reservation request + pet profile + vaccine state + location policy + capacity snapshot; deterministic stage does hard stops/eligibility; AI only explains/options/drafts.
- `domain/src/booking_triage.rs`: current typestate requires pet profile and policy snapshot before `ReadyForPolicyDecision`.
- `domain/src/entities.rs`: canonical Location, Customer, Pet, Reservation, service/status/source/add-on/hard-stop, deposit/payment alias, audit, and actor surfaces.
- `domain/src/policy.rs`: vaccine requirement, group-play eligibility, review gates, automation levels, and conservative play-eligibility policy.
- `domain/src/payment/mod.rs`: deposit amount/refundable-until/status/payment-reference model and `requires_collection` semantics.
- `domain/src/operations.rs`: occupancy, arrival/departure, labor, care watchlist, operations risk/action, staff task surfaces.
- `docs/integrations/gingr/sdk-endpoint-catalog.md`: read-only Gingr endpoint catalog and operational source boundaries.
- `docs/workflows/payments-pricing.md`: canonical pricing/deposit/payment policy and AI/payment boundaries.
- `docs/workflows/staff-operations-parts/inputs.md`: staff-operation input collection and readiness lanes.
- Boarding/daycare implication docs cited below for capacity, holiday, check-in/out, care, group assignment, ratios, throughput, and incidents.
- Prior canonical data-model input handoff `t_32dc302c` supplied product scope, MVP scope, vaccination policy assumptions, booking/capacity assumptions, and global approval gates.

Missing or provisional source gaps:

- `docs/product/pet-resort-product-map.md` is still absent in this repo; use `t_32dc302c` metadata and implication docs until copied in.
- Exact live location values are not present: room/run counts, group/yard limits, suite/condo labels, service menu/prices, deposit amounts, holiday dates, blackout windows, minimum stays, cancellation windows, staffing ratios by location, special-care surcharges, late pickup/checkout rules, and manager override policies.
- Gingr response schemas for many endpoints are not fully documented in local docs; raw provider payloads must be quarantined and mapped at adapter boundaries before policy decisions.

## Canonical inputs for booking triage

### 1. Reservation/request input

Required semantic facts:

- Request identity/source: source channel/provider reference (`Portal`, `WebsiteForm`, `PhoneTranscript`, `Sms`, `Email`, `StaffCreated`) and request timestamp/evidence.
- Location: `LocationId`, timezone, capabilities, and `LocationPolicyRefs` for vaccine, deposit, playgroup, and future local policies.
- Customer: `CustomerId`, name, contact channels, preferred contact, portal/account ref, pickup/contact authorization if available.
- Pet set: one or more `PetId`s; per-pet evaluation is required for multi-pet stays.
- Service: `ServiceKind::{Boarding, DayPlay, DayBoarding, Grooming, Training, DaySpa}` plus requested variant/accommodation/care lane where known.
- Dates/windows: start/end timestamps; for boarding, local check-in/check-out dates and positive nights; for daycare/grooming/training, requested operating-day/time window.
- Status/lifecycle context: current `ReservationStatus` if modifying an existing reservation (`Inquiry`, `Requested`, `MissingInfo`, `VaccinePending`, `SpecialReview`, `Waitlisted`, `Offered`, `Confirmed`, `CheckedIn`, `Active`, `CheckedOut`, `Cancelled`, `Rejected`).
- Requested add-ons: group play, individual play, webcam suite, exit bath, Pawgress report, medication administration, grooming/training/DaySpa/other approved add-ons.
- Existing hard stops: missing vaccine, group-play ineligibility, in heat, below minimum age, medical/medication review, behavior review, deposit required.

Primary sources: `domain/src/entities.rs`; `docs/domain/petsuites/boarding/implications/01-capacity-management.md` lines 43-56; `docs/domain/petsuites/boarding/implications/04-check-in-check-out-flows.md` lines 51-63; `docs/domain/petsuites/daycare/implications/08-fast-front-desk-throughput.md` lines 27-39.

### 2. Pet/customer profile and safety input

Per pet, triage needs:

- Species, birth date/age, sex, spay/neuter status.
- Care profile: feeding instructions, medications, allergies, medical conditions, emergency contact, veterinarian contact.
- Medication details: name, dose, schedule, review requirement; executable medication tasks require reviewed name/dose/schedule/source.
- Temperament: group-play observation, people orientation, rating, behavior observations, redacted staff notes.
- Behavior and incident evidence: anxiety/stress, bite history, dog/human selectivity, escape risk, food guarding, manager-review flags, suspending/unresolved incidents, incident restrictions.
- Vaccine/document state: required proof from licensed vet, document source, verification status, review status, policy version, expiration where available.
- Special care/handling: mobility, isolation, anxiety, medical/safety, medication/feeding complexity, special room/lane fit, emergency/vet clarification needs.

Primary sources: `domain/src/entities.rs` lines 114-187 and 258-267; `domain/src/temperament.rs`; `domain/src/policy.rs` lines 74-158 and 176-244; `docs/domain/petsuites/boarding/implications/06-medication-feeding-behavior-notes.md` lines 45-83 and 85-107; `docs/domain/petsuites/daycare/implications/04-incident-tracking.md` lines 33-65 and 66-87.

### 3. Capacity, availability, room/group, and staffing input

Triage must treat capacity as typed, location-scoped evidence, not a boolean:

- Boarding capacity: segmented inventory by accommodation kind (dog classic/luxury suites, cat condos or local equivalents), nightly booked counts, held counts, closures/out-of-service/turnover state, waitlist posture, source timestamp/freshness, and provider conflicts.
- Boarding request fit: per-pet species/accommodation compatibility, same-suite/multi-pet compatibility, requested accommodation/flexibility, minimum stay, local timezone, positive stay range.
- Daycare capacity: playgroup/care-lane roster, capacity limits, current assignments, waitlist state, hybrid play+room lane availability, cat individual enrichment lane where relevant.
- Staffing: scheduled staff count, roles/qualifications, shift windows, breaks/absences, lane assignments, staff-to-pet ratio policy and coverage decision.
- Operations context: arrivals/departures, late-departure risk, room turnover/cleaning backlog, unresolved incidents, manager holds, care-load clusters, labor risk.
- Snapshot integrity: policy id/version, evidence/snapshot id, observed-at/source timestamp, freshness/staleness decision, audit refs.

Decision outcomes should be typed: available for staff approval, limited, waitlist, deny, manager review, stale/unknown/provider conflict. Stale or missing evidence routes to staff/manager review, not optimistic availability.

Primary sources: `domain/src/operations.rs` lines 199-321 and 303-419; `docs/domain/petsuites/boarding/implications/01-capacity-management.md` lines 43-89 and 90-117; `docs/domain/petsuites/boarding/implications/02-room-suite-availability.md` lines 33-76 and 78-92; `docs/domain/petsuites/daycare/implications/02-group-assignment.md` lines 29-65 and 66-87; `docs/domain/petsuites/daycare/implications/03-staff-to-pet-ratios.md` lines 27-65.

### 4. Services, pricing, deposit, holiday, and blackout input

Triage needs policy-backed, not inferred, values for:

- Service catalog by location: boarding, daycare/day play, day boarding/individual play, grooming/bathing/DaySpa, training where offered, packages/memberships, add-ons.
- Pricing dimensions: service line, location/provider context, pet count/species, boarding stay dates/nights/accommodation/add-ons, daycare variant/package, grooming breed/coat/menu, training package, retail/package lines, approved surcharges/fees/taxes.
- Deposit decision facts: service/location/customer/pets/dates/reservation id/policy version, amount model, due rule, refundability, reminder/expiration/hold rule, holiday/peak classification, customer history/high-value policy inputs if approved.
- Deposit status: not required/required/paid/refunded/failed/waived by manager, amount, refundable-until, payment reference, requires-collection flag.
- Holiday/peak/local-event policy: named period, demand class, blackout/manager-hold intervals, minimum-stay rules, deposit/cancellation overlays, capacity holds, waitlist posture, escalation urgency.
- Late pickup/checkout/extended checkout: only as location policy/add-on/surcharge candidates until approved values exist.

Payment/deposit state may block automated confirmation/check-in/check-out readiness or create staff tasks, but must not by itself confirm, cancel, reject, hold/release capacity, charge/refund/waive, or mutate live reservations.

Primary sources: `domain/src/payment/mod.rs`; `docs/workflows/payments-pricing.md` lines 29-52, 54-89, 90-147, and 148-169; `docs/domain/petsuites/boarding/implications/03-holiday-demand-spikes.md` lines 43-80 and 92-105; `docs/domain/petsuites/boarding/implications/04-check-in-check-out-flows.md` lines 51-63 and 142-153.

### 5. Provider/source input

Gingr/local-provider reads available in the current SDK/catalog:

- Reservations by date/checked-in/location; by animal; by owner; future/past/waitlisted/confirmed/completed/cancelled filters.
- Reservation types, reservation widget data, back-of-house digital whiteboard data, additional services by reservation type, existing reservation estimates, recently cancelled reservations.
- Owner lookup by owner/animal/reservation/phone/email; owner/animal forms and custom fields; animals; feeding and medication info.
- Reference data: locations, species, breeds, vets, temperaments, immunization types, animal immunizations.
- Commerce/payment-adjacent reads: invoices/estimates, transactions, subscriptions/packages, retail items.
- Labor ops: timeclock report.

Boundary: local docs say Gingr public API is read-only and v0 excludes `quick_checkin` because it checks in pets/creates reservations as a side effect. Provider responses, PII, payment-sensitive data, custom fields, arbitrary SQL-like filters, and raw JSON must be redacted/quarantined before entering policy decisions.

Primary sources: `docs/integrations/gingr/sdk-endpoint-catalog.md` lines 5-12, 14-29, 30-66, and 68-75; `integrations/gingr/src/endpoint/reservations.rs`; `integrations/gingr/src/endpoint/owners_animals.rs`.

## Deterministic triage constraints

Hard stops / review routes:

- Missing required vaccine or unverified vaccine/document proof -> document/medical review; never final approval by OCR/AI alone.
- Below minimum age, pet in heat, male cat not neutered, and group-play spay/neuter concerns -> review/alternative-care path per local policy.
- Non-dog species are not dog group-play eligible; cats use individual enrichment/condo semantics.
- Intact or unknown spay/neuter status, bite history, human selectivity, manager-review flag, stressed/needs-intro group observation, suspending incident, or stale/missing temperament evidence -> behavior/staff/manager review before group play.
- Medication/feeding/allergy/medical ambiguity -> care-team/manager/vet clarification; no executable medication tasks from vague notes.
- Capacity unavailable, stale, conflicting, full, held, closed, over-capacity, holiday/blackout, unsupported accommodation, special-care room fit, multi-pet split, insufficient staff/ratio -> waitlist, deny, or manager/staff review; no availability promise.
- Deposit/payment missing, failed, late, disputed, partial, ambiguous, waiver/refund/forfeit/discount/credit exception -> payment/manager review; no autonomous money movement or status mutation.
- Incident/health/safety/customer complaint/sensitive behavior -> manager/safety/customer-message approval before owner-facing language.

Recommended readiness buckets for downstream cards:

- `ready_for_staff_approval`: deterministic checks pass, but staff approval/tool boundary still required for confirmation or provider update.
- `missing_info`: collectable front-desk info or documents without policy judgment.
- `vaccine_pending`: vaccine/document proof needs review.
- `special_review`: care/medical/behavior/incident/special-care/staffing/capacity/payment/holiday exception requires human owner.
- `waitlisted`: capacity/staffing/holiday posture blocks immediate offer but waitlist path exists.
- `offered`: staff-approved offer exists, usually pending deposit/confirmation.
- `confirmed`: provider/ledger/customer-visible booking state only after approved execution path; AI cannot independently produce this state.
- `rejected`: human/manager-approved or deterministic hard-denial path only; AI can recommend but not reject autonomously.

## AI recommendation role and approval boundaries to preserve

Allowed for AI/agent workflow:

- Read typed snapshots, detect missing/stale/conflicting inputs, summarize evidence, explain deterministic rule outcomes, suggest status/readiness buckets, draft internal tasks, draft customer-safe scripts, draft manager packets, and identify source gaps.

Never automatic without explicit approved path:

- Confirm booking or update live provider reservation status.
- Reject a customer booking/request.
- Accept special-care/medical/medication/behavior exceptions.
- Override behavior/group-play restrictions or reinstate/suspend after incidents.
- Promise availability, assign rooms/runs/groups, release holds, overbook, or promote from waitlist.
- Charge, refund, waive, discount, forfeit, or alter payment/deposit policy.
- Send customer-facing messages involving medical, incident, behavior, safety, payment, policy exceptions, rejection, or availability promises.

Human approval gates explicitly required by the parent workstream: confirmed booking automation, rejection, special-care acceptance, behavior exceptions.

## Minimum downstream output input object

Downstream cards should model booking triage around an input packet with at least:

- `request`: reservation/request id/source/status/location/service/dates/add-ons.
- `customer`: customer id/contact/portal refs and identity confidence.
- `pets[]`: species/age/sex/spay-neuter/care/medication/feeding/allergy/medical/temperament/incident/vaccine evidence per pet.
- `policy_snapshot`: location policy refs/versions for vaccine, playgroup, deposit/payment, capacity, holiday/peak, check-in/out/late rules, staff coverage.
- `availability_snapshot`: segmented room/group/care-lane/roster/staff snapshot with freshness and evidence ids.
- `payment_snapshot`: deposit requirement/status, quote/estimate/payment request refs, package/membership/balance where applicable.
- `provider_snapshot`: trusted mapped provider records plus raw source quarantine refs, never unredacted raw JSON in policy output.
- `audit_context`: actor, workflow event, source timestamps, evidence refs, previous approvals/review gates.
- `source_gaps[]`: missing/stale/conflicting facts that block automation or require review.
