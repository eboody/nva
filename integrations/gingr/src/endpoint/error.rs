#[derive(Debug, thiserror::Error, PartialEq, Eq)]
/// Errors raised when Gingr request inputs cannot be represented as safe endpoint parameters.
pub enum Error {
    #[error("missing required Gingr endpoint parameter {parameter}")]
    /// Typed request builder is missing a required Gingr parameter.
    MissingRequiredParameter {
        /// Name of the provider parameter missing from a typed endpoint builder.
        parameter: &'static str,
    },
}
