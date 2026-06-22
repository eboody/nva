use chrono::{DateTime, Utc};
use uuid::Uuid;

use app::checkout_completion;
use domain::{entities, policy, source};

#[test]
fn checkout_completion_contract_requires_staff_handoff_and_provenance_before_completed_status() {
    let request = checkout_completion::Request::builder()
        .reservation_id(reservation_id())
        .source_provenance(source_provenance())
        .observed_source_status(source::reservation::Status::CheckedOut)
        .staff_handoff(resolved_staff_handoff())
        .build();

    let packet = checkout_completion::Workflow::evaluate(request);

    assert_eq!(
        packet.completion_status(),
        checkout_completion::CompletionStatus::StaffVerifiedCheckout
    );
    assert_eq!(
        packet.suggested_reservation_status(),
        Some(entities::reservation::Status::CheckedOut)
    );
    assert_eq!(
        packet.required_review_gates(),
        &[policy::ReviewGate::CustomerMessageApproval]
    );
    assert!(
        packet
            .safe_agent_actions()
            .contains(&checkout_completion::SafeAgentAction::DraftRetentionFollowUpForReview)
    );
    assert!(
        packet
            .safe_agent_actions()
            .contains(&checkout_completion::SafeAgentAction::CreateInternalHandoffTask)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&checkout_completion::BlockedAction::SendCustomerMessage)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&checkout_completion::BlockedAction::MutateProviderOrPmsRecord)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&checkout_completion::BlockedAction::MoveRefundDiscountOrPayment)
    );
    assert_eq!(packet.provenance().record_id().as_str(), "reservation-42");
    assert!(
        packet
            .audit_event_drafts()
            .contains(&checkout_completion::AuditEventDraft::SourceCheckoutObserved)
    );
    assert!(
        packet
            .audit_event_drafts()
            .contains(&checkout_completion::AuditEventDraft::StaffHandoffRecorded)
    );
    assert!(
        packet
            .audit_event_drafts()
            .contains(&checkout_completion::AuditEventDraft::CheckoutCompletionSuggested)
    );
}

#[test]
fn checkout_completion_with_open_staff_handoff_routes_to_review_without_checkout_status_suggestion()
{
    let request = checkout_completion::Request::builder()
        .reservation_id(reservation_id())
        .source_provenance(source_provenance())
        .observed_source_status(source::reservation::Status::CheckedOut)
        .staff_handoff(open_staff_handoff())
        .build();

    let packet = checkout_completion::Workflow::evaluate(request);

    assert_eq!(
        packet.completion_status(),
        checkout_completion::CompletionStatus::NeedsStaffHandoffReview
    );
    assert_eq!(packet.suggested_reservation_status(), None);
    assert_eq!(
        packet.required_review_gates(),
        &[policy::ReviewGate::ManagerApproval]
    );
    assert!(
        packet
            .safe_agent_actions()
            .contains(&checkout_completion::SafeAgentAction::CreateInternalHandoffTask)
    );
    assert!(
        !packet
            .safe_agent_actions()
            .contains(&checkout_completion::SafeAgentAction::DraftRetentionFollowUpForReview)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&checkout_completion::BlockedAction::SuggestCheckedOutStatus)
    );
    assert!(
        packet
            .audit_event_drafts()
            .contains(&checkout_completion::AuditEventDraft::StaffHandoffReviewRequested)
    );
}

#[test]
fn checkout_exception_packet_names_unresolved_work_and_reviewed_outcome_without_side_effects() {
    let request = checkout_completion::Request::builder()
        .reservation_id(reservation_id())
        .source_provenance(source_provenance())
        .observed_source_status(source::reservation::Status::CheckedOut)
        .staff_handoff(open_staff_handoff())
        .payment_exception(checkout_completion::PaymentException::BalanceOrRefundReviewRequired)
        .source_exception(checkout_completion::SourceException::ProviderRecordConflict)
        .estimated_manual_audit_minutes(checkout_completion::LaborMinutes::try_new(18).unwrap())
        .estimated_packet_review_minutes(checkout_completion::LaborMinutes::try_new(6).unwrap())
        .build();

    let packet = checkout_completion::Workflow::evaluate(request);

    assert_eq!(
        packet.completion_status(),
        checkout_completion::CompletionStatus::NeedsStaffHandoffReview
    );
    assert_eq!(
        packet.unresolved_exceptions(),
        &[
            checkout_completion::UnresolvedException::Belongings,
            checkout_completion::UnresolvedException::Care,
            checkout_completion::UnresolvedException::Payment(
                checkout_completion::PaymentException::BalanceOrRefundReviewRequired
            ),
            checkout_completion::UnresolvedException::Source(
                checkout_completion::SourceException::ProviderRecordConflict
            ),
        ]
    );
    assert_eq!(
        packet.staff_task_drafts(),
        &[
            checkout_completion::StaffTaskDraft::VerifyBelongingsReturn,
            checkout_completion::StaffTaskDraft::ReviewCareAndDepartureNotes,
            checkout_completion::StaffTaskDraft::ResolvePaymentException,
            checkout_completion::StaffTaskDraft::ReconcileSourceStatus,
        ]
    );
    assert_eq!(
        packet.reviewed_disposition(),
        checkout_completion::ReviewedDisposition::ManagerReviewRequired
    );
    assert_eq!(packet.labor_impact().manual_audit_minutes().get(), 18);
    assert_eq!(packet.labor_impact().packet_review_minutes().get(), 6);
    assert_eq!(packet.labor_impact().estimated_minutes_saved(), Some(12));
    assert!(
        packet
            .blocked_actions()
            .contains(&checkout_completion::BlockedAction::MoveRefundDiscountOrPayment)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&checkout_completion::BlockedAction::MutateProviderOrPmsRecord)
    );
}

#[test]
fn checkout_completion_without_source_checkout_does_not_emit_false_checkout_observed_audit() {
    let request = checkout_completion::Request::builder()
        .reservation_id(reservation_id())
        .source_provenance(source_provenance())
        .observed_source_status(source::reservation::Status::CheckedIn)
        .staff_handoff(resolved_staff_handoff())
        .build();

    let packet = checkout_completion::Workflow::evaluate(request);

    assert_eq!(
        packet.completion_status(),
        checkout_completion::CompletionStatus::SourceNotCheckedOut
    );
    assert_eq!(packet.suggested_reservation_status(), None);
    assert_eq!(
        packet.required_review_gates(),
        &[policy::ReviewGate::ManagerApproval]
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&checkout_completion::BlockedAction::SuggestCheckedOutStatus)
    );
    assert!(
        !packet
            .audit_event_drafts()
            .contains(&checkout_completion::AuditEventDraft::SourceCheckoutObserved)
    );
    assert!(
        !packet
            .audit_event_drafts()
            .contains(&checkout_completion::AuditEventDraft::CheckoutCompletionSuggested)
    );
    assert!(
        !packet
            .audit_event_drafts()
            .contains(&checkout_completion::AuditEventDraft::CustomerMessageApprovalRequested)
    );
    assert!(
        packet
            .audit_event_drafts()
            .contains(&checkout_completion::AuditEventDraft::StaffHandoffRecorded)
    );
    assert!(
        packet
            .audit_event_drafts()
            .contains(&checkout_completion::AuditEventDraft::StaffHandoffReviewRequested)
    );
}

fn reservation_id() -> entities::reservation::Id {
    entities::reservation::Id(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0042))
}

fn resolved_staff_handoff() -> checkout_completion::StaffHandoff {
    checkout_completion::StaffHandoff::builder()
        .completed_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin").unwrap(),
        })
        .completed_at(DateTime::<Utc>::UNIX_EPOCH)
        .belongings_status(checkout_completion::BelongingsStatus::ReturnedToCustomer)
        .care_summary(
            checkout_completion::CareSummary::try_new(
                "Ate dinner, medication given, calm departure.",
            )
            .unwrap(),
        )
        .departure_notes_review(checkout_completion::DepartureNotesReview::StaffReviewed)
        .build()
}

fn open_staff_handoff() -> checkout_completion::StaffHandoff {
    checkout_completion::StaffHandoff::builder()
        .completed_by(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("front-desk-erin").unwrap(),
        })
        .completed_at(DateTime::<Utc>::UNIX_EPOCH)
        .belongings_status(checkout_completion::BelongingsStatus::NeedsStaffFollowUp)
        .care_summary(
            checkout_completion::CareSummary::try_new("Medication bag needs second staff check.")
                .unwrap(),
        )
        .departure_notes_review(checkout_completion::DepartureNotesReview::ManagerReviewRequired)
        .build()
}

fn source_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new("GET /reservations/{id}").unwrap())
        .record_id(source::record::Id::try_new("reservation-42").unwrap())
        .extraction_batch(source::ExtractionBatchId::try_new("checkout-batch-local").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-06-17T00:00:00Z").unwrap())
        .request_scope(source::RequestScope::try_new("local-checkout-completion-contract").unwrap())
        .schema_version(source::SchemaVersion::try_new("gingr-v0-readonly").unwrap())
        .payload_hash(source::PayloadHash::try_new("sha256:checkoutfixture").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/reservation-check-out.json").unwrap(),
        )
        .build()
}
