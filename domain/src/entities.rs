//! Core pet-resort entities and operational records.
//!
//! ## Operator-summary
//!
//! This module supports the shared staff view of pets, customers, reservations, care profiles,
//! documents, vaccine records, care notes, incidents, messages, and approval records. It can
//! reduce labor by keeping the facts needed for triage, safety review, handoff, document review,
//! customer-message approval, and manager queues in one normalized shape instead of scattering
//! them across source-system payloads and free-text notes.
//!
//! It must not automate live booking changes, provider writes, customer sends, payment/refund
//! actions, medical/vaccine/behavior decisions, incident closure, or policy exceptions.
//! Authoritative facts remain the named source record, source document/storage object, policy
//! snapshot, reviewer approval, audit event, and typed domain value for each field. Review
//! gates protect pets, customers, and staff by tying sensitive records to explicit approval
//! targets and lifecycle states before downstream workflows may treat them as cleared.
//!
//! The root preserves the established `domain::entities` contract while each aggregate and value
//! family is implemented by its semantic owner below this module.

/// Error returned when a sentinel UUID is offered as a production domain identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("domain identity UUID cannot be nil")]
pub struct NilIdentityError;

macro_rules! non_nil_uuid_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Constructs an identity for trusted static/programmatic values and rejects nil.
            #[track_caller]
            pub fn new(value: Uuid) -> Self {
                Self::try_new(value).expect("domain identity UUID cannot be nil")
            }

            /// Promotes an untrusted UUID only when it is not the nil sentinel.
            pub const fn try_new(
                value: Uuid,
            ) -> std::result::Result<Self, $crate::entities::NilIdentityError> {
                if value.is_nil() {
                    Err($crate::entities::NilIdentityError)
                } else {
                    Ok(Self(value))
                }
            }

            /// Returns the validated UUID representation.
            pub const fn get(self) -> Uuid {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = Uuid::deserialize(deserializer)?;
                Self::try_new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

mod actor;
pub use actor::{ActorRef, ManagerId, StaffId};

/// Approval records and review lifecycle evidence.
pub mod approval;

/// Care-note vocabulary and records.
pub mod care_note;
pub use care_note::CareNote;

mod customer;
pub use customer::{ContactChannel, Customer, PortalAccountRef, PortalProvider};

mod document;
pub use document::{Document, DocumentError, DocumentSubject, VaccineRecord, VaccineRecordError};

mod identifiers;
pub use identifiers::{
    CustomerId, DocumentId, IncidentId, LocationId, MessageId, PetId, VaccineRecordId,
};

mod incident;
pub use incident::{Incident, IncidentBuilder, IncidentError, IncidentSubject, incident_record};

mod location;
pub use location::{Brand, Location, LocationPolicyRefs};

mod message;
pub use message::{Message, MessageSubject, message_record};

mod pet;
pub use pet::{
    CareProfile, MedicationInstruction, Pet, Sex, SpayNeuterStatus, Species, TemperamentProfile,
};

/// Reservation vocabulary and checked aggregate construction.
pub mod reservation;
pub use reservation::{AddOn, HardStop, Reservation, ServiceKind};
