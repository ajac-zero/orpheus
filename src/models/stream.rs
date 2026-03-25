use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_lite::{Stream, StreamExt};
use http_body_util::BodyExt;

use crate::{Error, Result};

/// All possible streaming events from the Open Responses API.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ResponseEvent {
    // Lifecycle
    Created {
        response: open_responses::ResponseResource,
    },
    InProgress {
        response: open_responses::ResponseResource,
    },
    Queued {
        response: open_responses::ResponseResource,
    },

    // Output items
    OutputItemAdded {
        output_index: i64,
        item: open_responses::OutputItem,
    },
    OutputItemDone {
        output_index: i64,
        item: open_responses::OutputItem,
    },

    // Content parts
    ContentPartAdded {
        item_id: String,
        output_index: i64,
        content_index: i64,
        part: open_responses::ContentPart,
    },
    ContentPartDone {
        item_id: String,
        output_index: i64,
        content_index: i64,
        part: open_responses::ContentPart,
    },

    // Text
    TextDelta {
        item_id: String,
        output_index: i64,
        content_index: i64,
        delta: String,
    },
    TextDone {
        item_id: String,
        output_index: i64,
        content_index: i64,
        text: String,
    },
    TextAnnotationAdded {
        item_id: String,
        output_index: i64,
        content_index: i64,
        annotation_index: i64,
        annotation: open_responses::Annotation,
    },

    // Function calls
    FunctionCallDelta {
        item_id: String,
        output_index: i64,
        delta: String,
    },
    FunctionCallDone {
        item_id: String,
        output_index: i64,
        arguments: String,
    },

    // Reasoning
    ReasoningDelta {
        item_id: String,
        output_index: i64,
        content_index: i64,
        delta: String,
    },
    ReasoningDone {
        item_id: String,
        output_index: i64,
        content_index: i64,
        text: String,
    },
    ReasoningSummaryDelta {
        item_id: String,
        output_index: i64,
        summary_index: i64,
        delta: String,
    },
    ReasoningSummaryDone {
        item_id: String,
        output_index: i64,
        summary_index: i64,
        text: String,
    },
    ReasoningSummaryPartAdded {
        item_id: String,
        output_index: i64,
        summary_index: i64,
        part: open_responses::ReasoningContentPart,
    },
    ReasoningSummaryPartDone {
        item_id: String,
        output_index: i64,
        summary_index: i64,
        part: open_responses::ReasoningContentPart,
    },

    // Refusal
    RefusalDelta {
        item_id: String,
        output_index: i64,
        content_index: i64,
        delta: String,
    },
    RefusalDone {
        item_id: String,
        output_index: i64,
        content_index: i64,
        refusal: String,
    },

    // Terminal
    Completed {
        response: open_responses::ResponseResource,
    },
    Failed {
        response: open_responses::ResponseResource,
    },
    Incomplete {
        response: open_responses::ResponseResource,
    },

    // Errors
    Error {
        code: String,
        message: String,
    },

    // Forward compatibility
    Unknown {
        event_type: String,
        data: String,
    },
}

impl ResponseEvent {
    /// Returns the text delta if this is a `TextDelta` event.
    pub fn as_text_delta(&self) -> Option<&str> {
        match self {
            ResponseEvent::TextDelta { delta, .. } => Some(delta),
            _ => None,
        }
    }

    /// Returns the function call arguments delta if this is a `FunctionCallDelta` event.
    pub fn as_function_call_delta(&self) -> Option<&str> {
        match self {
            ResponseEvent::FunctionCallDelta { delta, .. } => Some(delta),
            _ => None,
        }
    }

    /// Returns the completed response if this is a `Completed` event.
    pub fn as_completed(&self) -> Option<&open_responses::ResponseResource> {
        match self {
            ResponseEvent::Completed { response } => Some(response),
            _ => None,
        }
    }
}

fn parse_event(event_type: &str, data: &str) -> Result<ResponseEvent> {
    let event = match event_type {
        "response.created" => {
            let e: open_responses::ResponseCreatedStreamingEvent = serde_json::from_str(data)?;
            ResponseEvent::Created {
                response: e.response,
            }
        }
        "response.in_progress" => {
            let e: open_responses::ResponseInProgressStreamingEvent = serde_json::from_str(data)?;
            ResponseEvent::InProgress {
                response: e.response,
            }
        }
        "response.queued" => {
            let e: open_responses::ResponseQueuedStreamingEvent = serde_json::from_str(data)?;
            ResponseEvent::Queued {
                response: e.response,
            }
        }
        "response.output_item.added" => {
            let e: open_responses::ResponseOutputItemAddedStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::OutputItemAdded {
                output_index: e.output_index,
                item: e.item,
            }
        }
        "response.output_item.done" => {
            let e: open_responses::ResponseOutputItemDoneStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::OutputItemDone {
                output_index: e.output_index,
                item: e.item,
            }
        }
        "response.content_part.added" => {
            let e: open_responses::ResponseContentPartAddedStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::ContentPartAdded {
                item_id: e.item_id,
                output_index: e.output_index,
                content_index: e.content_index,
                part: e.part,
            }
        }
        "response.content_part.done" => {
            let e: open_responses::ResponseContentPartDoneStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::ContentPartDone {
                item_id: e.item_id,
                output_index: e.output_index,
                content_index: e.content_index,
                part: e.part,
            }
        }
        "response.output_text.delta" => {
            let e: open_responses::ResponseOutputTextDeltaStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::TextDelta {
                item_id: e.item_id,
                output_index: e.output_index,
                content_index: e.content_index,
                delta: e.delta,
            }
        }
        "response.output_text.done" => {
            let e: open_responses::ResponseOutputTextDoneStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::TextDone {
                item_id: e.item_id,
                output_index: e.output_index,
                content_index: e.content_index,
                text: e.text,
            }
        }
        "response.output_text.annotation.added" => {
            let e: open_responses::ResponseOutputTextAnnotationAddedStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::TextAnnotationAdded {
                item_id: e.item_id,
                output_index: e.output_index,
                content_index: e.content_index,
                annotation_index: e.annotation_index,
                annotation: e.annotation,
            }
        }
        "response.function_call_arguments.delta" => {
            let e: open_responses::ResponseFunctionCallArgumentsDeltaStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::FunctionCallDelta {
                item_id: e.item_id,
                output_index: e.output_index,
                delta: e.delta,
            }
        }
        "response.function_call_arguments.done" => {
            let e: open_responses::ResponseFunctionCallArgumentsDoneStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::FunctionCallDone {
                item_id: e.item_id,
                output_index: e.output_index,
                arguments: e.arguments,
            }
        }
        "response.reasoning.delta" => {
            let e: open_responses::ResponseReasoningDeltaStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::ReasoningDelta {
                item_id: e.item_id,
                output_index: e.output_index,
                content_index: e.content_index,
                delta: e.delta,
            }
        }
        "response.reasoning.done" => {
            let e: open_responses::ResponseReasoningDoneStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::ReasoningDone {
                item_id: e.item_id,
                output_index: e.output_index,
                content_index: e.content_index,
                text: e.text,
            }
        }
        "response.reasoning_summary.delta" => {
            let e: open_responses::ResponseReasoningSummaryDeltaStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::ReasoningSummaryDelta {
                item_id: e.item_id,
                output_index: e.output_index,
                summary_index: e.summary_index,
                delta: e.delta,
            }
        }
        "response.reasoning_summary.done" => {
            let e: open_responses::ResponseReasoningSummaryDoneStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::ReasoningSummaryDone {
                item_id: e.item_id,
                output_index: e.output_index,
                summary_index: e.summary_index,
                text: e.text,
            }
        }
        "response.reasoning_summary_part.added" => {
            let e: open_responses::ResponseReasoningSummaryPartAddedStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::ReasoningSummaryPartAdded {
                item_id: e.item_id,
                output_index: e.output_index,
                summary_index: e.summary_index,
                part: e.part,
            }
        }
        "response.reasoning_summary_part.done" => {
            let e: open_responses::ResponseReasoningSummaryPartDoneStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::ReasoningSummaryPartDone {
                item_id: e.item_id,
                output_index: e.output_index,
                summary_index: e.summary_index,
                part: e.part,
            }
        }
        "response.refusal.delta" => {
            let e: open_responses::ResponseRefusalDeltaStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::RefusalDelta {
                item_id: e.item_id,
                output_index: e.output_index,
                content_index: e.content_index,
                delta: e.delta,
            }
        }
        "response.refusal.done" => {
            let e: open_responses::ResponseRefusalDoneStreamingEvent =
                serde_json::from_str(data)?;
            ResponseEvent::RefusalDone {
                item_id: e.item_id,
                output_index: e.output_index,
                content_index: e.content_index,
                refusal: e.refusal,
            }
        }
        "response.completed" => {
            let e: open_responses::ResponseCompletedStreamingEvent = serde_json::from_str(data)?;
            ResponseEvent::Completed {
                response: e.response,
            }
        }
        "response.failed" => {
            let e: open_responses::ResponseFailedStreamingEvent = serde_json::from_str(data)?;
            ResponseEvent::Failed {
                response: e.response,
            }
        }
        "response.incomplete" => {
            let e: open_responses::ResponseIncompleteStreamingEvent = serde_json::from_str(data)?;
            ResponseEvent::Incomplete {
                response: e.response,
            }
        }
        "error" => {
            let e: open_responses::ErrorStreamingEvent = serde_json::from_str(data)?;
            ResponseEvent::Error {
                code: e.error.code,
                message: e.error.message,
            }
        }
        other => ResponseEvent::Unknown {
            event_type: other.to_string(),
            data: data.to_string(),
        },
    };

    Ok(event)
}

/// A streaming response from the Open Responses API.
pub struct ResponseStream {
    events: tokio::sync::mpsc::UnboundedReceiver<Result<ResponseEvent>>,
    result: Option<tokio::sync::oneshot::Receiver<Result<open_responses::ResponseResource>>>,
    rt: Option<Arc<tokio::runtime::Runtime>>,
}

impl ResponseStream {
    pub(crate) fn spawn(
        response: hyper::Response<hyper::body::Incoming>,
        rt: Option<Arc<tokio::runtime::Runtime>>,
    ) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();

        let producer = async move {
            let mut stream = response.into_data_stream();
            let mut buffer = Vec::new();
            let mut result_tx = Some(result_tx);

            loop {
                if let Some((event_type, data)) = extract_sse_frame(&mut buffer) {
                    if data == "[DONE]" {
                        break;
                    }

                    let parsed = resolve_event_type(event_type.as_deref(), &data);

                    match &parsed {
                        Ok(ResponseEvent::Completed { response }) => {
                            if let Some(tx) = result_tx.take() {
                                let _ = tx.send(Ok(response.clone()));
                            }
                        }
                        Ok(ResponseEvent::Failed { response }) => {
                            if let Some(tx) = result_tx.take() {
                                let _ = tx.send(Err(Error::StreamClosed));
                                let _ = event_tx.send(Ok(ResponseEvent::Failed {
                                    response: response.clone(),
                                }));
                                break;
                            }
                        }
                        Ok(ResponseEvent::Incomplete { response }) => {
                            if let Some(tx) = result_tx.take() {
                                let _ = tx.send(Err(Error::StreamClosed));
                                let _ = event_tx.send(Ok(ResponseEvent::Incomplete {
                                    response: response.clone(),
                                }));
                                break;
                            }
                        }
                        _ => {}
                    }

                    if event_tx.send(parsed).is_err() {
                        break;
                    }

                    continue;
                }

                match stream.next().await {
                    None => break,
                    Some(Ok(bytes)) => buffer.extend_from_slice(&bytes),
                    Some(Err(e)) => {
                        let _ = event_tx.send(Err(Error::Hyper(e)));
                        break;
                    }
                }
            }
        };

        match &rt {
            Some(handle) => {
                handle.spawn(producer);
            }
            None => {
                tokio::spawn(producer);
            }
        }

        Self {
            events: event_rx,
            result: Some(result_rx),
            rt,
        }
    }

    /// Consumes the stream and returns the final response.
    /// Drains all remaining events before returning.
    pub async fn final_result(mut self) -> Result<open_responses::ResponseResource> {
        while self.events.recv().await.is_some() {}

        self.result
            .take()
            .expect("final_result called only once")
            .await
            .map_err(|_| Error::StreamClosed)?
    }
}

/// Parse SSE frames from buffer. Returns (event_type, data) if a complete frame is found.
fn extract_sse_frame(buffer: &mut Vec<u8>) -> Option<(Option<String>, String)> {
    let buf_str = String::from_utf8_lossy(buffer);

    let frame_end = buf_str
        .find("\n\n")
        .map(|pos| (pos, pos + 2))
        .or_else(|| buf_str.find("\r\n\r\n").map(|pos| (pos, pos + 4)));

    let (content_end, drain_end) = frame_end?;

    let frame_str = String::from_utf8_lossy(&buffer[..content_end]).to_string();
    buffer.drain(..drain_end);

    let mut event_type = None;
    let mut data_lines = Vec::new();

    for line in frame_str.lines() {
        if let Some(value) = line.strip_prefix("event: ").or_else(|| line.strip_prefix("event:")) {
            event_type = Some(value.trim().to_string());
        } else if let Some(value) =
            line.strip_prefix("data: ").or_else(|| line.strip_prefix("data:"))
        {
            data_lines.push(value.to_string());
        }
    }

    if data_lines.is_empty() {
        return None;
    }

    let data = data_lines.join("\n");
    Some((event_type, data))
}

fn resolve_event_type(event_type: Option<&str>, data: &str) -> Result<ResponseEvent> {
    match event_type {
        Some(name) => parse_event(name, data),
        None => {
            let value: serde_json::Value = serde_json::from_str(data)?;
            let type_str = value
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            parse_event(type_str, data)
        }
    }
}

impl Stream for ResponseStream {
    type Item = Result<ResponseEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().events.poll_recv(cx)
    }
}

impl Iterator for ResponseStream {
    type Item = Result<ResponseEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let rt = self.rt.as_ref().expect("Iterator requires sync mode");
        rt.block_on(self.events.recv())
    }
}

impl std::fmt::Debug for ResponseStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseStream").finish()
    }
}
