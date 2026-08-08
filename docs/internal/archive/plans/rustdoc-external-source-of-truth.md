# Rustdoc External Source-of-Truth Completion Plan

> **For Hermes:** Execute through the `nva-labor-cost-doc-contracts` Kanban board using the pet-resort docs/code/reviewer profiles. Treat Rustdoc as an external-facing product surface, not merely a compiler lint target.

**Goal:** Make the generated Rustdocs at `https://nva.eman.network/` a reliable, always-code-derived source of truth for the NVA Pet Resorts labor-cost-reduction automation domain.

**Why this matters:** The repository is meant to show and preserve the operating model for reducing labor cost across pet resorts: safer automation packets, deterministic workflow gates, Gingr/source ingestion, service-line operations, manager briefings, data-quality repair, and storage/read-model boundaries. External Rustdocs should explain those contracts directly from source.

**Authority inputs:**
- Source code is the primary source of truth.
- `nva-pet-resorts-ai-context.md` supplies the business/domain frame: 170-resort portfolio, boarding/daycare/grooming/training/retail service lines, Gingr as likely operational system, and labor-cost reduction through automation/process/tooling.
- Existing README and module docs provide orientation but must not drift from code.

## Documentation bar

For every public or rustdoc-visible item in the workspace:

1. Document **what operational concept it represents** in pet-resort/labor-cost terms.
2. Document **why it exists at that layer**: domain contract, app workflow, storage projection, Gingr boundary, runtime adapter, or smoke/CLI entrypoint.
3. Document **authority and safety boundaries**: source facts vs derived facts, draft vs live action, manager/customer/medical review gates, no invented availability or policy overrides.
4. Document **inputs/outputs/evidence** for methods and constructors; avoid generic "runs the step" text.
5. Document enum variants and struct fields with the decision or state each variant/field carries.
6. Document traits and trait methods as contracts workers/adapters must honor.
7. Handle generated APIs that appear in Rustdoc, especially Bon builder/generated typestate surfaces: either hide non-semantic generated internals from public Rustdoc or provide a documented builder-facing surface if the generated surface should remain public.
8. Add module-level examples where they clarify how the source-derived docs should be read.

## Verification gates

Run these before closeout:

```bash
cargo fmt --all --check
scripts/check-rust-modernity.sh
scripts/test.sh
cargo doc --workspace --no-deps
RUSTDOCFLAGS='-D missing_docs' cargo doc --workspace --no-deps
```

The strict missing-docs gate may currently expose `statum` macro-generated artifacts in `app/src/booking_triage.rs`. The board must not wave that away. It should either:

- configure/hide/document generated surfaces so external Rustdoc does not show undocumented public artifacts, or
- record a precise upstream/tooling limitation with a source-level mitigation and a tracked follow-up.

## Kanban execution shape

Use serialized shared-checkout cards unless isolated worktrees are explicitly prepared. Cards should improve docs module-by-module and leave focused verification evidence in comments. The final reviewer card must inspect rendered docs for representative pages, including `app::agents::AgentPromptPacket`, not just rely on `missing_docs`.
