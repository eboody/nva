# First Step Plan: Rust Agent Foundation

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Establish a typed Rust foundation that can become the stable contract between a deterministic pet-resort operations app and Hermes-powered workflow agents.

**Architecture:** Rust owns entities, policies, workflow events, tool traits, and validation. Hermes/LLM agents receive typed prompt packets and return structured results that are policy-checked before any write or customer-facing action.

**Tech Stack:** Rust 2024 workspace, `serde`, `uuid`, `chrono`, `async-trait`, `thiserror`, `clap`; future integrations likely include Gingr/portal APIs, Postgres, OCR/document AI, email/SMS, payment provider, and Hermes Kanban/webhooks/cron.

---

## Why this is the right first step

For Tyler's “agents for a 170-location pet resort” ask, the risk is building a pile of prompt scripts without a durable operating model. The right wedge is a typed boundary:

- deterministic Rust policies decide hard stops;
- agents draft, extract, summarize, recommend, and flag uncertainty;
- tool traits hide incumbent systems such as Gingr, payment, SMS/email, file storage, OCR, and Hermes;
- review gates make safety boundaries explicit.

This lets the call tomorrow focus on validating assumptions rather than asking vague product questions.

## Task 1: Validate target company and incumbent systems

**Objective:** Confirm whether NVA/PetSuites/Gingr assumptions are correct.

**Files:**
- Reference: `/home/eran/.hermes/kanban/boards/pet-resort-meta/docs/context/nva-pet-resorts-context.md`
- Modify later: `README.md`

**Questions for Tyler:**
1. Is the operator NVA/PetSuites or a similar 170-location pet-resort group?
2. What system is source of truth today: Gingr, another PMS/kennel platform, internal DB, spreadsheets, or mixed?
3. Is the goal an AI layer over existing ops, replacement software, or prototypes for internal teams?
4. What workflow hurts most today?

**Verification:** Update README “Call prep” with confirmed answers.

## Task 2: Model the domain core

**Objective:** Capture location, customer, pet, reservation, service, payment/deposit, audit, and care-profile abstractions.

**Files:**
- Create/modify: `crates/domain/src/entities.rs`
- Test: compile via `cargo check --workspace`

**Key abstractions:**
- `Location`, `Brand`, `LocationPolicyRefs`
- `Customer`, `PortalAccountRef`, `PortalProvider::Gingr`
- `Pet`, `TemperamentProfile`, `CareProfile`, `MedicationInstruction`
- `Reservation`, `ServiceKind`, `ReservationStatus`, `Deposit`, `HardStop`
- `AuditEvent`, `ActorRef`

**Verification:** Types serialize with `serde` and compile.

## Task 3: Model review/automation policy as traits

**Objective:** Make safety boundaries explicit and testable.

**Files:**
- Create/modify: `crates/domain/src/policy.rs`
- Test: unit tests in same file

**Traits/types:**
- `AutomationLevel`
- `ReviewGate`
- `AutomationRule`
- `PlayEligibilityPolicy`
- `ConservativePlayEligibilityPolicy`

**Verification:** Unit tests prove intact/unknown dogs do not silently enter group play.

## Task 4: Model workflow events and structured results

**Objective:** Define the envelope passed into and out of agents.

**Files:**
- Create/modify: `crates/domain/src/workflow.rs`

**Types:**
- `WorkflowEvent`
- `WorkflowEventType`
- `WorkflowSubject`
- `PolicyContext`
- `AllowedAction`
- `WorkflowResult<T>`
- `RecommendedAction`

**Verification:** Compile and inspect JSON shape once example fixtures exist.

## Task 5: Model tool integrations as Rust traits

**Objective:** Keep external systems replaceable and dry-run/review-gated.

**Files:**
- Create/modify: `crates/domain/src/tools.rs`

**Tool candidates:**
- Gingr portal / incumbent PMS
- Payment provider
- SMS/email
- File storage
- OCR/document AI
- Webcam/camera provider
- Hermes Kanban / cron / webhooks
- Postgres

**Verification:** Traits compile without requiring concrete credentials.

## Task 6: Add a small operator CLI

**Objective:** Make the current model inspectable before real app code exists.

**Files:**
- Create/modify: `apps/cli/src/main.rs`

**Commands:**
- `cargo run -p cli -- agents`
- `cargo run -p cli -- tools`

**Verification:** Both commands print JSON.

## Task 7: Next concrete implementation slice

**Objective:** After Tyler confirms assumptions, build `booking_triage` as the first end-to-end typed workflow.

**Planned flow:**
1. Input: reservation request + pet profile + vaccine state + location policy + capacity snapshot.
2. Deterministic Rust stage: hard stops and eligibility checks.
3. Agent stage: draft explanation and missing-info/internal-task recommendations.
4. Validation stage: reject malformed or policy-violating output.
5. Output: `WorkflowResult<BookingTriageOutput>`.

**Why booking triage:** It touches most of the valuable abstractions: locations, services, pet eligibility, vaccines, deposits, customer messages, policy gates, and incumbent-system integration.
