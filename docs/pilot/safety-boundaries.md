# Pilot safety boundaries

This repository should be presented as a local, review-gated pilot proof with fixture-only inputs and disabled live side effects.

Current hard boundaries in code and tests:

- Source/provider systems are evidence, not automatic authority.
- Agent outputs are drafts, rankings, summaries, and internal actions.
- Customer-visible messaging is blocked until human review.
- Provider/PMS writes are blocked until explicit approval and system-of-record review.
- Payment/refund/discount actions are blocked.
- Schedule changes are blocked.
- Raw provider payload passthrough is avoided in public API contracts.
- Secrets and sensitive provider parameters are redacted in debug/display surfaces.
- Serialized outcome records and source evidence remain nonclaimable; no labor-savings claim issuer exists.

Recommended external framing:

> This is a local proof of the workflow and safety layer I would want before connecting to production systems. It uses fixture/local data only and keeps live side effects disabled.
