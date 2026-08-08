# Pet Resort semantic-code beautification closeout

Generated: 2026-06-14
Workspace: `/home/eran/code/nva`

## Decision

Status: **under active review; not yet universal-clean**.

This pass turned the broad semantic-code beautification board from a Modum-guided smell inventory into a much cleaner, verified source patch. Modum remains a discovery tool rather than an oracle. A follow-up independent review caught that the live JSON shape stores diagnostics under `report.diagnostics`; the current tree still has 33 Modum diagnostics, so this is not yet the final universal-beauty checkpoint.

## Current verified baseline

- `modum check --format json`: **33 diagnostics** under `report.diagnostics`.
- `./scripts/test.sh`: **pass**.
- `cargo fmt --check`: pass via script and pre-commit gate.
- `cargo test --workspace`: pass via script.
- Rust doctests: pass; no `domain` doctest crate-visibility failure remains.
- Frontend staff-web gates: typecheck, lint, and smoke tests pass via script.
- Kanban board `pet-resort-semantic-beautification`: previous diagnostic/review cards closed, but the closeout was too optimistic because the Modum JSON parser inspected the wrong field.
- Kanban diagnostics: 0 rows.

## Material refactors accepted

### Application workflow surfaces

The application layer now exposes more truthful workflow/tool facets rather than flat families that erased meaning at call sites:

- booking triage rule concepts live under `booking_triage::rule`.
- daily update concepts live under `daily_update::daily_care_update`.
- tool contracts are split into semantic facets such as document intake, OCR, task drafting, scheduling, and payment authorization/refund surfaces.

The review rule was: keep visible approval and draft-only boundaries explicit; do not hide safety semantics behind generic helper names.

### Domain contracts and service ownership

The domain layer now keeps more vocabulary under the concept that owns it:

- daily brief snapshot and capacity vocabulary moved into truthful child facets.
- daycare assignment uses the canonical parent-surface `daycare::assignment::PlaygroupId`; the old `playgroup_id::Id` path was removed rather than preserved as a compatibility alias.
- operations vocabulary is grouped under owners such as `operational`, `pet_resort`, `lodging_offer`, and `service_core`.
- staff/workflow/policy surfaces were tightened where the public path had been doing too little semantic work.

### Storage boundary

The storage boundary no longer exposes a catch-all `service` module. Service-line persistence code moved to `storage::service_line`, preserving the storage/domain distinction while making the module name match its responsibility.

## Alias and re-export review

The pass explicitly avoided these anti-patterns:

- no `pub type OldName = new::Name` compatibility blanket for removed surfaces;
- no `pub use nested::*` blanket to keep old call sites compiling;
- no fake parent export where the leaf name would lie about the domain concept.

Remaining `pub type`/`pub use` occurrences are not automatically defects. Current accepted classes include:

- module-local `Result<T>` aliases paired with semantic `Error` types;
- explicit parent error/result exports;
- deliberate boundary exports where the parent module is the canonical public surface;
- narrow storage compatibility names covered by storage contract tests.

## Fresh verification evidence

Commands run from repo root:

```bash
modum check --format json
./scripts/test.sh
```

Observed outcome:

- Modum diagnostics: 33 under `report.diagnostics`; follow-up cards are required.
- Rust unit/integration/doc tests: pass.
- API health/vaccine workflow contracts: pass.
- Storage/service-line contracts: pass.
- Frontend staff-web typecheck/lint/source-smoke: pass.

## Remaining quality posture

This patch is a strong semantic-cleanliness checkpoint, not the end of product work. The next work has two lanes:

1. finish the remaining Modum/semantic-ownership diagnostics without alias or re-export dodges;
2. proceed into contract-first MVP hardening, especially:

   1. checkout/completion state contracts;
   2. CRM/review-request/retention task contracts;
   3. one executable local/demo E2E smoke path from inquiry through checkout and retention;
   4. idempotency/replay assertions across the whole chain;
   5. log/client/audit redaction checks for the full chain;
   6. explicit rollback/outbound/provider/payment hold controls.

Those are product/contract gaps, not unresolved Modum-lint cleanup.
