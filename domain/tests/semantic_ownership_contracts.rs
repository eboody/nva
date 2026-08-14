use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use domain::{agent, audit, entities};
use uuid::Uuid;

#[test]
fn audit_events_are_owned_by_the_audit_module_and_extension_labels_are_validated() {
    let action = audit::Action::Extension(
        audit::ActionLabel::try_new("  message.send.blocked_stub  ").unwrap(),
    );
    let metadata = BTreeMap::from([(
        audit::MetadataKey::try_new(" workflow ").unwrap(),
        audit::MetadataValue::try_new(" daily-update ").unwrap(),
    )]);

    let event = audit::Event {
        at: Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
        actor: entities::ActorRef::Agent {
            workflow: agent::Name::try_new("daily-update").unwrap(),
        },
        subject: audit::Subject::Message(entities::MessageId(Uuid::from_u128(1))),
        action,
        metadata,
    };

    assert!(matches!(event.action, audit::Action::Extension(_)));
    assert_eq!(event.metadata.len(), 1);
    assert!(audit::ActionLabel::try_new("   ").is_err());
    assert!(audit::MetadataKey::try_new("").is_err());
    assert!(audit::MetadataValue::try_new("   ").is_err());
}
