use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Simple(String),
    Complex(Vec<Part>),
}

impl std::ops::Add for Content {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Simple(s1), Self::Simple(s2)) => Self::Simple(s1 + &s2),
            (Self::Simple(s), Self::Complex(parts)) => {
                let mut result = vec![Part::text(s)];
                result.extend(parts);
                Self::Complex(result)
            }
            (Self::Complex(mut parts), Self::Simple(s)) => {
                parts.push(Part::text(s));
                Self::Complex(parts)
            }
            (Self::Complex(mut parts1), Self::Complex(parts2)) => {
                parts1.extend(parts2);
                Self::Complex(parts1)
            }
        }
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_simple_to_simple() {
        let content1 = Content::Simple("Hello ".to_string());
        let content2 = Content::Simple("World!".to_string());
        let result = content1 + content2;

        assert_eq!(result, Content::Simple("Hello World!".to_string()));
    }

    #[test]
    fn test_add_simple_to_complex() {
        let content1 = Content::Simple("Hello".to_string());
        let content2 = Content::Complex(vec![
            Part::text("World".to_string()),
            Part::image_url("http://example.com/image.jpg".to_string(), None),
        ]);
        let result = content1 + content2;

        let expected = Content::Complex(vec![
            Part::text("Hello".to_string()),
            Part::text("World".to_string()),
            Part::image_url("http://example.com/image.jpg".to_string(), None),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_add_complex_to_simple() {
        let content1 = Content::Complex(vec![
            Part::text("Hello".to_string()),
            Part::file("test.txt".to_string(), "content".to_string()),
        ]);
        let content2 = Content::Simple(" World!".to_string());
        let result = content1 + content2;

        let expected = Content::Complex(vec![
            Part::text("Hello".to_string()),
            Part::file("test.txt".to_string(), "content".to_string()),
            Part::text(" World!".to_string()),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_add_complex_to_complex() {
        let content1 = Content::Complex(vec![
            Part::text("First".to_string()),
            Part::image_url(
                "http://example.com/1.jpg".to_string(),
                Some("high".to_string()),
            ),
        ]);
        let content2 = Content::Complex(vec![
            Part::text("Second".to_string()),
            Part::file("data.json".to_string(), "{}".to_string()),
        ]);
        let result = content1 + content2;

        let expected = Content::Complex(vec![
            Part::text("First".to_string()),
            Part::image_url(
                "http://example.com/1.jpg".to_string(),
                Some("high".to_string()),
            ),
            Part::text("Second".to_string()),
            Part::file("data.json".to_string(), "{}".to_string()),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_add_part_to_simple() {
        let content = Content::Simple("Hello".to_string());
        let part = Part::image_url("http://example.com/image.jpg".to_string(), None);
        let result = content.add_part(part);

        let expected = Content::Complex(vec![
            Part::text("Hello".to_string()),
            Part::image_url("http://example.com/image.jpg".to_string(), None),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_add_part_to_complex() {
        let content = Content::Complex(vec![Part::text("Existing".to_string())]);
        let part = Part::file("test.txt".to_string(), "data".to_string());
        let result = content.add_part(part);

        let expected = Content::Complex(vec![
            Part::text("Existing".to_string()),
            Part::file("test.txt".to_string(), "data".to_string()),
        ]);
        assert_eq!(result, expected);
    }
}
