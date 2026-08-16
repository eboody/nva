# Pilot workflow: Manager Daily Brief

Purpose: summarize high-friction operating exceptions into reviewed manager work.

What the code does today:

- Builds a local manager brief packet from service-demand facts, checkout exceptions, capacity recommendations, and data-quality issues. Reported retention packets remain evidence only and cannot contribute an action, queue item, task, or draft.
- Filters facts to the requested location and operating day.
- Ranks internal manager actions.
- Carries source evidence and data-quality warnings.
- Blocks live customer messages, provider writes, payment actions, and schedule changes.
- Captures manager feedback and labor-minute outcome evidence.

What it does not do yet:

- It does not connect to live Gingr/NVA systems.
- It does not send messages or update schedules.
- It does not replace manager review.

How to demo locally:

```sh
./scripts/smoke_manager_daily_brief_local_loop.sh
```
