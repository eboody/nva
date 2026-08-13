# Manager Daily Brief measurable labor loop contract

Purpose: define the first measurable labor-cost loop before building runtime automation. The loop turns reviewed source facts into manager/front-desk actions, keeps AI draft/review-only, captures staff feedback, and measures before/after labor minutes. The broader driver map and next-loop sequence live in [labor-cost-reduction-crosswalk.md](labor-cost-reduction-crosswalk.md).

## Repetitive work removed

The first brief removes repeated morning/manual checks that are already represented in typed contracts:

1. Demand-versus-staffing scan: manager compares reservation/service-demand dashboards to the schedule.
2. Checkout exception audit: front-desk lead scans open stays and handoffs to find unresolved checkout/completion issues.
3. Retention follow-up queue prioritization: front desk scans completed stays for safe follow-up opportunities.
4. Data-quality exception triage: manager keeps nonblocking source ambiguity visible instead of rediscovering it downstream.

## Affected personas

- General manager: owns demand/staffing review and data-quality visibility.
- Assistant general manager: can receive the same operating-day brief and manager approval gates.
- Front-desk lead: owns checkout exception and retention follow-up review queues.
- Front-desk agent: may execute approved internal tasks, but the contract does not authorize customer sends or source mutations.

## Required source facts

Every brief action must carry source evidence. The executable contract is `app::manager_daily_brief`:

- `analytics::service_demand::Fact` with `operations::operating_day::Key`, demand units, projection version, source record refs, and data-quality issues.
- `checkout_completion::Packet` from the checkout/completion contract, including source provenance and review gates.
- `crm_retention::Packet` from the retention contract, including staff evidence and draft-only follow-up eligibility.

Actions are valid only when their `SourceFact` entries have non-empty `source::RecordRef` evidence. Source data-quality issues are preserved as `SourceFactKind::SourceDataQualityIssue` and add manager review rather than being hidden.

## Brief/action schema

Each `BriefAction` names:

- action id;
- action kind;
- priority;
- owner persona;
- removed manual work;
- rationale;
- source facts;
- review gates;
- labor impact estimate with before minutes, after minutes, and minutes saved.

The current action kinds are:

- `ReviewDemandAgainstStaffingPlan`
- `ResolveCheckoutException`
- `ApproveRetentionFollowUpDraft`
- `InvestigateSourceDataQualityIssue`

## Review boundaries

Allowed AI actions are internal/draft-only:

- summarize source evidence;
- rank manager actions;
- draft internal tasks for review;
- record manager feedback;
- estimate labor minutes saved.

Blocked actions remain explicit no-go areas:

- change staff schedule;
- mutate provider/PMS record;
- send customer message;
- move refunds, discounts, or payments;
- hide source data-quality issues.

Retention actions preserve `CustomerMessageApproval`. Checkout/data-quality exceptions preserve `ManagerApproval`. The brief may recommend and prioritize; it does not execute live schedule, PMS, customer-message, payment, refund, or discount changes.

## Feedback/outcome capture

`OutcomeRecord` captures manager/staff feedback per action:

- action id;
- actor;
- outcome (`Completed`, `Deferred`, `SuppressedByManager`, `SourceFactWasWrong`);
- before minutes;
- actual minutes;
- optional manager feedback explaining the human/system-of-record disposition;
- source record refs.

Outcome capture is staff evidence only and returns the same blocked external actions. It records whether the loop actually reduced work without mutating provider systems. `Completed` outcomes can produce a supported `LaborSavingsClaim` only through the action-aware claim path that verifies the outcome matches the reviewable `BriefAction` and cites all source records behind that action's `SourceFact` evidence. Completed raw outcomes with no action/source proof, deferred outcomes, suppressed outcomes, and wrong-source outcomes stay auditable feedback but intentionally do not count as realized labor savings.

## Before/after labor metric

The first metric is minutes of manager/front-desk work avoided per operating day:

`minutes_saved = before_minutes - after_minutes`

Initial executable contract estimates:

- demand-versus-staffing scan: 45 min before, 15 min after;
- checkout exception audit: 20 min before, 8 min after;
- retention follow-up prioritization: 30 min before, 10 min after.

The packet also totals before/after minutes across ranked actions. Tests prove a source-grounded demand + retention brief produces 75 minutes before, 25 minutes after, and 50 minutes saved.

## Verification

Executable coverage lives in `app/tests/manager_daily_brief_workflow_contracts.rs` and proves:

- actions are source-grounded and persona-owned;
- removed manual work is explicit;
- review gates and blocked actions are preserved;
- nonblocking data-quality issues remain visible;
- outcome capture records actual minutes saved without external mutation.
