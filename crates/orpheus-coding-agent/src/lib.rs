#![deny(clippy::mod_module_files, clippy::unwrap_used)]

pub mod cli;
pub mod config;
pub mod runtime;
pub mod session;
pub mod tools;

pub use config::{LoadedSettings, OutputMode, Settings, ToolExecutionMode};
pub use runtime::{CodingAgentRuntime, RunOptions, RunResult};
pub use session::{SessionEntry, SessionManager, SessionRecord, SessionSummary};

pub const DEFAULT_MODEL: &str = "openai/gpt-4o-mini";
pub const DEFAULT_SYSTEM_PROMPT: &str = "You are Orpheus, a coding assistant operating in a local workspace. Use the available tools to inspect files, edit code, and run shell commands when needed. Keep responses concise and action-oriented. Read files before making edits when context is missing. When using the edit tool, make exact replacements against the current file contents.";

pub type Result<T, E = anyhow::Error> = core::result::Result<T, E>;
