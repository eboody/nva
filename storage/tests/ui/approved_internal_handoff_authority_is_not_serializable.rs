use storage::operations::ApprovedInternalHandoffAuthority;

fn serialize_authority(authority: &ApprovedInternalHandoffAuthority) {
    let _ = serde_json::to_string(authority);
}

fn main() {}