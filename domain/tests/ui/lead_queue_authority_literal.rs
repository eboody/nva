use domain::lead::response::{LegalContactAction, QueueableContact};

fn main() {
    let _forged = QueueableContact {
        action: LegalContactAction::QueueOnly,
    };
}
