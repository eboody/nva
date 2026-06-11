use pet_resort_worker::runtime;

#[test]
fn default_worker_runtime_is_fake_and_side_effect_safe() {
    let config = runtime::Config::from_env_defaults();

    assert_eq!(
        config.agent_runtime_mode(),
        runtime::AgentRuntimeMode::FakeDeterministic
    );
    assert_eq!(config.side_effect_mode(), runtime::SideEffectMode::Stubbed);
}
