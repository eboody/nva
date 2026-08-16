# Manager Daily Brief measurable labor loop contract

Purpose: define a reviewable labor-cost hypothesis before building runtime automation. The loop assembles deterministic caller/source evidence into non-executable recommendations, keeps AI draft-only, retains caller-reported feedback and minute labels, and proves no review, manager/front-desk action, completion, labor measurement, savings, or value. The broader driver map and next-loop sequence live in [labor-cost-reduction-crosswalk.md](labor-cost-reduction-crosswalk.md).

## Repetitive work removed

The first brief removes repeated morning/manual checks that are already represented in typed contracts:

1. Demand-versus-staffing scan: manager compares reservation/service-demand dashboards to the schedule.
2. Checkout exception audit: front-desk lead scans open stays and handoffs to find unresolved checkout/completion issues.
3. Reported retention evidence inspection: current serialized packets remain ineligible and cannot enter a retention queue, task, or draft path.
4. Data-quality exception triage: manager keeps nonblocking source ambiguity visible instead of rediscovering it downstream.

## Affected personas

- General manager: owns demand/staffing review and data-quality visibility.
- Assistant general manager: can receive the same operating-day brief and manager approval gates.
- Front-desk lead: owns checkout exception review and may inspect reported retention evidence; current retention packets create no review queue.
- Front-desk agent: may execute approved internal tasks, but the contract does not authorize customer sends or source mutations.

## Required source facts

Every brief action must carry source evidence. The executable contract is `app::manager_daily_brief`:

- `analytics::service_demand::Fact` with `operations::operating_day::Key`, demand units, projection version, source record refs, and data-quality issues.
- `checkout_completion::Packet` from the checkout/completion contract, including source provenance and review gates.
- `crm_retention::Packet` from the retention contract, containing ineligible reported evidence that cannot establish eligibility, queue work, a task, or a customer draft.

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
- labor impact estimate with before minutes, after minutes, and reported time difference.

The current workflow-emitted action kinds are:

- `ReviewDemandAgainstStaffingPlan`
- `ResolveCheckoutException`
- `InvestigateSourceDataQualityIssue`
- `ReviewCapacityLaborRecommendation`

`ApproveRetentionFollowUpDraft` remains a legacy serialized label for compatibility, but the current workflow never emits it and callers cannot promote it into queue, task, draft, completion, or value authority.

## Review boundaries

Allowed AI actions are internal/draft-only:

- summarize source evidence;
- rank manager actions;
- draft internal tasks for review;
- record manager feedback;
- estimate labor reported time difference.

Blocked actions remain explicit no-go areas:

- change staff schedule;
- mutate provider/PMS record;
- send customer message;
- move refunds, discounts, or payments;
- hide source data-quality issues.

Current retention packets produce no action. Checkout/data-quality exceptions preserve `ManagerApproval`. The brief may recommend and prioritize supported actions; it does not execute live schedule, PMS, customer-message, payment, refund, or discount changes.

## Feedback/outcome capture

`OutcomeRecord` captures manager/staff feedback per action:

- action id;
- actor;
- outcome (`Completed`, `Deferred`, `SuppressedByManager`, `SourceFactWasWrong`);
- before minutes;
- actual minutes;
- optional manager feedback explaining the human/system-of-record disposition;
- source record refs.

Outcome capture is reported staff evidence only and returns the same blocked external actions. `Completed` is a caller-serializable label, not proof that the loop reduced work. `LaborSavingsClaim` has no supported state, and both raw and action-aware claim paths remain nonclaimable until a future opaque authenticated value-authority issuer exists. Completed, deferred, suppressed, and wrong-source outcomes stay auditable feedback and never count as realized labor savings.

## Before/after labor metric

The first metric is minutes of manager/front-desk work avoided per operating day:

`reported_estimated_minutes_difference = before_minutes - after_minutes`

Initial executable contract estimates:

- demand-versus-staffing scan: 45 min before, 15 min after;
- checkout exception audit: 20 min before, 8 min after;
- retention follow-up prioritization: unavailable in the current executable contract; reported retention packets contribute zero actions and no labor estimate.

The packet totals before/after minutes only across supported ranked actions. Tests prove that adding a reported retention packet to a source-grounded demand brief does not change its 45 minutes before, 15 minutes after, or 30-minute reported estimate difference, because retention evidence contributes no action.

## Verification

Executable coverage lives in `app/tests/manager_daily_brief_workflow_contracts.rs` and proves:

- actions are source-grounded and persona-owned;
- removed manual work is explicit;
- review gates and blocked actions are preserved;
- nonblocking data-quality issues remain visible;
- outcome capture records reported actual minutes spent as nonclaimable evidence without external mutation.
