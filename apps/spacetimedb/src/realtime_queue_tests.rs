use crate::{
    adapter::{ActorDirectoryAdapter, BlockedActionLogAdapter},
    authz,
    read_model::{BlockedActionNoticeRow, ManagerQueueItemRow, StaffQueueItemRow},
    storage::review_queue::{
        BlockedActionAttemptRow, BlockedActionReasonColumn, ReviewQueueItemRow,
        ReviewQueueStatusColumn, codec,
    },
    tables::{
        ActorKindColumn, LocationScopeRow, ReviewerRoleColumn, RoleAssignmentRow, StaffActorRow,
    },
};
use app::data_quality_hygiene as hygiene;
use app::data_quality_hygiene::{ActorDirectory, AuthorizationPolicy, BlockedActionLog};

fn location_101() -> String {
    "101".to_owned()
}

fn pending_location_101_issue() -> ReviewQueueItemRow {
    ReviewQueueItemRow {
        action_id: "dq-action-location-101".to_owned(),
        location_id: location_101(),
        actor_id: None,
        claimed_by_actor_id: None,
        status: ReviewQueueStatusColumn::PendingStaffReview,
        source_ref_id: Some("gingr:reservation:abc".to_owned()),
        issue_ref: "dq-issue-location-101".to_owned(),
        recommendation: None,
        staff_disposition: None,
        manager_outcome: None,
        created_at: 1,
        updated_at: 1,
        requires_manager_approval: true,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    }
}

#[test]
fn staff_queue_projection_keeps_location_source_and_claim_state_visible() {
    let item: StaffQueueItemRow = codec::staff_queue_item(&pending_location_101_issue());

    assert_eq!(item.action_id, "dq-action-location-101");
    assert_eq!(item.location_id, "101");
    assert_eq!(item.claimed_by_actor_id, None);
    assert_eq!(item.status_label, "pending_staff_review");
    assert_eq!(item.source_ref_id.as_deref(), Some("gingr:reservation:abc"));
}

#[test]
fn manager_queue_projection_only_includes_manager_gated_work() {
    let manager_item: ManagerQueueItemRow =
        codec::manager_queue_item(&pending_location_101_issue())
            .expect("manager-gated rows should project to the manager read model");

    assert_eq!(manager_item.action_id, "dq-action-location-101");
    assert_eq!(manager_item.location_id, "101");
    assert!(manager_item.requires_manager_approval);

    let mut staff_only = pending_location_101_issue();
    staff_only.requires_manager_approval = false;
    assert!(codec::manager_queue_item(&staff_only).is_none());
}

#[test]
fn split_actor_role_and_scope_rows_promote_into_app_authorization_policy() {
    let directory = ActorDirectoryAdapter::new(
        vec![StaffActorRow {
            actor_id: "alice".to_owned(),
            identity: "identity-alice".to_owned(),
            actor_kind: ActorKindColumn::Staff,
            actor_ref: "staff-alice".to_owned(),
            schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
        }],
        vec![RoleAssignmentRow {
            id: 0,
            actor_id: "alice".to_owned(),
            review_role: ReviewerRoleColumn::FrontDeskLead,
            schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
        }],
        vec![LocationScopeRow {
            id: 0,
            actor_id: "alice".to_owned(),
            location_id: location_101(),
        }],
    );

    let alice_id = hygiene::ActorId::try_new("alice".to_owned()).unwrap();
    let alice = hygiene::ActorDirectory::resolve_actor(&directory, &alice_id)
        .expect("alice should promote from split rows");

    assert!(alice.covers_location(authz::parse_location_id("101").unwrap()));
}

#[test]
fn role_location_policy_allows_staff_queue_work_but_keeps_manager_gate_for_outcomes() {
    let directory = ActorDirectoryAdapter::new(
        vec![
            staff_actor(
                "alice",
                "identity-alice",
                ActorKindColumn::Staff,
                "staff-alice",
            ),
            staff_actor("sam", "identity-sam", ActorKindColumn::Staff, "staff-sam"),
            staff_actor(
                "morgan",
                "identity-morgan",
                ActorKindColumn::Manager,
                "manager-morgan",
            ),
            staff_actor("dq-ai", "identity-dq-ai", ActorKindColumn::System, "system"),
        ],
        vec![
            role_assignment("alice", ReviewerRoleColumn::FrontDeskLead),
            role_assignment("sam", ReviewerRoleColumn::FrontDeskLead),
            role_assignment("morgan", ReviewerRoleColumn::GeneralManager),
            role_assignment("dq-ai", ReviewerRoleColumn::OperationsAnalyst),
        ],
        vec![
            location_scope("alice", "101"),
            location_scope("sam", "202"),
            location_scope("morgan", "101"),
            location_scope("dq-ai", "101"),
        ],
    );
    let review_item = codec::review_queue_item(&pending_location_101_issue()).unwrap();
    let policy = hygiene::RoleLocationAuthorization;

    let alice = directory.resolve_actor(&actor_id("alice")).unwrap();
    let sam = directory.resolve_actor(&actor_id("sam")).unwrap();
    let morgan = directory.resolve_actor(&actor_id("morgan")).unwrap();
    let ai = directory.resolve_actor(&actor_id("dq-ai")).unwrap();

    assert!(policy.can_work_queue_item(&alice, &review_item));
    assert!(!policy.can_work_queue_item(&sam, &review_item));
    assert!(!policy.can_record_outcome(&alice, &review_item));
    assert!(policy.can_record_outcome(&morgan, &review_item));
    assert!(policy.can_work_queue_item(&ai, &review_item));
    assert!(!policy.can_record_outcome(&ai, &review_item));
}

#[test]
fn unknown_identity_is_not_promoted_into_a_business_actor() {
    let actors = [staff_actor(
        "alice",
        "identity-alice",
        ActorKindColumn::Staff,
        "staff-alice",
    )];

    assert_eq!(
        authz::actor_id_for_identity("identity-anonymous", actors.iter()),
        None
    );
}

#[test]
fn ai_service_actor_can_draft_but_outcome_cards_never_allow_live_delivery() {
    let directory = ActorDirectoryAdapter::new(
        vec![staff_actor(
            "dq-ai",
            "identity-dq-ai",
            ActorKindColumn::System,
            "system",
        )],
        vec![role_assignment(
            "dq-ai",
            ReviewerRoleColumn::OperationsAnalyst,
        )],
        vec![location_scope("dq-ai", "101")],
    );
    let review_item = codec::review_queue_item(&pending_location_101_issue()).unwrap();
    let ai = directory.resolve_actor(&actor_id("dq-ai")).unwrap();
    let policy = hygiene::RoleLocationAuthorization;

    assert!(policy.can_work_queue_item(&ai, &review_item));
    assert!(!policy.can_record_outcome(&ai, &review_item));

    let outcome_card = crate::read_model::HygieneOutcomeCardRow::new(
        "dq-action-location-101".to_owned(),
        "system".to_owned(),
        "Completed".to_owned(),
        25,
        9,
        "Gingr:reservation-101".to_owned(),
        "dq-issue-location-101".to_owned(),
    );
    assert!(!outcome_card.live_delivery_allowed);
}

#[test]
fn blocked_action_attempt_projects_public_notice_without_sensitive_payload() {
    let notice: BlockedActionNoticeRow = codec::blocked_action_notice(&BlockedActionAttemptRow {
        id: 7,
        action_id: "dq-action-location-101".to_owned(),
        actor_id: "sam".to_owned(),
        location_id: "101".to_owned(),
        attempted_side_effect: "send_customer_message".to_owned(),
        reason: BlockedActionReasonColumn::ActorLacksReviewGate,
        created_at: 9,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });

    assert_eq!(notice.action_id, "dq-action-location-101");
    assert_eq!(notice.actor_id, "sam");
    assert_eq!(notice.location_id, "101");
    assert_eq!(notice.attempted_side_effect, "send_customer_message");
    assert_eq!(notice.reason_label, "actor_lacks_review_gate");
}

#[test]
fn app_service_blocked_capture_rows_keep_review_location_for_public_notices() {
    let mut blocked_log = BlockedActionLogAdapter::new(vec![pending_location_101_issue()]);

    blocked_log.record_blocked_action(hygiene::BlockedActionRecord::new(
        hygiene::ActionId::try_new("dq-action-location-101".to_owned()).unwrap(),
        actor_id("sam"),
        hygiene::BlockedActionReason::ActorLacksReviewGate,
    ));

    let [blocked_row] = blocked_log.rows().try_into().unwrap();
    assert_eq!(blocked_row.location_id, "101");
    let notice = codec::blocked_action_notice(&blocked_row);
    assert_eq!(notice.location_id, "101");
    assert_eq!(notice.attempted_side_effect, "record_reviewed_outcome");
    assert_eq!(notice.reason_label, "actor_lacks_review_gate");
}

fn actor_id(id: &str) -> hygiene::ActorId {
    hygiene::ActorId::try_new(id.to_owned()).unwrap()
}

fn staff_actor(
    actor_id: &str,
    identity: &str,
    actor_kind: ActorKindColumn,
    actor_ref: &str,
) -> StaffActorRow {
    StaffActorRow {
        actor_id: actor_id.to_owned(),
        identity: identity.to_owned(),
        actor_kind,
        actor_ref: actor_ref.to_owned(),
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    }
}

fn role_assignment(actor_id: &str, review_role: ReviewerRoleColumn) -> RoleAssignmentRow {
    RoleAssignmentRow {
        id: 0,
        actor_id: actor_id.to_owned(),
        review_role,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    }
}

fn location_scope(actor_id: &str, location_id: &str) -> LocationScopeRow {
    LocationScopeRow {
        id: 0,
        actor_id: actor_id.to_owned(),
        location_id: location_id.to_owned(),
    }
}
