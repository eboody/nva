use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProjectionVersion(String);

impl ProjectionVersion {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyProjectionVersion).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub mod stay {
    use serde::{Deserialize, Serialize};

    use crate::{analytics, data_quality, source};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Id(String);

    impl Id {
        pub fn try_new(value: impl Into<String>) -> analytics::Result<Self> {
            analytics::trimmed_non_empty(value, analytics::Error::EmptyStayFactId).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DataQualityStatus {
        Complete,
        ManagerReviewRequired,
        BlockingIssues,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

        pub const fn id(&self) -> &Id {
            &self.id
        }

        pub const fn source_system(&self) -> source::System {
            self.provenance.source_system()
        }

        pub const fn provenance(&self) -> &source::Provenance {
            &self.provenance
        }

        pub const fn reservation_record_id(&self) -> &source::record::Id {
            &self.reservation_record_id
        }

        pub const fn customer_record_id(&self) -> &source::record::Id {
            &self.customer_record_id
        }

        pub const fn pet_record_id(&self) -> &source::record::Id {
            &self.pet_record_id
        }

        pub const fn location_record_id(&self) -> &source::record::Id {
            &self.location_record_id
        }

        pub const fn service_type_record_id(&self) -> &source::record::Id {
            &self.service_type_record_id
        }

        pub const fn projection_version(&self) -> &analytics::ProjectionVersion {
            &self.projection_version
        }

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

pub mod service_demand {
    use serde::{Deserialize, Serialize};

    use crate::{analytics, data_quality, operations, source};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Id(String);

    impl Id {
        pub fn try_new(value: impl Into<String>) -> analytics::Result<Self> {
            analytics::trimmed_non_empty(value, analytics::Error::EmptyServiceDemandFactId)
                .map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct DemandUnits(u32);

    impl DemandUnits {
        pub const fn try_new(value: u32) -> analytics::Result<Self> {
            if value == 0 {
                return Err(analytics::Error::EmptyDemandUnits);
            }
            Ok(Self(value))
        }

        pub const fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub enum DataQualityStatus {
        Complete,
        ManagerReviewRequired,
    }

    impl Fact {
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

        pub const fn id(&self) -> &Id {
            &self.id
        }

        pub const fn operating_day(&self) -> &operations::operating_day::Key {
            &self.operating_day
        }

        pub const fn demand_units(&self) -> DemandUnits {
            self.demand_units
        }

        pub fn source_record_refs(&self) -> &[source::RecordRef] {
            &self.source_record_refs
        }

        pub const fn projection_version(&self) -> &analytics::ProjectionVersion {
            &self.projection_version
        }

        pub const fn data_quality_status(&self) -> DataQualityStatus {
            self.data_quality_status
        }

        pub fn data_quality_issues(&self) -> &[data_quality::Issue] {
            &self.data_quality_issues
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub enum Error {
        #[error("service demand facts require source evidence")]
        MissingSourceEvidence,
    }

    pub type Result<T> = std::result::Result<T, Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("stay fact id must not be empty")]
    EmptyStayFactId,
    #[error("service demand fact id must not be empty")]
    EmptyServiceDemandFactId,
    #[error("service demand units must be greater than zero")]
    EmptyDemandUnits,
    #[error("projection version must not be empty")]
    EmptyProjectionVersion,
}

pub type Result<T> = std::result::Result<T, Error>;

fn trimmed_non_empty(value: impl Into<String>, empty_error: Error) -> Result<String> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(empty_error);
    }
    Ok(value)
}
