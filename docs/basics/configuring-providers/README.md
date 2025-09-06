---
icon: gear
---

# Configuring Providers

You can customize what providers your requests are routed to with the `preferences` parameter in the `chat` and `completions` builders. This allows you to modify your requests to best suit your use case. For example, you could:

* Set Groq as the only provider to get lightning-fast responses.
* Only allow providers that support tool-calling for Agents.
* Only allow providers that do not collect your data for privacy concerns.
* Allow providers with a specific level of quantization to reduce costs.
* Give priority to providers with lower latency/cost, or higher throughput.

The `preferences` parameter accepts a `ProviderPreferences` object that implements the builder pattern:

```rust
// Create a preferences object with any optional parameters
// We will go in-depth into each one in this section.
let preferences = ProviderPreferences::builder()
    .order(iter)
    .allow_fallbacks(value)
    .require_parameters(value)
    .data_collection(value)
    .only(iter)
    .ignore(iter)
    .quantizations(iter)
    .sort(value)
    .max_price(value)
    .build();

let res = client
    .chat("Who is the greatest general of all time?")
    .model("qwen/qwen3-32b")
    .preferences(preferences) // Pass your preferences to the request
    .send()
    .unwrap();
```

However, a more convenient way to do this would be via the `with_preferences` method in the chat request builder, which accepts a closure that builds the `preferences` object internally.

```rust
let res = client
    .chat("Who is the greatest general of all time?")
    .model("qwen/qwen3-32b")
    .with_preferences(|pref| {
        pref
            .order(iter)
            .allow_fallbacks(value)
            .require_parameters(value)
            .data_collection(value)
            .only(iter)
            .ignore(iter)
            .quantizations(iter)
            .sort(value)
            .max_price(value)
    })
    .send()
    .unwrap();
```
