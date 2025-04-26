use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyList},
};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use smallvec::SmallVec;

use crate::models::ArbitraryDict;

#[pyclass(get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    name: String,
    arguments: ArbitraryDict,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolCall {
    Function { id: String, function: Function },
}

#[pymethods]
impl ToolCall {
    #[new]
    fn pynew(id: String, name: String, arguments: &Bound<'_, PyDict>) -> PyResult<Self> {
        Ok(Self::Function {
            id,
            function: Function {
                name,
                arguments: arguments.extract()?,
            },
        })
    }
}

#[pyclass(frozen)]
#[derive(Debug, Deserialize)]
pub enum Part {
    #[pyo3(constructor=(text))]
    Text { text: String },
    #[pyo3(constructor=(url, detail=None))]
    Image { url: String, detail: Option<String> },
}

impl<'py> FromPyObject<'py> for Part {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let part_type = ob.get_item("type")?.extract::<String>()?;

        let part = match part_type.as_str() {
            "text" => Part::Text {
                text: ob.get_item("text")?.extract::<String>()?,
            },
            "image_url" => {
                let image_url = ob.get_item("image_url")?;

                Part::Image {
                    url: image_url.get_item("url")?.extract::<String>()?,
                    detail: image_url
                        .get_item("detail")
                        .and_then(|item| item.extract::<String>())
                        .map_or(None, Some),
                }
            }
            unknown => return Err(PyValueError::new_err(format!("Unknown type: {unknown}"))),
        };

        Ok(part)
    }
}

#[derive(Debug, Serialize)]
struct ImageUrl<'a> {
    url: &'a String,
    detail: &'a Option<String>,
}

impl Serialize for Part {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Part::Text { text } => {
                let mut state = serializer.serialize_struct("text", 1)?;
                state.serialize_field("type", "text")?;
                state.serialize_field("text", text)?;
                state.end()
            }
            Part::Image { url, detail } => {
                let mut state = serializer.serialize_struct("image_url", 1)?;
                state.serialize_field("type", "image_url")?;
                let image_url = ImageUrl { url, detail };
                state.serialize_field("image_url", &image_url)?;
                state.end()
            }
        }
    }
}

#[derive(Debug, IntoPyObject, FromPyObject, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    #[pyo3(transparent)]
    Simple(String),
    #[pyo3(transparent)]
    Complex(Parts),
}

impl Clone for Content {
    fn clone(&self) -> Self {
        match self {
            Content::Simple(string) => Content::Simple(string.clone()),
            Content::Complex(parts) => Python::with_gil(|py| {
                Content::Complex(Parts(
                    parts.0.iter().map(|item| item.clone_ref(py)).collect(),
                ))
            }),
        }
    }
}

type PartsArray = SmallVec<[Py<Part>; 4]>;

#[pyclass(sequence)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Parts(PartsArray);

#[pymethods]
impl Parts {
    fn __len__(&self) -> usize {
        self.0.len()
    }

    fn __getitem__(&self, py: Python, index: usize) -> PyResult<Py<Part>> {
        self.0
            .get(index)
            .ok_or(PyValueError::new_err("Index is out of range."))
            .map(|part_ref| part_ref.clone_ref(py))
    }
}

impl<'py> FromPyObject<'py> for Parts {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.extract::<PartsArray>()
            .or_else(|_| {
                ob.downcast::<PyList>()
                    .map_err(|_| {
                        PyValueError::new_err("Expected list[Messages] | list[MappedMessages]")
                    })
                    .and_then(|list| {
                        let py = list.py();
                        list.iter()
                            .map(|item| item.extract::<Part>().and_then(|msg| Py::new(py, msg)))
                            .collect::<PyResult<PartsArray>>()
                    })
            })
            .map(Self)
    }
}

#[pyclass(get_all)]
#[derive(Debug, Deserialize)]
pub struct Delta {
    role: Option<String>,
    content: Option<Content>,
    refusal: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[pyclass(frozen, get_all)]
#[serde_with::skip_serializing_none]
#[derive(Debug, FromPyObject, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum Message {
    #[pyo3(constructor=(content))]
    System {
        #[pyo3(item)]
        content: String,
    },
    #[pyo3(constructor=(content))]
    User {
        #[pyo3(item)]
        content: Content,
    },
    #[pyo3(constructor=(content=None, tool_calls=None))]
    Assistant {
        #[pyo3(item)]
        content: Option<Content>,
        #[pyo3(item)]
        tool_calls: Option<Vec<ToolCall>>,
    },
    #[pyo3(constructor=(content, tool_id))]
    Tool {
        #[pyo3(item)]
        content: String,
        #[pyo3(item)]
        tool_id: String,
    },
}

#[pymethods]
impl Message {
    #[getter]
    fn role(&self) -> &str {
        match self {
            Self::System { .. } => "system",
            Self::User { .. } => "user",
            Self::Assistant { .. } => "assistant",
            Self::Tool { .. } => "tool",
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

    #[getter]
    fn tool_calls(&self) -> Option<Vec<ToolCall>> {
        match self {
            Self::Assistant { tool_calls, .. } => tool_calls.clone(),
            _ => todo!(),
        }
    }
}

type MessageArray = SmallVec<[Py<Message>; 24]>;

#[derive(Debug, Serialize)]
pub struct Messages(MessageArray);

impl<'py> FromPyObject<'py> for Messages {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.extract::<MessageArray>()
            .or_else(|_| {
                ob.downcast::<PyList>()
                    .map_err(|_| {
                        PyValueError::new_err("Expected list[Messages] | list[MappedMessages]")
                    })
                    .and_then(|list| {
                        let py = list.py();
                        list.iter()
                            .map(|item| item.extract::<Message>().and_then(|msg| Py::new(py, msg)))
                            .collect::<PyResult<MessageArray>>()
                    })
            })
            .map(Self)
    }
}
