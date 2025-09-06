---
icon: person-chalkboard
---

# Getting Started

## Installation

Add Orpheus to your project with cargo:

```bash
cargo add orpheus
```

## Basic Usage

Let's learn how to use Orpheus with a practical example. Here, we will create a CLI program that allows us to send a chat request to an LLM.

{% code title="one_shot_prompt.rs" %}
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
}t
```
{% endcode %}

```
GPT-4o says: Hello! How can I assist you today?
```

Simple, right? Let's take it up a notch by making the CLI program interactive and adding a conversation history so the model can remember our previous messages.

We also probably don't want to hardcode our API key into the program, so let's initialize our client from environment variables instead.

<pre class="language-rust" data-title="message_history_prompt.rs"><code class="lang-rust">use orpheus::prelude::*;

fn main() {
    // Start the client by reading the key in the ORPHEUS_API_KEY environment variable
    let client = Orpheus::from_env().expect("ORPHEUS_API_KEY is set");
    
    // Create a vector that we will continually update with our message history.
    let mut messages = Vec::new();
    
    loop {
        // Boilerplate to read user input from the terminal into a variable
<strong>        let mut user_input = String::new();
</strong>        println!("User:");
        std::io::stdin().read_line(&#x26;mut user_input).unwrap();
        
        // Let's turn our user input into a proper message and add it to our message history
        messages.push(Message::user(user_input));

        let response = client
            .chat(&#x26;messages) // The chat method accepts our list of messages directly
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
</code></pre>

```
User:
hi!
Assistant:
Hello! How can I assist you today?
User:
who are you?
Assistant:
I'm an AI language model created by OpenAI, designed to assist with a wide range of inquiries by providing information, answering questions, and engaging in conversation. How can I help you today?
User:
that's awesome
Assistant:
Thank you! If there's anything specific you’d like to know or discuss, feel free to ask.
```
