---
description: Start building AI applications with the OpenRouter API
icon: hand-wave
---

# Introduction

Welcome, AI engineer! These docs cover guides, examples, references, and more to help you build applications using the [100+ models](https://openrouter.ai/models) available via [OpenRouter](https://openrouter.ai/) with the **Orpheus** library.

{% code title="examples/hello.rs" %}
```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;
    
    let response = client
        .chat("Say hello to our friend!")
        .model("openai/gpt-4o")
        .send()?;
        
    println!("Model says: {}", response.content()?);
    
    Ok(())
}
```
{% endcode %}

```
Model says: Hello there! It's great to meet you. How can I assist you today?
```

## Objective

Here are a few points that make the **Orpheus** library special:

* **Ergonomic:** Ease of use is a top priority. Orpheus keeps the learning curve low and the amount of interfaces to remember to a minimum.
* **OpenAI-Compatible:** By following the chat completion standard, Orpheus ensures compatibility with any service that offers an OpenAI-Compatible endpoint.
* **Full Feature Support:** Orpheus aims to stay up to date with the latest capabilities offered by providers. Whether it be tool calling, reasoning, multimodality, or something new.
* **Not a framework:** Avoids opaque abstractions by staying as close as possible to the underlying API while providing a clearer, more comfortable interface.
* **Ecosystem Integrations:** Out-of-the-box support for emerging standards such as MCP, A2A, and Generative UI.

## Key Features

Orpheus is a library made to make it as ergonomic as possible to create AI apps with OpenRouter, allowing immediate access to hundreds of models and dozens of providers you can mix and match to best suit your use case.

Orpheus also comes with out-of-the-box support for:

* Async
* Streaming
* Images and PDF
* Model Fallbacks
* Web Search
* Tool Calling
* MCP Client
* Structured Outputs
* Provider Selection
* Prompt Caching
* Message Transforms
* API Key Provisioning

## Overview

A quick overview of where to find what in our docs:

1. Getting Started: Learn the basics in less than 15 minutes, including how to easily switch between 100+ models, and how to stream the model response as it is generated.
2. Features:
3. Examples:
4. API Reference:

Alternatively, you can follow the links at the bottom of each page to go through this documentation as a proctored, in-depth guide!
