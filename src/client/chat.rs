use super::*;
use isahc::{HttpClient, Response};
use pyo3::exceptions::{PyStopIteration, PyValueError};
use pyo3::prelude::*;
use std::collections::VecDeque;
use std::io::Read;
use std::sync::Arc;

#[pyclass]
pub struct SyncChat {
    client: Arc<HttpClient>,
    url: Url,
    api_key: String,
}

#[pymethods]
impl SyncChat {
    #[pyo3(signature = (extra_headers=None, extra_query=None, **py_kwargs))]
    fn create(
        &self,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        py_kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<CompletionResponse> {
        let args = py_kwargs.ok_or(PyValueError::new_err("No keyword arguments passed."))?;

        let prompt = depythonize::<Prompt>(args)
            .map_err(|e| PyValueError::new_err(format!("Invalid arguments: {}", e)))?;

        let mut response = self
            .api_request("/chat/completions", &prompt, extra_headers, extra_query)
            .map_err(|e| PyIOError::new_err(format!("Failed to send request: {}", e)))?;

        if response.status() == 401 {
            return Err(PyIOError::new_err(
                "401 (Unauthorized) response; Is the API key valid?",
            ));
        };

        if prompt.is_stream() {
            let stream = Stream::new(response);

            Ok(CompletionResponse::Stream(stream))
        } else {
            let completion = response
                .json::<Completion>()
                .map_err(|e| PyValueError::new_err(format!("Failed to parse response: {}", e)))?;

            Ok(CompletionResponse::Completion(completion))
        }
    }

    #[getter]
    fn completions(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(pyo3::IntoPyObject)]
enum CompletionResponse {
    #[pyo3(transparent)]
    Completion(Completion),
    #[pyo3(transparent)]
    Stream(Stream),
}

#[pyclass]
struct Stream {
    buffer: String,
    body: isahc::Body,
    chunk: [u8; 1024],
    lines: VecDeque<String>,
}

impl Stream {
    fn new(response: Response<isahc::Body>) -> Self {
        Self {
            buffer: String::new(),
            body: response.into_body(),
            chunk: [0; 1024],
            lines: VecDeque::new(),
        }
    }
}

#[pymethods]
impl Stream {
    fn __next__(&mut self) -> PyResult<CompletionChunk> {
        if let Some(line) = self.lines.pop_front() {
            if line == "data: [DONE]" {
                Err(PyStopIteration::new_err("end of stream"))
            } else {
                let data = &line[6..];

                serde_json::from_str::<CompletionChunk>(data)
                    .map_err(|e| PyValueError::new_err(format!("Failed to parse chunk: {}", e)))
            }
        } else {
            let chunk = self.body.read(&mut self.chunk);

            match chunk {
                Ok(0) => Err(PyStopIteration::new_err("end of stream")),
                Ok(_) => {
                    let chunk_str = std::str::from_utf8(&self.chunk)
                        .expect("should convert chunk to string")
                        .trim_end_matches('\0');
                    self.buffer.push_str(chunk_str);

                    if self.buffer.ends_with("\n\n") {
                        self.lines = self
                            .buffer
                            .lines()
                            .filter(|l| !l.is_empty())
                            .map(|l| l.to_string())
                            .collect::<VecDeque<String>>();

                        self.buffer.clear();
                    };

                    self.__next__()
                }
                Err(e) => Err(PyValueError::new_err(format!(
                    "Failed to read chunk: {}",
                    e
                ))),
            }
        }
    }

    fn __iter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}
