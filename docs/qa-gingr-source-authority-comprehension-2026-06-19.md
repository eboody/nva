# QA: Gingr/source authority comprehension

Task: `t_94b224a0`
Date: 2026-06-19

Verdict: PASS, with non-blocking polish notes.

A non-coder can distinguish Gingr/source facts from NVA app-generated packets, drafts, recommendations, outcomes, and storage projections across the sampled entity-atlas and Gingr/source docs. The reviewed docs consistently say that Gingr/provider/import/document facts are evidence, not automatic domain truth or operational approval. They also name the human/system-of-record gates that prevent silent overwrites, customer sends, provider writes, schedule/capacity changes, payment movement, and safety approvals.

## Scope reviewed

Primary source/authority docs:

- `README.md`
- `docs/design/entity-index.md`
- `docs/design/source-provenance-data-quality-atlas.md`
- `docs/integrations/gingr/provider-boundary-atlas.md`
- `docs/integrations/gingr/source-inventory.md`
- `docs/integrations/gingr/sdk-endpoint-catalog.md`
- `docs/integrations/gingr/bi-read-model-contract.md`
- `docs/design/entity-atlas-petsuites-core-entities.md`
- `docs/design/entity-atlas-workflow-packets-agents.md`
- `docs/design/entity-atlas-review-safety-boundaries.md`
- `docs/design/entity-atlas-revenue-opportunity-entities.md`
- `docs/design/entity-atlas-non-coder-qa-findings.md`

Representative source-backed entities sampled:

- Source system and source fact
- Provenance and source ref
- Source reservation snapshot / Gingr reservation evidence
- Data-quality issue
- Customer / pet parent
- Pet, care, vaccine, temperament, and incident facts
- Reservation / stay
- Message / customer draft
- Workflow packet / review queue
- Data-quality hygiene packet
- Daily update / Pawgress draft packet
- Review gate / approval record / blocked action
- Gingr endpoint / DTO / response / webhook / mapping candidate
- Grooming/training/retail opportunity and Gingr commerce evidence
- Outcome record / labor minutes
- Storage projection / runtime shell

## Findings by acceptance criterion

### 1. Authority, import/sync, inference, and overwrite boundaries

PASS.

The docs repeatedly separate authority by question rather than assigning one blanket owner:

- `docs/design/source-provenance-data-quality-atlas.md` says source facts answer “where did this operational fact come from,” and explicitly says provider/import/document facts do not grant automation authority by themselves.
- `docs/integrations/gingr/provider-boundary-atlas.md` separates “what Gingr says” from “what NVA derives.” It says Gingr records provide source evidence, while NVA domain/app/storage contracts and human review own policy, workflow disposition, blocked actions, and outcomes.
- `docs/integrations/gingr/bi-read-model-contract.md` gives the clearest import/promotion chain: Gingr API/report/webhook facts -> raw DTOs -> Gingr snapshots with provenance -> source-agnostic reservation/stay snapshots -> analytics facts/read models -> future validator inputs. It explicitly says each arrow is a trust-boundary crossing and that conflicting pulls should emit data-quality issues instead of silently overwriting contradictions.
- Core entity docs say provider/source evidence may feed customer, pet, reservation, vaccine, care, temperament, incident, and payment context, but staff/manager/document/care reviewers remain authoritative for exceptions, safety, lifecycle execution, customer sends, and money movement.
- Runtime/storage docs in the index frame storage projections as durable evidence/proof, not the business decision-maker.

No reviewed page suggested that an app packet, storage row, read model, source ref, provenance object, or Gingr provider id can silently overwrite domain or provider facts.

### 2. Non-coder can distinguish Gingr/source facts from app-generated packets/recommendations

PASS.

The strongest non-coder distinction appears in these phrases and structures:

- Gingr/provider boundary page: “what Gingr says” vs. “what NVA derives.”
- Entity index relationship shortcut: provider/staff/import evidence -> source ref/provenance/data-quality status -> domain entity/service-line truth -> app workflow packet/review queue -> optional draft/summary -> human/system-of-record review -> outcome/storage proof.
- Workflow packets page: packets are source-backed review bundles that may draft, rank, validate, route, or record; they are not live resort authority.
- Review safety page: source facts explain why a recommendation exists; review gates name the unresolved human/system stop; blocked actions name what automation must not execute.
- Data-quality hygiene workflow: suspect source facts become reviewable cleanup candidates, not hidden normalization or provider repair.

A non-coder should be able to answer:

- Gingr/provider records are upstream evidence.
- DTOs/raw responses/webhooks are quarantined provider shapes until mapped.
- Mapping produces candidates/errors, not final operational truth.
- App packets/drafts/recommendations are review work product.
- Human/system-of-record approval decides live actions.
- Outcome/storage records prove reviewed disposition and labor evidence after the fact.

### 3. Source/Rustdoc/test evidence links and stale/ambiguous authority claims

PASS for blocking evidence links.

Verification performed:

- `python scripts/check_markdown_links.py` passed after fixing generated-Rustdoc links in `docs/design/entity-atlas-evidence-link-anchors.md`: 295 Markdown files scanned; 21 required README entries checked.
- A focused local link scan across the sampled authority docs checked 346 Markdown links and found 0 missing local targets.
- Key app/API/storage/Gingr evidence paths named by the source/authority docs exist, including:
  - `app/tests/booking_triage_mvp.rs`
  - `app/tests/checkout_completion_workflow_contracts.rs`
  - `app/tests/crm_retention_workflow_contracts.rs`
  - `app/tests/daily_care_update_mvp.rs`
  - `app/tests/data_quality_hygiene_workflow_contracts.rs`
  - `app/tests/manager_daily_brief_workflow_contracts.rs`
  - `apps/api/tests/manager_daily_brief_agent_context_contract.rs`
  - `apps/api/tests/manager_daily_brief_agent_drafts_contract.rs`
  - `apps/api/tests/manager_daily_brief_outcome_capture_contract.rs`
  - `apps/api/tests/data_quality_hygiene_agent_contract.rs`
  - `storage/tests/manager_daily_brief_outcome_storage.rs`
  - `storage/tests/data_quality_hygiene_outcome_storage.rs`
  - `integrations/gingr/tests/`

No stale or unsafe authority claim was found in the sampled docs. Gingr read-only claims are carefully narrowed: `sdk-endpoint-catalog.md` says Gingr docs describe the public API as read-only, but the repo still excludes documented side-effect endpoints such as `quick_checkin` and `receive_call` from the v0 read SDK. That distinction prevents the stale/ambiguous reading “all Gingr endpoints are safe because docs say read-only.”

## Non-blocking polish notes

1. Per-row test evidence is uneven in `docs/design/entity-index.md`. The canonical spine often links source/Rustdoc/family pages directly and relies on family pages for test evidence. This is understandable as a navigation index, not a blocker. If future acceptance requires every entity row to answer Q9 standalone, add a compact “test evidence” link or point each row to the family page’s contract-test section.

2. The source/provenance atlas is a family page with embedded per-entry metadata blocks, while most other atlas pages use one top-level frontmatter block. This was already noted in `entity-atlas-non-coder-qa-findings.md` and remains non-blocking because the page is clear and complete, but a future split into standalone source pages should convert each entry to normal frontmatter.

3. Gingr/source docs use both “source of record” and “source evidence.” They generally disambiguate “source of record for what?” well. Keep that question near future pages that cite Gingr, BI/read models, or analytics so non-coders do not read reporting projections as operational authority.

## Remediation

No blocking remediation cards are required.

Optional future docs polish only:

- Add direct test-evidence links to `docs/design/entity-index.md` rows if per-row standalone proof becomes required.
- Split source/provenance family entries into standalone atlas pages only if the docs are later published as independent operator pages.

## Verification commands

```bash
python scripts/check_markdown_links.py
```

Focused evidence scan performed with a local Python script over the sampled docs to verify local links and key evidence paths.
