---
icon: puzzle-piece-simple
---

# Plugins

Plugins extend language model capabilities by integrating external data sources and processing tools. Models can access real-time information and process files beyond their training data.

## Available Plugins

### Web Search Plugin

Enables models to search the internet for current information.

{% code title="use_web_plugin.rs" %}
```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let response = client
        .chat("What are the latest developments in renewable energy?")
        .model("google/gemini-2.0-flash-001")
        .plugins(Plugin::web())
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```
{% endcode %}

#### Configuration Options

```rust
let web_plugin = Plugin::web()
    .max_results(10)                    // Limit search results
    .search_prompt("custom query")      // Override search query
    .build();
```

| Parameter       | Type             | Description                      | Default                     |
| --------------- | ---------------- | -------------------------------- | --------------------------- |
| `max_results`   | `Option<i32>`    | Maximum number of search results | Provider default            |
| `search_prompt` | `Option<String>` | Custom search query              | Auto-generated from message |

### File Parser Plugin

Processes PDF documents using different parsing engines.

{% code title="use_file_parser_plugin.rs" %}
```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let response = client
        .chat("Analyze the uploaded document.")
        .model("openai/gpt-4o")
        .plugins(Plugin::file_parser())
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```
{% endcode %}

#### Parsing Engines

```rust
// Default engine (MistralOcr)
Plugin::file_parser().build()

// Specify engine by enum
Plugin::file_parser()
    .engine(ParsingEngine::PdfText)
    .build()

// Specify engine by string
Plugin::file_parser()
    .try_engine("native")?
    .build()
```

| Engine       | String Value    | Description                                    |
| ------------ | --------------- | ---------------------------------------------- |
| `PdfText`    | `"pdf-text"`    | Extract text directly from PDF                 |
| `MistralOcr` | `"mistral-ocr"` | OCR processing for scanned documents (default) |
| `Native`     | `"native"`      | Provider-specific parsing                      |

## Multiple Plugins

Use multiple plugins together:

```rust
let plugins = vec![
    Plugin::web()
        .max_results(5)
        .build(),
    Plugin::file_parser()
        .engine(ParsingEngine::PdfText)
        .build(),
];

let response = client
    .chat("Compare web findings with the uploaded document.")
    .model("anthropic/claude-3-5-sonnet")
    .plugins(plugins)
    .send()?;
```
