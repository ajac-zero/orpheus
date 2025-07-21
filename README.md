# 🎸 Orpheus

> *Can you hear the music?*

**Orpheus** is a library made to make it as ergonomic as possible to create AI apps with [OpenRouter](https://openrouter.ai/), allowing immediate access to hundreds of models and dozens of providers you can mix and match to best suit your use case.

Orpheus also comes with out-of-the-box support for:
- ⚡ Async
- 🌊 Streaming
- 🖼️ Images and PDF
- 🔄 Model Fallbacks
- 🔍 Web Search
- 🛠️ Tool Calling
- 🔌 MCP Client
- 📋 Structured Outputs
- ⚙️ Provider Selection
- 💾 Prompt Caching
- 🔧 Message Transforms
- 🔑 API Key Provisioning

## Installation
Add Orpheus to your project with cargo:

```bash
cargo add orpheus
```

You can also include some optional features that might be useful to your use case:

**anyhow**: For most applications, using the anyhow crate greatly simplifies error handling.

```bash
cargo add orpheus -F anyhow
```

**mcp**: If you require MCP integration in your program, enable this feature which uses the official rmcp package by Anthropic under the hood.

```bash
cargo add orpheus -F mcp
```

## Basic Usage

Let's learn how to use Orpheus with a practical example. Here, we will create a CLI program that allows us to send a chat request to an LLM.

### One-shot Prompt Example

```rust
// The prelude includes everything you will need to use Orpheus
use orpheus::prelude::*;

fn main() {
    // First, we have to start our central client
    let client = Orpheus::new("Your-API-Key");

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

```
>> GPT-4o says: Hello! How can I assist you today?
```

Simple, right? Let's take it up a notch by making the CLI program interactive and adding a conversation history so the model can remember our previous messages.

We also probably don't want to hardcode our API key into the program, so let's initialize our client from environment variables instead.

### With Message History Example

```rust
use orpheus::prelude::*;

fn main() {
    // Start the client by reading the key in the ORPHEUS_API_KEY environment variable
    let client = Orpheus::from_env().expect("ORPHEUS_API_KEY is set");

    // Create a vector that we will continually update with our message history.
    let mut messages = Vec::new();

    loop {
        // Boilerplate to read user input from the terminal into a variable
        let mut user_input = String::new();
        println!("User:");
        std::io::stdin().read_line(&mut user_input).unwrap();

        // Let's turn our user input into a proper message and add it to our message history
        messages.push(Message::user(user_input));

        let response = client
            .chat(&messages) // The chat method accepts our list of messages directly
            .model("openai/gpt-4o")
            .send()
            .unwrap();

        // The response from the model can be turned into a message in the same format as the user message.
        let ai_message = response.into_message().unwrap();

        println!("Assistant:");
        println!("{}", ai_message.content);

        // Add the response message to our list
        messages.push(ai_message);
    }
}
```

```
>> User:
>> hi!
>> Assistant:
>> Hello! How can I assist you today?
>> User:
>> who are you?
>> Assistant:
>> I'm an AI language model created by OpenAI, designed to assist with a wide range of inquiries by providing information, answering questions, and engaging in conversation. How can I help you today?
>> User:
>> that's awesome
>> Assistant:
>> Thank you! If there's anything specific you'd like to know or discuss, feel free to ask.
```

In AI apps, it is common to stream the response to reduce the perceived latency of the program. Let's see how we can use response streaming with Orpheus.

### Streaming Response Example

```rust
use std::io::Write;

use orpheus::prelude::*;

fn main() {
    let client = Orpheus::from_env().expect("ORPHEUS_API_KEY is set");

    let mut messages = Vec::new();

    loop {
        let mut user_input = String::new();
        println!("User:");
        std::io::stdin().read_line(&mut user_input).unwrap();

        messages.push(Message::user(user_input));

        let mut response = client
            .chat(&messages)
            .model("openai/gpt-4o")
            .stream() // By calling `stream` instead of `send`, we get an iterable over the response chunks
            .unwrap();

        // Create a buffer that we will continuously update with the content of each chunk
        let mut buffer = String::new();

        println!("Assistant:");
        // Loop until the iterator runs out of chunks
        while let Some(Ok(chunk)) = response.next() {
            // Get the content of the chunk and add it to the buffer
            let content = chunk.content().unwrap();
            buffer.push_str(content);

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

```
>> User:
>> hi
>> Assistant:
>> Hello! How can I assist you today?
```

> **Note**: You'll have to run it yourself to see the stream effect

--- Delve into it

If you want to learn about additional features, such as async support, structured output, tool calling, MCP, prompt caching, provider configuration, and more, head over to the documentation page!.
