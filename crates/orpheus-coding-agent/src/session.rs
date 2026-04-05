use std::{fs, path::{Path, PathBuf}};

use chrono::Utc;
use orpheus::models::ResponseExt;
use orpheus_agent::AgentRun;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub cwd: PathBuf,
    pub created_at: String,
    pub updated_at: String,
    pub model: String,
    pub last_response_id: Option<String>,
    #[serde(default)]
    pub entries: Vec<SessionEntry>,
}

impl SessionRecord {
    pub fn new(cwd: PathBuf, model: impl Into<String>) -> Self {
        let now = timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            cwd,
            created_at: now.clone(),
            updated_at: now,
            model: model.into(),
            last_response_id: None,
            entries: Vec::new(),
        }
    }

    pub fn push_user_message(&mut self, text: impl Into<String>) {
        self.entries.push(SessionEntry::User {
            timestamp: timestamp(),
            text: text.into(),
        });
        self.updated_at = timestamp();
    }

    pub fn push_agent_run(&mut self, run: &AgentRun) {
        for turn in &run.turns {
            for call in &turn.tool_calls {
                self.entries.push(SessionEntry::ToolCall {
                    timestamp: timestamp(),
                    turn: call.turn,
                    call_id: call.call_id.clone(),
                    name: call.name.clone(),
                    arguments: call.arguments.clone(),
                });
            }

            for output in &turn.tool_outputs {
                self.entries.push(SessionEntry::ToolOutput {
                    timestamp: timestamp(),
                    call_id: output.call_id.clone(),
                    name: output.name.clone(),
                    output: output.output.clone(),
                });
            }
        }

        if let Some(text) = run.response.output_text() {
            self.entries.push(SessionEntry::Assistant {
                timestamp: timestamp(),
                response_id: run.response.id.clone(),
                text,
            });
        }

        self.last_response_id = Some(run.response.id.clone());
        self.updated_at = timestamp();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEntry {
    User {
        timestamp: String,
        text: String,
    },
    Assistant {
        timestamp: String,
        response_id: String,
        text: String,
    },
    ToolCall {
        timestamp: String,
        turn: usize,
        call_id: String,
        name: String,
        arguments: String,
    },
    ToolOutput {
        timestamp: String,
        call_id: String,
        name: String,
        output: String,
    },
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub model: String,
    pub cwd: PathBuf,
    pub updated_at: String,
    pub last_response_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    root: PathBuf,
}

impl SessionManager {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn create_session(&self, cwd: PathBuf, model: impl Into<String>) -> Result<SessionRecord> {
        let session = SessionRecord::new(cwd, model);
        self.save(&session)?;
        Ok(session)
    }

    pub fn load(&self, id: &str) -> Result<SessionRecord> {
        let path = self.session_path(id);
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self, session: &SessionRecord) -> Result<()> {
        let dir = self.root.join(&session.id);
        fs::create_dir_all(&dir)?;
        let path = dir.join("session.json");
        let content = serde_json::to_string_pretty(session)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<SessionSummary>> {
        let mut sessions = Vec::new();

        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let session_path = entry.path().join("session.json");
            if !session_path.exists() {
                continue;
            }

            let content = fs::read_to_string(session_path)?;
            let session: SessionRecord = serde_json::from_str(&content)?;
            sessions.push(SessionSummary {
                id: session.id,
                model: session.model,
                cwd: session.cwd,
                updated_at: session.updated_at,
                last_response_id: session.last_response_id,
            });
        }

        sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        Ok(sessions)
    }

    fn session_path(&self, id: &str) -> PathBuf {
        self.root.join(id).join("session.json")
    }
}

fn timestamp() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::{SessionEntry, SessionManager};

    #[test]
    fn creates_saves_loads_and_lists_sessions() {
        let temp = tempdir().expect("tempdir creates");
        let manager = SessionManager::open(temp.path()).expect("manager opens");

        let mut session = manager
            .create_session(PathBuf::from("/tmp/project"), "openai/gpt-4o-mini")
            .expect("session creates");
        session.push_user_message("hello");
        manager.save(&session).expect("session saves");

        let loaded = manager.load(&session.id).expect("session loads");
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.model, "openai/gpt-4o-mini");
        assert!(matches!(loaded.entries.first(), Some(SessionEntry::User { text, .. }) if text == "hello"));

        let listed = manager.list().expect("sessions list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, session.id);
        assert_eq!(listed[0].model, "openai/gpt-4o-mini");
    }
}
