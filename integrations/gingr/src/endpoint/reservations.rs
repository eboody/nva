use super::{AnimalId, Date, DateRange, IsoDate, Limit, LocationId, Method, OwnerId, Request};

pub mod reservation {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct TypeId(u64);

    impl TypeId {
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
    pub struct Types {
        id: Option<TypeId>,
        active_only: Option<bool>,
    }

    impl Types {
        pub fn builder() -> TypesBuilder {
            TypesBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct TypesBuilder {
        id: Option<TypeId>,
        active_only: Option<bool>,
    }

    impl TypesBuilder {
        pub fn id(mut self, id: TypeId) -> Self {
            self.id = Some(id);
            self
        }

        pub fn active_only(mut self, active_only: bool) -> Self {
            self.active_only = Some(active_only);
            self
        }

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
    pub struct WidgetData {
        timestamp: Date,
    }

    impl WidgetData {
        pub fn builder() -> WidgetDataBuilder {
            WidgetDataBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct WidgetDataBuilder {
        timestamp: Option<Date>,
    }

    impl WidgetDataBuilder {
        pub fn timestamp(mut self, timestamp: Date) -> Self {
            self.timestamp = Some(timestamp);
            self
        }

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
    pub struct SearchFiltersBuilder {
        filters: SearchFilters,
    }

    impl SearchFiltersBuilder {
        pub fn from_date(mut self, date: IsoDate) -> Self {
            self.filters.from_date = Some(date);
            self
        }

        pub fn to_date(mut self, date: IsoDate) -> Self {
            self.filters.to_date = Some(date);
            self
        }

        pub fn reservation_type_id(mut self, id: TypeId) -> Self {
            self.filters.reservation_type_ids.push(id);
            self
        }

        pub fn animal_id(mut self, id: AnimalId) -> Self {
            self.filters.animal_ids.push(id);
            self
        }

        pub fn cancelled_only(mut self, value: bool) -> Self {
            self.filters.cancelled_only = Some(value);
            self
        }

        pub fn confirmed_only(mut self, value: bool) -> Self {
            self.filters.confirmed_only = Some(value);
            self
        }

        pub fn completed_only(mut self, value: bool) -> Self {
            self.filters.completed_only = Some(value);
            self
        }

        pub fn limit(mut self, limit: Limit) -> Self {
            self.filters.limit = Some(limit);
            self
        }

        pub fn build(self) -> SearchFilters {
            self.filters
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reservations {
    checked_in: bool,
    range: Option<DateRange>,
    location: Option<LocationId>,
}

impl Reservations {
    pub fn checked_in() -> Builder {
        Builder {
            checked_in: true,
            range: None,
            location: None,
        }
    }

    pub fn for_range(range: DateRange) -> Builder {
        Builder {
            checked_in: false,
            range: Some(range),
            location: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Builder {
    checked_in: bool,
    range: Option<DateRange>,
    location: Option<LocationId>,
}

impl Builder {
    pub fn location(mut self, location: LocationId) -> Self {
        self.location = Some(location);
        self
    }

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
pub enum RestrictTo {
    PendingRequests,
    CurrentlyCheckedIn,
    Future,
    Past,
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

pub mod by {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Animal {
        animal_id: AnimalId,
        restrict_to: Option<RestrictTo>,
        filters: Option<reservation::SearchFilters>,
    }

    impl Animal {
        pub const LOCATION_SCOPE_CAVEAT: &'static str = "Reservation data for this endpoint is only pulled for the location the API user is currently logged into.";

        pub fn builder() -> AnimalBuilder {
            AnimalBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct AnimalBuilder {
        animal_id: Option<AnimalId>,
        restrict_to: Option<RestrictTo>,
        filters: Option<reservation::SearchFilters>,
    }

    impl AnimalBuilder {
        pub fn animal_id(mut self, id: AnimalId) -> Self {
            self.animal_id = Some(id);
            self
        }

        pub fn restrict_to(mut self, restrict_to: RestrictTo) -> Self {
            self.restrict_to = Some(restrict_to);
            self
        }

        pub fn filter(mut self, filters: reservation::SearchFilters) -> Self {
            self.filters = Some(filters);
            self
        }

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
    pub struct Owner {
        owner_id: OwnerId,
        restrict_to: Option<RestrictTo>,
        filters: Option<reservation::SearchFilters>,
    }

    impl Owner {
        pub const LOCATION_SCOPE_CAVEAT: &'static str = Animal::LOCATION_SCOPE_CAVEAT;

        pub fn builder() -> OwnerBuilder {
            OwnerBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct OwnerBuilder {
        owner_id: Option<OwnerId>,
        restrict_to: Option<RestrictTo>,
        filters: Option<reservation::SearchFilters>,
    }

    impl OwnerBuilder {
        pub fn owner_id(mut self, id: OwnerId) -> Self {
            self.owner_id = Some(id);
            self
        }

        pub fn restrict_to(mut self, restrict_to: RestrictTo) -> Self {
            self.restrict_to = Some(restrict_to);
            self
        }

        pub fn filter(mut self, filters: reservation::SearchFilters) -> Self {
            self.filters = Some(filters);
            self
        }

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
pub struct MinutesFuture(u64);

impl MinutesFuture {
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
pub struct BackOfHouse {
    location: LocationId,
    reservation_type_ids: Vec<reservation::TypeId>,
    minutes_future: Option<MinutesFuture>,
    full_day: Option<bool>,
}

impl BackOfHouse {
    pub fn builder() -> BackOfHouseBuilder {
        BackOfHouseBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct BackOfHouseBuilder {
    location: Option<LocationId>,
    reservation_type_ids: Vec<reservation::TypeId>,
    minutes_future: Option<MinutesFuture>,
    full_day: Option<bool>,
}

impl BackOfHouseBuilder {
    pub fn location(mut self, location: LocationId) -> Self {
        self.location = Some(location);
        self
    }

    pub fn reservation_type_id(mut self, id: reservation::TypeId) -> Self {
        self.reservation_type_ids.push(id);
        self
    }

    pub fn minutes_future(mut self, minutes: MinutesFuture) -> Self {
        self.minutes_future = Some(minutes);
        self
    }

    pub fn full_day(mut self, full_day: bool) -> Self {
        self.full_day = Some(full_day);
        self
    }

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
pub struct GetServicesByType {
    type_id: reservation::TypeId,
    location: Option<LocationId>,
}

impl GetServicesByType {
    pub fn new(type_id: reservation::TypeId) -> Self {
        Self {
            type_id,
            location: None,
        }
    }

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
