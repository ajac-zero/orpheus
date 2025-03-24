use either::Either;
use pyo3::{
    exceptions::{PyIndexError, PyValueError},
    prelude::*,
    types::{PyList, PyTuple},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Function {
    name: String,
    arguments: String,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    id: String,
    r#type: String,
    function: Function,
}

#[derive(Debug, IntoPyObject, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    #[pyo3(item)]
    url: String,
    #[pyo3(item)]
    detail: Option<String>,
}

impl<'py> FromPyObject<'py> for ImageUrl {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let url = ob.get_item("url")?.extract::<String>()?;
        let detail = ob
            .get_item("detail")
            .ok()
            .map(|x| x.extract::<String>())
            .transpose()?;

        Ok(Self { url, detail })
    }
}

#[derive(Debug, IntoPyObject, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text {
        #[pyo3(item)]
        text: String,
    },
    ImageUrl {
        #[pyo3(item)]
        image_url: ImageUrl,
    },
}

impl<'py> FromPyObject<'py> for Part {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let type_ = ob.get_item("type")?.extract::<String>()?;

        let part = match type_.as_str() {
            "text" => {
                let text = ob.get_item("text")?.extract::<String>()?;

                Part::Text { text }
            }
            "image_url" => {
                let image_url = ob.get_item("image_url")?.extract::<ImageUrl>()?;

                Part::ImageUrl { image_url }
            }
            unknown => return Err(PyValueError::new_err(format!("Unknown type: {unknown}"))),
        };

        Ok(part)
    }
}

#[derive(FromPyObject, IntoPyObject, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    #[pyo3(transparent)]
    Simple(String),
    #[pyo3(transparent)]
    Complex(Vec<Part>),
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    role: Option<String>,
    content: Option<Content>,
    refusal: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[pyclass]
#[skip_serializing_none]
#[derive(FromPyObject, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    #[pyo3(constructor=(content, role="system".into()))]
    System {
        #[pyo3(item)]
        content: String,
        #[pyo3(item)]
        role: String,
    },
    #[pyo3(constructor=(content, role="user".into()))]
    User {
        #[pyo3(item)]
        content: Content,
        #[pyo3(item)]
        role: String,
    },
    #[pyo3(constructor=(content=None, tool_calls=None, role="assistant".into()))]
    Assistant {
        #[pyo3(item)]
        content: Option<Content>,
        #[pyo3(item)]
        tool_calls: Option<Vec<ToolCall>>,
        #[pyo3(item)]
        role: String,
    },
    #[pyo3(constructor=(content, tool_id, role="tool".into()))]
    Tool {
        #[pyo3(item)]
        content: String,
        #[pyo3(item)]
        tool_id: String,
        #[pyo3(item)]
        role: String,
    },
}

#[pymethods]
impl Message {
    #[new]
    #[pyo3(signature = (role, content = None, tool_calls = None, tool_id = None))]
    fn new(
        role: String,
        content: Option<Content>,
        tool_calls: Option<Vec<ToolCall>>,
        tool_id: Option<String>,
    ) -> PyResult<Self> {
        let message = match role.as_str() {
            "system" => {
                if tool_calls.is_some() {
                    return Err(PyValueError::new_err(
                        "If role = 'user', tool_calls must be None",
                    ));
                };

                if tool_id.is_some() {
                    return Err(PyValueError::new_err(
                        "If role = 'user', tool_id must be None",
                    ));
                };

                let content = if let Some(content) = content {
                    content
                } else {
                    return Err(PyValueError::new_err(
                        "If role = 'user', content must not be None",
                    ));
                };

                let content = match content {
                    Content::Simple(content) => content,
                    Content::Complex(_) => {
                        return Err(PyValueError::new_err(
                            "If role = 'system', content must be str",
                        ))
                    }
                };

                Self::System { role, content }
            }
            "user" => {
                if tool_calls.is_some() {
                    return Err(PyValueError::new_err(
                        "If role = 'user', tool_calls must be None",
                    ));
                };

                if tool_id.is_some() {
                    return Err(PyValueError::new_err(
                        "If role = 'user', tool_id must be None",
                    ));
                };

                let content = if let Some(content) = content {
                    content
                } else {
                    return Err(PyValueError::new_err(
                        "If role = 'user', content must not be None",
                    ));
                };

                Self::User { role, content }
            }
            "assistant" => {
                if tool_id.is_some() {
                    return Err(PyValueError::new_err(
                        "If role = 'assistant', tool_id must be None",
                    ));
                };

                Self::Assistant {
                    role,
                    content,
                    tool_calls,
                }
            }
            "tool" => {
                let tool_id = if let Some(tool_id) = tool_id {
                    tool_id
                } else {
                    return Err(PyValueError::new_err(
                        "If role = 'tool', tool_id must not be None",
                    ));
                };

                let content = if let Some(content) = content {
                    content
                } else {
                    return Err(PyValueError::new_err(
                        "If role = 'user', content must not be None",
                    ));
                };

                let content = match content {
                    Content::Simple(content) => content,
                    Content::Complex(_) => {
                        return Err(PyValueError::new_err(
                            "If role = 'system', content must be str",
                        ))
                    }
                };

                Self::Tool {
                    role,
                    content,
                    tool_id,
                }
            }
            _ => {
                return Err(PyValueError::new_err(
                    "Invalid role; role can be one of 'user', 'assistant', 'system', 'tool'.",
                ))
            }
        };

        Ok(message)
    }

    #[getter]
    fn role(&self) -> &str {
        match self {
            Self::System { role, .. } => role,
            Self::User { role, .. } => role,
            Self::Assistant { role, .. } => role,
            Self::Tool { role, .. } => role,
        }
    }

    #[getter]
    fn content(&self) -> &str {
        match self {
            Self::System { content, .. } => content,
            Self::User { content, .. } => match content {
                Content::Simple(content) => content,
                Content::Complex(_) => todo!(),
            },
            Self::Assistant { content, .. } => match content {
                Some(content) => match content {
                    Content::Simple(content) => content,
                    Content::Complex(_) => todo!(),
                },
                None => "",
            },
            Self::Tool { content, .. } => content,
        }
    }
}

#[pyclass(frozen, sequence)]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Messages(Vec<Py<Message>>);

impl<'py> FromPyObject<'py> for Messages {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let py = ob.py();

        let messages = ob
            .extract::<Vec<Message>>()?
            .into_iter()
            .map(|x| Py::new(py, x).expect("bind to GIL"))
            .collect::<Vec<Py<Message>>>();

        Ok(Self(messages))
    }
}

#[pymethods]
impl Messages {
    #[new]
    #[pyo3(signature = (*args))]
    fn new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let messages = args.extract::<Vec<Py<Message>>>()?;

        Ok(Self(messages))
    }

    fn __len__(&self) -> usize {
        self.0.len()
    }

    fn __getitem__(&self, py: Python, index: usize) -> PyResult<Py<Message>> {
        let message = self
            .0
            .get(index)
            .ok_or_else(|| PyIndexError::new_err("Index out of range"))?;

        Ok(message.clone_ref(py))
    }
}

pub type EitherMessages = Either<Py<Messages>, Py<PyList>>;
