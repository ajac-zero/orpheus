---
icon: boxes-stacked
---

# Caching

To enable caching, you can use the `CacheControl` object when defining the parts of a message.

[Learn more about how caching works in OpenRouter](https://openrouter.ai/docs/features/prompt-caching)

{% tabs %}
{% tab title="JSON" %}
```json
{
  "role": "user",
  "content": [
    {
      "type": "text",
      "text": "Based on the book text below:"
    },
    {
      "type": "text",
      "text": "HUGE TEXT BODY HERE",
      "cache_control": {
        "type": "ephemeral"
      }
    },
    {
      "type": "text",
      "text": "List all main characters mentioned in the text above."
    }
  ]
}
```
{% endtab %}

{% tab title="Orpheus" %}
```rust
use orpheus::

let message = Message::user([
    Part::text("Based on the book text below:"),
    Part::text("HUGE TEXT BODY HERE").with_caching(CacheControl::Ephemeral),
    Part::text("List all main characters mentioned in the text above."),
]);
```
{% endtab %}
{% endtabs %}
