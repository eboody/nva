use app::checkout_completion;

fn main() {
    let _ = checkout_completion::CompletionStatus::ReportedStaffCheckout;
    let _ = checkout_completion::StaffHandoff::builder();
}
