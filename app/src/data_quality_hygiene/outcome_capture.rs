use super::*;

#[derive(Clone, PartialEq, Eq)]
/// Actor resolved by an app-owned directory before caller-reported data-quality hygiene evidence capture is authorized.
pub struct ActorAssignment {
    actor_id: ActorId,
    actor: entities::ActorRef,
    review_role: ReviewerRole,
    location_ids: Vec<entities::LocationId>,
}

impl ActorAssignment {
    /// Names a review-capable app actor and the locations where the actor may operate.
    pub fn new(
        actor_id: ActorId,
        actor: entities::ActorRef,
        review_role: ReviewerRole,
        location_ids: Vec<entities::LocationId>,
    ) -> Self {
        Self {
            actor_id,
            actor,
            review_role,
            location_ids,
        }
    }

    /// Returns the app actor id used by adapters when resolving staff and manager identities.
    pub const fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    /// Returns the actor label admitted after directory lookup; it does not prove underlying review.
    pub const fn actor(&self) -> &entities::ActorRef {
        &self.actor
    }

    /// Returns the actor's data-quality hygiene review role.
    pub const fn review_role(&self) -> ReviewerRole {
        self.review_role
    }

    /// Reports whether this actor is scoped to the supplied location.
    pub fn covers_location(&self, location_id: entities::LocationId) -> bool {
        self.location_ids.contains(&location_id)
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        AsRef,
        Serialize,
        Deserialize
    )
)]
/// App actor lookup id used by outcome-capture ports without exposing auth-provider or SpacetimeDB row ids.
pub struct ActorId(String);

/// App-owned actor lookup port for resolving submitted staff/manager ids before review-gated capture.
pub trait ActorDirectory {
    /// Resolves an app actor id to a review role and domain actor reference.
    fn resolve_actor(&self, actor_id: &ActorId) -> Option<ActorAssignment>;
}

/// App-owned policy port for location-scoped review-gate authorization.
pub trait AuthorizationPolicy {
    /// Reports whether an actor may view, claim, draft, or route a queue item without closing protected review gates.
    fn can_work_queue_item(&self, actor: &ActorAssignment, review_item: &ReviewQueueItem) -> bool;

    /// Reports whether an actor may record an outcome for the review queue item.
    fn can_record_outcome(&self, actor: &ActorAssignment, review_item: &ReviewQueueItem) -> bool;
}

/// App-owned review queue port for loading the packet/action metadata an outcome is tied to.
pub trait ReviewQueueStore {
    /// Loads a review queue item by action id.
    fn review_item_for_action(&self, action_id: &ActionId) -> Option<ReviewQueueItem>;
}

/// App-owned port for recording caller-reported outcome facts.
pub trait OutcomeRecorder {
    /// Records admitted reported evidence and returns an app receipt.
    fn record_outcome(&mut self, outcome: OutcomeRecord) -> OutcomeReceipt;
}

/// App-owned audit port for append-only outcome capture facts.
pub trait AuditLog {
    /// Appends an audit record for accepted review-gated capture.
    fn append_audit_record(&mut self, record: AuditRecord);
}

/// App-owned blocked-action port for failed authorization attempts and other fail-closed decisions.
pub trait BlockedActionLog {
    /// Records a blocked outcome-capture attempt without persisting reported evidence.
    fn record_blocked_action(&mut self, record: BlockedActionRecord);
}

#[derive(Clone, PartialEq, Eq)]
/// Review queue item metadata required to authorize a data-quality hygiene outcome without coupling app code to storage rows.
pub struct ReviewQueueItem {
    action_id: ActionId,
    location_id: entities::LocationId,
    required_review_gates: Vec<policy::ReviewGate>,
}

impl ReviewQueueItem {
    /// Creates review queue metadata for a source-grounded data-quality hygiene action.
    pub fn new(
        action_id: ActionId,
        location_id: entities::LocationId,
        required_review_gates: Vec<policy::ReviewGate>,
    ) -> Self {
        Self {
            action_id,
            location_id,
            required_review_gates,
        }
    }

    /// Returns the reviewed action id.
    pub const fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    /// Returns the location whose review policy scopes this item.
    pub const fn location_id(&self) -> entities::LocationId {
        self.location_id
    }

    /// Returns the review gates that must be satisfied before outcome capture.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }
}

#[derive(Clone, PartialEq, Eq)]
/// Request to retain caller-reported data-quality hygiene outcome evidence through app-owned ports.
pub struct OutcomeCaptureRequest {
    actor_id: ActorId,
    outcome: OutcomeRecord,
}

impl OutcomeCaptureRequest {
    /// Combines the submitted app actor id with a structurally validated reported-outcome record.
    pub const fn new(actor_id: ActorId, outcome: OutcomeRecord) -> Self {
        Self { actor_id, outcome }
    }
}

#[derive(Clone, PartialEq, Eq)]
/// Receipt returned after app authorization admits caller-reported evidence for persistence.
///
/// Admission proves permission to retain the report, not review, cleanup, completion, or value.
pub struct OutcomeReceipt {
    action_id: ActionId,
}

impl OutcomeReceipt {
    /// Creates a receipt for persisted caller-reported evidence.
    pub fn new(action_id: ActionId) -> Self {
        Self { action_id }
    }

    /// Returns the action id accepted through the app capture port.
    pub const fn action_id(&self) -> &ActionId {
        &self.action_id
    }
}

#[derive(Clone, PartialEq, Eq)]
/// Audit record emitted after app-owned authorization accepts reported-evidence capture.
pub struct AuditRecord {
    action_id: ActionId,
    actor: entities::ActorRef,
    blocked_actions: Vec<BlockedAction>,
}

impl AuditRecord {
    /// Creates an audit fact for accepted reported-evidence capture.
    pub fn new(
        action_id: ActionId,
        actor: entities::ActorRef,
        blocked_actions: Vec<BlockedAction>,
    ) -> Self {
        Self {
            action_id,
            actor,
            blocked_actions,
        }
    }

    /// Returns the audited action id.
    pub const fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    /// Returns the actor recorded on the audit fact.
    pub const fn actor(&self) -> &entities::ActorRef {
        &self.actor
    }

    /// Returns protected side effects that remain blocked even though the outcome was recorded.
    pub fn blocked_actions(&self) -> &[BlockedAction] {
        &self.blocked_actions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Reason a reported-outcome capture attempt was blocked before persistence.
pub enum BlockedActionReason {
    /// Actor id did not resolve to a known app actor.
    ActorNotFound,
    /// Review queue metadata for the action was absent.
    ReviewQueueItemNotFound,
    /// Actor lacks the required role or location scope for the review gate.
    ActorLacksReviewGate,
}

#[derive(Clone, PartialEq, Eq)]
/// Fail-closed record for data-quality hygiene capture attempts that must not persist an outcome.
pub struct BlockedActionRecord {
    action_id: ActionId,
    actor_id: ActorId,
    reason: BlockedActionReason,
}

impl BlockedActionRecord {
    /// Creates a blocked-action audit fact for an outcome capture attempt.
    pub fn new(action_id: ActionId, actor_id: ActorId, reason: BlockedActionReason) -> Self {
        Self {
            action_id,
            actor_id,
            reason,
        }
    }

    /// Returns the blocked action id.
    pub const fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    /// Returns the actor id submitted on the blocked capture attempt.
    pub const fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    /// Returns why capture was blocked.
    pub const fn reason(&self) -> BlockedActionReason {
        self.reason
    }
}

macro_rules! impl_sensitive_debug {
    ($($type:ident),+ $(,)?) => {
        $(
            impl std::fmt::Debug for $type {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str(concat!(stringify!($type), "([REDACTED])"))
                }
            }
        )+
    };
}

impl_sensitive_debug!(
    IssueRef,
    ActionId,
    ContextPacketId,
    CorrelationId,
    ActionRationale,
    Candidate,
    Action,
    Request,
    Packet,
    DraftAction,
    DraftSubmission,
    OutcomeSourceRecordRefs,
    OutcomeIssueRefs,
    ActorAssignment,
    ReviewQueueItem,
    OutcomeCaptureRequest,
    OutcomeReceipt,
    AuditRecord,
    BlockedActionRecord,
);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Default app policy for data-quality hygiene outcomes: managers satisfy manager review gates, staff do not.
pub struct RoleLocationAuthorization;

impl AuthorizationPolicy for RoleLocationAuthorization {
    fn can_work_queue_item(&self, actor: &ActorAssignment, review_item: &ReviewQueueItem) -> bool {
        actor.covers_location(review_item.location_id())
            && matches!(
                actor.actor(),
                entities::ActorRef::Staff { .. }
                    | entities::ActorRef::Manager { .. }
                    | entities::ActorRef::System
                    | entities::ActorRef::Agent { .. }
            )
    }

    fn can_record_outcome(&self, actor: &ActorAssignment, review_item: &ReviewQueueItem) -> bool {
        actor.covers_location(review_item.location_id())
            && review_item.required_review_gates().iter().all(|gate| {
                matches!(
                    (actor.review_role(), gate),
                    (
                        ReviewerRole::GeneralManager,
                        policy::ReviewGate::ManagerApproval
                    ) | (
                        ReviewerRole::RegionalOperator,
                        policy::ReviewGate::ManagerApproval
                    )
                )
            })
            && !matches!(
                actor.actor(),
                entities::ActorRef::System | entities::ActorRef::Agent { .. }
            )
    }
}

/// Trait-backed app service for caller-reported data-quality hygiene evidence capture.
pub struct OutcomeCaptureService<D, P, Q, O, A, B> {
    actor_directory: D,
    authorization_policy: P,
    review_queue: Q,
    outcome_recorder: O,
    audit_log: A,
    blocked_action_log: B,
}

impl<D, P, Q, O, A, B> OutcomeCaptureService<D, P, Q, O, A, B> {
    /// Wires the app-owned ports used to record caller-reported data-quality hygiene evidences.
    pub fn new(
        actor_directory: D,
        authorization_policy: P,
        review_queue: Q,
        outcome_recorder: O,
        audit_log: A,
        blocked_action_log: B,
    ) -> Self {
        Self {
            actor_directory,
            authorization_policy,
            review_queue,
            outcome_recorder,
            audit_log,
            blocked_action_log,
        }
    }

    /// Returns the actor directory port for fixture inspection or setup.
    pub const fn actor_directory(&self) -> &D {
        &self.actor_directory
    }

    /// Returns the outcome recorder port for fixture inspection.
    pub const fn outcome_recorder(&self) -> &O {
        &self.outcome_recorder
    }

    /// Returns the audit log port for fixture inspection.
    pub const fn audit_log(&self) -> &A {
        &self.audit_log
    }

    /// Returns the blocked action log port for fixture inspection.
    pub const fn blocked_action_log(&self) -> &B {
        &self.blocked_action_log
    }
}

impl<D, P, Q, O, A, B> OutcomeCaptureService<D, P, Q, O, A, B>
where
    D: ActorDirectory,
    P: AuthorizationPolicy,
    Q: ReviewQueueStore,
    O: OutcomeRecorder,
    A: AuditLog,
    B: BlockedActionLog,
{
    /// Records caller-reported evidence only after actor, queue, role, and location admission checks pass.
    /// Those checks authorize record admission; they do not prove the reported review or completion occurred.
    pub fn record_reported_outcome(
        &mut self,
        request: OutcomeCaptureRequest,
    ) -> Result<OutcomeReceipt> {
        let action_id = request.outcome.action_id().clone();
        let actor = match self.actor_directory.resolve_actor(&request.actor_id) {
            Some(actor) => actor,
            None => {
                self.blocked_action_log
                    .record_blocked_action(BlockedActionRecord::new(
                        action_id,
                        request.actor_id,
                        BlockedActionReason::ActorNotFound,
                    ));
                return Err(Error::ActorNotFound);
            }
        };
        let review_item = match self
            .review_queue
            .review_item_for_action(request.outcome.action_id())
        {
            Some(review_item) => review_item,
            None => {
                self.blocked_action_log
                    .record_blocked_action(BlockedActionRecord::new(
                        action_id,
                        request.actor_id,
                        BlockedActionReason::ReviewQueueItemNotFound,
                    ));
                return Err(Error::ReviewQueueItemNotFound);
            }
        };
        if !self
            .authorization_policy
            .can_record_outcome(&actor, &review_item)
        {
            self.blocked_action_log
                .record_blocked_action(BlockedActionRecord::new(
                    action_id,
                    request.actor_id,
                    BlockedActionReason::ActorLacksReviewGate,
                ));
            return Err(Error::ActorNotAuthorized);
        }

        let audit_record = AuditRecord::new(
            request.outcome.action_id().clone(),
            actor.actor().clone(),
            request.outcome.blocked_actions(),
        );
        let receipt = self.outcome_recorder.record_outcome(request.outcome);
        self.audit_log.append_audit_record(audit_record);
        Ok(receipt)
    }
}
