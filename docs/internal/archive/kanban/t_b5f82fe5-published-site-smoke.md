# QA: published-site smoke and link checks (t_b5f82fe5)

Date: 2026-06-19
Workspace: `/home/eran/code/nva`

## Scope

Published-site smoke and link checks for the entity-centered docs system. The documented published artifact path is `./scripts/build_public_docs.sh`, which renders Rustdocs and copies `docs/public/index.html` to the Rustdoc root.

Because this shared worktree had many simultaneous kanban workers running `cargo doc`, `cargo test --doc`, and at least one `cargo clean`, I used an isolated Cargo target directory for the representative published-site artifact after first exercising the documented wrapper directly.

Isolated artifact used for smoke/link audit:

- `/tmp/nva-public-docs-smoke-t_b5f82fe5/doc/index.html`

## Commands run

```sh
./scripts/build_public_docs.sh
```

Result: passed. It generated Rustdocs in `target/doc`, copied the landing page, and printed:

```text
public docs landing smoke passed: target/doc/index.html
```

```sh
./scripts/check_docs.sh
```

Result: failed during `cargo test -p domain --doc` with missing dependency artifacts such as `target/debug/deps/libserde-b5cdaa0e8bc438ce.rlib`, `libbon-e2efecd79d18b484.rlib`, `libnutype-d43e857bdbf298e8.rlib`, `libuuid-79ccbad197044c01.rlib`, and `libchrono-efb7184627b415f5.rlib`. Process inspection showed multiple other workers concurrently running `cargo doc`, `cargo test --doc`, and one `cargo clean` in the same shared worktree/target dir, so I treated this as an environment/concurrency limitation rather than a content finding.

```sh
./scripts/check_markdown_links.py
```

Result: failed. The deterministic Markdown gate reported local links to generated Rustdoc pages under `../../target/doc/...` as missing after concurrent target cleanup. This is a real fragility in the gate when run before/after generated docs are unavailable, and was amplified by the concurrent `cargo clean` observed in this worktree.

```sh
OUT=/tmp/nva-public-docs-smoke-t_b5f82fe5
rm -rf "$OUT"
CARGO_TARGET_DIR="$OUT" cargo doc --workspace --no-deps
cp docs/public/index.html "$OUT/doc/index.html"
python - <<'PY'
from pathlib import Path
html = Path('/tmp/nva-public-docs-smoke-t_b5f82fe5/doc/index.html').read_text(encoding='utf-8')
required = ['Safe AI workflows for reducing pet-resort labor cost','Manager Daily Brief','Data-Quality Hygiene','Gingr/source-system integration','domain/','app/','storage/','gingr/','pet_resort_api/','pet_resort_worker/','cli/']
missing=[x for x in required if x not in html]
if missing: raise SystemExit(f'missing required landing fragments: {missing}')
print('/tmp/nva-public-docs-smoke-t_b5f82fe5/doc/index.html')
PY
```

Result: passed. This created a clean generated published-site artifact independent of the shared `target/` race.

```sh
python -m http.server 8765 --directory /tmp/nva-public-docs-smoke-t_b5f82fe5/doc
```

Browser tool could not launch because `/usr/bin/chromium` is not installed in this worker environment, so I used HTTP fetches against the local server for the smoke checks.

## Landing/navigation smoke

Checked landing page text and representative Rustdoc navigation through the local server at `http://127.0.0.1:8765/`.

Landing sections found:

- `Safe AI workflows for reducing pet-resort labor cost`
- `Browse the pet-resort entities before the crate names`
- `Workflow map`
- `Safety boundary`
- `Rustdocs as code-derived evidence`

Representative pages fetched successfully with HTTP 200:

- `/`
- `/app/manager_daily_brief/`
- `/app/data_quality_hygiene/`
- `/domain/boarding/`
- `/domain/reservation/`
- `/domain/policy/`
- `/domain/workflow/`
- `/gingr/endpoint/`
- `/gingr/mapping/`
- `/storage/operations/`

These cover the landing page, workflow pages, service-line entity pages, review/safety gates, Gingr/provider boundary, and storage/outcome evidence.

## Static link audit

A scripted audit parsed `href` values from `/tmp/nva-public-docs-smoke-t_b5f82fe5/doc/index.html`.

Results:

- Local relative links: 70 ok, 0 failures.
- External landing links: 41 checked.

External links returning HTTP 200:

- `https://github.com/eboody/nva/blob/main/nva-pet-resorts-ai-context.md`
- `https://github.com/eboody/nva/blob/main/docs/design/labor-cost-reduction-crosswalk.md`
- `https://github.com/eboody/nva/blob/main/docs/audits/2026-06-18-labor-cost-platform-readiness.md`
- `https://github.com/eboody/nva/blob/main/docs/design/manager-daily-brief-measurable-labor-loop.md`
- `https://github.com/eboody/nva/blob/main/docs/ops/manager-daily-brief-local-smoke.md`
- `https://github.com/eboody/nva/blob/main/docs/design/data-quality-hygiene-labor-loop.md`
- `https://github.com/eboody/nva/blob/main/docs/ops/data-quality-hygiene-local-smoke.md`
- `https://github.com/eboody/nva/blob/main/docs/integrations/gingr/source-inventory.md`
- `https://github.com/eboody/nva/blob/main/docs/architecture/agent-app-infrastructure.md`
- `https://github.com/eboody/nva/blob/main/docs/architecture/agent-app-infrastructure.md#trust-boundary-and-threat-model`
- `https://github.com/eboody/nva/blob/main/docs/security/pet-resort-security-audit.md`

External links returning HTTP 404 on GitHub `main`:

- `docs/design/entity-index.md`
- `docs/design/entity-atlas-relationships.md`
- `docs/design/entity-atlas-petsuites-core-entities.md`
- `docs/design/entity-atlas-workflow-packets-agents.md`
- `docs/glossary-architecture-terms.md` and anchors `#domain`, `#app`, `#storage`, `#dto`, `#source-ref-domainsourcerecordref`, `#provenance-domainsourceprovenance`
- `docs/glossary-source-data-terms.md` and anchors `#domainsourcesystemgingr-gingr`, `#provider-record`
- `docs/glossary-workflow-state-terms.md` and anchors `#workflow-packet`, `#draft`, `#review-gate`, `#blocked-action`, `#outcome-capture`
- `docs/glossary.md`
- `docs/workflows/operator/README.md`
- `docs/workflows/operator/manager-daily-brief.md`
- `docs/workflows/operator/booking-triage.md`
- `docs/workflows/operator/data-quality-hygiene.md`
- `docs/workflows/operator/checkout-completion.md`
- `docs/workflows/operator/grooming-rebooking-retention.md`
- `docs/workflows/operator/daily-updates-pawgress-drafts.md`
- `docs/workflows/operator/regional-labor-exceptions.md`
- `docs/design/entity-atlas-audience-paths.md`

## Findings

1. Broken published-site external links: the landing page points to many GitHub `main` URLs for new entity atlas, glossary, and operator workflow docs that are present locally in this worktree but not published on GitHub `main`; GitHub returns 404. If the published docs are meant to be consumed before those files are merged/published, these are broken links.

2. No broken local relative landing links in the isolated generated Rustdoc artifact. Representative Rustdoc entity/workflow/provider/storage pages served successfully.

3. The repo-local Markdown gate is sensitive to generated `target/doc` availability. It currently reports generated Rustdoc links in Markdown as missing if `target/doc` has not been generated or was removed by another worker. In this shared worktree, concurrent `cargo clean` made that limitation visible.

4. Browser visual/console QA was not possible in this worker because Chromium is missing at `/usr/bin/chromium`. HTTP and static HTML checks were used instead.
