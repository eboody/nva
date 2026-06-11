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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GetAllRetailItems;

impl Request for GetAllRetailItems {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListTransactions {
    from_date: Date,
    to_date: Date,
}

impl ListTransactions {
    pub fn builder() -> ListTransactionsBuilder {
        ListTransactionsBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ListTransactionsBuilder {
    from_date: Option<Date>,
    to_date: Option<Date>,
}

impl ListTransactionsBuilder {
    pub fn from_date(mut self, from_date: Date) -> Self {
        self.from_date = Some(from_date);
        self
    }

    pub fn to_date(mut self, to_date: Date) -> Self {
        self.to_date = Some(to_date);
        self
    }

    pub fn build(self) -> Result<ListTransactions> {
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
        Ok(ListTransactions { from_date, to_date })
    }
}

impl Request for ListTransactions {
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
pub struct TransactionId(u64);

impl TransactionId {
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
pub enum ResponseSensitivity {
    PaymentSensitive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    id: TransactionId,
}

impl Transaction {
    pub fn new(id: TransactionId) -> Self {
        Self { id }
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvoicePagination {
    per_page: u64,
    page: u64,
}

impl InvoicePagination {
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
pub struct ListInvoices {
    pagination: Option<InvoicePagination>,
    complete: Option<bool>,
    closed_only: Option<bool>,
    from_date: Option<Date>,
    to_date: Option<Date>,
}

impl ListInvoices {
    pub fn builder() -> ListInvoicesBuilder {
        ListInvoicesBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ListInvoicesBuilder {
    pagination: Option<InvoicePagination>,
    complete: Option<bool>,
    closed_only: Option<bool>,
    from_date: Option<Date>,
    to_date: Option<Date>,
}

impl ListInvoicesBuilder {
    pub fn pagination(mut self, pagination: InvoicePagination) -> Self {
        self.pagination = Some(pagination);
        self
    }

    pub fn complete(mut self, complete: bool) -> Self {
        self.complete = Some(complete);
        self
    }

    pub fn closed_only(mut self, closed_only: bool) -> Self {
        self.closed_only = Some(closed_only);
        self
    }

    pub fn from_date(mut self, from_date: Date) -> Self {
        self.from_date = Some(from_date);
        self
    }

    pub fn to_date(mut self, to_date: Date) -> Self {
        self.to_date = Some(to_date);
        self
    }

    pub fn build(self) -> Result<ListInvoices> {
        let cutover = cutover_date();
        for date in [&self.from_date, &self.to_date].into_iter().flatten() {
            if date < &cutover {
                return Err(Error::LegacyDateBoundary {
                    date: date.to_string(),
                    boundary: "list_invoices only returns invoices created on or after 2019-08-01",
                });
            }
        }
        Ok(ListInvoices {
            pagination: self.pagination,
            complete: self.complete,
            closed_only: self.closed_only,
            from_date: self.from_date,
            to_date: self.to_date,
        })
    }
}

impl Request for ListInvoices {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
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
pub struct GetSubscription {
    id: SubscriptionId,
}

impl GetSubscription {
    pub fn new(id: SubscriptionId) -> Self {
        Self { id }
    }
}

impl Request for GetSubscription {
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
pub struct BillDayOfMonth(u8);

impl BillDayOfMonth {
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
pub struct PackageId(u64);

impl PackageId {
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
pub struct SubscriptionPagination {
    limit: u64,
    offset: u64,
}

impl SubscriptionPagination {
    pub fn new(limit: u64, offset: u64) -> Self {
        Self { limit, offset }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GetSubscriptions {
    include_deleted: Option<bool>,
    bill_day_of_month: Option<BillDayOfMonth>,
    owner_id: Option<OwnerId>,
    pagination: Option<SubscriptionPagination>,
    location_id: Option<LocationId>,
    package_id: Option<PackageId>,
}

impl GetSubscriptions {
    pub fn builder() -> GetSubscriptionsBuilder {
        GetSubscriptionsBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct GetSubscriptionsBuilder {
    include_deleted: Option<bool>,
    bill_day_of_month: Option<BillDayOfMonth>,
    owner_id: Option<OwnerId>,
    pagination: Option<SubscriptionPagination>,
    location_id: Option<LocationId>,
    package_id: Option<PackageId>,
}

impl GetSubscriptionsBuilder {
    pub fn include_deleted(mut self, include_deleted: bool) -> Self {
        self.include_deleted = Some(include_deleted);
        self
    }

    pub fn bill_day_of_month(mut self, bill_day_of_month: BillDayOfMonth) -> Self {
        self.bill_day_of_month = Some(bill_day_of_month);
        self
    }

    pub fn owner_id(mut self, owner_id: OwnerId) -> Self {
        self.owner_id = Some(owner_id);
        self
    }

    pub fn pagination(mut self, pagination: SubscriptionPagination) -> Self {
        self.pagination = Some(pagination);
        self
    }

    pub fn location_id(mut self, location_id: LocationId) -> Self {
        self.location_id = Some(location_id);
        self
    }

    pub fn package_id(mut self, package_id: PackageId) -> Self {
        self.package_id = Some(package_id);
        self
    }

    pub fn build(self) -> GetSubscriptions {
        GetSubscriptions {
            include_deleted: self.include_deleted,
            bill_day_of_month: self.bill_day_of_month,
            owner_id: self.owner_id,
            pagination: self.pagination,
            location_id: self.location_id,
            package_id: self.package_id,
        }
    }
}

impl Request for GetSubscriptions {
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
