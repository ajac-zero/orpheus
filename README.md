# 🎸 Orpheus

> *Can you hear the music?*

**Orpheus** is a library made to make it as ergonomic as possible to create AI apps with [OpenRouter](https://openrouter.ai/), allowing immediate access to hundreds of models and dozens of providers you can mix and match to best suit your use case.

Orpheus also comes with out-of-the-box support for:
- ⚡ [Async](https://orpheus.ajac-zero.com/features/async-support)
- 🌊 [Streaming](https://orpheus.ajac-zero.com/basics/getting-started/streaming-responses)
- 🖼️ [Images, PDF, and Audio](https://orpheus.ajac-zero.com/features/multimodality)
- 🔄 [Model Fallbacks](https://orpheus.ajac-zero.com/basics/fallback-models)
- 🔍 [Web Search](https://orpheus.ajac-zero.com/features/plugins#web-search-plugin)
- 🛠️ [Tool Calling](https://orpheus.ajac-zero.com/features/tool-calling)
- 🔌 [MCP](https://orpheus.ajac-zero.com/integrations/mcp)
- 📋 [Structured Outputs](https://orpheus.ajac-zero.com/features/structured-output)
- ⚙️ [Provider Selection](https://orpheus.ajac-zero.com/basics/configuring-providers)
- 💾 [Prompt Caching](https://orpheus.ajac-zero.com/features/caching)
- 🔧 [Message Transforms](https://orpheus.ajac-zero.com/features/transforms)
- 🔑 [API Key Provisioning]()

## Installation
Add Orpheus to your project with cargo:

```bash
cargo add orpheus
```

## Quickstart

Let's learn how to use Orpheus with a practical example. Here, we will create a program that allows us to send a chat request to an LLM.

```rust
// The prelude includes everything you will need to use Orpheus
use orpheus::prelude::*;

fn main() {
    // Start the client by reading the ORPHEUS_API_KEY environment variable
    let client = Orpheus::from_env().expect("ORPHEUS_API_KEY is set");

    // With our client, we can call the `chat` method to begin a chat completion request
    // The request follows a builder pattern, iteratively adding arguments before finally sending it with `send`
    let response = client
        .chat("Hello!") // The chat method takes your prompt as an initial argument
        .model("openai/gpt-4o") // Select a model by passing an OpenRouter model ID
        .send() // Finally, send the request with the arguments set thus far to the model
        .unwrap();

    // Get the content of the response (if any) and print it to the console
    let content = response.content().unwrap();
    println!("GPT-4o says: {}", content);
}
```

```txt
GPT-4o says: Hello! How can I assist you today?
```

Simple, right? Let's take it up a notch by adding a conversation history so the model can remember our previous messages.

```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::from_env().expect("ORPHEUS_API_KEY is set");

    // Create a vector that we will continually update with our message history.
    let mut messages = Vec::new();

    let canned_messages = vec!["Hello!", "My name is Anibal", "What's my name again?"];

    for message in canned_messages {
        // Let's turn our user input into a proper message and add it to our message history
        println!("User: {}", message);
        messages.push(Message::user(message));

        let response = client
            .chat(&messages) // The chat method accepts our list of messages directly
            .model("mistralai/magistral-small-2506")
            .send()
            .unwrap();

        // The response from the model can be turned into a message in the same format as the user message.
        let ai_message = response.into_message().unwrap();

        println!("Assistant: {}", ai_message.content);

        // Add the response message to our list
        messages.push(ai_message);
    }
}
```

```txt
User: Hello!
Assistant: Hello! 😊 How can I assist you today?
User: My name is Anibal
Assistant: Nice to meet you, Anibal! 😊 It's a great name with interesting roots—it comes from Latin, meaning "related to Brains."

How can I help you today? Whether you have questions, need advice, or just want to chat, I'm here for it all.
User: What's my name again?
Assistant: Oh, good catch! I like this game. 😄

**Your name is Anibal**—or at least, that’s what you told me in your last message. (Unless you’re testing my memory, in which case I’m pretending not to notice!)

What’s up next? Need a name origin deep dive, or just a random fun fact? Either way, hit me!

*(P.S. If you’d rather swap to a different name right now, I’m cool with it. Just say the word.)*
```

In AI apps, it is common to stream the response to reduce the perceived latency of the program. Let's see how we can use response streaming with Orpheus.

### Streaming Response Example

```rust
use std::io::Write;

use orpheus::prelude::*;

fn main() {
    // Start the client by reading the key in the ORPHEUS_API_KEY environment variable
    let client = Orpheus::from_env().expect("ORPHEUS_API_KEY is set");

    // Create a vector that we will continually update with our message history.
    let mut messages = Vec::new();

    let canned_messages = vec!["Hello!", "My name is Anibal", "What's my name again?"];

    for message in canned_messages {
        // Let's turn our user input into a proper message and add it to our message history
        println!("User: {}", message);
        messages.push(Message::user(message));

        let mut response = client
            .chat(&messages)
            .model("x-ai/grok-3-mini")
            .stream() // By calling `stream` instead of `send`, we get an iterable over the response chunks
            .unwrap();

        // Create a buffer that we will continuously update with the content of each chunk
        let mut buffer = String::new();

        print!("Assistant: ");
        // Loop until the iterator runs out of chunks
        while let Some(Ok(chunk)) = response.next() {
            // Get the content of the chunk and add it to the buffer
            let content = chunk.content().unwrap();
            buffer.push_str(&content.to_string());

            // Boilerplate to print the response as it comes in
            print!("{}", content);
            std::io::stdout().flush().unwrap();
        }
        println!();

        // Add the completed buffer to the message history
        messages.push(Message::assistant(buffer));
    }
}
```

```txt
User: Hello!
Assistant: Hello! I'm Grok, your AI assistant from xAI. How can I help you today? 😊
User: My name is Anibal
Assistant: Nice to meet you, Anibal! I'm Grok, your AI assistant from xAI. How can I assist you today? 😊
User: What's my name again?
Assistant: Oh, right! You told me your name is Anibal. How can I assist you further today? 😊
```

> **Note**: You'll have to run it yourself to see the stream effect

# Doc Site

If you want to learn about additional features, such as async support, structured output, tool calling, MCP, prompt caching, provider configuration, and more, head over to the [docs](https://orpheus.ajac-zero.com/)!
