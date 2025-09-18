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

{% code title="output" %}
```
Model says: Hello there! It's great to meet you. How can I assist you today?
```
{% endcode %}

## Objective

Orpheus aims to be three things:

* **Ergonomic:** Ease of use is a top priority. We keep the learning curve low and the amount of interfaces to remember to a minimum.
* **Fully Featured:** Stay up to date with the latest capabilities offered by providers. This includes tool calling, reasoning, multimodality, or something entirely new.
* **Not a framework:** Avoid opaque abstractions 4 staying as close as possible to the underlying API while providing a clearer, more comfortable interface.

## Key Features

Orpheus also comes with out-of-the-box support for:

* ⚡ [Async](https://orpheus.ajac-zero.com/features/async-support)
* 🌊 [Streaming](https://orpheus.ajac-zero.com/basics/getting-started/streaming-responses)
* 🖼️ [Images, PDF, and Audio](https://orpheus.ajac-zero.com/features/multimodality)
* 🔄 [Model Fallbacks](https://orpheus.ajac-zero.com/basics/fallback-models)
* 🔍 [Web Search](https://orpheus.ajac-zero.com/features/plugins#web-search-plugin)
* 🛠️ [Tool Calling](https://orpheus.ajac-zero.com/features/tool-calling)
* 🔌 [MCP](https://orpheus.ajac-zero.com/integrations/mcp)
* 📋 [Structured Outputs](https://orpheus.ajac-zero.com/features/structured-output)
* ⚙️ [Provider Selection](https://orpheus.ajac-zero.com/basics/configuring-providers)
* 💾 [Prompt Caching](https://orpheus.ajac-zero.com/features/caching)
* 🔧 [Message Transforms](https://orpheus.ajac-zero.com/features/transforms)
