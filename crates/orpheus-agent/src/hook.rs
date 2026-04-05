use crate::{AgentToolCall, AgentToolOutput};

/// Mutable context passed to `before_tool_call` hooks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeforeToolCallContext {
    pub turn: usize,
    pub response_id: String,
    pub tool_call: AgentToolCall,
}

/// Decision returned from a `before_tool_call` hook.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BeforeToolCall {
    #[default]
    Continue,
    Block {
        output: String,
    },
}

/// Mutable context passed to `after_tool_call` hooks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AfterToolCallContext {
    pub turn: usize,
    pub response_id: String,
    pub tool_call: AgentToolCall,
    pub output: AgentToolOutput,
}
