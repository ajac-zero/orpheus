use std::{collections::HashMap, fmt, sync::Arc, thread};

use orpheus::{
    client::{AsyncOrpheus, Orpheus},
    models::{AsyncResponseRequestBuilder, Input, ResponseExt, ResponseRequestBuilder},
    responses::ResponseResource,
};

use crate::{AgentError, AgentEvent, AgentTool, AgentToolCall, AgentToolOutput, Result};

type EventHook = dyn Fn(&AgentEvent) + Send + Sync;

#[derive(Clone)]
struct AgentConfig {
    tools: Vec<AgentTool>,
    model: Option<String>,
    instructions: Option<String>,
    max_turns: usize,
    tool_execution: ToolExecution,
    on_event: Option<Arc<EventHook>>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            tools: Vec::new(),
            model: None,
            instructions: None,
            max_turns: 8,
            tool_execution: ToolExecution::Sequential,
            on_event: None,
        }
    }
}

/// Controls how multiple tool calls are executed during a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolExecution {
    Sequential,
    Parallel,
}

/// Multi-turn tool orchestration built on top of the sync Orpheus client.
pub struct Agent<'a> {
    client: &'a Orpheus,
    config: AgentConfig,
}

/// Multi-turn tool orchestration built on top of the async Orpheus client.
pub struct AsyncAgent<'a> {
    client: &'a AsyncOrpheus,
    config: AgentConfig,
}

impl<'a> Agent<'a> {
    /// Create a new agent bound to an Orpheus client.
    pub fn new(client: &'a Orpheus) -> Self {
        Self {
            client,
            config: AgentConfig::default(),
        }
    }

    /// Set the model for every turn in the agent loop.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = Some(model.into());
        self
    }

    /// Set the instructions for every turn in the agent loop.
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.config.instructions = Some(instructions.into());
        self
    }

    /// Add a single executable tool.
    pub fn tool(mut self, tool: AgentTool) -> Self {
        self.config.tools.push(tool);
        self
    }

    /// Add multiple executable tools.
    pub fn tools(mut self, tools: impl IntoIterator<Item = AgentTool>) -> Self {
        self.config.tools.extend(tools);
        self
    }

    /// Set the maximum number of request turns.
    pub fn max_turns(mut self, max_turns: usize) -> Self {
        self.config.max_turns = max_turns.max(1);
        self
    }

    /// Select how tool calls are executed when the model requests more than one.
    pub fn tool_execution(mut self, tool_execution: ToolExecution) -> Self {
        self.config.tool_execution = tool_execution;
        self
    }

    /// Register a hook that receives high-level agent events.
    pub fn on_event(mut self, hook: impl Fn(&AgentEvent) + Send + Sync + 'static) -> Self {
        self.config.on_event = Some(Arc::new(hook));
        self
    }

    /// Run the agent loop until the model stops requesting tools or max turns is reached.
    pub fn run(&self, input: impl Into<Input>) -> Result<AgentRun> {
        let tool_registry = build_tool_registry(&self.config)?;
        let mut input = input.into();
        let mut previous_response_id = None;
        let mut turns = Vec::new();

        for turn in 1..=self.config.max_turns {
            let response = self.send_turn(&input, previous_response_id.as_deref())?;
            let tool_calls = collect_tool_calls(turn, &response);

            let tool_outputs = if tool_calls.is_empty() {
                Vec::new()
            } else {
                execute_tool_calls(&self.config, &tool_registry, &tool_calls)?
            };

            emit(
                &self.config,
                AgentEvent::TurnCompleted {
                    turn,
                    response_id: response.id.clone(),
                    function_calls: tool_calls.len(),
                },
            );

            let is_complete = tool_calls.is_empty();
            let response_id = response.id.clone();

            turns.push(AgentTurn {
                response: response.clone(),
                tool_calls,
                tool_outputs: tool_outputs.clone(),
            });

            if is_complete {
                emit(
                    &self.config,
                    AgentEvent::Completed {
                        turns: turn,
                        response_id,
                    },
                );

                return Ok(AgentRun { response, turns });
            }

            input = Input(Vec::new());
            for output in tool_outputs {
                input.push_function_output(output.call_id, output.output);
            }
            previous_response_id = Some(response.id);
        }

        let last_response_id = turns
            .last()
            .map(|turn| turn.response.id.clone())
            .unwrap_or_default();

        Err(AgentError::MaxTurnsReached {
            max_turns: self.config.max_turns,
            last_response_id,
        })
    }

    fn send_turn(
        &self,
        input: &Input,
        previous_response_id: Option<&str>,
    ) -> Result<ResponseResource> {
        let request = configure_request(
            self.client.respond(input),
            &self.config,
            previous_response_id,
        );
        request.send().map_err(AgentError::from)
    }
}

impl<'a> AsyncAgent<'a> {
    /// Create a new agent bound to an async Orpheus client.
    pub fn new(client: &'a AsyncOrpheus) -> Self {
        Self {
            client,
            config: AgentConfig::default(),
        }
    }

    /// Set the model for every turn in the agent loop.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = Some(model.into());
        self
    }

    /// Set the instructions for every turn in the agent loop.
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.config.instructions = Some(instructions.into());
        self
    }

    /// Add a single executable tool.
    pub fn tool(mut self, tool: AgentTool) -> Self {
        self.config.tools.push(tool);
        self
    }

    /// Add multiple executable tools.
    pub fn tools(mut self, tools: impl IntoIterator<Item = AgentTool>) -> Self {
        self.config.tools.extend(tools);
        self
    }

    /// Set the maximum number of request turns.
    pub fn max_turns(mut self, max_turns: usize) -> Self {
        self.config.max_turns = max_turns.max(1);
        self
    }

    /// Select how tool calls are executed when the model requests more than one.
    pub fn tool_execution(mut self, tool_execution: ToolExecution) -> Self {
        self.config.tool_execution = tool_execution;
        self
    }

    /// Register a hook that receives high-level agent events.
    pub fn on_event(mut self, hook: impl Fn(&AgentEvent) + Send + Sync + 'static) -> Self {
        self.config.on_event = Some(Arc::new(hook));
        self
    }

    /// Run the agent loop until the model stops requesting tools or max turns is reached.
    pub async fn run(&self, input: impl Into<Input>) -> Result<AgentRun> {
        let tool_registry = build_tool_registry(&self.config)?;
        let mut input = input.into();
        let mut previous_response_id = None;
        let mut turns = Vec::new();

        for turn in 1..=self.config.max_turns {
            let response = self
                .send_turn(&input, previous_response_id.as_deref())
                .await?;
            let tool_calls = collect_tool_calls(turn, &response);

            let tool_outputs = if tool_calls.is_empty() {
                Vec::new()
            } else {
                execute_tool_calls(&self.config, &tool_registry, &tool_calls)?
            };

            emit(
                &self.config,
                AgentEvent::TurnCompleted {
                    turn,
                    response_id: response.id.clone(),
                    function_calls: tool_calls.len(),
                },
            );

            let is_complete = tool_calls.is_empty();
            let response_id = response.id.clone();

            turns.push(AgentTurn {
                response: response.clone(),
                tool_calls,
                tool_outputs: tool_outputs.clone(),
            });

            if is_complete {
                emit(
                    &self.config,
                    AgentEvent::Completed {
                        turns: turn,
                        response_id,
                    },
                );

                return Ok(AgentRun { response, turns });
            }

            input = Input(Vec::new());
            for output in tool_outputs {
                input.push_function_output(output.call_id, output.output);
            }
            previous_response_id = Some(response.id);
        }

        let last_response_id = turns
            .last()
            .map(|turn| turn.response.id.clone())
            .unwrap_or_default();

        Err(AgentError::MaxTurnsReached {
            max_turns: self.config.max_turns,
            last_response_id,
        })
    }

    async fn send_turn(
        &self,
        input: &Input,
        previous_response_id: Option<&str>,
    ) -> Result<ResponseResource> {
        let request = configure_async_request(
            self.client.respond(input),
            &self.config,
            previous_response_id,
        );
        request.send().await.map_err(AgentError::from)
    }
}

fn build_tool_registry(config: &AgentConfig) -> Result<HashMap<String, AgentTool>> {
    let mut registry = HashMap::with_capacity(config.tools.len());

    for tool in &config.tools {
        let name = tool.name().to_owned();
        if registry.insert(name.clone(), tool.clone()).is_some() {
            return Err(AgentError::DuplicateToolName { name });
        }
    }

    Ok(registry)
}

fn collect_tool_calls(turn: usize, response: &ResponseResource) -> Vec<AgentToolCall> {
    response
        .function_calls()
        .into_iter()
        .map(|call| AgentToolCall::from_function_call(turn, call))
        .collect()
}

fn configure_request<'a>(
    mut request: ResponseRequestBuilder<'a>,
    config: &AgentConfig,
    previous_response_id: Option<&str>,
) -> ResponseRequestBuilder<'a> {
    request = request.parallel_tool_calls(matches!(config.tool_execution, ToolExecution::Parallel));

    if !config.tools.is_empty() {
        request = request.tools(config.tools.iter().map(|tool| tool.schema().clone()));
    }

    if let Some(model) = &config.model {
        request = request.model(model.clone());
    }

    if let Some(instructions) = &config.instructions {
        request = request.instructions(instructions.clone());
    }

    if let Some(previous_response_id) = previous_response_id {
        request = request.previous_response_id(previous_response_id);
    }

    request
}

fn configure_async_request<'a>(
    mut request: AsyncResponseRequestBuilder<'a>,
    config: &AgentConfig,
    previous_response_id: Option<&str>,
) -> AsyncResponseRequestBuilder<'a> {
    request = request.parallel_tool_calls(matches!(config.tool_execution, ToolExecution::Parallel));

    if !config.tools.is_empty() {
        request = request.tools(config.tools.iter().map(|tool| tool.schema().clone()));
    }

    if let Some(model) = &config.model {
        request = request.model(model.clone());
    }

    if let Some(instructions) = &config.instructions {
        request = request.instructions(instructions.clone());
    }

    if let Some(previous_response_id) = previous_response_id {
        request = request.previous_response_id(previous_response_id);
    }

    request
}

fn execute_tool_calls(
    config: &AgentConfig,
    tool_registry: &HashMap<String, AgentTool>,
    tool_calls: &[AgentToolCall],
) -> Result<Vec<AgentToolOutput>> {
    match config.tool_execution {
        ToolExecution::Sequential => {
            execute_tool_calls_sequential(config, tool_registry, tool_calls)
        }
        ToolExecution::Parallel => execute_tool_calls_parallel(config, tool_registry, tool_calls),
    }
}

fn execute_tool_calls_sequential(
    config: &AgentConfig,
    tool_registry: &HashMap<String, AgentTool>,
    tool_calls: &[AgentToolCall],
) -> Result<Vec<AgentToolOutput>> {
    let mut outputs = Vec::with_capacity(tool_calls.len());

    for tool_call in tool_calls {
        let tool = tool_registry
            .get(&tool_call.name)
            .ok_or_else(|| AgentError::MissingTool {
                name: tool_call.name.clone(),
            })?;

        emit(
            config,
            AgentEvent::ToolStarted {
                turn: tool_call.turn,
                call_id: tool_call.call_id.clone(),
                name: tool_call.name.clone(),
                arguments: tool_call.arguments.clone(),
            },
        );

        match tool.call(tool_call.clone()) {
            Ok(output) => {
                emit(
                    config,
                    AgentEvent::ToolFinished {
                        turn: tool_call.turn,
                        call_id: output.call_id.clone(),
                        name: output.name.clone(),
                        output: output.output.clone(),
                    },
                );
                outputs.push(output);
            }
            Err(error) => {
                emit(
                    config,
                    AgentEvent::ToolFailed {
                        turn: tool_call.turn,
                        call_id: tool_call.call_id.clone(),
                        name: tool_call.name.clone(),
                        error: error.to_string(),
                    },
                );
                return Err(error);
            }
        }
    }

    Ok(outputs)
}

fn execute_tool_calls_parallel(
    config: &AgentConfig,
    tool_registry: &HashMap<String, AgentTool>,
    tool_calls: &[AgentToolCall],
) -> Result<Vec<AgentToolOutput>> {
    let mut scheduled = Vec::with_capacity(tool_calls.len());

    for tool_call in tool_calls {
        let tool =
            tool_registry
                .get(&tool_call.name)
                .cloned()
                .ok_or_else(|| AgentError::MissingTool {
                    name: tool_call.name.clone(),
                })?;
        scheduled.push((tool_call.clone(), tool));
    }

    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(scheduled.len());

        for (tool_call, tool) in scheduled {
            let on_event = config.on_event.clone();
            let join_call = tool_call.clone();

            let handle = scope.spawn(move || {
                emit_with(
                    &on_event,
                    AgentEvent::ToolStarted {
                        turn: tool_call.turn,
                        call_id: tool_call.call_id.clone(),
                        name: tool_call.name.clone(),
                        arguments: tool_call.arguments.clone(),
                    },
                );

                let result = tool.call(tool_call.clone());

                match &result {
                    Ok(output) => emit_with(
                        &on_event,
                        AgentEvent::ToolFinished {
                            turn: tool_call.turn,
                            call_id: output.call_id.clone(),
                            name: output.name.clone(),
                            output: output.output.clone(),
                        },
                    ),
                    Err(error) => emit_with(
                        &on_event,
                        AgentEvent::ToolFailed {
                            turn: tool_call.turn,
                            call_id: tool_call.call_id.clone(),
                            name: tool_call.name.clone(),
                            error: error.to_string(),
                        },
                    ),
                }

                result
            });

            handles.push((join_call, handle));
        }

        let mut outputs = Vec::with_capacity(handles.len());

        for (tool_call, handle) in handles {
            match handle.join() {
                Ok(Ok(output)) => outputs.push(output),
                Ok(Err(error)) => return Err(error),
                Err(_) => {
                    let error = AgentError::ToolPanicked {
                        name: tool_call.name.clone(),
                        call_id: tool_call.call_id.clone(),
                    };
                    emit(
                        config,
                        AgentEvent::ToolFailed {
                            turn: tool_call.turn,
                            call_id: tool_call.call_id,
                            name: tool_call.name,
                            error: error.to_string(),
                        },
                    );
                    return Err(error);
                }
            }
        }

        Ok(outputs)
    })
}

fn emit(config: &AgentConfig, event: AgentEvent) {
    emit_with(&config.on_event, event);
}

impl fmt::Debug for Agent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Agent")
            .field("model", &self.config.model)
            .field("instructions", &self.config.instructions)
            .field("max_turns", &self.config.max_turns)
            .field("tool_execution", &self.config.tool_execution)
            .field(
                "tools",
                &self
                    .config
                    .tools
                    .iter()
                    .map(AgentTool::name)
                    .collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for AsyncAgent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncAgent")
            .field("model", &self.config.model)
            .field("instructions", &self.config.instructions)
            .field("max_turns", &self.config.max_turns)
            .field("tool_execution", &self.config.tool_execution)
            .field(
                "tools",
                &self
                    .config
                    .tools
                    .iter()
                    .map(AgentTool::name)
                    .collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

/// Full result of an agent loop.
#[derive(Debug, Clone)]
pub struct AgentRun {
    pub response: ResponseResource,
    pub turns: Vec<AgentTurn>,
}

/// A single response turn inside an agent run.
#[derive(Debug, Clone)]
pub struct AgentTurn {
    pub response: ResponseResource,
    pub tool_calls: Vec<AgentToolCall>,
    pub tool_outputs: Vec<AgentToolOutput>,
}

fn emit_with(on_event: &Option<Arc<EventHook>>, event: AgentEvent) {
    if let Some(on_event) = on_event {
        on_event(&event);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Agent, AgentError, AgentTool, AsyncAgent};

    #[test]
    fn rejects_duplicate_tool_names_before_sending_requests() {
        let client = orpheus::client::Orpheus::default();
        let schema = orpheus::models::Tool::function("echo").empty();

        let agent = Agent::new(&client)
            .tool(AgentTool::new(schema.clone(), |_| {
                Ok(String::from("first"))
            }))
            .tool(AgentTool::new(schema, |_| Ok(String::from("second"))));

        let error = match agent.run("hello") {
            Ok(_) => panic!("expected duplicate tool registration to fail"),
            Err(error) => error,
        };

        match error {
            AgentError::DuplicateToolName { name } => assert_eq!(name, "echo"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn async_agent_rejects_duplicate_tool_names_before_sending_requests() {
        let client = orpheus::client::AsyncOrpheus::default();
        let schema = orpheus::models::Tool::function("echo").empty();

        let agent = AsyncAgent::new(&client)
            .tool(AgentTool::new(schema.clone(), |_| {
                Ok(String::from("first"))
            }))
            .tool(AgentTool::new(schema, |_| Ok(String::from("second"))));

        let error = match agent.run("hello").await {
            Ok(_) => panic!("expected duplicate tool registration to fail"),
            Err(error) => error,
        };

        match error {
            AgentError::DuplicateToolName { name } => assert_eq!(name, "echo"),
            other => panic!("unexpected error: {other}"),
        }
    }
}
