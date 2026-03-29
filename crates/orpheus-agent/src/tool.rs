use std::{fmt, sync::Arc};

use crate::{AgentError, BoxError, Result};

type ToolHandler = dyn Fn(AgentToolCall) -> core::result::Result<String, BoxError> + Send + Sync;

/// Runtime representation of an executable tool.
#[derive(Clone)]
pub struct AgentTool {
    schema: orpheus::models::Tool,
    handler: Arc<ToolHandler>,
}

impl AgentTool {
    /// Create a new executable tool from a model-visible schema and a runtime handler.
    pub fn new(
        schema: orpheus::models::Tool,
        handler: impl Fn(AgentToolCall) -> core::result::Result<String, BoxError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            schema,
            handler: Arc::new(handler),
        }
    }

    /// Returns the schema sent to the model.
    pub fn schema(&self) -> &orpheus::models::Tool {
        &self.schema
    }

    /// Returns the tool name from the underlying schema.
    pub fn name(&self) -> &str {
        match &self.schema {
            orpheus::models::Tool::Function { name, .. } => name,
        }
    }

    pub(crate) fn call(&self, call: AgentToolCall) -> Result<AgentToolOutput> {
        let name = call.name.clone();
        let call_id = call.call_id.clone();
        let output = (self.handler)(call).map_err(|source| AgentError::ToolExecutionFailed {
            name: name.clone(),
            call_id: call_id.clone(),
            source,
        })?;

        Ok(AgentToolOutput {
            name,
            call_id,
            output,
        })
    }
}

impl fmt::Debug for AgentTool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentTool")
            .field("name", &self.name())
            .field("schema", &self.schema)
            .finish_non_exhaustive()
    }
}

/// Function call information passed to runtime tool handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentToolCall {
    pub turn: usize,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

impl AgentToolCall {
    pub(crate) fn from_function_call(turn: usize, call: &orpheus::responses::FunctionCall) -> Self {
        Self {
            turn,
            call_id: call.call_id.clone(),
            name: call.name.clone(),
            arguments: call.arguments.clone(),
        }
    }
}

/// Tool output that gets fed back into the next model turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentToolOutput {
    pub call_id: String,
    pub name: String,
    pub output: String,
}
