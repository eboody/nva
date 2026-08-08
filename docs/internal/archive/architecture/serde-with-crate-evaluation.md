# `serde_with` Crate Evaluation

**Date:** 2026-06-19

**Scope:** evaluate whether `serde_with` should be added for Gingr provider DTOs, app API DTOs, storage records, URL/query/date handling, and transparent wrappers. The decision rule is deliberately conservative: add the crate only if it removes repeated bespoke serializers or prevents data-contract drift.

## Recommendation

Do **not** add `serde_with` yet.

The current serialization surface is mostly plain Serde derive, `#[serde(default)]`, `#[serde(flatten)]`, `#[serde(alias = ...)]`, `#[serde(rename_all = "snake_case")]`, and intentional `#[serde(transparent)]` wrappers. Those attributes are contract-shaped and locally readable at the boundary where they matter. The repeated custom code that exists today is mostly invariant validation for semantic newtypes, not format conversion boilerplate. Replacing those impls with `serde_with` would add another dependency without reducing enough repetition or clarifying the data contract.

The one high-value future trigger is **repeated same-format date/time or number/string coercion across provider DTOs**. If Gingr DTOs start carrying many `YYYY-MM-DD`, Unix timestamp, or string-or-number id fields that must deserialize into typed wrappers while preserving provider quirks, re-open this decision and pilot `serde_with` in `integrations/gingr` only.

## Current dependency posture

Workspace serialization dependencies today:

```toml
chrono = { version = "0.4", features = ["serde"] }
nutype = { version = "0.7", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_urlencoded = "0.7"
uuid = { version = "1.10", features = ["serde", "v4"] }
```

Crates.io currently reports:

```text
serde_with = "3.21.0"
```

## Inventory

### Gingr provider DTOs

Observed files:

- `integrations/gingr/src/response.rs`
- `integrations/gingr/src/dto/retail.rs`
- `integrations/gingr/src/webhook.rs`

Current needs:

- transparent provider wrappers such as `HttpStatus`, `provider::Error`, `provider::Email`, and provider ids;
- optional provider fields that should default to `None` when Gingr omits them;
- aliasing one known provider field (`retail_category` -> `category`, `cell` -> `cell_phone`);
- flattened unknown-field maps that quarantine unmodeled provider payloads for audit;
- raw `serde_json::Value` for webhook/entity payloads whose schema is intentionally not promoted yet.

Decision: keep plain Serde attributes. `serde_with` does not improve the visible contract for `default`, `alias`, `flatten`, or transparent newtypes. The raw webhook/entity payloads should stay quarantined as `serde_json::Value`; using helper adapters there would hide a deliberate trust-boundary decision.

Verification added: `integrations/gingr/tests/expanded_endpoint_contracts.rs` now asserts exact serialized JSON for owner, animal, and retail DTOs, roundtrips those wire strings through `serde_json`, and checks the `retail_category` alias still maps to `category`.

### App API DTOs

Observed file:

- `apps/api/src/http.rs`

Current needs:

- route/query/body structs using `Deserialize` or `Serialize`;
- many in-memory/demo response payloads;
- `NaiveDate`, `DateTime<Utc>`, `Uuid`, strings, booleans, and vectors using existing serde implementations;
- `#[serde(default)]` on query types so omitted filters are explicit `None`/empty inputs.

Decision: keep plain Serde. App DTOs are mostly HTTP shell glue and deterministic workflow-demo payloads; there is no repeated custom serializer. If an API route later needs a stable non-default date format that appears across multiple request/response DTOs, prefer a tiny app-local module first, then consider `serde_with` only if the format repeats enough to justify the dependency.

### Storage records

Observed files:

- `storage/src/operations.rs`
- `storage/src/service_line/*.rs`

Current needs:

- stable snake_case enum codes;
- transparent wrappers around storage quantities;
- flattened optional service-line record shapes with discriminator validation;
- manual `Deserialize` impls for storage newtypes that must call `try_new` and report domain/storage validation errors;
- JSON encode/decode helpers that make the persistence gate explicit.

Decision: keep current manual validation and plain Serde attributes. `serde_with` can express some conversion patterns, but these storage wrappers are not just conversion mechanics; their deserializers are the validation gate for persisted records. The explicit impls keep the invariant close to the storage field and preserve semantic error text.

Rejected case: replacing manual `Deserialize` for non-zero quantities with generic `TryFromInto`-style adapters. That would reduce a few lines but make the storage gate less obvious and would require adding `TryFrom`/`From` shapes solely for serde plumbing.

### URL/query/form handling

Observed files:

- `integrations/gingr/src/transport.rs`
- `integrations/gingr/src/endpoint/*.rs`

Current needs:

- typed endpoint builders assemble `Vec<(String, String)>` pairs;
- GET requests put pairs on the URL query; POST-style Gingr endpoints put pairs in form fields;
- secret key injection/redaction happens in transport, not endpoint DTO serialization;
- provider-specific bracketed keys such as `params[fromDate]`, `params[reservationTypeIds][]`, and `type_ids[]` remain visible in request code and tests.

Decision: do not route query/form parameters through `serde_with` or struct serialization today. The provider key spelling is business-critical source evidence, and the current explicit `parameters()` methods make the contract easy to audit. A derived query struct would obscure Gingr's bracketed parameter names and make redaction boundaries less direct.

Rejected case: deriving query structs then using serializer adapters for optional fields. This would save `push_optional` calls, but the exact provider key names and GET-vs-form separation are clearer as explicit request descriptors.

### Dates and transparent wrappers

Observed surfaces:

- Gingr endpoint `Date`/`IsoDate` parse and format `YYYY-MM-DD` manually for request parameters.
- Domain/app/storage use `chrono`'s serde support for `NaiveDate` and `DateTime<Utc>` where JSON payloads are already internal contracts.
- Provider DTOs currently keep ambiguous date strings raw when no semantic validation exists, e.g. Gingr animal birthday.
- Many id/quantity wrappers use `#[serde(transparent)]` or manual deserialization to protect invariants.

Decision: keep the local date wrappers and `Display` formatting for endpoint parameters. They are not JSON DTO fields; they are validated provider filters. Keeping them out of serde prevents accidental reuse as domain dates and keeps Gingr request formatting in the endpoint layer.

Future trigger: if multiple provider response DTOs begin deserializing the same date/time string format into typed fields, a scoped `serde_with` pilot may be justified to avoid drift between date adapters.

## Adoption rule if this changes later

Add `serde_with` only when all of these are true:

1. At least three production DTO fields repeat the same non-default serde transform.
2. The transform is mechanical and contract-preserving, not semantic validation that should be named in a domain/storage constructor.
3. A roundtrip test proves the exact wire shape, including aliases, omitted-field behavior, unknown-field preservation, and error handling.
4. The dependency is scoped to the crate with the repeated boundary problem; do not add it workspace-wide by default.

Likely candidates if the trigger appears:

- repeated `YYYY-MM-DD` provider response fields promoted into a provider DTO date wrapper;
- repeated string-or-number provider ids that must normalize to a provider id type;
- repeated empty-string-as-`None` fields if Gingr fixtures show that as stable behavior.

Non-candidates:

- storage/domain newtype invariant checks;
- flattened storage records with discriminator validation;
- webhook payloads intentionally quarantined as raw JSON;
- query/form builder code where provider parameter names, redaction, and GET-vs-form placement are semantically important.
