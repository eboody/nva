use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Error {
    #[error("payment reference must not be empty")]
    EmptyPaymentReference,
    #[error("payment reference must be 160 characters or fewer")]
    PaymentReferenceTooLong,
}

pub type Result<T> = std::result::Result<T, Error>;
