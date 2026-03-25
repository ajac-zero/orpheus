use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::{Stream, StreamExt};
use http_body_util::{BodyDataStream, BodyExt};

use crate::{
    Error, Result,
    client::{
        handler::Response,
        mode::{Async, Sync},
    },
};

/// All possible streaming events from the Open Responses API.
#[derive(Debug, Clone)]
pub enum ResponseEvent {
    Created(open_responses::ResponseCreatedStreamingEvent),
    InProgress(open_responses::ResponseInProgressStreamingEvent),
    Queued(open_responses::ResponseQueuedStreamingEvent),
    OutputItemAdded(open_responses::ResponseOutputItemAddedStreamingEvent),
    OutputItemDone(open_responses::ResponseOutputItemDoneStreamingEvent),
    ContentPartAdded(open_responses::ResponseContentPartAddedStreamingEvent),
    ContentPartDone(open_responses::ResponseContentPartDoneStreamingEvent),
    OutputTextDelta(open_responses::ResponseOutputTextDeltaStreamingEvent),
    OutputTextDone(open_responses::ResponseOutputTextDoneStreamingEvent),
    OutputTextAnnotationAdded(open_responses::ResponseOutputTextAnnotationAddedStreamingEvent),
    FunctionCallArgumentsDelta(
        open_responses::ResponseFunctionCallArgumentsDeltaStreamingEvent,
    ),
    FunctionCallArgumentsDone(open_responses::ResponseFunctionCallArgumentsDoneStreamingEvent),
    ReasoningDelta(open_responses::ResponseReasoningDeltaStreamingEvent),
    ReasoningDone(open_responses::ResponseReasoningDoneStreamingEvent),
    ReasoningSummaryDelta(open_responses::ResponseReasoningSummaryDeltaStreamingEvent),
    ReasoningSummaryDone(open_responses::ResponseReasoningSummaryDoneStreamingEvent),
    ReasoningSummaryPartAdded(open_responses::ResponseReasoningSummaryPartAddedStreamingEvent),
    ReasoningSummaryPartDone(open_responses::ResponseReasoningSummaryPartDoneStreamingEvent),
    RefusalDelta(open_responses::ResponseRefusalDeltaStreamingEvent),
    RefusalDone(open_responses::ResponseRefusalDoneStreamingEvent),
    Completed(open_responses::ResponseCompletedStreamingEvent),
    Failed(open_responses::ResponseFailedStreamingEvent),
    Incomplete(open_responses::ResponseIncompleteStreamingEvent),
    Error(open_responses::ErrorStreamingEvent),
}

impl ResponseEvent {
    /// Returns the text delta if this is an `OutputTextDelta` event.
    pub fn as_text_delta(&self) -> Option<&str> {
        match self {
            ResponseEvent::OutputTextDelta(e) => Some(&e.delta),
            _ => None,
        }
    }

    /// Returns the function call arguments delta if this is a `FunctionCallArgumentsDelta` event.
    pub fn as_function_call_delta(&self) -> Option<&str> {
        match self {
            ResponseEvent::FunctionCallArgumentsDelta(e) => Some(&e.delta),
            _ => None,
        }
    }

    /// Returns the completed response if this is a `Completed` event.
    pub fn as_completed(&self) -> Option<&open_responses::ResponseResource> {
        match self {
            ResponseEvent::Completed(e) => Some(&e.response),
            _ => None,
        }
    }
}

fn parse_event(event_type: &str, data: &str) -> Result<ResponseEvent> {
    let event = match event_type {
        "response.created" => ResponseEvent::Created(serde_json::from_str(data)?),
        "response.in_progress" => ResponseEvent::InProgress(serde_json::from_str(data)?),
        "response.queued" => ResponseEvent::Queued(serde_json::from_str(data)?),
        "response.output_item.added" => {
            ResponseEvent::OutputItemAdded(serde_json::from_str(data)?)
        }
        "response.output_item.done" => ResponseEvent::OutputItemDone(serde_json::from_str(data)?),
        "response.content_part.added" => {
            ResponseEvent::ContentPartAdded(serde_json::from_str(data)?)
        }
        "response.content_part.done" => {
            ResponseEvent::ContentPartDone(serde_json::from_str(data)?)
        }
        "response.output_text.delta" => {
            ResponseEvent::OutputTextDelta(serde_json::from_str(data)?)
        }
        "response.output_text.done" => ResponseEvent::OutputTextDone(serde_json::from_str(data)?),
        "response.output_text.annotation.added" => {
            ResponseEvent::OutputTextAnnotationAdded(serde_json::from_str(data)?)
        }
        "response.function_call_arguments.delta" => {
            ResponseEvent::FunctionCallArgumentsDelta(serde_json::from_str(data)?)
        }
        "response.function_call_arguments.done" => {
            ResponseEvent::FunctionCallArgumentsDone(serde_json::from_str(data)?)
        }
        "response.reasoning.delta" => ResponseEvent::ReasoningDelta(serde_json::from_str(data)?),
        "response.reasoning.done" => ResponseEvent::ReasoningDone(serde_json::from_str(data)?),
        "response.reasoning_summary.delta" => {
            ResponseEvent::ReasoningSummaryDelta(serde_json::from_str(data)?)
        }
        "response.reasoning_summary.done" => {
            ResponseEvent::ReasoningSummaryDone(serde_json::from_str(data)?)
        }
        "response.reasoning_summary_part.added" => {
            ResponseEvent::ReasoningSummaryPartAdded(serde_json::from_str(data)?)
        }
        "response.reasoning_summary_part.done" => {
            ResponseEvent::ReasoningSummaryPartDone(serde_json::from_str(data)?)
        }
        "response.refusal.delta" => ResponseEvent::RefusalDelta(serde_json::from_str(data)?),
        "response.refusal.done" => ResponseEvent::RefusalDone(serde_json::from_str(data)?),
        "response.completed" => ResponseEvent::Completed(serde_json::from_str(data)?),
        "response.failed" => ResponseEvent::Failed(serde_json::from_str(data)?),
        "response.incomplete" => ResponseEvent::Incomplete(serde_json::from_str(data)?),
        "error" => ResponseEvent::Error(serde_json::from_str(data)?),
        other => return Err(Error::UnknownStreamEvent(other.to_string())),
    };

    Ok(event)
}

/// A streaming response from the Open Responses API.
pub struct ResponseStream<M> {
    stream: BodyDataStream<hyper::Response<hyper::body::Incoming>>,
    buffer: Vec<u8>,
    mode: M,
}

impl<M> ResponseStream<M> {
    pub fn new(response: Response<M>, mode: M) -> Self {
        let stream = response.inner.into_data_stream();
        Self {
            stream,
            buffer: Vec::new(),
            mode,
        }
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

impl Stream for ResponseStream<Async> {
    type Item = Result<ResponseEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            if let Some((event_type, data)) = extract_sse_frame(&mut this.buffer) {
                if data == "[DONE]" {
                    return Poll::Ready(None);
                }
                return Poll::Ready(Some(resolve_event_type(
                    event_type.as_deref(),
                    &data,
                )));
            }

            match this.stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => match item {
                    Ok(bytes) => this.buffer.extend_from_slice(&bytes),
                    Err(e) => return Poll::Ready(Some(Err(Error::Hyper(e)))),
                },
            }
        }
    }
}

impl Iterator for ResponseStream<Sync> {
    type Item = Result<ResponseEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let rt = self.mode.rt.clone();

        loop {
            if let Some((event_type, data)) = extract_sse_frame(&mut self.buffer) {
                if data == "[DONE]" {
                    return None;
                }
                return Some(resolve_event_type(event_type.as_deref(), &data));
            }

            match rt.block_on(self.stream.next()) {
                None => return None,
                Some(item) => match item {
                    Ok(bytes) => self.buffer.extend_from_slice(&bytes),
                    Err(e) => return Some(Err(Error::Hyper(e))),
                },
            }
        }
    }
}

impl<M> std::fmt::Debug for ResponseStream<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseStream").finish()
    }
}
