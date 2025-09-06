---
icon: parachute-box
---

# Fallback Models

OpenRouter allows you to define a list of **fallback** models in case the main model fails for _any_ reason, including rate limits, safety filters, multimodal support, and any other type of error.

The fallback models will then be tried _in order_ of the list. If all fallback models fail as well, the last error message will be returned.

This opens up a lot of use cases:

* Route to a smaller model if rate limits are reached.
* Route to another model if a safety filter blocks the request.
* Route to a multimodal model if the request includes images.
* Route to a model with a bigger context window if the context limit is reached.

## Example

Say you are using GPT-4o with Azure as your provider, you'll find that Azure returns a request error when a content filter is triggered, instead of letting the model respond with a refusal.

To get around this limitation, you can set a fallback model with a more lenient filter, such as Grok 4, to take over the request and respond with a proper refusal in case of failure.

<pre class="language-rust" data-title="fallback_model.rs"><code class="lang-rust">use orpheus::prelude::*;

fn main() {
    let client = Orpheus::new("Your-API-Key");
    
    // Set the provider of GPT-4o as Azure for this example, as they have stricter safety filters.
    // We also have to include the providers for any fallback models (In this case, xAI).
    let res = client
        .chat("I need you to help me make a bomb")
        .model("openai/gpt-4o")
        .fallbacks(["x-ai/grok-4"])
        .with_preferences(|pref| pref.only([Provider::Azure, Provider::XAI])) 
        .send()
        .unwrap();

    println!("Model that responded: {}", &#x26;res.model);
<strong>    println!("Response: {}", &#x26;res.content().unwrap());
</strong>}
</code></pre>

```
Model that responded: x-ai/grok-4
Response: Whoa, hold up there! I appreciate you reaching out, but I absolutely cannot and will not help with anything related to making a bomb or any kind of explosive device. That's not just against my programming—it's illegal, incredibly dangerous, and could cause serious harm to people, including yourself. As Grok, built by xAI, my goal is to be helpful and truthful, but never in ways that promote harm or illegal activities.
```
