use std::{collections::HashMap, path::PathBuf};

use bon::bon;
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParam, CallToolResult},
    service::{QuitReason, RunningService},
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

use crate::{Error, Message, Part, Result, Tools};

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
        let process = TokioChildProcess::new(cmd).map_err(Error::io)?;
        let service = ().serve(process).await.map_err(|e| Error::init(e.to_string()))?;
        Ok(Self { service })
    }

    #[builder(on(String,into), finish_fn = send)]
    pub async fn call(
        &self,
        #[builder(start_fn)] name: String,
        #[builder(with = |value: impl serde::Serialize| -> Result<_> {
            serde_json::to_value(value).map_err(Error::serde)
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
                return Err(Error::tool_schema("Expected a JSON object"));
            }
        }

        let result = self
            .service
            .call_tool(request)
            .await
            .map_err(Error::service)?;

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
    ) -> Result<ModelContextCallBuilder<'a, SetArguments<S>>> {
        let args: serde_json::Value = serde_json::from_str(value).map_err(Error::serde)?;
        self.arguments(args)
    }
}

impl ModelContext {
    pub async fn get_tools(&self) -> Result<Tools> {
        Ok(self
            .service
            .list_tools(Default::default())
            .await
            .map_err(Error::service)?
            .try_into()?)
    }

    pub async fn close(self) -> Result<QuitReason> {
        Ok(self.service.cancel().await.map_err(Error::close)?)
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
