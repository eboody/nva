use super::{AnimalId, Method, OwnerId, Request, ReservationId, non_empty_text};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderWhereClause {
    field: String,
    value: String,
}

impl ProviderWhereClause {
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
pub struct Owners {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl Owners {
    pub fn builder() -> OwnersBuilder {
        OwnersBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct OwnersBuilder {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl OwnersBuilder {
    pub fn provider_where_clause(mut self, clause: ProviderWhereClause) -> Self {
        self.provider_where_clauses.push(clause);
        self
    }

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
pub struct Animals {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl Animals {
    pub fn builder() -> AnimalsBuilder {
        AnimalsBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct AnimalsBuilder {
    provider_where_clauses: Vec<ProviderWhereClause>,
}

impl AnimalsBuilder {
    pub fn provider_where_clause(mut self, clause: ProviderWhereClause) -> Self {
        self.provider_where_clauses.push(clause);
        self
    }

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
pub struct SensitiveLookup(String);

impl SensitiveLookup {
    pub fn new(value: impl Into<String>) -> super::Result<Self> {
        Ok(Self(non_empty_text(value)?))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerLookup {
    OwnerId(OwnerId),
    AnimalId(AnimalId),
    ReservationId(ReservationId),
    Phone(SensitiveLookup),
    Email(SensitiveLookup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Owner {
    lookup: OwnerLookup,
}

impl Owner {
    pub fn by_id(id: OwnerId) -> Self {
        Self {
            lookup: OwnerLookup::OwnerId(id),
        }
    }

    pub fn by_animal(id: AnimalId) -> Self {
        Self {
            lookup: OwnerLookup::AnimalId(id),
        }
    }

    pub fn by_reservation(id: ReservationId) -> Self {
        Self {
            lookup: OwnerLookup::ReservationId(id),
        }
    }

    pub fn by_phone(phone: SensitiveLookup) -> Self {
        Self {
            lookup: OwnerLookup::Phone(phone),
        }
    }

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
pub enum FormKind {
    Owner,
    Animal,
}

impl FormKind {
    pub const fn form_id(self) -> u64 {
        match self {
            Self::Owner => 1,
            Self::Animal => 2,
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
pub struct Form {
    kind: FormKind,
}

impl Form {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomFieldName(String);

impl CustomFieldName {
    pub fn new(value: impl Into<String>) -> super::Result<Self> {
        Ok(Self(non_empty_text(value)?))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomFieldSearch {
    form: FormKind,
    field_name: CustomFieldName,
    search: SensitiveLookup,
}

impl CustomFieldSearch {
    pub fn builder() -> CustomFieldSearchBuilder {
        CustomFieldSearchBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CustomFieldSearchBuilder {
    form: Option<FormKind>,
    field_name: Option<CustomFieldName>,
    search: Option<SensitiveLookup>,
}

impl CustomFieldSearchBuilder {
    pub fn form(mut self, form: FormKind) -> Self {
        self.form = Some(form);
        self
    }

    pub fn field_name(mut self, field_name: CustomFieldName) -> Self {
        self.field_name = Some(field_name);
        self
    }

    pub fn search(mut self, search: SensitiveLookup) -> Self {
        self.search = Some(search);
        self
    }

    pub fn build(self) -> CustomFieldSearch {
        CustomFieldSearch {
            form: self.form.expect("CustomFieldSearch requires form"),
            field_name: self
                .field_name
                .expect("CustomFieldSearch requires field_name"),
            search: self.search.expect("CustomFieldSearch requires search"),
        }
    }
}

impl Request for CustomFieldSearch {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimalCareInfo {
    path: &'static str,
    animal_id: AnimalId,
}

impl AnimalCareInfo {
    pub fn feeding(animal_id: AnimalId) -> Self {
        Self {
            path: "/api/v1/get_feeding_info",
            animal_id,
        }
    }

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
