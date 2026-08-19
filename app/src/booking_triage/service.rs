use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Classifies error values that drive the booking-readiness workflow.
pub enum Error {
    #[error("booking triage reservation repository could not load reservation {reservation_id}")]
    /// Identifies the missing reservation evidence that stops the workflow before any review packet or agent draft is produced.
    ReservationNotFound {
        /// Reservation id requested from the read-only source evidence repository.
        reservation_id: entities::reservation::Id,
    },
}

/// Shared app result type used across the booking triage gate.
pub type AppResult<T> = core::result::Result<T, Error>;

/// Reservation identifiers used by booking-triage packets and review evidence.
pub mod reservation {
    use super::entities;

    /// Read-only reservation repository used to retrieve source facts for booking triage evaluation.
    pub trait Repository {
        /// Fetches the reservation source record by id without confirming, cancelling, messaging, or mutating provider state.
        fn get(&self, id: entities::reservation::Id) -> Option<entities::Reservation>;
    }
}

impl<R> Service<R> where R: reservation::Repository {}
