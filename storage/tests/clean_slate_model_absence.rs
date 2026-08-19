use std::fs;
use std::path::Path;

#[test]
fn storage_has_no_caller_constructible_review_disposition_or_legacy_brief_schema() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let approval = fs::read_to_string(manifest.join("src/operations/approval_outbox.rs"))
        .expect("approval outbox source must be readable");
    let site_finance = fs::read_to_string(manifest.join("src/operations/site_finance.rs"))
        .expect("site finance source must be readable");
    let brief = fs::read_to_string(manifest.join("src/operations/manager_daily_brief.rs"))
        .expect("manager brief source must be readable");

    for forbidden in [
        "ApprovalDecisionEvidence",
        "ApprovalReviewDisposition",
        "ReportedApprovalReviewDisposition",
        "ReportedReviewEvidence",
        "ApprovalTargetBinding",
        ".disposition(",
    ] {
        assert!(
            !approval.contains(forbidden),
            "approval storage must not retain caller-constructible authority-shaped surface `{forbidden}`"
        );
    }
    assert!(!site_finance.contains("approved_by_manager"));
    assert!(!site_finance.contains("rejected_by_manager"));
    assert!(!site_finance.contains("SiteFinanceManagerApproval"));
    assert!(!brief.contains("SchemaVersion"));
    assert!(!brief.contains("Legacy"));
    assert!(!brief.contains("v0"));
}
