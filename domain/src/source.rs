use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum System {
    Gingr,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref().trim();
        if value.is_empty() {
            return Err(Error::EmptyTimestamp);
        }
        let parsed = value
            .parse::<DateTime<Utc>>()
            .map_err(|_| Error::InvalidTimestamp)?;
        Ok(Self(parsed))
    }

    pub const fn get(&self) -> &DateTime<Utc> {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PayloadHash(String);

impl PayloadHash {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyPayloadHash).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RawPayloadRef(String);

impl RawPayloadRef {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyRawPayloadRef).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub mod gingr {
    use bon::Builder;
    use serde::{Deserialize, Serialize};

    use crate::source;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Endpoint(String);

    impl Endpoint {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyEndpoint).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ProviderRecordId(String);

    impl ProviderRecordId {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyProviderRecordId).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RelatedProviderId {
        Owner(ProviderRecordId),
        Animal(ProviderRecordId),
        Location(ProviderRecordId),
        ReservationType(ProviderRecordId),
        Invoice(ProviderRecordId),
        Payment(ProviderRecordId),
        Service(ProviderRecordId),
    }

    impl RelatedProviderId {
        pub const fn owner(id: ProviderRecordId) -> Self {
            Self::Owner(id)
        }

        pub const fn animal(id: ProviderRecordId) -> Self {
            Self::Animal(id)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ExtractionBatchId(String);

    impl ExtractionBatchId {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyExtractionBatch).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct RequestScope(String);

    impl RequestScope {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyRequestScope).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ProviderSchemaVersion(String);

    impl ProviderSchemaVersion {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyProviderSchemaVersion).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ProviderStatus(String);

    impl ProviderStatus {
        pub fn try_new(value: impl Into<String>) -> source::Result<Self> {
            source::trimmed_non_empty(value, source::Error::EmptyProviderStatus).map(Self)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct Provenance {
        endpoint: Endpoint,
        provider_record_id: ProviderRecordId,
        #[builder(default)]
        related_provider_ids: Vec<RelatedProviderId>,
        extraction_batch: ExtractionBatchId,
        pulled_at: source::Timestamp,
        request_scope: RequestScope,
        provider_schema_version: ProviderSchemaVersion,
        source_payload_hash: source::PayloadHash,
        raw_payload_ref: source::RawPayloadRef,
    }

    impl Provenance {
        pub const fn source_system(&self) -> source::System {
            source::System::Gingr
        }

        pub const fn endpoint(&self) -> &Endpoint {
            &self.endpoint
        }

        pub const fn provider_record_id(&self) -> &ProviderRecordId {
            &self.provider_record_id
        }

        pub fn related_provider_ids(&self) -> &[RelatedProviderId] {
            &self.related_provider_ids
        }

        pub const fn extraction_batch(&self) -> &ExtractionBatchId {
            &self.extraction_batch
        }

        pub const fn pulled_at(&self) -> &source::Timestamp {
            &self.pulled_at
        }

        pub const fn raw_payload_ref(&self) -> &source::RawPayloadRef {
            &self.raw_payload_ref
        }
    }

    pub mod reservation {
        use serde::{Deserialize, Serialize};

        use super::{Provenance, ProviderRecordId, ProviderStatus};
        use crate::{data_quality, source};

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum OwnerPetRelationship {
            Resolved,
            Ambiguous { candidate_count: u16 },
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Snapshot {
            provenance: Provenance,
            owner_provider_id: Option<ProviderRecordId>,
            animal_provider_id: Option<ProviderRecordId>,
            location_provider_id: Option<ProviderRecordId>,
            service_type_provider_id: Option<ProviderRecordId>,
            provider_status: Option<ProviderStatus>,
            relationship: OwnerPetRelationship,
        }

        impl Snapshot {
            pub const fn builder() -> SnapshotBuilder {
                SnapshotBuilder::new()
            }
            pub const fn provenance(&self) -> &Provenance {
                &self.provenance
            }

            pub const fn owner_provider_id(&self) -> Option<&ProviderRecordId> {
                self.owner_provider_id.as_ref()
            }

            pub const fn animal_provider_id(&self) -> Option<&ProviderRecordId> {
                self.animal_provider_id.as_ref()
            }

            pub const fn location_provider_id(&self) -> Option<&ProviderRecordId> {
                self.location_provider_id.as_ref()
            }

            pub const fn service_type_provider_id(&self) -> Option<&ProviderRecordId> {
                self.service_type_provider_id.as_ref()
            }

            pub const fn provider_status(&self) -> Option<&ProviderStatus> {
                self.provider_status.as_ref()
            }

            pub const fn relationship(&self) -> &OwnerPetRelationship {
                &self.relationship
            }

            pub fn data_quality_issues(
                &self,
                detected_at: source::Timestamp,
            ) -> Vec<data_quality::Issue> {
                let mut issues = Vec::new();
                self.push_missing_issue(
                    &mut issues,
                    self.owner_provider_id.is_none(),
                    data_quality::SourceField::OwnerProviderId,
                    detected_at.clone(),
                );
                self.push_missing_issue(
                    &mut issues,
                    self.animal_provider_id.is_none(),
                    data_quality::SourceField::AnimalProviderId,
                    detected_at.clone(),
                );
                self.push_missing_issue(
                    &mut issues,
                    self.location_provider_id.is_none(),
                    data_quality::SourceField::LocationProviderId,
                    detected_at.clone(),
                );
                self.push_missing_issue(
                    &mut issues,
                    self.service_type_provider_id.is_none(),
                    data_quality::SourceField::ServiceTypeProviderId,
                    detected_at.clone(),
                );
                if self.provider_status.is_none() {
                    issues.push(data_quality::Issue::new(
                        data_quality::Kind::UnknownProviderStatus,
                        data_quality::Severity::Blocking,
                        self.provenance.clone(),
                        detected_at.clone(),
                        true,
                    ));
                }
                if matches!(self.relationship, OwnerPetRelationship::Ambiguous { .. }) {
                    issues.push(data_quality::Issue::new(
                        data_quality::Kind::AmbiguousOwnerPetRelationship,
                        data_quality::Severity::Blocking,
                        self.provenance.clone(),
                        detected_at,
                        true,
                    ));
                }
                issues
            }

            fn push_missing_issue(
                &self,
                issues: &mut Vec<data_quality::Issue>,
                missing: bool,
                field: data_quality::SourceField,
                detected_at: source::Timestamp,
            ) {
                if missing {
                    issues.push(data_quality::Issue::new(
                        data_quality::Kind::MissingRequiredField { field },
                        data_quality::Severity::Blocking,
                        self.provenance.clone(),
                        detected_at,
                        true,
                    ));
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct SnapshotBuilder {
            provenance: Option<Provenance>,
            owner_provider_id: Option<ProviderRecordId>,
            animal_provider_id: Option<ProviderRecordId>,
            location_provider_id: Option<ProviderRecordId>,
            service_type_provider_id: Option<ProviderRecordId>,
            provider_status: Option<ProviderStatus>,
            relationship: Option<OwnerPetRelationship>,
        }

        impl SnapshotBuilder {
            pub const fn new() -> Self {
                Self {
                    provenance: None,
                    owner_provider_id: None,
                    animal_provider_id: None,
                    location_provider_id: None,
                    service_type_provider_id: None,
                    provider_status: None,
                    relationship: None,
                }
            }

            pub fn provenance(mut self, provenance: Provenance) -> Self {
                self.provenance = Some(provenance);
                self
            }

            pub fn owner_provider_id(mut self, id: impl Into<Option<ProviderRecordId>>) -> Self {
                self.owner_provider_id = id.into();
                self
            }

            pub fn animal_provider_id(mut self, id: impl Into<Option<ProviderRecordId>>) -> Self {
                self.animal_provider_id = id.into();
                self
            }

            pub fn location_provider_id(mut self, id: impl Into<Option<ProviderRecordId>>) -> Self {
                self.location_provider_id = id.into();
                self
            }

            pub fn service_type_provider_id(
                mut self,
                id: impl Into<Option<ProviderRecordId>>,
            ) -> Self {
                self.service_type_provider_id = id.into();
                self
            }

            pub fn provider_status(mut self, status: impl Into<Option<ProviderStatus>>) -> Self {
                self.provider_status = status.into();
                self
            }

            pub fn relationship(mut self, relationship: OwnerPetRelationship) -> Self {
                self.relationship = Some(relationship);
                self
            }

            pub fn build(self) -> Snapshot {
                Snapshot {
                    provenance: self.provenance.expect("snapshot provenance is required"),
                    owner_provider_id: self.owner_provider_id,
                    animal_provider_id: self.animal_provider_id,
                    location_provider_id: self.location_provider_id,
                    service_type_provider_id: self.service_type_provider_id,
                    provider_status: self.provider_status,
                    relationship: self
                        .relationship
                        .expect("snapshot relationship is required"),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("timestamp must not be empty")]
    EmptyTimestamp,
    #[error("timestamp must be RFC3339 UTC-compatible text")]
    InvalidTimestamp,
    #[error("source payload hash must not be empty")]
    EmptyPayloadHash,
    #[error("raw payload reference must not be empty")]
    EmptyRawPayloadRef,
    #[error("Gingr endpoint must not be empty")]
    EmptyEndpoint,
    #[error("provider record id must not be empty")]
    EmptyProviderRecordId,
    #[error("extraction batch id must not be empty")]
    EmptyExtractionBatch,
    #[error("request scope must not be empty")]
    EmptyRequestScope,
    #[error("provider schema version must not be empty")]
    EmptyProviderSchemaVersion,
    #[error("provider status must not be empty")]
    EmptyProviderStatus,
}

pub type Result<T> = std::result::Result<T, Error>;

fn trimmed_non_empty(value: impl Into<String>, empty_error: Error) -> Result<String> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(empty_error);
    }
    Ok(value)
}
