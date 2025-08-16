use std::io::{BufRead, BufReader};

use crate::{Error, Result};

#[derive(Debug)]
pub struct ChatStream {
    reader: BufReader<reqwest::blocking::Response>,
    #[cfg(feature = "otel")]
    pub(crate) aggregator: super::otel::StreamAggregator,
}

impl ChatStream {
    pub fn new(response: reqwest::blocking::Response) -> Self {
        let reader = BufReader::new(response);
        Self {
            reader,
            #[cfg(feature = "otel")]
            aggregator: super::otel::StreamAggregator::default(),
        }
    }
}

impl Iterator for ChatStream {
    type Item = Result<super::ChatStreamChunk>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();

        let item = loop {
            line.clear();

            let bytes_read = match self.reader.read_line(&mut line) {
                Ok(bytes_read) => bytes_read,
                Err(e) => break Err(Error::io(e)),
            };

            if bytes_read == 0 {
                return None; // Stream is empty
            }

            let line = line.trim();
            if line.is_empty() || line.starts_with(":") {
                continue; // Skip comments/keepalives
            }

            if !line.starts_with("data: ") {
                break Err(Error::invalid_sse(line));
            }

            let json_str = &line[6..]; // Remove "data: " prefix and trailing whitespace

            if json_str == "[DONE]" {
                return None; // Stream is explicitly over
            }

            let chunk = match serde_json::from_str(json_str) {
                Ok(chunk) => chunk,
                Err(e) => break Err(Error::serde(e)),
            };

            break Ok(chunk);
        };

        #[cfg(feature = "otel")]
        if let Ok(ref chunk) = item {
            self.aggregator.aggregate_chunk(chunk);
        }

        Some(item)
    }
}
