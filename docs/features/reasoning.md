---
icon: brain-circuit
---

# Reasoning

Orpheus has built-in support for reasoning models by configuring the level of reasoning **effort** or setting a reasoning **budget**, which are both available through the `with_reasoning` configurator. Both methods cannot be used at the same time.

[Learn more about how reasoning is controlled internally](https://openrouter.ai/docs/use-cases/reasoning-tokens).

## Configuring Reasoning Effort

You can use the `Effort` enum to set the reasoning according to this [guide](https://openrouter.ai/docs/use-cases/reasoning-tokens#reasoning-effort-level).

{% code title="set_reasoning_effort.rs" %}
```rust
let response = client
    .chat("Are zebras black with white stripes, or white with black stripes?")
    .model("google/gemini-2.5-flash-lite-preview-06-17")
    .with_reasoning(|reasoning| reasoning.effort(Effort::Low))
    .send()?;
```
{% endcode %}

### Setting a Reasoning Budget

You can use the `max_tokens` method to set the reasoning according to this [guide](https://openrouter.ai/docs/use-cases/reasoning-tokens#max-tokens-for-reasoning).

{% code title="set_reasoning_budget.rs" %}
```rust
let response = client
    .chat("Are zebras black with white stripes, or white with black stripes?")
    .model("google/gemini-2.5-flash-lite-preview-06-17")
    .with_reasoning(|reasoning| reasoning.max_tokens(100))
    .send()?;
```
{% endcode %}
