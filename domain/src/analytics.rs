use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Typed projection version domain value that keeps raw primitives out of analytics workflows.
pub struct ProjectionVersion(String);

impl ProjectionVersion {
    /// Validates and creates the analytics value.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyProjectionVersion).map(Self)
    }

    /// Returns the provider or domain identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stay boundary for analytics contracts.
pub mod stay {
    use serde::{Deserialize, Serialize};

    use crate::{analytics, data_quality, source};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Provider or source identifier retained as the stable join key.
    pub struct Id(String);

    impl Id {
        /// Validates and creates the analytics value.
        pub fn try_new(value: impl Into<String>) -> analytics::Result<Self> {
            analytics::trimmed_non_empty(value, analytics::Error::EmptyStayFactId).map(Self)
        }

        /// Returns the provider or domain identifier as a string slice.
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for data quality status decisions in analytics workflows.
    pub enum DataQualityStatus {
        /// Complete analytics metric or operational summary dimension.
        Complete,
        /// Manager review required analytics metric or operational summary dimension.
        ManagerReviewRequired,
        /// Blocking issues analytics metric or operational summary dimension.
        BlockingIssues,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed fact domain value that keeps raw primitives out of analytics workflows.
    pub struct Fact {
        id: Id,
        provenance: source::Provenance,
        reservation_record_id: source::record::Id,
        customer_record_id: source::record::Id,
        pet_record_id: source::record::Id,
        location_record_id: source::record::Id,
        service_type_record_id: source::record::Id,
        projection_version: analytics::ProjectionVersion,
        data_quality_status: DataQualityStatus,
        data_quality_issues: Vec<data_quality::Issue>,
    }

    impl Fact {
        /// Returns the project from source reservation for this analytics value.
        pub fn project_from_source_reservation(
            id: Id,
            source_reservation: &source::reservation::Snapshot,
            projection_version: analytics::ProjectionVersion,
        ) -> std::result::Result<Self, Vec<data_quality::Issue>> {
            let issues = source_reservation
                .data_quality_issues(source_reservation.provenance().pulled_at().clone());
            if issues.iter().any(data_quality::Issue::workflow_blocking) {
                return Err(issues);
            }
            let data_quality_status = if issues.is_empty() {
                DataQualityStatus::Complete
            } else {
                DataQualityStatus::ManagerReviewRequired
            };

            let customer_record_id = source_reservation
                .customer_record_id()
                .expect("data_quality_issues guards customer presence")
                .clone();
            let pet_record_id = source_reservation
                .pet_record_id()
                .expect("data_quality_issues guards pet presence")
                .clone();
            let location_record_id = source_reservation
                .location_record_id()
                .expect("data_quality_issues guards location presence")
                .clone();
            let service_type_record_id = source_reservation
                .service_type_record_id()
                .expect("data_quality_issues guards service type presence")
                .clone();

            Ok(Self {
                id,
                provenance: source_reservation.provenance().clone(),
                reservation_record_id: source_reservation.provenance().record_id().clone(),
                customer_record_id,
                pet_record_id,
                location_record_id,
                service_type_record_id,
                projection_version,
                data_quality_status,
                data_quality_issues: issues,
            })
        }

        /// Returns this analytics value's id.
        pub const fn id(&self) -> &Id {
            &self.id
        }

        /// Returns this analytics value's source system.
        pub const fn source_system(&self) -> source::System {
            self.provenance.source_system()
        }

        /// Returns this analytics value's provenance.
        pub const fn provenance(&self) -> &source::Provenance {
            &self.provenance
        }

        /// Returns this analytics value's reservation record id.
        pub const fn reservation_record_id(&self) -> &source::record::Id {
            &self.reservation_record_id
        }

        /// Returns this analytics value's customer record id.
        pub const fn customer_record_id(&self) -> &source::record::Id {
            &self.customer_record_id
        }

        /// Returns this analytics value's pet record id.
        pub const fn pet_record_id(&self) -> &source::record::Id {
            &self.pet_record_id
        }

        /// Returns this analytics value's location record id.
        pub const fn location_record_id(&self) -> &source::record::Id {
            &self.location_record_id
        }

        /// Returns this analytics value's service type record id.
        pub const fn service_type_record_id(&self) -> &source::record::Id {
            &self.service_type_record_id
        }

        /// Returns this analytics value's projection version.
        pub const fn projection_version(&self) -> &analytics::ProjectionVersion {
            &self.projection_version
        }

        /// Returns this analytics value's data quality status.
        pub const fn data_quality_status(&self) -> DataQualityStatus {
            self.data_quality_status
        }

        /// Nonblocking source data-quality issues preserved on the projected stay fact.
        ///
        /// Workflow-blocking issues are returned as projection errors instead of producing a fact.
        pub fn data_quality_issues(&self) -> &[data_quality::Issue] {
            &self.data_quality_issues
        }
    }
}

/// Service demand boundary for analytics contracts.
pub mod service_demand {
    use serde::{Deserialize, Serialize};

    use crate::{analytics, data_quality, operations, source};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Provider or source identifier retained as the stable join key.
    pub struct Id(String);

    impl Id {
        /// Validates and creates the analytics value.
        pub fn try_new(value: impl Into<String>) -> analytics::Result<Self> {
            analytics::trimmed_non_empty(value, analytics::Error::EmptyServiceDemandFactId)
                .map(Self)
        }

        /// Returns the provider or domain identifier as a string slice.
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Typed demand units domain value that keeps raw primitives out of analytics workflows.
    pub struct DemandUnits(u32);

    impl DemandUnits {
        /// Promotes boundary input into a validated analytics domain value.
        pub const fn try_new(value: u32) -> analytics::Result<Self> {
            if value == 0 {
                return Err(analytics::Error::EmptyDemandUnits);
            }
            Ok(Self(value))
        }

        /// Exposes the validated scalar for serialization and adapter boundaries.
        pub const fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed fact domain value that keeps raw primitives out of analytics workflows.
    pub struct Fact {
        id: Id,
        operating_day: operations::operating_day::Key,
        demand_units: DemandUnits,
        source_record_refs: Vec<source::RecordRef>,
        projection_version: analytics::ProjectionVersion,
        data_quality_status: DataQualityStatus,
        data_quality_issues: Vec<data_quality::Issue>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for data quality status decisions in analytics workflows.
    pub enum DataQualityStatus {
        /// Complete analytics metric or operational summary dimension.
        Complete,
        /// Manager review required analytics metric or operational summary dimension.
        ManagerReviewRequired,
    }

    impl Fact {
        /// Validates and creates the analytics value.
        pub fn try_new(
            id: Id,
            operating_day: operations::operating_day::Key,
            demand_units: DemandUnits,
            source_record_refs: Vec<source::RecordRef>,
            projection_version: analytics::ProjectionVersion,
            data_quality_issues: Vec<data_quality::Issue>,
        ) -> Result<Self> {
            if source_record_refs.is_empty() {
                return Err(Error::MissingSourceEvidence);
            }
            let data_quality_status = if data_quality_issues.is_empty() {
                DataQualityStatus::Complete
            } else {
                DataQualityStatus::ManagerReviewRequired
            };

            Ok(Self {
                id,
                operating_day,
                demand_units,
                source_record_refs,
                projection_version,
                data_quality_status,
                data_quality_issues,
            })
        }

        /// Returns this analytics value's id.
        pub const fn id(&self) -> &Id {
            &self.id
        }

        /// Returns this analytics value's operating day.
        pub const fn operating_day(&self) -> &operations::operating_day::Key {
            &self.operating_day
        }

        /// Returns this analytics value's demand units.
        pub const fn demand_units(&self) -> DemandUnits {
            self.demand_units
        }

        /// Returns the source record refs for this analytics value.
        pub fn source_record_refs(&self) -> &[source::RecordRef] {
            &self.source_record_refs
        }

        /// Returns this analytics value's projection version.
        pub const fn projection_version(&self) -> &analytics::ProjectionVersion {
            &self.projection_version
        }

        /// Returns this analytics value's data quality status.
        pub const fn data_quality_status(&self) -> DataQualityStatus {
            self.data_quality_status
        }

        /// Returns the data quality issues for this analytics value.
        pub fn data_quality_issues(&self) -> &[data_quality::Issue] {
            &self.data_quality_issues
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Validation failures returned by analytics domain constructors.
    pub enum Error {
        #[error("service demand facts require source evidence")]
        /// Missing source evidence analytics metric or operational summary dimension.
        MissingSourceEvidence,
    }

    /// Result type returned by fallible analytics operations.
    pub type Result<T> = std::result::Result<T, Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by analytics domain constructors.
pub enum Error {
    #[error("stay fact id must not be empty")]
    /// Signals that stay fact id was blank or missing during analytics validation.
    EmptyStayFactId,
    #[error("service demand fact id must not be empty")]
    /// Signals that service demand fact id was blank or missing during analytics validation.
    EmptyServiceDemandFactId,
    #[error("service demand units must be greater than zero")]
    /// Signals that demand units was blank or missing during analytics validation.
    EmptyDemandUnits,
    #[error("projection version must not be empty")]
    /// Signals that projection version was blank or missing during analytics validation.
    EmptyProjectionVersion,
}

/// Result type returned by fallible analytics operations.
pub type Result<T> = std::result::Result<T, Error>;

fn trimmed_non_empty(value: impl Into<String>, empty_error: Error) -> Result<String> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(empty_error);
    }
    Ok(value)
}
