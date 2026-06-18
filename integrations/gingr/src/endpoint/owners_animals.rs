use super::{AnimalId, FormId, Method, OwnerId, Request, ReservationId, non_empty_text};

#[derive(Clone, Debug, PartialEq, Eq)]
/// Typed Gingr request/response value for provider where clause.
pub struct ProviderWhereClause {
    field: String,
    value: String,
}

impl ProviderWhereClause {
    /// Builds the validated storage wrapper for a known-good value.
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
/// Typed Gingr request/response value for owners.
pub struct Owners {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl Owners {
    /// Starts a typed builder for this Gingr endpoint request.
    pub fn builder() -> OwnersBuilder {
        OwnersBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
/// Typed Gingr request/response value for owners builder.
pub struct OwnersBuilder {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl OwnersBuilder {
    /// Applies a raw Gingr where clause while keeping it typed at the boundary.
    pub fn provider_where_clause(mut self, clause: ProviderWhereClause) -> Self {
        self.provider_where_clauses.push(clause);
        self
    }

    /// Builds the typed Gingr request after all parameters have been validated.
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
/// Typed Gingr request/response value for animals.
pub struct Animals {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl Animals {
    /// Starts a typed builder for this Gingr endpoint request.
    pub fn builder() -> AnimalsBuilder {
        AnimalsBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
/// Typed Gingr request/response value for animals builder.
pub struct AnimalsBuilder {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl AnimalsBuilder {
    /// Applies a raw Gingr where clause while keeping it typed at the boundary.
    pub fn provider_where_clause(mut self, clause: ProviderWhereClause) -> Self {
        self.provider_where_clauses.push(clause);
        self
    }

    /// Builds the typed Gingr request after all parameters have been validated.
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
/// Typed Gingr request/response value for sensitive lookup.
pub struct SensitiveLookup(String);

impl SensitiveLookup {
    /// Builds the validated storage wrapper for a known-good value.
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
/// Typed Gingr request/response value for owner.
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
/// Typed Gingr request/response value for form.
pub struct Form {
    kind: FormKind,
}

impl Form {
    /// Creates the wrapper from an already validated value.
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

/// Gingr custom field endpoint boundary with provider parameters kept explicit.
pub mod custom_field {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Typed Gingr request/response value for name.
    pub struct Name(String);

    impl Name {
        /// Builds the validated storage wrapper for a known-good value.
        pub fn new(value: impl Into<String>) -> super::super::Result<Self> {
            Ok(Self(non_empty_text(value)?))
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Typed Gingr request/response value for search.
    pub struct Search {
        form: FormKind,
        field_name: Name,
        search: SensitiveLookup,
    }

    impl Search {
        /// Starts a typed builder for this Gingr endpoint request.
        pub fn builder() -> SearchBuilder {
            SearchBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Typed Gingr request/response value for search builder.
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

        /// Builds the typed Gingr request after all parameters have been validated.
        pub fn build(self) -> Search {
            Search {
                form: self.form.expect("Search requires form"),
                field_name: self.field_name.expect("Search requires field_name"),
                search: self.search.expect("Search requires search"),
            }
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
/// Typed Gingr request/response value for animal care info.
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
