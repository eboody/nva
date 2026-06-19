use super::{
    AnimalId, Error, FormId, Method, OwnerId, Request, ReservationId, Result, non_empty_text,
};

#[derive(Clone, Debug, PartialEq, Eq)]
/// Opaque provider `where` clause string passed to Gingr owner/animal search endpoints.
pub struct ProviderWhereClause {
    field: String,
    value: String,
}

impl ProviderWhereClause {
    /// Captures the provider field/value filter exactly as Gingr expects it so reviewers can audit the lookup.
    pub fn new(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            value: value.into(),
        }
    }

    fn parameter_pair(&self) -> (String, String) {
        (format!("params[{}]", self.field), self.value.clone())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Request descriptor for Gingr owner/customer search results.
pub struct Owners {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl Owners {
    /// Starts a builder that makes each provider parameter explicit before request capture.
    pub fn builder() -> OwnersBuilder {
        OwnersBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
/// Builder for provider-side owner search filters and explicit sensitive lookup values.
pub struct OwnersBuilder {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl OwnersBuilder {
    /// Applies a raw Gingr where clause while keeping the provider filter visible for audit.
    pub fn provider_where_clause(mut self, clause: ProviderWhereClause) -> Self {
        self.provider_where_clauses.push(clause);
        self
    }

    /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
    pub fn build(self) -> Owners {
        Owners {
            provider_where_clauses: self.provider_where_clauses,
        }
    }
}

impl Request for Owners {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/owners"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        self.provider_where_clauses
            .iter()
            .map(ProviderWhereClause::parameter_pair)
            .collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Request descriptor for Gingr animal/pet search results.
pub struct Animals {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl Animals {
    /// Starts a builder that makes each provider parameter explicit before request capture.
    pub fn builder() -> AnimalsBuilder {
        AnimalsBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
/// Builder for provider-side animal search filters and explicit sensitive lookup values.
pub struct AnimalsBuilder {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl AnimalsBuilder {
    /// Applies a raw Gingr where clause while keeping the provider filter visible for audit.
    pub fn provider_where_clause(mut self, clause: ProviderWhereClause) -> Self {
        self.provider_where_clauses.push(clause);
        self
    }

    /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
    pub fn build(self) -> Animals {
        Animals {
            provider_where_clauses: self.provider_where_clauses,
        }
    }
}

impl Request for Animals {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/animals"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        self.provider_where_clauses
            .iter()
            .map(ProviderWhereClause::parameter_pair)
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Sensitive lookup term such as email or phone that must be redacted from request diagnostics.
pub struct SensitiveLookup(String);

impl SensitiveLookup {
    /// Captures the provider field/value filter exactly as Gingr expects it so reviewers can audit the lookup.
    pub fn new(value: impl Into<String>) -> super::Result<Self> {
        Ok(Self(non_empty_text(value)?))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Typed Gingr owner lookup strategies, separating sensitive search inputs from normal IDs.
pub enum OwnerLookup {
    /// Lookup uses a Gingr owner identifier.
    OwnerId(OwnerId),
    /// Lookup uses a Gingr animal identifier.
    AnimalId(AnimalId),
    /// Lookup uses a Gingr reservation identifier.
    ReservationId(ReservationId),
    /// Lookup uses an owner phone number and should be treated as sensitive.
    Phone(SensitiveLookup),
    /// Lookup uses an owner email address and should be treated as sensitive.
    Email(SensitiveLookup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for one Gingr owner/customer record by provider ID.
pub struct Owner {
    lookup: OwnerLookup,
}

impl Owner {
    /// Builds an owner lookup by Gingr owner identifier.
    pub fn by_id(id: OwnerId) -> Self {
        Self {
            lookup: OwnerLookup::OwnerId(id),
        }
    }

    /// Builds an owner lookup by Gingr animal identifier.
    pub fn by_animal(id: AnimalId) -> Self {
        Self {
            lookup: OwnerLookup::AnimalId(id),
        }
    }

    /// Builds an owner lookup by Gingr reservation identifier.
    pub fn by_reservation(id: ReservationId) -> Self {
        Self {
            lookup: OwnerLookup::ReservationId(id),
        }
    }

    /// Builds an owner lookup by phone number.
    pub fn by_phone(phone: SensitiveLookup) -> Self {
        Self {
            lookup: OwnerLookup::Phone(phone),
        }
    }

    /// Builds an owner lookup by email address.
    pub fn by_email(email: SensitiveLookup) -> Self {
        Self {
            lookup: OwnerLookup::Email(email),
        }
    }
}

impl Request for Owner {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/owner"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        match &self.lookup {
            OwnerLookup::OwnerId(id) => vec![("id".to_owned(), id.to_string())],
            OwnerLookup::AnimalId(id) => vec![("animal_id".to_owned(), id.to_string())],
            OwnerLookup::ReservationId(id) => vec![("reservation_id".to_owned(), id.to_string())],
            OwnerLookup::Phone(phone) => vec![("phone".to_owned(), phone.as_str().to_owned())],
            OwnerLookup::Email(email) => vec![("email".to_owned(), email.as_str().to_owned())],
        }
    }

    fn sensitive_parameter_names(&self) -> &'static [&'static str] {
        match self.lookup {
            OwnerLookup::Phone(_) => &["phone"],
            OwnerLookup::Email(_) => &["email"],
            _ => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Gingr owner/animal form classes supported by form search endpoints.
pub enum FormKind {
    /// Provider value refers to a Gingr owner/customer.
    Owner,
    /// Provider value refers to a Gingr animal/pet.
    Animal,
}

impl FormKind {
    /// Returns the Gingr form identifier associated with this form class.
    pub const fn form_id(self) -> FormId {
        match self {
            Self::Owner => FormId::new(1),
            Self::Animal => FormId::new(2),
        }
    }
}

impl core::fmt::Display for FormKind {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let value = match self {
            Self::Owner => "owner_form",
            Self::Animal => "animal_form",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for a Gingr form definition by provider form ID.
pub struct Form {
    kind: FormKind,
}

impl Form {
    /// Wraps an already-observed Gingr identifier without claiming anything beyond provider provenance.
    pub const fn new(kind: FormKind) -> Self {
        Self { kind }
    }
}

impl Request for Form {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/forms/get_form"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        vec![("form".to_owned(), self.kind.to_string())]
    }
}

/// Gingr custom-field search requests where field names and text values remain reviewable evidence.
pub mod custom_field {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Non-empty provider custom-field name used for Gingr custom-field search.
    pub struct Name(String);

    impl Name {
        /// Captures the provider field/value filter exactly as Gingr expects it so reviewers can audit the lookup.
        pub fn new(value: impl Into<String>) -> super::super::Result<Self> {
            Ok(Self(non_empty_text(value)?))
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Request descriptor for Gingr custom-field search across provider owner or animal fields.
    pub struct Search {
        form: FormKind,
        field_name: Name,
        search: SensitiveLookup,
    }

    impl Search {
        /// Starts a builder that makes each provider parameter explicit before request capture.
        pub fn builder() -> SearchBuilder {
            SearchBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Builder for custom-field searches that keeps field name, text value, and optional location explicit.
    pub struct SearchBuilder {
        form: Option<FormKind>,
        field_name: Option<Name>,
        search: Option<SensitiveLookup>,
    }

    impl SearchBuilder {
        /// Selects the Gingr owner/animal form to search.
        pub fn form(mut self, form: FormKind) -> Self {
            self.form = Some(form);
            self
        }

        /// Selects the provider form field being searched.
        pub fn field_name(mut self, field_name: Name) -> Self {
            self.field_name = Some(field_name);
            self
        }

        /// Sets the provider search string for form lookup.
        pub fn search(mut self, search: SensitiveLookup) -> Self {
            self.search = Some(search);
            self
        }

        /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
        pub fn build(self) -> Result<Search> {
            Ok(Search {
                form: self
                    .form
                    .ok_or(Error::MissingRequiredParameter { parameter: "form" })?,
                field_name: self.field_name.ok_or(Error::MissingRequiredParameter {
                    parameter: "field_name",
                })?,
                search: self.search.ok_or(Error::MissingRequiredParameter {
                    parameter: "search",
                })?,
            })
        }
    }

    impl Request for Search {
        fn method(&self) -> Method {
            Method::Get
        }

        fn path(&self) -> &'static str {
            "/api/v1/custom_field_search"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            vec![
                ("form_id".to_owned(), self.form.form_id().to_string()),
                ("field_name".to_owned(), self.field_name.0.clone()),
                ("search".to_owned(), self.search.as_str().to_owned()),
            ]
        }

        fn sensitive_parameter_names(&self) -> &'static [&'static str] {
            &["search"]
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for Gingr feeding or medication info tied to one animal record.
pub struct AnimalCareInfo {
    path: &'static str,
    animal_id: AnimalId,
}

impl AnimalCareInfo {
    /// Builds an animal-care endpoint request for feeding instructions.
    pub fn feeding(animal_id: AnimalId) -> Self {
        Self {
            path: "/api/v1/get_feeding_info",
            animal_id,
        }
    }

    /// Builds an animal-care endpoint request for medication instructions.
    pub fn medication(animal_id: AnimalId) -> Self {
        Self {
            path: "/api/v1/get_medication_info",
            animal_id,
        }
    }
}

impl Request for AnimalCareInfo {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        self.path
    }

    fn parameters(&self) -> Vec<(String, String)> {
        vec![("animal_id".to_owned(), self.animal_id.to_string())]
    }
}
