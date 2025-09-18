mod chunk;

use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub use chunk::ChatStreamChunk;
use futures_lite::{Stream, StreamExt};
use http_body_util::{BodyDataStream, BodyExt};

use crate::{
    Error, Result,
    client::{
        handler::Response,
        mode::{Async, Sync},
    },
};

pub struct ChatStream<M> {
    stream: BodyDataStream<hyper::Response<hyper::body::Incoming>>,
    buffer: Vec<u8>,
    #[cfg(feature = "otel")]
    pub(crate) aggregator: crate::otel::StreamAggregator,
    mode: M,
}

impl<M> ChatStream<M> {
    pub fn new(response: Response<M>, mode: M) -> Self {
        let stream = response.inner.into_data_stream();
        Self {
            stream,
            buffer: Vec::new(), // Initialize as Vec<u8>
            #[cfg(feature = "otel")]
            aggregator: crate::otel::StreamAggregator::default(),
            mode,
        }
    }
}

impl Stream for ChatStream<Async> {
    type Item = Result<ChatStreamChunk>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        let result = loop {
            // First, try to extract a complete line from existing buffer
            if let Some(line_bytes) = extract_line(&mut this.buffer) {
                let line = String::from_utf8_lossy(&line_bytes);
                let line = line.trim();

                // Skip empty lines and comments
                if line.is_empty() || line.starts_with(":") {
                    continue;
                }

                // Validate SSE format
                if !line.starts_with("data: ") {
                    break Some(Err(Error::invalid_sse(line)));
                }

                let json_str = &line[6..]; // Remove "data: " prefix
                if json_str == "[DONE]" {
                    break None;
                }

                break Some(serde_json::from_str(json_str).map_err(Error::Serde));
            }

            // No complete line found, need more data from stream
            match this.stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    // Stream ended - check if we have remaining data
                    if this.buffer.is_empty() {
                        return Poll::Ready(None);
                    } else {
                        // Process final incomplete line
                        let line_clone = this.buffer.clone();
                        let line = String::from_utf8_lossy(&line_clone);
                        this.buffer.clear();
                        let line = line.trim();

                        if line.is_empty() || line.starts_with(":") {
                            return Poll::Ready(None);
                        }

                        if !line.starts_with("data: ") {
                            return Poll::Ready(Some(Err(Error::invalid_sse(line))));
                        }

                        let json_str = &line[6..];
                        if json_str == "[DONE]" {
                            return Poll::Ready(None);
                        }

                        match serde_json::from_str::<ChatStreamChunk>(json_str) {
                            Ok(chunk) => return Poll::Ready(Some(Ok(chunk))),
                            Err(e) => {
                                return Poll::Ready(Some(Err(Error::Serde(e))));
                            }
                        }
                    }
                }
                Poll::Ready(Some(item)) => match item {
                    Ok(bytes) => this.buffer.extend_from_slice(&bytes),
                    Err(e) => break Some(Err(Error::Hyper(e))),
                },
            }
        };

        #[cfg(feature = "otel")]
        if let Some(Ok(ref chunk)) = result {
            this.aggregator.aggregate_chunk(chunk);
        }

        Poll::Ready(result)
    }
}

// Helper function to extract a complete line from buffer
fn extract_line(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    // Look for newline
    if let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
        // Extract the line including the newline
        let mut line: Vec<u8> = buffer.drain(0..=newline_pos).collect();

        // Remove the newline
        line.pop();

        // Remove carriage return if present (for \r\n line endings)
        if line.last() == Some(&b'\r') {
            line.pop();
        }

        Some(line)
    } else {
        None
    }
}

impl Iterator for ChatStream<Sync> {
    type Item = Result<ChatStreamChunk>;

    fn next(&mut self) -> Option<Self::Item> {
        let rt = self.mode.rt.clone();

        let result = loop {
            // First, try to extract a complete line from existing buffer
            if let Some(line_bytes) = extract_line(&mut self.buffer) {
                let line = String::from_utf8_lossy(&line_bytes);
                let line = line.trim();

                // Skip empty lines and comments
                if line.is_empty() || line.starts_with(":") {
                    continue;
                }

                // Validate SSE format
                if !line.starts_with("data: ") {
                    break Some(Err(Error::invalid_sse(line)));
                }

                let json_str = &line[6..]; // Remove "data: " prefix
                if json_str == "[DONE]" {
                    break None;
                }

                break Some(serde_json::from_str(json_str).map_err(Error::Serde));
            }

            // No complete line found, need more data from stream
            match rt.block_on(self.stream.next()) {
                None => {
                    // Stream ended - check if we have remaining data
                    if self.buffer.is_empty() {
                        return None;
                    } else {
                        // Process final incomplete line
                        let line_clone = self.buffer.clone();
                        let line = String::from_utf8_lossy(&line_clone);
                        self.buffer.clear();
                        let line = line.trim();

                        if line.is_empty() || line.starts_with(":") {
                            return None;
                        }

                        if !line.starts_with("data: ") {
                            return Some(Err(Error::invalid_sse(line)));
                        }

                        let json_str = &line[6..];
                        if json_str == "[DONE]" {
                            return None;
                        }

                        return Some(
                            serde_json::from_str::<ChatStreamChunk>(json_str).map_err(Error::Serde),
                        );
                    }
                }
                Some(item) => match item {
                    Ok(bytes) => self.buffer.extend_from_slice(&bytes),
                    Err(e) => break Some(Err(Error::Hyper(e))),
                },
            }
        };

        #[cfg(feature = "otel")]
        if let Some(Ok(ref chunk)) = result {
            self.aggregator.aggregate_chunk(chunk);
        }

        result
    }
}

impl<M> std::fmt::Debug for ChatStream<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatStream").finish()
    }
}
