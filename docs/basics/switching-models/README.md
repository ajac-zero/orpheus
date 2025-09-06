---
icon: arrows-repeat
---

# Switching Models

OpenRouter enables access to over 100 models by simply changing the model ID.

You can head over to the [Models](https://openrouter.ai/models) page to check out the full list. For any model, press the model ID button next to its name. This will copy the model ID into your clipboard, then you can paste the model ID into your code.

<figure><img src="../../.gitbook/assets/image.png" alt=""><figcaption></figcaption></figure>

{% code title="multiple_models.rs" fullWidth="false" %}
```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::new("Your-API-Key");

    let prompt = "Who are you?";

    let models = [
        "anthropic/claude-3.5-haiku".to_string(),
        "openai/chatgpt-4o-latest".into(),
        "moonshotai/kimi-k2".into(),
    ];

    for model in models.into_iter() {
        let res = client.chat(prompt).model(&model).send()?;
        println!("{}: {}\n", model, res.content()?);
    }

    Ok(())
}
```
{% endcode %}

```
anthropic/claude-3.5-haiku: I'm Claude, an AI created by Anthropic. I aim to be helpful, honest, and harmless while having substantive conversations. I won't pretend to be human or claim capabilities I don't have. I'm happy to chat, help with tasks, or provide information to the best of my knowledge and abilities.

openai/chatgpt-4o-latest: I'm ChatGPT, an AI language model created by OpenAI. I'm here to help answer your questions, provide information, explore ideas, and assist with a wide range of topics—from writing and learning to coding and conversation. How can I assist you today?

moonshotai/kimi-k2: I'm Kimi, your AI assistant from Moonshot AI. How can I help you today?
```
