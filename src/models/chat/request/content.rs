use serde::{Deserialize, Serialize};

/// Represents the content of a message, supporting both simple text and complex multimodal content.
///
/// Content can be either a simple string or a complex structure containing multiple parts
/// including text, images, and files. The enum automatically handles conversions between
/// simple and complex formats as multimodal elements are added.
///
/// # Examples
///
/// ```
/// use orpheus::models::chat::{Content, Part};
///
/// // Simple text content
/// let simple = Content::simple("Hello, world!");
///
/// // Complex multimodal content
/// let complex = Content::simple("Analyze this data:")
///     .add_part(Part::image_url("https://example.com/chart.png".to_string(), None))
///     .add_part(Part::file("manual.pdf".into(), "[pdf data]".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    /// Simple text content as a string
    Simple(String),
    /// Complex content containing multiple parts (text, images, files)
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
    /// Creates a new simple text content.
    ///
    /// This is the preferred way to create text-only content that may later
    /// be extended with multimodal elements using [`Content::add_part`].
    ///
    /// # Arguments
    ///
    /// * `content` - The text content
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::models::chat::Content;
    ///
    /// let content = Content::simple("Hello, world!");
    /// let from_string = Content::simple(String::from("Hello"));
    /// ```
    pub fn simple(content: impl Into<String>) -> Self {
        Content::Simple(content.into())
    }

    /// Consumes the current content and creates new content with the appended part.
    ///
    /// This method automatically handles the conversion from simple to complex content:
    /// - `Simple` content is converted to `Complex` with the original text as the first part
    /// - `Complex` content has the new part appended to the existing parts vector
    ///
    /// # Arguments
    ///
    /// * `part` - The part to add (text, image, or file)
    ///
    /// # Returns
    ///
    /// A new `Content` instance with the part added
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::models::chat::{Content, Part};
    ///
    /// // Single part addition
    /// let content = Content::simple("Look at this:")
    ///     .add_part(Part::image_url("https://example.com/image.jpg".to_string(), None));
    ///
    /// // Multiple parts
    /// let multimodal = Content::simple("Analysis request:")
    ///     .add_part(Part::file("data.csv".to_string(), "csv content".to_string()))
    ///     .add_part(Part::image_url("https://example.com/chart.png".to_string(), Some("high".to_string())));
    /// ```
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

/// Represents an image URL with optional detail level for processing.
///
/// The detail level affects how the AI model processes the image:
/// - `None`: Default resolution and processing
/// - `Some("low")`: Lower resolution, faster processing
/// - `Some("high")`: Higher resolution, more detailed analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    /// The URL of the image to process
    url: String,
    /// Optional detail level for image processing ("low", "high", or None for default)
    detail: Option<String>,
}

/// Represents a file with its name and content data.
///
/// Files can contain various types of data including PDFs, text documents,
/// CSV data, JSON, code files, and other structured or unstructured data.
/// The content should be provided as a string representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    /// The name of the file (used for context and identification)
    filename: String,
    /// The content of the file as a string
    file_data: String,
}

/// Represents audio input with base64-encoded data and format specification.
///
/// Audio input allows AI models to process and understand audio content
/// such as speech, music, or other audio signals. The audio data must be
/// base64-encoded and include the appropriate format specification.
///
/// # Examples
///
/// ```
/// use orpheus::models::chat::Part;
///
/// // Create audio input through Part
/// let audio_part = Part::input_audio(
///     "base64_encoded_audio_data".to_string(),
///     "wav".to_string()
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputAudio {
    /// Base64-encoded audio data
    data: String, // must be base64 encoded
    /// Audio format ("wav" | "mp3")
    format: String,
}

/// Represents a single part of multimodal content.
///
/// Parts are the building blocks of complex content, allowing messages to contain
/// a mix of text, images, and files. Each part is serialized with a type tag
/// to distinguish between different content types.
///
/// # Examples
///
/// ```
/// use orpheus::models::chat::Part;
///
/// // Text part
/// let text_part = Part::text("Hello, world!".to_string());
///
/// // Image part
/// let image_part = Part::image_url(
///     "https://example.com/image.jpg".to_string(),
///     Some("high".to_string())
/// );
///
/// // File part
/// let file_part = Part::file("data.csv".to_string(), "name,age\nAlice,25".to_string());
///
/// // Audio part
/// let audio_part = Part::input_audio("base64_audio_data".to_string(), "wav".to_string());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    /// Text content part
    Text { text: String },
    /// Image URL part with optional detail level
    ImageUrl { image_url: ImageUrl },
    /// File part with filename and content data
    File { file: File },

    /// Audio input part with base64-encoded data and format
    InputAudio { input_audio: InputAudio },
}

impl Part {
    /// Creates a new text part.
    ///
    /// # Arguments
    ///
    /// * `string` - The text content
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::models::chat::Part;
    ///
    /// let part = Part::text("Hello, world!".to_string());
    /// ```
    pub fn text(string: String) -> Self {
        Self::Text { text: string }
    }

    /// Creates a new image URL part with optional detail level.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the image
    /// * `detail` - Optional detail level for processing:
    ///   - `None` - Default resolution
    ///   - `Some("low")` - Lower resolution, faster processing
    ///   - `Some("high")` - Higher resolution, more detailed analysis
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::models::chat::Part;
    ///
    /// // Basic image
    /// let image = Part::image_url("https://example.com/photo.jpg".to_string(), None);
    ///
    /// // High detail image
    /// let detailed = Part::image_url(
    ///     "https://example.com/chart.png".to_string(),
    ///     Some("high".to_string())
    /// );
    /// ```
    pub fn image_url(url: String, detail: Option<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl { url, detail },
        }
    }

    /// Creates a new file part with filename and content data.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file (used for context)
    /// * `data` - The content of the file as a string
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::models::chat::Part;
    ///
    /// // PDF content
    /// let pdf = Part::file("report.pdf".to_string(), "PDF content here...".to_string());
    /// ```
    pub fn file(filename: String, data: String) -> Self {
        Self::File {
            file: File {
                filename,
                file_data: data,
            },
        }
    }

    /// Creates a new audio input part with base64-encoded data and format.
    ///
    /// # Arguments
    ///
    /// * `data` - Base64-encoded audio data
    /// * `format` - Audio format (e.g., "wav", "mp3", "m4a", "flac", "webm")
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::models::chat::Part;
    ///
    /// // WAV audio
    /// let wav_audio = Part::input_audio(
    ///     "base64_encoded_wav_data".to_string(),
    ///     "wav".to_string()
    /// );
    ///
    /// // MP3 audio
    /// let mp3_audio = Part::input_audio(
    ///     "base64_encoded_mp3_data".to_string(),
    ///     "mp3".to_string()
    /// );
    /// ```
    pub fn input_audio(data: String, format: String) -> Self {
        Self::InputAudio {
            input_audio: InputAudio { data, format },
        }
    }
}

impl std::fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Part::Text { text } => write!(f, "{}", text),
            Part::ImageUrl { image_url } => write!(f, "{}", format!("[Url: {}]", image_url.url)),
            Part::File { file } => write!(f, "{}", format!("[File: {}]", file.filename)),
            Part::InputAudio { input_audio } => {
                write!(f, "{}", format!("[Audio: {}]", input_audio.format))
            }
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
