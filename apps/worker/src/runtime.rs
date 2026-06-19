#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Classifies agent runtime modes used by the worker shell.
///
/// The mode chooses whether workflow packets are answered by deterministic fixtures or
/// skipped entirely; neither variant grants authority to perform live customer messaging
/// or provider writes.
pub enum AgentRuntimeMode {
    /// Uses deterministic fixtures so local workers can exercise packet flow without calling live agents.
    FakeDeterministic,
    /// Skips agent execution while keeping the worker process and side-effect stubs available.
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Classifies side-effect posture for the worker runtime.
///
/// Current workers expose only stubbed side effects so tests and demos cannot write to
/// Gingr, payment providers, SMS/email systems, or customer-facing channels.
pub enum SideEffectMode {
    /// Keeps provider writes, customer sends, and payment movement behind no-op test doubles.
    Stubbed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Configuration kept on the worker runtime.
///
/// The config is intentionally small: it selects deterministic agent execution and an
/// explicit side-effect posture so durable workflow workers can be inspected without
/// confusing storage evidence with permission to act in live systems.
pub struct Config {
    agent_runtime_mode: AgentRuntimeMode,
    side_effect_mode: SideEffectMode,
}

impl Config {
    /// Reads safe local defaults from the environment without enabling live side effects.
    ///
    /// `PET_RESORT_AGENT_RUNTIME_MODE=disabled` turns agent execution off; every other
    /// value falls back to deterministic fixtures. Side effects remain [`SideEffectMode::Stubbed`].
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

    /// Returns the agent runtime mode kept on this worker runtime value.
    pub fn agent_runtime_mode(&self) -> AgentRuntimeMode {
        self.agent_runtime_mode
    }

    /// Returns the side effect mode kept on this worker runtime value.
    pub fn side_effect_mode(&self) -> SideEffectMode {
        self.side_effect_mode
    }
}
