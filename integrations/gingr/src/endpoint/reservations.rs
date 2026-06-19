use super::{AnimalId, Date, DateRange, IsoDate, Limit, LocationId, Method, OwnerId, Request};

/// Reservation request builders whose filters stay tied to Gingr ids, dates, and status tokens for reconciliation.
pub mod reservation {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Provider reservation-type identifier used to classify boarding, daycare, grooming, training, or other Gingr service demand.
    pub struct TypeId(u64);

    impl TypeId {
        /// Wraps an already-observed Gingr identifier without claiming anything beyond provider provenance.
        pub const fn new(value: u64) -> Self {
            Self(value)
        }
    }

    impl core::fmt::Display for TypeId {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(formatter, "{}", self.0)
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    /// Request descriptor for Gingr reservation types, the provider lookup table behind service-line classification.
    pub struct Types {
        id: Option<TypeId>,
        active_only: Option<bool>,
    }

    impl Types {
        /// Starts a builder that makes each provider parameter explicit before request capture.
        pub fn builder() -> TypesBuilder {
            TypesBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Builder for reservation-type lookup parameters without asserting NVA service-line semantics.
    pub struct TypesBuilder {
        id: Option<TypeId>,
        active_only: Option<bool>,
    }

    impl TypesBuilder {
        /// Narrows the provider lookup to one Gingr identifier when supplied.
        pub fn id(mut self, id: TypeId) -> Self {
            self.id = Some(id);
            self
        }

        /// Restricts reservation-type results to active provider records.
        pub fn active_only(mut self, active_only: bool) -> Self {
            self.active_only = Some(active_only);
            self
        }

        /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
        pub fn build(self) -> Types {
            Types {
                id: self.id,
                active_only: self.active_only,
            }
        }
    }

    impl Request for Types {
        fn method(&self) -> Method {
            Method::Get
        }

        fn path(&self) -> &'static str {
            "/api/v1/reservation_types"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            let mut params = Vec::new();
            if let Some(id) = self.id {
                params.push(("id".to_owned(), id.to_string()));
            }
            if let Some(active_only) = self.active_only {
                params.push(("active_only".to_owned(), active_only.to_string()));
            }
            params
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Request descriptor for Gingr reservation widget data at a provider timestamp.
    pub struct WidgetData {
        timestamp: Date,
    }

    impl WidgetData {
        /// Starts a builder that makes each provider parameter explicit before request capture.
        pub fn builder() -> WidgetDataBuilder {
            WidgetDataBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Builder for the reservation-widget timestamp filter.
    pub struct WidgetDataBuilder {
        timestamp: Option<Date>,
    }

    impl WidgetDataBuilder {
        /// Sets the provider timestamp filter sent to the Gingr endpoint.
        pub fn timestamp(mut self, timestamp: Date) -> Self {
            self.timestamp = Some(timestamp);
            self
        }

        /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
        pub fn build(self) -> WidgetData {
            WidgetData {
                timestamp: self.timestamp.expect("WidgetData requires timestamp"),
            }
        }
    }

    impl Request for WidgetData {
        fn method(&self) -> Method {
            Method::Get
        }

        fn path(&self) -> &'static str {
            "/api/v1/reservation_widget_data"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            vec![("timestamp".to_owned(), self.timestamp.to_string())]
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    /// Optional provider-side reservation filters shared by reservation lookup endpoints.
    pub struct SearchFilters {
        from_date: Option<IsoDate>,
        to_date: Option<IsoDate>,
        reservation_type_ids: Vec<TypeId>,
        animal_ids: Vec<AnimalId>,
        cancelled_only: Option<bool>,
        confirmed_only: Option<bool>,
        completed_only: Option<bool>,
        limit: Option<Limit>,
    }

    impl SearchFilters {
        /// Starts a builder that makes each provider parameter explicit before request capture.
        pub fn builder() -> SearchFiltersBuilder {
            SearchFiltersBuilder::default()
        }

        pub(super) fn parameters(&self) -> Vec<(String, String)> {
            let mut params = Vec::new();
            if let Some(from_date) = self.from_date {
                params.push(("params[fromDate]".to_owned(), from_date.to_string()));
            }
            if let Some(to_date) = self.to_date {
                params.push(("params[toDate]".to_owned(), to_date.to_string()));
            }
            for id in &self.reservation_type_ids {
                params.push(("params[reservationTypeIds][]".to_owned(), id.to_string()));
            }
            for id in &self.animal_ids {
                params.push(("params[animalIds][]".to_owned(), id.to_string()));
            }
            if let Some(value) = self.cancelled_only {
                params.push(("params[cancelledOnly]".to_owned(), value.to_string()));
            }
            if let Some(value) = self.confirmed_only {
                params.push(("params[confirmedOnly]".to_owned(), value.to_string()));
            }
            if let Some(value) = self.completed_only {
                params.push(("params[completedOnly]".to_owned(), value.to_string()));
            }
            if let Some(limit) = self.limit {
                params.push(("params[limit]".to_owned(), limit.to_string()));
            }
            params
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Builder for provider reservation filters such as date, status flags, type IDs, and animal IDs.
    pub struct SearchFiltersBuilder {
        filters: SearchFilters,
    }

    impl SearchFiltersBuilder {
        /// Sets the inclusive provider start date sent to Gingr.
        pub fn from_date(mut self, date: IsoDate) -> Self {
            self.filters.from_date = Some(date);
            self
        }

        /// Sets the inclusive provider end date sent to Gingr.
        pub fn to_date(mut self, date: IsoDate) -> Self {
            self.filters.to_date = Some(date);
            self
        }

        /// Adds a Gingr reservation-type identifier as a provider filter.
        pub fn reservation_type_id(mut self, id: TypeId) -> Self {
            self.filters.reservation_type_ids.push(id);
            self
        }

        /// Adds a Gingr animal identifier as a provider filter.
        pub fn animal_id(mut self, id: AnimalId) -> Self {
            self.filters.animal_ids.push(id);
            self
        }

        /// Requests only provider records Gingr marks as cancelled.
        pub fn cancelled_only(mut self, value: bool) -> Self {
            self.filters.cancelled_only = Some(value);
            self
        }

        /// Requests only provider records Gingr marks as confirmed.
        pub fn confirmed_only(mut self, value: bool) -> Self {
            self.filters.confirmed_only = Some(value);
            self
        }

        /// Requests only provider records Gingr marks as completed.
        pub fn completed_only(mut self, value: bool) -> Self {
            self.filters.completed_only = Some(value);
            self
        }

        /// Sets the provider result limit so automation does not imply unbounded source coverage.
        pub fn limit(mut self, limit: Limit) -> Self {
            self.filters.limit = Some(limit);
            self
        }

        /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
        pub fn build(self) -> SearchFilters {
            self.filters
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for the `/api/v1/reservations` endpoint used as source evidence for occupancy and check-in workflows.
pub struct Reservations {
    checked_in: bool,
    range: Option<DateRange>,
    location: Option<LocationId>,
}

impl Reservations {
    /// Starts a reservations request for currently checked-in stays.
    pub fn checked_in() -> Builder {
        Builder {
            checked_in: true,
            range: None,
            location: None,
        }
    }

    /// Starts a reservations request for an inclusive provider date range.
    pub fn for_range(range: DateRange) -> Builder {
        Builder {
            checked_in: false,
            range: Some(range),
            location: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Builder for the primary reservations request, including checked-in/range mode and optional location scope.
pub struct Builder {
    checked_in: bool,
    range: Option<DateRange>,
    location: Option<LocationId>,
}

impl Builder {
    /// Scopes the Gingr request to a provider location identifier.
    pub fn location(mut self, location: LocationId) -> Self {
        self.location = Some(location);
        self
    }

    /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
    pub fn build(self) -> Reservations {
        Reservations {
            checked_in: self.checked_in,
            range: self.range,
            location: self.location,
        }
    }
}

impl Request for Reservations {
    fn method(&self) -> Method {
        Method::Post
    }

    fn path(&self) -> &'static str {
        "/api/v1/reservations"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        let mut params = vec![("checked_in".to_owned(), self.checked_in.to_string())];
        if let Some(range) = self.range {
            params.push(("start_date".to_owned(), range.start().to_string()));
            params.push(("end_date".to_owned(), range.end().to_string()));
        }
        if let Some(location) = self.location {
            params.push(("location_id".to_owned(), location.to_string()));
        }
        params
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Typed Gingr/provider codes for restrict to values.
pub enum RestrictTo {
    /// Restricts reservation lookup to pending Gingr requests.
    PendingRequests,
    /// Restricts reservation lookup to currently checked-in reservations.
    CurrentlyCheckedIn,
    /// Restricts reservation lookup to future reservations.
    Future,
    /// Restricts reservation lookup to past reservations.
    Past,
    /// Restricts reservation lookup to wait-listed reservations.
    WaitListed,
}

impl core::fmt::Display for RestrictTo {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let value = match self {
            Self::PendingRequests => "pending_requests",
            Self::CurrentlyCheckedIn => "currently_checked_in",
            Self::Future => "future",
            Self::Past => "past",
            Self::WaitListed => "wait_listed",
        };
        formatter.write_str(value)
    }
}

/// Reservation lookup endpoints keyed by related Gingr owner or animal records.
pub mod by {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Request descriptor for reservations related to one Gingr animal record.
    pub struct Animal {
        animal_id: AnimalId,
        restrict_to: Option<RestrictTo>,
        filters: Option<reservation::SearchFilters>,
    }

    impl Animal {
        /// Notes that Gingr scopes this lookup to the API user's current location.
        pub const LOCATION_SCOPE_CAVEAT: &'static str = "Reservation data for this endpoint is only pulled for the location the API user is currently logged into.";

        /// Starts a builder that makes each provider parameter explicit before request capture.
        pub fn builder() -> AnimalBuilder {
            AnimalBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Builder for animal-keyed reservation lookups and their provider filters.
    pub struct AnimalBuilder {
        animal_id: Option<AnimalId>,
        restrict_to: Option<RestrictTo>,
        filters: Option<reservation::SearchFilters>,
    }

    impl AnimalBuilder {
        /// Adds a Gingr animal identifier as a provider filter.
        pub fn animal_id(mut self, id: AnimalId) -> Self {
            self.animal_id = Some(id);
            self
        }

        /// Applies Gingr reservation-scope tokens such as future, past, or wait-listed.
        pub fn restrict_to(mut self, restrict_to: RestrictTo) -> Self {
            self.restrict_to = Some(restrict_to);
            self
        }

        /// Adds nested Gingr reservation search filters to the lookup request.
        pub fn filter(mut self, filters: reservation::SearchFilters) -> Self {
            self.filters = Some(filters);
            self
        }

        /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
        pub fn build(self) -> Animal {
            Animal {
                animal_id: self.animal_id.expect("Animal requires animal_id"),
                restrict_to: self.restrict_to,
                filters: self.filters,
            }
        }
    }

    impl Request for Animal {
        fn method(&self) -> Method {
            Method::Post
        }

        fn path(&self) -> &'static str {
            "/api/v1/reservations_by_animal"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            let mut params = vec![("id".to_owned(), self.animal_id.to_string())];
            if let Some(restrict_to) = self.restrict_to {
                params.push(("restrict_to".to_owned(), restrict_to.to_string()));
            }
            if let Some(filters) = &self.filters {
                params.extend(filters.parameters());
            }
            params
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Request descriptor for reservations related to one Gingr owner/customer record.
    pub struct Owner {
        owner_id: OwnerId,
        restrict_to: Option<RestrictTo>,
        filters: Option<reservation::SearchFilters>,
    }

    impl Owner {
        /// Notes that Gingr scopes this lookup to the API user's current location.
        pub const LOCATION_SCOPE_CAVEAT: &'static str = Animal::LOCATION_SCOPE_CAVEAT;

        /// Starts a builder that makes each provider parameter explicit before request capture.
        pub fn builder() -> OwnerBuilder {
            OwnerBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Builder for owner-keyed reservation lookups and their provider filters.
    pub struct OwnerBuilder {
        owner_id: Option<OwnerId>,
        restrict_to: Option<RestrictTo>,
        filters: Option<reservation::SearchFilters>,
    }

    impl OwnerBuilder {
        /// Narrows the provider request to one Gingr owner/customer identifier.
        pub fn owner_id(mut self, id: OwnerId) -> Self {
            self.owner_id = Some(id);
            self
        }

        /// Applies Gingr reservation-scope tokens such as future, past, or wait-listed.
        pub fn restrict_to(mut self, restrict_to: RestrictTo) -> Self {
            self.restrict_to = Some(restrict_to);
            self
        }

        /// Adds nested Gingr reservation search filters to the lookup request.
        pub fn filter(mut self, filters: reservation::SearchFilters) -> Self {
            self.filters = Some(filters);
            self
        }

        /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
        pub fn build(self) -> Owner {
            Owner {
                owner_id: self.owner_id.expect("Owner requires owner_id"),
                restrict_to: self.restrict_to,
                filters: self.filters,
            }
        }
    }

    impl Request for Owner {
        fn method(&self) -> Method {
            Method::Post
        }

        fn path(&self) -> &'static str {
            "/api/v1/reservations_by_owner"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            let mut params = vec![("id".to_owned(), self.owner_id.to_string())];
            if let Some(restrict_to) = self.restrict_to {
                params.push(("restrict_to".to_owned(), restrict_to.to_string()));
            }
            if let Some(filters) = &self.filters {
                params.extend(filters.parameters());
            }
            params
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Positive future-minute window used by Gingr back-of-house operational views.
pub struct MinutesFuture(u64);

impl MinutesFuture {
    /// Validates a positive future-minute window used to ask Gingr for near-term back-of-house records.
    pub fn new(value: u64) -> super::Result<Self> {
        if value == 0 {
            return Err(super::Error::InvalidPositiveInteger { value });
        }
        Ok(Self(value))
    }
}

impl core::fmt::Display for MinutesFuture {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for Gingr back-of-house views used as raw operational evidence for near-term labor planning.
pub struct BackOfHouse {
    location: LocationId,
    reservation_type_ids: Vec<reservation::TypeId>,
    minutes_future: Option<MinutesFuture>,
    full_day: Option<bool>,
}

impl BackOfHouse {
    /// Starts a builder that makes each provider parameter explicit before request capture.
    pub fn builder() -> BackOfHouseBuilder {
        BackOfHouseBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
/// Builder for back-of-house location, reservation-type, time-window, and full-day provider filters.
pub struct BackOfHouseBuilder {
    location: Option<LocationId>,
    reservation_type_ids: Vec<reservation::TypeId>,
    minutes_future: Option<MinutesFuture>,
    full_day: Option<bool>,
}

impl BackOfHouseBuilder {
    /// Scopes the Gingr request to a provider location identifier.
    pub fn location(mut self, location: LocationId) -> Self {
        self.location = Some(location);
        self
    }

    /// Adds a Gingr reservation-type identifier as a provider filter.
    pub fn reservation_type_id(mut self, id: reservation::TypeId) -> Self {
        self.reservation_type_ids.push(id);
        self
    }

    /// Limits back-of-house results to a future provider time window.
    pub fn minutes_future(mut self, minutes: MinutesFuture) -> Self {
        self.minutes_future = Some(minutes);
        self
    }

    /// Requests Gingr full-day back-of-house records when the provider supports that flag.
    pub fn full_day(mut self, full_day: bool) -> Self {
        self.full_day = Some(full_day);
        self
    }

    /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
    pub fn build(self) -> BackOfHouse {
        BackOfHouse {
            location: self.location.expect("BackOfHouse requires location"),
            reservation_type_ids: self.reservation_type_ids,
            minutes_future: self.minutes_future,
            full_day: self.full_day,
        }
    }
}

impl Request for BackOfHouse {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/back_of_house"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        let mut params = vec![("location_id".to_owned(), self.location.to_string())];
        for id in &self.reservation_type_ids {
            params.push(("type_ids[]".to_owned(), id.to_string()));
        }
        if let Some(minutes) = self.minutes_future {
            params.push(("mins_future".to_owned(), minutes.to_string()));
        }
        if let Some(full_day) = self.full_day {
            params.push(("full_day".to_owned(), full_day.to_string()));
        }
        params
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for Gingr service discovery by reservation type; DTO mapping is intentionally not guaranteed here.
pub struct GetServicesByType {
    type_id: reservation::TypeId,
    location: Option<LocationId>,
}

impl GetServicesByType {
    /// Builds a service-discovery request for one Gingr reservation type without promising a mapped service DTO.
    pub fn new(type_id: reservation::TypeId) -> Self {
        Self {
            type_id,
            location: None,
        }
    }

    /// Scopes the Gingr request to a provider location identifier.
    pub fn location(mut self, location: LocationId) -> Self {
        self.location = Some(location);
        self
    }
}

impl Request for GetServicesByType {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/get_services_by_type"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        let mut params = vec![("type_id".to_owned(), self.type_id.to_string())];
        if let Some(location) = self.location {
            params.push(("location_id".to_owned(), location.to_string()));
        }
        params
    }
}
