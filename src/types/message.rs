use either::Either;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyList},
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

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    url: String,
    detail: Option<String>,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(FromPyObject, IntoPyObject, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    #[pyo3(transparent)]
    Simple(String),
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
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    System {
        role: String,
        content: String,
    },
    User {
        role: String,
        content: Content,
    },
    Assistant {
        role: String,
        content: Option<Content>,
        tool_calls: Option<Vec<ToolCall>>,
    },
    Tool {
        role: String,
        content: String,
        tool_id: String,
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
}

#[pyclass(frozen)]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Messages {
    messages: Vec<Py<Message>>,
}

#[pymethods]
impl Messages {
    #[new]
    fn new(messages: Vec<Py<Message>>) -> Self {
        Self { messages }
    }
}

pub type EitherMessage = Either<Message, PyDict>;
pub type EitherMessages = Either<Py<Messages>, Py<PyList>>;
