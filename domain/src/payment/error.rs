#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("payment reference must not be empty")]
    EmptyReference,
    #[error("payment reference must be 160 characters or fewer")]
    ReferenceTooLong,
}

pub type Result<T> = std::result::Result<T, Error>;
