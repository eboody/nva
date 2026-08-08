# Rustdoc smell verification and evidence crosswalk — t_dbf6b850

Date: 2026-06-19
Workspace: `/home/eran/code/nva`

## Verdict

Fresh exact target-smell verification passes: the pass-2 filler phrases and copied template phrases from `t_dbf6b850` have 0 matches across Rust sources under `domain/src`, `storage/src`, `app/src`, `apps`, and `integrations`.

That exact banned-smell pass is narrower than the later non-coder/entity-contract QA in `t_9d9555b9`. The exact regex proves the original copied phrases are gone; it does not, by itself, prove every Rustdoc surface is high-context enough for non-coder review.

Follow-up remediation `t_c6c322dd` cleared the first broader residue set across `customer`, `daily_brief`, `source`, `temperament`, `incident`, `lead`, `reputation`, `staff`, `message`, `daycare::front_desk`, and `document`. Follow-up remediation `t_37d60b4f` then cleared the remaining source/daycare QA blockers called out by `t_9d9555b9`: source scalar wrappers/getters now name endpoint, record, batch, schema, payload, status, and reconciliation authority; reservation snapshot getters and builders now name promotion, data-quality, and reviewer contracts; cancelled status now explains active-workflow blocking while preserving source status; Gingr related ids and ambiguity counts now name reconciliation work; daycare care readiness now distinguishes care-team medical/behavior/handling gates from package/payment and manager policy readiness.

No code behavior changed; edits are localized Rustdoc/comment changes.

## Fresh scan commands

Exact target and copied-template smell scan:

```sh
rg -n "data-quality finding for cleanup or review|Returns this data quality value|Returns this analytics value|operations signal for labor, capacity|operating lane used for labor, capacity, reporting, or manager-review planning|retained for portfolio/service-line planning|Typed .* keeps raw primitives|Domain vocabulary for .* decisions in operations workflows|Promotes boundary input|Exposes the validated scalar|operational signal|Source-derived|source-derived operational|Domain vocabulary for .* decisions|document classification or pipeline state used for review and retention|Deposit collection was attempted but did not succeed|No deposit or review is needed for this reservation path|Physical or operational lane" domain/src storage/src app/src apps integrations --glob '*.rs'
# result: 0 matches; rg exits 1 because no matches were found
```

Related broad-language scan retained for reviewer evidence:

```text
13 broad matches remain for legitimate entity-contract language containing "used for", "feeds", or "typed domain value". They are crosswalked below and were not rewritten because each names a concrete domain contract, not filler.
```

## Localized cleanup crosswalk

| Surface | Changed occurrence(s) | Entity purpose / relationship / authority / contract evidence now named |
| --- | --- | --- |
| `domain/src/temperament.rs` | Replaced `Domain vocabulary for ... decisions in temperament workflows` on `GroupPlayObservation`, `PeopleOrientation`, `Rating`, and `BehaviorObservation`. | Purpose: behavior/play-safety review. Relationships: group-play assignment, handling, staffing, customer follow-up. Authority/safety: staff-observed temperament evidence gates play access and manager review. Labor value: consistent signals reduce manual note triage without overriding behavior policy. |
| `domain/src/incident.rs` | Replaced `Domain vocabulary for category/severity decisions in incident workflows`. | Purpose: incident routing. Relationships: injuries, behavior incidents, escapes, property, medication follow-up. Authority/safety: severity ranks manager attention and customer communication. Labor value: staff triage can prioritize safety work. |
| `domain/src/lead.rs` | Replaced `Domain vocabulary for source/intent/conversion stage decisions in lead workflows`. | Purpose: demand/intake tracking. Relationships: marketing source, boarding/daycare/grooming/training intent, booked/lost/inactive stage. Authority/contract: lead signals route follow-up but do not create reservations. Labor value: intake staff can focus follow-up by origin and stage. |
| `domain/src/reputation.rs` | Replaced `Domain vocabulary for sentiment/theme decisions in reputation workflows`. | Purpose: customer-feedback triage. Relationships: service recovery, staffing, facility, pricing evidence. Authority/safety: reputation signals are evidence for follow-up, not operational truth by themselves. Labor value: reviews route to the right manager queue. |
| `domain/src/staff.rs` | Replaced `Domain vocabulary for assignment/source decisions in staff workflows`. | Purpose: labor coverage modeling. Relationships: scheduled/backup/inactive labor and provider authority. Authority/contract: source system is retained for labor reconciliation. Labor value: staffing records remain auditable. |
| `domain/src/daily_brief.rs` | Replaced `Domain vocabulary for limit/follow-up/watch/revenue opportunity decisions`; replaced `Returns this daily brief value's booked/capacity`. | Purpose: manager daily-brief queue ranking. Relationships: pet/customer/revenue tasks, vaccination/incident/medication/temperament/feeding attention, occupancy pressure. Safety: watch reasons flag care blockers. Labor value: booked/capacity values rank labor pressure and overbooking risk. |
| `domain/src/source.rs` | Replaced `Domain vocabulary for related provider id decisions in source workflows`. | Purpose: source-evidence reconciliation. Relationships: provider IDs linked to related source records. Authority: source provenance is explicit for audit trails and promotion boundaries. |
| `domain/src/document.rs` | Replaced repeated `document classification or pipeline state used for review and retention`; fixed copied deposit comments on virus scan and PII redaction states. | Purpose: document intake/review pipeline. Relationships: vaccine proofs, waivers, photos, medical records, incident evidence, customer/staff/provider/migration sources. Authority/safety: stored object, scan/redaction/extraction/reviewer state determine whether evidence can support compliance, care, messaging, or automation. Labor value: scan/extraction queues reduce manual inspection while preserving reviewer gates. |
| `domain/src/daycare/front_desk.rs` | Replaced `Physical or operational lane used to sort daycare front-desk work`. | Purpose: front-desk throughput. Relationships: arrivals, pickups, incidents, billing, package help. Safety/workflow: lane selection routes daycare work without implying automated customer send. |
| `domain/src/customer.rs` | Rephrased lowercase module sentence `used for labor-saving automation...`. | Purpose: customer identity/contact context. Relationships: inbox drafts, reservation triage, follow-up. Labor value: automation receives identifiers without making the comment sound like a free-floating generic workflow. |
| `domain/src/message.rs` | Replaced copied `Deposit collection was attempted but did not succeed` on message `Failed`. | Purpose: customer/internal communication lifecycle. Relationship/safety: failed send needs retry, suppression, or staff review before customer contact. |
| `domain/src/boarding/mod.rs` | Rephrased `No deposit or review is needed...` on boarding deposit rule. | Purpose/contract: boarding reservation path can be secured without deposit collection; deposit requirement remains explicit when present. |
| `domain/src/payment/mod.rs` | Rephrased `No deposit or review is needed...` and `Deposit collection was attempted but did not succeed`. | Purpose/contract: deposit lifecycle distinguishes not-required, required, paid, refunded, failed, and manager-waived states. Authority/workflow: failed collection requires retry, waiver, or manager reconciliation. |

## Remaining broad related-language crosswalk

These are the only lines still caught by a deliberately broad scan for `used for|feeds` around labor/capacity/workflow terms. They are retained because each is entity-specific and tied to purpose, relationship, authority/contract, safety, or labor-value evidence.

| File:line | Text | Evidence basis / why retained |
| --- | --- | --- |
| `domain/src/operations.rs:394` | `Boarding add-on vocabulary used for reviewed upsell, capacity, and labor signals.` | Entity: boarding add-on. Contract: reviewed upsell/capacity/labor signal; review wording prevents automatic sale or capacity override. |
| `domain/src/entities.rs:14` | `snapshot, reviewer approval, audit event, and typed domain value for each field. Review` | Entity-atlas module contract: typed field values tie source snapshots to reviewer approval and audit events. This is a module-level purpose sentence, not a raw-primitive placeholder. |
| `domain/src/entities.rs:382` | `Resort service line used for labor planning, capacity, policy, upsell, and workflow routing.` | Entity: service line. Relationship: labor/capacity/policy/upsell/routing. This names why the enum exists across workflows. |
| `domain/src/boarding/accommodation.rs:13` | `Premium dog boarding suite option used for capacity matching and upgrade offers.` | Entity: accommodation type. Contract: capacity matching and upgrade offers, not generic operational language. |
| `domain/src/boarding/mod.rs:284` | `Room-cleaning cadence that feeds labor planning for the stay.` | Entity: housekeeping/care cadence. Labor-value evidence: room-cleaning work drives staffing. |
| `domain/src/training/mod.rs:420` | `Training program sold or fulfilled by the resort, used for capacity, package, and outcome planning.` | Entity: training program. Relationship: capacity, package ledger, outcome planning. |
| `storage/src/operations.rs:222` | `Business date used for labor and reporting aggregation.` | Storage projection contract: date dimension for labor/report aggregation, tied to storage-side grouping authority. |
| `storage/src/operations.rs:290` | `Business date used for labor and reporting aggregation.` | Same storage projection contract on a separate record shape. |
| `storage/src/operations.rs:318` | `Returns the aggregation dimensions used for labor reporting.` | Getter documents the aggregate dimensions, not a generic value placeholder. |
| `storage/src/operations.rs:404` | `Business date used for labor and reporting aggregation.` | Same storage projection contract on a separate record shape. |
| `storage/src/operations.rs:477` | `Business date used for labor and reporting aggregation.` | Same storage projection contract on a separate record shape. |
| `storage/src/operations.rs:505` | `Returns the aggregation dimensions used for labor reporting.` | Getter documents the aggregate dimensions, not a generic value placeholder. |
| `storage/src/service_line/retail.rs:3` | `Retail records preserve partner/product evidence used for operational upsell` | Storage/service-line relationship: partner/product evidence backs retail upsell; this is evidence-preservation language. |

## Broader residue remediation from `t_c6c322dd` and `t_37d60b4f`

The later non-coder review found additional repeated/copy-pasted Rustdoc language that was outside the original exact scan: daily-brief field/variant filler, source getter boilerplate, copied provider-role/status `Unknown` comments, message/staff/incident/reputation/lead template wording, and daycare/package readiness comments with the wrong care/payment/review semantics. The first broader residue patterns were remediated by `t_c6c322dd`; the remaining source/daycare blockers were remediated by `t_37d60b4f`. Any future matches for generic phrases outside those targeted remediation sets should be treated as a new artifact-family review, not evidence that the exact `t_dbf6b850` scan alone failed.

## Verification

| Command | Result |
| --- | --- |
| `rg -n "...target smell regex..." domain/src storage/src app/src apps integrations --glob '*.rs'` | Passed: 0 matches (`rg` exit 1 because no matches were found). |
| Broad related scan over Rustdoc comments for `used for|feeds|typed domain value` patterns | 13 retained matches, all crosswalked above as entity-specific evidence. |
| `rg -n "Returns the .*used to explain|used to explain|Candidate count attached|Number of provider records that could match|Promotes non-empty provider or import text into a source-lineage value|Returns the provider or domain identifier as a string slice|source-data role, provider status, or explicit normalization assumption|Provider reports the reservation was cancelled before or during care" domain/src/source.rs domain/src/daycare/front_desk.rs` | Passed for `t_37d60b4f`: 0 matches. |
| `rg -n "Care-team medical|Package/payment|manager policy|Specific care-team gate" domain/src/daycare/front_desk.rs` | Passed for `t_37d60b4f`: confirms daycare care-readiness docs distinguish medical/behavior/handling gates from package/payment and manager policy readiness. |
| `cargo fmt -p domain --check` | Passed. |
| `git diff --check -- domain/src/source.rs domain/src/daycare/front_desk.rs` plus whitespace scan for this untracked crosswalk | Passed. |
| `cargo test -p domain --lib --tests` | Passed: 94 tests. |
| `python3 scripts/check_rustdoc_completeness.py domain/src/source.rs domain/src/daycare/front_desk.rs` | Passed: strict `RUSTDOCFLAGS='-D missing_docs' cargo doc --workspace --no-deps`; rendered rustdoc smoke check passed. |
| `python3 scripts/check_markdown_links.py --repo-root .` | Passed: 316 markdown files scanned; 21 required README entries checked. |

## Caveats

The shared checkout contains many pre-existing modified/untracked files from adjacent documentation lanes. This task only changed localized Rustdoc/comments in the domain files named in the cleanup crosswalk and added this verification report.
