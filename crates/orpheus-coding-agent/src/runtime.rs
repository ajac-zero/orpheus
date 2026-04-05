use std::{
    io::{self, Write},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::bail;
use orpheus::models::ResponseExt;
use orpheus_agent::{Agent, AgentEvent};
use serde::Serialize;

use crate::{
    Result,
    config::{LoadedSettings, OutputMode, Settings},
    session::{SessionManager, SessionRecord},
    tools::BuiltinToolset,
};

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub max_turns: Option<usize>,
    pub stream: bool,
    pub use_tools: bool,
    pub output_mode: Option<OutputMode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunResult {
    pub session: SessionRecord,
    pub response_id: String,
    pub output_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CodingAgentRuntime {
    cwd: PathBuf,
    loaded_settings: LoadedSettings,
    session_manager: SessionManager,
}

impl CodingAgentRuntime {
    pub fn from_working_dir(cwd: impl Into<PathBuf>) -> Result<Self> {
        let cwd = cwd.into();
        let loaded_settings = LoadedSettings::load(&cwd)?;
        let session_manager = SessionManager::open(loaded_settings.settings.resolved_session_dir())?;

        Ok(Self {
            cwd,
            loaded_settings,
            session_manager,
        })
    }

    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    pub fn settings(&self) -> &Settings {
        &self.loaded_settings.settings
    }

    pub fn loaded_settings(&self) -> &LoadedSettings {
        &self.loaded_settings
    }

    pub fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }

    pub fn available_models(&self) -> Vec<String> {
        let mut models = vec![
            self.settings().resolved_model().to_string(),
            String::from("openai/gpt-4o-mini"),
            String::from("openai/gpt-4.1-mini"),
            String::from("anthropic/claude-3.5-sonnet"),
        ];
        models.sort();
        models.dedup();
        models
    }

    pub fn available_tools(&self) -> &'static [&'static str] {
        BuiltinToolset::names()
    }

    pub fn run_print(&self, prompt: &str, mut options: RunOptions) -> Result<RunResult> {
        let output_mode = self.resolve_output_mode(&options);
        let should_stream_text = options.stream && matches!(output_mode, OutputMode::Text);
        options.stream = should_stream_text;

        let result = self.run_prompt(prompt, options)?;
        match output_mode {
            OutputMode::Text => {
                if !should_stream_text {
                    if let Some(text) = &result.output_text {
                        println!("{text}");
                    }
                }
            }
            OutputMode::Json => {
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        Ok(result)
    }

    pub fn run_interactive(&self, session_id: Option<String>, mut options: RunOptions) -> Result<()> {
        if matches!(self.resolve_output_mode(&options), OutputMode::Json) {
            bail!("interactive mode does not support json output")
        }

        options.stream = true;
        options.session_id = session_id;

        let session = self.prepare_session(options.session_id.as_deref(), options.model.as_deref())?;
        println!("Session: {}", session.id);
        println!("Model: {}", session.model);
        println!("Working directory: {}", session.cwd.display());
        println!("Type /exit to quit.\n");

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut line = String::new();
            let bytes_read = io::stdin().read_line(&mut line)?;
            if bytes_read == 0 {
                println!();
                return Ok(());
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if matches!(line, "/exit" | "/quit") {
                return Ok(());
            }
            if line == "/help" {
                println!("Commands: /help, /exit, /quit");
                continue;
            }

            let result = self.run_prompt(line, RunOptions {
                session_id: Some(session.id.clone()),
                model: options.model.clone(),
                system_prompt: options.system_prompt.clone(),
                max_turns: options.max_turns,
                stream: true,
                use_tools: options.use_tools,
                output_mode: None,
            })?;

            if result.output_text.is_none() {
                println!("[no assistant text returned]");
            }
        }
    }

    pub fn run_prompt(&self, prompt: &str, options: RunOptions) -> Result<RunResult> {
        let client = orpheus::client::Orpheus::from_env()?;
        let mut session = self.prepare_session(options.session_id.as_deref(), options.model.as_deref())?;
        let model = options.model.clone().unwrap_or_else(|| session.model.clone());
        session.model = model.clone();
        session.push_user_message(prompt);
        self.session_manager.save(&session)?;

        let system_prompt = options
            .system_prompt
            .unwrap_or_else(|| self.settings().resolved_system_prompt().to_string());
        let max_turns = options.max_turns.unwrap_or_else(|| self.settings().resolved_max_turns());
        let tool_execution: orpheus_agent::ToolExecution = self.settings().tool_execution.clone().into();
        let disabled_tools = &self.settings().disabled_tools;
        let tools = if options.use_tools {
            BuiltinToolset::new(session.cwd.clone(), disabled_tools).into_tools()
        } else {
            Vec::new()
        };

        let streamed_text = Arc::new(AtomicBool::new(false));
        let mut agent = Agent::new(&client)
            .model(model.clone())
            .instructions(system_prompt)
            .max_turns(max_turns)
            .tool_execution(tool_execution);

        if options.stream {
            let printed_text = Arc::clone(&streamed_text);
            agent = agent.on_event(move |event| match event {
                AgentEvent::Response { event, .. } => {
                    if let Some(delta) = event.as_text_delta() {
                        printed_text.store(true, Ordering::Relaxed);
                        print!("{delta}");
                        let _ = io::stdout().flush();
                    }
                }
                AgentEvent::ToolStarted { name, .. } => {
                    let _ = writeln!(io::stderr(), "\n[tool start] {name}");
                }
                AgentEvent::ToolFinished { name, .. } => {
                    let _ = writeln!(io::stderr(), "[tool done] {name}");
                }
                AgentEvent::ToolFailed { name, error, .. } => {
                    let _ = writeln!(io::stderr(), "[tool failed] {name}: {error}");
                }
                _ => {}
            });
        }

        for tool in tools {
            agent = agent.tool(tool);
        }

        let run = if options.stream {
            agent.run_streaming_with_previous_response_id(
                prompt,
                session.last_response_id.as_deref(),
            )?
        } else {
            agent.run_with_previous_response_id(prompt, session.last_response_id.as_deref())?
        };

        if options.stream && streamed_text.load(Ordering::Relaxed) {
            println!();
        }

        session.push_agent_run(&run);
        self.session_manager.save(&session)?;

        Ok(RunResult {
            response_id: run.response.id.clone(),
            output_text: run.response.output_text(),
            session,
        })
    }

    fn prepare_session(&self, session_id: Option<&str>, model_override: Option<&str>) -> Result<SessionRecord> {
        match session_id {
            Some(session_id) => {
                let mut session = self.session_manager.load(session_id)?;
                if let Some(model) = model_override {
                    session.model = model.to_string();
                }
                Ok(session)
            }
            None => self
                .session_manager
                .create_session(
                    self.cwd.clone(),
                    model_override.unwrap_or(self.settings().resolved_model()),
                ),
        }
    }

    fn resolve_output_mode(&self, options: &RunOptions) -> OutputMode {
        options
            .output_mode
            .clone()
            .unwrap_or_else(|| self.settings().output_mode.clone())
    }
}
