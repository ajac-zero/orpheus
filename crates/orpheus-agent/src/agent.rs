use std::{collections::HashMap, fmt, future::poll_fn, pin::Pin, sync::Arc, thread};

use futures_core::Stream as FuturesStream;
use orpheus::{
    client::{AsyncOrpheus, Orpheus},
    models::{AsyncResponseRequestBuilder, Input, ResponseExt, ResponseRequestBuilder},
    responses::ResponseResource,
};

use crate::{
    AfterToolCallContext, AgentError, AgentEvent, AgentTool, AgentToolCall, AgentToolOutput,
    BeforeToolCall, BeforeToolCallContext, BoxError, Result,
};

type EventHook = dyn Fn(&AgentEvent) + Send + Sync;
type BeforeToolHook = dyn Fn(&mut BeforeToolCallContext) -> core::result::Result<BeforeToolCall, BoxError>
    + Send
    + Sync;
type AfterToolHook =
    dyn Fn(&mut AfterToolCallContext) -> core::result::Result<(), BoxError> + Send + Sync;

#[derive(Clone)]
struct AgentConfig {
    tools: Vec<AgentTool>,
    model: Option<String>,
    instructions: Option<String>,
    max_turns: usize,
    tool_execution: ToolExecution,
    on_event: Option<Arc<EventHook>>,
    before_tool_call: Option<Arc<BeforeToolHook>>,
    after_tool_call: Option<Arc<AfterToolHook>>,
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
            before_tool_call: None,
            after_tool_call: None,
        }
    }
}

enum PreparedToolCall {
    Blocked(AgentToolOutput),
    Ready {
        tool: AgentTool,
        tool_call: AgentToolCall,
    },
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

    /// Register a hook that can inspect, mutate, or block a tool call before execution.
    pub fn before_tool_call(
        mut self,
        hook: impl Fn(&mut BeforeToolCallContext) -> core::result::Result<BeforeToolCall, BoxError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.config.before_tool_call = Some(Arc::new(hook));
        self
    }

    /// Register a hook that can inspect or rewrite a tool output after execution.
    pub fn after_tool_call(
        mut self,
        hook: impl Fn(&mut AfterToolCallContext) -> core::result::Result<(), BoxError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.config.after_tool_call = Some(Arc::new(hook));
        self
    }

    /// Run the agent loop until the model stops requesting tools or max turns is reached.
    pub fn run(&self, input: impl Into<Input>) -> Result<AgentRun> {
        self.run_with_previous_response_id(input, None)
    }

    /// Run the agent loop starting from an existing response id.
    pub fn run_with_previous_response_id(
        &self,
        input: impl Into<Input>,
        previous_response_id: Option<&str>,
    ) -> Result<AgentRun> {
        let tool_registry = build_tool_registry(&self.config)?;
        let mut input = input.into();
        let mut previous_response_id = previous_response_id.map(ToOwned::to_owned);
        let mut turns = Vec::new();

        for turn in 1..=self.config.max_turns {
            emit(&self.config, AgentEvent::TurnStarted { turn });
            let response = self.send_turn(&input, previous_response_id.as_deref())?;
            let tool_calls = collect_tool_calls(turn, &response);

            let tool_outputs = if tool_calls.is_empty() {
                Vec::new()
            } else {
                execute_tool_calls(&self.config, &tool_registry, &response.id, &tool_calls)?
            };

            let is_complete = tool_calls.is_empty();
            let response_id = response.id.clone();

            emit(
                &self.config,
                AgentEvent::TurnCompleted {
                    turn,
                    response_id: response_id.clone(),
                    function_calls: tool_calls.len(),
                },
            );

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

            input = tool_outputs_to_input(tool_outputs);
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

    /// Run the agent loop while forwarding low-level response stream events through `AgentEvent`.
    pub fn run_streaming(&self, input: impl Into<Input>) -> Result<AgentRun> {
        self.run_streaming_with_previous_response_id(input, None)
    }

    /// Run the streaming agent loop starting from an existing response id.
    pub fn run_streaming_with_previous_response_id(
        &self,
        input: impl Into<Input>,
        previous_response_id: Option<&str>,
    ) -> Result<AgentRun> {
        let tool_registry = build_tool_registry(&self.config)?;
        let mut input = input.into();
        let mut previous_response_id = previous_response_id.map(ToOwned::to_owned);
        let mut turns = Vec::new();

        for turn in 1..=self.config.max_turns {
            emit(&self.config, AgentEvent::TurnStarted { turn });
            let response =
                self.send_turn_streaming(turn, &input, previous_response_id.as_deref())?;
            let tool_calls = collect_tool_calls(turn, &response);

            let tool_outputs = if tool_calls.is_empty() {
                Vec::new()
            } else {
                execute_tool_calls(&self.config, &tool_registry, &response.id, &tool_calls)?
            };

            let is_complete = tool_calls.is_empty();
            let response_id = response.id.clone();

            emit(
                &self.config,
                AgentEvent::TurnCompleted {
                    turn,
                    response_id: response_id.clone(),
                    function_calls: tool_calls.len(),
                },
            );

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

            input = tool_outputs_to_input(tool_outputs);
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

    fn send_turn_streaming(
        &self,
        turn: usize,
        input: &Input,
        previous_response_id: Option<&str>,
    ) -> Result<ResponseResource> {
        let request = configure_request(
            self.client.respond(input),
            &self.config,
            previous_response_id,
        );
        let mut stream = request.stream().map_err(AgentError::from)?;

        while let Some(event) = Iterator::next(&mut stream) {
            let event = event.map_err(AgentError::from)?;
            emit(&self.config, AgentEvent::Response { turn, event });
        }

        stream.final_result().map_err(AgentError::from)
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

    /// Register a hook that can inspect, mutate, or block a tool call before execution.
    pub fn before_tool_call(
        mut self,
        hook: impl Fn(&mut BeforeToolCallContext) -> core::result::Result<BeforeToolCall, BoxError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.config.before_tool_call = Some(Arc::new(hook));
        self
    }

    /// Register a hook that can inspect or rewrite a tool output after execution.
    pub fn after_tool_call(
        mut self,
        hook: impl Fn(&mut AfterToolCallContext) -> core::result::Result<(), BoxError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.config.after_tool_call = Some(Arc::new(hook));
        self
    }

    /// Run the agent loop until the model stops requesting tools or max turns is reached.
    pub async fn run(&self, input: impl Into<Input>) -> Result<AgentRun> {
        self.run_with_previous_response_id(input, None).await
    }

    /// Run the agent loop starting from an existing response id.
    pub async fn run_with_previous_response_id(
        &self,
        input: impl Into<Input>,
        previous_response_id: Option<&str>,
    ) -> Result<AgentRun> {
        let tool_registry = build_tool_registry(&self.config)?;
        let mut input = input.into();
        let mut previous_response_id = previous_response_id.map(ToOwned::to_owned);
        let mut turns = Vec::new();

        for turn in 1..=self.config.max_turns {
            emit(&self.config, AgentEvent::TurnStarted { turn });
            let response = self
                .send_turn(&input, previous_response_id.as_deref())
                .await?;
            let tool_calls = collect_tool_calls(turn, &response);

            let tool_outputs = if tool_calls.is_empty() {
                Vec::new()
            } else {
                execute_tool_calls(&self.config, &tool_registry, &response.id, &tool_calls)?
            };

            let is_complete = tool_calls.is_empty();
            let response_id = response.id.clone();

            emit(
                &self.config,
                AgentEvent::TurnCompleted {
                    turn,
                    response_id: response_id.clone(),
                    function_calls: tool_calls.len(),
                },
            );

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

            input = tool_outputs_to_input(tool_outputs);
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

    /// Run the agent loop while forwarding low-level response stream events through `AgentEvent`.
    pub async fn run_streaming(&self, input: impl Into<Input>) -> Result<AgentRun> {
        self.run_streaming_with_previous_response_id(input, None).await
    }

    /// Run the streaming agent loop starting from an existing response id.
    pub async fn run_streaming_with_previous_response_id(
        &self,
        input: impl Into<Input>,
        previous_response_id: Option<&str>,
    ) -> Result<AgentRun> {
        let tool_registry = build_tool_registry(&self.config)?;
        let mut input = input.into();
        let mut previous_response_id = previous_response_id.map(ToOwned::to_owned);
        let mut turns = Vec::new();

        for turn in 1..=self.config.max_turns {
            emit(&self.config, AgentEvent::TurnStarted { turn });
            let response = self
                .send_turn_streaming(turn, &input, previous_response_id.as_deref())
                .await?;
            let tool_calls = collect_tool_calls(turn, &response);

            let tool_outputs = if tool_calls.is_empty() {
                Vec::new()
            } else {
                execute_tool_calls(&self.config, &tool_registry, &response.id, &tool_calls)?
            };

            let is_complete = tool_calls.is_empty();
            let response_id = response.id.clone();

            emit(
                &self.config,
                AgentEvent::TurnCompleted {
                    turn,
                    response_id: response_id.clone(),
                    function_calls: tool_calls.len(),
                },
            );

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

            input = tool_outputs_to_input(tool_outputs);
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

    async fn send_turn_streaming(
        &self,
        turn: usize,
        input: &Input,
        previous_response_id: Option<&str>,
    ) -> Result<ResponseResource> {
        let request = configure_async_request(
            self.client.respond(input),
            &self.config,
            previous_response_id,
        );
        let mut stream = request.stream().await.map_err(AgentError::from)?;

        while let Some(event) = poll_fn(|cx| Pin::new(&mut stream).poll_next(cx)).await {
            let event = event.map_err(AgentError::from)?;
            emit(&self.config, AgentEvent::Response { turn, event });
        }

        stream.final_result().await.map_err(AgentError::from)
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
    response_id: &str,
    tool_calls: &[AgentToolCall],
) -> Result<Vec<AgentToolOutput>> {
    match config.tool_execution {
        ToolExecution::Sequential => {
            execute_tool_calls_sequential(config, tool_registry, response_id, tool_calls)
        }
        ToolExecution::Parallel => {
            execute_tool_calls_parallel(config, tool_registry, response_id, tool_calls)
        }
    }
}

fn execute_tool_calls_sequential(
    config: &AgentConfig,
    tool_registry: &HashMap<String, AgentTool>,
    response_id: &str,
    tool_calls: &[AgentToolCall],
) -> Result<Vec<AgentToolOutput>> {
    let mut outputs = Vec::with_capacity(tool_calls.len());

    for tool_call in tool_calls {
        match prepare_tool_call(config, tool_registry, response_id, tool_call.clone())? {
            PreparedToolCall::Blocked(output) => outputs.push(output),
            PreparedToolCall::Ready { tool, tool_call } => match tool.call(tool_call.clone()) {
                Ok(output) => {
                    outputs.push(finalize_tool_output(
                        config,
                        response_id,
                        &tool_call,
                        output,
                    )?);
                }
                Err(error) => {
                    emit_tool_failed(config, &tool_call, &error);
                    return Err(error);
                }
            },
        }
    }

    Ok(outputs)
}

fn execute_tool_calls_parallel(
    config: &AgentConfig,
    tool_registry: &HashMap<String, AgentTool>,
    response_id: &str,
    tool_calls: &[AgentToolCall],
) -> Result<Vec<AgentToolOutput>> {
    let mut prepared_calls = Vec::with_capacity(tool_calls.len());

    for tool_call in tool_calls {
        prepared_calls.push(prepare_tool_call(
            config,
            tool_registry,
            response_id,
            tool_call.clone(),
        )?);
    }

    thread::scope(|scope| {
        let mut outputs = vec![None; prepared_calls.len()];
        let mut handles = Vec::new();

        for (index, prepared_call) in prepared_calls.into_iter().enumerate() {
            match prepared_call {
                PreparedToolCall::Blocked(output) => outputs[index] = Some(output),
                PreparedToolCall::Ready { tool, tool_call } => {
                    let join_call = tool_call.clone();
                    let handle = scope.spawn(move || tool.call(tool_call));
                    handles.push((index, join_call, handle));
                }
            }
        }

        for (index, tool_call, handle) in handles {
            match handle.join() {
                Ok(Ok(output)) => {
                    outputs[index] = Some(finalize_tool_output(
                        config,
                        response_id,
                        &tool_call,
                        output,
                    )?);
                }
                Ok(Err(error)) => {
                    emit_tool_failed(config, &tool_call, &error);
                    return Err(error);
                }
                Err(_) => {
                    let error = AgentError::ToolPanicked {
                        name: tool_call.name.clone(),
                        call_id: tool_call.call_id.clone(),
                    };
                    emit_tool_failed(config, &tool_call, &error);
                    return Err(error);
                }
            }
        }

        Ok(outputs
            .into_iter()
            .map(|output| output.expect("every prepared tool call resolves to an output"))
            .collect())
    })
}

fn prepare_tool_call(
    config: &AgentConfig,
    tool_registry: &HashMap<String, AgentTool>,
    response_id: &str,
    tool_call: AgentToolCall,
) -> Result<PreparedToolCall> {
    let tool =
        tool_registry
            .get(&tool_call.name)
            .cloned()
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

    let mut tool_call = tool_call;

    if let Some(before_tool_call) = &config.before_tool_call {
        let mut context = BeforeToolCallContext {
            turn: tool_call.turn,
            response_id: response_id.to_owned(),
            tool_call,
        };

        match before_tool_call(&mut context).map_err(|source| AgentError::BeforeToolHookFailed {
            name: context.tool_call.name.clone(),
            call_id: context.tool_call.call_id.clone(),
            source,
        })? {
            BeforeToolCall::Continue => {
                tool_call = context.tool_call;
            }
            BeforeToolCall::Block { output } => {
                emit(
                    config,
                    AgentEvent::ToolBlocked {
                        turn: context.tool_call.turn,
                        call_id: context.tool_call.call_id.clone(),
                        name: context.tool_call.name.clone(),
                        output: output.clone(),
                    },
                );

                return Ok(PreparedToolCall::Blocked(AgentToolOutput {
                    call_id: context.tool_call.call_id,
                    name: context.tool_call.name,
                    output,
                }));
            }
        }
    }

    Ok(PreparedToolCall::Ready { tool, tool_call })
}

fn finalize_tool_output(
    config: &AgentConfig,
    response_id: &str,
    tool_call: &AgentToolCall,
    output: AgentToolOutput,
) -> Result<AgentToolOutput> {
    let output = if let Some(after_tool_call) = &config.after_tool_call {
        let mut context = AfterToolCallContext {
            turn: tool_call.turn,
            response_id: response_id.to_owned(),
            tool_call: tool_call.clone(),
            output,
        };

        after_tool_call(&mut context).map_err(|source| AgentError::AfterToolHookFailed {
            name: context.tool_call.name.clone(),
            call_id: context.tool_call.call_id.clone(),
            source,
        })?;

        context.output
    } else {
        output
    };

    emit(
        config,
        AgentEvent::ToolFinished {
            turn: tool_call.turn,
            call_id: output.call_id.clone(),
            name: output.name.clone(),
            output: output.output.clone(),
        },
    );

    Ok(output)
}

fn emit_tool_failed(config: &AgentConfig, tool_call: &AgentToolCall, error: &AgentError) {
    emit(
        config,
        AgentEvent::ToolFailed {
            turn: tool_call.turn,
            call_id: tool_call.call_id.clone(),
            name: tool_call.name.clone(),
            error: error.to_string(),
        },
    );
}

fn tool_outputs_to_input(tool_outputs: Vec<AgentToolOutput>) -> Input {
    let mut input = Input(Vec::new());

    for output in tool_outputs {
        input.push_function_output(output.call_id, output.output);
    }

    input
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
    use std::sync::Arc;

    use crate::{
        Agent, AgentError, AgentTool, AgentToolCall, AsyncAgent, BeforeToolCall, BoxError,
    };

    use super::{AgentConfig, build_tool_registry, execute_tool_calls_sequential};

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

    #[test]
    fn before_tool_call_can_mutate_arguments() {
        let config = AgentConfig {
            before_tool_call: Some(Arc::new(|context| {
                context.tool_call.arguments = String::from(r#"{"value":"rewritten"}"#);
                Ok(BeforeToolCall::Continue)
            })),
            ..Default::default()
        };
        let schema = orpheus::models::Tool::function("echo").empty();
        let tool = AgentTool::new(schema, |call| Ok(call.arguments));
        let registry = build_tool_registry(&AgentConfig {
            tools: vec![tool],
            before_tool_call: config.before_tool_call.clone(),
            ..Default::default()
        })
        .expect("tool registry builds");

        let outputs = execute_tool_calls_sequential(
            &AgentConfig {
                tools: Vec::new(),
                before_tool_call: config.before_tool_call,
                ..Default::default()
            },
            &registry,
            "resp_1",
            &[AgentToolCall {
                turn: 1,
                call_id: String::from("call_1"),
                name: String::from("echo"),
                arguments: String::from(r#"{"value":"original"}"#),
            }],
        )
        .expect("tool call executes");

        assert_eq!(outputs[0].output, r#"{"value":"rewritten"}"#);
    }

    #[test]
    fn before_tool_call_can_block_execution_and_after_hook_can_rewrite_output() {
        let schema = orpheus::models::Tool::function("echo").empty();
        let tool = AgentTool::new(schema.clone(), |_| Ok(String::from("original")));
        let registry = build_tool_registry(&AgentConfig {
            tools: vec![tool.clone()],
            before_tool_call: Some(Arc::new(|_| {
                Ok::<_, BoxError>(BeforeToolCall::Block {
                    output: String::from("blocked"),
                })
            })),
            ..Default::default()
        })
        .expect("tool registry builds");

        let blocked_outputs = execute_tool_calls_sequential(
            &AgentConfig {
                before_tool_call: Some(Arc::new(|_| {
                    Ok::<_, BoxError>(BeforeToolCall::Block {
                        output: String::from("blocked"),
                    })
                })),
                ..Default::default()
            },
            &registry,
            "resp_1",
            &[AgentToolCall {
                turn: 1,
                call_id: String::from("call_1"),
                name: String::from("echo"),
                arguments: String::from("{}"),
            }],
        )
        .expect("blocked tool call resolves to output");

        assert_eq!(blocked_outputs[0].output, "blocked");

        let registry = build_tool_registry(&AgentConfig {
            tools: vec![tool],
            after_tool_call: Some(Arc::new(|context| {
                context.output.output.push_str("-rewritten");
                Ok::<_, BoxError>(())
            })),
            ..Default::default()
        })
        .expect("tool registry builds");

        let rewritten_outputs = execute_tool_calls_sequential(
            &AgentConfig {
                after_tool_call: Some(Arc::new(|context| {
                    context.output.output.push_str("-rewritten");
                    Ok::<_, BoxError>(())
                })),
                ..Default::default()
            },
            &registry,
            "resp_1",
            &[AgentToolCall {
                turn: 1,
                call_id: String::from("call_2"),
                name: String::from("echo"),
                arguments: String::from("{}"),
            }],
        )
        .expect("tool call executes");

        assert_eq!(rewritten_outputs[0].output, "original-rewritten");
    }
}
