# Entity atlas non-coder QA findings

Task: `t_e119b371` — non-coder comprehension and safety-boundary review for the completed entity atlas.

Review question: can a sample manager or owner answer, for each important entity, “what is this and why does it matter to labor cost or safety?”

Verdict: PASS.

The completed atlas pages are usable by a non-coder/operator. Each sampled family explains the entity in plain resort language, names the labor or safety problem, identifies who or what is authoritative, separates allowed automation from blocked actions, and points to evidence/outcome fields. The pages repeatedly prevent unsafe readings such as “the agent can send to customers,” “the agent can write Gingr/PMS,” or “a draft equals an executed action.”

## Scope reviewed

Primary atlas and relationship artifacts:

- `docs/design/entity-atlas-inventory.md`
- `docs/design/entity-atlas-page-template.md`
- `docs/design/entity-atlas-workflow-packets-agents.md`
- `docs/design/entity-atlas-petsuites-core-entities.md`
- `docs/design/entity-atlas-revenue-opportunity-entities.md`
- `docs/design/source-provenance-data-quality-atlas.md`
- `docs/design/entity-atlas-review-safety-boundaries.md`
- `docs/design/entity-atlas-outcomes-operations-money.md`
- `docs/design/entity-atlas-runtime-storage-api-surfaces.md`
- `docs/design/entity-atlas-relationships.md`
- `docs/integrations/gingr/provider-boundary-atlas.md`

Operator and safety context sampled:

- `docs/workflows/operator/README.md`
- `docs/workflows/operator/manager-daily-brief.md`
- `docs/workflows/operator/booking-triage.md`
- `docs/workflows/operator/data-quality-hygiene.md`
- `docs/workflows/operator/checkout-completion.md`
- `docs/workflows/operator/grooming-rebooking-retention.md`
- `docs/workflows/operator/daily-updates-pawgress-drafts.md`
- `docs/workflows/operator/regional-labor-exceptions.md`
- `docs/safety/source-evidence-map.md`
- `docs/safety/review-boundaries-matrix.md`
- `docs/safety/evidence-policy-blocked-actions-outcomes.md`
- `docs/safety/agent-safety-model-for-operators.md`
- `docs/safety/labor-cost-with-human-review-crosswalk.md`

## Pass/fail by family

| Family | Sampled entities / sections | Result | Non-coder comprehension | Safety-boundary finding |
| --- | --- | --- | --- | --- |
| Workflow packets and agents | `Agent spec and workflow agent`, `Agent prompt packet`, `Workflow event/result`, `Manager daily brief packet`, `Booking triage packet`, `Data-quality hygiene packet`, `Checkout completion packet`, `CRM retention / grooming rebooking packet`, `Daily update / Pawgress draft packet`, `Domain daily brief and operations vocabulary` in `docs/design/entity-atlas-workflow-packets-agents.md` lines 221-316 | PASS | Each workflow-facing packet/agent entry uses “What it is” and “Why it exists” bullets. A manager can see whether the item is a packet, draft, queue, event/result, or domain vocabulary and why it saves reconciliation/writing/review labor. | Allowed and blocked actions are explicit at lines 132-177, and per-entry review gates/evidence are listed at lines 254-305. Daily updates explicitly use a blocked send stub until human approval. |
| Reservations, boarding, daycare, pet, customer, care | `Reservation`, `Boarding contract`, `Daycare contract`, `Pet`, `Customer`, `Vaccine record and vaccine policy`, `Temperament and group-play eligibility`, `Incident`, `Care profile`, `Location`, `Staff task` in `docs/design/entity-atlas-petsuites-core-entities.md` lines 91-276 | PASS | Entries explain concrete resort concepts rather than code names: booking/stay, care profile, customer, vaccine, temperament, incident, location, and staff task. Labor/safety purpose is stated in the frontmatter and in each entry. | The page blocks autonomous customer sends, PMS writes, booking/check-in/checkout/schedule/payment mutations, safety approvals, and incident closure/downgrade without review. |
| Grooming, training, retail, retention, revenue | `docs/design/entity-atlas-revenue-opportunity-entities.md` sections 41-207 | PASS | The page translates grooming, training, inventory, vendor/reorder, CRM retention, reputation, and lead concepts into revenue-opportunity language that a manager can connect to missed follow-up, rebooking, stock, and review-response labor. | The page frames recommendations as reviewable opportunities and blocks appointment moves, customer sends, discounts, payment/refund/POS movement, inventory/vendor changes, and reputation/customer-facing responses without approval. |
| Source, provenance, data quality, documents, audit | `Source system and source fact`, `Provenance and record ref`, source snapshots, data-quality issues, document evidence, audit evidence in `docs/design/source-provenance-data-quality-atlas.md` lines 21-571 | PASS | Strongest non-coder framing: “where did this fact come from,” “receipt,” “missing/stale/conflicting/sensitive facts,” and “none of these grant automation authority by themselves.” | Explicitly treats provider/import/document facts as evidence, not domain truth. Blocks source repair, provider/PMS mutation, schedule/payment/customer decisions, hiding/deleting source issues, and compliance/safety approvals without review. |
| Review gates, blocked actions, approval, messages | `Review gate`, `Policy rule / automation level`, `Allowed action / safe agent action`, `Blocked action`, `Draft / message`, `Approval record / human approval`, `Agent spec / prompt packet`, `Outcome capture / audit evidence` in `docs/design/entity-atlas-review-safety-boundaries.md` lines 182-245 | PASS | Clear operator vocabulary for what automation may read/draft/route/recommend/record and when it must stop. The page makes “draft/message” and “approval record” understandable without code knowledge. | This is the core safety page. It separates read-only, draft-only, review-required, and never-direct actions and blocks customer sends, provider/PMS writes, schedule/capacity changes, payment/refund/discount movement, source hiding, and safety approvals. |
| Outcomes, labor, operations, analytics, money | `Labor Minutes`, `Manager Daily Brief Outcome Record`, `Data-quality Hygiene Outcome Record`, `Analytics Facts`, `Operations Context and Service Offering`, `Money and Payment Evidence`, `Safe Result Envelope / Draft-Review-Blocked State` in `docs/design/entity-atlas-outcomes-operations-money.md` lines 224-388 | PASS | The page tells an owner what evidence proves labor savings or safe blocking, instead of claiming AI productivity without proof. Money/payment concepts are framed as evidence for review rather than authority to move funds. | Outcome fields, labor minutes, blocked reasons, source refs, reviewer ids, and draft/sent distinctions prevent unsafe claims. It blocks payment/refund/discount movement and treats sent/customer-facing states as review-dependent evidence. |
| Gingr provider boundary | `docs/integrations/gingr/provider-boundary-atlas.md` sections 52-242 | PASS | The page clearly separates “what Gingr says” from “what NVA derives,” using provider-boundary language that a manager can understand as source evidence quarantine/promotion. | Strong boundary: read/list/fetch/verify/map/draft/report only. Blocks provider writes, source deletion, customer sends, check-in/out, schedule/capacity mutations, payment/refund/discount changes, and secret exposure. |
| Runtime, storage, API, worker, CLI, contract tests | `Storage operations boundary`, `API request/response contracts`, `Worker runtime`, `CLI/local smoke`, `App tool ports/errors`, `App packets/outcomes`, `Contract tests` in `docs/design/entity-atlas-runtime-storage-api-surfaces.md` lines 78-393 | PASS | The page successfully translates runtime plumbing into “safe plumbing that stores review evidence, exposes typed requests/responses, runs future work, prints inspection JSON, and proves contracts.” | It states runtime shells do not own domain truth or live side-effect authority. Side effects are blocked by default; customer sends, PMS writes, schedule/payment/refund/discount/source hiding/safety approvals remain outside autonomous authority. |
| Cross-family relationships | `docs/design/entity-atlas-relationships.md` sections 41-158 | PASS | The diagram and edge tables answer how facts move from provider/source evidence to packet, draft, review, outcome, and storage. Useful for operators who need the whole map rather than individual pages. | Relationship edges preserve the safety story: source facts do not authorize action, packets may draft/rank/summarize, human review gates decide, and outcomes/storage prove what happened. |
| Inventory and template | `docs/design/entity-atlas-inventory.md` and `docs/design/entity-atlas-page-template.md` | PASS | Inventory gives the family queue and non-goals; template gives the reusable page contract in non-coder language. | Template requires allowed/blocked summaries and outcome fields; inventory explicitly says docs do not authorize autonomous sends, PMS/Gingr writes, schedule mutations, payment/refund moves, or unreviewed safety decisions. |

## Jargon-only definition check

No blocking jargon-only definitions found.

Examples of plain-English framing that work for a non-coder:

- Workflow page: “review packets, agent specs, prompt bundles, drafts, outcomes, and source-backed queues” plus the labor problem of reducing dashboard reconciliation, draft writing, review routing, and outcome measurement.
- Core entity page: “booking, stay, pet, customer, safety, and staff facts” before confirming, checking in, updating, checking out, or escalating work.
- Source page: source facts answer “where did this operational fact come from?” and provenance preserves the receipt.
- Runtime page: “safe plumbing” for review evidence, typed requests/responses, local inspection JSON, and contract tests.
- Gingr page: “what Gingr says” vs. “what NVA derives from it.”

Non-blocking note: the source/provenance page uses per-entry fenced metadata rather than one top-level frontmatter block. That is acceptable for this family page because each entry still includes the equivalent template fields and the first section explains the family in plain language.

## Missing purpose / authority / allowed / blocked / evidence fields

No blocking omissions found in completed family pages.

Observed coverage:

- Purpose / labor problem: present in the frontmatter or first purpose section for each family page.
- Authority: each family names source-of-record and authoritative human role, or an equivalent “source of record and human role” section.
- Allowed actions: present as `allowed_action_summary` and/or “Allowed actions” section.
- Blocked actions: present as `blocked_action_summary` and/or “Blocked actions and review gates” section.
- Evidence/outcome fields: present as `outcome_fields` and/or “Safe-use evidence and outcome fields” section.

The relationship map and inventory are not standalone entity pages, so they do not need every template field. They still reinforce the same relationship and safety concepts.

## Unsafe automation scan

Scan terms checked across atlas, operator workflow, and safety documents: `send`, `write`, `payment`, `refund`, `schedule`, `approve`, `delete`, `hide`.

Result: PASS.

The scanned uses are overwhelmingly blocked-action language, review-gate language, or evidence/outcome language. I did not find text implying that automation may directly:

- send customer-facing messages without approved sender review;
- write to Gingr/PMS/provider systems without an explicit human/system-of-record approval boundary;
- move schedule, check-in/out, waitlist, room, capacity, payment, refund, discount, or source records autonomously;
- approve medical/safety/policy decisions;
- hide/delete source evidence or data-quality issues to make work look complete.

Representative safe phrasing:

- `docs/design/entity-atlas-workflow-packets-agents.md` lines 298-305: Pawgress daily-update packets produce customer-message drafts, approval records, and a send stub blocked by human approval.
- `docs/design/entity-atlas-review-safety-boundaries.md` lines 140-160: blocked actions require named review gates and may not be treated as executed work.
- `docs/design/entity-atlas-outcomes-operations-money.md` lines 370-388: safe-result envelopes distinguish draft/review/blocked states and say draft status is not proof of send.
- `docs/integrations/gingr/provider-boundary-atlas.md` lines 169-183: provider writes, source deletion, customer sends, schedule/check-in/out, and payment/refund/discount changes are blocked.
- `docs/design/entity-atlas-runtime-storage-api-surfaces.md` lines 31-33 and family sections: runtime/storage/API shells expose inspection and evidence surfaces, not live side-effect authority.

## Workflow-facing packet/agent coverage

Every workflow-facing app packet/agent entity named in the workflow atlas was sampled and passed:

- Agent spec and workflow agent — PASS
- Agent prompt packet — PASS
- Workflow event/result — PASS
- Manager daily brief packet — PASS
- Booking triage packet — PASS
- Data-quality hygiene packet — PASS
- Checkout completion packet — PASS
- CRM retention / grooming rebooking packet — PASS
- Daily update / Pawgress draft packet — PASS
- Domain daily brief and operations vocabulary — PASS

These entries are concise enough for a sample manager to answer:

1. What is it?
2. Why does it reduce labor or safety risk?
3. What can the agent/app do with it?
4. What must a human or system of record still decide?
5. What evidence proves the result?

## Remediation tasks

No remediation cards are required for blocking gaps.

Optional future polish, not required for acceptance:

1. If these pages later move from draft to published docs, consider adding a one-screen “manager reading path” at the top of the atlas index: inventory -> relationship map -> workflow packets -> review/safety boundaries -> outcomes. This would reduce first-time navigation effort but is not a correctness gap.
2. If source/provenance entries are later split into standalone pages, convert each fenced per-entry metadata block into top-level frontmatter. Current family-page form is acceptable and understandable.

## Verification performed

Commands/checks:

- `python scripts/check_markdown_links.py` — passed: 272 markdown files scanned; 21 required README entries checked.
- Scripted section/field scan across the atlas files for template-field coverage: `plain_english_definition`, `primary_labor_problem`, `source_of_record`, `authoritative_human_role`, `allowed_action_summary`, `blocked_action_summary`, `outcome_fields`.
- Scripted unsafe-term scan across reviewed atlas pages for: `send`, `write`, `payment`, `refund`, `schedule`, `approve`, `delete`, `hide`.
- Manual non-coder review of sampled entity entries and every workflow-facing app packet/agent entity listed above.
