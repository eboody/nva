use chrono::{DateTime, Utc};
use uuid::Uuid;

use app::{checkout_completion, crm_retention};
use domain::{entities, grooming, message, policy, source};

#[test]
fn retention_follow_up_contract_builds_draft_only_review_packet_from_source_grounded_opportunity() {
    let request = crm_retention::Request::builder()
        .reservation_id(reservation_id())
        .customer_id(customer_id())
        .checkout_packet(checkout_packet())
        .contact_permission(email_contact_permission())
        .opportunities(vec![next_stay_opportunity()])
        .build();

    let packet = crm_retention::Workflow::evaluate(request);

    assert_eq!(
        packet.eligibility(),
        crm_retention::FollowUpEligibility::Eligible {
            reason: crm_retention::EligibilityReason::SourceGroundedRetentionOpportunity
        }
    );
    assert_eq!(packet.draft_channel(), Some(message::Channel::Email));
    assert_eq!(
        packet.required_review_gates(),
        &[policy::ReviewGate::CustomerMessageApproval]
    );
    assert!(packet.review_packet().requires_human_review());
    assert_eq!(
        packet.review_packet().staff_evidence()[0].reason_code(),
        crm_retention::SourceGroundedReasonCode::CompletedBoardingStay
    );
    assert_eq!(
        packet.review_packet().staff_evidence()[0]
            .provenance()
            .record_id()
            .as_str(),
        "reservation-42"
    );
    assert!(
        packet
            .source_record_refs()
            .contains(&source::RecordRef::from_provenance(&contact_provenance()))
    );
    assert!(
        packet
            .safe_agent_actions()
            .contains(&crm_retention::SafeAgentAction::DraftCustomerFollowUpForReview)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&crm_retention::BlockedAction::SendCustomerMessage)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&crm_retention::BlockedAction::MutateProviderOrPmsRecord)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&crm_retention::BlockedAction::MoveRefundDiscountOrPayment)
    );
}

#[test]
fn retention_follow_up_contract_requires_source_grounded_contact_permission_for_customer_draft() {
    let request = crm_retention::Request::builder()
        .reservation_id(reservation_id())
        .customer_id(customer_id())
        .checkout_packet(checkout_packet())
        .contact_permission(ungrounded_email_contact_permission())
        .opportunities(vec![next_stay_opportunity()])
        .build();

    let packet = crm_retention::Workflow::evaluate(request);

    assert_eq!(
        packet.eligibility(),
        crm_retention::FollowUpEligibility::Ineligible {
            reason: crm_retention::IneligibilityReason::ContactPermissionNotSourceGrounded
        }
    );
    assert_eq!(packet.draft_channel(), Some(message::Channel::Email));
    assert_eq!(
        packet.required_review_gates(),
        &[policy::ReviewGate::ManagerApproval]
    );
    assert!(
        !packet
            .safe_agent_actions()
            .contains(&crm_retention::SafeAgentAction::DraftCustomerFollowUpForReview)
    );
}

#[test]
fn retention_follow_up_contract_suppresses_customer_draft_when_contact_consent_is_missing() {
    let request = crm_retention::Request::builder()
        .reservation_id(reservation_id())
        .customer_id(customer_id())
        .checkout_packet(checkout_packet())
        .contact_permission(no_contact_permission())
        .opportunities(vec![next_stay_opportunity()])
        .build();

    let packet = crm_retention::Workflow::evaluate(request);

    assert_eq!(
        packet.eligibility(),
        crm_retention::FollowUpEligibility::Ineligible {
            reason: crm_retention::IneligibilityReason::ContactConsentMissing
        }
    );
    assert_eq!(packet.draft_channel(), None);
    assert_eq!(
        packet.required_review_gates(),
        &[policy::ReviewGate::ManagerApproval]
    );
    assert!(
        !packet
            .safe_agent_actions()
            .contains(&crm_retention::SafeAgentAction::DraftCustomerFollowUpForReview)
    );
    assert!(
        packet
            .safe_agent_actions()
            .contains(&crm_retention::SafeAgentAction::CreateInternalStaffReviewTask)
    );
}

#[test]
fn retention_follow_up_contract_distinguishes_opt_out_from_unavailable_channel() {
    let opted_out = crm_retention::Workflow::evaluate(
        crm_retention::Request::builder()
            .reservation_id(reservation_id())
            .customer_id(customer_id())
            .checkout_packet(checkout_packet())
            .contact_permission(opted_out_contact_permission())
            .opportunities(vec![next_stay_opportunity()])
            .build(),
    );

    assert_eq!(
        opted_out.eligibility(),
        crm_retention::FollowUpEligibility::Ineligible {
            reason: crm_retention::IneligibilityReason::ContactOptedOut
        }
    );
    assert_eq!(opted_out.draft_channel(), None);

    let unavailable_channel = crm_retention::Workflow::evaluate(
        crm_retention::Request::builder()
            .reservation_id(reservation_id())
            .customer_id(customer_id())
            .checkout_packet(checkout_packet())
            .contact_permission(unavailable_preferred_channel_permission())
            .opportunities(vec![next_stay_opportunity()])
            .build(),
    );

    assert_eq!(
        unavailable_channel.eligibility(),
        crm_retention::FollowUpEligibility::Ineligible {
            reason: crm_retention::IneligibilityReason::PreferredChannelNotAllowed
        }
    );
    assert_eq!(unavailable_channel.draft_channel(), None);
}

#[test]
fn grooming_rebooking_packet_carries_reviewable_draft_suppression_and_outcome_boundaries() {
    let request = crm_retention::Request::builder()
        .reservation_id(reservation_id())
        .customer_id(customer_id())
        .checkout_packet(checkout_packet())
        .contact_permission(email_contact_permission())
        .opportunities(vec![grooming_rebook_opportunity()])
        .suppression_flags(vec![
            crm_retention::SuppressionFlag::ComplaintOrServiceRecoveryReview,
        ])
        .build();

    let packet = crm_retention::Workflow::evaluate(request);

    assert_eq!(
        packet.review_packet().opportunities()[0].reason(),
        crm_retention::OpportunityReason::GroomingCadenceDue {
            status: grooming::rebooking::Status::DueNow,
            rationale: grooming::rebooking::Rationale::LastCompletedServiceCadence,
        }
    );
    assert_eq!(
        packet.review_packet().draft_follow_up().review_state(),
        message::ReviewState::Suppressed
    );
    assert_eq!(
        packet.review_packet().draft_follow_up().channel(),
        message::Channel::Email
    );
    assert!(
        packet
            .review_packet()
            .draft_follow_up()
            .suppression_flags()
            .contains(&crm_retention::SuppressionFlag::ComplaintOrServiceRecoveryReview)
    );
    assert!(
        packet
            .safe_agent_actions()
            .contains(&crm_retention::SafeAgentAction::CreateInternalStaffReviewTask)
    );
    assert!(
        !packet
            .safe_agent_actions()
            .contains(&crm_retention::SafeAgentAction::DraftCustomerFollowUpForReview)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&crm_retention::BlockedAction::CreateOrChangeBooking)
    );

    let suppressed = crm_retention::OutcomeRecord::builder()
        .reservation_id(reservation_id())
        .customer_id(customer_id())
        .recorded_by(staff_actor())
        .recorded_at(DateTime::<Utc>::UNIX_EPOCH)
        .outcome(crm_retention::FollowUpOutcome::Suppressed {
            reason: crm_retention::SuppressionFlag::ComplaintOrServiceRecoveryReview,
        })
        .source_provenance(source_provenance())
        .evidence(vec![grooming_rebook_evidence()])
        .build();
    assert!(suppressed.records_staff_evidence_only());

    let converted = crm_retention::OutcomeRecord::builder()
        .reservation_id(reservation_id())
        .customer_id(customer_id())
        .recorded_by(staff_actor())
        .recorded_at(DateTime::<Utc>::UNIX_EPOCH)
        .outcome(crm_retention::FollowUpOutcome::Converted {
            conversion: crm_retention::ConversionKind::GroomingRebooked,
        })
        .source_provenance(source_provenance())
        .build();
    assert_eq!(
        converted.outcome(),
        crm_retention::FollowUpOutcome::Converted {
            conversion: crm_retention::ConversionKind::GroomingRebooked,
        }
    );

    let deferred = crm_retention::OutcomeRecord::builder()
        .reservation_id(reservation_id())
        .customer_id(customer_id())
        .recorded_by(staff_actor())
        .recorded_at(DateTime::<Utc>::UNIX_EPOCH)
        .outcome(crm_retention::FollowUpOutcome::Deferred {
            reason: crm_retention::DeferralReason::WaitingOnCustomerOrStaffReview,
        })
        .source_provenance(source_provenance())
        .build();
    assert_eq!(
        deferred.outcome(),
        crm_retention::FollowUpOutcome::Deferred {
            reason: crm_retention::DeferralReason::WaitingOnCustomerOrStaffReview,
        }
    );

    let wrong_source = crm_retention::OutcomeRecord::builder()
        .reservation_id(reservation_id())
        .customer_id(customer_id())
        .recorded_by(staff_actor())
        .recorded_at(DateTime::<Utc>::UNIX_EPOCH)
        .outcome(crm_retention::FollowUpOutcome::WrongSource)
        .source_provenance(source_provenance())
        .build();
    assert_eq!(
        wrong_source.outcome(),
        crm_retention::FollowUpOutcome::WrongSource
    );
}

#[test]
fn retention_outcome_capture_records_staff_evidence_without_live_provider_mutation() {
    let outcome = crm_retention::OutcomeRecord::builder()
        .reservation_id(reservation_id())
        .customer_id(customer_id())
        .recorded_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin").unwrap(),
        })
        .recorded_at(DateTime::<Utc>::UNIX_EPOCH)
        .outcome(crm_retention::FollowUpOutcome::BookedNextStay)
        .source_provenance(source_provenance())
        .evidence(vec![next_stay_evidence()])
        .build();

    assert_eq!(
        outcome.outcome(),
        crm_retention::FollowUpOutcome::BookedNextStay
    );
    assert_eq!(
        outcome.evidence()[0].reason_code(),
        crm_retention::SourceGroundedReasonCode::CompletedBoardingStay
    );
    assert!(outcome.records_staff_evidence_only());
    assert!(
        outcome
            .blocked_actions()
            .contains(&crm_retention::BlockedAction::MutateProviderOrPmsRecord)
    );
    assert!(
        outcome
            .blocked_actions()
            .contains(&crm_retention::BlockedAction::SendCustomerMessage)
    );
}

fn checkout_packet() -> checkout_completion::Packet {
    let request = checkout_completion::Request::builder()
        .reservation_id(reservation_id())
        .source_provenance(source_provenance())
        .observed_source_status(source::reservation::Status::CheckedOut)
        .staff_handoff(
            checkout_completion::StaffHandoff::builder()
                .completed_by(entities::ActorRef::Staff {
                    staff_id: entities::StaffId::try_new("front-desk-erin").unwrap(),
                })
                .completed_at(DateTime::<Utc>::UNIX_EPOCH)
                .belongings_status(checkout_completion::BelongingsStatus::ReturnedToCustomer)
                .care_summary(
                    checkout_completion::CareSummary::try_new(
                        "Calm checkout; owner mentioned another trip next month.",
                    )
                    .unwrap(),
                )
                .departure_notes_review(checkout_completion::DepartureNotesReview::StaffReviewed)
                .build(),
        )
        .build();

    checkout_completion::Workflow::evaluate(request)
}

fn next_stay_opportunity() -> crm_retention::RetentionOpportunity {
    crm_retention::RetentionOpportunity::builder()
        .kind(crm_retention::OpportunityKind::NextBoardingStay)
        .reason(crm_retention::OpportunityReason::BoardingStayCompleted)
        .evidence(next_stay_evidence())
        .build()
}

fn grooming_rebook_opportunity() -> crm_retention::RetentionOpportunity {
    crm_retention::RetentionOpportunity::builder()
        .kind(crm_retention::OpportunityKind::GroomingRebook)
        .reason(crm_retention::OpportunityReason::GroomingCadenceDue {
            status: grooming::rebooking::Status::DueNow,
            rationale: grooming::rebooking::Rationale::LastCompletedServiceCadence,
        })
        .evidence(grooming_rebook_evidence())
        .build()
}

fn next_stay_evidence() -> crm_retention::OpportunityEvidence {
    crm_retention::OpportunityEvidence::builder()
        .reason_code(crm_retention::SourceGroundedReasonCode::CompletedBoardingStay)
        .summary(
            crm_retention::EvidenceSummary::try_new(
                "Completed boarding stay with clean staff handoff and owner travel intent noted.",
            )
            .unwrap(),
        )
        .provenance(source_provenance())
        .build()
}

fn grooming_rebook_evidence() -> crm_retention::OpportunityEvidence {
    crm_retention::OpportunityEvidence::builder()
        .reason_code(crm_retention::SourceGroundedReasonCode::CompletedGroomingVisit)
        .summary(
            crm_retention::EvidenceSummary::try_new(
                "Completed grooming service is due for rebooking on the ordinary cadence.",
            )
            .unwrap(),
        )
        .provenance(grooming_provenance())
        .build()
}

fn email_contact_permission() -> crm_retention::ContactPermission {
    crm_retention::ContactPermission::builder()
        .preferred_channel(message::Channel::Email)
        .allowed_channels(vec![message::Channel::Email, message::Channel::Portal])
        .marketing_consent(crm_retention::ConsentStatus::Granted)
        .transactional_consent(crm_retention::ConsentStatus::Granted)
        .source_record_refs(vec![source::RecordRef::from_provenance(
            &contact_provenance(),
        )])
        .build()
}

fn ungrounded_email_contact_permission() -> crm_retention::ContactPermission {
    crm_retention::ContactPermission::builder()
        .preferred_channel(message::Channel::Email)
        .allowed_channels(vec![message::Channel::Email, message::Channel::Portal])
        .marketing_consent(crm_retention::ConsentStatus::Granted)
        .transactional_consent(crm_retention::ConsentStatus::Granted)
        .build()
}

fn no_contact_permission() -> crm_retention::ContactPermission {
    crm_retention::ContactPermission::builder()
        .preferred_channel(message::Channel::Email)
        .allowed_channels(vec![message::Channel::Email])
        .marketing_consent(crm_retention::ConsentStatus::Missing)
        .transactional_consent(crm_retention::ConsentStatus::Missing)
        .build()
}

fn opted_out_contact_permission() -> crm_retention::ContactPermission {
    crm_retention::ContactPermission::builder()
        .preferred_channel(message::Channel::Email)
        .allowed_channels(vec![message::Channel::Email])
        .marketing_consent(crm_retention::ConsentStatus::OptedOut)
        .transactional_consent(crm_retention::ConsentStatus::Granted)
        .build()
}

fn unavailable_preferred_channel_permission() -> crm_retention::ContactPermission {
    crm_retention::ContactPermission::builder()
        .preferred_channel(message::Channel::Sms)
        .allowed_channels(vec![message::Channel::Email])
        .marketing_consent(crm_retention::ConsentStatus::Granted)
        .transactional_consent(crm_retention::ConsentStatus::Granted)
        .build()
}

fn contact_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new("GET /customers/{id}/contact-permissions").unwrap())
        .record_id(source::record::Id::try_new("customer-contact-99").unwrap())
        .extraction_batch(source::ExtractionBatchId::try_new("retention-batch-local").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-06-17T00:00:00Z").unwrap())
        .request_scope(source::RequestScope::try_new("local-retention-follow-up-contract").unwrap())
        .schema_version(source::SchemaVersion::try_new("gingr-v0-readonly").unwrap())
        .payload_hash(source::PayloadHash::try_new("sha256:retentioncontactfixture").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/customer-contact-permissions.json")
                .unwrap(),
        )
        .build()
}

fn reservation_id() -> entities::reservation::Id {
    entities::reservation::Id(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0042))
}

fn customer_id() -> entities::CustomerId {
    entities::CustomerId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0099))
}

fn staff_actor() -> entities::ActorRef {
    entities::ActorRef::Staff {
        staff_id: entities::StaffId::try_new("front-desk-erin").unwrap(),
    }
}

fn source_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new("GET /reservations/{id}").unwrap())
        .record_id(source::record::Id::try_new("reservation-42").unwrap())
        .extraction_batch(source::ExtractionBatchId::try_new("retention-batch-local").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-06-17T00:00:00Z").unwrap())
        .request_scope(source::RequestScope::try_new("local-retention-follow-up-contract").unwrap())
        .schema_version(source::SchemaVersion::try_new("gingr-v0-readonly").unwrap())
        .payload_hash(source::PayloadHash::try_new("sha256:retentionfixture").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/reservation-retention.json").unwrap(),
        )
        .build()
}

fn grooming_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new("GET /grooming/services/{id}").unwrap())
        .record_id(source::record::Id::try_new("grooming-service-77").unwrap())
        .extraction_batch(source::ExtractionBatchId::try_new("retention-batch-local").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-06-17T00:00:00Z").unwrap())
        .request_scope(source::RequestScope::try_new("local-retention-follow-up-contract").unwrap())
        .schema_version(source::SchemaVersion::try_new("gingr-v0-readonly").unwrap())
        .payload_hash(source::PayloadHash::try_new("sha256:groomingretentionfixture").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/grooming-retention.json").unwrap(),
        )
        .build()
}
