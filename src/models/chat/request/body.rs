use std::collections::HashMap;

#[cfg(feature = "otel")]
use tracing::Span;
use tracing::debug;

use crate::{
    Error, Result,
    client::core::{Async, AsyncExecutor, Executor, Mode, Sync},
    models::{
        Format, Plugin, ProviderPreferences, ReasoningConfig, UsageConfig,
        chat::{AsyncStream, ChatCompletion, ChatHandler, ChatStream, Messages, Tools},
        common::{
            ProviderPreferencesBuilder, ReasoningConfigBuilder, provider_preferences_builder,
            reasoning_config_builder,
        },
    },
};

/// Core request structure for chat completion API calls.
///
/// This module contains the `ChatRequest` struct which represents all possible
/// parameters that can be sent to a chat completion API. It supports both
/// synchronous and asynchronous operations through the `Mode` generic parameter.
///
/// Key features include:
/// - **Structured Output**: Define JSON schemas for model responses
/// - **Tool Calling**: Enable function calling capabilities
/// - **Streaming**: Support for real-time response streaming
/// - **Provider Selection**: Configure routing and fallback preferences
/// - **Advanced Parameters**: Fine-tune model behavior with sampling parameters

#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, bon::Builder)]
#[builder(on(String, into), derive(Debug), builder_type(vis = "pub"))]
pub(crate) struct ChatRequest<M: Mode> {
    #[cfg(feature = "otel")]
    #[serde(skip)]
    #[builder(start_fn)]
    pub span: Span,

    #[serde(skip)]
    #[builder(start_fn)]
    handler: Option<ChatHandler<M>>,

    /// List of messages in the conversation
    #[builder(into, start_fn)]
    pub messages: Messages,

    /// Enable streaming of results. Defaults to false
    #[builder(field)]
    pub stream: Option<bool>,

    /// Preferences for provider routing.
    #[builder(field)]
    pub provider: Option<ProviderPreferences>,

    /// Configuration for model reasoning/thinking tokens
    #[builder(field)]
    pub reasoning: Option<ReasoningConfig>,

    /// The model ID to use. If unspecified, the user's default is used.
    pub model: Option<String>,

    /// Optional structured output format specification.
    ///
    /// When provided, this constrains the model to generate responses that conform
    /// to the specified JSON schema. This is particularly useful for:
    /// - Data extraction tasks
    /// - API responses that need consistent structure
    /// - Integration with type-safe deserialization
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// let format = Format::json("user_profile")
    ///     .with_schema(|schema| {
    ///         schema
    ///             .property("name", Param::string())
    ///             .property("age", Param::integer())
    ///             .required(["name", "age"])
    ///     })
    ///     .build();
    /// ```
    #[builder(into)]
    pub response_format: Option<Format>,

    /// Alternate list of models for routing overrides.
    #[builder(name = "fallbacks", with = |models: impl IntoIterator<Item: Into<String>>| models.into_iter().map(Into::into).collect())]
    pub models: Option<Vec<String>>,

    /// Optional collection of tools (functions) the model can call.
    ///
    /// Tools enable the model to interact with external systems, retrieve data,
    /// or perform actions beyond text generation. Each tool defines a function
    /// signature with typed parameters that the model can invoke during conversation.
    ///
    /// Tools work in conjunction with structured output - when a model calls a tool,
    /// the parameters follow the schema defined in the tool's parameter specification.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// let tools = vec![
    ///     Tool::function("get_weather")
    ///         .description("Get current weather for a location")
    ///         .with_parameters(|params| {
    ///             params
    ///                 .property("location", Param::string())
    ///                 .property("units", Param::string().enums(["celsius", "fahrenheit"]))
    ///                 .required(["location"])
    ///         })
    ///         .build()
    /// ];
    /// ```
    #[builder(into)]
    pub tools: Option<Tools>,

    #[builder(into)]
    pub plugins: Option<Vec<Plugin>>,

    /// Whether to include usage information in the response
    pub usage: Option<UsageConfig>,

    /// List of prompt transforms (OpenRouter-only).
    pub transforms: Option<Vec<String>>,

    /// Maximum number of tokens (range: [1, context_length)).
    pub max_tokens: Option<i32>,

    /// Sampling temperature (range: [0, 2]).
    pub temperature: Option<f64>,

    /// Seed for deterministic outputs.
    pub seed: Option<i32>,

    /// Top-p sampling value (range: (0, 1]).
    pub top_p: Option<f64>,

    /// Top-k sampling value (range: [1, Infinity)).
    pub top_k: Option<i32>,

    /// Frequency penalty (range: [-2, 2]).
    pub frequency_penalty: Option<f64>,

    /// Presence penalty (range: [-2, 2]).
    pub presence_penalty: Option<f64>,

    /// Repetition penalty (range: (0, 2]).
    pub repetition_penalty: Option<f64>,

    /// Mapping of token IDs to bias values.
    pub logit_bias: Option<HashMap<String, f64>>,

    /// Number of top log probabilities to return.
    pub top_logprobs: Option<i32>,

    /// Minimum probability threshold (range: [0, 1]).
    pub min_p: Option<f64>,

    /// Alternate top sampling parameter (range: [0, 1]).
    pub top_a: Option<f64>,

    /// A stable identifier for your end-users. Used to help detect and prevent abuse.
    pub user: Option<String>,
}

impl<M: Mode, S: chat_request_builder::State> ChatRequestBuilder<M, S> {
    /// Sets provider routing preferences for model selection.
    ///
    /// This method allows you to specify preferences for how models should be
    /// selected and routed. This works alongside structured output to ensure
    /// compatible models are chosen for schema-constrained responses.
    pub fn preferences(mut self, preferences: ProviderPreferences) -> Self {
        self.provider = Some(preferences);
        self
    }

    pub fn with_preferences<F, C>(mut self, build_preferences: F) -> Self
    where
        F: FnOnce(ProviderPreferencesBuilder) -> ProviderPreferencesBuilder<C>,
        C: provider_preferences_builder::IsComplete,
    {
        let builder = ProviderPreferences::builder();
        let preferences = build_preferences(builder).build();
        self.provider = Some(preferences);
        self
    }

    pub fn reasoning(mut self, config: ReasoningConfig) -> Self {
        self.reasoning = Some(config);
        self
    }

    pub fn with_reasoning<F, C>(mut self, build_reasoning: F) -> Self
    where
        F: FnOnce(ReasoningConfigBuilder) -> ReasoningConfigBuilder<C>,
        C: reasoning_config_builder::IsComplete,
    {
        let builder = ReasoningConfig::builder();
        let config = build_reasoning(builder).build();
        self.reasoning = Some(config);
        self
    }
}

impl<S: chat_request_builder::State> ChatRequestBuilder<Sync, S>
where
    S: chat_request_builder::IsComplete,
{
    /// Sends the chat request and returns a complete response.
    ///
    /// This method executes the chat completion request synchronously and returns
    /// the full response once the model has finished generating. When used with
    /// structured output (`response_format`), the response content will conform
    /// to the specified JSON schema.
    ///
    /// # Returns
    ///
    /// A `ChatCompletion` containing the model's response, which can include:
    /// - Text content (potentially structured JSON if `response_format` was specified)
    /// - Tool calls (if tools were provided and the model chose to use them)
    /// - Usage statistics and metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use orpheus::prelude::*;
    ///
    /// let client = Orpheus::from_env()?;
    ///
    /// // Regular text response
    /// let response = client
    ///     .chat("Hello!")
    ///     .model("openai/gpt-4o")
    ///     .send()?;
    ///
    /// // Structured JSON response
    /// let format = Format::json("person")
    ///     .with_schema(|schema| {
    ///         schema
    ///             .property("name", Param::string())
    ///             .property("age", Param::integer())
    ///             .required(["name", "age"])
    ///     })
    ///     .build();
    ///
    /// let response = client
    ///     .chat("Extract info: John is 25 years old")
    ///     .model("openai/gpt-4o")
    ///     .response_format(format)
    ///     .send()?;
    ///
    /// Ok(())
    /// # }
    /// ```
    pub fn send(mut self) -> Result<ChatCompletion> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

        // Disable streaming for complete response
        self.stream = Some(false);
        let body = self.build();
        debug!(chat_request_body = ?body);

        let response = handler.execute(body)?;

        let chat_completion = response.json::<ChatCompletion>().map_err(Error::http)?;
        debug!(chat_completion_response = ?chat_completion);

        #[cfg(feature = "otel")]
        crate::otel::record_completion(span, &chat_completion);

        Ok(chat_completion)
    }

    /// Sends the chat request and returns a streaming response.
    ///
    /// This method executes the chat completion request with streaming enabled,
    /// allowing you to process the response as it's being generated. This can
    /// reduce perceived latency for long responses.
    ///
    /// **Note**: Structured output (`response_format`) may not work optimally
    /// with streaming, as JSON schema validation typically requires the complete
    /// response. Consider using `send()` instead for structured output use cases.
    ///
    /// # Returns
    ///
    /// A `ChatStream` that yields response chunks as they arrive from the model.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use orpheus::prelude::*;
    ///
    /// let client = Orpheus::from_env()?;
    ///
    /// let mut stream = client
    ///     .chat("Write a story about a robot")
    ///     .model("openai/gpt-4o")
    ///     .stream()?;
    ///
    /// while let Some(Ok(chunk)) = stream.next() {
    ///     if let Ok(content) = chunk.content() {
    ///         let text = content.to_string();
    ///         print!("{}", text); // Print as it arrives
    ///     }
    /// }
    ///
    /// Ok(())
    /// # }
    /// ```
    pub fn stream(mut self) -> Result<ChatStream> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

        // Enable streaming for real-time response
        self.stream = Some(true);
        let body = self.build();
        let response = handler.execute(body)?;

        #[allow(unused_mut)]
        let mut stream = ChatStream::new(response);

        #[cfg(feature = "otel")]
        stream.aggregator.set_span(span);

        Ok(stream)
    }
}

impl<S: chat_request_builder::State> ChatRequestBuilder<Async, S>
where
    S: chat_request_builder::IsComplete,
{
    /// Asynchronously sends the chat request and returns a complete response.
    ///
    /// This is the async variant of `send()`. It executes the chat completion
    /// request asynchronously and returns the full response once the model has
    /// finished generating. When used with structured output (`response_format`),
    /// the response content will conform to the specified JSON schema.
    ///
    /// # Returns
    ///
    /// A `ChatCompletion` containing the model's response, including structured
    /// JSON if `response_format` was specified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use orpheus::prelude::*;
    ///
    /// let client = AsyncOrpheus::from_env()?;
    ///
    /// // Structured output with async
    /// let format = Format::json("analysis")
    ///     .with_schema(|schema| {
    ///         schema
    ///             .property("sentiment", Param::string().enums(["positive", "negative", "neutral"]))
    ///             .property("confidence", Param::number())
    ///             .required(["sentiment", "confidence"])
    ///     })
    ///     .build();
    ///
    /// let response = client
    ///     .chat("This movie was amazing!")
    ///     .model("openai/gpt-4o")
    ///     .response_format(format)
    ///     .send()
    ///     .await?;
    ///
    /// Ok(())
    /// # }
    /// ```
    pub async fn send(mut self) -> Result<ChatCompletion> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

        // Disable streaming for complete response
        self.stream = Some(false);
        let body = self.build();
        debug!(chat_request_body = ?body);

        let response = handler.execute(body).await?;

        let chat_completion = response
            .json::<ChatCompletion>()
            .await
            .map_err(Error::http)?;
        debug!(chat_completion_response = ?chat_completion);

        #[cfg(feature = "otel")]
        crate::otel::record_completion(span, &chat_completion);

        Ok(chat_completion)
    }

    /// Asynchronously sends the chat request and returns a streaming response.
    ///
    /// This is the async variant of `stream()`. It executes the chat completion
    /// request with streaming enabled, allowing you to process the response as
    /// it's being generated asynchronously.
    ///
    /// **Note**: As with the sync version, structured output (`response_format`)
    /// may not work optimally with streaming. Consider using `send()` for
    /// structured output use cases.
    ///
    /// # Returns
    ///
    /// An `AsyncStream` that yields response chunks as they arrive from the model.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use orpheus::prelude::*;
    /// use futures_lite::StreamExt;
    ///
    /// let client = AsyncOrpheus::from_env()?;
    /// let mut stream = client
    ///     .chat("Explain quantum computing")
    ///     .model("openai/gpt-4o")
    ///     .stream()
    ///     .await?;
    ///
    /// while let Some(Ok(chunk)) = stream.next().await {
    ///     if let Ok(content) = chunk.content() {
    ///         let text = content.to_string();
    ///         print!("{}", text); // Print as it arrives
    ///     }
    /// }
    ///
    /// Ok(())
    /// # }
    /// ```
    pub async fn stream(mut self) -> Result<AsyncStream> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

        // Enable streaming for real-time response
        self.stream = Some(true);
        let body = self.build();

        let response = handler.execute(body).await?;

        #[allow(unused_mut)]
        let mut stream = AsyncStream::new(response);

        #[cfg(feature = "otel")]
        stream.aggregator.set_span(span);

        Ok(stream)
    }
}
