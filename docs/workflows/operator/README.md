# Operator workflow pages

These pages explain NVA pet-resort labor-saving workflows in operator language before code language. Each page starts with the resort job it saves, the source data it needs, what an agent may draft or rank, what a human must approve, what outcome is measured, and where the source/Rustdoc evidence lives.

Current draft pages:

- [Manager Daily Brief](manager-daily-brief.md) and its [end-to-end walkthrough](manager-daily-brief-walkthrough.md)
- [Booking Triage](booking-triage.md)
- [Data Quality Hygiene](data-quality-hygiene.md)
- [Checkout Completion](checkout-completion.md)
- [Grooming Rebooking / Retention](grooming-rebooking-retention.md)
- [Daily Updates / Pawgress Drafts](daily-updates-pawgress-drafts.md)
- [Regional Labor Exceptions / Future Portfolio View](regional-labor-exceptions.md)

Older workflow/spec pages under `docs/workflows/*.md` remain useful as detailed design and proof artifacts, but they are not the current reader spine. Use the [docs successor and archive map](../../design/successor-archive-map.md#older-workflow-and-specification-docs) when deciding whether an older agent spec, part directory, audit, or Kanban packet should be read as current, supporting proof, background, QA, planning, or archived/superseded evidence.

Safety rule for every page: inherit the default boundary from [evidence, policy, blocked actions, and outcomes](../../safety/evidence-policy-blocked-actions-outcomes.md) and the [review boundaries matrix](../../safety/review-boundaries-matrix.md), then keep only the workflow-specific human approvals and blocked actions on the page itself.

Glossary bridge: link first-use workflow terms to the [glossary index](../../glossary.md) when they affect authority or safety. High-value targets are [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet), [draft](../../glossary-workflow-state-terms.md#draft), [review gate](../../glossary-workflow-state-terms.md#review-gate), [blocked action](../../glossary-workflow-state-terms.md#blocked-action), [outcome capture](../../glossary-workflow-state-terms.md#outcome-capture), [source refs/provenance](../../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence), [source-of-record](../../glossary-source-data-terms.md#source-of-record), and [data-quality issue](../../glossary-source-data-terms.md#domaindata_qualityissue-data-quality-issue).

Evidence maps used for this draft pass:

- [Workflow-to-entity navigation map](../../design/workflow-to-entity-navigation-map.md) — start here when a reader begins with a workflow and needs to land on the entity families, review gates, contracts, and outcome records that make the workflow safe.
- [Operator workflow page inventory](../../design/operator-workflow-page-inventory.md)
- [Workflow page source and Rustdoc backing map](../../design/workflow-page-source-rustdoc-map.md)
- [Workflow packet contract crosswalk](../../entity-atlas/contract-crosswalk/workflow-packets.md) — source-of-truth for which entities each workflow consumes/produces, what remains draft-only, what must be reviewed, and which tests/Rustdocs prove the packet.
- [Runtime exposure crosswalk](../../entity-atlas/contract-crosswalk/runtime-exposure.md) — source-of-truth for API, worker, CLI, web, smoke, and bridge surfaces that expose workflow packets without granting live authority.

Entity-first rule: these workflow pages are not the documentation spine. They are entrypoints into the entity atlas. Each workflow should name the reservation, customer, pet, source/provenance, service-line, review-gate, packet, storage/outcome, and authority entities it reads or produces, then send readers to the relevant atlas page for the full operating model.
