# Builder modernization classification

Source audit: `docs/kanban/2026-06-19-rust-crate-modernization-audit.md`.

This document classifies the 21 hand-written builders found by the audit. The goal is not to remove every manual builder; it is to choose the construction shape that best preserves provider/domain invariants while reducing panic-based required-field enforcement and boilerplate.

## Classification vocabulary

- `bon-convertible`: ordinary optional/required field assembly that `bon::Builder` can express without hiding domain semantics.
- `statum/typestate-worthy`: construction encodes a phase, mode, or protocol state that should be unrepresentable in the wrong order or state.
- `Result-returning semantic constructor`: missing required fields, provider-specific invalid combinations, or boundary validation should be returned as typed errors.
- `nonempty evidence constructor`: a collection is semantically invalid when empty and should use `nonempty` or an equivalent typed evidence wrapper.
- `intentionally manual`: manual shape carries endpoint mode, redaction, or provenance meaning that a generic derive would obscure.

## Executive classification

| Builder | Location | Classification | Proposed shape |
|---|---:|---|---|
| `source::reservation::SnapshotBuilder` | `domain/src/source.rs:591` | Result-returning semantic constructor | Replace `expect` required fields with `build() -> source::reservation::Result<Snapshot>` or a small domain-local `snapshot::Error`; `bon` is acceptable only if build remains fallible and semantic. |
| `source::gingr::reservation::SnapshotBuilder` | `domain/src/source.rs:1082` | Result-returning semantic constructor | Same fallible snapshot constructor, with Gingr-specific missing provenance/relationship errors. |
| `training::progress::ReportBuilder` | `domain/src/training/mod.rs:758` | nonempty evidence constructor + Result-returning semantic constructor | Use `nonempty::NonEmpty<ProgressEvidence>` or a `ProgressEvidenceSet`; convert missing IDs from `expect` to typed errors. |
| `training::outcome::DocumentationBuilder` | `domain/src/training/mod.rs:927` | nonempty evidence constructor + Result-returning semantic constructor | Use `nonempty::NonEmpty<Claim>` or a `ReviewedClaims` wrapper; convert missing IDs to typed errors. |
| `transport::RequestPartsBuilder` | `integrations/gingr/src/transport.rs:104` | intentionally manual | Keep manual unless a fallible transport descriptor constructor is introduced; this builder names redaction-sensitive request assembly. |
| `reference_data::GetVetsBuilder` | `integrations/gingr/src/endpoint/reference_data.rs:62` | bon-convertible | Replace with `#[derive(bon::Builder)]` or a direct constructor/default method; no required fields. |
| `labor_ops::TimeclockReportBuilder` | `integrations/gingr/src/endpoint/labor_ops.rs:50` | Result-returning semantic constructor | Keep fallible construction; `bon` can reduce setters if it preserves `MissingRequiredParameter` errors for start/end/location. |
| `report_cards_files::ReportCardFilesBuilder` | `integrations/gingr/src/endpoint/report_cards_files.rs:30` | bon-convertible | Replace with `bon` optional setters or plain `Default` plus setters; no required fields. |
| `commerce_retail::get::SubscriptionsBuilder` | `integrations/gingr/src/endpoint/commerce_retail.rs:155` | bon-convertible | Replace with `bon`; all filters are optional and wrappers validate their own invariants. |
| `commerce_retail::list::TransactionsBuilder` | `integrations/gingr/src/endpoint/commerce_retail.rs:259` | Result-returning semantic constructor | Keep fallible constructor because it enforces legacy cutover and required date window. |
| `commerce_retail::list::InvoicesBuilder` | `integrations/gingr/src/endpoint/commerce_retail.rs:362` | Result-returning semantic constructor | Keep fallible constructor because invoice dates have an on/after cutover invariant. |
| `owners_animals::OwnersBuilder` | `integrations/gingr/src/endpoint/owners_animals.rs` | bon-converted provider grammar builder | Use `bon::Builder` on `Owners`; keep provider query grammar quarantined in `ProviderWhereClause`. |
| `owners_animals::AnimalsBuilder` | `integrations/gingr/src/endpoint/owners_animals.rs` | bon-converted provider grammar builder | Same as `OwnersBuilder`; the typed clause collection preserves provider expression keys. |
| `owners_animals::custom_field::SearchBuilder` | `integrations/gingr/src/endpoint/owners_animals.rs` | bon-converted required-field builder | Use `bon::Builder` so form, field name, and sensitive search are compile-time required builder fields. |
| `reservations::reservation::TypesBuilder` | `integrations/gingr/src/endpoint/reservations.rs:40` | bon-convertible | Replace with `bon`; all filters are optional. |
| `reservations::reservation::WidgetDataBuilder` | `integrations/gingr/src/endpoint/reservations.rs:103` | bon-convertible | Replace with `bon` required `timestamp`; no extra semantic validation beyond `Date`. |
| `reservations::reservation::SearchFiltersBuilder` | `integrations/gingr/src/endpoint/reservations.rs:187` | bon-convertible | Replace with `bon` optional/repeated filters if array parameter emission stays tested. |
| `reservations::Builder` | `integrations/gingr/src/endpoint/reservations.rs:277` | statum/typestate-worthy | Keep or formalize as mode constructor: `Reservations::checked_in()` vs `Reservations::for_range(range)` are mutually exclusive request modes. |
| `reservations::by::AnimalBuilder` | `integrations/gingr/src/endpoint/reservations.rs:374` | Result-returning semantic constructor | Replace required `animal_id` panic with typed error or `bon` required field plus fallible/checked build. |
| `reservations::by::OwnerBuilder` | `integrations/gingr/src/endpoint/reservations.rs:450` | Result-returning semantic constructor | Same as `AnimalBuilder` for required `owner_id`. |
| `reservations::BackOfHouseBuilder` | `integrations/gingr/src/endpoint/reservations.rs:545` | Result-returning semantic constructor | Replace required `location` panic with typed error; consider nonempty type IDs only if provider semantics require at least one type. |

## Per-builder audit details

### 1. `source::reservation::SnapshotBuilder`

- Current behavior: assembles source-agnostic reservation source evidence from required `provenance` and required `relationship`, optional provider record IDs, optional normalized status, and reviewer assumptions.
- Required invariants: provenance must exist; owner/pet relationship confidence must exist; optional record IDs and status remain optional because incomplete source facts become typed data-quality issues later.
- Panic/expect usage: `expect("snapshot provenance is required")` and `expect("snapshot relationship is required")`.
- Classification: Result-returning semantic constructor.
- Proposed crate/shape: fallible domain constructor, e.g. `build() -> source::reservation::Result<Snapshot>` with `Error::MissingSnapshotProvenance` and `Error::MissingSnapshotRelationship`; optionally use `bon` for setter generation only if fallibility remains explicit at the call site.
- Verification tests named: keep `domain/tests/reservation_source_contracts.rs::source_agnostic_reservation_snapshot_preserves_provenance_without_gingr_paths`, `missing_and_ambiguous_source_facts_emit_typed_data_quality_issues`, and `incomplete_source_reservation_facts_return_typed_data_quality_issues_instead_of_stay_fact`; add `source_snapshot_builder_returns_missing_provenance_error` and `source_snapshot_builder_returns_missing_relationship_error`.

### 2. `source::gingr::reservation::SnapshotBuilder`

- Current behavior: assembles a Gingr-specific reservation snapshot from required Gingr provenance and required Gingr owner/animal relationship, optional Gingr owner/animal/location/service IDs, and optional provider status; promotion maps it into the source-agnostic snapshot and adds provisional assumptions.
- Required invariants: Gingr provenance and relationship confidence must exist before promotion; optional status remains optional and causes refresh-policy assumptions.
- Panic/expect usage: `expect("snapshot provenance is required")` and `expect("snapshot relationship is required")`.
- Classification: Result-returning semantic constructor.
- Proposed crate/shape: fallible Gingr snapshot constructor returning a source or Gingr-local error (`MissingGingrProvenance`, `MissingGingrRelationship`); `bon` is acceptable only as setter derivation, not as a panic-based required-field mechanism.
- Verification tests named: keep `domain/tests/reservation_source_contracts.rs::gingr_reservation_snapshot_promotes_to_source_agnostic_snapshot_with_assumptions` and `source_specific_gingr_fixture_must_promote_before_projection_consumers_see_source_records`; add `gingr_snapshot_builder_returns_missing_provenance_error` and `gingr_snapshot_builder_returns_missing_relationship_error`.

### 3. `training::progress::ReportBuilder`

- Current behavior: builds a training progress report from required report/enrollment/session identifiers, evidence, milestones, and optional approval defaulting to `Draft`; returns `Err(Error::ProgressEvidenceRequired)` for empty evidence.
- Required invariants: report id, enrollment id, session ref, and at least one evidence item; approval may default to draft; milestones may be empty unless the business later requires milestone evidence.
- Panic/expect usage: `expect("report_id is required")`, `expect("enrollment_id is required")`, and `expect("session_ref is required")`; evidence absence is already typed.
- Classification: nonempty evidence constructor plus Result-returning semantic constructor.
- Proposed crate/shape: promote evidence to `nonempty::NonEmpty<ProgressEvidence>` or a domain `progress::EvidenceSet`; add typed missing-field errors for IDs/session; `bon` can help setters after evidence/missing-field semantics are explicit.
- Verification tests named: keep `domain/tests/petsuites_core_service_contracts.rs::progress_report_cannot_be_parent_facing_until_approved_even_when_evidence_exists`; add `progress_report_builder_rejects_empty_evidence`, `progress_report_builder_returns_missing_report_id_error`, `progress_report_builder_returns_missing_enrollment_id_error`, and `progress_report_builder_returns_missing_session_ref_error`.

### 4. `training::outcome::DocumentationBuilder`

- Current behavior: builds outcome documentation from required documentation/enrollment/pet/location IDs, claims, and optional review defaulting to `Draft`; returns `Err(Error::OutcomeClaimRequired)` for empty claims.
- Required invariants: all four identity fields and at least one claim; individual `Claim::from_evidence` already enforces evidence for achieved/readiness claims.
- Panic/expect usage: `expect("documentation_id is required")`, `expect("enrollment_id is required")`, `expect("pet_id is required")`, and `expect("location_id is required")`; empty claims are already typed.
- Classification: nonempty evidence constructor plus Result-returning semantic constructor.
- Proposed crate/shape: use `nonempty::NonEmpty<Claim>` or an `outcome::Claims` wrapper; return typed missing-field errors instead of panicking; keep default draft review.
- Verification tests named: keep `domain/tests/petsuites_core_service_contracts.rs::achieved_outcome_claim_requires_evidence_before_documentation_can_be_member_facing`; add `outcome_documentation_builder_rejects_empty_claims`, `outcome_documentation_builder_returns_missing_documentation_id_error`, `outcome_documentation_builder_returns_missing_enrollment_id_error`, `outcome_documentation_builder_returns_missing_pet_id_error`, and `outcome_documentation_builder_returns_missing_location_id_error`.

### 5. `transport::RequestPartsBuilder`

- Current behavior: central transport descriptor builder for method, path, parameters, and sensitive parameter names; later logic turns GET params into query pairs and POST params into form pairs, then redacts configured sensitive names in diagnostics.
- Required invariants: method and path are required; parameters may be empty; sensitive names may be empty but must be preserved for redaction.
- Panic/expect usage: `expect("request method is required")` and `expect("request path is required")`.
- Classification: intentionally manual.
- Proposed crate/shape: keep manual redaction-centered shape for now; if changed, prefer `build() -> gingr::endpoint::Result<RequestParts>` with `MissingRequiredParameter { parameter: "method" | "path" }`. Avoid a generic derive that hides the diagnostic-redaction role.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::get_locations_is_a_get_request_with_api_key_added_by_transport`, `commerce_retail_endpoints_preserve_legacy_date_boundaries_and_payment_sensitivity`, and `integrations/gingr/tests/expanded_endpoint_contracts.rs::reservations_range_post_uses_form_body_and_rejects_ranges_longer_than_30_days`; add `request_parts_builder_returns_missing_method_error`, `request_parts_builder_returns_missing_path_error`, and `request_parts_builder_redacts_sensitive_parameter_names` if fallible build is introduced.

### 6. `reference_data::GetVetsBuilder`

- Current behavior: single optional `include_all_information` flag; when true emits `vetFlag=true`, otherwise no params.
- Required invariants: none beyond boolean flag semantics.
- Panic/expect usage: none.
- Classification: bon-convertible.
- Proposed crate/shape: `#[derive(bon::Builder)]` with `#[builder(default)] include_all_information: bool`, or remove the builder and expose `GetVets::default().include_all_information(true)` if call-site ergonomics remain clear.
- Verification tests named: keep `integrations/gingr/tests/expanded_endpoint_contracts.rs::owner_lookup_requires_one_discriminator_and_reference_endpoints_stay_typed`; add/keep `get_vets_builder_emits_vet_flag_only_when_extended_information_requested`.

### 7. `labor_ops::TimeclockReportBuilder`

- Current behavior: builds a timeclock report with required date range and location, optional deleted/clocked-in flags, and repeated user IDs; returns `Error::MissingRequiredParameter` for missing required fields.
- Required invariants: start date, end date, and location are required; user IDs may be empty to query all users; date format is guaranteed by `Date`.
- Panic/expect usage: none; already typed via `ok_or(Error::MissingRequiredParameter { ... })`.
- Classification: Result-returning semantic constructor.
- Proposed crate/shape: keep fallible build; optionally use `bon` for setters if it does not replace provider-specific `MissingRequiredParameter` errors or obscure the `date_range(start, end)` semantic grouping.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::subscriptions_timeclock_and_report_card_files_expose_documented_filters`; add `timeclock_report_builder_requires_start_date_end_date_and_location` and `timeclock_report_builder_preserves_repeated_user_ids`.

### 8. `report_cards_files::ReportCardFilesBuilder`

- Current behavior: optional number-days, limit, and location filters copied directly to query parameters when present.
- Required invariants: none in the builder; any positive/bounded semantics for day/limit are not currently encoded.
- Panic/expect usage: none.
- Classification: bon-convertible.
- Proposed crate/shape: derive `bon::Builder` with optional fields, or replace with direct `ReportCardFiles` builder derive; consider semantic `ReportCardLookbackDays` and `ReportCardFileLimit` later if invalid zero/unbounded queries matter.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::subscriptions_timeclock_and_report_card_files_expose_documented_filters`; add `report_card_files_builder_omits_absent_optional_filters` if the builder is converted.

### 9. `commerce_retail::get::SubscriptionsBuilder`

- Current behavior: optional filters for include-deleted, bill day, owner, pagination, location, and package; pagination expands to `limit`/`offset`.
- Required invariants: none in the builder; `BillDayOfMonth` and `SubscriptionPagination` own validation.
- Panic/expect usage: none.
- Classification: bon-convertible.
- Proposed crate/shape: derive `bon::Builder` on `Subscriptions` with defaults for all options; preserve wrapper constructors for bill day and pagination.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::subscriptions_timeclock_and_report_card_files_expose_documented_filters`; add `subscriptions_builder_omits_absent_filters_and_expands_pagination` if converted.

### 10. `commerce_retail::list::TransactionsBuilder`

- Current behavior: requires `from_date` and `to_date`; rejects dates on or after the 2019-08-01 invoice cutover because the endpoint is legacy pre-cutover POS transactions.
- Required invariants: both dates are required; both must be before cutover; currently no explicit from <= to check in this builder.
- Panic/expect usage: none; missing and boundary errors are typed.
- Classification: Result-returning semantic constructor.
- Proposed crate/shape: keep fallible build; optionally introduce a semantic `LegacyTransactionDateRange` smart constructor and let the request hold that instead of separate fields.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::commerce_retail_endpoints_preserve_legacy_date_boundaries_and_payment_sensitivity`; add `transactions_builder_requires_from_and_to_dates` and `transactions_builder_rejects_dates_on_or_after_invoice_cutover` if split from the existing combined test.

### 11. `commerce_retail::list::InvoicesBuilder`

- Current behavior: optional pagination, complete/closed flags, and optional date filters; any provided date must be on or after the 2019-08-01 invoice cutover.
- Required invariants: date filters, when present, are invoice-era dates; `InvoicePagination::new` enforces nonzero/per-page page stepping.
- Panic/expect usage: none.
- Classification: Result-returning semantic constructor.
- Proposed crate/shape: keep fallible build; optionally introduce `InvoiceDateFilter` or `InvoiceDateRange` smart constructors if paired date semantics are later required.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::list_invoices_requires_paired_pagination_and_on_or_after_legacy_cutover_dates`; add `invoices_builder_allows_absent_dates` and `invoices_builder_rejects_pre_cutover_dates` if converted.

### 12. `owners_animals::OwnersBuilder`

- Current behavior: accumulates provider where clauses and emits them as `params[...]` query pairs against `/api/v1/owners`.
- Required invariants: none requiring nonempty clauses; zero clauses represent unfiltered provider listing if the caller chooses it. `ProviderWhereClause` carries provider key/value grammar.
- Panic/expect usage: none.
- Classification: intentionally manual.
- Proposed crate/shape: keep manual accumulator to keep provider where-clause grammar explicit and quarantined; do not flatten into domain search fields until owner lookup semantics are known.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::owners_and_animals_filters_are_quarantined_as_provider_where_clauses`; add `owners_builder_allows_multiple_provider_where_clauses` if coverage is expanded.

### 13. `owners_animals::AnimalsBuilder`

- Current behavior: same provider where-clause accumulator as owners, targeting `/api/v1/animals`.
- Required invariants: none requiring nonempty clauses; provider grammar stays explicit in `ProviderWhereClause`.
- Panic/expect usage: none.
- Classification: intentionally manual.
- Proposed crate/shape: keep manual accumulator; provider-side field expressions such as `month(from_unixtime(birthday))` should remain visibly quarantined instead of becoming domain predicates.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::owners_and_animals_filters_are_quarantined_as_provider_where_clauses`; add `animals_builder_preserves_provider_expression_keys` if coverage is expanded.

### 14. `owners_animals::custom_field::SearchBuilder`

- Current behavior: builds a custom-field search from required form kind, non-empty provider field name, and sensitive lookup text; emitted `search` parameter is marked sensitive for redaction.
- Required invariants: form, field name, and search are required; field/search text must be non-empty via their wrappers; search must remain redacted in diagnostics.
- Panic/expect usage: `expect("Search requires form")`, `expect("Search requires field_name")`, and `expect("Search requires search")`.
- Classification: Result-returning semantic constructor.
- Proposed crate/shape: change `build() -> endpoint::Result<Search>` with missing form/field/search parameter errors; keep manual or `bon` with a fallible finalizer, but preserve explicit sensitivity via `sensitive_parameter_names()`.
- Verification tests named: keep `integrations/gingr/tests/expanded_endpoint_contracts.rs::forms_custom_field_and_back_of_house_are_explicitly_sensitive_or_v0_safe`; add `custom_field_search_builder_returns_missing_form_error`, `custom_field_search_builder_returns_missing_field_name_error`, `custom_field_search_builder_returns_missing_search_error`, and `custom_field_search_redacts_sensitive_lookup`.

### 15. `reservations::reservation::TypesBuilder`

- Current behavior: optional reservation-type id and optional active-only flag; emits only present filters.
- Required invariants: none beyond wrapper types.
- Panic/expect usage: none.
- Classification: bon-convertible.
- Proposed crate/shape: derive `bon::Builder` on `Types` with optional fields, or use a plain `Types::default()` plus setters if preferred.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::reservation_types_keeps_optional_filters_typed_and_redacted_diagnostics_safe`; add `reservation_types_builder_omits_absent_filters`.

### 16. `reservations::reservation::WidgetDataBuilder`

- Current behavior: one required timestamp parameter for `/api/v1/reservation_widget_data`.
- Required invariants: timestamp must be supplied and already be a valid provider `Date`.
- Panic/expect usage: `expect("WidgetData requires timestamp")`.
- Classification: bon-convertible.
- Proposed crate/shape: derive `bon::Builder` with required `timestamp`; a direct `WidgetData::new(timestamp)` would be even clearer because there is only one field.
- Verification tests named: keep `integrations/gingr/tests/endpoint_contracts.rs::reservation_widget_data_requires_a_yyyy_mm_dd_date_parameter`; add `widget_data_constructor_requires_timestamp_at_compile_time_or_returns_missing_timestamp_error` depending on chosen shape.

### 17. `reservations::reservation::SearchFiltersBuilder`

- Current behavior: mutable wrapper around a `SearchFilters` value; accumulates optional from/to dates, repeated reservation type IDs, repeated animal IDs, status flags, and limit.
- Required invariants: none requiring a nonempty filter set; repeated fields must retain insertion and emit Gingr array parameter names.
- Panic/expect usage: none.
- Classification: bon-convertible.
- Proposed crate/shape: derive `bon::Builder` if repeated setters can remain ergonomic (`reservation_type_id`, `animal_id`) and if emitted array parameter names remain tested; otherwise keep manual accumulator as low-risk.
- Verification tests named: keep `integrations/gingr/tests/expanded_endpoint_contracts.rs::reservations_by_pet_and_owner_encode_restrict_to_and_location_scope_caveat_filters`; add `reservation_search_filters_builder_preserves_repeated_type_and_animal_ids` and `reservation_search_filters_builder_omits_absent_status_flags`.

### 18. `reservations::Builder`

- Current behavior: not default-constructible; callers must enter via `Reservations::checked_in()` or `Reservations::for_range(range)`, then may add location, then build; this preserves checked-in vs range request mode.
- Required invariants: request mode must be one of checked-in or range; location is optional; range validity is owned by `DateRange`.
- Panic/expect usage: none.
- Classification: statum/typestate-worthy.
- Proposed crate/shape: keep current manual mode constructor or formalize with a typestate/mode type (`ReservationsBuilder<CheckedInMode>` / `ReservationsBuilder<RangeMode>`). `statum` is only worth it if more reservation request phases appear; do not collapse this into a default `bon` builder with `checked_in: bool` plus `Option<DateRange>` because that allows ambiguous states.
- Verification tests named: keep `integrations/gingr/tests/expanded_endpoint_contracts.rs::reservations_range_post_uses_form_body_and_rejects_ranges_longer_than_30_days`; add `reservations_checked_in_mode_does_not_emit_range_dates` and `reservations_range_mode_emits_start_and_end_dates`.

### 19. `reservations::by::AnimalBuilder`

- Current behavior: requires animal ID and optionally includes a `restrict_to` token plus nested search filters; emits POST form params to `/api/v1/reservations_by_animal`.
- Required invariants: animal ID is required; restrict/filter values are optional; filters must retain array parameter emission.
- Panic/expect usage: `expect("Animal requires animal_id")`.
- Classification: Result-returning semantic constructor.
- Proposed crate/shape: return typed missing parameter error for absent `animal_id`; `bon` can express required `animal_id` if it improves call-site ergonomics, but provider errors are preferable for runtime construction paths.
- Verification tests named: keep `integrations/gingr/tests/expanded_endpoint_contracts.rs::reservations_by_pet_and_owner_encode_restrict_to_and_location_scope_caveat_filters`; add `reservations_by_animal_builder_returns_missing_animal_id_error` and `reservations_by_animal_builder_preserves_nested_filters`.

### 20. `reservations::by::OwnerBuilder`

- Current behavior: requires owner ID and optionally includes `restrict_to` and nested filters; emits POST form params to `/api/v1/reservations_by_owner`.
- Required invariants: owner ID is required; restrict/filter values are optional; endpoint caveat about current-location scoping must remain visible.
- Panic/expect usage: `expect("Owner requires owner_id")`.
- Classification: Result-returning semantic constructor.
- Proposed crate/shape: return typed missing parameter error for absent `owner_id`; preserve `LOCATION_SCOPE_CAVEAT` and the provider-scope token semantics.
- Verification tests named: keep `integrations/gingr/tests/expanded_endpoint_contracts.rs::reservations_by_pet_and_owner_encode_restrict_to_and_location_scope_caveat_filters`; add `reservations_by_owner_builder_returns_missing_owner_id_error` and `reservations_by_owner_builder_preserves_location_scope_caveat`.

### 21. `reservations::BackOfHouseBuilder`

- Current behavior: requires location, optionally accumulates reservation type IDs, minutes-future window, and full-day flag; emits `/api/v1/back_of_house` query params.
- Required invariants: location is required; `MinutesFuture::new` already enforces positive minutes; reservation type IDs are currently optional because no source confirms the provider requires at least one.
- Panic/expect usage: `expect("BackOfHouse requires location")`.
- Classification: Result-returning semantic constructor.
- Proposed crate/shape: return typed missing-location error. Consider `nonempty::NonEmpty<reservation::TypeId>` only after confirming the endpoint/domain requires at least one type; otherwise keep optional/repeated IDs.
- Verification tests named: keep `integrations/gingr/tests/expanded_endpoint_contracts.rs::forms_custom_field_and_back_of_house_are_explicitly_sensitive_or_v0_safe`; add `back_of_house_builder_returns_missing_location_error`, `back_of_house_builder_preserves_type_id_array_parameters`, and `minutes_future_rejects_zero`.

## Migration order recommendation

1. Convert the pure optional endpoint builders first (`GetVetsBuilder`, `ReportCardFilesBuilder`, `SubscriptionsBuilder`, `TypesBuilder`, `SearchFiltersBuilder`) because they are low-risk `bon` candidates.
2. Convert panic-based integration endpoint builders to typed errors next (`WidgetDataBuilder`, `AnimalBuilder`, `OwnerBuilder`, `BackOfHouseBuilder`, and optionally `RequestPartsBuilder`). `custom_field::SearchBuilder` is now `bon`-generated with compile-time required fields instead of runtime missing-parameter errors.
3. Convert domain evidence builders last, adding `nonempty`/typed evidence wrappers and semantic error variants before changing call sites (`ReportBuilder`, `DocumentationBuilder`, both source snapshot builders).
4. Preserve or formalize mode builders (`reservations::Builder`) after the simpler conversions so the team can decide whether plain manual mode constructors are enough or `statum` adds value.

## Verification command

This task changes documentation only. Run:

```bash
cargo fmt --check
```
