use std::{error::Error as StdError, fmt};

/// Boxed error type returned from runtime tool handlers.
pub type BoxError = Box<dyn StdError + Send + Sync>;

/// Result type used throughout `orpheus-agent`.
pub type Result<T, E = AgentError> = core::result::Result<T, E>;

/// Errors returned by the agent orchestration layer.
#[derive(Debug)]
pub enum AgentError {
    Client(orpheus::Error),
    DuplicateToolName {
        name: String,
    },
    BeforeToolHookFailed {
        name: String,
        call_id: String,
        source: BoxError,
    },
    AfterToolHookFailed {
        name: String,
        call_id: String,
        source: BoxError,
    },
    MissingTool {
        name: String,
    },
    ToolExecutionFailed {
        name: String,
        call_id: String,
        source: BoxError,
    },
    ToolPanicked {
        name: String,
        call_id: String,
    },
    MaxTurnsReached {
        max_turns: usize,
        last_response_id: String,
    },
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::Client(err) => write!(f, "client error: {err}"),
            AgentError::DuplicateToolName { name } => {
                write!(f, "duplicate agent tool registered: {name}")
            }
            AgentError::BeforeToolHookFailed {
                name,
                call_id,
                source,
            } => write!(
                f,
                "before_tool_call hook failed for `{name}` and call `{call_id}`: {source}"
            ),
            AgentError::AfterToolHookFailed {
                name,
                call_id,
                source,
            } => write!(
                f,
                "after_tool_call hook failed for `{name}` and call `{call_id}`: {source}"
            ),
            AgentError::MissingTool { name } => {
                write!(f, "model requested an unregistered tool: {name}")
            }
            AgentError::ToolExecutionFailed {
                name,
                call_id,
                source,
            } => write!(f, "tool `{name}` failed for call `{call_id}`: {source}"),
            AgentError::ToolPanicked { name, call_id } => {
                write!(f, "tool `{name}` panicked for call `{call_id}`")
            }
            AgentError::MaxTurnsReached {
                max_turns,
                last_response_id,
            } => write!(
                f,
                "agent reached max turns ({max_turns}) before finishing; last response id: {last_response_id}"
            ),
        }
    }
}

impl StdError for AgentError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AgentError::Client(err) => Some(err),
            AgentError::BeforeToolHookFailed { source, .. }
            | AgentError::AfterToolHookFailed { source, .. }
            | AgentError::ToolExecutionFailed { source, .. } => Some(source.as_ref()),
            AgentError::DuplicateToolName { .. }
            | AgentError::MissingTool { .. }
            | AgentError::ToolPanicked { .. }
            | AgentError::MaxTurnsReached { .. } => None,
        }
    }
}

impl From<orpheus::Error> for AgentError {
    fn from(value: orpheus::Error) -> Self {
        AgentError::Client(value)
    }
}
