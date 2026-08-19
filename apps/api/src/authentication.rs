use axum::http::HeaderMap;
use std::fmt;
use uuid::Uuid;

use crate::error::{AuthenticationFailure, AuthorizationFailure, ErrorKind};

#[derive(Clone)]
pub(crate) struct Context {
    actor: Option<Actor>,
}

#[derive(Clone)]
struct Actor {
    actor_id: String,
    role: Role,
    location_id: Uuid,
}

impl fmt::Debug for Context {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Context([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    FrontDeskLead,
    MedicalReviewer,
    GeneralManager,
    Unauthorized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mutation {
    InquiryIntake,
    ManagerDailyBriefDraft,
    DataQualityHygieneDraft,
    VaccineDocumentUpload,
    VaccineReviewDecision,
    ManagerDailyBriefOutcome,
    DataQualityHygieneOutcome,
    InformationLifespanDemo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Read {
    StaffInquiries,
    PermissionedKnowledge,
    OperationalContext,
    SiteFinance,
    OperationalMetrics,
    SourceQualityBacklog,
    InformationLifespanReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rejection {
    MissingTrustedActorContext,
    BodyActorClaimMismatch,
    ActorLocationNotAuthorized,
    ActorRoleNotAuthorized,
}

impl From<Rejection> for ErrorKind {
    fn from(rejection: Rejection) -> Self {
        match rejection {
            Rejection::MissingTrustedActorContext => {
                ErrorKind::Authentication(AuthenticationFailure::MissingTrustedActorContext)
            }
            Rejection::BodyActorClaimMismatch => {
                ErrorKind::Authorization(AuthorizationFailure::BodyActorClaimMismatch)
            }
            Rejection::ActorLocationNotAuthorized => {
                ErrorKind::Authorization(AuthorizationFailure::ActorLocationNotAuthorized)
            }
            Rejection::ActorRoleNotAuthorized => {
                ErrorKind::Authorization(AuthorizationFailure::ActorRoleNotAuthorized)
            }
        }
    }
}

impl Context {
    pub(crate) const fn missing() -> Self {
        Self { actor: None }
    }

    pub(crate) fn from_test_headers(headers: &HeaderMap) -> Self {
        let actor = headers
            .get("x-test-auth-actor-id")
            .and_then(|value| value.to_str().ok())
            .zip(
                headers
                    .get("x-test-auth-role")
                    .and_then(|value| value.to_str().ok()),
            )
            .zip(
                headers
                    .get("x-test-auth-location-id")
                    .and_then(|value| value.to_str().ok()),
            )
            .and_then(|((actor_id, role), location_id)| {
                Some(Actor {
                    actor_id: actor_id.to_owned(),
                    role: role_from_header(role),
                    location_id: Uuid::parse_str(location_id).ok()?,
                })
            });

        Self { actor }
    }
}

fn role_from_header(raw: &str) -> Role {
    match raw {
        "front_desk_lead" => Role::FrontDeskLead,
        "medical_reviewer" => Role::MedicalReviewer,
        "general_manager" => Role::GeneralManager,
        _ => Role::Unauthorized,
    }
}

pub(crate) fn authorize(
    context: &Context,
    mutation: Mutation,
    body_actor_claim: &str,
    location_id: Uuid,
) -> Result<(), Rejection> {
    authorize_actor(context, mutation, body_actor_claim)?;
    authorize_location(context, location_id)
}

pub(crate) fn authorize_actor(
    context: &Context,
    mutation: Mutation,
    body_actor_claim: &str,
) -> Result<(), Rejection> {
    let actor = context
        .actor
        .as_ref()
        .ok_or(Rejection::MissingTrustedActorContext)?;

    if actor.actor_id != body_actor_claim {
        return Err(Rejection::BodyActorClaimMismatch);
    }
    if !role_allows(actor.role, mutation) {
        return Err(Rejection::ActorRoleNotAuthorized);
    }

    Ok(())
}

pub(crate) fn authorize_location(context: &Context, location_id: Uuid) -> Result<(), Rejection> {
    let actor = context
        .actor
        .as_ref()
        .ok_or(Rejection::MissingTrustedActorContext)?;
    if actor.location_id != location_id {
        return Err(Rejection::ActorLocationNotAuthorized);
    }
    Ok(())
}

pub(crate) fn authorize_persona_claim(
    context: &Context,
    body_persona_claim: &str,
) -> Result<(), Rejection> {
    let actor = context
        .actor
        .as_ref()
        .ok_or(Rejection::MissingTrustedActorContext)?;
    if !actor.role.matches_claim(body_persona_claim) {
        return Err(Rejection::BodyActorClaimMismatch);
    }
    Ok(())
}

pub(crate) fn authorize_source_ingest(
    context: &Context,
    mutation: Mutation,
    location_id: Uuid,
) -> Result<(), Rejection> {
    let actor = context
        .actor
        .as_ref()
        .ok_or(Rejection::MissingTrustedActorContext)?;
    if actor.location_id != location_id {
        return Err(Rejection::ActorLocationNotAuthorized);
    }
    if !role_allows(actor.role, mutation) {
        return Err(Rejection::ActorRoleNotAuthorized);
    }
    Ok(())
}

pub(crate) fn authenticate(context: &Context) -> Result<(), Rejection> {
    context
        .actor
        .as_ref()
        .map(|_| ())
        .ok_or(Rejection::MissingTrustedActorContext)
}

pub(crate) fn authorize_read(
    context: &Context,
    read: Read,
    location_id: Option<Uuid>,
    role_claim: Option<&str>,
) -> Result<(), Rejection> {
    let actor = context
        .actor
        .as_ref()
        .ok_or(Rejection::MissingTrustedActorContext)?;
    if location_id.is_some_and(|location_id| actor.location_id != location_id) {
        return Err(Rejection::ActorLocationNotAuthorized);
    }
    if role_claim.is_some_and(|claim| !actor.role.matches_claim(claim)) {
        return Err(Rejection::BodyActorClaimMismatch);
    }
    if !read_allowed(actor.role, read) {
        return Err(Rejection::ActorRoleNotAuthorized);
    }
    Ok(())
}

pub(crate) fn actor_id(context: &Context) -> Result<&str, Rejection> {
    context
        .actor
        .as_ref()
        .map(|actor| actor.actor_id.as_str())
        .ok_or(Rejection::MissingTrustedActorContext)
}

pub(crate) fn actor_location_id(context: &Context) -> Result<Uuid, Rejection> {
    context
        .actor
        .as_ref()
        .map(|actor| actor.location_id)
        .ok_or(Rejection::MissingTrustedActorContext)
}

pub(crate) fn actor_role_claim(context: &Context) -> Result<&'static str, Rejection> {
    context
        .actor
        .as_ref()
        .map(|actor| actor.role.claim())
        .ok_or(Rejection::MissingTrustedActorContext)
}

impl Role {
    const fn claim(self) -> &'static str {
        match self {
            Self::FrontDeskLead => "front_desk_lead",
            Self::MedicalReviewer => "medical_reviewer",
            Self::GeneralManager => "general_manager",
            Self::Unauthorized => "unauthorized",
        }
    }

    fn matches_claim(self, claim: &str) -> bool {
        claim == self.claim() || matches!((self, claim), (Self::FrontDeskLead, "front_desk"))
    }
}

fn read_allowed(role: Role, read: Read) -> bool {
    match read {
        Read::StaffInquiries => matches!(role, Role::FrontDeskLead | Role::GeneralManager),
        Read::PermissionedKnowledge => !matches!(role, Role::Unauthorized),
        Read::OperationalContext | Read::InformationLifespanReport => {
            matches!(role, Role::FrontDeskLead | Role::GeneralManager)
        }
        Read::SiteFinance | Read::OperationalMetrics | Read::SourceQualityBacklog => {
            matches!(role, Role::GeneralManager)
        }
    }
}

fn role_allows(role: Role, mutation: Mutation) -> bool {
    match mutation {
        Mutation::InquiryIntake => {
            matches!(role, Role::FrontDeskLead | Role::GeneralManager)
        }
        Mutation::ManagerDailyBriefDraft => matches!(role, Role::GeneralManager),
        Mutation::DataQualityHygieneDraft => {
            matches!(role, Role::FrontDeskLead | Role::GeneralManager)
        }
        Mutation::VaccineDocumentUpload => {
            matches!(role, Role::FrontDeskLead | Role::MedicalReviewer)
        }
        Mutation::VaccineReviewDecision => matches!(role, Role::MedicalReviewer),
        Mutation::ManagerDailyBriefOutcome => {
            matches!(role, Role::FrontDeskLead | Role::GeneralManager)
        }
        Mutation::DataQualityHygieneOutcome => {
            matches!(role, Role::FrontDeskLead | Role::GeneralManager)
        }
        Mutation::InformationLifespanDemo => {
            matches!(role, Role::FrontDeskLead | Role::GeneralManager)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(role: Role, location_id: Uuid) -> Context {
        Context {
            actor: Some(Actor {
                actor_id: "actor-1".to_owned(),
                role,
                location_id,
            }),
        }
    }

    #[test]
    fn authenticated_context_debug_redacts_actor_identity_and_location() {
        let actor_id = "private-actor-id";
        let location_id = Uuid::parse_str("00000000-0000-4000-8000-000000000777").unwrap();
        let context = Context {
            actor: Some(Actor {
                actor_id: actor_id.to_owned(),
                role: Role::GeneralManager,
                location_id,
            }),
        };

        let debug = format!("{context:?}");
        assert_eq!(debug, "Context([REDACTED])");
        assert!(!debug.contains(actor_id));
        assert!(!debug.contains(&location_id.to_string()));
    }

    #[test]
    fn source_ingest_rejects_wrong_location_and_disallowed_role() {
        let location_id = Uuid::new_v4();

        assert_eq!(
            authorize_source_ingest(
                &context(Role::GeneralManager, location_id),
                Mutation::InquiryIntake,
                Uuid::new_v4(),
            ),
            Err(Rejection::ActorLocationNotAuthorized)
        );
        assert_eq!(
            authorize_source_ingest(
                &context(Role::Unauthorized, location_id),
                Mutation::InquiryIntake,
                location_id,
            ),
            Err(Rejection::ActorRoleNotAuthorized)
        );
    }

    #[test]
    fn reads_reject_wrong_location_and_disallowed_role() {
        let location_id = Uuid::new_v4();
        assert_eq!(
            authorize_read(
                &context(Role::GeneralManager, location_id),
                Read::StaffInquiries,
                Some(Uuid::new_v4()),
                None,
            ),
            Err(Rejection::ActorLocationNotAuthorized)
        );
        assert_eq!(
            authorize_read(
                &context(Role::MedicalReviewer, location_id),
                Read::StaffInquiries,
                Some(location_id),
                None,
            ),
            Err(Rejection::ActorRoleNotAuthorized)
        );
    }

    #[test]
    fn role_claims_preserve_medical_and_untrusted_vocabulary() {
        assert_eq!(Role::MedicalReviewer.claim(), "medical_reviewer");
        assert_eq!(Role::Unauthorized.claim(), "unauthorized");
    }
}
