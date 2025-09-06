---
icon: bolt
---

# Async Support

Calling LLMs takes a long time, and it is largely an IO-bound task, which means your app will spend a lot of time idling while waiting for the response to come in.

Because of this, you will probably want to take advantage of `async` code so your app can work while waiting on the LLM.

Orpheus has native async support with [tokio](https://tokio.rs/). The code remains exactly the same, except you will want to use the alternative async client and await your requests.

{% code title="async_client.rs" %}
```rust
use orpheus::prelude::*;

#[tokio::main]
async fn main() {
    // Use the alternative async client
    let client = AsyncOrpheus::from_env().expect("ORPHEUS_API_KEY is set");

    let res = client
        .chat("Who would win in a fist fight, Einstein or Oppenheimer?")
        .model("openai/gpt-4o")
        .send()
        .await // Await the response after calling `send`
        .unwrap();
    println!("{}", res.content().unwrap());
}
```
{% endcode %}

```
Predicting the outcome of a hypothetical fist fight between Albert Einstein and J. Robert Oppenheimer is highly speculative and not particularly meaningful, as both individuals were renowned for their intellectual contributions rather than physical prowess. Einstein is famous for his theories of relativity, while Oppenheimer is best known for his role in the development of the atomic bomb during the Manhattan Project.
```

## Async Streaming

This alternative client also supports streaming responses by implementing the `Stream` extension trait from `futures_lite` for the response object.

{% code title="async_streaming.rs" %}
```rust
use std::io::Write;

use futures_lite::StreamExt;
use orpheus::prelude::*;

#[tokio::main]
async fn main() {
    let client = AsyncOrpheus::from_env().expect("ORPHEUS_API_KEY is set");

    let mut res = client
        .chat("Who would win in a fist fight, Einstein or Oppenheimer?")
        .model("openai/gpt-4o")
        .stream()
        .await // Await the response after calling `send`
        .unwrap();

    while let Some(Ok(chunk)) = res.next().await {
        let content = chunk.content().unwrap();

        print!("{}", content);
        std::io::stdout().flush().unwrap();
    }
}
```
{% endcode %}

```
In a hypothetical scenario where Albert Einstein and J. Robert Oppenheimer were to engage in a fistfight, it's difficult to predict the outcome as neither were known for physical prowess but rather for their intellectual contributions to science. Both were theoretical physicists who made groundbreaking contributions in their fields—Einstein with his theory of relativity and Oppenheimer as a leading figure in the development of the atomic bomb.
```
