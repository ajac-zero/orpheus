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
    Message, Part, Tools,
    error::{McpError, RuntimeError},
};

pub struct ModelContext {
    pub service: RunningService<RoleClient, ()>,
}

#[bon]
impl ModelContext {
    #[builder(finish_fn = run)]
    pub async fn new(
        #[builder(into)] command: String,
        #[builder(with = |keys: impl IntoIterator<Item: Into<String>>| keys.into_iter().map(Into::into).collect())]
        args: Vec<String>,
        cwd: Option<PathBuf>,
        env: Option<HashMap<String, String>>,
    ) -> crate::Result<Self> {
        let cmd = Command::new(&command).configure(|cmd| {
            cmd.args(&args);
            if let Some(cwd) = cwd {
                cmd.current_dir(cwd);
            }
            if let Some(env) = env {
                cmd.envs(env);
            }
        });
        let process = TokioChildProcess::new(cmd).map_err(RuntimeError::Io)?;
        let service = ().serve(process).await.map_err(|e| McpError::Init(e.to_string()))?;
        Ok(Self { service })
    }

    #[builder(on(String,into), finish_fn = send)]
    pub async fn call(
        &self,
        #[builder(start_fn)] name: String,
        #[builder(with = |value: impl serde::Serialize| -> crate::Result<_> {
            serde_json::to_value(value).map_err(|e| RuntimeError::Serde(e).into())
        })]
        arguments: Option<serde_json::Value>,
    ) -> crate::Result<ToolResult> {
        let mut request = CallToolRequestParam {
            name: name.into(),
            arguments: None,
        };

        if let Some(args) = arguments {
            if let serde_json::Value::Object(map) = args {
                request.arguments = Some(map);
            } else {
                return Err(McpError::ToolSchema("Expected a JSON object".to_string()).into());
            }
        }

        let result = self
            .service
            .call_tool(request)
            .await
            .map_err(McpError::Service)?;

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

use model_context_call_builder::{IsUnset, SetArguments, State};

impl<'a, S: State> ModelContextCallBuilder<'a, S>
where
    S::Arguments: IsUnset,
{
    pub fn literal_arguments(
        self,
        value: &str,
    ) -> crate::Result<ModelContextCallBuilder<'a, SetArguments<S>>> {
        let args: serde_json::Value = serde_json::from_str(value).map_err(RuntimeError::Serde)?;
        self.arguments(args)
    }
}

impl ModelContext {
    pub async fn get_tools(&self) -> crate::Result<Tools> {
        Ok(self
            .service
            .list_tools(Default::default())
            .await
            .map_err(McpError::Service)?
            .try_into()?)
    }

    pub async fn close(self) -> crate::Result<QuitReason> {
        Ok(self.service.cancel().await.map_err(McpError::Close)?)
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
