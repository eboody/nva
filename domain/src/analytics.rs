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
    }

    impl Fact {
        pub fn project_from_reservation_snapshot(
            id: Id,
            snapshot: &source::reservation::Snapshot,
            projection_version: analytics::ProjectionVersion,
        ) -> std::result::Result<Self, Vec<data_quality::Issue>> {
            let issues = snapshot.data_quality_issues(snapshot.provenance().pulled_at().clone());
            if issues.iter().any(data_quality::Issue::workflow_blocking) {
                return Err(issues);
            }

            let customer_record_id = snapshot
                .customer_record_id()
                .expect("data_quality_issues guards customer presence")
                .clone();
            let pet_record_id = snapshot
                .pet_record_id()
                .expect("data_quality_issues guards pet presence")
                .clone();
            let location_record_id = snapshot
                .location_record_id()
                .expect("data_quality_issues guards location presence")
                .clone();
            let service_type_record_id = snapshot
                .service_type_record_id()
                .expect("data_quality_issues guards service type presence")
                .clone();

            Ok(Self {
                id,
                provenance: snapshot.provenance().clone(),
                reservation_record_id: snapshot.provenance().record_id().clone(),
                customer_record_id,
                pet_record_id,
                location_record_id,
                service_type_record_id,
                projection_version,
                data_quality_status: DataQualityStatus::Complete,
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
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("stay fact id must not be empty")]
    EmptyStayFactId,
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
