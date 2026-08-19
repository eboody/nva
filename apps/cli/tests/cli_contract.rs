use std::process::Command;

#[test]
fn tools_command_emits_the_provider_neutral_portal_capability() {
    let output = Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("tools")
        .output()
        .expect("CLI tools command runs");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("CLI output is UTF-8 JSON");
    assert!(stdout.contains("ProviderPortal"));
    assert!(!stdout.contains("Gingr"));
}
