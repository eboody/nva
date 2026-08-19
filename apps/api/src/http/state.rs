use super::*;

pub(super) static VACCINE_DOCUMENT_STATE: std::sync::OnceLock<VaccineDocumentState> =
    std::sync::OnceLock::new();

#[derive(Clone)]
/// In-memory state kept on the API shell for deterministic workflow demos and tests.
///
/// The state stores documents, review packets, inquiry intake records, and labor-evidence
/// projections so HTTP handlers can demonstrate runtime rules without connecting to
/// live databases, customer messaging, or provider write APIs.
pub struct VaccineDocumentState {
    pub(super) store: Arc<Mutex<VaccineDocumentStore>>,
    pub(super) observability: ObservabilityRuntime,
    pub(super) contract_observation: contract_observation::State,
}

impl Default for VaccineDocumentState {
    fn default() -> Self {
        Self {
            store: Arc::new(Mutex::new(VaccineDocumentStore::default())),
            observability: ObservabilityRuntime::default(),
            contract_observation: contract_observation::State::default(),
        }
    }
}

impl VaccineDocumentState {
    /// Replaces the local telemetry runtime while preserving workflow state ownership.
    pub fn with_observability(mut self, observability: ObservabilityRuntime) -> Self {
        self.observability = observability;
        self
    }
}

#[derive(Default)]
pub(super) struct VaccineDocumentStore {
    pub(super) documents: BTreeMap<Uuid, DocumentRecord>,
    pub(super) extractions: BTreeMap<Uuid, VaccineExtractionRecord>,
    pub(super) vaccine_records: BTreeMap<Uuid, VaccineRecord>,
    pub(super) review_packets: BTreeMap<Uuid, ReviewPacket>,
    pub(super) approvals: BTreeMap<Uuid, ApprovalRecord>,
    pub(super) eligibility: BTreeMap<Uuid, PetEligibility>,
    pub(super) manager_daily_brief_outcomes: storage::workflow_repository::InMemoryOutcomes<
        storage::operations::ManagerDailyBriefOutcomeRecord,
    >,
    pub(super) data_quality_hygiene_outcomes: storage::workflow_repository::InMemoryOutcomes<
        storage::operations::DataQualityHygieneOutcomeRecord,
    >,
    pub(super) data_quality_hygiene_persistence_records:
        Vec<storage::operations::DataQualityHygieneLocalPersistenceRecords>,
    pub(super) data_quality_hygiene_idempotency: BTreeMap<String, DataQualityHygieneReplay>,
    pub(super) inquiry_intake_records: Vec<InquiryIntakeRecord>,
    pub(super) audit_events: Vec<AuditEvent>,
}

#[derive(Debug, Clone)]
pub(super) struct DataQualityHygieneReplay {
    pub(super) payload_fingerprint: String,
    pub(super) response: Value,
}

impl workflow_repository::Repository for VaccineDocumentStore {
    fn runtime_counters(&self) -> workflow_repository::RuntimeCounters {
        workflow_repository::RuntimeCounters {
            inquiry_count: self.inquiry_intake_records.len(),
            review_packet_count: self.review_packets.len(),
            audit_event_count: self.audit_events.len(),
            outcome_count: self.manager_daily_brief_outcomes.outcomes().len()
                + self.data_quality_hygiene_outcomes.outcomes().len(),
            internal_outbox_candidate_count: self
                .data_quality_hygiene_persistence_records
                .iter()
                .filter(|records| records.outbox_candidate.is_some())
                .count(),
            review_gated_internal_outbox_count: self
                .data_quality_hygiene_persistence_records
                .iter()
                .filter(|records| {
                    records.outbox_candidate.as_ref().is_some_and(|candidate| {
                        candidate.status() == storage::operations::OutboxStatusCode::Pending
                            && candidate
                                .payload()
                                .get("live_delivery_allowed")
                                .and_then(Value::as_bool)
                                == Some(false)
                    })
                })
                .count(),
        }
    }
}

impl VaccineDocumentStore {
    pub(super) fn apply_vaccine_review_decision(
        &mut self,
        review_packet_id: Uuid,
        decision: VaccineReviewDecision,
        evidence: VaccineReviewDecisionEvidence,
    ) -> Result<VaccineDocumentWorkflowPayload, VaccineReviewDecisionRejection> {
        let packet = self
            .review_packets
            .get(&review_packet_id)
            .cloned()
            .ok_or(VaccineReviewDecisionRejection::PacketNotFound { review_packet_id })?;

        if let Some(existing) = VaccineReviewDecision::from_decided_status(packet.status) {
            return Err(VaccineReviewDecisionRejection::PacketAlreadyDecided {
                review_packet_id,
                existing,
                attempted: decision,
            });
        }

        let document_id = packet.document_id;
        let vaccine_record_id = packet.vaccine_record_id;
        let pet_id = self
            .vaccine_records
            .get(&vaccine_record_id)
            .map(|record| record.pet_id)
            .ok_or(VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_record_not_found",
            })?;
        self.documents.get(&document_id).ok_or(
            VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_document_not_found",
            },
        )?;
        self.extractions.get(&document_id).ok_or(
            VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_extraction_not_found",
            },
        )?;
        self.eligibility.get(&pet_id).ok_or(
            VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_eligibility_not_found",
            },
        )?;

        self.review_packets
            .get_mut(&review_packet_id)
            .ok_or(VaccineReviewDecisionRejection::PacketNotFound { review_packet_id })?
            .status = decision.status_code();
        self.documents
            .get_mut(&document_id)
            .ok_or(VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_document_not_found",
            })?
            .verification_status = decision.document_verification_status();
        self.vaccine_records
            .get_mut(&vaccine_record_id)
            .ok_or(VaccineReviewDecisionRejection::BrokenWorkflowState {
                review_packet_id,
                code: "vaccine_review_record_not_found",
            })?
            .status = decision.vaccine_record_status();
        let eligibility = decision.eligibility(pet_id, vaccine_record_id);
        self.eligibility.insert(pet_id, eligibility);

        let approval = ApprovalRecord {
            id: Uuid::new_v4(),
            review_packet_id,
            target_document_id: document_id,
            target_vaccine_record_id: vaccine_record_id,
            gate: "medical_document_review",
            status: decision.status_code(),
            decided_by_staff_id: evidence.reviewed_by_staff_id.clone(),
            decided_at: evidence.decided_at,
            reason: evidence.reason,
        };
        self.approvals.insert(approval.id, approval.clone());
        self.audit_events.push(audit(
            "approval.decision.recorded",
            &evidence.reviewed_by_staff_id,
            "approval",
            approval.id,
            [("status", approval.status.to_owned())],
        ));
        self.audit_events.push(audit(
            "pet.eligibility.updated",
            &evidence.reviewed_by_staff_id,
            "pet",
            pet_id,
            [(
                "rabies_current",
                matches!(decision, VaccineReviewDecision::Approve).to_string(),
            )],
        ));

        Ok(self.payload(
            document_id,
            vaccine_record_id,
            review_packet_id,
            Some(approval),
        ))
    }

    pub(super) fn payload_for_review_conflict(
        &self,
        review_packet_id: Uuid,
        existing: VaccineReviewDecision,
        attempted: VaccineReviewDecision,
    ) -> Value {
        let Some(packet) = self.review_packets.get(&review_packet_id) else {
            return json!({
                "api_contract": api_dto_contract_payload("vaccine_document_review"),
                "accepted": false,
                "error": {"code": "vaccine_review_packet_not_found"},
                "review_packet_id": review_packet_id
            });
        };
        let workflow = self.payload(
            packet.document_id,
            packet.vaccine_record_id,
            review_packet_id,
            None,
        );
        json!({
            "api_contract": api_dto_contract_payload("vaccine_document_review"),
            "accepted": false,
            "error": {
                "code": "vaccine_review_packet_already_decided",
                "message": "A vaccine review packet accepts exactly one approve or reject decision."
            },
            "review_packet_id": review_packet_id,
            "existing_decision": existing.status_code(),
            "attempted_decision": attempted.status_code(),
            "document": workflow.document,
            "vaccine_record": workflow.vaccine_record,
            "review_packet": workflow.review_packet,
            "eligibility": workflow.eligibility,
            "approval": workflow.approval
        })
    }

    pub(super) fn payload(
        &self,
        document_id: Uuid,
        vaccine_record_id: Uuid,
        review_packet_id: Uuid,
        approval: Option<ApprovalRecord>,
    ) -> VaccineDocumentWorkflowPayload {
        let document = self.documents.get(&document_id).expect("document").clone();
        let extraction = self
            .extractions
            .get(&document_id)
            .expect("extraction")
            .clone();
        let vaccine_record = self
            .vaccine_records
            .get(&vaccine_record_id)
            .expect("vaccine record")
            .clone();
        let review_packet = self
            .review_packets
            .get(&review_packet_id)
            .expect("review packet")
            .clone();
        let eligibility = self
            .eligibility
            .get(&vaccine_record.pet_id)
            .expect("eligibility")
            .clone();
        VaccineDocumentWorkflowPayload {
            api_contract: api_dto_contract("vaccine_document_review"),
            document,
            extraction,
            vaccine_record,
            review_packet,
            approval,
            eligibility,
            audit_events: self.audit_events.clone(),
        }
    }
}
