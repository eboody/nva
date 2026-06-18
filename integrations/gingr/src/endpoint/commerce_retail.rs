use super::{Date, Error, LocationId, Method, OwnerId, Request, Result};

fn cutover_date() -> Date {
    Date::parse("2019-08-01").expect("documented Gingr commerce cutover date is valid")
}

fn push_optional<T: core::fmt::Display>(
    params: &mut Vec<(String, String)>,
    key: &str,
    value: Option<T>,
) {
    if let Some(value) = value {
        params.push((key.to_owned(), value.to_string()));
    }
}

/// Gingr get endpoint boundary with provider parameters kept explicit.
pub mod get {
    use super::*;

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    /// Request descriptor for the Gingr retail-item catalog used as source evidence for inventory and upsell workflows.
    pub struct AllRetailItems;

    impl Request for AllRetailItems {
        fn method(&self) -> Method {
            Method::Get
        }

        fn path(&self) -> &'static str {
            "/api/v1/get_all_retail_items"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            Vec::new()
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Provider subscription identifier used when requesting one Gingr package/subscription record.
    pub struct SubscriptionId(u64);

    impl SubscriptionId {
        /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
        pub fn new(value: u64) -> Self {
            Self(value)
        }
    }

    impl core::fmt::Display for SubscriptionId {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(formatter, "{}", self.0)
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Request descriptor for one Gingr subscription/package record by provider ID.
    pub struct Subscription {
        id: SubscriptionId,
    }

    impl Subscription {
        /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
        pub fn new(id: SubscriptionId) -> Self {
            Self { id }
        }
    }

    impl Request for Subscription {
        fn method(&self) -> Method {
            Method::Get
        }

        fn path(&self) -> &'static str {
            "/api/v1/get_subscription"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            vec![("id".to_owned(), self.id.to_string())]
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Validated month-day filter accepted by Gingr subscription endpoints.
    pub struct BillDayOfMonth(u8);

    impl BillDayOfMonth {
        /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
        pub fn new(value: u8) -> Result<Self> {
            if (1..=31).contains(&value) {
                Ok(Self(value))
            } else {
                Err(Error::InvalidBillDayOfMonth { value })
            }
        }
    }

    impl core::fmt::Display for BillDayOfMonth {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(formatter, "{}", self.0)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Provider package identifier used to filter Gingr subscriptions.
    pub struct PackageId(u64);

    impl PackageId {
        /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
        pub fn new(value: u64) -> Self {
            Self(value)
        }
    }

    impl core::fmt::Display for PackageId {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(formatter, "{}", self.0)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Provider pagination controls for subscription list requests.
    pub struct SubscriptionPagination {
        limit: u64,
        offset: u64,
    }

    impl SubscriptionPagination {
        /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
        pub fn new(limit: u64, offset: u64) -> Self {
            Self { limit, offset }
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    /// Request descriptor for Gingr subscriptions/packages, including owner, bill-day, location, package, and deletion filters.
    pub struct Subscriptions {
        include_deleted: Option<bool>,
        bill_day_of_month: Option<BillDayOfMonth>,
        owner_id: Option<OwnerId>,
        pagination: Option<SubscriptionPagination>,
        location_id: Option<LocationId>,
        package_id: Option<PackageId>,
    }

    impl Subscriptions {
        /// Starts a builder that makes each provider parameter explicit before request capture.
        pub fn builder() -> SubscriptionsBuilder {
            SubscriptionsBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Builder for Gingr subscription filters without turning package data into NVA revenue facts.
    pub struct SubscriptionsBuilder {
        include_deleted: Option<bool>,
        bill_day_of_month: Option<BillDayOfMonth>,
        owner_id: Option<OwnerId>,
        pagination: Option<SubscriptionPagination>,
        location_id: Option<LocationId>,
        package_id: Option<PackageId>,
    }

    impl SubscriptionsBuilder {
        /// Includes deleted provider records when Gingr supports that filter.
        pub fn include_deleted(mut self, include_deleted: bool) -> Self {
            self.include_deleted = Some(include_deleted);
            self
        }

        /// Filters subscriptions by provider bill day of month.
        pub fn bill_day_of_month(mut self, bill_day_of_month: BillDayOfMonth) -> Self {
            self.bill_day_of_month = Some(bill_day_of_month);
            self
        }

        /// Narrows the provider request to one Gingr owner/customer identifier.
        pub fn owner_id(mut self, owner_id: OwnerId) -> Self {
            self.owner_id = Some(owner_id);
            self
        }

        /// Applies provider pagination controls to the request.
        pub fn pagination(mut self, pagination: SubscriptionPagination) -> Self {
            self.pagination = Some(pagination);
            self
        }

        /// Scopes the Gingr endpoint request to a location.
        pub fn location_id(mut self, location_id: LocationId) -> Self {
            self.location_id = Some(location_id);
            self
        }

        /// Filters subscription requests to a package identifier.
        pub fn package_id(mut self, package_id: PackageId) -> Self {
            self.package_id = Some(package_id);
            self
        }

        /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
        pub fn build(self) -> Subscriptions {
            Subscriptions {
                include_deleted: self.include_deleted,
                bill_day_of_month: self.bill_day_of_month,
                owner_id: self.owner_id,
                pagination: self.pagination,
                location_id: self.location_id,
                package_id: self.package_id,
            }
        }
    }

    impl Request for Subscriptions {
        fn method(&self) -> Method {
            Method::Get
        }

        fn path(&self) -> &'static str {
            "/api/v1/get_subscriptions"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            let mut params = Vec::new();
            push_optional(&mut params, "include_deleted", self.include_deleted);
            push_optional(&mut params, "bill_day_of_month", self.bill_day_of_month);
            push_optional(&mut params, "owner_id", self.owner_id);
            if let Some(pagination) = self.pagination {
                params.push(("limit".to_owned(), pagination.limit.to_string()));
                params.push(("offset".to_owned(), pagination.offset.to_string()));
            }
            push_optional(&mut params, "location_id", self.location_id);
            push_optional(&mut params, "package_id", self.package_id);
            params
        }
    }
}

/// Gingr list endpoint boundary with provider parameters kept explicit.
pub mod list {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Request descriptor for Gingr transaction lists over a validated provider date window.
    pub struct Transactions {
        from_date: Date,
        to_date: Date,
    }

    impl Transactions {
        /// Starts a builder that makes each provider parameter explicit before request capture.
        pub fn builder() -> TransactionsBuilder {
            TransactionsBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Builder for the transaction date window used by commerce reconciliation workflows.
    pub struct TransactionsBuilder {
        from_date: Option<Date>,
        to_date: Option<Date>,
    }

    impl TransactionsBuilder {
        /// Sets the inclusive provider start date sent to Gingr.
        pub fn from_date(mut self, from_date: Date) -> Self {
            self.from_date = Some(from_date);
            self
        }

        /// Sets the inclusive provider end date sent to Gingr.
        pub fn to_date(mut self, to_date: Date) -> Self {
            self.to_date = Some(to_date);
            self
        }

        /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
        pub fn build(self) -> Result<Transactions> {
            let from_date = self.from_date.ok_or(Error::MissingRequiredParameter {
                parameter: "from_date",
            })?;
            let to_date = self.to_date.ok_or(Error::MissingRequiredParameter {
                parameter: "to_date",
            })?;
            let cutover = cutover_date();
            if from_date >= cutover {
                return Err(Error::LegacyDateBoundary {
                    date: from_date.to_string(),
                    boundary: "list_transactions only returns POS transactions before 2019-08-01",
                });
            }
            if to_date >= cutover {
                return Err(Error::LegacyDateBoundary {
                    date: to_date.to_string(),
                    boundary: "list_transactions only returns POS transactions before 2019-08-01",
                });
            }
            Ok(Transactions { from_date, to_date })
        }
    }

    impl Request for Transactions {
        fn method(&self) -> Method {
            Method::Get
        }

        fn path(&self) -> &'static str {
            "/api/v1/list_transactions"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            vec![
                ("from_date".to_owned(), self.from_date.to_string()),
                ("to_date".to_owned(), self.to_date.to_string()),
            ]
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Provider pagination controls for invoice list requests after the documented Gingr cutover date.
    pub struct InvoicePagination {
        per_page: u64,
        page: u64,
    }

    impl InvoicePagination {
        /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
        pub fn new(per_page: u64, page: u64) -> Result<Self> {
            if per_page == 0 {
                return Err(Error::InvalidPagination {
                    reason: "list_invoices per_page must be greater than zero",
                });
            }
            if page == 0 || !(page - 1).is_multiple_of(per_page) {
                return Err(Error::InvalidPagination {
                    reason: "list_invoices page is a one-based starting result number incremented by per_page",
                });
            }
            Ok(Self { per_page, page })
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    /// Request descriptor for Gingr invoice lists used as raw billing evidence, not payment-policy authority.
    pub struct Invoices {
        pagination: Option<InvoicePagination>,
        complete: Option<bool>,
        closed_only: Option<bool>,
        from_date: Option<Date>,
        to_date: Option<Date>,
    }

    impl Invoices {
        /// Starts a builder that makes each provider parameter explicit before request capture.
        pub fn builder() -> InvoicesBuilder {
            InvoicesBuilder::default()
        }
    }

    #[derive(Clone, Debug, Default)]
    /// Builder for invoice date, owner, location, and pagination filters.
    pub struct InvoicesBuilder {
        pagination: Option<InvoicePagination>,
        complete: Option<bool>,
        closed_only: Option<bool>,
        from_date: Option<Date>,
        to_date: Option<Date>,
    }

    impl InvoicesBuilder {
        /// Applies provider pagination controls to the request.
        pub fn pagination(mut self, pagination: InvoicePagination) -> Self {
            self.pagination = Some(pagination);
            self
        }

        /// Filters invoice results by completion state.
        pub fn complete(mut self, complete: bool) -> Self {
            self.complete = Some(complete);
            self
        }

        /// Restricts invoice results to closed Gingr invoices.
        pub fn closed_only(mut self, closed_only: bool) -> Self {
            self.closed_only = Some(closed_only);
            self
        }

        /// Sets the inclusive provider start date sent to Gingr.
        pub fn from_date(mut self, from_date: Date) -> Self {
            self.from_date = Some(from_date);
            self
        }

        /// Sets the inclusive provider end date sent to Gingr.
        pub fn to_date(mut self, to_date: Date) -> Self {
            self.to_date = Some(to_date);
            self
        }

        /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
        pub fn build(self) -> Result<Invoices> {
            let cutover = cutover_date();
            for date in [&self.from_date, &self.to_date].into_iter().flatten() {
                if date < &cutover {
                    return Err(Error::LegacyDateBoundary {
                        date: date.to_string(),
                        boundary: "list_invoices only returns invoices created on or after 2019-08-01",
                    });
                }
            }
            Ok(Invoices {
                pagination: self.pagination,
                complete: self.complete,
                closed_only: self.closed_only,
                from_date: self.from_date,
                to_date: self.to_date,
            })
        }
    }

    impl Request for Invoices {
        fn method(&self) -> Method {
            Method::Get
        }

        fn path(&self) -> &'static str {
            "/api/v1/list_invoices"
        }

        fn parameters(&self) -> Vec<(String, String)> {
            let mut params = Vec::new();
            if let Some(pagination) = self.pagination {
                params.push(("per_page".to_owned(), pagination.per_page.to_string()));
                params.push(("page".to_owned(), pagination.page.to_string()));
            }
            push_optional(&mut params, "complete", self.complete);
            push_optional(&mut params, "closed_only", self.closed_only);
            push_optional(&mut params, "from_date", self.from_date);
            push_optional(&mut params, "to_date", self.to_date);
            params
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Provider transaction identifier used when requesting one Gingr transaction record.
pub struct TransactionId(u64);

impl TransactionId {
    /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

impl core::fmt::Display for TransactionId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Classification for whether a Gingr response can be logged or must be quarantined.
pub enum ResponseSensitivity {
    /// Response may include payment-related details and must stay log-quarantined.
    PaymentSensitive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for one Gingr transaction record by provider ID.
pub struct Transaction {
    id: TransactionId,
}

impl Transaction {
    /// Constructs this typed Gingr boundary value after the caller has chosen the provider input to trust.
    pub fn new(id: TransactionId) -> Self {
        Self { id }
    }

    /// Describes whether a response payload should be quarantined from normal logs.
    pub fn sensitivity(&self) -> ResponseSensitivity {
        ResponseSensitivity::PaymentSensitive
    }
}

impl Request for Transaction {
    fn method(&self) -> Method {
        Method::Post
    }

    fn path(&self) -> &'static str {
        "/api/v1/transaction"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        vec![("id".to_owned(), self.id.to_string())]
    }
}
