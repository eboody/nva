# Evidence, Acceptance, and Authority

## Status

This document defines the pre-data semantic posture for NVA. The actual provider data model, identifiers, cardinalities, lifecycle history, and operational exceptions are not yet available. Current business shapes are therefore hypotheses to test against observed records, while the trust and conversion rules in this document are universal software invariants.

## The trust gradient

NVA distinguishes four kinds of value. Crossing from one kind to the next is a named, fallible semantic operation.

### Observed source evidence

An observed value reports what a source exposed. It preserves source identity, record identity, endpoint or record type, observation time, adapter/schema version, and raw evidence reference where available.

Observed evidence may be incomplete, duplicated, malformed, stale, or contradictory. Its existence does not make it owned truth and does not authorize an action.

### Candidate interpretation

A candidate is an NVA interpretation of observed evidence. It binds the mapping version, assumptions, confidence or uncertainty, and data-quality issues that explain the interpretation.

A candidate may be reviewed, reconciled, superseded, or rejected. It must not inhabit a type whose name claims acceptance or executable authority.

### Accepted owned fact

An accepted fact is a candidate or owned observation promoted through the domain owner’s acceptance policy. Acceptance evidence binds the exact subject, accepted value/version, reviewer or deterministic policy, decision time, and intended use required by that policy.

Types representing accepted, approved, verified, reviewed, confirmed, eligible, completed, current, or trusted state must satisfy constructor-equivalent rehydration. They either deserialize through checked promotion or intentionally do not implement public deserialization.

### Executable authority

Authority permits one consequential action for an exact subject, target, gate, scope, operation, and evidence set. Historical evidence remains serializable; executable authority is opaque, non-serializable, non-cloneable when one-use semantics matter, and issued only by a trusted application boundary.

Persisted status, historical approval, request-body actor claims, provider roles, and matching strings or enums cannot recreate authority. Authority issuance rechecks trusted actor identity, role/policy, location or tenant scope, target, gate, evidence version, and decision chronology applicable to the action.

## Relationship doctrine under source uncertainty

Relationships whose real cardinality is unknown remain explicit hypotheses. NVA does not prematurely encode assumptions such as one permanent pet owner, one provider ID namespace, or one result per workflow event.

The model may still require proof appropriate to the current action. For example, reservation intake need not claim one immutable pet owner, but it must eventually carry evidence that the booking party is authorized for every pet in the reservation.

Unknown relationships are represented as unresolved, ambiguous, candidate, or review-required states. They are not represented by nil identifiers, silently dropped rows, first-match selection, invented defaults, or unchecked polymorphic identifiers.

## Boundary rules

1. Provider DTOs remain provider-owned and may preserve undocumented fields.
2. Promotion into domain values is explicit, fallible, exhaustive, and versioned.
3. Missing provenance remains missing or blocks promotion; adapters do not invent plausible evidence.
4. Domain-to-storage projection is lossless for every known state.
5. Storage-to-domain promotion is fallible and constructor-equivalent.
6. Unknown stable codes and schema versions follow an explicit compatibility policy; they never silently become current authority.
7. OpenAPI and runtime serde share one canonical semantic owner and are structurally parity-tested.
8. Rust stable codes and SQL constraints are parity-tested.
9. Polymorphic or history-oriented storage shapes do not become executable authority merely because their strings and identifiers match.
10. Side effects remain disabled until durable identity, authority, idempotency, leasing, retry, dead-letter, and transaction behavior are proven.

## Current executable boundary

The pre-data build defines opaque queue, incident-closure, accepted-consent, marketing-use, and outbox capability protocols, but it intentionally exposes no production capability issuer. Generic serde can restore non-executable drafts, observations, historical approvals, and consent claims; it cannot restore queued, sent, delivered, closed, accepted-consent, or marketing-authority states from those historical fields. Incident closure and marketing use require opaque current authority or accepted evidence that ordinary callers cannot construct, deserialize, clone, or derive from historical records.

The CRM-retention application treats `EvidenceReviewStatus::Accepted` and
`ConsentStatus::Granted` as historical source claims only. Those public serializable labels cannot
create a personalized draft or claim a recovered booking. Until an authenticated acceptance path can
bind an exact opportunity to opaque accepted consent authority, the retention path remains internal
and fail-closed.

PostgreSQL outcome admission is part of the same boundary. It takes update-strength locks on the exact approval, review packet, and workflow event; checks the owned workflow/event contract, reviewed action, actor/persona/gate, chronology, location, correlation, and canonical source provenance; and freezes every referenced lineage row after admission. Reciprocal mutation guards and concurrent database tests prevent a writer from changing accepted meaning after or concurrently with outcome insertion.

Provider mapping versions are NVA-owned mapper labels, not claims about a verified Gingr endpoint or
schema. Opaque observed endpoint and schema labels are preserved until authoritative provider
contracts exist. Realtime authority evolves additively as well: the deployed SpacetimeDB `location_scope` shape is preserved byte-for-byte as migration input, while authorization reads only `location_scope_v1`. On the first authenticated operation for an actor, exact identity plus validated single-role state atomically promotes that actor's well-formed legacy locations into v1; sibling, malformed, duplicate, or ambiguous authority fails closed before insertion.

Business-value attribution follows the same boundary. Analytics `ReportedReviewedAction`, CRM booking observations, and site-finance reviewed-action records are serializable evidence candidates only. Their reporting APIs return non-claimable until a future authenticated boundary issues opaque accepted attribution. Persisted enums, recommendation references, audit references, manager-decision rows, and caller-provided booleans cannot manufacture a recovered-booking, revenue, retention, or ROI claim. Lead `ConversionObservation` and `ReviewApprovalEvidence` likewise cannot issue value attribution or queueable contact authority. Permissioned-knowledge `ActorContext`, applicability, and document approval labels are history rather than access authority; `AuthorizedEvidence` has no production issuer and cannot be cloned or serialized, so knowledge requests escalate until authenticated actor and document-approval roots exist.

A production issuer remains deferred until an authenticated actor owner can prove current identity, role, location or tenant scope, and policy version. Adding a public constructor from actor labels, persisted rows, or request DTOs would be a security regression, not integration progress. The system therefore stays fail-closed while provider writes, customer delivery, and other live side effects remain disabled.

## Assumption classification

Every consequential model statement should be identifiable as one of:

- **Universal invariant:** true regardless of provider implementation.
- **Business hypothesis:** plausible but not yet confirmed from actual workflows/data.
- **Source observation:** directly evidenced by a specific source record or contract.
- **Migration assumption:** temporarily required to reconcile incomplete legacy evidence.
- **Owned policy:** an intentional NVA decision rather than copied provider behavior.
- **Unknown:** explicitly unresolved and unable to authorize a consequential action.

## Review test

For every type or API that claims accepted truth or authority, reviewers ask:

1. What exact evidence entered?
2. Who owns interpretation and acceptance?
3. What subject, target, scope, version, and time is the proof bound to?
4. Can serialization, public construction, defaulting, or a string conversion bypass promotion?
5. Can the value be transferred to a sibling subject or operation?
6. Does persistence preserve every meaningful distinction?
7. Does the production consumer actually require this proof, or is the model test-only?

A type whose shape cannot answer these questions does not yet carry the claimed semantic status.
