---
icon: puzzle-piece-simple
---

# Plugins

Orpheus plugins extend the capabilities of language models by integrating external data sources and processing tools. Rather than being limited to their training data, models can access real-time information and process various file formats through the plugin system.

## Overview

Plugins in Orpheus work by enhancing the model's context with additional capabilities:

- **Web Search**: Access current information from the internet
- **File Processing**: Parse and analyze PDF documents and other files
- **Real-time Data**: Get up-to-date information beyond the model's training cutoff

The plugin system is designed to be seamless - you simply specify which plugins to use, and the model automatically leverages them when relevant to the conversation.

## Available Plugins

### Web Search Plugin

The Web plugin enables models to search the internet for current information, breaking free from training data limitations.

#### Basic Usage

```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let response = client
        .chat("What are the latest developments in renewable energy?")
        .model("google/gemini-2.0-flash-001")
        .plugins(vec![Plugin::Web {
            max_results: None,        // Use default number of results
            search_prompt: None,      // Use default search strategy
        }])
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```

#### Advanced Configuration

```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let response = client
        .chat("Find the latest news about SpaceX rocket launches")
        .model("anthropic/claude-3-5-sonnet")
        .plugins(vec![Plugin::Web {
            max_results: Some(10),   // Limit to 10 search results
            search_prompt: Some("SpaceX rocket launch news 2024".to_string()),
        }])
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```

#### Configuration Options

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `max_results` | `Option<u64>` | Maximum number of search results to retrieve | Provider default (usually 5-10) |
| `search_prompt` | `Option<String>` | Custom search query to use instead of the user's message | Auto-generated from user message |

### File Parser Plugin

The File Parser plugin enables models to process and analyze PDF documents using different parsing engines optimized for various document types.

#### Basic Usage

```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let response = client
        .chat("Please analyze the uploaded research paper and provide key findings.")
        .model("openai/gpt-4o")
        .plugins(vec![Plugin::FileParser {
            pdf: ParsingEngine::PdfText,
        }])
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```

#### Parsing Engines

The File Parser plugin supports multiple parsing engines, each optimized for different document types:

##### PdfText Engine
Best for: Text-based PDFs with selectable text
```rust
Plugin::FileParser {
    pdf: ParsingEngine::PdfText,
}
```

- **Pros**: Fast, accurate for text-based documents, preserves formatting
- **Cons**: Cannot process scanned documents or images
- **Use cases**: Research papers, reports, documentation

##### MistralOcr Engine
Best for: Scanned PDFs and image-based documents
```rust
Plugin::FileParser {
    pdf: ParsingEngine::MistralOcr,
}
```

- **Pros**: Can process scanned documents, handles images and complex layouts
- **Cons**: Slower processing, may have OCR errors
- **Use cases**: Scanned documents, forms, legacy documents

##### Native Engine
Best for: Provider-optimized processing
```rust
Plugin::FileParser {
    pdf: ParsingEngine::Native,
}
```

- **Pros**: Uses the model provider's native parsing capabilities
- **Cons**: Results may vary by provider
- **Use cases**: General-purpose parsing, leveraging provider strengths

#### Engine Selection Guide

```rust
use orpheus::prelude::*;

fn choose_parsing_engine(document_type: &str) -> ParsingEngine {
    match document_type {
        "research_paper" | "report" | "documentation" => ParsingEngine::PdfText,
        "scanned_document" | "form" | "image_heavy" => ParsingEngine::MistralOcr,
        "mixed_content" | "unknown" => ParsingEngine::Native,
        _ => ParsingEngine::PdfText, // Default fallback
    }
}

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;
    
    // Choose engine based on document type
    let engine = choose_parsing_engine("research_paper");
    
    let response = client
        .chat("Summarize the main conclusions from this document.")
        .model("openai/gpt-4o")
        .plugins(vec![Plugin::FileParser { pdf: engine }])
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```

## Using Multiple Plugins

You can combine multiple plugins to create powerful workflows that leverage both real-time data and file processing:

```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let response = client
        .chat("Compare the findings in this research paper with recent developments found online.")
        .model("anthropic/claude-3-5-sonnet")
        .plugins(vec![
            Plugin::Web {
                max_results: Some(5),
                search_prompt: Some("latest AI research developments".to_string()),
            },
            Plugin::FileParser {
                pdf: ParsingEngine::PdfText,
            },
        ])
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```

## Streaming with Plugins

Plugins work seamlessly with streaming responses:

```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let mut stream = client
        .chat("Search for the latest climate change research and provide a detailed analysis.")
        .model("google/gemini-2.0-flash-001")
        .plugins(vec![Plugin::Web {
            max_results: Some(3),
            search_prompt: None,
        }])
        .stream()?;

    let mut buffer = String::new();
    while let Some(Ok(chunk)) = stream.next() {
        if let Some(content) = chunk.content() {
            buffer.push_str(content);
            print!("{}", content); // Stream the response as it comes
        }
    }

    Ok(())
}
```

## Advanced Usage Patterns

### Conditional Plugin Usage

```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;
    
    let user_query = "What's the current stock price of Tesla?";
    let needs_current_data = user_query.contains("current") || user_query.contains("latest");
    
    let mut plugins = vec![];
    if needs_current_data {
        plugins.push(Plugin::Web {
            max_results: Some(3),
            search_prompt: None,
        });
    }

    let response = client
        .chat(user_query)
        .model("openai/gpt-4o")
        .plugins(plugins)
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```

### Plugin Configuration Based on Context

```rust
use orpheus::prelude::*;

fn create_research_assistant_plugins() -> Vec<Plugin> {
    vec![
        Plugin::Web {
            max_results: Some(10),
            search_prompt: Some("academic research papers".to_string()),
        },
        Plugin::FileParser {
            pdf: ParsingEngine::PdfText,
        },
    ]
}

fn create_news_assistant_plugins() -> Vec<Plugin> {
    vec![Plugin::Web {
        max_results: Some(5),
        search_prompt: Some("latest news".to_string()),
    }]
}

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;
    
    // Research workflow
    let research_response = client
        .chat("Find recent papers on quantum computing and analyze any PDFs.")
        .model("anthropic/claude-3-5-sonnet")
        .plugins(create_research_assistant_plugins())
        .send()?;

    // News workflow  
    let news_response = client
        .chat("What's happening in tech today?")
        .model("google/gemini-2.0-flash-001")
        .plugins(create_news_assistant_plugins())
        .send()?;

    println!("Research: {}", research_response.content()?);
    println!("News: {}", news_response.content()?);
    
    Ok(())
}
```

## Best Practices

### 1. Choose Appropriate Models

Different models work better with different plugins:

```rust
// Good model choices for web search
let web_models = vec![
    "google/gemini-2.0-flash-001",    // Excellent web integration
    "anthropic/claude-3-5-sonnet",    // Strong reasoning with external data
    "openai/gpt-4o",                  // Reliable with plugins
];

// Good model choices for file parsing
let file_models = vec![
    "openai/gpt-4o",                  // Strong document analysis
    "anthropic/claude-3-5-sonnet",    // Excellent text comprehension
    "google/gemini-2.0-flash-001",    // Fast processing
];
```

### 2. Optimize Plugin Configuration

```rust
use orpheus::prelude::*;

fn optimized_web_search(query: &str, domain: &str) -> Plugin {
    let max_results = match domain {
        "news" => 5,           // Fast, recent information
        "research" => 10,      // Comprehensive academic sources
        "shopping" => 15,      // Multiple product options
        _ => 5,               // Default
    };
    
    Plugin::Web {
        max_results: Some(max_results),
        search_prompt: Some(format!("{} site:{}", query, domain)),
    }
}
```

### 3. Handle Plugin Failures Gracefully

```rust
use orpheus::prelude::*;

fn resilient_search(client: &Orpheus, query: &str) -> anyhow::Result<String> {
    // Try with plugins first
    let response_with_plugins = client
        .chat(query)
        .model("openai/gpt-4o")
        .plugins(vec![Plugin::Web {
            max_results: Some(5),
            search_prompt: None,
        }])
        .send();

    match response_with_plugins {
        Ok(response) => Ok(response.content()?.to_string()),
        Err(_) => {
            // Fallback to without plugins
            println!("Plugin search failed, falling back to model knowledge...");
            let fallback_response = client
                .chat(&format!("{} (Note: Using model knowledge only)", query))
                .model("openai/gpt-4o")
                .send()?;
            
            Ok(fallback_response.content()?.to_string())
        }
    }
}
```

### 4. Combine with Structured Output

Plugins work excellently with structured output for data extraction:

```rust
use orpheus::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct NewsArticle {
    title: String,
    source: String,
    date: String,
    summary: String,
    url: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let format = Format::json("news_articles")
        .with_schema(|schema| {
            schema
                .property("articles", Param::array().items(
                    Param::object()
                        .property("title", Param::string())
                        .property("source", Param::string())
                        .property("date", Param::string())
                        .property("summary", Param::string())
                        .property("url", Param::string())
                        .required(["title", "source", "date", "summary"])
                        .end()
                ))
                .required(["articles"])
        })
        .build();

    let response = client
        .chat("Find the top 3 tech news articles from today")
        .model("openai/gpt-4o")
        .plugins(vec![Plugin::Web {
            max_results: Some(5),
            search_prompt: Some("tech news today".to_string()),
        }])
        .response_format(format)
        .send()?;

    let parsed: serde_json::Value = serde_json::from_str(&response.content()?)?;
    println!("{}", serde_json::to_string_pretty(&parsed)?);

    Ok(())
}
```

## Common Use Cases

### Research Assistant

```rust
use orpheus::prelude::*;

fn research_assistant(client: &Orpheus, topic: &str) -> anyhow::Result<String> {
    let response = client
        .chat(&format!("Research the topic '{}' using both current web sources and any academic papers. Provide a comprehensive overview with citations.", topic))
        .model("anthropic/claude-3-5-sonnet")
        .plugins(vec![
            Plugin::Web {
                max_results: Some(8),
                search_prompt: Some(format!("{} academic research papers", topic)),
            },
            Plugin::FileParser {
                pdf: ParsingEngine::PdfText,
            },
        ])
        .send()?;

    Ok(response.content()?.to_string())
}
```

### Document Analysis with Context

```rust
use orpheus::prelude::*;

fn analyze_document_with_context(client: &Orpheus, analysis_type: &str) -> anyhow::Result<String> {
    let response = client
        .chat(&format!("Analyze the uploaded document for {} and compare with current industry standards and best practices found online.", analysis_type))
        .model("openai/gpt-4o")
        .plugins(vec![
            Plugin::FileParser {
                pdf: ParsingEngine::PdfText,
            },
            Plugin::Web {
                max_results: Some(5),
                search_prompt: Some(format!("{} industry standards best practices", analysis_type)),
            },
        ])
        .send()?;

    Ok(response.content()?.to_string())
}
```

### Real-time Information Retrieval

```rust
use orpheus::prelude::*;

fn get_current_information(client: &Orpheus, query: &str) -> anyhow::Result<String> {
    let response = client
        .chat(&format!("Get the most current information about: {}", query))
        .model("google/gemini-2.0-flash-001")
        .plugins(vec![Plugin::Web {
            max_results: Some(3),
            search_prompt: Some(format!("latest {} news updates", query)),
        }])
        .send()?;

    Ok(response.content()?.to_string())
}
```

## Troubleshooting

### Plugin Not Working

1. **Check Model Compatibility**: Not all models support all plugins
2. **Verify API Access**: Ensure your API key has plugin access
3. **Review Rate Limits**: Some plugins may have usage restrictions

### Slow Response Times

1. **Reduce max_results**: Fewer search results = faster processing
2. **Use appropriate parsing engine**: Choose the right engine for your document type
3. **Consider model selection**: Some models process plugin data faster

### Unexpected Results

1. **Refine search_prompt**: More specific search terms improve relevance
2. **Check document format**: Ensure PDFs are compatible with chosen parsing engine
3. **Combine with explicit instructions**: Guide the model on how to use plugin data

## Limitations

### Current Limitations

- **Model Support**: Not all models support all plugins
- **File Types**: File Parser currently supports PDFs only
- **Rate Limits**: Plugin usage may be subject to additional rate limits
- **Cost**: Plugin usage may incur additional costs depending on the provider

### Future Enhancements

The plugin system is designed to be extensible. Future plugins may include:
- **Database Integration**: Query databases directly
- **API Connectors**: Connect to external APIs
- **Image Processing**: Analyze and process images
- **Code Execution**: Run code snippets safely

## See Also

- [Tool Calling](tool-calling/) - For structured function calling
- [Structured Output](structured-output.md) - For combining plugins with structured data extraction
- [Multimodality](multimodality.md) - For handling different types of content
- [Async Support](async-support.md) - For using plugins asynchronously