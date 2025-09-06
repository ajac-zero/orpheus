---
description: Start building AI applications with the OpenRouter API
icon: hand-wave
---

# Introduction

Welcome, AI engineer! These docs cover guides, examples, references, and more to help you build applications using the [100+ models](https://openrouter.ai/models) available via [OpenRouter](https://openrouter.ai/) with the **Orpheus** library.

{% code title="hello.rs" %}
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

Orpheus aims to be three things:

* **Ergonomic:** Ease of use is a top priority. We keep the learning curve low and the amount of interfaces to remember to a minimum.
* **Fully Featured:** Stay up to date with the latest capabilities offered by providers. This includes tool calling, reasoning, multimodality, or something entirely new.
* **Not a framework:** Avoid opaque abstractions 4 staying as close as possible to the underlying API while providing a clearer, more comfortable interface.

## Key Features

Orpheus also comes with out-of-the-box support for:

* [Async](features/async-support.md)
* [Streaming](basics/getting-started/streaming-responses.md)
* Images and PDF
* [Model Fallbacks](basics/fallback-models.md)
* [Web Search](features/plugins.md)
* [Tool Calling](features/tool-calling/)
* MCP Client
* [Structured Outputs](features/structured-output.md)
* [Provider Selection](basics/configuring-providers/)
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
