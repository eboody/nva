# Semantic type modernization policy

## Scope and inventory

This inventory covers every transparent `String` tuple newtype found in the current checkout, including domain, app, storage, and Gingr integration crates. The scan found 143 public `pub struct Type(String);` shapes:

- 109 already use `nutype`, including the now-validated source-record join key `source::record::Id`.
- 34 remain manually implemented.
- The largest remaining manual clusters are `domain/src/source.rs` (13), `domain/src/workflow.rs` (11), `domain/src/care.rs` (9), `domain/src/training/mod.rs` (9), `app/src/tools.rs` (17), and `app/src/data_quality_hygiene.rs` (5).

This document is an implementation policy, not a mandate to flatten semantic vocabulary. Keep the semantic module path as part of the public name: for example, `source::record::Id`, `training::package::Id`, and `workflow::task::Body` should stay distinct even when their leaf names are short.

## Decision summary

Use `nutype` as the default for simple string value objects whose invariant is expressible as trim, non-empty, and bounded length. Keep or write explicit constructors when the type crosses a trust boundary that needs semantic errors, custom normalization, non-string input, redacted debugging, or custom serde. Do not use `derive_more` as a blanket modernization dependency for semantic value-object conversion behavior; the accepted narrow exception is documented in [`storage-domain-conversion-audit.md`](storage-domain-conversion-audit.md): mechanical `Display` for non-sensitive provider parameter wrappers and `From`/`Into` only for transparent storage mirrors whose conversion performs no validation, authorization, normalization, or error-context translation.

| Family | Policy | Validation rules | Serde behavior | Display / `AsRef` behavior | Migration risk | Tests required |
| --- | --- | --- | --- | --- | --- | --- |
| Simple domain labels and refs | Prefer `nutype`. | `sanitize(trim)`, `validate(not_empty, len_char_max = domain-specific bound)`. Use existing bounds where present: names 120, phone 40, care notes 400/1000, IDs usually 120. | Derive `Serialize` and `Deserialize` through `nutype` so deserialization also validates. | Derive or expose read-only access only when call sites need it. Prefer `as_ref()`/`AsRef<str>` for internal evidence refs; avoid casual `Display` for sensitive copy. | Low for types already on `nutype`; medium when replacing manual `Deserialize`, because invalid persisted/test fixtures will start failing. | Constructor accepts trimmed valid input; rejects blank; rejects over-length; serde round-trip validates; call-site compile tests for changed accessors. |
| Sensitive care, medical, customer, and staff text | Keep `nutype` for validation, but keep manual redacted `Debug` where required. | Trim, non-empty, bounded; do not validate medical/customer truth beyond envelope constraints. | Transparent validated string serialization is acceptable; deserialization must validate and must not bypass constructor. | No `Display` unless a human-facing renderer explicitly owns disclosure. `AsRef<str>` only inside approved rendering/review paths. Redacted `Debug` remains manual in `domain/src/care.rs`. | Medium: deriving `Debug` accidentally leaks sensitive source text; changing serde can expose fixture defects. | Redacted `Debug` tests; validation tests; serde invalid-input tests; renderer/review tests proving disclosure is explicit. |
| Source provenance IDs and provider observed strings | Keep explicit constructors in `domain/src/source.rs` by default. The narrow accepted exception is `source::record::Id`: it is a source-agnostic reconciliation join key with trim/non-empty/120-char validated serde and existing `as_str()` call-site ergonomics. Consider later `nutype` migrations only after preserving or deliberately replacing the module-local error/call-site semantics. | Trim and non-empty; no arbitrary max until provider/source limits are documented. The `source::record::Id` bound is 120 chars because it is a stable join key, not a provider-native opaque payload/hash. Hash/schema/status values may need domain-specific parsers later. | Current manual derived serde can bypass constructors on deserialize; if touched, either move to `nutype` with validated serde or hand-write `Deserialize` through `try_new`. | Keep `as_str()` for provenance displays, reconciliation joins, and diagnostics. Avoid `Display` except for non-sensitive IDs in adapter logs. | High: source records are durable evidence and old payloads may contain blanks or provider quirks. | Existing source contract tests plus explicit invalid-deserialize tests; provenance builder tests; source-record relationship tests. |
| App workflow packet IDs and rationale strings | Prefer `nutype` for new code and migrate manual workflow IDs opportunistically. | Trim and non-empty; use 120 for IDs, 400 for rationale/summaries unless a workflow already documents a different bound. | Validated serde is preferred because workflow packets are stored and replayed. | `AsRef<str>` is acceptable for packet assembly and tests. Use `Display` only for stable IDs, not rationale/body text. | Medium: packet fixtures and replay data may include blank placeholders currently accepted by derived serde. | Workflow contract tests; packet serde fixtures; invalid blank ID/rationale rejection; downstream storage/API mapping tests. |
| Boundary DTO/provider adapter wrappers | Keep explicit manual wrappers when they normalize non-string input, redact diagnostics, or intentionally model provider wire quirks. | Boundary-specific: Gingr `EntityId` accepts non-empty string or integer JSON; `Subdomain` rejects URL/host suffix; `SensitiveLookup` must preserve redaction behavior. | DTO wrappers may use provider-shaped serde, but promotion into domain/app types must validate. | Manual `Display` is allowed for non-sensitive normalized IDs; sensitive lookup values must avoid display/debug leaks. | High: adapters must preserve wire compatibility and redaction guarantees. | Provider fixture tests, parse/normalize tests, redaction/debug tests, and mapping tests that prove promotion into domain types. |
| Storage/read-model wrappers | Keep explicit constructors unless the storage type is just a domain mirror. | Trim and non-empty; preserve DB compatibility and source-of-truth semantics. | Deserialization must not admit values the constructor rejects. | `as_str()` for SQL bindings and read-model comparisons; `TryFrom<String>` acceptable where storage row mapping uses it. | Medium/high: database rows may contain legacy blanks. | Storage row mapping tests, invalid-row tests, migration/backfill checks before enforcing validated deserialize in production. |
| Hashes, SKUs, payment refs, document digests | Keep explicit constructors until richer syntax validation is implemented. | Current minimum is trim/non-empty; target is domain-specific grammar: SHA-256 digest hex or `sha256:` prefix, SKU allowed character set, payment/POS reference envelope. | Prefer `Serialize`; add validated `Deserialize` only when fixtures/backfill are ready. | `as_str()`/`into_inner()` for adapters; `Display` only for non-sensitive refs that appear in audit messages. | Medium: tighter grammar can break fixtures and imported provider data. | Valid/invalid grammar tests; serde tests; adapter/storage tests; audit-display tests. |

## `nutype` policy

Use `nutype` when all of these are true:

1. The representation is exactly `String`.
2. Construction only needs local string hygiene: trim, non-empty, length bound, or a simple regex added later.
3. Error granularity can be the generated validation error, or callers already map constructor failures at a higher semantic boundary.
4. Serde should be transparent and should validate on deserialization.
5. The type does not need custom `Debug`, non-string input normalization, or provider-specific parse logic.

The standard derive set for ordinary non-sensitive domain strings is:

```rust
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct Name(String);
```

Adjust the length bound to the domain meaning. Do not copy `120` everywhere by habit: names and IDs often fit 120, phone numbers use shorter bounds, customer-facing text and review notes need 400-1000, and storage/object references may need their own bound.

## Explicit constructor policy

Keep or add explicit constructors when the type owns semantic behavior not captured by `nutype` attributes:

- It has a module-local error enum that callers use for precise workflow stops, such as `source::Error` or `data_quality_hygiene::Error`.
- It normalizes heterogeneous provider input, such as `integrations/gingr/src/webhook.rs` `EntityId` accepting JSON strings and integers.
- It protects secrets/PII in diagnostics, such as `SensitiveLookup` and care values with redacted `Debug`.
- It needs grammar validation beyond trim/non-empty, such as SHA-256 digests, SKUs, payment references, provider subdomains, or payload hashes.
- It is boundary code where provider wire compatibility is the invariant.

If a manual type derives `Deserialize`, treat that as a migration smell. Serde derives can construct private-field tuple structs without calling `try_new`. When touching those types, either migrate to `nutype` validated serde or hand-write `Deserialize` to call the constructor.

## `derive_more` policy

Use `derive_more` only for narrow, mechanical conversions or displays that have already proven they do not hide semantic promotion. The accepted scope for this pass is:

- `derive_more::Display` on non-sensitive provider numeric/string parameter wrappers where the output is the exact provider wire parameter already exposed by tests.
- `derive_more::From` / `derive_more::Into` on `#[serde(transparent)]` storage mirror wrappers where the conversion performs no validation, storage-code mapping, authorization, normalization, or error-context translation.

Still disallowed:

- deriving conversions that let raw strings bypass named constructors;
- replacing fallible storage/domain/provider promotion with generated `From` / `Into`;
- deriving `Display` for sensitive care, medical, customer-message, rationale, note, body, or lookup text;
- adding `derive_more` to a crate just to shorten a few semantic impls without tests proving the generated behavior.

Allowed alternatives remain:

- `nutype` derives for ordinary validated string wrappers;
- explicit `fmt::Display` for non-sensitive IDs that are meant to appear in audit/reconciliation strings;
- explicit `AsRef<str>` or `as_str()` when call sites need string slices and the boundary is clear;
- explicit `From`/`TryFrom` when the conversion is mechanical and local, or when fallible conversion must preserve typed error context.

Reconsider broader `derive_more` use only if a later code-change task introduces many identical, non-sensitive `Display`, `AsRef`, or `From` impls and can verify generated behavior with compile, serde, and boundary tests.

## Serde behavior rules

- Domain and app value objects that deserialize from stored packets should validate on deserialize.
- Boundary DTOs may deserialize provider-shaped raw values, but must promote into validated domain/app types before crossing into workflow logic.
- Sensitive values may serialize as strings when storage/replay requires it, but diagnostics must not use serde output as debug/log output.
- If a manual type keeps `Serialize` only, do not add `Deserialize` without an invalid-input test.
- If migrating manual `Deserialize` to `nutype`, run fixture and replay tests because previously accepted invalid values may fail.

## Display and string access rules

- Prefer `as_str()` for explicit local access on manual types whose module owns the invariant.
- Prefer `AsRef<str>` for simple value objects only when it improves generic ergonomics without hiding a trust boundary.
- Avoid blanket `Deref<Target = str>` in domain types; it makes semantic promotion too implicit at call sites.
- Avoid `Display` for sensitive care, medical, customer-message, rationale, note, and body text unless the display path is explicitly a human-facing renderer with safety gates.
- Allow `Display` for stable non-sensitive IDs such as webhook `EntityId` when tests prove no secrets leak.

## Migration order

1. Leave already-good `nutype` families alone except to add missing tests.
2. Fix manual types that derive `Deserialize` and only do trim/non-empty validation: `app/src/data_quality_hygiene.rs`, `domain/src/analytics.rs`, selected `domain/src/source.rs` refs, and `storage/src/operations.rs` after fixture review.
3. Preserve manual boundary wrappers in Gingr integration code until adapter tests prove equivalent behavior.
4. Preserve manual digest/SKU/payment/reference types until grammar-specific validators are written.
5. Add `Display`/`AsRef` only from demonstrated call-site needs; do not blanket-derive access traits during migration.

## Detailed inventory

| File | Count | Types |
| --- | ---: | --- |
| `app/src/booking_triage.rs` | 5 | Reservation@89 (nutype), PolicySnapshot@106 (nutype), EvidenceRef@237 (nutype), RecommendationText@254 (nutype), CustomerMessageDraft@271 (nutype) |
| `app/src/checkout_completion.rs` | 1 | CareSummary@45 (nutype) |
| `app/src/crm_retention.rs` | 1 | EvidenceSummary@23 (nutype) |
| `app/src/daily_update.rs` | 6 | LanguageTag@324 (nutype), ToneLabel@341 (nutype), RedactionProfile@358 (nutype), ReviewReason@375 (nutype), FlagMessage@392 (nutype), FactSummary@409 (nutype) |
| `app/src/data_quality_hygiene.rs` | 5 | IssueRef@24 (manual), ActionId@40 (manual), ContextPacketId@56 (manual), CorrelationId@72 (manual), ActionRationale@88 (manual) |
| `app/src/local_smoke.rs` | 3 | SourceEventKey@48 (manual), ReviewEvidenceRef@149 (manual), ReservationLabel@181 (manual) |
| `app/src/manager_daily_brief.rs` | 3 | BriefSummary@97 (nutype), ActionId@114 (nutype), ActionRationale@131 (nutype) |
| `app/src/tools.rs` | 17 | ServiceNotes@170 (nutype), CapacitySnapshotId@238 (nutype), Id@292 (nutype), AccountId@399 (nutype), ExternalRecordId@416 (nutype), IdempotencyKey@486 (nutype), Id@565 (nutype), Id@651 (nutype), Body@779 (nutype), Ref@874 (nutype), Text@936 (nutype), CameraId@1015 (nutype), Ref@1032 (nutype), Id@1105 (nutype), Name@1160 (nutype), Id@1177 (nutype), QueueName@1215 (nutype) |
| `domain/src/agent.rs` | 6 | Name@22 (nutype), Purpose@39 (nutype), ToolName@56 (nutype), ForbiddenAction@73 (nutype), PolicyInstruction@90 (nutype), OutputSchemaName@107 (nutype) |
| `domain/src/analytics.rs` | 3 | ProjectionVersion@16 (manual), Id@38 (manual), Id@197 (manual) |
| `domain/src/care.rs` | 9 | FeedingInstruction@36 (nutype), AllergyName@44 (nutype), MedicalConditionName@52 (nutype), MedicalNote@60 (nutype), ContactName@68 (nutype), MedicationName@76 (nutype), MedicationDose@84 (nutype), MedicationSchedule@92 (nutype), ReviewReason@100 (nutype) |
| `domain/src/customer.rs` | 3 | Name@31 (nutype), Email@52 (nutype), Phone@73 (nutype) |
| `domain/src/daily_brief.rs` | 1 | Id@64 (nutype) |
| `domain/src/daycare/assignment.rs` | 1 | Id@47 (nutype) |
| `domain/src/document.rs` | 6 | FileName@132 (nutype), MimeType@150 (nutype), Sha256Digest@190 (manual), StorageBucket@246 (nutype), StorageKey@264 (nutype), StorageVersion@282 (nutype) |
| `domain/src/entities.rs` | 6 | StaffId@123 (nutype), ManagerId@141 (nutype), Body@518 (nutype), ActionLabel@1014 (nutype), MetadataKey@1032 (nutype), MetadataValue@1050 (nutype) |
| `domain/src/grooming/mod.rs` | 1 | StyleNote@517 (nutype) |
| `domain/src/incident.rs` | 1 | Summary@98 (nutype) |
| `domain/src/lead.rs` | 2 | SourceName@31 (nutype), CampaignName@49 (nutype) |
| `domain/src/location.rs` | 2 | Name@30 (nutype), Timezone@52 (nutype) |
| `domain/src/message.rs` | 1 | BodyRef@77 (nutype) |
| `domain/src/operations.rs` | 3 | MetricName@39 (nutype), Observation@131 (nutype), Recommendation@153 (nutype) |
| `domain/src/payment/mod.rs` | 1 | Reference@34 (manual) |
| `domain/src/pet.rs` | 1 | Name@30 (nutype) |
| `domain/src/policy.rs` | 4 | Id@30 (nutype), VaccineName@48 (nutype), WorkflowName@66 (nutype), Rationale@93 (nutype) |
| `domain/src/portal.rs` | 1 | CustomerId@27 (nutype) |
| `domain/src/reputation.rs` | 2 | PlatformName@31 (nutype), Id@49 (nutype) |
| `domain/src/reservation/mod.rs` | 1 | AddOnLabel@97 (manual) |
| `domain/src/retail/product.rs` | 2 | Sku@24 (manual), Name@79 (nutype) |
| `domain/src/retail/recommendation.rs` | 2 | Text@33 (nutype), SafeCopy@206 (nutype) |
| `domain/src/source.rs` | 14 | Endpoint@81 (manual), ExtractionBatchId@97 (manual), RequestScope@113 (manual), SchemaVersion@129 (manual), PayloadHash@145 (manual), RawPayloadRef@161 (manual), ObservedStatus@177 (manual), Id@199 (nutype), Endpoint@698 (manual), ProviderRecordId@720 (manual), ExtractionBatchId@799 (manual), RequestScope@822 (manual), ProviderSchemaVersion@845 (manual), ProviderStatus@868 (manual) |
| `domain/src/staff.rs` | 1 | Evidence@38 (nutype) |
| `domain/src/temperament.rs` | 2 | StaffNote@34 (nutype), BehaviorObservationLabel@42 (nutype) |
| `domain/src/training/mod.rs` | 9 | Id@131 (nutype), Id@191 (nutype), SessionId@312 (nutype), SessionRef@329 (nutype), ProgressReportId@346 (nutype), EvidenceId@363 (nutype), OutcomeDocumentationId@380 (nutype), ProgressNote@397 (nutype), Id@1017 (nutype) |
| `domain/src/workflow.rs` | 11 | Summary@49 (nutype), RiskFlag@67 (nutype), VerificationNote@85 (nutype), ReviewReason@103 (nutype), Provider@127 (nutype), Id@145 (nutype), Title@170 (nutype), Body@188 (nutype), Channel@212 (nutype), Body@229 (nutype), Reason@258 (nutype) |
| `integrations/gingr/src/config.rs` | 1 | Subdomain@28 (manual) |
| `integrations/gingr/src/endpoint/owners_animals.rs` | 2 | SensitiveLookup@128 (manual), Name@289 (manual) |
| `integrations/gingr/src/response.rs` | 1 | Email@113 (manual) |
| `integrations/gingr/src/webhook.rs` | 1 | EntityId@191 (manual) |
| `storage/src/operations.rs` | 1 | StoredBrandName@652 (manual) |

## Hotspot recommendations

### `domain/src/customer.rs`

Keep `Name`, `Email`, and `Phone` on `nutype`. Their current trim, non-empty, and bounded validation is the right shape. Do not strengthen `Email` into full deliverability validation in this pass; it is a contact envelope, not consent or proof that outbound messaging is allowed. Tests should cover trim, blank rejection, max length, serde round-trip, and that workflows still gate messaging consent separately.

### `domain/src/care.rs`

Keep the nine care strings on `nutype`, but preserve manual redacted `Debug`. These values are sensitive source evidence, so the modernization rule is validated construction plus non-leaky diagnostics. Tests should assert redacted debug output for every care string, reject blanks/over-length values, and prove serde does not bypass validation.

### `domain/src/source.rs`

Keep manual constructors as the near-term default because the source module owns a rich `source::Error` surface and provenance semantics. The one accepted migration in this pass is `source::record::Id`, a source-agnostic reconciliation join key now covered by trim/non-empty/120-char validated serde tests while preserving `as_str()` at call sites. Future migrations in this family must preserve semantic error mapping or deliberately change callers to accept generated validation errors. Highest priority is not macro migration; it is fixing validated deserialization for manual types that currently derive `Deserialize`.

### `domain/src/training/mod.rs`

Keep the current `nutype` direction for training IDs, refs, evidence IDs, outcome documentation IDs, and progress notes. The repeated `Id` leaves are acceptable because the module path carries package/program/session semantics. Tests should focus on the public semantic path, not flattened names.

### `app/src/data_quality_hygiene.rs`

Migrate the five manual workflow strings to `nutype` or hand-written validated `Deserialize`. They are simple trim/non-empty workflow IDs/rationale values and currently duplicate boilerplate. If preserving `data_quality_hygiene::Error` variants is important to callers, use explicit `Deserialize` through `try_new` first and delay `nutype` until error mapping is settled. Tests must cover packet serde fixtures and invalid blank values.

### `integrations/gingr/src/webhook.rs`

Keep `EntityId` manual. It normalizes provider JSON where the identifier may arrive as a string or integer, and it has provider-specific failure modes. Preserve `Display` for normalized non-sensitive IDs only if tests continue to prove that webhook diagnostics avoid raw payload leakage.

## Acceptance checklist for implementation cards

- Inventory count remains 143 or the change explains added/removed newtypes.
- Every touched family documents its validation rule and has constructor tests.
- Every touched serde surface has round-trip and invalid-input tests.
- Sensitive families have redacted `Debug` tests or avoid `Debug` that leaks inner text.
- `Display`, `AsRef<str>`, `From`, and `TryFrom` are added only when a call site needs them and they do not bypass validation.
- Boundary wrappers retain provider fixture compatibility.
- Storage/read-model migrations check existing rows or fixtures before changing deserialize behavior.
