use std::collections::HashMap;

use bon::bon;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    Result,
    backend::traits::{AsyncRequestBuilder, Backend, Mode, RequestBuilder, SyncRequestBuilder},
    models::Input,
};

fn parse_api_error(status: u16, body: &str) -> open_responses::client::Error {
    #[derive(Deserialize)]
    struct Envelope {
        error: Body,
    }
    #[derive(Deserialize)]
    struct Body {
        message: String,
    }
    match serde_json::from_str::<Envelope>(body) {
        Ok(e) => open_responses::client::Error::Api { status, message: e.error.message },
        Err(_) => open_responses::client::Error::Unexpected { status, body: body.to_string() },
    }
}

pub(crate) const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

// ── Internal Gemini wire types ────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTools>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_config: Option<GeminiToolConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<GeminiFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_response: Option<GeminiFunctionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline_data: Option<GeminiInlineData>,
}

impl GeminiPart {
    fn text(text: impl Into<String>) -> Self {
        Self { text: Some(text.into()), function_call: None, function_response: None, inline_data: None }
    }
    fn function_call(fc: GeminiFunctionCall) -> Self {
        Self { text: None, function_call: Some(fc), function_response: None, inline_data: None }
    }
    fn function_response(fr: GeminiFunctionResponse) -> Self {
        Self { text: None, function_call: None, function_response: Some(fr), inline_data: None }
    }
    fn inline_data(d: GeminiInlineData) -> Self {
        Self { text: None, function_call: None, function_response: None, inline_data: Some(d) }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiInlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiTools {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Serialize)]
struct GeminiFunctionDeclaration {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiToolConfig {
    function_calling_config: GeminiFunctionCallingConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiFunctionCallingConfig {
    mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_function_names: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking_config: Option<GeminiThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_schema: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiThinkingConfig {
    thinking_budget: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    usage_metadata: Option<GeminiUsageMetadata>,
    #[serde(default)]
    model_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    #[serde(default)]
    prompt_token_count: Option<i64>,
    #[serde(default)]
    candidates_token_count: Option<i64>,
    #[serde(default)]
    total_token_count: Option<i64>,
}

// ── Translation: Open Responses → Gemini ─────────────────────────────────────

fn input_items_to_gemini(
    items: Vec<open_responses::InputItem>,
    instructions: Option<String>,
) -> (Vec<GeminiContent>, Option<GeminiContent>) {
    let mut contents: Vec<GeminiContent> = Vec::new();
    let mut system_parts: Vec<GeminiPart> = Vec::new();

    if let Some(instr) = instructions {
        system_parts.push(GeminiPart::text(instr));
    }

    for item in items {
        match item {
            open_responses::InputItem::UserMessage(m) => {
                let parts = content_value_to_parts(&m.content, "user");
                if !parts.is_empty() {
                    contents.push(GeminiContent { role: "user".into(), parts });
                }
            }
            open_responses::InputItem::AssistantMessage(m) => {
                let parts = content_value_to_parts(&m.content, "assistant");
                if !parts.is_empty() {
                    contents.push(GeminiContent { role: "model".into(), parts });
                }
            }
            open_responses::InputItem::SystemMessage(m) => {
                let parts = content_value_to_parts(&m.content, "system");
                system_parts.extend(parts);
            }
            open_responses::InputItem::DeveloperMessage(m) => {
                let parts = content_value_to_parts(&m.content, "developer");
                system_parts.extend(parts);
            }
            open_responses::InputItem::FunctionCall(fc) => {
                let args: serde_json::Value = serde_json::from_str(&fc.arguments)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                contents.push(GeminiContent {
                    role: "model".into(),
                    parts: vec![GeminiPart::function_call(GeminiFunctionCall {
                        name: fc.name,
                        args,
                    })],
                });
            }
            open_responses::InputItem::FunctionCallOutput(fco) => {
                // call_id isn't used in Gemini function responses — use the name from output if present,
                // otherwise fall back to the call_id as a placeholder name
                let name = fco.call_id.clone();
                let response = fco.output.clone();
                contents.push(GeminiContent {
                    role: "user".into(),
                    parts: vec![GeminiPart::function_response(GeminiFunctionResponse {
                        name,
                        response,
                    })],
                });
            }
            // Reasoning items and item references have no Gemini equivalent
            open_responses::InputItem::Reasoning(_) | open_responses::InputItem::ItemReference(_) => {}
        }
    }

    let system_instruction = if system_parts.is_empty() {
        None
    } else {
        Some(GeminiContent { role: "system".into(), parts: system_parts })
    };

    (contents, system_instruction)
}

fn content_value_to_parts(value: &serde_json::Value, _role: &str) -> Vec<GeminiPart> {
    match value {
        serde_json::Value::String(s) => vec![GeminiPart::text(s.clone())],
        serde_json::Value::Array(arr) => arr.iter().filter_map(content_part_value_to_part).collect(),
        _ => vec![],
    }
}

fn content_part_value_to_part(value: &serde_json::Value) -> Option<GeminiPart> {
    let type_ = value.get("type").and_then(|v| v.as_str())?;
    match type_ {
        "input_text" | "output_text" | "text" => {
            let text = value.get("text").and_then(|v| v.as_str())?;
            Some(GeminiPart::text(text.to_string()))
        }
        "input_image" => {
            // Prefer inline base64 data; fall back to URL (not directly embeddable in Gemini parts)
            if let Some(url) = value.get("image_url").and_then(|v| v.as_str()) {
                // Gemini fileData part for URL-based images
                Some(GeminiPart {
                    text: None,
                    function_call: None,
                    function_response: None,
                    inline_data: Some(GeminiInlineData {
                        mime_type: "image/jpeg".into(),
                        data: url.to_string(),
                    }),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn tools_to_gemini(tools: &[open_responses::FunctionToolParam]) -> Option<Vec<GeminiTools>> {
    if tools.is_empty() {
        return None;
    }
    let decls: Vec<GeminiFunctionDeclaration> = tools
        .iter()
        .map(|t| GeminiFunctionDeclaration {
            name: t.name.clone(),
            description: t.description.clone(),
            parameters: t.parameters.clone(),
        })
        .collect();
    Some(vec![GeminiTools { function_declarations: decls }])
}

fn tool_choice_to_gemini(choice: &open_responses::ToolChoiceParam) -> GeminiToolConfig {
    use open_responses::{ToolChoiceParam, ToolChoiceValueEnum};
    let (mode, allowed) = match choice {
        ToolChoiceParam::Mode(ToolChoiceValueEnum::None) => ("NONE".into(), None),
        ToolChoiceParam::Mode(ToolChoiceValueEnum::Auto) => ("AUTO".into(), None),
        ToolChoiceParam::Mode(ToolChoiceValueEnum::Required) => ("ANY".into(), None),
        ToolChoiceParam::AllowedTools(a) => ("ANY".into(), Some(a.tools.clone())),
        ToolChoiceParam::SpecificFunction(f) => ("ANY".into(), Some(vec![f.name.clone()])),
    };
    GeminiToolConfig {
        function_calling_config: GeminiFunctionCallingConfig {
            mode,
            allowed_function_names: allowed,
        },
    }
}

fn reasoning_to_thinking_budget(reasoning: &open_responses::ReasoningParam) -> i64 {
    use open_responses::ReasoningEffortEnum;
    match reasoning.effort {
        ReasoningEffortEnum::None => 0,
        ReasoningEffortEnum::Low => 1024,
        ReasoningEffortEnum::Medium => 8192,
        ReasoningEffortEnum::High => 24576,
        ReasoningEffortEnum::Xhigh => -1,
    }
}

// ── Translation: Gemini → Open Responses ─────────────────────────────────────

fn gemini_response_to_resource(
    response: GeminiResponse,
    model: &str,
    temperature: f64,
    top_p: f64,
    presence_penalty: f64,
    frequency_penalty: f64,
    max_output_tokens: Option<i64>,
    reasoning: Option<open_responses::ReasoningParam>,
    tools: Vec<open_responses::FunctionToolParam>,
    tool_choice: open_responses::ToolChoiceParam,
) -> open_responses::ResponseResource {
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut output: Vec<open_responses::OutputItem> = Vec::new();
    let mut call_index: u32 = 0;

    for candidate in response.candidates {
        let mut text_parts: Vec<open_responses::OutputContentPart> = Vec::new();

        for part in candidate.content.parts {
            if let Some(text) = part.text {
                text_parts.push(open_responses::OutputContentPart::OutputText(
                    open_responses::OutputTextContent {
                        type_: "output_text".into(),
                        text,
                        annotations: vec![],
                        logprobs: None,
                    },
                ));
            }
            if let Some(fc) = part.function_call {
                // Flush any accumulated text parts as a message first
                if !text_parts.is_empty() {
                    output.push(make_message(std::mem::take(&mut text_parts), created_at, call_index));
                    call_index += 1;
                }
                let call_id = format!("call_{created_at}_{call_index}");
                output.push(open_responses::OutputItem::FunctionCall(
                    open_responses::FunctionCall {
                        type_: "function_call".into(),
                        id: format!("fc_{created_at}_{call_index}"),
                        call_id,
                        status: open_responses::FunctionCallStatus::Completed,
                        name: fc.name,
                        arguments: fc.args.to_string(),
                    },
                ));
                call_index += 1;
            }
        }

        if !text_parts.is_empty() {
            output.push(make_message(text_parts, created_at, call_index));
            call_index += 1;
        }
    }

    let usage = response.usage_metadata.map(|u| open_responses::Usage {
        input_tokens: u.prompt_token_count.unwrap_or(0),
        output_tokens: u.candidates_token_count.unwrap_or(0),
        total_tokens: u.total_token_count.unwrap_or(0),
        input_tokens_details: open_responses::InputTokensDetails { cached_tokens: 0 },
        output_tokens_details: open_responses::OutputTokensDetails { reasoning_tokens: 0 },
    });

    open_responses::ResponseResource {
        id: format!("gemini-{created_at}"),
        object: "response".into(),
        created_at,
        completed_at: None,
        status: "completed".into(),
        incomplete_details: None,
        model: response.model_version.unwrap_or_else(|| model.to_string()),
        previous_response_id: None,
        instructions: serde_json::Value::Null,
        output,
        error: None,
        tools,
        tool_choice,
        truncation: open_responses::TruncationEnum::Auto,
        parallel_tool_calls: false,
        text: serde_json::Value::Null,
        top_p,
        presence_penalty,
        frequency_penalty,
        top_logprobs: 0,
        temperature,
        reasoning,
        usage,
        max_output_tokens,
        max_tool_calls: None,
        store: false,
        background: false,
        service_tier: "default".into(),
        metadata: serde_json::Value::Null,
        safety_identifier: None,
        prompt_cache_key: None,
    }
}

fn make_message(
    content: Vec<open_responses::OutputContentPart>,
    created_at: i64,
    index: u32,
) -> open_responses::OutputItem {
    open_responses::OutputItem::Message(open_responses::Message {
        type_: "message".into(),
        id: format!("msg_{created_at}_{index}"),
        status: open_responses::MessageStatus::Completed,
        role: open_responses::MessageRole::Assistant,
        content: content
            .into_iter()
            .map(|p| match p {
                open_responses::OutputContentPart::OutputText(t) => {
                    open_responses::ContentPart::OutputText(t)
                }
                open_responses::OutputContentPart::Refusal(r) => {
                    open_responses::ContentPart::Refusal(r)
                }
            })
            .collect(),
    })
}

// ── Backend struct ────────────────────────────────────────────────────────────

pub struct GeminiBackend<M> {
    pub(crate) api_key: SecretString,
    pub(crate) base_url: Url,
    pub(crate) headers: HashMap<String, String>,
    _mode: std::marker::PhantomData<M>,
}

pub struct Sync;
pub struct Async;

#[bon]
impl<M> GeminiBackend<M> {
    #[builder(on(SecretString, into), finish_fn = build_backend)]
    pub fn builder(
        #[builder(field)] headers: HashMap<String, String>,
        #[builder(default = Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"))]
        base_url: Url,
        api_key: Option<SecretString>,
    ) -> Self {
        Self {
            api_key: api_key.unwrap_or_else(|| SecretString::new("".into())),
            base_url,
            headers,
            _mode: std::marker::PhantomData,
        }
    }
}

impl<M, S: gemini_backend_builder::State> GeminiBackendBuilder<M, S> {
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> GeminiBackend<M> {
        self.build_backend()
    }
}

impl<M> GeminiBackend<M> {
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self::builder().api_key(api_key).build_backend()
    }

    pub fn from_env() -> crate::Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY")?;
        Ok(Self::new(api_key))
    }
}

// ── Mode impls ────────────────────────────────────────────────────────────────

pub struct GeminiMode<M>(std::marker::PhantomData<M>);

impl Mode for GeminiMode<Sync> {
    type RequestBuilder<'a> = GeminiRequestBuilder<'a, Sync>;
    type Response = open_responses::ResponseResource;
    type StreamResponse = open_responses::client::ResponseStream;
}

impl Mode for GeminiMode<Async> {
    type RequestBuilder<'a> = GeminiRequestBuilder<'a, Async>;
    type Response = open_responses::ResponseResource;
    type StreamResponse = open_responses::client::AsyncResponseStream;
}

impl Backend for GeminiBackend<Sync> {
    type Mode = GeminiMode<Sync>;

    fn create_request<'a>(&'a self, input: Input) -> GeminiRequestBuilder<'a, Sync> {
        GeminiRequestBuilder::new(self, input)
    }
}

impl Backend for GeminiBackend<Async> {
    type Mode = GeminiMode<Async>;

    fn create_request<'a>(&'a self, input: Input) -> GeminiRequestBuilder<'a, Async> {
        GeminiRequestBuilder::new(self, input)
    }
}

// ── Request builder ───────────────────────────────────────────────────────────

pub struct GeminiRequestBuilder<'a, M> {
    backend: &'a GeminiBackend<M>,
    input: Vec<open_responses::InputItem>,
    model: Option<String>,
    instructions: Option<String>,
    tools: Vec<open_responses::FunctionToolParam>,
    tool_choice: Option<open_responses::ToolChoiceParam>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    presence_penalty: f64,
    frequency_penalty: f64,
    max_output_tokens: Option<i64>,
    reasoning: Option<open_responses::ReasoningParam>,
    text_format: Option<crate::models::Format>,
}

impl<'a, M> GeminiRequestBuilder<'a, M> {
    fn new(backend: &'a GeminiBackend<M>, input: Input) -> Self {
        Self {
            backend,
            input: input.0,
            model: None,
            instructions: None,
            tools: vec![],
            tool_choice: None,
            temperature: None,
            top_p: None,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            max_output_tokens: None,
            reasoning: None,
            text_format: None,
        }
    }

    fn build_request(&self) -> (String, GeminiRequest) {
        let model = self.model.as_deref().unwrap_or("gemini-2.0-flash");

        let (contents, system_instruction) =
            input_items_to_gemini(self.input.clone(), self.instructions.clone());

        let gemini_tools = tools_to_gemini(&self.tools);

        let tool_config = self.tool_choice.as_ref().map(tool_choice_to_gemini);

        let thinking_config = self.reasoning.as_ref().map(|r| GeminiThinkingConfig {
            thinking_budget: reasoning_to_thinking_budget(r),
        });

        let (response_mime_type, response_schema) = match &self.text_format {
            Some(crate::models::Format::JsonSchema { schema, .. }) => {
                let schema_value = serde_json::to_value(schema).unwrap_or_default();
                (Some("application/json".into()), Some(schema_value))
            }
            None => (None, None),
        };

        let generation_config =
            if self.temperature.is_some()
                || self.top_p.is_some()
                || self.max_output_tokens.is_some()
                || thinking_config.is_some()
                || response_mime_type.is_some()
            {
                Some(GeminiGenerationConfig {
                    max_output_tokens: self.max_output_tokens,
                    temperature: self.temperature,
                    top_p: self.top_p,
                    thinking_config,
                    response_mime_type,
                    response_schema,
                })
            } else {
                None
            };

        let url = format!(
            "{}/models/{}:generateContent",
            self.backend.base_url.as_str().trim_end_matches('/'),
            model
        );

        (
            url,
            GeminiRequest {
                contents,
                system_instruction,
                tools: gemini_tools,
                tool_config,
                generation_config,
            },
        )
    }

    fn finalize_response(
        &self,
        gemini: GeminiResponse,
    ) -> open_responses::ResponseResource {
        let model = self.model.as_deref().unwrap_or("gemini-2.0-flash");
        gemini_response_to_resource(
            gemini,
            model,
            self.temperature.unwrap_or(1.0),
            self.top_p.unwrap_or(1.0),
            self.presence_penalty,
            self.frequency_penalty,
            self.max_output_tokens,
            self.reasoning.clone(),
            self.tools.clone(),
            self.tool_choice
                .clone()
                .unwrap_or(open_responses::ToolChoiceParam::Mode(open_responses::ToolChoiceValueEnum::Auto)),
        )
    }
}

impl<'a, M> RequestBuilder for GeminiRequestBuilder<'a, M> {
    fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    fn previous_response_id(self, _id: impl Into<String>) -> Self {
        self
    }

    fn tools(mut self, tools: impl IntoIterator<Item = crate::models::Tool>) -> Self {
        self.tools = tools.into_iter().map(Into::into).collect();
        self
    }

    fn tool_choice(mut self, choice: open_responses::ToolChoiceParam) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    fn metadata(self, _metadata: HashMap<String, String>) -> Self {
        self
    }

    fn text_format(mut self, format: crate::models::Format) -> Self {
        self.text_format = Some(format);
        self
    }

    fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    fn presence_penalty(mut self, presence_penalty: f64) -> Self {
        self.presence_penalty = presence_penalty;
        self
    }

    fn frequency_penalty(mut self, frequency_penalty: f64) -> Self {
        self.frequency_penalty = frequency_penalty;
        self
    }

    fn parallel_tool_calls(self, _parallel: bool) -> Self {
        self
    }

    fn max_output_tokens(mut self, max: i64) -> Self {
        self.max_output_tokens = Some(max);
        self
    }

    fn max_tool_calls(self, _max: i64) -> Self {
        self
    }

    fn reasoning(mut self, reasoning: open_responses::ReasoningParam) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    fn truncation(self, _truncation: open_responses::TruncationEnum) -> Self {
        self
    }

    fn include(self, _include: impl IntoIterator<Item = open_responses::IncludeEnum>) -> Self {
        self
    }

    fn store(self, _store: bool) -> Self {
        self
    }

    fn top_logprobs(self, _top_logprobs: i64) -> Self {
        self
    }
}

impl<'a> SyncRequestBuilder for GeminiRequestBuilder<'a, Sync> {
    fn send(self) -> Result<open_responses::ResponseResource> {
        let (url, body) = self.build_request();
        let api_key = self.backend.api_key.expose_secret().to_string();

        let mut req = reqwest::blocking::Client::new()
            .post(format!("{url}?key={api_key}"))
            .json(&body);

        for (k, v) in &self.backend.headers {
            req = req.header(k, v);
        }

        let response = req.send()?;
        let status = response.status().as_u16();
        let body_text = response.text()?;

        if status >= 400 {
            return Err(parse_api_error(status, &body_text));
        }

        let gemini: GeminiResponse = serde_json::from_str(&body_text)?;
        Ok(self.finalize_response(gemini))
    }

    fn stream(self) -> Result<open_responses::client::ResponseStream> {
        Err(open_responses::client::Error::Api {
            status: 501,
            message: "Streaming is not yet supported for the Gemini backend".into(),
        })
    }
}

impl<'a> AsyncRequestBuilder for GeminiRequestBuilder<'a, Async> {
    async fn send(self) -> Result<open_responses::ResponseResource> {
        let (url, body) = self.build_request();
        let api_key = self.backend.api_key.expose_secret().to_string();

        let mut req = reqwest::Client::new()
            .post(format!("{url}?key={api_key}"))
            .json(&body);

        for (k, v) in &self.backend.headers {
            req = req.header(k, v);
        }

        let response = req.send().await?;
        let status = response.status().as_u16();
        let body_text = response.text().await?;

        if status >= 400 {
            return Err(parse_api_error(status, &body_text));
        }

        let gemini: GeminiResponse = serde_json::from_str(&body_text)?;
        Ok(self.finalize_response(gemini))
    }

    async fn stream(self) -> Result<open_responses::client::AsyncResponseStream> {
        Err(open_responses::client::Error::Api {
            status: 501,
            message: "Streaming is not yet supported for the Gemini backend".into(),
        })
    }
}

impl<M> std::fmt::Debug for GeminiRequestBuilder<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiRequestBuilder")
            .field("model", &self.model)
            .field("temperature", &self.temperature)
            .field("max_output_tokens", &self.max_output_tokens)
            .finish()
    }
}
