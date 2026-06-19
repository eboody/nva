//! Analytics read models for resort operations after source validation.
//!
//! Analytics facts in this module sit after source ingestion and data-quality validation:
//! raw Gingr/provider records are preserved as provenance, blocking hygiene findings stop
//! projection, and nonblocking findings stay attached so manager briefs and labor-cost
//! dashboards can explain their evidence instead of inventing operational truth.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Version tag for a deterministic analytics projection.
///
/// This is the read-model side of the source-fact → validated-domain → workflow chain:
/// it records which projection logic turned provider reservations into labor, demand,
/// and manager-brief evidence so downstream reports can compare like with like.
pub struct ProjectionVersion(String);

impl ProjectionVersion {
    /// Validates the projection-version label before reports rely on it for comparisons.
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        trimmed_non_empty(value, Error::EmptyProjectionVersion).map(Self)
    }

    /// Returns the projection-version identifier for storage/read-model boundaries.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Projected stay facts for reservation records that passed source validation.
pub mod stay {
    use serde::{Deserialize, Serialize};

    use crate::{analytics, data_quality, source};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Stable analytics id for a projected stay fact, distinct from provider record ids.
    pub struct Id(String);

    impl Id {
        /// Builds a projected-stay analytics id so reports do not reuse raw provider record ids.
        pub fn try_new(value: impl Into<String>) -> analytics::Result<Self> {
            analytics::trimmed_non_empty(value, analytics::Error::EmptyStayFactId).map(Self)
        }

        /// Returns the provider or domain identifier as a string slice.
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Whether a stay projection is clean, reviewable, or blocked by source hygiene.
    pub enum DataQualityStatus {
        /// Source facts validated cleanly and can feed labor/read-model workflows directly.
        Complete,
        /// Projection is usable, but nonblocking hygiene issues should be visible to managers.
        ManagerReviewRequired,
        /// Source facts are not safe enough to power workflow or labor-cost decisions.
        BlockingIssues,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Projected stay fact used by analytics, manager briefs, and labor planning.
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
        /// Projects a validated stay fact from a source reservation snapshot.
        ///
        /// Blocking data-quality issues return the full issue set instead of producing a
        /// fact; nonblocking issues stay attached as evidence for reviewable read models.
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

        /// Returns the analytics fact id used to join reports without reusing provider record ids.
        pub const fn id(&self) -> &Id {
            &self.id
        }

        /// Returns the provider system that supplied the stay evidence.
        pub const fn source_system(&self) -> source::System {
            self.provenance.source_system()
        }

        /// Returns the source provenance managers can inspect before trusting a brief or labor report.
        pub const fn provenance(&self) -> &source::Provenance {
            &self.provenance
        }

        /// Returns the source reservation record that explains the projected stay.
        pub const fn reservation_record_id(&self) -> &source::record::Id {
            &self.reservation_record_id
        }

        /// Returns the source customer record needed for reviewed communication or cleanup workflows.
        pub const fn customer_record_id(&self) -> &source::record::Id {
            &self.customer_record_id
        }

        /// Returns the source pet record needed for care, eligibility, and safety review.
        pub const fn pet_record_id(&self) -> &source::record::Id {
            &self.pet_record_id
        }

        /// Returns the source location record used to keep demand tied to the correct resort.
        pub const fn location_record_id(&self) -> &source::record::Id {
            &self.location_record_id
        }

        /// Returns the source service-type record used before demand is grouped by service line.
        pub const fn service_type_record_id(&self) -> &source::record::Id {
            &self.service_type_record_id
        }

        /// Returns the projection version so dashboards compare facts produced by the same logic.
        pub const fn projection_version(&self) -> &analytics::ProjectionVersion {
            &self.projection_version
        }

        /// Returns whether the fact is clean or still needs manager-visible data-quality review.
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

/// Aggregated service-demand facts used to compare booked work against labor capacity.
pub mod service_demand {
    use serde::{Deserialize, Serialize};

    use crate::{analytics, data_quality, operations, source};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Provider or source identifier retained as the stable join key.
    pub struct Id(String);

    impl Id {
        /// Builds a service-demand fact only when source records prove the booked work being counted.
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
    /// Nonzero count of work units for a service line on an operating day.
    pub struct DemandUnits(u32);

    impl DemandUnits {
        /// Accepts only nonzero demand units so labor reports cannot hide real booked work.
        pub const fn try_new(value: u32) -> analytics::Result<Self> {
            if value == 0 {
                return Err(analytics::Error::EmptyDemandUnits);
            }
            Ok(Self(value))
        }

        /// Returns the nonzero work-unit count for reports, storage rows, and adapter payloads.
        pub const fn get(self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Source-backed service-demand fact for labor planning and exception reporting.
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
    /// Analytics data-quality state that controls whether service-demand facts can appear as clean or reviewable.
    pub enum DataQualityStatus {
        /// Demand fact has required source evidence and no attached hygiene findings.
        Complete,
        /// Demand fact can inform reports, but attached hygiene findings must stay visible to managers.
        ManagerReviewRequired,
    }

    impl Fact {
        /// Builds a service-demand fact only when source records prove the booked work being counted.
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

        /// Returns the analytics fact id used to join reports without reusing provider record ids.
        pub const fn id(&self) -> &Id {
            &self.id
        }

        /// Returns the resort/service/day bucket used to compare demand with staffing and capacity.
        pub const fn operating_day(&self) -> &operations::operating_day::Key {
            &self.operating_day
        }

        /// Returns the booked work units that drive labor planning and exception ranking.
        pub const fn demand_units(&self) -> DemandUnits {
            self.demand_units
        }

        /// Returns the source records that justify the demand units before labor reports rely on them.
        pub fn source_record_refs(&self) -> &[source::RecordRef] {
            &self.source_record_refs
        }

        /// Returns the projection version so dashboards compare facts produced by the same logic.
        pub const fn projection_version(&self) -> &analytics::ProjectionVersion {
            &self.projection_version
        }

        /// Returns whether the fact is clean or still needs manager-visible data-quality review.
        pub const fn data_quality_status(&self) -> DataQualityStatus {
            self.data_quality_status
        }

        /// Returns nonblocking hygiene findings that explain why demand evidence may need manager review.
        pub fn data_quality_issues(&self) -> &[data_quality::Issue] {
            &self.data_quality_issues
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Validation failures returned by analytics domain constructors.
    pub enum Error {
        #[error("service demand facts require source evidence")]
        /// A demand metric was attempted without source records to prove the underlying work.
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
