#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("minimum age must be at least one week")]
    EmptyMinimumAge,
    #[error("add-on label must not be empty")]
    EmptyAddOnLabel,
    #[error("add-on label must be 120 characters or fewer")]
    AddOnLabelTooLong,
}

pub type Result<T> = std::result::Result<T, Error>;
