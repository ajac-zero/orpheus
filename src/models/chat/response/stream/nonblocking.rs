use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::Stream;

use crate::{Error, Result};

pub struct AsyncStream {
    stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    buffer: Vec<u8>,
}

impl From<reqwest::Response> for AsyncStream {
    fn from(value: reqwest::Response) -> Self {
        Self::new(value)
    }
}

impl AsyncStream {
    pub fn new(response: reqwest::Response) -> Self {
        let stream = Box::pin(response.bytes_stream());
        Self {
            stream,
            buffer: Vec::new(), // Initialize as Vec<u8>
        }
    }
}

impl Stream for AsyncStream {
    type Item = Result<super::ChatStreamChunk>;

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

                break Some(serde_json::from_str(json_str).map_err(Error::serde));
            }

            // No complete line found, need more data from stream
            match this.stream.as_mut().poll_next(cx) {
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

                        match serde_json::from_str::<super::ChatStreamChunk>(json_str) {
                            Ok(chunk) => return Poll::Ready(Some(Ok(chunk))),
                            Err(e) => {
                                return Poll::Ready(Some(Err(Error::serde(e))));
                            }
                        }
                    }
                }
                Poll::Ready(Some(item)) => match item {
                    Ok(bytes) => this.buffer.extend_from_slice(&bytes),
                    Err(e) => break Some(Err(Error::http(e))),
                },
            }
        };

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

impl Debug for AsyncStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncStream").finish()
    }
}
