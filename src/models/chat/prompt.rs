use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::message;

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct ChatPrompt<'a> {
    messages: &'a message::Messages,
    model: String,
    stream: Option<bool>,
    tools: Option<Value>,
    #[serde(flatten)]
    extra: Option<Value>,
}

impl<'a> ChatPrompt<'a> {
    pub fn new(
        model: String,
        messages: &'a message::Messages,
        stream: Option<bool>,
        tools: Option<&[u8]>,
        extra: Option<&[u8]>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            model,
            messages,
            stream,
            tools: tools.map(serde_json::from_slice::<Value>).transpose()?,
            extra: extra.map(serde_json::from_slice::<Value>).transpose()?,
        })
    }

    pub fn is_stream(&self) -> bool {
        self.stream.is_some_and(|x| x)
    }
}
