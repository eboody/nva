# Domain Quality Gate

When a new pet-resort feature or tweak comes up, do not bolt it onto weak abstractions just to preserve momentum. If the change exposes a missing domain concept, pause the main task and refactor the model first.

## Pause and refactor first when you see

- A new raw `String`, UUID, integer, or boolean carrying business meaning across module boundaries.
- Duplicated trimming, validation, status parsing, policy checks, or workflow-readiness checks.
- Another scattered helper function where a named domain module/type should exist.
- Runtime readiness/status branching where illegal operations should be impossible through typestate.
- Positional constructors or struct literals that are becoming fragile at call sites.
- Broad imports that hide context better expressed as `module::Type`.

## Preferred setup move

1. Write a failing integration/API test for the desired domain shape.
2. Introduce or repair the domain module boundary.
3. Use the standard tools:
   - `nutype` for validated/sanitized semantic scalars.
   - `bon` for ordinary domain builders.
   - `statum` for staged workflows where later methods should not exist until earlier phases are complete.
4. Migrate the existing behavior onto the stronger abstraction.
5. Run the quality gates.
6. Resume the original feature only after the abstraction is green.

## Required gates

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If a CLI/operator surface emits JSON, also run it and parse the output, for example:

```bash
cargo run -q -p cli -- agents >/tmp/pet-resort-agents.json
python -m json.tool /tmp/pet-resort-agents.json >/dev/null
```

## Living example

See `crates/domain/tests/domain_quality_patterns.rs` for the current test-driven examples of:

- `pet::Name` via `nutype`;
- `agent::Spec` via `bon`;
- `booking_triage::Request<booking_triage::Intake>` progression via `statum`.
