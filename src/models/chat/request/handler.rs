use tracing::info;

use super::ChatRequest;
use crate::{
    Error, Result,
    constants::CHAT_COMPLETION_PATH,
    models::common::handler::{AsyncHandler, Handler},
    prelude::Role,
};

pub trait Mode {}

#[derive(Debug)]
pub struct Sync(reqwest::blocking::RequestBuilder);
impl Mode for Sync {}

#[derive(Debug)]
pub struct Async(reqwest::RequestBuilder);
impl Mode for Async {}

#[derive(Debug)]
pub struct ChatHandler<M: Mode> {
    builder: M,
}

impl Handler for ChatHandler<Sync> {
    const PATH: &str = CHAT_COMPLETION_PATH;
    type Input = ChatRequest<Sync>;

    fn new(builder: reqwest::blocking::RequestBuilder) -> Self {
        ChatHandler {
            builder: Sync(builder),
        }
    }

    fn execute(self, body: Self::Input) -> Result<reqwest::blocking::Response> {
        #[cfg(feature = "otel")]
        {
            let span = &body.span;
            let _guard = span.enter();

            span.record(
                "gen_ai.output.type",
                if body.response_format.is_some() {
                    "json"
                } else {
                    "text"
                },
            );
            span.record(
                "gen_ai.request.model",
                body.model.as_deref().unwrap_or("default"),
            );
            if let Some(seed) = body.seed {
                span.record("gen_ai.request.seed", seed);
            }
            if let Some(frequency_penalty) = body.frequency_penalty {
                span.record("gen_ai.request.frequency_penalty", frequency_penalty);
            }
            if let Some(max_tokens) = body.max_tokens {
                span.record("gen_ai.request.max_tokens", max_tokens);
            }
            if let Some(presence_penalty) = body.presence_penalty {
                span.record("gen_ai.request.presence_penalty", presence_penalty);
            }
            if let Some(temperature) = body.temperature {
                span.record("gen_ai.request.temperature", temperature);
            }
            if let Some(top_k) = body.top_k {
                span.record("gen_ai.request.top_k", top_k);
            }
            if let Some(top_p) = body.top_p {
                span.record("gen_ai.request.top_p", top_p);
            }

            for message in body.messages.iter() {
                let content = message.content.to_string();
                match message.role {
                    Role::System | Role::Developer => {
                        info!(name: "gen_ai.system.message", content)
                    }
                    Role::User => {
                        info!(name: "gen_ai.user.message", content)
                    }
                    Role::Assistant => {
                        info!(name: "gen_ai.assistant.message", content)
                    }
                    Role::Tool => {
                        info!(name: "gen_ai.tool.message", content)
                    }
                }
            }
        }

        let response = self.builder.0.json(&body).send().map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}

impl AsyncHandler for ChatHandler<Async> {
    const PATH: &str = CHAT_COMPLETION_PATH;
    type Input = ChatRequest<Async>;

    fn new(builder: reqwest::RequestBuilder) -> Self {
        ChatHandler {
            builder: Async(builder),
        }
    }

    async fn execute(self, body: Self::Input) -> Result<reqwest::Response> {
        #[cfg(feature = "otel")]
        {
            let span = &body.span;
            let _guard = span.enter();

            span.record(
                "gen_ai.output.type",
                if body.response_format.is_some() {
                    "json"
                } else {
                    "text"
                },
            );
            span.record(
                "gen_ai.request.model",
                body.model.as_deref().unwrap_or("default"),
            );
            if let Some(seed) = body.seed {
                span.record("gen_ai.request.seed", seed);
            }
            if let Some(frequency_penalty) = body.frequency_penalty {
                span.record("gen_ai.request.frequency_penalty", frequency_penalty);
            }
            if let Some(max_tokens) = body.max_tokens {
                span.record("gen_ai.request.max_tokens", max_tokens);
            }
            if let Some(presence_penalty) = body.presence_penalty {
                span.record("gen_ai.request.presence_penalty", presence_penalty);
            }
            if let Some(temperature) = body.temperature {
                span.record("gen_ai.request.temperature", temperature);
            }
            if let Some(top_k) = body.top_k {
                span.record("gen_ai.request.top_k", top_k);
            }
            if let Some(top_p) = body.top_p {
                span.record("gen_ai.request.top_p", top_p);
            }

            for message in body.messages.iter() {
                let content = message.content.to_string();
                match message.role {
                    Role::System | Role::Developer => {
                        info!(name: "gen_ai.system.message", content)
                    }
                    Role::User => {
                        info!(name: "gen_ai.user.message", content)
                    }
                    Role::Assistant => {
                        info!(name: "gen_ai.assistant.message", content)
                    }
                    Role::Tool => {
                        info!(name: "gen_ai.tool.message", content)
                    }
                }
            }
        }

        let response = self
            .builder
            .0
            .json(&body)
            .send()
            .await
            .map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().await.map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}
