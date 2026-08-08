# Workflow and contract comprehension QA

Task: `t_f0c21716` — audit workflow and contract pages from the non-coder/entity-first perspective.

Verdict: PASS.

A non-coder/operator can use the sampled workflow and contract pages to identify:

1. the participating entities and source facts,
2. the app/domain/storage/provider contracts involved,
3. what automation may draft, rank, summarize, validate, or record,
4. what humans or approved systems of record still decide, and
5. where source/Rustdoc/test/storage evidence backs or limits the claim.

No blocking broken chains, missing authority statements, or unclear human-review boundaries were found in the sampled workflow set. The pages consistently preserve the entity-first model: source evidence feeds app workflow packets; packets may draft or recommend; review gates and blocked actions keep live resort actions with humans or approved systems; outcomes/storage only prove reviewed work where a concrete record exists.

## Scope reviewed

Representative operator workflow pages:

- `docs/workflows/operator/booking-triage.md`
- `docs/workflows/operator/daily-updates-pawgress-drafts.md`
- `docs/workflows/operator/data-quality-hygiene.md`
- `docs/workflows/operator/manager-daily-brief.md`
- `docs/workflows/operator/grooming-rebooking-retention.md`
- `docs/workflows/operator/regional-labor-exceptions.md`
- `docs/workflows/operator/README.md`

Contract and boundary pages used to verify the chains:

- `docs/design/workflow-to-entity-navigation-map.md`
- `docs/entity-atlas/contract-crosswalk/workflow-packets.md`
- `docs/design/entity-atlas-review-safety-boundaries.md`
- `docs/safety/review-boundaries-matrix.md`
- `docs/design/entity-atlas-workflow-packets-agents.md`
- `docs/design/entity-atlas-outcomes-operations-money.md`
- `docs/design/source-provenance-data-quality-atlas.md`

## Acceptance-criteria findings

| Requirement | Result | Evidence |
| --- | --- | --- |
| Verify representative workflows from a non-coder view. | PASS | Each sampled workflow starts with the resort problem, roles whose time is saved, concrete source facts/entities, and plain-language examples before code citations. |
| Non-coder can identify participating entities. | PASS | The workflow pages include source-data/entity tables and related-entity lists. `workflow-to-entity-navigation-map.md` adds a workflow-to-family matrix for readers who start from a job-to-be-done. |
| Non-coder can identify contracts/rules between entities. | PASS | The sampled workflow pages include contract tables with “What it authorizes” and “What it does not authorize.” `workflow-packets.md` provides a cross-workflow consume/produce/draft/review/outcome map. |
| Non-coder can see where automation drafts/recommends vs humans decide. | PASS | Each sampled workflow has “Agent may,” “Human must approve,” and/or “Blocked by default” sections. `entity-atlas-review-safety-boundaries.md` and `review-boundaries-matrix.md` reinforce the review-lane vocabulary. |
| Include one cross-entity relationship-heavy workflow. | PASS | `manager-daily-brief.md` was sampled as the relationship-heavy aggregator: operating day/location, demand/occupancy/labor facts, checkout packets, retention packets, data-quality issues, source refs/provenance, labor estimates, and storage outcome records. |
| Include one safety/review-gated workflow. | PASS | `daily-updates-pawgress-drafts.md` was sampled as a review-gated workflow: care notes become draft customer copy, included/omitted facts, internal flags, approval record, and blocked send stub; every current customer send remains human-approved. `booking-triage.md` was also sampled for hard booking/payment/vaccine/provider-write stops. |
| Flag broken chains, missing authority statements, and unclear human-review boundaries. | PASS / none blocking | Link checking passed. Manual and scripted scans found source-of-record/authority, app/domain/storage/provider contract, human-review, blocked-action, outcome, and evidence sections in the representative workflow pages. |

## Workflow-by-workflow comprehension notes

### Booking Triage — safety and review-gated booking workflow

Non-coder comprehension: strong. The page names the inquiry/request, reservation state, customer, pet, service/date/capacity, vaccine/document evidence, payment/deposit state, behavior/care/incident notes, and policy/staffing snapshots before it names code. The contract table separates `app::booking_triage` packet behavior from domain policy/source/provider boundaries.

Authority and human-review boundary: clear. Deterministic readiness buckets are app-owned, while confirmations, denials, waitlist movement, capacity holds, room/group assignments, vaccine/document approval, behavior/care exceptions, payment/deposit actions, provider/PMS writes, and customer-facing copy remain staff/manager/reviewer decisions.

Broken-chain finding: none blocking. The page explicitly says no dedicated booking-triage outcome projection exists and frames durable labor reporting as future/planned, which avoids overclaiming.

### Daily Updates / Pawgress Drafts — safety/review-gated customer-message workflow

Non-coder comprehension: strong. The page explains care notes, pet/owner/reservation subject, message draft, included and omitted facts, internal flags, approval record, send stub, and media refs in operator language. A reader can tell the workflow is about safer draft writing, not automatic Pawgress delivery.

Authority and human-review boundary: clear. The app may build a preview packet and draft copy, but every current send is review-required; medical, behavior, incident, media/privacy, policy, and source-correction questions stay with humans or approved systems.

Broken-chain finding: none blocking. The page repeatedly preserves the MVP/draft-only status and notes that durable writing-time outcome storage is not yet identified.

### Manager Daily Brief — cross-entity relationship-heavy workflow

Non-coder comprehension: strong. This is the clearest relationship-heavy workflow: operating day/location + demand/occupancy/labor/source facts + checkout and retention packets + data-quality issues become a manager brief packet, ranked actions, labor estimates, and outcome records.

Authority and human-review boundary: clear. The app can rank, summarize, draft internal review work, estimate labor impact, and record reviewed outcomes. Humans approve staffing/schedule changes, source cleanup, policy exceptions, customer follow-up, provider/PMS updates, payments/refunds/discounts, and regional/personnel implications.

Broken-chain finding: none blocking. Unlike several other workflows, this page has concrete storage outcome evidence via `ManagerDailyBriefOutcomeRecord`, and the page correctly limits that evidence to reviewed local labor outcomes rather than production ROI proof.

### Data Quality Hygiene — source/provenance and cleanup contract workflow

Non-coder comprehension: strong. The page makes the central entity the suspect fact and its source proof, then shows how data-quality issues, source refs, provenance, field paths, sensitivity, workflow-blocking state, candidates/actions, and outcome records relate.

Authority and human-review boundary: clear. The app ranks and drafts internal cleanup work and validates submissions; humans approve duplicate reconciliation, destructive merges/deletes, provider/PMS repair, vaccine/medical/profile fixes, payment conflicts, policy-sensitive mapping, sensitive payload handling, and source ambiguity resolution.

Broken-chain finding: none blocking. This page has app/API/storage contract evidence and appropriately says outcome capture does not prove that the provider source was repaired.

### Grooming Rebooking / Retention — cross-workflow dependency from checkout to outreach

Non-coder comprehension: strong. The page makes the dependency chain understandable: staff-verified checkout/completed service evidence plus customer/pet/reservation identity, grooming cadence, contact permission, suppression flags, and source-grounded reason codes feed a retention packet and staff review draft.

Authority and human-review boundary: clear. Customer messages, offers, discounts, bookings, provider/calendar writes, DNC/consent handling, complaint-sensitive outreach, and payment movement remain human/approved-system decisions.

Broken-chain finding: none blocking. The page correctly identifies a storage gap: app-level `OutcomeRecord` exists, but no dedicated durable grooming-retention storage projection is claimed.

### Regional Labor Exceptions / Future Portfolio View — planned/future workflow

Non-coder comprehension: acceptable and intentionally conservative. The page clearly labels itself planned/future and states that existing evidence is indirect through manager daily brief outcomes, data-quality outcomes, operations vocabulary, storage portfolio records, and Gingr labor/reservation evidence.

Authority and human-review boundary: clear. Regional leaders/humans approve GM follow-up, staffing/personnel actions, pricing/policy changes, customer communications, provider writes, schedule mutation, source cleanup, and BI reclassification.

Broken-chain finding: none blocking. The page repeatedly states there is no dedicated regional app module, packet, API endpoint, tests, or durable regional outcome record. This is a known gap, not a broken claim.

## Cross-page chain audit

| Chain | Result | Notes |
| --- | --- | --- |
| Workflow entrypoint -> entity family pages | PASS | `workflow-to-entity-navigation-map.md` gives a row for every sampled workflow and links to entity families. |
| Workflow page -> app/domain/storage/provider contracts | PASS | Each sampled workflow cites source paths, module/type names, tests, and Rustdoc targets or explicitly states when a contract is missing/future. |
| Source/provider evidence -> app packet -> draft/recommendation | PASS | Workflow pages and `workflow-packets.md` consistently state that provider/source facts are evidence and app packets are reviewable bundles, not live authority. |
| Draft/recommendation -> human review gate | PASS | Review gates, reviewer lanes, and blocked actions are visible on the workflow pages and consolidated in the safety boundary docs. |
| Outcome/labor claim -> storage/outcome evidence | PASS | Manager brief and data-quality hygiene cite storage outcome records. Booking, checkout, retention, daily update, and regional pages correctly mark durable outcome storage as future/gap where applicable. |
| Planned/future claims -> current evidence | PASS | Regional labor exceptions and missing durable outcome projections are labeled planned/future instead of presented as shipped automation. |

## Broken chains / unclear boundaries flagged

No blocking findings.

Non-blocking caveats to preserve in future edits:

1. Several workflows have app/test evidence but not dedicated durable outcome storage: booking triage, checkout completion, grooming/CRM retention, and daily updates. Current pages handle this correctly; future edits should not turn qualitative labor value into measured savings unless storage/outcome evidence lands.
2. Regional labor exceptions remains a future portfolio concept. Future edits should keep the planned/future label until a dedicated app packet, API endpoint, tests, and durable regional outcome record exist.
3. Customer-message workflows should continue to say “draft,” “approval record,” and “blocked send stub” rather than “send” unless a later approved send path is implemented and cited.
4. Provider/Gingr and payment language should continue to distinguish source evidence from write authority; none of the sampled pages currently claims live provider/payment authority.

## Verification performed

- Manual non-coder read of representative workflow pages and contract/boundary pages listed above.
- Scripted broad-section scan for source/entity, contract, authority, agent, human, blocked, outcome, and evidence language across sampled pages.
- Generated Rustdoc evidence locally with `cargo doc --workspace --no-deps` so generated `target/doc` anchors existed for link validation.
- Markdown link check: `./scripts/check_markdown_links.py` passed with `302 markdown files scanned; 21 required README entries checked`.

## Conclusion

The representative workflow and contract docs meet the task acceptance criteria. A non-coder can trace participating entities, understand the contracts/rules between them, and distinguish automation drafting/recommendation from human decisions. The current gaps are labeled as gaps rather than hidden claims, and the sampled pages do not blur human-review boundaries or live side-effect authority.
