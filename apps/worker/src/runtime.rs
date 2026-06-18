#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Classifies agent runtime mode values that drive the worker runtime.
pub enum AgentRuntimeMode {
    /// Uses deterministic fixtures so local workers can exercise packet flow without calling live agents.
    FakeDeterministic,
    /// Skips agent execution while keeping the worker process and side-effect stubs available.
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Classifies side effect mode values that drive the worker runtime.
pub enum SideEffectMode {
    /// Keeps provider writes, customer sends, and payment movement behind no-op test doubles.
    Stubbed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Config carried by the worker runtime; it selects worker execution mode while keeping side effects explicit.
pub struct Config {
    agent_runtime_mode: AgentRuntimeMode,
    side_effect_mode: SideEffectMode,
}

impl Config {
    /// Reads safe local defaults from the environment without enabling live side effects.
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

    /// Returns the agent runtime mode carried by this worker runtime value.
    pub fn agent_runtime_mode(&self) -> AgentRuntimeMode {
        self.agent_runtime_mode
    }

    /// Returns the side effect mode carried by this worker runtime value.
    pub fn side_effect_mode(&self) -> SideEffectMode {
        self.side_effect_mode
    }
}
