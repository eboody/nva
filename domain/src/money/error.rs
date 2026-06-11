use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Error {
    #[error("money amount must contain at least one minor unit")]
    EmptyAmount,
}

pub type Result<T> = std::result::Result<T, Error>;
