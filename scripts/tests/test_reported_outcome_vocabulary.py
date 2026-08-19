import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class ReportedOutcomeVocabularyTest(unittest.TestCase):
    def test_caller_report_boundaries_do_not_claim_review_or_completion(self):
        crm = (REPO_ROOT / "app/src/crm_retention.rs").read_text(encoding="utf-8")
        checkout = (REPO_ROOT / "app/src/checkout_completion.rs").read_text(encoding="utf-8")
        hygiene = (REPO_ROOT / "app/src/data_quality_hygiene.rs").read_text(encoding="utf-8")
        reducers = (REPO_ROOT / "apps/spacetimedb/src/reducers.rs").read_text(encoding="utf-8")
        runtime = (REPO_ROOT / "apps/spacetimedb/src/runtime.rs").read_text(encoding="utf-8")
        storage = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (
                REPO_ROOT / "storage/src/operations.rs",
                REPO_ROOT / "storage/src/operations/approval_outbox.rs",
                REPO_ROOT / "storage/src/operations/service_catalog.rs",
                REPO_ROOT / "storage/src/operations/data_quality.rs",
                REPO_ROOT / "storage/src/operations/manager_daily_brief.rs",
                REPO_ROOT / "storage/src/operations/site_finance.rs",
                REPO_ROOT / "storage/src/operations/workflow.rs",
            )
        )
        api_contract = (REPO_ROOT / "apps/api/src/public_contract.rs").read_text(encoding="utf-8")
        api_http = (REPO_ROOT / "apps/api/src/http.rs").read_text(encoding="utf-8")
        workflow_source = (REPO_ROOT / "domain/src/workflow.rs").read_text(encoding="utf-8")
        lead_source = (REPO_ROOT / "domain/src/lead.rs").read_text(encoding="utf-8")
        labor_crosswalk = (REPO_ROOT / "docs/safety/labor-cost-with-human-review-crosswalk.md").read_text(encoding="utf-8")
        openapi = (REPO_ROOT / "apps/api/openapi/owned-operations-v1.openapi.json").read_text(encoding="utf-8")

        for forbidden in (
            "Uses completed boarding stay as source-grounded evidence",
            "Uses completed daycare visit as source-grounded evidence",
            "Uses completed grooming visit as source-grounded evidence",
        ):
            self.assertNotIn(forbidden, crm)

        self.assertNotIn("routes reviewed source facts", checkout)
        self.assertNotIn("Selects staff reviewed", checkout)

        for forbidden in (
            "reviewed data-quality hygiene outcome",
            "record_reviewed_outcome",
            "reviewed_resolution_status",
        ):
            self.assertNotIn(forbidden, hygiene.lower())

        for forbidden in (
            "record_reviewed_hygiene_outcome",
            "reviewed_outcome_provenance",
            "reviewed_resolution_status",
            "captures a reviewed data-quality hygiene outcome",
        ):
            self.assertNotIn(forbidden, reducers.lower())

        self.assertNotIn("record_reviewed_outcome", runtime)
        self.assertIn("record_reported_outcome", runtime)

        self.assertNotIn("resolution_status_after_review", api_contract)
        self.assertNotIn('"resolution_status_after_review"', openapi)
        self.assertNotIn('"resolution_status_after_review": record.', api_http)
        self.assertIn("reported_resolution_status", api_contract)
        self.assertIn('"reported_resolution_status"', openapi)
        self.assertNotIn("pub fn completed(", workflow_source)
        self.assertNotIn("pub fn is_met_at(", lead_source)
        self.assertNotIn("pub fn first_response_sla_is_met_at(", lead_source)
        self.assertNotIn("ordered reviewed attempt can become a packet", lead_source)
        for forbidden in (
            "Staff handoff used by the checkout completion workflow",
            "Returns the completed by evidence",
            "Returns the completed at evidence",
        ):
            self.assertNotIn(forbidden, checkout)
        for forbidden in ("reviewed_simulated_outcome", "met_by_reviewed_draft_attempt", "awaiting_reviewed_attempt"):
            self.assertNotIn(forbidden, api_http)
        for forbidden in ("Reviewed/deferred/wrong-source/completed outcomes", "estimated and actual minutes", "Reviewed/deferred/fixed/wrong-source outcome"):
            self.assertNotIn(forbidden, labor_crosswalk)

        manager_doc = (REPO_ROOT / "docs/workflows/operator/manager-daily-brief.md").read_text(encoding="utf-8")
        provenance_doc = (REPO_ROOT / "docs/design/source-provenance-data-quality-atlas.md").read_text(encoding="utf-8")
        overlay_template = (REPO_ROOT / "docs/safety/entity-action-overlays/template.md").read_text(encoding="utf-8")
        outcome_roles = (REPO_ROOT / "docs/safety/entity-action-overlays/outcome-audit-review-roles.md").read_text(encoding="utf-8")
        crm_retention = (REPO_ROOT / "app/src/crm_retention.rs").read_text(encoding="utf-8")
        staff_demo = (REPO_ROOT / "apps/staff-web/app/owned-platform-demo-data.ts").read_text(encoding="utf-8")
        processor = (REPO_ROOT / "apps/hermes-processor/processor.py").read_text(encoding="utf-8")
        information_lifespan = (REPO_ROOT / "app/src/information_lifespan.rs").read_text(encoding="utf-8")
        processor_schema = (REPO_ROOT / "schemas/information-lifespan-hermes-processor-output.schema.json").read_text(encoding="utf-8")
        architecture_glossary = (REPO_ROOT / "docs/glossary-architecture-terms.md").read_text(encoding="utf-8")
        outcome_policy = (REPO_ROOT / "docs/safety/evidence-policy-blocked-actions-outcomes.md").read_text(encoding="utf-8")
        overlay_readme = (REPO_ROOT / "docs/safety/entity-action-overlays/README.md").read_text(encoding="utf-8")
        worker_runtime = (REPO_ROOT / "apps/worker/src/runtime.rs").read_text(encoding="utf-8")
        training = (REPO_ROOT / "domain/src/training/mod.rs").read_text(encoding="utf-8")
        checkout_doc = (REPO_ROOT / "docs/workflows/operator/checkout-completion.md").read_text(encoding="utf-8")
        money_overlay = (REPO_ROOT / "docs/safety/entity-action-overlays/money-payment-provider-tools-source-data.md").read_text(encoding="utf-8")
        pet_health_overlay = (REPO_ROOT / "docs/safety/entity-action-overlays/pet-health-documents-vaccines-incidents.md").read_text(encoding="utf-8")
        retention_overlay = (REPO_ROOT / "docs/safety/entity-action-overlays/customer-communication-daily-updates-retention.md").read_text(encoding="utf-8")
        staff_queue_read_model = (REPO_ROOT / "apps/spacetimedb/src/read_model/staff_queue_item.rs").read_text(encoding="utf-8")
        outcome_policy = (REPO_ROOT / "docs/safety/evidence-policy-blocked-actions-outcomes.md").read_text(encoding="utf-8")
        for forbidden in (
            "outcome/labor evidence after review",
            "completed, deferred, suppressed by manager, and source fact was wrong",
        ):
            self.assertNotIn(forbidden, manager_doc)
        self.assertNotIn("stored source refs preserve proof after review", provenance_doc)
        self.assertNotIn("record a reviewed disposition, actual minutes", overlay_template)
        self.assertNotIn("OutcomeRecord(actual minutes, reviewed resolution status", provenance_doc)
        self.assertNotIn("reported time difference after review", outcome_roles)
        self.assertNotIn("matches_reviewed_packet", crm_retention)
        self.assertNotIn("ReviewedOutcomeClassification", crm_retention)
        for surface in (staff_demo, processor, information_lifespan, processor_schema):
            self.assertNotIn("reviewed packet", surface.lower())
        self.assertNotIn("reported time evidence after review", architecture_glossary)
        self.assertNotIn("actual minutes after review", outcome_policy)
        self.assertNotIn("before/actual minutes", outcome_policy)
        self.assertNotIn("disposition, actual minutes", overlay_template)
        self.assertNotIn("disposition, actual minutes", overlay_readme)
        self.assertNotIn('"actual minutes"', provenance_doc)
        data_quality_storage = storage.split("pub enum DataQualityResolutionStatusCode", 1)[0].rsplit("#[derive(", 1)[-1]
        self.assertNotIn("after review", data_quality_storage)
        for forbidden in ("reviewed outcome", "has_reviewed_outcome", "reviewed handoff candidate"):
            self.assertNotIn(forbidden, worker_runtime.lower())
        for forbidden in ("ActualLaborMinutes", "actual_minutes", "after review", "reviewer when measuring"):
            self.assertNotIn(forbidden, training)
        checkout_source = (REPO_ROOT / "app/src/checkout_completion.rs").read_text(encoding="utf-8")
        manager_brief_source = (REPO_ROOT / "app/src/manager_daily_brief.rs").read_text(encoding="utf-8")
        manager_labor_loop = (REPO_ROOT / "docs/design/manager-daily-brief-measurable-labor-loop.md").read_text(encoding="utf-8")
        for forbidden in ("ReviewedDisposition", "reviewed_disposition", "Routes the item to returned to customer", "Routes the item to needs staff follow up"):
            self.assertNotIn(forbidden, checkout_source)
        self.assertNotIn("from reviewed source facts", manager_brief_source)
        for forbidden in ("turns reviewed source facts into", "captures staff feedback", "measures before/after labor minutes"):
            self.assertNotIn(forbidden, manager_labor_loop)
        for forbidden in ("Reviewed disposition", "reviewed disposition/labor", "every serialized packet remains manager-reviewed"):
            self.assertNotIn(forbidden, checkout_doc)
        self.assertNotIn("record issue disposition/wrong-source findings after review", money_overlay)
        self.assertNotIn("records what happened after a draft, review packet, or manager action", outcome_policy)
        self.assertNotIn("turns a one-time review into reusable evidence", outcome_policy)
        self.assertNotIn("turns staff/manager review into durable evidence", outcome_policy)
        self.assertNotIn("mechanism that saves labor", outcome_policy)
        self.assertNotIn("what value was captured after review", pet_health_overlay)
        self.assertNotIn("Reviewed disposition still produces", retention_overlay)
        self.assertNotIn("Reviewed outcome label for display", staff_queue_read_model)
        for forbidden in ("record reviewed disposition", "audit reviewed disposition", "Reviewed dispositions become"):
            self.assertNotIn(forbidden, staff_demo)

        site_finance_impl = storage.split("impl SiteFinanceLocalPersistenceRecords", 1)[1].split("impl DataQualityHygieneLocalPersistenceRecords", 1)[0]
        self.assertNotIn("from_reviewed_outcome", site_finance_impl)
        self.assertIn("from_reported_outcome", site_finance_impl)
        self.assertNotIn("site_finance.reviewed_recommendation_recorded", storage)
        self.assertNotIn("data_quality_hygiene.reviewed_outcome_recorded", storage)
        self.assertNotIn('"reviewed_recommendation_recorded"', storage)

        data_quality_impl = storage.split("impl DataQualityHygieneLocalPersistenceRecords", 1)[1]
        self.assertNotIn("from_reviewed_outcome", data_quality_impl)
        self.assertIn("from_reported_outcome", data_quality_impl)


if __name__ == "__main__":
    unittest.main()
