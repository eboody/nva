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
        provenance: source::gingr::Provenance,
        reservation_provider_id: source::gingr::ProviderRecordId,
        customer_provider_id: source::gingr::ProviderRecordId,
        pet_provider_id: source::gingr::ProviderRecordId,
        location_provider_id: source::gingr::ProviderRecordId,
        service_type_provider_id: source::gingr::ProviderRecordId,
        projection_version: analytics::ProjectionVersion,
        data_quality_status: DataQualityStatus,
    }

    impl Fact {
        pub fn project_from_gingr_reservation(
            id: Id,
            snapshot: &source::gingr::reservation::Snapshot,
            projection_version: analytics::ProjectionVersion,
        ) -> std::result::Result<Self, Vec<data_quality::Issue>> {
            let issues = snapshot.data_quality_issues(snapshot.provenance().pulled_at().clone());
            if !issues.is_empty() {
                return Err(issues);
            }

            let owner_provider_id = snapshot
                .owner_provider_id()
                .expect("data_quality_issues guards owner presence")
                .clone();
            let animal_provider_id = snapshot
                .animal_provider_id()
                .expect("data_quality_issues guards animal presence")
                .clone();
            let location_provider_id = snapshot
                .location_provider_id()
                .expect("data_quality_issues guards location presence")
                .clone();
            let service_type_provider_id = snapshot
                .service_type_provider_id()
                .expect("data_quality_issues guards service type presence")
                .clone();

            Ok(Self {
                id,
                provenance: snapshot.provenance().clone(),
                reservation_provider_id: snapshot.provenance().provider_record_id().clone(),
                customer_provider_id: owner_provider_id,
                pet_provider_id: animal_provider_id,
                location_provider_id,
                service_type_provider_id,
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

        pub const fn reservation_provider_id(&self) -> &source::gingr::ProviderRecordId {
            &self.reservation_provider_id
        }

        pub const fn customer_provider_id(&self) -> &source::gingr::ProviderRecordId {
            &self.customer_provider_id
        }

        pub const fn pet_provider_id(&self) -> &source::gingr::ProviderRecordId {
            &self.pet_provider_id
        }

        pub const fn location_provider_id(&self) -> &source::gingr::ProviderRecordId {
            &self.location_provider_id
        }

        pub const fn service_type_provider_id(&self) -> &source::gingr::ProviderRecordId {
            &self.service_type_provider_id
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
