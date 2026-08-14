use crate::{
    adapter::{ActorDirectoryAdapter, BlockedActionLogAdapter},
    authz,
    read_model::{BlockedActionNoticeRow, ManagerQueueItemRow, StaffQueueItemRow},
    storage::review_queue::{
        ActorRefColumn, BlockedActionAttemptRow, BlockedActionColumn, BlockedActionReasonColumn,
        FeedbackOutcomeColumn, ManagerOutcomeColumn, ReviewGateColumn, ReviewQueueItemRow,
        ReviewQueueStatusColumn, SourceRecordRefColumn, SourceSystemColumn, StaffDispositionColumn,
        codec, transition,
    },
    tables::{
        ActorKindColumn, LocationScopeRow, ReviewerRoleColumn, RoleAssignmentRow, StaffActorRow,
    },
};
use app::data_quality_hygiene as hygiene;
use app::data_quality_hygiene::{ActorDirectory, AuthorizationPolicy, BlockedActionLog};

#[test]
fn external_callers_cannot_invoke_privileged_demo_seed_reducers() {
    assert!(!crate::reducers::fixture_seed_authorized(false));
    assert!(crate::reducers::fixture_seed_authorized(true));
}

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
        source_ref: Some(SourceRecordRefColumn {
            system: SourceSystemColumn::Gingr,
            record_id: "reservation:abc".to_owned(),
        }),
        issue_ref: "dq-issue-location-101".to_owned(),
        recommendation: None,
        staff_disposition: None,
        manager_outcome: None,
        created_at: 1,
        updated_at: 1,
        required_review_gates: vec![ReviewGateColumn::ManagerApproval],
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    }
}

#[test]
fn reviewed_outcome_provenance_is_derived_from_the_authorized_queue_row() {
    let row = pending_location_101_issue();
    let (source_ref, issue_ref) = crate::reducers::reviewed_outcome_provenance(&row)
        .expect("queue row carries the provenance required for reviewed outcome capture");

    assert_eq!(source_ref.system(), domain::source::System::Gingr);
    assert_eq!(source_ref.record_id().as_str(), "reservation:abc");
    assert_eq!(issue_ref.as_str(), "dq-issue-location-101");

    let mut missing_source = row;
    missing_source.source_ref = None;
    assert!(crate::reducers::reviewed_outcome_provenance(&missing_source).is_err());
}

#[test]
fn staff_queue_projection_keeps_location_source_and_claim_state_visible() {
    let item: StaffQueueItemRow = codec::staff_queue_item(&pending_location_101_issue());

    assert_eq!(item.action_id, "dq-action-location-101");
    assert_eq!(item.location_id, "101");
    assert_eq!(item.claimed_by_actor_id, None);
    assert_eq!(item.status_label, "pending_staff_review");
    assert_eq!(
        item.source_ref,
        Some(SourceRecordRefColumn {
            system: SourceSystemColumn::Gingr,
            record_id: "reservation:abc".to_owned(),
        })
    );
}

#[test]
fn manager_queue_projection_only_includes_manager_gated_work() {
    let manager_item: ManagerQueueItemRow =
        codec::manager_queue_item(&pending_location_101_issue())
            .expect("manager-gated rows should project to the manager read model");

    assert_eq!(manager_item.action_id, "dq-action-location-101");
    assert_eq!(manager_item.location_id, "101");
    assert_eq!(
        manager_item.required_review_gates,
        vec![ReviewGateColumn::ManagerApproval]
    );

    let mut staff_only = pending_location_101_issue();
    staff_only.required_review_gates.clear();
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
        ActorRefColumn::System,
        FeedbackOutcomeColumn::Completed,
        25,
        9,
        vec![SourceRecordRefColumn {
            system: SourceSystemColumn::Gingr,
            record_id: "reservation-101".to_owned(),
        }],
        vec!["dq-issue-location-101".to_owned()],
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
        attempted_side_effect: BlockedActionColumn::SendCustomerMessage,
        reason: BlockedActionReasonColumn::ActorLacksReviewGate,
        created_at: 9,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });

    assert_eq!(notice.action_id, "dq-action-location-101");
    assert_eq!(notice.actor_id, "sam");
    assert_eq!(notice.location_id, "101");
    assert_eq!(
        notice.attempted_side_effect,
        BlockedActionColumn::SendCustomerMessage
    );
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
    assert_eq!(
        notice.attempted_side_effect,
        BlockedActionColumn::RecordReviewedOutcome
    );
    assert_eq!(notice.reason_label, "actor_lacks_review_gate");
}

#[test]
fn blocked_action_rows_round_trip_typed_blocked_action_reasons_without_debug_formatting() {
    for action in BlockedActionColumn::ALL {
        if let Some(domain_action) = codec::blocked_action(action) {
            assert_eq!(codec::blocked_action_column(domain_action), action);
        }
    }
}

#[test]
fn review_queue_rows_round_trip_source_refs_review_gate_and_status_without_boolean_lossiness() {
    for gate in ReviewGateColumn::ALL {
        assert_eq!(codec::review_gate_column(codec::review_gate(gate)), gate);
    }
    for system in SourceSystemColumn::ALL {
        assert_eq!(
            codec::source_system_column(codec::source_system(system)),
            system
        );
    }
    for status in ReviewQueueStatusColumn::ALL {
        assert_eq!(codec::parse_status(codec::status_label(status)), Ok(status));
    }
}

#[test]
fn actor_and_outcome_codecs_round_trip_every_supported_variant() {
    use domain::{agent, entities};
    use uuid::Uuid;

    let actors = [
        entities::ActorRef::Customer(entities::CustomerId(Uuid::nil())),
        entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("staff-1").unwrap(),
        },
        entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("manager-1").unwrap(),
        },
        entities::ActorRef::System,
        entities::ActorRef::Agent {
            workflow: agent::Name::try_new("hygiene-review").unwrap(),
        },
    ];
    for actor in actors {
        let column = codec::actor_ref_column(&actor);
        assert_eq!(codec::actor_ref(&column).unwrap(), actor);
    }
    for outcome in FeedbackOutcomeColumn::ALL {
        assert_eq!(
            codec::feedback_outcome_column(codec::feedback_outcome(outcome)),
            outcome
        );
    }
}

#[test]
fn queue_transitions_reject_out_of_order_mutation_and_preserve_legal_sequence() {
    let mut row = pending_location_101_issue();

    assert_eq!(
        transition::attach_recommendation(&mut row, "alice", "review duplicate record".to_owned(),),
        Err(transition::Error::InvalidSourceState {
            operation: transition::Operation::AttachRecommendation,
            actual: ReviewQueueStatusColumn::PendingStaffReview,
        })
    );
    transition::claim(&mut row, "alice".to_owned()).unwrap();
    transition::attach_recommendation(&mut row, "alice", "review duplicate record".to_owned())
        .unwrap();
    transition::record_staff_disposition(
        &mut row,
        "alice",
        StaffDispositionColumn::RecommendForManagerApproval,
    )
    .unwrap();
    transition::record_manager_outcome(&mut row, ManagerOutcomeColumn::Approved).unwrap();
    transition::capture_outcome(&mut row, FeedbackOutcomeColumn::Completed).unwrap();
    assert_eq!(row.status, ReviewQueueStatusColumn::OutcomeRecorded);
    assert!(transition::capture_outcome(&mut row, FeedbackOutcomeColumn::Completed).is_err());
}

#[test]
fn rejected_transition_attempts_leave_queue_facts_unchanged() {
    let mut row = pending_location_101_issue();
    transition::claim(&mut row, "alice".to_owned()).unwrap();
    row.required_review_gates.clear();

    assert_eq!(
        transition::record_staff_disposition(
            &mut row,
            "alice",
            StaffDispositionColumn::RecommendForManagerApproval,
        ),
        Err(transition::Error::ManagerGateNotRequired)
    );
    assert_eq!(row.staff_disposition, None);
    assert_eq!(row.status, ReviewQueueStatusColumn::ClaimedByStaff);
}

#[test]
fn only_claim_owner_can_recommend_or_dispose_claimed_work() {
    let mut row = pending_location_101_issue();
    transition::claim(&mut row, "alice".to_owned()).unwrap();

    assert_eq!(
        transition::attach_recommendation(&mut row, "sam", "close duplicate".to_owned()),
        Err(transition::Error::ActorDoesNotOwnClaim)
    );
    assert_eq!(row.recommendation, None);
    assert_eq!(
        transition::record_staff_disposition(
            &mut row,
            "sam",
            StaffDispositionColumn::RecommendForManagerApproval,
        ),
        Err(transition::Error::ActorDoesNotOwnClaim)
    );
    assert_eq!(row.staff_disposition, None);
    assert_eq!(row.status, ReviewQueueStatusColumn::ClaimedByStaff);
}

#[test]
fn terminal_dispositions_only_authorize_semantically_matching_outcomes() {
    for (manager_outcome, feedback_outcome) in [
        (
            ManagerOutcomeColumn::Approved,
            FeedbackOutcomeColumn::Completed,
        ),
        (
            ManagerOutcomeColumn::Rejected,
            FeedbackOutcomeColumn::SuppressedByManager,
        ),
        (
            ManagerOutcomeColumn::Deferred,
            FeedbackOutcomeColumn::Deferred,
        ),
    ] {
        let mut row = pending_location_101_issue();
        transition::claim(&mut row, "alice".to_owned()).unwrap();
        transition::record_staff_disposition(
            &mut row,
            "alice",
            StaffDispositionColumn::RecommendForManagerApproval,
        )
        .unwrap();
        transition::record_manager_outcome(&mut row, manager_outcome).unwrap();
        transition::capture_outcome(&mut row, feedback_outcome).unwrap();
    }

    for staff_disposition in StaffDispositionColumn::ALL
        .into_iter()
        .filter(|disposition| *disposition != StaffDispositionColumn::RecommendForManagerApproval)
    {
        let mut row = pending_location_101_issue();
        row.required_review_gates.clear();
        transition::claim(&mut row, "alice".to_owned()).unwrap();
        transition::record_staff_disposition(&mut row, "alice", staff_disposition).unwrap();
        let feedback_outcome = match staff_disposition {
            StaffDispositionColumn::CompleteWithoutManagerApproval => {
                FeedbackOutcomeColumn::Completed
            }
            StaffDispositionColumn::Defer => FeedbackOutcomeColumn::Deferred,
            StaffDispositionColumn::RecommendForManagerApproval => unreachable!(),
        };
        transition::capture_outcome(&mut row, feedback_outcome).unwrap();
    }
}

#[test]
fn rejected_or_deferred_decisions_cannot_be_captured_as_completed() {
    for manager_outcome in [
        ManagerOutcomeColumn::Rejected,
        ManagerOutcomeColumn::Deferred,
    ] {
        let mut row = pending_location_101_issue();
        transition::claim(&mut row, "alice".to_owned()).unwrap();
        transition::record_staff_disposition(
            &mut row,
            "alice",
            StaffDispositionColumn::RecommendForManagerApproval,
        )
        .unwrap();
        transition::record_manager_outcome(&mut row, manager_outcome).unwrap();

        assert_eq!(
            transition::capture_outcome(&mut row, FeedbackOutcomeColumn::Completed),
            Err(transition::Error::OutcomeDoesNotMatchDisposition)
        );
        assert_ne!(row.status, ReviewQueueStatusColumn::OutcomeRecorded);
    }
}

#[test]
fn approved_decision_cannot_be_captured_as_deferred() {
    let mut row = pending_location_101_issue();
    transition::claim(&mut row, "alice".to_owned()).unwrap();
    transition::record_staff_disposition(
        &mut row,
        "alice",
        StaffDispositionColumn::RecommendForManagerApproval,
    )
    .unwrap();
    transition::record_manager_outcome(&mut row, ManagerOutcomeColumn::Approved).unwrap();

    assert_eq!(
        transition::capture_outcome(&mut row, FeedbackOutcomeColumn::Deferred),
        Err(transition::Error::OutcomeDoesNotMatchDisposition)
    );
    assert_ne!(row.status, ReviewQueueStatusColumn::OutcomeRecorded);
}

#[test]
fn blocked_side_effect_attempt_cannot_overwrite_recorded_outcome() {
    let mut row = pending_location_101_issue();
    transition::claim(&mut row, "alice".to_owned()).unwrap();
    transition::record_staff_disposition(
        &mut row,
        "alice",
        StaffDispositionColumn::RecommendForManagerApproval,
    )
    .unwrap();
    transition::record_manager_outcome(&mut row, ManagerOutcomeColumn::Approved).unwrap();
    transition::capture_outcome(&mut row, FeedbackOutcomeColumn::Completed).unwrap();

    assert_eq!(
        transition::block_unsafe_side_effect(&mut row),
        Err(transition::Error::InvalidSourceState {
            operation: transition::Operation::BlockUnsafeSideEffect,
            actual: ReviewQueueStatusColumn::OutcomeRecorded,
        })
    );
    assert_eq!(row.status, ReviewQueueStatusColumn::OutcomeRecorded);
}

#[test]
fn subscription_read_models_stay_private_until_server_scoped_authorization_exists() {
    let manager = include_str!("read_model/manager_queue_item.rs");
    let staff = include_str!("read_model/staff_queue_item.rs");

    assert!(!manager.contains("#[spacetimedb::table(accessor = manager_queue_item, public)]"));
    assert!(!staff.contains("public)]"));
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
