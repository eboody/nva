# Semantic code doctrine

This repository follows semantic fidelity: domain code should model business concepts with named types and legal states instead of passing raw primitives through the core. The final semantic-code-doctrine review approved the current baseline: recently migrated surfaces include `entities::Pet.name: pet::Name`, `entities::LocationPolicyRefs::*: policy::Id`, and `agents::AgentPromptPacket::{policies, output_schema_name}`. The detailed inventory and remaining debt are tracked in `docs/quality/semantic-code-doctrine-inventory.md`.

## Domain-core rules

- Prefer module-qualified semantic types at call sites, such as `pet::Name`, `policy::Id`, and `agent::OutputSchemaName`, when the module context matters.
- Use `nutype` for validated or normalized scalar values.
- Use `bon` for ordinary domain builders where named construction improves readability and safety.
- Use `statum` when phase or readiness changes should alter the legal method surface.
- Keep meaningful failures local with module-local `error.rs` files and `Result<T>` aliases.
- Encode business invariants as semantic enums, typestates, or value objects rather than `String`, `bool`, UUID, or integer fields with hidden meaning.

## Boundary exceptions

Raw strings, IDs, and payload maps are acceptable only at boundaries when they are quarantined and converted before domain behavior depends on them. Examples that are accepted for now include CLI JSON printing, audit payloads, opaque `Other(String)` labels, external tool escape hatches, and construction literals that are immediately converted to validated types.

If code starts branching on one of those payloads, it is no longer just a boundary escape hatch. Promote it to a semantic type, enum, or typed key before adding behavior.

## Quality-pause rule for next features

Future feature work may proceed only if it does not add behavior on raw primitive domain surfaces. If a feature touches known debt or exposes a new raw domain concept:

1. Pause the feature.
2. Write a failing semantic API/integration test for the intended domain shape.
3. Verify RED with a focused test command.
4. Migrate the touched surface to the owning semantic type, enum, value object, or typestate with minimal behavior change.
5. Run the focused test until green.
6. Run the full gates.
7. Resume the feature only after the migrated abstraction is green.

Do not add feature behavior on top of a raw field and promise to clean it up later.

## Accepted remaining debt

The final review accepted the following as non-blocking debt under the quality-pause rule:

- Care and medical profile primitives in `entities.rs`: feeding instructions, allergies, medical conditions, emergency/veterinarian contacts, and medication name/dose/schedule/review fields.
- Temperament observation primitives in `entities.rs`: group-play candidacy, people orientation, and staff notes.
- Reservation/payment primitives: `Deposit.amount_cents` and `HardStop::AgeBelowMinimumWeeks`.
- Workflow event/status-update debt: `WorkflowEvent.event_id` and status-update semantics that are not yet aligned to entity-specific status enums.
- Policy/tool boundary primitives: `VaccineRequirement.source_must_be_licensed_vet` and `AvailabilityRequest.service_notes`.
- Boundary escape hatches: opaque `Other(String)` labels, audit payloads, raw attribution IDs, `ToolResourceId::External`, and `ExternalFailure::Other`.

When a feature touches any item above, migrate that touched surface first.

## Verification commands

Use these gates after semantic migrations or domain-affecting changes:

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -q -p cli -- agents | python -m json.tool >/dev/null
cargo run -q -p cli -- tools | python -m json.tool >/dev/null
```

For documentation-only changes, run any available markdown-safe checks. If the documentation includes changed code snippets or API references, also run `cargo test --workspace`.
