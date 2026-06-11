#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRuntimeMode {
    FakeDeterministic,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffectMode {
    Stubbed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    agent_runtime_mode: AgentRuntimeMode,
    side_effect_mode: SideEffectMode,
}

impl Config {
    pub fn from_env_defaults() -> Self {
        let agent_runtime_mode = match std::env::var("PET_RESORT_AGENT_RUNTIME_MODE")
            .unwrap_or_else(|_| "fake".to_owned())
            .as_str()
        {
            "disabled" => AgentRuntimeMode::Disabled,
            _ => AgentRuntimeMode::FakeDeterministic,
        };

        Self {
            agent_runtime_mode,
            side_effect_mode: SideEffectMode::Stubbed,
        }
    }

    pub fn agent_runtime_mode(&self) -> AgentRuntimeMode {
        self.agent_runtime_mode
    }

    pub fn side_effect_mode(&self) -> SideEffectMode {
        self.side_effect_mode
    }
}
