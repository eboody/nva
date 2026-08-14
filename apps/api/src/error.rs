//! Canonical safe conversion from typed API failures to the public error envelope.

use crate::public_contract::{ApiError, ErrorDetail, ErrorEnvelope, LiveSideEffectsMode};
use axum::{Json, http::StatusCode, response::IntoResponse};

/// Request evidence safe to echo in a public error response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorContext {
    request_id: String,
    correlation_id: Option<String>,
}

impl ErrorContext {
    /// Creates the value from its required semantic inputs.
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            correlation_id: None,
        }
    }

    /// Returns the value with correlation id attached.
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}

/// Closed public failure taxonomy. Variants intentionally carry no private source error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// Represents the `Authentication` semantic case.
    Authentication(AuthenticationFailure),
    /// Represents the `Authorization` semantic case.
    Authorization(AuthorizationFailure),
    /// Stores the validation component of this boundary value.
    Validation {
        /// Public field-level details that contain no private source error.
        details: Vec<ErrorDetail>,
    },
    /// Represents the `ReviewConflict` semantic case.
    ReviewConflict,
    /// Represents the `IdempotencyConflict` semantic case.
    IdempotencyConflict,
    /// Represents the `SourceAmbiguity` semantic case.
    SourceAmbiguity,
    /// Stores the not found component of this boundary value.
    NotFound {
        /// Public resource identifiers safe to echo to the caller.
        details: Vec<ErrorDetail>,
    },
    /// Represents the `Unavailable` semantic case.
    Unavailable,
    /// Represents the `Internal` semantic case.
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Stable authentication failure representation used at this boundary.
pub enum AuthenticationFailure {
    /// Represents the `MissingTrustedActorContext` semantic case.
    MissingTrustedActorContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Stable authorization failure representation used at this boundary.
pub enum AuthorizationFailure {
    /// Represents the `BodyActorClaimMismatch` semantic case.
    BodyActorClaimMismatch,
    /// Represents the `ActorLocationNotAuthorized` semantic case.
    ActorLocationNotAuthorized,
    /// Represents the `ActorRoleNotAuthorized` semantic case.
    ActorRoleNotAuthorized,
}

impl AuthorizationFailure {
    fn code(self) -> &'static str {
        match self {
            Self::BodyActorClaimMismatch => "body_actor_claim_mismatch",
            Self::ActorLocationNotAuthorized => "actor_location_not_authorized",
            Self::ActorRoleNotAuthorized => "actor_role_not_authorized",
        }
    }
}

impl ErrorKind {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Authentication(_) => StatusCode::UNAUTHORIZED,
            Self::Authorization(_) => StatusCode::FORBIDDEN,
            Self::Validation { .. } | Self::SourceAmbiguity => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ReviewConflict | Self::IdempotencyConflict => StatusCode::CONFLICT,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn api_error(&self) -> ApiError {
        let (code, message, safe_error_class, details) = match self {
            Self::Authentication(AuthenticationFailure::MissingTrustedActorContext) => (
                "missing_trusted_actor_context",
                "Trusted runtime authentication is required.",
                "authentication_failed",
                Vec::new(),
            ),
            Self::Authorization(failure) => (
                failure.code(),
                "The authenticated actor is not authorized for this operation.",
                "authorization_failed",
                Vec::new(),
            ),
            Self::Validation { details } => (
                "workflow_validation_failed",
                "The request violates the owned workflow safety contract.",
                "validation_failed",
                details.clone(),
            ),
            Self::ReviewConflict => (
                "review_conflict",
                "The review decision conflicts with the current workflow state.",
                "review_conflict",
                Vec::new(),
            ),
            Self::IdempotencyConflict => (
                "idempotency_conflict",
                "The idempotency key was previously used for a different request.",
                "idempotency_conflict",
                Vec::new(),
            ),
            Self::SourceAmbiguity => (
                "source_ambiguity",
                "Source evidence is missing, conflicting, or ambiguous.",
                "source_ambiguity",
                Vec::new(),
            ),
            Self::NotFound { details } => (
                "not_found",
                "The requested owned operations resource is not available.",
                "not_found",
                details.clone(),
            ),
            Self::Unavailable => (
                "dependency_unavailable",
                "A required dependency is temporarily unavailable.",
                "unavailable",
                Vec::new(),
            ),
            Self::Internal => (
                "internal_error",
                "The request could not be completed safely.",
                "internal_error",
                Vec::new(),
            ),
        };

        ApiError {
            code: code.to_owned(),
            message: message.to_owned(),
            safe_error_class: safe_error_class.to_owned(),
            details,
        }
    }
}

/// A typed API failure whose only response conversion is the safe public envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicApiError {
    kind: ErrorKind,
    context: ErrorContext,
    public_code: Option<String>,
}

impl PublicApiError {
    /// Creates the value from its required semantic inputs.
    pub fn new(kind: ErrorKind, context: ErrorContext) -> Self {
        Self {
            kind,
            context,
            public_code: None,
        }
    }

    /// Returns the value with public code attached.
    pub fn with_public_code(mut self, public_code: impl Into<String>) -> Self {
        self.public_code = Some(public_code.into());
        self
    }

    /// Returns the safe public status code for this failure.
    pub fn status_code(&self) -> StatusCode {
        self.kind.status_code()
    }

    /// Returns the safe public envelope for this failure.
    pub fn envelope(&self) -> ErrorEnvelope {
        let mut error = self.kind.api_error();
        if let Some(public_code) = &self.public_code {
            error.code.clone_from(public_code);
        }
        ErrorEnvelope {
            error,
            request_id: self.context.request_id.clone(),
            correlation_id: self.context.correlation_id.clone(),
            live_side_effects: LiveSideEffectsMode::Disabled,
        }
    }
}

impl IntoResponse for PublicApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Json(self.envelope())).into_response()
    }
}
