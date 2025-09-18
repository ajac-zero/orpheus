---
icon: transformer-bolt
---

# Transforms

Transforms dictate the behaviour of the OpenRouter API when a prompt exceeds the token limit of the requested model.

You can apply transforms via the `Transform` enum directly to the chat request builder.

[Learn more about message transforms](https://openrouter.ai/docs/features/message-transforms)

```rust
use orpheus::{prelude::*, models::Transform};

let client = Orpheus::from_env().unwrap();

let response = client
    .chat("Really long text")
    .model("short-context-model")
    .transforms([Transform::MiddleOut])
    .send();
```

> NOTE: setting `.tranforms([])` disables all transforms.
