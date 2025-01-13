use super::*;
use async_std::sync::Mutex;
use futures_lite::AsyncReadExt;
use isahc::{AsyncReadResponseExt, HttpClient, Response};
use pyo3::exceptions::{PyStopAsyncIteration, PyValueError};
use pyo3::prelude::*;
use pythonize::depythonize;
use std::collections::VecDeque;
use std::sync::Arc;
use url::Url;

#[pyclass]
pub struct AsyncOrpheus {
    client: HttpClient,
    base_url: Url,
    api_key: String,
}

impl AsyncOrpheus {
    async fn api_request(
        &self,
        path: &str,
        prompt: &Prompt,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
    ) -> Result<Response<isahc::AsyncBody>, isahc::Error> {
        let request = build_request(
            path,
            prompt,
            &self.base_url,
            &self.api_key,
            extra_headers,
            extra_query,
        );

        self.client.send_async(request).await
    }
}

#[pymethods]
impl AsyncOrpheus {
    #[new]
    #[pyo3(signature = (base_url=None, api_key=None, default_headers=None, default_query=None))]
    fn new(
        base_url: Option<String>,
        api_key: Option<String>,
        default_headers: Option<HashMap<String, String>>,
        default_query: Option<HashMap<String, String>>,
    ) -> PyResult<Self> {
        let (client, base_url, api_key) = new(base_url, api_key, default_headers, default_query);

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    #[pyo3(signature = (extra_headers=None, extra_query=None, **py_kwargs))]
    async fn create(
        &self,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
        py_kwargs: Option<PyObject>,
    ) -> PyResult<CompletionResponse> {
        let args = py_kwargs.ok_or(PyValueError::new_err("No keyword arguments passed."))?;

        let prompt = Python::with_gil(|py| depythonize::<Prompt>(&args.into_bound(py)))
            .map_err(|e| PyValueError::new_err(format!("Invalid arguments: {}", e)))?;

        let mut response = self
            .api_request("/chat/completions", &prompt, extra_headers, extra_query)
            .await
            .map_err(|e| PyValueError::new_err(format!("Failed to send request: {}", e)))?;

        if prompt.is_stream() {
            let stream = Stream::new(response);

            Ok(CompletionResponse::Stream(stream))
        } else {
            let completion = response
                .json::<Completion>()
                .await
                .map_err(|e| PyValueError::new_err(format!("Failed to parse response: {}", e)))?;

            Ok(CompletionResponse::Completion(completion))
        }
    }

    #[getter]
    fn chat(self_: PyRef<Self>) -> PyRef<Self> {
        self_
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

struct InnerStream {
    buffer: String,
    body: isahc::AsyncBody,
    lines: VecDeque<String>,
}

#[pyclass]
struct Stream {
    stream: Arc<Mutex<InnerStream>>,
}

async fn next(stream: Arc<Mutex<InnerStream>>) -> Result<CompletionChunk, PyErr> {
    let mut stream = stream.lock().await;
    let mut chunk_slice = [0; 1024];
    loop {
        if let Some(line) = stream.lines.pop_front() {
            if line == "data: [DONE]" {
                return Err(PyStopAsyncIteration::new_err("end of stream"));
            } else {
                let data = &line[6..];

                return serde_json::from_str::<CompletionChunk>(data)
                    .map_err(|e| PyValueError::new_err(format!("Failed to parse chunk: {}", e)));
            }
        } else {
            match stream.body.read(&mut chunk_slice).await {
                Ok(_) => {
                    let chunk_str = std::str::from_utf8(&chunk_slice)
                        .expect("should convert chunk to string")
                        .trim_end_matches('\0');
                    stream.buffer.push_str(chunk_str);

                    if let Some(position) = stream.buffer.find("\n\n") {
                        let split_point = position + 2;
                        let moved = stream.buffer.drain(..split_point).collect::<String>();
                        let new_lines = moved
                            .lines()
                            .filter(|l| !l.is_empty())
                            .map(|l| l.to_string());

                        stream.lines.extend(new_lines);
                    };

                    continue;
                }
                Err(e) => {
                    return Err(PyValueError::new_err(format!(
                        "Failed to read chunk: {}",
                        e
                    )))
                }
            }
        }
    }
}

impl Stream {
    fn new(response: Response<isahc::AsyncBody>) -> Self {
        let inner = InnerStream {
            buffer: String::new(),
            body: response.into_body(),
            lines: VecDeque::new(),
        };

        Self {
            stream: Arc::new(Mutex::new(inner)),
        }
    }
}

#[pymethods]
impl Stream {
    fn __anext__<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let stream = self.stream.clone();
        pyo3_async_runtimes::async_std::future_into_py(py, next(stream))
    }

    fn __aiter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}
