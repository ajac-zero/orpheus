#![deny(clippy::mod_module_files, clippy::unwrap_used)]

mod agent;
mod error;
mod event;
mod tool;

pub use agent::{Agent, AgentRun, AgentTurn, AsyncAgent, ToolExecution};
pub use error::{AgentError, BoxError, Result};
pub use event::AgentEvent;
pub use orpheus;
pub use tool::{AgentTool, AgentToolCall, AgentToolOutput};

/// Single import with the most common agent-layer types.
pub mod prelude {
    pub use crate::{
        Agent, AgentError, AgentEvent, AgentRun, AgentTool, AgentToolCall, AgentToolOutput,
        AsyncAgent, ToolExecution,
    };
    pub use orpheus::prelude::*;
}
