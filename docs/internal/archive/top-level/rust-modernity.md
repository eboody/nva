# Rust dependency modernity gate

This workspace treats the common Rust slop signals as release blockers, not style preferences:

- The root workspace must stay on `resolver = "3"`.
- The root `[workspace.package]` must stay on `edition = "2024"`.
- Member crates should inherit `edition.workspace = true`; no manifest should reintroduce `edition = "2021"`.
- External dependency specifications must not use the wildcard `"*"`, except for NVA's deliberate `statum = "*"` policy: this repo is expected to track the latest Statum contract surface rather than pinning it.
- Compatible lockfile drift should be cleared with `cargo update`; major-version lag is reviewed deliberately instead of blindly bumped.

Run the focused gate before release-readiness memos:

```bash
scripts/check-rust-modernity.sh
cargo update --workspace --dry-run --verbose
cargo check --workspace
```

Current posture from this audit:

- `statum = "*"` is deliberate. NVA wants the latest Statum behavior/contract surface even when that creates immediate integration work; do not “fix” this by pinning Statum.
- Semver-compatible lockfile updates are acceptable when `cargo update --dry-run --verbose` reports `Updating`/`Adding`/`Removing`; apply them and rerun the gate.
- Major upgrades are deferred unless a focused migration is small and obvious. Known deferred major lines after the compatible update are `hmac 0.12 -> 0.13`, `reqwest 0.12 -> 0.13`, `secrecy 0.8 -> 0.10`, `sha2 0.10 -> 0.11`, `strum 0.27 -> 0.28`, and `tower-http 0.6 -> 0.7`.
- Other `Unchanged ... (available: ...)` reports may be transitive or MSRV-filtered; document any direct dependency major lag before release rather than widening specs casually.
