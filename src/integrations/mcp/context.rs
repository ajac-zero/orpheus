use std::{collections::HashMap, path::PathBuf};

use bon::bon;
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParam, CallToolResult},
    service::{QuitReason, RunningService},
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

use crate::{
    Error, Result,
    models::chat::{Message, Part, Tool},
};

pub struct Mcp {
    pub service: RunningService<RoleClient, ()>,
}

#[bon]
impl Mcp {
    #[builder(finish_fn = run)]
    pub async fn stdio(
        #[builder(into)] command: String,
        #[builder(with = |keys: impl IntoIterator<Item: Into<String>>| keys.into_iter().map(Into::into).collect())]
        args: Vec<String>,
        cwd: Option<PathBuf>,
        env: Option<HashMap<String, String>>,
    ) -> Result<Self> {
        let cmd = Command::new(&command).configure(|cmd| {
            cmd.args(&args);
            if let Some(cwd) = cwd {
                cmd.current_dir(cwd);
            }
            if let Some(env) = env {
                cmd.envs(env);
            }
        });
        let process = TokioChildProcess::new(cmd)?;
        let service = ().serve(process).await?;
        Ok(Self { service })
    }

    #[builder(on(String,into), finish_fn = send)]
    pub async fn call(
        &self,
        #[builder(start_fn)] name: String,
        #[builder(with = |value: impl serde::Serialize| -> Result<_> {
            serde_json::to_value(value).map_err(Error::Serde)
        })]
        arguments: Option<serde_json::Value>,
    ) -> Result<ToolResult> {
        let mut request = CallToolRequestParam {
            name: name.into(),
            arguments: None,
        };

        if let Some(args) = arguments {
            if let serde_json::Value::Object(map) = args {
                request.arguments = Some(map);
            } else {
                return Err(Error::Parsing("Expected a JSON object".into()));
            }
        }

        let result = self.service.call_tool(request).await?;

        Ok(ToolResult(result))
    }
}

pub struct ToolResult(CallToolResult);

impl ToolResult {
    pub fn into_message(self, tool_id: impl Into<String>) -> Message {
        let parts: Vec<Part> = self.0.content.into_iter().map(Into::into).collect();
        Message::tool(tool_id, parts)
    }
}

use mcp_call_builder::{IsUnset, SetArguments, State};

impl<'a, S: State> McpCallBuilder<'a, S>
where
    S::Arguments: IsUnset,
{
    pub fn literal_arguments(self, value: &str) -> Result<McpCallBuilder<'a, SetArguments<S>>> {
        let args: serde_json::Value = serde_json::from_str(value)?;
        self.arguments(args)
    }
}

impl Mcp {
    pub async fn get_tools(&self) -> Result<Vec<Tool>> {
        Ok(self
            .service
            .list_tools(Default::default())
            .await?
            .tools
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_>>()?)
    }

    pub async fn close(self) -> Result<QuitReason> {
        Ok(self.service.cancel().await?)
    }
}

impl From<rmcp::model::Annotated<rmcp::model::RawContent>> for Part {
    fn from(value: rmcp::model::Content) -> Self {
        let content = value.raw;
        match content {
            rmcp::model::RawContent::Text(raw) => Part::text(raw.text),
            _ => todo!(),
        }
    }
}
