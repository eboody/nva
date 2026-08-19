# NVA clean-slate remediation: Claude Code closeout

## Purpose

Finish this candidate and leave the repository beautiful: a verified commit on
`main`, no stale sprint worktrees or task branches, and only intentional
recoverable evidence outside the repository.

## Starting point

- Working branch: `codebase-quality-remediation-isolated`
- Base commit: `285df2585e2aa908a9b3be4fdce171550e080a2c`
- This is a deliberately dirty, staged-and-unstaged candidate. **Do not run
  `git reset`, `git clean`, or discard broad changes.**
- `.vscode/` is pre-existing untracked editor state. Never add it.
- The user explicitly authorizes a normal commit, local merge into `main`, and
  push to `origin/main` after verification. Do not force-push.

## Design constraints

NVA is a prelaunch clean-slate platform. It has no backward-compatibility
obligation. Prefer removing unsupported surface instead of preserving aliases,
codecs, migrations, historical spellings, or speculative public APIs.

Preserve these properties:

- fail-closed source evidence;
- private issuance and opaque approval/outbox authority;
- validated rehydration;
- sensitive diagnostics;
- semantic module ownership and intelligible paths;
- provider and storage mechanics quarantined at boundaries;
- canonical `/v1` API vocabulary;
- no live provider, payment, customer-message, deployment, or member-facing
  action.

## Current evidence

Passing at handoff:

```sh
python scripts/check_markdown_links.py --repo-root .
python scripts/check_architecture_quality.py --repo-root .
python scripts/check_workspace_quality.py --repo-root .
cargo test -p storage --test approval_outbox_authority_compile_fail --locked
```

Known focused failures are intentionally reproducible:

```sh
uv run --with pytest --with pyyaml pytest -q \
  scripts/tests/test_generate_public_api_inventory.py \
  scripts/tests/test_markdown_contracts.py \
  scripts/tests/test_rustdoc_completeness.py \
  scripts/tests/test_ci_release_contract.py \
  scripts/tests/test_semantic_modeling_audit_manifest.py \
  scripts/tests/test_reported_outcome_vocabulary.py
```

At handoff, four tests fail:

1. Three public-API-inventory tests expect `inventory.md` diagnostic output even
   when a consumerless public API makes the generator return nonzero. Restore
   that ordinary failure output, while keeping `--check` fail-closed and
   non-rewriting for a stale checked artifact.
2. `test_markdown_contracts` expects
   `check_deleted_path_references(root)` in `scripts/check_markdown_links.py`.
   Restore a deterministic helper that rejects active Markdown references to
   paths deleted by the candidate. Treat archive material separately.
3. The Rustdoc smoke now correctly targets `StaffEvaluationPacket`, but two
   expected prose fragments do not match rendered docs. Inspect the rendered
   page and make the expectation or canonical Rustdoc accurate.

Use strict RED-GREEN-REFACTOR for each repair. Do not weaken checks simply to
make them pass.

## Required verification

After focused repairs, run:

```sh
uv run --with pytest --with pyyaml pytest -q scripts/tests
python scripts/check_markdown_links.py --repo-root .
python scripts/check_rustdoc_completeness.py
python scripts/check_architecture_quality.py --repo-root .
python scripts/check_workspace_quality.py --repo-root .
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --all-targets --all-features --release --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked
git diff --check
```

Run database-backed and coverage gates only with their documented prerequisites.
Never invent database credentials or claim unrun gates passed.

## Commit and consolidate

1. Stage every intentional tracked candidate change and this handoff. Exclude
   `.vscode/` and any local editor/cache artifacts.
2. Review `git diff --cached --check`, staged status, and an exact staged diff
   hash before committing.
3. Create one clear conventional commit describing the clean-slate remediation
   and its closeout guide.
4. Perform a fresh independent read-only review of the exact committed tree.
   If it passes, merge/fast-forward locally into `main` and push `main` to
   `origin`. If it fails, fix the actual finding and repeat verification.
5. Verify `main` is clean and its local and remote HEAD agree.

## Worktree/branch hygiene

The repository currently has stale `kanban/demo-*` worktrees. Five contain
uncommitted historical work that has been safely archived outside the repo by
the prior operator. `kanban/demo-final-review` has one unique historical
commit (`81633a5`).

Before deleting any worktree:

1. inspect its diff/commit against finished `main`;
2. preserve any genuinely unique material in a recoverable patch archive;
3. explicitly record whether it was integrated or superseded;
4. remove worktrees first, then their local branches;
5. delete remote task branches only after confirming there is no open PR or
   still-needed material.

The end state is mandatory:

```sh
git worktree list
git branch -vv
git status --short --branch
git ls-remote origin refs/heads/main
```

There must be no lingering sprint worktrees or task branches after closeout.
