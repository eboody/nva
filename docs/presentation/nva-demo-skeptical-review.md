# NVA demo skeptical review

Status: outsider-readiness review for a hiring manager, job contact, or skeptical technical stakeholder. This review checks whether the presentation package sounds real, honest, useful, safe, runnable, technically credible, and clear about the next ask.

## Verdict

Presentation-ready with caveats. The package is credible because it repeatedly says what is real now: a local, fixture-backed, checked-contract proof of an NVA-owned operations API slice. It does not claim live NVA/Gingr access, production data, provider/PMS writes, customer/member sends, payment movement, schedule changes, medical/safety decisions, or production deployment.

No Red presentation blockers remain. The main Yellow risk is audience expectation management: the presenter must keep saying "local contract proof" and "read-only validation next," not "production integration" or "Gingr replacement today."

## Green / Yellow / Red table

| Category | Score | Skeptical read | Caveat / next-step ask |
| --- | --- | --- | --- |
| Reality | Green | The README, executive brief, walkthrough, talk track, checked OpenAPI artifact, wrapper command, smoke scripts, and expected markers make the runnable proof inspectable. | Run `./scripts/demo_owned_operations_api.sh` shortly before the conversation so output anchors are fresh. |
| Honesty | Green | The no-live-access and non-production boundaries are visible at the top of the presentation docs and repeated in the demo caveats. | Presenter should avoid shorthand like "we integrated Gingr" or "production-ready." |
| Business value | Green | The BI/downstream-cleanup pain and owned read-model/workflow value are understandable without reading Rust. | If the audience is mostly business-side, lead with BI cleanup and labor evidence before DTO/API details. |
| Safety | Green | Live customer/provider/payment/schedule/medical side effects are explicitly blocked; outbox is described as disabled/review-gated posture. | Future live actions require a separately approved adapter path, not a demo toggle. |
| Demo usability | Green | The one-command wrapper plus expected anchors prevents command fumbling; separate-lane commands are still documented. | Keep terminal output zoomed/readable and be ready to skip to expected anchors if the full check is slow. |
| Technical credibility | Yellow | The contract path, OpenAPI artifact, API/storage/migration/test references, and smoke scripts are concrete. | Still be explicit that API state is local/in-memory for the demo and production Postgres wiring, auth/location scope, worker leasing, durable traces, object storage, and rollback are gaps. |
| Next-step clarity | Green | The read-only ask is specific: docs, exports, sample data/source snapshots, and BI query inventory before any live writes or sends. | Phrase this as a narrow validation step, not a request for production credentials. |

## Top 5 likely objections and concise answers

1. **"Is this real without live data?"**
   - Yes as a local contract proof, not as a production integration. It proves the safety, review, outcome, audit, outbox, and BI-read-model shape before anyone grants live access. Real data validates mapping coverage later.

2. **"Why not just use Gingr?"**
   - Gingr may remain a source/system of record during migration, but provider tables should not own NVA's review queues, labor metrics, BI projections, audit posture, or automation boundaries. The owned API gives those concepts product contracts.

3. **"Is this production-ready?"**
   - No. It is presentation-ready architecture and local/demo contract proof. Production still needs approved access, durable persistence, auth/location scope, worker leasing/dead-letter handling, monitoring, rollback, and explicit approval for live side effects.

4. **"What would you need next?"**
   - Read-only validation first: endpoint docs, exports, sample source snapshots, and BI query inventory. Then compare owned read models against real source/BI shape before any dual-run pilot.

5. **"Could this accidentally contact customers or mutate a provider system?"**
   - Not in the current package. The demo is fixture/local, live side effects are disabled, unsafe actions are rejected, and outbox-shaped work is presented as a disabled/review-gated handoff only.

## Overclaim scan findings

No Red overclaim remained after the small fixes below.

Yellow items to keep visible:

- **Checked OpenAPI exists, but production schema/client hardening is not done.** The walkthrough now avoids saying the API is "not yet exported as OpenAPI" and instead says a checked presentation artifact exists while client/schema hardening remains future work.
- **Local API state is not production persistence.** The docs keep the caveat that API handlers are local/in-memory for the demo even though the Postgres migration/storage model is present.
- **Outbox is not permission to send.** The package consistently describes outbox-shaped work as disabled/review-gated posture, not a live sender.
- **BI value is plausible, not yet measured in NVA production.** The docs speak in terms of source-quality cleanup, read-model contracts, and labor evidence, with read-only validation as the next step.
- **Replacement language needs care.** "Owned API replacement path" is acceptable; "replaced Gingr" or "BI can turn off its database" is not.

## Small fixes applied

- Tightened `docs/presentation/job-presentation-walkthrough.md` status line to include schedule authority and medical/safety authority in the non-claim boundary.
- Added medical/safety decisions to the walkthrough's demo no-side-effects sentence.
- Replaced a stale OpenAPI caveat in the walkthrough with the current state: checked OpenAPI artifact exists; client/schema hardening, auth/location scope, and production repository wiring remain future work.
- Added medical/safety decisions to the talk track explanation of `live_side_effects_allowed=false`.
- Tightened the two smoke script safety banners so actual demo output also names medical/safety decisions as blocked.
- Created this skeptical review artifact so future presenters can see the remaining Yellow caveats and likely objections in one place.

## Deferred improvements

- Consider a short recorded/screenshot fallback only if browser/tooling is available; the current static HTML/SVG is acceptable as a visual artifact.
- Before any real pilot conversation, run the full wrapper and docs checks once more from a clean terminal and cite the final pushed commit from the checklist.

## Final presentation checklist recommendations

- Open in this order: executive brief, visual guide or HTML/SVG diagram, then job walkthrough.
- Run `./scripts/demo_owned_operations_api.sh` as the default demo.
- If the demo is too slow/noisy, show the expected anchors and explain the three lanes: checked OpenAPI contract, Data-Quality Hygiene workflow, disabled worker/outbox proof.
- Say "local contract proof" and "read-only validation next."
- Do not say "production integration," "replaced Gingr," "BI can turn off its database," "agents can contact customers," or "observability is complete."
- Ask for approved read-only docs, exports, sample source snapshots, and BI query inventory before any dual-run or live-action discussion.
