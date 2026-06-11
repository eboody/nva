# Vaccine policy by species and service

Status: draft policy proposal for the vaccine-document-agent workflow. This is not an approved live medical or booking policy. It defines the policy shape, conservative defaults, and approval questions needed before any vaccine fact can affect booking eligibility.

Human approval gate: final vaccine policy by service/species. A manager/admin or other approved policy owner must approve the required vaccine list, validity windows, source rules, and service mapping before this document is treated as authoritative.

## Source anchors

Repository anchors:

- `docs/workflows/vaccine-document-agent-parts/inputs.md`: no approved location-specific vaccine policy exists; vaccine document extraction must remain suggestions/unverified data until a medical-document review gate accepts it.
- `domain/src/entities.rs`: current species are `Dog`, `Cat`, and `Other`; current services are `Boarding`, `DayPlay`, `DayBoarding`, `Grooming`, `Training`, and `DaySpa`; booking status can become `VaccinePending`, `MissingInfo`, or `SpecialReview`; missing vaccine proof can appear as `HardStop::MissingRequiredVaccine(policy::VaccineName)`.
- `domain/src/policy.rs`: current vaccine policy model has `VaccineRequirement { species, service, vaccines, source_must_be_licensed_vet }`, `VaccineName`, `ReviewGate::MedicalDocumentReview`, and automation levels.
- `docs/workflows/booking-triage-parts/inputs.md`: missing, stale, ambiguous, or conflicting vaccine proof routes to document/medical review and must not become final approval by OCR/AI alone.
- Public PetSuites examples checked for plausible baseline only, not as location-approved policy: Addison FAQ and Lee's Summit get-started pages list dogs as Rabies, DHPP, Bordetella, Canine Influenza; cats as Rabies and FVRCP; all vaccinations administered by a veterinary professional; Lee's Summit says proof from a licensed veterinarian is required for all services.

External-source caveat: public PetSuites pages are useful defaults for a PetSuites/NVA-like product model, but local locations can differ. Do not copy these assumptions into live eligibility without a location policy owner approving them.

## Policy decision model

Every vaccine decision should be evaluated against a location-scoped policy snapshot:

- `vaccine_policy_id`: required for every comparison.
- `policy_version` / snapshot timestamp: required for audit and replay.
- `species`: dog, cat, or other.
- `service`: boarding, day play/daycare, day boarding/individual play, grooming, training, or DaySpa/bathing.
- `required_vaccines[]`: canonical vaccine names plus accepted aliases.
- `optional_or_location_specific_vaccines[]`: vaccines that may be required by some locations or packages but are not approved as universal requirements.
- `validity_rule`: explicit expiration/due date required, or an approved fallback validity window if no expiration date appears on the document.
- `source_rule`: whether proof must be from a licensed veterinarian/veterinary professional.
- `eligibility_effect`: how missing/expired/unverified proof maps to booking readiness.

Safe default if any of these policy fields are missing: extract and summarize document facts, but do not evaluate eligibility; route to `MedicalDocumentReview` or `SpecialReview`.

## Canonical vaccine names and aliases

Use canonical names in policy comparisons and keep aliases as extraction/mapping hints.

| Canonical name | Common aliases / extraction labels | Species | Draft status |
| --- | --- | --- | --- |
| Rabies | Rabies certificate, rabies 1-year, rabies 3-year | Dog, Cat | Required in the proposed baseline |
| DHPP | DHLPP, DA2PP, DAPP, distemper/parvo combo, distemper/hepatitis/parainfluenza/parvovirus; leptospirosis may appear as the extra `L` in DHLPP | Dog | Required in the proposed baseline; alias handling needs approval |
| Bordetella | Kennel cough, bordetella bronchiseptica | Dog | Required in the proposed baseline |
| Canine Influenza | CIV, canine flu, H3N2, H3N8, bivalent influenza | Dog | Required in public PetSuites examples; local approval needed, especially dose-series handling |
| FVRCP | Feline distemper combo, feline viral rhinotracheitis/calicivirus/panleukopenia | Cat | Required if cat boarding/daycare/DaySpa exists |
| FeLV | Feline leukemia | Cat | Unknown / location-specific; do not require unless approved |
| Other vaccine | Any label not mapped above | Any | Unknown; route to review if relevant to a requirement |

Alias rules:

- Accepting an alias as equivalent to a canonical requirement is a policy decision, not an OCR decision.
- `DHPP` and `DHLPP` are likely same policy family for resort-readiness purposes, but leptospirosis-specific handling must be explicitly approved if the location requires or excludes it.
- Canine influenza labels may identify H3N2, H3N8, or a bivalent product. The policy owner must decide whether any documented CIV product counts, whether both strains are required, and how partially completed series are handled.
- The extractor may suggest mappings with evidence spans; only reviewed mappings can become accepted vaccine facts.

## Proposed baseline requirements by species and service

This table is the recommended baseline to approve or edit. It intentionally separates `required`, `optional/location-specific`, and `unknown` so missing local policy does not become silent approval.

### Dogs

| Service | Required in proposed baseline | Optional / location-specific | Unknown / approval needed | Eligibility effect if missing, expired, or unverified |
| --- | --- | --- | --- | --- |
| Boarding | Rabies, DHPP/DHLPP family, Bordetella, Canine Influenza | None by default | Whether canine influenza first dose of a two-dose series is temporarily acceptable; whether DHLPP must include lepto; any puppy exceptions | `VaccinePending` with `MissingRequiredVaccine` hard stops per missing vaccine; no confirmation/check-in readiness until reviewed |
| DayPlay / daycare | Rabies, DHPP/DHLPP family, Bordetella, Canine Influenza | None by default | Same as boarding; same-day daycare freshness rules | `VaccinePending`; also blocks group-play/daycare readiness until reviewed |
| DayBoarding / individual play | Rabies, DHPP/DHLPP family, Bordetella, Canine Influenza | None by default | Whether individual-play/day-boarding can use a narrower set than group play | `VaccinePending` or `SpecialReview` if local policy allows alternative care lane |
| Grooming | Rabies, DHPP/DHLPP family, Bordetella, Canine Influenza, if the location adopts the public PetSuites `all services` rule | Service-specific exception may narrow this to rabies-only or no resort-core bundle | Exact local grooming vaccine requirement | If policy says required: `VaccinePending`; if unknown: `SpecialReview` rather than auto-eligible |
| Training | Rabies, DHPP/DHLPP family, Bordetella, Canine Influenza, if the location adopts the public PetSuites `all services` rule | Class/private-training variants may differ | Exact local training vaccine requirement | If policy says required: `VaccinePending`; if unknown: `SpecialReview` |
| DaySpa / bathing | Rabies, DHPP/DHLPP family, Bordetella, Canine Influenza, if the location adopts the public PetSuites `all services` rule | Bath-only or add-on contexts may differ | Exact local DaySpa/bathing vaccine requirement | If policy says required: `VaccinePending`; if unknown: `SpecialReview` |

### Cats

Cat services are present in the domain model as species support, and public PetSuites examples include cat boarding. If a location does not offer cat services, cat rows should be disabled in that location's policy snapshot rather than inferred.

| Service | Required in proposed baseline | Optional / location-specific | Unknown / approval needed | Eligibility effect if missing, expired, or unverified |
| --- | --- | --- | --- | --- |
| Boarding | Rabies, FVRCP | FeLV if local policy requires it | Whether male-cat neuter policy is part of vaccine eligibility or separate booking policy; any kitten exceptions | `VaccinePending` with `MissingRequiredVaccine` hard stops per missing vaccine; no confirmation/check-in readiness until reviewed |
| DayPlay / daycare | Unknown unless cat daycare/enrichment exists; if offered, start from Rabies and FVRCP | FeLV | Whether cat daycare exists and whether it follows boarding requirements | `SpecialReview` until location policy defines service; then `VaccinePending` for missing requirements |
| DayBoarding / individual enrichment | Rabies, FVRCP if cat day-boarding/enrichment exists | FeLV | Exact local cat day-care lane policy | `VaccinePending` if service policy exists; otherwise `SpecialReview` |
| Grooming | Rabies and FVRCP if the location adopts the public PetSuites `all services` rule | FeLV; rabies-only local exception | Exact local cat grooming requirement | If policy says required: `VaccinePending`; if unknown: `SpecialReview` |
| Training | Unknown; not assumed for cats unless location offers it | Rabies, FVRCP as possible baseline if offered | Whether cat training exists | `SpecialReview` until policy exists |
| DaySpa / bathing | Rabies and FVRCP if offered and local policy adopts all-services rule | FeLV; rabies-only local exception | Exact local DaySpa/bathing policy | If policy says required: `VaccinePending`; if unknown: `SpecialReview` |

### Other species

`Species::Other` has no vaccine baseline in this product model.

- Required status: unknown.
- Optional status: unknown.
- Eligibility effect: `SpecialReview` with no vaccine auto-evaluation.
- Staff task: create/recommend a manager or care-policy review packet with source evidence and requested service.

## Validity and expiration handling

Date handling is high risk. The vaccine-document agent may suggest dates, but only reviewed dates can satisfy requirements.

Proposed validity rules to approve:

1. Prefer explicit expiration/due/next-due date from the document.
   - A vaccine is `current_for_service` only if the reviewed expiration/due date is on or after the service end timestamp in the location timezone.
   - For boarding, current-through checkout/end date is required, not merely current on upload date or check-in date.
   - For daycare/day play/grooming/training/DaySpa, current on service date is required unless policy defines a wider window.

2. Missing expiration/due date defaults to `unknown_validity`, not accepted.
   - Do not infer one-year or three-year duration from administered date unless the approved policy defines a product/source-specific fallback.
   - If fallback windows are approved later, they must be encoded by vaccine, species, jurisdiction/location if needed, source confidence, and administered date evidence.

3. Expired before the relevant service date means not eligible under that requirement.
   - Output: `expired_or_stale_vaccine` risk flag.
   - Booking mapping: `VaccinePending` and `MissingRequiredVaccine(<name>)` until updated proof or an approved exception exists.

4. Future-dated administered dates, future-dated expiration anomalies, impossible date order, missing year, ambiguous locale formats, or conflicting duplicate dates require review.
   - Output: `ambiguous_vaccine_date` or `conflicting_vaccine_fact` risk flag.
   - Booking mapping: `SpecialReview` if conflict is material; otherwise `VaccinePending` for missing accepted proof.

5. Grace periods are not assumed.
   - If a location allows a grace period after expiration or before a booster due date, it must be explicit in policy and must identify which services it applies to.

6. Canine influenza series handling requires approval.
   - Public PetSuites examples mention at least the first round of a two-part series for visits, but this must be location-approved before eligibility logic relies on it.
   - Until approved, partially completed CIV proof should be `needs_medical_document_review` / `SpecialReview`, not accepted or rejected automatically.

## Source and proof handling

Proposed source rule:

- Required vaccines must be documented by a licensed veterinarian or veterinary professional.
- The source rule is required in the policy snapshot and should map to the existing `source_must_be_licensed_vet` field until a richer source-verification enum exists.
- Customer uploads, staff scans, emails, screenshots, portal exports, or provider lookups are evidence carriers only. Upload success is not verification.

Source states:

| Source state | Meaning | Booking effect |
| --- | --- | --- |
| `verified_veterinary_source` | Human reviewer accepted clinic/veterinary source evidence under policy | Vaccine fact may satisfy policy if dates and identity also pass |
| `source_suggested_unverified` | Extractor found a clinic/vet candidate but no human acceptance exists | `VaccinePending` / `MedicalDocumentReview` |
| `source_missing_or_ambiguous` | No clear source or conflicting source evidence | `VaccinePending` with `unverified_veterinary_source` risk flag |
| `provider_record_unverified` | Gingr/provider record exists but raw source or provenance is not accepted | Review packet input only; no automatic acceptance |

## Booking eligibility mapping

The vaccine-document agent should return policy-comparison suggestions, not final eligibility changes. Booking or provider-state changes require a reviewed execution path.

Per pet and service, compute a suggested vaccine readiness bucket:

| Suggested bucket | Conditions | Reservation/status suggestion | Hard stops / flags |
| --- | --- | --- | --- |
| `vaccine_requirements_satisfied_for_reviewed_policy` | Every required vaccine for species/service has accepted pet identity, accepted source, accepted canonical name, and current date through the service window | May suggest `ready_for_staff_approval`; never directly confirm | No missing-vaccine hard stop; audit policy snapshot and reviewer |
| `vaccine_pending_missing_proof` | At least one required vaccine has no accepted proof | Suggest `VaccinePending` | `MissingRequiredVaccine(<name>)`, `missing_required_vaccine_proof` |
| `vaccine_pending_expired_or_stale` | Required vaccine proof exists but expires before service end/date or policy snapshot is stale | Suggest `VaccinePending` | `MissingRequiredVaccine(<name>)`, `expired_or_stale_vaccine` |
| `vaccine_pending_source_unverified` | Vaccine/date found but veterinary source is unverified and policy requires source verification | Suggest `VaccinePending` | `unverified_veterinary_source` |
| `special_review_conflict` | Pet identity, species, date, vaccine name, duplicate records, provider data, or policy version conflicts | Suggest `SpecialReview` | `conflicting_pet_identity`, `ambiguous_vaccine_date`, `provider_payload_unverified`, or specific conflict flag |
| `policy_unknown_for_service` | No vaccine policy row for species/service/location | Suggest `SpecialReview`; do not evaluate eligibility | `policy_snapshot_missing` / `service_policy_unknown` |
| `not_required_for_service` | Approved policy explicitly says vaccine is not required for this species/service | May omit vaccine hard stop for that service only | Audit policy snapshot; do not generalize to other services |

Important boundaries:

- `Completed` workflow status means extraction/comparison output was produced; it does not mean vaccines were accepted.
- The agent may draft internal tasks and staff review packets.
- The agent must not approve medical eligibility, change provider records, confirm bookings, reject bookings, or send customer-facing messages.
- Customer follow-up copy about missing/expired/rejected vaccine proof is draft-only and requires customer-message approval unless a later deterministic template path is approved.

## Review and audit requirements

A reviewed vaccine fact should record:

- pet id, customer id, reservation id if relevant, and location id;
- policy id/version/snapshot used;
- canonical vaccine name and accepted alias/source label;
- administered date if accepted;
- expiration/due/next-due date if accepted;
- source document/evidence id and page/span/image references where available;
- source verification state and reviewer/actor;
- accepted/rejected/superseded state;
- comparison result by service/species;
- audit event id and workflow event id.

Every rejection, conflict, exception, or manual override should record a `MedicalDocumentReview` or manager/admin approval trail as appropriate.

## Policy assumptions requiring approval

The policy owner should approve or edit these before implementation treats this as final:

1. Whether the baseline required dog vaccines for all services are Rabies, DHPP/DHLPP family, Bordetella, and Canine Influenza.
2. Whether grooming, training, and DaySpa/bathing require the same vaccine set as boarding/daycare or narrower service-specific sets.
3. Whether canine influenza is always required, location-specific, seasonal, or optional; whether first-dose-only in a two-part series is acceptable and for how long.
4. Whether DHPP, DHLPP, DA2PP, and DAPP are equivalent for policy purposes; whether leptospirosis is required or merely accepted as part of a combo label.
5. Whether the baseline required cat vaccines are Rabies and FVRCP for all cat services, and whether FeLV is ever required.
6. Whether the resort offers cat daycare/day boarding/training/DaySpa or only cat boarding/grooming at specific locations.
7. Whether proof must always be from a licensed veterinarian/veterinary professional and how source verification is evidenced.
8. Whether missing explicit expiration dates can ever be accepted using fallback validity windows; if yes, define fallback by vaccine/species/source.
9. Whether any grace periods, puppy/kitten schedules, medical exemptions, titers, local legal exceptions, or manager override paths exist.
10. Which staff roles can accept vaccine facts, reject proof, approve exceptions, or trigger customer follow-up.

## Minimal configuration shape

A future typed policy could be represented as rows like:

```text
vaccine_policy_id
policy_version
location_id
species
service
requirement_status: required | optional | not_required | unknown
canonical_vaccine_name
accepted_aliases[]
source_must_be_licensed_vet
validity_mode: explicit_expiration_required | approved_fallback_window | not_applicable
fallback_validity_days: optional, policy-approved only
grace_period_days: optional, policy-approved only
series_rule: complete_series_required | first_dose_temporarily_allowed | not_applicable | unknown
booking_effect_when_missing: VaccinePending | SpecialReview | no_block
review_gate: MedicalDocumentReview | ManagerApproval | none
```

Until this or an equivalent structure is approved, the workflow should preserve extracted facts as suggestions and keep booking eligibility behind human review.
