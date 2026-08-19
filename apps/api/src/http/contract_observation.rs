use super::{VaccineDocumentState, authentication, authorization_error_payload};
use app::workflow_repository::OutcomeRepository as _;
use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

/// Closed vocabulary of live-effect intents that this runtime can recognize and deny.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeniedLiveEffectIntent {
    /// Deliver customer-facing copy without the required review and send authority.
    SendCustomerMessage,
    /// Change a provider or PMS record from this deterministic API shell.
    MutateProviderOrPmsRecord,
    /// Change a staff schedule or staffing assignment.
    ChangeStaffSchedule,
    /// Move money or alter a refund, discount, deposit, or payment.
    MoveRefundDiscountOrPayment,
    /// Conceal a source-backed data-quality issue from the workflow packet.
    HideSourceDataQualityIssue,
    /// Conceal or automatically resolve ambiguous source evidence.
    HideOrAutoResolveSourceAmbiguity,
    /// Return a sensitive provider payload that remains quarantined.
    ExposeQuarantinedSensitivePayload,
}

/// Workflow whose live-effect vocabulary is being checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeniedLiveEffectWorkflow {
    /// Manager Daily Brief workflow boundary.
    ManagerDailyBrief,
    /// Data Quality Hygiene workflow boundary.
    DataQualityHygiene,
}

impl DeniedLiveEffectIntent {
    const ALL: [Self; 7] = [
        Self::SendCustomerMessage,
        Self::MutateProviderOrPmsRecord,
        Self::ChangeStaffSchedule,
        Self::MoveRefundDiscountOrPayment,
        Self::HideSourceDataQualityIssue,
        Self::HideOrAutoResolveSourceAmbiguity,
        Self::ExposeQuarantinedSensitivePayload,
    ];

    /// Enumerates the canonical denied intents for one workflow.
    pub fn for_workflow(workflow: DeniedLiveEffectWorkflow) -> impl Iterator<Item = Self> {
        Self::ALL
            .into_iter()
            .filter(move |intent| intent.applies_to(workflow))
    }

    /// Stable request code for this denied intent.
    pub const fn code(self) -> &'static str {
        match self {
            Self::SendCustomerMessage => "send_customer_message",
            Self::MutateProviderOrPmsRecord => "mutate_provider_or_pms_record",
            Self::ChangeStaffSchedule => "change_staff_schedule",
            Self::MoveRefundDiscountOrPayment => "move_refund_discount_or_payment",
            Self::HideSourceDataQualityIssue => "hide_source_data_quality_issue",
            Self::HideOrAutoResolveSourceAmbiguity => "hide_or_auto_resolve_source_ambiguity",
            Self::ExposeQuarantinedSensitivePayload => "expose_quarantined_sensitive_payload",
        }
    }

    const fn applies_to(self, workflow: DeniedLiveEffectWorkflow) -> bool {
        match workflow {
            DeniedLiveEffectWorkflow::ManagerDailyBrief => matches!(
                self,
                Self::SendCustomerMessage
                    | Self::MutateProviderOrPmsRecord
                    | Self::ChangeStaffSchedule
                    | Self::MoveRefundDiscountOrPayment
                    | Self::HideSourceDataQualityIssue
            ),
            DeniedLiveEffectWorkflow::DataQualityHygiene => matches!(
                self,
                Self::SendCustomerMessage
                    | Self::MutateProviderOrPmsRecord
                    | Self::ChangeStaffSchedule
                    | Self::MoveRefundDiscountOrPayment
                    | Self::HideOrAutoResolveSourceAmbiguity
                    | Self::ExposeQuarantinedSensitivePayload
            ),
        }
    }

    fn recognize(workflow: DeniedLiveEffectWorkflow, requested: &str) -> Option<Self> {
        Self::for_workflow(workflow).find(|intent| intent.code() == requested.trim())
    }
}

#[derive(Default)]
struct DeniedLiveEffectBoundary {
    denied_intents: AtomicUsize,
}

impl DeniedLiveEffectBoundary {
    fn reject(
        &self,
        workflow: DeniedLiveEffectWorkflow,
        requested: &str,
    ) -> RequestedIntentRejection {
        match DeniedLiveEffectIntent::recognize(workflow, requested) {
            Some(intent) => {
                self.denied_intents.fetch_add(1, Ordering::SeqCst);
                RequestedIntentRejection::Denied(intent)
            }
            None => RequestedIntentRejection::Unsupported,
        }
    }

    fn denied_intent_count(&self) -> usize {
        self.denied_intents.load(Ordering::SeqCst)
    }
}

pub(super) enum RequestedIntentRejection {
    Denied(DeniedLiveEffectIntent),
    Unsupported,
}

#[derive(Clone)]
pub(super) struct State {
    handler_entries: HandlerEntries,
    denied_live_effects: Arc<DeniedLiveEffectBoundary>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            handler_entries: HandlerEntries::default(),
            denied_live_effects: Arc::new(DeniedLiveEffectBoundary::default()),
        }
    }
}

impl State {
    pub(super) fn handler_entries(&self) -> HandlerEntries {
        self.handler_entries.clone()
    }

    fn handler_entry_count(&self) -> usize {
        self.handler_entries.0.load(Ordering::SeqCst)
    }

    pub(super) fn reject_requested_intent(
        &self,
        workflow: DeniedLiveEffectWorkflow,
        requested: &str,
    ) -> RequestedIntentRejection {
        self.denied_live_effects.reject(workflow, requested)
    }

    fn denied_live_effect_intent_count(&self) -> usize {
        self.denied_live_effects.denied_intent_count()
    }
}

#[derive(Clone, Default)]
pub(super) struct HandlerEntries(Arc<AtomicUsize>);

pub(super) struct Protected(pub(super) authentication::Context);

impl<S> FromRequestParts<S> for Protected
where
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let context = parts
            .extensions
            .get::<authentication::Context>()
            .cloned()
            .unwrap_or_else(authentication::Context::missing);
        authentication::authenticate(&context).map_err(|rejection| {
            (
                rejection.status_code(),
                Json(authorization_error_payload(
                    rejection,
                    "authenticated_request",
                    "outcome_persisted",
                )),
            )
                .into_response()
        })?;
        let entries = parts
            .extensions
            .get::<HandlerEntries>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
        entries.0.fetch_add(1, Ordering::SeqCst);
        Ok(Self(context))
    }
}

impl VaccineDocumentState {
    /// Counts records held by the deterministic in-memory runtime.
    #[doc(hidden)]
    pub async fn persisted_record_count(&self) -> usize {
        let store = self.store.lock().await;
        store.documents.len()
            + store.extractions.len()
            + store.vaccine_records.len()
            + store.review_packets.len()
            + store.approvals.len()
            + store.eligibility.len()
            + store.manager_daily_brief_outcomes.outcomes().len()
            + store.data_quality_hygiene_outcomes.outcomes().len()
            + store.data_quality_hygiene_persistence_records.len()
            + store.data_quality_hygiene_idempotency.len()
            + store.inquiry_intake_records.len()
            + store.audit_events.len()
    }

    /// Counts entries into protected HTTP handlers after authentication succeeds.
    #[doc(hidden)]
    pub fn protected_handler_entry_count(&self) -> usize {
        self.contract_observation.handler_entry_count()
    }

    /// Counts recognized forbidden requests rejected by the sole denial boundary.
    #[doc(hidden)]
    pub fn denied_live_effect_intent_count(&self) -> usize {
        self.contract_observation.denied_live_effect_intent_count()
    }
}
