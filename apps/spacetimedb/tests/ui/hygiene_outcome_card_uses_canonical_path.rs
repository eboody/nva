use nva_spacetimedb::{
    read_model::HygieneOutcomeCardRow,
    storage::review_queue::{ActorRefColumn, FeedbackOutcomeColumn},
};

fn main() {
    let row = HygieneOutcomeCardRow::new(
        "reported-action".to_owned(),
        ActorRefColumn::System,
        FeedbackOutcomeColumn::Completed,
        4,
        Vec::new(),
        Vec::new(),
    );

    assert_eq!(row.reported_actual_minutes_spent, 4);
    assert!(!row.live_delivery_allowed);
}
