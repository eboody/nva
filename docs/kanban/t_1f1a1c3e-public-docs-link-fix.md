# BLOCKER fix: published-site external docs links (t_1f1a1c3e)

Date: 2026-06-19
Workspace: `/home/eran/code/nva`

## Change

Updated `docs/public/index.html` so the public Rustdoc landing page no longer points at unpublished GitHub `main` Markdown files for the entity atlas, glossary translation layer, workflow/operator index, contract crosswalk, or audience-path docs.

The landing page now keeps those entry points within already published/generated targets:

- same-page landing sections: `#entity-index`, `#glossary-translation`, `#workflows`, `#audiences`, `#rustdocs`
- generated Rustdoc pages: `domain/`, `domain/workflow/`, `domain/policy/`, `domain/source/`, `app/`, `storage/`, `storage/operations/`, `gingr/`, `gingr/dto/`
- existing GitHub `main` docs that already return HTTP 200: business context, labor crosswalk, readiness memo, manager/data-quality loops, ops smoke docs, Gingr source inventory, architecture/security docs

This fixes the public-launch blocker without requiring unpublished local docs to be merged before the landing page is usable.

## Verification

### Landing source check

```sh
python scripts/check_public_docs_landing.py
```

Result:

```text
public docs landing source check passed
```

### Scripted GitHub-main external-link check

Parsed `docs/public/index.html` hrefs and checked every remaining `https://github.com/eboody/nva/blob/main/...` URL with `urllib.request`.

Result: 11/11 GitHub-main links returned HTTP 200; no GitHub-main 404s remain.

Checked URLs:

- `docs/architecture/agent-app-infrastructure.md`
- `docs/architecture/agent-app-infrastructure.md#trust-boundary-and-threat-model`
- `docs/audits/2026-06-18-labor-cost-platform-readiness.md`
- `docs/design/data-quality-hygiene-labor-loop.md`
- `docs/design/labor-cost-reduction-crosswalk.md`
- `docs/design/manager-daily-brief-measurable-labor-loop.md`
- `docs/integrations/gingr/source-inventory.md`
- `docs/ops/data-quality-hygiene-local-smoke.md`
- `docs/ops/manager-daily-brief-local-smoke.md`
- `docs/security/pet-resort-security-audit.md`
- `nva-pet-resorts-ai-context.md`

### Isolated published Rustdoc landing build/link smoke

```sh
OUT=/tmp/nva-public-docs-t_1f1a1c3e
rm -rf "$OUT"
CARGO_TARGET_DIR="$OUT" cargo doc --workspace --no-deps
cp docs/public/index.html "$OUT/doc/index.html"
# parse landing hrefs; verify fragments and local relative generated Rustdoc targets
```

Result:

```text
isolated public docs landing smoke passed: /tmp/nva-public-docs-t_1f1a1c3e/doc/index.html
local relative links ok: 107 checked
```

Representative generated Rustdoc pages verified present:

- `app/manager_daily_brief/index.html`
- `app/data_quality_hygiene/index.html`
- `domain/boarding/index.html`
- `domain/reservation/index.html`
- `domain/policy/index.html`
- `domain/workflow/index.html`
- `gingr/endpoint/index.html`
- `gingr/mapping/index.html`
- `storage/operations/index.html`

## Caveats

This clears the published-site link blocker for the landing page. It does not claim that the separate P0 non-coder glossary/operator-language card is complete.
