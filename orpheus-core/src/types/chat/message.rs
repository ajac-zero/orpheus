use either::Either;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyList;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

impl<'py> FromPyObject<'py> for Part {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let part_type = ob.get_item("type")?.extract::<String>()?;

        let part = match part_type.as_str() {
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

const SYSTEM_ROLE: &str = "system";
const USER_ROLE: &str = "user";
const ASSISTANT_ROLE: &str = "assistant";
const TOOL_ROLE: &str = "tool";

#[pyclass(frozen, get_all)]
#[serde_with::skip_serializing_none]
#[derive(FromPyObject, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    #[pyo3(constructor=(content, role=SYSTEM_ROLE.into()))]
    System {
        #[pyo3(item)]
        content: String,
        #[pyo3(item)]
        role: String,
    },
    #[pyo3(constructor=(content, role=USER_ROLE.into()))]
    User {
        #[pyo3(item)]
        content: Content,
        #[pyo3(item)]
        role: String,
    },
    #[pyo3(constructor=(content=None, tool_calls=None, role=ASSISTANT_ROLE.into()))]
    Assistant {
        #[pyo3(item)]
        content: Option<Content>,
        #[pyo3(item)]
        tool_calls: Option<Vec<ToolCall>>,
        #[pyo3(item)]
        role: String,
    },
    #[pyo3(constructor=(content, tool_id, role=TOOL_ROLE.into()))]
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

const MESSAGES_LIMIT: usize = 24;

pub type Messages = SmallVec<[Py<Message>; MESSAGES_LIMIT]>;

pub type EitherMessages = Either<Messages, Py<PyList>>;
