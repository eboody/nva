# Lifecycle rehydration boundaries

Persisted JSON and storage rows are not trusted domain facts until they pass an explicit promotion point. For high-risk lifecycle aggregates, deserialization now routes through raw persisted representations and rejects contradictions before the value can be used as a domain aggregate.

## Enforced on rehydration

- `domain::payment::Deposit` rejects paid or refunded persisted states without a payment reference.
- `domain::payment::Deposit` rejects unpaid, not-required, failed, or manager-waived states that still carry a payment reference.
- `domain::entities::Reservation` rejects non-forward reservation periods where `ends_at <= starts_at`.
- `domain::entities::Reservation` rejects empty pet relationships.
- `domain::entities::Reservation` rejects `DepositRequired` hard stops unless the deposit is in a collectible state.
- `domain::entities::Reservation` rejects terminal cancelled, rejected, or checked-out rows that still carry active hard stops.
- `domain::entities::Message` rejects outbound drafts that claim queued, attempted, or delivered lifecycle states.
- `domain::entities::Message` rejects queued, approved-to-queue, attempted, or delivered messages without approval-gate evidence.
- `domain::entities::Message` rejects inbound messages that claim outbound delivery lifecycle states.
- `domain::entities::Message` rejects outbound-sent directions that still claim draft, approval, queue, suppressed, or cancelled lifecycle states.
- `domain::workflow::Event` rejects event-type and subject-family mismatches for the known workflow event categories.

## Intentionally tolerated legacy states

The following states are still tolerated as migration-compatible review evidence, not as live-action authority:

- `workflow::Subject::External` remains accepted for all workflow event types so historical provider/task objects can be quarantined and reviewed before later promotion to a customer, pet, or reservation subject.
- `reservation::Status::SpecialReview`, `MissingInfo`, `VaccinePending`, and `Waitlisted` may carry hard stops because those states are themselves review/triage states rather than terminal outcomes.
- `payment::DepositStatus::Refunded` is treated like `Paid` for reference requirements because a refund without the original payment/POS reference is not reconcilable.
- Message `Suppressed` and `Cancelled` are allowed on outbound drafts because they are safe terminal draft dispositions and do not imply delivery.

These exceptions are compatibility boundaries for durable imports and local demo storage. They do not authorize provider/PMS writes, customer-visible sends, payments/refunds/discounts, schedule changes, or production side effects.
