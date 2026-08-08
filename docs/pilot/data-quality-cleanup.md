# Pilot workflow: Data Quality Hygiene

Purpose: turn source-system inconsistencies into reviewed internal cleanup work.

What the code does today:

- Builds a source-grounded context packet from local fixture data.
- Classifies affected entities and issue categories.
- Produces internal cleanup action drafts.
- Requires review gates before any system-of-record change.
- Rejects drafts that hide ambiguity or request live provider side effects.
- Captures reviewed outcomes with source refs, issue refs, and labor-minute evidence.

What it does not do yet:

- It does not write to Gingr or any NVA production system.
- It does not use live NVA data.
- It does not claim actual labor savings from NVA operations.

How to demo locally:

```sh
./scripts/smoke_data_quality_hygiene_local_loop.sh
```
