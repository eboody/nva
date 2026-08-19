use domain::{access, agent, analytics, consent, customer, lead, operations, source};

fn file(path: &str) -> &'static str {
    match path {
        "domain/src/access.rs" => include_str!("../src/access.rs"),
        "domain/src/agent.rs" => include_str!("../src/agent.rs"),
        "domain/src/analytics.rs" => include_str!("../src/analytics.rs"),
        "domain/src/consent.rs" => include_str!("../src/consent.rs"),
        "domain/src/customer.rs" => include_str!("../src/customer.rs"),
        "domain/src/identity.rs" => include_str!("../src/identity.rs"),
        "domain/src/lead.rs" => include_str!("../src/lead.rs"),
        "domain/src/lib.rs" => include_str!("../src/lib.rs"),
        "domain/src/operations.rs" => include_str!("../src/operations.rs"),
        _ => unreachable!("unexpected source file in ownership contract test"),
    }
}

#[test]
fn strategic_ai_ops_concepts_have_canonical_semantic_owners() {
    let _: Option<lead::response::EventId> = None;
    let _: Option<customer::intelligence::NoteId> = None;
    let _: Option<operations::labor::Minutes> = None;
    let _: Option<operations::capacity::Quantity> = None;
    let _: Option<agent::assistant::AnswerText> = None;
    let _: Option<analytics::finance::InsightKind> = None;
    let _: Option<analytics::outcome::Workstream> = None;
    let _: source::System = source::System::Crm;
    let _: Option<access::ActorRole> = None;
    let _: Option<consent::Purpose> = None;
}

#[test]
fn canonical_semantic_owner_modules_do_not_hide_direct_bridge_reexports() {
    for path in [
        "domain/src/access.rs",
        "domain/src/agent.rs",
        "domain/src/analytics.rs",
        "domain/src/consent.rs",
        "domain/src/customer.rs",
        "domain/src/identity.rs",
        "domain/src/lead.rs",
        "domain/src/lib.rs",
        "domain/src/operations.rs",
    ] {
        let source = file(path);
        assert!(
            !source.contains("pub use crate::strategic_ai_ops_bridge"),
            "{path} is still a direct bridge re-export instead of owning canonical definitions"
        );
    }
}

#[test]
fn canonical_semantic_owner_modules_do_not_hide_renamed_bridge_reexports() {
    for path in [
        "domain/src/access.rs",
        "domain/src/agent.rs",
        "domain/src/analytics.rs",
        "domain/src/consent.rs",
        "domain/src/customer.rs",
        "domain/src/identity.rs",
        "domain/src/lead.rs",
        "domain/src/operations.rs",
    ] {
        let source = file(path);
        assert!(
            !source.contains("pub use crate::canonical_ai_ops_contracts"),
            "{path} still re-exports renamed bridge definitions instead of owning canonical definitions"
        );
    }
}

#[test]
fn historical_strategic_facade_is_not_an_active_domain_surface() {
    let root = file("domain/src/lib.rs");

    assert!(!root.contains("pub mod strategic_ai_ops"));
    assert!(!root.contains("strategic_ai_ops_bridge.rs"));
}
