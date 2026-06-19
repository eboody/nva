# Liveness and no-race execution protocol

Generated: 2026-06-19
Task: `t_07a0e7c3` — write liveness and no-race execution protocol
Scope: orchestrator/operator runbook for old/new NVA docs board reconciliation. This protocol is intentionally conservative: it tells an orchestrator when it may unpark exactly one next card and when it must wait.

## Rule zero

Do not unblock, reclaim, archive, dispatch, or start any shared `dir:/home/eran/code/nva` writer until you have proved all three facts:

1. Which board you are inspecting.
2. Which PIDs are alive and what tasks they belong to.
3. Which dirty files are already owned by running/done task output.

If any of those facts is ambiguous, stop and wait or ask a human. Never use global dispatch as a shortcut.

## Environment isolation before inspecting boards

When inspecting a board other than the card you are currently running, unset inherited board selectors so the CLI does not silently read the wrong database:

```bash
cd /home/eran/code/nva
unset HERMES_KANBAN_DB HERMES_KANBAN_BOARD HERMES_KANBAN_TASK HERMES_KANBAN_WORKSPACE
hermes kanban boards list --json
hermes kanban --board <board-slug> stats --json
hermes kanban --board <board-slug> list --status running --json
```

For bulk read-only inspection, prefer SQLite `mode=ro` and never update rows directly:

```bash
python - <<'PY'
import glob, os, sqlite3
for db in sorted(glob.glob('/home/eran/.hermes/kanban/boards/*/kanban.db')):
    board = os.path.basename(os.path.dirname(db))
    con = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
    rows = con.execute("select status, count(*) from tasks group by status order by status").fetchall()
    running = con.execute("""
        select id, title, assignee, worker_pid, workspace_path, last_heartbeat_at
        from tasks where status='running' order by started_at
    """).fetchall()
    print('\n##', board, rows)
    for row in running:
        print('RUNNING', row)
    con.close()
PY
```

Stop conditions:
- The expected board slug is missing or empty when prior artifacts say it should contain tasks.
- The CLI and read-only SQLite disagree on running/open counts.
- `PRAGMA integrity_check` reports anything other than a narrow index-only problem. Do not repair task rows; take a backup and ask a human.

## Current board snapshot command

Use this exact shape before every unblock/archive/reclaim decision:

```bash
python - <<'PY'
import glob, os, sqlite3, time
for db in sorted(glob.glob('/home/eran/.hermes/kanban/boards/*/kanban.db')):
    board = os.path.basename(os.path.dirname(db))
    con = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
    con.row_factory = sqlite3.Row
    counts = con.execute("select status, count(*) n from tasks group by status order by status").fetchall()
    print(f"{board}: " + ', '.join(f"{r['status']}={r['n']}" for r in counts))
    for r in con.execute("""
        select id, title, assignee, worker_pid, workspace_path, last_heartbeat_at, started_at
        from tasks where status in ('running','ready','todo','blocked')
        order by case status when 'running' then 0 when 'ready' then 1 when 'todo' then 2 else 3 end, started_at, created_at
    """):
        print(f"  {r['status']} {r['id']} pid={r['worker_pid']} hb={r['last_heartbeat_at']} assignee={r['assignee']} ws={r['workspace_path']} title={r['title']}")
    con.close()
PY
```

Record the output in the orchestrator card comment before taking action if the action mutates board state.

## Git status and dirty-file ownership

Run this before unblocking any card that can write `/home/eran/code/nva`:

```bash
cd /home/eran/code/nva
git status --short
git diff --name-only
git ls-files --others --exclude-standard
```

Then map dirty files to task ownership:

1. Read parent and sibling handoffs for every running/done card on the relevant boards.
2. Look for `metadata.changed_files`, `metadata.files/surfaces`, artifact paths, and caveats about pre-existing dirty files.
3. Treat untracked docs artifacts as owned by the card that first named them in a completion handoff, even if Git cannot prove ownership.
4. Treat broad source files (`README.md`, `app/README.md`, `domain/src/**`, `docs/public/index.html`, `scripts/check_docs.sh`) as high-conflict until every running writer touching the same area has completed.

Useful read-only query for done output and dirty-file clues:

```bash
python - <<'PY'
import json, os, sqlite3
board = '<board-slug>'
db = f'/home/eran/.hermes/kanban/boards/{board}/kanban.db'
con = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
con.row_factory = sqlite3.Row
for r in con.execute("""
    select t.id, t.title, t.assignee, t.status, tr.summary, tr.metadata
    from tasks t left join task_runs tr on tr.id = t.current_run_id
    where t.status in ('done','running')
    order by coalesce(t.completed_at, t.started_at, t.created_at)
"""):
    print('\n', r['status'], r['id'], r['title'])
    print('assignee:', r['assignee'])
    print('summary:', (r['summary'] or '')[:500])
    if r['metadata']:
        print('metadata:', r['metadata'][:1000])
con.close()
PY
```

Stop conditions:
- Dirty files include a path the candidate card will modify and there is an alive running PID for another card on the same path family.
- Dirty file ownership cannot be traced to a completed/running handoff.
- The candidate card's acceptance requires global formatting, doc generation, broad search/replace, or whole-repo validation while other writers are alive in the shared dir.

## PID liveness and stale-worker classification

For every running task, verify the stored worker PID:

```bash
# Replace with the PIDs from the board snapshot.
ps -o pid=,ppid=,stat=,etime=,cmd= -p <pid1>,<pid2>,<pid3>
```

Interpretation:

| Observation | Classification | Action |
| --- | --- | --- |
| PID appears with state not containing `Z`, command is `hermes ... work kanban task <task-id>` | Active or possibly active worker | Do not reclaim. Wait unless the card has an old heartbeat and a human authorizes deeper inspection. |
| PID appears as `Z`/defunct | Defunct worker | Comment with `ps` evidence, then use the board tool/CLI to reclaim or mark failed according to board policy. |
| PID missing from `ps`, task still `running` | Stale worker | Comment with board snapshot and `ps` miss before reclaim/archive. |
| PID command is unrelated to Hermes or task id does not match | PID reused / corrupt liveness | Stop and ask a human; do not reclaim based on PID alone. |
| PID alive but no heartbeat | Still active unless other evidence says otherwise | Wait. Heartbeat is useful but absence is not proof of death for short docs runs. |

Optional deeper check when a PID is alive but suspected idle:

```bash
pwdx <pid> || true
ps -o pid,ppid,stat,etime,pcpu,pmem,cmd -p <pid>
ls -l /proc/<pid>/fd 2>/dev/null | sed -n '1,40p'
```

Do not kill an alive Hermes worker just because it is sleeping. Most active agents spend time waiting on API calls and show `S`/`Ssl`.

## Active writer vs safe waiter decision

A task is an active writer if any of these are true:

- Status is `running`, PID is alive, and `workspace_path` is `/home/eran/code/nva`.
- The task title/body/handoff names source, docs, README, Rustdoc, scripts, public landing, generated docs, or QA report surfaces.
- The task is a broad QA/final gate that may run formatters, doc builds, link checks, or freshness checks.

A task is safe to ignore for write serialization only if all are true:

- It is `done`, `archived`, or `blocked` with no live PID.
- Its output artifact is already captured in the handoff or a repo file.
- The candidate next card does not rewrite that same artifact or invalidate the done-card evidence.

If there is any active writer in `/home/eran/code/nva`, the default action is wait. Unpark exactly one next card only when it is read-only or scoped to non-overlapping files, and write that scope into a comment first.

## Unparking exactly one next card

Use this sequence; do not batch-unblock:

1. Choose one candidate card from the current board's `ready`, `todo`, or `blocked` set.
2. Prove all parents are done and their outputs are preserved.
3. Prove no alive running PID is writing the same path family.
4. Prove the working tree dirty files are either unrelated or already owned by completed/running cards.
5. Add a card comment with:
   - board slug and task id,
   - current `git status --short` summary,
   - running PID table,
   - why this one card is non-overlapping,
   - exact stop condition for the spawned worker.
6. Unblock/promote only that one card.
7. Do not run global dispatch. Let the board's normal dispatcher pick it up, or dispatch only the named board/task if a human explicitly asks.

Comment template:

```text
no-race preflight for <task-id> on <board>:
- inspected with HERMES_KANBAN_DB/HERMES_KANBAN_BOARD unset
- running shared-dir workers: <task list with pid/state>
- dirty files: <summary and owner handoffs>
- decision: unpark exactly this card because <non-overlap/read-only reason>
- stop condition: if it needs to edit <paths> or run broad doc/cargo gates while listed PIDs are alive, block instead of writing
```

Stop conditions:
- More than one candidate seems eligible. Pick the safest one or wait; do not unpark a batch.
- The candidate card lacks a narrow path scope.
- The candidate would need to consume output from a still-running predecessor.

## Reclaim/archive protocol

Never reclaim or archive silently.

Before reclaiming a stale running card:

1. Confirm the PID is missing, defunct, or unrelated with `ps`.
2. Confirm no child process remains in the same process tree if the parent is gone:

```bash
pgrep -a -P <pid> || true
```

3. Capture the board snapshot and `ps` result in a comment on the card.
4. Preserve any partial artifacts already written in the repo; do not delete dirty files to make the board clean.
5. Reclaim only the stale card, not the whole board.

Before archiving a duplicate/misrouted card:

1. Read the card, parent handoffs, comments, and any done-card output it depends on.
2. Confirm successor board/card owns the acceptance.
3. Comment with the successor board/card or artifact that preserves the output.
4. Archive only after the comment is durable.

Archive comment template:

```text
archive preflight: duplicate/misrouted after successor migration.
- old card output preserved in: <artifact or done-card handoff>
- successor owner: <board>/<task or final QA artifact>
- no live PID for this card; current ps/board snapshot checked at <time>
- no dirty files discarded
```

## Done-card output preservation

Done-card summaries and metadata are durable evidence. Do not discard them because the old board is being closed.

For every old board you close or archive around, first harvest:

- artifact paths in `metadata.artifact`, `metadata.changed_files`, or summary text,
- QA caveats and failed checks,
- exact commands that succeeded or failed,
- public URLs and freshness timestamps,
- source/Rustdoc mappings that successor QA can cite.

If the successor board does not yet contain that evidence, add a comment to the successor final QA/synthesis card before archiving the old card. If no successor card exists, create a narrowly scoped follow-up on the right successor board rather than unarchiving the old workstream.

## Shared-dir serialization policy

All current NVA docs boards use `dir:/home/eran/code/nva` unless a card explicitly creates a per-card worktree. Therefore:

- Only one mutating writer should be intentionally unparked at a time for the shared dir.
- Read-only QA can run concurrently only when it does not run broad generated-output commands that dirty the tree.
- Broad commands such as `cargo fmt`, `cargo test --doc`, `./scripts/check_docs.sh`, public-docs build scripts, mass link rewriting, or whole-repo smell searches count as shared-dir mutations/invalidations for scheduling purposes.
- If parallel mutation is required, create explicit per-card git worktrees and make the card body name the worktree path and merge plan.

Per-card worktree pattern:

```bash
cd /home/eran/code/nva
git worktree add /home/eran/code/nva-worktrees/<task-id> -b wt/<task-id>
# Update the Kanban card/workspace to the worktree path before dispatching the worker.
```

Stop if no merge owner is named. Worktrees avoid file races but do not solve semantic merge conflicts.

## Safe wait protocol

If any stop condition triggers, do not unblock anything. Leave a short orchestrator comment instead:

```text
safe-wait: no card unparked.
Reason: active shared-dir workers still alive: <task ids / pid states>. Candidate <task-id> overlaps <paths or acceptance gate>. Recheck after those workers complete or after a human approves a per-card worktree.
```

Then wait for the running workers to complete and re-run the full preflight. Do not pollute the board with repeated wait comments; one comment per decision point is enough.

## Current observed state during this run

Read-only inspection during `t_07a0e7c3` found many alive Hermes workers still running against `/home/eran/code/nva`, including entity safety overlays, final QA, relationship crosswalk, Rustdoc cleanup, glossary, landing, and this reconciliation task. The safe default at generation time is therefore: do not unpark old-board implementation cards into the shared directory; wait for active successor workers or move a single narrow card to an explicit worktree.

This observation is not a permanent fact. Always rerun the commands above before acting.

## Minimal decision checklist

Use this as the final go/no-go gate:

- [ ] I unset `HERMES_KANBAN_DB` and `HERMES_KANBAN_BOARD` before inspecting other boards.
- [ ] I captured current board stats for the target board and relevant successor boards.
- [ ] I listed all `running` tasks with `worker_pid`, `workspace_path`, and title.
- [ ] I verified each running PID with `ps` and classified it as active, stale, defunct, or ambiguous.
- [ ] I ran `git status --short` in `/home/eran/code/nva`.
- [ ] I mapped dirty files to running/done card ownership or stopped because ownership is unclear.
- [ ] I confirmed the candidate card is the only card to unpark.
- [ ] I confirmed no active writer overlaps the candidate's file family or broad validation gate.
- [ ] I commented before any reclaim/archive/unblock.
- [ ] I preserved done-card output in successor comments/artifacts before archiving duplicates.
- [ ] I avoided global dispatch.

If every checkbox is true, unpark exactly one card. If any checkbox is false, safely wait or create an explicit worktree plan.
