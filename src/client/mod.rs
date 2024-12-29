use isahc::http::request::Builder;
use std::collections::HashMap;

use super::types::{Completion, CompletionChunk, Prompt};
pub use blocking::Orpheus;
pub use non_blocking::AsyncOrpheus;

const BASE_URL_ENV: &str = "ORPHEUS_BASE_URL";
const API_KEY_ENV: &str = "ORPHEUS_API_KEY";

macro_rules! py_err {
    ( $($arg:expr),* ) => {
        pyo3::exceptions::PyValueError::new_err(format!($($arg),*))
    };
}

fn apply_header(builder: Builder, maybe_headers: Option<HashMap<String, String>>) -> Builder {
    if let Some(headers) = maybe_headers {
        headers
            .into_iter()
            .fold(builder, |builder, (k, v)| builder.header(k, v))
    } else {
        builder
    }
}

mod blocking {
    use super::*;
    use isahc::{HttpClient, ReadResponseExt, Request, Response};
    use pyo3::exceptions::PyStopIteration;
    use pyo3::prelude::*;
    use pyo3::types::PyDict;
    use pythonize::depythonize;
    use std::collections::VecDeque;
    use std::env;
    use std::io::Read;
    use url::Url;

    #[pyclass]
    pub struct Orpheus {
        client: HttpClient,
        base_url: Url,
        api_key: String,
        default_headers: Option<HashMap<String, String>>,
        default_query: Option<HashMap<String, String>>,
    }

    impl Orpheus {
        fn api_request(
            &self,
            path: &str,
            prompt: Prompt,
            extra_headers: Option<HashMap<String, String>>,
            extra_query: Option<HashMap<String, String>>,
        ) -> Result<Response<isahc::Body>, isahc::Error> {
            let mut url = self.base_url.to_owned();

            url.path_segments_mut()
                .expect("get path segments")
                .pop_if_empty()
                .extend(path.split('/').filter(|s| !s.is_empty()));

            if let Some(headers) = self.default_query.as_ref() {
                url.query_pairs_mut().extend_pairs(headers);
            };

            if let Some(headers) = extra_query {
                url.query_pairs_mut().extend_pairs(headers);
            };

            let builder = Request::builder()
                .method("POST")
                .uri(url.as_str())
                .header("content-type", "application/json")
                .header("api-key", &self.api_key);

            let builder = apply_header(builder, self.default_headers.to_owned());

            let builder = apply_header(builder, extra_headers);

            let body = serde_json::to_vec(&prompt).unwrap();

            let request = builder.body(body).unwrap();

            self.client.send(request)
        }
    }

    #[pymethods]
    impl Orpheus {
        #[new]
        #[pyo3(signature = (base_url=None, api_key=None, default_headers=None, default_query=None))]
        fn new(
            base_url: Option<String>,
            api_key: Option<String>,
            default_headers: Option<HashMap<String, String>>,
            default_query: Option<HashMap<String, String>>,
        ) -> PyResult<Self> {
            let client = HttpClient::new().expect("should create http client.");

            let base_url = base_url
                .or_else(|| env::var(BASE_URL_ENV).ok())
                .and_then(|s| Url::parse(&s).ok())
                .ok_or_else(|| py_err!("{} environment variable not found.", BASE_URL_ENV))?;

            let api_key = api_key
                .or_else(|| env::var(API_KEY_ENV).ok())
                .ok_or_else(|| py_err!("{} environment variable not found.", API_KEY_ENV))?;

            Ok(Self {
                client,
                base_url,
                api_key,
                default_headers,
                default_query,
            })
        }

        #[pyo3(signature = (extra_headers=None, extra_query=None, **py_kwargs))]
        fn create(
            &self,
            extra_headers: Option<HashMap<String, String>>,
            extra_query: Option<HashMap<String, String>>,
            py_kwargs: Option<&Bound<'_, PyDict>>,
        ) -> PyResult<CompletionResponse> {
            let args = py_kwargs.ok_or_else(|| py_err!("No keyword arguments passed."))?;

            let prompt = depythonize::<Prompt>(args).map_err(|e| py_err!("{}", e))?;

            let is_stream = prompt.is_stream();

            let mut response = self
                .api_request("/chat/completions", prompt, extra_headers, extra_query)
                .map_err(|e| py_err!("{}", e))?;

            if is_stream {
                let stream = Stream::new(response);

                Ok(CompletionResponse::Stream(stream))
            } else {
                let completion = response
                    .json::<Completion>()
                    .map_err(|e| py_err!("{}", e))?;

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

                    serde_json::from_str::<CompletionChunk>(data).map_err(|e| py_err!("{}", e))
                }
            } else {
                let chunk = self.body.read(&mut self.chunk);

                match chunk {
                    Ok(0) => Err(PyStopIteration::new_err("end of stream")),
                    Ok(_) => {
                        let chunk_str = std::str::from_utf8(&self.chunk)
                            .unwrap()
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
                    Err(e) => Err(py_err!("{}", e)),
                }
            }
        }

        fn __iter__(self_: PyRef<Self>) -> PyRef<Self> {
            self_
        }
    }
}

mod non_blocking {
    use super::*;
    use async_std::sync::Mutex;
    use futures_lite::AsyncReadExt;
    use isahc::{AsyncReadResponseExt, HttpClient, Request, Response};
    use pyo3::exceptions::PyStopAsyncIteration;
    use pyo3::prelude::*;
    use pythonize::depythonize;
    use std::collections::VecDeque;
    use std::env;
    use std::sync::Arc;
    use url::Url;

    #[pyclass]
    pub struct AsyncOrpheus {
        client: HttpClient,
        base_url: Url,
        api_key: String,
        default_headers: Option<HashMap<String, String>>,
        default_query: Option<HashMap<String, String>>,
    }

    impl AsyncOrpheus {
        async fn api_request(
            &self,
            path: &str,
            prompt: Prompt,
            extra_headers: Option<HashMap<String, String>>,
            extra_query: Option<HashMap<String, String>>,
        ) -> Result<Response<isahc::AsyncBody>, isahc::Error> {
            let mut url = self.base_url.to_owned();

            url.path_segments_mut()
                .expect("get path segments")
                .pop_if_empty()
                .extend(path.split('/').filter(|s| !s.is_empty()));

            if let Some(headers) = self.default_query.as_ref() {
                url.query_pairs_mut().extend_pairs(headers);
            };

            if let Some(headers) = extra_query {
                url.query_pairs_mut().extend_pairs(headers);
            };

            let builder = Request::builder()
                .method("POST")
                .uri(url.as_str())
                .header("content-type", "application/json")
                .header("api-key", &self.api_key);

            let builder = apply_header(builder, self.default_headers.to_owned());

            let builder = apply_header(builder, extra_headers);

            let body = serde_json::to_vec(&prompt).unwrap();

            let request = builder.body(body).unwrap();

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
            let client = HttpClient::new().expect("should create http client.");

            let base_url = base_url
                .or_else(|| env::var(BASE_URL_ENV).ok())
                .and_then(|s| Url::parse(&s).ok())
                .ok_or_else(|| py_err!("{} environment variable not found.", BASE_URL_ENV))?;

            let api_key = api_key
                .or_else(|| env::var(API_KEY_ENV).ok())
                .ok_or_else(|| py_err!("{} environment variable not found.", API_KEY_ENV))?;

            Ok(Self {
                client,
                base_url,
                api_key,
                default_headers,
                default_query,
            })
        }

        #[pyo3(signature = (extra_headers=None, extra_query=None, **py_kwargs))]
        async fn create(
            &self,
            extra_headers: Option<HashMap<String, String>>,
            extra_query: Option<HashMap<String, String>>,
            py_kwargs: Option<PyObject>,
        ) -> PyResult<CompletionResponse> {
            let args = py_kwargs.ok_or_else(|| py_err!("No keyword arguments passed."))?;

            let prompt = Python::with_gil(|py| depythonize::<Prompt>(&args.into_bound(py)))
                .map_err(|e| py_err!("{}", e))?;

            let is_stream = prompt.is_stream();

            let mut response = self
                .api_request("/chat/completions", prompt, extra_headers, extra_query)
                .await
                .map_err(|e| py_err!("{}", e))?;

            if is_stream {
                let stream = Stream::new(response);

                Ok(CompletionResponse::Stream(stream))
            } else {
                let completion = response
                    .json::<Completion>()
                    .await
                    .map_err(|e| py_err!("{}", e))?;

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
                        .map_err(|e| py_err!("{}", e));
                }
            } else {
                match stream.body.read(&mut chunk_slice).await {
                    Ok(_) => {
                        let chunk_str = std::str::from_utf8(&chunk_slice)
                            .unwrap()
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
                    Err(e) => return Err(py_err!("{}", e)),
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
}
