#[cfg(feature = "otel")]
use crate::prelude::Role;
#[cfg(feature = "otel")]
use tracing::info;

use super::ChatRequest;
use crate::{
    Error, Result,
    constants::CHAT_COMPLETION_PATH,
    models::common::{
        handler::{AsyncExecutor, Executor, Handler},
        mode::{Async, Mode, Sync},
    },
};

#[derive(Debug)]
pub struct ChatHandler<M: Mode>(M);

impl<M: Mode> Handler<M> for ChatHandler<M> {
    const PATH: &str = CHAT_COMPLETION_PATH;
    type Input = ChatRequest<M>;
    type Response = M::Response;

    fn new(builder: M::Builder) -> Self {
        Self(M::new(builder))
    }
}

impl Executor for ChatHandler<Sync> {
    fn execute(self, body: Self::Input) -> Result<Self::Response> {
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

        let response = self.0.0.json(&body).send().map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}

impl AsyncExecutor for ChatHandler<Async> {
    async fn execute(self, body: Self::Input) -> Result<Self::Response> {
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

        let response = self.0.0.json(&body).send().await.map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().await.map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}
