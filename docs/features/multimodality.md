---
icon: image-polaroid
---

# Multimodality

Orpheus supports multimodal inputs, allowing you to include images, files, and audio alongside text in your messages.

## Basic Usage

Add images and files to messages using the builder methods:

```orpheus/docs/features/multimodality.md#L10-20
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let response = client
        .chat("What do you see in this image?")
        .model("openai/gpt-4o")
        .message(
            Message::user("Describe this image")
                .with_image("https://example.com/image.jpg", None)
        )
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```

## Image Input

Include images in messages by URL with optional detail level:

```orpheus/docs/features/multimodality.md#L24-35
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    // High detail image analysis
    let message = Message::user("Analyze this chart")
        .with_image("https://example.com/chart.png", Some("high".to_string()));

    let response = client
        .chat("Please analyze the data shown")
        .model("anthropic/claude-3-5-sonnet-20241022")
        .message(message)
        .send()?;

    Ok(())
}
```

## File Input

Attach files with filename and content data:

```orpheus/docs/features/multimodality.md#L41-52
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let csv_data = "name,age\nJohn,25\nJane,30";
    let message = Message::user("Analyze this CSV data")
        .with_file("data.csv", csv_data);

    let response = client
        .chat("What insights can you provide?")
        .model("openai/gpt-4o")
        .message(message)
        .send()?;

    Ok(())
}
```

## Audio Input

Include audio content with base64-encoded data and format specification:

```orpheus/docs/features/multimodality.md#L58-69
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let audio_data = "UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAA==";
    let message = Message::user("What do you hear in this audio?")
        .with_audio(audio_data, "wav");

    let response = client
        .chat("Please analyze the audio content")
        .model("openai/gpt-4o-audio-preview")
        .message(message)
        .send()?;

    Ok(())
}
```

## Multiple Attachments

Combine multiple images, files, and audio in a single message:

```orpheus/docs/features/multimodality.md#L76-92
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let message = Message::user("Analyze this multimedia content")
        .with_file("report.pdf", "PDF content here")
        .with_image("https://example.com/chart1.png", Some("high".to_string()))
        .with_audio("base64_audio_data", "mp3")
        .with_image("https://example.com/chart2.png", Some("low".to_string()))
        .with_file("summary.txt", "Executive summary text");

    let response = client
        .chat("Please provide a comprehensive analysis")
        .model("anthropic/claude-3-5-sonnet-20241022")
        .message(message)
        .send()?;

    Ok(())
}
```

## Direct Content Construction

Create multimodal content using the `Content` and `Part` types:

```orpheus/docs/features/multimodality.md#L98-117
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let content = Content::simple("Please review these items:")
        .add_part(Part::image_url("https://example.com/photo.jpg".to_string(), None))
        .add_part(Part::file("document.pdf".to_string(), "PDF data".to_string()))
        .add_part(Part::input_audio("base64_audio_data".to_string(), "wav".to_string()));

    let message = Message::new(Role::User, content);

    let response = client
        .chat("Analysis request")
        .model("openai/gpt-4o")
        .message(message)
        .send()?;

    Ok(())
}
```

## Configuration Options

### Image Detail Levels

| Detail Level | Description | Use Case |
|-------------|-------------|----------|
| `None` | Default resolution | General image understanding |
| `Some("low")` | Lower resolution, faster | Quick image identification |
| `Some("high")` | Higher resolution, detailed | Detailed analysis, text reading |

### Audio Formats

Supported audio formats for input (data must be base64-encoded):

| Format | Extension | Use Case |
|--------|-----------|----------|
| `"wav"` | .wav | Uncompressed audio, high quality |
| `"mp3"` | .mp3 | Compressed audio, speech recognition |
| `"m4a"` | .m4a | Apple audio format |
| `"flac"` | .flac | Lossless compression, music analysis |
| `"webm"` | .webm | Web audio format |

### File Types

The file input accepts any filename and content as strings. Common use cases include:

- **PDFs**: Pass PDF content as string data
- **Text files**: CSV, JSON, XML, plain text
- **Code files**: Source code for analysis
- **Data files**: Structured data in various formats

### Content Structure

Messages can contain:
- **Simple content**: Plain text string
- **Complex content**: Mix of text, images, files, and audio as `Part` objects

The content automatically converts between simple and complex forms as you add multimodal elements.

### Audio Processing Requirements

- **Encoding**: Audio data must be base64-encoded
- **Format**: Specify the correct audio format string
- **Model Support**: Use audio-capable models (e.g., "openai/gpt-4o-audio-preview")
- **Size Limits**: Check provider-specific audio file size limitations