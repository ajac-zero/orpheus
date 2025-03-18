use pyo3::{exceptions::PyValueError, prelude::*};
use serde::{Deserialize, Serialize};

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
struct ImageUrl {
    url: String,
    detail: Option<String>,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Part {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(FromPyObject, IntoPyObject, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Content {
    #[pyo3(transparent)]
    Simple(String),
    Complex(Vec<Part>),
}

#[pyclass(get_all)]
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    role: String,
    content: Option<Content>,
    tool_id: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[pymethods]
impl Message {
    #[new]
    #[pyo3(signature = (role, content=None, tool_calls=None, tool_id=None))]
    fn __init__(
        role: String,
        content: Option<Content>,
        tool_calls: Option<Vec<ToolCall>>,
        tool_id: Option<String>,
    ) -> PyResult<Self> {
        match role.as_str() {
            "user" | "system" | "developer" => {
                if tool_calls.is_some() {
                    return Err(PyValueError::new_err("If role = 'user' | 'system' | 'developer', tool_calls must be None"))
                };

                if tool_id.is_some() {
                    return Err(PyValueError::new_err("If role = 'user' | 'system' | 'developer', tool_id must be None"))
                };

                if content.is_none() {
                    return Err(PyValueError::new_err("If role = 'user' | 'system' | 'developer', content must not be None"))
                };

                Ok(Self {
                    role,
                    content,
                    tool_id: None,
                    tool_calls: None,
                })
            }
            "assistant" => {
                if tool_id.is_some() {
                    return Err(PyValueError::new_err("If role = 'assistant', tool_id must be None"))
                };

                Ok(Self{
                    role,
                    content,
                    tool_id: None,
                    tool_calls,
                })
            }
            "tool" => {
                if tool_id.is_none() {
                    return Err(PyValueError::new_err("If role = 'tool', tool_id must not be None"))
                };

                Ok(Self{
                    role,
                    content,
                    tool_id: None,
                    tool_calls,
                })
            }
            _ => Err(PyValueError::new_err("Invalid role; role can be one of 'user', 'assistant', 'developer', 'system', 'tool'."))
        }
    }
}

#[pyclass(sequence)]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Conversation(Vec<Message>);

#[pymethods]
impl Conversation {
    #[new]
    fn new(messages: Vec<Message>) -> Self {
        Self(messages)
    }
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    role: Option<String>,
    content: Option<Content>,
    refusal: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}
