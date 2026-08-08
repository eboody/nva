# Error Context Crate Evaluation

**Date:** 2026-06-19

**Scope:** compare the current `thiserror` posture with selective `snafu` adoption for the app, integration, and storage boundaries that need source endpoint, provider record, evidence batch, and workflow context. This is not a recommendation to churn simple domain validation errors.

## Recommendation

Adopt a **selective `snafu` pilot only at operational boundary seams**, while leaving the domain-core and simple semantic validation errors on `thiserror`.

The highest-value pilot is the Gingr integration boundary where endpoint request construction, transport, DTO parsing, webhook verification, and DTO-to-domain promotion need contextual selectors. A second candidate is storage JSON/projection diagnostics once persisted evidence batches become first-class records. App workflow errors should mostly keep `thiserror`, but add typed context helpers where the caller needs to preserve reservation ids, workflow/correlation ids, or evidence batches.

Do not replace every `thiserror` enum. Most current domain errors are small invariant failures such as `payment::Error::EmptyReference`, `reservation::Error::AddOnLabelTooLong`, `manager_daily_brief::Error::ZeroLaborMinutes`, and `data_quality_hygiene::Error::EmptyIssueRef`. They are already semantic, compact, comparable, and easy to snapshot.

## Current posture

Workspace-level error dependency:

```toml
thiserror = "2.0"
```

Crates.io currently reports:

```text
snafu = "0.9.1"
```

Observed code shape:

- Domain errors are `thiserror` enums with typed variants and no ambient context, e.g. `domain/src/payment/error.rs`, `domain/src/reservation/error.rs`, `domain/src/source.rs`, `domain/src/operations.rs`.
- App workflow errors are mostly small route/validation decisions, e.g. `app/src/booking_triage.rs`, `app/src/manager_daily_brief.rs`, `app/src/data_quality_hygiene.rs`, `app/src/daily_update.rs`, `app/src/tools/error.rs`.
- Integration errors already carry some provider vocabulary but often lose the operation that was being performed, e.g. `integrations/gingr/src/transport.rs`, `integrations/gingr/src/endpoint/mod.rs`, `integrations/gingr/src/mapping/mod.rs`, `integrations/gingr/src/webhook.rs`.
- Storage errors already name record families and fields, but codec errors do not yet include which persisted artifact/evidence batch was being decoded, e.g. `storage/src/operations.rs`.

## Decision rule

Use `thiserror` when the error is the domain fact:

- constructor validation failures;
- semantic enum/state rule failures;
- small app workflow gates that are compared in tests;
- errors that need `Clone`, `Copy`, `PartialEq`, or `Eq` and have no source chain;
- public domain-core errors where call sites should see the exact semantic variant.

Use `snafu` only when the failure site must attach operation context while preserving a lower-level source:

- endpoint/method/path/query/form context around a URL, HTTP, or provider response failure;
- provider record identity and provider field around DTO-to-domain promotion;
- source record/evidence batch ids around JSON decode/projection failures;
- workflow/correlation ids around app boundary orchestration failures;
- redacted request context that must appear in diagnostics without leaking secrets.

If a boundary error needs only one additional typed field and no source chain, prefer a strengthened `thiserror` helper over adding `snafu`.

## Candidate 1: Gingr transport and endpoint execution

Current shape in `integrations/gingr/src/transport.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("failed to construct Gingr URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("HTTP transport is not implemented for this SDK slice")]
    HttpNotImplemented,
}
```

Current failure construction around URL building:

```rust
let mut url = base_url.join_path(self.path)?;
```

What is missing operationally:

- provider endpoint path;
- method;
- redacted parameters;
- provider/system name;
- operation name, e.g. `build_url`, `send_request`, `decode_response`.

Selective `snafu` candidate:

```rust
use snafu::{ResultExt, Snafu};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to construct Gingr URL for {method} {path}"))]
    BuildUrl {
        method: endpoint::Method,
        path: endpoint::Path,
        redacted_parameters: Vec<(String, String)>,
        source: url::ParseError,
    },

    #[snafu(display("Gingr HTTP transport is not implemented for {method} {path}"))]
    HttpNotImplemented {
        method: endpoint::Method,
        path: endpoint::Path,
    },
}

fn url(&self, base_url: &config::BaseUrl) -> Result<url::Url> {
    base_url
        .join_path(self.path)
        .context(BuildUrlSnafu {
            method: self.method,
            path: self.path,
            redacted_parameters: self.redacted().parameters,
        })
}
```

Equivalent strengthened `thiserror` option:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("failed to construct Gingr URL for {method} {path}: {source}")]
    BuildUrl {
        method: endpoint::Method,
        path: endpoint::Path,
        redacted_parameters: Vec<(String, String)>,
        source: url::ParseError,
    },
}

impl TransportError {
    fn build_url(request: &RequestParts, source: url::ParseError) -> Self {
        Self::BuildUrl {
            method: request.method(),
            path: request.path(),
            redacted_parameters: request.redacted().parameters,
            source,
        }
    }
}
```

Recommendation: use this seam as the `snafu` pilot only once real HTTP transport lands. Until then, `HttpNotImplemented` is too small to justify churn. When network I/O exists, `snafu` selectors make the call site read as “attach request context here” instead of repeatedly hand-writing `map_err` helpers.

## Candidate 2: Gingr mapping from provider DTOs into domain candidates

Current shape in `integrations/gingr/src/mapping/mod.rs`:

```rust
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("missing required Gingr provider field: {field}")]
    MissingRequiredProviderField { field: ProviderField },

    #[error("invalid domain value promoted from Gingr provider field {field}: {reason}")]
    InvalidDomainValue { field: ProviderField, reason: String },
}
```

Current call-site shape in mapping modules is already clear for a single record field:

```rust
provider_record.name
    .as_deref()
    .ok_or(Error::MissingRequiredProviderField {
        field: ProviderField::RetailItemName,
    })?;
```

What is missing operationally when promotion fails in a batch:

- provider record id;
- endpoint/record collection that produced the DTO;
- source record ref or evidence batch id;
- mapper name, e.g. customer contact candidate vs retail product candidate.

Selective `snafu` candidate:

```rust
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Gingr {mapper} missing {field} on provider record {record_id}"))]
    MissingRequiredProviderField {
        mapper: Mapper,
        record_id: endpoint::ProviderRecordId,
        field: ProviderField,
    },

    #[snafu(display("Gingr {mapper} could not promote {field} from provider record {record_id}"))]
    InvalidDomainValue {
        mapper: Mapper,
        record_id: endpoint::ProviderRecordId,
        field: ProviderField,
        source_record: domain::source::RecordRef,
        source: domain::retail::Error,
    },
}
```

Strengthened `thiserror` option:

```rust
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("missing required Gingr {mapper} field {field} on provider record {record_id}")]
    MissingRequiredProviderField {
        mapper: Mapper,
        record_id: ProviderRecordId,
        field: ProviderField,
    },

    #[error("invalid domain value promoted by Gingr {mapper} from {field} on provider record {record_id}: {reason}")]
    InvalidDomainValue {
        mapper: Mapper,
        record_id: ProviderRecordId,
        field: ProviderField,
        reason: String,
    },
}
```

Recommendation: strengthen `thiserror` first unless a typed source chain matters. The current enum derives `Clone`, `PartialEq`, and `Eq`; those are valuable for mapping tests. `snafu` becomes attractive when mapping variants wrap non-clone source errors or when batch-level context needs consistent selector syntax across many `Option`/`Result` conversions.

## Candidate 3: Webhook verification and parse failures

Current shape in `integrations/gingr/src/webhook.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid Gingr webhook JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VerificationError {
    #[error("Gingr webhook is missing required field {field}")]
    MissingField { field: &'static str },
    #[error("unsupported Gingr webhook entity_id representation: {observed_type}")]
    UnsupportedEntityId { observed_type: String },
    #[error("malformed Gingr webhook signature: {reason}")]
    MalformedSignature { reason: String },
    #[error("Gingr webhook signature mismatch")]
    SignatureMismatch,
}
```

What is missing operationally:

- webhook type when available;
- entity type/id when available;
- delivery id or request id if the provider supplies one later;
- safe body-size/hash context for parse failures instead of raw body data.

Recommended path: keep `VerificationError` on `thiserror` because it is deterministic, `PartialEq`, and checked directly. Add typed context only to parse/ingest boundaries if a request envelope is introduced:

```rust
#[derive(Debug, Snafu)]
pub enum IngestError {
    #[snafu(display("invalid Gingr webhook JSON for delivery {delivery_id}"))]
    ParseJson {
        delivery_id: DeliveryId,
        body_sha256: BodySha256,
        source: serde_json::Error,
    },

    #[snafu(display("Gingr webhook verification failed for delivery {delivery_id}"))]
    Verify {
        delivery_id: DeliveryId,
        webhook_type: Option<WebhookType>,
        entity_id: Option<EntityId>,
        source: VerificationError,
    },
}
```

This keeps simple verification facts comparable while letting the outer boundary carry delivery/evidence context.

## Candidate 4: Storage codec and projection errors

Current shape in `storage/src/operations.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("storage codec error")]
    Codec(#[from] CodecError),
    #[error("{record:?} storage shape mismatch: {reason:?}")]
    StorageShapeMismatch { record: RecordKind, reason: ShapeMismatchReason },
    #[error("domain value rejected storage field {field:?}: {reason}")]
    InvalidDomainValue { field: StorageField, reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("failed to decode json: {source}")]
    JsonDecode { source: serde_json::Error },
    #[error("failed to encode json: {source}")]
    JsonEncode { source: serde_json::Error },
}
```

Current call sites repeat the same context-free conversion:

```rust
serde_json::from_str(raw).map_err(|source| CodecError::JsonDecode { source }.into())
```

What is missing operationally:

- record kind being decoded;
- persisted artifact/evidence batch id;
- source system and provider record ref if this is evidence-derived storage;
- storage operation, e.g. fixture decode vs Postgres row decode vs API response decode.

Selective `snafu` candidate:

```rust
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to decode {record:?} JSON from {artifact}"))]
    JsonDecode {
        record: RecordKind,
        artifact: StorageArtifactRef,
        source: serde_json::Error,
    },

    #[snafu(display("{record:?} storage shape mismatch for {artifact}: {reason:?}"))]
    StorageShapeMismatch {
        record: RecordKind,
        artifact: StorageArtifactRef,
        reason: ShapeMismatchReason,
    },
}

impl ServiceOfferingRecord {
    pub fn decode_json_from(raw: &str, artifact: StorageArtifactRef) -> Result<Self> {
        serde_json::from_str(raw).context(JsonDecodeSnafu {
            record: RecordKind::ServiceOffering,
            artifact,
        })
    }
}
```

Strengthened `thiserror` option:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("failed to decode {record:?} json from {artifact}: {source}")]
    JsonDecode {
        record: RecordKind,
        artifact: StorageArtifactRef,
        source: serde_json::Error,
    },
}
```

Recommendation: do not add `snafu` just for `serde_json::Error`; use a typed `StorageArtifactRef`/`EvidenceBatchRef` helper first. Consider `snafu` if storage gains multiple nested source errors across Postgres, JSON, fixture, and domain projection layers and the selectors reduce repeated `map_err` plumbing.

## Candidate 5: App workflow boundary errors

Current examples:

```rust
// app/src/booking_triage.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("booking triage reservation repository could not load requested reservation")]
    ReservationNotFound,
}
```

```rust
// app/src/daily_update.rs
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("daily update preview could not build a validated domain value: {0}")]
    InvalidDomainValue(String),
}
```

What is missing operationally:

- reservation id that failed repository lookup;
- workflow name;
- correlation/context packet id;
- source evidence batch used to make the decision.

Recommended strengthened `thiserror` shape:

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("booking triage reservation repository could not load reservation {reservation_id}")]
    ReservationNotFound {
        reservation_id: entities::reservation::Id,
        workflow: workflow::Name,
    },
}
```

Recommended helper pattern:

```rust
impl Error {
    pub fn reservation_not_found(reservation_id: entities::reservation::Id) -> Self {
        Self::ReservationNotFound {
            reservation_id,
            workflow: workflow::Name::BOOKING_TRIAGE,
        }
    }
}
```

Use `snafu` only if the repository boundary becomes fallible with a source error:

```rust
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("booking triage could not load reservation {reservation_id}"))]
    LoadReservation {
        reservation_id: entities::reservation::Id,
        evidence_batch: source::evidence::BatchId,
        source: reservation::repository::Error,
    },
}
```

## Do-not-churn list

Keep `thiserror` for these classes unless a concrete source/context problem appears:

- `domain/src/payment/error.rs` and `domain/src/reservation/error.rs`: simple constructor validation.
- `domain/src/boarding/*`, `domain/src/retail/*`, `domain/src/grooming/*`, `domain/src/training/*`: semantic scalar and invariant validation.
- `app/src/manager_daily_brief.rs` and `app/src/data_quality_hygiene.rs`: simple nonzero/nonempty workflow input validation.
- `integrations/gingr/src/endpoint/mod.rs`: input validation variants such as invalid date, reversed range, invalid pagination, and bill day. Strengthen only if endpoint/path context is not inferable from the type at the call site.

## Implementation plan if adopted

1. Add `snafu = "0.9.1"` to `[workspace.dependencies]` only after selecting the first real boundary pilot.
2. Start with one module, preferably `integrations/gingr/src/transport.rs` after real HTTP transport exists, or a new outer `webhook::IngestError` if webhook delivery metadata is introduced.
3. Add display snapshot-style tests for all changed operational diagnostics. At minimum assert the display string does not leak secrets and includes endpoint/path/record/evidence context.
4. Preserve semantic matching tests for inner errors. If an existing `PartialEq`/`Eq` error is useful in tests, keep it as an inner `thiserror` type and wrap it in an outer `snafu` boundary error.
5. Run:

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Final decision

Recommendation: **selective `snafu` adoption, not a wholesale migration**.

`thiserror` remains the default for domain and simple app errors because it preserves semantic enum shape, equality, and low ceremony. `snafu` is worth piloting only where boundary code repeatedly needs to say: “while doing this operation, for this endpoint/provider record/evidence batch/workflow, this source failed.” In the current codebase, that means future Gingr transport/ingest and possibly storage artifact decode/projection boundaries. For today, the safest immediate improvement is to strengthen `thiserror` context helpers for app workflow and storage errors, then introduce `snafu` when a boundary has enough source chaining to justify the new dependency.
