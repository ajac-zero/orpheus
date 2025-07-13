use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Simple(String),
    Complex(Vec<Part>),
}

impl From<Vec<Part>> for Content {
    fn from(value: Vec<Part>) -> Self {
        Self::Complex(value)
    }
}

impl Content {
    pub fn simple(content: impl Into<String>) -> Self {
        Content::Simple(content.into())
    }

    /// Consumes the current content and creates a new content with the appended part.
    /// 1. `Self::Simple` variant is transformed into a complex variant with the original text prepended as a "text" part.
    /// 2. `Self::Complex` variant is modified by appending the new part to the existing parts vector.
    pub fn add_part(self, part: Part) -> Self {
        let new_parts = match self {
            Self::Simple(string) => vec![Part::text(string), part],
            Self::Complex(mut parts) => {
                parts.push(part);
                parts
            }
        };
        Content::Complex(new_parts)
    }

    pub fn to_string(&self) -> String {
        match self {
            Content::Simple(s) => s.clone(),
            Content::Complex(_) => todo!(),
        }
    }
}

impl From<String> for Content {
    fn from(string: String) -> Self {
        Content::Simple(string)
    }
}

impl<'a> From<&'a str> for Content {
    fn from(s: &'a str) -> Self {
        Content::Simple(s.to_string())
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Simple(s) => write!(f, "{}", s),
            Content::Complex(v) => v.iter().try_for_each(|p| write!(f, "{}", p)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    url: String,
    detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    filename: String,
    file_data: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
    File { file: File },
}

impl Part {
    pub fn text(string: String) -> Self {
        Self::Text { text: string }
    }

    pub fn image_url(url: String, detail: Option<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl { url, detail },
        }
    }

    pub fn file(filename: String, data: String) -> Self {
        Self::File {
            file: File {
                filename,
                file_data: data,
            },
        }
    }
}

impl std::fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Part::Text { text } => write!(f, "{}", text),
            Part::ImageUrl { image_url } => write!(f, "{}", format!("[Url: {}]", image_url.url)),
            Part::File { file } => write!(f, "{}", format!("[File: {}]", file.filename)),
        }
    }
}
