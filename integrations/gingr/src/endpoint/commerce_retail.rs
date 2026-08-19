use super::{Date, LocationId, Method, OwnerId, Request};

fn push_optional<T: core::fmt::Display>(
    params: &mut Vec<(String, String)>,
    key: &str,
    value: Option<T>,
) {
    if let Some(value) = value {
        params.push((key.to_owned(), value.to_string()));
    }
}

/// GET-style retail and package requests where each Gingr id/filter remains visible for reconciliation.
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

    #[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Display)]
    /// Provider subscription identifier used when requesting one Gingr package/subscription record.
    pub struct SubscriptionId(u64);

    impl SubscriptionId {}

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Request descriptor for one Gingr subscription/package record by provider ID.
    pub struct Subscription {
        id: SubscriptionId,
    }

    impl Subscription {}

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

    #[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Display)]
    /// Validated month-day filter accepted by Gingr subscription endpoints.
    pub struct BillDayOfMonth(u8);

    impl BillDayOfMonth {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Display)]
    /// Provider package identifier used to filter Gingr subscriptions.
    pub struct PackageId(u64);

    impl PackageId {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Provider pagination controls for subscription list requests.
    pub struct SubscriptionPagination {
        limit: u64,
        offset: u64,
    }

    impl SubscriptionPagination {}

    #[derive(Clone, Debug, Default, PartialEq, Eq, bon::Builder)]
    /// Request descriptor for Gingr subscriptions/packages, including owner, bill-day, location, package, and deletion filters.
    pub struct Subscriptions {
        include_deleted: Option<bool>,
        bill_day_of_month: Option<BillDayOfMonth>,
        owner_id: Option<OwnerId>,
        pagination: Option<SubscriptionPagination>,
        location_id: Option<LocationId>,
        package_id: Option<PackageId>,
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

/// List-style retail and billing requests where date windows and pagination are explicit for reconciliation.
pub mod list {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// Request descriptor for Gingr transaction lists over a validated provider date window.
    pub struct Transactions {
        from_date: Date,
        to_date: Date,
    }

    impl Transactions {}

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

    impl InvoicePagination {}

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    /// Request descriptor for Gingr invoice lists used as raw billing evidence, not payment-policy authority.
    pub struct Invoices {
        pagination: Option<InvoicePagination>,
        complete: Option<bool>,
        closed_only: Option<bool>,
        from_date: Option<Date>,
        to_date: Option<Date>,
    }

    impl Invoices {}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Display)]
/// Provider transaction identifier used when requesting one Gingr transaction record.
pub struct TransactionId(u64);

impl TransactionId {}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Request descriptor for one Gingr transaction record by provider ID.
pub struct Transaction {
    id: TransactionId,
}

impl Transaction {}

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
