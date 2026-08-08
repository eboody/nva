# Full-chain local smoke harness

This local/demo harness proves the first end-to-end operational chain without enabling live sends or provider/PMS/payment mutations:

`inquiry -> profile -> vaccine docs -> booking triage -> confirmation draft -> check-in/today view -> staff note/daily update draft -> checkout/completion -> follow-up/retention`

Run it with:

```bash
./scripts/full-chain-local-smoke.sh
```

or directly with:

```bash
cargo test -p app --test full_chain_local_smoke -- --nocapture
```

The fixture is `fixtures/smoke/inquiry-received.json`. The harness uses semantic app/domain contracts where they exist and returns a review-gated `FullChainEvidence` packet. It intentionally keeps AI output draft-only, blocks live customer sends, blocks provider/PMS mutations, and blocks payment/refund/discount actions.

Current known seam: retention is still represented by a minimal typed local-smoke contract. Checkout/completion now uses the `app::checkout_completion` workflow packet with source provenance, staff handoff review, explicit completion status, customer-message approval gates, audit-event drafts, and blocks on live sends, provider/PMS mutations, and payment/refund/discount actions. The next implementation slice should promote retention/rebooking draft policy into the same production-facing semantic contract shape.
