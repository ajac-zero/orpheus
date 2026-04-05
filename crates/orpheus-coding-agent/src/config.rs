use std::{env, fs, path::{Path, PathBuf}};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{DEFAULT_MODEL, DEFAULT_SYSTEM_PROMPT, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolExecutionMode {
    Sequential,
    Parallel,
}

impl Default for ToolExecutionMode {
    fn default() -> Self {
        Self::Sequential
    }
}

impl From<ToolExecutionMode> for orpheus_agent::ToolExecution {
    fn from(value: ToolExecutionMode) -> Self {
        match value {
            ToolExecutionMode::Sequential => orpheus_agent::ToolExecution::Sequential,
            ToolExecutionMode::Parallel => orpheus_agent::ToolExecution::Parallel,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Text,
    Json,
}

impl Default for OutputMode {
    fn default() -> Self {
        Self::Text
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub model: Option<String>,
    pub max_turns: Option<usize>,
    pub session_dir: Option<PathBuf>,
    #[serde(default)]
    pub tool_execution: ToolExecutionMode,
    #[serde(default)]
    pub output_mode: OutputMode,
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub disabled_tools: Vec<String>,
}

impl Settings {
    pub fn with_defaults() -> Self {
        Self {
            model: Some(DEFAULT_MODEL.to_string()),
            max_turns: Some(8),
            session_dir: None,
            tool_execution: ToolExecutionMode::Sequential,
            output_mode: OutputMode::Text,
            system_prompt: Some(DEFAULT_SYSTEM_PROMPT.to_string()),
            disabled_tools: Vec::new(),
        }
    }

    pub fn merge(&mut self, other: Self) {
        if other.model.is_some() {
            self.model = other.model;
        }
        if other.max_turns.is_some() {
            self.max_turns = other.max_turns;
        }
        if other.session_dir.is_some() {
            self.session_dir = other.session_dir;
        }
        self.tool_execution = other.tool_execution;
        self.output_mode = other.output_mode;
        if other.system_prompt.is_some() {
            self.system_prompt = other.system_prompt;
        }
        if !other.disabled_tools.is_empty() {
            self.disabled_tools = other.disabled_tools;
        }
    }

    pub fn resolved_model(&self) -> &str {
        self.model.as_deref().unwrap_or(DEFAULT_MODEL)
    }

    pub fn resolved_max_turns(&self) -> usize {
        self.max_turns.unwrap_or(8).max(1)
    }

    pub fn resolved_system_prompt(&self) -> &str {
        self.system_prompt
            .as_deref()
            .unwrap_or(DEFAULT_SYSTEM_PROMPT)
    }

    pub fn resolved_session_dir(&self) -> PathBuf {
        if let Some(path) = &self.session_dir {
            return path.clone();
        }

        if let Some(project_dirs) = ProjectDirs::from("com", "ajac-zero", "orpheus") {
            return project_dirs.data_local_dir().join("sessions");
        }

        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".orpheus")
            .join("sessions")
    }
}

#[derive(Debug, Clone)]
pub struct LoadedSettings {
    pub settings: Settings,
    pub global_path: PathBuf,
    pub project_path: PathBuf,
}

impl LoadedSettings {
    pub fn load(cwd: &Path) -> Result<Self> {
        let global_path = global_settings_path();
        let project_path = cwd.join(".orpheus").join("settings.toml");

        let mut settings = Settings::with_defaults();

        if global_path.exists() {
            settings.merge(read_settings_file(&global_path)?);
        }

        if project_path.exists() {
            settings.merge(read_settings_file(&project_path)?);
        }

        Ok(Self {
            settings,
            global_path,
            project_path,
        })
    }
}

pub fn global_settings_path() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("com", "ajac-zero", "orpheus") {
        return project_dirs.config_dir().join("settings.toml");
    }

    PathBuf::from(".orpheus/settings.toml")
}

fn read_settings_file(path: &Path) -> Result<Settings> {
    let content = fs::read_to_string(path)?;
    let settings = toml::from_str(&content)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::{OutputMode, Settings, ToolExecutionMode};

    #[test]
    fn merge_prefers_populated_fields_from_other_settings() {
        let mut settings = Settings::with_defaults();
        let update = Settings {
            model: Some(String::from("openai/gpt-4.1-mini")),
            max_turns: Some(4),
            session_dir: None,
            tool_execution: ToolExecutionMode::Parallel,
            output_mode: OutputMode::Json,
            system_prompt: Some(String::from("custom")),
            disabled_tools: vec![String::from("bash")],
        };

        settings.merge(update);

        assert_eq!(settings.model.as_deref(), Some("openai/gpt-4.1-mini"));
        assert_eq!(settings.max_turns, Some(4));
        assert_eq!(settings.tool_execution, ToolExecutionMode::Parallel);
        assert_eq!(settings.output_mode, OutputMode::Json);
        assert_eq!(settings.system_prompt.as_deref(), Some("custom"));
        assert_eq!(settings.disabled_tools, vec![String::from("bash")]);
    }
}
