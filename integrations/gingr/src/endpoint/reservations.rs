use super::{
    AnimalId, Date, DateRange, Error, IsoDate, Limit, LocationId, Method, OwnerId, Request, Result,
};

/// Reservation request builders whose filters stay tied to Gingr ids, dates, and status tokens for reconciliation.
pub mod reservation {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Display)]
    /// Provider reservation-type identifier used to classify boarding, daycare, grooming, training, or other Gingr service demand.
    pub struct TypeId(u64);

    impl TypeId {}

    #[derive(Clone, Debug, Default, PartialEq, Eq, bon::Builder)]
    /// Request descriptor for Gingr reservation types, the provider lookup table behind service-line classification.
    pub struct Types {
        id: Option<TypeId>,
        active_only: Option<bool>,
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

    #[derive(Clone, Debug, PartialEq, Eq, bon::Builder)]
    /// Request descriptor for Gingr reservation widget data at a provider timestamp.
    pub struct WidgetData {
        timestamp: Date,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for the `/api/v1/reservations` endpoint used as source evidence for occupancy and check-in workflows.
pub struct Reservations {
    checked_in: bool,
    range: Option<DateRange>,
    location: Option<LocationId>,
}

impl Reservations {}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Builder for the primary reservations request, including checked-in/range mode and optional location scope.
pub struct Builder {
    checked_in: bool,
    range: Option<DateRange>,
    location: Option<LocationId>,
}

impl Builder {}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Display)]
/// Typed Gingr/provider codes for restrict to values.
pub enum RestrictTo {
    /// Restricts reservation lookup to pending Gingr requests.
    #[display("pending_requests")]
    PendingRequests,
    /// Restricts reservation lookup to currently checked-in reservations.
    #[display("currently_checked_in")]
    CurrentlyCheckedIn,
    /// Restricts reservation lookup to future reservations.
    #[display("future")]
    Future,
    /// Restricts reservation lookup to past reservations.
    #[display("past")]
    Past,
    /// Restricts reservation lookup to wait-listed reservations.
    #[display("wait_listed")]
    WaitListed,
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

    impl Animal {}

    #[bon::bon]
    impl Animal {
        /// Starts a builder that keeps the provider animal id as a runtime-checked request parameter.
        #[builder]
        pub fn new(
            /// Gingr animal identifier required by `/api/v1/reservations_by_animal`.
            animal_id: Option<AnimalId>,
            /// Optional Gingr reservation-scope token such as future, past, or wait-listed.
            restrict_to: Option<RestrictTo>,
            /// Optional nested Gingr reservation search filters for this lookup.
            #[builder(name = filter)]
            filters: Option<reservation::SearchFilters>,
        ) -> Result<Animal> {
            Ok(Animal {
                animal_id: animal_id.ok_or(Error::MissingRequiredParameter {
                    parameter: "animal_id",
                })?,
                restrict_to,
                filters,
            })
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

    impl Owner {}

    #[bon::bon]
    impl Owner {
        /// Starts a builder that keeps the provider owner id as a runtime-checked request parameter.
        #[builder]
        pub fn new(
            /// Gingr owner/customer identifier required by `/api/v1/reservations_by_owner`.
            owner_id: Option<OwnerId>,
            /// Optional Gingr reservation-scope token such as future, past, or wait-listed.
            restrict_to: Option<RestrictTo>,
            /// Optional nested Gingr reservation search filters for this lookup.
            #[builder(name = filter)]
            filters: Option<reservation::SearchFilters>,
        ) -> Result<Owner> {
            Ok(Owner {
                owner_id: owner_id.ok_or(Error::MissingRequiredParameter {
                    parameter: "owner_id",
                })?,
                restrict_to,
                filters,
            })
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Display)]
/// Positive future-minute window used by Gingr back-of-house operational views.
pub struct MinutesFuture(u64);

impl MinutesFuture {}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for Gingr back-of-house views used as raw operational evidence for near-term labor planning.
pub struct BackOfHouse {
    location: LocationId,
    reservation_type_ids: Vec<reservation::TypeId>,
    minutes_future: Option<MinutesFuture>,
    full_day: Option<bool>,
}

impl BackOfHouse {}

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

impl GetServicesByType {}

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
